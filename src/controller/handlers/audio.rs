use super::{Controller, App, AudioEvent, Entity, Wiremann, ControllerError, Duration, duration_to_slider, SystemIntegrationCommand, CacherCommand, ScannerCommand, HashSet, ImageKind, ImageProcessorCommand, LyricsState, LyricsStatus};

impl Controller {
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
                                        Duration::default()
                                    };
                                    this.set_value(duration_to_slider(*pos, duration), cx);
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
                            duration: track.duration.as_secs(),
                        })
                        .ok();

                    self.cacher_tx
                        .send(CacherCommand::GetLyrics(*track_id))
                        .ok();

                    let lyrics_state = cx.global::<LyricsState>().0.clone();

                    lyrics_state.update(cx, |this, cx| {
                        this.status = LyricsStatus::Fetching;
                        this.lyrics = None;
                        this.track_id = Some(*track_id);

                        cx.notify();
                    });
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
                cx.notify(view.entity_id());
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
}
