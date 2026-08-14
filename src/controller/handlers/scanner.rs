use super::{
    App, Arc, CacherCommand, Controller, ControllerError, Entity, HashSet, ImageKind,
    ImageProcessorCommand, Instant, PathBuf, PlaylistId, ScannerCommand, ScannerEvent,
    ScanningStatus, ToastKind, ToastPhase, TrackId, Wiremann,
};
use tracing::{info, trace};

impl Controller {
    pub fn handle_scanner_event(
        &mut self,
        cx: &mut App,
        event: &ScannerEvent,
        view: &Entity<Wiremann>,
    ) -> Result<(), ControllerError> {
        match event {
            ScannerEvent::UpsertTracks(tracks) => {
                let mut modified_playlists = HashSet::new();
                let start = Instant::now();
                let batch_size = tracks.len();

                trace!(thread_id = ?std::thread::current().id(), "controller handling UpsertTracks batch size={}", tracks.len());

                self.state.update(cx, |this, cx| {
                    this.library.tracks.reserve(tracks.len());

                    for (scanned, playlist_id) in tracks {
                        let track_id = match this.library.upsert_scanned_track(scanned) {
                            Ok(id) => id,
                            Err(_) => continue,
                        };

                        if let Some(pid) = playlist_id
                            && let Some(playlist) = this.library.playlists.get_mut(pid)
                        {
                            if !playlist.tracks.contains(&track_id) {
                                playlist.tracks.push(track_id);
                            }

                            modified_playlists.insert(*pid);
                        }
                    }

                    cx.notify();
                });

                info!(
                    batch_size,
                    elapsed_ms = ?start.elapsed().as_millis(),
                    "UpsertTracks batch handled"
                );

                let state = self.state.read(cx).library.clone();
                let _ = self.cacher_tx.send(CacherCommand::WriteLibraryState(state));
            }
            ScannerEvent::InsertTracksIntoPlaylist(pid, tids) => {
                let start = Instant::now();
                trace!(thread_id = ?std::thread::current().id(), "controller handling InsertTracksIntoPlaylist pid={:?} count={}", pid, tids.len());
                self.state.update(cx, |this, cx| {
                    if let Some(playlist) = this.library.playlists.get_mut(pid) {
                        for tid in tids {
                            if !playlist.tracks.contains(tid) {
                                playlist.tracks.push(*tid);
                            }
                        }
                    }
                    cx.notify();
                });
                info!(
                    pid = ?pid,
                    count = tids.len(),
                    elapsed_ms = ?start.elapsed().as_millis(),
                    "InsertTracksIntoPlaylist handled"
                );
                let state = self.state.read(cx).library.clone();
                let _ = self.cacher_tx.send(CacherCommand::WriteLibraryState(state));
            }
            ScannerEvent::AddTrackSource(id, source) => {
                self.state.update(cx, |this, cx| {
                    if let Some(track) = this.library.tracks.get_mut(id) {
                        Arc::make_mut(track).sources.push(source.clone());
                    }

                    cx.notify();
                });
                let state = self.state.read(cx).library.clone();
                let _ = self.cacher_tx.send(CacherCommand::WriteLibraryState(state));
            }
            ScannerEvent::RemoveTrackSource(id, path) => {
                self.state.update(cx, |this, cx| {
                    if let Some(track) = this.library.tracks.get_mut(id)
                        && let Some(source) =
                            track.sources.iter().position(|this| this.path == *path)
                    {
                        Arc::make_mut(track).sources.remove(source);
                    }

                    cx.notify();
                });
                let state = self.state.read(cx).library.clone();
                let _ = self.cacher_tx.send(CacherCommand::WriteLibraryState(state));
            }
            ScannerEvent::InsertPlaylist(playlist) => {
                self.state.update(cx, |this, cx| {
                    this.library.playlists.insert(playlist.id, playlist.clone());

                    cx.notify();
                });

                let state = self.state.read(cx).library.clone();
                let _ = self.cacher_tx.send(CacherCommand::WriteLibraryState(state));
            }
            ScannerEvent::ScanStarted => {
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
                    if !this.is_discovering {
                        this.is_discovering = true;
                    }

                    this.discovered = *discovered;

                    cx.notify();
                });
            }
            ScannerEvent::Processed { processed, total } => {
                let scanning_status = cx.global_mut::<ScanningStatus>().0.clone();

                scanning_status.update(cx, |this, cx| {
                    if this.is_discovering {
                        this.is_discovering = false;
                    }
                    if !this.is_processing {
                        this.is_processing = true;
                    }

                    this.total = *total;
                    this.processed = *processed;
                    cx.notify();
                });
            }
            ScannerEvent::ScanFinished => {
                let start = Instant::now();
                trace!(thread_id = ?std::thread::current().id(), "controller handling ScanFinished");
                self.scanner_tx.send(ScannerCommand::StartNextScan).ok();
                let tracks = self.state.read(cx).library.tracks.clone();

                info!(
                    library_tracks = tracks.len(),
                    "ScanFinished: total tracks in library"
                );

                let to_request: HashSet<(TrackId, PathBuf)> = tracks
                    .iter()
                    .filter(|(_, track)| track.image_id.is_none())
                    .filter_map(|(id, track)| {
                        track
                            .get_valid_source()
                            .map(|src| src.path.clone())
                            .map(|path| (*id, path))
                    })
                    .collect();
                let _ = self
                    .image_processor_tx
                    .send(ImageProcessorCommand::GetThumbnails(
                        to_request,
                        ImageKind::ThumbnailSmall,
                    ));
                trace!(thread_id = ?std::thread::current().id(), elapsed_ms = ?start.elapsed().as_millis(), "controller finished ScanFinished work");

                // Batch fetch online album art for every track that has no image_id.
                // This fills in covers for all songs without requiring them to be played first.
                let state_ref = self.state.read(cx);
                let art_missing = tracks.iter().filter(|(_, t)| t.image_id.is_none()).count();
                info!(
                    tracks_without_art = art_missing,
                    "requesting online album art for tracks missing embedded covers"
                );
                for (id, track) in &tracks {
                    if track.image_id.is_some() {
                        continue;
                    }
                    let mut title = track.title.to_string();
                    let mut artist = track
                        .artists(&state_ref.library)
                        .map(|a| a.name.to_string())
                        .collect::<Vec<_>>()
                        .join(", ");
                    let album = track
                        .album(&state_ref.library)
                        .map(|a| a.name.to_string())
                        .unwrap_or_default();
                    if let Some(idx) = title.find(" - ") {
                        let prefix = title[..idx].trim().to_string();
                        let suffix = title[idx + 3..].trim().to_string();
                        if !prefix.is_empty() && !suffix.is_empty()
                            && (artist.is_empty()
                                || artist.eq_ignore_ascii_case("Unknown Artist")
                                || artist.eq_ignore_ascii_case(&prefix))
                        {
                            artist = prefix;
                            title = suffix;
                        }
                    }
                    let _ = self.image_processor_tx.send(
                        ImageProcessorCommand::FetchAlbumArtOnline {
                            id: *id,
                            title,
                            artist,
                            album,
                        },
                    );
                }

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

                let library = cx.global::<Controller>().state.read(cx).library.clone();
                let missing: Vec<PlaylistId> = library
                    .playlists
                    .iter()
                    .filter_map(|(id, playlist)| {
                        if playlist.image_id.is_none() {
                            Some(*id)
                        } else {
                            None
                        }
                    })
                    .collect();
                self.request_playlist_thumbnails(&missing, cx);
            }
        }
        Ok(())
    }
}
