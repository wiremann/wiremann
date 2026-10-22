use crate::ui::{animations::page_transition, theme::Theme};
use gpui::{
    App, AppContext, Context, DragMoveEvent, Entity, EntityId, InteractiveElement, IntoElement,
    MouseButton, ParentElement, Pixels, Point, Render, RenderOnce, StatefulInteractiveElement,
    Styled, Window, div, px, rgba,
};
use std::time::Duration;
use wgpui_motion::prelude::{MotionExt, Transition};

#[derive(Clone, Copy, PartialEq)]
pub enum ResizeSide {
    Left,
    Right,
}

#[derive(Clone)]
struct ResizeDrag {
    id: EntityId,
    start_position: Point<Pixels>,
    start_width: f32,
}

#[derive(Clone, Copy)]
struct ResizeDragSource(EntityId);

impl Render for ResizeDrag {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        gpui::Empty
    }
}

pub struct ResizeState {
    pub width: f32,
    min: f32,
    max: f32,
    side: ResizeSide,
    is_resizing: bool,
    drag_start: Option<(Point<Pixels>, f32)>,
}

impl ResizeState {
    #[must_use]
    pub fn new(side: ResizeSide, width: f32, min: f32, max: f32) -> Self {
        Self {
            width,
            min,
            max,
            side,
            is_resizing: false,
            drag_start: None,
        }
    }

    #[must_use]
    pub fn width(&self) -> f32 {
        self.width
    }

    pub fn is_resizing(&self) -> bool {
        self.is_resizing
    }

    fn update_from_drag(
        &mut self,
        start_position: Point<Pixels>,
        start_width: f32,
        position: Point<Pixels>,
        cx: &mut Context<Self>,
    ) {
        self.width = resized_width(
            self.side,
            start_width,
            start_position.x.to_f64() as f32,
            position.x.to_f64() as f32,
            self.min,
            self.max,
        );
        cx.notify();
    }

    fn finish_drag(&mut self, cx: &mut Context<Self>) {
        self.drag_start = None;
        if self.is_resizing {
            self.is_resizing = false;
            cx.notify();
        }
    }
}

fn resized_width(
    side: ResizeSide,
    start_width: f32,
    start_x: f32,
    current_x: f32,
    min: f32,
    max: f32,
) -> f32 {
    let delta = current_x - start_x;
    let direction = match side {
        ResizeSide::Left => 1.0,
        ResizeSide::Right => -1.0,
    };
    (start_width + direction * delta).clamp(min, max)
}

pub fn resize_transition(is_resizing: bool) -> Transition {
    if is_resizing {
        Transition::new(Duration::from_millis(1))
    } else {
        page_transition()
    }
}

#[cfg(test)]
mod tests {
    use super::{ResizeSide, resized_width};

    #[test]
    fn left_resize_tracks_pointer_delta_from_drag_start() {
        assert_eq!(
            resized_width(ResizeSide::Left, 256.0, 100.0, 140.0, 220.0, 420.0),
            296.0
        );
    }

    #[test]
    fn right_resize_tracks_pointer_delta_from_drag_start() {
        assert_eq!(
            resized_width(ResizeSide::Right, 480.0, 100.0, 140.0, 360.0, 760.0),
            440.0
        );
    }

    #[test]
    fn resize_width_is_clamped_to_configured_limits() {
        assert_eq!(
            resized_width(ResizeSide::Left, 256.0, 100.0, 0.0, 220.0, 420.0),
            220.0
        );
        assert_eq!(
            resized_width(ResizeSide::Right, 480.0, 100.0, 0.0, 360.0, 760.0),
            580.0
        );
        assert_eq!(
            resized_width(ResizeSide::Right, 480.0, 100.0, -400.0, 360.0, 760.0),
            760.0
        );
        assert_eq!(
            resized_width(ResizeSide::Left, 256.0, 100.0, 400.0, 220.0, 420.0),
            420.0
        );
        assert_eq!(
            resized_width(ResizeSide::Right, 480.0, 100.0, 400.0, 360.0, 760.0),
            360.0
        );
    }
}

#[derive(IntoElement)]
pub struct ResizeHandle {
    state: Entity<ResizeState>,
}

impl ResizeHandle {
    #[must_use]
    pub fn new(state: &Entity<ResizeState>) -> Self {
        Self {
            state: state.clone(),
        }
    }
}

impl RenderOnce for ResizeHandle {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let entity_id = self.state.entity_id();
        let state_for_drag = self.state.clone();
        let state_for_mouse_down = self.state.clone();
        let state_for_mouse_up = self.state.clone();
        let state_for_mouse_up_out = self.state.clone();
        let theme = cx.global::<Theme>();

        div()
            .id(("resize_handle", entity_id))
            .w(px(6.0))
            .h_full()
            .flex_shrink_0()
            .cursor_col_resize()
            .flex()
            .justify_center()
            .items_center()
            .bg(rgba(0x00000000))
            .child(
                div()
                    .id("resize_handle_inner")
                    .w(px(1.0))
                    .h_full()
                    .bg(theme.resize_handle),
            )
            .opacity(0.55)
            .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                let position = window.mouse_position();
                state_for_mouse_down.update(cx, |state, _| {
                    state.drag_start = Some((position, state.width));
                });
            })
            .on_drag(ResizeDragSource(entity_id), move |source, _, window, cx| {
                let resize_state = state_for_drag.read(cx);
                let (start_position, start_width) = resize_state
                    .drag_start
                    .unwrap_or((window.mouse_position(), resize_state.width()));
                state_for_drag.update(cx, |state, cx| {
                    state.is_resizing = true;
                    cx.notify();
                });
                cx.new(|_| ResizeDrag {
                    id: source.0,
                    start_position,
                    start_width,
                })
            })
            .on_drag_move(window.listener_for(
                &self.state,
                move |state, e: &DragMoveEvent<ResizeDrag>, _, cx| {
                    let position = e.event.position;
                    let (drag_id, start_position, start_width) = {
                        let drag = e.drag(cx);
                        (drag.id, drag.start_position, drag.start_width)
                    };
                    if drag_id == entity_id {
                        state.update_from_drag(start_position, start_width, position, cx);
                    }
                },
            ))
            .on_mouse_up(MouseButton::Left, move |_, _, cx| {
                state_for_mouse_up.update(cx, |state, cx| state.finish_drag(cx));
            })
            .on_mouse_up_out(MouseButton::Left, move |_, _, cx| {
                state_for_mouse_up_out.update(cx, |state, cx| state.finish_drag(cx));
            })
            .motion(("resize_handle", entity_id))
            .transition(page_transition())
            .while_hovered(|style| style.bg(rgba(0xFFFFFF10)).opacity(1.0))
    }
}
