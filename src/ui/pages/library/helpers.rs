use crate::controller::Controller;
use crate::controller::state::LibraryState;
use crate::controller::state::PlaylistId;
use crate::controller::state::TrackId;
use crate::ui::components::Page;
use crate::ui::components::image_cache::ImageCache;
use crate::ui::theme::Theme;
use gpui::prelude::FluentBuilder;
use gpui::{
    App, Div, FontWeight, ImageSource, InteractiveElement, ObjectFit, ParentElement, Pixels,
    StatefulInteractiveElement, Styled, StyledImage, div, img, px,
};

#[allow(dead_code)]
#[derive(Clone, PartialEq)]
pub(super) enum HeaderKind {
    Playlists,
    Tracks,
    Albums,
}

#[allow(dead_code)]
pub(super) enum LibraryRow {
    Header(HeaderKind),
    PlaylistGridRow(Vec<PlaylistId>),
    TrackTableHeader,
    TrackRow(usize, TrackId),
    Empty(HeaderKind),
}

pub(super) fn render_header(kind: &HeaderKind, height: Pixels, cx: &App) -> Div {
    let heading = match kind {
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
        .justify_between()
        .text_lg()
        .font_weight(FontWeight::MEDIUM)
        .text_color(theme.library_header_text)
        .child(heading)
        .child(if *kind == HeaderKind::Playlists {
            div()
                .id("create_playlist")
                .flex()
                .items_center()
                .justify_center()
                .gap_2()
                .px_4()
                .py_1()
                .rounded_lg()
                .border_1()
                .border_color(theme.library_header_button_border)
                .text_color(theme.library_header_button_text)
                .text_base()
                .cursor_pointer()
                .hover(|this| this.bg(theme.library_header_button_bg_hover))
                .on_click(move |_, _, cx| {
                    let controller = cx.global::<Controller>().clone();
                    cx.spawn(async move |_| {
                        if let Some(folder) = rfd::AsyncFileDialog::new().pick_folder().await {
                            controller.scan_dir(folder.path().into());
                        }
                    })
                    .detach();
                })
                .child("Open Folder")
        } else if *kind == HeaderKind::Tracks {
            div()
                .id("add_track")
                .flex()
                .items_center()
                .justify_center()
                .gap_2()
                .px_4()
                .py_1()
                .rounded_lg()
                .border_1()
                .border_color(theme.library_header_button_border)
                .text_base()
                .text_color(theme.library_header_button_text)
                .cursor_pointer()
                .hover(|this| this.bg(theme.library_header_button_bg_hover))
                .on_click(move |_, _, cx| {
                    let controller = cx.global::<Controller>().clone();
                    cx.spawn(async move |_| {
                        if let Some(files) = rfd::AsyncFileDialog::new().pick_files().await {
                            for file in files {
                                controller.scan_track(file.path().into());
                            }
                        }
                    })
                    .detach();
                })
                .child("Add Track")
        } else {
            div().id("")
        })
}

pub(super) fn render_playlist_grid(ids: &Vec<PlaylistId>, height: Pixels, cx: &mut App) -> Div {
    let controller = cx.global::<Controller>().clone();
    let theme = *cx.global::<Theme>();

    div()
        .h(height)
        .flex()
        .gap_8()
        .py_2()
        .items_center()
        .children({
            let state = controller.state.read(cx).clone();

            controller.request_playlist_thumbnails(ids, cx);

            let cache = cx.global_mut::<ImageCache>();

            let mut elements = Vec::new();

            for pid in ids {
                if let Some(playlist) = state.library.playlists.get(pid) {
                    let thumbnail = playlist.image_id.and_then(|id| cache.get(&id));

                    let el = div()
                        .id(format!("playlist_{}", playlist.id.0))
                        .bg(theme.library_playlist_bg)
                        .size_full()
                        .max_w_64()
                        .flex()
                        .flex_col()
                        .items_start()
                        .justify_center()
                        .text_color(theme.library_playlist_text)
                        .p_3()
                        .rounded_lg()
                        .hover(|this| this.bg(theme.library_playlist_bg_hover))
                        .cursor_pointer()
                        .on_click({
                            let id = playlist.id;
                            move |_, _, cx| {
                                let controller = cx.global::<Controller>().clone();

                                controller.load_playlist(id, cx);
                                *cx.global_mut::<Page>() = Page::Player;
                            }
                        })
                        .when(
                            state.playback.current_playlist == Some(playlist.id),
                            |this| this.bg(theme.library_playlist_bg_active),
                        )
                        .child(match thumbnail {
                            Some(image) => div().size_full().mb_3().child(
                                img(ImageSource::Render(image.clone()))
                                    .object_fit(ObjectFit::Contain)
                                    .border_1()
                                    .border_color(theme.border)
                                    .size_full()
                                    .rounded_lg(),
                            ),
                            None => div().size_full().mb_3().child(
                                img("icons/placeholder.svg")
                                    .object_fit(ObjectFit::Contain)
                                    .border_1()
                                    .border_color(theme.border)
                                    .size_full()
                                    .rounded_lg(),
                            ),
                        })
                        .child(
                            div()
                                .text_base()
                                .text_color(theme.library_playlist_title_text)
                                .font_weight(FontWeight::MEDIUM)
                                .child(playlist.name.clone()),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(theme.library_playlist_meta_text)
                                .font_weight(FontWeight::MEDIUM)
                                .child(format!("{} tracks", playlist.tracks.len())),
                        );

                    elements.push(el);
                }
            }

            elements
        })
}

pub(super) fn render_track_table_header(height: Pixels, cx: &mut App) -> Div {
    let theme = cx.global::<Theme>();

    div()
        .h(height)
        .w_full()
        .flex()
        .items_center()
        .text_xs()
        .font_weight(FontWeight::NORMAL)
        .text_color(theme.library_table_header_text)
        .border_b_1()
        .border_color(theme.library_table_border)
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

pub(super) fn build_rows(library: &LibraryState, cols: usize) -> (Vec<LibraryRow>, Vec<Pixels>) {
    let mut rows = Vec::new();
    let mut heights = Vec::new();

    rows.push(LibraryRow::Header(HeaderKind::Playlists));
    heights.push(px(60.0));

    if library.playlists.is_empty() {
        rows.push(LibraryRow::Empty(HeaderKind::Playlists));
        heights.push(px(192.0));
    } else {
        let mut chunk = Vec::with_capacity(cols);

        for pid in library.playlists.keys() {
            chunk.push(*pid);

            if chunk.len() == cols {
                rows.push(LibraryRow::PlaylistGridRow(chunk));
                heights.push(px(280.0));
                chunk = Vec::with_capacity(cols);
            }
        }

        if !chunk.is_empty() {
            rows.push(LibraryRow::PlaylistGridRow(chunk));
            heights.push(px(280.0));
        }
    }
    rows.push(LibraryRow::Header(HeaderKind::Tracks));
    heights.push(px(60.0));

    if library.tracks.is_empty() {
        rows.push(LibraryRow::Empty(HeaderKind::Tracks));
        heights.push(px(192.0));
    } else {
        let mut sorted_tracks: Vec<_> = library.tracks.values().collect();

        sorted_tracks.sort_by(|a, b| a.title.cmp(&b.title));

        rows.push(LibraryRow::TrackTableHeader);
        heights.push(px(40.0));

        for (i, track) in sorted_tracks.iter().enumerate() {
            rows.push(LibraryRow::TrackRow(i + 1, track.id));
            heights.push(px(60.0));
        }
    }
    (rows, heights)
}
