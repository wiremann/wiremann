use anyhow::Result;
use rusqlite::{Connection, Error as RusqliteError, Transaction};
use sea_query::{Expr, ExprTrait, OnConflict, Order, Query, SelectStatement, SqliteQueryBuilder};
use sea_query_rusqlite::RusqliteBinder;
use std::time::Duration;

use crate::controller::state::{ImageId, PlaylistId, TrackId};
use crate::db::models::DbTrackSummary;
use crate::db::models::InsertedTrack;
use crate::db::tables::{AlbumArtists, Albums, Artists, TrackArtists, TrackSources, Tracks};
use crate::scanner::{ScannedTrack, ScannedTrackSource};

pub fn upsert_scanned_tracks(
    conn: &mut Connection,
    tracks: &[ScannedTrack],
    playlist_id: Option<PlaylistId>,
) -> Result<Vec<InsertedTrack>> {
    let tx = conn.transaction()?;

    // If persisting into a playlist, determine starting position once.
    let mut position = None;
    if let Some(pid) = &playlist_id {
        position = Some(crate::db::queries::playlists::get_playlist_next_position(
            &tx, &pid.0,
        )?);
    }

    let mut committed_rows: Vec<InsertedTrack> = Vec::new();

    for track in tracks {
        let (db_track_id, maybe_row) = upsert_scanned_track_returning_row(&tx, track)?;

        if let Some(pid) = &playlist_id {
            let pos = position.unwrap_or(0);
            crate::db::queries::playlists::insert_playlist_track(&tx, &pid.0, db_track_id, pos)?;
            if let Some(ref mut p) = position {
                *p += 1;
            }

            // For playlist insertions, always produce an InsertedTrack for UI consumption.
            if let Some(row) = maybe_row {
                committed_rows.push(row);
            } else {
                // Existing track — query metadata to build an InsertedTrack
                if let Ok(existing) = fetch_inserted_track(&tx, db_track_id) {
                    committed_rows.push(existing);
                }
            }
        } else if let Some(row) = maybe_row {
            committed_rows.push(row);
        }
    }

    tx.commit()?;
    Ok(committed_rows)
}

fn fetch_inserted_track(tx: &Transaction, db_track_id: i64) -> Result<InsertedTrack> {
    use crate::db::tables::{Albums, Artists, TrackArtists, Tracks};

    // Fetch basic track info
    let select = Query::select()
        .columns([
            Tracks::Id,
            Tracks::TrackHash,
            Tracks::Name,
            Tracks::AlbumId,
            Tracks::Duration,
            Tracks::ImageHash,
        ])
        .from(Tracks::Table)
        .and_where(Expr::col(Tracks::Id).eq(Expr::val(db_track_id)))
        .to_owned();

    let (sql, values) = select.build_rusqlite(SqliteQueryBuilder);
    let params = values.as_params();
    let mut stmt = tx.prepare(&sql)?;

    let row = stmt.query_row(params.as_slice(), |row| {
        let id: i64 = row.get(0)?;
        let track_hash: Vec<u8> = row.get(1)?;
        let name: String = row.get(2)?;
        let album_id: Option<i64> = row.get(3)?;
        let duration_ms: i64 = row.get(4)?;
        let image_hash: Option<Vec<u8>> = row.get(5)?;

        Ok((id, track_hash, name, album_id, duration_ms, image_hash))
    })?;

    let (id, track_hash, name, album_id, duration_ms, image_hash) = row;

    // Fetch artists for the track
    let artist_q = Query::select()
        .column(Artists::Name)
        .from(Artists::Table)
        .join(
            sea_query::JoinType::InnerJoin,
            TrackArtists::Table,
            Expr::col((TrackArtists::Table, TrackArtists::ArtistId))
                .equals((Artists::Table, Artists::Id)),
        )
        .and_where(Expr::col((TrackArtists::Table, TrackArtists::TrackId)).eq(Expr::val(id)))
        .to_owned();

    let (sql2, values2) = artist_q.build_rusqlite(SqliteQueryBuilder);
    let params2 = values2.as_params();
    let mut stmt2 = tx.prepare(&sql2)?;

    let artist_rows = stmt2.query_map(params2.as_slice(), |r| r.get::<_, String>(0))?;
    let mut artists: Vec<String> = Vec::new();
    for ar in artist_rows {
        artists.push(ar?);
    }

    // Fetch album name if present
    let album_name = if let Some(aid) = album_id {
        let sel = Query::select()
            .column(Albums::Name)
            .from(Albums::Table)
            .and_where(Expr::col(Albums::Id).eq(Expr::val(aid)))
            .to_owned();
        let (sql3, vals3) = sel.build_rusqlite(SqliteQueryBuilder);
        let params3 = vals3.as_params();
        let mut s3 = tx.prepare(&sql3)?;
        match s3.query_row(params3.as_slice(), |r| r.get::<_, String>(0)) {
            Ok(n) => Some(n),
            Err(_) => None,
        }
    } else {
        None
    };

    Ok(InsertedTrack {
        id,
        track_hash,
        artists: artists.join(", "),
        name,
        album: album_name,
        duration_ms,
        image_hash,
    })
}

