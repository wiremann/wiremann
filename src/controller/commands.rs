use crate::cacher::ImageKind;
use crate::controller::state::{LibraryState, PlaybackState, PlaybackStatus, QueueState};
use crate::library::playlists::PlaylistId;
use crate::library::{ImageId, TrackId};
use crate::lyrics_manager::Lyrics;
use std::collections::HashSet;
use std::path::PathBuf;
use std::time::Duration;

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

    GetImage(HashSet<ImageId>, ImageKind),
    WriteImage {
        id: ImageId,
        kind: ImageKind,
        width: u32,
        height: u32,
        image: Vec<u8>,
    },

    GetLyrics(TrackId),
    WriteLyrics(TrackId, Lyrics),
}

pub enum SystemIntegrationCommand {
    SetMetadata {
        title: String,
        artist: String,
        album: String,
        image: Option<(u32, u32, Vec<u8>)>,
        duration: u64,
    },
    SetPosition(u64),
    SetPlaybackStatus(PlaybackStatus, u64),
}

pub enum LyricsCommand {
    GetLyrics {
        id: TrackId,
        title: String,
        artist: String,
        album: String,
        duration: Duration,
    },
}
