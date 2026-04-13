use crate::ui::components::element_ext::ElementExt;
use gpui::{
    App, AppContext, Bounds, Context, DragMoveEvent, Entity, EntityId, EventEmitter,
    InteractiveElement, IntoElement, MouseButton, MouseDownEvent, ParentElement as _, Pixels,
    Point, Refineable, Render, RenderOnce, SharedString, StatefulInteractiveElement,
    StyleRefinement, Styled, Window, div, px, relative, transparent_black, white,
};

pub enum SliderEvent {
    Change(f32),
}

pub struct SliderState {
    min: f32,
    max: f32,
    step: f32,
    value: f32,
    percentage: f32,
    bounds: Bounds<Pixels>,
}

#[derive(Clone)]
struct DragSlider(EntityId);

impl Render for DragSlider {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        gpui::Empty
    }
}

impl Default for SliderState {
    fn default() -> Self {
        Self::new()
    }
}

impl SliderState {
    #[must_use]
    pub fn new() -> Self {
        Self {
            min: 0.0,
            max: 1.0,
            step: 0.01,
            value: 0.0,
            percentage: 0.0,
            bounds: Bounds::default(),
        }
    }

    #[must_use]
    pub fn min(mut self, v: f32) -> Self {
        self.min = v;
        self
    }

    #[must_use]
    pub fn max(mut self, v: f32) -> Self {
        self.max = v;
        self
    }

    #[must_use]
    pub fn step(mut self, v: f32) -> Self {
        self.step = v;
        self
    }

    #[must_use]
    pub fn default_value(mut self, v: f32) -> Self {
        self.value = v;
        self.percentage = self.value_to_percentage(v);
        self
    }

    #[must_use]
    pub fn value(&self) -> f32 {
        self.value
    }

    pub fn set_value(&mut self, v: f32, cx: &mut Context<Self>) {
        self.value = v.clamp(self.min, self.max);
        self.percentage = self.value_to_percentage(self.value);

        cx.notify();
    }

    fn value_to_percentage(&self, v: f32) -> f32 {
        let range = self.max - self.min;
        if range == 0.0 {
            0.0
        } else {
            (v - self.min) / range
        }
    }

    fn percentage_to_value(&self, p: f32) -> f32 {
        self.min + (self.max - self.min) * p
    }

    fn update_from_position(
        &mut self,
        position: Point<Pixels>,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let inner = position.x - self.bounds.left();
        let total = self.bounds.size.width;

        let p = (inner / total).clamp(0.0, 1.0);

        let mut value = self.percentage_to_value(p);
        value = (value / self.step).round() * self.step;

        self.value = value;
        self.percentage = p;

        cx.emit(SliderEvent::Change(value));
        cx.notify();
    }
}

impl EventEmitter<SliderEvent> for SliderState {}

#[derive(IntoElement)]
pub struct Slider {
    state: Entity<SliderState>,
    style: StyleRefinement,
    id: SharedString,
    height: Pixels,
}

impl Slider {
    pub fn new<T: Into<SharedString>>(state: &Entity<SliderState>, id: T, height: f32) -> Self {
        Self {
            state: state.clone(),
            style: StyleRefinement::default(),
            id: id.into(),
            height: px(height),
        }
    }
}

impl Styled for Slider {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for Slider {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let percentage = self.state.read(cx).percentage;

        let bar_color = self
            .style
            .background
            .clone()
            .and_then(|bg| bg.color())
            .unwrap_or(white().into());

        let fill_color = self.style.text.color.unwrap_or_else(white);

        let mut root = div()
            .id(("slider", self.state.entity_id()))
            .h(px(24.))
            .w_full()
            .flex()
            .items_center();

        let entity_id = self.state.entity_id();

        root.style().refine(&self.style);

        root.bg(transparent_black()).child(
            div()
                .id(self.id)
                .relative()
                .cursor_pointer()
                .w_full()
                .h(px(24.))
                .flex()
                .items_center()
                .on_mouse_down(
                    MouseButton::Left,
                    window.listener_for(
                        &self.state,
                        move |state, e: &MouseDownEvent, window, cx| {
                            state.update_from_position(e.position, window, cx);
                        },
                    ),
                )
                .on_drag(DragSlider(entity_id), |drag, _, _, cx| {
                    cx.new(|_| drag.clone())
                })
                .on_drag_move(window.listener_for(
                    &self.state,
                    move |state, e: &DragMoveEvent<DragSlider>, window, cx| match e.drag(cx) {
                        DragSlider(id) => {
                            if *id != entity_id {
                                return;
                            }

                            state.update_from_position(e.event.position, window, cx);
                        }
                    },
                ))
                .child(
                    div()
                        .id("inner_visual_bar")
                        .relative()
                        .w_full()
                        .h(self.height)
                        .bg(bar_color)
                        .rounded_full()
                        .child(
                            div()
                                .absolute()
                                .left(px(0.))
                                .right(relative(1.0 - percentage))
                                .h_full()
                                .bg(fill_color)
                                .rounded_full(),
                        )
                        .child(
                            div()
                                .absolute()
                                .left(relative(percentage))
                                .ml(-px(6.))
                                .size(px(12.))
                                .rounded_full(),
                        )
                        .hover(|this| this.bg(bar_color))
                        .active(|this| this.bg(bar_color))
                        .on_prepaint({
                            let state = self.state.clone();
                            move |bounds, _, cx| state.update(cx, |s, _| s.bounds = bounds)
                        }),
                ),
        )
    }
}
