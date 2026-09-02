pub mod bounds_observer;
pub mod icons;
pub mod image_cache;
pub mod keybinds_overlay;
pub mod resize_handle;
pub mod scrollbar;
pub mod slider;
pub mod toasts;
pub mod virtual_grid;
pub mod virtual_list;
pub mod window_border;

#[derive(Clone, Copy, PartialEq)]
pub enum Page {
    Library,
    Player,
}
