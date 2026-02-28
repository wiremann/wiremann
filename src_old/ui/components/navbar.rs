use super::Page;
use crate::ui::theme::Theme;

use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::StyledExt;

#[derive(Clone)]
pub struct NavBar;

impl NavBar {
    pub fn new() -> Self {
        NavBar {}
    }
}

impl Render for NavBar {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>();
        let page = cx.global::<Page>();

        div()
            .h_full()
            .w_auto()
            .flex()
            .gap_2()
            // .px_8()
            .child(
                div()
                    .id("library")
                    .w_auto()
                    .h_full()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .pt_1()
                    .px_6()
                    .child("Library")
                    .text_color(theme.text_muted)
                    .on_click(|_, _, cx| *cx.global_mut::<Page>() = Page::Library)
                    .when(page == &Page::Library, |this| {
                        this.child(
                            div()
                                .w_full()
                                .h(px(2.0))
                                .absolute()
                                .bottom_0()
                                .left_0()
                                .bg(theme.accent),
                        )
                        .bg(theme.white_10)
                        .font_black()
                        .text_color(theme.text_primary)
                    }),
            )
            .child(
                div()
                    .id("player")
                    .w_auto()
                    .h_full()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .pt_1()
                    .px_6()
                    .child("Player")
                    .text_color(theme.text_muted)
                    .on_click(|_, _, cx| *cx.global_mut::<Page>() = Page::Player)
                    .when(page == &Page::Player, |this| {
                        this.child(
                            div()
                                .w_full()
                                .h(px(2.0))
                                .absolute()
                                .bottom_0()
                                .left_0()
                                .bg(theme.accent),
                        )
                        .bg(theme.white_10)
                        .font_black()
                        .text_color(theme.text_primary)
                    }),
            )
            .child(
                div()
                    .id("playlists")
                    .w_auto()
                    .h_full()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .pt_1()
                    .px_6()
                    .child("Playlists")
                    .text_color(theme.text_muted)
                    .on_click(|_, _, cx| *cx.global_mut::<Page>() = Page::Playlists)
                    .when(page == &Page::Playlists, |this| {
                        this.child(
                            div()
                                .w_full()
                                .h(px(2.0))
                                .absolute()
                                .bottom_0()
                                .left_0()
                                .bg(theme.accent),
                        )
                        .bg(theme.white_10)
                        .font_black()
                        .text_color(theme.text_primary)
                    }),
            )
    }
}
