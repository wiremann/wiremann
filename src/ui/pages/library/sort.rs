use std::hash::{DefaultHasher, Hash, Hasher};
use std::time::Duration;

use gpui::{
    App, Context, Div, Entity, InteractiveElement, IntoElement, MouseButton, ParentElement, Pixels,
    Render, StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, px, rems,
};
use wgpui_motion::{MotionExt, prelude::Transition};

use crate::ui::{
    components::icons::{Icons, icon},
    theme::Theme,
};

const SORT_MENU_WIDTH: f32 = 156.0;

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

pub struct SortMenu {
    id: &'static str,
    sort: Entity<SortControlsState>,
    fields: &'static [SortField],
    height: Pixels,
    top: Pixels,
    right: Pixels,
    open: bool,
    present: bool,
}

impl SortMenu {
    pub fn new(
        id: &'static str,
        sort: Entity<SortControlsState>,
        fields: &'static [SortField],
        height: Pixels,
        top: Pixels,
        right: Pixels,
    ) -> Self {
        Self {
            id,
            sort,
            fields,
            height,
            top,
            right,
            open: false,
            present: false,
        }
    }
}

pub fn sort_controls(
    id: &'static str,
    state: Entity<SortControlsState>,
    menu: Entity<SortMenu>,
    cx: &mut App,
) -> Div {
    let theme = *cx.global::<Theme>();
    let current = *state.read(cx);
    let menu_for_toggle = menu.clone();
    div()
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
                .gap_x_2()
                .px_3()
                .py_2()
                .rounded_md()
                .text_sm()
                .text_color(theme.library_sort_controls_text)
                .hover(|this| this.bg(theme.library_sort_controls_hover_bg))
                .cursor_pointer()
                .on_click(move |_, window, cx| {
                    menu_for_toggle.update(cx, |menu, cx| {
                        menu.open = !menu.open;
                        if menu.open {
                            menu.present = true;
                        }
                        cx.notify();
                    });
                    window.refresh();
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
        )
}

impl Render for SortMenu {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.present {
            return div().into_any_element();
        }

        let theme = *cx.global::<Theme>();
        let sort = *self.sort.read(cx);
        let menu = cx.entity();
        let rows = self.fields.iter().copied().map(|field| {
            let selected = field == sort.field;
            let sort = self.sort.clone();
            let menu_for_select = menu.clone();
            let hover_bg = theme.library_sort_menu_item_bg_hover;
            let hover_text = theme.library_sort_menu_item_text_hover;
            div()
                .id(format!("{}_option_{field:?}", self.id))
                .w_full()
                .max_w_full()
                .h(px(30.0))
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
                    sort.update(cx, |sort, cx| {
                        sort.field = field;
                        cx.notify();
                    });
                    menu_for_select.update(cx, |menu, cx| {
                        menu.open = false;
                        cx.notify();
                    });
                })
                .child(field.label())
                .motion(format!("{}_option_motion_{field:?}", self.id))
                .transition(Transition::new(Duration::from_millis(90)))
                .while_hovered(move |this| {
                    this.bg(hover_bg)
                        .text_color(hover_text)
                        .text_size(rems(0.875))
                })
        });

        let menu_for_exit = menu.clone();
        div()
            .id(format!("{}_menu", self.id))
            .absolute()
            .top(self.top)
            .right(self.right)
            .w(px(SORT_MENU_WIDTH))
            .h(self.height)
            .p_2()
            .flex()
            .flex_col()
            .items_stretch()
            .gap_y_1()
            .rounded_lg()
            .border_1()
            .border_color(theme.library_sort_menu_border)
            .bg(theme.library_sort_menu_bg)
            .occlude()
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .on_click(|_, _, cx| cx.stop_propagation())
            .children(rows)
            .motion(format!("{}_menu_motion", self.id))
            .transition(Transition::new(Duration::from_millis(90)))
            .enter(|style| style.opacity(0.0))
            .base(|style| style.opacity(1.0))
            .exit_when(!self.open, |style| style.opacity(0.0))
            .on_exit_complete(move |_, _, cx| {
                menu_for_exit.update(cx, |menu, cx| {
                    if !menu.open {
                        menu.present = false;
                        cx.notify();
                    }
                });
            })
            .into_any_element()
    }
}
