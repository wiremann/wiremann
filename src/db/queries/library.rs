use anyhow::Result;
use rusqlite::{Connection, Error as RusqliteError, types::Type};
use sea_query::{Alias, Expr, ExprTrait, Func, JoinType, Order, Query, SqliteQueryBuilder};
use sea_query_rusqlite::RusqliteBinder;

use crate::controller::state::{self, ImageId, PlaylistId, TrackId};
use crate::db::tables::{
    Albums, Artists, PlaylistTracks, Playlists, TrackArtists, TrackSources, Tracks,
};
use crate::ui::pages::library::models::{LibraryPlaylistRow, LibraryTrackRow};

use uuid::Uuid;

pub fn get_library_playlists(conn: &Connection) -> Result<Vec<LibraryPlaylistRow>> {
    let track_count = Alias::new("track_count");

    let query = Query::select()
        .expr(Expr::col((Playlists::Table, Playlists::Id)))
        .expr(Expr::col((Playlists::Table, Playlists::Name)))
        .expr(Expr::col((Playlists::Table, Playlists::ImageHash)))
        .expr_as(
            Func::count(Expr::col((PlaylistTracks::Table, PlaylistTracks::TrackId))),
            track_count.clone(),
        )
        .from(Playlists::Table)
        .join(
            JoinType::LeftJoin,
            PlaylistTracks::Table,
            Expr::col((PlaylistTracks::Table, PlaylistTracks::PlaylistId))
                .equals((Playlists::Table, Playlists::Id)),
        )
        .group_by_col((Playlists::Table, Playlists::Id))
        .order_by((Playlists::Table, Playlists::Name), Order::Asc)
        .to_owned();

    let (sql, values) = query.build_rusqlite(SqliteQueryBuilder);

    let params = values.as_params();

    let mut stmt = conn.prepare(&sql)?;

    let rows = stmt.query_map(params.as_slice(), |row| {
        let id_str: String = row.get(0)?;
        let name: String = row.get(1)?;
        let image_hash: Option<Vec<u8>> = row.get(2)?;
        let track_count: usize = row.get(3)?;

        let uuid = Uuid::parse_str(&id_str)
            .map_err(|e| RusqliteError::FromSqlConversionFailure(0, Type::Text, Box::new(e)))?;

        // If image_hash exists but has invalid length, treat it as missing instead of erroring.
        let image_id: Option<state::ImageId> = match image_hash {
            Some(hash) => {
                if hash.len() == 16 {
                    let arr: [u8; 16] = hash.try_into().unwrap();
                    Some(ImageId(arr))
                } else {
                    None
                }
            }
            None => None,
        };

        Ok(LibraryPlaylistRow {
            id: PlaylistId(uuid),
            name,
            image_id,
            track_count,
        })
    })?;

    let mut playlists = Vec::new();

    for row in rows {
        playlists.push(row?);
    }

    Ok(playlists)
}

pub fn get_total_track_count(conn: &Connection) -> Result<usize> {
    let query = Query::select()
        .expr(Func::count(Expr::col((Tracks::Table, Tracks::Id))))
        .from(Tracks::Table)
        .to_owned();

    let (sql, values) = query.build_rusqlite(SqliteQueryBuilder);

    let params = values.as_params();

    let count: i64 = conn.query_row(&sql, params.as_slice(), |row| row.get(0))?;

    Ok(count as usize)
}

pub fn get_tracks_page(conn: &Connection, limit: u64, offset: u64) -> Result<Vec<LibraryTrackRow>> {
    let artists_alias = Alias::new("artists");

    let query = Query::select()
        .expr(Expr::col((Tracks::Table, Tracks::TrackHash)))
        .expr(Expr::col((Tracks::Table, Tracks::Name)))
        .expr_as(
            Func::cust(Alias::new("group_concat"))
                .arg(Expr::col((Artists::Table, Artists::Name)))
                .arg(Expr::val(", ")),
            artists_alias,
        )
        .expr(Expr::col((Albums::Table, Albums::Name)))
        .expr(Expr::col((Tracks::Table, Tracks::Duration)))
        .expr(Expr::col((Tracks::Table, Tracks::ImageHash)))
        .from(Tracks::Table)
        .join(
            JoinType::LeftJoin,
            TrackArtists::Table,
            Expr::col((TrackArtists::Table, TrackArtists::TrackId))
                .equals((Tracks::Table, Tracks::Id)),
        )
        .join(
            JoinType::LeftJoin,
            Artists::Table,
            Expr::col((Artists::Table, Artists::Id))
                .equals((TrackArtists::Table, TrackArtists::ArtistId)),
        )
        .join(
            JoinType::LeftJoin,
            Albums::Table,
            Expr::col((Albums::Table, Albums::Id)).equals((Tracks::Table, Tracks::AlbumId)),
        )
        .group_by_col((Tracks::Table, Tracks::Id))
        .order_by((Tracks::Table, Tracks::Id), Order::Asc)
        .limit(limit)
        .offset(offset)
        .to_owned();

    let (sql, values) = query.build_rusqlite(SqliteQueryBuilder);

    let params = values.as_params();

    let mut stmt = conn.prepare(&sql)?;

    let rows = stmt.query_map(params.as_slice(), |row| {
        let track_hash: Vec<u8> = row.get(0)?;
        let title: String = row.get(1)?;
        let artists: Option<String> = row.get(2)?;
        let album: Option<String> = row.get(3)?;
        let duration_ms: i64 = row.get(4)?;
        let image_hash: Option<Vec<u8>> = row.get(5)?;

        let hash: [u8; 16] = track_hash.try_into().map_err(|_| {
            RusqliteError::FromSqlConversionFailure(
                0,
                Type::Blob,
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "invalid track hash length",
                )),
            )
        })?;

        // If image_hash exists but has invalid length, ignore it and return None.
        let image_id: Option<state::ImageId> = match image_hash {
            Some(hash) => {
                if hash.len() == 16 {
                    let arr: [u8; 16] = hash.try_into().unwrap();
                    Some(ImageId(arr))
                } else {
                    None
                }
            }
            None => None,
        };

        Ok(LibraryTrackRow {
            id: TrackId(hash),
            title,
            artists: artists.unwrap_or_else(|| "Unknown Artist".into()),
            album: album.unwrap_or_else(|| "Unknown Album".into()),
            duration_ms,
            image_id,
        })
    })?;

    let mut tracks = Vec::new();

    for row in rows {
        tracks.push(row?);
    }

    Ok(tracks)
}
