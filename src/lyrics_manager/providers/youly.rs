use std::time::Duration;

use crate::{
    errors::LyricsError,
    lyrics_manager::{APP_USER_AGENT, Lyrics, LyricsProvider},
};

pub struct YouLY;

impl LyricsProvider for YouLY {
    fn get_lyrics(
        &self,
        title: &str,
        artist: &str,
        album: &str,
        duration: Duration,
    ) -> Result<Option<Lyrics>, LyricsError> {
        let endpoint = self.endpoint();

        let client = reqwest::blocking::Client::builder()
            .user_agent(APP_USER_AGENT)
            .build()?;

        let duration = duration.as_millis().to_string();

        let query = vec![
            ("title", title),
            ("artist", artist),
            ("album", album),
            ("duration", duration.as_str()),
        ];

        let resp = client
            .get(endpoint)
            .query(&query)
            .timeout(Duration::from_secs(4))
            .send()?;

        println!("url: {}", resp.url().to_string());
        println!("got response: {resp:#?}");

        Ok(None)
    }

    fn endpoint(&self) -> &'static str {
        "https://lyricsplus.prjktla.workers.dev/v2/lyrics/get"
    }

    fn name(&self) -> &'static str {
        "YouLY"
    }

    fn priority(&self) -> u8 {
        100
    }
}
