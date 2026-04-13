use crate::app::AppPaths;
use crate::controller::commands::CacherCommand;
use crate::controller::events::CacherEvent;
use crate::controller::state::{AppState, LibraryState, PlaybackState, PlaybackStatus, QueueState};
use crate::errors::CacherError;
use crate::library::playlists::{Playlist, PlaylistId, PlaylistSource};
use crate::library::{ImageId, Track, TrackId, TrackSource};
use bitcode::{Decode, Encode};
use crossbeam_channel::{Receiver, Sender, select, tick};
use gpui::RenderImage;
use image::Frame;
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};
use smallvec::smallvec;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;
use walkdir::WalkDir;

#[derive(Clone)]
pub struct Cacher {
    pub tx: Sender<CacherEvent>,
    pub rx: Receiver<CacherCommand>,
    app_paths: AppPaths,
}

enum CacheJob {
    WriteLibraryState(LibraryState),
    WritePlaybackState(PlaybackState),
    WriteQueueState(QueueState),
    WriteImage {
        id: ImageId,
        kind: ImageKind,
        width: u32,
        height: u32,
        image: Vec<u8>,
    },
    LoadAppState,
    LoadThumbnails(HashSet<ImageId>, ImageKind),
    LoadAlbumArt(ImageId),
    LoadPlaylistThumbnail(ImageId),
}

#[derive(Encode, Decode)]
struct CacheFile<T> {
    version: u32,
    payload: T,
}

#[derive(Copy, Clone, PartialEq, Debug, Eq, Hash)]
pub enum ImageKind {
    ThumbnailSmall,
    ThumbnailLarge,
    AlbumArt,
    Playlist,
}
#[derive(Debug, Clone, PartialEq, Default, Encode, Decode)]
struct CachedTrack {
    pub id: [u8; 16],
    pub sources: Vec<CachedTrackSource>,

    pub title: String,
    pub artist: String,
    pub album: String,

    pub duration: u64,

    pub image_id: Option<[u8; 16]>,
}

#[derive(Debug, Clone, PartialEq, Default, Hash, Eq, Encode, Decode)]
pub struct CachedTrackSource {
    path: String,
    size: u64,
    modified: u64,
}

#[derive(Debug, Clone, PartialEq, Default, Encode, Decode)]
enum CachedPlaylistSource {
    User,
    #[default]
    Folder,
    Generated,
}

#[derive(Encode, Decode)]
struct CachedImage {
    width: u32,
    height: u32,
    image: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Default, Encode, Decode)]
struct CachedPlaylist {
    pub id: String,
    pub name: String,
    pub source: CachedPlaylistSource,
    pub tracks: Vec<[u8; 16]>,

    pub folder_path: Option<String>,

    pub duration: u64,

    pub image_id: Option<[u8; 16]>,
}

#[derive(Debug, Clone, PartialEq, Default, Encode, Decode)]
struct CachedLibraryState {
    pub tracks: HashMap<[u8; 16], CachedTrack>,
    pub playlists: HashMap<String, CachedPlaylist>,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
struct CachedPlaybackState {
    pub current: Option<[u8; 16]>,
    pub current_playlist: Option<String>,
    pub current_index: usize,

    pub status: PlaybackStatus,
    pub position: u64,

