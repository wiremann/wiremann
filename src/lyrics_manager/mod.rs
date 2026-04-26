pub mod providers;

use std::{cmp::Reverse, sync::Arc, time::Duration};

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

#[derive(Clone, PartialEq, Debug)]
pub struct LyricLine {
    pub text: String,
    pub start: Option<Duration>,
    pub end: Option<Duration>,
    pub words: Option<Vec<LyricWord>>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct LyricWord {
    pub start: Duration,
    pub end: Duration,
    pub text: String,
}

#[derive(Clone, PartialEq, Debug)]
pub enum SyncType {
    Unsynced,
    Line,
    Word,
}

pub struct LyricsManager {
    pub tx: Sender<LyricsEvent>,
    pub rx: Receiver<LyricsCommand>,

    pub providers: Arc<Vec<Box<dyn LyricsProvider>>>,
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
                providers: Arc::new(providers),
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
                    id,
                    title,
                    artist,
                    album,
                    duration,
                } => {
                    let providers = self.providers.clone();
                    let tx = self.tx.clone();

                    std::thread::spawn(move || {
                        let mut found = None;

                        for provider in &*providers {
                            match provider.get_lyrics(
                                title.as_str(),
                                artist.as_str(),
                                album.as_str(),
                                duration,
                            ) {
                                Ok(Some(lyrics)) => {
                                    found = Some(lyrics);
                                    break;
                                }
                                Ok(None) => {
                                    eprintln!("{} returned no lyrics", provider.name());
                                }
                                Err(e) => {
                                    eprintln!("{} failed: {:?}", provider.name(), e);
                                }
                            }
                        }

                        tx.send(LyricsEvent::Lyrics(id, found)).ok();
                    });
                }
            }
        }
    }
}
