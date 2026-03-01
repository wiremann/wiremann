use crate::controller::commands::CacherCommand;
use crate::controller::events::CacherEvent;
use crate::controller::state::{AppState, LibraryState, PlaybackState, PlaybackStatus, QueueState};
use crate::errors::CacherError;
use crate::library::playlists::PlaylistId;
use crate::library::{Track, TrackId};
use bitcode::{Decode, Encode};
use crossbeam_channel::{Receiver, Sender};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;

pub struct Cacher {
    pub tx: Sender<CacherEvent>,
    pub rx: Receiver<CacherCommand>,
    base_dir: PathBuf,
}

#[derive(Encode, Decode)]
struct CacheFile<T> {
    version: u32,
    payload: T,
}

enum ImageKind {
    Thumbnail,
    AlbumArt,
}
#[derive(Debug, Clone, PartialEq, Default, Encode, Decode)]
struct CachedTrack {
    pub id: [u8; 32],
    pub path: String,

    pub title: String,
    pub artist: String,
    pub album: String,

    pub duration: u64,
    pub size: u64,
    pub modified: u64,
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
    pub tracks: Vec<[u8; 32]>,
}

#[derive(Debug, Clone, PartialEq, Default, Encode, Decode)]
struct CachedTracks {
    pub tracks: HashMap<[u8; 32], CachedTrack>,
}

#[derive(Debug, Clone, PartialEq, Default, Encode, Decode)]
struct CachedPlaylists {
    pub playlists: HashMap<String, CachedPlaylist>,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
struct CachedPlaybackState {
    pub current: Option<[u8; 32]>,
    pub current_playlist: Option<String>,

    pub status: PlaybackStatus,
    pub position: u64,

    pub volume: f32,
    pub mute: bool,
    pub shuffling: bool,
    pub repeat: bool,
}

#[derive(Debug, Clone, PartialEq, Default, Encode, Decode)]
pub struct CachedQueueState {
    pub tracks: Vec<[u8; 32]>,
    pub order: Vec<usize>,
    pub index: usize,
}

impl From<&Track> for CachedTrack {
    fn from(track: &Track) -> Self {
        Self {
            id: track.id.0,
            path: track.path.to_string_lossy().to_string(),
            title: track.title.clone(),
            artist: track.artist.clone(),
            album: track.album.clone(),
            duration: track.duration,
            size: track.size,
            modified: track.modified,
        }
    }
}

impl From<CachedTrack> for Track {
    fn from(c: CachedTrack) -> Self {
        Self {
            id: TrackId(c.id),
            path: PathBuf::from(c.path),
            title: c.title,
            artist: c.artist,
            album: c.album,
            duration: c.duration,
            size: c.size,
            modified: c.modified,
        }
    }
}

impl From<&LibraryState> for CachedTracks {
    fn from(state: &LibraryState) -> Self {
        let tracks = state
            .tracks
            .iter()
            .map(|(id, track)| (id.0, CachedTrack::from(track.as_ref())))
            .collect();

        Self { tracks }
    }
}

impl From<CachedTracks> for LibraryState {
    fn from(cache: CachedTracks) -> Self {
        let tracks = cache
            .tracks
            .into_iter()
            .map(|(id, track)| {
                let track: Track = track.into();
                (TrackId(id), Arc::new(track))
            })
            .collect();

        Self {
            tracks,
            playlists: HashMap::new(),
        }
    }
}

impl From<&PlaybackState> for CachedPlaybackState {
    fn from(p: &PlaybackState) -> Self {
        Self {
            current: p.current.map(|id| id.0),
            current_playlist: p.current_playlist.map(|id| id.0.to_string()),
            status: p.status,
            position: p.position,
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
            current_playlist: c.current_playlist
                .and_then(|s| Some(PlaylistId(Uuid::from_str(&s).unwrap_or_default()))),
            status: c.status,
            position: c.position,
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
            index: q.index,
        }
    }
}

impl From<CachedQueueState> for QueueState {
    fn from(c: CachedQueueState) -> Self {
        Self {
            tracks: c.tracks.into_iter().map(TrackId).collect(),
            order: c.order,
            index: c.index,
        }
    }
}

impl Cacher {
    pub fn new() -> (Self, Sender<CacherCommand>, Receiver<CacherEvent>) {
        let (cmd_tx, cmd_rx) = crossbeam_channel::unbounded();
        let (event_tx, event_rx) = crossbeam_channel::unbounded();

        let base_dir = dirs::audio_dir().unwrap_or_default().join("wiremann").join("cache");
        fs::create_dir_all(base_dir.clone()).expect("failed to create cache directory");

        let cacher = Cacher {
            tx: event_tx,
            rx: cmd_rx,
            base_dir,
        };

        (cacher, cmd_tx, event_rx)
    }

