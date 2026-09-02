use crate::app::AppPaths;
use crate::cacher::{Cacher, paths::CachePaths};
use crate::controller::commands::ImageProcessorCommand;
use crate::controller::events::ImageProcessorEvent;
use crate::controller::state::PlaylistId;
use crate::controller::state::{ImageId, TrackId};
use crate::{
    cacher::ImageKind,
    errors::ImageProcessorError,
    scanner::metadata,
    title::strip_search_suffixes,
};
use crossbeam_channel::{Receiver, Sender, select, tick};
use dashmap::DashSet;
use gpui::RenderImage;
use image::{DynamicImage, EncodableLayout, Frame, imageops};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{error, info, trace, warn};

pub struct ImageProcessor {
    pub tx: Sender<ImageProcessorEvent>,
    pub rx: Receiver<ImageProcessorCommand>,

    app_paths: AppPaths,
    seen_images: Arc<DashSet<(ImageId, ImageKind)>>,
}

enum ImageJob {
    Thumbnail(TrackId, PathBuf, ImageKind, Arc<HashSet<ImageId>>),
    AlbumArt(TrackId, PathBuf),
    AlbumArtOnline(TrackId, String, String, String),
    PlaylistThumbnail(PlaylistId, Vec<PathBuf>),
}

impl ImageProcessor {
    #[must_use]
    pub fn new(
        app_paths: AppPaths,
    ) -> (
        Self,
        Sender<ImageProcessorCommand>,
        Receiver<ImageProcessorEvent>,
    ) {
        let (cmd_tx, cmd_rx) = crossbeam_channel::unbounded();
        let (event_tx, event_rx) = crossbeam_channel::unbounded();

        let scanner = ImageProcessor {
            tx: event_tx,
            rx: cmd_rx,

            app_paths,
            seen_images: Arc::new(DashSet::new()),
        };

        (scanner, cmd_tx, event_rx)
    }

    #[allow(clippy::missing_errors_doc)]
    pub fn run(&mut self, thumbnail_workers: usize) -> Result<(), ImageProcessorError> {
        let (thumb_tx, thumb_rx) = crossbeam_channel::unbounded();
        let (album_art_tx, album_art_rx) = crossbeam_channel::unbounded();
        let (playlist_thumb_tx, playlist_thumb_rx) = crossbeam_channel::unbounded();

        self.spawn_thumbnail_workers(&thumb_rx, thumbnail_workers);
        self.spawn_album_art_workers(&album_art_rx, 2);
        self.spawn_playlist_thumbnail_worker(playlist_thumb_rx);

        let mut inflight_playlists = HashSet::new();

        loop {
            match self.rx.recv()? {
                ImageProcessorCommand::GetThumbnails(images, kind) => {
                    let cached_thumbnails_index = Arc::new(Cacher::build_cached_thumbnails_index(
                        self.app_paths.cache.as_path(),
                        kind,
                    ));
                    for image in images {
                        let _ = thumb_tx.send(ImageJob::Thumbnail(
                            image.0,
                            image.1,
                            kind,
                            cached_thumbnails_index.clone(),
                        ));
                    }
                }
                ImageProcessorCommand::GetCurrentAlbumArt(id, path) => {
                    info!(track_id = ?id, ?path, "Received GetCurrentAlbumArt command");
                    let _ = album_art_tx.send(ImageJob::AlbumArt(id, path));
                }
                ImageProcessorCommand::FetchAlbumArtOnline { id, title, artist, album } => {
                    info!(track_id = ?id, title = %title, artist = %artist, "Received FetchAlbumArtOnline command");
                    let _ = album_art_tx.send(ImageJob::AlbumArtOnline(id, title, artist, album));
                }
                ImageProcessorCommand::PlaylistThumbnail { id, tracks } => {
                    if inflight_playlists.insert(id) {
                        let _ = playlist_thumb_tx.send(ImageJob::PlaylistThumbnail(id, tracks));
                    }
                }
                ImageProcessorCommand::PlaylistJobFinished(id) => {
                    inflight_playlists.remove(&id);
                }
            }
        }
    }

