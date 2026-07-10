use gpui::{
    App, Context, Div, Element, FontWeight, ImageSource, InteractiveElement, IntoElement,
    ObjectFit, ParentElement, Render, StatefulInteractiveElement, Styled, StyledImage,
    UniformListScrollHandle, Window, div, img, prelude::FluentBuilder, px, rems, uniform_list,
};

use crate::{
    controller::{Controller, state::TrackId},
    ui::{
        components::{
            Page,
            image_cache::ImageCache,
            scrollbar::{RightPad, floating_scrollbar},
        },
        theme::Theme,
    },
};

const THUMBNAIL_MARGIN: usize = 16;

pub struct TracksSection {
    pub scroll_handle: UniformListScrollHandle,
}

impl TracksSection {
    fn render_track(index: usize, id: TrackId, cx: &mut App) -> Div {
        let controller = cx.global::<Controller>().clone();
        let theme = *cx.global::<Theme>();

        let (track, is_current, artists, album, image_id) = {
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
                .hover(|this| this.bg(theme.library_track_bg_hover))
                .when(is_current, |this| {
                    this.bg(theme.library_track_bg_active)
                        .text_color(theme.library_track_title_text_active)
                        .font_weight(FontWeight::MEDIUM)
                })
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
                        .text_color(theme.library_tracks_section_table_slno)
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
                        .gap_x_2()
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
                                    .text_color(theme.library_tracks_section_table_title)
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
                        .text_color(theme.library_tracks_section_table_artist)
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
                        .text_color(theme.library_tracks_section_table_album)
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
                        .text_color(theme.library_tracks_section_table_duration)
                        .child(format!(
                            "{:02}:{:02}",
                            track.duration.as_secs() / 60,
                            track.duration.as_secs() % 60
                        )),
                ),
        )
    }
}

impl Render for TracksSection {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let controller = cx.global::<Controller>().clone();

        let state = controller.state.read(cx);
        let track_ids = state.library.tracks.keys().copied().collect::<Vec<_>>();
        let len = track_ids.len();
        let _ = state;

        let scroll_handle = self.scroll_handle.clone();

        div()
            .w_full()
            .h_full()
            .flex()
            .flex_col()
            .child(
                div().py_4().px_12().flex().gap_x_4().child(
                    div()
                        .text_size(rems(2.0))
                        .font_weight(FontWeight::BOLD)
                        .tracking_tight()
                        .text_color(theme.library_tracks_section_title)
                        .child("Tracks")
                        .child(
                            div()
                                .h(px(2.0))
                                .w_16()
                                .mt_1()
                                .bg(theme.library_tracks_section_title),
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
                    .text_color(theme.library_tracks_section_table_header_text)
                    .border_b_1()
                    .border_color(theme.library_tracks_section_table_header_border)
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
                div()
                    .flex_1()
                    .relative()
                    .px_12()
                    .pb_2()
                    .child(
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
                    )
                    .child(floating_scrollbar(
                        "tracks_section_scrollbar",
                        scroll_handle,
                        RightPad::Pad,
                    )),
            )
    }
}
