use gpui::{Context, ElementId, Entity, IntoElement, Pixels, Render, ScrollHandle};
use std::{ops::Range, rc::Rc};

pub use gpui::VirtualListScrollController;

pub fn vlist<R, V>(
    view: Entity<V>,
    id: impl Into<ElementId>,
    heights: Rc<Vec<Pixels>>,
    scroll_handle: ScrollHandle,
    controller: &VirtualListScrollController,
    f: impl 'static
        + Fn(
            &mut V,
            Range<usize>,
            &mut gpui::core::window::Window,
            &mut Context<V>,
        ) -> Vec<R>,
) -> gpui::VirtualList
where
    R: IntoElement + 'static,
    V: 'static,
{
    gpui::vlist(
        view,
        id,
        heights,
        scroll_handle,
        controller.clone(),
        f,
    )
}
