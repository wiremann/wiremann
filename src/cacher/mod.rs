use crate::controller::commands::CacherCommand;
use crate::controller::events::CacherEvent;
use crate::controller::state::{AppState, LibraryState, PlaybackState, PlaybackStatus, QueueState};
use crate::errors::CacherError;
use crate::library::playlists::{Playlist, PlaylistId, PlaylistSource};
use crate::library::{gen_track_id, Track, TrackId};
use bitcode::{Decode, Encode};
use crossbeam_channel::{select, tick, Receiver, Sender};
use gpui::RenderImage;
use image::Frame;
use serde::{Deserialize, Serialize};
use smallvec::smallvec;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{Cursor, Write};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

#[derive(Clone)]
pub struct Cacher {
    pub tx: Sender<CacherEvent>,
    pub rx: Receiver<CacherCommand>,
    base_dir: PathBuf,
}

enum CacheJob {
    WriteLibraryState(LibraryState),
    WritePlaybackState(PlaybackState),
    WriteQueueState(QueueState),
    WriteImage {
        id: TrackId,
        kind: ImageKind,
        width: u32,
        height: u32,
        image: Vec<u8>,
    },
    LoadAppState,
    LoadThumbnails(HashSet<TrackId>),
    LoadAlbumArt(PathBuf),
}

#[derive(Encode, Decode)]
struct CacheFile<T> {
    version: u32,
    payload: T,
}

pub enum ImageKind {
    Thumbnail,
    AlbumArt,
}
#[derive(Debug, Clone, PartialEq, Default, Encode, Decode)]
struct CachedTrack {
    pub id: [u8; 32],
    pub path: String,

    pub title: String,
    pub artist: String,
    pub album: String,

    pub duration: u64,
    pub size: u64,
    pub modified: u64,
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
    pub tracks: Vec<[u8; 32]>,
}

#[derive(Debug, Clone, PartialEq, Default, Encode, Decode)]
struct CachedLibraryState {
    pub tracks: HashMap<[u8; 32], CachedTrack>,
    pub playlists: HashMap<String, CachedPlaylist>,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
struct CachedPlaybackState {
    pub current: Option<[u8; 32]>,
    pub current_playlist: Option<String>,

    pub status: PlaybackStatus,
    pub position: u64,

    pub volume: f32,
    pub mute: bool,
    pub shuffling: bool,
    pub repeat: bool,
}

#[derive(Debug, Clone, PartialEq, Default, Encode, Decode)]
pub struct CachedQueueState {
    pub tracks: Vec<[u8; 32]>,
    pub order: Vec<usize>,
    pub index: usize,
}

impl From<&Track> for CachedTrack {
    fn from(track: &Track) -> Self {
        Self {
            id: track.id.0,
            path: track.path.to_string_lossy().to_string(),
            title: track.title.clone(),
            artist: track.artist.clone(),
            album: track.album.clone(),
            duration: track.duration,
            size: track.size,
            modified: track.modified,
        }
    }
}

impl From<CachedTrack> for Track {
    fn from(c: CachedTrack) -> Self {
        Self {
            id: TrackId(c.id),
            path: PathBuf::from(c.path),
            title: c.title,
            artist: c.artist,
            album: c.album,
            duration: c.duration,
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
            tracks: playlist.tracks.iter().map(|t| t.0).collect(),
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
            tracks: cached_playlist.tracks.iter().map(|t| TrackId(*t)).collect(),
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
                .and_then(|s| Some(PlaylistId(Uuid::from_str(&s).unwrap_or_default()))),
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
            index: q.index,
        }
    }
}

impl From<CachedQueueState> for QueueState {
    fn from(c: CachedQueueState) -> Self {
        Self {
            tracks: c.tracks.into_iter().map(TrackId).collect(),
            order: c.order,
            index: c.index,
        }
    }
}

impl Cacher {
    pub fn new() -> (Self, Sender<CacherCommand>, Receiver<CacherEvent>) {
        let (cmd_tx, cmd_rx) = crossbeam_channel::unbounded();
        let (event_tx, event_rx) = crossbeam_channel::unbounded();

        let base_dir = dirs::audio_dir()
            .unwrap_or_default()
            .join("wiremann")
            .join("cache");
        fs::create_dir_all(base_dir.clone()).expect("failed to create cache directory");

        let cacher = Cacher {
            tx: event_tx,
            rx: cmd_rx,
            base_dir,
        };

        (cacher, cmd_tx, event_rx)
    }

