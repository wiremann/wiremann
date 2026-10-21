pub mod controlbar;
pub mod lyrics;
pub mod queue;

use crate::{
    controller::{Controller, state::PlaybackStatus},
    ui::{
        animations::page_transition,
        components::{
            bounds_observer::observe_bounds,
            icons::{Icons, icon},
            image_cache::ImageCache,
            resize_handle::{ResizeHandle, ResizeSide, ResizeState},
            scrollbar::{RightPad, floating_scrollbar},
        },
        theme::{DominantColors, Theme},
    },
};
use controlbar::ControlBar;
use lyrics::LyricsView;
use queue::Queue;

use gpui::{
    App, AppContext, Bounds, Context, ElementId, Entity, FontWeight, InteractiveElement,
    IntoElement, ObjectFit, ParentElement, Pixels, Render, ScrollHandle,
    StatefulInteractiveElement, Styled, StyledImage, UniformListScrollHandle, Window, div,
    gradient_color_stop, img, prelude::FluentBuilder, px, relative, rgba,
};
use wgpui_motion::prelude::MotionExt;

#[derive(Clone)]
pub struct PlayerPage {
    pub queue: Entity<Queue>,
    queue_scroll_handle: UniformListScrollHandle,
    pub lyrics: Entity<LyricsView>,
    lyrics_scroll_handle: UniformListScrollHandle,
    pub controlbar: Entity<ControlBar>,
    show_panel: Entity<bool>,
    current_panel: Entity<Panel>,
    queue_tab_hovered: Entity<bool>,
    lyrics_tab_hovered: Entity<bool>,
    panel_width: Entity<ResizeState>,
    album_bounds: Option<Bounds<Pixels>>,
}

#[derive(Clone, Copy, PartialEq)]
enum Panel {
    Lyrics,
    Queue,
}

#[derive(Clone, Copy)]
struct PanelTransitionState {
    settled: Panel,
    from: Panel,
    target: Panel,
    generation: u64,
}

impl PlayerPage {
    pub fn new(cx: &mut App, controlbar: Entity<ControlBar>) -> Self {
        let queue_scroll_handle = UniformListScrollHandle::new();
        let show_panel = cx.new(|_| true);
        let current_panel = cx.new(|_| Panel::Queue);
        let queue_tab_hovered = cx.new(|_| false);
        let lyrics_tab_hovered = cx.new(|_| false);
        let panel_width = cx.new(|_| ResizeState::new(ResizeSide::Right, 480.0, 360.0, 760.0));

        PlayerPage {
            queue: Queue::new(cx, queue_scroll_handle.clone()),
            queue_scroll_handle,
            lyrics: LyricsView::new(cx, ScrollHandle::new()),
            lyrics_scroll_handle: UniformListScrollHandle::new(),
            controlbar,
            show_panel,
            current_panel,
            queue_tab_hovered,
            lyrics_tab_hovered,
            panel_width,
            album_bounds: None,
        }
    }
}

impl Render for PlayerPage {
    #[allow(clippy::too_many_lines)]
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let dominant_colors = *cx.global::<DominantColors>();

        let controller = cx.global::<Controller>().clone();
        let selected_panel = *self.current_panel.read(cx);
        let panel_transition_state =
            window.use_keyed_state("player_panel_transition", cx, |_, _| PanelTransitionState {
                settled: selected_panel,
                from: selected_panel,
                target: selected_panel,
                generation: 0,
            });
        if panel_transition_state.read(cx).target != selected_panel {
            panel_transition_state.update(cx, |state, _| {
                state.from = state.target;
                state.target = selected_panel;
                state.generation = state.generation.wrapping_add(1);
            });
        }
        let panel_transition = *panel_transition_state.read(cx);
        let panel_is_transitioning = panel_transition.target != panel_transition.settled
            || panel_transition.from != panel_transition.target;

        let state = controller.state.read(cx);
        let thumbnail = cx.global::<ImageCache>().current.clone();
        let queue_scroll_handle = self.queue_scroll_handle.clone();
        let lyrics_scroll_handle = self.lyrics_scroll_handle.clone();
        let queue_tab_hovered = self.queue_tab_hovered.clone();
        let lyrics_tab_hovered = self.lyrics_tab_hovered.clone();
        let show_panel = self.show_panel.clone();

        let current = if let Some(id) = state.playback.current {
            state.library.tracks.get(&id)
        } else {
            None
        };

