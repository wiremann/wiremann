use super::Page;
use crate::ui::theme::Theme;

use gpui::prelude::FluentBuilder;
use gpui::{
    Animation, AnimationExt as _, Context, ElementId, FontWeight, InteractiveElement, IntoElement,
    ParentElement, Render, StatefulInteractiveElement, Styled, Window, div, px,
};

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
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let page = *cx.global::<Page>();

        let active_highlight_offset = match page {
            Page::Library => 0.0,
            Page::Player => 96.0,
            Page::Playlists => 192.0,
        };

        div()
            .flex()
            .w_auto()
            .h_full()
            .rounded_full()
            .items_center()
            .justify_center()
            .bg(theme.switcher_bg)
            .border_1()
            .border_color(theme.border)
            .child({
                let pill_state = window
                    .use_keyed_state("navbar_pill", cx, |_, _| (page, active_highlight_offset));

                let (prev_page, prev_offset) = *pill_state.read(cx);
                let duration = std::time::Duration::from_millis(250);

                div()
                    .id("active_highlight")
                    .absolute()
                    .top_0()
                    .h_full()
                    .w_24()
                    .rounded_full()
                    .bg(theme.switcher_active)
                    .map(move |this| {
                        if prev_page == page {
                            this.left(px(active_highlight_offset)).into_any_element()
                        } else {
                            cx.spawn({
                                let pill_state = pill_state.clone();
                                async move |_, cx| {
                                    cx.background_executor().timer(duration).await;
                                    () = pill_state.update(cx, |state, _| {
                                        *state = (page, active_highlight_offset);
                                    });
                                }
                            })
                            .detach();

                            this.with_animation(
                                ElementId::NamedInteger("pill_move".into(), page as u64),
                                Animation::new(duration).with_easing(gpui::ease_out_quint()),
                                move |this, delta| {
                                    let x = prev_offset
                                        + (active_highlight_offset - prev_offset) * delta;
                                    this.left(px(x))
                                },
                            )
                            .into_any_element()
                        }
                    })
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
                    .on_click(|_, _, cx| *cx.global_mut::<Page>() = Page::Library)
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
                    .on_click(|_, _, cx| *cx.global_mut::<Page>() = Page::Player)
                    .when(page == Page::Player, |this| {
                        this.text_color(theme.switcher_text_active)
                    })
                    .child("Player"),
            )
            .child(
                div()
                    .id("playlists")
                    .h_full()
                    .w_24()
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_sm()
                    .text_color(theme.switcher_text)
                    .font_weight(FontWeight::MEDIUM)
                    .on_click(|_, _, cx| *cx.global_mut::<Page>() = Page::Playlists)
                    .hover(|this| {
                        if page == Page::Playlists {
                            this
                        } else {
                            this.text_color(theme.switcher_text_hover)
                        }
                    })
                    .when(page == Page::Playlists, |this| {
                        this.text_color(theme.switcher_text_active)
                    })
                    .child("Playlists"),
            )
    }
}
