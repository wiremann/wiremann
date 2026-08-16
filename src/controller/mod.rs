pub mod commands;
pub mod events;
pub mod handlers;
pub mod state;
use crate::cacher::ImageKind;
use crate::controller::commands::{
    CacherCommand, ImageProcessorCommand, LyricsCommand, SystemIntegrationCommand,
};
use crate::controller::events::{
    CacherEvent, ImageProcessorEvent, LyricsEvent, SystemIntegrationEvent,
};
use crate::controller::state::PlaybackStatus;
use crate::controller::state::{AlbumId, ArtistId, PlaylistId};
use crate::controller::state::{Track, TrackId};
use crate::controller::state::{MetricsSession, TrackListenMetrics};
use crate::ui::components::toasts::scanning_status::ScanningStatus;
use crate::ui::components::toasts::{ToastKind, ToastPhase};
use crate::ui::helpers::{drop_image_from_app, duration_to_slider};
use crate::ui::theme::DominantColors;
use crate::ui::wiremann::Wiremann;
use crate::{
    controller::state::AppState, errors::ControllerError, ui::components::image_cache::ImageCache,
};
use commands::{AudioCommand, ScannerCommand};
use crossbeam_channel::{Receiver, Sender};
use events::{AudioEvent, ScannerEvent};
use gpui::{App, Entity, Global, Rgba, rgb};
use tracing::info;
use okmain::rgb::Rgb;
use rand::rng;
use rand::seq::{IteratorRandom, SliceRandom};
use std::cmp::Reverse;
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};
use std::{path::PathBuf, sync::Arc};

#[derive(Clone)]
pub struct Controller {
    pub state: Entity<AppState>,

    // Audio channel
    pub audio_tx: Sender<AudioCommand>,
    pub audio_rx: Receiver<AudioEvent>,

    // Scanner channel
    pub scanner_tx: Sender<ScannerCommand>,
    pub scanner_rx: Receiver<ScannerEvent>,

    // Cacher channel
    pub cacher_tx: Sender<CacherCommand>,
    pub cacher_rx: Receiver<CacherEvent>,

    // Image processor channel
    pub image_processor_tx: Sender<ImageProcessorCommand>,
    pub image_processor_rx: Receiver<ImageProcessorEvent>,

    // System integration channel
    pub system_integration_tx: Sender<SystemIntegrationCommand>,
    pub system_integration_rx: Receiver<SystemIntegrationEvent>,

    // Lyrics manager channel
    pub lyrics_manager_tx: Sender<LyricsCommand>,
    pub lyrics_manager_rx: Receiver<LyricsEvent>,
}

#[derive(Debug, Clone, Default)]
pub struct ListenStats {
    pub total_play_time: Duration,
    pub total_plays: u64,
    pub total_skips: u64,
    pub total_tracks_listened: u64,
    pub first_listen: Option<u64>,
    pub last_listen: Option<u64>,
    pub top_tracks: Vec<(TrackId, TrackListenMetrics)>,
    pub top_artists: Vec<(ArtistId, u32)>,
    pub top_albums: Vec<(AlbumId, u32)>,
}

