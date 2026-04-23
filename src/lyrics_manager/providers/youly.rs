use crate::{
    errors::LyricsError,
    lyrics_manager::{Lyrics, LyricsProvider},
};

pub struct YouLY;

impl LyricsProvider for YouLY {
    fn get_lyrics(
        &self,
        title: &str,
        artist: &str,
        album: &str,
        duration: u64,
    ) -> Result<Option<Lyrics>, LyricsError> {
        Ok(None)
    }

    fn name(&self) -> &'static str {
        "YouLY"
    }

    fn priority(&self) -> u8 {
        20
    }
}
