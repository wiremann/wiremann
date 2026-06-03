use anyhow::Result;

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

pub fn test(
    pool: &Pool<SqliteConnectionManager>,
) -> Result<()> {
    let conn = pool.get()?;

    conn.execute(
        "
        INSERT INTO tracks (
            track_hash,
            name,
            duration
        )
        VALUES (?1, ?2, ?3)
        ",
        (
            vec![1_u8, 2, 3, 4],
            "Test Track",
            123456_i64,
        ),
    )?;

    let mut stmt = conn.prepare(
        "
        SELECT name, duration
        FROM tracks
        LIMIT 1
        ",
    )?;

    let mut rows = stmt.query([])?;

    while let Some(row) = rows.next()? {
        let name: String = row.get(0)?;
        let duration: i64 = row.get(1)?;

        tracing::info!(
            "TRACK: {} ({})",
            name,
            duration
        );
    }

    Ok(())
}
