use serde::{Deserialize, Serialize};
use std::hash::Hasher;
use std::io;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;
use std::{collections::HashMap, sync::Arc, time::Duration};
use twox_hash::{XxHash3_64, XxHash3_128};
use uuid::Uuid;

use crate::scanner::metadata::ScannedTrack;

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
    pub artists: HashMap<ArtistId, Arc<Artist>>,
    pub albums: HashMap<AlbumId, Arc<Album>>,
}

#[derive(Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize, Debug, Default)]
pub struct TrackId(pub [u8; 16]);

#[derive(Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize, Debug, Default)]
pub struct ImageId(pub [u8; 16]);

#[derive(Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize, Debug)]
pub struct PlaylistId(pub Uuid);

#[derive(Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize, Debug, Default)]
pub struct AlbumId(pub u64);

#[derive(Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize, Debug, Default)]
pub struct ArtistId(pub u64);

#[derive(Clone, Debug, PartialEq)]
pub struct Track {
    pub id: TrackId,
    pub sources: Vec<TrackSource>,

    pub title: Arc<str>,
    pub artists: Vec<ArtistId>,
    pub album: AlbumId,

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
    pub name: Arc<str>,
    pub source: PlaylistSource,

    pub folder_path: Option<PathBuf>,

    pub duration: Duration,

    pub tracks: Vec<TrackId>,
    pub image_id: Option<ImageId>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Album {
    pub id: AlbumId,
    pub name: Arc<str>,
    pub artists: Vec<ArtistId>,

    pub duration: Duration,

    pub tracks: Vec<TrackId>,
    pub image_id: Option<ImageId>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Artist {
    pub id: ArtistId,
    pub name: Arc<str>,

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
    pub fn generate(album: &str, artists: &[&str]) -> Result<Self, io::Error> {
        let mut hasher = XxHash3_64::with_seed(ALBUM_HASH_SEED);

        let album = album.trim().to_lowercase();
        let artist = artists.join("#").trim().to_lowercase();

        hasher.write(album.as_bytes());
        hasher.write(b"#");
        hasher.write(artist.as_bytes());

        Ok(AlbumId(hasher.finish()))
    }
}

impl ArtistId {
    pub fn generate(artist: &str) -> Result<Self, io::Error> {
        let mut hasher = XxHash3_64::with_seed(ARTIST_HASH_SEED);

        let artist = artist.trim().to_lowercase();

        hasher.write(artist.as_bytes());

        Ok(ArtistId(hasher.finish()))
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
            .duration_since(UNIX_EPOCH)
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

impl LibraryState {
    pub fn track(&self, id: TrackId) -> Option<&Arc<Track>> {
        self.tracks.get(&id)
    }

    pub fn artist(&self, id: ArtistId) -> Option<&Arc<Artist>> {
        self.artists.get(&id)
    }

    pub fn album(&self, id: AlbumId) -> Option<&Arc<Album>> {
        self.albums.get(&id)
    }
}

impl Track {
    pub fn album<'a>(&self, lib: &'a LibraryState) -> Option<&'a Arc<Album>> {
        lib.album(self.album)
    }

    pub fn artists<'a>(&self, lib: &'a LibraryState) -> impl Iterator<Item = &'a Arc<Artist>> {
        self.artists.iter().filter_map(|id| lib.artist(*id))
    }
}

impl LibraryState {
    pub fn upsert_scanned_track(&mut self, scanned: &ScannedTrack) -> Result<TrackId, io::Error> {
        let artist_ids: Vec<ArtistId> = scanned
            .artists
            .iter()
            .map(|artist| ArtistId::generate(artist))
            .collect::<Result<_, _>>()?;

        for (name, id) in scanned.artists.iter().zip(&artist_ids) {
            self.artists.entry(*id).or_insert_with(|| {
                Arc::new(Artist {
                    id: *id,
                    name: name.clone().into(),
                    tracks: Vec::new(),
                    albums: Vec::new(),
                    image_id: None,
                })
            });
        }

        let artist_names: Vec<&str> = scanned.artists.iter().map(String::as_str).collect();

        let album_id = AlbumId::generate(&scanned.album, &artist_names)?;

        self.albums.entry(album_id).or_insert_with(|| {
            Arc::new(Album {
                id: album_id,
                name: scanned.album.clone().into(),
                artists: artist_ids.clone(),
                duration: Duration::ZERO,
                tracks: Vec::new(),
                image_id: None,
            })
        });

        let track_id =
            TrackId::generate(&scanned.title, &scanned.artists.join(", "), &scanned.album)?;

        if let Some(existing) = self.tracks.get_mut(&track_id) {
            let existing = Arc::make_mut(existing);

            if !existing
                .sources
                .iter()
                .any(|s| s.path == scanned.source.path)
            {
                existing.sources.push(scanned.source.clone());
            }

            if existing.title.is_empty() {
                existing.title = scanned.title.clone().into();
            }

            if existing.artists.is_empty() {
                existing.artists = artist_ids.clone();
            }

            if existing.album == AlbumId::default() {
                existing.album = album_id;
            }
        } else {
            self.tracks.insert(
                track_id,
                Arc::new(Track {
                    id: track_id,
                    sources: vec![scanned.source.clone()],
                    title: scanned.title.clone().into(),
                    artists: artist_ids.clone(),
                    album: album_id,
                    duration: scanned.duration,
                    image_id: None,
                }),
            );
        }

        for artist_id in &artist_ids {
            if let Some(artist) = self.artists.get_mut(artist_id) {
                let artist = Arc::make_mut(artist);

                if !artist.tracks.contains(&track_id) {
                    artist.tracks.push(track_id);
                }

                if !artist.albums.contains(&album_id) {
                    artist.albums.push(album_id);
                }
            }
        }

        if let Some(album) = self.albums.get_mut(&album_id) {
            let album = Arc::make_mut(album);

            if !album.tracks.contains(&track_id) {
                album.tracks.push(track_id);
            }
        }

        Ok(track_id)
    }
}
