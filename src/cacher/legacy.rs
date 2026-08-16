use crate::controller::state::{
    Album, AlbumId, Artist, ArtistId, ImageId, LibraryState, ListenMetrics, Playlist,
    PlaylistId, PlaylistSource, QueueState, Track, TrackId, TrackListenMetrics, TrackSource,
};
use crate::errors::CacherError;
use bitcode::{Decode, Encode};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

/// A one-time import of the legacy bitcode caches (`library.bin`,
/// `queue.bin`, `favorites.bin`, `metrics.bin`) that were used before the
/// SQLite migration. The on-disk formats are preserved verbatim here so we can
/// decode them into the current in-memory state types and store them in the DB.
///
/// The playback session (`session.ron`) predates and postdates this migration
/// unchanged, so it is not touched here.

#[derive(Encode, Decode)]
struct CacheFile<T> {
    version: u32,
    payload: T,
}

fn read_cache<T>(path: &Path) -> Result<Option<T>, CacherError>
where
    T: for<'a> Decode<'a>,
{
    if !path.exists() {
        return Ok(None);
    }

    let bytes = std::fs::read(path)?;
    let file: CacheFile<T> = bitcode::decode(&bytes)?;

    if file.version != 1 {
        return Ok(None);
    }

    Ok(Some(file.payload))
}

#[derive(Debug, Clone, PartialEq, Default, Encode, Decode)]
struct CachedTrack {
    pub id: [u8; 16],
    pub sources: Vec<CachedTrackSource>,
    pub title: String,
    pub artists: Vec<u64>,
    pub album: u64,
    pub duration: u64,
    pub image_id: Option<[u8; 16]>,
}

#[derive(Debug, Clone, PartialEq, Default, Hash, Eq, Encode, Decode)]
struct CachedTrackSource {
    pub path: String,
    pub size: u64,
    pub modified: u64,
}

impl From<&CachedTrackSource> for TrackSource {
    fn from(c: &CachedTrackSource) -> Self {
        TrackSource {
            path: c.path.clone().into(),
            size: c.size,
            modified: c.modified,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default, Encode, Decode)]
enum CachedPlaylistSource {
    User,
    #[default]
    Folder,
    Generated,
}

#[derive(Debug, Clone, PartialEq, Default, Encode, Decode)]
struct CachedPlaylist {
    pub id: String,
    pub name: String,
    pub source: CachedPlaylistSource,
    pub tracks: Vec<[u8; 16]>,
    pub folder_path: Option<String>,
    pub duration: u64,
    pub image_id: Option<[u8; 16]>,
}

#[derive(Debug, Clone, PartialEq, Default, Encode, Decode)]
struct CachedLibraryState {
    pub tracks: HashMap<[u8; 16], CachedTrack>,
    pub playlists: HashMap<String, CachedPlaylist>,
    pub artists: HashMap<u64, CachedArtist>,
    pub albums: HashMap<u64, CachedAlbum>,
}

#[derive(Debug, Clone, PartialEq, Default, Encode, Decode)]
struct CachedAlbum {
    pub id: u64,
    pub name: String,
    pub artists: Vec<u64>,
    pub duration: u64,
    pub tracks: Vec<[u8; 16]>,
    pub image_id: Option<[u8; 16]>,
}

#[derive(Debug, Clone, PartialEq, Default, Encode, Decode)]
struct CachedArtist {
    pub id: u64,
    pub name: String,
    pub tracks: Vec<[u8; 16]>,
    pub albums: Vec<u64>,
    pub image_id: Option<[u8; 16]>,
}

#[derive(Debug, Clone, PartialEq, Default, Encode, Decode)]
struct CachedQueueState {
    pub tracks: Vec<[u8; 16]>,
    pub order: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq, Default, Encode, Decode)]
struct CachedFavorites {
    pub tracks: Vec<[u8; 16]>,
}

#[derive(Debug, Clone, PartialEq, Default, Encode, Decode)]
struct CachedTrackMetrics {
    pub play_count: u32,
    pub play_time: u64,
    pub first_played: Option<u64>,
    pub last_played: Option<u64>,
    pub skip_count: u32,
}

#[derive(Debug, Clone, PartialEq, Default, Encode, Decode)]
struct CachedListenMetrics {
    pub tracks: Vec<([u8; 16], CachedTrackMetrics)>,
}

