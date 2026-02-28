use crate::controller::player::Controller;
use crate::ui::theme::Theme;

use super::slider::{Slider, SliderState};
use crate::ui::icons::Icons;
// use crate::ui::icons::Icons;
use gpui::*;
use gpui_component::Icon;
// use gpui_component::Icon;

#[derive(Clone)]
pub struct ControlBar {
    pub playback_slider_state: Entity<SliderState>,
    pub vol_slider_state: Entity<SliderState>,
}

impl ControlBar {
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
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>();
        let state = cx.global::<Controller>().player_state.clone();

        let duration = state.meta.as_ref().map(|m| m.duration).unwrap_or(0);

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
                        Slider::new(&self.playback_slider_state.clone()).text_color(theme.accent),
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
                                    .text_color(theme.text_muted)
                                    .child(format!(
                                        "{:02}:{:02}",
                                        state.position / 60,
                                        state.position % 60
                                    )),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_shrink_0()
                                    .font_family("JetBrains Mono")
                                    .text_sm()
                                    .text_color(theme.text_muted)
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
                                            .on_click(|_, _, cx| cx.global::<Controller>().mute())
                                            .child(
                                                Icon::new(if state.mute {
                                                    Icons::VolumeMute
                                                } else {
                                                    match state.volume.clamp(0.0, 1.0) {
                                                        v if v == 0.0 => Icons::Volume0,
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
                                            Slider::new(&self.vol_slider_state)
                                                .text_color(theme.accent),
                                        ),
                                    ),
                            ),
                    ),
            )
    }
}
