use crate::{
    controller::{
        Controller,
        state::{Playlist, PlaylistId, TrackId},
    },
    ui::{
        components::{
            Page,
            icons::{Icon, Icons, icon},
            image_cache::ImageCache,
            scrollbar::{RightPad, floating_scrollbar},
        },
        theme::Theme,
    },
};
use gpui::Entity;
use gpui::{
    App, Context, Div, FontWeight, ImageSource, InteractiveElement, IntoElement, ObjectFit,
    ParentElement, Render, StatefulInteractiveElement, Styled, StyledImage,
    UniformListScrollHandle, Window, div, img, px, rems, uniform_list,
};

const THUMBNAIL_MARGIN: usize = 16;

pub struct PlaylistViewSection {
    pub playlist_id: Entity<Option<PlaylistId>>,
    pub scroll_handle: UniformListScrollHandle,
}

impl PlaylistViewSection {
    fn render_track(index: usize, id: TrackId, cx: &mut App) -> Div {
        let controller = cx.global::<Controller>().clone();
        let theme = *cx.global::<Theme>();

        let (track, _, artists, album, _image_id) = {
            let state = controller.state.read(cx);

            let track = match state.library.tracks.get(&id) {
                Some(track) => track.clone(),
                None => return div().h_16(),
            };

            let artists = track
                .artists(&state.library)
                .map(|artist| artist.name.to_string())
                .collect::<Vec<_>>()
                .join(", ");

            let album = track
                .album(&state.library)
                .map(|album| album.name.to_string())
                .unwrap_or_else(|| "Unknown".to_string());

            (
                track.clone(),
                Some(id) == state.playback.current,
                artists,
                album,
                track.image_id,
            )
        };

        let thumbnail = track
            .image_id
            .and_then(|id| cx.global_mut::<ImageCache>().get(&id));

        div().h_16().py_1().child(
            div()
                .id(format!("track_{:?}", track.id.0))
                .size_full()
                .flex()
                .items_center()
                .rounded_md()
                .cursor_pointer()
                .hover(|this| this.bg(theme.library_playlist_section_bg_hover))
                .on_click(move |_, _, cx| {
                    let controller = cx.global::<Controller>().clone();

                    controller.load_track(id, cx);
                    *cx.global_mut::<Page>() = Page::Player;
                })
                .child(
                    div()
                        .w_10()
                        .h_full()
                        .px_3()
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_color(theme.library_playlist_section_table_slno)
                        .text_sm()
                        .font_family("JetBrains Mono")
                        .child(format!("{index:02}")),
                )
                .child(
                    div()
                        .flex_1()
                        .h_full()
                        .px_3()
                        .flex()
                        .gap_x_4()
                        .items_center()
                        .justify_start()
                        .child(match thumbnail {
                            Some(image) => div().w_11().h_11().flex_shrink_0().child(
                                img(ImageSource::Render(image.clone()))
                                    .object_fit(ObjectFit::Contain)
                                    .size_full()
                                    .border_1()
                                    .border_color(theme.border)
                                    .rounded_md(),
                            ),
                            None => div().w_11().h_11().flex_shrink_0().child(
                                img("icons/placeholder.svg")
                                    .object_fit(ObjectFit::Contain)
                                    .size_full()
                                    .border_1()
                                    .border_color(theme.border)
                                    .rounded_md(),
                            ),
                        })
                        .child(
                            div().flex_1().flex().flex_col().justify_center().child(
                                div()
                                    .text_color(theme.library_playlist_section_table_title)
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_base()
                                    .overflow_hidden()
                                    .whitespace_nowrap()
                                    .text_ellipsis()
                                    .child(track.title.to_string()),
                            ),
                        )
                        .overflow_hidden(),
                )
                .child(
                    div()
                        .w_1_3()
                        .max_w_1_3()
                        .h_full()
                        .px_3()
                        .flex()
                        .items_center()
                        .justify_start()
                        .text_color(theme.library_playlist_section_table_artist)
                        .text_sm()
                        .child(artists)
                        .overflow_hidden()
                        .whitespace_nowrap()
                        .text_ellipsis(),
                )
                .child(
                    div()
                        .w_1_4()
                        .max_w_1_4()
                        .h_full()
                        .px_3()
                        .flex()
                        .items_center()
                        .justify_start()
                        .text_color(theme.library_playlist_section_table_album)
                        .text_sm()
                        .child(album)
                        .overflow_hidden()
                        .whitespace_nowrap()
                        .text_ellipsis(),
                )
                .child(
                    div()
                        .w_20()
                        .max_w_20()
                        .h_full()
                        .px_3()
                        .flex()
                        .items_center()
                        .justify_end()
                        .font_family("JetBrains Mono")
                        .text_sm()
                        .text_color(theme.library_playlist_section_table_duration)
                        .child(format!(
                            "{:02}:{:02}",
                            track.duration.as_secs() / 60,
                            track.duration.as_secs() % 60
                        )),
                ),
        )
    }
}

impl Render for PlaylistViewSection {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let controller = cx.global::<Controller>().clone();

        let state = controller.state.read(cx).clone();

        let Some(playlist) = self
            .playlist_id
            .read(cx)
            .as_ref()
            .and_then(|id| state.library.playlists.get(id))
        else {
            return div();
        };

        controller.request_playlist_thumbnails(&[playlist.id], cx);

        let cache = cx.global_mut::<ImageCache>();
        let thumbnail = playlist.image_id.and_then(|id| cache.get(&id));

        let track_ids = playlist.tracks.clone();
        let len = track_ids.len();
        let _ = state;

        let scroll_handle = self.scroll_handle.clone();

