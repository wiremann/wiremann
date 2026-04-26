use crate::{
    controller::Controller,
    controller::state::PlaybackStatus,
    ui::{
        components::controlbar::ControlBar,
        components::icons::{Icon, Icons},
        components::image_cache::ImageCache,
        components::queue::Queue,
        components::scrollbar::{RightPad, floating_scrollbar},
        theme::Theme,
    },
};
use gpui::prelude::FluentBuilder;
use gpui::{
    App, AppContext, Context, Entity, FontWeight, InteractiveElement, IntoElement, ObjectFit,
    ParentElement, Render, StatefulInteractiveElement, Styled, StyledImage,
    UniformListScrollHandle, Window, div, img, px,
};

#[derive(Clone)]
pub struct PlayerPage {
    pub queue: Entity<Queue>,
    queue_scroll_handle: UniformListScrollHandle,
    pub controlbar: Entity<ControlBar>,
    show_panel: Entity<bool>,
    current_panel: Entity<Panel>,
}

enum Panel {
    Lyrics,
    Queue,
}

impl PlayerPage {
    pub fn new(cx: &mut App, controlbar: Entity<ControlBar>) -> Self {
        let queue_scroll_handle = UniformListScrollHandle::new();
        let show_panel = cx.new(|_| true);
        let current_panel = cx.new(|_| Panel::Queue);

        PlayerPage {
            queue: Queue::new(cx, queue_scroll_handle.clone()),
            queue_scroll_handle,
            controlbar,
            show_panel,
            current_panel,
        }
    }
}

impl Render for PlayerPage {
    #[allow(clippy::too_many_lines)]
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();

        let controller = cx.global::<Controller>().clone();
        let state = controller.state.read(cx);
        let thumbnail = cx.global::<ImageCache>().current.clone();
        let scroll_handle = self.queue_scroll_handle.clone();
        let show_panel = self.show_panel.clone();

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
                    .bg(theme.player_bg)
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
                                        .rounded_xl()
                                        .border_2()
                                        .border_color(theme.border),
                                )
                            } else {
                                div().flex().flex_1().child(
                                    img("icons/placeholder.svg")
                                        .object_fit(ObjectFit::Contain)
                                        .size_full()
                                        .rounded_xl()
                                        .border_2()
                                        .border_color(theme.border),
                                )
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
                                            .text_color(theme.player_title_text)
                                            .font_weight(FontWeight(500.0))
                                            .max_w_96()
                                            .truncate()
                                            .child(track.title.clone()),
                                    )
                                    .child(
                                        div()
                                            .text_base()
                                            .text_color(theme.player_artist_text)
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
                                    .text_color(theme.player_icons_text)
                                    .when(
                                        cx.global::<Controller>().state.read(cx).playback.shuffling,
                                        |this| {
                                            this.text_color(theme.player_icons_text_active)
                                                .bg(theme.player_icons_bg_active)
                                        },
                                    )
                                    .hover(|this| {
                                        this.bg(theme.player_icons_bg_hover)
                                            .text_color(theme.player_icons_text_hover)
                                    })
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
                                    .hover(|this| {
                                        this.bg(theme.player_icons_bg_hover)
                                            .text_color(theme.player_icons_text_hover)
                                    })
                                    .on_click(|_, _, cx| cx.global::<Controller>().clone().prev(cx))
                                    .text_color(theme.player_icons_text)
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
                                    .bg(theme.player_play_pause_bg)
                                    .hover(|this| this.bg(theme.player_play_pause_hover))
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
                                    .text_color(theme.player_play_pause_text)
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
                                    .hover(|this| {
                                        this.bg(theme.player_icons_bg_hover)
                                            .text_color(theme.player_icons_text_hover)
                                    })
                                    .on_click(|_, _, cx| cx.global::<Controller>().clone().next(cx))
                                    .cursor_pointer()
                                    .text_color(theme.player_icons_text)
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
                                    .cursor_pointer()
                                    .hover(|this| {
                                        this.bg(theme.player_icons_bg_hover)
                                            .text_color(theme.player_icons_text_hover)
                                    })
                                    .text_color(theme.player_icons_text)
                                    .when(
                                        cx.global::<Controller>().state.read(cx).playback.repeat,
                                        |this| {
                                            this.text_color(theme.player_icons_text_active)
                                                .bg(theme.player_icons_bg_active)
                                        },
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
            .child(div().w(px(1.0)).h_full().bg(theme.border))
            .child(if *show_panel.read(cx) {
                div()
                    .h_full()
                    .w_1_4()
                    .flex_shrink_0()
                    .flex()
                    .flex_col()
                    .bg(theme.player_panel_bg)
                    .border_l_1()
                    .border_color(theme.border)
                    .child(
                        div()
                            .w_full()
                            .h_12()
                            .flex()
                            .items_center()
                            .justify_start()
                            .child(
                                div()
                                    .w_full()
                                    .h_full()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .text_base()
                                    .text_color(theme.player_panel_heading_text)
                                    .font_weight(FontWeight(500.0))
                                    .child("Queue"),
                            )
                            .child(
                                div()
                                    .w_full()
                                    .h_full()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .text_base()
                                    .text_color(theme.player_panel_heading_text)
                                    .font_weight(FontWeight(500.0))
                                    .child("Lyrics"),
                            ),
                    )
                    .child(
                        div()
                            .id("queue_container")
                            .w_full()
                            .h_full()
                            .px_4()
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
                    .text_color(theme.player_panel_show_hide_text)
                    .cursor_pointer()
                    .hover(|this| {
                        this.bg(theme.player_panel_show_hide_bg_hover)
                            .text_color(theme.player_panel_show_hide_text_hover)
                    })
                    .on_click(move |_, _, cx| show_panel.update(cx, |this, _| *this = !*this))
                    .child(if *self.show_panel.read(cx) {
                        "Hide"
                    } else {
                        "Show Queue"
                    }),
            )
    }
}
