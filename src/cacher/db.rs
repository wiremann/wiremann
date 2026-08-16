use crate::controller::state::{
    Album, AlbumId, Artist, ArtistId, ImageId, LibraryState, ListenMetrics, Playlist,
    PlaylistId, PlaylistSource, QueueState, Track, TrackId, TrackListenMetrics, TrackSource,
};
use crate::errors::CacherError;
use sqlx::{Row, Sqlite, SqlitePool, Transaction};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

/// Operations against the on-disk SQLite database that backs all data which
/// grows with use: the library, queue, favorites and listen metrics.
///
/// The playback session (`session.ron`) and image/lyrics blob caches are kept
/// on disk as files rather than in the DB.
pub struct Db {
    pool: SqlitePool,
}

fn track_hex(id: TrackId) -> String {
    hex::encode(id.0)
}

fn image_hex(id: ImageId) -> String {
    hex::encode(id.0)
}

fn from_track_hex(s: &str) -> TrackId {
    let mut arr = [0u8; 16];
    let _ = hex::decode_to_slice(s, &mut arr);
    TrackId(arr)
}

fn from_image_hex(s: &str) -> ImageId {
    let mut arr = [0u8; 16];
    let _ = hex::decode_to_slice(s, &mut arr);
    ImageId(arr)
}

fn album_hex(id: AlbumId) -> String {
    format!("{:016x}", id.0)
}

fn artist_hex(id: ArtistId) -> String {
    format!("{:016x}", id.0)
}

fn from_album_hex(s: &str) -> AlbumId {
    AlbumId(u64::from_str_radix(s, 16).unwrap_or_default())
}

fn from_artist_hex(s: &str) -> ArtistId {
    ArtistId(u64::from_str_radix(s, 16).unwrap_or_default())
}

fn source_str(source: &PlaylistSource) -> &'static str {
    match source {
        PlaylistSource::User => "user",
        PlaylistSource::Folder => "folder",
        PlaylistSource::Generated => "generated",
    }
}

fn source_from_str(s: &str) -> PlaylistSource {
    match s {
        "user" => PlaylistSource::User,
        "generated" => PlaylistSource::Generated,
        _ => PlaylistSource::Folder,
    }
}

