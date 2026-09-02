use crate::ui::components::bounds_observer::observe_bounds;
use gpui::{App, Bounds, Context, Entity, IntoElement, Pixels, Point, RenderOnce, Window, div, point, px, public_window_callback};
use gpui::Styled;

#[derive(Clone, Copy, PartialEq)]
pub enum ResizeSide {
    Left,
    Right,
}

pub struct ResizeState {
    pub width: f32,
    min: f32,
    max: f32,
    side: ResizeSide,
    bounds: Bounds<Pixels>,
}

impl ResizeState {
    #[must_use]
    pub fn new(side: ResizeSide, width: f32, min: f32, max: f32) -> Self {
        Self { width, min, max, side, bounds: Bounds::default() }
    }

    #[must_use]
    pub fn width(&self) -> f32 { self.width }

    fn update_from_position(&mut self, position: Point<Pixels>, viewport_width: Pixels, cx: &mut Context<Self>) {
        let viewport_width = viewport_width.to_f64() as f32;
        let new_width = match self.side {
            ResizeSide::Left => (position.x - self.bounds.left() + px(self.width)).to_f64() as f32,
            ResizeSide::Right => viewport_width - position.x.to_f64() as f32,
        };
        self.width = new_width.clamp(self.min, self.max);
        cx.notify();
    }
}

#[derive(IntoElement)]
pub struct ResizeHandle {
    state: Entity<ResizeState>,
}

impl ResizeHandle {
    pub fn new(state: &Entity<ResizeState>) -> Self { Self { state: state.clone() } }
}

impl RenderOnce for ResizeHandle {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let state_for_bounds = self.state.clone();
        let state_for_move = self.state.clone();
        let handle = div()
            .id(("resize_handle", self.state.entity_id()))
            .w(px(6.0))
            .h_full()
            .flex_shrink_0()
            .cursor_col_resize()
            .on_mouse_move(public_window_callback(move |event, window, _app| {
                if event.dragging() {
                    let position = point(event.position[0], event.position[1]);
                    let viewport_width = window.bounds().size.width;
                    state_for_move.update((), |state, cx| {
                        state.update_from_position(position, viewport_width, cx);
                    });
                }
            }));

        observe_bounds(
            ("resize_handle_bounds", self.state.entity_id()),
            handle,
            move |bounds| {
                state_for_bounds.update((), |state, cx| {
                    if state.bounds != bounds {
                        state.bounds = bounds;
                        cx.notify();
                    }
                });
            },
        )
    }
}
