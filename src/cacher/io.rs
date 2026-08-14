use crate::controller::state::ImageId;
use crate::controller::state::{AppState, LibraryState, PlaybackState, QueueState};
use crate::errors::CacherError;
use bitcode::{Decode, Encode};
use ron::ser::PrettyConfig;
use std::collections::HashSet;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Instant;
use tracing::info;

use super::schema::{
    CacheFile, CachedLibraryState, CachedPlaybackState, CachedQueueState, ImageKind,
};

#[derive(Clone)]
pub enum CacheJob {
    WriteLibraryState(LibraryState),
    WritePlaybackState(PlaybackState),
    WriteQueueState(QueueState),
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

pub fn write_cache<T: Encode>(
    tmp: &PathBuf,
    final_path: &PathBuf,
    payload: T,
) -> Result<(), CacherError> {
    let wrapped = CacheFile {
        version: 1,
        payload,
    };

    let bytes = bitcode::encode(&wrapped);

    {
        let mut file = fs::File::create(tmp)?;
        file.write_all(&bytes)?;
        file.sync_all()?;
    }

    fs::rename(tmp, final_path)?;

    Ok(())
}

pub fn read_cache<T>(path: &PathBuf) -> Result<Option<T>, CacherError>
where
    T: for<'a> Decode<'a>,
{
    if !path.exists() {
        return Ok(None);
    }

    let bytes = fs::read(path)?;

    let file: CacheFile<T> = bitcode::decode(&bytes)?;

    if file.version != 1 {
        return Ok(None);
    }

    Ok(Some(file.payload))
}

pub fn write_library_state_to_disk(
    cache_dir: &Path,
    state: &LibraryState,
) -> Result<(), CacherError> {
    let start = Instant::now();
    let tmp_path = cache_dir.join("library.tmp");
    let final_path = cache_dir.join("library.bin");

    let library = CachedLibraryState::from(state);

    write_cache(&tmp_path, &final_path, library)?;

    info!(
        elapsed_ms = ?start.elapsed().as_millis(),
        tracks = state.tracks.len(),
        "library state written to disk"
    );

    Ok(())
}

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

pub fn write_queue_state_to_disk(cache_dir: &Path, state: &QueueState) -> Result<(), CacherError> {
    let tmp_path = cache_dir.join("queue.tmp");
    let final_path = cache_dir.join("queue.bin");

    let queue = CachedQueueState::from(state);

    write_cache(&tmp_path, &final_path, queue)?;

    Ok(())
}

pub fn read_library_state_from_disk(cache_dir: &Path) -> Result<LibraryState, CacherError> {
    let path = cache_dir.join("library.bin");

    if !path.exists() {
        return Ok(LibraryState::default());
    }

    match read_cache::<CachedLibraryState>(&path)? {
        Some(cached_state) => Ok(LibraryState::from(cached_state)),
        None => Ok(LibraryState::default()),
    }
}

pub fn read_queue_state_from_disk(cache_dir: &Path) -> Result<QueueState, CacherError> {
    let path = cache_dir.join("queue.bin");

    if !path.exists() {
        return Ok(QueueState::default());
    }

    match read_cache::<CachedQueueState>(&path)? {
        Some(cached_state) => Ok(QueueState::from(cached_state)),
        None => Ok(QueueState::default()),
    }
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

pub fn load_app_state(cache_dir: &Path) -> Result<AppState, CacherError> {
    let playback = read_playback_state_from_disk(cache_dir)?;
    let library = read_library_state_from_disk(cache_dir)?;
    let queue = read_queue_state_from_disk(cache_dir)?;

    Ok(AppState {
        playback,
        library,
        queue,
    })
}