impl Db {
    /// Opens (creating if necessary) the database at `cache_dir/wiremann.db`.
    pub async fn connect(cache_dir: &Path) -> Result<Self, CacherError> {
        let url = cache_dir.join("wiremann.db");
        let options = sqlx::sqlite::SqliteConnectOptions::new()
            .filename(&url)
            .create_if_missing(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
            .synchronous(sqlx::sqlite::SqliteSynchronous::Normal)
            .foreign_keys(true);

        let pool = SqlitePool::connect_with(options).await?;

        let db = Self { pool };
        db.init_schema().await?;
        db.migrate_from_legacy(cache_dir).await?;
        Ok(db)
    }

    /// One-time import of the pre-SQLite bitcode caches. When the DB has no
    /// tracks but the old `*.bin` files exist, decode them and populate the
    /// tables, then remove the legacy files so a stale `scan_record.bin` no
    /// longer suppresses a future scan.
    async fn migrate_from_legacy(&self, cache_dir: &Path) -> Result<(), CacherError> {
        let track_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM tracks")
            .fetch_one(&self.pool)
            .await?;

        if track_count > 0 {
            return Ok(());
        }

        let library_bin = cache_dir.join("library.bin");
        if !library_bin.exists() {
            return Ok(());
        }

        if let Some(library) = crate::cacher::legacy::read_legacy_library(cache_dir) {
            tracing::info!(tracks = library.tracks.len(), "migrating legacy library into SQLite");
            self.write_library(&library).await?;
            std::fs::remove_file(&library_bin).ok();
        }

        if let Some(queue) = crate::cacher::legacy::read_legacy_queue(cache_dir) {
            self.write_queue(&queue).await?;
            std::fs::remove_file(cache_dir.join("queue.bin")).ok();
        }

        if let Some(favorites) = crate::cacher::legacy::read_legacy_favorites(cache_dir) {
            self.write_favorites(&favorites).await?;
            std::fs::remove_file(cache_dir.join("favorites.bin")).ok();
        }

        if let Some(metrics) = crate::cacher::legacy::read_legacy_metrics(cache_dir) {
            self.write_metrics(&metrics).await?;
            std::fs::remove_file(cache_dir.join("metrics.bin")).ok();
        }

        Ok(())
    }

    async fn init_schema(&self) -> Result<(), CacherError> {
        sqlx::query(
            "
            CREATE TABLE IF NOT EXISTS tracks (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                album TEXT NOT NULL,
                duration_ms INTEGER NOT NULL,
                image_id TEXT
            );
            CREATE TABLE IF NOT EXISTS track_sources (
                track_id TEXT NOT NULL,
                position INTEGER NOT NULL,
                path TEXT NOT NULL,
                size INTEGER NOT NULL,
                modified INTEGER NOT NULL,
                PRIMARY KEY (track_id, position)
            );
            CREATE TABLE IF NOT EXISTS track_artists (
                track_id TEXT NOT NULL,
                position INTEGER NOT NULL,
                artist_id TEXT NOT NULL,
                PRIMARY KEY (track_id, position)
            );
            CREATE TABLE IF NOT EXISTS artists (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                image_id TEXT
            );
            CREATE TABLE IF NOT EXISTS albums (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                duration_ms INTEGER NOT NULL,
                image_id TEXT
            );
            CREATE TABLE IF NOT EXISTS album_artists (
                album_id TEXT NOT NULL,
                position INTEGER NOT NULL,
                artist_id TEXT NOT NULL,
                PRIMARY KEY (album_id, position)
            );
            CREATE TABLE IF NOT EXISTS artist_tracks (
                artist_id TEXT NOT NULL,
                position INTEGER NOT NULL,
                track_id TEXT NOT NULL,
                PRIMARY KEY (artist_id, position)
            );
            CREATE TABLE IF NOT EXISTS artist_albums (
                artist_id TEXT NOT NULL,
                position INTEGER NOT NULL,
                album_id TEXT NOT NULL,
                PRIMARY KEY (artist_id, position)
            );
            CREATE TABLE IF NOT EXISTS album_tracks (
                album_id TEXT NOT NULL,
                position INTEGER NOT NULL,
                track_id TEXT NOT NULL,
                PRIMARY KEY (album_id, position)
            );
            CREATE TABLE IF NOT EXISTS playlists (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                source TEXT NOT NULL,
                folder_path TEXT,
                duration_secs INTEGER NOT NULL,
                image_id TEXT
            );
            CREATE TABLE IF NOT EXISTS playlist_tracks (
                playlist_id TEXT NOT NULL,
                position INTEGER NOT NULL,
                track_id TEXT NOT NULL,
                PRIMARY KEY (playlist_id, position)
            );
            CREATE TABLE IF NOT EXISTS queue_tracks (
                position INTEGER PRIMARY KEY,
                track_id TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS queue_order (
                position INTEGER PRIMARY KEY,
                order_index INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS favorites (
                position INTEGER PRIMARY KEY,
                track_id TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS track_metrics (
                track_id TEXT PRIMARY KEY,
                play_count INTEGER NOT NULL,
                play_time_secs INTEGER NOT NULL,
                first_played INTEGER,
                last_played INTEGER,
                skip_count INTEGER NOT NULL
            );
            ",
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn write_library(&self, state: &LibraryState) -> Result<(), CacherError> {
        let mut tx = self.pool.begin().await?;

        self.clear_library(&mut tx).await?;

        // Artists
        let mut artist_ids = state.artists.values().collect::<Vec<_>>();
        artist_ids.sort_by_key(|a| a.id.0);
        for artist in artist_ids {
            sqlx::query(
                "INSERT INTO artists (id, name, image_id) VALUES (?1, ?2, ?3)",
            )
            .bind(artist_hex(artist.id))
            .bind(artist.name.to_string())
            .bind(artist.image_id.map(image_hex))
            .execute(&mut *tx)
            .await?;
        }

        // Albums
        let mut album_ids = state.albums.values().collect::<Vec<_>>();
        album_ids.sort_by_key(|a| a.id.0);
        for album in album_ids {
            sqlx::query(
                "INSERT INTO albums (id, name, duration_ms, image_id) VALUES (?1, ?2, ?3, ?4)",
            )
            .bind(album_hex(album.id))
            .bind(album.name.to_string())
            .bind(album.duration.as_millis() as i64)
            .bind(album.image_id.map(image_hex))
            .execute(&mut *tx)
            .await?;

            for (pos, artist) in album.artists.iter().enumerate() {
                sqlx::query(
                    "INSERT INTO album_artists (album_id, position, artist_id) VALUES (?1, ?2, ?3)",
                )
                .bind(album_hex(album.id))
                .bind(pos as i64)
                .bind(artist_hex(*artist))
                .execute(&mut *tx)
                .await?;
            }

            for (pos, track) in album.tracks.iter().enumerate() {
                sqlx::query(
                    "INSERT INTO album_tracks (album_id, position, track_id) VALUES (?1, ?2, ?3)",
                )
                .bind(album_hex(album.id))
                .bind(pos as i64)
                .bind(track_hex(*track))
                .execute(&mut *tx)
                .await?;
            }
        }

        // Tracks
        let mut track_ids = state.tracks.values().collect::<Vec<_>>();
        track_ids.sort_by_key(|t| t.id.0);
        for track in track_ids {
            sqlx::query(
                "INSERT INTO tracks (id, title, album, duration_ms, image_id) VALUES (?1, ?2, ?3, ?4, ?5)",
            )
            .bind(track_hex(track.id))
            .bind(track.title.to_string())
            .bind(album_hex(track.album))
            .bind(track.duration.as_millis() as i64)
            .bind(track.image_id.map(image_hex))
            .execute(&mut *tx)
            .await?;

            for (pos, source) in track.sources.iter().enumerate() {
                sqlx::query(
                    "INSERT INTO track_sources (track_id, position, path, size, modified) VALUES (?1, ?2, ?3, ?4, ?5)",
                )
                .bind(track_hex(track.id))
                .bind(pos as i64)
                .bind(source.path.to_string_lossy().to_string())
                .bind(source.size as i64)
                .bind(source.modified as i64)
                .execute(&mut *tx)
                .await?;
            }

            for (pos, artist) in track.artists.iter().enumerate() {
                sqlx::query(
                    "INSERT INTO track_artists (track_id, position, artist_id) VALUES (?1, ?2, ?3)",
                )
                .bind(track_hex(track.id))
                .bind(pos as i64)
                .bind(artist_hex(*artist))
                .execute(&mut *tx)
                .await?;
            }
        }

        // Artist track/album assignments
        let mut artist_ids = state.artists.values().collect::<Vec<_>>();
        artist_ids.sort_by_key(|a| a.id.0);
        for artist in artist_ids {
            for (pos, track) in artist.tracks.iter().enumerate() {
                sqlx::query(
                    "INSERT INTO artist_tracks (artist_id, position, track_id) VALUES (?1, ?2, ?3)",
                )
                .bind(artist_hex(artist.id))
                .bind(pos as i64)
                .bind(track_hex(*track))
                .execute(&mut *tx)
                .await?;
            }

            for (pos, album) in artist.albums.iter().enumerate() {
                sqlx::query(
                    "INSERT INTO artist_albums (artist_id, position, album_id) VALUES (?1, ?2, ?3)",
                )
                .bind(artist_hex(artist.id))
                .bind(pos as i64)
                .bind(album_hex(*album))
                .execute(&mut *tx)
                .await?;
            }
        }

        // Playlists
        let mut playlist_ids = state.playlists.values().collect::<Vec<_>>();
        playlist_ids.sort_by_key(|p| p.id.0.as_u128());
        for playlist in playlist_ids {
            sqlx::query(
                "INSERT INTO playlists (id, name, source, folder_path, duration_secs, image_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            )
            .bind(playlist.id.0.to_string())
            .bind(playlist.name.to_string())
            .bind(source_str(&playlist.source))
            .bind(playlist.folder_path.as_ref().map(|p| p.to_string_lossy().to_string()))
            .bind(playlist.duration.as_secs() as i64)
            .bind(playlist.image_id.map(image_hex))
            .execute(&mut *tx)
            .await?;

            for (pos, track) in playlist.tracks.iter().enumerate() {
                sqlx::query(
                    "INSERT INTO playlist_tracks (playlist_id, position, track_id) VALUES (?1, ?2, ?3)",
                )
                .bind(playlist.id.0.to_string())
                .bind(pos as i64)
                .bind(track_hex(*track))
                .execute(&mut *tx)
                .await?;
            }
        }

        tx.commit().await?;
        Ok(())
    }

    async fn clear_library(&self, tx: &mut Transaction<'_, Sqlite>) -> Result<(), CacherError> {
        for table in [
            "track_sources",
            "track_artists",
            "artist_albums",
            "artist_tracks",
            "album_tracks",
            "album_artists",
            "playlist_tracks",
            "playlists",
            "tracks",
            "albums",
            "artists",
        ] {
            sqlx::query(&format!("DELETE FROM {table}"))
                .execute(&mut **tx)
                .await?;
        }
        Ok(())
    }

    pub async fn load_library(&self) -> Result<LibraryState, CacherError> {
        let mut tracks: HashMap<TrackId, Arc<Track>> = HashMap::new();

        let rows = sqlx::query(
            "SELECT id, title, album, duration_ms, image_id FROM tracks",
        )
        .fetch_all(&self.pool)
        .await?;

        for row in rows {
            let id = from_track_hex(row.get::<String, _>("id").as_str());

            let image_id: Option<String> = row.get("image_id");
            let image_id = image_id.as_deref().map(from_image_hex);

            tracks.insert(
                id,
                Arc::new(Track {
                    id,
                    sources: Vec::new(),
                    title: row.get::<String, _>("title").into(),
                    artists: Vec::new(),
                    album: from_album_hex(row.get::<String, _>("album").as_str()),
                    duration: Duration::from_millis(row.get::<i64, _>("duration_ms") as u64),
                    image_id,
                }),
            );
        }

        // Sources
        let rows = sqlx::query(
            "SELECT track_id, path, size, modified FROM track_sources ORDER BY track_id, position",
        )
        .fetch_all(&self.pool)
        .await?;
        for row in rows {
            let track_id = from_track_hex(row.get::<String, _>("track_id").as_str());
            if let Some(track) = tracks.get_mut(&track_id) {
                Arc::make_mut(track).sources.push(TrackSource {
                    path: row.get::<String, _>("path").into(),
                    size: row.get::<i64, _>("size") as u64,
                    modified: row.get::<i64, _>("modified") as u64,
                });
            }
        }

        // Track-artists
        let rows = sqlx::query(
            "SELECT track_id, artist_id FROM track_artists ORDER BY track_id, position",
        )
        .fetch_all(&self.pool)
        .await?;
        for row in rows {
            let track_id = from_track_hex(row.get::<String, _>("track_id").as_str());
            let artist_id = from_artist_hex(row.get::<String, _>("artist_id").as_str());
            if let Some(track) = tracks.get_mut(&track_id) {
                Arc::make_mut(track).artists.push(artist_id);
            }
        }

        let mut artists: HashMap<ArtistId, Arc<Artist>> = HashMap::new();
        let rows = sqlx::query("SELECT id, name, image_id FROM artists").fetch_all(&self.pool).await?;
        for row in rows {
            let id = from_artist_hex(row.get::<String, _>("id").as_str());
            let image_id: Option<String> = row.get("image_id");
            artists.insert(
                id,
                Arc::new(Artist {
                    id,
                    name: row.get::<String, _>("name").into(),
                    tracks: Vec::new(),
                    albums: Vec::new(),
                    image_id: image_id.as_deref().map(from_image_hex),
                }),
            );
        }

        let rows = sqlx::query(
            "SELECT artist_id, track_id FROM artist_tracks ORDER BY artist_id, position",
        )
        .fetch_all(&self.pool)
        .await?;
        for row in rows {
            let artist_id = from_artist_hex(row.get::<String, _>("artist_id").as_str());
            let track_id = from_track_hex(row.get::<String, _>("track_id").as_str());
            if let Some(artist) = artists.get_mut(&artist_id) {
                Arc::make_mut(artist).tracks.push(track_id);
            }
        }

        let rows = sqlx::query(
            "SELECT artist_id, album_id FROM artist_albums ORDER BY artist_id, position",
        )
        .fetch_all(&self.pool)
        .await?;
        for row in rows {
            let artist_id = from_artist_hex(row.get::<String, _>("artist_id").as_str());
            let album_id = from_album_hex(row.get::<String, _>("album_id").as_str());
            if let Some(artist) = artists.get_mut(&artist_id) {
                Arc::make_mut(artist).albums.push(album_id);
            }
        }

        let mut albums: HashMap<AlbumId, Arc<Album>> = HashMap::new();
        let rows = sqlx::query(
            "SELECT id, name, duration_ms, image_id FROM albums",
        )
        .fetch_all(&self.pool)
        .await?;
        for row in rows {
            let id = from_album_hex(row.get::<String, _>("id").as_str());
            let image_id: Option<String> = row.get("image_id");
            albums.insert(
                id,
                Arc::new(Album {
                    id,
                    name: row.get::<String, _>("name").into(),
                    artists: Vec::new(),
                    duration: Duration::from_millis(row.get::<i64, _>("duration_ms") as u64),
                    tracks: Vec::new(),
                    image_id: image_id.as_deref().map(from_image_hex),
                }),
            );
        }

        let rows = sqlx::query(
            "SELECT album_id, artist_id FROM album_artists ORDER BY album_id, position",
        )
        .fetch_all(&self.pool)
        .await?;
        for row in rows {
            let album_id = from_album_hex(row.get::<String, _>("album_id").as_str());
            let artist_id = from_artist_hex(row.get::<String, _>("artist_id").as_str());
            if let Some(album) = albums.get_mut(&album_id) {
                Arc::make_mut(album).artists.push(artist_id);
            }
        }

        let rows = sqlx::query(
            "SELECT album_id, track_id FROM album_tracks ORDER BY album_id, position",
        )
        .fetch_all(&self.pool)
        .await?;
        for row in rows {
            let album_id = from_album_hex(row.get::<String, _>("album_id").as_str());
            let track_id = from_track_hex(row.get::<String, _>("track_id").as_str());
            if let Some(album) = albums.get_mut(&album_id) {
                Arc::make_mut(album).tracks.push(track_id);
            }
        }

        let mut playlists: HashMap<PlaylistId, Playlist> = HashMap::new();
        let rows = sqlx::query(
            "SELECT id, name, source, folder_path, duration_secs, image_id FROM playlists",
        )
        .fetch_all(&self.pool)
        .await?;
        for row in rows {
            let id = PlaylistId(uuid::Uuid::parse_str(row.get::<String, _>("id").as_str()).unwrap_or_default());
            let folder: Option<String> = row.get("folder_path");
            let image_id: Option<String> = row.get("image_id");
            playlists.insert(
                id,
                Playlist {
                    id,
                    name: row.get::<String, _>("name").into(),
                    source: source_from_str(row.get::<String, _>("source").as_str()),
                    folder_path: folder.map(Into::into),
                    duration: Duration::from_secs(row.get::<i64, _>("duration_secs") as u64),
                    tracks: Vec::new(),
                    image_id: image_id.as_deref().map(from_image_hex),
                },
            );
        }

        let rows = sqlx::query(
            "SELECT playlist_id, track_id FROM playlist_tracks ORDER BY playlist_id, position",
        )
        .fetch_all(&self.pool)
        .await?;
        for row in rows {
            let playlist_id = PlaylistId(uuid::Uuid::parse_str(row.get::<String, _>("playlist_id").as_str()).unwrap_or_default());
            let track_id = from_track_hex(row.get::<String, _>("track_id").as_str());
            if let Some(playlist) = playlists.get_mut(&playlist_id) {
                playlist.tracks.push(track_id);
            }
        }

        Ok(LibraryState {
            tracks,
            playlists,
            artists,
            albums,
        })
    }

    pub async fn write_queue(&self, state: &QueueState) -> Result<(), CacherError> {
        let mut tx = self.pool.begin().await?;
        sqlx::query("DELETE FROM queue_tracks").execute(&mut *tx).await?;
        sqlx::query("DELETE FROM queue_order").execute(&mut *tx).await?;

        for (pos, track) in state.tracks.iter().enumerate() {
            sqlx::query("INSERT INTO queue_tracks (position, track_id) VALUES (?1, ?2)")
                .bind(pos as i64)
                .bind(track_hex(*track))
                .execute(&mut *tx)
                .await?;
        }

        for (pos, index) in state.order.iter().enumerate() {
            sqlx::query("INSERT INTO queue_order (position, order_index) VALUES (?1, ?2)")
                .bind(pos as i64)
                .bind(*index as i64)
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    pub async fn load_queue(&self) -> Result<QueueState, CacherError> {
        let rows = sqlx::query("SELECT track_id FROM queue_tracks ORDER BY position")
            .fetch_all(&self.pool)
            .await?;

        let tracks = rows
            .iter()
            .map(|row| from_track_hex(row.get::<String, _>("track_id").as_str()))
            .collect::<Vec<_>>();

        let rows = sqlx::query("SELECT order_index FROM queue_order ORDER BY position")
            .fetch_all(&self.pool)
            .await?;

        let order = rows
            .iter()
            .map(|row| row.get::<i64, _>("order_index") as usize)
            .collect::<Vec<_>>();

        Ok(QueueState { tracks, order })
    }

    pub async fn write_favorites(&self, ids: &[TrackId]) -> Result<(), CacherError> {
        let mut tx = self.pool.begin().await?;
        sqlx::query("DELETE FROM favorites").execute(&mut *tx).await?;

        for (pos, id) in ids.iter().enumerate() {
            sqlx::query("INSERT INTO favorites (position, track_id) VALUES (?1, ?2)")
                .bind(pos as i64)
                .bind(track_hex(*id))
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    pub async fn load_favorites(&self) -> Result<Vec<TrackId>, CacherError> {
        let rows = sqlx::query("SELECT track_id FROM favorites ORDER BY position")
            .fetch_all(&self.pool)
            .await?;

        Ok(rows
            .iter()
            .map(|row| from_track_hex(row.get::<String, _>("track_id").as_str()))
            .collect())
    }

    pub async fn write_metrics(&self, metrics: &ListenMetrics) -> Result<(), CacherError> {
        let mut tx = self.pool.begin().await?;
        sqlx::query("DELETE FROM track_metrics").execute(&mut *tx).await?;

        let mut pairs = metrics.tracks.iter().collect::<Vec<_>>();
        pairs.sort_by_key(|(id, _)| id.0);

        for (id, m) in pairs {
            sqlx::query(
                "INSERT INTO track_metrics (track_id, play_count, play_time_secs, first_played, last_played, skip_count) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            )
            .bind(track_hex(*id))
            .bind(m.play_count as i64)
            .bind(m.play_time.as_secs() as i64)
            .bind(m.first_played.map(|v| v as i64))
            .bind(m.last_played.map(|v| v as i64))
            .bind(m.skip_count as i64)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    pub async fn load_metrics(&self) -> Result<ListenMetrics, CacherError> {
        let rows = sqlx::query(
            "SELECT track_id, play_count, play_time_secs, first_played, last_played, skip_count FROM track_metrics",
        )
        .fetch_all(&self.pool)
        .await?;

        let mut out = HashMap::new();
        for row in rows {
            let id = from_track_hex(row.get::<String, _>("track_id").as_str());
            out.insert(
                id,
                TrackListenMetrics {
                    play_count: row.get::<i64, _>("play_count") as u32,
                    play_time: Duration::from_secs(row.get::<i64, _>("play_time_secs") as u64),
                    first_played: row.get::<Option<i64>, _>("first_played").map(|v| v as u64),
                    last_played: row.get::<Option<i64>, _>("last_played").map(|v| v as u64),
                    skip_count: row.get::<i64, _>("skip_count") as u32,
                },
            );
        }

        Ok(ListenMetrics { tracks: out })
    }

    /// Loads the full [`crate::controller::state::AppState`]: structured data
    /// from SQLite plus the playback session from its RON file.
    pub async fn load_app_state(
        &self,
        cache_dir: &Path,
    ) -> Result<crate::controller::state::AppState, CacherError> {
        let playback = crate::cacher::io::read_playback_state_from_disk(cache_dir)?;
        let library = self.load_library().await?;
        let queue = self.load_queue().await?;
        let favorites = self.load_favorites().await?;
        let metrics = self.load_metrics().await?;

        Ok(crate::controller::state::AppState {
            playback,
            library,
            queue,
            favorites,
            metrics,
            metrics_session: None,
            last_playback_write: None,
        })
    }
}
