use crate::cacher::ImageKind;
use crate::controller::state::AppState;
use crate::library::TrackId;
use std::collections::HashSet;
use std::path::PathBuf;

pub enum AudioCommand {
    Load(PathBuf),
    GetPosition,
    CheckTrackEnded,
    Play,
    Pause,
    Stop,
    SetVolume(f32),
    Seek(u64),
}

pub enum ScannerCommand {
    GetTrackMetadata {
        path: PathBuf,
        track_id: TrackId,
    },
    ScanFolder {
        path: PathBuf,
        tracks: HashSet<TrackId>,
    },
    GetCurrentAlbumArt(PathBuf),
}

pub enum CacherCommand {
    GetAppState,
    GetAlbumArt(TrackId),
    GetThumbnails(HashSet<TrackId>),
    WriteAppState(AppState),
    WriteImage {
        id: TrackId,
        kind: ImageKind,
        width: u32,
        height: u32,
        image: Vec<u8>,
    },
}