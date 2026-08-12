mod helpers;

use crate::controller::Controller;
use crate::controller::state::PlaylistId;
use crate::controller::state::TrackId;
use crate::ui::components::icons::{Icon, Icons};
use crate::ui::components::image_cache::ImageCache;
use crate::ui::components::scrollbar::{RightPad, floating_scrollbar};
use crate::ui::helpers::{fingerprint_playlists, fingerprint_tracks};
use crate::ui::theme::Theme;
use gpui::prelude::FluentBuilder;
use gpui::{
    App, AppContext, Context, Div, Entity, FontWeight, ImageSource,
    InteractiveElement, IntoElement, MouseButton, ObjectFit, ParentElement, Pixels, Render,
    ScrollHandle, StatefulInteractiveElement, Styled, StyledImage, UniformListScrollHandle,
    VirtualListScrollController, Window, div, img, px, uniform_list, vlist,
};
use helpers::{PlaylistsRows, build_rows, render_header, render_track_table_header};
use std::rc::Rc;

const THUMBNAIL_MARGIN: usize = 16;

#[derive(Clone)]
pub struct PlaylistsPage {
    sidebar_scroll_handle: UniformListScrollHandle,
    main_scroll_handle: ScrollHandle,

    rows: Rc<Vec<PlaylistsRows>>,
    heights: Rc<Vec<Pixels>>,

    selected_playlist: Entity<Option<PlaylistId>>,
    sidebar_width: Entity<f32>,
    sidebar_open: Entity<bool>,
    last_fp: u128,
    pub list_controller: VirtualListScrollController,
}

impl PlaylistsPage {
    pub fn new(cx: &mut App) -> Self {
        let sidebar_scroll_handle = UniformListScrollHandle::new();
        let main_scroll_handle = ScrollHandle::new();

        let current_playlist = cx
            .global::<Controller>()
            .state
            .read(cx)
            .playback
            .current_playlist;

        // Auto-select the first playlist so there is never an empty state
        let initial = current_playlist.or_else(|| {
            cx.global::<Controller>()
                .state
                .read(cx)
                .library
                .playlists
                .keys()
                .next()
                .copied()
        });

        PlaylistsPage {
            sidebar_scroll_handle,
            main_scroll_handle,
            rows: Rc::new(Vec::new()),
            heights: Rc::new(Vec::new()),
            selected_playlist: cx.new(|_| initial),
            sidebar_width: cx.new(|_| 260.0),
            sidebar_open: cx.new(|_| false),
            last_fp: 0,
            list_controller: VirtualListScrollController::new(),
        }
    }

    fn render_track(i: usize, id: &TrackId, height: Pixels, cx: &mut App) -> Div {
        let image_id = {
            let state = cx.global::<Controller>().state.read(cx);
            state.library.tracks.get(id).and_then(|t| t.image_id)
        };

        let thumbnail = image_id.and_then(|id| cx.global_mut::<ImageCache>().get(&id));

        let controller = cx.global::<Controller>().clone();
        let theme = *cx.global::<Theme>();
        let state = controller.state.read(cx).clone();
        let is_current = Some(id) == state.playback.current.as_ref();

        if let Some(track) = state.library.tracks.get(id) {
            div()
                .h(height)
                .py_1()
                .px_4()
                .border_b_1()
                .border_color(theme.playlist_track_border)
                .child(
                    div()
                        .id(format!("track_{:?}", track.id.0))
                        .size_full()
                        .flex()
                        .items_center()
                        .rounded_md()
                        .cursor_pointer()
                        .hover(|this| this.bg(theme.playlist_track_bg_hover))
                        .when(is_current, |this| this.bg(theme.playlist_track_bg_current))
                        .on_click({
                            let id = *id;
                            move |_, _, cx| {
                                let controller = cx.global::<Controller>().clone();
                                controller.load_track(id, cx);
                            }
                        })
                        .child(
                            div()
                                .w_20()
                                .h_full()
                                .flex()
                                .px_6()
                                .items_center()
                                .justify_start()
                                .child(format!("{i:02}")),
                        )
                        .child(
                            div()
                                .w_2_3()
                                .max_w_2_3()
                                .h_full()
                                .px_6()
                                .py_1()
                                .flex()
                                .gap_x_3()
                                .items_center()
                                .justify_start()
                                .child(match thumbnail {
                                    Some(image) => div().size_11().flex_shrink_0().child(
                                        img(ImageSource::Render(image.clone()))
                                            .object_fit(ObjectFit::Contain)
                                            .size_full()
                                            .rounded_sm(),
                                    ),
                                    None => div().size_11().flex_shrink_0().child(
                                        img("icons/placeholder.svg")
                                            .object_fit(ObjectFit::Contain)
                                            .size_full()
                                            .rounded_sm(),
                                    ),
                                })
                                .when(is_current, |this| {
                                    this.text_color(theme.playlist_track_title_current)
                                        .font_weight(FontWeight::MEDIUM)
                                })
                                .child(track.title.clone())
                                .overflow_hidden()
                                .whitespace_nowrap()
                                .text_ellipsis(),
                        )
                        .child(
                            div()
                                .w_1_3()
                                .px_6()
                                .max_w_1_3()
                                .h_full()
                                .flex()
                                .items_center()
                                .justify_start()
                                .child(track.artist.clone())
                                .overflow_hidden()
                                .whitespace_nowrap()
                                .text_ellipsis(),
                        )
                        .child(
                            div()
                                .w_1_3()
                                .max_w_1_3()
                                .px_6()
                                .h_full()
                                .flex()
                                .items_center()
                                .justify_start()
                                .child(track.album.clone())
                                .overflow_hidden()
                                .whitespace_nowrap()
                                .text_ellipsis(),
                        )
                        .child(
                            div()
                                .w_24()
                                .max_w_24()
                                .h_full()
                                .flex()
                                .px_4()
                                .items_center()
                                .justify_start()
                                .text_sm()
                                .font_family("JetBrains Mono")
                                .child(format!(
                                    "{:02}:{:02}",
                                    track.duration.as_secs() / 60,
                                    track.duration.as_secs() % 60
                                ))
                                .overflow_hidden()
                                .whitespace_nowrap()
                                .text_ellipsis(),
                        ),
                )
        } else {
            div().h(height).py_2()
        }
    }
}

