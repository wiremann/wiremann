use crate::ui::theme::Theme;
use gpui::{App, Decorations, Description, Edges, Hsla, IntoElement, MouseButton, Pixels, Point, RenderOnce, ResizeEdge, Size, Window, div, point, px, public_window_callback, transparent_black};
use gpui::Styled;

#[cfg(not(target_os = "linux"))]
const SHADOW_SIZE: Pixels = px(0.0);
#[cfg(target_os = "linux")]
const SHADOW_SIZE: Pixels = px(12.0);
const RESIZE_HANDLE_SIZE: Pixels = px(5.0);
const BORDER_SIZE: Pixels = px(1.0);
#[cfg(target_os = "windows")]
pub(crate) const BORDER_RADIUS: Pixels = px(0.0);
#[cfg(not(target_os = "windows"))]
pub(crate) const BORDER_RADIUS: Pixels = px(8.0);

/// Renders the client-decoration border and keeps resize initiation at the
/// public WGPUI window boundary.
#[derive(Default, gpui::IntoElement)]
pub struct WindowBorder {
    children: Vec<Description>,
}

pub fn window_border() -> WindowBorder { WindowBorder::new() }

impl WindowBorder {
    pub fn new() -> Self { Self::default() }

    pub fn child<E: IntoElement>(mut self, child: E) -> Self {
        self.children.push(child.into_description());
        self
    }
}

pub fn window_paddings(window: &Window) -> Edges {
    match window.window_decorations() {
        Decorations::Server => Edges::all(0.0),
        Decorations::Client { tiling } => {
            let mut paddings = Edges::all(SHADOW_SIZE.value());
            if tiling.top { paddings.top = 0.0; }
            if tiling.bottom { paddings.bottom = 0.0; }
            if tiling.left { paddings.left = 0.0; }
            if tiling.right { paddings.right = 0.0; }
            paddings
        }
    }
}

impl RenderOnce for WindowBorder {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let decorations = window.window_decorations();
        window.set_client_inset(SHADOW_SIZE);

        let mut backdrop = div()
            .id("window-backdrop")
            .size_full()
            .bg(transparent_black());

        if let Decorations::Client { tiling } = decorations {
            backdrop = backdrop
                .when(!(tiling.top || tiling.right), |element| element.rounded_tr(BORDER_RADIUS.value()))
                .when(!(tiling.top || tiling.left), |element| element.rounded_tl(BORDER_RADIUS.value()))
                .when(!tiling.top, |element| element.pt(SHADOW_SIZE))
                .when(!tiling.bottom, |element| element.pb(SHADOW_SIZE))
                .when(!tiling.left, |element| element.pl(SHADOW_SIZE))
                .when(!tiling.right, |element| element.pr(SHADOW_SIZE))
                .on_mouse_down(MouseButton::Left, public_window_callback(|_, window, _app| {
                    let size = window.bounds().size;
                    let position = window.mouse_position();
                    if let Some(edge) = resize_edge(position, RESIZE_HANDLE_SIZE, size)
                        && let Err(error) = window.start_window_resize(edge)
                    {
                        tracing::warn!(?error, "unable to start client-window resize");
                    }
                }));
        }

        let mut content = div()
            .id("window-content")
            .size_full()
            .bg(transparent_black())
            .children(self.children);

        if let Decorations::Client { tiling } = decorations {
            content = content
                .when(!(tiling.top || tiling.right), |element| element.rounded_tr(BORDER_RADIUS.value()))
                .when(!(tiling.top || tiling.left), |element| element.rounded_tl(BORDER_RADIUS.value()))
                .border_color(theme.border)
                .when(!tiling.top, |element| element.border_t(BORDER_SIZE))
                .when(!tiling.bottom, |element| element.border_b(BORDER_SIZE))
                .when(!tiling.left, |element| element.border_l(BORDER_SIZE))
                .when(!tiling.right, |element| element.border_r(BORDER_SIZE))
                .when(!tiling.is_tiled(), |element| {
                    element.shadow(vec![gpui::BoxShadow {
                        color: Hsla { h: 0.0, s: 0.0, l: 0.0, a: 0.3 },
                        blur_radius: px(SHADOW_SIZE.value() / 2.0),
                        spread_radius: px(0.0),
                        offset: point(px(0.0), px(0.0)),
                    }])
                });
        }

        backdrop.child(content)
    }
}

fn resize_edge(position: Point<Pixels>, resize_size: Pixels, size: Size<Pixels>) -> Option<ResizeEdge> {
    let edge = if position.y < resize_size && position.x < resize_size {
        ResizeEdge::TopLeft
    } else if position.y < resize_size && position.x > size.width - resize_size {
        ResizeEdge::TopRight
    } else if position.y < resize_size {
        ResizeEdge::Top
    } else if position.y > size.height - resize_size && position.x < resize_size {
        ResizeEdge::BottomLeft
    } else if position.y > size.height - resize_size && position.x > size.width - resize_size {
        ResizeEdge::BottomRight
    } else if position.y > size.height - resize_size {
        ResizeEdge::Bottom
    } else if position.x < resize_size {
        ResizeEdge::Left
    } else if position.x > size.width - resize_size {
        ResizeEdge::Right
    } else {
        return None;
    };
    Some(edge)
}