    pub volume: f32,
    pub mute: bool,
    pub shuffling: bool,
    pub repeat: bool,
}

#[derive(Debug, Clone, PartialEq, Default, Encode, Decode)]
pub struct CachedQueueState {
    pub tracks: Vec<[u8; 16]>,
    pub order: Vec<usize>,
}

impl From<&Track> for CachedTrack {
    fn from(track: &Track) -> Self {
        Self {
            id: track.id.0,
            sources: track.sources.iter().map(Into::into).collect(),
            title: track.title.clone(),
            artist: track.artist.clone(),
            album: track.album.clone(),
            duration: track.duration,
            image_id: track.image_id.map(|id| id.0),
        }
    }
}

impl From<CachedTrack> for Track {
    fn from(c: CachedTrack) -> Self {
        Self {
            id: TrackId(c.id),
            sources: c.sources.iter().map(Into::into).collect(),
            title: c.title,
            artist: c.artist,
            album: c.album,
            duration: c.duration,
            image_id: c.image_id.map(ImageId),
        }
    }
}

impl From<&TrackSource> for CachedTrackSource {
    fn from(c: &TrackSource) -> Self {
        CachedTrackSource {
            path: c.path.to_string_lossy().to_string(),
            size: c.size,
            modified: c.modified,
        }
    }
}

impl From<&CachedTrackSource> for TrackSource {
    fn from(c: &CachedTrackSource) -> Self {
        TrackSource {
            path: PathBuf::from(c.path.clone()),
            size: c.size,
            modified: c.modified,
        }
    }
}

impl From<&Playlist> for CachedPlaylist {
    fn from(playlist: &Playlist) -> Self {
        CachedPlaylist {
            id: playlist.id.0.to_string(),
            name: playlist.name.clone(),
            source: match playlist.source {
                PlaylistSource::Folder => CachedPlaylistSource::Folder,
                PlaylistSource::Generated => CachedPlaylistSource::Generated,
                PlaylistSource::User => CachedPlaylistSource::User,
            },
            folder_path: playlist
                .folder_path
                .clone()
                .map(|path| path.to_string_lossy().to_string()),
            tracks: playlist.tracks.iter().map(|t| t.0).collect(),
            duration: playlist.duration.as_secs(),
            image_id: playlist.image_id.map(|id| id.0),
        }
    }
}

impl From<CachedPlaylist> for Playlist {
    fn from(cached_playlist: CachedPlaylist) -> Self {
        Playlist {
            id: PlaylistId(Uuid::from_str(cached_playlist.id.as_str()).unwrap_or_default()),
            name: cached_playlist.name,
            source: match cached_playlist.source {
                CachedPlaylistSource::Folder => PlaylistSource::Folder,
                CachedPlaylistSource::Generated => PlaylistSource::Generated,
                CachedPlaylistSource::User => PlaylistSource::User,
            },
            folder_path: cached_playlist.folder_path.map(PathBuf::from),
            tracks: cached_playlist.tracks.iter().map(|t| TrackId(*t)).collect(),
            duration: Duration::from_secs(cached_playlist.duration),
            image_id: cached_playlist.image_id.map(ImageId),
        }
    }
}

impl From<&LibraryState> for CachedLibraryState {
    fn from(state: &LibraryState) -> Self {
        let tracks = state
            .tracks
            .iter()
            .map(|(id, track)| (id.0, CachedTrack::from(track.as_ref())))
            .collect();

        let playlists = state
            .playlists
            .iter()
            .map(|(id, playlist)| (id.0.to_string(), CachedPlaylist::from(playlist)))
            .collect();

        Self { tracks, playlists }
    }
}

impl From<CachedLibraryState> for LibraryState {
    fn from(cache: CachedLibraryState) -> Self {
        let tracks = cache
            .tracks
            .into_iter()
            .map(|(id, track)| {
                let track: Track = track.into();
                (TrackId(id), Arc::new(track))
            })
            .collect();

        let playlists = cache
            .playlists
            .into_iter()
            .map(|(id, playlist)| {
                let playlist: Playlist = playlist.into();
                (
                    PlaylistId(Uuid::from_str(id.as_str()).unwrap_or_default()),
                    playlist,
                )
            })
            .collect();

        Self { tracks, playlists }
    }
}

impl From<&PlaybackState> for CachedPlaybackState {
    fn from(p: &PlaybackState) -> Self {
        Self {
            current: p.current.map(|id| id.0),
            current_playlist: p.current_playlist.map(|id| id.0.to_string()),
            current_index: p.current_index,
            status: p.status,
            position: p.position,
            volume: p.volume,
            mute: p.mute,
            shuffling: p.shuffling,
            repeat: p.repeat,
        }
    }
}

impl From<CachedPlaybackState> for PlaybackState {
    fn from(c: CachedPlaybackState) -> Self {
        Self {
            current: c.current.map(TrackId),
            current_playlist: c
                .current_playlist
                .map(|s| PlaylistId(Uuid::from_str(&s).unwrap_or_default())),
            current_index: c.current_index,
            status: c.status,
            position: c.position,
            volume: c.volume,
            mute: c.mute,
            shuffling: c.shuffling,
            repeat: c.repeat,
        }
    }
}

impl From<&QueueState> for CachedQueueState {
    fn from(q: &QueueState) -> Self {
        Self {
            tracks: q.tracks.iter().map(|id| id.0).collect(),
            order: q.order.clone(),
        }
    }
}

impl From<CachedQueueState> for QueueState {
    fn from(c: CachedQueueState) -> Self {
        Self {
            tracks: c.tracks.into_iter().map(TrackId).collect(),
            order: c.order,
        }
    }
}

impl Cacher {
    pub fn new(app_paths: AppPaths) -> (Self, Sender<CacherCommand>, Receiver<CacherEvent>) {
        let (cmd_tx, cmd_rx) = crossbeam_channel::unbounded();
        let (event_tx, event_rx) = crossbeam_channel::unbounded();

        let cacher = Cacher {
            tx: event_tx,
            rx: cmd_rx,
            app_paths,
        };

        (cacher, cmd_tx, event_rx)
    }

