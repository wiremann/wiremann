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

use crate::controller::state::{
    Album, AlbumId, Artist, ArtistId, LibraryState, ListenMetrics, TrackListenMetrics,
};

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
    pub artists: Vec<u64>,
    pub album: u64,

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
    pub artists: HashMap<u64, CachedArtist>,
    pub albums: HashMap<u64, CachedAlbum>,
}

#[derive(Debug, Clone, PartialEq, Default, Encode, Decode)]
pub struct CachedAlbum {
    pub id: u64,
    pub name: String,
    pub artists: Vec<u64>,
    pub duration: u64,
    pub tracks: Vec<[u8; 16]>,
    pub image_id: Option<[u8; 16]>,
}

#[derive(Debug, Clone, PartialEq, Default, Encode, Decode)]
pub struct CachedArtist {
    pub id: u64,
    pub name: String,
    pub tracks: Vec<[u8; 16]>,
    pub albums: Vec<u64>,
    pub image_id: Option<[u8; 16]>,
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

#[derive(Debug, Clone, PartialEq, Default, Encode, Decode)]
pub struct CachedFavorites {
    pub tracks: Vec<[u8; 16]>,
}

#[derive(Debug, Clone, PartialEq, Default, Encode, Decode)]
pub struct CachedTrackMetrics {
    pub play_count: u32,
    pub play_time: u64,
    pub first_played: Option<u64>,
    pub last_played: Option<u64>,
    pub skip_count: u32,
}

#[derive(Debug, Clone, PartialEq, Default, Encode, Decode)]
pub struct CachedListenMetrics {
    pub tracks: Vec<([u8; 16], CachedTrackMetrics)>,
}

// Conversion implementations

impl From<&Track> for CachedTrack {
    fn from(track: &Track) -> Self {
        Self {
            id: track.id.0,
            sources: track.sources.iter().map(Into::into).collect(),
            title: track.title.to_string(),
            artists: track.artists.iter().map(|a| a.0).collect(),
            album: track.album.0,
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
            title: c.title.into(),
            artists: c.artists.into_iter().map(ArtistId).collect(),
            album: AlbumId(c.album),
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
            name: playlist.name.to_string(),
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
            name: cached_playlist.name.into(),
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

        let artists = state
            .artists
            .iter()
            .map(|(id, artist)| {
                (
                    id.0,
                    CachedArtist {
                        id: id.0,
                        name: artist.name.to_string(),
                        tracks: artist.tracks.iter().map(|t| t.0).collect(),
                        albums: artist.albums.iter().map(|a| a.0).collect(),
                        image_id: artist.image_id.map(|i| i.0),
                    },
                )
            })
            .collect();

        let albums = state
            .albums
            .iter()
            .map(|(id, album)| {
                (
                    id.0,
                    CachedAlbum {
                        id: id.0,
                        name: album.name.to_string(),
                        artists: album.artists.iter().map(|a| a.0).collect(),
                        duration: album.duration.as_millis() as u64,
                        tracks: album.tracks.iter().map(|t| t.0).collect(),
                        image_id: album.image_id.map(|i| i.0),
                    },
                )
            })
            .collect();

        Self {
            tracks,
            playlists,
            artists,
            albums,
        }
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

        let artists = cache
            .artists
            .into_iter()
            .map(|(id, a)| {
                (
                    ArtistId(id),
                    Arc::new(Artist {
                        id: ArtistId(a.id),
                        name: a.name.into(),
                        tracks: a.tracks.into_iter().map(TrackId).collect(),
                        albums: a.albums.into_iter().map(AlbumId).collect(),
                        image_id: a.image_id.map(ImageId),
                    }),
                )
            })
            .collect();

        let albums = cache
            .albums
            .into_iter()
            .map(|(id, a)| {
                (
                    AlbumId(id),
                    Arc::new(Album {
                        id: AlbumId(a.id),
                        name: a.name.into(),
                        artists: a.artists.into_iter().map(ArtistId).collect(),
                        duration: Duration::from_millis(a.duration),
                        tracks: a.tracks.into_iter().map(TrackId).collect(),
                        image_id: a.image_id.map(ImageId),
                    }),
                )
            })
            .collect();

        Self {
            tracks,
            playlists,
            artists,
            albums,
        }
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

impl From<&[TrackId]> for CachedFavorites {
    fn from(ids: &[TrackId]) -> Self {
        Self {
            tracks: ids.iter().map(|id| id.0).collect(),
        }
    }
}

impl From<CachedFavorites> for Vec<TrackId> {
    fn from(c: CachedFavorites) -> Self {
        c.tracks.into_iter().map(TrackId).collect()
    }
}

impl From<&ListenMetrics> for CachedListenMetrics {
    fn from(m: &ListenMetrics) -> Self {
        Self {
            tracks: m
                .tracks
                .iter()
                .map(|(id, t)| {
                    (
                        id.0,
                        CachedTrackMetrics {
                            play_count: t.play_count,
                            play_time: t.play_time.as_secs(),
                            first_played: t.first_played,
                            last_played: t.last_played,
                            skip_count: t.skip_count,
                        },
                    )
                })
                .collect(),
        }
    }
}

impl From<CachedListenMetrics> for ListenMetrics {
    fn from(c: CachedListenMetrics) -> Self {
        Self {
            tracks: c
                .tracks
                .into_iter()
                .map(|(id, t)| {
                    (
                        TrackId(id),
                        TrackListenMetrics {
                            play_count: t.play_count,
                            play_time: Duration::from_secs(t.play_time),
                            first_played: t.first_played,
                            last_played: t.last_played,
                            skip_count: t.skip_count,
                        },
                    )
                })
                .collect(),
        }
    }
}
