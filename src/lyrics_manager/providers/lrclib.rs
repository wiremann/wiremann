use std::time::Duration;

use crate::lyrics_manager::{APP_USER_AGENT, LyricsProvider};
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
        let endpoint = self.endpoint();
        let client = reqwest::blocking::Client::builder()
            .user_agent(APP_USER_AGENT)
            .build()?;
        let duration = duration.as_secs().to_string();
        let query = vec![
            ("track_name", title),
            ("artist_name", artist),
            ("album_name", album),
            ("duration", duration.as_str()),
        ];

        let resp = match client
            .get(endpoint)
            .query(&query)
            .timeout(Duration::from_secs(32))
            .send()
        {
            Ok(r) => r,
            Err(e) => {
                eprintln!("LRCLIB request failed: {:?}", e);
                return Ok(None);
            }
        };

        println!("sent request: {}", resp.url().as_str());

        if !resp.status().is_success() {
            return Ok(None);
        }

        let text = match resp.text() {
            Ok(t) => t,
            Err(e) => {
                eprintln!("Failed to read response: {:?}", e);
                return Ok(None);
            }
        };

        println!("{text}");
        Ok(None)
    }

    fn endpoint(&self) -> &'static str {
        "https://lrclib.net/api/get"
    }

    fn name(&self) -> &'static str {
        "LRCLIB"
    }

    fn priority(&self) -> u8 {
        20
    }
}
