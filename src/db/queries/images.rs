use anyhow::Result;
use rusqlite::{types::Type, Connection, Error as RusqliteError};
use sea_query::{Expr, ExprTrait, Order, Query, SqliteQueryBuilder};
use sea_query_rusqlite::RusqliteBinder;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use crate::controller::state::{PlaylistId, TrackId};
use crate::db::tables::{PlaylistTracks, Playlists, TrackSources, Tracks};
use uuid::Uuid;

pub fn get_tracks_missing_thumbnails(conn: &Connection) -> Result<HashSet<(TrackId, PathBuf)>> {
    let query = Query::select()
        .expr(Expr::col((Tracks::Table, Tracks::TrackHash)))
        .expr(Expr::col((TrackSources::Table, TrackSources::Path)))
        .from(TrackSources::Table)
        .inner_join(
            Tracks::Table,
            Expr::col((TrackSources::Table, TrackSources::TrackId))
                .equals((Tracks::Table, Tracks::Id)),
        )
        .to_owned();

    let (sql, values) = query.build_rusqlite(SqliteQueryBuilder);
    let params = values.as_params();
    let mut stmt = conn.prepare(&sql)?;

    let rows = stmt.query_map(params.as_slice(), |row| {
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
    let query = Query::select()
        .expr(Expr::col((Playlists::Table, Playlists::Id)))
        .expr(Expr::col((TrackSources::Table, TrackSources::Path)))
        .from(PlaylistTracks::Table)
        .inner_join(
            Playlists::Table,
            Expr::col((PlaylistTracks::Table, PlaylistTracks::PlaylistId))
                .equals((Playlists::Table, Playlists::Id)),
        )
        .inner_join(
            TrackSources::Table,
            Expr::col((PlaylistTracks::Table, PlaylistTracks::TrackId))
                .equals((TrackSources::Table, TrackSources::TrackId)),
        )
        .order_by(
            (PlaylistTracks::Table, PlaylistTracks::Position),
            Order::Asc,
        )
        .to_owned();

    let (sql, values) = query.build_rusqlite(SqliteQueryBuilder);
    let params = values.as_params();
    let mut stmt = conn.prepare(&sql)?;

    let rows = stmt.query_map(params.as_slice(), |row| {
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
