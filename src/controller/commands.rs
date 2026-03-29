use crate::cacher::ImageKind;
use crate::controller::state::{LibraryState, PlaybackState, QueueState};
use crate::library::playlists::PlaylistId;
use crate::library::{ImageId, Track, TrackId};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;

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
    GetTrackMetadata {
        path: PathBuf,
        track_id: TrackId,
    },
    GetThumbnails(HashSet<(TrackId, PathBuf)>, ImageKind),

    ScanTrack(PathBuf),
    ScanFolder {
        path: PathBuf,
        tracks: HashMap<TrackId, Arc<Track>>,
    },
    GetCurrentAlbumArt(TrackId, PathBuf),
    PlaylistThumbnail {
        id: PlaylistId,
        tracks: Vec<PathBuf>,
    },
    MetaJobFinished(TrackId),
    PlaylistThumbnailJobFinished(PlaylistId),
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
