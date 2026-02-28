use super::navbar::NavBar;
use crate::ui::icons::Icons;
use crate::ui::theme::Theme;
use gpui::*;

#[derive(Clone)]
pub struct Titlebar {
    pub navbar: Entity<NavBar>,
}

impl Render for Titlebar {
    fn render(&mut self, win: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>();
        div()
            .id("titlebar")
            .h_10()
            .w_full()
            .flex()
            .items_center()
            .justify_between()
            .border_b_1()
            .border_color(theme.white_05)
            .bg(theme.bg_titlebar)
            .child(
                div()
                    .flex()
                    .flex_shrink_0()
                    .h_full()
                    .w_auto()
                    .child(self.navbar.clone()),
            )
            .child(
                div()
                    .flex()
                    .h_full()
                    .flex_1()
                    .window_control_area(WindowControlArea::Drag),
            )
            .child(
                div()
                    .h_full()
                    .flex_shrink_0()
                    .flex()
                    .justify_end()
                    .child(
                        div()
                            .id("win_min")
                            .h_full()
                            .w_12()
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(Icons::WinMin)
                            .hover(|this| this.bg(theme.white_08))
                            .window_control_area(WindowControlArea::Min),
                    )
                    .child(
                        div()
                            .id("win_max")
                            .h_full()
                            .w_12()
                            .flex()
                            .items_center()
                            .justify_center()
                            .hover(|this| this.bg(theme.white_08))
                            .child(if win.is_maximized() {
                                Icons::WinRes
                            } else {
                                Icons::WinMax
                            })
                            .window_control_area(WindowControlArea::Max),
                    )
                    .child(
                        div()
                            .h_full()
                            .w_12()
                            .flex()
                            .items_center()
                            .justify_center()
                            .hover(|this| this.bg(rgb(0xe81123)))
                            .child(Icons::WinClose)
                            .window_control_area(WindowControlArea::Close),
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
