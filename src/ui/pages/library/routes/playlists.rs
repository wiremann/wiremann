use gpui::{
    App, Context, Div, Entity, FontWeight, ImageSource, InteractiveElement, IntoElement, ObjectFit,
    ParentElement, Render, ScrollHandle, StatefulInteractiveElement, Styled, StyledImage, Window,
    div, img, prelude::FluentBuilder, px, rems,
};
use std::sync::Arc;

use crate::{
    controller::{Controller, state::PlaylistId},
    ui::{
        components::{
            image_cache::{ImageCache, image_with_loader},
            scrollbar::{RightPad, floating_scrollbar},
            virtual_grid::{VirtualGridScrollController, vgrid},
        },
        pages::library::LibraryRoutes,
        pages::library::sort::{
            SortCacheKey, SortControlsState, SortField, SortFingerprint, sort_controls, sort_menu,
        },
        theme::Theme,
    },
};

pub struct PlaylistsSection {
    pub scroll_handle: ScrollHandle,
    pub grid_controller: VirtualGridScrollController,
    pub sort: Entity<SortControlsState>,
    pub(in crate::ui::pages::library) sorted_ids: Arc<Vec<PlaylistId>>,
    pub(in crate::ui::pages::library) sort_cache_key: Option<SortCacheKey>,
}

impl PlaylistsSection {
    const SORT_FIELDS: &'static [SortField] =
        &[SortField::Name, SortField::TrackCount, SortField::Duration];

