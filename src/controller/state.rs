use serde::{Deserialize, Serialize};
use std::io;
use std::path::{Path, PathBuf};
use std::{collections::HashMap, sync::Arc, time::Duration};
use twox_hash::XxHash3_128;
use uuid::Uuid;

const AUDIO_HASH_SEED: u64 = 0x3141_5926_5358_9793;
const IMAGE_HASH_SEED: u64 = 0x2718_2818_2845_9045;
const ALBUM_HASH_SEED: u64 = 0x1618_0339_8874_9894;
const ARTIST_HASH_SEED: u64 = 0x1414_2135_6237_3095;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct AppState {
    pub playback: PlaybackState,
    pub library: LibraryState,
    pub queue: QueueState,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct LibraryState {
    pub tracks: HashMap<TrackId, Arc<Track>>,
    pub playlists: HashMap<PlaylistId, Playlist>,
    pub db_tracks: Vec<crate::db::queries::library::LibraryRow>,
}

#[derive(Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize, Debug, Default)]
pub struct TrackId(pub [u8; 16]);

#[derive(Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize, Debug, Default)]
pub struct ImageId(pub [u8; 16]);

#[derive(Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize, Debug)]
pub struct PlaylistId(pub Uuid);

#[derive(Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize, Debug, Default)]
pub struct AlbumId(pub [u8; 16]);

#[derive(Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize, Debug, Default)]
pub struct ArtistId(pub [u8; 16]);

#[derive(Clone, Debug, PartialEq)]
pub struct Track {
    pub id: TrackId,
    pub sources: Vec<TrackSource>,

    pub title: String,
    pub artist: String,
    pub album: String,

    pub duration: Duration,

    pub image_id: Option<ImageId>,
}

#[derive(Clone, Debug, PartialEq, Hash, Eq)]
pub struct TrackSource {
    pub path: PathBuf,
    pub size: u64,
    pub modified: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub enum PlaybackStatus {
    #[default]
    Stopped,
    Playing,
    Paused,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlaybackState {
    pub current: Option<TrackId>,
    pub current_playlist: Option<PlaylistId>,
    pub current_index: usize,

    pub status: PlaybackStatus,
    pub position: Duration,

    pub volume: f32,
    pub mute: bool,
    pub shuffling: bool,
    pub repeat: bool,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct QueueState {
    pub tracks: Vec<TrackId>,
    pub order: Vec<usize>,
}

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

    pub folder_path: Option<PathBuf>,

    pub duration: Duration,

    pub tracks: Vec<TrackId>,
    pub image_id: Option<ImageId>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Album {
    pub id: AlbumId,
    pub name: String,
    pub artists: Vec<ArtistId>,

    pub duration: Duration,

    pub tracks: Vec<TrackId>,
    pub image_id: Option<ImageId>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Artist {
    pub id: ArtistId,
    pub name: String,

    pub tracks: Vec<TrackId>,
    pub albums: Vec<AlbumId>,

    pub image_id: Option<ImageId>,
}

impl TrackId {
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

impl AlbumId {
    pub fn generate(album: &str) -> Result<Self, io::Error> {
        let mut hasher = XxHash3_128::with_seed(ALBUM_HASH_SEED);

        let album = album.trim().to_lowercase();

        hasher.write(album.as_bytes());

        Ok(AlbumId(hasher.finish_128().to_le_bytes()))
    }
}

impl ArtistId {
    pub fn generate(artist: &str) -> Result<Self, io::Error> {
        let mut hasher = XxHash3_128::with_seed(ARTIST_HASH_SEED);

        let artist = artist.trim().to_lowercase();

        hasher.write(artist.as_bytes());

        Ok(ArtistId(hasher.finish_128().to_le_bytes()))
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
    #[must_use]
    pub fn get_valid_source(&self) -> Option<&TrackSource> {
        self.sources.iter().find(|&t| t.path.exists())
    }
}

impl TrackSource {
    #[allow(clippy::missing_errors_doc)]
    pub fn generate(path: &Path) -> Result<Self, io::Error> {
        let meta = std::fs::metadata(path)?;
        let modified = meta
            .modified()?
            .elapsed()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
            .as_secs();

        let size = meta.len();

        Ok(TrackSource {
            path: path.to_path_buf(),
            modified,
            size,
        })
    }
}

impl Default for PlaybackState {
    fn default() -> Self {
        PlaybackState {
            current: None,
            current_playlist: None,
            current_index: 0,
            status: PlaybackStatus::Stopped,
            position: Duration::from_secs(0),
            volume: 1.0,
            mute: false,
            shuffling: false,
            repeat: false,
        }
    }
}

impl QueueState {
    #[must_use]
    pub fn get_id(&self, index: usize) -> Option<TrackId> {
        self.order
            .get(index)
            .and_then(|&i| self.tracks.get(i))
            .copied()
    }

    #[must_use]
    pub fn get_index(&self, id: TrackId) -> Option<usize> {
        let track_idx = self.tracks.iter().position(|&t| t == id)?;
        self.order.iter().position(|&o| o == track_idx)
    }
}
