mod helpers;
pub mod models;

use crate::controller::Controller;
use crate::controller::state::TrackId;
use crate::db::Database;
use crate::ui::components::Page;
use crate::ui::components::image_cache::ImageCache;
use crate::ui::components::scrollbar::{RightPad, floating_scrollbar};
use crate::ui::pages::library::helpers::build_initial_rows;
use crate::ui::pages::library::models::{LibraryPlaylistRow, LibraryTrackRow};
use crate::ui::theme::Theme;
use gpui::prelude::FluentBuilder;
use gpui::{
    App, Context, Div, FontWeight, ImageSource, InteractiveElement, IntoElement, ObjectFit,
    ParentElement, Pixels, Render, ScrollHandle, StatefulInteractiveElement, Styled, StyledImage,
    VirtualListScrollController, Window, div, img, px, vlist,
};
use helpers::{
    HeaderKind, LibraryRow, render_header, render_playlist_grid, render_track_table_header,
};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

const THUMBNAIL_MARGIN: usize = 16;
const TRACK_PAGE_SIZE: usize = 128;
const TRACK_PREFETCH_PAGES: usize = 2;

#[derive(Clone)]
pub struct LibraryPage {
    scroll_handle: ScrollHandle,
    list_controller: VirtualListScrollController,

    playlists: Vec<LibraryPlaylistRow>,

    track_pages: HashMap<usize, Vec<LibraryTrackRow>>,

    loaded_pages: HashSet<usize>,
    loading_pages: HashSet<usize>,

    total_track_count: usize,

    rows: Rc<Vec<LibraryRow>>,
    heights: Rc<Vec<Pixels>>,

    grid_cols: usize,
}

impl LibraryPage {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let scroll_handle = ScrollHandle::new();

        let cols = 4;

        let (rows, heights) = build_initial_rows(0, 0, cols, &[]);
        let db = cx.global::<Database>().clone();

