use crate::controller::Controller;
use crate::ui::theme::Theme;

use super::slider::{Slider, SliderState};
use crate::ui::components::icons::{Icon, Icons};
use gpui::{
    Context, Entity, InteractiveElement, IntoElement, ParentElement, Render,
    StatefulInteractiveElement, Styled, Window, div,
};

#[derive(Clone)]
pub struct ControlBar {
    pub playback_slider_state: Entity<SliderState>,
    pub vol_slider_state: Entity<SliderState>,
}

impl ControlBar {
    #[must_use]
    pub fn new(
        playback_slider_state: Entity<SliderState>,
        vol_slider_state: Entity<SliderState>,
    ) -> Self {
        ControlBar {
            playback_slider_state,
            vol_slider_state,
        }
    }
}

impl Render for ControlBar {
    #[allow(clippy::too_many_lines)]
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>();
        let controller = cx.global::<Controller>();
        let state = controller.state.read(cx);

        let current = if let Some(id) = state.playback.current {
            state.library.tracks.get(&id)
        } else {
            None
        };

        let duration = if let Some(track) = current {
            track.duration.as_secs()
        } else {
            0
        };

        div()
            .w_full()
            .h_auto()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap_y_5()
            .child(
                div()
                    .w_2_3()
                    .h_auto()
                    .p_4()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .child(
                        Slider::new(&self.playback_slider_state.clone(), "playback_slider", 6.0)
                            .text_color(theme.playback_slider_fill)
                            .bg(theme.playback_slider_track),
                    )
                    .child(
                        div()
                            .w_full()
                            .h_auto()
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .flex()
                                    .flex_shrink_0()
                                    .font_family("JetBrains Mono")
                                    .text_sm()
                                    .text_color(theme.playback_position_text)
                                    .child(format!(
                                        "{:02}:{:02}",
                                        state.playback.position / 60,
                                        state.playback.position % 60
                                    )),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_shrink_0()
                                    .font_family("JetBrains Mono")
                                    .text_sm()
                                    .text_color(theme.playback_position_text)
                                    .child(format!("{:02}:{:02}", duration / 60, duration % 60)),
                            ),
                    )
                    .child(
                        div()
                            .w_full()
                            .h_auto()
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .w_auto()
                                    .h_auto()
                                    .flex()
                                    .flex_shrink_0()
                                    .items_center()
                                    .justify_start()
                                    .gap_x_3()
                                    .pt_2()
                                    .child(
                                        div()
                                            .id("volume_icon")
                                            .on_click({
                                                let controller = controller.clone();
                                                move |_, _, cx| controller.set_mute(cx)
                                            })
                                            .text_color(theme.volume_icon)
                                            .child(
                                                Icon::new(if state.playback.mute {
                                                    Icons::VolumeMute
                                                } else {
                                                    match state.playback.volume.clamp(0.0, 1.0) {
                                                        0.0 => Icons::Volume0,
                                                        v if v < 0.4 => Icons::Volume0,
                                                        v if v < 0.8 => Icons::Volume1,
                                                        _ => Icons::Volume2,
                                                    }
                                                })
                                                .size_4(),
                                            ),
                                    )
                                    .child(
                                        div().w_40().flex().flex_shrink_0().child(
                                            Slider::new(
                                                &self.vol_slider_state,
                                                "volume_slider",
                                                4.0,
                                            )
                                            .bg(theme.volume_slider_track)
                                            .text_color(theme.volume_slider_fill),
                                        ),
                                    ),
                            ),
                    ),
            )
    }
}
