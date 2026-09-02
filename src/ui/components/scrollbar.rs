use gpui::{Bounds, ElementId, IntoElement, Pixels, RenderOnce, ScrollHandle, Window, App, div, px};
use gpui::Styled;

/// Scroll handle accepted by Wireman's floating scrollbar component.
///
/// WGPUI 2.0 uses one retained handle for ordinary and virtualized scrolling,
/// so this wrapper no longer needs the legacy uniform-list storage variant.
#[derive(Clone)]
pub struct ScrollableHandle(ScrollHandle);

impl From<ScrollHandle> for ScrollableHandle {
    fn from(handle: ScrollHandle) -> Self {
        Self(handle)
    }
}

impl ScrollableHandle {
    pub fn handle(&self) -> &ScrollHandle {
        &self.0
    }

    #[must_use]
    pub fn bounds(&self) -> Bounds<Pixels> {
        self.0.bounds()
    }

    #[must_use]
    pub fn offset(&self) -> gpui::Point<Pixels> {
        self.0.offset()
    }

    #[must_use]
    pub fn max_offset(&self) -> gpui::Size<Pixels> {
        self.0.max_offset()
    }

    pub fn set_offset(&self, offset: gpui::Point<Pixels>) {
        self.0.set_offset(offset);
    }

    #[must_use]
    pub fn total_content_height(&self) -> f32 {
        (self.0.bounds().size.height + self.0.max_offset().height).value()
    }
}

/// Retained scrollbar description backed by WGPUI's tested scrollbar
/// controller. This keeps pointer capture, paging, wheel propagation, and
/// drag clamping in the framework instead of duplicating legacy paint code.
#[derive(gpui::IntoElement)]
pub struct Scrollbar {
    id: Option<ElementId>,
    scroll_handle: Option<ScrollableHandle>,
}

impl Scrollbar {
    #[must_use]
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = Some(id.into());
        self
    }

    #[must_use]
    pub fn scroll_handle(mut self, handle: ScrollableHandle) -> Self {
        self.scroll_handle = Some(handle);
        self
    }
}

impl RenderOnce for Scrollbar {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let Some(handle) = self.scroll_handle else {
            return div().w(px(8.0)).h_full().into_description();
        };

        let controller = gpui::RetainedScrollbar::vertical(handle.handle());
        let viewport = handle.bounds();
        controller.set_track_bounds(Bounds::new(
            gpui::point(px(0.0), px(0.0)),
            gpui::size(px(8.0), viewport.size.height),
        ));

        let description = controller.description();
        match self.id {
            Some(id) => description.id(id),
            None => description,
        }
    }
}

#[must_use]
pub fn scrollbar() -> Scrollbar {
    Scrollbar {
        id: None,
        scroll_handle: None,
    }
}

#[derive(PartialEq, Eq)]
pub enum RightPad {
    None,
    Pad,
}

#[derive(IntoElement)]
pub struct FloatingScrollbar {
    id: ElementId,
    handle: ScrollableHandle,
    right_pad: RightPad,
}

impl RenderOnce for FloatingScrollbar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        div()
            .absolute()
            .top_0()
            .right(if self.right_pad == RightPad::Pad { px(4.0) } else { px(0.0) })
            .bottom_0()
            .my(px(4.0))
            .occlude()
            .child(scrollbar().id(self.id).scroll_handle(self.handle))
            .into_description_in(window.interaction_mut(), cx)
    }
}

pub fn floating_scrollbar(
    id: impl Into<ElementId>,
    handle: impl Into<ScrollableHandle>,
    right_pad: RightPad,
) -> FloatingScrollbar {
    FloatingScrollbar {
        id: id.into(),
        handle: handle.into(),
        right_pad,
    }
}
