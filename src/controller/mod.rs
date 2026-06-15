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
use crate::controller::state::PlaylistId;
use crate::controller::state::{Track, TrackId};
use crate::ui::components::lyrics::{LyricsState, LyricsStatus};
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
use okmain::rgb::Rgb;
use rand::rng;
use rand::seq::{IteratorRandom, SliceRandom};
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

        if let Some(track_id) = state.queue.get_id(state.playback.current_index)
            && let Some(track) = state.library.tracks.get(&track_id)
            && let Some(source) = track.get_valid_source()
        {
            self.audio_tx
                .send(AudioCommand::Load(track_id, source.path.clone()))
                .ok();
            self.image_processor_tx
                .send(ImageProcessorCommand::GetCurrentAlbumArt(
                    track_id,
                    source.path.clone(),
                ))
                .ok();
        }
    }

    pub fn get_pos(&self) {
        let _ = self.audio_tx.send(AudioCommand::GetPosition);
    }

    pub fn scan_dir(&self, path: PathBuf) {
        let _ = self.scanner_tx.send(ScannerCommand::ScanDir(path));
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
}

impl Global for Controller {}

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
