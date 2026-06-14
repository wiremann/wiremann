pub mod m0001_init;
pub mod m0002_playback_queue;

use anyhow::Result;
use rusqlite::Connection;
use sea_query::{ColumnDef, SqliteQueryBuilder, Table};

pub struct Migration {
    pub version: i64,
    pub name: &'static str,
    pub run: fn(&Connection) -> Result<()>,
}

pub const MIGRATIONS: &[Migration] = &[Migration {
    version: 1,
    name: "m0001_init",
    run: m0001_init::run,
}, Migration {
    version: 2,
    name: "m0002_playback_queue",
    run: m0002_playback_queue::run,
}];

pub fn run(conn: &Connection) -> Result<()> {
    create_migrations_table(conn)?;

    let current = get_current_version(conn)?;

    for migration in MIGRATIONS {
        if migration.version > current {
            tracing::info!(
                "Running migration {} ({})",
                migration.version,
                migration.name
            );

            (migration.run)(conn)?;

            conn.execute(
                "INSERT INTO migrations (version, name, applied_at)
                 VALUES (?1, ?2, unixepoch())",
                (migration.version, migration.name),
            )?;
        }
    }

    Ok(())
}

fn create_migrations_table(conn: &Connection) -> Result<()> {
    let query = Table::create()
        .table(sea_query::Alias::new("migrations"))
        .if_not_exists()
        .col(
            ColumnDef::new(sea_query::Alias::new("version"))
                .integer()
                .not_null()
                .primary_key(),
        )
        .col(
            ColumnDef::new(sea_query::Alias::new("name"))
                .text()
                .not_null(),
        )
        .col(
            ColumnDef::new(sea_query::Alias::new("applied_at"))
                .big_integer()
                .not_null(),
        )
        .to_owned();

    conn.execute(&query.to_string(SqliteQueryBuilder), [])?;

    Ok(())
}

fn get_current_version(conn: &Connection) -> Result<i64> {
    let version = conn.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM migrations",
        [],
        |row| row.get(0),
    )?;

    Ok(version)
}