fn upsert_scanned_track_returning_row(
    tx: &Transaction,
    track: &ScannedTrack,
) -> Result<(i64, Option<InsertedTrack>)> {
    let track_hash = TrackId::generate(
        &track.title,
        &track.artists.join(", "),
        track.album.as_deref().unwrap_or(""),
    )?;

    let select = Query::select()
        .column(Tracks::Id)
        .from(Tracks::Table)
        .and_where(Expr::col(Tracks::TrackHash).eq(Expr::val(track_hash.0.to_vec())))
        .to_owned();

    if let Ok(id) = query_i64_tx(tx, &select) {
        let db_track_id = id;

        upsert_track_source(tx, db_track_id, &track.source)?;

        for artist_name in &track.artists {
            let artist_id = upsert_artist(tx, artist_name)?;
            if let Some(album_name) = &track.album {
                let aid = upsert_album(tx, album_name)?;
                insert_album_artist(tx, aid, artist_id)?;
            }
            insert_track_artist(tx, db_track_id, artist_id)?;
        }

        return Ok((db_track_id, None));
    }

    let image_hash = track
        .image
        .as_deref()
        .map(ImageId::generate)
        .transpose()?
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
        image_hash.as_ref().map(|h| h.as_slice()),
        &track_hash.0,
    )?;

    upsert_track_source(tx, db_track_id, &track.source)?;

    for artist_name in &track.artists {
        let artist_id = upsert_artist(tx, artist_name)?;
        if let Some(aid) = album_id {
            insert_album_artist(tx, aid, artist_id)?;
        }
        insert_track_artist(tx, db_track_id, artist_id)?;
    }

    // Build LibraryTrackRow directly from scanned metadata
    let row = InsertedTrack {
        id: db_track_id,
        track_hash: track_hash.0.to_vec(),
        artists: track.artists.join(", "),
        name: track.title.clone(),
        album: track.album.clone(),
        duration_ms: track.duration.as_millis() as i64,
        image_hash,
    };

    Ok((db_track_id, Some(row)))
}

fn upsert_album(tx: &Transaction, name: &str) -> Result<i64> {
    let insert = Query::insert()
        .into_table(Albums::Table)
        .columns([Albums::Name])
        .values_panic([Expr::val(name)])
        .on_conflict(OnConflict::column(Albums::Name).do_nothing().to_owned())
        .to_owned();

    execute_tx(tx, &insert)?;

    let select = Query::select()
        .column(Albums::Id)
        .from(Albums::Table)
        .and_where(Expr::col(Albums::Name).eq(name))
        .to_owned();

    query_i64_tx(tx, &select)
}

