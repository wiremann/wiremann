use gpui::{
    Animation, AnimationExt, Context, ElementId, FontWeight, InteractiveElement, IntoElement,
    ParentElement, Render, StatefulInteractiveElement, Styled, Window, div, gradient_color_stop,
    linear_gradient, prelude::FluentBuilder, px, relative, transparent_black,
};

use crate::ui::{
    components::icons::{Icons, icon},
    pages::library::LibrarySection,
    theme::Theme,
};

pub struct Sidebar;

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

        div()
            .id(format!("library_sidebar_item_{text}"))
            .mx_3()
            .px_3()
            .py_2()
            .flex()
            .items_center()
            .gap_3()
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
            )
    }
}

impl Render for Sidebar {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let current = *cx.global::<LibrarySection>();
        let current_offset = current.sidebar_offset();

        let indicator_state =
            window.use_keyed_state("sidebar_indicator", cx, |_, _| (current, current_offset));

        let (prev_section, prev_offset) = *indicator_state.read(cx);

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
                    .h(px(32.0))
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
                            .right(px(4.0))
                            .top_0()
                            .bottom_0()
                            .w(px(120.0))
                            .bg(linear_gradient(
                                180.0,
                                gradient_color_stop(theme.library_sidebar_indicator_glow, 0.0),
                                gradient_color_stop(transparent_black(), 1.0),
                            )),
                    );

                if prev_section == current {
                    indicator.top(px(current_offset)).into_any_element()
                } else {
                    cx.spawn({
                        let indicator_state = indicator_state.clone();

                        async move |_, cx| {
                            cx.background_executor().timer(duration).await;

                            let _ = indicator_state.update(cx, |state, _| {
                                *state = (current, current_offset);
                            });
                        }
                    })
                    .detach();

                    indicator
                        .with_animation(
                            ElementId::NamedInteger("sidebar_indicator".into(), current as u64),
                            Animation::new(duration).with_easing(gpui::ease_out_quint()),
                            move |this, delta| {
                                let y = prev_offset + (current_offset - prev_offset) * delta;

                                this.top(px(y))
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
