use gpui::{Context, Element, FontWeight, IntoElement, ParentElement, Render, Styled, Window, div};

use crate::ui::theme::Theme;

pub struct HomeSection;

impl Render for HomeSection {
    fn render(self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();

        div().w_full().h_full().flex().flex_col().child(
            div().w_full().py_4().px_8().flex().items_center().child(
                div()
                    .text_xl()
                    .font_weight(FontWeight::BOLD)
                    .text_color(theme.library_home_section_title)
                    .child("Home"),
            ),
        )
    }
}
