use crate::errors::ScannerError;
use crate::library::{Track, TrackId, TrackSource};
use std::fs::File;
use std::path::Path;
use std::time::UNIX_EPOCH;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::{MetadataOptions, MetadataRevision, StandardTagKey};
use symphonia::core::probe::Hint;

fn read_full(path: &Path) -> Result<(Track, Option<Box<[u8]>>), ScannerError> {
    let mut hint = Hint::new();

    if let Some(ext) = path.extension().and_then(|this| this.to_str()) {
        hint.with_extension(ext);
    }

    let source = File::open(path)?;
    let file_meta = source.metadata()?;
    let mss = MediaSourceStream::new(Box::new(source), Default::default());
    let mut probed = symphonia::default::get_probe().format(
        &hint,
        mss,
        &FormatOptions::default(),
        &MetadataOptions::default(),
    )?;

    let mut title = None;
    let mut artist = None;
    let mut album = None;
    let mut image: Option<Box<[u8]>> = None;

    if let Some(meta) = probed.metadata.get().as_ref().and_then(|m| m.current()) {
        apply_metadata(meta, &mut title, &mut artist, &mut album, &mut image);
    }

    if let Some(meta) = probed.format.metadata().current() {
        apply_metadata(meta, &mut title, &mut artist, &mut album, &mut image);
    }

    let title = title.unwrap_or_else(|| {
        path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown")
            .to_string()
    });

    let artist = artist.unwrap_or_else(|| "Unknown Artist".to_string());
    let album = album.unwrap_or_else(|| "Unknown Album".to_string());

    let sources = {
        let size = file_meta.len();
        let modified = file_meta.modified()?.duration_since(UNIX_EPOCH)?.as_secs();
        let path = path.to_path_buf();
        vec![TrackSource {
            path,
            size,
            modified,
        }]
    };

    let track = Track {
        id: TrackId::generate(&title, &artist, &album)?,
        sources,

        title,
        artist,
        album,

        duration,
        image_id: None,
    };
}

fn apply_metadata(
    meta: &MetadataRevision,
    title: &mut Option<String>,
    artist: &mut Option<String>,
    album: &mut Option<String>,
    image: &mut Option<Box<[u8]>>,
) {
    for tag in meta.tags() {
        match tag.std_key {
            Some(StandardTagKey::TrackTitle) if title.is_none() => {
                *title = Some(tag.value.to_string());
            }
            Some(StandardTagKey::Artist) if artist.is_none() => {
                *artist = Some(tag.value.to_string());
            }
            Some(StandardTagKey::Album) if album.is_none() => {
                *album = Some(tag.value.to_string());
            }
            _ => {}
        }
    }

    if image.is_none() {
        if let Some(pic) = meta.visuals().first() {
            *image = Some(pic.data.clone());
        }
    }
}
