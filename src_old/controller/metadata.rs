use anyhow::Result;
use lofty::{prelude::*, probe::Probe};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Default, PartialEq, Clone, Deserialize, Serialize)]
pub struct Metadata {
    pub title: String,
    pub artists: Vec<String>,
    pub album: String,
    pub genre: String,
    pub duration: u64,
    pub writer: String,
    pub producer: String,
    pub publisher: String,
    pub label: String,
    pub thumbnail: Option<Vec<u8>>,
}

#[derive(Debug, Default, PartialEq, Clone, Deserialize, Serialize)]
pub struct Thumbnail {
    pub image: Vec<u8>,
    pub format: String,
}

impl Metadata {
    pub fn read(path: PathBuf) -> Result<Self> {
        let tagged_file = Probe::open(path.clone())?.guess_file_type()?.read()?;

        let tag = match tagged_file.primary_tag() {
            Some(primary_tag) => primary_tag,
            None => tagged_file
                .first_tag()
                .expect("ERROR: could not find any tags!"),
        };

        let thumbnail = match tag.pictures().get(0) {
            Some(data) => Some(data.data().to_vec()),
            None => None,
        };

        let title = tag
            .get_string(ItemKey::TrackTitle)
            .unwrap_or("None")
            .to_string();
        let artists: Vec<String> = tag
            .get_strings(ItemKey::TrackArtist)
            .map(|s| s.to_owned())
            .collect();
        let album = tag
            .get_string(ItemKey::AlbumTitle)
            .unwrap_or("None")
            .to_string();
        let genre = tag.get_string(ItemKey::Genre).unwrap_or("None").to_string();
        let duration = tagged_file.properties().duration().as_secs();
        let writer = tag
            .get_string(ItemKey::Writer)
            .or_else(|| tag.get_string(ItemKey::Composer))
            .unwrap_or("None")
            .to_string();
        let producer = tag
            .get_string(ItemKey::Producer)
            .unwrap_or("None")
            .to_string();
        let publisher = tag
            .get_string(ItemKey::Publisher)
            .unwrap_or("None")
            .to_string();
        let label = tag.get_string(ItemKey::Label).unwrap_or("None").to_string();

        Ok(Metadata {
            title,
            artists,
            album,
            genre,
            duration,
            writer,
            producer,
            publisher,
            label,
            thumbnail,
        })
    }
}
