use gpui::{Bounds, Description, ElementId, IntoElement, Pixels, point, px, size};

/// Attach a retained layout observer to an element.
pub fn observe_bounds<E>(
    id: impl Into<ElementId>,
    child: E,
    mut on_change: impl FnMut(Bounds<Pixels>) + 'static,
) -> Description
where
    E: IntoElement,
{
    child
        .into_description()
        .id(id)
        .on_layout_changed(move |layout| {
            on_change(Bounds::new(
                point(px(layout.x), px(layout.y)),
                size(px(layout.width), px(layout.height)),
            ));
            false
        })
}
