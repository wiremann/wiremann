use super::{
    App, Arc, CacherCommand, Controller, ControllerError, Entity, ImageCache, ImageKind,
    ImageProcessorCommand, ImageProcessorEvent, SystemIntegrationCommand, Wiremann,
    drop_images_from_app,
};
use crate::controller::state::ImageId;
use gpui::RenderImage;

fn persist_render_image(
    cx: &App,
    cacher_tx: crossbeam_channel::Sender<CacherCommand>,
    id: ImageId,
    kind: ImageKind,
    image: Arc<RenderImage>,
) {
    cx.background_executor()
        .spawn(async move {
            let width = image.size(0).width.0.cast_unsigned();
            let height = image.size(0).height.0.cast_unsigned();
            if let Some(image) = image.as_bytes(0) {
                let _ = cacher_tx.send(CacherCommand::WriteImage {
                    id,
                    kind,
                    width,
                    height,
                    image: image.to_vec(),
                });
            }
        })
        .detach();
}

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

                persist_render_image(
                    cx,
                    self.cacher_tx.clone(),
                    *image_id,
                    ImageKind::AlbumArt,
                    image.clone(),
                );

                let image = image.clone();
                let width = image.size(0).width.0.cast_unsigned();
                let height = image.size(0).height.0.cast_unsigned();
                if let Some(image) = image.as_bytes(0) {
                    let image = image.to_vec();
                    let state = self.state.read(cx);
                    if let Some(track_id) = &state.playback.current
                        && let Some(track) = state.library.tracks.get(track_id)
                    {
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
                                image: Some((width, height, image)),
                                duration: track.duration.as_secs(),
                            })
                            .ok();
                    }
                }

                cx.notify(view.entity_id());
            }
            ImageProcessorEvent::InsertThumbnails(thumbnails, kind) => {
                let mut evicted_images = Vec::new();
                for (id, image) in thumbnails {
                    persist_render_image(cx, self.cacher_tx.clone(), *id, *kind, image.clone());
                    let evicted = {
                        let thumbnail_cache = cx.global_mut::<ImageCache>();
                        thumbnail_cache.add_kind(*id, *kind, image.clone())
                    };

                    if let Some(image) = evicted {
                        evicted_images.push(image);
                    }
                }
                drop_images_from_app(cx, evicted_images);
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

                thumbnail_cache.add_kind(*image_id, ImageKind::Playlist, image.clone());

                let _ = self
                    .image_processor_tx
                    .send(ImageProcessorCommand::PlaylistJobFinished(*id));

                persist_render_image(
                    cx,
                    self.cacher_tx.clone(),
                    *image_id,
                    ImageKind::Playlist,
                    image.clone(),
                );

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
