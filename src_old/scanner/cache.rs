use crate::audio::engine::PlaybackState;
use crate::controller::metadata::Metadata;
use crate::controller::player::{PlayerState, Track};
use crate::scanner::{Playlist, ScannerState};
use bitcode::{Decode, Encode};
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Clone)]
pub struct CacheManager {
    pub cache_dir: PathBuf,
}

#[derive(Clone, Serialize, Deserialize, Default, PartialEq, Debug)]
pub struct CachedPlaylistIndexes {
    pub playlists: Vec<CachedPlaylistIndex>,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug, Default)]
pub struct CachedPlaylistIndex {
    pub id: String,
    pub name: String,
    pub path: String,
}

#[derive(Clone, Encode, Decode)]
pub struct PlaylistCache {
    pub id: String,
    pub name: String,
    pub path: String,
    pub tracks: Vec<TrackCache>,
}

#[derive(Clone, Encode, Decode)]
pub struct TrackCache {
    pub path: String,
    pub meta: MetadataCache,
}

#[derive(Clone, Encode, Decode)]
pub struct MetadataCache {
    pub title: String,
    pub artists: Vec<String>,
    pub album: String,
    pub genre: String,
    pub duration: u64,
    pub writer: String,
    pub producer: String,
    pub publisher: String,
    pub label: String,
}

#[derive(Clone, Encode, Decode)]
pub struct ThumbnailsCached {
    pub thumbnails: HashMap<String, Vec<u8>>,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug, Default)]
pub struct AppStateCache {
    // PlayerState
    pub current: Option<String>,
    pub state: PlaybackState,
    pub position: u64,
    pub volume: f32,
    pub mute: bool,
    pub shuffling: bool,
    pub repeat: bool,
    pub index: usize,

    // ScannerState
    pub queue_order: Vec<usize>,
    pub playlist: String,
}

impl From<Playlist> for PlaylistCache {
    fn from(value: Playlist) -> Self {
        PlaylistCache {
            id: value.id.to_string(),
            name: value.name,
            path: value
                .path
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default(),
            tracks: value.tracks.into_iter().map(TrackCache::from).collect(),
        }
    }
}

impl From<PlaylistCache> for Playlist {
    fn from(value: PlaylistCache) -> Self {
        Playlist {
            id: Uuid::parse_str(&value.id).expect("invalid uuid in cache"),
            name: value.name,
            path: Some(PathBuf::from(value.path)),
            tracks: value.tracks.into_iter().map(Track::from).collect(),
        }
    }
}

impl From<Track> for TrackCache {
    fn from(value: Track) -> Self {
        TrackCache {
            path: value.path.to_string_lossy().to_string(),
            meta: value.meta.into(),
        }
    }
}

impl From<TrackCache> for Track {
    fn from(value: TrackCache) -> Self {
        Track {
            path: PathBuf::from(value.path),
            meta: value.meta.into(),
        }
    }
}

impl From<Metadata> for MetadataCache {
    fn from(value: Metadata) -> Self {
        MetadataCache {
            title: value.title,
            artists: value.artists,
            album: value.album,
            genre: value.genre,
            duration: value.duration,
            writer: value.writer,
            producer: value.producer,
            publisher: value.publisher,
            label: value.label,
        }
    }
}

impl From<MetadataCache> for Metadata {
    fn from(value: MetadataCache) -> Self {
        Metadata {
            title: value.title,
            artists: value.artists,
            album: value.album,
            genre: value.genre,
            duration: value.duration,
            writer: value.writer,
            producer: value.producer,
            publisher: value.publisher,
            label: value.label,
            thumbnail: None,
        }
    }
}

impl CacheManager {
    pub fn init() -> Self {
        let cache_dir = dirs::audio_dir()
            .unwrap_or_default()
            .join("wiremann")
            .join("cache");
        fs::create_dir_all(cache_dir.clone()).expect("failed to create cache directory");

        CacheManager { cache_dir }
    }

    pub fn read_cached_playlist_indexes(&self) -> CachedPlaylistIndexes {
        let playlist_indexes: CachedPlaylistIndexes =
            match File::open(self.cache_dir.join("playlists.ron")) {
                Ok(mut file) => {
                    let mut playlists = String::new();
                    file.read_to_string(&mut playlists)
                        .expect("couldnt read to string");
                    ron::from_str(&playlists).unwrap_or_default()
                }
                Err(_) => CachedPlaylistIndexes::default(),
            };

        playlist_indexes
    }

