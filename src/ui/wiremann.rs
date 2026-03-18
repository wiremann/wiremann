use crate::controller::Controller;
use crate::ui::components;
use crate::ui::components::controlbar::ControlBar;
use crate::ui::components::slider::{SliderEvent, SliderState};
use crate::ui::helpers::slider_to_secs;
use crate::ui::theme::Theme;
use components::{image_cache::ImageCache, pages::{library::LibraryPage, player::PlayerPage}, titlebar::Titlebar, Page};
use gpui::{div, AppContext, BorrowAppContext, Context, Entity, InteractiveElement, IntoElement, ParentElement, Render, Styled, Window};

pub struct Wiremann {
    pub titlebar: Entity<Titlebar>,
    pub player_page: Entity<PlayerPage>,
    pub library_page: Entity<LibraryPage>,
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

        let titlebar = cx.new(|cx| Titlebar::new(cx));
        let controlbar = cx.new(|_| ControlBar::new(playback_slider_state, vol_slider_state));
        let player_page = cx.new(|cx| PlayerPage::new(cx, controlbar));
        let library_page = cx.new(|cx| LibraryPage::new(cx));

        cx.global::<Controller>().load_cached_app_state();

        Self {
            titlebar,
            player_page,
            library_page,
        }
    }
}

impl Render for Wiremann {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>();
        div()
            .id("main_container")
            .size_full()
            .font_family("Space Grotesk")
            .flex()
            .flex_col()
            .justify_center()
            .items_center()
            .bg(theme.bg_main)
            .child(self.titlebar.clone())
            .child(match cx.global::<Page>() {
                Page::Player => div().w_full().h_full().child(self.player_page.clone()),
                Page::Library => div().w_full().h_full().child(self.library_page.clone()),
                _ => div(),
            })
    }
}
