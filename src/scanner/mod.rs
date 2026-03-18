use crate::cacher::ImageKind;
use crate::library::playlists::{Playlist, PlaylistId, PlaylistSource};
use crate::library::{gen_track_id, ImageId, Track};
use crate::{
    controller::{commands::ScannerCommand, events::ScannerEvent},
    errors::ScannerError,
    library::TrackId,
};
use crossbeam_channel::{select, tick, Receiver, Sender};
use fast_image_resize as fr;
use gpui::RenderImage;
use image::{imageops, DynamicImage, Frame, ImageReader, RgbaImage};
use lofty::{prelude::*, probe::Probe};
use smallvec::smallvec;
use std::collections::{HashMap, HashSet};
use std::io::Cursor;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use std::{fs, path::PathBuf, time::UNIX_EPOCH};
use uuid::Uuid;
use walkdir::WalkDir;

pub struct Scanner {
    pub tx: Sender<ScannerEvent>,
    pub rx: Receiver<ScannerCommand>,
}

enum ScanJob {
    Metadata(PathBuf, TrackId),
    Thumbnail(TrackId, ImageId, Vec<u8>),
    AlbumArt(PathBuf),
    PlaylistThumbnail(PlaylistId, Vec<PathBuf>),
}

impl Scanner {
    #[must_use]
    pub fn new() -> (Self, Sender<ScannerCommand>, Receiver<ScannerEvent>) {
        let (cmd_tx, cmd_rx) = crossbeam_channel::unbounded();
        let (event_tx, event_rx) = crossbeam_channel::unbounded();

        let scanner = Scanner {
            tx: event_tx,
            rx: cmd_rx,
        };

        (scanner, cmd_tx, event_rx)
    }

    #[allow(clippy::missing_errors_doc)]
    pub fn run(
        &mut self,
        metadata_workers: usize,
        thumbnail_workers: usize,
    ) -> Result<(), ScannerError> {
        let (meta_tx, meta_rx) = crossbeam_channel::unbounded();
        let (thumb_tx, thumb_rx) = crossbeam_channel::unbounded();
        let (album_art_tx, album_art_rx) = crossbeam_channel::unbounded();
        let (playlist_thumb_tx, playlist_thumb_rx) = crossbeam_channel::unbounded();

        self.spawn_metadata_worker(&meta_rx, &thumb_tx, metadata_workers);
        self.spawn_thumbnail_workers(&thumb_rx, thumbnail_workers);
        self.spawn_album_art_worker(album_art_rx);
        self.spawn_playlist_thumbnail_worker(playlist_thumb_rx);

        let mut inflight_tracks = HashSet::new();
        let mut inflight_playlists = HashSet::new();

        loop {
            match self.rx.recv()? {
                ScannerCommand::GetTrackMetadata { path, track_id } => {
                    if inflight_tracks.insert(track_id) {
                        self.enqueue_track(path, track_id, &meta_tx);
                    }
                }
                ScannerCommand::ScanFolder { tracks, path } => {
                    self.enqueue_folder(&tracks, &path, &meta_tx)?;
                }
                ScannerCommand::GetCurrentAlbumArt(path) => {
                    let _ = album_art_tx.send(ScanJob::AlbumArt(path));
                }
                ScannerCommand::PlaylistThumbnail { id, tracks } => {
                    if inflight_playlists.insert(id) {
                        let _ = playlist_thumb_tx.send(ScanJob::PlaylistThumbnail(id, tracks));
                    }
                }
                ScannerCommand::MetaJobFinished(id) => {
                    inflight_tracks.remove(&id);
                }
                ScannerCommand::PlaylistThumbnailJobFinished(id) => {
                    inflight_playlists.remove(&id);
                }
            }
        }
    }

