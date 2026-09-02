use gpui::{
    App, Context, Div, FontWeight, ImageSource, IntoElement, ObjectFit, Render, ScrollHandle,
    Styled, Window,
    div, gradient_color_stop, img, linear_gradient, px, rems,
};

use crate::{
    controller::{
        Controller,
        ListenStats,
        state::{AlbumId, ArtistId, TrackId, TrackListenMetrics},
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

const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

fn fmt_play_time(duration: std::time::Duration) -> String {
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

fn fmt_date(unix_secs: u64) -> String {
    let z = i64::try_from(unix_secs / 86_400).unwrap_or(i64::MAX) + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if month <= 2 { year + 1 } else { year };

    format!("{} {} {}", MONTHS[(month - 1) as usize], day, year)
}

pub struct StatsSection {
    pub scroll_handle: ScrollHandle,
}

impl StatsSection {
    fn section_header(title: &str, cx: &mut App) -> Div {
        let theme = *cx.global::<Theme>();

        div()
            .text_lg()
            .font_weight(FontWeight::MEDIUM)
            .tracking_tight()
            .text_color(theme.library_stats_section_title)
            .child(title.to_string())
    }

    fn horizontal_row(children: Vec<Div>) -> Div {
        div()
            .w_full()
            .flex()
            .gap_4()
            .py_1()
            .overflow_x_scroll()
            .children(children)
    }

    fn banner_pill(label: &str, lines: Vec<String>, theme: Theme) -> Div {
        div()
            .w_56()
            .px_5()
            .py_3()
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

    fn banner(
        stats: &ListenStats,
        top_track: Option<(String, String, u32)>,
        top_artist: Option<(String, u32)>,
        theme: Theme,
    ) -> Div {
        div()
            .w_full()
            .relative()
            .overflow_hidden()
            .rounded_2xl()
            .bg(linear_gradient(
                100.0,
                gradient_color_stop(theme.library_stats_banner_gradient_a, 0.0),
                gradient_color_stop(theme.library_stats_banner_gradient_b, 1.0),
            ))
            .child(
                div()
                    .absolute()
                    .top(px(-90.0))
                    .right(px(96.0))
                    .size(px(300.0))
                    .rounded_full()
                    .bg(theme.library_stats_banner_accent),
            )
            .child(
                div()
                    .absolute()
                    .bottom(px(-140.0))
                    .left(px(280.0))
                    .size(px(320.0))
                    .rounded_full()
                    .bg(theme.library_stats_banner_accent),
            )
            .child(
                div()
                    .relative()
                    .w_full()
                    .px_8()
                    .py_10()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap_8()
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .flex()
                            .flex_col()
                            .items_start()
                            .gap_2()
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(FontWeight::MEDIUM)
                                    .tracking_widest()
                                    .text_color(theme.library_stats_banner_muted)
                                    .child("TOTAL LISTENING TIME"),
                            )
                            .child(
                                div()
                                    .text_size(rems(3.0))
                                    .font_weight(FontWeight::BOLD)
                                    .tracking_tight()
                                    .text_color(theme.library_stats_banner_text)
                                    .child(fmt_play_time(stats.total_play_time)),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(theme.library_stats_banner_muted)
                                    .child(format!(
                                        "across {} tracks · {} plays · {} skips",
                                        stats.total_tracks_listened,
                                        stats.total_plays,
                                        stats.total_skips
                                    )),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_4()
                            .child(Self::banner_pill(
                                "TOP ARTIST",
                                top_artist
                                    .map(|(name, plays)| vec![name, format!("{plays} plays")])
                                    .unwrap_or_default(),
                                theme,
                            ))
                            .child(Self::banner_pill(
                                "TOP TRACK",
                                top_track
                                    .map(|(title, artist, plays)| {
                                        vec![title, format!("{artist} · {plays} plays")]
                                    })
                                    .unwrap_or_default(),
                                theme,
                            )),
                    ),
            )
    }

    fn stat_card(value: String, label: &str, theme: Theme) -> Div {
        div()
            .flex_1()
            .min_w_0()
            .p_5()
            .rounded_xl()
            .bg(theme.library_stats_card_bg)
            .flex()
            .flex_col()
            .gap_1()
            .child(
                div()
                    .text_xl()
                    .font_weight(FontWeight::BOLD)
                    .text_color(theme.library_stats_card_value)
                    .child(value),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(theme.library_stats_card_label)
                    .child(label.to_string()),
            )
    }

    fn stat_grid(stats: &ListenStats, theme: Theme) -> Div {
        div()
            .w_full()
            .flex()
            .gap_4()
            .child(Self::stat_card(
                stats.total_plays.to_string(),
                "Total plays",
                theme,
            ))
            .child(Self::stat_card(
                stats.total_tracks_listened.to_string(),
                "Tracks played",
                theme,
            ))
            .child(Self::stat_card(
                stats.total_skips.to_string(),
                "Skips",
                theme,
            ))
            .child(Self::stat_card(
                stats
                    .first_listen
                    .map(fmt_date)
                    .unwrap_or_else(|| "—".to_string()),
                "First listen",
                theme,
            ))
    }

    fn render_track_row(
        index: usize,
        id: TrackId,
        metrics: &TrackListenMetrics,
        controller: &Controller,
        theme: Theme,
        cx: &mut App,
    ) -> Div {
        let (track, artists) = {
            let state = controller.state.read(cx);

            let Some(track) = state.library.tracks.get(&id) else {
                return div();
            };

            let artists = track
                .artists(&state.library)
                .map(|artist| artist.name.to_string())
                .collect::<Vec<_>>()
                .join(", ");

            (track.clone(), artists)
        };

        let thumbnail = track
            .image_id
            .and_then(|id| cx.global_mut::<ImageCache>().get(&id));

        div()
            .w_full()
            .flex()
            .items_center()
            .gap_4()
            .px_3()
            .py_2()
            .rounded_md()
            .hover(|this| this.bg(theme.library_stats_card_bg))
            .cursor_pointer()
            .on_click(move |_, _, cx| {
                let controller = cx.global::<Controller>().clone();

                controller.load_track(id, cx);
                *cx.global_mut::<Page>() = Page::Player;
            })
            .child(
                div()
                    .w_10()
                    .flex_shrink_0()
                    .flex()
                    .items_center()
                    .justify_center()
                    .font_family("JetBrains Mono")
                    .text_sm()
                    .text_color(theme.library_stats_rank)
                    .child(format!("{:02}", index + 1)),
            )
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
                div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .justify_center()
                    .child(
                        div()
                            .text_base()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(theme.library_stats_table_text)
                            .overflow_hidden()
                            .whitespace_nowrap()
                            .text_ellipsis()
                            .child(track.title.to_string()),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(theme.library_stats_table_meta)
                            .overflow_hidden()
                            .whitespace_nowrap()
                            .text_ellipsis()
                            .child(artists),
                    ),
            )
            .child(
                div()
                    .w_24()
                    .flex_shrink_0()
                    .flex()
                    .items_center()
                    .justify_end()
                    .text_sm()
                    .text_color(theme.library_stats_table_meta)
                    .child(format!("{} plays", metrics.play_count)),
            )
            .child(
                div()
                    .w_24()
                    .flex_shrink_0()
                    .flex()
                    .items_center()
                    .justify_end()
                    .font_family("JetBrains Mono")
                    .text_sm()
                    .text_color(theme.library_stats_table_meta)
                    .child(fmt_play_time(metrics.play_time)),
            )
    }

    fn top_tracks_section(stats: &ListenStats, cx: &mut App) -> Div {
        let theme = *cx.global::<Theme>();
        let controller = cx.global::<Controller>().clone();

        let rows = stats
            .top_tracks
            .iter()
            .enumerate()
            .map(|(index, (id, metrics))| {
                Self::render_track_row(index, *id, metrics, &controller, theme, cx)
            })
            .collect::<Vec<_>>();

        div()
            .w_full()
            .flex()
            .flex_col()
            .gap_3()
            .child(Self::section_header("Top Tracks", cx))
            .child(div().w_full().flex().flex_col().gap_1().children(rows))
    }

    fn render_artist_card(id: ArtistId, plays: u32, cx: &mut App) -> Div {
        let controller = cx.global::<Controller>().clone();
        let theme = *cx.global::<Theme>();

        let (artist, image_id) = {
            let state = controller.state.read(cx);

            let Some(artist) = state.library.artists.get(&id) else {
                return div();
            };

            let image_id = artist.image_id.or_else(|| {
                artist.tracks.first().and_then(|track_id| {
                    state.library.tracks.get(track_id).and_then(|t| t.image_id)
                })
            });

            (artist.clone(), image_id)
        };

        let thumbnail = image_id.and_then(|id| cx.global_mut::<ImageCache>().get(&id));

        div().w(px(180.0)).flex_shrink_0().child(
            div()
                .id(format!("stats_artist_{}", artist.id.0))
                .w_full()
                .flex()
                .flex_col()
                .gap_2()
                .p_3()
                .rounded_xl()
                .bg(theme.library_stats_card_bg)
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
                        .text_base()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme.library_stats_table_text)
                        .overflow_hidden()
                        .whitespace_nowrap()
                        .text_ellipsis()
                        .child(artist.name.to_string()),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(theme.library_stats_table_meta)
                        .child(format!("{plays} plays")),
                ),
        )
    }

    fn render_album_card(id: AlbumId, plays: u32, cx: &mut App) -> Div {
        let controller = cx.global::<Controller>().clone();
        let theme = *cx.global::<Theme>();

        let (album, image_id) = {
            let state = controller.state.read(cx);

            let Some(album) = state.library.albums.get(&id) else {
                return div();
            };

            let image_id = album.image_id.or_else(|| {
                album.tracks.first().and_then(|track_id| {
                    state.library.tracks.get(track_id).and_then(|t| t.image_id)
                })
            });

            (album.clone(), image_id)
        };

        let thumbnail = image_id.and_then(|id| cx.global_mut::<ImageCache>().get(&id));

        div().w(px(180.0)).flex_shrink_0().child(
            div()
                .id(format!("stats_album_{}", album.id.0))
                .w_full()
                .flex()
                .flex_col()
                .gap_2()
                .p_3()
                .rounded_xl()
                .bg(theme.library_stats_card_bg)
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
                        .text_base()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme.library_stats_table_text)
                        .overflow_hidden()
                        .whitespace_nowrap()
                        .text_ellipsis()
                        .child(album.name.to_string()),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(theme.library_stats_table_meta)
                        .child(format!("{plays} plays")),
                ),
        )
    }

    fn top_artists_section(stats: &ListenStats, cx: &mut App) -> Div {
        let rows = stats
            .top_artists
            .iter()
            .map(|(id, plays)| Self::render_artist_card(*id, *plays, cx))
            .collect();

        div()
            .w_full()
            .flex()
            .flex_col()
            .gap_3()
            .child(Self::section_header("Top Artists", cx))
            .child(Self::horizontal_row(rows))
    }

    fn top_albums_section(stats: &ListenStats, cx: &mut App) -> Div {
        let rows = stats
            .top_albums
            .iter()
            .map(|(id, plays)| Self::render_album_card(*id, *plays, cx))
            .collect();

        div()
            .w_full()
            .flex()
            .flex_col()
            .gap_3()
            .child(Self::section_header("Top Albums", cx))
            .child(Self::horizontal_row(rows))
    }

    fn empty_state(theme: Theme) -> Div {
        div()
            .flex_1()
            .w_full()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap_2()
            .text_base()
            .text_color(theme.library_stats_card_label)
            .child(div().text_size(rems(1.4)).child("No stats yet"))
            .child(div().text_sm().child("Play some music to unlock your wrapped."))
    }
}

