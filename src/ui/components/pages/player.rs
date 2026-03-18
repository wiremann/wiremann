use crate::{
    controller::state::PlaybackStatus,
    controller::Controller,
    ui::{
        components::controlbar::ControlBar,
        components::icons::{Icon, Icons},
        components::image_cache::ImageCache,
        components::queue::Queue,
        components::scrollbar::{floating_scrollbar, RightPad},
        theme::Theme,
    },
};
use gpui::prelude::FluentBuilder;
use gpui::{div, img, px, App, AppContext, Context, Entity, FontWeight, InteractiveElement, IntoElement, ObjectFit, ParentElement, Render, StatefulInteractiveElement, Styled, StyledImage, UniformListScrollHandle, Window};

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
    #[allow(clippy::too_many_lines)]
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>();

        let controller = cx.global::<Controller>().clone();
        let state = controller.state.read(cx);
        let thumbnail = cx.global::<ImageCache>().current.clone();
        // let scanner_state = cx.global::<Controller>().scanner_state.clone();
        let scroll_handle = self.queue_scroll_handle.clone();
        let show_queue = self.show_queue.clone();

        let current = if let Some(id) = state.playback.current {
            state.library.tracks.get(&id)
        } else {
            None
        };

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
                    .child(if let Some(track) = current {
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
                                            .child(track.title.clone()),
                                    )
                                    .child(
                                        div()
                                            .text_base()
                                            .text_color(theme.text_muted)
                                            .font_weight(FontWeight(400.0))
                                            .max_w_96()
                                            .truncate()
                                            .child(track.artist.clone()),
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
                                    .text_color(theme.text_primary)
                                    .when(
                                        cx.global::<Controller>().state.read(cx).playback.shuffling,
                                        |this| this.text_color(theme.accent),
                                    )
                                    .hover(|this| this.bg(theme.white_05))
                                    .on_click({
                                        let controller = controller.clone();
                                        move |_, _, cx| controller.set_shuffle(cx)
                                    })
                                    .cursor_pointer()
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
                                    .on_click(|_, _, cx| cx.global::<Controller>().clone().prev(cx))
                                    .text_color(theme.text_primary)
                                    .cursor_pointer()
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
                                        match cx
                                            .global::<Controller>()
                                            .state
                                            .read(cx)
                                            .playback
                                            .status
                                        {
                                            PlaybackStatus::Paused | PlaybackStatus::Stopped => {
                                                cx.global::<Controller>().play();
                                            }
                                            PlaybackStatus::Playing => {
                                                cx.global::<Controller>().pause();
                                            }
                                        }
                                    })
                                    .text_color(theme.text_primary)
                                    .cursor_pointer()
                                    .child(
                                        if cx.global::<Controller>().state.read(cx).playback.status
                                            == PlaybackStatus::Playing
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
                                    .on_click(|_, _, cx| cx.global::<Controller>().clone().next(cx))
                                    .cursor_pointer()
                                    .text_color(theme.text_primary)
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
                                    .text_color(theme.text_primary)
                                    .when(
                                        cx.global::<Controller>().state.read(cx).playback.repeat,
                                        |this| this.text_color(theme.accent),
                                    )
                                    .on_click({
                                        let controller = controller.clone();
                                        move |_, _, cx| controller.set_repeat(cx)
                                    })
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
                                RightPad::Pad,
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
                    .cursor_pointer()
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
