use gpui::{
    Context, Entity, FontWeight, IntoElement, ParentElement, Render, Styled, Window, div, px, rems,
};

use crate::{controller::state::AlbumId, ui::theme::Theme};

pub struct AlbumViewSection {
    pub album_id: Entity<Option<AlbumId>>,
}

impl Render for AlbumViewSection {
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
                        .text_color(theme.library_home_section_title)
                        .child("Home")
                        .child(
                            div()
                                .h(px(2.0))
                                .w_16()
                                .mt_1()
                                .bg(theme.library_home_section_title),
                        ),
                ),
        )
    }
}
