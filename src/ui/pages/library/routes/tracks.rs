use gpui::{
    App, Context, Div, Entity, FontWeight, ImageSource, InteractiveElement, IntoElement, ObjectFit,
    ParentElement, Render, StatefulInteractiveElement, Styled, StyledImage,
    UniformListScrollHandle, Window, div, img, px, rems,
};
use std::sync::Arc;

use crate::{
    controller::{Controller, state::TrackId},
    ui::{
        components::{
            Page,
            image_cache::{ImageCache, image_with_loader},
            scrollbar::{RightPad, floating_scrollbar},
            uniform_list::uniform_list_with_scroll_past,
        },
        pages::library::sort::{
            SortCacheKey, SortControlsState, SortField, SortFingerprint, sort_controls,
        },
        theme::Theme,
    },
};

const THUMBNAIL_MARGIN: usize = 16;

pub struct TracksSection {
    pub scroll_handle: UniformListScrollHandle,
    pub sort: Entity<SortControlsState>,
    pub(in crate::ui::pages::library) sorted_ids: Arc<Vec<TrackId>>,
    pub(in crate::ui::pages::library) sort_cache_key: Option<SortCacheKey>,
}

impl TracksSection {
    const SORT_FIELDS: &'static [SortField] = &[
        SortField::Title,
        SortField::Artist,
        SortField::Album,
        SortField::Duration,
        SortField::FileName,
    ];

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
                .map_or_else(|| "Unknown".to_string(), |album| album.name.to_string());

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
        let loading = track.image_id.is_some_and(|id| {
            cx.global::<ImageCache>()
                .is_loading(id, crate::cacher::ImageKind::ThumbnailSmall)
        }) || cx
            .global::<ImageCache>()
            .is_track_loading(track.id, crate::cacher::ImageKind::ThumbnailSmall);

        div().h_16().py_1().child(
            div()
                .id(format!("track_{:?}", track.id.0))
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
                        .child(
                            image_with_loader(
                                match thumbnail {
                                    Some(image) => img(ImageSource::Render(image.clone()))
                                        .object_fit(ObjectFit::Contain)
                                        .size_full()
                                        .border_1()
                                        .border_color(theme.border)
                                        .rounded_md(),
                                    None => img("icons/placeholder.svg")
                                        .object_fit(ObjectFit::Contain)
                                        .size_full()
                                        .border_1()
                                        .border_color(theme.border)
                                        .rounded_md(),
                                },
                                loading,
                                format!("track_loader_{:?}", track.id.0),
                            )
                            .w_11()
                            .h_11()
                            .flex_shrink_0(),
                        )
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
        let sort = *self.sort.read(cx);

        let state = controller.state.read(cx);
        let mut fingerprint = SortFingerprint::default();
        for (id, track) in &state.library.tracks {
            fingerprint.add(id);
            match sort.field {
                SortField::Title => fingerprint.add(track.title.as_ref()),
                SortField::Artist => {
                    for artist in track.artists(&state.library) {
                        fingerprint.add(artist.name.as_ref());
                    }
                }
                SortField::Album => {
                    fingerprint.add(
                        &track
                            .album(&state.library)
                            .map_or("Unknown", |album| album.name.as_ref()),
                    );
                }
                SortField::Duration => fingerprint.add(&track.duration),
                SortField::FileName => {
                    fingerprint.add(
                        &track
                            .sources
                            .first()
                            .and_then(|source| source.path.file_name()),
                    );
                }
                SortField::Name | SortField::TrackCount | SortField::AlbumCount => {
                    fingerprint.add(track.title.as_ref());
                }
            }
        }
        let cache_key = SortCacheKey {
            field: sort.field,
            descending: sort.descending,
            fingerprint: fingerprint.finish(),
        };
        if self.sort_cache_key != Some(cache_key) {
            let library = &state.library;
            let mut ids = library.tracks.keys().copied().collect::<Vec<_>>();
            match sort.field {
                SortField::Title => ids.sort_by_cached_key(|id| {
                    let track = &library.tracks[id];
                    (track.title.to_lowercase(), id.0)
                }),
                SortField::Artist => ids.sort_by_cached_key(|id| {
                    let track = &library.tracks[id];
                    let artist = track
                        .artists(library)
                        .map(|artist| artist.name.as_ref())
                        .collect::<Vec<_>>()
                        .join(", ")
                        .to_lowercase();
                    (artist, id.0)
                }),
                SortField::Album => ids.sort_by_cached_key(|id| {
                    let track = &library.tracks[id];
                    (
                        track.album(library).map_or_else(
                            || "Unknown".to_lowercase(),
                            |album| album.name.to_lowercase(),
                        ),
                        id.0,
                    )
                }),
                SortField::Duration => {
                    ids.sort_by_cached_key(|id| (library.tracks[id].duration, id.0))
                }
                SortField::FileName => ids.sort_by_cached_key(|id| {
                    let file_name = library.tracks[id]
                        .sources
                        .first()
                        .and_then(|source| source.path.file_name())
                        .map(|name| name.to_string_lossy().to_lowercase())
                        .unwrap_or_default();
                    (file_name, id.0)
                }),
                SortField::Name | SortField::TrackCount | SortField::AlbumCount => {
                    ids.sort_by_cached_key(|id| (library.tracks[id].title.to_lowercase(), id.0));
                }
            }
            if sort.descending {
                ids.reverse();
            }
            self.sorted_ids = Arc::new(ids);
            self.sort_cache_key = Some(cache_key);
        }
        let track_ids = self.sorted_ids.clone();
        let len = track_ids.len();
        let scroll_handle = self.scroll_handle.clone();

        div()
            .w_full()
            .h_full()
            .flex()
            .flex_col()
            .child(
                div()
                    .py_4()
                    .px_12()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
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
                    )
                    .child(sort_controls(
                        "tracks_sort",
                        self.sort.clone(),
                        Self::SORT_FIELDS,
                        cx,
                    )),
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
                        uniform_list_with_scroll_past(
                            "tracks",
                            len,
                            px(128.0),
                            px(64.0),
                            move |range, _, cx| {
                                let start = range.start.saturating_sub(THUMBNAIL_MARGIN);
                                let end = (range.end + THUMBNAIL_MARGIN).min(len);

                                let thumb_tracks: Vec<TrackId> =
                                    (start..end).map(|i| track_ids[i]).collect();

                                controller.request_track_thumbnails(&thumb_tracks, cx);

                                range
                                    .map(|i| Self::render_track(i + 1, track_ids[i], cx))
                                    .collect()
                            },
                        )
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
