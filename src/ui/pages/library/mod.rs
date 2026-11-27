pub(crate) mod dock;
mod helpers;
mod routes;
mod sidebar;
mod sort;

use crate::controller::state::{AlbumId, ArtistId, PlaylistId};
use crate::ui::animations::page_transition;
use crate::ui::components::resize_handle::{ResizeHandle, ResizeSide, ResizeState};
use crate::ui::components::virtual_grid::VirtualGridScrollController;
use crate::ui::pages::library::routes::album::AlbumViewSection;
use crate::ui::pages::library::routes::albums::AlbumsSection;
use crate::ui::pages::library::routes::artist::ArtistViewSection;
use crate::ui::pages::library::routes::artists::ArtistsSection;
use crate::ui::pages::library::routes::favorites::FavoritesSection;
use crate::ui::pages::library::routes::home::HomeSection;
use crate::ui::pages::library::routes::playlist::PlaylistViewSection;
use crate::ui::pages::library::routes::playlists::PlaylistsSection;
use crate::ui::pages::library::routes::plugins::PluginsSection;
use crate::ui::pages::library::routes::settings::SettingsSection;
use crate::ui::pages::library::routes::stats::StatsSection;
use crate::ui::pages::library::routes::tracks::TracksSection;
use crate::ui::pages::library::sidebar::{COLLAPSED_SIDEBAR_WIDTH, Sidebar};
use crate::ui::pages::library::sort::{SortControlsState, SortField, SortMenu};
use crate::ui::theme::Theme;
use gpui::prelude::FluentBuilder;
use gpui::{
    App, AppContext, Context, ElementId, Entity, Global, InteractiveElement, IntoElement,
    ParentElement, Render, ScrollHandle, Styled, UniformListScrollHandle, Window, div, px,
};
use wgpui_motion::prelude::MotionExt;

#[repr(u64)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LibraryRoutes {
    // Discovery
    Home,
    Favorites,
    Stats,

    // Collection
    Tracks,
    Albums,
    Artists,
    Playlists,

    // Individual item views
    Album(AlbumId),
    Artist(ArtistId),
    Playlist(PlaylistId),

    // System
    Settings,
    Plugins,
}

#[derive(Clone, Copy)]
struct LibraryTransitionState {
    settled: LibraryRoutes,
    from: LibraryRoutes,
    target: LibraryRoutes,
    generation: u64,
}

#[derive(Clone)]
pub struct LibraryPage {
    sidebar: Entity<Sidebar>,
    sidebar_collapsed: Entity<bool>,
    sidebar_width: Entity<ResizeState>,
    album: Entity<AlbumViewSection>,
    albums: Entity<AlbumsSection>,
    artist: Entity<ArtistViewSection>,
    artists: Entity<ArtistsSection>,
    playlist: Entity<PlaylistViewSection>,
    playlists: Entity<PlaylistsSection>,
    favorites: Entity<FavoritesSection>,
    home: Entity<HomeSection>,
    tracks: Entity<TracksSection>,
    plugins: Entity<PluginsSection>,
    settings: Entity<SettingsSection>,
    stats: Entity<StatsSection>,
}

impl LibraryRoutes {
    #[must_use]
    pub const fn index(self) -> i32 {
        match self {
            Self::Home => 0,
            Self::Favorites => 1,
            Self::Stats => 2,

            Self::Tracks => 3,
            Self::Albums => 4,
            Self::Artists => 5,
            Self::Playlists => 6,

            Self::Album(_) => 7,
            Self::Artist(_) => 8,
            Self::Playlist(_) => 9,

            Self::Settings => 10,
            Self::Plugins => 11,
        }
    }
}

