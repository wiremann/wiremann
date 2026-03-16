use std::rc::Rc;

use crate::{
    controller::Controller,
    ui::theme::Theme,
};

use crate::controller::state::LibraryState;
use crate::library::playlists::PlaylistId;
use crate::library::TrackId;
use crate::ui::components::image_cache::ImageCache;
use crate::ui::components::scrollbar::{floating_scrollbar, RightPad};
use crate::ui::components::virtual_list::vlist;
use gpui::{div, img, px, white, App, AppContext, Context, Div, Entity, FontWeight, IntoElement, ObjectFit, ParentElement, Pixels, Render, ScrollHandle, Styled, StyledImage, Window};

#[derive(Clone)]
pub struct LibraryPage {
    scroll_handle: ScrollHandle,
    show_playlists: Entity<bool>,
    rows: Rc<Vec<LibraryRow>>,
    heights: Rc<Vec<Pixels>>,
    grid_cols: usize,
}

#[derive(Clone)]
enum HeaderKind {
    Playlists,
    Tracks,
    Albums,
}

#[derive(Clone)]
enum LibraryRow {
    Header(HeaderKind),
    PlaylistGridRow(Vec<PlaylistId>),
    TrackRow(TrackId),
}

impl LibraryPage {
    pub fn new(cx: &mut App) -> Self {
        let scroll_handle = ScrollHandle::new();
        let show_playlists = cx.new(|_| true);

        let library = &cx.global::<Controller>().state.read(cx).library;

        let cols = 4;

        let (rows, heights) = build_rows(library, cols);

        LibraryPage {
            scroll_handle,
            show_playlists,
            rows: Rc::new(rows),
            heights: Rc::new(heights),
            grid_cols: cols,
        }
    }

    fn render_header(kind: &HeaderKind, height: Pixels, cx: &App) -> Div {
        let text = match kind {
            HeaderKind::Playlists => "Playlists",
            HeaderKind::Tracks => "Tracks",
            HeaderKind::Albums => "Albums",
        };

        let theme = cx.global::<Theme>();

        div()
            .h(height)
            .w_full()
            .flex()
            .items_center()
            .py_4()
            .px_6()
            .text_lg()
            .font_weight(FontWeight::MEDIUM)
            .text_color(theme.text_primary)
            .child(text)
    }

    fn render_playlist_grid(ids: &Vec<PlaylistId>, height: Pixels, cx: &mut App) -> Div {
        let controller = cx.global::<Controller>().clone();

        div()
            .h(height)
            .flex()
            .gap_10()
            .px_6()
            .items_center()
            .children({
                let state = controller.state.read(cx).clone();

                controller.request_playlist_thumbnails(&ids, cx);

                let cache = cx.global_mut::<ImageCache>();

                let mut elements = Vec::new();

                for pid in ids {
                    if let Some(playlist) = state.library.playlists.get(pid) {
                        let thumbnail =
                            playlist.image_id.and_then(|id| cache.get(&id));

                        let el = div()
                            .h_auto()
                            .w_auto()
                            .flex()
                            .flex_col()
                            .items_center()
                            .justify_center()
                            .gap_y_4()
                            .text_color(white())
                            .child(match thumbnail {
                                Some(image) => div().size_64().flex_shrink_0().child(
                                    img(image.clone())
                                        .object_fit(ObjectFit::Contain)
                                        .size_full()
                                        .rounded_lg(),
                                ),
                                None => div().size_64().flex_shrink_0(),
                            })
                            .child(playlist.name.clone());

                        elements.push(el);
                    }
                }

                elements
            })
    }
}

impl Render for LibraryPage {
    #[allow(clippy::too_many_lines)]
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>();

        let controller = cx.global::<Controller>().clone();
        let state = controller.state.read(cx);
        let scroll_handle = self.scroll_handle.clone();

        let width = window.bounds().size.width;
        let tile = 256.0;

        let cols = ((width.to_f64() / tile) as usize).max(1);

        if cols != self.grid_cols {
            let library = &state.library;

            let (rows, heights) = build_rows(library, cols);

            self.rows = Rc::new(rows);
            self.heights = Rc::new(heights);
            self.grid_cols = cols;
        }

        let _show_playlists = self.show_playlists.clone();

        let _current = if let Some(id) = state.playback.current {
            state.library.tracks.get(&id)
        } else {
            None
        };

        let rows = self.rows.clone();
        let heights = self.heights.clone();

        div()
            .size_full()
            .bg(theme.bg_main)
            .text_color(theme.text_primary)
            .p_4()
            .child(
                vlist(cx.entity(), "library", heights.clone(), scroll_handle, move |_this, range, _, cx| {
                    range
                        .map(|i| {
                            match &rows[i] {
                                LibraryRow::Header(kind) => Self::render_header(kind, heights[i], cx),

                                LibraryRow::PlaylistGridRow(ids) => Self::render_playlist_grid(ids, heights[i], cx),

                                LibraryRow::TrackRow(id) => {
                                    let controller = cx.global::<Controller>().clone();
                                    let track = controller.state.read(cx)
                                        .library
                                        .tracks
                                        .get(id)
                                        .unwrap();

                                    div()
                                        .h(heights[i])
                                        .flex()
                                        .items_center()
                                        .gap(px(10.0))
                                        .pl(px(12.0))
                                        .child(track.title.clone())
                                }
                            }
                        })
                        .collect::<Vec<_>>()
                })
            )
            .child(floating_scrollbar(
                "queue_scrollbar",
                self.scroll_handle.clone(),
                RightPad::Pad,
            ))
    }
}

fn build_rows(
    library: &LibraryState,
    cols: usize,
) -> (Vec<LibraryRow>, Vec<Pixels>) {
    let mut rows = Vec::new();
    let mut heights = Vec::new();

    if !library.playlists.is_empty() {
        rows.push(LibraryRow::Header(HeaderKind::Playlists));
        heights.push(px(40.0));

        let mut chunk = Vec::with_capacity(cols);

        for pid in library.playlists.keys() {
            chunk.push(*pid);

            if chunk.len() == cols {
                rows.push(LibraryRow::PlaylistGridRow(chunk));
                heights.push(px(160.0));
                chunk = Vec::with_capacity(cols);
            }
        }

        if !chunk.is_empty() {
            rows.push(LibraryRow::PlaylistGridRow(chunk));
            heights.push(px(160.0));
        }
    }

    if !library.tracks.is_empty() {
        rows.push(LibraryRow::Header(HeaderKind::Tracks));
        heights.push(px(40.0));

        for id in library.tracks.keys() {
            rows.push(LibraryRow::TrackRow(*id));
            heights.push(px(32.0));
        }
    }

    (rows, heights)
}

