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
            .items_start()
            .justify_end()
            .children({
                let toasts = self.toasts.read(cx);

                let mut elements = Vec::new();

                for toast in toasts.iter() {
                    let el = match &toast.kind {
                        ToastKind::ScanProgress => {
                            div().child(cx.new(|_| ScanningStatusToast::new()))
                        }

                        ToastKind::Message(msg) => div()
                            .px_4()
                            .py_2()
                            .bg(theme.titlebar_bg)
                            .text_color(white())
                            .rounded_md()
                            .child(msg.clone()),
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
