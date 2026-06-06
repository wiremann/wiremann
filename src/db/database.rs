use anyhow::Result;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

#[derive(Clone)]
pub struct Database {
    pool: Pool<SqliteConnectionManager>,
}

impl Database {
    pub fn open(path: &str) -> Result<Self> {
        let manager = SqliteConnectionManager::file(path);

        let pool = Pool::builder().max_size(8).build(manager)?;

        {
            let conn = pool.get()?;

            conn.pragma_update(None, "journal_mode", "WAL")?;
            conn.pragma_update(None, "foreign_keys", "ON")?;

            crate::db::migrations::run(&conn)?;
        }

        Ok(Self { pool })
    }

    pub fn pool(&self) -> &Pool<SqliteConnectionManager> {
        &self.pool
    }
}

impl gpui::Global for Database {}
