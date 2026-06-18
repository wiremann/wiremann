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

fn fallback_metadata(_path: &Path) -> (String, Vec<String>, String) {
    let title = _path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown")
        .to_string();

    (
        title,
        vec!["Unknown Artist".to_string()],
        "Unknown Album".to_string(),
    )
}
