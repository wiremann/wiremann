pub mod metadata;

use crate::app::AppPaths;
use crate::cacher::CachedTrackSource;
use crate::controller::scan_manager::ScanJob;
use crate::controller::{
    commands::ScannerCommand,
    events::ScannerEvent,
    state::{PlaylistId, TrackId},
};
use crate::errors::ScannerError;

use crossbeam_channel::{Receiver, Sender, select, tick};

use dashmap::DashMap;

use std::io;
use std::{
    collections::HashMap,
    ffi::OsStr,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    time::Duration,
};

use walkdir::WalkDir;

pub struct Scanner {
    pub tx: Sender<ScannerEvent>,
    pub rx: Receiver<ScannerCommand>,

    app_paths: AppPaths,

    scan_progress: Arc<ScanProgress>,
    scan_record: ScanRecord,
}

#[derive(Clone, PartialEq, Debug)]
pub struct ScannedTrack {
    pub source: ScannedTrackSource,

    pub title: String,
    pub artists: Vec<String>,
    pub album: Option<String>,

    pub duration: Duration,

    pub image: Option<Box<[u8]>>,
}

#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub struct ScannedTrackSource {
    pub path: PathBuf,
    pub size: u64,
    pub modified: u64,
}

struct ScanProgress {
    discovery_done: AtomicBool,
    total: AtomicUsize,
    processed: AtomicUsize,
    finished_sent: AtomicBool,
}

type ScanRecord = Arc<DashMap<ScannedTrackSource, TrackId>>;

impl ScannedTrackSource {
    #[allow(clippy::missing_errors_doc)]
    pub fn generate(path: &Path) -> Result<Self, io::Error> {
        let meta = std::fs::metadata(path)?;
        let modified = meta
            .modified()?
            .elapsed()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
            .as_secs();

        let size = meta.len();

        Ok(ScannedTrackSource {
            path: path.to_path_buf(),
            modified,
            size,
        })
    }
}

impl Scanner {
    #[must_use]
    pub fn new(app_paths: AppPaths) -> (Self, Sender<ScannerCommand>, Receiver<ScannerEvent>) {
        let (cmd_tx, cmd_rx) = crossbeam_channel::unbounded();
        let (event_tx, event_rx) = crossbeam_channel::unbounded();

        let scanner = Self {
            tx: event_tx,
            rx: cmd_rx,

            app_paths,

            scan_progress: Arc::new(ScanProgress {
                discovery_done: AtomicBool::new(false),
                total: AtomicUsize::new(0),
                processed: AtomicUsize::new(0),
                finished_sent: AtomicBool::new(false),
            }),

            scan_record: Arc::new(DashMap::new()),
        };

        (scanner, cmd_tx, event_rx)
    }

    pub fn run(&mut self, metadata_workers: usize) -> Result<(), ScannerError> {
        let (worker_tx, worker_rx) = crossbeam_channel::bounded::<(ScanJob, PathBuf)>(64);

        self.spawn_metadata_workers(&worker_rx, metadata_workers);

        loop {
            match self.rx.recv()? {
                ScannerCommand::ScanDir(job) => {
                    self.scan_folder(job, &worker_tx);
                }

                ScannerCommand::ScanTrack(path) => {
                    let job = ScanJob {
                        id: 0,
                        path: path.clone(),
                        playlist_id: None,
                    };

                    worker_tx.send((job, path)).ok();
                }
            }
        }
    }

    fn spawn_metadata_workers(&self, worker_rx: &Receiver<(ScanJob, PathBuf)>, workers: usize) {
        let ticker = tick(Duration::from_millis(128));

        for _ in 0..workers {
            let worker_rx = worker_rx.clone();
            let scan_progress = self.scan_progress.clone();
            let tx = self.tx.clone();
            let scan_record = self.scan_record.clone();
            let ticker = ticker.clone();

            std::thread::spawn(move || {
                let mut new_tracks: Vec<(ScannedTrack, u64)> = Vec::with_capacity(32);

                let mut existing_tracks: HashMap<PlaylistId, Vec<TrackId>> =
                    HashMap::with_capacity(32);

                loop {
                    select! {
                        recv(worker_rx) -> msg => {
                            if let Ok((job, path)) = msg {
                                Self::handle_job(
                                    &job,
                                    path.as_path(),
                                    &scan_record,
                                    &scan_progress,
                                    &tx,
                                    &mut existing_tracks,
                                    &mut new_tracks,
                                );
                            }
                        }

                        recv(ticker) -> _ => {
                            Self::flush_batches(
                                &tx,
                                &mut existing_tracks,
                                &mut new_tracks,
                            );
                        }
                    }
                }
            });
        }
    }

