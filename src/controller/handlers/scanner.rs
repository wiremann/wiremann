use super::{Controller, App, ScannerEvent, Entity, Wiremann, ControllerError, HashSet, Arc, CacherCommand, ScanningStatus, ScannerCommand, TrackId, PathBuf, ImageProcessorCommand, ImageKind, ToastKind, ToastPhase, Instant, PlaylistId};

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
                self.state.update(cx, |this, cx| {
                    this.library.tracks.reserve(tracks.len());
                    for (track, playlist_id) in tracks {
                        let id = track.id;

                        if let Some(existing) = this.library.tracks.get_mut(&id) {
                            let existing = Arc::make_mut(existing);

                            for src in &track.sources {
                                if !existing.sources.iter().any(|s| s.path == src.path) {
                                    existing.sources.push(src.clone());
                                }
                            }

                            if existing.title.is_empty() && !track.title.is_empty() {
                                existing.title.clone_from(&track.title);
                            }

                            if existing.artist.is_empty() && !track.artist.is_empty() {
                                existing.artist.clone_from(&track.artist);
                            }

                            if existing.album.is_empty() && !track.album.is_empty() {
                                existing.album.clone_from(&track.album);
                            }
                        } else {
                            this.library.tracks.insert(id, Arc::new(track.clone()));
                        }

                        if let Some(pid) = playlist_id
                            && let Some(playlist) = this.library.playlists.get_mut(pid)
                        {
                            if !playlist.tracks.contains(&id) {
                                playlist.tracks.push(id);
                            }
                            modified_playlists.insert(*pid);
                        }
                    }
                    cx.notify();
                });
                let state = self.state.read(cx).library.clone();
                let _ = self.cacher_tx.send(CacherCommand::WriteLibraryState(state));
            }
            ScannerEvent::InsertTracksIntoPlaylist(pid, tids) => {
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
                self.scanner_tx.send(ScannerCommand::StartNextScan).ok();
                let tracks = self.state.read(cx).library.tracks.clone();

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
