pub mod playlists;
use crate::errors::ScannerError;
use blake3::Hasher;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize, Debug, Default)]
pub struct TrackId(pub [u8; 32]);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Track {
    pub id: TrackId,
    pub path: PathBuf,

    pub title: String,
    pub artist: String,
    pub album: String,

    pub duration: u64,
    pub size: u64,
    pub modified: u64,
}

pub fn gen_track_id(path: &PathBuf) -> Result<TrackId, ScannerError> {
    let mut hasher = Hasher::new();

    hasher.update(path.to_string_lossy().as_bytes());

    Ok(TrackId(*hasher.finalize().as_bytes()))
}
