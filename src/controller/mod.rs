pub mod commands;
pub mod events;
pub mod state;
use crate::cacher::ImageKind;
use crate::controller::commands::{CacherCommand, ImageProcessorCommand, SystemIntegrationCommand};
use crate::controller::events::{CacherEvent, ImageProcessorEvent, SystemIntegrationEvent};
use crate::controller::state::PlaybackStatus;
use crate::library::playlists::PlaylistId;
use crate::library::{Track, TrackId};
use crate::ui::components::toasts::scanning_status::{ScanningStatus, ScanningStatusToast};
use crate::ui::components::toasts::{Toast, ToastKind, ToastPhase};
use crate::ui::helpers::{drop_image_from_app, secs_to_slider};
use crate::ui::wiremann::Wiremann;
use crate::{
    controller::state::AppState, errors::ControllerError, ui::components::image_cache::ImageCache,
};
use commands::{AudioCommand, ScannerCommand};
use crossbeam_channel::{Receiver, Sender};
use events::{AudioEvent, ScannerEvent};
use gpui::{App, AppContext, Entity, Global};
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
        }
    }

    #[allow(clippy::missing_errors_doc, clippy::too_many_lines)]
    pub fn handle_audio_event(
        &mut self,
        cx: &mut App,
        event: &AudioEvent,
        view: &Entity<Wiremann>,
    ) -> Result<(), ControllerError> {
        match event {
            AudioEvent::Position(pos) => {
                let last_pos = self.state.read(cx).playback.position;

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
                                    cx.notify();
                                });
                            });
                        });
                        cx.notify();
                    });
                    self.state.update(cx, |this, cx| {
                        this.playback.position = *pos;
                        cx.notify();
                    });

                    self.system_integration_tx
                        .send(SystemIntegrationCommand::SetPosition(*pos))
                        .ok();

                    let state = self.state.read(cx).playback.clone();
                    let _ = self
                        .cacher_tx
                        .send(CacherCommand::WritePlaybackState(state));
                }
            }
            AudioEvent::TrackLoaded(track_id, path) => {
                let state = self.state.read(cx);
                if !state.library.tracks.contains_key(track_id) {
                    let _ = self
                        .scanner_tx
                        .send(ScannerCommand::ScanTrack(path.clone()));
                }

                if let Some(track) = state.library.tracks.get(track_id) {
                    if let Some(image_id) = track.image_id {
                        let _ = self.cacher_tx.send(CacherCommand::GetImage(
                            HashSet::from([image_id]),
                            ImageKind::AlbumArt,
                        ));
                    } else {
                        let _ = self.image_processor_tx.send(
                            ImageProcessorCommand::GetCurrentAlbumArt(*track_id, path.clone()),
                        );
                    }

                    self.system_integration_tx
                        .send(SystemIntegrationCommand::SetMetadata {
                            title: track.title.clone(),
                            artist: track.artist.clone(),
                            album: track.album.clone(),
                            image: None,
                            duration: track.duration,
                        })
                        .ok();
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
                self.system_integration_tx
                    .send(SystemIntegrationCommand::SetPlaybackStatus(
                        *status,
                        state.position,
                    ))
                    .ok();
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

    #[allow(clippy::missing_errors_doc, clippy::too_many_lines)]
    pub fn handle_scanner_event(
        &mut self,
        cx: &mut App,
        event: &ScannerEvent,
        view: &Entity<Wiremann>,
    ) -> Result<(), ControllerError> {
        match event {
            ScannerEvent::UpsertTracks(tracks) => {
                let mut modified_playlists = HashSet::new();
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
                                existing.title.clone_from(&track.title);
                            }

                            if existing.artist.is_empty() && !track.artist.is_empty() {
                                existing.artist.clone_from(&track.artist);
                            }

                            if existing.album.is_empty() && !track.album.is_empty() {
                                existing.album.clone_from(&track.album);
                            }
                        } else {
                            this.library.tracks.insert(id, Arc::new(track.clone()));
                        }

                        if let Some(pid) = playlist_id
                            && let Some(playlist) = this.library.playlists.get_mut(pid)
                        {
                            if !playlist.tracks.contains(&id) {
                                playlist.tracks.push(id);
                            }
                            modified_playlists.insert(*pid);
                        }
                    }
                    cx.notify();
                });
                let state = self.state.read(cx).library.clone();
                let _ = self.cacher_tx.send(CacherCommand::WriteLibraryState(state));
            }
            ScannerEvent::InsertTracksIntoPlaylist(pid, tids) => {
                self.state.update(cx, |this, cx| {
                    if let Some(playlist) = this.library.playlists.get_mut(pid) {
                        for tid in tids {
                            if !playlist.tracks.contains(tid) {
                                playlist.tracks.push(*tid);
                            }
                        }
                    }
                    cx.notify();
                });
                let state = self.state.read(cx).library.clone();
                let _ = self.cacher_tx.send(CacherCommand::WriteLibraryState(state));
            }
            ScannerEvent::AddTrackSource(id, source) => {
                self.state.update(cx, |this, cx| {
                    if let Some(track) = this.library.tracks.get_mut(id) {
                        Arc::make_mut(track).sources.push(source.clone());
                    }

                    cx.notify();
                });
                let state = self.state.read(cx).library.clone();
                let _ = self.cacher_tx.send(CacherCommand::WriteLibraryState(state));
            }
            ScannerEvent::RemoveTrackSource(id, path) => {
                self.state.update(cx, |this, cx| {
                    if let Some(track) = this.library.tracks.get_mut(id)
                        && let Some(source) =
                            track.sources.iter().position(|this| this.path == *path)
                    {
                        Arc::make_mut(track).sources.remove(source);
                    }

                    cx.notify();
                });
                let state = self.state.read(cx).library.clone();
                let _ = self.cacher_tx.send(CacherCommand::WriteLibraryState(state));
            }
            ScannerEvent::InsertPlaylist(playlist) => {
                self.state.update(cx, |this, cx| {
                    this.library.playlists.insert(playlist.id, playlist.clone());

                    cx.notify();
                });

                let state = self.state.read(cx).library.clone();
                let _ = self.cacher_tx.send(CacherCommand::WriteLibraryState(state));
            }
            ScannerEvent::ScanFinished => {
                self.scanner_tx.send(ScannerCommand::StartNextScan).ok();
                let tracks = self.state.read(cx).library.tracks.clone();

                let to_request: HashSet<(TrackId, PathBuf)> = tracks
                    .iter()
                    .filter(|(_, track)| track.image_id.is_none())
                    .filter_map(|(id, track)| {
                        track
                            .get_valid_source()
                            .map(|src| src.path.clone())
                            .map(|path| (*id, path))
                    })
                    .collect();
                let _ = self
                    .image_processor_tx
                    .send(ImageProcessorCommand::GetThumbnails(
                        to_request,
                        ImageKind::ThumbnailSmall,
                    ));

                // self.request_playlist_thumbnails(
                //     &modified_playlists
                //         .iter()
                //         .copied()
                //         .collect::<Vec<PlaylistId>>(),
                //     cx,
                // );
            }
            ScannerEvent::ScanStarted => {
                let scanning_status = cx.global_mut::<ScanningStatus>().clone().0;

                scanning_status.update(cx, |this, cx| {
                    this.is_scanning = true;
                    this.is_discovering = true;

                    cx.notify()
                });

                view.update(cx, |this, cx| {
                    this.toast_manager.update(cx, |this, cx| {
                        this.info("Scanning started...", cx);
                        this.scanning_status(cx);
                    });
                    cx.notify();
                });
            }
            ScannerEvent::Discovered(discovered) => {
                let scanning_status = cx.global_mut::<ScanningStatus>().0.clone();

                scanning_status.update(cx, |this, cx| {
                    if !this.is_discovering {
                        this.is_discovering = true
                    }

                    this.discovered = *discovered;

                    cx.notify();
                })
            }
            ScannerEvent::Processed { processed, total } => {
                let scanning_status = cx.global_mut::<ScanningStatus>().0.clone();

                scanning_status.update(cx, |this, cx| {
                    if this.is_discovering {
                        this.is_discovering = false
                    }
                    if !this.is_processing {
                        this.is_processing = true
                    }

                    this.total = *total;
                    this.processed = *processed;
                    cx.notify();
                })
            }
        }
        Ok(())
    }

    #[allow(clippy::missing_errors_doc)]
    pub fn handle_image_processor_event(
        &mut self,
        cx: &mut App,
        event: &ImageProcessorEvent,
        view: &Entity<Wiremann>,
    ) -> Result<(), ControllerError> {
        match event {
            ImageProcessorEvent::InsertAlbumArt(image_id, image) => {
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
                        image: image.clone(),
                    });
                    let state = self.state.read(cx);
                    if let Some(track_id) = &state.playback.current
                        && let Some(track) = state.library.tracks.get(track_id)
                    {
                        self.system_integration_tx
                            .send(SystemIntegrationCommand::SetMetadata {
                                title: track.title.clone(),
                                artist: track.artist.clone(),
                                album: track.album.clone(),
                                image: Some((width, height, image)),
                                duration: track.duration,
                            })
                            .ok();
                    }
                }

                cx.notify(view.entity_id());
            }
            ImageProcessorEvent::InsertThumbnails(thumbnails, kind) => {
                for (id, image) in thumbnails {
                    let width = image.size(0).width.0.cast_unsigned();
                    let height = image.size(0).height.0.cast_unsigned();
                    if let Some(image) = image.as_bytes(0) {
                        let image = image.to_vec();
                        let _ = self.cacher_tx.send(CacherCommand::WriteImage {
                            id: *id,
                            kind: *kind,
                            width,
                            height,
                            image,
                        });
                    }

                    let evicted = {
                        let thumbnail_cache = cx.global_mut::<ImageCache>();
                        thumbnail_cache.inflight.remove(id);
                        thumbnail_cache.add(*id, image.clone())
                    };

                    if let Some(img) = evicted {
                        drop_image_from_app(cx, img);
                    }
                }
                cx.notify(view.entity_id());
            }
            ImageProcessorEvent::UpdateImageLookup(lookup) => {
                self.state.update(cx, |this, cx| {
                    for (id, image_id) in lookup {
                        if let Some(track) = this.library.tracks.get_mut(id) {
                            Arc::make_mut(track).image_id = Some(*image_id);
                        }
                    }

                    cx.notify();
                });
                let state = self.state.read(cx).library.clone();
                let _ = self.cacher_tx.send(CacherCommand::WriteLibraryState(state));
            }
            ImageProcessorEvent::InsertPlaylistThumbnail(id, image_id, image) => {
                let thumbnail_cache = cx.global_mut::<ImageCache>();

                thumbnail_cache.add(*image_id, image.clone());

                thumbnail_cache.inflight.remove(image_id);

                let _ = self
                    .image_processor_tx
                    .send(ImageProcessorCommand::PlaylistJobFinished(*id));

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

                self.state.update(cx, |this, cx| {
                    if let Some(playlist) = this.library.playlists.get_mut(id) {
                        playlist.image_id = Some(*image_id);
                    }
                    cx.notify();
                });
                let state = self.state.read(cx).library.clone();
                let _ = self.cacher_tx.send(CacherCommand::WriteLibraryState(state));
            }
        }

        Ok(())
    }

    #[allow(clippy::missing_errors_doc, clippy::too_many_lines)]
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

                let duration = if let Some(current) = playback_state.current
                    && let Some(track) = state.library.tracks.get(&current)
                {
                    Some(track.duration)
                } else {
                    None
                };

                view.update(cx, |this, cx| {
                    this.player_page.update(cx, |this, cx| {
                        this.controlbar.update(cx, |this, cx| {
                            this.vol_slider_state.update(cx, |this, cx| {
                                this.set_value(playback_state.volume * 100.0, cx);
                            });
                            this.playback_slider_state.update(cx, |this, cx| {
                                if let Some(duration) = duration {
                                    this.set_value(
                                        secs_to_slider(playback_state.position, duration),
                                        cx,
                                    );
                                }
                            });
                        });
                    });
                });
            }
            CacherEvent::Thumbnails(thumbnails) => {
                for (id, image) in thumbnails {
                    let evicted = {
                        let thumbnail_cache = cx.global_mut::<ImageCache>();
                        thumbnail_cache.inflight.remove(id);
                        thumbnail_cache.add(*id, image.clone())
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

                let image = image.clone();
                let width = image.size(0).width.0.cast_unsigned();
                let height = image.size(0).height.0.cast_unsigned();
                if let Some(image) = image.as_bytes(0) {
                    let image = image.to_vec();
                    let state = self.state.read(cx);
                    if let Some(track_id) = &state.playback.current
                        && let Some(track) = state.library.tracks.get(track_id)
                    {
                        self.system_integration_tx
                            .send(SystemIntegrationCommand::SetMetadata {
                                title: track.title.clone(),
                                artist: track.artist.clone(),
                                album: track.album.clone(),
                                image: Some((width, height, image)),
                                duration: track.duration,
                            })
                            .ok();
                    }
                }
                cx.notify(view.entity_id());
            }
            CacherEvent::PlaylistThumbnail(id, thumbnail) => {
                cx.global_mut::<ImageCache>().inflight.remove(id);

                let evicted = {
                    let image_cache = cx.global_mut::<ImageCache>();
                    image_cache.add(*id, thumbnail.clone())
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

                let track_id = tracks.iter().find_map(|(tid, track)| {
                    if track.image_id == Some(*id) {
                        Some(tid)
                    } else {
                        None
                    }
                });

                if let Some(track_id) = track_id
                    && let Some(track) = tracks.get(track_id)
                    && let Some(source) = track.get_valid_source()
                {
                    let _ =
                        self.image_processor_tx
                            .send(ImageProcessorCommand::GetCurrentAlbumArt(
                                *track_id,
                                source.path.clone(),
                            ));
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
                    let track_id = tracks.iter().find_map(|(tid, track)| {
                        if track.image_id == Some(*id) {
                            Some(tid)
                        } else {
                            None
                        }
                    });

                    if let Some(track_id) = track_id
                        && let Some(track) = tracks.get(track_id)
                        && let Some(source) = track.get_valid_source()
                    {
                        let mut set = HashSet::new();
                        set.insert((*track_id, source.path.clone()));
                        let _ = self
                            .image_processor_tx
                            .send(ImageProcessorCommand::GetThumbnails(
                                set,
                                ImageKind::ThumbnailSmall,
                            ));
                    }
                }
            }
            CacherEvent::MissingPlaylistThumbnail(id) => {
                cx.global_mut::<ImageCache>().inflight.remove(id);

                let state = self.state.read(cx);
                let playlists = state.library.playlists.clone();

                let playlist_id = playlists.iter().find_map(|(pid, playlist)| {
                    if playlist.image_id == Some(*id) {
                        Some(pid)
                    } else {
                        None
                    }
                });

                if let Some(playlist_id) = playlist_id
                    && let Some(playlist) = playlists.get(playlist_id)
                {
                    let playlist_tracks = playlist.tracks.clone();
                    let thumb_tracks = {
                        let state = self.state.read(cx);

                        pick_playlist_thumbnail_tracks(&state.library.tracks, &playlist_tracks, 4)
                    };

                    let _ =
                        self.image_processor_tx
                            .send(ImageProcessorCommand::PlaylistThumbnail {
                                id: *playlist_id,
                                tracks: thumb_tracks,
                            });
                }
            }
        }
        Ok(())
    }

    #[allow(clippy::missing_errors_doc, clippy::too_many_lines)]
    pub fn handle_system_integration_event(
        &mut self,
        cx: &mut App,
        event: &SystemIntegrationEvent,
        _view: &Entity<Wiremann>,
    ) -> Result<(), ControllerError> {
        match event {
            SystemIntegrationEvent::PlayPause => {
                let status = self.state.read(cx).playback.status;

                if status == PlaybackStatus::Stopped || status == PlaybackStatus::Paused {
                    self.play();
                } else {
                    self.pause();
                }
            }
            SystemIntegrationEvent::Play => {
                self.play();
            }
            SystemIntegrationEvent::Pause => {
                self.pause();
            }
            SystemIntegrationEvent::Stop => {
                self.stop();
            }
            SystemIntegrationEvent::Next => {
                self.next(cx);
            }
            SystemIntegrationEvent::Prev => {
                self.prev(cx);
            }
            SystemIntegrationEvent::SeekForward(duration) => {
                let pos = self.state.read(cx).playback.position;

                self.seek(pos.saturating_add(*duration));
            }
            SystemIntegrationEvent::SeekBackward(duration) => {
                let pos = self.state.read(cx).playback.position;

                self.seek(pos.saturating_sub(*duration));
            }
            SystemIntegrationEvent::Volume(vol) => {
                #[allow(clippy::cast_possible_truncation)]
                self.set_volume(*vol as f32, cx);
            }
            SystemIntegrationEvent::Position(pos) => {
                self.seek(*pos);
            }
        }

        Ok(())
    }

    pub fn load_audio(&self, id: &TrackId, cx: &App) {
        let state = self.state.read(cx);
        if let Some(track) = state.library.tracks.get(id)
            && let Some(source) = track.get_valid_source()
        {
            let _ = self
                .audio_tx
                .send(AudioCommand::Load(*id, source.path.clone()));
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
