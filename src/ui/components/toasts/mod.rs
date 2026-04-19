pub mod scanning_status;

use crate::ui::{components::toasts::scanning_status::ScanningStatusToast, theme::Theme};
use gpui::{
    Animation, AnimationExt, App, AppContext, Context,ElementId, Entity, InteractiveElement,
    IntoElement, ParentElement, Render, StatefulInteractiveElement, Styled, Window,
     div, prelude::FluentBuilder, px,
};
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct Toast {
    pub id: u64,
    pub kind: ToastKind,
    pub created_at: Instant,
    pub duration: Option<Duration>,
    pub phase: ToastPhase,
    pub anim_phase: ToastPhase,
}

#[derive(Clone)]
pub enum ToastKind {
    ScanProgress(Entity<ScanningStatusToast>),
    Message(String),
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

                        ToastKind::Message(msg) => div()
                            .id(format!("toast_msg_{}", id))
                            .relative()
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
                            .child(msg.clone())
                            .on_click(move |_, _, cx| {
                                toasts.update(cx, |list, _| {
                                    for t in list.iter_mut() {
                                        if t.id == id {
                                            t.phase = ToastPhase::Exiting;
                                        }
                                    }
                                });
                            }),
                    };

                    let duration = Duration::from_millis(250);

                    let prev_phase = toast.anim_phase;

                    let el = base.map(|this| {
                        if prev_phase == phase {
                            match phase {
                                ToastPhase::Entering => this.left(px(240.0)).opacity(0.0).into_any_element(),
                                ToastPhase::Idle => this.left(px(0.0)).opacity(1.0).into_any_element(),
                                ToastPhase::Exiting => this.left(px(240.0)).opacity(0.0).into_any_element(),
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
                    .timer(Duration::from_millis(256))
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
                                if t.phase == ToastPhase::Exiting {
                                    return false;
                                }
                                t.phase = ToastPhase::Exiting;
                            }
                        }

                        true
                    });
                });
            }
        })
        .detach();

        ToastManager { toasts }
    }
}
