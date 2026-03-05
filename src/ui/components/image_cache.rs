use crate::library::TrackId;
use gpui::RenderImage;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Default)]
pub struct ImageCache {
    pub current: Option<Arc<RenderImage>>,
    pub thumbs: HashMap<TrackId, Arc<RenderImage>>,
}

impl ImageCache {
    #[must_use] 
    pub fn get(&self, id: &TrackId) -> Option<Arc<RenderImage>> {
        self.thumbs.get(id).cloned()
    }

    pub fn clear(&mut self) {
        self.thumbs.clear();
    }

    pub fn add(&mut self, id: TrackId, thumbnail: Arc<RenderImage>) {
        self.thumbs.insert(id, thumbnail);
    }
}

impl gpui::Global for ImageCache {}
