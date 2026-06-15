use super::{Controller, App, ImageProcessorEvent, Entity, Wiremann, ControllerError, ImageCache, CacherCommand, ImageKind, SystemIntegrationCommand, drop_image_from_app, Arc, ImageProcessorCommand};

impl Controller {
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
                                duration: track.duration.as_secs(),
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
}
