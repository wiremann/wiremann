use crate::controller::state::LibraryState;
use crate::controller::Controller;
use crate::library::TrackId;
use crate::ui::components::scrollbar::{floating_scrollbar, RightPad};
use crate::ui::components::virtual_list::vlist;
use crate::ui::helpers::{fingerprint_playlists, fingerprint_tracks};
use crate::ui::theme::Theme;
use gpui::{div, px, Context, IntoElement, ParentElement, Pixels, Render, ScrollHandle, Styled, UniformListScrollHandle, Window};
use std::rc::Rc;

const THUMBNAIL_MARGIN: usize = 16;

#[derive(Clone)]
pub struct PlaylistsPage {
    sidebar_scroll_handle: UniformListScrollHandle,
    main_scroll_handle: ScrollHandle,

    rows: Rc<Vec<PlaylistsRows>>,
    heights: Rc<Vec<Pixels>>,
    last_fp: u128,
}

#[derive(Clone)]
enum PlaylistsRows {
    Header,
    TrackTableHeader,
    TrackRow(usize, TrackId),
    Empty,
}

impl PlaylistsPage {
    pub fn new() -> Self {
        let sidebar_scroll_handle = UniformListScrollHandle::new();
        let main_scroll_handle = ScrollHandle::new();

        PlaylistsPage {
            sidebar_scroll_handle,
            main_scroll_handle,
            rows: Rc::new(Vec::new()),
            heights: Rc::new(Vec::new()),
            last_fp: 0,
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

        let width = window.bounds().size.width;
        let tile = 256.0;

        let rows = self.rows.clone();
        let heights = self.heights.clone();

        div()
            .size_full()
            .bg(theme.bg_main)
            .text_color(theme.text_primary)
            .px_12()
            .py_10()
            .child(vlist(
                cx.entity(),
                "library",
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
                            PlaylistsRows::Header => Self::render_header(heights[idx], cx),

                            PlaylistsRows::TrackTableHeader => {
                                Self::render_track_table_header(heights[idx], cx)
                            }

                            PlaylistsRows::TrackRow(i, id) => {
                                Self::render_track(*i, id, heights[idx], cx)
                            }

                            PlaylistsRows::Empty => match kind {
                                crate::ui::components::pages::library::HeaderKind::Playlists => div()
                                    .w_full()
                                    .h_48()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .text_lg()
                                    .text_color(theme.text_muted)
                                    .child("No playlists loaded."),
                                crate::ui::components::pages::library::HeaderKind::Tracks => div()
                                    .w_full()
                                    .h_48()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .text_lg()
                                    .text_color(theme.text_muted)
                                    .child("No tracks loaded."),
                                crate::ui::components::pages::library::HeaderKind::Albums => div()
                                    .w_full()
                                    .h_48()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .text_lg()
                                    .text_color(theme.text_muted)
                                    .child("No albums loaded."),
                            },
                        })
                        .collect::<Vec<_>>()
                },
            ))
            .child(floating_scrollbar(
                "queue_scrollbar",
                self.main_scroll_handle.clone(),
                RightPad::Pad,
            ))
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
