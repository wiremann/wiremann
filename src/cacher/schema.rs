use crate::controller::state::{PlaybackState, PlaybackStatus};
use crate::controller::state::{PlaylistId, TrackId, TrackSource};
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;
use uuid::Uuid;

#[derive(Copy, Clone, PartialEq, Debug, Eq, Hash)]
pub enum ImageKind {
    ThumbnailSmall,
    ThumbnailLarge,
    AlbumArt,
    Playlist,
}

#[derive(Encode, Decode)]
pub struct CachedImage {
    pub width: u32,
    pub height: u32,
    pub image: Vec<u8>,
}

// The growing state (library, queue, favorites, listen metrics) lives in the
// SQLite database; only the small playback session is persisted as RON here.

/// A tracks-source key used to persist the scanner's diff record between runs.
#[derive(Debug, Clone, PartialEq, Default, Hash, Eq, Encode, Decode)]
pub struct CachedTrackSource {
    pub path: String,
    pub size: u64,
    pub modified: u64,
}

impl From<&TrackSource> for CachedTrackSource {
    fn from(c: &TrackSource) -> Self {
        CachedTrackSource {
            path: c.path.to_string_lossy().to_string(),
            size: c.size,
            modified: c.modified,
        }
    }
}

impl From<&CachedTrackSource> for TrackSource {
    fn from(c: &CachedTrackSource) -> Self {
        TrackSource {
            path: PathBuf::from(c.path.clone()),
            size: c.size,
            modified: c.modified,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct CachedPlaybackState {
    pub current: Option<[u8; 16]>,
    pub current_playlist: Option<String>,
    pub current_index: usize,

    pub status: PlaybackStatus,
    pub position: u64,

    pub volume: f32,
    pub mute: bool,
    pub shuffling: bool,
    pub repeat: bool,
}

impl From<&PlaybackState> for CachedPlaybackState {
    fn from(p: &PlaybackState) -> Self {
        Self {
            current: p.current.map(|id| id.0),
            current_playlist: p.current_playlist.map(|id| id.0.to_string()),
            current_index: p.current_index,
            status: p.status,
            position: p.position.as_millis() as u64,
            volume: p.volume,
            mute: p.mute,
            shuffling: p.shuffling,
            repeat: p.repeat,
        }
    }
}

impl From<CachedPlaybackState> for PlaybackState {
    fn from(c: CachedPlaybackState) -> Self {
        Self {
            current: c.current.map(TrackId),
            current_playlist: c
                .current_playlist
                .map(|s| PlaylistId(Uuid::from_str(&s).unwrap_or_default())),
            current_index: c.current_index,
            status: c.status,
            position: Duration::from_millis(c.position),
            volume: c.volume,
            mute: c.mute,
            shuffling: c.shuffling,
            repeat: c.repeat,
        }
    }
}
