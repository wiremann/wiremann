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
    UpsertTracks(Vec<Track>, Option<PlaylistId>),
    InsertTrackIntoPlaylist(PlaylistId, TrackId),

    AddTrackSource(TrackId, TrackSource),
    RemoveTrackSource(TrackId, PathBuf),

    InsertPlaylist(Playlist),

    InsertAlbumArt(ImageId, Arc<RenderImage>),
    InsertThumbnails(HashMap<ImageId, Arc<RenderImage>>),
    InsertPlaylistThumbnail(PlaylistId, ImageId, Arc<RenderImage>),
    UpdateImageLookup(HashMap<TrackId, ImageId>),
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
