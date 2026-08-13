use gpui::{
    App, AppContext, Context, Entity, FontWeight, Global, InteractiveElement, IntoElement,
    ParentElement, Render, ScrollHandle, StatefulInteractiveElement, Styled, Window, div, px, rgba,
    rgb,
};

use crate::ui::theme::Theme;

#[derive(Clone)]
pub struct KeybindsOverlay {
    pub visible: Entity<bool>,
    scroll_handle: ScrollHandle,
}

#[derive(Clone)]
pub struct KeybindsOverlayHandle(pub Entity<KeybindsOverlay>);

impl Global for KeybindsOverlayHandle {}

impl KeybindsOverlay {
    pub fn new(cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self {
            visible: cx.new(|_| false),
            scroll_handle: ScrollHandle::new(),
        })
    }
}

impl KeybindsOverlay {
    pub fn toggle(&self, cx: &mut App) {
        self.visible.update(cx, |v, cx| {
            *v = !*v;
            cx.notify();
        });
    }
}

impl Render for KeybindsOverlay {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let visible = *self.visible.read(cx);

        if !visible {
            return div().into_any_element();
        }

        let shortcuts = vec![
            ("Space / K", "Play / Pause"),
            ("Ctrl+Left", "Previous Track"),
            ("Ctrl+Right", "Next Track"),
            ("Left", "Seek Back 5s"),
            ("Right", "Seek Forward 5s"),
            ("Shift+S", "Toggle Shuffle"),
            ("Shift+R", "Toggle Repeat"),
            ("Ctrl+Tab", "Next Page"),
            ("Ctrl+Shift+Tab", "Previous Page"),
            ("Ctrl+1", "Library Page"),
            ("Ctrl+2", "Player Page"),
            ("Ctrl+3", "Playlists Page"),
            ("?", "Toggle This Help"),
        ];

        let scroll_handle = self.scroll_handle.clone();

        div()
            .id("keybinds_overlay_bg")
            .absolute()
            .size_full()
            .top_0()
            .left_0()
            .bg(rgba(0x000000B3))
            .flex()
            .items_center()
            .justify_center()
            .on_click({
                let visible = self.visible.clone();
                move |_, _, cx| {
                    visible.update(cx, |v, _| *v = false);
                }
            })
            .child(
                div()
                    .id("keybinds_overlay_modal")
                    .bg(rgb(0x1A1A1A))
                    .border_1()
                    .border_color(theme.border)
                    .rounded_xl()
                    .p_8()
                    .min_w(px(400.0))
                    .h(px(540.0))
                    .flex()
                    .flex_col()
                    .on_click(|_, _, cx| cx.stop_propagation())
                    .child(
                        div()
                            .id("keybinds_header")
                            .text_xl()
                            .font_weight(FontWeight::BOLD)
                            .text_color(theme.library_text)
                            .child("Keyboard Shortcuts"),
                    )
                    .child(div().id("keybinds_sep_top").w_full().h(px(1.0)).bg(theme.border))
                    .child(
                        div()
                            .id("keybinds_shortcuts_scroll")
                            .flex_1()
                            .flex()
                            .flex_col()
                            .overflow_scroll()
                            .track_scroll(&scroll_handle)
                            .children(shortcuts.into_iter().map(|(key, action)| {
                                div()
                                    .id(format!("shortcut_{}", action))
                                    .w_full()
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .py_1()
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(theme.library_text)
                                            .child(action),
                                    )
                                    .child(
                                        div()
                                            .px_3()
                                            .py_1()
                                            .rounded_md()
                                            .bg(rgba(0xFFFFFF14))
                                            .text_sm()
                                            .font_family("JetBrains Mono")
                                            .text_color(theme.library_text)
                                            .child(key),
                                    )
                            })),
                    )
                    .child(div().id("keybinds_sep_bot").w_full().h(px(1.0)).bg(theme.border))
                    .child(
                        div()
                            .id("keybinds_close_hint")
                            .flex_shrink_0()
                            .text_sm()
                            .text_color(theme.library_text)
                            .child("Click anywhere outside to close"),
                    ),
            )
            .into_any_element()
    }
}
