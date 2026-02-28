use crate::library::TrackId;
use gpui::RenderImage;
use std::collections::HashMap;
use std::sync::Arc;

pub struct ImageCache {
    pub current: Option<Arc<RenderImage>>,
    pub thumbs: HashMap<TrackId, Arc<RenderImage>>,
}

impl Default for ImageCache {
    fn default() -> Self {
        Self {
            current: None,
            thumbs: HashMap::new(),
        }
    }
}

impl ImageCache {
    pub fn get(&self, id: &TrackId) -> Option<Arc<RenderImage>> {
        if let Some(thumbnail) = self.thumbs.get(id) {
            Some(thumbnail.clone())
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        self.thumbs.clear();
    }

    pub fn add(&mut self, id: TrackId, thumbnail: Arc<RenderImage>) {
        self.thumbs.insert(id, thumbnail);
    }
}

impl gpui::Global for ImageCache {}
