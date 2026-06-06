use anyhow::Result;

use rusqlite::Connection;

use sea_query::{Expr, JoinType, Query, SqliteQueryBuilder};

use sea_query_rusqlite::RusqliteBinder;

use std::collections::HashMap;
use std::path::PathBuf;

use crate::{controller::state::PlaylistId, db::tables::*};

pub fn get_tracks_missing_thumbnails(
    conn: &Connection,
) -> Result<std::collections::HashSet<(crate::controller::state::TrackId, PathBuf)>> {
    let query = Query::select()
        .columns([Tracks::TrackHash, TrackSources::Path])
        .from(Tracks::Table)
        .join(
            JoinType::InnerJoin,
            TrackSources::Table,
            Expr::col((TrackSources::Table, TrackSources::TrackId))
                .equals((Tracks::Table, Tracks::Id)),
        )
        .and_where(Expr::col(Tracks::ImageHash).is_null())
        .to_owned();

    let (sql, values) = query.build_rusqlite(SqliteQueryBuilder);

    let mut stmt = conn.prepare(&sql)?;

    let mut rows = stmt.query(values.as_params())?;

    let mut out = std::collections::HashSet::new();

    while let Some(row) = rows.next()? {
        let hash: Vec<u8> = row.get(0)?;

        let path: String = row.get(1)?;

        let mut id = [0_u8; 16];

        id.copy_from_slice(&hash);

        out.insert((crate::controller::state::TrackId(id), PathBuf::from(path)));
    }

    Ok(out)
}

pub fn get_playlist_thumbnail_jobs(conn: &Connection) -> Result<HashMap<PlaylistId, Vec<PathBuf>>> {
    let query = Query::select()
        .columns([PlaylistTracks::PlaylistId, TrackSources::Path])
        .from(PlaylistTracks::Table)
        .join(
            JoinType::InnerJoin,
            TrackSources::Table,
            Expr::col((TrackSources::Table, TrackSources::TrackId))
                .equals((PlaylistTracks::Table, PlaylistTracks::TrackId)),
        )
        .to_owned();

    let (sql, values) = query.build_rusqlite(SqliteQueryBuilder);

    let mut stmt = conn.prepare(&sql)?;

    let mut rows = stmt.query(values.as_params())?;

    let mut out = HashMap::new();

    while let Some(row) = rows.next()? {
        let playlist_id: String = row.get(0)?;

        let path: String = row.get(1)?;

        let pid = PlaylistId(uuid::Uuid::parse_str(&playlist_id)?);

        out.entry(pid)
            .or_insert_with(Vec::new)
            .push(PathBuf::from(path));
    }

    Ok(out)
}
