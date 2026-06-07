mod helpers;

use crate::controller::Controller;
use crate::controller::state::TrackId;
use crate::ui::components::Page;
use crate::ui::components::image_cache::ImageCache;
use crate::ui::components::scrollbar::{RightPad, floating_scrollbar};
use crate::ui::helpers::{fingerprint_playlists, fingerprint_tracks};
use crate::ui::theme::Theme;
use gpui::prelude::FluentBuilder;
use gpui::{
    App, Context, Div, FontWeight, ImageSource, InteractiveElement, IntoElement, ObjectFit,
    ParentElement, Pixels, Render, ScrollHandle, StatefulInteractiveElement, Styled, StyledImage,
    VirtualListScrollController, Window, div, img, vlist,
};
use helpers::{
    HeaderKind, LibraryRow, build_rows, build_rows_from_db, render_header, render_playlist_grid,
    render_track_table_header,
};
use std::rc::Rc;

const THUMBNAIL_MARGIN: usize = 16;

#[derive(Clone)]
pub struct LibraryPage {
    scroll_handle: ScrollHandle,
    rows: Rc<Vec<LibraryRow>>,
    heights: Rc<Vec<Pixels>>,
    pub sorted_tracks: Vec<&'static TrackId>,
    grid_cols: usize,
    last_fp: u128,
    pub list_controller: VirtualListScrollController,
}

impl LibraryPage {
    pub fn new(cx: &mut App) -> Self {
        let scroll_handle = ScrollHandle::new();
        let library = &cx.global::<Controller>().state.read(cx).library;

        let cols = 4;

        let (rows, heights) = if !library.db_tracks.is_empty() {
            build_rows_from_db(&library.db_tracks, &library.playlists, cols)
        } else {
            build_rows(library, cols)
        };

        LibraryPage {
            scroll_handle,
            rows: Rc::new(rows),
            heights: Rc::new(heights),
            grid_cols: cols,
            sorted_tracks: Vec::new(),
            last_fp: 0,
            list_controller: VirtualListScrollController::new(),
        }
    }
    #[allow(clippy::too_many_lines)]
    fn render_track(i: usize, id: &TrackId, height: Pixels, cx: &mut App) -> Div {
        let controller = cx.global::<Controller>().clone();
        let theme = *cx.global::<Theme>();

        let (maybe_image_id, title, artist, album, duration) = {
            let state = controller.state.read(cx);
            if let Some(row) = state.library.db_index.get(id) {
                let image_id = row.image_hash.as_deref().and_then(|b| {
                    if b.len() == 16 {
                        let mut h = [0u8; 16];
                        h.copy_from_slice(&b[..16]);
                        Some(crate::controller::state::ImageId(h))
                    } else {
                        None
                    }
                });

                (
                    image_id,
                    row.name.clone(),
                    row.artists.clone(),
                    row.album.clone().unwrap_or_default(),
                    std::time::Duration::from_millis(row.duration_ms as u64),
                )
            } else if let Some(track) = state.library.tracks.get(id) {
                (
                    track.image_id,
                    track.title.clone(),
                    track.artist.clone(),
                    track.album.clone(),
                    track.duration,
                )
            } else {
                (
                    None,
                    String::new(),
                    String::new(),
                    String::new(),
                    std::time::Duration::from_secs(0),
                )
            }
        };

        let thumbnail = maybe_image_id.and_then(|id| cx.global_mut::<ImageCache>().get(&id));

        let state = controller.state.read(cx).clone();
        let is_current = Some(id) == state.playback.current.as_ref();

        div()
            .h(height)
            .py_1()
            .border_b_1()
            .border_color(theme.library_track_border)
            .child(
                div()
                    .id(format!("track_{:?}", id.0))
                    .size_full()
                    .flex()
                    .items_center()
                    .rounded_md()
                    .cursor_pointer()
                    .hover(|this| this.bg(theme.library_track_bg_hover))
                    .when(is_current, |this| this.bg(theme.library_track_bg_active))
                    .on_click({
                        let id = *id;
                        move |_, _, cx| {
                            let controller = cx.global::<Controller>().clone();

                            controller.load_track(id, cx);

                            *cx.global_mut::<Page>() = Page::Player;
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
                            .child(format! {"{i:02}"}),
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
                                        .border_1()
                                        .border_color(theme.border)
                                        .rounded_sm(),
                                ),
                                None => div().size_11().flex_shrink_0().child(
                                    img("icons/placeholder.svg")
                                        .object_fit(ObjectFit::Contain)
                                        .size_full()
                                        .border_1()
                                        .border_color(theme.border)
                                        .rounded_sm(),
                                ),
                            })
                            .when(is_current, |this| {
                                this.text_color(theme.library_track_title_text_active)
                                    .font_weight(FontWeight::MEDIUM)
                            })
                            .child(title)
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
                            .child(artist)
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
                            .child(album)
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
                                duration.as_secs() / 60,
                                duration.as_secs() % 60
                            ))
                            .overflow_hidden()
                            .whitespace_nowrap()
                            .text_ellipsis(),
                    ),
            )
    }
}

