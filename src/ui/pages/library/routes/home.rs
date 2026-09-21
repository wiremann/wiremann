use crate::ui::components::bounds_observer::observe_bounds;
use gpui::{
    App, Context, Div, Entity, FontWeight, ImageSource, InteractiveElement, IntoElement, ObjectFit,
    ParentElement, Pixels, Render, ScrollHandle, StatefulInteractiveElement, Styled, StyledImage,
    Window, div, gradient_color_stop, img, linear_gradient, prelude::FluentBuilder, px, rems,
};

use crate::{
    controller::{
        Controller, ListenStats,
        state::{AlbumId, ArtistId, PlaylistId, TrackId},
    },
    ui::{
        components::{
            Page,
            image_cache::ImageCache,
            scrollbar::{RightPad, floating_scrollbar},
        },
        pages::library::LibraryRoutes,
        theme::Theme,
    },
};

const ROW_COUNT: usize = 12;
const HOME_GRID_GAP: f32 = 16.0;

pub struct HomeSection {
    pub scroll_handle: ScrollHandle,
    pub avail_width: Entity<f64>,
}

impl HomeSection {
    fn calculate_grid_geometry(
        available_width: Pixels,
        min_card_width: Pixels,
        gap: Pixels,
    ) -> (usize, Pixels) {
        let available = available_width.to_f64();
        let min_width = min_card_width.to_f64();
        let gap = gap.to_f64();

        let columns = ((available + gap) / (min_width + gap)).floor().max(1.0) as usize;

        let card_width = ((available - (columns - 1) as f64 * gap) / columns as f64).max(0.0);

        (columns, px(card_width as f32))
    }
    fn section_header(title: &str, route: Option<LibraryRoutes>, cx: &mut App) -> Div {
        let theme = *cx.global::<Theme>();

        div()
            .w_full()
            .flex()
            .items_center()
            .justify_between()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::MEDIUM)
                    .tracking_tight()
                    .text_color(theme.library_home_section_title)
                    .child(title.to_string()),
            )
            .when_some(route, |this, route| {
                this.child(
                    div()
                        .id(format!(
                            "home_see_all_{}",
                            title.to_lowercase().replace(' ', "_")
                        ))
                        .px_2()
                        .py_1()
                        .rounded_md()
                        .text_sm()
                        .text_color(theme.library_home_section_see_all_text)
                        .hover(|this| {
                            this.text_color(theme.library_home_section_see_all_text_hover)
                        })
                        .cursor_pointer()
                        .on_click(move |_, _, cx| {
                            *cx.global_mut::<LibraryRoutes>() = route;
                        })
                        .child("See all"),
                )
            })
    }

    fn render_album_card(id: AlbumId, cx: &mut App) -> Div {
        let controller = cx.global::<Controller>().clone();
        let theme = *cx.global::<Theme>();

        let state = controller.state.read(cx).clone();

        let album = match state.library.albums.get(&id) {
            Some(a) => a.clone(),
            None => return div(),
        };

        let thumbnail = album
            .image_id
            .or_else(|| {
                album.tracks.first().and_then(|track_id| {
                    state.library.tracks.get(track_id).and_then(|t| t.image_id)
                })
            })
            .and_then(|id| cx.global_mut::<ImageCache>().get(&id));

        div().child(
            div()
                .id(format!("home_album_{}", album.id.0))
                .w_full()
                .flex()
                .flex_col()
                .gap_2()
                .p_3()
                .rounded_xl()
                .border_1()
                .border_color(theme.library_home_section_card_border)
                .bg(theme.library_home_section_card_bg)
                .hover(|this| this.bg(theme.library_home_section_card_bg_hover))
                .cursor_pointer()
                .on_click({
                    let id = album.id;

                    move |_, _, cx| {
                        *cx.global_mut::<LibraryRoutes>() = LibraryRoutes::Album(id);
                    }
                })
                .child(
                    div().w_full().aspect_square().child(match thumbnail {
                        Some(image) => img(ImageSource::Render(image.clone()))
                            .size_full()
                            .object_fit(ObjectFit::Contain)
                            .rounded_md()
                            .border_1()
                            .border_color(theme.border),
                        None => img("icons/placeholder.svg")
                            .size_full()
                            .object_fit(ObjectFit::Contain)
                            .rounded_md()
                            .border_1()
                            .border_color(theme.border),
                    }),
                )
                .child(
                    div()
                        .pt_1()
                        .text_base()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme.library_home_section_card_title)
                        .overflow_hidden()
                        .whitespace_nowrap()
                        .text_ellipsis()
                        .child(album.name.to_string()),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(theme.library_home_section_card_meta)
                        .child(format!("{} Tracks", album.tracks.len())),
                ),
        )
    }

    fn render_artist_card(id: ArtistId, cx: &mut App) -> Div {
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

        div().child(
            div()
                .id(format!("home_artist_{}", artist.id.0))
                .w_full()
                .flex()
                .flex_col()
                .gap_2()
                .p_3()
                .rounded_xl()
                .border_1()
                .border_color(theme.library_home_section_card_border)
                .bg(theme.library_home_section_card_bg)
                .hover(|this| this.bg(theme.library_home_section_card_bg_hover))
                .cursor_pointer()
                .on_click({
                    let id = artist.id;

                    move |_, _, cx| {
                        *cx.global_mut::<LibraryRoutes>() = LibraryRoutes::Artist(id);
                    }
                })
                .child(
                    div().w_full().aspect_square().child(match thumbnail {
                        Some(image) => img(ImageSource::Render(image.clone()))
                            .size_full()
                            .object_fit(ObjectFit::Contain)
                            .rounded_full()
                            .border_1()
                            .border_color(theme.border),
                        None => img("icons/placeholder.svg")
                            .size_full()
                            .object_fit(ObjectFit::Contain)
                            .rounded_full()
                            .border_1()
                            .border_color(theme.border),
                    }),
                )
                .child(
                    div()
                        .pt_1()
                        .text_base()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme.library_home_section_card_title)
                        .overflow_hidden()
                        .whitespace_nowrap()
                        .text_ellipsis()
                        .child(artist.name.to_string()),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(theme.library_home_section_card_meta)
                        .child(format!("{} Tracks", artist.tracks.len())),
                ),
        )
    }

    fn render_playlist_card(id: PlaylistId, cx: &mut App) -> Div {
        let controller = cx.global::<Controller>().clone();
        let theme = *cx.global::<Theme>();

        let state = controller.state.read(cx).clone();

        let playlist = match state.library.playlists.get(&id) {
            Some(p) => p.clone(),
            None => return div(),
        };

        let thumbnail = playlist
            .image_id
            .or_else(|| {
                playlist.tracks.first().and_then(|track_id| {
                    state.library.tracks.get(track_id).and_then(|t| t.image_id)
                })
            })
            .and_then(|id| cx.global_mut::<ImageCache>().get(&id));

        div().child(
            div()
                .id(format!("home_playlist_{}", playlist.id.0))
                .w_full()
                .flex()
                .flex_col()
                .gap_2()
                .p_3()
                .rounded_xl()
                .border_1()
                .border_color(theme.library_home_section_card_border)
                .bg(theme.library_home_section_card_bg)
                .hover(|this| this.bg(theme.library_home_section_card_bg_hover))
                .cursor_pointer()
                .on_click({
                    let id = playlist.id;

                    move |_, _, cx| {
                        *cx.global_mut::<LibraryRoutes>() = LibraryRoutes::Playlist(id);
                    }
                })
                .child(
                    div().w_full().aspect_square().child(match thumbnail {
                        Some(image) => img(ImageSource::Render(image.clone()))
                            .size_full()
                            .object_fit(ObjectFit::Contain)
                            .rounded_md()
                            .border_1()
                            .border_color(theme.border),
                        None => img("icons/placeholder.svg")
                            .size_full()
                            .object_fit(ObjectFit::Contain)
                            .rounded_md()
                            .border_1()
                            .border_color(theme.border),
                    }),
                )
                .child(
                    div()
                        .pt_1()
                        .text_base()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme.library_home_section_card_title)
                        .overflow_hidden()
                        .whitespace_nowrap()
                        .text_ellipsis()
                        .child(playlist.name.to_string()),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(theme.library_home_section_card_meta)
                        .child(format!("{} Tracks", playlist.tracks.len())),
                ),
        )
    }

    fn render_track_card(prefix: &str, id: TrackId, cx: &mut App) -> Div {
        let controller = cx.global::<Controller>().clone();
        let theme = *cx.global::<Theme>();

        let state = controller.state.read(cx).clone();

        let track = match state.library.tracks.get(&id) {
            Some(t) => t.clone(),
            None => return div(),
        };

        let artist = track
            .artists(&state.library)
            .map(|artist| artist.name.to_string())
            .collect::<Vec<_>>()
            .join(", ");

        let thumbnail = track
            .image_id
            .and_then(|id| cx.global_mut::<ImageCache>().get(&id));

        div().child(
            div()
                .id(format!("home_{prefix}_{:?}", track.id.0))
                .w_full()
                .flex()
                .flex_col()
                .gap_2()
                .p_3()
                .rounded_xl()
                .border_1()
                .border_color(theme.library_home_section_card_border)
                .bg(theme.library_home_section_card_bg)
                .hover(|this| this.bg(theme.library_home_section_card_bg_hover))
                .cursor_pointer()
                .on_click({
                    let controller = controller.clone();
                    let id = track.id;

                    move |_, _, cx| {
                        controller.load_track(id, cx);
                        *cx.global_mut::<Page>() = Page::Player;
                    }
                })
                .child(
                    div().w_full().aspect_square().child(match thumbnail {
                        Some(image) => img(ImageSource::Render(image.clone()))
                            .size_full()
                            .object_fit(ObjectFit::Contain)
                            .rounded_md()
                            .border_1()
                            .border_color(theme.border),
                        None => img("icons/placeholder.svg")
                            .size_full()
                            .object_fit(ObjectFit::Contain)
                            .rounded_md()
                            .border_1()
                            .border_color(theme.border),
                    }),
                )
                .child(
                    div()
                        .pt_1()
                        .text_base()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme.library_home_section_card_title)
                        .overflow_hidden()
                        .whitespace_nowrap()
                        .text_ellipsis()
                        .child(track.title.to_string()),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(theme.library_home_section_card_meta)
                        .overflow_hidden()
                        .whitespace_nowrap()
                        .text_ellipsis()
                        .child(artist),
                ),
        )
    }

    fn summary_pill(label: &str, lines: Vec<String>, theme: Theme) -> Div {
        div()
            .w_48()
            .px_4()
            .py_2()
            .rounded_lg()
            .bg(theme.library_stats_pill_bg)
            .flex()
            .flex_col()
            .gap_1()
            .child(
                div()
                    .text_xs()
                    .font_weight(FontWeight::MEDIUM)
                    .tracking_widest()
                    .text_color(theme.library_stats_banner_muted)
                    .child(label.to_string()),
            )
            .children(lines.into_iter().map(|line| {
                div()
                    .text_sm()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(theme.library_stats_banner_text)
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .text_ellipsis()
                    .child(line)
            }))
    }

    fn stats_summary_banner(
        stats: &ListenStats,
        top_track: Option<(String, String)>,
        top_artist: Option<String>,
        theme: Theme,
    ) -> Div {
        div()
            .w_full()
            .relative()
            .overflow_hidden()
            .rounded_2xl()
            .border_1()
            .border_color(theme.library_stats_banner_border)
            .bg(linear_gradient(
                100.0,
                gradient_color_stop(theme.library_stats_banner_gradient_a, 0.0),
                gradient_color_stop(theme.library_stats_banner_gradient_b, 1.0),
            ))
            .child(
                div()
                    .absolute()
                    .top(px(-80.0))
                    .right(px(120.0))
                    .size(px(240.0))
                    .rounded_full()
                    .bg(theme.library_stats_banner_accent),
            )
            .child(
                div()
                    .relative()
                    .w_full()
                    .px_6()
                    .py_5()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap_6()
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(FontWeight::MEDIUM)
                                    .tracking_widest()
                                    .text_color(theme.library_stats_banner_muted)
                                    .child("YOUR LISTENING"),
                            )
                            .child(
                                div()
                                    .text_size(rems(2.0))
                                    .font_weight(FontWeight::BOLD)
                                    .tracking_tight()
                                    .text_color(theme.library_stats_banner_text)
                                    .child(Self::fmt_time(stats.total_play_time)),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(theme.library_stats_banner_muted)
                                    .child(format!(
                                        "across {} tracks · {} plays",
                                        stats.total_tracks_listened, stats.total_plays
                                    )),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_3()
                            .when_some(top_artist, |this, name| {
                                this.child(Self::summary_pill("TOP ARTIST", vec![name], theme))
                            })
                            .when_some(top_track, |this, (title, artist)| {
                                this.child(Self::summary_pill(
                                    "TOP TRACK",
                                    vec![title, artist],
                                    theme,
                                ))
                            })
                            .child(
                                div()
                                    .id("home_see_your_stats")
                                    .px_4()
                                    .py_2()
                                    .rounded_lg()
                                    .bg(theme.library_stats_banner_button_bg)
                                    .text_color(theme.library_stats_banner_button_text)
                                    .text_sm()
                                    .font_weight(FontWeight::MEDIUM)
                                    .cursor_pointer()
                                    .hover(|this| this.opacity(0.85))
                                    .on_click(|_, _, cx| {
                                        *cx.global_mut::<LibraryRoutes>() = LibraryRoutes::Stats;
                                    })
                                    .child("See your stats"),
                            ),
                    ),
            )
    }

    fn fmt_time(duration: std::time::Duration) -> String {
        let total = duration.as_secs();
        let hours = total / 3600;
        let minutes = (total % 3600) / 60;
        let seconds = total % 60;

        if hours > 0 {
            format!("{hours}h {minutes:02}m")
        } else if minutes > 0 {
            format!("{minutes}m {seconds:02}s")
        } else {
            format!("{seconds}s")
        }
    }

    fn responsive_grid(children: Vec<Div>, available_width: Pixels, min_card_width: Pixels) -> Div {
        let gap = px(HOME_GRID_GAP);

        let (columns, cell_width) =
            Self::calculate_grid_geometry(available_width, min_card_width, gap);

        let mut rows = Vec::new();
        let mut iter = children.into_iter();

        loop {
            let mut row_children = Vec::new();

            for _ in 0..columns {
                if let Some(child) = iter.next() {
                    row_children.push(child.w(cell_width).flex_shrink_0().into_any_element());
                } else {
                    break;
                }
            }

            if row_children.is_empty() {
                break;
            }

            rows.push(div().w_full().flex().gap(gap).children(row_children));
        }

        div()
            .w_full()
            .min_w_0()
            .flex()
            .flex_col()
            .gap(gap)
            .children(rows)
    }
}

