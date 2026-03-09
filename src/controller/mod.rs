pub mod commands;
pub mod events;
pub mod state;
use crate::cacher::ImageKind;
use crate::controller::commands::CacherCommand;
use crate::controller::events::CacherEvent;
use crate::controller::state::PlaybackStatus;
use crate::library::TrackId;
use crate::ui::helpers::{drop_image_from_app, secs_to_slider};
use crate::ui::wiremann::Wiremann;
use crate::{
    controller::state::AppState, errors::ControllerError, library::gen_track_id,
    ui::components::image_cache::ImageCache,
};
use commands::{AudioCommand, ScannerCommand};
use crossbeam_channel::{Receiver, Sender};
use events::{AudioEvent, ScannerEvent};
use gpui::{App, Entity, Global};
use rand::rng;
use rand::seq::{IteratorRandom, SliceRandom};
use std::collections::HashSet;
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
            AudioEvent::TrackLoaded(path) => {
                let track_id = gen_track_id(path)?;
                let state = self.state.read(cx);
                if !state.library.tracks.contains_key(&track_id) {
                    let _ = self.scanner_tx.send(ScannerCommand::GetTrackMetadata {
                        path: path.clone(),
                        track_id,
                    });
                }

                if let Some(image_id) = state.library.image_lookup.get(&track_id) {
                    let _ = self.cacher_tx.send(CacherCommand::GetAlbumArt(image_id.clone()));
                } else {
                    let _ = self.scanner_tx.send(ScannerCommand::GetCurrentAlbumArt(path.clone()));
                }

                self.state.update(cx, |this, cx| {
                    this.playback.current = Some(track_id);

                    if let Some(idx) = this.queue.get_index(track_id) {
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
            ScannerEvent::Tracks(tracks) => {
                self.state.update(cx, |this, cx| {
                    this.library.tracks.reserve(tracks.len());
                    for track in tracks {
                        this.library
                            .tracks
                            .insert(track.id, Arc::new(track.clone()));
                    }
                    cx.notify();
                });
                let state = self.state.read(cx).library.clone();
                let _ = self.cacher_tx.send(CacherCommand::WriteLibraryState(state));
            }
            ScannerEvent::Playlist(playlist) => {
                let id = playlist.id;
                let playlist_tracks = playlist.tracks.clone();

                let thumb_tracks = {
                    let state = self.state.read(cx);
                    let library_tracks = &state.library.tracks;

                    let mut rng = rand::rng();
                    let mut chosen = Vec::with_capacity(4);
                    let mut albums = HashSet::with_capacity(4);

                    let candidates = playlist_tracks
                        .iter()
                        .copied()
                        .sample(&mut rng, 12);

                    for id in candidates {
                        if let Some(track) = library_tracks.get(&id) {
                            if albums.insert(track.album.clone()) {
                                chosen.push(track.path.clone());
                            }
                        }

                        if chosen.len() == 4 {
                            break;
                        }
                    }

                    if chosen.len() < 4 {
                        for id in &playlist_tracks {
                            if chosen.len() == 4 {
                                break;
                            }

                            if let Some(track) = library_tracks.get(id) {
                                if albums.insert(track.album.clone()) {
                                    chosen.push(track.path.clone());
                                }
                            }
                        }
                    }

                    chosen
                };

                self.state.update(cx, |this, cx| {
                    this.library
                        .playlists
                        .insert(playlist.id, playlist.clone());
                    this.playback.current_playlist = Some(playlist.id);
                    this.queue.tracks.clone_from(&playlist.tracks);
                    this.queue.order = (0..playlist.tracks.len()).collect();

                    cx.notify();
                });

                let _ = self.scanner_tx.send(ScannerCommand::PlaylistThumbnail { id, tracks: thumb_tracks });

                let state = self.state.read(cx).queue.clone();
                let _ = self.cacher_tx.send(CacherCommand::WriteQueueState(state));
            }
            ScannerEvent::AlbumArt(image_id, image) => {
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
            ScannerEvent::Thumbnails(thumbnails) => {
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
                        thumbnail_cache.add_track(id.clone(), image.clone())
                    };

                    if let Some(img) = evicted {
                        drop_image_from_app(cx, img);
                    }
                }
            }
            ScannerEvent::ImageLookup(lookup) => {
                self.state.update(cx, |this, _| {
                    this.library.image_lookup.extend(lookup.clone());
                });
            }
            ScannerEvent::PlaylistThumbnail(id, thumbnail) => {
                let thumbnail_cache = cx.global_mut::<ImageCache>();

                thumbnail_cache.playlist_thumbs.put(id.clone(), thumbnail.clone());
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
                        thumbnail_cache.add_track(id.clone(), image.clone())
                    };

                    if let Some(img) = evicted {
                        drop_image_from_app(cx, img);
                    }
                }
            }
            CacherEvent::AlbumArt(image) => {
                let image_cache = cx.global_mut::<ImageCache>();

                image_cache.current = Some(image.clone());

                cx.notify(view.entity_id());
            }
            CacherEvent::MissingAlbumArt(id) => {
                let state = self.state.read(cx);
                let lookup = state.library.image_lookup.clone();
                let tracks = state.library.tracks.clone();

                let track_id = lookup.iter().find_map(|(track, image)| { if image == id { Some(track) } else { None } });

                if let Some(track_id) = track_id {
                    if let Some(track) = tracks.get(track_id) {
                        let _ = self
                            .scanner_tx
                            .send(ScannerCommand::GetCurrentAlbumArt(track.path.clone()));
                    }
                }
            }
            CacherEvent::MissingThumbnails(ids) => {
                let state = self.state.read(cx);
                let lookup = state.library.image_lookup.clone();
                let tracks = state.library.tracks.clone();

                for id in ids {
                    let track_id = lookup.iter().find_map(|(track, image)| { if image == id { Some(track) } else { None } });

                    if let Some(track_id) = track_id {
                        if let Some(track) = tracks.get(track_id) {
                            let path = track.path.clone();
                            let _ = self.scanner_tx.send(ScannerCommand::GetTrackMetadata {
                                path,
                                track_id: *track_id,
                            });
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub fn load_audio(&self, path: PathBuf) {
        let _ = self.audio_tx.send(AudioCommand::Load(path.clone()));
    }

    pub fn load_queue_current(&self, cx: &App) {
        let state = self.state.read(cx);

        if let Some(track_id) = state.queue.get_id(state.playback.current_index)
            && let Some(track) = state.library.tracks.get(&track_id)
        {
            let path = track.path.clone();
            let _ = self.audio_tx.send(AudioCommand::Load(path.clone()));
            let _ = self
                .scanner_tx
                .send(ScannerCommand::GetCurrentAlbumArt(path));
        }
    }

    pub fn get_pos(&self) {
        let _ = self.audio_tx.send(AudioCommand::GetPosition);
    }

    pub fn scan_folder(&self, tracks: &HashSet<TrackId>, path: PathBuf) {
        let _ = self.scanner_tx.send(ScannerCommand::ScanFolder {
            path,
            tracks: tracks.clone(),
        });
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
}

impl Global for Controller {}
