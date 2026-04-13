use gpui::{Rgba, rgb, rgba};

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub struct Theme {
    // Backgrounds
    pub bg_main: Rgba,

    // Accent
    pub accent: Rgba,

    // Text
    pub text_primary: Rgba,
    pub text_secondary: Rgba,
    pub text_muted: Rgba,

    // White overlays
    pub white_05: Rgba,
    pub white_08: Rgba,
    pub white_10: Rgba,

    // Accent overlays
    pub accent_10: Rgba,
    pub accent_12: Rgba,
    pub accent_15: Rgba,
    pub accent_30: Rgba,

    // REFACTOR
    // Titlebar
    pub titlebar_bg: Rgba,

    // Page Switcher
    pub switcher_bg: Rgba,
    pub switcher_active: Rgba,
    pub switcher_text: Rgba,
    pub switcher_text_hover: Rgba,
    pub switcher_text_active: Rgba,

    // Player
    pub player_bg: Rgba,
    pub player_title_text: Rgba,
    pub player_artist_text: Rgba,
    pub player_icons_text: Rgba,
    pub player_icons_text_hover: Rgba,
    pub player_icons_text_active: Rgba,
    pub player_icons_bg: Rgba,
    pub player_icons_bg_hover: Rgba,
    pub player_icons_bg_active: Rgba,
    pub player_play_pause_bg: Rgba,
    pub player_play_pause_hover: Rgba,
    pub player_play_pause_text: Rgba,

    // Queue
    pub queue_bg: Rgba,
    pub queue_heading_text: Rgba,
    pub queue_show_hide_text: Rgba,
    pub queue_show_hide_text_hover: Rgba,
    pub queue_show_hide_bg_hover: Rgba,
    pub queue_item_title: Rgba,
    pub queue_item_title_current: Rgba,
    pub queue_item_artist: Rgba,
    pub queue_item_bg_hover: Rgba,
    pub queue_item_bg_current: Rgba,

    // Controlbar
    pub playback_slider_track: Rgba,
    pub playback_slider_fill: Rgba,
    pub playback_position_text: Rgba,
    pub volume_icon: Rgba,
    pub volume_slider_track: Rgba,
    pub volume_slider_fill: Rgba,

    // Library
    pub library_bg: Rgba,
    pub library_text: Rgba,

    pub library_header_text: Rgba,
    pub library_header_button_border: Rgba,
    pub library_header_button_text: Rgba,
    pub library_header_button_bg_hover: Rgba,

    pub library_playlist_bg: Rgba,
    pub library_playlist_bg_hover: Rgba,
    pub library_playlist_bg_active: Rgba,
    pub library_playlist_text: Rgba,
    pub library_playlist_title_text: Rgba,
    pub library_playlist_meta_text: Rgba,

    pub library_table_header_text: Rgba,
    pub library_table_border: Rgba,

    pub library_track_border: Rgba,
    pub library_track_bg_hover: Rgba,
    pub library_track_bg_active: Rgba,
    pub library_track_title_text_active: Rgba,

    pub library_empty_text: Rgba,

    // Common
    pub border: Rgba,
}

impl Default for Theme {
    #[allow(clippy::unreadable_literal)]
    fn default() -> Self {
        Theme {
            bg_main: rgb(0x0A070F),

            accent: rgb(0x8B7BF7),

            text_primary: rgb(0xFFFFFF),
            text_secondary: rgb(0x6B6B7B),
            text_muted: rgb(0x5A5A6B),

            white_05: rgba(0xFFFFFF0D),
            white_08: rgba(0xFFFFFF14),
            white_10: rgba(0xFFFFFF1A),

            accent_10: rgba(0x8B7BF71A),
            accent_12: rgba(0x8B7BF71F),
            accent_15: rgba(0x8B7BF726),
            accent_30: rgba(0x8B7BF74D),

            titlebar_bg: rgb(0x0A070F),

            switcher_bg: rgba(0xFFFFFF0D),
            switcher_active: rgb(0x8B7BF7),
            switcher_text: rgba(0xFFFFFFCC),
            switcher_text_hover: rgb(0xFFFFFF),
            switcher_text_active: rgb(0x0A070F),

            player_bg: rgb(0x0A070F),
            player_title_text: rgb(0xFFFFFF),
            player_artist_text: rgb(0x6B6B7B),

            player_icons_text: rgb(0x5A5A6B),
            player_icons_text_hover: rgb(0x8B7BF7),
            player_icons_text_active: rgb(0x8B7BF7),

            player_icons_bg: rgba(0xFFFFFF00),
            player_icons_bg_hover: rgba(0xFFFFFF14),
            player_icons_bg_active: rgba(0xFFFFFF14),

            player_play_pause_bg: rgb(0x8B7BF7),
            player_play_pause_hover: rgba(0x8B7BF7E6),
            player_play_pause_text: rgb(0x0A070F),

            queue_bg: rgb(0x0A070F),
            queue_heading_text: rgb(0xFFFFFF),

            queue_show_hide_text: rgb(0x6B6B7B),
            queue_show_hide_text_hover: rgb(0xFFFFFF),
            queue_show_hide_bg_hover: rgba(0xFFFFFF14),

            queue_item_title: rgb(0xFFFFFF),
            queue_item_title_current: rgb(0x8B7BF7),
            queue_item_artist: rgb(0x5A5A6B),

            queue_item_bg_hover: rgba(0xFFFFFF0D),
            queue_item_bg_current: rgba(0x8B7BF71A),

            playback_slider_track: rgba(0xFFFFFF1A),
            playback_slider_fill: rgb(0x8B7BF7),
            playback_position_text: rgb(0x6B6B7B),

            volume_icon: rgb(0x6B6B7B),
            volume_slider_track: rgba(0xFFFFFF1A),
            volume_slider_fill: rgb(0x8B7BF7),

            library_bg: rgb(0x0A070F),
            library_text: rgb(0xFFFFFF),

            library_header_text: rgb(0xFFFFFF),
            library_header_button_border: rgb(0x8B7BF7),
            library_header_button_text: rgb(0x8B7BF7),
            library_header_button_bg_hover: rgba(0x8B7BF726),

            library_playlist_bg: rgb(0x0A070F),
            library_playlist_bg_hover: rgba(0x8B7BF71A),
            library_playlist_bg_active: rgba(0x8B7BF726),
            library_playlist_text: rgb(0xFFFFFF),
            library_playlist_title_text: rgb(0xFFFFFF),
            library_playlist_meta_text: rgb(0x5A5A6B),

            library_table_header_text: rgb(0x5A5A6B),
            library_table_border: rgba(0xFFFFFF1A),

            library_track_border: rgba(0xFFFFFF1A),
            library_track_bg_hover: rgba(0x8B7BF71A),
            library_track_bg_active: rgba(0x8B7BF726),
            library_track_title_text_active: rgb(0x8B7BF7),

            library_empty_text: rgb(0x5A5A6B),

            border: rgba(0xFFFFFF1A),
        }
    }
}

impl gpui::Global for Theme {}
