use gpui::{
    App, Context, Div, FontWeight, ImageSource, InteractiveElement, IntoElement, ObjectFit,
    ParentElement, Render, ScrollHandle, StatefulInteractiveElement, Styled, StyledImage, Window,
    div, img, prelude::FluentBuilder, px, rems,
};

use crate::{
    controller::{Controller, state::ArtistId},
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

pub struct ArtistsSection {
    pub scroll_handle: ScrollHandle,
    pub grid_controller: VirtualGridScrollController,
}

impl ArtistsSection {
    fn render_artist(id: ArtistId, cx: &mut App) -> Div {
        let controller = cx.global::<Controller>().clone();
        let theme = *cx.global::<Theme>();

        let state = controller.state.read(cx).clone();

        let artist = match state.library.artists.get(&id) {
            Some(a) => a.clone(),
            None => return div(),
        };

        let thumbnail = artist
            .image_id
            .or_else(|| {
                artist.tracks.first().and_then(|track_id| {
                    state.library.tracks.get(track_id).and_then(|t| t.image_id)
                })
            })
            .and_then(|id| cx.global_mut::<ImageCache>().get(&id));

        div().p_4().size_full().child(
            div()
                .id(format!("artist_{}", artist.id.0))
                .size_full()
                .bg(theme.library_artists_section_bg)
                .rounded_xl()
                .hover(|this| this.bg(theme.library_artists_section_bg_hover))
                .cursor_pointer()
                .on_click({
                    let id = artist.id;

                    move |_, _, cx| {
                        *cx.global_mut::<LibraryRoutes>() = LibraryRoutes::Artist(id);
                    }
                })
                .flex()
                .flex_col()
                .child(match thumbnail {
                    Some(image) => div().w_full().aspect_square().child(
                        img(ImageSource::Render(image.clone()))
                            .size_full()
                            .object_fit(ObjectFit::Contain)
                            .rounded_full()
                            .border_1()
                            .border_color(theme.border),
                    ),

                    None => div().w_full().aspect_square().child(
                        img("icons/placeholder.svg")
                            .size_full()
                            .object_fit(ObjectFit::Contain)
                            .rounded_full()
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
                                .text_color(theme.library_artists_section_title_text)
                                .overflow_hidden()
                                .whitespace_nowrap()
                                .text_ellipsis()
                                .child(artist.name.to_string()),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(theme.library_artists_section_meta_text)
                                .child(format!("{} Tracks", artist.tracks.len())),
                        ),
                ),
        )
    }
}

impl Render for ArtistsSection {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let controller = cx.global::<Controller>().clone();

        let state = controller.state.read(cx);

        let artist_ids = state.library.artists.keys().copied().collect::<Vec<_>>();

        let len = artist_ids.len();

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
                    .flex_col()
                    .justify_center()
                    .child(
                        div()
                            .text_size(rems(2.0))
                            .font_weight(FontWeight::BOLD)
                            .tracking_tight()
                            .text_color(theme.library_artists_section_title)
                            .child("Artists")
                            .child(
                                div()
                                    .h(px(2.0))
                                    .w_16()
                                    .mt_1()
                                    .bg(theme.library_artists_section_title),
                            ),
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
                    .text_color(theme.library_artists_section_empty_text)
                    .child(div().text_size(rems(1.4)).child("No artists yet"))
                    .child(
                        div()
                            .mt_2()
                            .text_sm()
                            .child("Add some tracks to create artists."),
                    )
            } else {
                div()
                    .size_full()
                    .child(vgrid(
                        cx.entity(),
                        "artists_grid",
                        len,
                        px(280.0),
                        px(56.0),
                        px(2.0),
                        self.scroll_handle.clone(),
                        &self.grid_controller,
                        move |_, range, _, _, cx| {
                            controller.request_artist_thumbnails(&artist_ids[range.clone()], cx);

                            range
                                .map(|i| Self::render_artist(artist_ids[i], cx))
                                .collect::<Vec<_>>()
                        },
                    ))
                    .child(floating_scrollbar(
                        "artists_scrollbar",
                        self.scroll_handle.clone(),
                        RightPad::Pad,
                    ))
            }))
    }
}
