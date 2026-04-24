use std::time::Duration;

use crate::{
    errors::LyricsError,
    lyrics_manager::{APP_USER_AGENT, Lyrics, LyricsProvider},
};

pub struct YouLY;

// Times are all in milliseconds
struct YouLYLyric {
    time: u64,
    duration: u64,
    text: String,
    syllabus: Vec<YouLYWord>
}

struct YouLYWord {
    time: u64,
    duration: u64,
    text: String
}

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

        // TODO: streamline this, idk how
        let attempts: Vec<Vec<(&str, &str)>> = vec![
            vec![
                ("title", title),
                ("artist", artist),
                ("album", album),
                ("duration", duration.as_str()),
            ],
            vec![
                ("title", title),
                ("artist", artist),
                ("duration", duration.as_str()),
            ],
            vec![("title", title), ("artist", artist), ("album", album)],
            vec![("title", title), ("artist", artist)],
        ];

        for query in attempts {
            let resp = match client
                .get(endpoint)
                .query(&query)
                .timeout(Duration::from_secs(4))
                .send()
            {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("YouLY request failed: {:?}", e);
                    continue;
                }
            };

            if !resp.status().is_success() {
                continue;
            }

            let text = match resp.text() {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("Failed to read response: {:?}", e);
                    continue;
                }
            };

            println!("got response; {text:#?}");
        }

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
