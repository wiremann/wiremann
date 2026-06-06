#[derive(Debug, Clone)]
pub struct DbAlbum {
    pub id: i64,
    pub name: String,
    pub image_hash: Option<Vec<u8>>,
}
