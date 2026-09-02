use gpui::{
    App, Context, Div, FontWeight, ImageSource, IntoElement, ObjectFit, Render, ScrollHandle,
    Styled, Window, div, img, px, rems,
};

use crate::{
    controller::{Controller, state::PlaylistId},
    ui::{
        components::{
            Page,
            image_cache::ImageCache,
            scrollbar::{RightPad, floating_scrollbar},
            virtual_grid::{VirtualGridScrollController, vgrid},
        },
        pages::library::LibraryRoutes,
        theme::Theme,
    },
};

pub struct PlaylistsSection {
    pub scroll_handle: ScrollHandle,
    pub grid_controller: VirtualGridScrollController,
}

impl PlaylistsSection {
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
            .and_then(|id| cx.global_mut::<ImageCache>().get(&id));

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
                .child(match thumbnail {
                    Some(image) => div().w_full().aspect_square().child(
                        img(ImageSource::Render(image.clone()))
                            .size_full()
                            .object_fit(ObjectFit::Contain)
                            .rounded_xl()
                            .border_1()
                            .border_color(theme.border),
                    ),

                    None => div().w_full().aspect_square().child(
                        img("icons/placeholder.svg")
                            .size_full()
                            .object_fit(ObjectFit::Contain)
                            .rounded_xl()
                            .border_1()
                            .border_color(theme.border),
                    ),
                })
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

        let state = controller.state.read(cx);

        let playlist_ids = state.library.playlists.keys().copied().collect::<Vec<_>>();

        let len = playlist_ids.len();

        let _ = state;

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
                                            cx.spawn(async move {
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
                    ))
                    .child(floating_scrollbar(
                        "playlists_scrollbar",
                        self.scroll_handle.clone(),
                        RightPad::Pad,
                    ))
            }))
    }
}
