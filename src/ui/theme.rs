use gpui::{Rgba, rgb, rgba};

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub struct Theme {
    // Backgrounds
    pub bg_main: Rgba,
    pub bg_titlebar: Rgba,
    pub bg_queue: Rgba,

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

    // Common
    pub border: Rgba,
}

impl Default for Theme {
    #[allow(clippy::unreadable_literal)]
    fn default() -> Self {
        Theme {
            bg_main: rgb(0x0A070F),
            bg_titlebar: rgb(0x0A0515),
            bg_queue: rgb(0x0F0C17),

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

            border: rgba(0xFFFFFF1A),
        }
    }
}

impl gpui::Global for Theme {}
