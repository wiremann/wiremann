use anyhow::anyhow;
use gpui::{AssetSource, Result, SharedString, Window};
use rust_embed::RustEmbed;
use std::borrow::Cow;

#[derive(RustEmbed, Clone)]
#[folder = "$CARGO_MANIFEST_DIR/assets"]
#[include = "icons/**/*.svg"]
#[include = "fonts/**/*.ttf"]
pub struct Assets;

impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        if path.is_empty() {
            return Ok(None);
        }

        Self::get(path)
            .map(|f| Some(f.data))
            .ok_or_else(|| anyhow!("could not find asset at path \"{path}\""))
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        Ok(Self::iter()
            .filter_map(|p| p.starts_with(path).then(|| p.into()))
            .collect())
    }
}

impl Assets {
    #[allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]
    pub fn load_fonts(&self, window: &mut Window) -> Result<()> {
        let font_paths = self.list("fonts")?;
        let mut embedded_fonts = Vec::new();

        for font_path in font_paths {
            if font_path.ends_with(".ttf") {
                let font_bytes = Self::get(&font_path)
                    .expect("Assets should never return None")
                    .data;

                embedded_fonts.push(font_bytes.to_vec());
            }
        }

        window.text_engine().borrow_mut().add_fonts(embedded_fonts);
        Ok(())
    }
}
