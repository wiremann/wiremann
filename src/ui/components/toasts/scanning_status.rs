use crate::ui::components::icons::{Icon, Icons};
use crate::ui::theme::Theme;
use gpui::{
    App, AppContext, Context, Entity, InteractiveElement, IntoElement, ParentElement, Render,
    Styled, Window, WindowControlArea, div, px, white,
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
        let theme = cx.global::<Theme>().clone();
        let status = cx.global::<ScanningStatus>().0.read(cx).clone();

        let progress = if status.total > 0 {
            status.processed as f32 / status.total as f32
        } else {
            0.0
        };

        let message = if status.is_discovering {
            format!("Scanning for files… ({} found)", status.discovered)
        } else if status.is_processing {
            format!("Processing {} of {} tracks", status.processed, status.total)
        } else {
            "Preparing scan…".to_string()
        };

        div()
            .relative()
            .flex()
            .flex_col()
            .gap_2()
            .px_4()
            .py_3()
            .min_w_80()
            .max_w_128()
            .bg(theme.toast_bg)
            .border_1()
            .border_color(theme.toast_info_accent)
            .rounded_xl()
            .block_mouse_except_scroll()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_4()
                    .child(
                        div()
                            .size_8()
                            .flex()
                            .items_center()
                            .justify_center()
                            .rounded_lg()
                            .child(
                                Icon::new(Icons::Loader)
                                    .size_8()
                                    .text_color(theme.toast_info_accent),
                            ),
                    )
                    .child(
                        div()
                            .flex_1()
                            .text_color(theme.toast_text)
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .child(message),
                    ),
            )
            .child(
                div()
                    .w_full()
                    .h_1()
                    .bg(theme.toast_progress_bg)
                    .rounded_full()
                    .child(
                        div()
                            .h_full()
                            .bg(theme.toast_info_accent)
                            .rounded_full()
                            .w(px(progress * 200.0)),
                    ),
            )
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
