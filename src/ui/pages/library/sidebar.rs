use gpui::{
    App, Context, Div, FontWeight, Global, ImageSource, InteractiveElement, IntoElement, ObjectFit,
    ParentElement, Pixels, Render, ScrollHandle, StatefulInteractiveElement, Styled, StyledImage,
    VirtualListScrollController, Window, div, img, relative, vlist, white,
};

use crate::ui::theme::Theme;

pub struct Sidebar;

impl Render for Sidebar {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        div()
            .w(relative(0.32))
            .h_full()
            .bg(theme.library_sidebar_bg)
            .flex()
            .flex_col()
            .overflow_x_hidden()
            .child(
                div()
                    .w_full()
                    .py_4()
                    .child("DISCOVERY")
                    .text_sm()
                    .font_weight(FontWeight::LIGHT)
                    .text_color(theme.library_sidebar_group_text),
            )
    }
}
