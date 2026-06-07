use anyhow::Result;
use rusqlite::{Connection, Error as RusqliteError, types::Type};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use crate::controller::state::{PlaylistId, TrackId};
use uuid::Uuid;

pub fn get_tracks_missing_thumbnails(conn: &Connection) -> Result<HashSet<(TrackId, PathBuf)>> {
    let mut stmt = conn.prepare(
        "SELECT t.track_hash, s.path FROM track_sources s
         JOIN tracks t ON s.track_id = t.id",
    )?;

    let rows = stmt.query_map([], |row| {
        let hash: Vec<u8> = row.get(0)?;
        let path: String = row.get(1)?;
        let hash: [u8; 16] = hash.try_into().map_err(|_| {
            RusqliteError::FromSqlConversionFailure(
                0,
                Type::Blob,
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "invalid track hash length",
                )),
            )
        })?;

        Ok((TrackId(hash), PathBuf::from(path)))
    })?;

    let mut result = HashSet::new();
    for row in rows {
        result.insert(row?);
    }

    Ok(result)
}

pub fn get_playlist_thumbnail_jobs(conn: &Connection) -> Result<Vec<(PlaylistId, Vec<PathBuf>)>> {
    let mut stmt = conn.prepare(
        "SELECT p.id, s.path FROM playlist_tracks pt
         JOIN playlists p ON pt.playlist_id = p.id
         JOIN track_sources s ON pt.track_id = s.track_id
         ORDER BY pt.position",
    )?;

    let rows = stmt.query_map([], |row| {
        let playlist_id: String = row.get(0)?;
        let path: String = row.get(1)?;

        let uuid = Uuid::parse_str(&playlist_id)
            .map_err(|e| RusqliteError::FromSqlConversionFailure(0, Type::Text, Box::new(e)))?;

        Ok((PlaylistId(uuid), PathBuf::from(path)))
    })?;

    let mut jobs: HashMap<PlaylistId, Vec<PathBuf>> = HashMap::new();

    for row in rows {
        let (id, path) = row?;
        jobs.entry(id).or_default().push(path);
    }

    Ok(jobs.into_iter().collect())
}
