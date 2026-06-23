use crate::controller::Controller;
use crate::controller::state::TrackId;
use crate::lyrics_manager::{LyricLine, Lyrics, SyncType};
use crate::ui::components::bounds_observer::observe_bounds;
use crate::ui::components::icons::{Icon, Icons};
use crate::ui::theme::Theme;
use ahash::AHashMap;
use gpui::prelude::FluentBuilder;
use std::cell::RefCell;

use crate::ui::components::virtual_list::{VirtualListScrollController, vlist};
use gpui::{
    Animation, AnimationExt, App, AppContext, Bounds, Context, Entity, FontWeight, Global,
    InteractiveElement, IntoElement, ParentElement, Pixels, Render, ScrollHandle, Styled, Window,
    div, gradient_color_stop, linear, linear_gradient, percentage, px, relative, rgba,
};

use std::rc::Rc;
use std::time::Duration;

const LYRICS_TEXT_SIZE: Pixels = px(38.0);

#[derive(Debug, PartialEq)]
pub struct LyricsStateInner {
    pub status: LyricsStatus,
    pub lyrics: Option<Lyrics>,
    pub track_id: Option<TrackId>,
}

#[derive(Debug, PartialEq)]
pub enum LyricsStatus {
    Fetching,
    Available,
    Unavailable,
}

pub struct LyricsState(pub Entity<LyricsStateInner>);

impl Global for LyricsState {}

impl Default for LyricsStateInner {
    fn default() -> Self {
        Self::new()
    }
}

impl LyricsStateInner {
    #[must_use]
    pub fn new() -> Self {
        Self {
            status: LyricsStatus::Unavailable,
            lyrics: None,
            track_id: None,
        }
    }
}

pub struct LyricLineView {
    pub line: LyricLine,
    pub idx: usize,
    pub sync_type: SyncType,
    pub word_bounds: Rc<RefCell<AHashMap<(usize, usize), Bounds<Pixels>>>>,
}

impl LyricLineView {
    pub fn new(
        cx: &mut App,
        line: LyricLine,
        idx: usize,
        sync_type: SyncType,
        word_bounds: Rc<RefCell<AHashMap<(usize, usize), Bounds<Pixels>>>>,
    ) -> Entity<Self> {
        cx.new(|_| Self {
            line,
            idx,
            sync_type,
            word_bounds,
        })
    }
}

