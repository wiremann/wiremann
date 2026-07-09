use gpui::{
    Context, Element, FontWeight, IntoElement, ParentElement, Render, Styled, Window, div, px, rems,
};

use crate::ui::theme::Theme;

pub struct TracksSection;

impl Render for TracksSection {
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
                        .text_color(theme.library_tracks_section_title)
                        .child("Tracks")
                        .child(
                            div()
                                .h(px(2.0))
                                .w_16()
                                .mt_1()
                                .bg(theme.library_tracks_section_title),
                        ),
                )
                .child(
                    div()
                        .h_16()
                        .w_full()
                        .flex()
                        .items_center()
                        .text_xs()
                        .font_weight(FontWeight::NORMAL)
                        .text_color(theme.library_table_header_text)
                        .border_b_1()
                        .border_color(theme.library_table_border)
                        .child(
                            div()
                                .w_20()
                                .h_full()
                                .flex()
                                .items_center()
                                .justify_center()
                                .child("#"),
                        )
                        .child(
                            div()
                                .w_3_5()
                                .h_full()
                                .flex()
                                .items_center()
                                .justify_center()
                                .child("TITLE"),
                        )
                        .child(
                            div()
                                .w_1_2()
                                .h_full()
                                .flex()
                                .items_center()
                                .justify_center()
                                .child("ARTIST"),
                        )
                        .child(
                            div()
                                .w_1_2()
                                .h_full()
                                .flex()
                                .items_center()
                                .justify_center()
                                .child("ALBUM"),
                        )
                        .child(
                            div()
                                .w_24()
                                .h_full()
                                .flex()
                                .items_center()
                                .justify_center()
                                .child("DURATION"),
                        ),
                ),
        )
    }
}
