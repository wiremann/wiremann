use anyhow::Result;
use rusqlite::Connection;

use sea_query::{ColumnDef, Iden, SqliteQueryBuilder, Table};

#[derive(Iden)]
pub enum Tracks {
    Table,
    Id,
    TrackHash,
    Name,
    AlbumId,
    Duration,
    ImageHash,
}

pub fn run(conn: &Connection) -> Result<()> {
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
        .col(ColumnDef::new(Tracks::Duration).integer())
        .col(ColumnDef::new(Tracks::ImageHash).blob())
        .to_owned();

    conn.execute(&query.to_string(SqliteQueryBuilder), [])?;

    Ok(())
}
