use super::Page;
use crate::ui::theme::Theme;

use gpui::prelude::FluentBuilder;
use gpui::{
    Context, FontWeight, InteractiveElement, IntoElement, ParentElement, Render,
    StatefulInteractiveElement, Styled, Window, div, px,
};
use wgpui_motion::prelude::{DurationExt, MotionExt, Transition, quint_out};

#[derive(Clone)]
pub struct NavBar;

impl NavBar {
    #[allow(clippy::new_without_default)]
    #[must_use]
    pub fn new() -> Self {
        NavBar {}
    }
}

impl Render for NavBar {
    #[allow(clippy::too_many_lines)]
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let page = *cx.global::<Page>();

        let active_highlight_offset = match page {
            Page::Library => 0.0,
            Page::Player => 96.0,
        };

        div()
            .flex()
            .w_auto()
            .h_full()
            .py_1()
            .rounded_full()
            .items_center()
            .justify_center()
            .bg(theme.switcher_bg)
            .border_1()
            .border_color(theme.border)
            .child({
                div()
                    .id("active_highlight")
                    .absolute()
                    .top_0()
                    .h_full()
                    .w_24()
                    .rounded_full()
                    .bg(theme.switcher_active)
                    .left(px(active_highlight_offset))
                    .motion("navbar-pill")
                    .transition(Transition::new(250.ms()).curve(quint_out()))
            })
            .child(
                div()
                    .id("library")
                    .h_full()
                    .w_24()
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_sm()
                    .text_color(theme.switcher_text)
                    .font_weight(FontWeight::MEDIUM)
                    .hover(|this| {
                        if page == Page::Library {
                            this
                        } else {
                            this.text_color(theme.switcher_text_hover)
                        }
                    })
                    .on_click(|_, _, cx| {
                        cx.stop_propagation();
                        *cx.global_mut::<Page>() = Page::Library;
                    })
                    .when(page == Page::Library, |this| {
                        this.text_color(theme.switcher_text_active)
                    })
                    .child("Library"),
            )
            .child(
                div()
                    .id("player")
                    .h_full()
                    .w_24()
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_sm()
                    .text_color(theme.switcher_text)
                    .font_weight(FontWeight::MEDIUM)
                    .hover(|this| {
                        if page == Page::Player {
                            this
                        } else {
                            this.text_color(theme.switcher_text_hover)
                        }
                    })
                    .on_click(|_, _, cx| {
                        cx.stop_propagation();
                        *cx.global_mut::<Page>() = Page::Player;
                    })
                    .when(page == Page::Player, |this| {
                        this.text_color(theme.switcher_text_active)
                    })
                    .child("Player"),
            )
    }
}
