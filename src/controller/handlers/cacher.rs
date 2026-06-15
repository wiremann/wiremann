use super::{Controller, App, CacherEvent, Entity, Wiremann, ControllerError, PlaybackStatus, duration_to_slider, ImageCache, drop_image_from_app, Rgb, Rgba, rgb, SystemIntegrationCommand, DominantColors, ImageProcessorCommand, HashSet, ImageKind, pick_playlist_thumbnail_tracks, LyricsState, LyricsStatus};

impl Controller {
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
                                        duration_to_slider(playback_state.position, duration),
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
                    fn rgb_to_rgba(color: Rgb<u8>) -> Rgba {
                        rgb((u32::from(color.r) << 16)
                            | (u32::from(color.g) << 8)
                            | u32::from(color.b))
                    }

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
                                image: Some((width, height, image.clone())),
                                duration: track.duration.as_secs(),
                            })
                            .ok();
                    }

                    let mut rgb_bytes = Vec::with_capacity((width * height * 3) as usize);
                    for chunk in image.as_slice().chunks_exact(4) {
                        rgb_bytes.push(chunk[0]);
                        rgb_bytes.push(chunk[1]);
                        rgb_bytes.push(chunk[2]);
                    }

                    let input =
                        okmain::InputImage::from_bytes(width as u16, height as u16, &rgb_bytes)
                            .unwrap();

                    let colors = okmain::colors(input);

                    let dominant = DominantColors {
                        color1: colors.first().copied().map_or(rgb(0x000000), rgb_to_rgba),

                        color2: colors.get(1).copied().map_or(rgb(0x000000), rgb_to_rgba),

                        color3: colors.get(2).copied().map_or(rgb(0x000000), rgb_to_rgba),

                        color4: colors.get(3).copied().map_or(rgb(0x000000), rgb_to_rgba),
                    };
                    *cx.global_mut::<DominantColors>() = dominant;
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
            CacherEvent::Lyrics(id, lyrics) => {
                let current = cx.global::<Controller>().state.read(cx).playback.current;

                if let Some(current) = current
                    && current == *id
                {
                    let lyrics_state = cx.global::<LyricsState>().0.clone();

                    lyrics_state.update(cx, |this, cx| {
                        this.lyrics.clone_from(lyrics);
                        this.track_id = Some(current);
                        this.status = if lyrics.is_some() {
                            LyricsStatus::Available
                        } else {
                            LyricsStatus::Unavailable
                        };
                        cx.notify();
                    });
                }
            }
            CacherEvent::MissingLyrics(id) => {
                if let Some(track) = self.state.read(cx).library.tracks.get(id) {
                    self.get_lyrics(
                        *id,
                        &track.title,
                        &track.artist,
                        &track.album,
                        track.duration,
                    );
                }
            }
        }
        Ok(())
    }
}
