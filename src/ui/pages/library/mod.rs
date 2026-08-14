mod helpers;
mod routes;
mod sidebar;

use crate::controller::state::{AlbumId, ArtistId, PlaylistId};
use crate::ui::animations::ease_in_out_expo;
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
use crate::ui::pages::library::routes::tracks::TracksSection;
use crate::ui::pages::library::sidebar::{Sidebar, SidebarBounds, SidebarIndicator};
use crate::ui::theme::Theme;
use gpui::prelude::FluentBuilder;
use gpui::{
    Animation, AnimationExt, App, AppContext, Context, ElementId, Entity, Global,
    InteractiveElement, IntoElement, ParentElement, Render, ScrollHandle, Styled,
    UniformListScrollHandle, Window, div, px,
};

#[repr(u64)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LibraryRoutes {
    // Discovery
    Home,
    Favorites,

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

#[derive(Clone)]
pub struct LibraryPage {
    sidebar: Entity<Sidebar>,
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
}

impl LibraryRoutes {
    pub const fn index(self) -> i32 {
        match self {
            Self::Home => 0,
            Self::Favorites => 1,

            Self::Tracks => 2,
            Self::Albums => 3,
            Self::Artists => 4,
            Self::Playlists => 5,

            Self::Album(_) => 6,
            Self::Artist(_) => 7,
            Self::Playlist(_) => 8,

            Self::Settings => 9,
            Self::Plugins => 10,
        }
    }
}

impl LibraryPage {
    pub fn new(cx: &mut App) -> Self {
        cx.set_global(SidebarIndicator {
            top: 0.0,
            height: 32.0,
        });

        cx.set_global(SidebarBounds { top: 0.0 });

        LibraryPage {
            sidebar: cx.new(|_| Sidebar),
            album: cx.new(|cx| AlbumViewSection {
                album_id: cx.new(|_| None),
                scroll_handle: UniformListScrollHandle::new(),
            }),
            albums: cx.new(|_| AlbumsSection {
                scroll_handle: ScrollHandle::new(),
                grid_controller: VirtualGridScrollController::new(),
            }),
            artist: cx.new(|cx| ArtistViewSection {
                artist_id: cx.new(|_| None),
                scroll_handle: UniformListScrollHandle::new(),
            }),
            artists: cx.new(|_| ArtistsSection {
                scroll_handle: ScrollHandle::new(),
                grid_controller: VirtualGridScrollController::new(),
            }),
            playlist: cx.new(|cx| PlaylistViewSection {
                playlist_id: cx.new(|_| None),
                menu_open: cx.new(|_| false),
                menu_button_bounds: None,
                root_bounds: None,
                scroll_handle: UniformListScrollHandle::new(),
            }),
            playlists: cx.new(|_| PlaylistsSection {
                scroll_handle: ScrollHandle::new(),
                grid_controller: VirtualGridScrollController::new(),
            }),
            favorites: cx.new(|_| FavoritesSection),
            home: cx.new(|_| HomeSection),
            tracks: cx.new(|_| TracksSection {
                scroll_handle: UniformListScrollHandle::new(),
            }),
            plugins: cx.new(|_| PluginsSection),
            settings: cx.new(|_| SettingsSection),
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

        let section_el = match section {
            LibraryRoutes::Home => div().w_full().h_full().child(self.home.clone()),
            LibraryRoutes::Favorites => div().w_full().h_full().child(self.favorites.clone()),
            LibraryRoutes::Tracks => div().w_full().h_full().child(self.tracks.clone()),
            LibraryRoutes::Albums => div().w_full().h_full().child(self.albums.clone()),
            LibraryRoutes::Artists => div().w_full().h_full().child(self.artists.clone()),
            LibraryRoutes::Playlists => div().w_full().h_full().child(self.playlists.clone()),
            LibraryRoutes::Settings => div().w_full().h_full().child(self.settings.clone()),
            LibraryRoutes::Plugins => div().w_full().h_full().child(self.plugins.clone()),
            LibraryRoutes::Album(id) => {
                self.album.update(cx, |this, cx| {
                    this.album_id.update(cx, |this, _| *this = Some(id));
                    cx.notify()
                });
                div().w_full().h_full().child(self.album.clone())
            }
            LibraryRoutes::Artist(id) => {
                self.artist.update(cx, |this, cx| {
                    this.artist_id.update(cx, |this, _| *this = Some(id));
                    cx.notify()
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
                    cx.notify()
                });
                div().w_full().h_full().child(self.playlist.clone())
            }
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
                                    ElementId::Name(format!("section_slide_{:?}", section).into()),
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
    }
}

impl Global for LibraryRoutes {}