impl Render for PlaylistsPage {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let is_portrait = window.bounds().size.width < px(1000.0);

        let controller = cx.global::<Controller>().clone();
        let state = controller.state.read(cx);
        let sidebar_scroll_handle = self.sidebar_scroll_handle.clone();
        let main_scroll_handle = self.main_scroll_handle.clone();
        let selected = self.selected_playlist.clone();

        let tracks_fp = fingerprint_tracks(state.library.tracks.keys().copied());
        let playlists_fp = fingerprint_playlists(state.library.playlists.keys().copied());

        let selected_id = selected.read(cx).map_or(0, |p| p.0.as_u128());
        let combined_fp = tracks_fp ^ playlists_fp ^ selected_id;

        if combined_fp != self.last_fp {
            let (rows, heights) = build_rows(&state.library, *self.selected_playlist.read(cx));

            self.rows = Rc::new(rows);
            self.heights = Rc::new(heights);
            self.last_fp = combined_fp;
        }

        let rows = self.rows.clone();
        let heights = self.heights.clone();

        let playlists: Vec<_> = state.library.playlists.values().cloned().collect();
        let sidebar_w = *self.sidebar_width.read(cx);
        let entity = cx.entity();
        let selected_val = *selected.read(cx);

        // Build landscape layout (sidebar + divider + content)
        let landscape: gpui::AnyElement = div().size_full().flex().child(
            sidebar(
                &theme,
                playlists.clone(),
                selected.clone(),
                sidebar_scroll_handle.clone(),
                px(sidebar_w),
            ),
        )
        .child(
            div()
                .id("sidebar_resize_handle")
                .w(px(1.0))
                .h_full()
                .flex_shrink_0()
                .bg(theme.border),
        )
        .child(main_content(
            entity.clone(),
            rows.clone(),
            heights.clone(),
            selected_val,
            controller.clone(),
            main_scroll_handle.clone(),
            self.list_controller.clone(),
        ))
        .into_any_element();

