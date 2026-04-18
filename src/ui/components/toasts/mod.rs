pub mod scanning_status;

use crate::ui::{components::toasts::scanning_status::ScanningStatusToast, theme::Theme};
use gpui::{
    App, AppContext, Context, Div, Entity, InteractiveElement, IntoElement, ParentElement, Render,
    Styled, Window, WindowControlArea, div, prelude::FluentBuilder, transparent_black, white,
};
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct Toast {
    pub id: u64,
    pub kind: ToastKind,
    pub created_at: Instant,
    pub duration: Option<Duration>,
}

#[derive(Clone)]
pub enum ToastKind {
    ScanProgress,
    Message(String),
}

#[derive(Clone)]
pub struct ToastManager {
    pub toasts: Entity<Vec<Toast>>,
}

impl Render for ToastManager {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>().clone();
        div()
            .id("toast_manager")
            .absolute()
            .h_full()
            .w_full()
            .flex()
            .flex_col()
            .gap_4()
            .pt_16()
            .pr_4()
            .items_end()
            .justify_start()
            .children({
                let toasts = self.toasts.read(cx).clone();

                let mut elements = Vec::new();

                for toast in toasts.iter() {
                    let el = match &toast.kind {
                        ToastKind::ScanProgress => div()
                            .id("toast_scan_progress")
                            .child(cx.new(|_| ScanningStatusToast::new())),

                        ToastKind::Message(msg) => div()
                            .id(format!("toast_msg_{}", toast.id))
                            .px_4()
                            .py_2()
                            .min_w_80()
                            .min_h_16()
                            .flex()
                            .items_center()
                            .justify_start()
                            .bg(theme.toast_bg)
                            .border_2()
                            .border_color(theme.toast_border)
                            .text_color(theme.toast_msg_text)
                            .rounded_xl()
                            .child(msg.clone())
                            .block_mouse_except_scroll(),
                    };

                    elements.push(el);
                }

                elements
            })
    }
}

impl ToastManager {
    pub fn new(cx: &mut App) -> Self {
        ToastManager {
            toasts: cx.new(|_| Vec::new()),
        }
    }
}