impl LibraryPage {
    pub fn new(cx: &mut App) -> Self {
        let sidebar_collapsed = cx.new(|_| false);
        LibraryPage {
            sidebar: cx.new(|_| Sidebar {
                collapsed: sidebar_collapsed.clone(),
            }),
            sidebar_collapsed,
            sidebar_width: cx.new(|_| ResizeState::new(ResizeSide::Left, 256.0, 220.0, 420.0)),
            album: cx.new(|cx| AlbumViewSection {
                album_id: cx.new(|_| None),
                scroll_handle: UniformListScrollHandle::new(),
            }),
            albums: cx.new(|cx| {
                let sort = cx.new(|_| SortControlsState {
                    field: SortField::Name,
                    descending: false,
                });
                AlbumsSection {
                    scroll_handle: ScrollHandle::new(),
                    grid_controller: VirtualGridScrollController::new(),
                    sort_menu: cx.new(|_| {
                        SortMenu::new(
                            "albums_sort",
                            sort.clone(),
                            AlbumsSection::SORT_FIELDS,
                            px(150.0),
                            px(72.0),
                            px(32.0),
                        )
                    }),
                    sort,
                    sorted_ids: Default::default(),
                    sort_cache_key: None,
                }
            }),
            artist: cx.new(|cx| ArtistViewSection {
                artist_id: cx.new(|_| None),
                scroll_handle: UniformListScrollHandle::new(),
            }),
            artists: cx.new(|cx| {
                let sort = cx.new(|_| SortControlsState {
                    field: SortField::Name,
                    descending: false,
                });
                ArtistsSection {
                    scroll_handle: ScrollHandle::new(),
                    grid_controller: VirtualGridScrollController::new(),
                    sort_menu: cx.new(|_| {
                        SortMenu::new(
                            "artists_sort",
                            sort.clone(),
                            ArtistsSection::SORT_FIELDS,
                            px(116.0),
                            px(72.0),
                            px(32.0),
                        )
                    }),
                    sort,
                    sorted_ids: Default::default(),
                    sort_cache_key: None,
                }
            }),
            playlist: cx.new(|cx| PlaylistViewSection {
                playlist_id: cx.new(|_| None),
                menu_open: cx.new(|_| false),
                menu_button_bounds: None,
                root_bounds: None,
                scroll_handle: UniformListScrollHandle::new(),
            }),
            playlists: cx.new(|cx| {
                let sort = cx.new(|_| SortControlsState {
                    field: SortField::Name,
                    descending: false,
                });
                PlaylistsSection {
                    scroll_handle: ScrollHandle::new(),
                    grid_controller: VirtualGridScrollController::new(),
                    sort_menu: cx.new(|_| {
                        SortMenu::new(
                            "playlists_sort",
                            sort.clone(),
                            PlaylistsSection::SORT_FIELDS,
                            px(116.0),
                            px(72.0),
                            px(220.0),
                        )
                    }),
                    sort,
                    sorted_ids: Default::default(),
                    sort_cache_key: None,
                }
            }),
            favorites: cx.new(|_| FavoritesSection {
                scroll_handle: UniformListScrollHandle::new(),
            }),
            home: cx.new(|cx| HomeSection {
                scroll_handle: ScrollHandle::new(),
                avail_width: cx.new(|_| 800.0),
            }),
            tracks: cx.new(|cx| {
                let sort = cx.new(|_| SortControlsState {
                    field: SortField::Title,
                    descending: false,
                });
                TracksSection {
                    scroll_handle: UniformListScrollHandle::new(),
                    sort_menu: cx.new(|_| {
                        SortMenu::new(
                            "tracks_sort",
                            sort.clone(),
                            TracksSection::SORT_FIELDS,
                            px(184.0),
                            px(72.0),
                            px(48.0),
                        )
                    }),
                    sort,
                    sorted_ids: Default::default(),
                    sort_cache_key: None,
                }
            }),
            plugins: cx.new(|_| PluginsSection),
            settings: cx.new(|_| SettingsSection),
            stats: cx.new(|_| StatsSection {
                scroll_handle: ScrollHandle::new(),
            }),
        }
    }

    pub fn notify_image_views(&mut self, cx: &mut Context<Self>) {
        match *cx.global::<LibraryRoutes>() {
            LibraryRoutes::Home => self.home.update(cx, |_, cx| cx.notify()),
            LibraryRoutes::Favorites => self.favorites.update(cx, |_, cx| cx.notify()),
            LibraryRoutes::Stats => self.stats.update(cx, |_, cx| cx.notify()),
            LibraryRoutes::Tracks => self.tracks.update(cx, |_, cx| cx.notify()),
            LibraryRoutes::Albums => self.albums.update(cx, |_, cx| cx.notify()),
            LibraryRoutes::Artists => self.artists.update(cx, |_, cx| cx.notify()),
            LibraryRoutes::Playlists => self.playlists.update(cx, |_, cx| cx.notify()),
            LibraryRoutes::Album(_) => self.album.update(cx, |_, cx| cx.notify()),
            LibraryRoutes::Artist(_) => self.artist.update(cx, |_, cx| cx.notify()),
            LibraryRoutes::Playlist(_) => self.playlist.update(cx, |_, cx| cx.notify()),
            LibraryRoutes::Settings | LibraryRoutes::Plugins => {}
        }
    }
}

impl Render for LibraryPage {
    #[allow(clippy::too_many_lines)]
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();

        // let controller = cx.global::<Controller>().clone();
        // let state = controller.state.read(cx);

        // let tracks_fp = fingerprint_tracks(state.library.tracks.keys().copied());
        // let playlists_fp = fingerprint_playlists(state.library.playlists.keys().copied());

        // let combined_fp = tracks_fp ^ playlists_fp;

