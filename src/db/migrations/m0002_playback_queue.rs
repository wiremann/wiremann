use crate::db::tables::*;
use anyhow::Result;
use rusqlite::Connection;
use sea_query::{ColumnDef, ForeignKey, Index, SqliteQueryBuilder, Table};

pub fn run(conn: &Connection) -> Result<()> {
    create_playbacks_table(conn)?;
    create_queue_table(conn)?;

    Ok(())
}

fn create_playbacks_table(conn: &Connection) -> Result<()> {
    let query = Table::create()
        .table(Playbacks::Table)
        .if_not_exists()
        .col(
            ColumnDef::new(Playbacks::Id)
                .integer()
                .not_null()
                .primary_key(),
        )
        .col(ColumnDef::new(Playbacks::CurrentTrack).blob())
        .col(ColumnDef::new(Playbacks::CurrentPlaylist).text())
        .col(ColumnDef::new(Playbacks::CurrentIndex).integer().not_null())
        .col(ColumnDef::new(Playbacks::Status).text().not_null())
        .col(ColumnDef::new(Playbacks::Position).big_integer().not_null())
        .col(ColumnDef::new(Playbacks::Volume).double().not_null())
        .col(ColumnDef::new(Playbacks::Mute).boolean().not_null())
        .col(ColumnDef::new(Playbacks::Shuffling).boolean().not_null())
        .col(ColumnDef::new(Playbacks::Repeat).boolean().not_null())
        .to_owned();

    conn.execute(&query.to_string(SqliteQueryBuilder), [])?;

    Ok(())
}

fn create_queue_table(conn: &Connection) -> Result<()> {
    let query = Table::create()
        .table(Queue::Table)
        .if_not_exists()
        .col(
            ColumnDef::new(Queue::Position)
                .integer()
                .not_null()
                .primary_key(),
        )
        .col(ColumnDef::new(Queue::TrackHash).blob().not_null())
        .to_owned();

    conn.execute(&query.to_string(SqliteQueryBuilder), [])?;

    Ok(())
}
