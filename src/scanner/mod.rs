pub mod metadata;
use crate::app::AppPaths;
use crate::cacher::CachedTrackSource;
use crate::library::playlists::{Playlist, PlaylistId, PlaylistSource};
use crate::library::{Track, TrackSource};
use crate::{
    controller::{commands::ScannerCommand, events::ScannerEvent},
    errors::ScannerError,
    library::TrackId,
};
use crossbeam_channel::{Receiver, Sender, select, tick};
use dashmap::DashMap;
use std::cmp::PartialEq;
use std::collections::{HashMap, VecDeque};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Duration;
use uuid::Uuid;
use walkdir::WalkDir;

pub struct Scanner {
    pub tx: Sender<ScannerEvent>,
    pub rx: Receiver<ScannerCommand>,

    state: State,
    queue: VecDeque<PathBuf>,

    app_paths: AppPaths,

    scan_progress: Arc<ScanProgress>,
    scan_record: ScanRecord,
}

#[derive(PartialEq)]
enum State {
    Idle,
    Scanning,
}

struct ScanProgress {
    discovery_done: AtomicBool,
    total: AtomicUsize,
    processed: AtomicUsize,
}

type ScanRecord = Arc<DashMap<TrackSource, TrackId>>;

impl Scanner {
    #[must_use]
    pub fn new(app_paths: AppPaths) -> (Self, Sender<ScannerCommand>, Receiver<ScannerEvent>) {
        let (cmd_tx, cmd_rx) = crossbeam_channel::unbounded();
        let (event_tx, event_rx) = crossbeam_channel::unbounded();

        let scanner = Scanner {
            tx: event_tx,
            rx: cmd_rx,

            state: State::Idle,
            queue: VecDeque::new(),

            app_paths,

            scan_progress: Arc::new(ScanProgress {
                discovery_done: AtomicBool::new(false),
                total: AtomicUsize::new(0),
                processed: AtomicUsize::new(0),
            }),
            scan_record: Arc::new(DashMap::new()),
        };

        (scanner, cmd_tx, event_rx)
    }

    #[allow(clippy::missing_errors_doc)]
    pub fn run(&mut self, metadata_workers: usize) -> Result<(), ScannerError> {
        let (worker_tx, worker_rx) = crossbeam_channel::bounded(64);

        self.spawn_metadata_workers(&worker_rx, metadata_workers);

        loop {
            match self.rx.recv()? {
                ScannerCommand::ScanDir(path) => {
                    self.queue.push_back(path);

                    if self.state == State::Idle
                        && let Some(path) = self.queue.pop_front()
                    {
                        self.state = State::Scanning;
                        self.scan_folder(path, &worker_tx);
                    }
                }
                ScannerCommand::StartNextScan => {
                    self.state = State::Idle;
                    self.write_scan_record();

                    if self.state == State::Idle
                        && let Some(path) = self.queue.pop_front()
                    {
                        self.state = State::Scanning;
                        self.scan_folder(path, &worker_tx);
                    }
                }
                ScannerCommand::ScanTrack(path) => {
                    worker_tx.send((path, None)).ok();
                }
            }
        }
    }

    fn spawn_metadata_workers(
        &self,
        worker_rx: &Receiver<(PathBuf, Option<PlaylistId>)>,
        workers: usize,
    ) {
        let ticker = tick(Duration::from_millis(128));

        for _ in 0..workers {
            let worker_rx = worker_rx.clone();
            let scan_progress = self.scan_progress.clone();
            let tx = self.tx.clone();
            let scan_record = self.scan_record.clone();
            let ticker = ticker.clone();

            std::thread::spawn(move || {
                let mut new: Vec<(Track, Option<PlaylistId>)> = Vec::with_capacity(32);
                let mut existing: HashMap<PlaylistId, Vec<TrackId>> = HashMap::with_capacity(32);

                loop {
                    select! {
                        recv(worker_rx) -> job => {
                            if let Ok((path, pid)) = job {
                                Self::handle_job(
                                    path.as_path(),
                                    pid,
                                    &scan_record,
                                    &scan_progress,
                                    &tx,
                                    &mut existing,
                                    &mut new,
                                );
                            }
                        }

                        recv(ticker) -> _ => {
                            Self::flush_batches(&tx, &mut existing, &mut new);
                        }
                    }
                }
            });
        }
    }