impl Controller {
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        state: Entity<AppState>,
        audio_tx: Sender<AudioCommand>,
        audio_rx: Receiver<AudioEvent>,
        scanner_tx: Sender<ScannerCommand>,
        scanner_rx: Receiver<ScannerEvent>,
        cacher_tx: Sender<CacherCommand>,
        cacher_rx: Receiver<CacherEvent>,
        image_processor_tx: Sender<ImageProcessorCommand>,
        image_processor_rx: Receiver<ImageProcessorEvent>,
        system_integration_tx: Sender<SystemIntegrationCommand>,
        system_integration_rx: Receiver<SystemIntegrationEvent>,
        lyrics_manager_tx: Sender<LyricsCommand>,
        lyrics_manager_rx: Receiver<LyricsEvent>,
    ) -> Self {
        Controller {
            state,
            audio_tx,
            audio_rx,
            scanner_tx,
            scanner_rx,
            cacher_tx,
            cacher_rx,
            image_processor_tx,
            image_processor_rx,
            system_integration_tx,
            system_integration_rx,
            lyrics_manager_tx,
            lyrics_manager_rx,
        }
    }

    pub fn load_audio(&self, id: &TrackId, cx: &App) {
        let state = self.state.read(cx);
        if let Some(track) = state.library.tracks.get(id)
            && let Some(source) = track.get_valid_source()
        {
            self.audio_tx
                .send(AudioCommand::Load(*id, source.path.clone()))
                .ok();
            self.image_processor_tx
                .send(ImageProcessorCommand::GetCurrentAlbumArt(
                    *id,
                    source.path.clone(),
                ))
                .ok();
        }
    }

    pub fn load_queue_current(&self, cx: &App) {
        let state = self.state.read(cx);

        info!(
            current_index = state.playback.current_index,
            queue_len = state.queue.tracks.len(),
            "load_queue_current called"
        );

        if let Some(track_id) = state.queue.get_id(state.playback.current_index)
            && let Some(track) = state.library.tracks.get(&track_id)
            && let Some(source) = track.get_valid_source()
        {
            info!(?track_id, path = ?source.path, "load_queue_current sending commands");
            self.audio_tx
                .send(AudioCommand::Load(track_id, source.path.clone()))
                .ok();
            self.image_processor_tx
                .send(ImageProcessorCommand::GetCurrentAlbumArt(
                    track_id,
                    source.path.clone(),
                ))
                .ok();
        } else {
            info!("load_queue_current: condition failed — check index/track/exists");
        }
    }

    pub fn get_pos(&self) {
        let _ = self.audio_tx.send(AudioCommand::GetPosition);
    }

    pub fn scan_dir(&self, path: PathBuf) {
        let _ = self.scanner_tx.send(ScannerCommand::ScanDir(path));
    }

    pub fn delete_playlist(&self, id: PlaylistId, cx: &mut App) {
        self.state.update(cx, |this, cx| {
            this.library.playlists.remove(&id);

            if this.playback.current_playlist == Some(id) {
                this.playback.current_playlist = None;
                this.playback.current = None;
                this.playback.current_index = 0;
                this.playback.status = PlaybackStatus::Stopped;
                this.queue.tracks.clear();
                this.queue.order.clear();
            }

            cx.notify();
        });

        let _ = self.audio_tx.send(AudioCommand::Stop);

        let library = self.state.read(cx).library.clone();
        let _ = self.cacher_tx.send(CacherCommand::WriteLibraryState(library));
        let playback = self.state.read(cx).playback.clone();
        let _ = self.cacher_tx.send(CacherCommand::WritePlaybackState(playback));
        let queue = self.state.read(cx).queue.clone();
        let _ = self.cacher_tx.send(CacherCommand::WriteQueueState(queue));
    }

    pub fn rescan_playlist(&self, id: PlaylistId, cx: &mut App) {
        let path = self
            .state
            .read(cx)
            .library
            .playlists
            .get(&id)
            .and_then(|p| p.folder_path.clone());

        if let Some(path) = path {
            let _ = self
                .scanner_tx
                .send(ScannerCommand::ScanDirRescan { path, playlist: id });
        }
    }

    pub fn load_playlist(&self, id: PlaylistId, cx: &mut App) {
        self.state.update(cx, |this, cx| {
            if let Some(playlist) = this.library.playlists.get(&id) {
                this.playback.current_playlist = Some(playlist.id);
                this.queue.tracks.clone_from(&playlist.tracks);
                this.queue.order = (0..playlist.tracks.len()).collect();
                this.playback.current_index = 0;
                this.playback.shuffling = false;
                this.playback.repeat = false;

                cx.notify();
            }
        });

        self.load_queue_current(cx);
        let state = self.state.read(cx).queue.clone();
        let _ = self.cacher_tx.send(CacherCommand::WriteQueueState(state));
    }

    pub fn load_album(&self, id: AlbumId, cx: &mut App) {
        self.state.update(cx, |this, cx| {
            if let Some(album) = this.library.albums.get(&id) {
                this.playback.current_playlist = None;
                this.queue.tracks.clone_from(&album.tracks);
                this.queue.order = (0..album.tracks.len()).collect();
                this.playback.current_index = 0;
                this.playback.shuffling = false;
                this.playback.repeat = false;

                cx.notify();
            }
        });

        self.load_queue_current(cx);
        let state = self.state.read(cx).queue.clone();
        let _ = self.cacher_tx.send(CacherCommand::WriteQueueState(state));
    }

    pub fn load_artist(&self, id: ArtistId, cx: &mut App) {
        self.state.update(cx, |this, cx| {
            if let Some(artist) = this.library.artists.get(&id) {
                this.playback.current_playlist = None;
                this.queue.tracks.clone_from(&artist.tracks);
                this.queue.order = (0..artist.tracks.len()).collect();
                this.playback.current_index = 0;
                this.playback.shuffling = false;
                this.playback.repeat = false;

                cx.notify();
            }
        });

        self.load_queue_current(cx);
        let state = self.state.read(cx).queue.clone();
        let _ = self.cacher_tx.send(CacherCommand::WriteQueueState(state));
    }

    pub fn load_track(&self, track_id: TrackId, cx: &mut App) {
        self.state.update(cx, |this, _| {
            let queue = &mut this.queue;

            let insert_pos = this.playback.current_index + 1;

            if !queue.tracks.contains(&track_id) {
                if queue.tracks.is_empty() {
                    queue.tracks.push(track_id);
                } else {
                    queue.tracks.insert(insert_pos, track_id);
                }

                queue.order = (0..queue.tracks.len()).collect();

                this.playback.current_index = insert_pos;
            }

            this.playback.current = Some(track_id);

            if let Some(idx) = this.queue.get_index(track_id) {
                this.playback.current_index = idx;
            }

            this.playback.current_playlist = None;
        });

        self.load_queue_current(cx);
        let state = self.state.read(cx).queue.clone();
        let _ = self.cacher_tx.send(CacherCommand::WriteQueueState(state));
    }

    pub fn scan_track(&self, path: PathBuf) {
        let _ = self.scanner_tx.send(ScannerCommand::ScanTrack(path));
    }

    pub fn play(&self) {
        let _ = self.audio_tx.send(AudioCommand::Play);
    }

    pub fn pause(&self) {
        let _ = self.audio_tx.send(AudioCommand::Pause);
    }

    pub fn stop(&self) {
        let _ = self.audio_tx.send(AudioCommand::Stop);
    }

    pub fn set_repeat(&self, cx: &mut App) {
        self.state.update(cx, |this, _| {
            this.playback.repeat = !this.playback.repeat;
        });
        let state = self.state.read(cx).playback.clone();
        let _ = self
            .cacher_tx
            .send(CacherCommand::WritePlaybackState(state));
    }

    pub fn set_mute(&self, cx: &mut App) {
        self.state.update(cx, |this, _| {
            this.playback.mute = !this.playback.mute;

            let _ = self
                .audio_tx
                .send(AudioCommand::SetVolume(if this.playback.mute {
                    0.0
                } else {
                    this.playback.volume
                }));
        });
        let state = self.state.read(cx).playback.clone();
        let _ = self
            .cacher_tx
            .send(CacherCommand::WritePlaybackState(state));
    }

    pub fn set_volume(&self, vol: f32, cx: &mut App) {
        self.state.update(cx, |this, _| {
            this.playback.volume = vol;
        });

        let muted = self.state.read(cx).playback.mute;

        let _ = self
            .audio_tx
            .send(AudioCommand::SetVolume(if muted { 0.0 } else { vol }));

        let state = self.state.read(cx).playback.clone();
        let _ = self
            .cacher_tx
            .send(CacherCommand::WritePlaybackState(state));
    }

    pub fn set_shuffle(&self, cx: &mut App) {
        self.state.update(cx, |this, _| {
            this.playback.shuffling = !this.playback.shuffling;

            if this.queue.tracks.is_empty() {
                return;
            }

            let current = this.queue.order[this.playback.current_index];

            if this.playback.shuffling {
                let mut rng = rng();
                this.queue.order = (0..this.queue.tracks.len()).collect();

                this.queue.order.shuffle(&mut rng);

                if let Some(pos) = this.queue.order.iter().position(|&x| x == current) {
                    this.queue.order.swap(0, pos);
                }

                this.playback.current_index = 0;
            } else {
                this.queue.order = (0..this.queue.tracks.len()).collect();

                this.playback.current_index = current;
            }
        });

        let state = self.state.read(cx).clone();
        let _ = self
            .cacher_tx
            .send(CacherCommand::WriteQueueState(state.queue));
        let _ = self
            .cacher_tx
            .send(CacherCommand::WritePlaybackState(state.playback));
    }

    pub fn next(&self, cx: &mut App) {
        self.state.update(cx, |this, _| {
            this.playback.current_index =
                (this.playback.current_index + 1).clamp(0, this.library.tracks.len());
        });

        self.load_queue_current(cx);

        let state = self.state.read(cx).clone();
        let _ = self
            .cacher_tx
            .send(CacherCommand::WriteQueueState(state.queue));
        let _ = self
            .cacher_tx
            .send(CacherCommand::WritePlaybackState(state.playback));
    }
    pub fn prev(&self, cx: &mut App) {
        self.state.update(cx, |this, _| {
            this.playback.current_index = this.playback.current_index.saturating_sub(1);
        });

        self.load_queue_current(cx);

        let state = self.state.read(cx).clone();
        let _ = self
            .cacher_tx
            .send(CacherCommand::WriteQueueState(state.queue));
        let _ = self
            .cacher_tx
            .send(CacherCommand::WritePlaybackState(state.playback));
    }

    pub fn seek(&self, pos: Duration) {
        let _ = self.audio_tx.send(AudioCommand::Seek(pos));
    }

    pub fn check_track_ended(&self) {
        let _ = self.audio_tx.send(AudioCommand::CheckTrackEnded);
    }

    pub fn load_cached_app_state(&self) {
        let _ = self.cacher_tx.send(CacherCommand::GetAppState);
    }

    pub fn request_track_thumbnails(&self, track_ids: &[TrackId], cx: &mut App) {
        let mut cache_ids = Vec::new();
        let mut scan_jobs = HashSet::new();

        let state = self.state.read(cx);
        let tracks = &state.library.tracks;

        for tid in track_ids {
            if let Some(track) = tracks.get(tid) {
                if let Some(image_id) = track.image_id {
                    cache_ids.push(image_id);
                } else if let Some(source) = track.get_valid_source() {
                    scan_jobs.insert((track.id, source.path.clone()));
                }
            }
        }

        cx.global_mut::<ImageCache>().request(
            cache_ids,
            &self.cacher_tx,
            ImageKind::ThumbnailSmall,
        );

        self.image_processor_tx
            .send(ImageProcessorCommand::GetThumbnails(
                scan_jobs,
                ImageKind::ThumbnailSmall,
            ))
            .ok();
    }

    pub fn request_playlist_thumbnails(&self, playlist_ids: &[PlaylistId], cx: &mut App) {
        let mut cache_ids = Vec::new();

        let state = self.state.read(cx);
        let playlists = &state.library.playlists;

        for pid in playlist_ids {
            if let Some(playlist) = playlists.get(pid) {
                if let Some(image_id) = playlist.image_id {
                    cache_ids.push(image_id);
                } else {
                    let playlist_tracks = playlist.tracks.clone();
                    let thumb_tracks = {
                        let state = self.state.read(cx);

                        pick_playlist_thumbnail_tracks(&state.library.tracks, &playlist_tracks, 4)
                    };

                    if thumb_tracks.len() >= 4 {
                        let _ = self.image_processor_tx.send(
                            ImageProcessorCommand::PlaylistThumbnail {
                                id: *pid,
                                tracks: thumb_tracks,
                            },
                        );
                    }
                }
            }
        }

        cx.global_mut::<ImageCache>()
            .request(cache_ids, &self.cacher_tx, ImageKind::Playlist);
    }

    pub fn request_album_thumbnails(&self, album_ids: &[AlbumId], cx: &mut App) {
        let mut cache_ids = Vec::new();

        let state = self.state.read(cx);
        let albums = &state.library.albums;
        let tracks = &state.library.tracks;

        for aid in album_ids {
            if let Some(album) = albums.get(aid) {
                if let Some(image_id) = album.image_id {
                    cache_ids.push(image_id);
                } else if let Some(image_id) = album
                    .tracks
                    .iter()
                    .find_map(|id| tracks.get(id).and_then(|t| t.image_id))
                {
                    cache_ids.push(image_id);
                }
            }
        }

        cx.global_mut::<ImageCache>().request(
            cache_ids,
            &self.cacher_tx,
            ImageKind::ThumbnailLarge,
        );
    }

    pub fn request_artist_thumbnails(&self, artist_ids: &[ArtistId], cx: &mut App) {
        let mut cache_ids = Vec::new();

        let state = self.state.read(cx);
        let artists = &state.library.artists;
        let tracks = &state.library.tracks;

        for aid in artist_ids {
            if let Some(artist) = artists.get(aid) {
                if let Some(image_id) = artist.image_id {
                    cache_ids.push(image_id);
                } else if let Some(track_id) = artist.tracks.first() {
                    if let Some(track) = tracks.get(track_id) {
                        if let Some(image_id) = track.image_id {
                            cache_ids.push(image_id);
                        }
                    }
                }
            }
        }

        cx.global_mut::<ImageCache>().request(
            cache_ids,
            &self.cacher_tx,
            ImageKind::ThumbnailLarge,
        );
    }

    pub fn get_lyrics(
        &self,
        id: TrackId,
        title: &str,
        artist: &str,
        album: &str,
        duration: Duration,
    ) {
        self.lyrics_manager_tx
            .send(LyricsCommand::GetLyrics {
                id,
                title: title.to_string(),
                artist: artist.to_string(),
                album: album.to_string(),
                duration,
            })
            .ok();
    }

    pub fn is_favorite(&self, id: TrackId, cx: &App) -> bool {
        self.state.read(cx).is_favorite(id)
    }

    pub fn toggle_favorite(&self, id: TrackId, cx: &mut App) {
        self.state.update(cx, |this, _| {
            this.toggle_favorite(id);
        });

        let favorites = self.state.read(cx).favorites.clone();
        let _ = self
            .cacher_tx
            .send(CacherCommand::WriteFavorites(favorites));
    }

    pub fn reveal_in_folder(&self, id: TrackId, cx: &App) {
        let Some(path) = self
            .state
            .read(cx)
            .library
            .track(id)
            .and_then(|track| track.get_valid_source())
            .map(|source| source.path.clone())
        else {
            return;
        };

        reveal_in_os(&path);
    }

    fn now_secs() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }

    pub fn finalize_metrics_session(&self, cx: &mut App, completed: bool) {
        let should_send = self.state.update(cx, |this, _| {
            let Some(session) = this.metrics_session.take() else {
                return false;
            };

            let metrics = this.metrics.tracks.entry(session.track_id).or_default();

            if session.played > Duration::ZERO {
                metrics.play_time += session.played;
            }

            let duration = this
                .library
                .tracks
                .get(&session.track_id)
                .map(|t| t.duration)
                .unwrap_or_default();

            if !completed && duration > Duration::ZERO && session.played * 5 < duration * 4 {
                metrics.skip_count += 1;
            }

            true
        });

        if should_send {
            let metrics = self.state.read(cx).metrics.clone();
            let _ = self
                .cacher_tx
                .send(CacherCommand::WriteMetrics(metrics));
        }
    }

    pub fn start_metrics_session(&self, track_id: TrackId, cx: &mut App) {
        self.state.update(cx, |this, _| {
            let now = Self::now_secs();

            let metrics = this.metrics.tracks.entry(track_id).or_default();
            metrics.play_count += 1;
            metrics.first_played.get_or_insert(now);
            metrics.last_played = Some(now);

            this.metrics_session = Some(MetricsSession {
                track_id,
                last_position: Duration::ZERO,
                played: Duration::ZERO,
            });
        });

        let metrics = self.state.read(cx).metrics.clone();
        let _ = self.cacher_tx.send(CacherCommand::WriteMetrics(metrics));
    }

    pub fn top_tracks(&self, cx: &App, limit: usize) -> Vec<TrackId> {
        let state = self.state.read(cx);

        let mut ranked = state
            .metrics
            .tracks
            .iter()
            .filter(|(id, _)| state.library.tracks.contains_key(id))
            .map(|(id, m)| {
                (
                    *id,
                    m.play_count,
                    m.play_time.as_secs(),
                    m.last_played.unwrap_or(0),
                )
            })
            .collect::<Vec<_>>();

        ranked.sort_by(|a, b| b.1.cmp(&a.1).then(b.2.cmp(&a.2)).then(b.3.cmp(&a.3)));

        ranked
            .into_iter()
            .take(limit)
            .map(|(id, _, _, _)| id)
            .collect()
    }

    pub fn recently_played(&self, cx: &App, limit: usize) -> Vec<TrackId> {
        let state = self.state.read(cx);

        let mut ranked = state
            .metrics
            .tracks
            .iter()
            .filter(|(id, m)| state.library.tracks.contains_key(id) && m.last_played.is_some())
            .map(|(id, m)| (*id, m.last_played.unwrap_or(0)))
            .collect::<Vec<_>>();

        ranked.sort_by(|a, b| b.1.cmp(&a.1));

        ranked.into_iter().take(limit).map(|(id, _)| id).collect()
    }

    pub fn top_artists(&self, cx: &App, limit: usize) -> Vec<ArtistId> {
        let state = self.state.read(cx);

        let mut agg = HashMap::<ArtistId, u32>::new();

        for (id, m) in state.metrics.tracks.iter() {
            if let Some(track) = state.library.tracks.get(id) {
                for artist_id in track.artists.iter() {
                    *agg.entry(*artist_id).or_default() += m.play_count;
                }
            }
        }

        let mut ranked = agg
            .into_iter()
            .map(|(id, plays)| {
                let name = state
                    .library
                    .artists
                    .get(&id)
                    .map(|a| a.name.to_string())
                    .unwrap_or_default();
                (id, plays, name)
            })
            .collect::<Vec<_>>();

        // Break ties by name so equal play counts render in a stable order
        // instead of flickering (the aggregate map is rebuilt every render).
        ranked.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.2.cmp(&b.2)));

        ranked.into_iter().take(limit).map(|(id, _, _)| id).collect()
    }

    pub fn listen_stats(&self, cx: &App) -> ListenStats {
        let state = self.state.read(cx);

        let mut stats = ListenStats::default();

        let mut artist_plays = HashMap::<ArtistId, u32>::new();
        let mut album_plays = HashMap::<AlbumId, u32>::new();

        let mut top_tracks = Vec::new();

        for (id, m) in state.metrics.tracks.iter() {
            let Some(track) = state.library.tracks.get(id) else {
                continue;
            };

            stats.total_plays += u64::from(m.play_count);
            stats.total_skips += u64::from(m.skip_count);
            stats.total_play_time += m.play_time;

            if m.play_count > 0 {
                stats.total_tracks_listened += 1;
            }

            if let Some(first) = m.first_played {
                stats.first_listen = Some(match stats.first_listen {
                    Some(prev) => prev.min(first),
                    None => first,
                });
            }

            if let Some(last) = m.last_played {
                stats.last_listen = Some(match stats.last_listen {
                    Some(prev) => prev.max(last),
                    None => last,
                });
            }

            top_tracks.push((*id, m.clone()));

            for artist_id in track.artists.iter() {
                *artist_plays.entry(*artist_id).or_default() += m.play_count;
            }

            *album_plays.entry(track.album).or_default() += m.play_count;
        }

        top_tracks.sort_by(|a, b| {
            b.1.play_count
                .cmp(&a.1.play_count)
                .then(b.1.play_time.cmp(&a.1.play_time))
                .then(b.1.last_played.cmp(&a.1.last_played))
        });
        stats.top_tracks = top_tracks.into_iter().take(10).collect();

        let mut top_artists = artist_plays
            .into_iter()
            .map(|(id, plays)| {
                let name = state
                    .library
                    .artists
                    .get(&id)
                    .map(|a| a.name.to_string())
                    .unwrap_or_default();
                (id, plays, name)
            })
            .collect::<Vec<_>>();
        top_artists.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.2.cmp(&b.2)));
        stats.top_artists = top_artists
            .into_iter()
            .take(10)
            .map(|(id, plays, _)| (id, plays))
            .collect();

        let mut top_albums = album_plays
            .into_iter()
            .map(|(id, plays)| {
                let name = state
                    .library
                    .albums
                    .get(&id)
                    .map(|a| a.name.to_string())
                    .unwrap_or_default();
                (id, plays, name)
            })
            .collect::<Vec<_>>();
        top_albums.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.2.cmp(&b.2)));
        stats.top_albums = top_albums
            .into_iter()
            .take(10)
            .map(|(id, plays, _)| (id, plays))
            .collect();

        stats
    }
}

