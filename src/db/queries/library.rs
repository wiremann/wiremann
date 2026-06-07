use anyhow::Result;
use rusqlite::Connection;

#[derive(Debug, Clone, PartialEq)]
pub struct LibraryRow {
    pub track_hash: Vec<u8>,
    pub name: String,
    pub artists: String,
    pub album: Option<String>,
    pub duration_ms: i64,
    pub path: Option<String>,
    pub size: Option<i64>,
    pub modified: Option<i64>,
    pub image_hash: Option<Vec<u8>>,
}

pub fn load_library_tracks(conn: &Connection) -> Result<Vec<LibraryRow>> {
    let mut stmt = conn.prepare(
        "SELECT t.track_hash, t.name,
            group_concat(ar.name, ', ') as artists,
            al.name as album,
            t.duration,
            MIN(s.path) as path,
            MIN(s.size) as size,
            MIN(s.modified) as modified,
            t.image_hash
         FROM tracks t
         LEFT JOIN track_artists ta ON ta.track_id = t.id
         LEFT JOIN artists ar ON ar.id = ta.artist_id
         LEFT JOIN albums al ON al.id = t.album_id
         LEFT JOIN track_sources s ON s.track_id = t.id
         GROUP BY t.id
         ORDER BY t.id",
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(LibraryRow {
            track_hash: row.get(0)?,
            name: row.get(1)?,
            artists: row
                .get::<_, Option<String>>(2)?
                .unwrap_or_else(|| "Unknown Artist".into()),
            album: row.get(3)?,
            duration_ms: row.get(4)?,
            path: row.get(5)?,
            size: row.get(6)?,
            modified: row.get(7)?,
            image_hash: row.get(8)?,
        })
    })?;

    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }

    Ok(out)
}
