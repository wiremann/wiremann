use anyhow::Result;
use rusqlite::Connection;

pub fn run(conn: &Connection) -> Result<()> {
    conn.execute(
        "
        CREATE TABLE IF NOT EXISTS tracks (
            id INTEGER PRIMARY KEY,
            track_hash BLOB UNIQUE,
            name TEXT NOT NULL,
            duration INTEGER
        )
        ",
        [],
    )?;

    Ok(())
}
