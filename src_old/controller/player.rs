use super::metadata::Metadata;
use crate::audio::engine::PlaybackState;
use crate::scanner::cache::AppStateCache;
use crate::scanner::ScannerState;
use crossbeam_channel::{Receiver, Sender};
use gpui::*;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Controller {
    pub audio_cmd_tx: Sender<AudioCommand>,
    pub audio_events_rx: Receiver<AudioEvent>,
    pub scanner_cmd_tx: Sender<ScannerCommand>,
    pub scanner_events_rx: Receiver<ScannerEvent>,
    pub player_state: PlayerState,
    pub scanner_state: ScannerState,
}

#[derive(Debug, PartialEq, Clone)]
pub struct PlayerState {
    pub current: Option<PathBuf>,
    pub state: PlaybackState,
    pub position: u64,
    pub volume: f32,
    pub mute: bool,
    pub shuffling: bool,
    pub repeat: bool,
    pub index: usize,
    pub meta: Option<Metadata>,
    pub thumbnail: Option<Arc<RenderImage>>,
}

pub enum AudioCommand {
    Load(String),
    LoadId(usize),
    Play,
    Pause,
    Volume(f32),
    Mute,
    Seek(u64),
    Stop,
    Next,
    Prev,
    ScannerState(ScannerState),
    Repeat,
    Shuffle,
    SetAppState { app_state_cache: AppStateCache, scanner_cmd_tx: Sender<ScannerCommand> },
}

#[derive(Debug, Clone, PartialEq)]
pub enum AudioEvent {
    PlayerStateChanged(PlayerState),
    ScannerStateChanged(ScannerState),
    TrackLoaded(PathBuf),
    TrackEnded,
}

pub enum ScannerCommand {
    Load(String),
    GetPlayerCache,
    WritePlayerCache((PlayerState, ScannerState)),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ScannerEvent {
    State(ScannerState),
    Thumbnail {
        path: PathBuf,
        image: Arc<RenderImage>,
    },
    ClearImageCache,
    AppStateCache(AppStateCache),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    Audio(AudioEvent),
    Scanner(ScannerEvent),
}

impl Controller {
    pub fn new(
        audio_cmd_tx: Sender<AudioCommand>,
        audio_events_rx: Receiver<AudioEvent>,
        scanner_cmd_tx: Sender<ScannerCommand>,
        scanner_events_rx: Receiver<ScannerEvent>,
        player_state: PlayerState,
        scanner_state: ScannerState,
    ) -> Controller {
        Controller {
            audio_cmd_tx,
            audio_events_rx,
            scanner_cmd_tx,
            scanner_events_rx,
            player_state,
            scanner_state,
        }
    }

    pub fn play(&self) {
        let _ = self.audio_cmd_tx.send(AudioCommand::Play);
    }

    pub fn pause(&self) {
        let _ = self.audio_cmd_tx.send(AudioCommand::Pause);
    }

    pub fn load(&self, path: String) {
        let _ = self.audio_cmd_tx.send(AudioCommand::Load(path));
    }

    pub fn load_id(&self, id: usize) {
        let _ = self.audio_cmd_tx.send(AudioCommand::LoadId(id));
    }

    pub fn volume(&self, volume: f32) {
        let _ = self.audio_cmd_tx.send(AudioCommand::Volume(volume / 100.0));
    }

    pub fn mute(&self) {
        let _ = self.audio_cmd_tx.send(AudioCommand::Mute);
    }

    pub fn seek(&self, secs: u64) {
        let _ = self.audio_cmd_tx.send(AudioCommand::Seek(secs));
    }

    pub fn next(&self) {
        let _ = self.audio_cmd_tx.send(AudioCommand::Next);
    }

    pub fn prev(&self) {
        let _ = self.audio_cmd_tx.send(AudioCommand::Prev);
    }

    pub fn set_scanner_state_in_engine(&self, scanner_state: ScannerState) {
        let _ = self
            .audio_cmd_tx
            .send(AudioCommand::ScannerState(scanner_state));
    }

    pub fn load_playlist(&self, path: String) {
        let _ = self.scanner_cmd_tx.send(ScannerCommand::Load(path));
    }

    pub fn set_repeat(&self) {
        let _ = self.audio_cmd_tx.send(AudioCommand::Repeat);
    }

    pub fn set_shuffle(&self) {
        let _ = self.audio_cmd_tx.send(AudioCommand::Shuffle);
    }

    pub fn get_app_state_cache(&self) {
        let _ = self.scanner_cmd_tx.send(ScannerCommand::GetPlayerCache);
    }

    pub fn write_app_state_cache(&self) {
        let _ = self.scanner_cmd_tx.send(ScannerCommand::WritePlayerCache((
            self.player_state.clone(),
            self.scanner_state.clone(),
        )));
    }

    pub fn send_app_state_cache(&self, app_state_cache: AppStateCache, scanner_cmd_tx: Sender<ScannerCommand>) {
        let _ = self
            .audio_cmd_tx
            .send(AudioCommand::SetAppState { app_state_cache, scanner_cmd_tx });
    }
}

impl gpui::Global for Controller {}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            current: None,
            state: PlaybackState::Stopped,
            position: 0,
            volume: 1.0,
            meta: None,
            mute: false,
            shuffling: false,
            repeat: false,
            index: 0,
            thumbnail: None,
        }
    }
}

#[derive(Clone, Copy)]
pub struct ResHandler {}

impl ResHandler {
    pub fn handle(&mut self, cx: &mut Context<Self>, event: Event) {
        cx.emit(event);
        cx.notify();
    }
}

pub enum PlayerStateEvent {
    Position(u64),
}

#[derive(Clone, PartialEq, Debug, Default)]
pub struct Track {
    pub path: PathBuf,
    pub meta: Metadata,
}

impl EventEmitter<Event> for ResHandler {}
impl EventEmitter<PlayerStateEvent> for PlayerState {}
