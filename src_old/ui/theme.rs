use gpui::{Rgba, rgb, rgba};

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
}

impl Default for Theme {
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
        }
    }
}

impl gpui::Global for Theme {}
