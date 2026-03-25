use crate::controller::state::LibraryState;
use crate::controller::Controller;
use crate::library::playlists::PlaylistId;
use crate::library::TrackId;
use crate::ui::components::image_cache::ImageCache;
use crate::ui::components::scrollbar::{floating_scrollbar, RightPad};
use crate::ui::components::virtual_list::vlist;
use crate::ui::helpers::{fingerprint_playlists, fingerprint_tracks};
use crate::ui::theme::Theme;
use gpui::prelude::FluentBuilder;
use gpui::{div, img, px, uniform_list, App, AppContext, Context, Div, Entity, FontWeight, InteractiveElement, IntoElement, ObjectFit, ParentElement, Pixels, Render, ScrollHandle, StatefulInteractiveElement, Styled, StyledImage, UniformListScrollHandle, Window};
use std::rc::Rc;

const THUMBNAIL_MARGIN: usize = 16;

#[derive(Clone)]
pub struct PlaylistsPage {
    sidebar_scroll_handle: UniformListScrollHandle,
    main_scroll_handle: ScrollHandle,

    rows: Rc<Vec<PlaylistsRows>>,
    heights: Rc<Vec<Pixels>>,

    selected_playlist: Entity<Option<PlaylistId>>,
    last_fp: u128,
}

#[derive(Clone)]
enum PlaylistsRows {
    Header,
    TrackTableHeader,
    TrackRow(usize, TrackId),
}

impl PlaylistsPage {
    pub fn new(cx: &mut App) -> Self {
        let sidebar_scroll_handle = UniformListScrollHandle::new();
        let main_scroll_handle = ScrollHandle::new();

        PlaylistsPage {
            sidebar_scroll_handle,
            main_scroll_handle,
            rows: Rc::new(Vec::new()),
            heights: Rc::new(Vec::new()),
            selected_playlist: cx.new(|_| None),
            last_fp: 0,
        }
    }

    fn render_header(height: Pixels, id: Option<PlaylistId>, cx: &mut App) -> Div {
        let theme = cx.global::<Theme>();
        let controller = cx.global::<Controller>().clone();

        let state = controller.state.read(cx).clone();


        let cache = cx.global_mut::<ImageCache>();

        if let Some(id) = id && let Some(playlist) = state.library.playlists.get(&id) {
            controller.request_playlist_thumbnails(&[id], cx);
            let thumbnail = playlist.image_id.and_then(|id| cache.get(&id));

            div()
                .flex()
                .w_full()
                .h(height)
                .bg(theme.bg_queue)
                .child(
                    div()
                        .size(height)
                        .p_12()
                        .child(
                            match thumbnail {
                                Some(image) => div().size_full().child(
                                    img(image.clone())
                                        .object_fit(ObjectFit::Contain)
                                        .size_full()
                                        .rounded_lg(),
                                ),
                                None => div().size(height).flex_shrink_0(),
                            }
                        )
                )
                .child(
                    div()
                        .w_full()
                        .h(height)
                        .flex()
                        .flex_col()
                        .px_10()
                        .py_12()
                        .gap_y_4()
                        .child(
                            div().text_xl().text_color(theme.text_primary).child(playlist.name)
                        )
                        .child(
                            div().text_base().text_color(theme.text_secondary)
                                .child(format!("{} tracks", playlist.tracks.len()))
                        )
                )
        } else {
            div()
        }
    }

    fn render_track_table_header(height: Pixels, cx: &mut App) -> Div {
        let theme = cx.global::<Theme>();

        div()
            .h(height)
            .w_full()
            .flex()
            .items_center()
            .text_xs()
            .font_weight(FontWeight::NORMAL)
            .text_color(theme.text_muted)
            .border_b_1()
            .border_color(theme.white_05)
            .child(
                div()
                    .w_20()
                    .h_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .child("#"),
            )
            .child(
                div()
                    .w_3_5()
                    .h_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .child("TITLE"),
            )
            .child(
                div()
                    .w_1_2()
                    .h_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .child("ARTIST"),
            )
            .child(
                div()
                    .w_1_2()
                    .h_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .child("ALBUM"),
            )
            .child(
                div()
                    .w_24()
                    .h_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .child("DURATION"),
            )
    }

