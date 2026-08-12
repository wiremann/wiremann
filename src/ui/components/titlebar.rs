use super::navbar::NavBar;
use crate::controller::state::PlaybackStatus;
use crate::controller::Controller;
use crate::ui::components::icons::{Icon, Icons};
use crate::ui::components::image_cache::ImageCache;
use crate::ui::components::Page;
use crate::ui::theme::Theme;
use gpui::prelude::FluentBuilder;
use gpui::{
    Animation, AnimationExt, App, AppContext, Context, ElementId, Entity, InteractiveElement,
    IntoElement, MouseButton, ParentElement, Render, StatefulInteractiveElement, Styled,
    StyledImage, Window, div, img, px, white,
};
use std::time::Duration;

#[derive(Clone)]
pub struct Titlebar {
    pub navbar: Entity<NavBar>,
}

impl Render for Titlebar {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>().clone();
        let page = *cx.global::<Page>();
        let is_maximized = window.is_maximized();

        let controller = cx.global::<Controller>().clone();
        let state = controller.state.read(cx);
        let is_playing = state.playback.status == PlaybackStatus::Playing;
        let current_track = state.playback.current.and_then(|id| {
            state.library.tracks.get(&id).map(|t| t.clone())
        });
        let current_image = cx.global::<ImageCache>().current.clone();

        let show_mini_player = page != Page::Player && current_track.is_some();

        div()
            .id("titlebar")
            .h_12()
            .w_full()
            .flex()
            .items_center()
            .border_b_1()
            .border_color(theme.border)
            .bg(theme.titlebar_bg)
            .on_mouse_down(MouseButton::Left, |_, window, _| {
                if window.is_fullscreen() {
                    window.toggle_fullscreen();
                }
                window.start_window_move();
            })
            .on_click(|event, window, _| {
                if event.click_count() >= 2 {
                    window.zoom_window();
                }
            })
            .child(
                div()
                    .id("titlebar_left")
                    .flex_1()
                    .flex()
                    .items_center()
                    .justify_start()
                    .when(show_mini_player, |parent| {
                        let track = current_track.as_ref().unwrap();
                        parent.child(mini_player(&theme, track, &current_image, is_playing, controller.clone()))
                    }),
            )
            .child(
                div()
                    .id("titlebar_nav")
                    .flex_shrink_0()
                    .flex()
                    .items_center()
                    .px_4()
                    .py_1()
                    .text_color(white())
                    .child(self.navbar.clone()),
            )
            .child(
                div()
                    .id("titlebar_right")
                    .flex_1()
                    .flex()
                    .items_center()
                    .justify_end()
                    .gap_1()
                    .px_4()
                    .text_color(white())
                    .child(
                        div()
                            .id("help_btn")
                            .h_8()
                            .w_8()
                            .rounded_full()
                            .flex()
                            .items_center()
                            .justify_center()
                            .hover(|this| this.bg(theme.titlebar_window_icons_bg_hover))
                            .text_color(theme.titlebar_window_icons_text)
                            .cursor_pointer()
                            .on_mouse_down(MouseButton::Left, |_, window, cx| {
                                window.prevent_default();
                                cx.stop_propagation();
                            })
                            .on_click({
                                move |_, _, cx| {
                                    let handle = cx
                                        .global::<
                                            crate::ui::components::keybinds_overlay::KeybindsOverlayHandle,
                                        >()
                                        .clone();
                                    handle.0.update(cx, |overlay, cx| overlay.toggle(cx));
                                }
                            })
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .child("?"),
                            ),
                    )
                    .child(
                        div()
                            .id("win_min_btn")
                            .h_8()
                            .w_8()
                            .rounded_full()
                            .flex()
                            .items_center()
                            .justify_center()
                            .hover(|this| this.bg(theme.titlebar_window_icons_bg_hover))
                            .text_color(theme.titlebar_window_icons_text)
                            .cursor_pointer()
                            .on_mouse_down(MouseButton::Left, |_, window, cx| {
                                window.prevent_default();
                                cx.stop_propagation();
                            })
                            .on_click(|_, window, _| window.minimize_window())
                            .child(Icons::WinMin),
                    )
                    .child(
                        div()
                            .id("win_max_btn")
                            .h_8()
                            .w_8()
                            .rounded_full()
                            .flex()
                            .items_center()
                            .justify_center()
                            .hover(|this| this.bg(theme.titlebar_window_icons_bg_hover))
                            .text_color(theme.titlebar_window_icons_text)
                            .cursor_pointer()
                            .on_mouse_down(MouseButton::Left, |_, window, cx| {
                                window.prevent_default();
                                cx.stop_propagation();
                            })
                            .on_click(|_, window, _| window.zoom_window())
                            .child(if is_maximized {
                                Icons::WinRes
                            } else {
                                Icons::WinMax
                            }),
                    )
                    .child(
                        div()
                            .id("win_close_btn")
                            .h_8()
                            .w_8()
                            .rounded_full()
                            .flex()
                            .items_center()
                            .justify_center()
                            .hover(|this| this.bg(theme.titlebar_window_icons_bg_hover))
                            .text_color(theme.titlebar_window_icons_text)
                            .cursor_pointer()
                            .on_mouse_down(MouseButton::Left, |_, window, cx| {
                                window.prevent_default();
                                cx.stop_propagation();
                            })
                            .on_click(|_, window, _| window.remove_window())
                            .child(Icons::WinClose),
                    ),
            )
    }
}