impl Render for StatsSection {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let controller = cx.global::<Controller>().clone();

        let stats = controller.listen_stats(cx);
        let has_data = stats.total_plays > 0;

        let (top_track, top_artist) = if has_data {
            let state = controller.state.read(cx);

            let top_track = stats.top_tracks.first().and_then(|(id, m)| {
                let track = state.library.tracks.get(id)?;

                let artist = track
                    .artists(&state.library)
                    .map(|artist| artist.name.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");

                Some((track.title.to_string(), artist, m.play_count))
            });

            let top_artist = stats.top_artists.first().and_then(|(id, plays)| {
                let artist = state.library.artists.get(id)?;

                Some((artist.name.to_string(), *plays))
            });

            (top_track, top_artist)
        } else {
            (None, None)
        };

        let top_track_ids = stats
            .top_tracks
            .iter()
            .map(|(id, _)| *id)
            .collect::<Vec<_>>();
        let top_artist_ids = stats
            .top_artists
            .iter()
            .map(|(id, _)| *id)
            .collect::<Vec<_>>();
        let top_album_ids = stats
            .top_albums
            .iter()
            .map(|(id, _)| *id)
            .collect::<Vec<_>>();

        controller.request_track_thumbnails(&top_track_ids, cx);
        controller.request_artist_thumbnails(&top_artist_ids, cx);
        controller.request_album_thumbnails(&top_album_ids, cx);

        let mut rows: Vec<Div> = Vec::new();

        if has_data {
            rows.push(Self::banner(&stats, top_track, top_artist, theme));
            rows.push(Self::stat_grid(&stats, theme));

            if !stats.top_tracks.is_empty() {
                rows.push(Self::top_tracks_section(&stats, cx));
            }

            if !stats.top_artists.is_empty() {
                rows.push(Self::top_artists_section(&stats, cx));
            }

            if !stats.top_albums.is_empty() {
                rows.push(Self::top_albums_section(&stats, cx));
            }
        }

        let scroll_handle = self.scroll_handle.clone();

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
                            .text_color(theme.library_stats_section_title)
                            .child("Stats")
                            .child(
                                div()
                                    .h(px(2.0))
                                    .w_16()
                                    .mt_1()
                                    .bg(theme.library_stats_section_title),
                            ),
                    )
                    .child(
                        div()
                            .mt_1()
                            .text_sm()
                            .text_color(theme.library_stats_card_label)
                            .child("Your listening, wrapped"),
                    ),
            )
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .min_w_0()
                    .relative()
                    .child(
                        div()
                            .id("stats_scroll")
                            .w_full()
                            .h_full()
                            .overflow_y_scroll()
                            .track_scroll(&scroll_handle)
                            .px_8()
                            .pb_8()
                            .pt_4()
                            .flex()
                            .flex_col()
                            .gap_y_8()
                            .children(if has_data {
                                rows
                            } else {
                                vec![Self::empty_state(theme)]
                            }),
                    ),
            )
            .child(floating_scrollbar(
                "stats_scrollbar",
                scroll_handle,
                RightPad::Pad,
            ))
    }
}
