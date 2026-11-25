use crate::cacher::ImageKind;
use crate::controller::commands::CacherCommand;
use crate::controller::state::ImageId;
use crossbeam_channel::Sender;
use gpui::RenderImage;
use lru::LruCache;
use std::collections::HashSet;
use std::num::NonZeroUsize;
use std::sync::Arc;

const IMAGE_CACHE_CAPACITY: usize = 256;

pub struct ImageCache {
    pub current: Option<Arc<RenderImage>>,
    pub images: LruCache<(ImageId, ImageKind), Arc<RenderImage>>,

    pub inflight: HashSet<(ImageId, ImageKind)>,
}

impl Default for ImageCache {
    fn default() -> Self {
        ImageCache {
            current: None,
            images: LruCache::new(NonZeroUsize::new(IMAGE_CACHE_CAPACITY).unwrap()),
            inflight: HashSet::new(),
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
