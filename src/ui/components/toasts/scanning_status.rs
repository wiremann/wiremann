use crate::ui::components::icons::Icons;
use crate::ui::theme::Theme;
use gpui::{
    App, AppContext, Context, Entity, InteractiveElement, IntoElement, ParentElement, Render,
    Styled, Window, WindowControlArea, div, white,
};

#[derive(Clone)]
pub struct ScanningStatusInner {
    pub is_scanning: bool,
    pub is_discovering: bool,
    pub is_processing: bool,

    pub discovered: usize,
    pub total: usize,
    pub processed: usize,
}

#[derive(Clone)]
pub struct ScanningStatus(pub Entity<ScanningStatusInner>);

#[derive(Clone)]
pub struct ScanningStatusToast {
    display_processed: f32,
    velocity: f32,
}

impl Render for ScanningStatusToast {
    #[allow(clippy::unreadable_literal)]
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>().clone();
        let status = cx.global::<ScanningStatus>().0.read(cx).clone();
        let discovered = status.discovered;
        let total = status.total;
        let target = status.processed as f32;

        if target == 0.0 && self.display_processed != 0.0 {
            self.display_processed = 0.0;
            self.velocity = 0.0;
        }

        let diff = target - self.display_processed;

        self.velocity += diff * 0.15;
        self.velocity *= 0.8;

        self.velocity = self.velocity.clamp(-50.0, 50.0);

        self.display_processed += self.velocity;

        if diff.abs() < 0.5 && self.velocity.abs() < 0.5 {
            self.display_processed = target;
            self.velocity = 0.0;
        } else {
            cx.notify();
        }

        let processed = self.display_processed as usize;
        div()
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
            .block_mouse_except_scroll()
            .child({
                if status.is_discovering {
                    format!("Discovered: {} files...", discovered)
                } else {
                    format!("Processing: {} / {}", processed, total)
                }
            })
    }
}

impl ScanningStatusToast {
    pub fn new() -> ScanningStatusToast {
        ScanningStatusToast {
            display_processed: 0.0,
            velocity: 0.0,
        }
    }
}

impl ScanningStatus {
    pub fn new(cx: &mut App) -> Self {
        ScanningStatus(cx.new(|_| ScanningStatusInner {
            is_scanning: false,
            is_discovering: false,
            is_processing: false,

            discovered: 0,
            total: 0,
            processed: 0,
        }))
    }
}

impl gpui::Global for ScanningStatus {}
