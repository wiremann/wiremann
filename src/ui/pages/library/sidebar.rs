use gpui::{
    Context, ElementId, FontWeight, InteractiveElement, IntoElement, ParentElement, Render,
    StatefulInteractiveElement, Styled, Window, div, px,
};
use wgpui_motion::prelude::MotionExt;

use crate::ui::{
    animations::page_transition,
    components::icons::{Icons, icon},
    pages::library::LibraryRoutes,
    theme::Theme,
};

pub struct Sidebar;

#[derive(Clone, Copy)]
struct SidebarItemMotion {
    from_active: bool,
    from_hovered: bool,
    target_active: bool,
    target_hovered: bool,
    generation: u64,
}

impl SidebarItemMotion {
    fn new(active: bool) -> Self {
        Self {
            from_active: active,
            from_hovered: false,
            target_active: active,
            target_hovered: false,
            generation: 0,
        }
    }

    fn transition_to(&mut self, active: bool, hovered: bool) {
        if self.target_active != active || self.target_hovered != hovered {
            self.from_active = self.target_active;
            self.from_hovered = self.target_hovered;
            self.target_active = active;
            self.target_hovered = hovered;
            self.generation = self.generation.wrapping_add(1);
        }
    }
}

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
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let active = current == section;
        let motion = window.use_keyed_state(format!("sidebar_item_motion_{text}"), cx, |_, _| {
            SidebarItemMotion::new(active)
        });
        let state = *motion.read(cx);
        if state.target_active != active {
            motion.update(cx, |state, _| {
                state.transition_to(active, state.target_hovered);
            });
        }
        let state = *motion.read(cx);
        let is_transitioning =
            state.from_active != state.target_active || state.from_hovered != state.target_hovered;

        let item_color = |active: bool, hovered: bool| {
            if active || hovered {
                theme.library_sidebar_item_text_active
            } else {
                theme.library_sidebar_item_text
            }
        };
        let icon_color = |active: bool, hovered: bool| {
            if active {
                theme.library_sidebar_bg
            } else {
                item_color(active, hovered)
            }
        };

        let highlight = div()
            .absolute()
            .size(px(44.0))
            .left(px(-10.0))
            .top(px(-10.0))
            .rounded_lg()
            .bg(theme.library_sidebar_item_bg_active);
        let highlight = if is_transitioning {
            highlight
                .motion(ElementId::NamedInteger(
                    format!("sidebar_icon_highlight_{text}").into(),
                    state.generation,
                ))
                .transition(page_transition())
                .enter(move |style| style.opacity(f32::from(state.from_active)))
                .base(move |style| style.opacity(f32::from(state.target_active)))
                .into_any_element()
        } else {
            highlight
                .opacity(f32::from(state.target_active))
                .into_any_element()
        };

        let icon_layer = |color, opacity| {
            div()
                .absolute()
                .inset_0()
                .flex()
                .items_center()
                .justify_center()
                .opacity(opacity)
                .child(icon(icon_type).size_5().text_color(color))
        };
        let icon_layers = if is_transitioning {
            div()
                .relative()
                .size_6()
                .flex_none()
                .child(highlight)
                .child(
                    icon_layer(icon_color(state.from_active, state.from_hovered), 1.0)
                        .motion(ElementId::NamedInteger(
                            format!("sidebar_icon_from_{text}").into(),
                            state.generation,
                        ))
                        .transition(page_transition())
                        .enter(|style| style.opacity(1.0))
                        .base(|style| style.opacity(0.0)),
                )
                .child(
                    icon_layer(icon_color(state.target_active, state.target_hovered), 0.0)
                        .motion(ElementId::NamedInteger(
                            format!("sidebar_icon_target_{text}").into(),
                            state.generation,
                        ))
                        .transition(page_transition())
                        .enter(|style| style.opacity(0.0))
                        .base(|style| style.opacity(1.0)),
                )
        } else {
            div()
                .relative()
                .size_6()
                .flex_none()
                .child(highlight)
                .child(icon_layer(
                    icon_color(state.target_active, state.target_hovered),
                    1.0,
                ))
        };

        let from_label_color = item_color(state.from_active, state.from_hovered);
        let target_label_color = item_color(state.target_active, state.target_hovered);
        let target_label = div()
            .text_sm()
            .font_weight(if state.target_active {
                FontWeight::MEDIUM
            } else {
                FontWeight::NORMAL
            })
            .text_color(target_label_color)
            .child(text);
        let label = if is_transitioning {
            let from_label = div()
                .absolute()
                .inset_0()
                .text_sm()
                .font_weight(if state.from_active {
                    FontWeight::MEDIUM
                } else {
                    FontWeight::NORMAL
                })
                .text_color(from_label_color)
                .child(text)
                .motion(ElementId::NamedInteger(
                    format!("sidebar_label_from_{text}").into(),
                    state.generation,
                ))
                .transition(page_transition())
                .enter(|style| style.opacity(1.0))
                .base(|style| style.opacity(0.0));
            let target_label = target_label
                .motion(ElementId::NamedInteger(
                    format!("sidebar_label_target_{text}").into(),
                    state.generation,
                ))
                .transition(page_transition())
                .enter(|style| style.opacity(0.0))
                .base(|style| style.opacity(1.0))
                .on_settle({
                    let motion = motion.clone();
                    let generation = state.generation;
                    move |_, _, cx| {
                        motion.update(cx, |state, cx| {
                            if state.generation == generation {
                                state.from_active = state.target_active;
                                state.from_hovered = state.target_hovered;
                                cx.notify();
                            }
                        });
                    }
                });
            div().relative().child(from_label).child(target_label)
        } else {
            div().child(target_label)
        };

        div()
            .id(format!("library_sidebar_item_{text}"))
            .w_full()
            .px_6()
            .py_3()
            .child(
                div()
                    .w_full()
                    .flex()
                    .items_center()
                    .gap_5()
                    .rounded_lg()
                    .cursor_pointer()
                    .hover(|this| this.bg(theme.library_sidebar_item_bg_hover))
                    .on_hover({
                        let motion = motion.clone();
                        move |hovered, _, cx| {
                            motion.update(cx, |state, cx| {
                                state.transition_to(state.target_active, *hovered);
                                cx.notify();
                            });
                        }
                    })
                    .on_click(move |_, _, cx| {
                        *cx.global_mut::<LibraryRoutes>() = section;
                    })
                    .child(icon_layers)
                    .child(label),
            )
    }
}