impl Render for LibraryPage {
    #[allow(clippy::too_many_lines)]
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();

        let controller = cx.global::<Controller>().clone();
        let state = controller.state.read(cx);
        let scroll_handle = self.scroll_handle.clone();

        let use_db = !state.library.db_tracks.is_empty();

        let width = window.bounds().size.width;
        let tile = 256.0;

        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let cols = ((width.to_f64() / tile) as usize).max(1);

        if cols != self.grid_cols || use_db != (self.last_fp != 0) {
            if use_db {
                let (rows, heights) =
                    build_rows_from_db(&state.library.db_tracks, &state.library.playlists, cols);
                self.rows = Rc::new(rows);
                self.heights = Rc::new(heights);
                self.last_fp = 1;
                self.grid_cols = cols;
            } else {
                let library = &state.library;
                let (rows, heights) = build_rows(library, cols);
                self.rows = Rc::new(rows);
                self.heights = Rc::new(heights);
                self.last_fp = 0;
                self.grid_cols = cols;
            }
        }

        let rows = self.rows.clone();
        let heights = self.heights.clone();

        div()
            .size_full()
            .bg(theme.library_bg)
            .text_color(theme.library_text)
            .px_12()
            .pt_10()
            .child(vlist(
                cx.entity(),
                "library",
                heights.clone(),
                scroll_handle,
                self.list_controller.clone(),
                move |_this, range, _, cx| {
                    let len = rows.len();

                    let start = range.start.saturating_sub(THUMBNAIL_MARGIN);
                    let end = (range.end + THUMBNAIL_MARGIN).min(len);

                    let thumb_track_ids: Vec<TrackId> = (start..end)
                        .filter_map(|idx| match &rows[idx] {
                            LibraryRow::TrackRow(_, id) => Some(*id),
                            _ => None,
                        })
                        .collect();

                    controller.request_track_thumbnails(&thumb_track_ids, cx);

                    range
                        .map(|idx| match &rows[idx] {
                            LibraryRow::Header(kind) => render_header(kind, heights[idx], cx),

                            LibraryRow::PlaylistGridRow(ids) => {
                                render_playlist_grid(ids, heights[idx], cx)
                            }

                            LibraryRow::TrackTableHeader => {
                                render_track_table_header(heights[idx], cx)
                            }

                            LibraryRow::TrackRow(i, id) => {
                                Self::render_track(*i, id, heights[idx], cx)
                            }

                            LibraryRow::Empty(kind) => match kind {
                                HeaderKind::Playlists => div()
                                    .w_full()
                                    .h_48()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .text_lg()
                                    .text_color(theme.library_empty_text)
                                    .child("No playlists loaded."),
                                HeaderKind::Tracks => div()
                                    .w_full()
                                    .h_48()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .text_lg()
                                    .text_color(theme.library_empty_text)
                                    .child("No tracks loaded."),
                                HeaderKind::Albums => div()
                                    .w_full()
                                    .h_48()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .text_lg()
                                    .text_color(theme.library_empty_text)
                                    .child("No albums loaded."),
                            },
                        })
                        .collect::<Vec<_>>()
                },
            ))
            .child(floating_scrollbar(
                "queue_scrollbar",
                self.scroll_handle.clone(),
                RightPad::Pad,
            ))
    }
}