    pub fn run(&self) -> Result<(), CacherError> {
        let (app_state_tx, app_state_rx) = crossbeam_channel::unbounded();
        let (thumb_tx, thumb_rx) = crossbeam_channel::unbounded();
        let (album_art_tx, album_art_rx) = crossbeam_channel::unbounded();

        self.spawn_app_state_worker(app_state_rx)?;
        self.spawn_thumbnail_workers(thumb_rx)?;
        self.spawn_album_art_worker(album_art_rx)?;

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
                    ImageKind::Thumbnail => {
                        let _ = thumb_tx.send(CacheJob::WriteImage {
                            id,
                            kind: ImageKind::Thumbnail,
                            width,
                            height,
                            image,
                        });
                    }
                },
                CacherCommand::GetAppState => {
                    let _ = app_state_tx.send(CacheJob::LoadAppState);
                }
                CacherCommand::GetThumbnails(ids) => {
                    let _ = thumb_tx.send(CacheJob::LoadThumbnails(ids));
                }
                CacherCommand::GetAlbumArt(path) => {
                    let _ = album_art_tx.send(CacheJob::LoadAlbumArt(path));
                }
            }
        }
    }

    fn write_library_state(&self, state: &LibraryState) -> Result<(), CacherError> {
        let tmp_path = self.base_dir.join("library.tmp");
        let final_path = self.base_dir.join("library.bin");

        let library = CachedLibraryState::from(state);

        write_cache(&tmp_path, &final_path, library)?;

        Ok(())
    }

    fn write_playback_state(&self, state: &PlaybackState) -> Result<(), CacherError> {
        let tmp_path = self.base_dir.join("session.tmp");
        let final_path = self.base_dir.join("session.ron");

        let payload = CachedPlaybackState::from(state);

        let ron = ron::ser::to_string_pretty(&payload, Default::default())?;

        {
            let mut file = fs::File::create(tmp_path.clone())?;
            file.write_all(&ron.as_bytes())?;
            file.sync_all()?;
        }

        fs::rename(tmp_path, final_path)?;

        Ok(())
    }

    fn write_queue_state(&self, state: &QueueState) -> Result<(), CacherError> {
        let tmp_path = self.base_dir.join("queue.tmp");
        let final_path = self.base_dir.join("queue.bin");

        let queue = CachedQueueState::from(state);

        write_cache(&tmp_path, &final_path, queue)?;

        Ok(())
    }

    fn cached_image_path(&self, id: TrackId, kind: ImageKind) -> PathBuf {
        let hex = hex::encode(id.0);
        let folder = &hex[0..2];

        let name = match kind {
            ImageKind::Thumbnail => format!("{hex}_thumb.bgra.zstd"),
            ImageKind::AlbumArt => format!("{hex}_art.bgra.zstd"),
        };

        self.base_dir.join("images").join(folder).join(name)
    }

    fn write_cached_image(
        &self,
        id: TrackId,
        kind: ImageKind,
        cached_image: CachedImage,
    ) -> Result<(), CacherError> {
        let final_path = self.cached_image_path(id, kind);
        let tmp_path = final_path.with_extension("tmp");

        if final_path.exists() {
            return Ok(());
        }

        fs::create_dir_all(final_path.parent().unwrap())?;

        let bytes = bitcode::encode(&cached_image);

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
        let library = self.read_library_state()?;
        let playback = self.read_playback_state()?;
        let queue = self.read_queue_state()?;

        Ok(AppState {
            library,
            playback,
            queue,
        })
    }

    fn read_library_state(&self) -> Result<LibraryState, CacherError> {
        let path = self.base_dir.join("library").join("tracks.bin");

        if !path.exists() {
            return Ok(LibraryState::default());
        }

        match read_cache::<CachedLibraryState>(&path)? {
            Some(cached_state) => Ok(LibraryState::from(cached_state)),
            None => Ok(LibraryState::default()),
        }
    }

    fn read_queue_state(&self) -> Result<QueueState, CacherError> {
        let path = self.base_dir.join("queue.bin");

        if !path.exists() {
            return Ok(QueueState::default());
        }

        match read_cache::<CachedQueueState>(&path)? {
            Some(cached_state) => Ok(QueueState::from(cached_state)),
            None => Ok(QueueState::default()),
        }
    }

    fn read_playback_state(&self) -> Result<PlaybackState, CacherError> {
        let path = self.base_dir.join("session.bin");

        if !path.exists() {
            return Ok(PlaybackState::default());
        }

        let ron = fs::read_to_string(path)?;
        let cached: CachedPlaybackState = ron::de::from_str(&ron)?;

        Ok(cached.into())
    }

    fn read_cached_image(
        &self,
        id: TrackId,
        kind: ImageKind,
    ) -> Result<Option<Arc<RenderImage>>, CacherError> {
        let path = self.cached_image_path(id.clone(), kind);

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

    fn spawn_app_state_worker(&self, rx: Receiver<CacheJob>) -> Result<(), CacherError> {
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
                        eprintln!("Error occurred: {:#?}", err);
                    }
                }
            }
        });

        Ok(())
    }

    fn spawn_thumbnail_workers(&self, rx: Receiver<CacheJob>) -> Result<(), CacherError> {
        let ticker = tick(Duration::from_millis(128));
        let threads = num_cpus::get() - 2;

        for _ in 0..threads {
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
                                    match cacher.write_cached_image(id, kind, cached_image) {
                                        Ok(_) => {}
                                        Err(err) => {eprintln!("Error occurred: {:#?}", err);}
                                    }
                                }
                                Ok(CacheJob::LoadThumbnails(ids)) => {
                                    for id in ids {
                                        match cacher.read_cached_image(id, ImageKind::Thumbnail) {
                                            Ok(Some(image)) => {batch.insert(id, image);},
                                            Ok(None) | Err(_) => {missing.push(id);},
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

        Ok(())
    }

    fn spawn_album_art_worker(&self, rx: Receiver<CacheJob>) -> Result<(), CacherError> {
        let cacher = Arc::new(self.clone().to_owned());

        std::thread::spawn(move || {
            while let Ok(job) = rx.recv() {
                match job {
                    CacheJob::LoadAlbumArt(path) => {
                        if let Ok(id) = gen_track_id(&path) {
                            match cacher.read_cached_image(id, ImageKind::AlbumArt) {
                                Ok(Some(image)) => {
                                    let _ = cacher.tx.send(CacherEvent::AlbumArt(image));
                                }
                                Err(e) => {
                                    eprintln!("Error loading album art: {}", e);
                                    let _ = cacher.tx.send(CacherEvent::MissingAlbumArt(path));
                                }
                                _ => {
                                    let _ = cacher.tx.send(CacherEvent::MissingAlbumArt(path));
                                }
                            }
                        } else {
                            let _ = cacher.tx.send(CacherEvent::MissingAlbumArt(path));
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
                        match cacher.write_cached_image(id, kind, cached_image) {
                            Ok(_) => {}
                            Err(err) => {
                                eprintln!("Error occurred: {:#?}", err);
                            }
                        }
                    }
                    _ => {}
                }
            }
        });

        Ok(())
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