        // Build portrait layout (hamburger toolbar + content + slide-in drawer)
        let sidebar_open = self.sidebar_open.clone();
        let is_open = *sidebar_open.read(cx);
        let portrait: gpui::AnyElement = div()
            .relative()
            .size_full()
            .overflow_hidden()
            .flex()
            .flex_col()
            .child(
                div()
                    .id("playlist_toolbar")
                    .w_full()
                    .h_10()
                    .flex_shrink_0()
                    .flex()
                    .items_center()
                    .px_4()
                    .child(
                        div()
                            .id("hamburger_btn")
                            .p_2()
                            .rounded_md()
                            .cursor_pointer()
                            .hover(|this| this.bg(theme.playlist_sidebar_item_bg_hover))
                            .on_click({
                                let sidebar_open = sidebar_open.clone();
                                move |_, _, cx| {
                                    let new = !*sidebar_open.read(cx);
                                    sidebar_open.update(cx, |s, _| *s = new);
                                }
                            })
                            .child(if is_open {
                                Icon::new(Icons::WinClose).size_5().text_color(theme.playlist_page_text)
                            } else {
                                Icon::new(Icons::Menu).size_5().text_color(theme.playlist_page_text)
                            }),
                    ),
            )
            .child(main_content(
                entity.clone(),
                rows.clone(),
                heights.clone(),
                selected_val,
                controller.clone(),
                main_scroll_handle.clone(),
                self.list_controller.clone(),
            ))
            // Dark backdrop overlay (clickable to close) — only when open
            .when(is_open, |root| {
                root.child(
                    div()
                        .id("sidebar_backdrop")
                        .absolute()
                        .size_full()
                        .top_0()
                        .left_0()
                        .bg(gpui::rgba(0x00000080))
                        .on_click({
                            let sidebar_open = sidebar_open.clone();
                            move |_, _, cx| {
                                sidebar_open.update(cx, |s, _| *s = false);
                            }
                        }),
                )
            })
            // Slide-in drawer from the left
            .when(is_open, |root| {
                root.child(
                    div()
                        .id("sidebar_drawer")
                        .absolute()
                        .top_0()
                        .left_0()
                        .h_full()
                        .w(px(280.0))
                        .bg(theme.playlist_page_bg)
                        .child(
                            div()
                                .w_full()
                                .flex()
                                .items_center()
                                .justify_start()
                                .p_4()
                                .child(
                                    div()
                                        .text_base()
                                        .text_color(theme.playlist_sidebar_item_title)
                                        .font_weight(FontWeight(500.0))
                                        .child("Playlists"),
                                ),
                        )
                        .child(
                            div()
                                .id("playlist_sidebar_container")
                                .w_full()
                                .h_full()
                                .px_4()
                                .pb_4()
                                .flex()
                                .relative()
                                .size_full()
                                .flex_1()
                                .child(sidebar_list(
                                    &theme,
                                    playlists.clone(),
                                    selected.clone(),
                                    sidebar_scroll_handle.clone(),
                                )),
                        ),
                )
            })
            .into_any_element();

        div()
            .size_full()
            .bg(theme.playlist_page_bg)
            .text_color(theme.playlist_page_text)
            .flex()
            .child(if is_portrait { portrait } else { landscape })
    }
}

/// Reusable sidebar (landscape) or popover content (portrait)
fn sidebar(
    theme: &Theme,
    playlists: Vec<crate::controller::state::Playlist>,
    selected: Entity<Option<PlaylistId>>,
    sidebar_scroll_handle: UniformListScrollHandle,
    width: Pixels,
) -> impl IntoElement {
    let theme = *theme;
    div()
        .w(width)
        .h_full()
        .flex_shrink_0()
        .flex()
        .flex_col()
        .gap_2()
        .border_r_1()
        .border_color(theme.border)
        .child(
            div()
                .w_full()
                .flex()
                .items_center()
                .justify_start()
                .p_4()
                .child(
                    div()
                        .text_base()
                        .text_color(theme.playlist_sidebar_item_title)
                        .font_weight(FontWeight(500.0))
                        .child("Playlists"),
                ),
        )
        .child(sidebar_list(&theme, playlists, selected, sidebar_scroll_handle))
}

