pub mod providers;

use std::{cmp::Reverse, time::Duration};

use crate::{
    controller::{commands::LyricsCommand, events::LyricsEvent},
    errors::LyricsError,
    lyrics_manager::providers::{lrclib::LrcLib, youly::YouLY},
};
use crossbeam_channel::{Receiver, Sender};

pub static APP_USER_AGENT: &str = concat!(
    env!("CARGO_PKG_NAME"),
    " ",
    env!("CARGO_PKG_VERSION"),
    "(https://github.com/wiremann/wiremann)"
);
pub trait LyricsProvider: Send + Sync {
    fn get_lyrics(
        &self,
        title: &str,
        artist: &str,
        album: &str,
        duration: Duration,
    ) -> Result<Option<Lyrics>, LyricsError>;

    fn name(&self) -> &'static str;

    fn endpoint(&self) -> &'static str;

    fn priority(&self) -> u8;
}

#[derive(Clone, PartialEq, Debug)]
pub struct Lyrics {
    pub lines: Vec<LyricLine>,
    pub sync_type: SyncType,
}

#[derive(Debug)]
pub struct LyricLine {
    pub text: String,
    pub start: Option<Duration>,
    pub end: Option<Duration>,
    pub words: Option<Vec<LyricWord>>,
}

#[derive(Debug)]
pub struct LyricWord {
    pub start: Duration,
    pub end: Duration,
    pub text: String,
}

#[derive(Debug)]
pub enum SyncType {
    Unsynced,
    Line,
    Word,
}

pub struct LyricsManager {
    pub tx: Sender<LyricsEvent>,
    pub rx: Receiver<LyricsCommand>,

    pub providers: Vec<Box<dyn LyricsProvider>>,
}

impl LyricsManager {
    #[allow(unused_variables)]
    #[must_use]
    pub fn new() -> (Self, Sender<LyricsCommand>, Receiver<LyricsEvent>) {
        let (cmd_tx, cmd_rx) = crossbeam_channel::unbounded();
        let (event_tx, event_rx) = crossbeam_channel::unbounded();

        let youly: Box<dyn LyricsProvider> = Box::new(YouLY {});
        let lrclib: Box<dyn LyricsProvider> = Box::new(LrcLib {});

        let mut providers = vec![youly, lrclib];

        providers.sort_by_key(|p| Reverse(p.priority()));

        (
            Self {
                tx: event_tx,
                rx: cmd_rx,
                providers,
            },
            cmd_tx,
            event_rx,
        )
    }

    #[allow(clippy::missing_errors_doc)]
    pub fn run(&mut self) -> Result<(), LyricsError> {
        loop {
            match self.rx.recv()? {
                LyricsCommand::GetLyrics {
                    title,
                    artist,
                    album,
                    duration,
                } => {
                    if let Some(provider) = self.providers.first() {
                        if let Ok(Some(lyrics)) = provider.get_lyrics(
                            title.as_str(),
                            artist.as_str(),
                            album.as_str(),
                            duration,
                        ) {
                            self.tx.send(LyricsEvent::Lyrics(lyrics)).ok();
                        }
                    }
                }
            }
        }
    }
}
