use crate::controller::state::{ImageId, PlaybackState, QueueState};
use crate::controller::state::{LibraryState, ListenMetrics, TrackId};
use crate::errors::CacherError;
use ron::ser::PrettyConfig;
use std::collections::HashSet;
use std::fs;
use std::io::Write;
use std::path::Path;

use super::schema::{CachedPlaybackState, ImageKind};

#[derive(Clone)]
pub enum CacheJob {
    WriteLibraryState(LibraryState),
    WritePlaybackState(PlaybackState),
    WriteQueueState(QueueState),
    WriteFavorites(Vec<TrackId>),
    WriteMetrics(ListenMetrics),
    WriteImage {
        id: ImageId,
        kind: ImageKind,
        width: u32,
        height: u32,
        image: Vec<u8>,
    },
    LoadAppState,
    LoadThumbnails(HashSet<ImageId>, ImageKind),
    LoadAlbumArt(ImageId),
    LoadPlaylistThumbnail(ImageId),
}

/// Writes the small playback session (current track, position, volume, ...) to
/// a RON file. The sizeable, growing state (library, queue, favorites,
/// metrics) lives in the SQLite database instead.
pub fn write_playback_state_to_disk(
    cache_dir: &Path,
    state: &PlaybackState,
) -> Result<(), CacherError> {
    let tmp_path = cache_dir.join("session.tmp");
    let final_path = cache_dir.join("session.ron");

    let payload = CachedPlaybackState::from(state);

    let ron = ron::ser::to_string_pretty(&payload, PrettyConfig::default())?;

    {
        let mut file = fs::File::create(tmp_path.clone())?;
        file.write_all(ron.as_bytes())?;
        file.sync_all()?;
    }

    fs::rename(tmp_path, final_path)?;

    Ok(())
}

pub fn read_playback_state_from_disk(cache_dir: &Path) -> Result<PlaybackState, CacherError> {
    let path = cache_dir.join("session.ron");

    if !path.exists() {
        return Ok(PlaybackState::default());
    }

    let ron = fs::read_to_string(path)?;
    let cached: CachedPlaybackState = ron::de::from_str(&ron)?;

    Ok(cached.into())
}
