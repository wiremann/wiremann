use crate::db::tables::*;
use anyhow::Result;
use rusqlite::Connection;
use sea_query::{ColumnDef, Index, SqliteQueryBuilder, Table};

pub fn run(conn: &Connection) -> Result<()> {
    create_tracks_table(conn)?;
    create_track_sources_table(conn)?;
    create_albums_table(conn)?;
    create_artists_table(conn)?;
    create_playlists_table(conn)?;
    create_track_artists_table(conn)?;
    create_album_artists_table(conn)?;
    create_playlist_tracks_table(conn)?;create_indices(conn)?;

    Ok(())
}

pub fn create_tracks_table(conn: &Connection) -> Result<()> {
    let query = Table::create()
        .table(Tracks::Table)
        .if_not_exists()
        .col(
            ColumnDef::new(Tracks::Id)
                .integer()
                .not_null()
                .primary_key()
                .auto_increment(),
        )
        .col(
            ColumnDef::new(Tracks::TrackHash)
                .blob()
                .not_null()
                .unique_key(),
        )
        .col(ColumnDef::new(Tracks::Name).text().not_null())
        .col(ColumnDef::new(Tracks::AlbumId).integer())
        .col(ColumnDef::new(Tracks::Duration).big_integer())
        .col(ColumnDef::new(Tracks::ImageHash).blob())
        .to_owned();

    conn.execute(&query.to_string(SqliteQueryBuilder), [])?;

    Ok(())
}

pub fn create_track_sources_table(conn: &Connection) -> Result<()> {
    let query = Table::create()
        .table(TrackSources::Table)
        .if_not_exists()
        .col(
            ColumnDef::new(TrackSources::Id)
                .integer()
                .not_null()
                .primary_key()
                .auto_increment(),
        )
        .col(ColumnDef::new(TrackSources::TrackId).integer().not_null())
        .col(
            ColumnDef::new(TrackSources::Path)
                .text()
                .not_null()
                .unique_key(),
        )
        .col(ColumnDef::new(TrackSources::Size).big_integer().not_null())
        .col(
            ColumnDef::new(TrackSources::Modified)
                .big_integer()
                .not_null(),
        )
        .to_owned();

    conn.execute(&query.to_string(SqliteQueryBuilder), [])?;

    Ok(())
}

fn create_albums_table(conn: &Connection) -> Result<()> {
    let query = Table::create()
        .table(Albums::Table)
        .if_not_exists()
        .col(
            ColumnDef::new(Albums::Id)
                .integer()
                .not_null()
                .primary_key()
                .auto_increment(),
        )
        .col(ColumnDef::new(Albums::Name).text().not_null())
        .col(ColumnDef::new(Albums::ImageHash).blob())
        .to_owned();

    conn.execute(&query.to_string(SqliteQueryBuilder), [])?;

    Ok(())
}

fn create_artists_table(conn: &Connection) -> Result<()> {
    let query = Table::create()
        .table(Artists::Table)
        .if_not_exists()
        .col(
            ColumnDef::new(Artists::Id)
                .integer()
                .not_null()
                .primary_key()
                .auto_increment(),
        )
        .col(ColumnDef::new(Artists::Name).text().not_null())
        .col(ColumnDef::new(Artists::ImageHash).blob())
        .to_owned();

    conn.execute(&query.to_string(SqliteQueryBuilder), [])?;

    Ok(())
}

fn create_playlists_table(conn: &Connection) -> Result<()> {
    let query = Table::create()
        .table(Playlists::Table)
        .if_not_exists()
        .col(
            ColumnDef::new(Playlists::Id)
                .text()
                .not_null()
                .primary_key(),
        )
        .col(ColumnDef::new(Playlists::Name).text().not_null())
        .col(ColumnDef::new(Playlists::ImageHash).blob())
        .col(ColumnDef::new(Playlists::Source).text().not_null())
        .to_owned();

    conn.execute(&query.to_string(SqliteQueryBuilder), [])?;

    Ok(())
}

fn create_track_artists_table(conn: &Connection) -> Result<()> {
    let query = Table::create()
        .table(TrackArtists::Table)
        .if_not_exists()
        .primary_key(
            Index::create()
                .col(TrackArtists::TrackId)
                .col(TrackArtists::ArtistId),
        )
        .col(ColumnDef::new(TrackArtists::TrackId).integer().not_null())
        .col(ColumnDef::new(TrackArtists::ArtistId).integer().not_null())
        .to_owned();

    conn.execute(&query.to_string(SqliteQueryBuilder), [])?;

    Ok(())
}

fn create_album_artists_table(conn: &Connection) -> Result<()> {
    let query = Table::create()
        .table(AlbumArtists::Table)
        .if_not_exists()
        .primary_key(
            Index::create()
                .col(AlbumArtists::AlbumId)
                .col(AlbumArtists::ArtistId),
        )
        .col(ColumnDef::new(AlbumArtists::AlbumId).integer().not_null())
        .col(ColumnDef::new(AlbumArtists::ArtistId).integer().not_null())
        .to_owned();

    conn.execute(&query.to_string(SqliteQueryBuilder), [])?;

    Ok(())
}

fn create_playlist_tracks_table(conn: &Connection) -> Result<()> {
    let query = Table::create()
        .table(PlaylistTracks::Table)
        .if_not_exists()
        .primary_key(
            Index::create()
                .col(PlaylistTracks::PlaylistId)
                .col(PlaylistTracks::Position),
        )
        .col(
            ColumnDef::new(PlaylistTracks::PlaylistId)
                .text()
                .not_null(),
        )
        .col(ColumnDef::new(PlaylistTracks::TrackId).integer().not_null())
        .col(
            ColumnDef::new(PlaylistTracks::Position)
                .integer()
                .not_null(),
        )
        .to_owned();

    conn.execute(&query.to_string(SqliteQueryBuilder), [])?;

    Ok(())
}

fn create_indices(conn: &Connection) -> Result<()> {
    let queries = vec![
        Index::create()
            .name("idx_tracks_hash")
            .table(Tracks::Table)
            .col(Tracks::TrackHash)
            .unique()
            .to_owned(),

        Index::create()
            .name("idx_track_sources_path")
            .table(TrackSources::Table)
            .col(TrackSources::Path)
            .unique()
            .to_owned(),

        Index::create()
            .name("idx_track_sources_track_id")
            .table(TrackSources::Table)
            .col(TrackSources::TrackId)
            .to_owned(),

        Index::create()
            .name("idx_track_artists_track")
            .table(TrackArtists::Table)
            .col(TrackArtists::TrackId)
            .to_owned(),

        Index::create()
            .name("idx_track_artists_artist")
            .table(TrackArtists::Table)
            .col(TrackArtists::ArtistId)
            .to_owned(),

        Index::create()
            .name("idx_playlist_tracks_playlist")
            .table(PlaylistTracks::Table)
            .col(PlaylistTracks::PlaylistId)
            .to_owned(),
    ];

    for query in queries {
        conn.execute(&query.to_string(SqliteQueryBuilder), [])?;
    }

    Ok(())
}
