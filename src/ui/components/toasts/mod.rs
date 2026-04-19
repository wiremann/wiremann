pub mod scanning_status;

use crate::ui::{components::toasts::scanning_status::ScanningStatusToast, theme::Theme};
use gpui::{
    Animation, AnimationExt, App, AppContext, Context, Div, ElementId, Entity, InteractiveElement,
    IntoElement, ParentElement, Render, StatefulInteractiveElement, Styled, Window,
    WindowControlArea, div, prelude::FluentBuilder, px, transparent_black, white,
};
use std::time::{Duration, Instant};

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
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>().clone();
        div()
            .id("toast_manager")
            .absolute()
            .h_full()
            .w_full()
            .flex()
            .flex_col()
            .gap_4()
            .pt_16()
            .pr_4()
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

                    let state =
                        window.use_keyed_state(format!("toast_anim_{}", id), cx, |_, _| phase);

                    let prev_phase = *state.read(cx);

                    let el = base.map(|this| {
                        if prev_phase == phase {
                            match phase {
                                ToastPhase::Entering | ToastPhase::Idle => {
                                    this.left(px(0.0)).opacity(1.0).into_any_element()
                                }
                                ToastPhase::Exiting => {
                                    this.left(px(80.0)).opacity(0.0).into_any_element()
                                }
                            }
                        } else {
                            // ✅ schedule ONCE like navbar
                            cx.spawn({
                                let state = state.clone();
                                async move |_, cx| {
                                    cx.background_executor().timer(duration).await;
                                    let _ = state.update(cx, |s, _| *s = phase);
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
                            t.phase = ToastPhase::Idle;
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
