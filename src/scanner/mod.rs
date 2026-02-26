use crate::library::playlists::{Playlist, PlaylistId, PlaylistSource};
use crate::library::{gen_track_id, Track};
use crate::{
    controller::{commands::ScannerCommand, events::ScannerEvent},
    errors::ScannerError,
    library::TrackId,
};
use crossbeam_channel::{select, tick, Receiver, Sender};
use gpui::RenderImage;
use image::imageops::thumbnail;
use image::{Frame, ImageReader};
use lofty::{prelude::*, probe::Probe};
use rayon::prelude::*;
use smallvec::smallvec;
use std::collections::{HashMap, HashSet};
use std::io::Cursor;
use std::sync::Arc;
use std::time::Duration;
use std::{fs, path::PathBuf, time::UNIX_EPOCH};
use uuid::Uuid;
use walkdir::WalkDir;

pub struct Scanner {
    pub tx: Sender<ScannerEvent>,
    pub rx: Receiver<ScannerCommand>,
}

struct ScanResult {
    id: TrackId,
    track: Option<Track>,
    image: Option<Vec<u8>>,
}

enum ScanJob {
    Metadata(PathBuf, TrackId),
    Thumbnail(TrackId, Vec<u8>),
    AlbumArt(PathBuf),
}

impl Scanner {
    pub fn new() -> (Self, Sender<ScannerCommand>, Receiver<ScannerEvent>) {
        let (cmd_tx, cmd_rx) = crossbeam_channel::unbounded();
        let (event_tx, event_rx) = crossbeam_channel::unbounded();

        let scanner = Scanner {
            tx: event_tx,
            rx: cmd_rx,
        };

        (scanner, cmd_tx, event_rx)
    }

    pub fn run(&mut self) -> Result<(), ScannerError> {
        let (meta_tx, meta_rx) = crossbeam_channel::unbounded();
        let (thumb_tx, thumb_rx) = crossbeam_channel::unbounded();
        let (album_art_tx, album_art_rx) = crossbeam_channel::unbounded();

        self.spawn_metadata_worker(meta_rx, thumb_tx.clone())?;
        self.spawn_thumbnail_workers(thumb_rx)?;
        self.spawn_album_art_worker(album_art_rx)?;

        loop {
            match self.rx.recv()? {
                ScannerCommand::GetTrackMetadata { path, track_id } => self.enqueue_track(path, track_id, meta_tx.clone())?,
                ScannerCommand::ScanFolder { tracks, path } => self.enqueue_folder(tracks, path, meta_tx.clone())?,
                ScannerCommand::GetCurrentAlbumArt(path) => { let _ = album_art_tx.send(ScanJob::AlbumArt(path)); }
            }
        }
    }

    fn spawn_metadata_worker(
        &self,
        meta_rx: Receiver<ScanJob>,
        thumb_tx: Sender<ScanJob>,
    ) -> Result<(), ScannerError> {
        let events_tx = self.tx.clone();

        std::thread::spawn(move || {
            let mut batch: Vec<Track> = Vec::with_capacity(16);
            let ticker = tick(Duration::from_millis(128));

            loop {
                select! {
                    recv(meta_rx) -> job => {
                        match job {
                            Ok(ScanJob::Metadata(path, track_id)) => {
                                match get_track_metadata(path, track_id) {
                                    Ok((track, image)) => {
                                        let id = track.id.clone();
                                        batch.push(track);

                                        if batch.len() >= 16 {
                                            let _ = events_tx.send(
                                                ScannerEvent::Tracks(std::mem::take(&mut batch))
                                            );
                                        }

                                        if let Some(bytes) = image {
                                            let _ = thumb_tx.send(ScanJob::Thumbnail(id, bytes));
                                        }
                                    }
                                    Err(err) => eprintln!("Failed to get track metadata: {}", err),
                                }
                            }
                            _ => {}
                        }
                    }

                     recv(ticker) -> _ => {
                        if !batch.is_empty() {
                            let _ = events_tx.send(
                                ScannerEvent::Tracks(std::mem::take(&mut batch))
                            );
                        }
                    }
                }
            }
        });

        Ok(())
    }

