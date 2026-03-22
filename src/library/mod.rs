pub mod playlists;

use serde::{Deserialize, Serialize};
use std::io;
use std::path::PathBuf;
use twox_hash::XxHash3_128;

const AUDIO_HASH_SEED: u64 = 0x3141_5926_5358_9793;
const IMAGE_HASH_SEED: u64 = 0x2718_2818_2845_9045;

#[derive(Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize, Debug, Default)]
pub struct TrackId(pub [u8; 16]);

#[derive(Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize, Debug, Default)]
pub struct ImageId(pub [u8; 16]);

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


impl TrackId {
    #[allow(clippy::missing_errors_doc)]
    pub fn generate(name: &str, artist: &str, album: &str) -> Result<Self, io::Error> {
        let mut hasher = XxHash3_128::with_seed(AUDIO_HASH_SEED);

        let name = name.trim().to_lowercase();
        let artist = artist.trim().to_lowercase();
        let album = album.trim().to_lowercase();

        hasher.write(name.as_bytes());
        hasher.write(b"#");
        hasher.write(artist.as_bytes());
        hasher.write(b"#");
        hasher.write(album.as_bytes());

        Ok(TrackId(hasher.finish_128().to_le_bytes()))
    }
}

impl ImageId {
    pub fn generate(bytes: &[u8]) -> Result<Self, io::Error> {
        let mut hasher = XxHash3_128::with_seed(IMAGE_HASH_SEED);

        hasher.write(bytes);

        Ok(ImageId(hasher.finish_128().to_le_bytes()))
    }
}

impl Track {
    pub fn get_valid_source(&self) -> Option<&TrackSource> {
        self.sources.iter().find(|&t| t.path.exists())
    }
}
