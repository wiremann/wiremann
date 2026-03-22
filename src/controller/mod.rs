pub mod commands;
pub mod events;
pub mod state;
use crate::cacher::ImageKind;
use crate::controller::commands::CacherCommand;
use crate::controller::events::CacherEvent;
use crate::controller::state::PlaybackStatus;
use crate::library::playlists::PlaylistId;
use crate::library::{Track, TrackId};
use crate::ui::helpers::{drop_image_from_app, secs_to_slider};
use crate::ui::wiremann::Wiremann;
use crate::{
    controller::state::AppState, errors::ControllerError,
    ui::components::image_cache::ImageCache,
};
use commands::{AudioCommand, ScannerCommand};
use crossbeam_channel::{Receiver, Sender};
use events::{AudioEvent, ScannerEvent};
use gpui::{App, Entity, Global};
use rand::rng;
use rand::seq::{IteratorRandom, SliceRandom};
use std::collections::{HashMap, HashSet};
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
}

impl Controller {
    #[must_use]
    pub fn new(
        state: Entity<AppState>,
        audio_tx: Sender<AudioCommand>,
        audio_rx: Receiver<AudioEvent>,
        scanner_tx: Sender<ScannerCommand>,
        scanner_rx: Receiver<ScannerEvent>,
        cacher_tx: Sender<CacherCommand>,
        cacher_rx: Receiver<CacherEvent>,
    ) -> Self {
        Controller {
            state,
            audio_tx,
            audio_rx,
            scanner_tx,
            scanner_rx,
            cacher_tx,
            cacher_rx,
        }
    }

    #[allow(clippy::missing_errors_doc)]
    pub fn handle_audio_event(
        &mut self,
        cx: &mut App,
        event: &AudioEvent,
        view: &Entity<Wiremann>,
    ) -> Result<(), ControllerError> {
        match event {
            AudioEvent::Position(pos) => {
                let last_pos = self.state.read(cx).playback.position.clone();

                if *pos != last_pos {
                    view.update(cx, |this, cx| {
                        this.player_page.update(cx, |this, cx| {
                            this.controlbar.update(cx, |this, cx| {
                                this.playback_slider_state.update(cx, |this, cx| {
                                    let state = cx.global::<Controller>().state.read(cx);
                                    let current = if let Some(id) = state.playback.current {
                                        state.library.tracks.get(&id)
                                    } else {
                                        None
                                    };

                                    let duration = if let Some(track) = current {
                                        track.duration
                                    } else {
                                        0
                                    };
                                    this.set_value(secs_to_slider(*pos, duration), cx);
                                });
                            });
                        });
                    });
                    self.state.update(cx, |this, cx| {
                        this.playback.position = *pos;
                        cx.notify();
                    });

                    let state = self.state.read(cx).playback.clone();
                    let _ = self
                        .cacher_tx
                        .send(CacherCommand::WritePlaybackState(state));
                }
            }
            AudioEvent::TrackLoaded(track_id, path) => {
                let state = self.state.read(cx);
                if !state.library.tracks.contains_key(&track_id) {
                    let _ = self.scanner_tx.send(ScannerCommand::GetTrackMetadata {
                        path: path.clone(),
                        track_id: *track_id,
                    });
                }

                if let Some(track) = state.library.tracks.get(&track_id) && let Some(image_id) = track.image_id {
                    let _ = self.cacher_tx.send(CacherCommand::GetImage(HashSet::from([image_id.clone()]), ImageKind::AlbumArt));
                } else {
                    let _ = self.scanner_tx.send(ScannerCommand::GetCurrentAlbumArt(*track_id, path.clone()));
                }

                self.state.update(cx, |this, cx| {
                    this.playback.current = Some(*track_id);

                    if let Some(idx) = this.queue.get_index(*track_id) {
                        this.playback.current_index = idx;
                    }

                    cx.notify();
                });

                let state = self.state.read(cx).playback.clone();
                let _ = self
                    .cacher_tx
                    .send(CacherCommand::WritePlaybackState(state));
            }
            AudioEvent::PlaybackStatus(status) => {
                self.state.update(cx, |this, cx| {
                    this.playback.status = *status;
                    cx.notify();
                });
                let state = self.state.read(cx).playback.clone();
                let _ = self
                    .cacher_tx
                    .send(CacherCommand::WritePlaybackState(state));
            }
            AudioEvent::TrackEnded => {
                let repeat = self.state.read(cx).playback.repeat;

                if repeat {
                    self.load_queue_current(cx);
                } else {
                    self.next(cx);
                }
            }
        }
        Ok(())
    }

