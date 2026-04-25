use gpui::{Entity, Global};

use crate::lyrics_manager::{Lyrics, SyncType};

#[derive(Debug, PartialEq)]
pub struct LyricsStateInner {
    pub status: LyricsStatus,
    pub lyrics: Option<Lyrics>,
}

#[derive(Debug, PartialEq)]
pub enum LyricsStatus {
    Fetching,
    Available,
    Unavailable,
}

pub struct LyricsState(pub Entity<LyricsStateInner>);

impl Global for LyricsState {}

impl LyricsStateInner {
    pub fn new() -> Self {
        LyricsStateInner {
            status: LyricsStatus::Unavailable,
            lyrics: None,
        }
    }
}
