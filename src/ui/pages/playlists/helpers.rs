use crate::controller::Controller;
use crate::controller::state::LibraryState;
use crate::controller::state::PlaylistId;
use crate::controller::state::TrackId;
use crate::ui::components::Page;
use crate::ui::components::icons::{Icon, Icons};
use crate::ui::components::image_cache::ImageCache;
use crate::ui::theme::Theme;
use gpui::{
    App, Div, FontWeight, ImageSource, InteractiveElement, ObjectFit, ParentElement, Pixels,
    StatefulInteractiveElement, Styled, StyledImage, div, img, px, rems,
};

pub(super) enum PlaylistsRows {
    Header,
    TrackTableHeader,
    TrackRow(usize, TrackId),
}

pub(super) fn render_header(height: Pixels, id: Option<PlaylistId>, cx: &mut App) -> Div {
    let theme = *cx.global::<Theme>();
    let controller = cx.global::<Controller>().clone();

    let state = controller.state.read(cx).clone();

    if let Some(id) = id
        && let Some(playlist) = state.library.playlists.get(&id)
    {
        controller.request_playlist_thumbnails(&[id], cx);

        let cache = cx.global_mut::<ImageCache>();
        let thumbnail = playlist.image_id.and_then(|id| cache.get(&id));

        div()
            .flex()
            .w_full()
            .h(height)
            .child(
                div().size(height).p_6().child(match thumbnail {
                    Some(image) => div().size_full().child(
                        img(ImageSource::Render(image.clone()))
                            .object_fit(ObjectFit::Contain)
                            .size_full()
                            .rounded_lg(),
                    ),
                    None => div().size(height).flex_shrink_0(),
                }),
            )
            .child(
                div()
                    .w_full()
                    .h(height)
                    .flex()
                    .flex_col()
                    .justify_end()
                    .px_2()
                    .py_4()
                    .child(
                        div()
                            .text_size(rems(3.2))
                            .font_weight(FontWeight::BLACK)
                            .truncate()
                            .text_ellipsis()
                            .text_color(theme.playlist_header_title)
                            .child(playlist.name.to_string()),
                    )
                    .child(
                        div()
                            .text_base()
                            .text_color(theme.playlist_header_meta)
                            .child(format!("{} tracks", playlist.tracks.len())),
                    )
                    .child(
                        div()
                            .flex()
                            .gap_x_5()
                            .my_2()
                            .child(
                                div()
                                    .id("play_playlist")
                                    .py_1()
                                    .px_4()
                                    .text_base()
                                    .text_color(theme.playlist_header_button_text)
                                    .bg(theme.playlist_header_button_bg)
                                    .border_2()
                                    .border_color(theme.playlist_header_button_border)
                                    .rounded_md()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .gap_3()
                                    .child(Icon::new(Icons::Play).size_4())
                                    .child("Play")
                                    .cursor_pointer()
                                    .hover(|this| this.bg(theme.playlist_header_button_hover))
                                    .on_click({
                                        let id = playlist.id;
                                        move |_, _, cx| {
                                            let controller = cx.global::<Controller>().clone();
                                            controller.load_playlist(id, cx);
                                            *cx.global_mut::<Page>() = Page::Player;
                                        }
                                    }),
                            )
                            .child(
                                div()
                                    .id("shuffle_play_playlist")
                                    .py_1()
                                    .px_4()
                                    .text_base()
                                    .text_color(theme.playlist_header_button_text)
                                    .bg(theme.playlist_header_button_bg)
                                    .border_2()
                                    .border_color(theme.playlist_header_button_border)
                                    .rounded_md()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .gap_3()
                                    .child(Icon::new(Icons::Shuffle).size_4())
                                    .child("Shuffle Play")
                                    .cursor_pointer()
                                    .hover(|this| this.bg(theme.playlist_header_button_hover))
                                    .on_click({
                                        let id = playlist.id;
                                        move |_, _, cx| {
                                            let controller = cx.global::<Controller>().clone();
                                            controller.load_playlist(id, cx);
                                            controller.set_shuffle(cx);
                                            *cx.global_mut::<Page>() = Page::Player;
                                        }
                                    }),
                            ),
                    ),
            )
    } else {
        div()
    }
}

pub(super) fn render_track_table_header(height: Pixels, cx: &mut App) -> Div {
    let theme = cx.global::<Theme>();

    div()
        .h(height)
        .w_full()
        .flex()
        .px_3()
        .items_center()
        .text_xs()
        .font_weight(FontWeight::NORMAL)
        .text_color(theme.playlist_table_header_text)
        .border_b_1()
        .border_color(theme.playlist_table_header_border)
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

pub(super) fn build_rows(
    library: &LibraryState,
    selected: Option<PlaylistId>,
) -> (Vec<PlaylistsRows>, Vec<Pixels>) {
    let mut rows = Vec::new();
    let mut heights = Vec::new();

    rows.push(PlaylistsRows::Header);
    heights.push(px(240.0));

    if let Some(pid) = selected
        && let Some(playlist) = library.playlists.get(&pid)
        && !playlist.tracks.is_empty()
    {
        let mut tracks: Vec<_> = playlist
            .tracks
            .iter()
            .filter_map(|id| library.tracks.get(id))
            .collect();

        tracks.sort_by(|a, b| a.title.cmp(&b.title));

        rows.push(PlaylistsRows::TrackTableHeader);
        heights.push(px(40.0));

        for (i, track) in tracks.iter().enumerate() {
            rows.push(PlaylistsRows::TrackRow(i + 1, track.id));
            heights.push(px(60.0));
        }
    }

    (rows, heights)
}