    #[allow(clippy::missing_errors_doc)]
    pub fn handle_scanner_event(
        &mut self,
        cx: &mut App,
        event: &ScannerEvent,
        view: &Entity<Wiremann>,
    ) -> Result<(), ControllerError> {
        match event {
            ScannerEvent::UpsertTracks(tracks) => {
                self.state.update(cx, |this, cx| {
                    this.library.tracks.reserve(tracks.len());
                    for (track, playlist_id) in tracks {
                        let id = track.id;

                        if let Some(existing) = this.library.tracks.get_mut(&id) {
                            let existing = Arc::make_mut(existing);

                            for src in &track.sources {
                                if !existing.sources.iter().any(|s| s.path == src.path) {
                                    existing.sources.push(src.clone());
                                }
                            }

                            if existing.title.is_empty() && !track.title.is_empty() {
                                existing.title = track.title.clone();
                            }

                            if existing.artist.is_empty() && !track.artist.is_empty() {
                                existing.artist = track.artist.clone();
                            }

                            if existing.album.is_empty() && !track.album.is_empty() {
                                existing.album = track.album.clone();
                            }
                        } else {
                            this.library.tracks.insert(id, Arc::new(track.clone()));
                        }

                        let _ = self.scanner_tx.send(ScannerCommand::MetaJobFinished(id));

                        if let Some(pid) = playlist_id {
                            if let Some(playlist) = this.library.playlists.get_mut(pid) {
                                if !playlist.tracks.contains(&id) {
                                    playlist.tracks.push(id);
                                }
                            }
                        }
                    }
                    cx.notify();
                });
                let state = self.state.read(cx).library.clone();
                let _ = self.cacher_tx.send(CacherCommand::WriteLibraryState(state));
            }
            ScannerEvent::InsertTrackIntoPlaylist(pid, tid) => {
                self.state.update(cx, |this, cx| {
                    if let Some(playlist) = this.library.playlists.get_mut(pid) {
                        playlist.tracks.push(*tid);
                    }
                    cx.notify()
                });
                let state = self.state.read(cx).library.clone();
                let _ = self.cacher_tx.send(CacherCommand::WriteLibraryState(state));
            }
            ScannerEvent::AddTrackSource(id, source) => {
                self.state.update(cx, |this, cx| {
                    if let Some(track) = this.library.tracks.get_mut(&id) {
                        Arc::make_mut(track).sources.push(source.clone());
                    }

                    cx.notify();
                });
                let state = self.state.read(cx).library.clone();
                let _ = self.cacher_tx.send(CacherCommand::WriteLibraryState(state));
            }
            ScannerEvent::RemoveTrackSource(id, path) => {
                self.state.update(cx, |this, cx| {
                    if let Some(track) = this.library.tracks.get_mut(&id) {
                        if let Some(source) = track.sources.iter().position(|this| this.path == *path) {
                            Arc::make_mut(track).sources.remove(source);
                        }
                    }

                    cx.notify();
                });
                let state = self.state.read(cx).library.clone();
                let _ = self.cacher_tx.send(CacherCommand::WriteLibraryState(state));
            }
            ScannerEvent::InsertPlaylist(playlist) => {
                self.state.update(cx, |this, cx| {
                    this.library
                        .playlists
                        .insert(playlist.id, playlist.clone());

                    cx.notify();
                });

                let state = self.state.read(cx).library.clone();
                let _ = self.cacher_tx.send(CacherCommand::WriteLibraryState(state));
            }
            ScannerEvent::InsertAlbumArt(image_id, image) => {
                let image_cache = cx.global_mut::<ImageCache>();

                image_cache.current = Some(image.clone());

                let image = image.clone();
                let width = image.size(0).width.0.cast_unsigned();
                let height = image.size(0).height.0.cast_unsigned();
                if let Some(image) = image.as_bytes(0) {
                    let image = image.to_vec();
                    let _ = self.cacher_tx.send(CacherCommand::WriteImage {
                        id: *image_id,
                        kind: ImageKind::AlbumArt,
                        width,
                        height,
                        image,
                    });
                }

                cx.notify(view.entity_id());
            }
            ScannerEvent::InsertThumbnails(thumbnails) => {
                for (id, image) in thumbnails {
                    let width = image.size(0).width.0.cast_unsigned();
                    let height = image.size(0).height.0.cast_unsigned();
                    if let Some(image) = image.as_bytes(0) {
                        let image = image.to_vec();
                        let _ = self.cacher_tx.send(CacherCommand::WriteImage {
                            id: *id,
                            kind: ImageKind::Thumbnail,
                            width,
                            height,
                            image,
                        });
                    }

                    let evicted = {
                        let thumbnail_cache = cx.global_mut::<ImageCache>();
                        thumbnail_cache.add(id.clone(), image.clone())
                    };

                    if let Some(img) = evicted {
                        drop_image_from_app(cx, img);
                    }
                }
            }
            ScannerEvent::UpdateImageLookup(lookup) => {
                self.state.update(cx, |this, cx| {
                    for (id, image_id) in lookup {
                        if let Some(track) = this.library.tracks.get_mut(&id) {
                            Arc::make_mut(track).image_id = Some(*image_id);
                        }
                    }

                    cx.notify();
                });
                let state = self.state.read(cx).library.clone();
                let _ = self.cacher_tx.send(CacherCommand::WriteLibraryState(state));
            }
            ScannerEvent::InsertPlaylistThumbnail(id, image_id, image) => {
                let thumbnail_cache = cx.global_mut::<ImageCache>();

                thumbnail_cache.add(*image_id, image.clone());

                let _ = self.scanner_tx.send(ScannerCommand::PlaylistThumbnailJobFinished(*id));

                let width = image.size(0).width.0.cast_unsigned();
                let height = image.size(0).height.0.cast_unsigned();
                if let Some(image) = image.as_bytes(0) {
                    let image = image.to_vec();
                    let _ = self.cacher_tx.send(CacherCommand::WriteImage {
                        id: *image_id,
                        kind: ImageKind::Playlist,
                        width,
                        height,
                        image,
                    });
                }

                self.state.update(cx, |this, _| {
                    if let Some(playlist) = this.library.playlists.get_mut(id) {
                        playlist.image_id = Some(*image_id);
                    }
                });
                let state = self.state.read(cx).library.clone();
                let _ = self.cacher_tx.send(CacherCommand::WriteLibraryState(state));
            }
        }
        Ok(())
    }