        let section = *cx.global::<LibraryRoutes>();

        let sidebar_width = self.sidebar_width.read(cx).width();
        let sidebar_resizing = self.sidebar_width.read(cx).is_resizing();
        let sidebar_collapsed = *self.sidebar_collapsed.read(cx);

        let section_el = match section {
            LibraryRoutes::Home => div().w_full().h_full().child(self.home.clone()),
            LibraryRoutes::Favorites => div().w_full().h_full().child(self.favorites.clone()),
            LibraryRoutes::Stats => div().w_full().h_full().child(self.stats.clone()),
            LibraryRoutes::Tracks => div().w_full().h_full().child(self.tracks.clone()),
            LibraryRoutes::Albums => div().w_full().h_full().child(self.albums.clone()),
            LibraryRoutes::Artists => div().w_full().h_full().child(self.artists.clone()),
            LibraryRoutes::Playlists => div().w_full().h_full().child(self.playlists.clone()),
            LibraryRoutes::Settings => div().w_full().h_full().child(self.settings.clone()),
            LibraryRoutes::Plugins => div().w_full().h_full().child(self.plugins.clone()),
            LibraryRoutes::Album(id) => {
                self.album.update(cx, |this, cx| {
                    this.album_id.update(cx, |this, _| *this = Some(id));
                    cx.notify();
                });
                div().w_full().h_full().child(self.album.clone())
            }
            LibraryRoutes::Artist(id) => {
                self.artist.update(cx, |this, cx| {
                    this.artist_id.update(cx, |this, _| *this = Some(id));
                    cx.notify();
                });
                div().w_full().h_full().child(self.artist.clone())
            }
            LibraryRoutes::Playlist(id) => {
                self.playlist.update(cx, |this, cx| {
                    if *this.playlist_id.read(cx) != Some(id) {
                        this.playlist_id.update(cx, |this, _| *this = Some(id));
                        this.menu_open.update(cx, |open, cx| {
                            *open = false;
                            cx.notify();
                        });
                    }
                    cx.notify();
                });
                div().w_full().h_full().child(self.playlist.clone())
            }
        };
        let transition_state =
            window.use_keyed_state("library_transition", cx, |_, _| LibraryTransitionState {
                settled: section,
                from: section,
                target: section,
                generation: 0,
            });
        if transition_state.read(cx).target != section {
            transition_state.update(cx, |state, _| {
                state.from = state.target;
                state.target = section;
                state.generation = state.generation.wrapping_add(1);
            });
        }
        let transition = *transition_state.read(cx);

        let direction = (section.index() - transition.from.index()).signum() as f32;
        let is_transitioning =
            transition.target != transition.settled || transition.from != transition.target;

        let sidebar_container = div()
            .w(px(if sidebar_collapsed {
                COLLAPSED_SIDEBAR_WIDTH
            } else {
                sidebar_width
            }))
            .h_full()
            .flex_shrink_0()
            .overflow_hidden()
            .child(self.sidebar.clone());
        let sidebar_container = if sidebar_resizing {
            sidebar_container.into_any_element()
        } else {
            sidebar_container
                .motion(ElementId::NamedInteger(
                    "library_sidebar_width".into(),
                    self.sidebar_width.read(cx).motion_generation(),
                ))
                .transition(page_transition())
                .into_any_element()
        };

        let library_section = div()
            .id("library_page_content")
            .w_full()
            .h_full()
            .map(move |this| {
                if !is_transitioning {
                    this.child(section_el).into_any_element()
                } else {
                    let transition_state = transition_state.clone();
                    let generation = transition.generation;

                    this.child(section_el)
                        .motion(ElementId::NamedInteger(
                            "library_section".into(),
                            generation,
                        ))
                        .transition(page_transition())
                        .enter(move |style| style.top(px(360.0 * direction)).opacity(0.0))
                        .base(|style| style.top(px(0.0)).opacity(1.0))
                        .on_settle(move |_, _, cx| {
                            transition_state.update(cx, |state, cx| {
                                if state.generation == generation && state.target == section {
                                    state.settled = section;
                                    state.from = section;
                                    cx.notify();
                                }
                            });
                        })
                        .into_any_element()
                }
            });

        div()
            .size_full()
            .bg(theme.library_bg)
            .text_color(theme.library_text)
            .px_2()
            .pt_2()
            .flex()
            .child(sidebar_container)
            .when(!sidebar_collapsed, |this| {
                this.child(ResizeHandle::new(&self.sidebar_width))
            })
            .child(
                div()
                    .id("library_content_area")
                    .w_full()
                    .h_full()
                    .child(library_section),
            )
    }
}

impl Global for LibraryRoutes {}
