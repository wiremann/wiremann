pub mod images;
pub mod io;
pub mod lyrics;
pub mod paths;
pub mod schema;

use crate::app::AppPaths;
use crate::controller::commands::CacherCommand;
use crate::controller::events::CacherEvent;
use crate::controller::state::{
    LibraryState, ListenMetrics, PlaybackState, QueueState, TrackId,
};
use crate::errors::CacherError;
use crossbeam_channel::{Receiver, Sender};

pub use io::CacheJob;
pub use schema::{CachedImage, CachedTrackSource, ImageKind};
use tracing::error;

#[derive(Clone)]
pub struct Cacher {
    pub tx: Sender<CacherEvent>,
    pub rx: Receiver<CacherCommand>,
    app_paths: AppPaths,
}

impl Cacher {
    #[must_use]
    pub fn new(app_paths: AppPaths) -> (Self, Sender<CacherCommand>, Receiver<CacherEvent>) {
        let (cmd_tx, cmd_rx) = crossbeam_channel::unbounded();
        let (event_tx, event_rx) = crossbeam_channel::unbounded();

        let cacher = Cacher {
            tx: event_tx,
            rx: cmd_rx,
            app_paths,
        };

        (cacher, cmd_tx, event_rx)
    }

    pub fn run(&self, workers: usize) -> Result<(), CacherError> {
        let (app_state_tx, app_state_rx) = crossbeam_channel::unbounded();
        let (thumb_tx, thumb_rx) = crossbeam_channel::unbounded();
        let (album_art_tx, album_art_rx) = crossbeam_channel::unbounded();
        let (playlist_thumbnail_tx, playlist_thumbnail_rx) = crossbeam_channel::unbounded();

        self.spawn_app_state_worker(app_state_rx);
        self.spawn_thumbnail_workers(&thumb_rx, workers);
        self.spawn_album_art_worker(album_art_rx);
        self.spawn_playlist_thumbnail_worker(playlist_thumbnail_rx);

        loop {
            match self.rx.recv()? {
                CacherCommand::WriteLibraryState(state) => {
                    let _ = app_state_tx.send(CacheJob::WriteLibraryState(state));
                }
                CacherCommand::WritePlaybackState(state) => {
                    let _ = app_state_tx.send(CacheJob::WritePlaybackState(state));
                }
                CacherCommand::WriteQueueState(state) => {
                    let _ = app_state_tx.send(CacheJob::WriteQueueState(state));
                }
                CacherCommand::WriteFavorites(ids) => {
                    let _ = app_state_tx.send(CacheJob::WriteFavorites(ids));
                }
                CacherCommand::WriteMetrics(metrics) => {
                    let _ = app_state_tx.send(CacheJob::WriteMetrics(metrics));
                }
                CacherCommand::WriteImage {
                    id,
                    kind,
                    width,
                    height,
                    image,
                } => match kind {
                    ImageKind::AlbumArt => {
                        let _ = album_art_tx.send(CacheJob::WriteImage {
                            id,
                            kind: ImageKind::AlbumArt,
                            width,
                            height,
                            image,
                        });
                    }
                    ImageKind::ThumbnailSmall => {
                        let _ = thumb_tx.send(CacheJob::WriteImage {
                            id,
                            kind: ImageKind::ThumbnailSmall,
                            width,
                            height,
                            image,
                        });
                    }
                    ImageKind::ThumbnailLarge => {
                        let _ = thumb_tx.send(CacheJob::WriteImage {
                            id,
                            kind: ImageKind::ThumbnailLarge,
                            width,
                            height,
                            image,
                        });
                    }
                    ImageKind::Playlist => {
                        let _ = playlist_thumbnail_tx.send(CacheJob::WriteImage {
                            id,
                            kind: ImageKind::Playlist,
                            width,
                            height,
                            image,
                        });
                    }
                },
                CacherCommand::GetAppState => {
                    let _ = app_state_tx.send(CacheJob::LoadAppState);
                }
                CacherCommand::GetImage(ids, kind) => match kind {
                    ImageKind::ThumbnailSmall => {
                        let _ =
                            thumb_tx.send(CacheJob::LoadThumbnails(ids, ImageKind::ThumbnailSmall));
                    }
                    ImageKind::ThumbnailLarge => {
                        let _ =
                            thumb_tx.send(CacheJob::LoadThumbnails(ids, ImageKind::ThumbnailLarge));
                    }
                    ImageKind::AlbumArt => {
                        for id in ids {
                            let _ = album_art_tx.send(CacheJob::LoadAlbumArt(id));
                        }
                    }
                    ImageKind::Playlist => {
                        for id in ids {
                            let _ = playlist_thumbnail_tx.send(CacheJob::LoadPlaylistThumbnail(id));
                        }
                    }
                },
                CacherCommand::GetLyrics(id) => {
                    if let Ok(lyrics) = self.read_cached_lyrics(id) {
                        self.tx.send(CacherEvent::Lyrics(id, lyrics)).ok();
                    } else {
                        self.tx.send(CacherEvent::MissingLyrics(id)).ok();
                    }
                }
                CacherCommand::WriteLyrics(id, lyrics) => {
                    if let Err(e) = self.write_cached_lyrics(id, &lyrics) {
                        error!(error = ?e, "Error occured while writing cached lyrics");
                    }
                }
            }
        }
    }

