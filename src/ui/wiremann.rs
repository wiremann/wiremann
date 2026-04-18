use std::time::{Duration, Instant};

use crate::controller::Controller;
use crate::ui::animations::ease_in_out_expo;
use crate::ui::components::controlbar::ControlBar;
use crate::ui::components::pages::playlists::PlaylistsPage;
use crate::ui::components::slider::{SliderEvent, SliderState};
use crate::ui::components::toasts::scanning_status::ScanningStatus;
use crate::ui::components::toasts::{Toast, ToastKind, ToastManager};
use crate::ui::helpers::slider_to_secs;
use crate::ui::theme::Theme;
use crate::ui::{components, global_keybinds};
use components::{
    Page,
    image_cache::ImageCache,
    pages::{library::LibraryPage, player::PlayerPage},
    titlebar::Titlebar,
};
use gpui::prelude::FluentBuilder;
use gpui::{
    Animation, AnimationExt as _, AppContext, BorrowAppContext, Context, ElementId, Entity,
    InteractiveElement, IntoElement, ParentElement, Render, Styled, Window, div, px,
};

pub struct Wiremann {
    pub titlebar: Entity<Titlebar>,
    pub player_page: Entity<PlayerPage>,
    pub library_page: Entity<LibraryPage>,
    pub playlists_page: Entity<PlaylistsPage>,
    pub toast_manager: Entity<ToastManager>,
}

impl Wiremann {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let vol_slider_state = cx.new(|_| {
            SliderState::new()
                .min(0.0)
                .max(100.0)
                .default_value(100.0)
                .step(1.0)
        });

        let playback_slider_state = cx.new(|_| {
            SliderState::new()
                .min(0.0)
                .max(100.0)
                .default_value(0.0)
                .step(1.0)
        });

        cx.subscribe(
            &vol_slider_state,
            |_, _, event: &SliderEvent, cx| match event {
                SliderEvent::Change(value) => {
                    let controller = cx.global::<Controller>().clone();

                    controller.set_volume(*value / 100.0, cx);
                    cx.notify();
                }
            },
        )
        .detach();

        cx.subscribe(
            &playback_slider_state,
            |_, _, event: &SliderEvent, cx| match event {
                SliderEvent::Change(value) => {
                    let controller = cx.global::<Controller>();
                    let state = controller.state.read(cx);
                    let current = if let Some(id) = state.playback.current {
                        state.library.tracks.get(&id)
                    } else {
                        None
                    };

                    let duration = if let Some(track) = current {
                        track.duration
                    } else {
                        0
                    };

                    controller.seek(slider_to_secs(*value, duration));

                    cx.notify();
                }
            },
        )
        .detach();

        cx.set_global(Theme::default());
        cx.set_global(Page::Player);
        cx.set_global(ImageCache::default());
        let scanning_status = ScanningStatus::new(cx).clone();
        cx.set_global(scanning_status);

        global_keybinds::register_keybinds(cx);

        let titlebar = cx.new(|cx| Titlebar::new(cx));
        let controlbar = cx.new(|_| ControlBar::new(playback_slider_state, vol_slider_state));
        let player_page = cx.new(|cx| PlayerPage::new(cx, controlbar));
        let library_page = cx.new(|cx| LibraryPage::new(cx));
        let playlists_page = cx.new(|cx| PlaylistsPage::new(cx));
        let toast_manager = cx.new(|cx| ToastManager::new(cx));

        cx.global::<Controller>().load_cached_app_state();

        Self {
            titlebar,
            player_page,
            library_page,
            playlists_page,
            toast_manager,
        }
    }
}

impl Render for Wiremann {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();

        let page = *cx.global::<Page>();

        let page_state = window.use_keyed_state("page_transition", cx, |_, _| page);
        let prev_page = *page_state.read(cx);

        let direction = match (prev_page, page) {
            (Page::Library, Page::Player) | (Page::Player, Page::Playlists) => 1.0,
            (Page::Playlists, Page::Player) | (Page::Player, Page::Library) => -1.0,
            _ => 0.0,
        };

        let page_el = match page {
            Page::Player => div().w_full().h_full().child(self.player_page.clone()),
            Page::Library => div().w_full().h_full().child(self.library_page.clone()),
            Page::Playlists => div().w_full().h_full().child(self.playlists_page.clone()),
        };

        div()
            .id("main_container")
            .size_full()
            .font_family("Space Grotesk")
            .relative()
            .flex()
            .flex_col()
            .justify_center()
            .items_center()
            .bg(theme.app_bg)
            .child(self.titlebar.clone())
            .child(
                div()
                    .id("animation_container")
                    .w_full()
                    .h_full()
                    .map(move |this| {
                        if prev_page == page {
                            this.child(page_el).into_any_element()
                        } else {
                            let duration = std::time::Duration::from_millis(300);

                            cx.spawn({
                                let page_state = page_state.clone();
                                async move |_, cx| {
                                    cx.background_executor().timer(duration).await;
                                    () = page_state.update(cx, |state, _| {
                                        *state = page;
                                    });
                                }
                            })
                            .detach();

                            this.child(page_el)
                                .with_animation(
                                    ElementId::NamedInteger("page_slide".into(), page as u64),
                                    Animation::new(duration).with_easing(ease_in_out_expo()),
                                    move |this, delta| {
                                        let offset = 360.0 * direction * (1.0 - delta);
                                        this.left(px(offset)).opacity(delta)
                                    },
                                )
                                .into_any_element()
                        }
                    }),
            )
            .child(self.toast_manager.clone())
    }
}