    fn render_playlist(id: PlaylistId, cx: &mut App) -> Div {
        let controller = cx.global::<Controller>().clone();
        let theme = *cx.global::<Theme>();

        let state = controller.state.read(cx).clone();

        let playlist = match state.library.playlists.get(&id) {
            Some(p) => p.clone(),
            None => return div(),
        };

        let thumbnail = playlist
            .image_id
            .and_then(|id| cx.global_mut::<ImageCache>().get_playlist(&id));
        let loading = {
            let cache = cx.global::<ImageCache>();
            cache.is_playlist_loading(playlist.id)
                || playlist
                    .image_id
                    .is_some_and(|id| cache.is_loading(id, crate::cacher::ImageKind::Playlist))
        };

        div().p_4().size_full().child(
            div()
                .id(format!("playlist_{}", playlist.id.0))
                .size_full()
                .bg(theme.library_playlists_section_bg)
                .rounded_xl()
                .hover(|this| this.bg(theme.library_playlists_section_bg_hover))
                .cursor_pointer()
                .when(
                    state.playback.current_playlist == Some(playlist.id),
                    |this| this.bg(theme.library_playlists_section_bg_active),
                )
                .on_click({
                    let id = playlist.id;

                    move |_, _, cx| {
                        // let controller = cx.global::<Controller>().clone();

                        // controller.load_playlist(id, cx);
                        *cx.global_mut::<LibraryRoutes>() = LibraryRoutes::Playlist(id);
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
                        format!("playlist-loader-{}", playlist.id.0),
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
                                .text_color(theme.library_playlists_section_title_text)
                                .overflow_hidden()
                                .whitespace_nowrap()
                                .text_ellipsis()
                                .child(playlist.name.to_string()),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(theme.library_playlists_section_meta_text)
                                .child(format!("{} Tracks", playlist.tracks.len())),
                        ),
                ),
        )
    }
}

impl Render for PlaylistsSection {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let controller = cx.global::<Controller>().clone();
        let sort = *self.sort.read(cx);
        let state = controller.state.read(cx);
        let library = &state.library;
        let mut fingerprint = SortFingerprint::default();
        for (id, playlist) in &library.playlists {
            fingerprint.add(id);
            match sort.field {
                SortField::TrackCount => fingerprint.add(&playlist.tracks.len()),
                SortField::Duration => fingerprint.add(&playlist.duration),
                _ => fingerprint.add(playlist.name.as_ref()),
            }
        }
        let cache_key = SortCacheKey {
            field: sort.field,
            descending: sort.descending,
            fingerprint: fingerprint.finish(),
        };
        if self.sort_cache_key != Some(cache_key) {
            let mut ids = library.playlists.keys().copied().collect::<Vec<_>>();
            match sort.field {
                SortField::TrackCount => {
                    ids.sort_by_cached_key(|id| (library.playlists[id].tracks.len(), id.0))
                }
                SortField::Duration => {
                    ids.sort_by_cached_key(|id| (library.playlists[id].duration, id.0))
                }
                _ => ids.sort_by_cached_key(|id| (library.playlists[id].name.to_lowercase(), id.0)),
            }
            if sort.descending {
                ids.reverse();
            }
            self.sorted_ids = Arc::new(ids);
            self.sort_cache_key = Some(cache_key);
        }
        let playlist_ids = self.sorted_ids.clone();
        let len = playlist_ids.len();
        div()
            .w_full()
            .h_full()
            .relative()
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
                            .text_color(theme.library_playlists_section_title)
                            .child("Playlists")
                            .child(
                                div()
                                    .h(px(2.0))
                                    .w_16()
                                    .mt_1()
                                    .bg(theme.library_playlists_section_title),
                            ),
                    )
                    .child(
                        div().flex()
                            .items_center()
                            .gap_2()
                            .child(sort_controls(
                                "playlists_sort",
                                self.sort.clone(),
                                cx,
                            ))
                            .child(
                                div()
                                .id("library_playlists_section_create_button")
                                    .ml_4()
                                    .px_6()
                                    .py_2()
                                    .rounded_md()
                                    .bg(theme.library_playlists_section_create_button_bg)
                                    .hover(|this| {
                                        this.bg(theme.library_playlists_section_create_button_bg_hover)
                                    })
                                    .cursor_pointer()
                                    .on_click({
                                        move |_, _, cx| {
                                            let controller = cx.global::<Controller>().clone();
                                            cx.spawn(async move |_| {
                                                if let Some(folder) = rfd::AsyncFileDialog::new().pick_folder().await {
                                                    controller.scan_dir(folder.path().into());
                                                }
                                            })
                                            .detach();
                                        }
                                    })
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            // .child(
                                            //     img("icons/plus.svg")
                                            //         .size(px(16.0), px(16.0))
                                            //         .object_fit(ObjectFit::Contain),
                                            // )
                                            .child(
                                                div()
                                                    .ml_2()
                                                    .text_sm()
                                                    .font_weight(FontWeight::MEDIUM)
                                                    .text_color(
                                                        theme
                                                            .library_playlists_section_create_button_text,
                                                    )
                                                    .child("Create Playlist"),
                                            ),
                                    ),
                            )
                    ),
            )
            .child(div().flex_1().relative().px_8().pb_4().child(if len == 0 {
                div()
                    .size_full()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .text_base()
                    .text_color(theme.library_playlists_section_empty_text)
                    .child(div().text_size(rems(1.4)).child("No playlists yet"))
                    .child(
                        div()
                            .mt_2()
                            .text_sm()
                            .child("Create your first playlist to get started."),
                    )
            } else {
                div()
                    .size_full()
                    .child(vgrid(
                        cx.entity(),
                        "playlists_grid",
                        len,
                        px(280.0),
                        px(56.0),
                        px(2.0),
                        self.scroll_handle.clone(),
                        &self.grid_controller,
                        move |_, range, _, _, cx| {
                            controller
                                .request_playlist_thumbnails(&playlist_ids[range.clone()], cx);

                            range
                                .map(|i| Self::render_playlist(playlist_ids[i], cx))
                                .collect::<Vec<_>>()
                        },
                    )
                    .scroll_past(px(128.0)))
                    .child(floating_scrollbar(
                        "playlists_scrollbar",
                        self.scroll_handle.clone(),
                        RightPad::Pad,
                    ))
            }))
            .when_some(
                sort_menu(
                    "playlists_sort",
                    self.sort.clone(),
                    Self::SORT_FIELDS,
                    px(72.0),
                    px(220.0),
                    cx,
                ),
                |this, menu| this.child(menu),
            )
    }
}
