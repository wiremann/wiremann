use std::time::Duration;

use serde::Deserialize;
use serde_json::Value;

use crate::{
    errors::LyricsError,
    lyrics_manager::{APP_USER_AGENT, LyricLine, LyricWord, Lyrics, LyricsProvider, SyncType},
};

pub struct YouLY;

// Times are all in milliseconds
#[derive(Deserialize, Debug)]
struct YouLYLine {
    time: u64,
    duration: u64,
    text: String,
    #[serde(default)]
    syllabus: Vec<YouLYWord>,
}

#[derive(Deserialize, Debug)]
struct YouLYWord {
    time: u64,
    duration: u64,
    text: String,
}

impl LyricsProvider for YouLY {
    fn get_lyrics(
        &self,
        title: &str,
        artist: &str,
        _album: &str,
        _duration: Duration,
    ) -> Result<Option<Lyrics>, LyricsError> {
        let endpoint = self.endpoint();

        let client = reqwest::blocking::Client::builder()
            .user_agent(APP_USER_AGENT)
            .build()?;

        let query = vec![("title", title), ("artist", artist)];

        let resp = match client
            .get(endpoint)
            .query(&query)
            .timeout(Duration::from_secs(4))
            .send()
        {
            Ok(r) => r,
            Err(e) => {
                eprintln!("YouLY request failed: {:?}", e);
                return Ok(None);
            }
        };

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

        return self.parse(text);
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

impl YouLY {
    fn parse(&self, data: String) -> Result<Option<Lyrics>, LyricsError> {
        let json: Value = match serde_json::from_str(&data) {
            Ok(j) => j,
            Err(e) => {
                eprintln!("YouLY JSON parse failed: {:?}", e);
                return Ok(None);
            }
        };

        let lyrics_value = match json.get("lyrics") {
            Some(v) => v.clone(),
            None => {
                eprintln!("YouLY missing 'lyrics' field");
                return Ok(None);
            }
        };

        let lines: Vec<YouLYLine> = match serde_json::from_value(lyrics_value) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("YouLY lyrics parse failed: {:?}", e);
                return Ok(None);
            }
        };

        let has_words = lines.iter().any(|l| !l.syllabus.is_empty());

        let lyrics = Lyrics {
            lines: lines.into_iter().map(Into::into).collect(),
            sync_type: if has_words {
                SyncType::Word
            } else {
                SyncType::Line
            },
        };

        Ok(Some(lyrics))
    }
}

impl From<YouLYLine> for LyricLine {
    fn from(value: YouLYLine) -> Self {
        let start = Duration::from_millis(value.time);
        let end = start + Duration::from_millis(value.duration);

        let words = if value.syllabus.is_empty() {
            None
        } else {
            Some(
                value
                    .syllabus
                    .into_iter()
                    .map(|w| {
                        let w_start = Duration::from_millis(w.time);
                        let w_end = w_start + Duration::from_millis(w.duration);

                        LyricWord {
                            start: w_start,
                            end: w_end,
                            text: w.text,
                        }
                    })
                    .collect(),
            )
        };
        LyricLine {
            text: value.text,
            start: Some(start),
            end: Some(end),
            words,
        }
    }
}
