use gpui::{
    Context, FontWeight, InteractiveElement, IntoElement, ParentElement, Render,
    StatefulInteractiveElement, Styled, Window, div, px,
};
use wgpui_motion::prelude::MotionExt;

use crate::ui::{
    animations::page_transition,
    components::icons::{Icons, icon},
    pages::library::LibraryRoutes,
    theme::Theme,
};

pub struct Sidebar;

impl Sidebar {
    fn section_header(text: &'static str, theme: &Theme) -> impl IntoElement {
        div()
            .px_6()
            .pt_8()
            .pb_4()
            .text_xs()
            .font_weight(FontWeight::MEDIUM)
            .text_color(theme.library_sidebar_group_text)
            .tracking_widest()
            .child(text)
    }

    fn item(
        icon_type: Icons,
        text: &'static str,
        section: LibraryRoutes,
        current: LibraryRoutes,
        theme: &Theme,
    ) -> impl IntoElement {
        let active = current == section;
        let hover_text_color = theme.library_sidebar_item_text_hover;
        let hover_bg_color = theme.library_sidebar_item_bg_hover;
        let active_icon_color = theme.library_sidebar_bg;
        let highlight = div()
            .absolute()
            .size(px(44.0))
            .left(px(-10.0))
            .top(px(-10.0))
            .rounded_lg()
            .bg(theme.library_sidebar_item_bg_active)
            .motion(format!("sidebar_icon_highlight_{text}"))
            .transition(page_transition())
            .opacity(f32::from(active));

        let icon = div()
            .relative()
            .size_6()
            .flex_none()
            .flex()
            .items_center()
            .justify_center()
            .child(highlight)
            .child(icon(icon_type).size_5())
            .motion(format!("sidebar_icon_{text}"))
            .transition(page_transition())
            .variant_when(active, move |style| style.text_color(active_icon_color));

        let label = div()
            .text_sm()
            .font_weight(if active {
                FontWeight::MEDIUM
            } else {
                FontWeight::NORMAL
            })
            .child(text);

        div().w_full().px_8().py_3().child(
            div()
                .id(format!("library_sidebar_item_{text}"))
                .w_full()
                .flex()
                .items_center()
                .gap_5()
                .rounded_lg()
                .cursor_pointer()
                .text_color(if active {
                    theme.library_sidebar_item_text_active
                } else {
                    theme.library_sidebar_item_text
                })
                .on_click(move |_, window, cx| {
                    *cx.global_mut::<LibraryRoutes>() = section;
                    window.refresh();
                })
                .child(icon)
                .child(label)
                .motion(format!("sidebar_item_{text}"))
                .transition(page_transition())
                .while_hovered(move |style| style.bg(hover_bg_color).text_color(hover_text_color)),
        )
    }
}

impl Render for Sidebar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let current = *cx.global::<LibraryRoutes>();

        div()
            .relative()
            .w_full()
            .h_full()
            .bg(theme.library_sidebar_bg)
            .flex()
            .flex_col()
            .child(Self::section_header("DISCOVERY", &theme))
            .child(Self::item(
                Icons::Home,
                "Home",
                LibraryRoutes::Home,
                current,
                &theme,
            ))
            .child(Self::item(
                Icons::Heart,
                "Favorites",
                LibraryRoutes::Favorites,
                current,
                &theme,
            ))
            .child(Self::item(
                Icons::Chart,
                "Stats",
                LibraryRoutes::Stats,
                current,
                &theme,
            ))
            .child(Self::section_header("COLLECTION", &theme))
            .child(Self::item(
                Icons::Music,
                "Tracks",
                LibraryRoutes::Tracks,
                current,
                &theme,
            ))
            .child(Self::item(
                Icons::Disc,
                "Albums",
                LibraryRoutes::Albums,
                current,
                &theme,
            ))
            .child(Self::item(
                Icons::User,
                "Artists",
                LibraryRoutes::Artists,
                current,
                &theme,
            ))
            .child(Self::item(
                Icons::Playlist,
                "Playlists",
                LibraryRoutes::Playlists,
                current,
                &theme,
            ))
            .child(Self::section_header("SYSTEM", &theme))
            .child(Self::item(
                Icons::Plugins,
                "Plugins",
                LibraryRoutes::Plugins,
                current,
                &theme,
            ))
            .child(Self::item(
                Icons::Settings,
                "Settings",
                LibraryRoutes::Settings,
                current,
                &theme,
            ))
            .child(div().flex_grow())
    }
}

#[cfg(test)]
mod tests {
    use crate::ui::{animations::PAGE_TRANSITION_DURATION, theme::Theme};
    use gpui::rgb;

    #[test]
    fn sidebar_colors_and_transition_match_hover_and_active_states() {
        let theme = Theme::default();

        assert_eq!(theme.library_sidebar_item_text, rgb(0x71717A));
        assert_eq!(theme.library_sidebar_item_text_hover, rgb(0xFFFFFF));
        assert_eq!(theme.library_sidebar_item_text_active, rgb(0xFFFFFF));
        assert_eq!(theme.library_sidebar_bg, rgb(0x050505));
        assert_eq!(
            PAGE_TRANSITION_DURATION,
            std::time::Duration::from_millis(300)
        );
    }
}
