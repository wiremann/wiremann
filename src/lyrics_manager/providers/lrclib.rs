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

        return self.parse(text);
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
                return Self::parse_lrc(v.as_str().unwrap_or_default());
            }
            None => match json.get("plainLyrics") {
                Some(v) => {
                    let mut lyrics = Lyrics {
                        lines: Vec::new(),
                        sync_type: SyncType::Unsynced,
                    };

                    for line in v.as_str().unwrap_or_default().lines() {
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

    pub fn parse_lrc(data: &str) -> Result<Option<Lyrics>, LyricsError> {
        let mut lyrics = Lyrics {
            lines: Vec::new(),
            sync_type: SyncType::Unsynced,
        };

        let data = data.replace("\\n", "\n");

        for line in data.lines() {
            if let Some((time_part, text)) = line.split_once("] ") {
                let timestamp = time_part.trim_start_matches('[');

                let mut parts = timestamp.split(':');
                let minutes = parts.next().and_then(|m| m.parse::<u64>().ok());
                let rest = parts.next();

                if let (Some(min), Some(rest)) = (minutes, rest) {
                    let mut sec_parts = rest.split('.');
                    let seconds = sec_parts.next().and_then(|s| s.parse::<u64>().ok());
                    let centis = sec_parts.next().and_then(|ms| ms.parse::<u64>().ok());

                    if let (Some(sec), Some(cs)) = (seconds, centis) {
                        let millis = cs * 10;
                        let total = min * 60_000 + sec * 1_000 + millis;

                        let start = Duration::from_millis(total);

                        lyrics.lines.push(LyricLine {
                            text: text.to_string(),
                            start: Some(start),
                            end: None,
                            words: None,
                        });
                    }
                }
            }
        }

        for i in 0..lyrics.lines.len().saturating_sub(1) {
            if lyrics.lines[i].end.is_none() {
                lyrics.lines[i].end = lyrics.lines[i + 1].start;
            }
        }

        if let Some(last) = lyrics.lines.last_mut() {
            if last.end.is_none() {
                last.end = last.start;
            }
        }

        Ok(None)
    }
}
