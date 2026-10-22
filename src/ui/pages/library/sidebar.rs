use gpui::{
    Context, FontWeight, InteractiveElement, IntoElement, ParentElement, Render,
    StatefulInteractiveElement, Styled, Window, div, px,
};
use wgpui_motion::prelude::MotionExt;

use crate::ui::{
    animations::{page_transition, sidebar_transition},
    components::icons::{Icons, icon},
    pages::library::LibraryRoutes,
    theme::Theme,
};

pub(super) const COLLAPSED_SIDEBAR_WIDTH: f32 = 72.0;

pub struct Sidebar {
    pub(super) collapsed: gpui::Entity<bool>,
}

impl Sidebar {
    fn section_header(text: &'static str, collapsed: bool, theme: &Theme) -> impl IntoElement {
        div()
            .relative()
            .w_full()
            .px_6()
            .pt_8()
            .pb_4()
            .text_xs()
            .font_weight(FontWeight::MEDIUM)
            .text_color(theme.library_sidebar_group_text)
            .tracking_widest()
            .overflow_hidden()
            .child(
                div()
                    .w_full()
                    .truncate()
                    .opacity(if collapsed { 0.0 } else { 1.0 })
                    .motion(format!("sidebar_header_{text}"))
                    .transition(page_transition())
                    .child(text),
            )
            .child(
                div()
                    .absolute()
                    .left(px(24.0))
                    .top(px(14.0))
                    .w(px(if collapsed { 24.0 } else { 0.0 }))
                    .h(px(1.0))
                    .bg(theme.library_sidebar_group_text)
                    .opacity(if collapsed { 1.0 } else { 0.0 })
                    .motion(format!("sidebar_separator_{text}"))
                    .transition(page_transition()),
            )
    }

    fn item(
        icon_type: Icons,
        text: &'static str,
        section: LibraryRoutes,
        current: LibraryRoutes,
        collapsed: bool,
        theme: &Theme,
    ) -> impl IntoElement {
        let active = current == section;
        let hover_text_color = theme.library_sidebar_item_text_hover;
        let active_icon_color = theme.library_sidebar_bg;
        let highlight = div()
            .absolute()
            .size(px(44.0))
            .left(px(-10.0))
            .top(px(-10.0))
            .rounded_lg()
            .bg(theme.library_sidebar_item_bg_active)
            .motion(format!("sidebar_icon_highlight_{text}"))
            .transition(sidebar_transition())
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
            .transition(sidebar_transition())
            .variant_when(active, move |style| style.text_color(active_icon_color));

        let label = div()
            .flex_1()
            .min_w_0()
            .overflow_hidden()
            .opacity(if collapsed { 0.0 } else { 1.0 })
            .motion(format!("sidebar_label_{text}"))
            .transition(page_transition())
            .child(
                div()
                    .text_sm()
                    .font_weight(if active {
                        FontWeight::MEDIUM
                    } else {
                        FontWeight::NORMAL
                    })
                    .truncate()
                    .child(text),
            );

        div()
            .id(format!("library_sidebar_item_{text}"))
            .w_full()
            .px_8()
            .py_3()
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
            .motion(format!("sidebar_item_{text}"))
            .transition(sidebar_transition())
            .while_hovered(move |style| style.text_color(hover_text_color))
            .child(
                div()
                    .w_full()
                    .flex()
                    .items_center()
                    .gap_5()
                    .rounded_lg()
                    .child(icon)
                    .child(label),
            )
    }

    fn collapse_control(&self, theme: &Theme) -> impl IntoElement {
        let toggle = self.collapsed.clone();
        let click_toggle = toggle.clone();
        let key_toggle = toggle;
        let hover_text_color = theme.library_sidebar_item_text_hover;

        div()
            .id("library_sidebar_collapse")
            .w_full()
            .px_8()
            .py_3()
            .flex()
            .items_center()
            .cursor_pointer()
            .text_color(theme.library_sidebar_item_text)
            .tab_index(0)
            .on_click(move |_, _, cx| {
                click_toggle.update(cx, |collapsed, cx| {
                    *collapsed = !*collapsed;
                    cx.notify();
                });
            })
            .on_key_down(move |event, _, cx| {
                if !event.is_held && matches!(event.keystroke.key.as_str(), "enter" | "space") {
                    key_toggle.update(cx, |collapsed, cx| {
                        *collapsed = !*collapsed;
                        cx.notify();
                    });
                }
            })
            .motion("library_sidebar_collapse_control")
            .transition(sidebar_transition())
            .while_hovered(move |style| style.text_color(hover_text_color))
            .child(icon(Icons::Menu).size_6())
    }
}

