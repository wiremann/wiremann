use gpui::{
    Context, ElementId, FontWeight, Global, InteractiveElement, IntoElement, ParentElement, Render,
    StatefulInteractiveElement, Styled, Window, div, px,
};
use wgpui_motion::prelude::MotionExt;

use crate::ui::{
    animations::page_transition,
    components::{
        bounds_observer::observe_bounds,
        icons::{Icons, icon},
    },
    pages::library::LibraryRoutes,
    theme::Theme,
};

pub struct Sidebar;

#[derive(Clone, Copy)]
pub struct SidebarIndicator {
    pub top: f32,
    pub height: f32,
    pub route: LibraryRoutes,
}

#[derive(Clone, Copy)]
struct IndicatorTransitionState {
    settled: LibraryRoutes,
    settled_top: f32,
    settled_height: f32,
    target: LibraryRoutes,
    target_top: f32,
    target_height: f32,
    generation: u64,
}

#[derive(Clone, Copy, Default)]
pub struct SidebarBounds {
    pub top: f32,
}

impl Global for SidebarIndicator {}
impl Global for SidebarBounds {}

impl Sidebar {
    fn section_header(text: &'static str, theme: &Theme) -> impl IntoElement {
        div()
            .px_6()
            .pt_8()
            .pb_4()
            .text_xs()
            .font_weight(FontWeight::MEDIUM)
            .text_color(theme.library_sidebar_group_text)
            .tracking_widest()
            .child(text)
    }

    fn item(
        icon_type: Icons,
        text: &'static str,
        section: LibraryRoutes,
        current: LibraryRoutes,
        theme: &Theme,
    ) -> impl IntoElement {
        let active = current == section;

        observe_bounds(
            format!("sidebar_bounds_{text}"),
            div()
                .id(format!("library_sidebar_item_{text}"))
                .w_full()
                .px_6()
                .py_3()
                .flex()
                .items_center()
                .gap_3()
                .cursor_pointer()
                .hover(|this| this.bg(theme.library_sidebar_item_bg_hover))
                .on_click(move |_, _, cx| {
                    *cx.global_mut::<LibraryRoutes>() = section;
                })
                .child(
                    div()
                        .relative()
                        .size_6()
                        .flex_none()
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(
                            div()
                                .absolute()
                                .size(px(44.0))
                                .left(px(-10.0))
                                .top(px(-10.0))
                                .rounded_md()
                                .bg(theme.library_sidebar_item_bg_active)
                                .motion(format!("sidebar_icon_highlight_{text}"))
                                .transition(page_transition())
                                .enter(|style| style.opacity(0.0))
                                .base(move |style| style.opacity(f32::from(active))),
                        )
                        .child(icon(icon_type).size_5().text_color(if active {
                            theme.library_sidebar_bg
                        } else {
                            theme.library_sidebar_item_text
                        })),
                )
                .child(
                    div()
                        .child(text)
                        .text_sm()
                        .font_weight(if active {
                            FontWeight::MEDIUM
                        } else {
                            FontWeight::NORMAL
                        })
                        .text_color(if active {
                            theme.library_sidebar_item_text_active
                        } else {
                            theme.library_sidebar_item_text
                        }),
                ),
            move |bounds, window, cx| {
                if *cx.global::<LibraryRoutes>() == section {
                    let sidebar_top = cx.global::<SidebarBounds>().top;
                    let indicator = cx.global_mut::<SidebarIndicator>();

                    let top = bounds.top().to_f64() as f32 - sidebar_top;
                    let height = bounds.size.height.to_f64() as f32;
                    if indicator.route != section
                        || indicator.top != top
                        || indicator.height != height
                    {
                        indicator.top = top;
                        indicator.height = height;
                        indicator.route = section;
                        window.refresh();
                    }
                }
            },
        )
    }
}

impl Render for Sidebar {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let current = *cx.global::<LibraryRoutes>();

        let indicator_data = *cx.global::<SidebarIndicator>();

        let indicator_state =
            window.use_keyed_state("sidebar_indicator", cx, |_, _| IndicatorTransitionState {
                settled: current,
                settled_top: indicator_data.top,
                settled_height: indicator_data.height,
                target: current,
                target_top: indicator_data.top,
                target_height: indicator_data.height,
                generation: 0,
            });

