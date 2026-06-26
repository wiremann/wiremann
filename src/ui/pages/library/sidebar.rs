use gpui::{
    Context, FontWeight, IntoElement, ParentElement, Render, Styled, Window, div, relative,
};

use crate::ui::{
    components::icons::{Icons, icon},
    theme::Theme,
};

pub struct Sidebar;

impl Sidebar {
    fn section_header(text: &'static str, theme: Theme) -> impl IntoElement {
        div()
            .px_5()
            .pt_5()
            .pb_2()
            .child(text)
            .text_sm()
            .font_weight(FontWeight::LIGHT)
            .text_color(theme.library_sidebar_group_text)
    }

    fn item(icon_type: Icons, text: &'static str, theme: Theme) -> impl IntoElement {
        div()
            .mx_3()
            .px_3()
            .py_2()
            .rounded_md()
            .flex()
            .items_center()
            .gap_3()
            .bg(theme.library_sidebar_item_bg)
            .child(
                icon(icon_type)
                    .size_4()
                    .text_color(theme.library_sidebar_item_text),
            )
            .child(
                div()
                    .child(text)
                    .font_weight(FontWeight::NORMAL)
                    .text_color(theme.library_sidebar_item_text),
            )
    }
}

impl Render for Sidebar {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();

        div()
            .w(relative(0.32))
            .h_full()
            .bg(theme.library_sidebar_bg)
            .flex()
            .flex_col()
            .child(div().h_px().mx_4().bg(theme.library_sidebar_separator))
            .child(Self::section_header("DISCOVERY", theme))
            .child(Self::item(Icons::Home, "Home", theme))
            .child(Self::item(Icons::Heart, "Favorites", theme))
            .child(Self::section_header("COLLECTION", theme))
            .child(Self::item(Icons::Music, "Tracks", theme))
            .child(Self::item(Icons::MusicList, "Albums", theme))
            .child(Self::item(Icons::User, "Artists", theme))
            .child(Self::item(Icons::Playlist, "Playlists", theme))
            .child(Self::section_header("SYSTEM", theme))
            .child(Self::item(Icons::Plugins, "Plugins", theme))
            .child(Self::item(Icons::Settings, "Settings", theme))
            .child(div().flex_grow())
            .child(
                div()
                    .h_px()
                    .mx_4()
                    .mb_3()
                    .bg(theme.library_sidebar_separator),
            )
            .child(
                div()
                    .px_5()
                    .pb_5()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .child("Library")
                            .text_sm()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(theme.library_sidebar_footer_title),
                    )
                    .child(
                        div()
                            .child("4,829 tracks")
                            .text_xs()
                            .text_color(theme.library_sidebar_footer_meta),
                    ),
            )
    }
}