    fn spawn_thumbnail_workers(&self, thumb_rx: &Receiver<ImageJob>, workers: usize) {
        let ticker = tick(Duration::from_millis(128));

        for _ in 0..workers {
            let events_tx = self.tx.clone();
            let ticker = ticker.clone();
            let thumb_rx = thumb_rx.clone();
            let seen_images = self.seen_images.clone();

            std::thread::spawn(move || {
                info!("image processor thumbnail worker started");
                let mut image_batch = HashMap::with_capacity(64);
                let mut lookup_batch = HashMap::with_capacity(64);
                let mut last_kind = ImageKind::ThumbnailSmall;

                loop {
                    select! {
                        recv(thumb_rx) -> job => {
                            let job_start = Instant::now();
                            if let Ok(ImageJob::Thumbnail(id, path, kind, cached_images)) = job &&
                                let Ok(Some(bytes)) = metadata::read_album_art(&path) && let Ok(hash) = ImageId::generate(&bytes) {
                                    trace!(thread_id = ?std::thread::current().id(), worker = "image-processor", "received job {:?} {}", id, path.display());
                                    lookup_batch.insert(id, hash);
                                    last_kind = kind;
                                    if seen_images.insert((hash, kind)) && !cached_images.contains(&hash) {
                                        match render_album_art(&bytes, kind) {
                                            Ok(image) => {
                                                image_batch.insert(hash, image.clone());
                                                if image_batch.len() >= 64 {
                                                    let _ = events_tx.send(
                                                        ImageProcessorEvent::InsertThumbnails(std::mem::take(&mut image_batch), kind)
                                                    );
                                                    let _ = events_tx.send(ImageProcessorEvent::UpdateImageLookup(std::mem::take(&mut lookup_batch)));
                                                }
                                            }
                                            Err(e) => {
                                                error!(error = ?e, "Error occurred while processing image");
                                            }
                                        }
                                    }
                                    trace!(thread_id = ?std::thread::current().id(), worker = "image-processor", elapsed_ms = ?job_start.elapsed().as_millis(), "finished job {:?} {}", id, path.display());
                                } else {
                                    trace!(thread_id = ?std::thread::current().id(), worker = "image-processor", elapsed_ms = ?job_start.elapsed().as_millis(), "skipped/invalid thumbnail job");
                                }
                        }

                        recv(ticker) -> _ => {
                            if !image_batch.is_empty() || !lookup_batch.is_empty() {
                                let _ = events_tx.send(
                                    ImageProcessorEvent::InsertThumbnails(std::mem::take(&mut image_batch), last_kind)
                                );
                                let _ = events_tx.send(ImageProcessorEvent::UpdateImageLookup(std::mem::take(&mut lookup_batch)));
                            }
                        }
                    }
                }
            });
        }
    }