        cx.spawn(async move |this, cx| {
            let (playlists, total_tracks) = smol::unblock(move || {
                let conn = db.pool().get()?;

                let playlists = crate::db::queries::library::get_library_playlists(&conn)?;

                let total_tracks = crate::db::queries::library::get_total_track_count(&conn)?;

                anyhow::Ok((playlists, total_tracks))
            })
            .await
            .unwrap();

            this.update(cx, |view, cx| {
                view.playlists = playlists;

                view.total_track_count = total_tracks;

                let (rows, heights) = build_initial_rows(
                    view.playlists.len(),
                    total_tracks,
                    view.grid_cols,
                    &view.playlists,
                );

                view.rows = Rc::new(rows);
                view.heights = Rc::new(heights);

                for page in 0..=TRACK_PREFETCH_PAGES {
                    view.request_page(page, cx);
                }

                cx.notify();
            })
            .ok();
        })
        .detach();
        LibraryPage {
            scroll_handle,
            rows: Rc::new(rows),
            heights: Rc::new(heights),
            grid_cols: cols,
            list_controller: VirtualListScrollController::new(),
            playlists: Vec::new(),
            track_pages: HashMap::new(),
            loaded_pages: HashSet::new(),
            loading_pages: HashSet::new(),
            total_track_count: 0,
        }
    }

    fn patch_loaded_page(&mut self, page: usize) {
        let Some(page_rows) = self.track_pages.get(&page) else {
            return;
        };

        let rows = Rc::make_mut(&mut self.rows);

        let base_index = page * TRACK_PAGE_SIZE;

        let track_row_start = rows
            .iter()
            .position(|r| matches!(r, LibraryRow::TrackTableHeader));

        let Some(track_header_idx) = track_row_start else {
            return;
        };

        let first_track_row = track_header_idx + 1;

        for offset in 0..page_rows.len() {
            let absolute = base_index + offset;

            let row_index = first_track_row + absolute;

            if row_index >= rows.len() {
                break;
            }

            rows[row_index] = LibraryRow::LoadedTrack {
                absolute_index: absolute,
                page,
                offset,
            };
        }
    }

    fn request_page(&mut self, page: usize, cx: &mut Context<Self>) {
        if self.loaded_pages.contains(&page) || self.loading_pages.contains(&page) {
            return;
        }

        self.loading_pages.insert(page);

        let db = cx.global::<Database>().clone();

        cx.spawn(async move |this, cx| {
            let offset = page * TRACK_PAGE_SIZE;

            let rows = smol::unblock(move || {
                let conn = db.pool().get()?;

                crate::db::queries::library::get_tracks_page(
                    &conn,
                    TRACK_PAGE_SIZE as u64,
                    offset as u64,
                )
            })
            .await
            .unwrap();

            this.update(cx, |view, cx| {
                view.track_pages.insert(page, rows);

                view.loaded_pages.insert(page);

                view.loading_pages.remove(&page);

                view.patch_loaded_page(page);

                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    pub fn append_committed_rows(
        &mut self,
        new_rows: Vec<LibraryTrackRow>,
        cx: &mut Context<Self>,
    ) {
        if new_rows.is_empty() {
            return;
        }

        let old_total = self.total_track_count;
        let added = new_rows.len();

        // Determine where track rows begin using an immutable reference first
        let rows_ref = self.rows.clone();

        // Find TrackTableHeader index if present
        let track_header_idx_opt = rows_ref
            .iter()
            .position(|r| matches!(r, LibraryRow::TrackTableHeader));

        let first_track_row_idx: usize;
        let insert_pos: usize;

        if let Some(track_header_idx) = track_header_idx_opt {
            first_track_row_idx = track_header_idx + 1;
            insert_pos = first_track_row_idx + old_total;
        } else {
            // If no TrackTableHeader, find Empty(HeaderKind::Tracks)
            if let Some(empty_idx) = rows_ref
                .iter()
                .position(|r| matches!(r, LibraryRow::Empty(HeaderKind::Tracks)))
            {
                // We'll replace the empty slot after obtaining mutable access
                first_track_row_idx = empty_idx + 1;
                insert_pos = first_track_row_idx;
            } else {
                // Fallback: append at end
                first_track_row_idx = Self::first_track_row_index(&rows_ref);
                insert_pos = rows_ref.len();
            }
        }

        // Now obtain mutable access to rows/heights and apply structural changes
        let rows_mut = Rc::make_mut(&mut self.rows);
        let heights_mut = Rc::make_mut(&mut self.heights);

        // If we decided to replace an Empty slot with TrackTableHeader, do it now
        if track_header_idx_opt.is_none() {
            if insert_pos > 0 && insert_pos - 1 < rows_mut.len() {
                // Check if the slot we targeted is Empty(HeaderKind::Tracks) and replace it
                let header_candidate_idx = insert_pos - 1;
                if matches!(
                    rows_mut[header_candidate_idx],
                    LibraryRow::Empty(HeaderKind::Tracks)
                ) {
                    rows_mut[header_candidate_idx] = LibraryRow::TrackTableHeader;
                    heights_mut[header_candidate_idx] = px(40.0);
                }
            }
        }

        // Insert matching placeholders and heights at insert_pos
        for i in 0..added {
            let pos = insert_pos + i;
            rows_mut.insert(pos, helpers::LibraryRow::PlaceholderTrack);
            heights_mut.insert(pos, px(60.0));
        }

        // Update total
        self.total_track_count = old_total + added;

        // Populate page cache for loaded pages only and patch loaded rows
        for (i, track) in new_rows.into_iter().enumerate() {
            let absolute = old_total + i;
            let page = absolute / TRACK_PAGE_SIZE;
            let offset = absolute % TRACK_PAGE_SIZE;

            if self.loaded_pages.contains(&page) {
                let page_vec = self.track_pages.entry(page).or_insert_with(|| Vec::new());

                if page_vec.len() <= offset {
                    page_vec.resize(
                        offset + 1,
                        LibraryTrackRow {
                            id: TrackId::default(),
                            title: "".into(),
                            artists: "".into(),
                            album: "".into(),
                            duration_ms: 0,
                            image_id: None,
                        },
                    );
                }

                page_vec[offset] = track.clone();

                let row_index = first_track_row_idx + absolute;
                if row_index < rows_mut.len() {
                    rows_mut[row_index] = helpers::LibraryRow::LoadedTrack {
                        absolute_index: absolute,
                        page,
                        offset,
                    };
                }
            }
        }

        cx.notify();
    }
}

impl LibraryPage {
    fn first_track_row_index(rows: &Rc<Vec<helpers::LibraryRow>>) -> usize {
        rows.iter()
            .position(|r| matches!(r, helpers::LibraryRow::TrackTableHeader))
            .map(|i| i + 1)
            .unwrap_or(rows.len())
    }

    fn render_loaded_track(
        absolute_index: usize,
        track: &models::LibraryTrackRow,
        height: Pixels,
        cx: &mut Context<Self>,
    ) -> Div {
        let theme = *cx.global::<Theme>();

        let mins = (track.duration_ms / 1000) / 60;
        let secs = (track.duration_ms / 1000) % 60;

        div()
            .h(height)
            .w_full()
            .border_b_1()
            .border_color(theme.library_track_border)
            .child(
                div()
                    .w_20()
                    .h_full()
                    .items_center()
                    .child(format!("{}", absolute_index + 1)),
            )
            .child(
                div()
                    .w_3_5()
                    .h_full()
                    .items_center()
                    .child(track.title.clone()),
            )
            .child(
                div()
                    .w_1_2()
                    .h_full()
                    .items_center()
                    .child(track.artists.clone()),
            )
            .child(
                div()
                    .w_1_2()
                    .h_full()
                    .items_center()
                    .child(track.album.clone()),
            )
            .child(
                div()
                    .w_24()
                    .h_full()
                    .items_center()
                    .child(format!("{:02}:{:02}", mins, secs)),
            )
    }
}

impl Render for LibraryPage {
    #[allow(clippy::too_many_lines)]
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();

        let controller = cx.global::<Controller>().clone();
        let _state = controller.state.read(cx);
        let scroll_handle = self.scroll_handle.clone();

        let width = window.bounds().size.width;
        let tile = 256.0;

        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let cols = ((width.to_f64() / tile) as usize).max(1);

        if cols != self.grid_cols {
            let (rows, heights) = build_initial_rows(
                self.playlists.len(),
                self.total_track_count,
                cols,
                &self.playlists,
            );

            self.rows = Rc::new(rows);
            self.heights = Rc::new(heights);
            self.grid_cols = cols;

            let loaded_pages: Vec<_> = self.loaded_pages.iter().copied().collect();

            for page in loaded_pages {
                self.patch_loaded_page(page);
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
                    let first_track_row = Self::first_track_row_index(&rows);

                    let visible_track_index = range.start.saturating_sub(first_track_row);

                    let first_page = visible_track_index / TRACK_PAGE_SIZE;

                    for page in first_page.saturating_sub(1)..=(first_page + TRACK_PREFETCH_PAGES) {
                        _this.request_page(page, cx);
                    }

                    let mut visible_track_ids = Vec::new();

                    for idx in start..end {
                        if let LibraryRow::LoadedTrack { page, offset, .. } = &rows[idx] {
                            if let Some(page_rows) = _this.track_pages.get(page) {
                                if let Some(track) = page_rows.get(*offset) {
                                    visible_track_ids.push(track.id);
                                }
                            }
                        }
                    }

                    controller.request_track_thumbnails(&visible_track_ids, cx);

                    range
                        .map(|idx| match &rows[idx] {
                            LibraryRow::Header(kind) => render_header(kind, heights[idx], cx),

                            LibraryRow::PlaylistGridRow(playlists) => {
                                render_playlist_grid(playlists, heights[idx], cx)
                            }

                            LibraryRow::TrackTableHeader => {
                                render_track_table_header(heights[idx], cx)
                            }

                            LibraryRow::LoadedTrack {
                                absolute_index,
                                page,
                                offset,
                            } => {
                                if let Some(page_rows) = _this.track_pages.get(page) {
                                    if let Some(track) = page_rows.get(*offset) {
                                        Self::render_loaded_track(
                                            *absolute_index,
                                            track,
                                            heights[idx],
                                            cx,
                                        )
                                    } else {
                                        div().h(heights[idx])
                                    }
                                } else {
                                    div()
                                        .h(heights[idx])
                                        .w_full()
                                        .border_b_1()
                                        .border_color(theme.library_track_border)
                                }
                            }

                            LibraryRow::PlaceholderTrack => div()
                                .h(heights[idx])
                                .w_full()
                                .border_b_1()
                                .border_color(theme.library_track_border),
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
