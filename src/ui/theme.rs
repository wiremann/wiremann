use gpui::{Rgba, rgb, rgba};

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub struct Theme {
    // App
    pub app_bg: Rgba,

    // Titlebar
    pub titlebar_bg: Rgba,
    pub titlebar_window_icons_text: Rgba,
    pub titlebar_window_icons_bg_hover: Rgba,

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

    // Playlists page
    pub playlist_page_bg: Rgba,
    pub playlist_page_text: Rgba,

    pub playlist_header_title: Rgba,
    pub playlist_header_meta: Rgba,

    pub playlist_header_button_text: Rgba,
    pub playlist_header_button_bg: Rgba,
    pub playlist_header_button_border: Rgba,
    pub playlist_header_button_hover: Rgba,

    pub playlist_table_header_text: Rgba,
    pub playlist_table_header_border: Rgba,

    pub playlist_track_border: Rgba,
    pub playlist_track_bg_hover: Rgba,
    pub playlist_track_bg_current: Rgba,
    pub playlist_track_title_current: Rgba,

    pub playlist_sidebar_item_title: Rgba,
    pub playlist_sidebar_item_title_current: Rgba,
    pub playlist_sidebar_item_meta: Rgba,

    pub playlist_sidebar_item_bg_hover: Rgba,
    pub playlist_sidebar_item_bg_current: Rgba,

    pub playlist_empty_text: Rgba,

    // Toasts
    pub toast_bg: Rgba,
    pub toast_border: Rgba,
    pub toast_msg_text: Rgba,

    // Common
    pub border: Rgba,
    pub scrollbar_thumb: Rgba,
}

impl Default for Theme {
    #[allow(clippy::unreadable_literal)]
    fn default() -> Self {
        Theme {
            app_bg: rgb(0x0A070F),

            titlebar_bg: rgb(0x0A070F),
            titlebar_window_icons_text: rgba(0xFFFFFFCC),
            titlebar_window_icons_bg_hover: rgba(0xFFFFFF29),

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

            playlist_page_bg: rgb(0x0A070F),
            playlist_page_text: rgb(0xFFFFFF),

            playlist_header_title: rgb(0xFFFFFF),
            playlist_header_meta: rgb(0x6B6B7B),

            playlist_header_button_text: rgb(0xFFFFFF),
            playlist_header_button_bg: rgba(0x8B7BF71A),
            playlist_header_button_border: rgba(0x8B7BF74D),
            playlist_header_button_hover: rgba(0x8B7BF74D),

            playlist_table_header_text: rgb(0x5A5A6B),
            playlist_table_header_border: rgba(0xFFFFFF1A),

            playlist_track_border: rgba(0xFFFFFF1A),
            playlist_track_bg_hover: rgba(0x8B7BF71A),
            playlist_track_bg_current: rgba(0x8B7BF726),
            playlist_track_title_current: rgb(0x8B7BF7),

            playlist_sidebar_item_title: rgb(0xFFFFFF),
            playlist_sidebar_item_title_current: rgb(0x8B7BF7),
            playlist_sidebar_item_meta: rgb(0x5A5A6B),

            playlist_sidebar_item_bg_hover: rgba(0xFFFFFF0D),
            playlist_sidebar_item_bg_current: rgba(0x8B7BF726),

            playlist_empty_text: rgb(0x5A5A6B),

            toast_bg: rgb(0x0A070F),
            toast_border: rgba(0xFFFFFF29),
            toast_msg_text: rgba(0xFFFFFFCC),

            border: rgba(0xFFFFFF29),
            scrollbar_thumb: rgb(0x8B7BF7),
        }
    }
}

impl gpui::Global for Theme {}
