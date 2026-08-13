use crate::controller::state::{TrackId, TrackSource};
use crate::errors::ScannerError;
use lofty::file::{AudioFile, TaggedFileExt};
use lofty::read_from_path;
use lofty::tag::ItemKey;
use std::path::Path;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ScannedTrack {
    pub id: TrackId,
    pub source: TrackSource,

    pub title: String,
    pub artists: Vec<String>,
    pub album: String,

    pub duration: Duration,
}

#[allow(clippy::missing_errors_doc)]
pub fn read_metadata(track_source: TrackSource) -> Result<ScannedTrack, ScannerError> {
    let path = track_source.path.as_path();

    let file = read_from_path(path).ok();

    let (mut title, mut artists, mut album) = fallback_metadata(path);
    let mut duration = Duration::ZERO;

    if let Some(tagged_file) = file {
        if let Some(tag) = tagged_file
            .primary_tag()
            .or_else(|| tagged_file.first_tag())
        {
            if let Some(t) = tag.get_string(ItemKey::TrackTitle) {
                title = t.to_string();
            }

            let tag_artists: Vec<String> = tag
                .get_strings(ItemKey::TrackArtist)
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(ToOwned::to_owned)
                .collect();

            if !tag_artists.is_empty() {
                artists = tag_artists;
            }

            if let Some(a) = tag.get_string(ItemKey::AlbumTitle) {
                album = a.to_string();
            }
        }

        duration = tagged_file.properties().duration();
    }

    // YouTube rips and bad tags often store "Artist - Title" as the title field
    // while leaving the artist tag empty or wrong. Try to repair this.
    clean_title_from_artists(&mut title, &mut artists);

    // If the artist is still unknown after tag reading and title cleaning,
    // attempt to parse it from the filename (the fallback may have had it
    // before tags overwrote it).
    if artists.is_empty() || artists.iter().any(|a| a.eq_ignore_ascii_case("Unknown Artist")) {
        if let Some(idx) = path
            .file_stem()
            .and_then(|s| s.to_str())
            .and_then(|stem| stem.find(" - "))
        {
            let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
            let parsed_artist = stem[..idx].trim();
            let parsed_title = stem[idx + 3..].trim();
            if !parsed_artist.is_empty() && !parsed_title.is_empty() {
                artists = vec![parsed_artist.to_string()];
                // If the tag had a correct title (no "Artist - " prefix), keep it.
                // Otherwise use the parsed title from filename.
                if title.eq_ignore_ascii_case(&format!("{} - {}", parsed_artist, parsed_title))
                    || title.is_empty()
                {
                    title = parsed_title.to_string();
                }
            }
        }
    }

    Ok(ScannedTrack {
        id: TrackId::generate(&title, &artists.join(", "), &album)?,
        source: track_source,
        title,
        artists,
        album,
        duration,
    })
}

#[allow(clippy::missing_errors_doc)]
pub fn read_album_art(path: &Path) -> Result<Option<Box<[u8]>>, ScannerError> {
    let file = read_from_path(path).ok();

    if let Some(tagged_file) = file
        && let Some(tag) = tagged_file
            .primary_tag()
            .or_else(|| tagged_file.first_tag())
    {
        return Ok(tag.pictures().first().map(|data| Box::from(data.data())));
    }

    Ok(None)
}

/// If the title looks like "Artist - Title" but the artist field is empty or
/// mismatched, split it apart. This is common for YouTube rips and poor tags.
fn clean_title_from_artists(title: &mut String, artists: &mut Vec<String>) {
    if let Some(idx) = title.find(" - ") {
        let prefix = title[..idx].trim();
        let suffix = title[idx + 3..].trim();

        if !prefix.is_empty() && !suffix.is_empty() {
            let artist_is_unknown = artists.is_empty()
                || artists.iter().any(|a| {
                    a.eq_ignore_ascii_case("Unknown Artist") || a.eq_ignore_ascii_case(prefix)
                });

            if artist_is_unknown {
                *artists = vec![prefix.to_string()];
                *title = suffix.to_string();
            }
        }
    }
}

fn fallback_metadata(path: &Path) -> (String, Vec<String>, String) {
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown");

    // Try to parse "Artist - Title" convention from the filename.
    if let Some(idx) = stem.find(" - ") {
        let (artist, title) = stem.split_at(idx);
        let title = title.trim_start_matches(" - ").trim();
        let artist = artist.trim();
        if !artist.is_empty() && !title.is_empty() {
            return (
                title.to_string(),
                vec![artist.to_string()],
                "Unknown Album".to_string(),
            );
        }
    }

    (
        stem.to_string(),
        vec!["Unknown Artist".to_string()],
        "Unknown Album".to_string(),
    )
}
