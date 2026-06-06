#[derive(Debug, Clone)]
pub struct DbArtist {
    pub id: i64,
    pub name: String,
    pub image_hash: Option<Vec<u8>>,
}
