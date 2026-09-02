use gpui::{
    App, Context, Div, FontWeight, ImageSource, IntoElement, ObjectFit, Render, Styled,
    UniformListScrollHandle, Window, div, img, px, rems, uniform_list,
};

use crate::{
    controller::{Controller, state::TrackId},
    ui::{
        components::{
            Page,
            icons::{Icons, icon},
            image_cache::ImageCache,
            scrollbar::{RightPad, floating_scrollbar},
        },
        theme::Theme,
    },
};

const THUMBNAIL_MARGIN: usize = 16;

pub struct FavoritesSection {
    pub scroll_handle: UniformListScrollHandle,
}

impl FavoritesSection {
    fn render_track(index: usize, id: TrackId, cx: &mut App) -> Div {
        let controller = cx.global::<Controller>().clone();
        let theme = *cx.global::<Theme>();

        let (track, artists, album) = {
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

            (track.clone(), artists, album)
        };

        let thumbnail = track
            .image_id
            .and_then(|id| cx.global_mut::<ImageCache>().get(&id));

        let is_favorite = controller.is_favorite(id, cx);

        div().h_16().py_1().child(
            div()
                .id(format!("favorite_track_{:?}", track.id.0))
                .size_full()
                .flex()
                .items_center()
                .rounded_md()
                .cursor_pointer()
                .hover(|this| this.bg(theme.library_tracks_section_bg_hover))
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
                )
                .child(
                    div()
                        .id(format!("favorite_toggle_{:?}", track.id.0))
                        .w_12()
                        .h_full()
                        .px_3()
                        .flex()
                        .items_center()
                        .justify_end()
                        .cursor_pointer()
                        .text_color(if is_favorite {
                            theme.library_favorites_section_title
                        } else {
                            theme.library_tracks_section_table_slno
                        })
                        .on_click({
                            let controller = controller.clone();
                            move |_, _, cx| {
                                controller.toggle_favorite(id, cx);
                                cx.stop_propagation();
                            }
                        })
                        .child(icon(Icons::Heart).size_4()),
                ),
        )
    }
}

impl Render for FavoritesSection {
    #[allow(clippy::too_many_lines)]
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let controller = cx.global::<Controller>().clone();
        let render_controller = controller.clone();

        let state = controller.state.read(cx);
        let favorite_ids = state
            .favorites
            .iter()
            .filter(|id| state.library.tracks.contains_key(id))
            .copied()
            .collect::<Vec<_>>();
        let len = favorite_ids.len();
        let _ = state;

        let scroll_handle = self.scroll_handle.clone();

        div()
            .w_full()
            .h_full()
            .flex()
            .flex_col()
            .child(
                div().py_4().px_12().flex().flex_col().child(
                    div()
                        .text_size(rems(2.0))
                        .font_weight(FontWeight::BOLD)
                        .tracking_tight()
                        .text_color(theme.library_favorites_section_title)
                        .child("Favorites")
                        .child(
                            div()
                                .h(px(2.0))
                                .w_16()
                                .mt_1()
                                .bg(theme.library_favorites_section_title),
                        ),
                ),
            )
            .child(
                div()
                    .px_12()
                    .pb_2()
                    .text_sm()
                    .text_color(theme.library_tracks_section_table_header_text)
                    .child(format!("{len} tracks")),
            )
            .when(len > 0, |this| {
                this.child(
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
                        )
                        .child(div().w_12().h_full()),
                )
            })
            .child(if len > 0 {
                div().flex_1().relative().px_12().pb_2().child(
                    div()
                        .id("favorites_list_container")
                        .size_full()
                        .child(
                            uniform_list("favorites", len, cx.processor(move |_, range, _, cx| {
                                let start = range.start.saturating_sub(THUMBNAIL_MARGIN);
                                let end = (range.end + THUMBNAIL_MARGIN).min(len);

                                let thumb_tracks: Vec<TrackId> =
                                    (start..end).map(|i| favorite_ids[i]).collect();

                                 render_controller.request_track_thumbnails(&thumb_tracks, cx);

                                range
                                    .map(|i| Self::render_track(i + 1, favorite_ids[i], cx))
                                    .collect()
                            }))
                            .w_full()
                            .h_full()
                            .flex()
                            .flex_col()
                            .track_scroll(&scroll_handle),
                        ),
                )
            } else {
                div()
                    .flex_1()
                    .w_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_base()
                    .text_color(theme.library_tracks_section_table_header_text)
                    .child("No favorites yet — hit the heart on the player to add some")
            })
            .when(len > 0, |this| {
                this.child(floating_scrollbar(
                    "favorites_section_scrollbar",
                    scroll_handle,
                    RightPad::Pad,
                ))
            })
    }
}
