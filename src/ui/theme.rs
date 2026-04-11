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

            titlebar_bg: rgb(0x090909),

            switcher_bg: rgb(0x161616),
            switcher_active: rgb(0xFFFFFF),
            switcher_text: rgba(0xFFFFFF99),
            switcher_text_hover: rgb(0xFFFFFF),
            switcher_text_active: rgb(0x090909),

            player_bg: rgb(0x0a0a0a),
            player_title_text: rgba(0xFFFFFFF2),
            player_artist_text: rgba(0xFFFFFF66),
            player_icons_text: rgba(0xFFFFFF80),
            player_icons_text_hover: rgb(0xFFFFFF),
            player_icons_text_active: rgb(0xFFFFFF),
            player_icons_bg: rgba(0xFFFFFF00),
            player_icons_bg_hover: rgba(0xFFFFFF1A),
            player_icons_bg_active: rgba(0xFFFFFF1A),
            player_play_pause_bg: rgb(0xFFFFFF),
            player_play_pause_hover: rgba(0xFFFFFFE6),
            player_play_pause_text: rgb(0x090909),

            queue_bg: rgba(0x090909),
            queue_heading_text: rgba(0xFFFFFFF2),
            queue_show_hide_text: rgba(0xFFFFFFCC),
            queue_show_hide_text_hover: rgb(0xFFFFFF),
            queue_show_hide_bg_hover: rgba(0xFFFFFF1A),
            queue_item_title: rgba(0xFFFFFFF2),
            queue_item_title_current: rgb(0xFFFFFF),
            queue_item_artist: rgba(0xFFFFFF80),
            queue_item_bg_hover: rgba(0xFFFFFF1A),
            queue_item_bg_current: rgba(0xFFFFFF1A),

            playback_slider_track: rgba(0xFFFFFF1A),
            playback_slider_fill: rgb(0xFFFFFF),
            playback_position_text: rgba(0xFFFFFF66),
            volume_icon: rgba(0xFFFFFF66),
            volume_slider_track: rgba(0xFFFFFF1A),
            volume_slider_fill: rgb(0xFFFFFF),

            border: rgba(0xFFFFFF33),
        }
    }
}

impl gpui::Global for Theme {}
