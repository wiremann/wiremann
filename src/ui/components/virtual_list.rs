use std::{cmp, ops::Range, rc::Rc};

use gpui::{
    AnyElement, App, Bounds, ContentMask, Context, Div, Element, ElementId, Entity,
    GlobalElementId, Hitbox, InteractiveElement, IntoElement, Pixels, Render, ScrollHandle, Size,
    Stateful, StatefulInteractiveElement, Styled, Window, div, point, px, size,
};
use smallvec::SmallVec;

#[allow(clippy::type_complexity)]
pub struct VirtualList {
    id: ElementId,
    base: Stateful<Div>,
    scroll_handle: ScrollHandle,
    heights: Rc<Vec<Pixels>>,
    offsets: Vec<Pixels>,
    content_height: Pixels,
    render: Box<
        dyn for<'a> Fn(Range<usize>, &'a mut Window, &'a mut App) -> SmallVec<[AnyElement; 32]>,
    >,
    overscan: usize,
}

pub fn vlist<R, V>(
    view: Entity<V>,
    id: impl Into<ElementId>,
    heights: Rc<Vec<Pixels>>,
    scroll_handle: ScrollHandle,
    f: impl 'static + Fn(&mut V, Range<usize>, &mut Window, &mut Context<V>) -> Vec<R>,
) -> VirtualList
where
    R: IntoElement,
    V: Render,
{
    let id = id.into();

    let render = move |range: Range<usize>, window: &mut Window, cx: &mut App| {
        view.update(cx, |this, cx| {
            f(this, range, window, cx)
                .into_iter()
                .map(gpui::IntoElement::into_any_element)
                .collect()
        })
    };

    let mut offsets = Vec::with_capacity(heights.len());
    let mut sum = px(0.0);

    for h in heights.iter() {
        offsets.push(sum);
        sum += *h;
    }

    let base = div()
        .id(id.clone())
        .size_full()
        .overflow_scroll()
        .track_scroll(&scroll_handle);

    VirtualList {
        id,
        base,
        scroll_handle,
        heights,
        offsets,
        content_height: sum,
        render: Box::new(render),
        overscan: 16,
    }
}

impl VirtualList {
    fn find_index(&self, pos: Pixels) -> usize {
        self.offsets.partition_point(|&o| o < pos)
    }
}

pub struct FrameState {
    items: SmallVec<[AnyElement; 32]>,
}

impl IntoElement for VirtualList {
    type Element = Self;
    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for VirtualList {
    type RequestLayoutState = FrameState;
    type PrepaintState = Option<Hitbox>;

    fn id(&self) -> Option<ElementId> {
        Some(self.id.clone())
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        global_id: Option<&GlobalElementId>,
        inspector_id: Option<&gpui::InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (gpui::LayoutId, Self::RequestLayoutState) {
        let layout_id = self.base.interactivity().request_layout(
            global_id,
            inspector_id,
            window,
            cx,
            |style, window, cx| window.request_layout(style, None, cx),
        );

        (
            layout_id,
            FrameState {
                items: SmallVec::new(),
            },
        )
    }

    fn prepaint(
        &mut self,
        global_id: Option<&GlobalElementId>,
        inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let viewport_height = bounds.size.height;
        let scroll = self.scroll_handle.offset().y;

        let mut start = self.find_index(-scroll);
        let mut end = self.find_index(-scroll + viewport_height);

        start = start.saturating_sub(self.overscan);
        end = cmp::min(end + self.overscan + 1, self.heights.len());

        let visible = start..end;

        let items = (self.render)(visible.clone(), window, cx);

        let _content_bounds = Bounds {
            origin: bounds.origin,
            size: size(bounds.size.width, self.content_height),
        };

        let content_mask = ContentMask { bounds };

        window.with_content_mask(Some(content_mask), |window| {
            for (mut item, ix) in items.into_iter().zip(visible.clone()) {
                let y = self.offsets[ix] + scroll;

                let origin = bounds.origin + point(px(0.), y);

                let available = size(
                    gpui::AvailableSpace::Definite(bounds.size.width),
                    gpui::AvailableSpace::Definite(self.heights[ix]),
                );

                item.layout_as_root(available, window, cx);
                item.prepaint_at(origin, window, cx);

                layout.items.push(item);
            }
        });

        self.base.interactivity().prepaint(
            global_id,
            inspector_id,
            bounds,
            Size {
                width: bounds.size.width,
                height: self.content_height,
            },
            window,
            cx,
            |_style, _, hitbox, _, _| hitbox,
        )
    }

    fn paint(
        &mut self,
        global_id: Option<&GlobalElementId>,
        inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        layout: &mut Self::RequestLayoutState,
        hitbox: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        self.base.interactivity().paint(
            global_id,
            inspector_id,
            bounds,
            hitbox.as_ref(),
            window,
            cx,
            |_, window, cx| {
                for item in &mut layout.items {
                    item.paint(window, cx);
                }
            },
        );
    }
}
