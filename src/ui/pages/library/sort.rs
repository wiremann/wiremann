use std::hash::{DefaultHasher, Hash, Hasher};

use gpui::{
    AnyElement, App, Entity, InteractiveElement, IntoElement, MouseButton, ParentElement, Pixels,
    StatefulInteractiveElement, Styled, div, prelude::FluentBuilder, px,
};
use wgpui_motion::{
    MotionExt,
    prelude::{MotionToken, Transition},
};

use crate::ui::{
    components::bounds_observer::observe_bounds,
    components::icons::{Icons, icon},
    theme::Theme,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SortField {
    Title,
    Artist,
    Album,
    Duration,
    FileName,
    Name,
    TrackCount,
    AlbumCount,
}

impl SortField {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Title => "Title",
            Self::Artist => "Artist Name",
            Self::Album => "Album Name",
            Self::Duration => "Duration",
            Self::FileName => "File Name",
            Self::Name => "Name",
            Self::TrackCount => "Track Count",
            Self::AlbumCount => "Album Count",
        }
    }

    const fn direction_icon(self, descending: bool) -> Icons {
        match (self, descending) {
            (Self::Duration | Self::TrackCount | Self::AlbumCount, false) => Icons::ArrowUp01,
            (Self::Duration | Self::TrackCount | Self::AlbumCount, true) => Icons::ArrowDown10,
            (_, false) => Icons::ArrowUpAZ,
            (_, true) => Icons::ArrowDownZA,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SortControlsState {
    pub field: SortField,
    pub descending: bool,
    pub menu_open: bool,
    pub menu_present: bool,
    pub control_width: Pixels,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SortCacheKey {
    pub field: SortField,
    pub descending: bool,
    pub fingerprint: u64,
}

#[derive(Default)]
pub struct SortFingerprint(DefaultHasher);

impl SortFingerprint {
    pub fn add<T: Hash + ?Sized>(&mut self, value: &T) {
        value.hash(&mut self.0);
    }

    pub fn finish(self) -> u64 {
        self.0.finish()
    }
}

pub fn sort_controls(
    id: &'static str,
    state: Entity<SortControlsState>,
    cx: &mut App,
) -> impl IntoElement {
    let theme = *cx.global::<Theme>();
    let current = *state.read(cx);
    let state_for_toggle = state.clone();
    let controls = div()
        .relative()
        .flex()
        .items_center()
        .gap_x_1()
        .p_1()
        .rounded_lg()
        .border_1()
        .border_color(theme.library_sort_controls_border)
        .bg(theme.library_sort_controls_bg)
        .child(
            div()
                .id(format!("{id}_field"))
                .flex()
                .items_center()
                .justify_between()
                .w(px(136.0))
                .px_3()
                .py_2()
                .rounded_md()
                .text_sm()
                .text_color(theme.library_sort_controls_text)
                .hover(|this| this.bg(theme.library_sort_controls_hover_bg))
                .cursor_pointer()
                .on_click(move |_, _, cx| {
                    state_for_toggle.update(cx, |state, cx| {
                        state.menu_open = !state.menu_open;
                        if state.menu_open {
                            state.menu_present = true;
                        }
                        cx.notify();
                    });
                })
                .child(
                    icon(Icons::Sort)
                        .size_4()
                        .text_color(theme.library_sort_controls_icon),
                )
                .child(current.field.label())
                .child(
                    icon(Icons::ChevronDown)
                        .size_4()
                        .text_color(theme.library_sort_controls_icon),
                ),
        )
        .child(
            div()
                .w(px(1.0))
                .h_5()
                .bg(theme.library_sort_controls_divider),
        )
        .child(
            div()
                .id(format!("{id}_direction"))
                .flex()
                .items_center()
                .justify_center()
                .size_9()
                .rounded_md()
                .text_color(theme.library_sort_controls_text)
                .hover(|this| this.bg(theme.library_sort_controls_hover_bg))
                .cursor_pointer()
                .on_click({
                    let state = state.clone();
                    move |_, _, cx| {
                        state.update(cx, |state, cx| {
                            state.descending = !state.descending;
                            cx.notify();
                        });
                    }
                })
                .child(
                    icon(current.field.direction_icon(current.descending))
                        .size_4()
                        .text_color(theme.library_sort_controls_icon),
                ),
        );

    observe_bounds(
        format!("{id}_controls_bounds"),
        controls,
        move |bounds, _, cx| {
            let width = bounds.size.width;
            state.update(cx, |state, cx| {
                if state.control_width != width {
                    state.control_width = width;
                    cx.notify();
                }
            });
        },
    )
}

pub fn sort_menu(
    id: &'static str,
    state: Entity<SortControlsState>,
    fields: &'static [SortField],
    top: gpui::Pixels,
    right: gpui::Pixels,
    cx: &mut App,
) -> Option<AnyElement> {
    let theme = *cx.global::<Theme>();
    let current = *state.read(cx);
    if !current.menu_present {
        return None;
    }

    let width = current.control_width;
    if width == px(0.0) {
        return None;
    }

    let row_height = 32.0;
    let row_gap = 4.0;
    let vertical_padding = 12.0;
    let menu_height = fields.len() as f32 * row_height
        + fields.len().saturating_sub(1) as f32 * row_gap
        + vertical_padding;
    let state_for_exit = state.clone();
    let rows = fields.iter().copied().map(|field| {
        let selected = field == current.field;
        let state = state.clone();
        let hover_bg = theme.library_sort_menu_item_bg_hover;
        let hover_text = theme.library_sort_menu_item_text_hover;
        div()
            .id(format!("{id}_option_{field:?}"))
            .w_full()
            .h(px(row_height))
            .flex()
            .flex_shrink_0()
            .items_center()
            .px_3()
            .rounded_md()
            .text_sm()
            .text_color(if selected {
                theme.library_sort_menu_item_text_selected
            } else {
                theme.library_sort_menu_item_text
            })
            .whitespace_nowrap()
            .when(selected, |this| {
                this.bg(theme.library_sort_menu_item_bg_selected)
            })
            .cursor_pointer()
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .on_click(move |_, _, cx| {
                cx.stop_propagation();
                state.update(cx, |state, cx| {
                    state.field = field;
                    state.menu_open = false;
                    cx.notify();
                });
            })
            .child(field.label())
            .motion(format!("{id}_item_{field:?}"))
            .transition(Transition::token(MotionToken::EffectsFast))
            .while_hovered(move |this| this.bg(hover_bg).text_color(hover_text))
    });

    Some(
        div()
            .id(format!("{id}_menu"))
            .absolute()
            .top(top)
            .right(right)
            .w(width)
            .h(px(menu_height))
            .p_1_5()
            .rounded_lg()
            .border_1()
            .border_color(theme.library_sort_menu_border)
            .bg(theme.library_sort_menu_bg)
            .flex()
            .flex_col()
            .items_stretch()
            .gap_y_1()
            .occlude()
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .on_click(|_, _, cx| cx.stop_propagation())
            .motion(format!("{id}_menu_motion"))
            .transition(Transition::token(MotionToken::EffectsFast))
            .enter(|style| style.opacity(0.0))
            .base(|style| style.opacity(1.0))
            .exit_when(!current.menu_open, |style| style.opacity(0.0))
            .on_exit_complete(move |_, _, cx| {
                state_for_exit.update(cx, |state, cx| {
                    if !state.menu_open {
                        state.menu_present = false;
                        cx.notify();
                    }
                });
            })
            .children(rows)
            .into_any_element(),
    )
}