    pub fn run(&self, workers: usize) -> Result<(), CacherError> {
        let (app_state_tx, app_state_rx) = crossbeam_channel::unbounded();
        let (thumb_tx, thumb_rx) = crossbeam_channel::unbounded();
        let (album_art_tx, album_art_rx) = crossbeam_channel::unbounded();
        let (playlist_thumbnail_tx, playlist_thumbnail_rx) = crossbeam_channel::unbounded();

        self.spawn_app_state_worker(app_state_rx);
        self.spawn_thumbnail_workers(&thumb_rx, workers);
        self.spawn_album_art_worker(album_art_rx);
        self.spawn_playlist_thumbnail_worker(playlist_thumbnail_rx);

        loop {
            match self.rx.recv()? {
                CacherCommand::WriteLibraryState(state) => {
                    let _ = app_state_tx.send(CacheJob::WriteLibraryState(state));
                }
                CacherCommand::WritePlaybackState(state) => {
                    let _ = app_state_tx.send(CacheJob::WritePlaybackState(state));
                }
                CacherCommand::WriteQueueState(state) => {
                    let _ = app_state_tx.send(CacheJob::WriteQueueState(state));
                }
                CacherCommand::WriteImage {
                    id,
                    kind,
                    width,
                    height,
                    image,
                } => match kind {
                    ImageKind::AlbumArt => {
                        let _ = album_art_tx.send(CacheJob::WriteImage {
                            id,
                            kind: ImageKind::AlbumArt,
                            width,
                            height,
                            image,
                        });
                    }
                    ImageKind::ThumbnailSmall => {
                        let _ = thumb_tx.send(CacheJob::WriteImage {
                            id,
                            kind: ImageKind::ThumbnailSmall,
                            width,
                            height,
                            image,
                        });
                    }
                    ImageKind::ThumbnailLarge => {
                        let _ = thumb_tx.send(CacheJob::WriteImage {
                            id,
                            kind: ImageKind::ThumbnailLarge,
                            width,
                            height,
                            image,
                        });
                    }
                    ImageKind::Playlist => {
                        let _ = playlist_thumbnail_tx.send(CacheJob::WriteImage {
                            id,
                            kind: ImageKind::Playlist,
                            width,
                            height,
                            image,
                        });
                    }
                },
                CacherCommand::GetAppState => {
                    let _ = app_state_tx.send(CacheJob::LoadAppState);
                }
                CacherCommand::GetImage(ids, kind) => match kind {
                    ImageKind::ThumbnailSmall => {
                        let _ =
                            thumb_tx.send(CacheJob::LoadThumbnails(ids, ImageKind::ThumbnailSmall));
                    }
                    ImageKind::ThumbnailLarge => {
                        let _ =
                            thumb_tx.send(CacheJob::LoadThumbnails(ids, ImageKind::ThumbnailLarge));
                    }
                    ImageKind::AlbumArt => {
                        for id in ids {
                            let _ = album_art_tx.send(CacheJob::LoadAlbumArt(id));
                        }
                    }
                    ImageKind::Playlist => {
                        for id in ids {
                            let _ = playlist_thumbnail_tx.send(CacheJob::LoadPlaylistThumbnail(id));
                        }
                    }
                },
            }
        }
    }

    fn write_library_state(&self, state: &LibraryState) -> Result<(), CacherError> {
        let tmp_path = self.app_paths.cache.join("library.tmp");
        let final_path = self.app_paths.cache.join("library.bin");

        let library = CachedLibraryState::from(state);

        write_cache(&tmp_path, &final_path, library)?;

        Ok(())
    }

    fn write_playback_state(&self, state: &PlaybackState) -> Result<(), CacherError> {
        let tmp_path = self.app_paths.cache.join("session.tmp");
        let final_path = self.app_paths.cache.join("session.ron");

        let payload = CachedPlaybackState::from(state);

        let ron = ron::ser::to_string_pretty(&payload, PrettyConfig::default())?;

        {
            let mut file = fs::File::create(tmp_path.clone())?;
            file.write_all(ron.as_bytes())?;
            file.sync_all()?;
        }

        fs::rename(tmp_path, final_path)?;

        Ok(())
    }

