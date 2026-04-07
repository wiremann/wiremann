pub mod metadata;

use crate::app::AppPaths;
use crate::cacher::{Cacher, ImageKind};
use crate::library::playlists::{Playlist, PlaylistId, PlaylistSource};
use crate::library::{ImageId, Track, TrackSource};
use crate::{
    controller::{commands::ScannerCommand, events::ScannerEvent},
    errors::ScannerError,
    library::TrackId,
};
use crossbeam_channel::{Receiver, Sender, select, tick};
use dashmap::DashSet;
use fast_image_resize as fr;
use garb::bytes::rgba_to_bgra_inplace;
use gpui::RenderImage;
use image::{DynamicImage, EncodableLayout, Frame, GenericImageView, RgbaImage, imageops};
use lofty::prelude::*;
use smallvec::smallvec;
use std::cmp::PartialEq;
use std::collections::{HashMap, HashSet, VecDeque};
use std::ffi::OsStr;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Duration;
use std::{path::PathBuf, time::UNIX_EPOCH};
use uuid::Uuid;
use walkdir::WalkDir;

pub struct Scanner {
    pub tx: Sender<ScannerEvent>,
    pub rx: Receiver<ScannerCommand>,

    app_paths: AppPaths,

    scan_progress: Arc<ScanProgress>,
    queue: VecDeque<PathBuf>,
    scan_record: Arc<DashSet<TrackSource>>,

    seen_images: Arc<DashSet<(ImageId, ImageKind)>>,
}

struct ScanProgress {
    discovery_done: AtomicBool,
    total: AtomicUsize,
    processed: AtomicUsize,
}

impl Scanner {
    #[must_use]
    pub fn new(app_paths: AppPaths) -> (Self, Sender<ScannerCommand>, Receiver<ScannerEvent>) {
        let (cmd_tx, cmd_rx) = crossbeam_channel::unbounded();
        let (event_tx, event_rx) = crossbeam_channel::unbounded();

        let scanner = Scanner {
            tx: event_tx,
            rx: cmd_rx,

            app_paths,

            scan_progress: Arc::new(ScanProgress {
                discovery_done: AtomicBool::new(false),
                total: AtomicUsize::new(0),
                processed: AtomicUsize::new(0),
            }),
            queue: VecDeque::new(),
            scan_record: Arc::new(DashSet::new()),

            seen_images: Arc::new(DashSet::new()),
        };

        (scanner, cmd_tx, event_rx)
    }

    #[allow(clippy::missing_errors_doc)]
    pub fn run(
        &mut self,
        metadata_workers: usize,
        _thumbnail_workers: usize,
    ) -> Result<(), ScannerError> {
        let (worker_tx, worker_rx) = crossbeam_channel::unbounded();

        self.spawn_metadata_workers(&worker_rx, metadata_workers);

        loop {
            match self.rx.recv()? {
                ScannerCommand::ScanFolder(path) => self.scan_folder(path, &worker_tx),
            }
        }
    }

    fn spawn_metadata_workers(&self, worker_rx: &Receiver<PathBuf>, workers: usize) {
        for _ in 0..workers {
            let worker_rx = worker_rx.clone();
            let scan_progress = self.scan_progress.clone();
            let tx = self.tx.clone();

            // let batch = Vec::with_capacity(64);

            while let Ok(path) = worker_rx.recv() {
                println!("{path:#?}")
            }
        }
    }

    fn scan_folder(&self, path: PathBuf, worker_tx: &Sender<PathBuf>) {
        let exts = ["mp3", "wav", "ogg", "aac", "m4a"];

        for entry in WalkDir::new(&path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .and_then(OsStr::to_str)
                    .map(|ext| exts.contains(&ext))
                    .unwrap_or(false)
            })
        {
            self.scan_progress.total.fetch_add(1, Ordering::Relaxed);

            let _ = worker_tx.send(entry.path().to_path_buf());
        }

        self.scan_progress
            .discovery_done
            .store(true, Ordering::Relaxed);
    }
}

fn render_album_art(
    bytes: &[u8],
    kind: ImageKind,
    resizer: &mut fr::Resizer,
) -> Result<Arc<RenderImage>, ScannerError> {
    let raw_img = image::load_from_memory(bytes)?;

    let image = match kind {
        ImageKind::AlbumArt => {
            let mut rgba = raw_img.into_rgba8();
            rgba_to_bgra_inplace(rgba.as_mut())?;
            rgba
        }
        ImageKind::ThumbnailSmall | ImageKind::ThumbnailLarge => {
            let (new_w, new_h) = match kind {
                ImageKind::ThumbnailSmall => (128, 128),
                ImageKind::ThumbnailLarge => (256, 256),
                _ => unreachable!(),
            };

            let mut dst = fr::images::Image::new(new_w, new_h, fr::PixelType::U8x4);

            resizer.resize(
                &raw_img,
                &mut dst,
                &fr::ResizeOptions::new()
                    .resize_alg(fr::ResizeAlg::Convolution(fr::FilterType::Bilinear)),
            )?;

            let mut buf = dst.into_vec();
            rgba_to_bgra_inplace(&mut buf)?;

            RgbaImage::from_raw(new_w, new_h, buf).unwrap()
        }
        _ => unreachable!(),
    };

    let frame = Frame::new(image);

    Ok(Arc::new(RenderImage::new(smallvec![frame])))
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

    let hash = if let Ok(hash) = ImageId::generate(image.as_bytes()) {
        Some(hash)
    } else {
        None
    };

    rgba_to_bgra_inplace(image.as_mut()).ok();

    let frame = Frame::new(image);

    let render_image = Arc::new(RenderImage::new(smallvec![frame]));

    (Some(render_image), hash)
}