impl Global for Controller {}

fn reveal_in_os(path: &std::path::Path) {
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("explorer.exe")
            .arg("/select,")
            .arg(path)
            .spawn();
    }

    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg("-R").arg(path).spawn();
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        if let Some(folder) = path.parent() {
            let _ = std::process::Command::new("xdg-open").arg(folder).spawn();
        }
    }
}

#[must_use]
pub fn pick_playlist_thumbnail_tracks<S: ::std::hash::BuildHasher>(
    library_tracks: &HashMap<TrackId, Arc<Track>, S>,
    playlist_tracks: &[TrackId],
    count: usize,
) -> Vec<PathBuf> {
    let mut rng = rand::rng();
    let mut chosen = Vec::with_capacity(count);
    let mut albums = HashSet::with_capacity(count);

    let candidates = playlist_tracks.iter().copied().sample(&mut rng, count * 3);

    for id in candidates {
        if let Some(track) = library_tracks.get(&id)
            && albums.insert(track.album.clone())
            && let Some(source) = track.get_valid_source()
        {
            chosen.push(source.path.clone());
        }

        if chosen.len() == count {
            return chosen;
        }
    }

    if chosen.len() < count {
        for id in playlist_tracks {
            if chosen.len() == count {
                break;
            }

            if let Some(track) = library_tracks.get(id)
                && albums.insert(track.album.clone())
                && let Some(source) = track.get_valid_source()
            {
                chosen.push(source.path.clone());
            }
        }
    }

    chosen
}
