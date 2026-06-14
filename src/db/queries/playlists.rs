use anyhow::Result;
use rusqlite::{Connection, Transaction};
use sea_query::{Expr, ExprTrait, Func, OnConflict, Order, Query, SqliteQueryBuilder};
use sea_query_rusqlite::RusqliteBinder;
use uuid::Uuid;

use crate::controller::state::{ImageId, PlaylistId, PlaylistSource, TrackId};
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
    // there are no tracks for the playlist.
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

pub struct PlaylistProjection {
    pub id: PlaylistId,
    pub name: String,
    pub source: PlaylistSource,
    pub image_id: Option<ImageId>,
    pub tracks: Vec<TrackId>,
}

pub fn load_playlists_with_tracks(conn: &Connection) -> Result<Vec<PlaylistProjection>> {
    use crate::db::tables::{PlaylistTracks, Playlists, Tracks};
    use sea_query::{Expr, Order, Query, SqliteQueryBuilder};

    let query = Query::select()
        .expr(Expr::col((Playlists::Table, Playlists::Id)))
        .expr(Expr::col((Playlists::Table, Playlists::Name)))
        .expr(Expr::col((Playlists::Table, Playlists::Source)))
        .expr(Expr::col((Playlists::Table, Playlists::ImageHash)))
        .expr(Expr::col((PlaylistTracks::Table, PlaylistTracks::Position)))
        .expr(Expr::col((Tracks::Table, Tracks::TrackHash)))
        .from(Playlists::Table)
        .join(
            sea_query::JoinType::LeftJoin,
            PlaylistTracks::Table,
            Expr::col((PlaylistTracks::Table, PlaylistTracks::PlaylistId))
                .equals((Playlists::Table, Playlists::Id)),
        )
        .join(
            sea_query::JoinType::LeftJoin,
            Tracks::Table,
            Expr::col((Tracks::Table, Tracks::Id))
                .equals((PlaylistTracks::Table, PlaylistTracks::TrackId)),
        )
        .order_by((Playlists::Table, Playlists::Name), Order::Asc)
        .order_by(
            (PlaylistTracks::Table, PlaylistTracks::Position),
            Order::Asc,
        )
        .to_owned();

    let (sql, values) = query.build_rusqlite(SqliteQueryBuilder);
    let params = values.as_params();

    let mut stmt = conn.prepare(&sql)?;

    let rows = stmt.query_map(params.as_slice(), |row| {
        let id_str: String = row.get(0)?;
        let name: String = row.get(1)?;
        let source: String = row.get(2)?;
        let image_hash: Option<Vec<u8>> = row.get(3)?;
        let _pos: Option<i64> = row.get(4)?;
        let track_hash: Option<Vec<u8>> = row.get(5)?;

        Ok((id_str, name, source, image_hash, track_hash))
    })?;

    use std::collections::HashMap;
    let mut map: HashMap<Uuid, PlaylistProjection> = HashMap::new();
    let mut order: Vec<Uuid> = Vec::new();

    for r in rows {
        let (id_str, name, source, image_hash, track_hash) = r?;
        let uuid = Uuid::parse_str(&id_str)?;

        if !map.contains_key(&uuid) {
            let src = match source.as_str() {
                "folder" => PlaylistSource::Folder,
                "generated" => PlaylistSource::Generated,
                _ => PlaylistSource::User,
            };

            let image_id = match image_hash {
                Some(h) if h.len() == 16 => {
                    let arr: [u8; 16] = h
                        .try_into()
                        .map_err(|_| anyhow::anyhow!("invalid image hash"))?;
                    Some(ImageId(arr))
                }
                _ => None,
            };

            map.insert(
                uuid,
                PlaylistProjection {
                    id: PlaylistId(uuid),
                    name: name.clone(),
                    source: src,
                    image_id,
                    tracks: Vec::new(),
                },
            );
            order.push(uuid);
        }

        if let Some(hash) = track_hash {
            if hash.len() == 16 {
                let arr: [u8; 16] = hash.try_into().unwrap();
                if let Some(p) = map.get_mut(&uuid) {
                    p.tracks.push(TrackId(arr));
                }
            }
        }
    }

    let mut out = Vec::new();
    for id in order {
        if let Some(p) = map.remove(&id) {
            out.push(p);
        }
    }

    Ok(out)
}

pub fn set_playlist_image(conn: &Connection, playlist_id: &Uuid, image_hash: &[u8]) -> Result<()> {
    use sea_query::{Expr, ExprTrait, OnConflict, Query, SqliteQueryBuilder};

    let update = Query::update()
        .table(Playlists::Table)
        .values([(Playlists::ImageHash, Expr::val(image_hash.to_vec()))])
        .and_where(
            Expr::col((Playlists::Table, Playlists::Id)).eq(Expr::val(playlist_id.to_string())),
        )
        .to_owned();

    let (sql, values) = update.build_rusqlite(SqliteQueryBuilder);
    let params = values.as_params();
    conn.execute(&sql, params.as_slice())?;

    Ok(())
}

fn execute_tx(tx: &Transaction, query: &sea_query::InsertStatement) -> Result<()> {
    let (sql, values) = query.build_rusqlite(SqliteQueryBuilder);
    let params = values.as_params();
    tx.execute(&sql, params.as_slice())?;
    Ok(())
}
