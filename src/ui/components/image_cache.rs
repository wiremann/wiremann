use crate::controller::commands::CacherCommand;
use crate::library::ImageId;
use crossbeam_channel::Sender;
use gpui::RenderImage;
use lru::LruCache;
use std::collections::HashSet;
use std::num::NonZeroUsize;
use std::sync::Arc;

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
    pub fn get_track(&mut self, id: &ImageId) -> Option<Arc<RenderImage>> {
        self.images.get(id).cloned()
    }

    pub fn clear_tracks(&mut self) {
        self.images.clear();
    }

    pub fn add_track(
        &mut self,
        id: ImageId,
        thumbnail: Arc<RenderImage>,
    ) -> Option<Arc<RenderImage>> {
        let evicted = self.images.put(id, thumbnail);
        self.inflight.remove(&id);

        evicted
    }

    pub fn request_track<I>(
        &mut self,
        ids: I,
        tx: &Sender<CacherCommand>,
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
            let _ = tx.send(CacherCommand::GetThumbnails(to_request));
        }
    }
}

impl gpui::Global for ImageCache {}
