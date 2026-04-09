use crate::cacher::ImageKind;
use crate::controller::state::{LibraryState, PlaybackState, QueueState};
use crate::library::playlists::PlaylistId;
use crate::library::{ImageId, TrackId};
use std::collections::HashSet;
use std::path::PathBuf;

pub enum AudioCommand {
    Load(TrackId, PathBuf),
    GetPosition,
    CheckTrackEnded,
    Play,
    Pause,
    Stop,
    SetVolume(f32),
    Seek(u64),
}

pub enum ScannerCommand {
    ScanDir(PathBuf),
    ScanTrack(PathBuf),
    StartNextScan,
}

pub enum ImageProcessorCommand {
    GetThumbnails(HashSet<(TrackId, PathBuf)>, ImageKind),
    GetCurrentAlbumArt(TrackId, PathBuf),
    PlaylistThumbnail {
        id: PlaylistId,
        tracks: Vec<PathBuf>,
    },
    PlaylistJobFinished(PlaylistId),
}

pub enum CacherCommand {
    GetAppState,
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
    GetImage(HashSet<ImageId>, ImageKind),
}
