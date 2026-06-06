use anyhow::Result;

use rusqlite::Connection;

use sea_query::{
    Expr,
    Func,
    OnConflict,
    Query,
    SqliteQueryBuilder,
};

use sea_query_rusqlite::RusqliteBinder;

use crate::{
    controller::state::PlaylistId,
    db::tables::*,
    scanner::ScannedTrack,
};

pub fn upsert_scanned_track(
    conn: &mut Connection,
    track: &ScannedTrack,
    playlist_id: Option<PlaylistId>,
) -> Result<()> {
    let tx = conn.transaction()?;

    let album_id = get_or_create_album(
        &tx,
        track.album.as_deref(),
    )?;

    let track_id = get_or_create_track(
        &tx,
        track,
        album_id,
    )?;

    for artist in &track.artists {
        let artist_id = get_or_create_artist(
            &tx,
            artist,
        )?;

        let query = Query::insert()
            .into_table(TrackArtists::Table)
            .columns([
                TrackArtists::TrackId,
                TrackArtists::ArtistId,
            ])
            .values([
                track_id.into(),
                artist_id.into(),
            ])?
            .on_conflict(
                OnConflict::columns([
                    TrackArtists::TrackId,
                    TrackArtists::ArtistId,
                ])
                .do_nothing()
                .to_owned(),
            )
            .to_owned();

        let (sql, values) =
            query.build_rusqlite(SqliteQueryBuilder);

        tx.execute(&sql, values.as_params())?;
    }

    let query = Query::insert()
        .into_table(TrackSources::Table)
        .columns([
            TrackSources::TrackId,
            TrackSources::Path,
            TrackSources::Size,
            TrackSources::Modified,
        ])
        .values([
            track_id.into(),
            track.source.path.to_string_lossy().to_string().into(),
            (track.source.size as i64).into(),
            (track.source.modified as i64).into(),
        ])?
        .on_conflict(
            OnConflict::column(TrackSources::Path)
                .do_nothing()
                .to_owned(),
        )
        .to_owned();

    let (sql, values) =
        query.build_rusqlite(SqliteQueryBuilder);

    tx.execute(&sql, values.as_params())?;

    if let Some(pid) = playlist_id {
        let query = Query::select()
            .expr(
                Func::max(
                    Expr::col(PlaylistTracks::Position),
                ),
            )
            .from(PlaylistTracks::Table)
            .and_where(
                Expr::col(PlaylistTracks::PlaylistId)
                    .eq(pid.0.to_string()),
            )
            .to_owned();

        let (sql, values) =
            query.build_rusqlite(SqliteQueryBuilder);

        let max_position: Option<i64> =
            tx.query_row(
                &sql,
                values.as_params(),
                |row| row.get(0),
            )?;

        let position =
            max_position.unwrap_or(-1) + 1;

        let query = Query::insert()
            .into_table(PlaylistTracks::Table)
            .columns([
                PlaylistTracks::PlaylistId,
                PlaylistTracks::TrackId,
                PlaylistTracks::Position,
            ])
            .values([
                pid.0.to_string().into(),
                track_id.into(),
                position.into(),
            ])?
            .on_conflict(
                OnConflict::columns([
                    PlaylistTracks::PlaylistId,
                    PlaylistTracks::Position,
                ])
                .do_nothing()
                .to_owned(),
            )
            .to_owned();

        let (sql, values) =
            query.build_rusqlite(SqliteQueryBuilder);

        tx.execute(&sql, values.as_params())?;
    }

    tx.commit()?;

    Ok(())
}

fn get_or_create_album(
    conn: &Connection,
    album: Option<&str>,
) -> Result<Option<i64>> {
    let Some(album) = album else {
        return Ok(None);
    };

    let query = Query::select()
        .column(Albums::Id)
        .from(Albums::Table)
        .and_where(
            Expr::col(Albums::Name).eq(album),
        )
        .limit(1)
        .to_owned();

    let (sql, values) =
        query.build_rusqlite(SqliteQueryBuilder);

    let existing = conn.query_row(
        &sql,
        values.as_params(),
        |row| row.get(0),
    );

    if let Ok(id) = existing {
        return Ok(Some(id));
    }

    let query = Query::insert()
        .into_table(Albums::Table)
        .columns([Albums::Name])
        .values([album.into()])?
        .to_owned();

    let (sql, values) =
        query.build_rusqlite(SqliteQueryBuilder);

    conn.execute(&sql, values.as_params())?;

    Ok(Some(conn.last_insert_rowid()))
}

fn get_or_create_artist(
    conn: &Connection,
    artist: &str,
) -> Result<i64> {
    let query = Query::select()
        .column(Artists::Id)
        .from(Artists::Table)
        .and_where(
            Expr::col(Artists::Name).eq(artist),
        )
        .limit(1)
        .to_owned();

    let (sql, values) =
        query.build_rusqlite(SqliteQueryBuilder);

    let existing = conn.query_row(
        &sql,
        values.as_params(),
        |row| row.get(0),
    );

    if let Ok(id) = existing {
        return Ok(id);
    }

    let query = Query::insert()
        .into_table(Artists::Table)
        .columns([Artists::Name])
        .values([artist.into()])?
        .to_owned();

    let (sql, values) =
        query.build_rusqlite(SqliteQueryBuilder);

    conn.execute(&sql, values.as_params())?;

    Ok(conn.last_insert_rowid())
}

fn get_or_create_track(
    conn: &Connection,
    track: &ScannedTrack,
    album_id: Option<i64>,
) -> Result<i64> {
    let query = Query::select()
        .column(Tracks::Id)
        .from(Tracks::Table)
        .and_where(
            Expr::col(Tracks::TrackHash)
                .eq(track.id.0.to_vec()),
        )
        .limit(1)
        .to_owned();

    let (sql, values) =
        query.build_rusqlite(SqliteQueryBuilder);

    let existing = conn.query_row(
        &sql,
        values.as_params(),
        |row| row.get(0),
    );

    if let Ok(id) = existing {
        return Ok(id);
    }

    let query = Query::insert()
        .into_table(Tracks::Table)
        .columns([
            Tracks::TrackHash,
            Tracks::Name,
            Tracks::AlbumId,
            Tracks::Duration,
        ])
        .values([
            track.id.0.to_vec().into(),
            track.title.clone().into(),
            album_id.into(),
            (track.duration.as_millis() as i64).into(),
        ])?
        .to_owned();

    let (sql, values) =
        query.build_rusqlite(SqliteQueryBuilder);

    conn.execute(&sql, values.as_params())?;

    Ok(conn.last_insert_rowid())
}
