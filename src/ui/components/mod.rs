pub mod controlbar;
pub mod image_cache;
pub mod navbar;
pub mod pages;
pub mod queue;
pub mod scrollbar;
pub mod slider;
pub mod titlebar;
pub mod icons;
mod element_ext;

#[derive(Clone, Copy, PartialEq)]
pub enum Page {
    Library,
    Player,
    Playlists,
    Settings,
}

impl gpui::Global for Page {}
