pub mod playlists;

use crate::errors::ScannerError;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use twox_hash::XxHash3_128;

const AUDIO_HASH_SEED: u64 = 0x3141_5926_5358_9793;
const IMAGE_HASH_SEED: u64 = 0x2718_2818_2845_9045;
const CHUNK_SIZE: usize = 65536;

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
    pub fn generate(path: &Path) -> Result<Self, ScannerError> {
        let mut hasher = XxHash3_128::with_seed(AUDIO_HASH_SEED);

        let mut file = File::open(path)?;

        let length = file.metadata()?.len();

        if length > (CHUNK_SIZE * 3) as u64 {
            let offsets = [
                length / 4,
                length / 2,
                (length * 3) / 4,
            ];

            for &offset in &offsets {
                file.seek(SeekFrom::Start(offset))?;
                let mut buf: [u8; CHUNK_SIZE];
                file.read_exact(buf.as_mut())?;

                hasher.write(&buf);
            }

            Ok(TrackId(hasher.finish_128().to_le_bytes()))
        } else {
            // hasher.write();
        }
    }
}
