#[derive(Debug, Clone)]
pub struct DbTrack {
    pub id: i64,
    pub track_hash: Vec<u8>,
    pub name: String,
    pub album_id: Option<i64>,
    pub duration: Option<i64>,
    pub image_hash: Option<Vec<u8>>,
}

#[derive(Debug, Clone)]
pub struct DbTrackSource {
    pub id: i64,
    pub track_id: i64,
    pub path: String,
    pub size: i64,
    pub modified: i64,
}
