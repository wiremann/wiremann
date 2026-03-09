use crate::controller::state::{AppState, PlaybackStatus};
use crate::library::playlists::{Playlist, PlaylistId};
use crate::library::{ImageId, Track, TrackId};
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
    AlbumArt(ImageId, Arc<RenderImage>),
    Thumbnails(HashMap<ImageId, Arc<RenderImage>>),
    ImageLookup(HashMap<TrackId, ImageId>),
    PlaylistThumbnail(PlaylistId, Arc<RenderImage>),
}

#[derive(Clone, PartialEq, Debug)]
pub enum CacherEvent {
    AppState(AppState),
    AlbumArt(Arc<RenderImage>),
    Thumbnails(HashMap<ImageId, Arc<RenderImage>>),
    MissingThumbnails(Vec<ImageId>),
    MissingAlbumArt(ImageId),
}
