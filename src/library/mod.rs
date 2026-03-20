pub mod playlists;
use crate::errors::ScannerError;
use blake3::Hasher;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const AUDIO_HASH_SEED: u64 = 0x3141_5926_5358_9793;
const IMAGE_HASH_SEED: u64 = 0x2718_2818_2845_9045;

#[derive(Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize, Debug, Default)]
pub struct TrackId(pub [u8; 32]);

#[derive(Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize, Debug, Default)]
pub struct ImageId(pub [u8; 32]);

#[derive(Clone, Debug, PartialEq)]
pub struct Track {
    pub id: TrackId,
    pub sources: Vec<TrackSource>,

    pub title: String,
    pub artist: String,
    pub album: String,

    pub duration: u64,

    pub image_id: Option<ImageId>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TrackSource {
    pub path: PathBuf,
    pub size: u64,
    pub modified: u64,
}

#[allow(clippy::missing_errors_doc)]
pub fn gen_track_id(path: &Path) -> Result<TrackId, ScannerError> {
    let mut hasher = Hasher::new();

    hasher.update(path.to_string_lossy().as_bytes());

    Ok(TrackId(*hasher.finalize().as_bytes()))
}
