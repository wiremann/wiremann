use crate::controller::state::{AppState, PlaybackStatus};
use crate::library::playlists::Playlist;
use crate::library::{Track, TrackId};
use gpui::RenderImage;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Clone, PartialEq, Debug)]
pub enum AudioEvent {
    TrackLoaded(PathBuf),
    Position(u64),
    PlaybackStatus(PlaybackStatus),
    TrackEnded,
}

#[derive(Clone, PartialEq, Debug)]
pub enum ScannerEvent {
    Tracks(Vec<Track>),
    Playlist(Playlist),
    AlbumArt(TrackId, Arc<RenderImage>),
    Thumbnails(HashMap<TrackId, Arc<RenderImage>>),
    ScanFinished,
}

#[derive(Clone, PartialEq, Debug)]
pub enum CacherEvent {
    AppState(AppState),
    AlbumArt(Arc<RenderImage>),
    Thumbnails(HashMap<TrackId, Arc<RenderImage>>),
    MissingThumbnails(Vec<TrackId>),
    MissingAlbumArt(PathBuf),
}