fn upsert_artist(tx: &Transaction, name: &str) -> Result<i64> {
    let insert = Query::insert()
        .into_table(Artists::Table)
        .columns([Artists::Name])
        .values_panic([Expr::val(name)])
        .on_conflict(OnConflict::column(Artists::Name).do_nothing().to_owned())
        .to_owned();

    execute_tx(tx, &insert)?;

    let select = Query::select()
        .column(Artists::Id)
        .from(Artists::Table)
        .and_where(Expr::col(Artists::Name).eq(name))
        .to_owned();

    query_i64_tx(tx, &select)
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

    let insert = Query::insert()
        .into_table(Tracks::Table)
        .columns([
            Tracks::TrackHash,
            Tracks::Name,
            Tracks::AlbumId,
            Tracks::Duration,
            Tracks::ImageHash,
        ])
        .values_panic([
            Expr::val(track_hash),
            Expr::val(name),
            Expr::val(album_id),
            Expr::val(duration_ms),
            Expr::val(image_hash.map(|hash| hash.to_vec())),
        ])
        .on_conflict(
            OnConflict::column(Tracks::TrackHash)
                .update_columns([
                    Tracks::Name,
                    Tracks::AlbumId,
                    Tracks::Duration,
                    Tracks::ImageHash,
                ])
                .to_owned(),
        )
        .to_owned();

    execute_tx(tx, &insert)?;

    let select = Query::select()
        .column(Tracks::Id)
        .from(Tracks::Table)
        .and_where(Expr::col(Tracks::TrackHash).eq(Expr::val(track_hash)))
        .to_owned();

    query_i64_tx(tx, &select)
}

fn upsert_track_source(tx: &Transaction, track_id: i64, source: &ScannedTrackSource) -> Result<()> {
    let insert = Query::insert()
        .into_table(TrackSources::Table)
        .columns([
            TrackSources::TrackId,
            TrackSources::Path,
            TrackSources::Size,
            TrackSources::Modified,
        ])
        .values_panic([
            Expr::val(track_id),
            Expr::val(source.path.to_string_lossy().to_string()),
            Expr::val(source.size as i64),
            Expr::val(source.modified as i64),
        ])
        .on_conflict(
            OnConflict::column(TrackSources::Path)
                .update_columns([
                    TrackSources::TrackId,
                    TrackSources::Size,
                    TrackSources::Modified,
                ])
                .to_owned(),
        )
        .to_owned();

    execute_tx(tx, &insert)?;

    Ok(())
}

fn insert_track_artist(tx: &Transaction, track_id: i64, artist_id: i64) -> Result<()> {
    let insert = Query::insert()
        .into_table(TrackArtists::Table)
        .columns([TrackArtists::TrackId, TrackArtists::ArtistId])
        .values_panic([Expr::val(track_id), Expr::val(artist_id)])
        .on_conflict(
            OnConflict::columns([TrackArtists::TrackId, TrackArtists::ArtistId])
                .do_nothing()
                .to_owned(),
        )
        .to_owned();

    execute_tx(tx, &insert)?;

    Ok(())
}

fn insert_album_artist(tx: &Transaction, album_id: i64, artist_id: i64) -> Result<()> {
    let insert = Query::insert()
        .into_table(AlbumArtists::Table)
        .columns([AlbumArtists::AlbumId, AlbumArtists::ArtistId])
        .values_panic([Expr::val(album_id), Expr::val(artist_id)])
        .on_conflict(
            OnConflict::columns([AlbumArtists::AlbumId, AlbumArtists::ArtistId])
                .do_nothing()
                .to_owned(),
        )
        .to_owned();

    execute_tx(tx, &insert)?;

    Ok(())
}

pub fn get_all_tracks(conn: &Connection) -> Result<Vec<DbTrackSummary>> {
    let query = Query::select()
        .columns([
            Tracks::Id,
            Tracks::TrackHash,
            Tracks::Name,
            Tracks::AlbumId,
            Tracks::Duration,
        ])
        .from(Tracks::Table)
        .order_by(Tracks::Id, Order::Asc)
        .to_owned();

    let (sql, values) = query.build_rusqlite(SqliteQueryBuilder);
    let params = values.as_params();
    let mut stmt = conn.prepare(&sql)?;

    let rows = stmt.query_map(params.as_slice(), |row| DbTrackSummary::from_row(row))?;

    let mut tracks = Vec::new();
    for row in rows {
        tracks.push(row?);
    }

    Ok(tracks)
}

fn execute_tx(tx: &Transaction, query: &sea_query::InsertStatement) -> Result<()> {
    let (sql, values) = query.build_rusqlite(SqliteQueryBuilder);
    let params = values.as_params();
    tx.execute(&sql, params.as_slice())?;
    Ok(())
}

fn query_i64_tx(tx: &Transaction, query: &SelectStatement) -> Result<i64> {
    let (sql, values) = query.build_rusqlite(SqliteQueryBuilder);
    let params = values.as_params();
    Ok(tx.query_row(&sql, params.as_slice(), |row| row.get(0))?)
}