    fn render_track(i: usize, id: &TrackId, height: Pixels, cx: &mut App) -> Div {
        let image_id = {
            let state = cx.global::<Controller>().state.read(cx);
            state.library.tracks.get(id).and_then(|t| t.image_id)
        };

        let thumbnail = image_id.and_then(|id| cx.global_mut::<ImageCache>().get(&id));

        let controller = cx.global::<Controller>().clone();
        let theme = cx.global::<Theme>().clone();
        let state = controller.state.read(cx).clone();
        let is_current = Some(id) == state.playback.current.as_ref();

        if let Some(track) = state.library.tracks.get(id) {
            div()
                .h(height)
                .py_1()
                .border_b_1()
                .border_color(theme.white_05)
                .child(
                    div()
                        .id(format!("track_{:?}", track.id.0))
                        .size_full()
                        .flex()
                        .items_center()
                        .rounded_md()
                        .cursor_pointer()
                        .hover(|this| this.bg(theme.accent_10))
                        .when(is_current, |this| this.bg(theme.accent_15))
                        .on_click({
                            let id = *id;
                            move |_, _, cx| {
                                let controller = cx.global::<Controller>().clone();

                                controller.load_track(id, cx)
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
                                .child(format! {"{:02}", i}),
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
                                        img(image.clone())
                                            .object_fit(ObjectFit::Contain)
                                            .size_full()
                                            .rounded_sm(),
                                    ),
                                    None => div().size_11().flex_shrink_0(),
                                })
                                .when(is_current, |this| {
                                    this.text_color(theme.accent)
                                        .font_weight(FontWeight::MEDIUM)
                                })
                                .child(track.title.to_string())
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
                                .child(track.artist.to_string())
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
                                .child(track.album.to_string())
                                .overflow_hidden()
                                .whitespace_nowrap()
                                .text_ellipsis(),
                        )
                        .child(
                            div()
                                .w_24()
                                .max_w_24()
                                .h_full()
                                .px_4()
                                .flex()
                                .items_center()
                                .justify_start()
                                .text_sm()
                                .font_family("JetBrains Mono")
                                .child(format!(
                                    "{:02}:{:02}",
                                    track.duration / 60,
                                    track.duration % 60
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
    #[allow(clippy::too_many_lines)]
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>().clone();

        let controller = cx.global::<Controller>().clone();
        let state = controller.state.read(cx);
        let sidebar_scroll_handle = self.sidebar_scroll_handle.clone();
        let main_scroll_handle = self.main_scroll_handle.clone();

        let tracks_fp = fingerprint_tracks(state.library.tracks.keys().cloned());
        let playlists_fp = fingerprint_playlists(state.library.playlists.keys().cloned());

        let combined_fp = tracks_fp ^ playlists_fp;

        let rows = self.rows.clone();
        let heights = self.heights.clone();

        let playlists: Vec<_> = state.library.playlists.values().cloned().collect();
        let selected = self.selected_playlist.clone();
        let len = playlists.len();

        div()
            .size_full()
            .bg(theme.bg_main)
            .text_color(theme.text_primary)
            .flex()
            .child(
                div().w_1_3().h_full().flex().flex_col().gap_3()
                    .bg(theme.bg_queue)
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
                                    .text_color(theme.text_primary)
                                    .font_weight(FontWeight(500.0))
                                    .child("Playlists"),
                            ),
                    )
                    .child(
                        uniform_list("playlist_sidebar", len, move |range, _, cx| {
                            range.map(|i| {
                                let playlist = &playlists[i];

                                div()
                                    .id(format!("playlist_sidebar_{}", playlist.id.0))
                                    .px_4()
                                    .py_3()
                                    .cursor_pointer()
                                    .rounded_md()
                                    .hover(|d| d.bg(theme.accent_10))
                                    .when(
                                        Some(playlist.id) == *self.selected_playlist.read(cx),
                                        |d| d.bg(theme.accent_15),
                                    )
                                    .on_click({
                                        let id = playlist.id;
                                        move |_, _, cx| {
                                            selected.update(cx, |this, cx| {
                                                *this = Some(id);
                                                cx.notify();
                                            });
                                        }
                                    })
                                    .child(playlist.name.clone())
                            }).collect::<Vec<_>>()
                        })
                            .track_scroll(&sidebar_scroll_handle)
                    )
                    .child(floating_scrollbar(
                        "queue_scrollbar",
                        self.sidebar_scroll_handle.clone(),
                        RightPad::Pad,
                    ))
            )
            .child(
                div().w_full().h_full().flex().flex_grow().child(vlist(
                    cx.entity(),
                    "playlists_main",
                    heights.clone(),
                    main_scroll_handle,
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
                                PlaylistsRows::Header => Self::render_header(heights[idx], self.selected_playlist.read(cx).clone(), cx),

                                PlaylistsRows::TrackTableHeader => {
                                    Self::render_track_table_header(heights[idx], cx)
                                }

                                PlaylistsRows::TrackRow(i, id) => {
                                    Self::render_track(*i, id, heights[idx], cx)
                                }
                            })
                            .collect::<Vec<_>>()
                    },
                ))
                    .child(floating_scrollbar(
                        "queue_scrollbar",
                        self.main_scroll_handle.clone(),
                        RightPad::Pad,
                    ))
            )
    }
}

fn build_rows(library: &LibraryState) -> (Vec<PlaylistsRows>, Vec<Pixels>) {
    let mut rows = Vec::new();
    let mut heights = Vec::new();

    rows.push(PlaylistsRows::Header);
    heights.push(px(120.0));

    if !library.tracks.is_empty() {
        let mut sorted_tracks: Vec<_> = library.tracks.values().collect();

        sorted_tracks.sort_by(|a, b| a.title.cmp(&b.title));

        rows.push(PlaylistsRows::TrackTableHeader);
        heights.push(px(40.0));

        for (i, track) in sorted_tracks.iter().enumerate() {
            rows.push(PlaylistsRows::TrackRow(i + 1, track.id));
            heights.push(px(60.0));
        }
    }

    (rows, heights)
}
