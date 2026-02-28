use crate::{
    library::playlists::{Playlist, PlaylistId},
    library::{Track, TrackId},
};
use std::{collections::HashMap, sync::Arc};

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
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
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

    pub status: PlaybackStatus,
    pub position: u64,

    pub volume: f32,
    pub mute: bool,
    pub shuffling: bool,
    pub repeat: bool,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct QueueState {
    pub tracks: Vec<TrackId>,
    pub order: Vec<usize>,
    pub index: usize,
}

impl Default for PlaybackState {
    fn default() -> Self {
        PlaybackState {
            current: None,
            current_playlist: None,
            status: PlaybackStatus::Stopped,
            position: 0,
            volume: 1.0,
            mute: false,
            shuffling: false,
            repeat: false,
        }
    }
}