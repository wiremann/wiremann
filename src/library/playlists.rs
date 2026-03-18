use super::{ImageId, TrackId};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use uuid::Uuid;

#[derive(Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize, Debug)]
pub struct PlaylistId(pub Uuid);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum PlaylistSource {
    User,
    Folder,
    Generated,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Playlist {
    pub id: PlaylistId,
    pub name: String,
    pub source: PlaylistSource,

    pub duration: Duration,

    pub tracks: Vec<TrackId>,
    pub image_id: Option<ImageId>,
}
