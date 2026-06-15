use crate::controller::state::{ImageId, Track, TrackId, TrackSource};
use crate::controller::state::{PlaybackState, PlaybackStatus, QueueState};
use crate::controller::state::{Playlist, PlaylistId, PlaylistSource};
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

use crate::controller::state::LibraryState;

#[derive(Copy, Clone, PartialEq, Debug, Eq, Hash)]
pub enum ImageKind {
    ThumbnailSmall,
    ThumbnailLarge,
    AlbumArt,
    Playlist,
}

#[derive(Encode, Decode)]
pub struct CacheFile<T> {
    pub version: u32,
    pub payload: T,
}

#[derive(Debug, Clone, PartialEq, Default, Encode, Decode)]
pub struct CachedTrack {
    pub id: [u8; 16],
    pub sources: Vec<CachedTrackSource>,

    pub title: String,
    pub artist: String,
    pub album: String,

    pub duration: u64,

    pub image_id: Option<[u8; 16]>,
}

#[derive(Debug, Clone, PartialEq, Default, Hash, Eq, Encode, Decode)]
pub struct CachedTrackSource {
    pub path: String,
    pub size: u64,
    pub modified: u64,
}

#[derive(Debug, Clone, PartialEq, Default, Encode, Decode)]
pub enum CachedPlaylistSource {
    User,
    #[default]
    Folder,
    Generated,
}

#[derive(Encode, Decode)]
pub struct CachedImage {
    pub width: u32,
    pub height: u32,
    pub image: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Default, Encode, Decode)]
pub struct CachedPlaylist {
    pub id: String,
    pub name: String,
    pub source: CachedPlaylistSource,
    pub tracks: Vec<[u8; 16]>,

    pub folder_path: Option<String>,

    pub duration: u64,

    pub image_id: Option<[u8; 16]>,
}

#[derive(Debug, Clone, PartialEq, Default, Encode, Decode)]
pub struct CachedLibraryState {
    pub tracks: HashMap<[u8; 16], CachedTrack>,
    pub playlists: HashMap<String, CachedPlaylist>,
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

#[derive(Debug, Clone, PartialEq, Default, Encode, Decode)]
pub struct CachedQueueState {
    pub tracks: Vec<[u8; 16]>,
    pub order: Vec<usize>,
}

// Conversion implementations

impl From<&Track> for CachedTrack {
    fn from(track: &Track) -> Self {
        Self {
            id: track.id.0,
            sources: track.sources.iter().map(Into::into).collect(),
            title: track.title.clone(),
            artist: track.artist.clone(),
            album: track.album.clone(),
            duration: track.duration.as_millis() as u64,
            image_id: track.image_id.map(|id| id.0),
        }
    }
}

impl From<CachedTrack> for Track {
    fn from(c: CachedTrack) -> Self {
        Self {
            id: TrackId(c.id),
            sources: c.sources.iter().map(Into::into).collect(),
            title: c.title,
            artist: c.artist,
            album: c.album,
            duration: Duration::from_millis(c.duration),
            image_id: c.image_id.map(ImageId),
        }
    }
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

impl From<&Playlist> for CachedPlaylist {
    fn from(playlist: &Playlist) -> Self {
        CachedPlaylist {
            id: playlist.id.0.to_string(),
            name: playlist.name.clone(),
            source: match playlist.source {
                PlaylistSource::Folder => CachedPlaylistSource::Folder,
                PlaylistSource::Generated => CachedPlaylistSource::Generated,
                PlaylistSource::User => CachedPlaylistSource::User,
            },
            folder_path: playlist
                .folder_path
                .clone()
                .map(|path| path.to_string_lossy().to_string()),
            tracks: playlist.tracks.iter().map(|t| t.0).collect(),
            duration: playlist.duration.as_secs(),
            image_id: playlist.image_id.map(|id| id.0),
        }
    }
}

impl From<CachedPlaylist> for Playlist {
    fn from(cached_playlist: CachedPlaylist) -> Self {
        Playlist {
            id: PlaylistId(Uuid::from_str(cached_playlist.id.as_str()).unwrap_or_default()),
            name: cached_playlist.name,
            source: match cached_playlist.source {
                CachedPlaylistSource::Folder => PlaylistSource::Folder,
                CachedPlaylistSource::Generated => PlaylistSource::Generated,
                CachedPlaylistSource::User => PlaylistSource::User,
            },
            folder_path: cached_playlist.folder_path.map(PathBuf::from),
            tracks: cached_playlist.tracks.iter().map(|t| TrackId(*t)).collect(),
            duration: Duration::from_secs(cached_playlist.duration),
            image_id: cached_playlist.image_id.map(ImageId),
        }
    }
}

impl From<&LibraryState> for CachedLibraryState {
    fn from(state: &LibraryState) -> Self {
        let tracks = state
            .tracks
            .iter()
            .map(|(id, track)| (id.0, CachedTrack::from(track.as_ref())))
            .collect();

        let playlists = state
            .playlists
            .iter()
            .map(|(id, playlist)| (id.0.to_string(), CachedPlaylist::from(playlist)))
            .collect();

        Self { tracks, playlists }
    }
}

impl From<CachedLibraryState> for LibraryState {
    fn from(cache: CachedLibraryState) -> Self {
        let tracks = cache
            .tracks
            .into_iter()
            .map(|(id, track)| {
                let track: Track = track.into();
                (TrackId(id), Arc::new(track))
            })
            .collect();

        let playlists = cache
            .playlists
            .into_iter()
            .map(|(id, playlist)| {
                let playlist: Playlist = playlist.into();
                (
                    PlaylistId(Uuid::from_str(id.as_str()).unwrap_or_default()),
                    playlist,
                )
            })
            .collect();

        Self { tracks, playlists }
    }
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

impl From<&QueueState> for CachedQueueState {
    fn from(q: &QueueState) -> Self {
        Self {
            tracks: q.tracks.iter().map(|id| id.0).collect(),
            order: q.order.clone(),
        }
    }
}

impl From<CachedQueueState> for QueueState {
    fn from(c: CachedQueueState) -> Self {
        Self {
            tracks: c.tracks.into_iter().map(TrackId).collect(),
            order: c.order,
        }
    }
}