    fn write_queue_state(&self, state: &QueueState) -> Result<(), CacherError> {
        let tmp_path = self.app_paths.cache.join("queue.tmp");
        let final_path = self.app_paths.cache.join("queue.bin");

        let queue = CachedQueueState::from(state);

        write_cache(&tmp_path, &final_path, queue)?;

        Ok(())
    }

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
        kind: &ImageKind,
        cached_image: &CachedImage,
    ) -> Result<(), CacherError> {
        let final_path = self.cached_image_path(id, *kind);
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

    fn load_app_state(&self) -> Result<AppState, CacherError> {
        let playback = self.read_playback_state()?;
        let library = self.read_library_state()?;
        let queue = self.read_queue_state()?;

        Ok(AppState {
            playback,
            library,
            queue,
        })
    }

    fn read_library_state(&self) -> Result<LibraryState, CacherError> {
        let path = self.app_paths.cache.join("library.bin");

        if !path.exists() {
            return Ok(LibraryState::default());
        }

        match read_cache::<CachedLibraryState>(&path)? {
            Some(cached_state) => Ok(LibraryState::from(cached_state)),
            None => Ok(LibraryState::default()),
        }
    }

    fn read_queue_state(&self) -> Result<QueueState, CacherError> {
        let path = self.app_paths.cache.join("queue.bin");

        if !path.exists() {
            return Ok(QueueState::default());
        }

        match read_cache::<CachedQueueState>(&path)? {
            Some(cached_state) => Ok(QueueState::from(cached_state)),
            None => Ok(QueueState::default()),
        }
    }

    fn read_playback_state(&self) -> Result<PlaybackState, CacherError> {
        let path = self.app_paths.cache.join("session.ron");

        if !path.exists() {
            return Ok(PlaybackState::default());
        }

        let ron = fs::read_to_string(path)?;
        let cached: CachedPlaybackState = ron::de::from_str(&ron)?;

        Ok(cached.into())
    }

    fn read_cached_image(
        &self,
        id: ImageId,
        kind: &ImageKind,
    ) -> Result<Option<Arc<RenderImage>>, CacherError> {
        let path = self.cached_image_path(id, *kind);

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

    fn spawn_app_state_worker(&self, rx: Receiver<CacheJob>) {
        let cacher = self.clone();

        std::thread::spawn(move || {
            loop {
                while let Ok(job) = rx.recv() {
                    let result: Result<(), CacherError> = (|| {
                        match job {
                            CacheJob::WriteLibraryState(state) => {
                                cacher.write_library_state(&state)?;
                            }
                            CacheJob::WritePlaybackState(state) => {
                                cacher.write_playback_state(&state)?;
                            }
                            CacheJob::WriteQueueState(state) => {
                                cacher.write_queue_state(&state)?;
                            }
                            CacheJob::LoadAppState => {
                                let state = cacher.load_app_state()?;
                                let _ = cacher.tx.send(CacherEvent::AppState(state));
                            }

                            _ => {}
                        }

                        Ok(())
                    })();

                    if let Err(err) = result {
                        eprintln!("Error occurred: {err:#?}");
                    }
                }
            }
        });
    }

    fn spawn_thumbnail_workers(&self, rx: &Receiver<CacheJob>, workers: usize) {
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
                                    match cacher.write_cached_image(id, &kind, &cached_image) {
                                        Ok(()) => {}
                                        Err(err) => {eprintln!("Error occurred: {err:#?}");}
                                    }
                                }
                                Ok(CacheJob::LoadThumbnails(ids, kind)) => {
                                    for id in ids {
                                        match cacher.read_cached_image(id, &kind) {
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

    fn spawn_album_art_worker(&self, rx: Receiver<CacheJob>) {
        let cacher = Arc::new(self.clone());

        std::thread::spawn(move || {
            while let Ok(job) = rx.recv() {
                match job {
                    CacheJob::LoadAlbumArt(id) => {
                        match cacher.read_cached_image(id, &ImageKind::AlbumArt) {
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
                        match cacher.write_cached_image(id, &kind, &cached_image) {
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

    fn spawn_playlist_thumbnail_worker(&self, rx: Receiver<CacheJob>) {
        let cacher = Arc::new(self.clone());

        std::thread::spawn(move || {
            while let Ok(job) = rx.recv() {
                match job {
                    CacheJob::LoadPlaylistThumbnail(id) => {
                        match cacher.read_cached_image(id, &ImageKind::Playlist) {
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
                        match cacher.write_cached_image(id, &kind, &cached_image) {
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

fn write_cache<T: Encode>(
    tmp: &PathBuf,
    final_path: &PathBuf,
    payload: T,
) -> Result<(), CacherError> {
    let wrapped = CacheFile {
        version: 1,
        payload,
    };

    let bytes = bitcode::encode(&wrapped);

    {
        let mut file = fs::File::create(tmp)?;
        file.write_all(&bytes)?;
        file.sync_all()?;
    }

    fs::rename(tmp, final_path)?;

    Ok(())
}

fn read_cache<T>(path: &PathBuf) -> Result<Option<T>, CacherError>
where
    T: for<'a> Decode<'a>,
{
    if !path.exists() {
        return Ok(None);
    }

    let bytes = fs::read(path)?;

    let file: CacheFile<T> = bitcode::decode(&bytes)?;

    if file.version != 1 {
        return Ok(None);
    }

    Ok(Some(file.payload))
}