        let current_id = state.playback.current;
        let panel_width = self.panel_width.read(cx).width();

        let (current_artist_str, _current_album_str) = if let Some(track) = current {
            (
                track
                    .artists(&state.library)
                    .map(|a| a.name.to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
                track
                    .album(&state.library)
                    .map(|a| a.name.to_string())
                    .unwrap_or_default(),
            )
        } else {
            (String::new(), String::new())
        };

        let gradient_pos = self.album_bounds.map(|bounds| {
            let center_x = bounds.origin.x + bounds.size.width / 2.0;
            let center_y = bounds.origin.y + bounds.size.height / 2.0;

            (
                center_x / px(window.viewport_size().width.to_f64() as f32),
                center_y / px(window.viewport_size().height.to_f64() as f32),
            )
        });
        let (gx, gy) = gradient_pos.unwrap_or((0.5, 0.4));

        let is_portrait = window.bounds().size.width < px(1000.0);

        div()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .child(div().absolute().size_full().bg(gpui::radial_gradient(
                gx,
                gy - 0.03,
                0.72,
                0.58,
                gradient_color_stop(dominant_colors.color1, 0.0),
                gradient_color_stop(rgba(0x00000000), 1.0),
            )))
            .child(div().absolute().size_full().bg(gpui::radial_gradient(
                gx,
                gy - 0.03,
                1.9,
                1.4,
                gradient_color_stop(dominant_colors.color1.blend(dominant_colors.color2), 0.0),
                gradient_color_stop(rgba(0x00000000), 1.0),
            )))
            .when(is_portrait, gpui::Styled::flex_col)
            .child(
                // Main content: album art + title + controls + controlbar
                div()
                    .w_full()
                    .flex()
                    .flex_col()
                    .when(is_portrait, |this| this.h(relative(0.5)))
                    .when(!is_portrait, |this| this.flex_1().h_full())
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
                                observe_bounds(
                                    "album_bounds",
                                    div().flex().flex_1().child(
                                        img(thumbnail)
                                            .object_fit(ObjectFit::Cover)
                                            .size_full()
                                            .rounded_xl()
                                            .border_2()
                                            .border_color(theme.border),
                                    ),
                                    {
                                        let entity = cx.entity();

                                        move |bounds, _, cx| {
                                            entity.update(cx, |this, cx| {
                                                this.album_bounds = Some(bounds);
                                                cx.notify();
                                            });
                                        }
                                    },
                                )
                            } else {
                                observe_bounds(
                                    "album_placeholder_bounds",
                                    div().flex().flex_1().child(
                                        img("icons/placeholder.svg")
                                            .object_fit(ObjectFit::Contain)
                                            .size_full()
                                            .rounded_xl()
                                            .border_2()
                                            .border_color(theme.border),
                                    ),
                                    {
                                        let entity = cx.entity();

                                        move |bounds, _, cx| {
                                            entity.update(cx, |this, cx| {
                                                this.album_bounds = Some(bounds);
                                                cx.notify();
                                            });
                                        }
                                    },
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
                                            .child(track.title.to_string()),
                                    )
                                    .child(
                                        div()
                                            .text_base()
                                            .text_color(theme.player_artist_text)
                                            .font_weight(FontWeight(400.0))
                                            .max_w_96()
                                            .truncate()
                                            .child(current_artist_str),
                                    ),
                            )
                    } else {
                        div()
                            .w_auto()
                            .h_auto()
                            .flex()
                            .flex_col()
                            .items_center()
                            .justify_center()
                            .gap_y_4()
                            .flex_shrink_0()
                            .flex_1()
                            .child(
                                icon(Icons::Music)
                                    .size_16()
                                    .text_color(theme.player_artist_text),
                            )
                            .child(
                                div()
                                    .text_xl()
                                    .text_color(theme.player_artist_text)
                                    .child("No track selected"),
                            )
                            .child(
                                div()
                                    .text_base()
                                    .text_color(theme.player_artist_text)
                                    .child("Browse your library to find music"),
                            )
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
                                    .id("favorite")
                                    .p_4()
                                    .rounded_md()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .cursor_pointer()
                                    .text_color(theme.player_icons_text)
                                    .hover(|this| {
                                        this.bg(theme.player_icons_bg_hover)
                                            .text_color(theme.player_icons_text_hover)
                                    })
                                    .when(
                                        current_id.is_some_and(|id| controller.is_favorite(id, cx)),
                                        |this| {
                                            this.text_color(theme.player_icons_text_active)
                                                .bg(theme.player_icons_bg_active)
                                        },
                                    )
                                    .on_click({
                                        let controller = controller.clone();
                                        move |_, _, cx| {
                                            if let Some(id) = cx
                                                .global::<Controller>()
                                                .state
                                                .read(cx)
                                                .playback
                                                .current
                                            {
                                                controller.toggle_favorite(id, cx);
                                            }
                                        }
                                    })
                                    .child(icon(Icons::Heart).size_4()),
                            )
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
                                    .child(icon(Icons::Shuffle).size_4()),
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
                                    .child(icon(Icons::Prev).size_4()),
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
                                            icon(Icons::Pause).size_5()
                                        } else {
                                            icon(Icons::Play).size_5()
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
                                    .child(icon(Icons::Next).size_4()),
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
                                    .child(icon(Icons::Repeat).size_4()),
                            )
                            .child(
                                div()
                                    .id("reveal_in_folder")
                                    .p_4()
                                    .rounded_md()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .cursor_pointer()
                                    .text_color(theme.player_icons_text)
                                    .hover(|this| {
                                        this.bg(theme.player_icons_bg_hover)
                                            .text_color(theme.player_icons_text_hover)
                                    })
                                    .on_click(|_, _, cx| {
                                        if let Some(id) = cx
                                            .global::<Controller>()
                                            .state
                                            .read(cx)
                                            .playback
                                            .current
                                        {
                                            cx.global::<Controller>().reveal_in_folder(id, cx);
                                        }
                                    })
                                    .child(icon(Icons::FolderOpen).size_4()),
                            ),
                    )
                    .child(self.controlbar.clone()),
            )
            .when(!is_portrait && *show_panel.read(cx), |this| {
                this.child(
                    div()
                        .h_full()
                        .flex_shrink_0()
                        .bg(theme.player_panel_bg)
                        .child(ResizeHandle::new(&self.panel_width)),
                )
            })
            .when(*show_panel.read(cx), |this| {
                this.child(
                    div()
                        .h_full()
                        .when(is_portrait, |this| this.w_full().flex_1())
                        .when(!is_portrait, |this| this.w(px(panel_width)).h_full())
                        .flex_shrink_0()
                        .flex()
                        .flex_col()
                        .bg(theme.player_panel_bg)
                        .child({
                            let current_panel = self.current_panel.clone();

                            div()
                                .w_full()
                                .relative()
                                .flex()
                                .items_center()
                                .justify_center()
                                .py_3()
                                .child(
                                    div()
                                        .relative()
                                        .h_full()
                                        .flex()
                                        .items_center()
                                        .child({
                                            div()
                                                .id("panel_switcher_queue")
                                                .py_1()
                                                .px_5()
                                                .flex()
                                                .items_center()
                                                .justify_center()
                                                .cursor_pointer()
                                                .relative()
                                                .on_hover({
                                                    let queue_tab_hovered =
                                                        queue_tab_hovered.clone();
                                                    move |hovered, _, cx| {
                                                        queue_tab_hovered.update(
                                                            cx,
                                                            |value, cx| {
                                                                if *value != *hovered {
                                                                    *value = *hovered;
                                                                    cx.notify();
                                                                }
                                                            },
                                                        );
                                                    }
                                                })
                                                .on_click({
                                                    let current_panel = current_panel.clone();
                                                    move |_, _, cx| {
                                                        current_panel.update(cx, |p, _| {
                                                            *p = Panel::Queue;
                                                        });
                                                    }
                                                })
                                                .child(
                                                    div()
                                                        .w_full()
                                                        .h_full()
                                                        .bg(theme.player_panel_tab_bg_active)
                                                        .rounded_md()
                                                        .absolute()
                                                        .motion("player_queue_tab_highlight")
                                                        .transition(page_transition())
                                                        .base(|style| style.opacity(0.0))
                                                        .variant_when(
                                                            selected_panel == Panel::Queue,
                                                            |style| style.opacity(1.0),
                                                        )
                                                        .variant_when(
                                                            selected_panel != Panel::Queue
                                                                && *queue_tab_hovered.read(cx),
                                                            |style| style.opacity(0.5),
                                                        ),
                                                )
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .font_weight(FontWeight(500.0))
                                                        .tracking_widest()
                                                        .text_color(
                                                            if selected_panel == Panel::Queue {
                                                                theme.player_panel_tab_text_active
                                                            } else {
                                                                theme.player_panel_tab_text
                                                            },
                                                        )
                                                        .child("QUEUE"),
                                                )
                                        })
                                        .child({
                                            let current_panel = current_panel.clone();
                                            let lyrics_tab_hovered = lyrics_tab_hovered.clone();

                                            div()
                                                .id("panel_switcher_lyrics")
                                                .py_1()
                                                .px_5()
                                                .flex()
                                                .justify_center()
                                                .items_center()
                                                .cursor_pointer()
                                                .relative()
                                                .on_hover({
                                                    let hovered_state = lyrics_tab_hovered.clone();
                                                    move |hovered, _, cx| {
                                                        hovered_state.update(cx, |value, cx| {
                                                            if *value != *hovered {
                                                                *value = *hovered;
                                                                cx.notify();
                                                            }
                                                        });
                                                    }
                                                })
                                                .on_click({
                                                    let current_panel = current_panel.clone();
                                                    move |_, _, cx| {
                                                        current_panel.update(cx, |p, _| {
                                                            *p = Panel::Lyrics;
                                                        });
                                                    }
                                                })
                                                .child(
                                                    div()
                                                        .w_full()
                                                        .h_full()
                                                        .bg(theme.player_panel_tab_bg_active)
                                                        .rounded_md()
                                                        .absolute()
                                                        .motion("player_lyrics_tab_highlight")
                                                        .transition(page_transition())
                                                        .base(|style| style.opacity(0.0))
                                                        .variant_when(
                                                            selected_panel == Panel::Lyrics,
                                                            |style| style.opacity(1.0),
                                                        )
                                                        .variant_when(
                                                            selected_panel != Panel::Lyrics
                                                                && *lyrics_tab_hovered.read(cx),
                                                            |style| style.opacity(0.5),
                                                        ),
                                                )
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .font_weight(FontWeight(500.0))
                                                        .tracking_widest()
                                                        .text_color(
                                                            if selected_panel == Panel::Lyrics {
                                                                theme.player_panel_tab_text_active
                                                            } else {
                                                                theme.player_panel_tab_text
                                                            },
                                                        )
                                                        .child("LYRICS"),
                                                )
                                        }),
                                )
                        })
                        .child({
                            let panel_state = self.current_panel.clone();
                            let transition_state = panel_transition_state.clone();
                            let panel = if selected_panel == Panel::Queue {
                                div()
                                    .id("queue_container")
                                    .w_full()
                                    .h_full()
                                    .child(self.queue.clone())
                                    .child(floating_scrollbar(
                                        "queue_scrollbar",
                                        queue_scroll_handle,
                                        RightPad::Pad,
                                    ))
                            } else {
                                div()
                                    .id("lyrics_container")
                                    .w_full()
                                    .h_full()
                                    .px_8()
                                    .child(self.lyrics.clone())
                                    .child(floating_scrollbar(
                                        "lyrics_scrollbar",
                                        lyrics_scroll_handle,
                                        RightPad::Pad,
                                    ))
                            };

                            let panel = div()
                                .w_full()
                                .h_full()
                                .px_4()
                                .flex()
                                .relative()
                                .child(panel);
                            if panel_is_transitioning {
                                let generation = panel_transition.generation;
                                let offset = match selected_panel {
                                    Panel::Lyrics => 24.0,
                                    Panel::Queue => -24.0,
                                };
                                panel
                                    .motion(ElementId::NamedInteger(
                                        "player_panel".into(),
                                        generation,
                                    ))
                                    .transition(page_transition())
                                    .enter(move |style| style.left(px(offset)).opacity(0.0))
                                    .base(|style| style.left(px(0.0)).opacity(1.0))
                                    .on_settle(move |_, _, cx| {
                                        if *panel_state.read(cx) == selected_panel {
                                            transition_state.update(cx, |state, cx| {
                                                if state.generation == generation
                                                    && state.target == selected_panel
                                                {
                                                    state.settled = selected_panel;
                                                    state.from = selected_panel;
                                                    cx.notify();
                                                }
                                            });
                                        }
                                    })
                                    .into_any_element()
                            } else {
                                panel.into_any_element()
                            }
                        }),
                )
            })
            .child(
                div()
                    .id("show_hide_queue")
                    .px_2()
                    .py_2()
                    .absolute()
                    .top_2()
                    .right_2()
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
                    .child(icon(Icons::PanelRight).size_4()),
            )
    }
}
