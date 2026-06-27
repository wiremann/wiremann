use gpui::{
    Animation, AnimationExt, Context, ElementId, FontWeight, Global, InteractiveElement,
    IntoElement, ParentElement, Render, StatefulInteractiveElement, Styled, Window, div,
    gradient_color_stop, linear_gradient, prelude::FluentBuilder, px, relative, transparent_black,
};

use crate::ui::{
    components::{
        bounds_observer::observe_bounds,
        icons::{Icons, icon},
    },
    pages::library::LibrarySection,
    theme::Theme,
};

pub struct Sidebar;

#[derive(Clone, Copy, Default)]
pub struct SidebarIndicator {
    pub top: f32,
    pub height: f32,
}

impl Global for SidebarIndicator {}

impl Sidebar {
    fn section_header(text: &'static str, theme: Theme) -> impl IntoElement {
        div()
            .px_5()
            .pt_5()
            .pb_2()
            .child(text)
            .text_xs()
            .font_weight(FontWeight::LIGHT)
            .text_color(theme.library_sidebar_group_text)
    }

    fn item(
        icon_type: Icons,
        text: &'static str,
        section: LibrarySection,
        current: LibrarySection,
        theme: Theme,
    ) -> impl IntoElement {
        let active = current == section;

        observe_bounds(
            format!("sidebar_bounds_{text}"),
            div()
                .id(format!("library_sidebar_item_{text}"))
                .mx_3()
                .px_6()
                .py_3()
                .flex()
                .items_center()
                .gap_4()
                .cursor_pointer()
                .hover(|this| this.bg(theme.library_sidebar_item_bg_hover))
                .on_click(move |_, _, cx| {
                    *cx.global_mut::<LibrarySection>() = section;
                })
                .child(icon(icon_type).size_4().text_color(if active {
                    theme.library_sidebar_item_text_active
                } else {
                    theme.library_sidebar_item_text
                }))
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
            move |bounds, _, cx| {
                if *cx.global::<LibrarySection>() == section {
                    let indicator = cx.global_mut::<SidebarIndicator>();

                    indicator.top = bounds.top().to_f64() as f32;
                    indicator.height = bounds.size.height.to_f64() as f32;
                }
            },
        )
    }
}

impl Render for Sidebar {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let current = *cx.global::<LibrarySection>();
        let current_offset = current.sidebar_offset();

        let indicator_data = *cx.global::<SidebarIndicator>();

        let indicator_state = window.use_keyed_state("sidebar_indicator", cx, |_, _| {
            (current, indicator_data.top, indicator_data.height)
        });

        let (prev_section, prev_top, prev_height) = *indicator_state.read(cx);

        let duration = std::time::Duration::from_millis(250);

        div()
            .relative()
            .w(relative(0.32))
            .min_w_56()
            .max_w_64()
            .h_full()
            .bg(theme.library_sidebar_bg)
            .border_r_1()
            .border_color(theme.border)
            .flex()
            .flex_col()
            .child({
                let indicator = div()
                    .absolute()
                    .right_0()
                    .w_full()
                    .child(
                        div()
                            .absolute()
                            .right_0()
                            .top_0()
                            .bottom_0()
                            .w(px(4.0))
                            .bg(theme.library_sidebar_indicator),
                    )
                    .child(
                        div()
                            .absolute()
                            .left_0()
                            .top_0()
                            .bottom_0()
                            .right(px(4.0))
                            .bg(linear_gradient(
                                90.0,
                                gradient_color_stop(theme.library_sidebar_indicator_glow, 0.0),
                                gradient_color_stop(transparent_black(), 1.0),
                            )),
                    );

                if prev_section == current {
                    indicator
                        .top(px(indicator_data.top))
                        .h(px(indicator_data.height))
                        .into_any_element()
                } else {
                    cx.spawn({
                        let indicator_state = indicator_state.clone();

                        async move |_, cx| {
                            cx.background_executor().timer(duration).await;

                            let _ = indicator_state.update(cx, |state, _| {
                                *state = (current, indicator_data.top, indicator_data.height);
                            });
                        }
                    })
                    .detach();

                    indicator
                        .with_animation(
                            ElementId::NamedInteger("sidebar_indicator".into(), current as u64),
                            Animation::new(duration).with_easing(gpui::ease_out_quint()),
                            move |this, delta| {
                                let y = prev_top + (indicator_data.top - prev_top) * delta;

                                let h = prev_height + (indicator_data.height - prev_height) * delta;

                                this.top(px(y)).h(px(h))
                            },
                        )
                        .into_any_element()
                }
            })
            .child(div().h_px().mx_4().bg(theme.library_sidebar_separator))
            .child(Self::section_header("DISCOVERY", theme))
            .child(Self::item(
                Icons::Home,
                "Home",
                LibrarySection::Home,
                current,
                theme,
            ))
            .child(Self::item(
                Icons::Heart,
                "Favorites",
                LibrarySection::Favorites,
                current,
                theme,
            ))
            .child(Self::section_header("COLLECTION", theme))
            .child(Self::item(
                Icons::Music,
                "Tracks",
                LibrarySection::Tracks,
                current,
                theme,
            ))
            .child(Self::item(
                Icons::MusicList,
                "Albums",
                LibrarySection::Albums,
                current,
                theme,
            ))
            .child(Self::item(
                Icons::User,
                "Artists",
                LibrarySection::Artists,
                current,
                theme,
            ))
            .child(Self::item(
                Icons::Playlist,
                "Playlists",
                LibrarySection::Playlists,
                current,
                theme,
            ))
            .child(Self::section_header("SYSTEM", theme))
            .child(Self::item(
                Icons::Plugins,
                "Plugins",
                LibrarySection::Tools,
                current,
                theme,
            ))
            .child(Self::item(
                Icons::Settings,
                "Settings",
                LibrarySection::Settings,
                current,
                theme,
            ))
            .child(div().flex_grow())
            .child(
                div()
                    .h_px()
                    .mx_4()
                    .mb_3()
                    .bg(theme.library_sidebar_separator),
            )
    }
}
