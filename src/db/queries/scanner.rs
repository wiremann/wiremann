use anyhow::Result;
use rusqlite::{Connection, Transaction, params};
use std::time::Duration;

use crate::controller::state::{ImageId, TrackId};
use crate::scanner::{ScannedTrack, ScannedTrackSource};

pub struct DbTrackSummary {
    pub id: i64,
    pub track_hash: Vec<u8>,
    pub name: String,
    pub album_id: Option<i64>,
    pub duration_ms: i64,
}

pub fn upsert_scanned_tracks(conn: &mut Connection, tracks: &[ScannedTrack]) -> Result<()> {
    let tx = conn.transaction()?;

    for track in tracks {
        upsert_scanned_track(&tx, track)?;
    }

    tx.commit()?;
    Ok(())
}

fn upsert_scanned_track(tx: &Transaction, track: &ScannedTrack) -> Result<i64> {
    let track_hash = TrackId::generate(
        &track.title,
        &track.artists.join(", "),
        track.album.as_deref().unwrap_or(""),
    )?;

    let image_hash = track
        .image
        .as_deref()
        .and_then(|bytes| ImageId::generate(bytes).ok())
        .map(|id| id.0.to_vec());

    let album_id = if let Some(album_name) = &track.album {
        Some(upsert_album(tx, album_name)?)
    } else {
        None
    };

    let db_track_id = upsert_track(
        tx,
        &track.title,
        album_id,
        &track.duration,
        image_hash.as_deref(),
        &track_hash.0,
    )?;

    upsert_track_source(tx, db_track_id, &track.source)?;

    for artist_name in &track.artists {
        let artist_id = upsert_artist(tx, artist_name)?;
        insert_track_artist(tx, db_track_id, artist_id)?;
    }

    Ok(db_track_id)
}

fn upsert_album(tx: &Transaction, name: &str) -> Result<i64> {
    tx.execute(
        "INSERT OR IGNORE INTO albums (name) VALUES (?1)",
        params![name],
    )?;

    Ok(tx.query_row(
        "SELECT id FROM albums WHERE name = ?1",
        params![name],
        |row| row.get(0),
    )?)
}

fn upsert_artist(tx: &Transaction, name: &str) -> Result<i64> {
    tx.execute(
        "INSERT OR IGNORE INTO artists (name) VALUES (?1)",
        params![name],
    )?;

    Ok(tx.query_row(
        "SELECT id FROM artists WHERE name = ?1",
        params![name],
        |row| row.get(0),
    )?)
}

fn upsert_track(
    tx: &Transaction,
    name: &str,
    album_id: Option<i64>,
    duration: &Duration,
    image_hash: Option<&[u8]>,
    track_hash: &[u8],
) -> Result<i64> {
    let duration_ms = duration.as_millis() as i64;

    tx.execute(
        "INSERT INTO tracks (track_hash, name, album_id, duration, image_hash)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(track_hash) DO UPDATE SET
             name = excluded.name,
             album_id = excluded.album_id,
             duration = excluded.duration,
             image_hash = excluded.image_hash",
        params![track_hash, name, album_id, duration_ms, image_hash],
    )?;

    Ok(tx.query_row(
        "SELECT id FROM tracks WHERE track_hash = ?1",
        params![track_hash],
        |row| row.get(0),
    )?)
}

fn upsert_track_source(tx: &Transaction, track_id: i64, source: &ScannedTrackSource) -> Result<()> {
    tx.execute(
        "INSERT INTO track_sources (track_id, path, size, modified)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(path) DO UPDATE SET
             track_id = excluded.track_id,
             size = excluded.size,
             modified = excluded.modified",
        params![
            track_id,
            source.path.to_string_lossy(),
            source.size as i64,
            source.modified as i64
        ],
    )?;

    Ok(())
}

fn insert_track_artist(tx: &Transaction, track_id: i64, artist_id: i64) -> Result<()> {
    tx.execute(
        "INSERT OR IGNORE INTO track_artists (track_id, artist_id) VALUES (?1, ?2)",
        params![track_id, artist_id],
    )?;

    Ok(())
}

pub fn get_all_tracks(conn: &Connection) -> Result<Vec<DbTrackSummary>> {
    let mut stmt =
        conn.prepare("SELECT id, track_hash, name, album_id, duration FROM tracks ORDER BY id")?;

    let rows = stmt.query_map([], |row| {
        Ok(DbTrackSummary {
            id: row.get(0)?,
            track_hash: row.get(1)?,
            name: row.get(2)?,
            album_id: row.get(3)?,
            duration_ms: row.get(4)?,
        })
    })?;

    let mut tracks = Vec::new();
    for row in rows {
        tracks.push(row?);
    }

    Ok(tracks)
}