impl Render for LyricLineView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = cx.global::<Controller>().state.read(cx);
        let theme = *cx.global::<Theme>();

        let playback = state.playback.position + Duration::from_millis(80);

        let lyrics = cx.global::<LyricsState>().0.read(cx).lyrics.clone();

        let Some(lyrics) = lyrics else {
            return div().into_any_element();
        };

        let active_line = lyrics
            .lines
            .iter()
            .enumerate()
            .rfind(|(_, line)| line.start.is_some_and(|s| playback >= s))
            .map_or(0, |(idx, _)| idx);

        let is_active_line = self.idx == active_line;
        let distance = self.idx.abs_diff(active_line) as f32;
        let inactive_opacity = match distance as i32 {
            0 => 0.35,
            1 => 0.32,
            2 => 0.26,
            3 => 0.18,
            _ => 0.10,
        };
        let active_opacity = match distance as i32 {
            0 => 1.0,
            1 => 0.75,
            2 => 0.45,
            _ => 0.2,
        };
        match self.sync_type {
            SyncType::Line => {
                let opacity = if is_active_line { 1.0 } else { 0.4 };

                div()
                    .id(("line", self.idx))
                    .w_full()
                    .min_w_0()
                    .py_2()
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        div()
                            .max_w_full()
                            .text_center()
                            .text_size(LYRICS_TEXT_SIZE)
                            .font_weight(FontWeight::BOLD)
                            .text_color(theme.lyrics_text_active)
                            .opacity(opacity)
                            .child(self.line.text.to_string()),
                    )
                    .into_any_element()
            }

            SyncType::Word => {
                let words = self.line.words.clone().unwrap_or_default();

                div()
                    .id(("line", self.idx))
                    .w_full()
                    .min_w_0()
                    .py_2()
                    .flex()
                    .justify_center()
                    .child(
                        div()
                            .w_full()
                            .min_w_0()
                            .flex()
                            .flex_row()
                            .flex_wrap()
                            .justify_center()
                            .children(words.iter().enumerate().map(|(word_idx, word)| {
                                let progress = {
                                    let next_start = self
                                        .line
                                        .words
                                        .as_ref()
                                        .and_then(|w| w.get(word_idx + 1))
                                        .map_or(word.end, |w| w.start);

                                    if playback < word.start {
                                        0.0
                                    } else if playback >= next_start {
                                        1.0
                                    } else {
                                        let duration = next_start.saturating_sub(word.start);

                                        let elapsed = playback.saturating_sub(word.start);

                                        if duration.is_zero() {
                                            1.0
                                        } else {
                                            elapsed.as_secs_f32() / duration.as_secs_f32()
                                        }
                                    }
                                }
                                .clamp(0.0, 1.0);

                                observe_bounds(
                                    format!("word_measure_{}_{}", self.idx, word_idx),
                                    div()
                                        .relative()
                                        .flex_none()
                                        .child(
                                            div()
                                                .id(format!("base_word_{}_{}", self.idx, word_idx))
                                                .text_size(LYRICS_TEXT_SIZE)
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(theme.lyrics_text_inactive)
                                                .opacity(inactive_opacity)
                                                .child(word.text.to_string()),
                                        )
                                        .child(
                                            div()
                                                .absolute()
                                                .h_full()
                                                .top_0()
                                                .left_0()
                                                .overflow_hidden()
                                                .when_some(
                                                    {
                                                        let bounds_cache =
                                                            self.word_bounds.borrow();

                                                        bounds_cache
                                                            .get(&(self.idx, word_idx))
                                                            .map(|b| b.size.width * progress)
                                                    },
                                                    gpui::Styled::w,
                                                )
                                                .child(
                                                    div()
                                                        .h_full()
                                                        .flex()
                                                        .items_center()
                                                        .text_size(LYRICS_TEXT_SIZE)
                                                        .font_weight(FontWeight::BOLD)
                                                        .text_color(theme.lyrics_text_active)
                                                        .opacity(active_opacity)
                                                        .child(word.text.to_string()),
                                                ),
                                        ),
                                    {
                                        let bounds_cache = self.word_bounds.clone();

                                        let line_idx = self.idx;

                                        move |bounds, _, _cx| {
                                            bounds_cache
                                                .borrow_mut()
                                                .insert((line_idx, word_idx), bounds);
                                        }
                                    },
                                )
                            })),
                    )
                    .into_any_element()
            }
            SyncType::Unsynced => div()
                .id(("unsynced", self.idx))
                .w_full()
                .min_w_0()
                .py_1()
                .child(
                    div()
                        .text_size(LYRICS_TEXT_SIZE)
                        .font_weight(FontWeight::BOLD)
                        .text_color(theme.lyrics_text_active)
                        .child(self.line.text.to_string()),
                )
                .into_any_element(),
        }
    }
}

#[derive(Clone)]
pub struct LyricsView {
    pub views: Entity<AHashMap<usize, Entity<LyricLineView>>>,

    pub scroll_handle: ScrollHandle,
    pub list_controller: VirtualListScrollController,

    pub last_active_line: usize,
    pub last_track_id: Option<TrackId>,

    pub panel_bounds: Option<Bounds<Pixels>>,
    pub measured_heights: Vec<Pixels>,
    pub word_bounds: Rc<RefCell<AHashMap<(usize, usize), Bounds<Pixels>>>>,
}

impl LyricsView {
    pub fn new(cx: &mut App, scroll_handle: ScrollHandle) -> Entity<Self> {
        cx.new(|cx| Self {
            views: cx.new(|_| AHashMap::new()),
            scroll_handle,
            last_active_line: 0,
            panel_bounds: None,
            measured_heights: Vec::new(),
            list_controller: VirtualListScrollController::new(),
            word_bounds: Rc::new(RefCell::new(AHashMap::new())),
            last_track_id: None,
        })
    }

    fn get_or_create_line(
        views: &Entity<AHashMap<usize, Entity<LyricLineView>>>,
        line: LyricLine,
        idx: usize,
        sync_type: SyncType,
        word_bounds: Rc<RefCell<AHashMap<(usize, usize), Bounds<Pixels>>>>,
        cx: &mut App,
    ) -> Entity<LyricLineView> {
        views.update(cx, |this, cx| {
            this.entry(idx)
                .or_insert_with(|| LyricLineView::new(cx, line, idx, sync_type, word_bounds))
                .clone()
        })
    }

    fn active_line(lines: &[LyricLine], playback: Duration) -> usize {
        lines
            .iter()
            .enumerate()
            .rfind(|(_, line)| line.start.is_some_and(|s| playback >= s))
            .map_or(0, |(idx, _)| idx)
    }
}