    fn spawn_thumbnail_workers(
        &self,
        thumb_rx: Receiver<ScanJob>,
    ) -> Result<(), ScannerError> {
        let ticker = tick(Duration::from_millis(128));
        let threads = num_cpus::get() - 2;

        for _ in 0..threads {
            let events_tx = self.tx.clone();
            let ticker = ticker.clone();
            let thumb_rx = thumb_rx.clone();

            std::thread::spawn(move || {
                let mut batch = HashMap::with_capacity(16);

                loop {
                    select! {
                        recv(thumb_rx) -> job => {
                            match job {
                                Ok(ScanJob::Thumbnail(id, bytes)) => {
                                    if let Ok(image) = render_album_art(&bytes, true) {
                                        batch.insert(id, image);

                                        if batch.len() >= 16 {
                                            let _ = events_tx.send(
                                                ScannerEvent::Thumbnails(std::mem::take(&mut batch))
                                            );
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }

                        recv(ticker) -> _ => {
                            if !batch.is_empty() {
                                let _ = events_tx.send(
                                    ScannerEvent::Thumbnails(std::mem::take(&mut batch))
                                );
                            }
                        }
            }
                }
            });
        }

        Ok(())
    }

    fn spawn_album_art_worker(
        &self,
        album_art_rx: Receiver<ScanJob>,
    ) -> Result<(), ScannerError> {
        let events_tx = self.tx.clone();

        std::thread::spawn(move || {
            while let Ok(ScanJob::AlbumArt(path)) = album_art_rx.recv() {
                match get_album_art(path) {
                    Ok(Some(image)) => {
                        if let Ok(album_art) = render_album_art(&image, false) {
                            let _ = events_tx.send(
                                ScannerEvent::AlbumArt(album_art)
                            );
                        }
                    }
                    Ok(None) => {}
                    Err(err) => eprintln!("Failed album art: {}", err),
                }
            }
        });

        Ok(())
    }

    fn enqueue_track(&self, path: PathBuf, track_id: TrackId, meta_tx: Sender<ScanJob>) -> Result<(), ScannerError> {
        let _ = meta_tx.send(ScanJob::Metadata(path, track_id));
        Ok(())
    }

    fn enqueue_folder(
        &self,
        existing_tracks: HashSet<TrackId>,
        path: PathBuf,
        meta_tx: Sender<ScanJob>,
    ) -> Result<(), ScannerError> {
        let supported = ["mp3", "flac", "wav", "ogg", "m4a"];

        let mut track_ids = Vec::new();

        if path.is_dir() {
            for entry in WalkDir::new(&path)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file())
            {
                let file = entry.path().to_path_buf();

                let ext_ok = file.extension()
                    .and_then(|e| e.to_str())
                    .map(|e| supported.contains(&e.to_lowercase().as_str()))
                    .unwrap_or(false);

                if !ext_ok {
                    continue;
                }

                let id = gen_track_id(&file)?;
                track_ids.push(id.clone());

                if !existing_tracks.contains(&id) {
                    let _ = meta_tx.send(ScanJob::Metadata(file, id));
                }
            }
        }

        let playlist = Playlist {
            id: PlaylistId(Uuid::new_v4()),
            name: path.file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("Unnamed Playlist")
                .to_string(),
            source: PlaylistSource::Folder(path),
            tracks: track_ids,
        };

        let _ = self.tx.send(ScannerEvent::Playlist(playlist));

        Ok(())
    }
}

fn render_album_art(bytes: &[u8], is_thumbnail: bool) -> Result<Arc<RenderImage>, ScannerError> {
    let image = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()?
        .decode()?;

    let mut image = if is_thumbnail {
        thumbnail(&image.into_rgba8(), 64, 64)
    } else {
        image.into_rgba8()
    };

    let buf: &mut [u8] = image.as_mut();

    for px in buf.chunks_exact_mut(4) {
        px.swap(0, 2);
    }

    let frame = Frame::new(image);

    Ok(Arc::new(RenderImage::new(smallvec![frame])))
}

fn get_track_metadata(
    path: PathBuf,
    track_id: TrackId,
) -> Result<(Track, Option<Vec<u8>>), ScannerError> {
    let tagged_file = match Probe::open(path.clone())
        .and_then(|p| Ok(p.guess_file_type()?))
        .and_then(|p| p.read())
    {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Metadata decode failed {:?}: {:?}", path, e);

            let file_metadata = fs::metadata(path.clone())?;
            let duration = 0;
            let size = file_metadata.len();
            let modified = file_metadata
                .modified()?
                .duration_since(UNIX_EPOCH)?
                .as_secs();

            let title = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Unknown")
                .to_string();

            return Ok((
                Track {
                    path,
                    id: track_id,
                    title,
                    artist: "Unknown Artist".to_string(),
                    album: "Unknown Album".to_string(),
                    duration,
                    modified,
                    size,
                },
                None,
            ));
        }
    };

    let file_metadata = fs::metadata(path.clone())?;

    let tag = tagged_file
        .primary_tag()
        .or_else(|| tagged_file.first_tag());

    let title;
    let artist;
    let album;
    let thumbnail;

    if let Some(tag) = tag {
        title = tag
            .get_string(ItemKey::TrackTitle)
            .unwrap_or("Untitled")
            .to_string();

        let artists: Vec<String> = tag
            .get_strings(ItemKey::TrackArtist)
            .map(|s| s.to_owned())
            .collect();

        artist = if artists.is_empty() {
            "Unknown Artist".to_string()
        } else {
            artists.join(", ")
        };

        album = tag
            .get_string(ItemKey::AlbumTitle)
            .unwrap_or("Unknown Album")
            .to_string();

        thumbnail = match tag.pictures().get(0) {
            Some(data) => Some(data.data().to_vec()),
            None => None,
        };
    } else {
        title = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Untitled")
            .to_string();

        artist = "Unknown Artist".to_string();
        album = "Unknown Album".to_string();

        thumbnail = None;
    }

    let duration = tagged_file.properties().duration().as_secs();
    let size = file_metadata.len();
    let modified = file_metadata
        .modified()?
        .duration_since(UNIX_EPOCH)?
        .as_secs();

    Ok((
        Track {
            path,
            id: track_id,
            title,
            artist,
            album,
            duration,
            modified,
            size,
        },
        thumbnail,
    ))
}

fn get_album_art(
    path: PathBuf,
) -> Result<Option<Vec<u8>>, ScannerError> {
    let tagged_file = match Probe::open(path.clone())
        .and_then(|p| Ok(p.guess_file_type()?))
        .and_then(|p| p.read())
    {
        Ok(file) => file,
        Err(e) => return Err(ScannerError::from(e))
    };

    let tag = tagged_file
        .primary_tag()
        .or_else(|| tagged_file.first_tag());

    let thumbnail;

    if let Some(tag) = tag {
        thumbnail = match tag.pictures().get(0) {
            Some(data) => Some(data.data().to_vec()),
            None => None,
        };
    } else {
        thumbnail = None;
    }

    Ok(
        thumbnail,
    )
}