    fn handle_job(
        path: &Path,
        pid: Option<PlaylistId>,
        scan_record: &ScanRecord,
        scan_progress: &ScanProgress,
        tx: &Sender<ScannerEvent>,
        existing: &mut HashMap<PlaylistId, Vec<TrackId>>,
        new: &mut Vec<(Track, Option<PlaylistId>)>,
    ) {
        let mut incremented = false;

        let Ok(ts) = TrackSource::generate(path) else {
            scan_progress.processed.fetch_add(1, Ordering::Relaxed);
            return;
        };

        if let Some(entry) = scan_record.get(&ts) {
            if let Some(pid) = pid {
                let batch = existing.entry(pid).or_default();
                batch.push(*entry.value());

                if batch.len() >= 32 {
                    let to_send = std::mem::take(batch);
                    tx.send(ScannerEvent::InsertTracksIntoPlaylist(pid, to_send))
                        .ok();
                }

                scan_progress.processed.fetch_add(1, Ordering::Relaxed);
                incremented = true;
            }
        } else {
            if let Ok(track) = metadata::read_metadata(ts.clone()) {
                let id = track.id;
                new.push((track, pid));

                if new.len() >= 32 {
                    let to_send = std::mem::take(new);
                    tx.send(ScannerEvent::UpsertTracks(to_send)).ok();
                }

                scan_record.insert(ts, id);
            }

            scan_progress.processed.fetch_add(1, Ordering::Relaxed);
            incremented = true;
        }

        let processed = scan_progress.processed.load(Ordering::Relaxed);
        let total = scan_progress.total.load(Ordering::Relaxed);

        if incremented && (processed.is_multiple_of(16) || processed == total) {
            tx.send(ScannerEvent::Processed { processed, total }).ok();
        }
        if processed == total && scan_progress.discovery_done.load(Ordering::Acquire) {
            tx.send(ScannerEvent::ScanFinished).ok();
        }
    }

    fn flush_batches(
        tx: &Sender<ScannerEvent>,
        existing: &mut HashMap<PlaylistId, Vec<TrackId>>,
        new: &mut Vec<(Track, Option<PlaylistId>)>,
    ) {
        for (pid, batch) in existing.iter_mut() {
            if !batch.is_empty() {
                let to_send = std::mem::take(batch);

                tx.send(ScannerEvent::InsertTracksIntoPlaylist(*pid, to_send))
                    .ok();
            }
        }

        if !new.is_empty() {
            let to_send = std::mem::take(new);

            tx.send(ScannerEvent::UpsertTracks(to_send)).ok();
        }
    }

    fn scan_folder(&self, path: PathBuf, worker_tx: &Sender<(PathBuf, Option<PlaylistId>)>) {
        self.scan_progress.total.store(0, Ordering::Relaxed);
        self.scan_progress.processed.store(0, Ordering::Relaxed);
        self.scan_progress
            .discovery_done
            .store(false, Ordering::Release);

        self.read_scan_record();

        self.tx.send(ScannerEvent::ScanStarted).ok();

        let exts = ["mp3", "wav", "ogg", "aac", "m4a"];

        if path.is_dir() {
            let playlist_id = PlaylistId(Uuid::new_v4());

            let playlist = Playlist {
                id: playlist_id,
                name: path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Unnamed Playlist")
                    .to_string(),
                source: PlaylistSource::Folder,
                folder_path: Some(path.clone()),
                tracks: Vec::new(),
                duration: Duration::from_secs(0),
                image_id: None,
            };

            let _ = self.tx.send(ScannerEvent::InsertPlaylist(playlist));

            let scan_progress = self.scan_progress.clone();
            let worker_tx = worker_tx.clone();
            let tx = self.tx.clone();

            std::thread::spawn(move || {
                let mut paths = Vec::with_capacity(1024);

                for entry in WalkDir::new(&path)
                    .into_iter()
                    .filter_map(Result::ok)
                    .filter(|e| {
                        e.path()
                            .extension()
                            .and_then(OsStr::to_str)
                            .is_some_and(|ext| exts.contains(&ext))
                    })
                {
                    if paths.len() % 16 == 0 {
                        tx.send(ScannerEvent::Discovered(paths.len())).ok();
                    }
                    paths.push(entry.path().to_path_buf());
                }

                let total = paths.len();
                scan_progress.total.store(total, Ordering::Relaxed);

                scan_progress.discovery_done.store(true, Ordering::Release);

                for path in paths {
                    let _ = worker_tx.send((path, Some(playlist_id)));
                }
            });
        }

        self.scan_progress
            .discovery_done
            .store(true, Ordering::Release);
    }

    fn write_scan_record(&self) {
        let path = self.app_paths.cache.join("scan_record.bin");

        let map: HashMap<CachedTrackSource, [u8; 16]> = self
            .scan_record
            .iter()
            .map(|entry| (entry.key().into(), entry.value().0))
            .collect();

        let bytes = bitcode::encode(&map);
        std::fs::write(path, bytes).unwrap();
    }

    fn read_scan_record(&self) {
        let path = self.app_paths.cache.join("scan_record.bin");

        let file = std::fs::read(path).ok();

        if let Some(bytes) = file {
            let raw: HashMap<CachedTrackSource, [u8; 16]> =
                bitcode::decode(&bytes).unwrap_or_default();

            let map: HashMap<TrackSource, TrackId> =
                raw.iter().map(|(k, v)| (k.into(), TrackId(*v))).collect();

            self.scan_record.clear();

            for (k, v) in map {
                self.scan_record.insert(k, v);
            }
        }
    }
}
