use std::time::Duration;

use crate::lyrics_manager::LyricsProvider;
use crate::{errors::LyricsError, lyrics_manager::Lyrics};

pub struct LrcLib;

impl LyricsProvider for LrcLib {
    fn get_lyrics(
        &self,
        title: &str,
        artist: &str,
        album: &str,
        duration: Duration,
    ) -> Result<Option<Lyrics>, LyricsError> {
        Ok(None)
    }

    fn endpoint(&self) -> &'static str {
        ""
    }

    fn name(&self) -> &'static str {
        "LRCLIB"
    }

    fn priority(&self) -> u8 {
        20
    }
}
