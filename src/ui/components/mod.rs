pub mod controlbar;
mod element_ext;
pub mod icons;
pub mod image_cache;
pub mod navbar;
pub mod pages;
pub mod queue;
pub mod scrollbar;
pub mod slider;
pub mod toasts;
pub mod titlebar;
pub mod virtual_list;

#[derive(Clone, Copy, PartialEq)]
pub enum Page {
    Library,
    Player,
    Playlists,
}

impl gpui::Global for Page {}
