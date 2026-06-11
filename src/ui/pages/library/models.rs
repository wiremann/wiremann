use crate::controller::state::{ImageId, PlaylistId, TrackId};

#[derive(Clone)]
pub struct LibraryTrackRow {
    pub id: TrackId,
    pub title: String,
    pub artists: String,
    pub album: String,
    pub duration_ms: i64,
    pub image_id: Option<ImageId>,
}

#[derive(Clone)]
pub struct LibraryPlaylistRow {
    pub id: PlaylistId,
    pub name: String,
    pub track_count: usize,
    pub image_id: Option<ImageId>,
}
