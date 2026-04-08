use crate::errors::ScannerError;
use crate::library::{Track, TrackId, TrackSource};
use lofty::file::{AudioFile, TaggedFileExt};
use lofty::read_from_path;
use lofty::tag::ItemKey;
use std::fs::File;
use std::path::Path;
use std::time::UNIX_EPOCH;

pub fn read_metadata(track_source: TrackSource) -> Result<Track, ScannerError> {
    let path = track_source.path.as_path();

    let file = read_from_path(path).ok();

    let (mut title, mut artist, mut album) = fallback_metadata(path);
    let mut duration = 0;

    if let Some(tagged_file) = file {
        if let Some(tag) = tagged_file
            .primary_tag()
            .or_else(|| tagged_file.first_tag())
        {
            if let Some(t) = tag.get_string(ItemKey::TrackTitle) {
                title = t.to_string();
            }

            let mut iter = tag.get_strings(ItemKey::TrackArtist);
            artist = match iter.next() {
                None => "Unknown Artist".to_string(),
                Some(first) => {
                    let mut result = first.to_string();
                    for a in iter {
                        result.push_str(", ");
                        result.push_str(a);
                    }
                    result
                }
            };

            if let Some(a) = tag.get_string(ItemKey::AlbumTitle) {
                album = a.to_string();
            }
        }

        duration = tagged_file.properties().duration().as_secs();
    }

    let track_id = TrackId::generate(&title, &artist, &album)?;

    Ok(Track {
        sources: vec![track_source],
        id: track_id,
        title,
        artist,
        album,
        duration,
        image_id: None,
    })
}

pub fn read_album_art(path: &Path) -> Result<Option<Box<[u8]>>, ScannerError> {
    let file = read_from_path(path).ok();

    if let Some(tagged_file) = file
        && let Some(tag) = tagged_file
            .primary_tag()
            .or_else(|| tagged_file.first_tag())
    {
        return Ok(tag.pictures().first().map(|data| Box::from(data.data())));
    } else {
        return Ok(None);
    }
}

fn fallback_metadata(path: &Path) -> (String, String, String) {
    let title = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown")
        .to_string();

    (
        title,
        "Unknown Artist".to_string(),
        "Unknown Album".to_string(),
    )
}