    #[allow(clippy::missing_errors_doc)]
    pub fn handle_cacher_event(
        &mut self,
        cx: &mut App,
        event: &CacherEvent,
        view: &Entity<Wiremann>,
    ) -> Result<(), ControllerError> {
        match event {
            CacherEvent::AppState(state) => {
                let playback_state = state.playback.clone();
                self.state.update(cx, |this, _| {
                    *this = state.clone();
                });

                self.load_queue_current(cx);
                self.set_volume(playback_state.volume, cx);
                self.seek(playback_state.position);

                match playback_state.status {
                    PlaybackStatus::Stopped => self.stop(),
                    PlaybackStatus::Paused => self.pause(),
                    PlaybackStatus::Playing => self.play(),
                }

                view.update(cx, |this, cx| {
                    this.player_page.update(cx, |this, cx| {
                        this.controlbar.update(cx, |this, cx| {
                            this.vol_slider_state.update(cx, |this, cx| {
                                this.set_value(playback_state.volume * 100.0, cx);
                            });
                        });
                    });
                });
            }
            CacherEvent::Thumbnails(thumbnails) => {
                for (id, image) in thumbnails {
                    let evicted = {
                        let thumbnail_cache = cx.global_mut::<ImageCache>();
                        thumbnail_cache.add(id.clone(), image.clone())
                    };

                    if let Some(img) = evicted {
                        drop_image_from_app(cx, img);
                    }
                }
                cx.notify(view.entity_id());
            }
            CacherEvent::AlbumArt(image) => {
                let image_cache = cx.global_mut::<ImageCache>();

                image_cache.current = Some(image.clone());

                cx.notify(view.entity_id());
            }
            CacherEvent::PlaylistThumbnail(id, thumbnail) => {
                let evicted = {
                    let image_cache = cx.global_mut::<ImageCache>();
                    image_cache.add(id.clone(), thumbnail.clone())
                };

                if let Some(img) = evicted {
                    drop_image_from_app(cx, img);
                }
                cx.notify(view.entity_id());
            }
            CacherEvent::MissingAlbumArt(id) => {
                cx.global_mut::<ImageCache>().inflight.remove(id);

                let state = self.state.read(cx);
                let tracks = state.library.tracks.clone();

                let track_id = tracks.iter().find_map(|(tid, track)| { if track.image_id == Some(*id) { Some(tid) } else { None } });

                if let Some(track_id) = track_id {
                    if let Some(track) = tracks.get(track_id) {
                        if let Some(source) = track.get_valid_source() {
                            let _ = self
                                .scanner_tx
                                .send(ScannerCommand::GetCurrentAlbumArt(*track_id, source.path.clone()));
                        }
                    }
                }
            }
            CacherEvent::MissingThumbnails(ids) => {
                let cache = cx.global_mut::<ImageCache>();

                for id in ids {
                    cache.inflight.remove(id);
                }

                let state = self.state.read(cx);
                let tracks = state.library.tracks.clone();

                for id in ids {
                    let track_id = tracks.iter().find_map(|(tid, track)| { if track.image_id == Some(*id) { Some(tid) } else { None } });

                    if let Some(track_id) = track_id {
                        if let Some(track) = tracks.get(track_id) {
                            if let Some(source) = track.get_valid_source() {
                                let _ = self.scanner_tx.send(ScannerCommand::GetTrackMetadata {
                                    path: source.path.clone(),
                                    track_id: *track_id,
                                });
                            }
                        }
                    }
                }
            }
            CacherEvent::MissingPlaylistThumbnail(id) => {
                let state = self.state.read(cx);
                let playlists = state.library.playlists.clone();

                let playlist_id = playlists.iter().find_map(|(pid, playlist)| { if playlist.image_id == Some(*id) { Some(pid) } else { None } });

                if let Some(playlist_id) = playlist_id {
                    if let Some(playlist) = playlists.get(playlist_id) {
                        let playlist_tracks = playlist.tracks.clone();
                        let thumb_tracks = {
                            let state = self.state.read(cx);

                            pick_playlist_thumbnail_tracks(
                                &state.library.tracks,
                                &playlist_tracks,
                                4,
                            )
                        };

                        let _ = self.scanner_tx.send(ScannerCommand::PlaylistThumbnail { id: playlist_id.clone(), tracks: thumb_tracks });
                    }
                }
            }
        }
        Ok(())
    }