impl Render for LyricsView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let theme = *cx.global::<Theme>();

        let state = cx.global::<Controller>().state.read(cx);

        let playback = state.playback.position;

        let lyrics_state = cx.global::<LyricsState>().0.read(cx);

        match lyrics_state.status {
            LyricsStatus::Fetching => {
                return div()
                    .size_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_3()
                            .items_center()
                            .child(
                                Icon::new(Icons::Loader)
                                    .size_6()
                                    .text_color(theme.lyrics_loading_icon)
                                    .with_animation(
                                        "lyrics_loader_spin",
                                        Animation::new(Duration::from_secs(1))
                                            .repeat()
                                            .with_easing(linear),
                                        |icon, delta| icon.rotate(percentage(delta)),
                                    ),
                            )
                            .child(
                                div()
                                    .text_color(theme.lyrics_loading_text)
                                    .text_lg()
                                    .child("Fetching lyrics..."),
                            ),
                    )
                    .into_any_element();
            }

            LyricsStatus::Unavailable => {
                return div()
                    .size_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        div()
                            .text_color(theme.lyrics_empty_text)
                            .text_lg()
                            .child("No lyrics"),
                    )
                    .into_any_element();
            }

            LyricsStatus::Available => {}
        }

        let Some(lyrics) = lyrics_state.lyrics.clone() else {
            return div().into_any_element();
        };

        if self.measured_heights.len() != lyrics.lines.len() {
            self.measured_heights =
                vec![px(LYRICS_TEXT_SIZE.to_f64() as f32 * 2.5); lyrics.lines.len()];
        }

        if self.last_track_id != lyrics_state.track_id {
            self.last_track_id = lyrics_state.track_id;
            self.views.update(cx, |this, _| this.clear());
            self.word_bounds.borrow_mut().clear();
            self.measured_heights.clear();

            self.last_active_line = 0;
            self.list_controller.scroll_to_item(0);
        }

        let active_line = Self::active_line(&lyrics.lines, playback);

        let display_line = {
            if let Some(current) = lyrics.lines.get(active_line) {
                if let Some(end) = current.end {
                    if playback >= end {
                        (active_line + 1).min(lyrics.lines.len().saturating_sub(1))
                    } else {
                        active_line
                    }
                } else {
                    active_line
                }
            } else {
                active_line
            }
        };

        if display_line != self.last_active_line {
            self.last_active_line = display_line;

            self.list_controller.scroll_to_item(display_line);
        }

        let views = self.views.clone();

        let lines = lyrics.lines.clone();

        let sync_type = lyrics.sync_type.clone();

        let measured_heights = Rc::new(self.measured_heights.clone());
        let word_bounds = self.word_bounds.clone();
        let list_entity = entity.clone();
        let list = vlist(
            cx.entity(),
            "lyrics",
            measured_heights.clone(),
            self.scroll_handle.clone(),
            &self.list_controller,
            move |_this, range, _, cx| {
                range
                    .map(|idx| {
                        let line = lines[idx].clone();

                        observe_bounds(
                            ("lyrics_line_measure", idx),
                            div()
                                .font_family("Space Grotesk")
                                .id(("lyrics_line", idx))
                                .w_full()
                                .min_w_0()
                                .child(LyricsView::get_or_create_line(
                                    &views,
                                    line,
                                    idx,
                                    sync_type.clone(),
                                    word_bounds.clone(),
                                    cx,
                                )),
                            {
                                let entity = list_entity.clone();

                                move |bounds, _, cx| {
                                    entity.update(cx, |this, cx| {
                                        let height = bounds.size.height;

                                        if let Some(existing) = this.measured_heights.get_mut(idx)
                                            && *existing != height
                                        {
                                            *existing = height;

                                            cx.notify();
                                        }
                                    });
                                }
                            },
                        )
                    })
                    .collect::<Vec<_>>()
            },
        );

        let root = div()
            .font_family("Space Grotesk")
            .relative()
            .w_full()
            .min_w_0()
            .h_full()
            .min_h_0()
            .child(list)
            .child(
                div()
                    .absolute()
                    .top_0()
                    .left_0()
                    .right_0()
                    .h(relative(0.32))
                    .bg(linear_gradient(
                        180.,
                        gradient_color_stop(theme.lyrics_fade_top, 0.0),
                        gradient_color_stop(rgba(0x00000000), 1.0),
                    )),
            )
            .child(
                div()
                    .absolute()
                    .bottom_0()
                    .left_0()
                    .right_0()
                    .h(relative(0.32))
                    .bg(linear_gradient(
                        0.,
                        gradient_color_stop(theme.lyrics_fade_bottom, 0.0),
                        gradient_color_stop(rgba(0x00000000), 1.0),
                    )),
            );

        observe_bounds("lyrics_panel_bounds", root, move |bounds, _, cx| {
            entity.update(cx, |this, cx| {
                this.panel_bounds = Some(bounds);

                cx.notify();
            });
        })
        .into_any_element()
    }
}
