use crate::ui::components::bounds_observer::observe_bounds;
use gpui::{App, Bounds, Context, Entity, Hsla, IntoElement, MouseButton, Pixels, Point, RenderOnce, SharedString, Window, div, point, px, public_window_callback, relative, white};
use gpui::Styled;

pub struct SliderState {
    min: f32,
    max: f32,
    step: f32,
    value: f32,
    percentage: f32,
    bounds: Bounds<Pixels>,
    on_change: Option<Box<dyn FnMut(f32, &mut Context<Self>)>>,
}

impl Default for SliderState {
    fn default() -> Self { Self::new() }
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
            on_change: None,
        }
    }

    #[must_use]
    pub fn min(mut self, value: f32) -> Self { self.min = value; self }
    #[must_use]
    pub fn max(mut self, value: f32) -> Self { self.max = value; self }
    #[must_use]
    pub fn step(mut self, value: f32) -> Self { self.step = value; self }

    #[must_use]
    pub fn default_value(mut self, value: f32) -> Self {
        self.value = value;
        self.percentage = self.value_to_percentage(value);
        self
    }

    #[must_use]
    pub fn on_change(mut self, callback: impl FnMut(f32, &mut Context<Self>) + 'static) -> Self {
        self.on_change = Some(Box::new(callback));
        self
    }

    #[must_use]
    pub fn value(&self) -> f32 { self.value }

    pub fn set_value(&mut self, value: f32, cx: &mut Context<Self>) {
        self.value = value.clamp(self.min, self.max);
        self.percentage = self.value_to_percentage(self.value);
        cx.notify();
    }

    fn value_to_percentage(&self, value: f32) -> f32 {
        let range = self.max - self.min;
        if range == 0.0 { 0.0 } else { (value - self.min) / range }
    }

    fn percentage_to_value(&self, percentage: f32) -> f32 {
        self.min + (self.max - self.min) * percentage
    }

    fn update_from_position(&mut self, position: Point<Pixels>, cx: &mut Context<Self>) {
        let total = self.bounds.size.width;
        if total <= px(0.0) { return; }

        let percentage = ((position.x - self.bounds.left()) / total).clamp(0.0, 1.0);
        let value = (self.percentage_to_value(percentage) / self.step).round() * self.step;
        self.value = value.clamp(self.min, self.max);
        self.percentage = self.value_to_percentage(self.value);
        if let Some(callback) = self.on_change.as_mut() {
            callback(self.value, cx);
        }
        cx.notify();
    }
}

#[derive(IntoElement)]
pub struct Slider {
    state: Entity<SliderState>,
    fill_color: Hsla,
    track_color: Hsla,
    id: SharedString,
    height: Pixels,
}

impl Slider {
    pub fn new<T: Into<SharedString>>(state: &Entity<SliderState>, id: T, height: f32) -> Self {
        Self {
            state: state.clone(),
            fill_color: white(),
            track_color: white(),
            id: id.into(),
            height: px(height),
        }
    }

    #[must_use]
    pub fn text_color(mut self, color: impl Into<Hsla>) -> Self {
        self.fill_color = color.into();
        self
    }

    #[must_use]
    pub fn bg(mut self, color: impl Into<Hsla>) -> Self {
        self.track_color = color.into();
        self
    }
}

impl RenderOnce for Slider {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let percentage = self.state.read(cx).percentage;
        let track = div()
            .id(self.id.as_str())
            .relative()
            .cursor_pointer()
            .w_full()
            .h(px(24.0))
            .flex()
            .items_center()
            .on_mouse_down(MouseButton::Left, public_window_callback({
                let state = self.state.clone();
                move |event, _window, _app| {
                    if let gpui::InputEvent::MouseDown(event) = event {
                        state.update((), |state, cx| {
                            state.update_from_position(point(event.position[0], event.position[1]), cx);
                        });
                    }
                }
            }))
            .on_mouse_move(public_window_callback({
                let state = self.state.clone();
                move |event, _window, _app| {
                    if event.dragging() {
                        state.update((), |state, cx| {
                            state.update_from_position(point(event.position[0], event.position[1]), cx);
                        });
                    }
                }
            }))
            .child(
                div()
                    .id("inner_visual_bar")
                    .relative()
                    .w_full()
                    .h(self.height)
                    .bg(self.track_color)
                    .rounded_full()
                    .child(
                        div()
                            .absolute()
                            .left(px(0.0))
                            .right(relative(1.0 - percentage))
                            .h_full()
                            .bg(self.fill_color)
                            .rounded_full(),
                    )
                    .child(
                        div()
                            .absolute()
                            .left(relative(percentage))
                            .ml(-px(6.0))
                            .size(px(12.0))
                            .rounded_full()
                            .bg(self.fill_color),
                    ),
            );

        let state = self.state.clone();
        observe_bounds("slider_track_bounds", track, move |bounds| {
            state.update((), |state, cx| {
                if state.bounds != bounds {
                    state.bounds = bounds;
                    cx.notify();
                }
            });
        })
    }
}
