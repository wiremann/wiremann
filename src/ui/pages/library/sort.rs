use std::hash::{DefaultHasher, Hash, Hasher};

use gpui::{
    App, Div, Entity, InteractiveElement, ParentElement, StatefulInteractiveElement, Styled, div,
    prelude::FluentBuilder,
};

use crate::ui::theme::Theme;

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
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SortControlsState {
    pub field: SortField,
    pub descending: bool,
    pub menu_open: bool,
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
    fields: &'static [SortField],
    cx: &mut App,
) -> Div {
    let theme = *cx.global::<Theme>();
    let current = *state.read(cx);
    let state_for_toggle = state.clone();

    div()
        .relative()
        .flex()
        .items_center()
        .gap_2()
        .child(
            div()
                .id(format!("{id}_field"))
                .px_3()
                .py_2()
                .rounded_md()
                .border_1()
                .border_color(theme.border)
                .text_sm()
                .text_color(theme.library_text)
                .hover(|this| this.bg(theme.library_tracks_section_bg_hover))
                .cursor_pointer()
                .on_click(move |_, _, cx| {
                    state_for_toggle.update(cx, |state, cx| {
                        state.menu_open = !state.menu_open;
                        cx.notify();
                    });
                })
                .child(format!("{}  v", current.field.label())),
        )
        .child(
            div()
                .id(format!("{id}_direction"))
                .px_3()
                .py_2()
                .rounded_md()
                .border_1()
                .border_color(theme.border)
                .text_sm()
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
                .child(if current.descending {
                    "Descending"
                } else {
                    "Ascending"
                }),
        )
        .when(current.menu_open, {
            let state = state.clone();
            move |this| {
                this.child(
                    div()
                        .absolute()
                        .top_10()
                        .right_0()
                        .min_w_40()
                        .p_1()
                        .rounded_md()
                        .border_1()
                        .border_color(theme.border)
                        .bg(theme.library_bg)
                        .flex()
                        .flex_col()
                        .children(fields.iter().copied().map(|field| {
                            let selected = field == current.field;
                            let state = state.clone();
                            div()
                                .id(format!("{id}_option_{field:?}"))
                                .w_full()
                                .px_3()
                                .py_2()
                                .rounded_sm()
                                .text_sm()
                                .text_color(theme.library_text)
                                .when(selected, |this| {
                                    this.bg(theme.library_tracks_section_bg_hover)
                                })
                                .hover(|this| this.bg(theme.library_tracks_section_bg_hover))
                                .cursor_pointer()
                                .on_click(move |_, _, cx| {
                                    state.update(cx, |state, cx| {
                                        state.field = field;
                                        state.menu_open = false;
                                        cx.notify();
                                    });
                                })
                                .child(field.label())
                        })),
                )
            }
        })
}
