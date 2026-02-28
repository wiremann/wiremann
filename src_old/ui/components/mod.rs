pub mod controlbar;
pub mod navbar;
pub mod pages;
mod queue;
mod scrollbar;
pub mod slider;
pub mod titlebar;

#[derive(Clone, Copy, PartialEq)]
pub enum Page {
    Library,
    Player,
    Playlists,
    Settings,
}

impl gpui::Global for Page {}
