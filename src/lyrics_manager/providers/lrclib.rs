use std::time::Duration;

use serde_json::Value;

use crate::lyrics_manager::{APP_USER_AGENT, LyricLine, LyricsProvider, SyncType};
use crate::{errors::LyricsError, lyrics_manager::Lyrics};

pub struct LrcLib;

impl LyricsProvider for LrcLib {
    fn get_lyrics(
        &self,
        title: &str,
        artist: &str,
        _album: &str,
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
            // lrclib allows filtering by album, but doing so can be too restrictive.
            // In practice, specifying an album may exclude valid synced lyrics
            // if they are indexed under a different or missing album entry.
            // We intentionally pass an empty string to broaden the search.
            ("album_name", ""),
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

        self.parse(text);
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

impl LrcLib {
    pub fn parse(&self, data: String) -> Result<Option<Lyrics>, LyricsError> {
        let json: Value = match serde_json::from_str(&data) {
            Ok(j) => j,
            Err(e) => {
                eprintln!("LRCLIB JSON parse failed: {:?}", e);
                return Ok(None);
            }
        };

        match json.get("syncedLyrics") {
            Some(v) => {
                return Self::parse_lrc(v.to_string());
            }
            None => match json.get("plainLyrics") {
                Some(v) => {
                    let mut lyrics = Lyrics {
                        lines: Vec::new(),
                        sync_type: SyncType::Unsynced,
                    };

                    for line in v.to_string().lines() {
                        lyrics.lines.push(LyricLine {
                            text: line.to_string(),
                            start: None,
                            end: None,
                            words: None,
                        });
                    }

                    return Ok(Some(lyrics));
                }
                None => {
                    eprintln!("LRCLIB no lyrics found");
                    return Ok(None);
                }
            },
        };
    }

    pub fn parse_lrc(data: String) -> Result<Option<Lyrics>, LyricsError> {
        let mut lyrics = Lyrics {
            lines: Vec::new(),
            sync_type: SyncType::Unsynced,
        };

        for line in data.to_string().lines() {
            println!("line: {line}");
        }

        Ok(None)
    }
}