impl Render for Sidebar {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let current = *cx.global::<LibraryRoutes>();

        div()
            .relative()
            .w_full()
            .h_full()
            .bg(theme.library_sidebar_bg)
            .flex()
            .flex_col()
            .child(Self::section_header("DISCOVERY", &theme))
            .child(Self::item(
                Icons::Home,
                "Home",
                LibraryRoutes::Home,
                current,
                &theme,
                window,
                cx,
            ))
            .child(Self::item(
                Icons::Heart,
                "Favorites",
                LibraryRoutes::Favorites,
                current,
                &theme,
                window,
                cx,
            ))
            .child(Self::item(
                Icons::Chart,
                "Stats",
                LibraryRoutes::Stats,
                current,
                &theme,
                window,
                cx,
            ))
            .child(Self::section_header("COLLECTION", &theme))
            .child(Self::item(
                Icons::Music,
                "Tracks",
                LibraryRoutes::Tracks,
                current,
                &theme,
                window,
                cx,
            ))
            .child(Self::item(
                Icons::Disc,
                "Albums",
                LibraryRoutes::Albums,
                current,
                &theme,
                window,
                cx,
            ))
            .child(Self::item(
                Icons::User,
                "Artists",
                LibraryRoutes::Artists,
                current,
                &theme,
                window,
                cx,
            ))
            .child(Self::item(
                Icons::Playlist,
                "Playlists",
                LibraryRoutes::Playlists,
                current,
                &theme,
                window,
                cx,
            ))
            .child(Self::section_header("SYSTEM", &theme))
            .child(Self::item(
                Icons::Plugins,
                "Plugins",
                LibraryRoutes::Plugins,
                current,
                &theme,
                window,
                cx,
            ))
            .child(Self::item(
                Icons::Settings,
                "Settings",
                LibraryRoutes::Settings,
                current,
                &theme,
                window,
                cx,
            ))
            .child(div().flex_grow())
    }
}
