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
pub struct ScanningStatusToast;

impl Render for ScanningStatusToast {
    #[allow(clippy::unreadable_literal)]
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>();
        let status = cx.global::<ScanningStatus>().0.read(cx);
        let discovered = status.discovered;
        let processed = status.processed;
        let total = status.total;

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
        ScanningStatusToast {}
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
