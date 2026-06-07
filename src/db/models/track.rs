use rusqlite::Row;

pub struct DbTrackSummary {
    pub id: i64,
    pub track_hash: Vec<u8>,
    pub name: String,
    pub album_id: Option<i64>,
    pub duration_ms: i64,
}

impl DbTrackSummary {
    pub fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(DbTrackSummary {
            id: row.get(0)?,
            track_hash: row.get(1)?,
            name: row.get(2)?,
            album_id: row.get(3)?,
            duration_ms: row.get(4)?,
        })
    }
}
