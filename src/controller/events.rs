use crate::controller::state::{AppState, PlaybackStatus};
use crate::library::playlists::{Playlist, PlaylistId};
use crate::library::{ImageId, Track, TrackId, TrackSource};
use gpui::RenderImage;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Clone, PartialEq, Debug)]
pub enum AudioEvent {
    TrackLoaded(TrackId, PathBuf),
    Position(u64),
    PlaybackStatus(PlaybackStatus),
    TrackEnded,
}

#[derive(Clone, PartialEq, Debug)]
pub enum ScannerEvent {
    InsertTracks(Vec<Track>),
    AddTrackSource(TrackId, TrackSource),
    RemoveTrackSource(TrackId, PathBuf),
    InsertTrackIntoPlaylist(TrackId),
    Playlist(Playlist),
    AlbumArt(ImageId, Arc<RenderImage>),
    Thumbnails(HashMap<ImageId, Arc<RenderImage>>),
    ImageLookup(HashMap<TrackId, ImageId>),
    PlaylistThumbnail(PlaylistId, ImageId, Arc<RenderImage>),
}

#[derive(Clone, PartialEq, Debug)]
pub enum CacherEvent {
    AppState(AppState),
    AlbumArt(Arc<RenderImage>),
    Thumbnails(HashMap<ImageId, Arc<RenderImage>>),
    PlaylistThumbnail(ImageId, Arc<RenderImage>),
    MissingThumbnails(Vec<ImageId>),
    MissingAlbumArt(ImageId),
    MissingPlaylistThumbnail(ImageId),
}
