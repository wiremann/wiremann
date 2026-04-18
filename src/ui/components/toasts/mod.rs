pub mod scanning_status;

use crate::ui::theme::Theme;
use gpui::{
    App, AppContext, Context, Entity, InteractiveElement, IntoElement, ParentElement, Render,
    Styled, Window, WindowControlArea, div, transparent_black, white,
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
    ScanProgress {
        discovered: usize,
        processed: usize,
        total: usize,
        phase: ScanPhase,
    },
    Message(String),
}

#[derive(Clone)]
pub enum ScanPhase {
    Discovering,
    Processing,
    Finished,
}

#[derive(Clone)]
pub struct ToastManager {
    pub toasts: Entity<Vec<Toast>>,
}

impl Render for ToastManager {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>();
        div()
            .id("toast_manager")
            .absolute()
            .h_full()
            .w_full()
            .flex()
            .flex_col()
            .items_start()
            .justify_end()
            .bg(transparent_black())
    }
}

impl ToastManager {
    pub fn new(cx: &mut App) -> Self {
        ToastManager {
            toasts: cx.new(|_| Vec::new()),
        }
    }
}
