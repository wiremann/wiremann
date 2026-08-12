use std::time::Duration;

use serde_json::Value;
use tracing::warn;

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
        let client = reqwest::blocking::Client::builder()
            .user_agent(APP_USER_AGENT)
            .build()?;

        // First try exact match by track_name + artist_name
        let exact = self.search(&client, title, Some(artist), duration, "api/get")?;
        if exact.is_some() {
            return Ok(exact);
        }

        // Fallback: search by track_name only — the artist field may differ
        // (e.g. YouTube rips, compilation albums, featuring artists).
        self.search(&client, title, None, duration, "api/search")
    }

    fn endpoint(&self) -> &'static str {
        "https://lrclib.net"
    }

    fn name(&self) -> &'static str {
        "LRCLIB"
    }

    fn priority(&self) -> u8 {
        20
    }
}

impl LrcLib {
    fn search(
        &self,
        client: &reqwest::blocking::Client,
        title: &str,
        artist: Option<&str>,
        duration: Duration,
        endpoint: &str,
    ) -> Result<Option<Lyrics>, LyricsError> {
        let dur_secs = duration.as_secs().to_string();

        let mut query: Vec<(&str, &str)> = vec![("track_name", title)];
        if let Some(a) = artist {
            query.push(("artist_name", a));
        }
        query.push(("duration", &dur_secs));

        let url = format!("https://lrclib.net/{}", endpoint);

        let resp = match client
            .get(&url)
            .query(&query)
            .timeout(Duration::from_secs(32))
            .send()
        {
            Ok(r) => r,
            Err(e) => {
                warn!(error = ?e, "LRCLIB request failed");
                return Ok(None);
            }
        };

        if !resp.status().is_success() {
            return Ok(None);
        }

        let text = match resp.text() {
            Ok(t) => t,
            Err(e) => {
                warn!(error = ?e, "Failed to read response");
                return Ok(None);
            }
        };

        // api/get returns a single JSON object; api/search returns an array.
        if endpoint == "api/search" {
            Self::parse_search_results(&text, dur_secs.parse().unwrap_or(0))
        } else {
            self.parse(&text)
        }
    }

    /// Parse the search endpoint response (array of results). Picks the best
    /// match by comparing the duration difference.
    fn parse_search_results(data: &str, target_secs: u64) -> Result<Option<Lyrics>, LyricsError> {
        let items: Vec<Value> = match serde_json::from_str(data) {
            Ok(a) => a,
            Err(_) => return Ok(None),
        };

        if items.is_empty() {
            return Ok(None);
        }

        // Score each result by how close its duration is to the target.
        let mut scored: Vec<(i64, &Value)> = items
            .iter()
            .filter_map(|item| {
                let dur = item
                    .get("duration")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0) as u64;
                let diff = (dur as i64 - target_secs as i64).abs();
                if diff > 10 {
                    return None; // skip anything more than 10s off
                }
                Some((diff, item))
            })
            .collect();

        scored.sort_by_key(|(diff, _)| *diff);

        if let Some((_, best)) = scored.first() {
            Self::parse_single(best)
        } else {
            // fallback: return the first result even if duration doesn't match
            Self::parse_single(&items[0])
        }
    }

    fn parse_single(item: &Value) -> Result<Option<Lyrics>, LyricsError> {
        let synced = item.get("syncedLyrics").and_then(|v| v.as_str());
        let plain = item.get("plainLyrics").and_then(|v| v.as_str());

        if let Some(lrc) = synced {
            Self::parse_lrc(lrc)
        } else if let Some(p) = plain {
            let lines: Vec<LyricLine> = p
                .lines()
                .map(|line| LyricLine {
                    text: line.to_string().into(),
                    start: None,
                    end: None,
                    words: None,
                })
                .collect();

            Ok(Some(Lyrics {
                lines: lines.into(),
                sync_type: SyncType::Unsynced,
            }))
        } else {
            warn!(provider = "LRCLIB", "search result has no lyrics field");
            Ok(None)
        }
    }

    pub fn parse(&self, data: &str) -> Result<Option<Lyrics>, LyricsError> {
        let json: Value = match serde_json::from_str(data) {
            Ok(j) => j,
            Err(e) => {
                warn!(error = ?e, "LRCLIB JSON parse failed");
                return Ok(None);
            }
        };

        match json.get("syncedLyrics") {
            Some(v) => {
                if let Some(s) = v.as_str() {
                    Self::parse_lrc(s)
                } else {
                    warn!(provider = "LRCLIB", "LRCLIB syncedLyrics not a string");
                    Ok(None)
                }
            }
            None => {
                if let Some(v) = json.get("plainLyrics") {
                    let mut lyrics = Lyrics {
                        lines: Vec::new().into(),
                        sync_type: SyncType::Unsynced,
                    };
                    if let Some(s) = v.as_str() {
                        let mut lines = Vec::new();
                        for line in s.lines() {
                            lines.push(LyricLine {
                                text: line.to_string().into(),
                                start: None,
                                end: None,
                                words: None,
                            });
                        }

                        lyrics.lines = lines.into();
                        Ok(Some(lyrics))
                    } else {
                        warn!(provider = "LRCLIB", "plainLyrics is not a string");
                        Ok(None)
                    }
                } else {
                    warn!(provider = "LRCLIB", "no lyrics found");
                    Ok(None)
                }
            }
        }
    }

    pub fn parse_lrc(data: &str) -> Result<Option<Lyrics>, LyricsError> {
        let mut lyrics = Lyrics {
            lines: Vec::new().into(),
            sync_type: SyncType::Line,
        };

        let data = data.replace("\\n", "\n");

        let mut lines = vec![];
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

                        lines.push(LyricLine {
                            text: text.to_string().into(),
                            start: Some(start),
                            end: None,
                            words: None,
                        });
                    }
                }
            }
        }

        for i in 0..lines.len().saturating_sub(1) {
            if lines[i].end.is_none() {
                lines[i].end = lines[i + 1].start;
            }
        }

        if let Some(last) = lines.last_mut()
            && last.end.is_none()
        {
            last.end = last.start;
        }

        lyrics.lines = lines.into();

        Ok(Some(lyrics))
    }
}