    fn spawn_metadata_worker(
        &self,
        meta_rx: &Receiver<ScanJob>,
        thumb_tx: &Sender<ScanJob>,
        workers: usize,
    ) {
        let ticker = tick(Duration::from_millis(128));

        for _ in 0..workers {
            let meta_rx = meta_rx.clone();
            let thumb_tx = thumb_tx.clone();
            let events_tx = self.tx.clone();
            let ticker = ticker.clone();

            std::thread::spawn(move || {
                let mut batch: Vec<Track> = Vec::with_capacity(16);

                loop {
                    select! {
                        recv(meta_rx) -> job => {
                            if let Ok(ScanJob::Metadata(path, track_id)) = job {
                                match get_track_metadata(path, track_id) {
                                    Ok((track, image)) => {
                                        let id = track.id;
                                        batch.push(track);

                                        if batch.len() >= 16 {
                                            let _ = events_tx.send(
                                                ScannerEvent::Tracks(std::mem::take(&mut batch))
                                            );
                                        }

                                        if let Some(bytes) = image {
                                            let hash = ImageId(<[u8; 32]>::from(blake3::hash(&bytes)));

                                            let _ = thumb_tx.send(ScanJob::Thumbnail(id, hash, bytes));
                                        }
                                    }
                                    Err(err) => eprintln!("Failed to get track metadata: {err}" ),
                                }
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
        }
    }

    fn spawn_thumbnail_workers(&self, thumb_rx: &Receiver<ScanJob>, workers: usize) {
        let ticker = tick(Duration::from_millis(128));

        for _ in 0..workers {
            let events_tx = self.tx.clone();
            let ticker = ticker.clone();
            let thumb_rx = thumb_rx.clone();

            std::thread::spawn(move || {
                let mut image_batch = HashMap::with_capacity(16);
                let mut lookup_batch = HashMap::with_capacity(16);

                loop {
                    select! {
                        recv(thumb_rx) -> job => {
                            if let Ok(ScanJob::Thumbnail(id, hash, bytes)) = job {
                                 let path = get_cached_image_path(hash, ImageKind::Thumbnail);

                                if path.exists() {
                                    lookup_batch.insert(id, hash);
                                } else {
                                if let Ok(image) = render_album_art(&bytes, true) {
                                    image_batch.insert(hash, image);
                                    lookup_batch.insert(id, hash);

                                    if image_batch.len() >= 16 {
                                        let _ = events_tx.send(
                                            ScannerEvent::Thumbnails(std::mem::take(&mut image_batch))
                                        );
                                        let _ = events_tx.send(ScannerEvent::ImageLookup(std::mem::take(&mut lookup_batch)));
                                    }
                                }}
                            }
                        }

                        recv(ticker) -> _ => {
                            if !image_batch.is_empty() || !lookup_batch.is_empty() {
                                let _ = events_tx.send(
                                    ScannerEvent::Thumbnails(std::mem::take(&mut image_batch))
                                );
                                let _ = events_tx.send(ScannerEvent::ImageLookup(std::mem::take(&mut lookup_batch)));
                            }
                        }
                    }
                }
            });
        }
    }

    fn spawn_album_art_worker(&self, album_art_rx: Receiver<ScanJob>) {
        let events_tx = self.tx.clone();

        std::thread::spawn(move || {
            while let Ok(ScanJob::AlbumArt(path)) = album_art_rx.recv() {
                match get_album_art(&path) {
                    Ok((id, Some(image))) => {
                        let hash = ImageId(<[u8; 32]>::from(blake3::hash(&image)));

                        let path = get_cached_image_path(hash, ImageKind::AlbumArt);

                        if !path.exists() {
                            if let Ok(album_art) = render_album_art(&image, false) {
                                let _ = events_tx.send(ScannerEvent::AlbumArt(hash, album_art));
                                let _ = events_tx
                                    .send(ScannerEvent::ImageLookup(HashMap::from([(id, hash)])));
                            }
                        } else {
                            let _ = events_tx
                                .send(ScannerEvent::ImageLookup(HashMap::from([(id, hash)])));
                        }
                    }
                    Err(err) => eprintln!("Failed album art: {err}"),
                    _ => {}
                }
            }
        });
    }

    fn spawn_playlist_thumbnail_worker(&self, playlist_thumb_rx: Receiver<ScanJob>) {
        let events_tx = self.tx.clone();

        std::thread::spawn(move || {
            while let Ok(ScanJob::PlaylistThumbnail(id, tracks)) = playlist_thumb_rx.recv() {
                let mut images = Vec::with_capacity(4);

                println!("received tracks: {:#?}", tracks);

                for path in tracks {
                    if images.len() == 4 {
                        break;
                    }

                    match get_album_art(&path) {
                        Ok((_tid, Some(image))) => {
                            if let Ok(img) = image::load_from_memory(&image) {
                                images.push(img);
                            } else {
                                eprintln!("Invalid album art in {:?}", path);
                            }
                        }
                        Ok((_tid, None)) => {}
                        Err(err) => eprintln!("Failed album art for {:?}: {err}", path),
                    }
                }

                if images.is_empty() {
                    continue;
                }

                match render_playlist_thumbnail(images) {
                    (Some(thumbnail), Some(hash)) => {
                        let _ =
                            events_tx.send(ScannerEvent::PlaylistThumbnail(id, hash, thumbnail));
                    }
                    _ => eprintln!("Failed to generate playlist thumbnail"),
                }
            }
        });
    }

    fn enqueue_track(&self, path: PathBuf, track_id: TrackId, meta_tx: &Sender<ScanJob>) {
        let _ = meta_tx.send(ScanJob::Metadata(path, track_id));
    }

    fn enqueue_folder(
        &self,
        existing_tracks: &HashSet<TrackId>,
        path: &PathBuf,
        meta_tx: &Sender<ScanJob>,
    ) -> Result<(), ScannerError> {
        let supported = ["mp3", "flac", "wav", "ogg", "m4a"];

        let mut track_ids = Vec::new();

        if path.is_dir() {
            for entry in WalkDir::new(path)
                .into_iter()
                .filter_map(Result::ok)
                .filter(|e| e.file_type().is_file())
            {
                let file = entry.path().to_path_buf();

                let ext_ok = file
                    .extension()
                    .and_then(|e| e.to_str())
                    .is_some_and(|e| supported.contains(&e.to_lowercase().as_str()));

                if !ext_ok {
                    continue;
                }

                let id = gen_track_id(&file)?;
                track_ids.push(id);

                if !existing_tracks.contains(&id) {
                    let _ = meta_tx.send(ScanJob::Metadata(file, id));
                }
            }
        }

        let playlist = Playlist {
            id: PlaylistId(Uuid::new_v4()),
            name: path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("Unnamed Playlist")
                .to_string(),
            source: PlaylistSource::Folder,
            tracks: track_ids,
            duration: Duration::from_secs(0),
            image_id: None,
        };

        let _ = self.tx.send(ScannerEvent::Playlist(playlist));

        Ok(())
    }
}

fn render_album_art(bytes: &[u8], is_thumbnail: bool) -> Result<Arc<RenderImage>, ScannerError> {
    let img = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()?
        .decode()?;

    let rgba = img.into_rgba8();

    let image = if is_thumbnail {
        let (src_w, src_h) = rgba.dimensions();

        let src =
            fr::images::Image::from_vec_u8(src_w, src_h, rgba.into_raw(), fr::PixelType::U8x4)?;

        let mut dst = fr::images::Image::new(256, 256, fr::PixelType::U8x4);

        let mut resizer = fr::Resizer::new();

        resizer.resize(
            &src,
            &mut dst,
            &fr::ResizeOptions::new()
                .resize_alg(fr::ResizeAlg::Convolution(fr::FilterType::Bilinear)),
        )?;

        RgbaImage::from_raw(256, 256, dst.into_vec()).unwrap()
    } else {
        rgba
    };

    let mut image = image;

    for px in <[u8] as AsMut<[u8]>>::as_mut(&mut image).chunks_exact_mut(4) {
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
        .and_then(Probe::read)
    {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Metadata decode failed {}: {e:?}", path.display());

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
                    image_id: None,
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
            .map(ToOwned::to_owned)
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

        thumbnail = tag.pictures().first().map(|data| data.data().to_vec());
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
            image_id: None,
        },
        thumbnail,
    ))
}

fn get_album_art(path: &Path) -> Result<(TrackId, Option<Vec<u8>>), ScannerError> {
    let tagged_file = match Probe::open(path)
        .and_then(|p| Ok(p.guess_file_type()?))
        .and_then(Probe::read)
    {
        Ok(file) => file,
        Err(e) => return Err(ScannerError::from(e)),
    };

    let id = gen_track_id(path)?;

    let tag = tagged_file
        .primary_tag()
        .or_else(|| tagged_file.first_tag());

    let thumbnail;

    if let Some(tag) = tag {
        thumbnail = tag.pictures().first().map(|data| data.data().to_vec());
    } else {
        thumbnail = None;
    }

    Ok((id, thumbnail))
}

fn render_playlist_thumbnail(
    mut images: Vec<DynamicImage>,
) -> (Option<Arc<RenderImage>>, Option<ImageId>) {
    let mut canvas = DynamicImage::new_rgba8(256, 256);

    match images.len() {
        1 => {
            let img = images
                .remove(0)
                .resize_exact(256, 256, imageops::FilterType::Lanczos3);

            imageops::overlay(&mut canvas, &img, 0, 0);
        }

        2 => {
            for (i, img) in images.into_iter().enumerate() {
                let resized = img.resize_exact(128, 256, imageops::FilterType::Lanczos3);
                imageops::overlay(&mut canvas, &resized, (i * 128) as i64, 0);
            }
        }

        3 => {
            let a = images
                .remove(0)
                .resize_exact(128, 128, imageops::FilterType::Lanczos3);
            let b = images
                .remove(0)
                .resize_exact(128, 128, imageops::FilterType::Lanczos3);
            let c = images
                .remove(0)
                .resize_exact(256, 128, imageops::FilterType::Lanczos3);

            imageops::overlay(&mut canvas, &a, 0, 0);
            imageops::overlay(&mut canvas, &b, 128, 0);
            imageops::overlay(&mut canvas, &c, 0, 128);
        }

        _ => {
            for (i, img) in images.into_iter().take(4).enumerate() {
                let resized = img.resize_exact(128, 128, imageops::FilterType::Lanczos3);

                let x = (i % 2) * 128;
                let y = (i / 2) * 128;

                imageops::overlay(&mut canvas, &resized, x as i64, y as i64);
            }
        }
    }

    let mut image = canvas.to_rgba8();

    let hash = ImageId(<[u8; 32]>::from(blake3::hash(image.as_raw())));

    for px in <[u8] as AsMut<[u8]>>::as_mut(&mut image).chunks_exact_mut(4) {
        px.swap(0, 2);
    }

    let frame = Frame::new(image);

    let render_image = Arc::new(RenderImage::new(smallvec![frame]));

    (Some(render_image), Some(hash))
}

fn get_cached_image_path(id: ImageId, kind: ImageKind) -> PathBuf {
    let base_dir = dirs::audio_dir()
        .unwrap_or_default()
        .join("wiremann")
        .join("cache");

    let hex = hex::encode(id.0);
    let folder = &hex[0..2];

    let name = match kind {
        ImageKind::Thumbnail => format!("{hex}_thumb.bgra.zstd"),
        ImageKind::AlbumArt => format!("{hex}_art.bgra.zstd"),
        ImageKind::Playlist => format!("{hex}_playlist.bgra.zstd"),
    };

    base_dir.join("images").join(folder).join(name)
}
