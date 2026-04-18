use crate::ui::components::icons::Icons;
use crate::ui::theme::Theme;
use gpui::{
    App, AppContext, Context, Entity, InteractiveElement, IntoElement, ParentElement, Render,
    Styled, Window, WindowControlArea, div, white,
};

#[derive(Clone)]
pub struct ScanningStatus {
    pub is_scanning: Entity<bool>,
    pub is_discovering: Entity<bool>,
    pub is_processing: Entity<bool>,

    pub discovered: Entity<usize>,
    pub total: Entity<usize>,
    pub processed: Entity<usize>,
}

#[derive(Clone)]
pub struct ScanningStatusToast;

impl Render for ScanningStatusToast {
    #[allow(clippy::unreadable_literal)]
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>();
        let status = cx.global::<ScanningStatus>();
        let discovered = *status.discovered.read(cx);
        let processed = *status.processed.read(cx);
        let total = *status.total.read(cx);

        div()
            .px_4()
            .py_3()
            .bg(theme.titlebar_bg)
            .rounded_lg()
            .border_1()
            .border_color(theme.border)
            .child({
                if *status.is_discovering.read(cx) {
                    format!("Discovering: {} files...", discovered)
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
        ScanningStatus {
            is_scanning: cx.new(|_| false),
            is_discovering: cx.new(|_| false),
            is_processing: cx.new(|_| false),

            discovered: cx.new(|_| 0),
            total: cx.new(|_| 0),
            processed: cx.new(|_| 0),
        }
    }
}

impl gpui::Global for ScanningStatus {}
