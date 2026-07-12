use gpui::{
    Context, FontWeight, IntoElement, ParentElement, Render, Styled, Window, div, px, rems,
};

use crate::ui::theme::Theme;

pub struct AlbumsSection;

impl Render for AlbumsSection {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();

        div().w_full().h_full().flex().flex_col().child(
            div()
                .py_4()
                .px_8()
                .flex()
                .flex_col()
                .justify_center()
                .child(
                    div()
                        .text_size(rems(2.0))
                        .font_weight(FontWeight::BOLD)
                        .tracking_tight()
                        .text_color(theme.library_albums_section_title)
                        .child("Albums")
                        .child(
                            div()
                                .h(px(2.0))
                                .w_16()
                                .mt_1()
                                .bg(theme.library_albums_section_title),
                        ),
                ),
        )
    }
}