/// Loads the legacy library state from `library.bin` (if present), converting
/// it into the current in-memory [`LibraryState`].
#[must_use]
pub fn read_legacy_library(cache_dir: &Path) -> Option<LibraryState> {
    let path = cache_dir.join("library.bin");
    let cached: CachedLibraryState = read_cache(&path).ok()??;

    let tracks = cached
        .tracks
        .into_iter()
        .map(|(id, t)| {
            let track = Track {
                id: TrackId(id),
                sources: t.sources.iter().map(Into::into).collect(),
                title: t.title.into(),
                artists: t.artists.into_iter().map(ArtistId).collect(),
                album: AlbumId(t.album),
                duration: Duration::from_millis(t.duration),
                image_id: t.image_id.map(ImageId),
            };
            (TrackId(id), Arc::new(track))
        })
        .collect();

    let playlists = cached
        .playlists
        .into_iter()
        .map(|(id, p)| {
            let playlist = Playlist {
                id: PlaylistId(Uuid::parse_str(&id).unwrap_or_default()),
                name: p.name.into(),
                source: match p.source {
                    CachedPlaylistSource::User => PlaylistSource::User,
                    CachedPlaylistSource::Folder => PlaylistSource::Folder,
                    CachedPlaylistSource::Generated => PlaylistSource::Generated,
                },
                folder_path: p.folder_path.map(Into::into),
                duration: Duration::from_secs(p.duration),
                tracks: p.tracks.iter().map(|t| TrackId(*t)).collect(),
                image_id: p.image_id.map(ImageId),
            };
            (PlaylistId(Uuid::parse_str(&id).unwrap_or_default()), playlist)
        })
        .collect();

    let artists = cached
        .artists
        .into_iter()
        .map(|(id, a)| {
            let artist = Artist {
                id: ArtistId(a.id),
                name: a.name.into(),
                tracks: a.tracks.iter().map(|t| TrackId(*t)).collect(),
                albums: a.albums.iter().map(|al| AlbumId(*al)).collect(),
                image_id: a.image_id.map(ImageId),
            };
            (ArtistId(id), Arc::new(artist))
        })
        .collect();

    let albums = cached
        .albums
        .into_iter()
        .map(|(id, a)| {
            let album = Album {
                id: AlbumId(a.id),
                name: a.name.into(),
                artists: a.artists.into_iter().map(ArtistId).collect(),
                duration: Duration::from_millis(a.duration),
                tracks: a.tracks.iter().map(|t| TrackId(*t)).collect(),
                image_id: a.image_id.map(ImageId),
            };
            (AlbumId(id), Arc::new(album))
        })
        .collect();

    Some(LibraryState {
        tracks,
        playlists,
        artists,
        albums,
    })
}

/// Loads the legacy queue state from `queue.bin` (if present).
#[must_use]
pub fn read_legacy_queue(cache_dir: &Path) -> Option<QueueState> {
    let path = cache_dir.join("queue.bin");
    let cached: CachedQueueState = read_cache(&path).ok()??;

    Some(QueueState {
        tracks: cached.tracks.into_iter().map(TrackId).collect(),
        order: cached.order,
    })
}

/// Loads the legacy favorites from `favorites.bin` (if present).
#[must_use]
pub fn read_legacy_favorites(cache_dir: &Path) -> Option<Vec<TrackId>> {
    let path = cache_dir.join("favorites.bin");
    let cached: CachedFavorites = read_cache(&path).ok()??;

    Some(cached.tracks.into_iter().map(TrackId).collect())
}

/// Loads the legacy listen metrics from `metrics.bin` (if present).
#[must_use]
pub fn read_legacy_metrics(cache_dir: &Path) -> Option<ListenMetrics> {
    let path = cache_dir.join("metrics.bin");
    let cached: CachedListenMetrics = read_cache(&path).ok()??;

    let tracks = cached
        .tracks
        .into_iter()
        .map(|(id, m)| {
            let metrics = TrackListenMetrics {
                play_count: m.play_count,
                play_time: Duration::from_secs(m.play_time),
                first_played: m.first_played,
                last_played: m.last_played,
                skip_count: m.skip_count,
            };
            (TrackId(id), metrics)
        })
        .collect();

    Some(ListenMetrics { tracks })
}