impl Render for HomeSection {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let controller = cx.global::<Controller>().clone();

        let state = controller.state.read(cx).clone();

        let track_count = state.library.tracks.len();
        let album_count = state.library.albums.len();
        let artist_count = state.library.artists.len();
        let playlist_count = state.library.playlists.len();

        let top_track_ids = controller.top_tracks(cx, ROW_COUNT);
        let recent_track_ids = controller.recently_played(cx, ROW_COUNT);
        let top_artist_ids = controller.top_artists(cx, ROW_COUNT);

        let mut albums = state
            .library
            .albums
            .iter()
            .map(|(id, a)| (*id, a.name.to_string()))
            .collect::<Vec<_>>();
        albums.sort_by_key(|(_, name)| name.to_lowercase());
        let album_ids = albums
            .into_iter()
            .take(ROW_COUNT)
            .map(|(id, _)| id)
            .collect::<Vec<_>>();

        let mut artists = state
            .library
            .artists
            .iter()
            .map(|(id, a)| (*id, a.name.to_string()))
            .collect::<Vec<_>>();
        artists.sort_by_key(|(_, name)| name.to_lowercase());
        let artist_ids = artists
            .into_iter()
            .take(ROW_COUNT)
            .map(|(id, _)| id)
            .collect::<Vec<_>>();

