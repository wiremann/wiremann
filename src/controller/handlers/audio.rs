use super::{
    App, AudioEvent, CacherCommand, Controller, ControllerError, Duration, Entity, HashSet,
    ImageKind, ImageProcessorCommand, Instant, PlaybackStatus, ScannerCommand,
    SystemIntegrationCommand, Wiremann, duration_to_slider,
};
use crate::ui::pages::player::lyrics::{LyricsState, LyricsStatus};

/// How often `session.ron` is refreshed while the playback position ticks.
/// Position changes stream in every frame (~16 ms); persisting each one would
/// hammer the disk. State transitions (track load, pause, stop) still write
/// immediately, so the worst case loss on a crash is a few seconds of position.
const PLAYBACK_STATE_PERSIST_INTERVAL: Duration = Duration::from_secs(5);

/// Helper: if the title looks like "Artist - RealTitle" and the artist is
/// unknown or matches the prefix, split it apart.
fn clean_track_title(title: &mut String, artist: &mut String) {
    if let Some(idx) = title.find(" - ") {
        let prefix = title[..idx].trim().to_string();
        let suffix = title[idx + 3..].trim().to_string();
        if !prefix.is_empty() && !suffix.is_empty() {
            let do_clean = artist.is_empty()
                || artist.eq_ignore_ascii_case("Unknown Artist")
                || artist.eq_ignore_ascii_case(&prefix);
            if do_clean {
                *artist = prefix;
                *title = suffix;
            }
        }
    }
}

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
                    view.update(&mut *cx, |this, cx| {
                        this.player_page.update(&mut *cx, |this, cx| {
                            this.controlbar.update(&mut *cx, |this, cx| {
                                this.playback_slider_state.update(&mut *cx, |this, cx| {
                                    let duration = {
                                        let state = cx.global::<Controller>().state.read(cx);
                                        state
                                            .playback
                                            .current
                                            .and_then(|id| state.library.tracks.get(&id))
                                            .map_or(Duration::default(), |track| track.duration)
                                    };
                                    this.set_value(duration_to_slider(*pos, duration), cx);
                                    cx.notify();
                                });
                            });
                        });
                        cx.notify();
                    });
                    let should_persist = self.state.update(&mut *cx, |this, cx| {
                        this.playback.position = *pos;

                        if let Some(session) = this.metrics_session.as_mut() {
                            if *pos > session.last_position {
                                let delta = *pos - session.last_position;

                                if this.playback.status == PlaybackStatus::Playing {
                                    session.played += delta;
                                }
                            }

                            session.last_position = *pos;
                        }

                        let now = Instant::now();
                        let should_persist = this.last_playback_write.map_or(true, |last| {
                            now.duration_since(last) >= PLAYBACK_STATE_PERSIST_INTERVAL
                        });

                        if should_persist {
                            this.last_playback_write = Some(now);
                        }

                        cx.notify();

                        should_persist
                    });

                    self.system_integration_tx
                        .send(SystemIntegrationCommand::SetPosition(*pos))
                        .ok();

                    if should_persist {
                        let state = self.state.read(cx).playback.clone();
                        let _ = self
                            .cacher_tx
                            .send(CacherCommand::WritePlaybackState(state));
                    }
                }
            }
            AudioEvent::TrackLoaded(track_id, path) => {
                let state = self.state.read(cx);
                if !state.library.tracks.contains_key(track_id) {
                    let _ = self
                        .scanner_tx
                        .send(ScannerCommand::ScanTrack(path.clone()));
                }

                // Always send album art extraction — the path is valid even if the track
                // hasn't been scanned into the library yet (race condition fix).
                self.image_processor_tx
                    .send(ImageProcessorCommand::GetCurrentAlbumArt(
                        *track_id,
                        path.clone(),
                    ))
                    .ok();

                if let Some(track) = state.library.tracks.get(track_id) {
                    // Also request cached album art if image_id is already known
                    if let Some(image_id) = track.image_id {
                        let _ = self.cacher_tx.send(CacherCommand::GetImage(
                            HashSet::from([image_id]),
                            ImageKind::AlbumArt,
                        ));
                    } else {
                        // No known album art — try online fallback
                        let mut t = track.title.to_string();
                        let mut a = track
                            .artists(&state.library)
                            .map(|a| a.name.to_string())
                            .collect::<Vec<_>>()
                            .join(", ");
                        let album = track
                            .album(&state.library)
                            .map(|a| a.name.to_string())
                            .unwrap_or_default();
                        clean_track_title(&mut t, &mut a);
                        self.image_processor_tx
                            .send(ImageProcessorCommand::FetchAlbumArtOnline {
                                id: *track_id,
                                title: t,
                                artist: a,
                                album,
                            })
                            .ok();
                    }

                    let artist_str = track
                        .artists(&state.library)
                        .map(|a| a.name.to_string())
                        .collect::<Vec<_>>()
                        .join(", ");
                    let album_str = track
                        .album(&state.library)
                        .map(|a| a.name.to_string())
                        .unwrap_or_default();

                    self.system_integration_tx
                        .send(SystemIntegrationCommand::SetMetadata {
                            title: track.title.to_string(),
                            artist: artist_str,
                            album: album_str,
                            image: None,
                            duration: track.duration.as_secs(),
                        })
                        .ok();

                    self.cacher_tx
                        .send(CacherCommand::GetLyrics(*track_id))
                        .ok();

                    let lyrics_state = cx.global::<LyricsState>().0.clone();

                    lyrics_state.update(&mut *cx, |this, cx| {
                        this.status = LyricsStatus::Fetching;
                        this.lyrics = None;
                        this.track_id = Some(*track_id);

                        cx.notify();
                    });
                }
                drop(state);
                self.state.update(&mut *cx, |this, cx| {
                    this.playback.current = Some(*track_id);

                    if let Some(idx) = this.queue.get_index(*track_id) {
                        this.playback.current_index = idx;
                    }

                    cx.notify();
                });

                {
                    let prev = self
                        .state
                        .read(cx)
                        .metrics_session
                        .as_ref()
                        .map(|s| s.track_id);

                    if prev != Some(*track_id) {
                        self.finalize_metrics_session(cx, false);
                        self.start_metrics_session(*track_id, cx);
                    }
                }

                let state = self.state.read(cx).playback.clone();
                let _ = self
                    .cacher_tx
                    .send(CacherCommand::WritePlaybackState(state));
            }
            AudioEvent::PlaybackStatus(status) => {
                self.state.update(&mut *cx, |this, cx| {
                    this.playback.status = *status;
                    cx.notify();
                });
                cx.notify(view.entity_id());

                if *status == PlaybackStatus::Stopped {
                    self.finalize_metrics_session(cx, false);
                }

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
                self.finalize_metrics_session(cx, true);

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