fn mini_player(
    theme: &Theme,
    track: &crate::controller::state::Track,
    current_image: &Option<std::sync::Arc<gpui::RenderImage>>,
    is_playing: bool,
    controller: Controller,
) -> impl IntoElement {
    let title = track.title.clone();
    let char_w = 8.5_f32;
    let container_w = 100.0_f32;
    let est_text_w = title.len() as f32 * char_w;
    let scroll_range = (est_text_w - container_w).max(0.0);

    div()
        .id("mini_player")
        .h_full()
        .flex()
        .items_center()
        .gap_2()
        .px_3()
        .child(
            div()
                .id("mini_album_art")
                .size_8()
                .flex_shrink_0()
                .rounded_md()
                .overflow_hidden()
                .child(match current_image {
                    Some(img_src) => img(img_src.clone())
                        .object_fit(gpui::ObjectFit::Contain)
                        .size_full()
                        .into_any_element(),
                    None => {
                        let text_color = theme.titlebar_window_icons_text;
                        Icon::new(Icons::Music)
                            .size_full()
                            .text_color(text_color)
                            .into_any_element()
                    }
                }),
        )
        .child(
            div()
                .id("mini_track_info")
                .flex_col()
                .gap_0()
                .min_w_0()
                .child({
                    if scroll_range > 0.0 {
                        let scroll_dur = Duration::from_secs_f32(
                            1.5 + title.len() as f32 * 0.12,
                        );
                        let total_width = container_w + scroll_range;
                        div()
                            .id("mini_title_scroll_container")
                            .w(px(container_w))
                            .overflow_hidden()
                            .child(
                                div()
                                    .id("mini_title_scroll_text")
                                    .flex()
                                    .flex_row()
                                    .text_color(white())
                                    .text_sm()
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .whitespace_nowrap()
                                    .child(title.clone())
                                    .child(title)
                                    .with_animation(
                                        ElementId::Name("mini_title_scroll".into()),
                                        Animation::new(scroll_dur).repeat(),
                                        move |this, delta| {
                                            this.left(px(-total_width * delta))
                                        },
                                    ),
                            )
                            .into_any_element()
                    } else {
                        div()
                            .id("mini_title_static")
                            .w(px(container_w))
                            .overflow_hidden()
                            .child(
                                div()
                                    .text_color(white())
                                    .text_sm()
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .whitespace_nowrap()
                                    .text_ellipsis()
                                    .child(title),
                            )
                            .into_any_element()
                    }
                })
                .child(
                    div()
                        .text_color(theme.titlebar_window_icons_text)
                        .text_xs()
                        .overflow_hidden()
                        .whitespace_nowrap()
                        .text_ellipsis()
                        .child(track.artist.clone()),
                ),
        )
        .child(
            div()
                .id("mini_controls")
                .flex()
                .items_center()
                .gap_1()
                .child(
                    div()
                        .id("mini_seek_back")
                        .h_8()
                        .w_8()
                        .rounded_md()
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_color(theme.titlebar_window_icons_text)
                        .text_xs()
                        .cursor_pointer()
                        .hover(|this| {
                            this.bg(theme.titlebar_window_icons_bg_hover)
                        })
                        .on_mouse_down(MouseButton::Left, |_, window, cx| {
                            window.prevent_default();
                            cx.stop_propagation();
                        })
                        .on_click({
                            let controller = controller.clone();
                            move |_, _, cx| {
                                let state = controller.state.read(cx);
                                let pos = state.playback.position;
                                let new_pos = pos.saturating_sub(Duration::from_secs(5));
                                controller.seek(new_pos);
                            }
                        })
                        .child(Icons::Prev),
                )
                .child(
                    div()
                        .id("mini_play_pause")
                        .h_8()
                        .w_8()
                        .rounded_md()
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_color(theme.titlebar_window_icons_text)
                        .cursor_pointer()
                        .hover(|this| {
                            this.bg(theme.titlebar_window_icons_bg_hover)
                        })
                        .on_mouse_down(MouseButton::Left, |_, window, cx| {
                            window.prevent_default();
                            cx.stop_propagation();
                        })
                        .on_click(move |_, _, cx| {
                            let controller = cx.global::<Controller>().clone();
                            let status = controller.state.read(cx).playback.status;
                            if status == PlaybackStatus::Playing {
                                controller.pause();
                            } else {
                                controller.play();
                            }
                        })
                        .child(if is_playing {
                            Icons::Pause
                        } else {
                            Icons::Play
                        }),
                )
                .child(
                    div()
                        .id("mini_seek_forward")
                        .h_8()
                        .w_8()
                        .rounded_md()
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_color(theme.titlebar_window_icons_text)
                        .text_xs()
                        .cursor_pointer()
                        .hover(|this| {
                            this.bg(theme.titlebar_window_icons_bg_hover)
                        })
                        .on_mouse_down(MouseButton::Left, |_, window, cx| {
                            window.prevent_default();
                            cx.stop_propagation();
                        })
                        .on_click({
                            let controller = controller.clone();
                            move |_, _, cx| {
                                let state = controller.state.read(cx);
                                let pos = state.playback.position;
                                let track = state.playback.current.and_then(|id| {
                                    state.library.tracks.get(&id)
                                });
                                let max_dur = track.map_or(Duration::ZERO, |t| t.duration);
                                let new_pos = (pos + Duration::from_secs(5)).min(max_dur);
                                controller.seek(new_pos);
                            }
                        })
                        .child(Icons::Next),
                ),
        )
}

impl Titlebar {
    pub fn new(cx: &mut App) -> Titlebar {
        let navbar = cx.new(|_| NavBar::new());
        Titlebar { navbar }
    }
}