    fn spawn_album_art_workers(&self, album_art_rx: &Receiver<ImageJob>, workers: usize) {
        for _ in 0..workers {
            let events_tx = self.tx.clone();
            let cache_path = self.app_paths.cache.clone();
            let album_art_rx = album_art_rx.clone();

            std::thread::spawn(move || {
                while let Ok(job) = album_art_rx.recv() {
                    match job {
                        ImageJob::AlbumArt(id, path) => {
                            match metadata::read_album_art(&path) {
                                Ok(Some(image)) => {
                                    if let Ok(hash) = ImageId::generate(&image) {
                                        let path = CachePaths::image_cache_path(
                                            cache_path.as_path(),
                                            hash,
                                            ImageKind::AlbumArt,
                                        );

                                        if path.exists() {
                                            let _ = events_tx.send(
                                                ImageProcessorEvent::UpdateImageLookup(
                                                    HashMap::from([(id, hash)]),
                                                ),
                                            );
                                        } else if let Ok(album_art) =
                                            render_album_art(&image, ImageKind::AlbumArt)
                                        {
                                            let _ = events_tx.send(
                                                ImageProcessorEvent::InsertAlbumArt(hash, album_art),
                                            );
                                            let _ = events_tx.send(
                                                ImageProcessorEvent::UpdateImageLookup(
                                                    HashMap::from([(id, hash)]),
                                                ),
                                            );
                                        }
                                    }
                                }
                                Err(err) => warn!(error = ?err, "Failed to read album art"),
                                Ok(None) => info!(
                                    track_id = ?id,
                                    path = ?path,
                                    "No embedded album art found in file"
                                ),
                            }
                        }
                        ImageJob::AlbumArtOnline(id, title, artist, album) => {
                            let start = Instant::now();
                            fetch_album_art_online(
                                &events_tx, &cache_path, id, &title, &artist, &album,
                            );
                            info!(
                                track_id = ?id,
                                elapsed_ms = ?start.elapsed().as_millis(),
                                "online album art fetch finished"
                            );
                        }
                        _ => {} // ignore other job types sent to this channel by mistake
                    }
                }
            });
        }
    }

    fn spawn_playlist_thumbnail_worker(&self, playlist_thumb_rx: Receiver<ImageJob>) {
        let events_tx = self.tx.clone();

        std::thread::spawn(move || {
            while let Ok(ImageJob::PlaylistThumbnail(id, tracks)) = playlist_thumb_rx.recv() {
                let mut images = Vec::with_capacity(4);

                for path in tracks {
                    if images.len() == 4 {
                        break;
                    }

                    match metadata::read_album_art(&path) {
                        Ok(Some(image)) => {
                            if let Ok(img) = image::load_from_memory(&image) {
                                images.push(img);
                            } else {
                                warn!(path = %path.display(), "Invalid album art");
                            }
                        }
                        Ok(None) => {}
                        Err(err) => {
                            warn!(error = ?err, path = %path.display(), "Failed to read album art");
                        }
                    }
                }

                if images.is_empty() {
                    continue;
                }

                if let (Some(thumbnail), Some(hash)) = render_playlist_thumbnail(images) {
                    let _ = events_tx.send(ImageProcessorEvent::InsertPlaylistThumbnail(
                        id, hash, thumbnail,
                    ));
                } else {
                    warn!("Failed to generate playlist thumbnail")
                }
            }
        });
    }
}

fn fetch_album_art_online(
    events_tx: &Sender<ImageProcessorEvent>,
    _cache_path: &PathBuf,
    id: TrackId,
    title: &str,
    artist: &str,
    album: &str,
) {
    // Filenames without metadata often trail "[Official Music Video]" or
    // similar bracket groups that pollute the search. Try the stripped title
    // first, falling back to the raw title and album.
    let stripped = strip_search_suffixes(title);
    let title_attempts: &[&str] = if stripped != title && !stripped.is_empty() {
        &[stripped, title]
    } else {
        &[title]
    };

    for title_attempt in title_attempts {
        // Build search query. Skip artist when it's unset — "Unknown Artist" pollutes the search.
        let query = if artist.is_empty() || artist.eq_ignore_ascii_case("Unknown Artist") {
            (*title_attempt).to_string()
        } else {
            format!("{} {title_attempt}", artist)
        };

        if let Some(cover_url) = deezer_search(&query, 3) {
            download_and_send_album_art(events_tx, id, &cover_url);
            return;
        }
    }

    // Fallback: try artist + album when the album name is real.
    if !album.is_empty() && album != "Unknown Album" {
        let q2 = format!("{} {}", artist, album);
        if let Some(cover_url) = deezer_search(&q2, 1) {
            download_and_send_album_art(events_tx, id, &cover_url);
            return;
        }
    }

    warn!(track_id = ?id, "No album art found online for {title}");
}

