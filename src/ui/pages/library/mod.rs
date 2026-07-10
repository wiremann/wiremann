mod helpers;
mod sections;
mod sidebar;

use crate::controller::Controller;
use crate::controller::state::TrackId;
use crate::ui::animations::ease_in_out_expo;
use crate::ui::components::Page;
use crate::ui::components::image_cache::ImageCache;
use crate::ui::components::scrollbar::{RightPad, floating_scrollbar};
use crate::ui::helpers::{fingerprint_playlists, fingerprint_tracks};
use crate::ui::pages::library::sections::albums::AlbumsSection;
use crate::ui::pages::library::sections::artists::ArtistsSection;
use crate::ui::pages::library::sections::favorites::FavoritesSection;
use crate::ui::pages::library::sections::home::HomeSection;
use crate::ui::pages::library::sections::playlists::PlaylistsSection;
use crate::ui::pages::library::sections::plugins::PluginsSection;
use crate::ui::pages::library::sections::settings::SettingsSection;
use crate::ui::pages::library::sections::tracks::TracksSection;
use crate::ui::pages::library::sidebar::{Sidebar, SidebarBounds, SidebarIndicator};
use crate::ui::theme::Theme;
use gpui::prelude::FluentBuilder;
use gpui::{
    Animation, AnimationExt, App, AppContext, Context, Div, ElementId, Entity, FontWeight, Global,
    ImageSource, InteractiveElement, IntoElement, ObjectFit, ParentElement, Pixels, Render,
    ScrollHandle, StatefulInteractiveElement, Styled, StyledImage, UniformListScrollHandle,
    VirtualListScrollController, Window, div, img, px, vlist,
};
use helpers::{
    HeaderKind, LibraryRow, build_rows, render_header, render_playlist_grid,
    render_track_table_header,
};
use std::rc::Rc;

const THUMBNAIL_MARGIN: usize = 16;

#[repr(u64)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LibrarySection {
    // Discovery
    Home,
    Favorites,

    // Collection
    Tracks,
    Albums,
    Artists,
    Playlists,

    // System
    Settings,
    Plugins,
}

#[derive(Clone)]
pub struct LibraryPage {
    scroll_handle: ScrollHandle,
    rows: Rc<Vec<LibraryRow>>,
    heights: Rc<Vec<Pixels>>,
    pub sorted_tracks: Vec<&'static TrackId>,
    grid_cols: usize,
    last_fp: u128,
    pub list_controller: VirtualListScrollController,

    sidebar: Entity<Sidebar>,
}

impl LibrarySection {
    pub const fn index(self) -> i32 {
        match self {
            Self::Home => 0,
            Self::Favorites => 1,

            Self::Tracks => 2,
            Self::Albums => 3,
            Self::Artists => 4,
            Self::Playlists => 5,

            Self::Settings => 6,
            Self::Plugins => 7,
        }
    }

    pub const fn sidebar_offset(self) -> f32 {
        match self {
            Self::Home => 38.0,
            Self::Favorites => 70.0,

            Self::Tracks => 136.0,
            Self::Albums => 168.0,
            Self::Artists => 200.0,
            Self::Playlists => 232.0,

            Self::Plugins => 298.0,
            Self::Settings => 330.0,
        }
    }
}