        let indicator_matches_route = indicator_data.route == current;
        if indicator_matches_route {
            let state = *indicator_state.read(cx);
            if state.target != current {
                indicator_state.update(cx, |state, _| {
                    state.target = current;
                    state.target_top = indicator_data.top;
                    state.target_height = indicator_data.height;
                    state.generation = state.generation.wrapping_add(1);
                });
            } else if state.settled == current
                && (state.settled_top != indicator_data.top
                    || state.settled_height != indicator_data.height)
            {
                indicator_state.update(cx, |state, _| {
                    state.settled_top = indicator_data.top;
                    state.settled_height = indicator_data.height;
                    state.target_top = indicator_data.top;
                    state.target_height = indicator_data.height;
                });
            }
        }
        let transition = *indicator_state.read(cx);
        let is_transitioning = indicator_matches_route && transition.target != transition.settled;
        let settled_dot_top =
            transition.settled_top + (transition.settled_height - 6.0).max(0.0) / 2.0;
        let target_dot_top =
            transition.target_top + (transition.target_height - 6.0).max(0.0) / 2.0;

        observe_bounds(
            "sidebar_root",
            div()
                .relative()
                .w_full()
                .h_full()
                .bg(theme.library_sidebar_bg)
                .flex()
                .flex_col()
                .child({
                    let indicator = div()
                        .absolute()
                        .right_6()
                        .size(px(6.0))
                        .rounded_full()
                        .bg(theme.library_sidebar_indicator);
                    if !is_transitioning {
                        indicator.top(px(settled_dot_top)).into_any_element()
                    } else {
                        let indicator_state = indicator_state.clone();
                        let generation = transition.generation;
                        indicator
                            .motion(ElementId::NamedInteger(
                                "sidebar_indicator".into(),
                                generation,
                            ))
                            .transition(page_transition())
                            .enter(move |style| style.top(px(settled_dot_top)))
                            .base(move |style| style.top(px(target_dot_top)))
                            .on_settle(move |_, _, cx| {
                                indicator_state.update(cx, |state, cx| {
                                    if state.generation == generation && state.target == current {
                                        state.settled = current;
                                        state.settled_top = state.target_top;
                                        state.settled_height = state.target_height;
                                        cx.notify();
                                    }
                                });
                            })
                            .into_any_element()
                    }
                })
                .child(Self::section_header("DISCOVERY", &theme))
                .child(Self::item(
                    Icons::Home,
                    "Home",
                    LibraryRoutes::Home,
                    current,
                    &theme,
                ))
                .child(Self::item(
                    Icons::Heart,
                    "Favorites",
                    LibraryRoutes::Favorites,
                    current,
                    &theme,
                ))
                .child(Self::item(
                    Icons::Chart,
                    "Stats",
                    LibraryRoutes::Stats,
                    current,
                    &theme,
                ))
                .child(Self::section_header("COLLECTION", &theme))
                .child(Self::item(
                    Icons::Music,
                    "Tracks",
                    LibraryRoutes::Tracks,
                    current,
                    &theme,
                ))
                .child(Self::item(
                    Icons::Disc,
                    "Albums",
                    LibraryRoutes::Albums,
                    current,
                    &theme,
                ))
                .child(Self::item(
                    Icons::User,
                    "Artists",
                    LibraryRoutes::Artists,
                    current,
                    &theme,
                ))
                .child(Self::item(
                    Icons::Playlist,
                    "Playlists",
                    LibraryRoutes::Playlists,
                    current,
                    &theme,
                ))
                .child(Self::section_header("SYSTEM", &theme))
                .child(Self::item(
                    Icons::Plugins,
                    "Plugins",
                    LibraryRoutes::Plugins,
                    current,
                    &theme,
                ))
                .child(Self::item(
                    Icons::Settings,
                    "Settings",
                    LibraryRoutes::Settings,
                    current,
                    &theme,
                ))
                .child(div().flex_grow()),
            |bounds, _, cx| {
                cx.global_mut::<SidebarBounds>().top = bounds.top().to_f64() as f32;
            },
        )
    }
}
