#[derive(Debug, Clone)]
pub struct DbPlaylist {
    pub id: String,
    pub name: String,
    pub source: String,
    pub image_hash: Option<Vec<u8>>,
}