    pub fn write_cached_playlist_indexes(&self, playlist_indexes: CachedPlaylistIndexes) {
        let bytes = ron::ser::to_string_pretty(&playlist_indexes, PrettyConfig::default())
            .unwrap_or_default();
        let tmp_path = self.cache_dir.join("playlists.tmp");
        let final_path = self.cache_dir.join("playlists.ron");

        fs::write(&tmp_path, &bytes).expect("write failed");
        fs::rename(&tmp_path, &final_path).expect("rename failed");
    }

    pub fn write_playlist(&mut self, playlist: Playlist, thumbnails: Vec<(PathBuf, Vec<u8>)>) {
        let base = match dirs::audio_dir() {
            Some(dir) => dir,
            None => return,
        };

        let cache_dir = base
            .join("wiremann")
            .join("cache")
            .join(playlist.id.to_string());

        fs::create_dir_all(&cache_dir).expect("couldnt create cache dir");

        let playlist: PlaylistCache = playlist.into();

        let mut thumbnails_cached = ThumbnailsCached {
            thumbnails: HashMap::new(),
        };

        for (path, image) in thumbnails {
            thumbnails_cached
                .thumbnails
                .insert(path.to_string_lossy().to_string(), image);
        }

        let playlist_encoded = bitcode::encode(&playlist);
        let thumbnails_encoded = bitcode::encode(&thumbnails_cached);

        let tracks_tmp = cache_dir.join("tracks.tmp");
        let tracks_final = cache_dir.join("tracks.bin");

        let thumbnails_tmp = cache_dir.join("thumbnails.tmp");
        let thumbnails_final = cache_dir.join("thumbnails.bin");

        fs::write(&tracks_tmp, &playlist_encoded).expect("write failed");
        fs::rename(&tracks_tmp, &tracks_final).expect("rename failed");

        fs::write(&thumbnails_tmp, &thumbnails_encoded).expect("write failed");
        fs::rename(&thumbnails_tmp, &thumbnails_final).expect("rename failed");
    }

    pub fn read_playlist(&self, id: String) -> Option<(Playlist, HashMap<PathBuf, Vec<u8>>)> {
        let path = self.cache_dir.join(id);

        let playlist: PlaylistCache = match File::open(path.clone().join("tracks.bin")) {
            Ok(mut file) => {
                let mut bytes = vec![];
                file.read_to_end(&mut bytes).expect("couldnt read bytes");

                match bitcode::decode(bytes.as_ref()) {
                    Ok(decoded) => decoded,
                    Err(_) => return None,
                }
            }
            Err(_) => return None,
        };

        let thumbnails_cache: ThumbnailsCached = match File::open(path.join("thumbnails.bin")) {
            Ok(mut file) => {
                let mut bytes = vec![];
                file.read_to_end(&mut bytes).expect("couldnt read bytes");

                match bitcode::decode(bytes.as_ref()) {
                    Ok(decoded) => decoded,
                    Err(_) => return None,
                }
            }
            Err(_) => return None,
        };

        let mut thumbnails = HashMap::new();

        for (path, image) in thumbnails_cache.thumbnails {
            thumbnails.insert(PathBuf::from(path), image);
        }

        Some((playlist.into(), thumbnails))
    }

    pub fn write_app_state(&self, player_state: PlayerState, scanner_state: ScannerState) {
        let tmp_path = self.cache_dir.join("app_state.tmp");
        let final_path = self.cache_dir.join("app_state.ron");

        let PlayerState {
            current,
            state,
            position,
            volume,
            mute,
            shuffling,
            repeat,
            index,
            ..
        } = player_state.clone();
        let ScannerState { queue_order, current_playlist, .. } = scanner_state.clone();

        let playlist = current_playlist.unwrap_or_default().path.unwrap_or_default().to_string_lossy().to_string();

        let app_state_cache = AppStateCache {
            current: current.map(|this| this.to_string_lossy().to_string()),
            state,
            position,
            volume,
            mute,
            shuffling,
            repeat,
            index,
            queue_order,
            playlist,
        };

        let bytes = ron::ser::to_string_pretty(&app_state_cache, PrettyConfig::default())
            .expect("couldnt serialize state");

        fs::write(&tmp_path, &bytes).expect("write failed");
        fs::rename(&tmp_path, &final_path).expect("rename failed");
    }

    pub fn read_app_state(&self) -> Option<AppStateCache> {
        let app_state_cache: AppStateCache = match File::open(self.cache_dir.join("app_state.ron"))
        {
            Ok(mut file) => {
                let mut app_state_bytes = String::new();
                file.read_to_string(&mut app_state_bytes)
                    .expect("couldnt read to string");
                ron::from_str(&app_state_bytes).unwrap_or_default()
            }
            Err(_) => return None,
        };

        Some(app_state_cache)
    }
}
