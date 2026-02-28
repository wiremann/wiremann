use crate::controller::commands::CacherCommand;
use crate::controller::events::CacherEvent;
use crate::controller::state::{LibraryState, PlaybackStatus};
use crate::errors::CacherError;
use crate::library::playlists::Playlist;
use bitcode::{Decode, Encode};
use crossbeam_channel::{Receiver, Sender};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

pub struct Cacher {
    pub tx: Sender<CacherEvent>,
    pub rx: Receiver<CacherCommand>,
    base_dir: PathBuf,
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
                CacherCommand::WriteAppState(app_state) => {}
                _ => {}
            }
        }
    }

    fn write_library_state(&self, state: LibraryState) -> Result<(), CacherError> {
        let dir = self.base_dir.join("library");

        fs::create_dir_all(dir.clone())?;

        let tracks_tmp_path = dir.join("tracks.tmp");
        let tracks_final_path = dir.join("tracks.bin");

        let playlists_tmp_path = dir.join("playlists.tmp");
        let playlists_final_path = dir.join("playlists.bin");

        Ok(())
    }
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
    Folder(String),
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
    pub playlists: HashMap<String, Playlist>,
}

#[derive(Debug, Clone, PartialEq, Default, Encode, Decode)]
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