        let playlist_name = playlist.name.to_string();

        div()
            .w_full()
            .h_full()
            .flex()
            .flex_col()
            .font_family("Space Grotesk")
            .child(
                div()
                    .px_12()
                    .py_8()
                    .flex()
                    .gap_8()
                    .items_end()
                    .child(
                        div().size_64().flex_shrink_0().child(match thumbnail {
                            Some(image) => img(ImageSource::Render(image.clone()))
                                .size_full()
                                .object_fit(ObjectFit::Contain)
                                .rounded_xl()
                                .border_1()
                                .border_color(theme.border),

                            None => img("icons/placeholder.svg")
                                .size_full()
                                .object_fit(ObjectFit::Contain)
                                .rounded_xl()
                                .border_1()
                                .border_color(theme.border),
                        }),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .justify_end()
                            .gap_2()
                            .pb_2()
                            .child(
                                div()
                                    .text_size(rems(3.2))
                                    .font_family("Space Grotesk")
                                    .font_weight(FontWeight::BOLD)
                                    .tracking_tight()
                                    .truncate()
                                    .text_ellipsis()
                                    .text_color(theme.library_playlist_section_header_title)
                                    .child(playlist_name),
                            )
                            .child(
                                div()
                                    .text_base()
                                    .text_color(theme.library_playlist_section_header_meta)
                                    .child(format!("{} Tracks", len)),
                            )
                            .child(
                                div()
                                    .mt_3()
                                    .flex()
                                    .gap_3()
                                    .child(
                                        div()
                                            .id("playlist_play_button")
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .gap_x_3()
                                            .px_8()
                                            .py_2()
                                            .rounded_lg()
                                            .bg(theme.library_playlist_section_button_bg)
                                            .text_color(theme.library_playlist_section_button_text)
                                            .font_weight(FontWeight::MEDIUM)
                                            .cursor_pointer()
                                            .child(icon(Icons::Play).size_4())
                                            .child("Play")
                                            .on_click({
                                                let id = playlist.id;
                                                move |_, _, cx| {
                                                    let controller =
                                                        cx.global::<Controller>().clone();
                                                    controller.load_playlist(id, cx);
                                                    *cx.global_mut::<Page>() = Page::Player;
                                                }
                                            }),
                                    )
                                    .child(
                                        div()
                                            .id("playlist_shuffle_button")
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .gap_x_3()
                                            .px_8()
                                            .py_2()
                                            .rounded_lg()
                                            .bg(theme.library_playlist_section_button_secondary_bg)
                                            .text_color(
                                                theme
                                                    .library_playlist_section_button_secondary_text,
                                            )
                                            .font_weight(FontWeight::MEDIUM)
                                            .cursor_pointer()
                                            .child(icon(Icons::Shuffle).size_4())
                                            .child("Shuffle")
                                            .on_click({
                                                let id = playlist.id;
                                                move |_, _, cx| {
                                                    let controller =
                                                        cx.global::<Controller>().clone();
                                                    controller.load_playlist(id, cx);
                                                    controller.set_shuffle(cx);
                                                    *cx.global_mut::<Page>() = Page::Player;
                                                }
                                            }),
                                    ),
                            ),
                    ),
            )
            .child(
                div()
                    .h_16()
                    .w_full()
                    .flex()
                    .items_center()
                    .text_xs()
                    .font_weight(FontWeight::NORMAL)
                    .text_color(theme.library_playlist_section_table_header_text)
                    .border_b_1()
                    .border_color(theme.library_playlist_section_table_header_border)
                    .px_12()
                    .child(
                        div()
                            .w_10()
                            .h_full()
                            .flex()
                            .items_center()
                            .justify_center()
                            .child("#"),
                    )
                    .child(
                        div()
                            .flex_1()
                            .h_full()
                            .px_3()
                            .flex()
                            .items_center()
                            .justify_start()
                            // Align header with title text after accounting for thumbnail size and padding
                            .ml(rems(3.75))
                            .child("TITLE"),
                    )
                    .child(
                        div()
                            .w_1_3()
                            .max_w_1_3()
                            .h_full()
                            .px_3()
                            .flex()
                            .items_center()
                            .justify_start()
                            .child("ARTIST"),
                    )
                    .child(
                        div()
                            .w_1_4()
                            .max_w_1_4()
                            .h_full()
                            .px_3()
                            .flex()
                            .items_center()
                            .justify_start()
                            .child("ALBUM"),
                    )
                    .child(
                        div()
                            .w_20()
                            .max_w_20()
                            .h_full()
                            .px_3()
                            .flex()
                            .items_center()
                            .justify_end()
                            .child("DURATION"),
                    ),
            )
            .child(
                div().flex_1().relative().px_12().pb_2().child(
                    div().id("tracks_list_container").size_full().child(
                        uniform_list("tracks", len, move |range, _, cx| {
                            let start = range.start.saturating_sub(THUMBNAIL_MARGIN);
                            let end = (range.end + THUMBNAIL_MARGIN).min(len);

                            let thumb_tracks: Vec<TrackId> =
                                (start..end).map(|i| track_ids[i]).collect();

                            controller.request_track_thumbnails(&thumb_tracks, cx);

                            range
                                .map(|i| Self::render_track(i + 1, track_ids[i], cx))
                                .collect()
                        })
                        .w_full()
                        .h_full()
                        .flex()
                        .flex_col()
                        .track_scroll(&scroll_handle),
                    ),
                ),
            )
            .child(floating_scrollbar(
                "tracks_section_scrollbar",
                scroll_handle,
                RightPad::Pad,
            ))
    }
}
