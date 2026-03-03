use crate::cacher::ImageKind;
use crate::controller::state::{LibraryState, PlaybackState, QueueState};
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
    GetAlbumArt(PathBuf),
    GetThumbnails(HashSet<TrackId>),
    WriteLibraryState(LibraryState),
    WritePlaybackState(PlaybackState),
    WriteQueueState(QueueState),
    WriteImage {
        id: TrackId,
        kind: ImageKind,
        width: u32,
        height: u32,
        image: Vec<u8>,
    },
}
