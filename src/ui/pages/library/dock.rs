use crate::{
    controller::{Controller, state::PlaybackStatus},
    ui::{
        animations::{page_transition, sidebar_transition},
        components::{
            icons::{Icons, icon},
            image_cache::ImageCache,
            slider::{Slider, SliderState},
        },
        theme::Theme,
    },
};
use gpui::{
    Context, Entity, FontWeight, Hsla, InteractiveElement, IntoElement, ObjectFit, ParentElement,
    Render, StatefulInteractiveElement, Styled, StyledImage, Window, div, img, point, px,
};
use wgpui_motion::prelude::MotionExt;

#[derive(Clone)]
pub struct Dock {
    playback_slider_state: Entity<SliderState>,
    vol_slider_state: Entity<SliderState>,
    volume_hovered: (bool, bool),
}

impl Dock {
    #[must_use]
    pub fn new(
        playback_slider_state: Entity<SliderState>,
        vol_slider_state: Entity<SliderState>,
    ) -> Self {
        Self {
            playback_slider_state,
            vol_slider_state,
            volume_hovered: (false, false),
        }
    }
}

impl Render for Dock {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let controller = cx.global::<Controller>().clone();
        let thumbnail = cx.global::<ImageCache>().current.clone();
        let (
            is_playing,
            total_duration,
            elapsed,
            seek_percentage,
            volume_percentage,
            volume_icon,
            current_id,
            title,
            artist,
        ) = {
            let state = controller.state.read(cx);
            let current = state
                .playback
                .current
                .and_then(|id| state.library.tracks.get(&id));
            let total_duration = current.map(|track| track.duration).unwrap_or_default();
            let elapsed = state.playback.position.min(total_duration);
            let seek_percentage = if total_duration.as_secs_f32() > 0.0 {
                elapsed.as_secs_f32() / total_duration.as_secs_f32() * 100.0
            } else {
                0.0
            };
            let volume_percentage = state.playback.volume.clamp(0.0, 1.0) * 100.0;
            let volume_icon = if state.playback.mute {
                Icons::VolumeMute
            } else {
                match state.playback.volume.clamp(0.0, 1.0) {
                    0.0 => Icons::Volume0,
                    v if v < 0.4 => Icons::Volume1,
                    _ => Icons::Volume2,
                }
            };
            let title = current
                .map(|track| track.title.to_string())
                .unwrap_or_else(|| "No track selected".to_string());
            let artist = current
                .map(|track| {
                    track
                        .artists(&state.library)
                        .map(|artist| artist.name.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                })
                .filter(|artist| !artist.is_empty())
                .unwrap_or_else(|| "Browse your library to find music".to_string());

            (
                state.playback.status == PlaybackStatus::Playing,
                total_duration,
                elapsed,
                seek_percentage,
                volume_percentage,
                volume_icon,
                state.playback.current,
                title,
                artist,
            )
        };
        let is_favorite = current_id.is_some_and(|id| controller.is_favorite(id, cx));

        self.playback_slider_state.update(cx, |slider, cx| {
            if (slider.value() - seek_percentage).abs() > 0.5 {
                slider.set_value(seek_percentage, cx);
            }
        });
        self.vol_slider_state.update(cx, |slider, cx| {
            if (slider.value() - volume_percentage).abs() > 0.5 {
                slider.set_value(volume_percentage, cx);
            }
        });

        let track_art = match thumbnail {
            Some(image) => img(image)
                .size_full()
                .object_fit(ObjectFit::Cover)
                .rounded_md()
                .border_1()
                .border_color(theme.dock_border)
                .into_any_element(),
            None => img("icons/placeholder.svg")
                .size_full()
                .object_fit(ObjectFit::Contain)
                .rounded_md()
                .border_1()
                .border_color(theme.dock_border)
                .into_any_element(),
        };
        let is_compact = window.bounds().size.width < px(900.0);
        let is_narrow = window.bounds().size.width < px(1200.0);
        let metadata_max_width = if is_compact { px(240.0) } else { px(280.0) };
        let show_volume_slider = is_narrow && (self.volume_hovered.0 || self.volume_hovered.1);
        let dock = cx.entity();
        let sec_to_text = |duration: std::time::Duration| {
            let total = duration.as_secs();
            format!("{:02}:{:02}", total / 60, total % 60)
        };
        let volume_toggle = div()
            .id("library_dock_volume_toggle")
            .size(px(32.0))
            .flex_shrink_0()
            .flex()
            .items_center()
            .justify_center()
            .rounded_md()
            .cursor_pointer()
            .text_color(theme.dock_icon)
            .on_click({
                let controller = controller.clone();
                move |_, _, cx| controller.set_mute(cx)
            })
            .motion("library_dock_volume")
            .transition(sidebar_transition())
            .while_hovered(move |style| {
                style
                    .text_color(theme.dock_icon_hover)
                    .bg(theme.dock_icon_bg_hover)
            })
            .child(icon(volume_icon).size_4());
        let volume_control = if is_narrow {
            let mut control = div()
                .id("library_dock_volume")
                .w(px(32.0))
                .flex_shrink_0()
                .relative()
                .flex()
                .items_center()
                .justify_center()
                .child(volume_toggle);
            if show_volume_slider {
                control = control.child(
                    div()
                        .id("library_dock_volume_popout")
                        .absolute()
                        .right(px(32.0))
                        .top(px(-8.0))
                        .w(px(116.0))
                        .h(px(48.0))
                        .flex()
                        .items_center()
                        .child(
                            Slider::new(
                                &self.vol_slider_state,
                                "library_dock_volume_popout_slider",
                                4.0,
                            )
                            .bg(theme.dock_volume_track)
                            .text_color(theme.dock_volume_fill),
                        )
                        .on_hover({
                            let dock = dock.clone();
                            move |hovered, _, cx| {
                                dock.update(cx, |dock, cx| {
                                    let popout_hovered = &mut dock.volume_hovered.1;
                                    if *popout_hovered != *hovered {
                                        *popout_hovered = *hovered;
                                        cx.notify();
                                    }
                                });
                            }
                        })
                        .motion("library_dock_volume_popout")
                        .transition(sidebar_transition()),
                );
            }
            control
                .on_hover({
                    let dock = dock.clone();
                    move |hovered, _, cx| {
                        dock.update(cx, |dock, cx| {
                            if dock.volume_hovered.0 != *hovered {
                                dock.volume_hovered.0 = *hovered;
                                cx.notify();
                            }
                        });
                    }
                })
                .into_any_element()
        } else {
            div()
                .w(px(140.0))
                .flex_shrink_0()
                .flex()
                .items_center()
                .gap_2()
                .child(volume_toggle)
                .child(
                    div().flex_1().min_w_0().child(
                        Slider::new(&self.vol_slider_state, "library_dock_volume", 4.0)
                            .bg(theme.dock_volume_track)
                            .text_color(theme.dock_volume_fill),
                    ),
                )
                .into_any_element()
        };

        div()
            .w_4_5()
            .max_w(px(1200.0))
            .min_w_0()
            .rounded_2xl()
            .border_1()
            .border_color(theme.dock_border)
            .bg(theme.dock_surface)
            .shadow(vec![gpui::BoxShadow {
                color: Hsla {
                    h: 0.0,
                    s: 0.0,
                    l: 0.0,
                    a: 0.38,
                },
                blur_radius: px(22.0),
                spread_radius: px(0.0),
                offset: point(px(0.0), px(8.0)),
            }])
            .motion("library_dock")
            .transition(page_transition())
            .child(
                div()
                    .w_full()
                    .flex()
                    .items_center()
                    .gap_4()
                    .px_4()
                    .py_3()
                    .child(
                        div()
                            .w_auto()
                            .max_w(metadata_max_width)
                            .min_w_0()
                            .flex_shrink_0()
                            .flex()
                            .items_center()
                            .gap_3()
                            .child(
                                div()
                                    .size(if is_compact { px(48.0) } else { px(56.0) })
                                    .flex_shrink_0()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .rounded_md()
                                    .overflow_hidden()
                                    .child(track_art),
                            )
                            .child(
                                div()
                                    .max_w(if is_compact { px(136.0) } else { px(168.0) })
                                    .min_w_0()
                                    .flex_shrink_0()
                                    .flex()
                                    .flex_col()
                                    .gap_0()
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(FontWeight(500.0))
                                            .text_color(theme.dock_text_primary)
                                            .truncate()
                                            .child(title),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(theme.dock_text_secondary)
                                            .truncate()
                                            .child(artist),
                                    ),
                            )
                            .child(
                                div()
                                    .id("library_dock_favorite")
                                    .size(px(32.0))
                                    .flex_shrink_0()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .rounded_md()
                                    .cursor_pointer()
                                    .text_color(if is_favorite {
                                        theme.dock_icon_active
                                    } else {
                                        theme.dock_icon
                                    })
                                    .bg(if is_favorite {
                                        theme.dock_icon_bg_active
                                    } else {
                                        theme.dock_icon_bg
                                    })
                                    .on_click({
                                        let controller = controller.clone();
                                        move |_, _, cx| {
                                            if let Some(id) = current_id {
                                                controller.toggle_favorite(id, cx);
                                            }
                                        }
                                    })
                                    .motion("library_dock_favorite")
                                    .transition(sidebar_transition())
                                    .variant_when(is_favorite, move |style| {
                                        style
                                            .text_color(theme.dock_icon_active)
                                            .bg(theme.dock_icon_bg_active)
                                    })
                                    .while_hovered(move |style| {
                                        style
                                            .text_color(theme.dock_icon_hover)
                                            .bg(theme.dock_icon_bg_hover)
                                    })
                                    .while_pressed(|style| style.opacity(0.8))
                                    .child(icon(Icons::Heart).size_4()),
                            ),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .flex()
                            .flex_col()
                            .justify_center()
                            .items_center()
                            .gap_2()
                            .child(
                                div().w_full().child(
                                    Slider::new(
                                        &self.playback_slider_state,
                                        "library_dock_seek",
                                        4.0,
                                    )
                                    .bg(theme.dock_seek_track)
                                    .text_color(theme.dock_seek_fill),
                                ),
                            )
                            .child(
                                div()
                                    .w_full()
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .child(
                                        div()
                                            .flex_shrink_0()
                                            .font_family("JetBrains Mono")
                                            .text_xs()
                                            .text_color(theme.dock_text_secondary)
                                            .child(sec_to_text(elapsed)),
                                    )
                                    .child(
                                        div()
                                            .flex_shrink_0()
                                            .font_family("JetBrains Mono")
                                            .text_xs()
                                            .text_color(theme.dock_text_secondary)
                                            .child(sec_to_text(total_duration)),
                                    ),
                            ),
                    )
                    .child(
                        div()
                            .flex_shrink_0()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .id("library_dock_prev")
                                    .size(px(32.0))
                                    .flex_shrink_0()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .rounded_md()
                                    .cursor_pointer()
                                    .text_color(theme.dock_icon)
                                    .on_click(|_, _, cx| cx.global::<Controller>().clone().prev(cx))
                                    .motion("library_dock_prev")
                                    .transition(sidebar_transition())
                                    .while_hovered(move |style| {
                                        style
                                            .text_color(theme.dock_icon_hover)
                                            .bg(theme.dock_icon_bg_hover)
                                    })
                                    .while_pressed(|style| style.opacity(0.8))
                                    .child(icon(Icons::Prev).size_4()),
                            )
                            .child(
                                div()
                                    .id("library_dock_play_pause")
                                    .size(px(44.0))
                                    .flex_shrink_0()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .rounded_full()
                                    .cursor_pointer()
                                    .bg(theme.dock_play_pause_bg)
                                    .text_color(theme.dock_play_pause_text)
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
                                    .motion("library_dock_play_pause")
                                    .transition(sidebar_transition())
                                    .while_hovered(move |style| {
                                        style.bg(theme.dock_play_pause_bg_hover)
                                    })
                                    .while_pressed(|style| style.opacity(0.9))
                                    .child(if is_playing {
                                        icon(Icons::Pause).size_5()
                                    } else {
                                        icon(Icons::Play).size_5()
                                    }),
                            )
                            .child(
                                div()
                                    .id("library_dock_next")
                                    .size(px(32.0))
                                    .flex_shrink_0()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .rounded_md()
                                    .cursor_pointer()
                                    .text_color(theme.dock_icon)
                                    .on_click(|_, _, cx| cx.global::<Controller>().clone().next(cx))
                                    .motion("library_dock_next")
                                    .transition(sidebar_transition())
                                    .while_hovered(move |style| {
                                        style
                                            .text_color(theme.dock_icon_hover)
                                            .bg(theme.dock_icon_bg_hover)
                                    })
                                    .while_pressed(|style| style.opacity(0.8))
                                    .child(icon(Icons::Next).size_4()),
                            ),
                    )
                    .child(volume_control),
            )
    }
}
