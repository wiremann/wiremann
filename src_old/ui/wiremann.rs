use super::{
    components::{
        controlbar::ControlBar,
        pages::player::PlayerPage,
        slider::{SliderEvent, SliderState},
        titlebar::Titlebar,
    },
    image_cache::ImageCache,
    theme::Theme,
};
use crate::{audio::engine::PlaybackState, controller::player::Controller, ui::components::Page};
use gpui::*;

pub struct Wiremann {
    pub titlebar: Entity<Titlebar>,
    pub player_page: Entity<PlayerPage>,
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
                    cx.global::<Controller>().volume(value.start());
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
                    if controller.player_state.state == PlaybackState::Playing {
                        if let Some(meta) = controller.player_state.clone().meta {
                            controller.seek(slider_to_secs(value.start(), meta.duration));
                        }
                    }

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

        // cx.global::<Controller>()
        //     .load_playlist("E:\\music\\violence ft. doomguy".to_string());
        // cx.global::<Controller>()
        //     .load("E:\\music\\violence ft. doomguy\\468 - GIVE ME A REASON.mp3".to_string());

        cx.global::<Controller>().get_app_state_cache();

        Self {
            titlebar,
            player_page,
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
                _ => div(),
            })
    }
}

fn slider_to_secs(slider: f32, duration_secs: u64) -> u64 {
    ((slider.clamp(0.0, 100.0) / 100.0) * duration_secs as f32) as u64
}
