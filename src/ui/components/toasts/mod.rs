pub mod scanning_status;

use crate::ui::{
    components::{
        icons::{Icons, icon},
        toasts::scanning_status::ScanningStatusToast,
    },
    theme::Theme,
};
use gpui::{
    App, AppContext, Context, Entity, InteractiveElement, IntoElement, ParentElement, Render,
    StatefulInteractiveElement, Styled, Window, div, px,
};
use std::time::{Duration, Instant};
use wgpui_motion::prelude::{DurationExt, MotionExt, Transition, quint_out};

#[derive(Clone)]
pub struct Toast {
    pub id: u64,
    pub kind: ToastKind,
    pub created_at: Instant,
    pub duration: Option<Duration>,
    pub phase: ToastPhase,
}

#[derive(Clone)]
pub enum ToastKind {
    Info(String),
    Success(String),
    Error(String),
    ScanProgress(Entity<ScanningStatusToast>),
}

#[derive(Clone, Copy, PartialEq)]
pub enum ToastPhase {
    Idle,
    Exiting,
}

#[derive(Clone)]
pub struct ToastManager {
    pub toasts: Entity<Vec<Toast>>,
    next_id: u64,
}

impl Render for ToastManager {
    #[allow(clippy::too_many_lines)]
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        div()
            .id("toast_manager")
            .absolute()
            .h_full()
            .w_full()
            .flex()
            .flex_col()
            .gap_4()
            .pt_20()
            .pr_8()
            .items_end()
            .justify_start()
            .children({
                let toasts_vec = self.toasts.read(cx).clone();
                let mut elements = Vec::new();

                for toast in &toasts_vec {
                    let id = toast.id;
                    let phase = toast.phase;
                    let toasts = self.toasts.clone();

                    let base = match &toast.kind {
                        ToastKind::ScanProgress(el) => div()
                            .id(format!("toast_scan_{id}"))
                            .relative()
                            .child(el.clone())
                            .on_click(move |_, _, cx| {
                                toasts.update(cx, |list, cx| {
                                    for t in list.iter_mut() {
                                        if t.id == id {
                                            t.phase = ToastPhase::Exiting;
                                        }
                                    }
                                    cx.notify();
                                });
                            }),

                        ToastKind::Info(msg) | ToastKind::Success(msg) | ToastKind::Error(msg) => {
                            let (accent, icon) = match &toast.kind {
                                ToastKind::Info(_) => {
                                    (theme.toast_info_accent, icon(Icons::ToastInfo))
                                }
                                ToastKind::Success(_) => {
                                    (theme.toast_success_accent, icon(Icons::ToastSuccess))
                                }
                                ToastKind::Error(_) => {
                                    (theme.toast_error_accent, icon(Icons::ToastError))
                                }
                                ToastKind::ScanProgress(_) => unreachable!(),
                            };

                            div()
                                .id(format!("toast_msg_{id}"))
                                .relative()
                                .flex()
                                .items_center()
                                .gap_4()
                                .px_3()
                                .py_3()
                                .min_w_80()
                                .max_w_128()
                                .min_h_16()
                                .max_h_32()
                                .bg(theme.toast_bg)
                                .border_1()
                                .border_color(accent)
                                .rounded_xl()
                                .block_mouse_except_scroll()
                                .child(
                                    div()
                                        .size_8()
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .rounded_lg()
                                        .child(icon.size_8().text_color(accent)),
                                )
                                .child(
                                    div()
                                        .flex_1()
                                        .text_color(theme.toast_text)
                                        .font_weight(gpui::FontWeight::MEDIUM)
                                        .child(msg.clone()),
                                )
                                .on_click(move |_, _, cx| {
                                    toasts.update(cx, |list, cx| {
                                        for t in list.iter_mut() {
                                            if t.id == id {
                                                t.phase = ToastPhase::Exiting;
                                            }
                                        }
                                        cx.notify();
                                    });
                                })
                        }
                    };

                    let toasts = self.toasts.clone();
                    elements.push(
                        base.relative()
                            .left(px(0.0))
                            .opacity(1.0)
                            .motion(("toast", id))
                            .transition(Transition::new(250.ms()).curve(quint_out()))
                            .enter(|style| style.left(px(80.0)).opacity(0.0))
                            .exit_when(phase == ToastPhase::Exiting, |style| {
                                style.left(px(80.0)).opacity(0.0)
                            })
                            .on_exit_complete(move |_, _, cx| {
                                toasts.update(cx, |list, cx| {
                                    list.retain(|toast| toast.id != id);
                                    cx.notify();
                                });
                            })
                            .into_any_element(),
                    );
                }

                elements
            })
    }
}

impl ToastManager {
    pub fn new(cx: &mut App) -> Self {
        let toasts: Entity<Vec<Toast>> = cx.new(|_| Vec::new());

        let toasts_clone = toasts.clone();

        cx.spawn(async move |cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(128))
                    .await;

                toasts_clone.update(cx, |toasts, cx| {
                    let mut changed = false;
                    toasts.retain_mut(|t| {
                        let now = Instant::now();

                        if let Some(duration) = t.duration
                            && now.duration_since(t.created_at) >= duration
                            && t.phase != ToastPhase::Exiting
                        {
                            t.phase = ToastPhase::Exiting;
                            changed = true;
                        }

                        true
                    });
                    if changed {
                        cx.notify();
                    }
                });
            }
        })
        .detach();

        ToastManager { toasts, next_id: 0 }
    }

    pub fn spawn(&mut self, kind: ToastKind, duration: Option<Duration>, cx: &mut Context<Self>) {
        let id = self.next_id;
        self.next_id += 1;

        self.toasts.update(cx, |toasts, cx| {
            toasts.push(Toast {
                id,
                kind,
                created_at: Instant::now(),
                duration,
                phase: ToastPhase::Idle,
            });
            cx.notify();
        });
    }

    pub fn info(&mut self, msg: impl Into<String>, cx: &mut Context<Self>) {
        self.spawn(
            ToastKind::Info(msg.into()),
            Some(Duration::from_secs(4)),
            cx,
        );
    }

    pub fn success(&mut self, msg: impl Into<String>, cx: &mut Context<Self>) {
        self.spawn(
            ToastKind::Success(msg.into()),
            Some(Duration::from_secs(4)),
            cx,
        );
    }

    pub fn error(&mut self, msg: impl Into<String>, cx: &mut Context<Self>) {
        self.spawn(
            ToastKind::Error(msg.into()),
            Some(Duration::from_secs(6)),
            cx,
        );
    }

    pub fn scanning_status(&mut self, cx: &mut Context<Self>) {
        self.spawn(
            ToastKind::ScanProgress(cx.new(|_| ScanningStatusToast::new())),
            None,
            cx,
        );
    }
}
