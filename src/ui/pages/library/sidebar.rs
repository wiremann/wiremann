use gpui::{
    Context, FontWeight, InteractiveElement, IntoElement, ParentElement, Render, Styled, Window,
    div, relative,
};

use crate::ui::{
    components::icons::{Icons, icon},
    pages::library::LibrarySection,
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
            .text_xs()
            .font_weight(FontWeight::LIGHT)
            .text_color(theme.library_sidebar_group_text)
    }

    fn item(
        icon_type: Icons,
        text: &'static str,
        section: LibrarySection,
        current: LibrarySection,
        theme: Theme,
    ) -> impl IntoElement {
        let active = current == section;

        div()
            .id(format!("library_sidebar_item_{}", text))
            .mx_3()
            .px_3()
            .py_2()
            .rounded_md()
            .flex()
            .items_center()
            .gap_3()
            .cursor_pointer()
            .when(active, |this| this.bg(theme.library_sidebar_item_bg_active))
            .hover(|this| this.bg(theme.library_sidebar_item_bg_hover))
            .on_click(move |_, _, cx| {
                *cx.global_mut::<LibrarySection>() = section;
                cx.notify();
            })
            .child(
                icon(icon_type)
                    .size_4()
                    .text_color(theme.library_sidebar_item_text),
            )
            .child(
                div()
                    .child(text)
                    .text_sm()
                    .font_weight(FontWeight::NORMAL)
                    .text_color(theme.library_sidebar_item_text),
            )
    }
}

impl Render for Sidebar {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let current = *cx.global::<LibrarySection>();
        div()
            .w(relative(0.32))
            .max_w_80()
            .h_full()
            .bg(theme.library_sidebar_bg)
            .border_r_1()
            .border_color(theme.border)
            .flex()
            .flex_col()
            .child(div().h_px().mx_4().bg(theme.library_sidebar_separator))
            .child(Self::section_header("DISCOVERY", theme))
            .child(Self::item(
                Icons::Home,
                "Home",
                LibrarySection::Home,
                current,
                theme,
            ))
            .child(Self::item(
                Icons::Heart,
                "Favorites",
                LibrarySection::Favorites,
                current,
                theme,
            ))
            .child(Self::section_header("COLLECTION", theme))
            .child(Self::item(
                Icons::Music,
                "Tracks",
                LibrarySection::Tracks,
                current,
                theme,
            ))
            .child(Self::item(
                Icons::MusicList,
                "Albums",
                LibrarySection::Albums,
                current,
                theme,
            ))
            .child(Self::item(
                Icons::User,
                "Artists",
                LibrarySection::Artists,
                current,
                theme,
            ))
            .child(Self::item(
                Icons::Playlist,
                "Playlists",
                LibrarySection::Playlists,
                current,
                theme,
            ))
            .child(Self::section_header("SYSTEM", theme))
            .child(Self::item(
                Icons::Plugins,
                "Plugins",
                LibrarySection::Tools,
                current,
                theme,
            ))
            .child(Self::item(
                Icons::Settings,
                "Settings",
                LibrarySection::Settings,
                current,
                theme,
            ))
            .child(div().flex_grow())
            .child(
                div()
                    .h_px()
                    .mx_4()
                    .mb_3()
                    .bg(theme.library_sidebar_separator),
            )
    }
}
