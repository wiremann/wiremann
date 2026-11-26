use std::{
    hash::{DefaultHasher, Hash, Hasher},
    time::Duration,
};

use gpui::{
    AnyElement, App, Div, Entity, InteractiveElement, IntoElement, MouseButton, ParentElement,
    StatefulInteractiveElement, Styled, div, prelude::FluentBuilder, px,
};
use wgpui_motion::{
    MotionExt,
    prelude::{Transition, quint_out},
};

use crate::ui::{
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SortControlsState {
    pub field: SortField,
    pub descending: bool,
    pub menu_open: bool,
    pub menu_present: bool,
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

pub fn sort_controls(id: &'static str, state: Entity<SortControlsState>, cx: &mut App) -> Div {
    let theme = *cx.global::<Theme>();
    let current = *state.read(cx);
    let state_for_toggle = state.clone();

    div()
        .relative()
        .flex()
        .items_center()
        .gap_x_1()
        .p_1()
        .rounded_lg()
        .border_1()
        .border_color(theme.border)
        .bg(theme.library_bg)
        .child(
            div()
                .id(format!("{id}_field"))
                .flex()
                .items_center()
                .gap_x_2()
                .px_3()
                .py_2()
                .rounded_md()
                .text_sm()
                .text_color(theme.library_text)
                .hover(|this| this.bg(theme.library_tracks_section_bg_hover))
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
                .child(icon(Icons::Sort).size_4().text_color(theme.library_text))
                .child(current.field.label())
                .child(
                    icon(Icons::ChevronDown)
                        .size_4()
                        .text_color(theme.library_tracks_section_table_header_text),
                ),
        )
        .child(div().w(px(1.0)).h_5().bg(theme.border))
        .child(
            div()
                .id(format!("{id}_direction"))
                .flex()
                .items_center()
                .justify_center()
                .size_9()
                .rounded_md()
                .text_color(theme.library_text)
                .hover(|this| this.bg(theme.library_tracks_section_bg_hover))
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
                        .text_color(theme.library_text),
                ),
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

    let state_for_exit = state.clone();
    let rows = fields.iter().copied().map(|field| {
        let selected = field == current.field;
        let state = state.clone();
        div()
            .id(format!("{id}_option_{field:?}"))
            .w(px(112.0))
            .h(px(30.0))
            .flex()
            .flex_shrink_0()
            .items_center()
            .px_3()
            .rounded_md()
            .text_sm()
            .text_color(theme.library_text)
            .whitespace_nowrap()
            .when(selected, |this| {
                this.bg(theme.library_tracks_section_bg_hover)
            })
            .hover(|this| this.bg(theme.library_tracks_section_bg_hover))
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
    });

    Some(
        div()
            .id(format!("{id}_menu"))
            .absolute()
            .top(top)
            .right(right)
            .w(px(128.0))
            .p_1_5()
            .rounded_lg()
            .border_1()
            .border_color(theme.border)
            .bg(theme.toast_bg)
            .flex()
            .flex_col()
            .items_stretch()
            .occlude()
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .on_click(|_, _, cx| cx.stop_propagation())
            .motion(format!("{id}_menu_motion"))
            .transition(Transition::new(Duration::from_millis(180)).curve(quint_out()))
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
