pub mod audio;
pub mod cacher;
pub mod image_processor;
pub mod lyrics;
pub mod scanner;
pub mod system_integration;

use super::{
    App, Arc, AudioEvent, CacherCommand, CacherEvent, Controller, ControllerError, DominantColors,
    Duration, Entity, HashSet, ImageCache, ImageKind, ImageProcessorCommand, ImageProcessorEvent,
    Instant, LyricsEvent, PathBuf, PlaybackStatus, PlaylistId, Rgb, Rgba, ScannerCommand,
    ScannerEvent, ScanningStatus, SystemIntegrationCommand, SystemIntegrationEvent, ToastKind,
    ToastPhase, TrackId, Wiremann, drop_images_from_app, duration_to_slider,
    pick_playlist_thumbnail_tracks, rgb,
};

pub(super) fn notify_image_views(cx: &mut App, view: &Entity<Wiremann>) {
    view.update(cx, |this, cx| this.notify_image_views(cx));
}