    pub fn load_audio(&self, id: &TrackId, cx: &App) {
        let state = self.state.read(cx);
        if let Some(track) = state.library.tracks.get(id) {
            if let Some(source) = track.get_valid_source() {
                let _ = self.audio_tx.send(AudioCommand::Load(*id, source.path.clone()));
                let _ = self
                    .scanner_tx
                    .send(ScannerCommand::GetCurrentAlbumArt(*id, source.path.clone()));
            }
        }
    }

    pub fn load_queue_current(&self, cx: &App) {
        let state = self.state.read(cx);

        if let Some(track_id) = state.queue.get_id(state.playback.current_index)
            && let Some(track) = state.library.tracks.get(&track_id)
        {
            if let Some(source) = track.get_valid_source() {
                let _ = self.audio_tx.send(AudioCommand::Load(track_id, source.path.clone()));
                let _ = self
                    .scanner_tx
                    .send(ScannerCommand::GetCurrentAlbumArt(track_id, source.path.clone()));
            }
        }
    }

    pub fn get_pos(&self) {
        let _ = self.audio_tx.send(AudioCommand::GetPosition);
    }

    pub fn scan_folder(&self, tracks: HashMap<TrackId, Arc<Track>>, path: PathBuf) {
        let _ = self.scanner_tx.send(ScannerCommand::ScanFolder {
            path,
            tracks,
        });
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

    pub fn seek(&self, pos: u64) {
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
        let mut scan_jobs = Vec::new();

        let state = self.state.read(cx);
        let tracks = &state.library.tracks;

        for tid in track_ids {
            if let Some(track) = tracks.get(tid) {
                if let Some(image_id) = track.image_id {
                    cache_ids.push(image_id);
                } else {
                    if let Some(source) = track.get_valid_source() {
                        scan_jobs.push((track.id, source.path.clone()));
                    }
                }
            }
        }

        cx.global_mut::<ImageCache>().request(cache_ids, &self.cacher_tx, ImageKind::Thumbnail);

        for (track_id, path) in scan_jobs {
            let _ = self.scanner_tx.send(ScannerCommand::GetTrackMetadata {
                path,
                track_id,
            });
        }
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

                        pick_playlist_thumbnail_tracks(
                            &state.library.tracks,
                            &playlist_tracks,
                            4,
                        )
                    };

                    let _ = self.scanner_tx.send(ScannerCommand::PlaylistThumbnail { id: *pid, tracks: thumb_tracks });
                }
            }
        }

        cx.global_mut::<ImageCache>().request(cache_ids, &self.cacher_tx, ImageKind::Playlist);
    }
}

impl Global for Controller {}

pub fn pick_playlist_thumbnail_tracks(
    library_tracks: &HashMap<TrackId, Arc<Track>>,
    playlist_tracks: &[TrackId],
    count: usize,
) -> Vec<PathBuf> {
    let mut rng = rand::rng();
    let mut chosen = Vec::with_capacity(count);
    let mut albums = HashSet::with_capacity(count);

    let candidates = playlist_tracks
        .iter()
        .copied()
        .sample(&mut rng, count * 3);

    for id in candidates {
        if let Some(track) = library_tracks.get(&id) {
            if albums.insert(track.album.clone()) {
                if let Some(source) = track.get_valid_source() {
                    chosen.push(source.path.clone());
                }
            }
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

            if let Some(track) = library_tracks.get(id) {
                if albums.insert(track.album.clone()) {
                    if let Some(source) = track.get_valid_source() {
                        chosen.push(source.path.clone());
                    }
                }
            }
        }
    }

    chosen
}