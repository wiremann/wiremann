use super::TrackId;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize, Debug)]
pub struct PlaylistId(pub Uuid);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum PlaylistSource {
    User,
    Folder(PathBuf),
    Generated,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Playlist {
    pub id: PlaylistId,
    pub name: String,
    pub source: PlaylistSource,
    pub tracks: Vec<TrackId>,
}