    pub fn run(&self) -> Result<(), CacherError> {
        loop {
            match self.rx.recv()? {
                CacherCommand::WriteAppState(app_state) => {
                    self.write_library_state(&app_state.library)?;
                    self.write_playback_state(&app_state.playback)?;
                    self.write_queue_state(&app_state.queue)?;
                }
                CacherCommand::WriteAlbumArt { id, image } => self.write_cached_image(id, ImageKind::AlbumArt, &image)?,
                CacherCommand::WriteThumbnail { id, image } => self.write_cached_image(id, ImageKind::Thumbnail, &image)?,
                CacherCommand::GetAppState => {
                    let state = self.load_app_state()?;
                    let _ = self.tx.send(CacherEvent::AppState(state));
                }
                _ => {}
            }
        }
    }

    fn write_library_state(&self, state: &LibraryState) -> Result<(), CacherError> {
        let dir = self.base_dir.join("library");
        fs::create_dir_all(&dir)?;

        let tracks = CachedTracks::from(state);

        write_cache(
            &dir.join("tracks.tmp"),
            &dir.join("tracks.bin"),
            tracks,
        )?;

        Ok(())
    }

    fn write_playback_state(&self, state: &PlaybackState) -> Result<(), CacherError> {
        let tmp_path = self.base_dir.join("session.tmp");
        let final_path = self.base_dir.join("session.ron");

        let payload = CachedPlaybackState::from(state);

        let ron = ron::ser::to_string_pretty(&payload, Default::default())?;

        {
            let mut file = fs::File::create(tmp_path.clone())?;
            file.write_all(&ron.as_bytes())?;
            file.sync_all()?;
        }

        fs::rename(tmp_path, final_path)?;

        Ok(())
    }

    fn write_queue_state(&self, state: &QueueState) -> Result<(), CacherError> {
        let tmp_path = self.base_dir.join("queue.tmp");
        let final_path = self.base_dir.join("queue.bin");

        let queue = CachedQueueState::from(state);

        write_cache(
            &tmp_path,
            &final_path,
            queue,
        )?;

        Ok(())
    }

    fn cached_image_path(&self, id: TrackId, kind: ImageKind) -> PathBuf {
        let hex = hex::encode(id.0);
        let folder = &hex[0..2];

        let name = match kind {
            ImageKind::Thumbnail => format!("{hex}_thumb.bgra.zstd"),
            ImageKind::AlbumArt => format!("{hex}_art.bgra.zstd"),
        };

        self.base_dir
            .join("images")
            .join(folder)
            .join(name)
    }

    fn write_cached_image(&self, id: TrackId, kind: ImageKind, bytes: &[u8]) -> Result<(), CacherError> {
        let final_path = self.cached_image_path(id, kind);
        let tmp_path = final_path.with_extension("tmp");

        if final_path.exists() {
            return Ok(());
        }

        fs::create_dir_all(final_path.parent().unwrap())?;

        let compressed = zstd::encode_all(bytes, 3)?;

        {
            let mut file = fs::File::create(&tmp_path)?;
            file.write_all(&compressed)?;
            file.sync_all()?;
        }

        fs::rename(tmp_path, final_path)?;

        Ok(())
    }

    fn load_app_state(&self) -> Result<AppState, CacherError> {
        let library = self.read_library_state()?;
        let playback = self.read_playback_state()?;
        let queue = self.read_queue_state()?;

        Ok(AppState {
            library,
            playback,
            queue,
        })
    }

    fn read_queue_state(&self) -> Result<QueueState, CacherError> {
        let path = self.base_dir.join("queue.bin");

        if !path.exists() {
            return Ok(QueueState::default());
        }

        let cached: CachedQueueState = read_cache(&path)?;
        Ok(cached.into())
    }

    fn read_playback_state(&self) -> Result<PlaybackState, CacherError> {
        let path = self.base_dir.join("session.bin");

        if !path.exists() {
            return Ok(PlaybackState::default());
        }

        let ron = fs::read_to_string(path)?;
        let cached: CachedPlaybackState = ron::de::from_str(&ron)?;

        Ok(cached.into())
    }
}

fn write_cache<T: Encode>(
    tmp: &PathBuf,
    final_path: &PathBuf,
    payload: T,
) -> Result<(), CacherError> {
    let wrapped = CacheFile {
        version: 1,
        payload,
    };

    let bytes = bitcode::encode(&wrapped);

    {
        let mut file = fs::File::create(tmp)?;
        file.write_all(&bytes)?;
        file.sync_all()?;
    }

    fs::rename(tmp, final_path)?;

    Ok(())
}

fn read_cache<T>(
    path: &PathBuf,
) -> Result<T, CacherError>
where
    T: for<'a> Decode<'a>,
{
    if !path.exists() {
        return Ok(None);
    }

    let bytes = fs::read(path)?;

    let file: CacheFile<T> = bitcode::decode(&bytes)?;

    if file.version != 1 {
        return Ok(None);
    }

    Ok(file.payload)
}