/// The scrollable list of playlists in the sidebar
fn sidebar_list(
    theme: &Theme,
    playlists: Vec<crate::controller::state::Playlist>,
    selected: Entity<Option<PlaylistId>>,
    sidebar_scroll_handle: UniformListScrollHandle,
) -> impl IntoElement {
    let theme = *theme;
    let len = playlists.len();
    div()
        .id("playlist_sidebar_container")
        .w_full()
        .h_full()
        .px_4()
        .pb_4()
        .flex()
        .relative()
        .size_full()
        .flex_1()
        .child(
            uniform_list("playlist_sidebar", len, {
                let selected = selected.clone();

                move |range, _, cx| {
                    range
                        .map(|i| {
                            let playlist = &playlists[i];
                            let is_current = Some(playlist.id) == *selected.read(cx);

                            let controller = cx.global_mut::<Controller>().clone();
                            controller.request_playlist_thumbnails(&[playlist.id], cx);

                            let thumbnail =
                                playlist.image_id.and_then(|id| {
                                    cx.global_mut::<ImageCache>().get(&id)
                                });

                            div().py(px(2.0)).child(
                                div()
                                    .id(format!("playlist_sidebar_{}", playlist.id.0))
                                    .h(px(64.))
                                    .w_full()
                                    .flex()
                                    .items_center()
                                    .p_3()
                                    .mb_2()
                                    .gap_4()
                                    .rounded_lg()
                                    .hover(|d| d.bg(theme.playlist_sidebar_item_bg_hover))
                                    .when(is_current, |d| {
                                        d.bg(theme.playlist_sidebar_item_bg_current)
                                    })
                                    .cursor_pointer()
                                    .on_click({
                                        let id = playlist.id;
                                        let selected = selected.clone();
                                        move |_, _, cx| {
                                            selected.update(cx, |this, cx| {
                                                *this = Some(id);
                                                cx.notify();
                                            });
                                        }
                                    })
                                    .child(match thumbnail {
                                        Some(image) => div().size_12().flex_shrink_0().child(
                                            img(ImageSource::Render(image.clone()))
                                                .object_fit(ObjectFit::Contain)
                                                .size_full()
                                                .border_1()
                                                .border_color(theme.border)
                                                .rounded_md(),
                                        ),
                                        None => div().size_12().flex_shrink_0().child(
                                            img("icons/placeholder.svg")
                                                .object_fit(ObjectFit::Contain)
                                                .size_full()
                                                .border_1()
                                                .border_color(theme.border)
                                                .rounded_md(),
                                        ),
                                    })
                                    .child(
                                        div()
                                            .flex_col()
                                            .flex_1()
                                            .justify_center()
                                            .child(
                                                div()
                                                    .text_base()
                                                    .truncate()
                                                    .text_color(if is_current {
                                                        theme.playlist_sidebar_item_title_current
                                                    } else {
                                                        theme.playlist_sidebar_item_title
                                                    })
                                                    .child(playlist.name.clone()),
                                            )
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .text_color(theme.playlist_sidebar_item_meta)
                                                    .truncate()
                                                    .child(format!("{} tracks", playlist.tracks.len())),
                                            ),
                                    ),
                            )
                        })
                        .collect::<Vec<_>>()
                }
            })
            .track_scroll(&sidebar_scroll_handle)
            .w_full()
            .h_full()
            .flex()
            .flex_col(),
        )
        .child(floating_scrollbar(
            "queue_scrollbar",
            sidebar_scroll_handle,
            RightPad::Pad,
        ))
}

/// The track-list main area
#[allow(clippy::too_many_arguments)]
fn main_content(
    entity: Entity<PlaylistsPage>,
    rows: Rc<Vec<PlaylistsRows>>,
    heights: Rc<Vec<Pixels>>,
    selected: Option<PlaylistId>,
    controller: Controller,
    main_scroll_handle: ScrollHandle,
    list_controller: VirtualListScrollController,
) -> impl IntoElement {
    let render_fn = |i: usize, id: &TrackId, h: Pixels, cx: &mut App| -> Div {
        PlaylistsPage::render_track(i, id, h, cx)
    };
    let msh = main_scroll_handle.clone();
    let msh_for_scrollbar = msh.clone();
    div()
        .w_full()
        .h_full()
        .flex()
        .flex_grow()
        .child(vlist(
            entity,
            "playlists_main",
            heights.clone(),
            msh,
            list_controller,
            {
                let rows = rows.clone();
                move |_this, range, _, cx| {
                    let len = rows.len();
                    let start = range.start.saturating_sub(THUMBNAIL_MARGIN);
                    let end = (range.end + THUMBNAIL_MARGIN).min(len);

                    let thumb_track_ids: Vec<TrackId> = (start..end)
                        .filter_map(|idx| match &rows[idx] {
                            PlaylistsRows::TrackRow(_, id) => Some(*id),
                            _ => None,
                        })
                        .collect();

                    controller.request_track_thumbnails(&thumb_track_ids, cx);

                    range
                        .map(|idx| match &rows[idx] {
                            PlaylistsRows::Header => {
                                render_header(heights[idx], selected, cx)
                            }
                            PlaylistsRows::TrackTableHeader => {
                                render_track_table_header(heights[idx], cx)
                            }
                            PlaylistsRows::TrackRow(i, id) => {
                                render_fn(*i, id, heights[idx], cx)
                            }
                        })
                        .collect::<Vec<_>>()
                }
            },
        ))
        .child(floating_scrollbar(
            "queue_scrollbar",
            msh_for_scrollbar,
            RightPad::Pad,
        ))
}
