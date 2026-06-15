use super::{
    App, Controller, ControllerError, Entity, ImageKind, ImageProcessorCommand, Instant,
    ScannerEvent, ScanningStatus, ToastKind, ToastPhase, Wiremann,
};

use crate::controller::state::{ImageId, TrackId};
use crate::db::Database;
use crate::db::models::InsertedTrack;

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
                    let db_for_commit = db.clone();

                    // Clone tracks for DB commit so we don't move the original `tracks`
                    // which we need later for mapping sources into runtime state.
                    let tracks_db = tracks.clone();

                    let committed = smol::unblock(move || {
                            let mut conn = db_for_commit.pool().get()?;

                            crate::db::queries::scanner::upsert_scanned_tracks(&mut conn, &tracks_db, playlist_id)
                        })
                    .await
                    .unwrap_or_else(|_| Vec::new());

                    if !committed.is_empty() {
                        // Build a mapping from scanned metadata to source info so we can
                        // populate runtime `Track` sources for immediate thumbnail work.
                        use std::collections::HashMap;
                        use crate::scanner::ScannedTrack as ScannerScannedTrack;

                        let mut hash_to_source: HashMap<Vec<u8>, crate::scanner::ScannedTrackSource> = HashMap::new();
                        for st in tracks.iter() {
                            if let Ok(id) = crate::controller::state::TrackId::generate(
                                &st.title,
                                &st.artists.join(", "),
                                st.album.as_deref().unwrap_or(""),
                            ) {
                                hash_to_source.insert(id.0.to_vec(), st.source.clone());
                            }
                        }
                        // Convert DB projection to UI rows and update UI on main thread after DB commit
                        view.update(app_cx, move |this, cx| {
                            let mut ui_rows: Vec<crate::ui::pages::library::models::LibraryTrackRow> = Vec::with_capacity(committed.len());
                            let mut inserted_ids: Vec<TrackId> = Vec::with_capacity(committed.len());

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

                                    inserted_ids.push(id);
                                    // capture source if available
                                    let source_opt = hash_to_source.get(&it.track_hash).cloned();
                                    let _source = source_opt;
                                }
                            }

                            if !ui_rows.is_empty() {
                                // Insert Track objects into controller state so thumbnails and other
                                // track-based operations can run immediately without requiring a
                                // restart to load DB-backed state. Populate `sources` from the
                                // scanned metadata we captured above when available.
                                let controller = cx.global::<Controller>().clone();
                                controller.state.update(cx, |state, _| {
                                    for it in ui_rows.iter() {
                                        use crate::controller::state::{Track, TrackSource};
                                        let id = it.id;
                                        if !state.library.tracks.contains_key(&id) {
                                            let mut sources: Vec<TrackSource> = Vec::new();
                                            if let Some(src) = hash_to_source.get(&it.id.0.to_vec()) {
                                                sources.push(TrackSource {
                                                    path: src.path.clone(),
                                                    size: src.size,
                                                    modified: src.modified,
                                                });
                                            }

                                            let track = Track {
                                                id,
                                                sources,
                                                title: it.title.clone(),
                                                artist: it.artists.clone(),
                                                album: it.album.clone(),
                                                duration: std::time::Duration::from_millis(it.duration_ms as u64),
                                                image_id: it.image_id,
                                            };

                                            state.library.tracks.insert(id, std::sync::Arc::new(track));
                                        }
                                    }
                                    // notify will be called below
                                });

                                // Append rows into the library UI
                                this.library_page.update(cx, |page, cx| {
                                    page.append_committed_rows(ui_rows, cx);
                                });

                                // Request thumbnails for newly-inserted tracks (state now contains tracks)
                                let controller = cx.global::<Controller>().clone();
                                controller.request_track_thumbnails(&inserted_ids, cx);

                                // Do NOT mutate playlist canonical state here — we'll refresh from DB below.
                            }

                            cx.notify();
                        });

                        // If these tracks were inserted into a playlist, refresh derived playlist snapshot from DB
                        if let Some(pid) = playlist_id.clone() {
                            let db2 = db.clone();
                            let pid2 = pid;

                            let app_cx2 = app_cx.clone();
                            app_cx.spawn(async move |_cx| {
                                let proj = smol::unblock(move || {
                                    let conn = db2.pool().get()?;
                                    let pls = crate::db::queries::playlists::load_playlists_with_tracks(&conn)?;
                                    Ok::<_, anyhow::Error>(pls.into_iter().find(|p| p.id == pid2))
                                })
                                .await
                                .ok();

                                if let Some(Some(p)) = proj {
                                    app_cx2.update(|app| {
                                        let controller = app.global::<Controller>().clone();
                                        controller.state.update(app, |state, cx| {
                                            state.library.playlists.insert(
                                                p.id,
                                                crate::controller::state::Playlist {
                                                    id: p.id,
                                                    name: p.name,
                                                    source: p.source,
                                                    folder_path: None,
                                                    duration: std::time::Duration::from_secs(0),
                                                    tracks: p.tracks,
                                                    image_id: p.image_id,
                                                },
                                            );
                                            cx.notify();
                                        });
                                    });
                                }
                            }).detach();
                        }
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