impl LibraryPage {
    pub fn new(cx: &mut App) -> Self {
        let scroll_handle = ScrollHandle::new();
        let library = &cx.global::<Controller>().state.read(cx).library;

        let cols = 4;

        let (rows, heights) = build_rows(library, cols);

        cx.set_global(SidebarIndicator {
            top: 0.0,
            height: 32.0,
        });

        cx.set_global(SidebarBounds { top: 0.0 });

        LibraryPage {
            scroll_handle,
            rows: Rc::new(rows),
            heights: Rc::new(heights),
            grid_cols: cols,
            sorted_tracks: Vec::new(),
            last_fp: 0,
            list_controller: VirtualListScrollController::new(),
            sidebar: cx.new(|_| Sidebar),
        }
    }
    #[allow(clippy::too_many_lines)]
    fn render_track(i: usize, id: &TrackId, height: Pixels, cx: &mut App) -> Div {
        let image_id = {
            let state = cx.global::<Controller>().state.read(cx);
            state.library.tracks.get(id).and_then(|t| t.image_id)
        };

        let thumbnail = image_id.and_then(|id| cx.global_mut::<ImageCache>().get(&id));

        let controller = cx.global::<Controller>().clone();
        let theme = *cx.global::<Theme>();
        let state = controller.state.read(cx).clone();
        let is_current = Some(id) == state.playback.current.as_ref();

        if let Some(track) = state.library.tracks.get(id) {
            div()
                .h(height)
                .py_1()
                .border_b_1()
                .border_color(theme.library_track_border)
                .child(
                    div()
                        .id(format!("track_{:?}", track.id.0))
                        .size_full()
                        .flex()
                        .items_center()
                        .rounded_md()
                        .cursor_pointer()
                        .hover(|this| this.bg(theme.library_track_bg_hover))
                        .when(is_current, |this| this.bg(theme.library_track_bg_active))
                        .on_click({
                            let id = *id;
                            move |_, _, cx| {
                                let controller = cx.global::<Controller>().clone();

                                controller.load_track(id, cx);

                                *cx.global_mut::<Page>() = Page::Player;
                            }
                        })
                        .child(
                            div()
                                .w_20()
                                .h_full()
                                .flex()
                                .px_6()
                                .items_center()
                                .justify_start()
                                .child(format! {"{i:02}"}),
                        )
                        .child(
                            div()
                                .w_2_3()
                                .max_w_2_3()
                                .h_full()
                                .px_6()
                                .py_1()
                                .flex()
                                .gap_x_3()
                                .items_center()
                                .justify_start()
                                .child(match thumbnail {
                                    Some(image) => div().size_11().flex_shrink_0().child(
                                        img(ImageSource::Render(image.clone()))
                                            .object_fit(ObjectFit::Contain)
                                            .size_full()
                                            .border_1()
                                            .border_color(theme.border)
                                            .rounded_sm(),
                                    ),
                                    None => div().size_11().flex_shrink_0().child(
                                        img("icons/placeholder.svg")
                                            .object_fit(ObjectFit::Contain)
                                            .size_full()
                                            .border_1()
                                            .border_color(theme.border)
                                            .rounded_sm(),
                                    ),
                                })
                                .when(is_current, |this| {
                                    this.text_color(theme.library_track_title_text_active)
                                        .font_weight(FontWeight::MEDIUM)
                                })
                                .child(track.title.to_string())
                                .overflow_hidden()
                                .whitespace_nowrap()
                                .text_ellipsis(),
                        )
                        .child(
                            div()
                                .w_1_3()
                                .px_6()
                                .max_w_1_3()
                                .h_full()
                                .flex()
                                .items_center()
                                .justify_start()
                                .child(
                                    track
                                        .artists(&state.library)
                                        .map(|a| a.name.to_string())
                                        .collect::<Vec<_>>()
                                        .join(", "),
                                )
                                .overflow_hidden()
                                .whitespace_nowrap()
                                .text_ellipsis(),
                        )
                        .child(
                            div()
                                .w_1_3()
                                .max_w_1_3()
                                .px_6()
                                .h_full()
                                .flex()
                                .items_center()
                                .justify_start()
                                .child(
                                    track
                                        .album(&state.library)
                                        .map(|a| a.name.to_string())
                                        .unwrap_or_default(),
                                )
                                .overflow_hidden()
                                .whitespace_nowrap()
                                .text_ellipsis(),
                        )
                        .child(
                            div()
                                .w_24()
                                .max_w_24()
                                .h_full()
                                .px_4()
                                .flex()
                                .items_center()
                                .justify_start()
                                .text_sm()
                                .font_family("JetBrains Mono")
                                .child(format!(
                                    "{:02}:{:02}",
                                    track.duration.as_secs() / 60,
                                    track.duration.as_secs() % 60
                                ))
                                .overflow_hidden()
                                .whitespace_nowrap()
                                .text_ellipsis(),
                        ),
                )
        } else {
            div().h(height).py_2()
        }
    }
}

impl Render for LibraryPage {
    #[allow(clippy::too_many_lines)]
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();

        let controller = cx.global::<Controller>().clone();
        let state = controller.state.read(cx);

        let tracks_fp = fingerprint_tracks(state.library.tracks.keys().copied());
        let playlists_fp = fingerprint_playlists(state.library.playlists.keys().copied());

        let combined_fp = tracks_fp ^ playlists_fp;

        let width = window.bounds().size.width;
        let tile = 256.0;

        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let cols = ((width.to_f64() / tile) as usize).max(1);

        if cols != self.grid_cols || combined_fp != self.last_fp {
            let library = &state.library;

            let (rows, heights) = build_rows(library, cols);

            self.rows = Rc::new(rows);
            self.heights = Rc::new(heights);
            self.last_fp = combined_fp;
            self.grid_cols = cols;
        }
        let section = *cx.global::<LibrarySection>();