    fn write_library_state(&self, state: &LibraryState) -> Result<(), CacherError> {
        io::write_library_state_to_disk(&self.app_paths.cache, state)
    }

    fn write_playback_state(&self, state: &PlaybackState) -> Result<(), CacherError> {
        io::write_playback_state_to_disk(&self.app_paths.cache, state)
    }

    fn write_queue_state(&self, state: &QueueState) -> Result<(), CacherError> {
        io::write_queue_state_to_disk(&self.app_paths.cache, state)
    }

    fn write_favorites(&self, ids: &[TrackId]) -> Result<(), CacherError> {
        io::write_favorites_to_disk(&self.app_paths.cache, ids)
    }

    fn write_metrics(&self, metrics: &ListenMetrics) -> Result<(), CacherError> {
        io::write_metrics_to_disk(&self.app_paths.cache, metrics)
    }

    fn load_app_state(&self) -> Result<crate::controller::state::AppState, CacherError> {
        io::load_app_state(&self.app_paths.cache)
    }

    #[allow(dead_code)]
    fn read_library_state(&self) -> Result<LibraryState, CacherError> {
        io::read_library_state_from_disk(&self.app_paths.cache)
    }

    #[allow(dead_code)]
    fn read_queue_state(&self) -> Result<QueueState, CacherError> {
        io::read_queue_state_from_disk(&self.app_paths.cache)
    }

    #[allow(dead_code)]
    fn read_playback_state(&self) -> Result<PlaybackState, CacherError> {
        io::read_playback_state_from_disk(&self.app_paths.cache)
    }

    fn spawn_app_state_worker(&self, rx: Receiver<CacheJob>) {
        let cacher = self.clone();

        std::thread::spawn(move || {
            loop {
                while let Ok(job) = rx.recv() {
                    let result: Result<(), CacherError> = (|| {
                        match job {
                            CacheJob::WriteLibraryState(state) => {
                                // Coalesce: a scan can queue many snapshots while it runs;
                                // any later snapshot queued behind this one supersedes it,
                                // so drain them and persist only the final state.
                                let mut state = state;
                                while let Ok(CacheJob::WriteLibraryState(later)) = rx.try_recv() {
                                    state = later;
                                }
                                cacher.write_library_state(&state)?;
                            }
                            CacheJob::WritePlaybackState(state) => {
                                cacher.write_playback_state(&state)?;
                            }
                            CacheJob::WriteQueueState(state) => {
                                cacher.write_queue_state(&state)?;
                            }
                            CacheJob::WriteFavorites(ids) => {
                                cacher.write_favorites(&ids)?;
                            }
                            CacheJob::WriteMetrics(metrics) => {
                                cacher.write_metrics(&metrics)?;
                            }
                            CacheJob::LoadAppState => {
                                let state = cacher.load_app_state()?;
                                let _ = cacher.tx.send(CacherEvent::AppState(state));
                            }
                            _ => {}
                        }

                        Ok(())
                    })();

                    if let Err(err) = result {
                        error!(error = ?err, "Error occurred");
                    }
                }
            }
        });
    }
}
