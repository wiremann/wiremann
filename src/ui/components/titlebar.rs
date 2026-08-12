use super::navbar::NavBar;
use crate::ui::components::icons::Icons;
use crate::ui::theme::Theme;
use gpui::{
    App, AppContext, Context, Entity, InteractiveElement, IntoElement, MouseButton, ParentElement,
    Render, StatefulInteractiveElement, Styled, Window, div, white,
};

#[derive(Clone)]
pub struct Titlebar {
    pub navbar: Entity<NavBar>,
}

impl Render for Titlebar {
    #[allow(clippy::unreadable_literal)]
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>();
        let is_maximized = window.is_maximized();

        div()
            .id("titlebar")
            .h_12()
            .w_full()
            .flex()
            .items_center()
            .justify_between()
            .border_b_1()
            .border_color(theme.border)
            .bg(theme.titlebar_bg)
            .on_mouse_down(MouseButton::Left, |_, window, _| {
                if window.is_fullscreen() {
                    window.toggle_fullscreen();
                }
                window.start_window_move();
            })
            .on_click(|event, window, _| {
                if event.click_count() >= 2 {
                    window.zoom_window();
                }
            })
            .child(div().id("titlebar_drag_left").h_full().flex_1())
            .child(
                div()
                    .id("titlebar_nav")
                    .h_full()
                    .flex()
                    .items_center()
                    .px_4()
                    .py_1()
                    .text_color(white())
                    .child(self.navbar.clone()),
            )
            .child(
                div()
                    .id("titlebar_right")
                    .h_full()
                    .flex_1()
                    .flex()
                    .items_center()
                    .justify_end()
                    .gap_1()
                    .px_4()
                    .text_color(white())
                    .child(
                        div()
                            .id("help_btn")
                            .h_8()
                            .w_8()
                            .rounded_full()
                            .flex()
                            .items_center()
                            .justify_center()
                            .hover(|this| this.bg(theme.titlebar_window_icons_bg_hover))
                            .text_color(theme.titlebar_window_icons_text)
                            .cursor_pointer()
                            .on_mouse_down(MouseButton::Left, |_, window, cx| {
                                window.prevent_default();
                                cx.stop_propagation();
                            })
                            .on_click({
                                move |_, _, cx| {
                                    let handle = cx
                                        .global::<
                                            crate::ui::components::keybinds_overlay::KeybindsOverlayHandle,
                                        >()
                                        .clone();
                                    handle.0.update(cx, |overlay, cx| overlay.toggle(cx));
                                }
                            })
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .child("?"),
                            ),
                    )
                    .child(
                        div()
                            .id("win_min_btn")
                            .h_8()
                            .w_8()
                            .rounded_full()
                            .flex()
                            .items_center()
                            .justify_center()
                            .hover(|this| this.bg(theme.titlebar_window_icons_bg_hover))
                            .text_color(theme.titlebar_window_icons_text)
                            .cursor_pointer()
                            .on_mouse_down(MouseButton::Left, |_, window, cx| {
                                window.prevent_default();
                                cx.stop_propagation();
                            })
                            .on_click(|_, window, _| window.minimize_window())
                            .child(Icons::WinMin),
                    )
                    .child(
                        div()
                            .id("win_max_btn")
                            .h_8()
                            .w_8()
                            .rounded_full()
                            .flex()
                            .items_center()
                            .justify_center()
                            .hover(|this| this.bg(theme.titlebar_window_icons_bg_hover))
                            .text_color(theme.titlebar_window_icons_text)
                            .cursor_pointer()
                            .on_mouse_down(MouseButton::Left, |_, window, cx| {
                                window.prevent_default();
                                cx.stop_propagation();
                            })
                            .on_click(|_, window, _| window.zoom_window())
                            .child(if is_maximized {
                                Icons::WinRes
                            } else {
                                Icons::WinMax
                            }),
                    )
                    .child(
                        div()
                            .id("win_close_btn")
                            .h_8()
                            .w_8()
                            .rounded_full()
                            .flex()
                            .items_center()
                            .justify_center()
                            .hover(|this| this.bg(theme.titlebar_window_icons_bg_hover))
                            .text_color(theme.titlebar_window_icons_text)
                            .cursor_pointer()
                            .on_mouse_down(MouseButton::Left, |_, window, cx| {
                                window.prevent_default();
                                cx.stop_propagation();
                            })
                            .on_click(|_, window, _| window.remove_window())
                            .child(Icons::WinClose),
                    ),
            )
    }
}

impl Titlebar {
    pub fn new(cx: &mut App) -> Titlebar {
        let navbar = cx.new(|_| NavBar::new());

        Titlebar { navbar }
    }
}