    fn handle_job(
        job: &ScanJob,
        path: &Path,
        scan_record: &ScanRecord,
        scan_progress: &ScanProgress,
        tx: &Sender<ScannerEvent>,
        existing_tracks: &mut HashMap<PlaylistId, Vec<TrackId>>,
        new_tracks: &mut Vec<(ScannedTrack, u64)>,
    ) {
        let Ok(source) = ScannedTrackSource::generate(path) else {
            scan_progress.processed.fetch_add(1, Ordering::Relaxed);
            return;
        };

        if let Some(existing) = scan_record.get(&source) {
            if let Some(pid) = job.playlist_id {
                existing_tracks
                    .entry(pid)
                    .or_default()
                    .push(*existing.value());
            }
        } else {
            if let Ok(track) = metadata::read_metadata(source.clone()) {
                let track_id = TrackId::generate(
                    &track.title,
                    &track.artists.join(", "),
                    track.album.as_deref().unwrap_or(""),
                )
                .unwrap();

                scan_record.insert(source, track_id);

                new_tracks.push((track, job.id));

                if new_tracks.len() >= 32 {
                    let batch = std::mem::take(new_tracks);

                    tx.send(ScannerEvent::UpsertTracks(batch)).ok();
                }
            }
        }

        let processed = scan_progress.processed.fetch_add(1, Ordering::Relaxed) + 1;

        let total = scan_progress.total.load(Ordering::Relaxed);

        if processed.is_multiple_of(16) || processed == total {
            tx.send(ScannerEvent::Processed { processed, total }).ok();
        }

        if processed == total
            && scan_progress.discovery_done.load(Ordering::Acquire)
            && !scan_progress.finished_sent.swap(true, Ordering::AcqRel)
        {
            Self::flush_batches(tx, existing_tracks, new_tracks);

            tx.send(ScannerEvent::ScanFinished(job.id)).ok();
        }
    }

    fn flush_batches(
        tx: &Sender<ScannerEvent>,
        existing_tracks: &mut HashMap<PlaylistId, Vec<TrackId>>,
        new_tracks: &mut Vec<(ScannedTrack, u64)>,
    ) {
        for (playlist_id, tracks) in existing_tracks.iter_mut() {
            if !tracks.is_empty() {
                let batch = std::mem::take(tracks);

                tx.send(ScannerEvent::InsertTracksIntoPlaylist(*playlist_id, batch))
                    .ok();
            }
        }

        if !new_tracks.is_empty() {
            let batch = std::mem::take(new_tracks);

            tx.send(ScannerEvent::UpsertTracks(batch)).ok();
        }
    }

    fn scan_folder(&self, job: ScanJob, worker_tx: &Sender<(ScanJob, PathBuf)>) {
        self.scan_progress.total.store(0, Ordering::Relaxed);

        self.scan_progress.processed.store(0, Ordering::Relaxed);

        self.scan_progress
            .discovery_done
            .store(false, Ordering::Release);

        self.scan_progress
            .finished_sent
            .store(false, Ordering::Release);

        self.read_scan_record();

        self.tx.send(ScannerEvent::ScanStarted(job.id)).ok();

        let exts = ["mp3", "wav", "ogg", "aac", "m4a", "flac"];

        let scan_progress = self.scan_progress.clone();
        let worker_tx = worker_tx.clone();
        let tx = self.tx.clone();

        std::thread::spawn(move || {
            let mut paths = Vec::with_capacity(1024);

            for entry in WalkDir::new(&job.path)
                .into_iter()
                .filter_map(Result::ok)
                .filter(|e| {
                    e.path()
                        .extension()
                        .and_then(OsStr::to_str)
                        .is_some_and(|ext| exts.contains(&ext))
                })
            {
                paths.push(entry.path().to_path_buf());

                if paths.len().is_multiple_of(16) {
                    tx.send(ScannerEvent::Discovered(paths.len())).ok();
                }
            }

            let total = paths.len();

            scan_progress.total.store(total, Ordering::Relaxed);

            scan_progress.discovery_done.store(true, Ordering::Release);

            if total == 0 {
                tx.send(ScannerEvent::ScanFinished(job.id)).ok();
                return;
            }

            for path in paths {
                worker_tx.send((job.clone(), path)).ok();
            }
        });
    }

    fn write_scan_record(&self) {
        let path = self.app_paths.cache.join("scan_record.bin");

        let map: HashMap<CachedTrackSource, [u8; 16]> = self
            .scan_record
            .iter()
            .map(|entry| (entry.key().into(), entry.value().0))
            .collect();

        let bytes = bitcode::encode(&map);

        std::fs::write(path, bytes).ok();
    }

    fn read_scan_record(&self) {
        let path = self.app_paths.cache.join("scan_record.bin");

        let Ok(bytes) = std::fs::read(path) else {
            return;
        };

        let raw: HashMap<CachedTrackSource, [u8; 16]> = bitcode::decode(&bytes).unwrap_or_default();

        self.scan_record.clear();

        for (k, v) in raw {
            self.scan_record.insert((&k).into(), TrackId(v));
        }
    }
}
