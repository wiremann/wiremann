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

    // Player panel
    pub player_panel_bg: Rgba,
    pub player_panel_tab_bg: Rgba,
    pub player_panel_tab_bg_hover: Rgba,
    pub player_panel_tab_bg_active: Rgba,
    pub player_panel_tab_text: Rgba,
    pub player_panel_tab_text_hover: Rgba,
    pub player_panel_tab_text_active: Rgba,
    pub player_panel_show_hide_text: Rgba,
    pub player_panel_show_hide_text_hover: Rgba,
    pub player_panel_show_hide_bg_hover: Rgba,

    // Queue
    pub queue_item_title: Rgba,
    pub queue_item_title_current: Rgba,
    pub queue_item_artist: Rgba,
    pub queue_item_bg_hover: Rgba,
    pub queue_item_bg_current: Rgba,

    // Lyrics
    pub lyrics_text_active: Rgba,
    pub lyrics_text_inactive: Rgba,

    pub lyrics_loading_icon: Rgba,
    pub lyrics_loading_text: Rgba,

    pub lyrics_empty_text: Rgba,

    pub lyrics_fade_top: Rgba,
    pub lyrics_fade_bottom: Rgba,

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

    pub library_table_header_text: Rgba,
    pub library_table_border: Rgba,

    pub library_track_border: Rgba,
    pub library_track_bg_hover: Rgba,
    pub library_track_bg_active: Rgba,
    pub library_track_title_text_active: Rgba,

    // Sidebar
    pub library_sidebar_bg: Rgba,
    pub library_sidebar_item_text: Rgba,
    pub library_sidebar_group_text: Rgba,
    pub library_sidebar_header_text: Rgba,

    pub library_sidebar_item_bg: Rgba,
    pub library_sidebar_item_bg_hover: Rgba,
    pub library_sidebar_item_bg_active: Rgba,
    pub library_sidebar_item_text_active: Rgba,

    pub library_sidebar_indicator: Rgba,
    pub library_sidebar_indicator_glow: Rgba,

    // Library Page Home Section
    pub library_home_section_title: Rgba,

    pub library_albums_section_title: Rgba,
    pub library_artists_section_title: Rgba,
    pub library_playlists_section_title: Rgba,
    pub library_favorites_section_title: Rgba,
    pub library_settings_section_title: Rgba,
    pub library_plugins_section_title: Rgba,

    // Library Page Tracks Section
    pub library_tracks_section_title: Rgba,
    pub library_tracks_section_track_count: Rgba,
    pub library_tracks_section_table_header_text: Rgba,
    pub library_tracks_section_table_header_border: Rgba,
    pub library_tracks_section_table_slno: Rgba,
    pub library_tracks_section_table_title: Rgba,
    pub library_tracks_section_table_artist: Rgba,
    pub library_tracks_section_table_album: Rgba,
    pub library_tracks_section_table_duration: Rgba,

    // Library Page Playlists Section
    pub library_playlists_section_bg: Rgba,
    pub library_playlists_section_bg_hover: Rgba,
    pub library_playlists_section_bg_active: Rgba,
    pub library_playlists_section_text: Rgba,
    pub library_playlists_section_title_text: Rgba,
    pub library_playlists_section_meta_text: Rgba,
    pub library_playlists_section_empty_text: Rgba,

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
    pub toast_text: Rgba,

    pub toast_info_accent: Rgba,
    pub toast_success_accent: Rgba,
    pub toast_error_accent: Rgba,
    pub toast_progress_bg: Rgba,
    pub toast_progress_fill: Rgba,

    // Common
    pub border: Rgba,
    pub scrollbar_thumb: Rgba,
}

