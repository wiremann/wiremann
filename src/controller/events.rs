use crate::cacher::ImageKind;
use crate::controller::state::{AppState, PlaybackStatus};
use crate::controller::state::{ImageId, TrackId, TrackSource};
use crate::controller::state::{Playlist, PlaylistId};
use crate::lyrics_manager::Lyrics;
use crate::scanner::ScannedTrack;
use gpui::RenderImage;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone, PartialEq, Debug)]
pub enum AudioEvent {
    TrackLoaded(TrackId, PathBuf),
    Position(Duration),
    PlaybackStatus(PlaybackStatus),
    TrackEnded,
}

#[derive(Clone, PartialEq, Debug)]
pub enum ScannerEvent {
    UpsertTracks(Vec<(ScannedTrack, u64)>),

    ScanStarted(u64),
    Discovered(usize),
    Processed { processed: usize, total: usize },
    ScanFinished(u64),
}

#[derive(Clone, PartialEq, Debug)]
pub enum ImageProcessorEvent {
    InsertAlbumArt(ImageId, Arc<RenderImage>),
    InsertThumbnails(HashMap<ImageId, Arc<RenderImage>>, ImageKind),
    InsertPlaylistThumbnail(PlaylistId, ImageId, Arc<RenderImage>),
    UpdateImageLookup(HashMap<TrackId, ImageId>),
}

#[derive(Clone, PartialEq, Debug)]
pub enum CacherEvent {
    AppState(AppState),

    AlbumArt(Arc<RenderImage>),
    Thumbnails(HashMap<ImageId, Arc<RenderImage>>),
    PlaylistThumbnail(ImageId, Arc<RenderImage>),

    Lyrics(TrackId, Option<Lyrics>),

    MissingThumbnails(Vec<ImageId>),
    MissingAlbumArt(ImageId),
    MissingPlaylistThumbnail(ImageId),

    MissingLyrics(TrackId),
}

#[derive(Clone, PartialEq, Debug)]
pub enum SystemIntegrationEvent {
    Play,
    Pause,
    PlayPause,
    Next,
    Prev,
    Stop,
    SeekForward(Duration),
    SeekBackward(Duration),
    Position(Duration),
    Volume(f64),
}

#[derive(Clone, PartialEq, Debug)]
pub enum LyricsEvent {
    Lyrics(TrackId, Option<Lyrics>),
}
