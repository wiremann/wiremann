use crate::{cacher::Cacher, errors::CacherError, library::TrackId, lyrics_manager::Lyrics};
use std::{
    fs,
    io::{Cursor, Write},
    path::PathBuf,
};

impl Cacher {
    fn cached_lyrics_path(&self, id: TrackId) -> PathBuf {
        let hex = hex::encode(id.0);
        let folder = &hex[0..2];

        self.app_paths
            .cache
            .join("lyrics")
            .join(folder)
            .join(format!("{hex}.lyrics.zstd"))
    }

    pub(super) fn write_cached_lyrics(
        &self,
        id: TrackId,
        cached_lyrics: &Lyrics,
    ) -> Result<(), CacherError> {
        let final_path = self.cached_lyrics_path(id);
        let tmp_path = final_path.with_extension("tmp");

        if final_path.exists() {
            return Ok(());
        }

        fs::create_dir_all(final_path.parent().unwrap())?;

        let bytes = bitcode::encode(cached_lyrics);

        let compressed = zstd::encode_all(Cursor::new(bytes), 4)?;

        {
            let mut file = fs::File::create(&tmp_path)?;
            file.write_all(&compressed)?;
            file.sync_all()?;
        }

        fs::rename(tmp_path, final_path)?;

        Ok(())
    }

    pub(super) fn read_cached_lyrics(&self, id: TrackId) -> Result<Option<Lyrics>, CacherError> {
        let path = self.cached_lyrics_path(id);

        let bytes = fs::read(path)?;

        let decompressed = zstd::decode_all(Cursor::new(bytes))?;

        let cached_lyrics: Lyrics = bitcode::decode(&decompressed)?;

        return Ok(Some(cached_lyrics));
    }
}
