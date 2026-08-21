use crate::ui::{components::element_ext::ElementExt, theme::Theme};
use gpui::{
    App, AppContext, Bounds, Context, DragMoveEvent, Entity, EntityId, InteractiveElement,
    IntoElement, ParentElement, Pixels, Point, Render, RenderOnce, StatefulInteractiveElement,
    Styled, Window, div, px,
};

#[derive(Clone, Copy, PartialEq)]
pub enum ResizeSide {
    Left,
    Right,
}

#[derive(Clone)]
struct ResizeDrag(EntityId);

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
    bounds: Bounds<Pixels>,
}

impl ResizeState {
    #[must_use]
    pub fn new(side: ResizeSide, width: f32, min: f32, max: f32) -> Self {
        Self {
            width,
            min,
            max,
            side,
            bounds: Bounds::default(),
        }
    }

    #[must_use]
    pub fn width(&self) -> f32 {
        self.width
    }

    pub fn update_from_position(
        &mut self,
        position: Point<Pixels>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let viewport_width = window.bounds().size.width.to_f64() as f32;

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
    pub fn new(state: &Entity<ResizeState>) -> Self {
        Self {
            state: state.clone(),
        }
    }
}

impl RenderOnce for ResizeHandle {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let entity_id = self.state.entity_id();
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
            .child(
                div()
                    .id("resize_handle_inner")
                    .w(px(2.0))
                    .h_full()
                    .bg(theme.resize_handle),
            )
            .on_drag(ResizeDrag(entity_id), |drag, _, _, cx| {
                cx.new(|_| drag.clone())
            })
            .on_drag_move(window.listener_for(
                &self.state,
                move |state, e: &DragMoveEvent<ResizeDrag>, window, cx| match e.drag(cx) {
                    ResizeDrag(id) => {
                        if *id != entity_id {
                            return;
                        }

                        state.update_from_position(e.event.position, window, cx);
                    }
                },
            ))
            .on_prepaint({
                let state = self.state.clone();
                move |bounds, _, cx| state.update(cx, |s, _| s.bounds = bounds)
            })
    }
}
