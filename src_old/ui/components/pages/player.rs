use crate::ui::theme::Theme;

use crate::audio::engine::PlaybackState;
use crate::controller::player::Controller;
use crate::ui::components::controlbar::ControlBar;
use crate::ui::components::queue::Queue;
use crate::ui::components::scrollbar::{RightPad, floating_scrollbar};
use crate::ui::icons::Icons;
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::Icon;

#[derive(Clone)]
pub struct PlayerPage {
    pub queue: Entity<Queue>,
    queue_scroll_handle: UniformListScrollHandle,
    pub controlbar: Entity<ControlBar>,
    show_queue: Entity<bool>,
}

impl PlayerPage {
    pub fn new(cx: &mut App, controlbar: Entity<ControlBar>) -> Self {
        let queue_scroll_handle = UniformListScrollHandle::new();
        let show_queue = cx.new(|_| true);
        PlayerPage {
            queue: Queue::new(cx, queue_scroll_handle.clone()),
            queue_scroll_handle,
            controlbar,
            show_queue,
        }
    }
}

impl Render for PlayerPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>();

        let player_state = cx.global::<Controller>().player_state.clone();
        let thumbnail = player_state.thumbnail;
        // let scanner_state = cx.global::<Controller>().scanner_state.clone();
        let scroll_handle = self.queue_scroll_handle.clone();
        let show_queue = self.show_queue.clone();

        div()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .child(
                div()
                    .h_full()
                    .w_full()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .px_16()
                    .pt_8()
                    .pb_2()
                    .child(if let Some(meta) = player_state.meta {
                        div()
                            .w_auto()
                            .h_auto()
                            .flex()
                            .flex_col()
                            .items_center()
                            .justify_center()
                            .gap_y_6()
                            .flex_shrink_0()
                            .flex_1()
                            .child(if let Some(thumbnail) = thumbnail {
                                div().flex().flex_1().child(
                                    img(thumbnail)
                                        .object_fit(ObjectFit::Contain)
                                        .size_full()
                                        .rounded_xl(),
                                )
                            } else {
                                div().size_full()
                            })
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_y_neg_1()
                                    .items_center()
                                    .justify_center()
                                    .child(
                                        div()
                                            .text_2xl()
                                            .text_color(theme.text_primary)
                                            .font_weight(FontWeight(500.0))
                                            .max_w_96()
                                            .truncate()
                                            .child(meta.title.clone()),
                                    )
                                    .child(
                                        div()
                                            .text_base()
                                            .text_color(theme.text_muted)
                                            .font_weight(FontWeight(400.0))
                                            .max_w_96()
                                            .truncate()
                                            .child(meta.artists.join(", ").clone()),
                                    ),
                            )
                    } else {
                        div()
                    })
                    .child(
                        div()
                            .w_full()
                            .h_auto()
                            .flex()
                            .flex_shrink_0()
                            .gap_x_6()
                            .items_center()
                            .justify_center()
                            .mt_6()
                            .child(
                                div()
                                    .id("shuffle")
                                    .p_4()
                                    .rounded_md()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .when(
                                        cx.global::<Controller>().player_state.shuffling,
                                        |this| this.text_color(theme.accent),
                                    )
                                    .hover(|this| this.bg(theme.white_05))
                                    .on_click(|_, _, cx| cx.global::<Controller>().set_shuffle())
                                    .child(Icon::new(Icons::Shuffle).size_4()),
                            )
                            .child(
                                div()
                                    .id("previous")
                                    .p_4()
                                    .rounded_md()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .hover(|this| this.bg(theme.white_05))
                                    .on_click(|_, _, cx| cx.global::<Controller>().prev())
                                    .child(Icon::new(Icons::Prev).size_4()),
                            )
                            .child(
                                div()
                                    .id("play_pause")
                                    .p_5()
                                    .rounded_full()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .bg(theme.accent)
                                    .hover(|this| this.bg(theme.accent_30))
                                    .on_click(|_, _, cx| {
                                        match cx.global::<Controller>().player_state.state {
                                            PlaybackState::Paused | PlaybackState::Stopped => {
                                                cx.global::<Controller>().play()
                                            }
                                            PlaybackState::Playing => {
                                                cx.global::<Controller>().pause()
                                            }
                                        }
                                    })
                                    .child(
                                        if cx.global::<Controller>().player_state.state
                                            == PlaybackState::Playing
                                        {
                                            Icon::new(Icons::Pause).size_5()
                                        } else {
                                            Icon::new(Icons::Play).size_5()
                                        },
                                    ),
                            )
                            .child(
                                div()
                                    .id("next")
                                    .p_4()
                                    .rounded_md()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .hover(|this| this.bg(theme.white_05))
                                    .on_click(|_, _, cx| cx.global::<Controller>().next())
                                    .child(Icon::new(Icons::Next).size_4()),
                            )
                            .child(
                                div()
                                    .id("repeat")
                                    .p_4()
                                    .rounded_md()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .hover(|this| this.bg(theme.white_05))
                                    .when(cx.global::<Controller>().player_state.repeat, |this| {
                                        this.text_color(theme.accent)
                                    })
                                    .on_click(|_, _, cx| cx.global::<Controller>().set_repeat())
                                    .child(Icon::new(Icons::Repeat).size_4()),
                            ),
                    )
                    .child(self.controlbar.clone()),
            )
            .child(div().w(px(1.0)).h_full().bg(theme.white_05))
            .child(if *show_queue.read(cx) {
                div()
                    .h_full()
                    .w_80()
                    .flex_shrink_0()
                    .flex()
                    .flex_col()
                    .bg(theme.bg_queue)
                    .child(
                        div()
                            .w_full()
                            .flex()
                            .items_center()
                            .justify_start()
                            .p_4()
                            .child(
                                div()
                                    .text_base()
                                    .text_color(theme.text_primary)
                                    .font_weight(FontWeight(500.0))
                                    .child("Queue"),
                            ),
                    )
                    .child(
                        div()
                            .id("queue_container")
                            .w_full()
                            .h_full()
                            .px_4()
                            .pb_4()
                            .flex()
                            .relative()
                            .child(self.queue.clone())
                            .child(floating_scrollbar(
                                "queue_scrollbar",
                                scroll_handle,
                                RightPad::None,
                            )),
                    )
            } else {
                div()
            })
            .child(
                div()
                    .id("show_hide_queue")
                    .px_3()
                    .py_1()
                    .absolute()
                    .top_4()
                    .right_3()
                    .text_center()
                    .rounded_md()
                    .text_sm()
                    .font_weight(FontWeight(400.0))
                    .text_color(theme.text_muted)
                    .hover(|this| this.bg(theme.white_05).text_color(theme.text_primary))
                    .on_click(move |_, _, cx| show_queue.update(cx, |this, _| *this = !*this))
                    .child(if *self.show_queue.read(cx) {
                        "Hide"
                    } else {
                        "Show Queue"
                    }),
            )
    }
}

pub fn get_img_format(format: String) -> ImageFormat {
    match format.as_str() {
        "png" => ImageFormat::Png,
        "jpeg" | "jpg" => ImageFormat::Jpeg,
        _ => ImageFormat::Bmp,
    }
}