/// Search Deezer and return the first `cover_big` URL, or `None`.
fn deezer_search(query: &str, limit: usize) -> Option<String> {
    let url = format!(
        "https://api.deezer.com/search?q={}&limit={}",
        urlencoding(query),
        limit
    );

    let client = reqwest::blocking::Client::builder()
        .user_agent(concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")))
        .timeout(Duration::from_secs(15))
        .build()
        .ok()?;

    let resp = client.get(&url).send().ok()?;
    let body = resp.text().ok()?;
    let json: serde_json::Value = serde_json::from_str(&body).ok()?;

    json.get("data")
        .and_then(|d| d.as_array())
        .and_then(|arr| {
            arr.iter().find_map(|entry| {
                entry
                    .get("album")
                    .and_then(|a| a.get("cover_big"))
                    .and_then(|c| c.as_str())
                    .map(String::from)
            })
        })
}

fn download_and_send_album_art(
    events_tx: &Sender<ImageProcessorEvent>,
    id: TrackId,
    cover_url: &str,
) {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .ok()
        .expect("Failed to build HTTP client");

    let img_bytes = match client.get(cover_url).send() {
        Ok(r) => match r.bytes() {
            Ok(b) => b.to_vec(),
            Err(e) => {
                warn!(error = ?e, "Failed to read cover image bytes");
                return;
            }
        },
        Err(e) => {
            warn!(error = ?e, "Failed to download cover image");
            return;
        }
    };

    if let Ok(hash) = ImageId::generate(&img_bytes) {
        match render_album_art(&img_bytes, ImageKind::AlbumArt) {
            Ok(album_art) => {
                let _ = events_tx
                    .send(ImageProcessorEvent::InsertAlbumArt(hash, album_art));
                let _ = events_tx.send(ImageProcessorEvent::UpdateImageLookup(
                    HashMap::from([(id, hash)]),
                ));
                // Also generate a small thumbnail for library lists.
                if let Ok(thumb) = render_album_art(&img_bytes, ImageKind::ThumbnailSmall) {
                    let _ = events_tx.send(ImageProcessorEvent::InsertThumbnails(
                        HashMap::from([(hash, thumb)]),
                        ImageKind::ThumbnailSmall,
                    ));
                }
                info!(track_id = ?id, "Fetched album art online");
            }
            Err(e) => warn!(error = ?e, "Failed to render downloaded album art"),
        }
    }
}

/// URL-encode a string for HTTP queries.
fn urlencoding(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    for byte in input.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(byte as char);
            }
            b' ' => result.push_str("%20"),
            _ => {
                result.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    result
}

fn render_album_art(
    bytes: &[u8],
    kind: ImageKind,
) -> Result<Arc<RenderImage>, ImageProcessorError> {
    let raw_img = image::load_from_memory(bytes)?;

    let image = match kind {
        ImageKind::AlbumArt => raw_img.into_rgba8(),
        ImageKind::ThumbnailSmall | ImageKind::ThumbnailLarge => {
            let (new_w, new_h) = match kind {
                ImageKind::ThumbnailSmall => (128, 128),
                ImageKind::ThumbnailLarge => (512, 512),
                _ => unreachable!(),
            };

            raw_img.thumbnail(new_w, new_h).into_rgba8()
        }
        ImageKind::Playlist => unreachable!(),
    };

    let render_image = Arc::new(RenderImage::new(vec![Frame::new(image)])?);

    Ok(render_image)
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
                #[allow(clippy::cast_possible_wrap)]
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

                #[allow(clippy::cast_possible_wrap)]
                imageops::overlay(&mut canvas, &resized, x as i64, y as i64);
            }
        }
    }

    let image = canvas.to_rgba8();

    let hash = ImageId::generate(image.as_bytes()).ok();

    let frame = Frame::new(image);

    let render_image = RenderImage::new(vec![frame]).ok().map(Arc::new);

    (render_image, hash)
}
