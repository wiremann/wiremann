use super::{
    App, Controller, ControllerError, Entity, ImageKind, ImageProcessorCommand, Instant,
    ScannerEvent, ScanningStatus, ToastKind, ToastPhase, Wiremann,
};

use crate::db::Database;
use crate::db::models::InsertedTrack;
use crate::controller::state::{TrackId, ImageId};

impl Controller {
    pub fn handle_scanner_event(
        &mut self,
        cx: &mut App,
        event: &ScannerEvent,
        view: &Entity<Wiremann>,
    ) -> Result<(), ControllerError> {
        match event {
            ScannerEvent::UpsertTracks(tracks, playlist_id) => {
                let db = cx.global::<Database>().clone();
                let tracks = tracks.clone();
                let playlist_id = playlist_id.clone();
                let view = view.clone();

                cx.spawn(async move |app_cx| {
                    let committed = smol::unblock(move || {
                        let mut conn = db.pool().get()?;

                        crate::db::queries::scanner::upsert_scanned_tracks(&mut conn, &tracks, playlist_id)
                    })
                    .await
                    .unwrap_or_else(|_| Vec::new());

                    if !committed.is_empty() {
                        // Convert DB projection to UI rows and update UI on main thread after DB commit
                        view.update(app_cx, move |this, cx| {
                            let mut ui_rows: Vec<crate::ui::pages::library::models::LibraryTrackRow> = Vec::with_capacity(committed.len());

                            for it in committed.into_iter() {
                                if let Ok(arr) = <[u8;16]>::try_from(it.track_hash.clone()) {
                                    let id = TrackId(arr);

                                    let image_id = it.image_hash.as_ref().and_then(|b| ImageId::generate(b).ok());

                                    ui_rows.push(crate::ui::pages::library::models::LibraryTrackRow {
                                        id,
                                        title: it.name.clone(),
                                        artists: it.artists.clone(),
                                        album: it.album.clone().unwrap_or_else(|| "Unknown Album".into()),
                                        duration_ms: it.duration_ms,
                                        image_id,
                                    });
                                }
                            }

                            if !ui_rows.is_empty() {
                                this.library_page.update(cx, |page, cx| {
                                    page.append_committed_rows(ui_rows, cx);
                                });
                            }

                            cx.notify();
                        });
                    }
                })
                .detach();
            }

            ScannerEvent::ScanStarted(_) => {
                let scanning_status = cx.global_mut::<ScanningStatus>().clone().0;

                scanning_status.update(cx, |this, cx| {
                    this.is_scanning = true;
                    this.is_discovering = true;

                    cx.notify();
                });

                view.update(cx, |this, cx| {
                    this.toast_manager.update(cx, |this, cx| {
                        this.info("Scanning started...", cx);
                        this.scanning_status(cx);
                    });

                    cx.notify();
                });
            }

            ScannerEvent::Discovered(discovered) => {
                let scanning_status = cx.global_mut::<ScanningStatus>().0.clone();

                scanning_status.update(cx, |this, cx| {
                    this.is_discovering = true;
                    this.discovered = *discovered;

                    cx.notify();
                });
            }

            ScannerEvent::Processed { processed, total } => {
                let scanning_status = cx.global_mut::<ScanningStatus>().0.clone();

                scanning_status.update(cx, |this, cx| {
                    this.is_discovering = false;
                    this.is_processing = true;

                    this.total = *total;
                    this.processed = *processed;

                    cx.notify();
                });
            }

            ScannerEvent::ScanFinished(_) => {
                self.start_next_scan();

                view.update(cx, |this, cx| {
                    this.toast_manager.update(cx, |this, cx| {
                        this.toasts.update(cx, |list, _| {
                            for t in list.iter_mut() {
                                if matches!(t.kind, ToastKind::ScanProgress(_))
                                    && t.phase != ToastPhase::Exiting
                                {
                                    t.phase = ToastPhase::Exiting;
                                    t.exiting_at = Some(Instant::now());
                                }
                            }
                        });

                        this.success("Scan complete!", cx);
                    });
                });

                let status = cx.global::<ScanningStatus>().0.clone();

                status.update(cx, |s, _| {
                    s.is_scanning = false;
                    s.is_discovering = false;
                    s.is_processing = false;

                    s.discovered = 0;
                    s.total = 0;
                    s.processed = 0;
                });

                let db = cx.global::<Database>().clone();
                let image_tx = self.image_processor_tx.clone();

                cx.spawn(async move |_cx| {
                    let missing = smol::unblock(move || {
                        let conn = db.pool().get()?;

                        crate::db::queries::images::get_tracks_missing_thumbnails(&conn)
                    })
                    .await
                    .unwrap();

                    image_tx
                        .send(ImageProcessorCommand::GetThumbnails(
                            missing,
                            ImageKind::ThumbnailSmall,
                        ))
                        .ok();
                })
                .detach();

                let db = cx.global::<Database>().clone();
                let image_tx = self.image_processor_tx.clone();

                cx.spawn(async move |_cx| {
                    let playlists = smol::unblock(move || {
                        let conn = db.pool().get()?;

                        crate::db::queries::images::get_playlist_thumbnail_jobs(&conn)
                    })
                    .await
                    .unwrap();

                    for (playlist_id, paths) in playlists {
                        image_tx
                            .send(ImageProcessorCommand::PlaylistThumbnail {
                                id: playlist_id,
                                tracks: paths,
                            })
                            .ok();
                    }
                })
                .detach();
            }
        }

        Ok(())
    }
}
