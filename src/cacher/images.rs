use std::{
    collections::{HashMap, HashSet},
    fs,
    io::{Cursor, Write},
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use crossbeam_channel::{Receiver, select, tick};
use gpui::RenderImage;
use image::Frame;
use smallvec::smallvec;
use walkdir::WalkDir;

use crate::{
    cacher::{CacheJob, CachedImage, ImageKind},
    controller::events::CacherEvent,
    errors::CacherError,
    library::ImageId,
};

use super::Cacher;

impl Cacher {
    fn cached_image_path(&self, id: ImageId, kind: ImageKind) -> PathBuf {
        let hex = hex::encode(id.0);
        let folder = &hex[0..2];

        let name = match kind {
            ImageKind::ThumbnailSmall => format!("{hex}_tmbhs.bgra.zstd"),
            ImageKind::ThumbnailLarge => format!("{hex}_tmbhl.bgra.zstd"),
            ImageKind::AlbumArt => format!("{hex}_art.bgra.zstd"),
            ImageKind::Playlist => format!("{hex}_playlist.bgra.zstd"),
        };

        self.app_paths.cache.join("images").join(folder).join(name)
    }

    fn write_cached_image(
        &self,
        id: ImageId,
        kind: ImageKind,
        cached_image: &CachedImage,
    ) -> Result<(), CacherError> {
        let final_path = self.cached_image_path(id, kind);
        let tmp_path = final_path.with_extension("tmp");

        if final_path.exists() {
            return Ok(());
        }

        fs::create_dir_all(final_path.parent().unwrap())?;

        let bytes = bitcode::encode(cached_image);

        let compressed = zstd::encode_all(Cursor::new(bytes), 4)?;

        {
            let mut file = fs::File::create(&tmp_path)?;
            file.write_all(&compressed)?;
            file.sync_all()?;
        }

        fs::rename(tmp_path, final_path)?;

        Ok(())
    }

    fn read_cached_image(
        &self,
        id: ImageId,
        kind: ImageKind,
    ) -> Result<Option<Arc<RenderImage>>, CacherError> {
        let path = self.cached_image_path(id, kind);

        let bytes = fs::read(path)?;

        let decompressed = zstd::decode_all(Cursor::new(bytes))?;

        let cached_image: CachedImage = bitcode::decode(&decompressed)?;

        match image::RgbaImage::from_raw(
            cached_image.width,
            cached_image.height,
            cached_image.image,
        ) {
            Some(image) => {
                let frame = Frame::new(image);

                Ok(Some(Arc::new(RenderImage::new(smallvec![frame]))))
            }
            None => Ok(None),
        }
    }

    pub fn spawn_thumbnail_workers(&self, rx: &Receiver<CacheJob>, workers: usize) {
        let ticker = tick(Duration::from_millis(128));

        for _ in 0..workers {
            let cacher = self.clone();
            let ticker = ticker.clone();
            let thumb_rx = rx.clone();

            std::thread::spawn(move || {
                let mut batch = HashMap::with_capacity(16);
                let mut missing = Vec::new();

                loop {
                    select! {
                        recv(thumb_rx) -> job => {
                            match job {
                                Ok(CacheJob::WriteImage {id, kind, width, height, image}) => {
                                    let cached_image = CachedImage {
                                        width,
                                        height,
                                        image
                                    };
                                    match cacher.write_cached_image(id, kind, &cached_image) {
                                        Ok(()) => {}
                                        Err(err) => {eprintln!("Error occurred: {err:#?}");}
                                    }
                                }
                                Ok(CacheJob::LoadThumbnails(ids, kind)) => {
                                    for id in ids {
                                        match cacher.read_cached_image(id, kind) {
                                            Ok(Some(image)) => { batch.insert(id, image); },
                                            Ok(None) | Err(_) => { missing.push(id); },
                                        }

                                        if batch.len() >= 16 {
                                            let _ = cacher.tx.send(CacherEvent::Thumbnails(std::mem::take(&mut batch)));
                                        }

                                        if missing.len() >= 16 {
                                            let _ = cacher.tx.send(CacherEvent::MissingThumbnails(std::mem::take(&mut missing)));
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }

                        recv(ticker) -> _ => {
                            if !batch.is_empty() {
                                let _ = cacher.tx.send(CacherEvent::Thumbnails(std::mem::take(&mut batch)));
                            }

                            if !missing.is_empty() {
                                let _ = cacher.tx.send(CacherEvent::MissingThumbnails(std::mem::take(&mut missing)));
                            }
                        }
                    }
                }
            });
        }
    }

    pub fn spawn_album_art_worker(&self, rx: Receiver<CacheJob>) {
        let cacher = Arc::new(self.clone());

        std::thread::spawn(move || {
            while let Ok(job) = rx.recv() {
                match job {
                    CacheJob::LoadAlbumArt(id) => {
                        match cacher.read_cached_image(id, ImageKind::AlbumArt) {
                            Ok(Some(image)) => {
                                let _ = cacher.tx.send(CacherEvent::AlbumArt(image));
                            }
                            Err(e) => {
                                eprintln!("Error loading album art: {e}");
                                let _ = cacher.tx.send(CacherEvent::MissingAlbumArt(id));
                            }
                            _ => {
                                let _ = cacher.tx.send(CacherEvent::MissingAlbumArt(id));
                            }
                        }
                    }
                    CacheJob::WriteImage {
                        id,
                        kind,
                        width,
                        height,
                        image,
                    } => {
                        let cached_image = CachedImage {
                            width,
                            height,
                            image,
                        };
                        match cacher.write_cached_image(id, kind, &cached_image) {
                            Ok(()) => {}
                            Err(err) => {
                                eprintln!("Error occurred: {err:#?}");
                            }
                        }
                    }
                    _ => {}
                }
            }
        });
    }

    pub fn spawn_playlist_thumbnail_worker(&self, rx: Receiver<CacheJob>) {
        let cacher = Arc::new(self.clone());

        std::thread::spawn(move || {
            while let Ok(job) = rx.recv() {
                match job {
                    CacheJob::LoadPlaylistThumbnail(id) => {
                        match cacher.read_cached_image(id, ImageKind::Playlist) {
                            Ok(Some(image)) => {
                                let _ = cacher.tx.send(CacherEvent::PlaylistThumbnail(id, image));
                            }
                            Err(e) => {
                                eprintln!("Error loading playlist thumbnail art: {e}");
                                let _ = cacher.tx.send(CacherEvent::MissingPlaylistThumbnail(id));
                            }
                            _ => {
                                let _ = cacher.tx.send(CacherEvent::MissingPlaylistThumbnail(id));
                            }
                        }
                    }
                    CacheJob::WriteImage {
                        id,
                        kind,
                        width,
                        height,
                        image,
                    } => {
                        let cached_image = CachedImage {
                            width,
                            height,
                            image,
                        };
                        match cacher.write_cached_image(id, kind, &cached_image) {
                            Ok(()) => {}
                            Err(err) => {
                                eprintln!("Error occurred: {err:#?}");
                            }
                        }
                    }
                    _ => {}
                }
            }
        });
    }

    pub fn build_cached_thumbnails_index(base: &Path, kind: ImageKind) -> HashSet<ImageId> {
        let mut set = HashSet::new();

        let images_dir = base.join("images");

        for entry in WalkDir::new(images_dir)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file())
        {
            let ends_with = match kind {
                ImageKind::ThumbnailSmall => "_thumb.tmbhs",
                ImageKind::ThumbnailLarge => "_thumb.tmbhl",
                _ => continue,
            };
            if let Some(name) = entry.file_name().to_str()
                && name.ends_with(ends_with)
                && let Some((hex_part, _rest)) = name.split_once('_')
            {
                let mut arr = [0u8; 16];

                if hex::decode_to_slice(hex_part, &mut arr).is_ok() {
                    set.insert(ImageId(arr));
                }
            }
        }

        set
    }
}
