use gpui::{
    AnyElement, App, Bounds, Element, ElementId, GlobalElementId, InspectorElementId, IntoElement,
    LayoutId, Pixels, Window,
};

pub struct BoundsObserver {
    id: ElementId,
    child: Option<AnyElement>,
    last_bounds: Option<Bounds<Pixels>>,
    on_change: Box<dyn FnMut(Bounds<Pixels>, &mut Window, &mut App) + 'static>,
}

pub fn observe_bounds<E>(
    id: impl Into<ElementId>,
    child: E,
    on_change: impl FnMut(Bounds<Pixels>, &mut Window, &mut App) + 'static,
) -> BoundsObserver
where
    E: IntoElement,
{
    BoundsObserver {
        id: id.into(),
        child: Some(child.into_any_element()),
        last_bounds: None,
        on_change: Box::new(on_change),
    }
}

impl Element for BoundsObserver {
    type RequestLayoutState = AnyElement;
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        Some(self.id.clone())
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut child = self.child.take().unwrap();

        let layout_id = child.request_layout(window, cx);

        (layout_id, child)
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        child: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        child.prepaint(window, cx);

        let changed = self.last_bounds != Some(bounds);

        if changed {
            self.last_bounds = Some(bounds);

            (self.on_change)(bounds, window, cx);
        }
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: Bounds<Pixels>,
        child: &mut Self::RequestLayoutState,
        _prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        child.paint(window, cx);
    }
}

impl IntoElement for BoundsObserver {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}
