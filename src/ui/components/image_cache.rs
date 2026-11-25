use crate::cacher::ImageKind;
use crate::controller::commands::CacherCommand;
use crate::controller::state::{ImageId, PlaylistId, TrackId};
use crossbeam_channel::Sender;
use gpui::{
    Animation, AnimationExt, Div, ElementId, IntoElement, ParentElement, RenderImage, Styled, div,
    linear, percentage, prelude::FluentBuilder, rgb,
};
use lru::LruCache;
use std::collections::HashSet;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::Duration;
use wgpui_motion::prelude::{MotionExt, Params, Timeline, VisualParams};

use super::icons::{Icons, icon};

const IMAGE_CACHE_CAPACITY: usize = 256;

pub struct ImageCache {
    pub current: Option<Arc<RenderImage>>,
    pub images: LruCache<(ImageId, ImageKind), Arc<RenderImage>>,

    pub inflight: HashSet<(ImageId, ImageKind)>,
    pub pending_tracks: HashSet<(TrackId, ImageKind)>,
    pub pending_playlists: HashSet<PlaylistId>,
}

impl Default for ImageCache {
    fn default() -> Self {
        ImageCache {
            current: None,
            images: LruCache::new(NonZeroUsize::new(IMAGE_CACHE_CAPACITY).unwrap()),
            inflight: HashSet::new(),
            pending_tracks: HashSet::new(),
            pending_playlists: HashSet::new(),
        }
    }
}

impl ImageCache {
    #[must_use]
    pub fn get(&mut self, id: &ImageId) -> Option<Arc<RenderImage>> {
        self.get_kind(id, ImageKind::ThumbnailSmall)
    }

    #[must_use]
    pub fn get_large(&mut self, id: &ImageId) -> Option<Arc<RenderImage>> {
        self.get_kind(id, ImageKind::ThumbnailLarge)
    }

    #[must_use]
    pub fn get_playlist(&mut self, id: &ImageId) -> Option<Arc<RenderImage>> {
        self.get_kind(id, ImageKind::Playlist)
    }

    fn get_kind(&mut self, id: &ImageId, kind: ImageKind) -> Option<Arc<RenderImage>> {
        self.images.get(&(*id, kind)).cloned()
    }

    #[must_use]
    pub fn is_loading(&self, id: ImageId, kind: ImageKind) -> bool {
        self.inflight.contains(&(id, kind))
    }

    #[must_use]
    pub fn is_track_loading(&self, id: TrackId, kind: ImageKind) -> bool {
        self.pending_tracks.contains(&(id, kind))
    }

    #[must_use]
    pub fn is_playlist_loading(&self, id: PlaylistId) -> bool {
        self.pending_playlists.contains(&id)
    }

    pub fn clear(&mut self) {
        self.images.clear();
    }

    pub fn add(&mut self, id: ImageId, image: Arc<RenderImage>) -> Option<Arc<RenderImage>> {
        self.add_kind(id, ImageKind::ThumbnailSmall, image)
    }

    pub fn request<I>(&mut self, ids: I, tx: &Sender<CacherCommand>, kind: ImageKind)
    where
        I: IntoIterator<Item = ImageId>,
    {
        let mut to_request = HashSet::new();

        for id in ids {
            let key = (id, kind);
            if self.images.contains(&key) {
                continue;
            }

            if !self.inflight.insert(key) {
                continue;
            }

            to_request.insert(id);
        }

        if !to_request.is_empty() {
            let _ = tx.send(CacherCommand::GetImage(to_request, kind));
        }
    }

    pub fn add_kind(
        &mut self,
        id: ImageId,
        kind: ImageKind,
        image: Arc<RenderImage>,
    ) -> Option<Arc<RenderImage>> {
        let key = (id, kind);
        let evicted = self.images.put(key, image);
        self.inflight.remove(&key);
        evicted
    }
}

impl gpui::Global for ImageCache {}

pub fn image_with_loader(
    image: impl IntoElement,
    loading: bool,
    loader_id: impl Into<ElementId>,
) -> Div {
    let loader_id = loader_id.into();
    div()
        .size_full()
        .relative()
        .child(image)
        .when(loading, |this| {
            this.child(
                div()
                    .absolute()
                    .top_0()
                    .right_0()
                    .bottom_0()
                    .left_0()
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        div()
                            .size_6()
                            .child(
                                icon(Icons::LoaderCircle)
                                    .size_6()
                                    .text_color(rgb(0xFFFFFF))
                                    .with_animation(
                                        format!("image_loader_spin_{loader_id:?}"),
                                        Animation::new(Duration::from_millis(900))
                                            .repeat()
                                            .with_easing(linear),
                                        |loader, delta| loader.rotate(percentage(delta)),
                                    ),
                            )
                            .motion(format!("image_loader_{loader_id:?}"))
                            .timeline(
                                Timeline::new(Duration::from_millis(900))
                                    .looping()
                                    .key(0.0, Params::new().opacity(0.9))
                                    .key(1.0, Params::new().opacity(1.0)),
                            ),
                    ),
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn image_kinds_have_independent_requests() {
        let mut cache = ImageCache::default();
        let image_id = ImageId([1; 16]);
        let (tx, rx) = crossbeam_channel::unbounded();

        cache.request([image_id], &tx, ImageKind::ThumbnailSmall);
        cache.request([image_id], &tx, ImageKind::ThumbnailLarge);
        cache.request([image_id], &tx, ImageKind::ThumbnailSmall);

        assert_eq!(rx.try_iter().count(), 2);
    }
}