        let section_el = match section {
            LibrarySection::Home => div().w_full().h_full().child(cx.new(|_| HomeSection {})),
            LibrarySection::Favorites => div()
                .w_full()
                .h_full()
                .child(cx.new(|_| FavoritesSection {})),
            LibrarySection::Tracks => div().w_full().h_full().child(cx.new(|_| TracksSection {
                scroll_handle: UniformListScrollHandle::new(),
            })),
            LibrarySection::Albums => div().w_full().h_full().child(cx.new(|_| AlbumsSection {})),
            LibrarySection::Artists => div().w_full().h_full().child(cx.new(|_| ArtistsSection {})),
            LibrarySection::Playlists => div()
                .w_full()
                .h_full()
                .child(cx.new(|_| PlaylistsSection {})),
            LibrarySection::Settings => div()
                .w_full()
                .h_full()
                .child(cx.new(|_| SettingsSection {})),
            LibrarySection::Plugins => div().w_full().h_full().child(cx.new(|_| PluginsSection {})),
        };
        let section_state = window.use_keyed_state("library_transition", cx, |_, _| section);
        let prev_page = *section_state.read(cx);

        let direction = (section.index() - prev_page.index()).signum() as f32;

        div()
            .size_full()
            .bg(theme.library_bg)
            .text_color(theme.library_text)
            .px_2()
            .pt_2()
            .flex()
            .child(self.sidebar.clone())
            .child(
                div()
                    .id("animation_container")
                    .w_full()
                    .h_full()
                    .map(move |this| {
                        if prev_page == section {
                            this.child(section_el).into_any_element()
                        } else {
                            let duration = std::time::Duration::from_millis(300);

                            cx.spawn({
                                let section_state = section_state.clone();
                                async move |_, cx| {
                                    cx.background_executor().timer(duration).await;
                                    let _ = section_state.update(cx, |state, _| {
                                        *state = section;
                                    });
                                }
                            })
                            .detach();

                            this.child(section_el)
                                .with_animation(
                                    ElementId::NamedInteger("section_slide".into(), section as u64),
                                    Animation::new(duration).with_easing(ease_in_out_expo()),
                                    move |this, delta| {
                                        let offset = 360.0 * direction * (1.0 - delta);
                                        this.top(px(offset)).opacity(delta)
                                    },
                                )
                                .into_any_element()
                        }
                    }),
            )
        // .child(vlist(
        //     cx.entity(),
        //     "library",
        //     heights.clone(),
        //     scroll_handle,
        //     self.list_controller.clone(),
        //     move |_this, range, _, cx| {
        //         let len = rows.len();

        //         let start = range.start.saturating_sub(THUMBNAIL_MARGIN);
        //         let end = (range.end + THUMBNAIL_MARGIN).min(len);

        //         let thumb_track_ids: Vec<TrackId> = (start..end)
        //             .filter_map(|idx| match &rows[idx] {
        //                 LibraryRow::TrackRow(_, id) => Some(*id),
        //                 _ => None,
        //             })
        //             .collect();

        //         controller.request_track_thumbnails(&thumb_track_ids, cx);

        //         range
        //             .map(|idx| match &rows[idx] {
        //                 LibraryRow::Header(kind) => render_header(kind, heights[idx], cx),

        //                 LibraryRow::PlaylistGridRow(ids) => {
        //                     render_playlist_grid(ids, heights[idx], cx)
        //                 }

        //                 LibraryRow::TrackTableHeader => {
        //                     render_track_table_header(heights[idx], cx)
        //                 }

        //                 LibraryRow::TrackRow(i, id) => {
        //                     Self::render_track(*i, id, heights[idx], cx)
        //                 }

        //                 LibraryRow::Empty(kind) => match kind {
        //                     HeaderKind::Playlists => div()
        //                         .w_full()
        //                         .h_48()
        //                         .flex()
        //                         .items_center()
        //                         .justify_center()
        //                         .text_lg()
        //                         .text_color(theme.library_empty_text)
        //                         .child("No playlists loaded."),
        //                     HeaderKind::Tracks => div()
        //                         .w_full()
        //                         .h_48()
        //                         .flex()
        //                         .items_center()
        //                         .justify_center()
        //                         .text_lg()
        //                         .text_color(theme.library_empty_text)
        //                         .child("No tracks loaded."),
        //                     HeaderKind::Albums => div()
        //                         .w_full()
        //                         .h_48()
        //                         .flex()
        //                         .items_center()
        //                         .justify_center()
        //                         .text_lg()
        //                         .text_color(theme.library_empty_text)
        //                         .child("No albums loaded."),
        //                 },
        //             })
        //             .collect::<Vec<_>>()
        //     },
        // ))
        // .child(floating_scrollbar(
        //     "library_scrollbar",
        //     self.scroll_handle.clone(),
        //     RightPad::Pad,
        // ))
    }
}

impl Global for LibrarySection {}
