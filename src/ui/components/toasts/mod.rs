pub mod scanning_status;

use crate::ui::{
    components::{
        icons::{Icon, Icons},
        toasts::scanning_status::ScanningStatusToast,
    },
    theme::Theme,
};
use gpui::{
    Animation, AnimationExt, App, AppContext, Context, ElementId, Entity, InteractiveElement,
    IntoElement, ParentElement, Render, StatefulInteractiveElement, Styled, Window, div,
    prelude::FluentBuilder, px,
};
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct Toast {
    pub id: u64,
    pub kind: ToastKind,
    pub created_at: Instant,
    pub duration: Option<Duration>,
    pub phase: ToastPhase,

    // temp states
    pub anim_phase: ToastPhase,
    pub exiting_at: Option<Instant>,
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
    Entering,
    Idle,
    Exiting,
}

#[derive(Clone)]
pub struct ToastManager {
    pub toasts: Entity<Vec<Toast>>,
    next_id: u64,
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
            .pt_20()
            .pr_8()
            .items_end()
            .justify_start()
            .children({
                let toasts_vec = self.toasts.read(cx).clone();
                let mut elements = Vec::new();

                for toast in toasts_vec.iter() {
                    let id = toast.id;
                    let phase = toast.phase;
                    let toasts = self.toasts.clone();

                    let base = match &toast.kind {
                        ToastKind::ScanProgress(el) => div()
                            .id(format!("toast_scan_{}", id))
                            .relative()
                            .child(el.clone())
                            .on_click(move |_, _, cx| {
                                toasts.update(cx, |list, _| {
                                    for t in list.iter_mut() {
                                        if t.id == id {
                                            t.phase = ToastPhase::Exiting;
                                        }
                                    }
                                });
                            }),

                        ToastKind::Info(msg) | ToastKind::Success(msg) | ToastKind::Error(msg) => {
                            let (accent, icon) = match &toast.kind {
                                ToastKind::Info(_) => {
                                    (theme.toast_info_accent, Icon::new(Icons::ToastInfo))
                                }
                                ToastKind::Success(_) => {
                                    (theme.toast_success_accent, Icon::new(Icons::ToastSuccess))
                                }
                                ToastKind::Error(_) => {
                                    (theme.toast_error_accent, Icon::new(Icons::ToastError))
                                }
                                _ => unreachable!(),
                            };

                            div()
                                .id(format!("toast_msg_{}", id))
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
                                        .text_sm()
                                        .font_weight(gpui::FontWeight::MEDIUM)
                                        .child(msg.clone()),
                                )
                                .on_click(move |_, _, cx| {
                                    toasts.update(cx, |list, _| {
                                        for t in list.iter_mut() {
                                            if t.id == id {
                                                t.phase = ToastPhase::Exiting;
                                                t.exiting_at = Some(Instant::now());
                                            }
                                        }
                                    });
                                })
                        }
                    };

                    let duration = Duration::from_millis(250);

                    let prev_phase = toast.anim_phase;

                    let el = base.map(|this| {
                        if prev_phase == phase {
                            match phase {
                                ToastPhase::Entering => {
                                    this.left_72().opacity(0.0).into_any_element()
                                }
                                ToastPhase::Idle => {
                                    this.left(px(0.0)).opacity(1.0).into_any_element()
                                }
                                ToastPhase::Exiting => {
                                    this.left_72().opacity(0.0).into_any_element()
                                }
                            }
                        } else {
                            cx.spawn({
                                let toasts = self.toasts.clone();
                                async move |_, cx| {
                                    cx.background_executor().timer(duration).await;
                                    toasts.update(cx, |list, _| {
                                        for t in list.iter_mut() {
                                            if t.id == id {
                                                t.anim_phase = t.phase;
                                            }
                                        }
                                    });
                                }
                            })
                            .detach();

                            this.with_animation(
                                ElementId::NamedInteger("toast_anim".into(), id),
                                Animation::new(duration).with_easing(gpui::ease_out_quint()),
                                move |this, delta| match (prev_phase, phase) {
                                    (ToastPhase::Entering, ToastPhase::Idle) => {
                                        let x = 80.0 * (1.0 - delta);
                                        this.left(px(x)).opacity(delta)
                                    }
                                    (ToastPhase::Idle, ToastPhase::Exiting) => {
                                        let x = 80.0 * delta;
                                        this.left(px(x)).opacity(1.0 - delta)
                                    }
                                    _ => this,
                                },
                            )
                            .into_any_element()
                        }
                    });

                    elements.push(el);
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

                toasts_clone.update(cx, |toasts, _| {
                    toasts.retain_mut(|t| {
                        let now = Instant::now();

                        if t.phase == ToastPhase::Entering {
                            if now.duration_since(t.created_at) > Duration::from_millis(250) {
                                t.phase = ToastPhase::Idle;
                            }
                        }
                        if let Some(duration) = t.duration {
                            if now.duration_since(t.created_at) >= duration {
                                if t.phase != ToastPhase::Exiting {
                                    t.phase = ToastPhase::Exiting;
                                    t.exiting_at = Some(now);
                                }
                            }
                        }

                        if let Some(exit_time) = t.exiting_at {
                            if now.duration_since(exit_time) >= Duration::from_millis(250) {
                                return false;
                            }
                        }

                        true
                    });
                });
            }
        })
        .detach();

        ToastManager { toasts, next_id: 0 }
    }

    pub fn spawn(&mut self, kind: ToastKind, duration: Option<Duration>, cx: &mut Context<Self>) {
        let id = self.next_id;
        self.next_id += 1;

        self.toasts.update(cx, |toasts, _| {
            toasts.push(Toast {
                id,
                kind,
                created_at: Instant::now(),
                duration,
                phase: ToastPhase::Entering,
                anim_phase: ToastPhase::Entering,
                exiting_at: None,
            });
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
