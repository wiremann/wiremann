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
        div()
            .id("titlebar")
            .h_10()
            .w_full()
            .flex()
            .items_center()
            .justify_center()
            .border_t_1()
            .border_color(theme.border)
            .bg(theme.titlebar_bg)
    }
}

impl ScanningStatusToast {
    pub fn new() -> ScanningStatusToast {
        ScanningStatusToast {}
    }
}

impl gpui::Global for ScanningStatus {}
