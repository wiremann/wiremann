use crate::{
    app::AppPaths,
    controller::{commands::LyricsCommand, events::LyricsEvent},
    errors::LyricsError,
};
use crossbeam_channel::{Receiver, Sender};

pub trait LyricsProvider: Send + Sync {
    fn get_lyrics(
        &self,
        title: &str,
        artist: &str,
        album: &str,
        duration: u64,
    ) -> Result<Option<Lyrics>, LyricsError>;
    fn name(&self) -> &'static str;
}

pub struct Lyrics {
    pub lines: Vec<LyricLine>,
    pub sync_type: SyncType,
}

pub struct LyricLine {
    pub text: String,
    pub start: Option<u64>,
    pub end: Option<u64>,
    pub words: Option<Vec<LyricWord>>,
}

pub struct LyricWord {
    pub start: u64,
    pub end: u64,
    pub text: String,
}

pub enum SyncType {
    Unsynced,
    Line,
    Word,
}

pub struct LyricsManager {
    pub tx: Sender<LyricsEvent>,
    pub rx: Receiver<LyricsCommand>,
    app_paths: AppPaths,

    pub providers: Vec<Box<dyn LyricsProvider>>,
}

impl LyricsManager {
    #[allow(unused_variables)]
    #[must_use]
    pub fn new(app_paths: AppPaths) -> (Self, Sender<LyricsCommand>, Receiver<LyricsEvent>) {
        let (cmd_tx, cmd_rx) = crossbeam_channel::unbounded();
        let (event_tx, event_rx) = crossbeam_channel::unbounded();

        (
            Self {
                tx: event_tx,
                rx: cmd_rx,
                app_paths,
                providers: Vec::new(),
            },
            cmd_tx,
            event_rx,
        )
    }

    #[allow(clippy::missing_errors_doc)]
    pub fn run(&mut self) -> Result<(), LyricsError> {
        loop {
            match self.rx.recv()? {
                _ => {}
            }
        }
    }
}
