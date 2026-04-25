use gpui::Global;

use crate::lyrics_manager::{Lyrics, SyncType};

pub struct LyricsState {
    pub sync_type: SyncType,
    pub lyrics: Option<Lyrics>,
}

impl Global for LyricsState {}
