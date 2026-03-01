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
    GetThumbnail(TrackId),
    GetAlbumArt(TrackId),
    WriteAppState(AppState),
    WriteThumbnail {
        id: TrackId,
        image: Vec<u8>,
    },
    WriteAlbumArt {
        id: TrackId,
        image: Vec<u8>,
    },
}