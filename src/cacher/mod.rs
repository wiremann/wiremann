pub mod db;
pub mod images;
pub mod io;
pub mod legacy;
pub mod lyrics;
pub mod paths;
pub mod schema;

use crate::app::AppPaths;
use crate::controller::commands::CacherCommand;
use crate::controller::events::CacherEvent;
use crate::errors::CacherError;
use crossbeam_channel::{Receiver, Sender};

pub use db::Db;
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

    fn spawn_app_state_worker(&self, rx: Receiver<CacheJob>) {
        let cacher = self.clone();

        std::thread::spawn(move || {
            // SQLx is async-first; run a small current-thread runtime inside
            // this worker so every job can await the database directly.
            let rt = match tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                Ok(rt) => rt,
                Err(e) => {
                    error!(error = ?e, "failed to start SQLite runtime");
                    return;
                }
            };

            let db = match rt.block_on(Db::connect(&cacher.app_paths.cache)) {
                Ok(db) => db,
                Err(e) => {
                    error!(error = ?e, "failed to open database");
                    return;
                }
            };

            loop {
                while let Ok(job) = rx.recv() {
                    let result: Result<(), CacherError> = match job {
                        CacheJob::WriteLibraryState(state) => {
                            // Coalesce: a scan can queue many snapshots while it runs;
                            // any later snapshot queued behind this one supersedes it,
                            // so drain them and persist only the final state.
                            let mut state = state;
                            while let Ok(CacheJob::WriteLibraryState(later)) = rx.try_recv() {
                                state = later;
                            }
                            rt.block_on(db.write_library(&state)).map_err(Into::into)
                        }
                        CacheJob::WriteQueueState(state) => {
                            rt.block_on(db.write_queue(&state)).map_err(Into::into)
                        }
                        CacheJob::WriteFavorites(ids) => {
                            rt.block_on(db.write_favorites(&ids)).map_err(Into::into)
                        }
                        CacheJob::WriteMetrics(metrics) => {
                            rt.block_on(db.write_metrics(&metrics)).map_err(Into::into)
                        }
                        CacheJob::WritePlaybackState(state) => {
                            io::write_playback_state_to_disk(&cacher.app_paths.cache, &state)
                        }
                        CacheJob::LoadAppState => {
                            let state = rt.block_on(db.load_app_state(&cacher.app_paths.cache));
                            match state {
                                Ok(state) => {
                                    let _ = cacher.tx.send(CacherEvent::AppState(state));
                                    Ok(())
                                }
                                Err(e) => Err(e),
                            }
                        }
                        _ => Ok(()),
                    };

                    if let Err(err) = result {
                        error!(error = ?err, "Error occurred");
                    }
                }
            }
        });
    }
}
