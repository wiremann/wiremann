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
