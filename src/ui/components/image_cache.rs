use crate::controller::commands::CacherCommand;
use crate::library::ImageId;
use crossbeam_channel::Sender;
use gpui::RenderImage;
use lru::LruCache;
use std::collections::HashSet;
use std::num::NonZeroUsize;
use std::sync::Arc;
use crate::cacher::ImageKind;

pub struct ImageCache {
    pub current: Option<Arc<RenderImage>>,
    pub images: LruCache<ImageId, Arc<RenderImage>>,

    pub inflight: HashSet<ImageId>,
}

impl Default for ImageCache {
    fn default() -> Self {
        ImageCache {
            current: None,
            images: LruCache::new(NonZeroUsize::new(128).unwrap()),
            inflight: HashSet::new(),
        }
    }
}

impl ImageCache {
    #[must_use]
    pub fn get(&mut self, id: &ImageId) -> Option<Arc<RenderImage>> {
        self.images.get(id).cloned()
    }

    pub fn clear(&mut self) {
        self.images.clear();
    }

    pub fn add(
        &mut self,
        id: ImageId,
        image: Arc<RenderImage>,
    ) -> Option<Arc<RenderImage>> {
        let evicted = self.images.put(id, image);
        self.inflight.remove(&id);

        evicted
    }

    pub fn request<I>(
        &mut self,
        ids: I,
        tx: &Sender<CacherCommand>,
        kind: ImageKind
    )
    where
        I: IntoIterator<Item=ImageId>,
    {
        let mut to_request = HashSet::new();

        for id in ids {
            if self.images.contains(&id) || self.inflight.contains(&id) {
                continue;
            }

            self.inflight.insert(id);
            to_request.insert(id);
        }

        if !to_request.is_empty() {
            let _ = tx.send(CacherCommand::GetImage(to_request, kind));
        }
    }
}

impl gpui::Global for ImageCache {}
