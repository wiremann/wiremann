use crate::{
    controller::{Controller, state::PlaybackStatus},
    ui::{
        components::{
            icons::{Icons, icon},
            image_cache::ImageCache,
        },
        pages::player::controlbar::ControlBar,
        theme::Theme,
        wiremann::Wiremann,
    },
};
use gpui::{
    App, Context, Entity, FontWeight, IntoElement, ObjectFit, Pixels, Render, Size, Styled, Window,
    div, img, px, size,
};

/// Global toggle for "pop out" (compact player window) mode.
///
/// When enabled the window shrinks down to a minimal always-playable player
/// and the app chrome (navbar, window controls, library/player pages) is
/// replaced by the compact [`PopOutPlayer`]. The previous window size is
/// remembered so exiting restores the original window.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct PopOutState {
    pub enabled: bool,
    pub previous_size: Option<Size<Pixels>>,
}


/// Handle to the root [`Wiremann`] view, used to repaint the tree after
/// toggling pop-out mode.
#[derive(Clone)]
pub struct PopOutHandle(pub Entity<Wiremann>);


/// The fixed size of the pop-out player window.
pub fn pop_out_size() -> Size<Pixels> {
    size(px(460.0), px(717.0))
}

/// Toggles pop-out mode, resizing the window and remembering the previous size.
pub fn toggle_pop_out(window: &mut Window, cx: &mut App) {
    let state = cx.global_mut::<PopOutState>();

    if state.enabled {
        state.enabled = false;
        let size = state.previous_size.take().unwrap_or_else(pop_out_size);
        window.resize(size);
    } else {
        state.enabled = true;
        state.previous_size = Some(window.bounds().size);
        window.resize(pop_out_size());
    }

    let root = cx.global::<PopOutHandle>().clone();
    root.0.update(cx, |_, cx| cx.notify());
}

#[derive(Clone)]
pub struct PopOutPlayer {
    pub controlbar: Entity<ControlBar>,
}

impl PopOutPlayer {
    #[must_use]
    pub fn new(controlbar: Entity<ControlBar>) -> Self {
        Self { controlbar }
    }
}

impl Render for PopOutPlayer {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let controller = cx.global::<Controller>().clone();
        let state = controller.state.read(cx);
        let thumbnail = cx.global::<ImageCache>().current.clone();

        let current = state.playback.current.and_then(|id| state.library.tracks.get(&id));

        let (title, artist) = match current {
            Some(track) => (
                track.title.to_string(),
                track
                    .artists(&state.library)
                    .map(|a| a.name.to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
            ),
            None => (
                String::from("No track selected"),
                String::from("Browse your library to find music"),
            ),
        };

        let is_playing = state.playback.status == PlaybackStatus::Playing;

        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(theme.app_bg)
            .child(
                div()
                    .id("popout_art")
                    .flex_1()
                    .min_h_0()
                    .w_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .px_8()
                    .child(match thumbnail {
                        Some(image) => img(image)
                            .size_full()
                            .object_fit(ObjectFit::Contain)
                            .rounded_xl()
                            .border_1()
                            .border_color(theme.border)
                            .into_any_element(),
                        None => img("icons/placeholder.svg")
                            .size_full()
                            .object_fit(ObjectFit::Contain)
                            .rounded_xl()
                            .border_1()
                            .border_color(theme.border)
                            .into_any_element(),
                    }),
            )
            .child(
                div()
                    .id("popout_info")
                    .mt_4()
                    .w_full()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap_1()
                    .child(
                        div()
                            .text_lg()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(theme.player_title_text)
                            .max_w_96()
                            .truncate()
                            .child(title),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(theme.player_artist_text)
                            .max_w_96()
                            .truncate()
                            .child(artist),
                    ),
            )
            .child(
                div()
                    .id("popout_transport")
                    .mt_4()
                    .flex()
                    .items_center()
                    .justify_center()
                    .gap_4()
                    .child(
                        div()
                            .id("popout_prev")
                            .p_3()
                            .rounded_md()
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_color(theme.player_icons_text)
                            .hover(|this| {
                                this.bg(theme.player_icons_bg_hover)
                                    .text_color(theme.player_icons_text_hover)
                            })
                            .cursor_pointer()
                            .on_click(|_, _, cx| {
                                cx.global::<Controller>().clone().prev(cx);
                            })
                            .child(icon(Icons::Prev).size_4()),
                    )
                    .child(
                        div()
                            .id("popout_play_pause")
                            .p_4()
                            .rounded_full()
                            .flex()
                            .items_center()
                            .justify_center()
                            .bg(theme.player_play_pause_bg)
                            .hover(|this| this.bg(theme.player_play_pause_hover))
                            .text_color(theme.player_play_pause_text)
                            .cursor_pointer()
                            .on_click(|_, _, cx| {
                                match cx.global::<Controller>().state.read(cx).playback.status {
                                    PlaybackStatus::Paused | PlaybackStatus::Stopped => {
                                        cx.global::<Controller>().play();
                                    }
                                    PlaybackStatus::Playing => {
                                        cx.global::<Controller>().pause();
                                    }
                                }
                            })
                            .child(if is_playing {
                                icon(Icons::Pause).size_5()
                            } else {
                                icon(Icons::Play).size_5()
                            }),
                    )
                    .child(
                        div()
                            .id("popout_next")
                            .p_3()
                            .rounded_md()
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_color(theme.player_icons_text)
                            .hover(|this| {
                                this.bg(theme.player_icons_bg_hover)
                                    .text_color(theme.player_icons_text_hover)
                            })
                            .cursor_pointer()
                            .on_click(|_, _, cx| {
                                cx.global::<Controller>().clone().next(cx);
                            })
                            .child(icon(Icons::Next).size_4()),
                    ),
            )
            .child(
                div()
                    .id("popout_controlbar")
                    .w_full()
                    .px_6()
                    .pt_2()
                    .pb_4()
                    .child(self.controlbar.clone()),
            )
    }
}
