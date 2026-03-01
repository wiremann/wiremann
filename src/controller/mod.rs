pub mod commands;
pub mod events;
pub mod state;
use crate::controller::commands::CacherCommand;
use crate::controller::events::CacherEvent;
use crate::library::TrackId;
use crate::ui::helpers::secs_to_slider;
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
use rand::seq::SliceRandom;
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

    pub fn handle_audio_event(
        &mut self,
        cx: &mut App,
        event: &AudioEvent,
        view: Entity<Wiremann>,
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
                                this.set_value(secs_to_slider(pos.clone(), duration), cx);
                            });
                        });
                    });
                });
                self.state.update(cx, |this, cx| {
                    this.playback.position = *pos;
                    cx.notify();
                });
            }
            AudioEvent::TrackLoaded(path) => {
                let track_id = gen_track_id(path)?;
                if !self.state.read(cx).library.tracks.contains_key(&track_id) {
                    let _ = self.scanner_tx.send(ScannerCommand::GetTrackMetadata {
                        path: path.clone(),
                        track_id: track_id.clone(),
                    });
                }

                self.state.update(cx, |this, cx| {
                    this.playback.current = Some(track_id);

                    if let Some(idx) = this.queue.get_index(track_id) {
                        this.queue.index = idx;
                    }

                    cx.notify();
                });
            }
            AudioEvent::PlaybackStatus(status) => self.state.update(cx, |this, cx| {
                this.playback.status = *status;
                cx.notify()
            }),
            AudioEvent::TrackEnded => {
                let repeat = self.state.read(cx).playback.repeat;

                if repeat {
                    self.load_queue_current(cx)
                } else {
                    self.next(cx)
                }
            }
        }
        Ok(())
    }

    pub fn handle_scanner_event(
        &mut self,
        cx: &mut App,
        event: &ScannerEvent,
        view: Entity<Wiremann>,
    ) -> Result<(), ControllerError> {
        match event {
            ScannerEvent::Tracks(tracks) => {
                self.state.update(cx, |this, cx| {
                    this.library.tracks.reserve(tracks.len());
                    for track in tracks {
                        this.library
                            .tracks
                            .insert(track.id.clone(), Arc::new(track.clone()));
                    }
                    cx.notify();
                });
            }
            ScannerEvent::Playlist(playlist) => {
                self.state.update(cx, |this, cx| {
                    this.library
                        .playlists
                        .insert(playlist.id.clone(), playlist.clone());
                    this.playback.current_playlist = Some(playlist.id.clone());
                    this.queue.tracks = playlist.tracks.clone();
                    this.queue.order = (0..playlist.tracks.len()).collect();

                    cx.notify();
                });
                let state = self.state.read(cx).clone();
                let _ = self.cacher_tx.send(CacherCommand::WriteAppState(state));
            }
            ScannerEvent::AlbumArt(image) => {
                let mut image_cache = cx.global_mut::<ImageCache>();

                image_cache.current = Some(image.clone());

                cx.notify(view.entity_id());
            }
            ScannerEvent::Thumbnails(thumbnails) => {
                let mut thumbnail_cache = cx.global_mut::<ImageCache>();

                thumbnail_cache.thumbs.extend(thumbnails.clone());
            }
        }
        Ok(())
    }

    pub fn handle_cacher_event(
        &mut self,
        cx: &mut App,
        event: &CacherEvent,
        _view: Entity<Wiremann>,
    ) -> Result<(), ControllerError> {
        match event {
            CacherEvent::AppState(state) => self.state.update(cx, |this, cx| *this = state.clone()),
            _ => {}
        }
        Ok(())
    }

    pub fn load_audio(&self, path: PathBuf) {
        let _ = self.audio_tx.send(AudioCommand::Load(path.clone()));
        let _ = self
            .scanner_tx
            .send(ScannerCommand::GetCurrentAlbumArt(path));
    }

    pub fn load_queue_current(&self, cx: &App) {
        let queue = &self.state.read(cx).queue;
        let library = &self.state.read(cx).library;

        if let Some(track_id) = queue.get_id() {
            if let Some(track) = library.tracks.get(&track_id) {
                let path = track.path.clone();
                let _ = self.audio_tx.send(AudioCommand::Load(path.clone()));
                let _ = self
                    .scanner_tx
                    .send(ScannerCommand::GetCurrentAlbumArt(path));
            }
        }
    }

    pub fn get_pos(&self) {
        let _ = self.audio_tx.send(AudioCommand::GetPosition);
    }

    pub fn scan_folder(&self, tracks: HashSet<TrackId>, path: PathBuf) {
        let _ = self
            .scanner_tx
            .send(ScannerCommand::ScanFolder { path, tracks });
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
        self.state.update(cx, |this, cx| {
            this.playback.repeat = !this.playback.repeat;
        })
    }

    pub fn set_mute(&self, cx: &mut App) {
        self.state.update(cx, |this, cx| {
            this.playback.mute = !this.playback.mute;

            let _ = self.audio_tx.send(AudioCommand::SetVolume(if this.playback.mute { 0.0 } else { this.playback.volume }));
        })
    }

    pub fn set_volume(&self, vol: f32, cx: &mut App) {
        self.state.update(cx, |this, _| {
            this.playback.volume = vol;
        });

        let muted = self.state.read(cx).playback.mute;

        let _ = self.audio_tx.send(AudioCommand::SetVolume(if muted { 0.0 } else { vol }));
    }

    pub fn set_shuffle(&self, cx: &mut App) {
        self.state.update(cx, |this, cx| {
            this.playback.shuffling = !this.playback.shuffling;

            if this.queue.tracks.is_empty() {
                return;
            }

            let current = this.queue.order[this.queue.index];

            if this.playback.shuffling {
                let mut rng = rng();
                this.queue.order =
                    (0..this.queue.tracks.len()).collect();

                this.queue.order.shuffle(&mut rng);

                if let Some(pos) =
                    this.queue.order.iter().position(|&x| x == current)
                {
                    this.queue.order.swap(0, pos);
                }

                this.queue.index = 0;
            } else {
                this.queue.order = (0..this.queue.tracks.len()).collect();

                this.queue.index = current;
            }
        });

        let state = self.state.read(cx).clone();
        let _ = self.cacher_tx.send(CacherCommand::WriteAppState(state));
    }

    pub fn next(&self, cx: &mut App) {
        self.state.update(cx, |this, cx| {
            this.queue.index = (this.queue.index + 1).clamp(0, this.library.tracks.len());
        });

        self.load_queue_current(cx);
    }
    pub fn prev(&self, cx: &mut App) {
        self.state.update(cx, |this, cx| {
            this.queue.index = this.queue.index.saturating_sub(1);
        });

        self.load_queue_current(cx);
    }

    pub fn seek(&self, pos: u64) {
        let _ = self.audio_tx.send(AudioCommand::Seek(pos));
    }

    pub fn check_track_ended(&self) {
        let _ = self.audio_tx.send(AudioCommand::CheckTrackEnded);
    }
}

impl Global for Controller {}