        let mut playlists = state
            .library
            .playlists
            .iter()
            .map(|(id, p)| (*id, p.name.to_string()))
            .collect::<Vec<_>>();
        playlists.sort_by_key(|(_, name)| name.to_lowercase());
        let playlist_ids = playlists
            .into_iter()
            .take(ROW_COUNT)
            .map(|(id, _)| id)
            .collect::<Vec<_>>();

        controller.request_track_thumbnails(&top_track_ids, cx);
        controller.request_track_thumbnails(&recent_track_ids, cx);
        controller.request_artist_thumbnails(&top_artist_ids, cx);
        controller.request_album_thumbnails(&album_ids, cx);
        controller.request_artist_thumbnails(&artist_ids, cx);
        controller.request_playlist_thumbnails(&playlist_ids, cx);

        let stats = controller.listen_stats(cx);
        let has_stats = stats.total_plays > 0;

        let (home_top_track, home_top_artist) = if has_stats {
            let state = controller.state.read(cx);

            let top_track = stats.top_tracks.first().and_then(|(id, _)| {
                let track = state.library.tracks.get(id)?;

                let artist = track
                    .artists(&state.library)
                    .map(|artist| artist.name.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");

                Some((track.title.to_string(), artist))
            });

            let top_artist = stats.top_artists.first().and_then(|(id, _)| {
                let artist = state.library.artists.get(id)?;

                Some(artist.name.to_string())
            });

            (top_track, top_artist)
        } else {
            (None, None)
        };

        let mut rows: Vec<Div> = Vec::new();
        let _avail_width = (*self.avail_width.read(cx) - 96.0) as f32;

        if track_count == 0 {
            rows.push(
                div()
                    .w_full()
                    .py_24()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .text_base()
                    .text_color(theme.library_albums_section_empty_text)
                    .child(div().text_size(rems(1.4)).child("Your library is empty"))
                    .child(
                        div()
                            .mt_2()
                            .text_sm()
                            .child("Add some tracks to get started."),
                    ),
            );
        } else if has_stats {
            rows.push(Self::stats_summary_banner(
                &stats,
                home_top_track,
                home_top_artist,
                theme,
            ));
        }

        let scroll_handle = self.scroll_handle.clone();

        div()
            .w_full()
            .h_full()
            .min_w_0()
            .flex()
            .flex_col()
            .px_8()
            .relative()
            .child(observe_bounds(
                "home-content-width",
                div().w_full().h_0().min_w_0().absolute(),
                {
                    let avail_width = self.avail_width.clone();

                    move |bounds, _, cx| {
                        let width = bounds.size.width.to_f64();

                        avail_width.update(cx, |v, _| {
                            *v = width;
                        });

                        println!("HOME WIDTH: {}", bounds.size.width);
                    }
                },
            ))
            .child(
                div()
                    .py_4()
                    .flex()
                    .flex_col()
                    .justify_center()
                    .child(
                        div()
                            .text_size(rems(2.0))
                            .font_weight(FontWeight::BOLD)
                            .tracking_tight()
                            .text_color(theme.library_home_section_title)
                            .child("Home")
                            .child(
                                div()
                                    .h(px(2.0))
                                    .w_16()
                                    .mt_1()
                                    .bg(theme.library_home_section_title),
                            ),
                    )
                    .child(
                        div()
                            .mt_1()
                            .text_sm()
                            .text_color(theme.library_home_section_meta)
                            .child(format!(
                                "{track_count} tracks · {album_count} albums · \
                         {artist_count} artists · {playlist_count} playlists"
                            )),
                    ),
            )
            .child(div().flex_1().min_h_0().min_w_0().p_4().children(rows))
            .child(floating_scrollbar(
                "home_scrollbar",
                scroll_handle,
                RightPad::Pad,
            ))
    }
}
