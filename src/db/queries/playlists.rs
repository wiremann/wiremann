use anyhow::Result;
use rusqlite::{Connection, Transaction};
use sea_query::{Expr, ExprTrait, Func, OnConflict, Order, Query, SqliteQueryBuilder};
use sea_query_rusqlite::RusqliteBinder;
use uuid::Uuid;

use crate::db::tables::{PlaylistTracks, Playlists};

pub fn insert_playlist(tx: &Transaction, id: &Uuid, name: &str, source: &str) -> Result<()> {
    let query = Query::insert()
        .into_table(Playlists::Table)
        .columns([Playlists::Id, Playlists::Name, Playlists::Source])
        .values_panic([id.to_string().into(), name.into(), source.into()])
        .on_conflict(
            OnConflict::column(Playlists::Id)
                .update_columns([Playlists::Name, Playlists::Source])
                .to_owned(),
        )
        .to_owned();

    execute_tx(tx, &query)?;

    Ok(())
}

pub fn insert_playlist_track(
    tx: &Transaction,
    playlist_id: &Uuid,
    track_id: i64,
    position: i64,
) -> Result<()> {
    let query = Query::insert()
        .into_table(PlaylistTracks::Table)
        .columns([
            PlaylistTracks::PlaylistId,
            PlaylistTracks::TrackId,
            PlaylistTracks::Position,
        ])
        .values_panic([
            playlist_id.to_string().into(),
            track_id.into(),
            position.into(),
        ])
        .on_conflict(
            OnConflict::columns([PlaylistTracks::PlaylistId, PlaylistTracks::Position])
                .do_nothing()
                .to_owned(),
        )
        .to_owned();

    execute_tx(tx, &query)?;

    Ok(())
}

pub fn get_playlist_next_position(tx: &Transaction, playlist_id: &Uuid) -> Result<i64> {
    // MAX(position) will always return one row; its value may be NULL when
    // there are no tracks for the playlist. Read it as Option<i64> to handle
    // the NULL case gracefully.
    let query = Query::select()
        .expr(Func::max(Expr::col(PlaylistTracks::Position)))
        .from(PlaylistTracks::Table)
        .and_where(Expr::col(PlaylistTracks::PlaylistId).eq(Expr::val(playlist_id.to_string())))
        .to_owned();

    let (sql, values) = query.build_rusqlite(SqliteQueryBuilder);
    let params = values.as_params();
    let pos: Option<i64> = tx.query_row(&sql, params.as_slice(), |row| row.get(0))?;

    Ok(pos.unwrap_or(-1) + 1)
}

pub fn load_playlists(conn: &Connection) -> Result<Vec<(Uuid, String)>> {
    let query = Query::select()
        .columns([Playlists::Id, Playlists::Name])
        .from(Playlists::Table)
        .order_by(Playlists::Name, Order::Asc)
        .to_owned();

    let (sql, values) = query.build_rusqlite(SqliteQueryBuilder);
    let params = values.as_params();
    let mut stmt = conn.prepare(&sql)?;

    let rows = stmt.query_map(params.as_slice(), |row| {
        let id_str: String = row.get(0)?;
        let name: String = row.get(1)?;
        let uuid = Uuid::parse_str(&id_str).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
        })?;
        Ok((uuid, name))
    })?;

    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }

    Ok(out)
}

pub fn load_playlist_tracks(conn: &Connection, playlist_id: &Uuid) -> Result<Vec<i64>> {
    let query = Query::select()
        .column(PlaylistTracks::TrackId)
        .from(PlaylistTracks::Table)
        .and_where(Expr::col(PlaylistTracks::PlaylistId).eq(Expr::val(playlist_id.to_string())))
        .order_by(PlaylistTracks::Position, Order::Asc)
        .to_owned();

    let (sql, values) = query.build_rusqlite(SqliteQueryBuilder);
    let params = values.as_params();
    let mut stmt = conn.prepare(&sql)?;

    let rows = stmt.query_map(params.as_slice(), |row| row.get(0))?;

    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }

    Ok(out)
}

fn execute_tx(tx: &Transaction, query: &sea_query::InsertStatement) -> Result<()> {
    let (sql, values) = query.build_rusqlite(SqliteQueryBuilder);
    let params = values.as_params();
    tx.execute(&sql, params.as_slice())?;
    Ok(())
}