impl Default for Theme {
    #[allow(clippy::unreadable_literal)]
    fn default() -> Self {
        Theme {
            app_bg: rgb(0x050505),

            titlebar_bg: rgb(0x050505),
            titlebar_window_icons_text: rgba(0xFFFFFFCC),
            titlebar_window_icons_bg_hover: rgba(0xFFFFFF14),

            switcher_bg: rgba(0xFFFFFF0A),
            switcher_active: rgb(0xF5F5F5),
            switcher_text: rgba(0xFFFFFF99),
            switcher_text_hover: rgb(0xFFFFFF),
            switcher_text_active: rgb(0x050505),

            player_bg: rgba(0x050505A0),
            player_title_text: rgb(0xFAFAFA),
            player_artist_text: rgb(0x71717A),

            player_icons_text: rgb(0x71717A),
            player_icons_text_hover: rgb(0xFFFFFF),
            player_icons_text_active: rgb(0xFFFFFF),

            player_icons_bg: rgba(0xFFFFFF00),
            player_icons_bg_hover: rgba(0xFFFFFF0F),
            player_icons_bg_active: rgba(0xFFFFFF14),

            player_play_pause_bg: rgb(0xF5F5F5),
            player_play_pause_hover: rgb(0xFFFFFF),
            player_play_pause_text: rgb(0x050505),

            player_panel_bg: rgba(0x050505A0),
            player_panel_tab_bg: rgba(0xFFFFFF00),
            player_panel_tab_bg_hover: rgba(0xFFFFFF0A),
            player_panel_tab_bg_active: rgba(0xFFFFFF14),

            player_panel_tab_text: rgb(0x71717A),
            player_panel_tab_text_hover: rgb(0xD4D4D8),
            player_panel_tab_text_active: rgb(0xFAFAFA),

            player_panel_show_hide_text: rgb(0x71717A),
            player_panel_show_hide_text_hover: rgb(0xFFFFFF),
            player_panel_show_hide_bg_hover: rgba(0xFFFFFF0A),

            queue_item_title: rgb(0xFAFAFA),
            queue_item_title_current: rgb(0xFFFFFF),
            queue_item_artist: rgb(0x71717A),

            queue_item_bg_hover: rgba(0xFFFFFF08),
            queue_item_bg_current: rgba(0xFFFFFF10),

            lyrics_text_active: rgb(0xFFFFFF),
            lyrics_text_inactive: rgb(0xFFFFFF),

            lyrics_loading_icon: rgba(0xFFFFFFA3),
            lyrics_loading_text: rgba(0xFFFFFFA3),

            lyrics_empty_text: rgba(0xFFFFFFA3),

            lyrics_fade_top: rgb(0x000000),
            lyrics_fade_bottom: rgb(0x000000),

            playback_slider_track: rgba(0xFFFFFF14),
            playback_slider_fill: rgb(0xFAFAFA),
            playback_position_text: rgb(0x71717A),

            volume_icon: rgb(0x71717A),
            volume_slider_track: rgba(0xFFFFFF14),
            volume_slider_fill: rgb(0xFAFAFA),

            library_bg: rgb(0x050505),
            library_text: rgb(0xFAFAFA),

            library_header_text: rgb(0xFAFAFA),

            library_header_button_border: rgba(0xFFFFFF14),
            library_header_button_text: rgb(0xFAFAFA),
            library_header_button_bg_hover: rgba(0xFFFFFF0A),

            library_table_header_text: rgb(0x71717A),
            library_table_border: rgba(0xFFFFFF1A),

            library_track_border: rgba(0xFFFFFF12),
            library_track_bg_hover: rgba(0xFFFFFF08),
            library_track_bg_active: rgba(0xFFFFFF10),
            library_track_title_text_active: rgb(0xFFFFFF),

            library_sidebar_bg: rgb(0x050505),
            library_sidebar_group_text: rgb(0x71717A),

            library_sidebar_item_bg: rgba(0xFFFFFF00),
            library_sidebar_item_bg_hover: rgba(0xFFFFFF00),
            library_sidebar_item_bg_active: rgba(0xFFFFFF00),
            library_sidebar_item_text: rgb(0x71717A),
            library_sidebar_item_text_active: rgb(0xFFFFFF),

            library_sidebar_header_text: rgb(0xFAFAFA),

            library_sidebar_indicator: rgb(0xFFFFFF),
            library_sidebar_indicator_glow: rgba(0xFFFFFF4D),

            library_home_section_title: rgb(0xFFFFFF),
            library_albums_section_title: rgb(0xFFFFFF),
            library_artists_section_title: rgb(0xFFFFFF),
            library_favorites_section_title: rgb(0xFFFFFF),
            library_settings_section_title: rgb(0xFFFFFF),
            library_plugins_section_title: rgb(0xFFFFFF),

            library_tracks_section_title: rgb(0xFFFFFF),
            library_tracks_section_track_count: rgb(0x71717A),
            library_tracks_section_table_header_text: rgb(0x71717A),
            library_tracks_section_table_header_border: rgba(0xFFFFFF14),
            library_tracks_section_table_slno: rgb(0x71717A),
            library_tracks_section_table_title: rgb(0xFAFAFA),
            library_tracks_section_table_artist: rgb(0x71717A),
            library_tracks_section_table_album: rgb(0x71717A),
            library_tracks_section_table_duration: rgb(0x71717A),

            library_playlists_section_title: rgb(0xFFFFFF),
            library_playlists_section_bg: rgba(0xFFFFFF00),
            library_playlists_section_bg_hover: rgba(0xFFFFFF08),
            library_playlists_section_bg_active: rgba(0xFFFFFF10),

            library_playlists_section_text: rgb(0xFAFAFA),
            library_playlists_section_title_text: rgb(0xFAFAFA),
            library_playlists_section_meta_text: rgb(0x71717A),
            library_playlists_section_empty_text: rgb(0x71717A),

            playlist_page_bg: rgb(0x050505),
            playlist_page_text: rgb(0xFAFAFA),

            playlist_header_title: rgb(0xFAFAFA),
            playlist_header_meta: rgb(0x71717A),

            playlist_header_button_text: rgb(0xFAFAFA),
            playlist_header_button_bg: rgba(0xFFFFFF08),
            playlist_header_button_border: rgba(0xFFFFFF0F),
            playlist_header_button_hover: rgba(0xFFFFFF12),

            playlist_table_header_text: rgb(0x71717A),
            playlist_table_header_border: rgba(0xFFFFFF14),

            playlist_track_border: rgba(0xFFFFFF12),
            playlist_track_bg_hover: rgba(0xFFFFFF08),
            playlist_track_bg_current: rgba(0xFFFFFF10),
            playlist_track_title_current: rgb(0xFFFFFF),

            playlist_sidebar_item_title: rgb(0xFAFAFA),
            playlist_sidebar_item_title_current: rgb(0xFFFFFF),
            playlist_sidebar_item_meta: rgb(0x71717A),

            playlist_sidebar_item_bg_hover: rgba(0xFFFFFF08),
            playlist_sidebar_item_bg_current: rgba(0xFFFFFF10),

            playlist_empty_text: rgb(0x71717A),

            toast_bg: rgb(0x111113),
            toast_border: rgba(0xFFFFFF0F),
            toast_text: rgba(0xFFFFFFE6),

            toast_info_accent: rgb(0xFFFFFF),
            toast_success_accent: rgb(0x22C55E),
            toast_error_accent: rgb(0xEF4444),

            toast_progress_bg: rgba(0xFFFFFF14),
            toast_progress_fill: rgb(0xFAFAFA),

            border: rgba(0xFFFFFF22),

            scrollbar_thumb: rgba(0xFFFFFF26),
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub struct DominantColors {
    pub color1: Rgba,
    pub color2: Rgba,
    pub color3: Rgba,
    pub color4: Rgba,
}

impl Default for DominantColors {
    fn default() -> Self {
        DominantColors {
            color1: rgb(0x000000),
            color2: rgb(0x000000),
            color3: rgb(0x000000),
            color4: rgb(0x000000),
        }
    }
}

impl gpui::Global for Theme {}
impl gpui::Global for DominantColors {}
