use gpui::{
    App, Context, Div, Entity, FontWeight, ImageSource, InteractiveElement, IntoElement, ObjectFit,
    ParentElement, Render, ScrollHandle, StatefulInteractiveElement, Styled, StyledImage, Window,
    div, img, prelude::FluentBuilder, px, rems,
};
use std::sync::Arc;

use crate::{
    controller::{Controller, state::AlbumId},
    ui::{
        components::{
            image_cache::{ImageCache, image_with_loader},
            scrollbar::{RightPad, floating_scrollbar},
            virtual_grid::{VirtualGridScrollController, vgrid},
        },
        pages::library::LibraryRoutes,
        pages::library::sort::{
            SortCacheKey, SortControlsState, SortField, SortFingerprint, sort_controls,
        },
        theme::Theme,
    },
};

pub struct AlbumsSection {
    pub scroll_handle: ScrollHandle,
    pub grid_controller: VirtualGridScrollController,
    pub sort: Entity<SortControlsState>,
    pub(in crate::ui::pages::library) sorted_ids: Arc<Vec<AlbumId>>,
    pub(in crate::ui::pages::library) sort_cache_key: Option<SortCacheKey>,
}

impl AlbumsSection {
    const SORT_FIELDS: &'static [SortField] = &[
        SortField::Name,
        SortField::Artist,
        SortField::TrackCount,
        SortField::Duration,
    ];

    fn render_album(id: AlbumId, cx: &mut App) -> Div {
        let controller = cx.global::<Controller>().clone();
        let theme = *cx.global::<Theme>();

        let state = controller.state.read(cx).clone();

        let album = match state.library.albums.get(&id) {
            Some(a) => a.clone(),
            None => return div(),
        };

        let image_id = album.image_id.or_else(|| {
            album
                .tracks
                .first()
                .and_then(|track_id| state.library.tracks.get(track_id).and_then(|t| t.image_id))
        });
        let cache = cx.global_mut::<ImageCache>();
        let thumbnail = image_id.and_then(|id| cache.get_large(&id));
        let loading = image_id
            .is_some_and(|id| cache.is_loading(id, crate::cacher::ImageKind::ThumbnailLarge))
            || album.tracks.first().is_some_and(|id| {
                cache.is_track_loading(*id, crate::cacher::ImageKind::ThumbnailLarge)
            });

        div().p_4().size_full().child(
            div()
                .id(format!("album_{}", album.id.0))
                .size_full()
                .bg(theme.library_albums_section_bg)
                .rounded_xl()
                .hover(|this| this.bg(theme.library_albums_section_bg_hover))
                .cursor_pointer()
                .on_click({
                    let id = album.id;

                    move |_, _, cx| {
                        *cx.global_mut::<LibraryRoutes>() = LibraryRoutes::Album(id);
                    }
                })
                .flex()
                .flex_col()
                .child(
                    image_with_loader(
                        match thumbnail {
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
                        },
                        loading,
                        format!("album-loader-{}", album.id.0),
                    )
                    .w_full()
                    .aspect_square(),
                )
                .child(
                    div()
                        .h(px(56.0))
                        .w_full()
                        .flex()
                        .items_start()
                        .justify_center()
                        .flex_col()
                        .mt_1()
                        .child(
                            div()
                                .text_base()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(theme.library_albums_section_title_text)
                                .overflow_hidden()
                                .whitespace_nowrap()
                                .text_ellipsis()
                                .child(album.name.to_string()),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(theme.library_albums_section_meta_text)
                                .child(format!("{} Tracks", album.tracks.len())),
                        ),
                ),
        )
    }
}

impl Render for AlbumsSection {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let controller = cx.global::<Controller>().clone();
        let sort = *self.sort.read(cx);
        let state = controller.state.read(cx);
        let library = &state.library;
        let mut fingerprint = SortFingerprint::default();
        for (id, album) in &library.albums {
            fingerprint.add(id);
            match sort.field {
                SortField::Artist => {
                    for artist in album.artists.iter().filter_map(|id| library.artist(*id)) {
                        fingerprint.add(artist.name.as_ref());
                    }
                }
                SortField::TrackCount => fingerprint.add(&album.tracks.len()),
                SortField::Duration => fingerprint.add(&album.duration),
                _ => fingerprint.add(album.name.as_ref()),
            }
        }
        let cache_key = SortCacheKey {
            field: sort.field,
            descending: sort.descending,
            fingerprint: fingerprint.finish(),
        };
        if self.sort_cache_key != Some(cache_key) {
            let mut ids = library.albums.keys().copied().collect::<Vec<_>>();
            match sort.field {
                SortField::Artist => ids.sort_by_cached_key(|id| {
                    let album = &library.albums[id];
                    let artists = album
                        .artists
                        .iter()
                        .filter_map(|id| library.artist(*id))
                        .map(|artist| artist.name.as_ref())
                        .collect::<Vec<_>>()
                        .join(", ")
                        .to_lowercase();
                    (artists, id.0)
                }),
                SortField::TrackCount => {
                    ids.sort_by_cached_key(|id| (library.albums[id].tracks.len(), id.0))
                }
                SortField::Duration => {
                    ids.sort_by_cached_key(|id| (library.albums[id].duration, id.0))
                }
                _ => ids.sort_by_cached_key(|id| (library.albums[id].name.to_lowercase(), id.0)),
            }
            if sort.descending {
                ids.reverse();
            }
            self.sorted_ids = Arc::new(ids);
            self.sort_cache_key = Some(cache_key);
        }
        let album_ids = self.sorted_ids.clone();
        let len = album_ids.len();
        div()
            .w_full()
            .h_full()
            .flex()
            .flex_col()
            .child(
                div()
                    .py_4()
                    .px_8()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .text_size(rems(2.0))
                            .font_weight(FontWeight::BOLD)
                            .tracking_tight()
                            .text_color(theme.library_albums_section_title)
                            .child("Albums")
                            .child(
                                div()
                                    .h(px(2.0))
                                    .w_16()
                                    .mt_1()
                                    .bg(theme.library_albums_section_title),
                            ),
                    )
                    .child(sort_controls(
                        "albums_sort",
                        self.sort.clone(),
                        Self::SORT_FIELDS,
                        cx,
                    )),
            )
            .child(div().flex_1().relative().px_8().pb_4().child(if len == 0 {
                div()
                    .size_full()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .text_base()
                    .text_color(theme.library_albums_section_empty_text)
                    .child(div().text_size(rems(1.4)).child("No albums yet"))
                    .child(
                        div()
                            .mt_2()
                            .text_sm()
                            .child("Add some tracks to create albums."),
                    )
            } else {
                div().size_full().child(
                    vgrid(
                        cx.entity(),
                        "albums_grid",
                        len,
                        px(280.0),
                        px(56.0),
                        px(2.0),
                        self.scroll_handle.clone(),
                        &self.grid_controller,
                        move |_, range, _, _, cx| {
                            controller.request_album_thumbnails(&album_ids[range.clone()], cx);

                            range
                                .map(|i| Self::render_album(album_ids[i], cx))
                                .collect::<Vec<_>>()
                        },
                    )
                    .scroll_past(px(128.0)),
                )
            }))
            .when(len > 0, |this| {
                this.child(floating_scrollbar(
                    "albums_scrollbar",
                    self.scroll_handle.clone(),
                    RightPad::Pad,
                ))
            })
    }
}
