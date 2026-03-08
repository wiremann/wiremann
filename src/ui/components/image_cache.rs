use crate::library::playlists::PlaylistId;
use crate::library::TrackId;
use gpui::RenderImage;
use lru::LruCache;
use std::collections::HashSet;
use std::num::NonZero;
use std::sync::Arc;

pub struct ImageCache {
    pub current: Option<Arc<RenderImage>>,
    pub track_thumbs: LruCache<TrackId, Arc<RenderImage>>,
    pub playlist_thumbs: LruCache<PlaylistId, Arc<RenderImage>>,

    pub inflight: HashSet<TrackId>,
}

impl Default for ImageCache {
    fn default() -> Self {
        ImageCache {
            current: None,
            track_thumbs: LruCache::new(NonZero(128)),
            playlist_thumbs: LruCache::new(NonZero(64)),
            inflight: HashSet::new(),
        }
    }
}

impl ImageCache {
    #[must_use]
    pub fn get_track(&mut self, id: &TrackId) -> Option<Arc<RenderImage>> {
        self.track_thumbs.get(id).cloned()
    }

    pub fn clear_tracks(&mut self) {
        self.track_thumbs.clear();
    }

    pub fn add_track(&mut self, id: TrackId, thumbnail: Arc<RenderImage>) {
        self.track_thumbs.put(id, thumbnail);
    }
}

impl gpui::Global for ImageCache {}