impl Render for Sidebar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let current = *cx.global::<LibraryRoutes>();
        let collapsed = *self.collapsed.read(cx);

        div()
            .relative()
            .w_full()
            .h_full()
            .bg(theme.library_sidebar_bg)
            .flex()
            .flex_col()
            .child(Self::section_header("DISCOVERY", collapsed, &theme))
            .child(Self::item(
                Icons::Home,
                "Home",
                LibraryRoutes::Home,
                current,
                collapsed,
                &theme,
            ))
            .child(Self::item(
                Icons::Heart,
                "Favorites",
                LibraryRoutes::Favorites,
                current,
                collapsed,
                &theme,
            ))
            .child(Self::item(
                Icons::Chart,
                "Stats",
                LibraryRoutes::Stats,
                current,
                collapsed,
                &theme,
            ))
            .child(Self::section_header("COLLECTION", collapsed, &theme))
            .child(Self::item(
                Icons::Music,
                "Tracks",
                LibraryRoutes::Tracks,
                current,
                collapsed,
                &theme,
            ))
            .child(Self::item(
                Icons::Disc,
                "Albums",
                LibraryRoutes::Albums,
                current,
                collapsed,
                &theme,
            ))
            .child(Self::item(
                Icons::User,
                "Artists",
                LibraryRoutes::Artists,
                current,
                collapsed,
                &theme,
            ))
            .child(Self::item(
                Icons::Playlist,
                "Playlists",
                LibraryRoutes::Playlists,
                current,
                collapsed,
                &theme,
            ))
            .child(Self::section_header("SYSTEM", collapsed, &theme))
            .child(Self::item(
                Icons::Plugins,
                "Plugins",
                LibraryRoutes::Plugins,
                current,
                collapsed,
                &theme,
            ))
            .child(Self::item(
                Icons::Settings,
                "Settings",
                LibraryRoutes::Settings,
                current,
                collapsed,
                &theme,
            ))
            .child(div().flex_grow())
            .child(self.collapse_control(&theme))
    }
}

#[cfg(test)]
mod tests {
    use super::COLLAPSED_SIDEBAR_WIDTH;
    use crate::ui::{
        animations::{
            PAGE_TRANSITION_DURATION, SIDEBAR_HOVER_SPRING, page_transition, sidebar_transition,
        },
        components::icons::{IconNamed, Icons},
        theme::Theme,
    };
    use gpui::rgb;
    use wgpui_motion::prelude::{Channel, Spring};

    #[test]
    fn sidebar_colors_and_transition_match_hover_and_active_states() {
        let theme = Theme::default();

        assert_eq!(theme.library_sidebar_item_text, rgb(0x71717A));
        assert_eq!(theme.library_sidebar_item_text_hover, rgb(0xFFFFFF));
        assert_eq!(theme.library_sidebar_item_text_active, rgb(0xFFFFFF));
        assert_eq!(theme.library_sidebar_bg, rgb(0x050505));
        assert_eq!(Icons::Menu.path(), "icons/menu.svg");
        assert_eq!(
            PAGE_TRANSITION_DURATION,
            std::time::Duration::from_millis(300)
        );
        assert_eq!(SIDEBAR_HOVER_SPRING, Spring::SNAPPY.with_mass(0.5));
        let transition = sidebar_transition();
        assert_eq!(
            transition.timing_for(Channel::Text).spring(),
            Some(SIDEBAR_HOVER_SPRING)
        );
        assert_eq!(
            transition.timing_for(Channel::Effects).spring(),
            Some(SIDEBAR_HOVER_SPRING)
        );
        assert_eq!(
            transition.timing_for(Channel::Text).duration(),
            transition.timing_for(Channel::Effects).duration()
        );
        assert!(
            transition.timing_for(Channel::Text).duration()
                <= std::time::Duration::from_millis(350)
        );
        assert_eq!(COLLAPSED_SIDEBAR_WIDTH, 72.0);
        assert_eq!(
            page_transition().timing_for(Channel::Layout).duration(),
            PAGE_TRANSITION_DURATION
        );
    }
}
