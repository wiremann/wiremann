pub mod bounds_observer;
mod element_ext;
pub mod icons;
pub mod image_cache;
pub mod scrollbar;
pub mod slider;
pub mod toasts;
pub mod virtual_grid;
pub mod virtual_list;

#[derive(Clone, Copy, PartialEq)]
pub enum Page {
    Library,
    Player,
}

impl gpui::Global for Page {}
