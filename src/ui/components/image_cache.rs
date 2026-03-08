use crate::library::playlists::PlaylistId;
use crate::library::TrackId;
use gpui::RenderImage;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Default)]
pub struct ImageCache {
    pub current: Option<Arc<RenderImage>>,
    pub track_thumbs: HashMap<TrackId, Arc<RenderImage>>,
    pub playlist_thumbs: HashMap<PlaylistId, Arc<RenderImage>>,
}

impl ImageCache {
    #[must_use]
    pub fn get_track(&self, id: &TrackId) -> Option<Arc<RenderImage>> {
        self.track_thumbs.get(id).cloned()
    }

    pub fn clear_tracks(&mut self) {
        self.track_thumbs.clear();
    }

    pub fn add_track(&mut self, id: TrackId, thumbnail: Arc<RenderImage>) {
        self.track_thumbs.insert(id, thumbnail);
    }
}

impl gpui::Global for ImageCache {}
