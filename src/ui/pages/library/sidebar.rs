use gpui::{
    App, Context, Div, FontWeight, Global, ImageSource, InteractiveElement, IntoElement, ObjectFit,
    ParentElement, Pixels, Render, ScrollHandle, StatefulInteractiveElement, Styled, StyledImage,
    VirtualListScrollController, Window, div, img, relative, vlist, white,
};

use crate::ui::{
    components::icons::{Icons, icon},
    theme::Theme,
};

pub struct Sidebar;

impl Render for Sidebar {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
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
                    .px_4()
                    .py_4()
                    .child("DISCOVERY")
                    .text_xs()
                    .font_weight(FontWeight::LIGHT)
                    .text_color(theme.library_sidebar_group_text),
            )
            .child(
                div()
                    .w_full()
                    .px_6()
                    .py_4()
                    .flex()
                    .items_center()
                    .gap_4()
                    .child(
                        icon(Icons::Home)
                            .size_6()
                            .text_color(theme.library_sidebar_item_text),
                    )
                    .child("Home")
                    .text_base()
                    .font_weight(FontWeight::NORMAL)
                    .text_color(theme.library_sidebar_item_text),
            )
    }
}
