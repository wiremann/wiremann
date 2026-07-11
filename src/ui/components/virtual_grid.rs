use gpui::{
    AnyElement, App, AvailableSpace, Bounds, ContentMask, Context, Div, Element, ElementId, Entity,
    GlobalElementId, Hitbox, InteractiveElement, IntoElement, Pixels, Render, ScrollHandle, Size,
    SmoothScrollState, Stateful, StatefulInteractiveElement, Styled, Window, div, point, px, size,
};
use smallvec::SmallVec;
use std::{cell::RefCell, cmp, ops::Range, rc::Rc};

#[derive(Clone, Copy, Debug)]
pub struct DeferredGridScroll {
    pub item_index: usize,
}

#[derive(Debug, Default)]
pub struct VirtualGridScrollState {
    pub deferred_scroll: Option<DeferredGridScroll>,
    pub smooth_scroll: SmoothScrollState,
}

#[derive(Clone)]
pub struct VirtualGridScrollController {
    pub state: Rc<RefCell<VirtualGridScrollState>>,
}

impl VirtualGridScrollController {
    pub fn new() -> Self {
        Self {
            state: Rc::new(RefCell::new(VirtualGridScrollState::default())),
        }
    }

    pub fn scroll_to_item(&self, item_index: usize) {
        self.state.borrow_mut().deferred_scroll = Some(DeferredGridScroll { item_index });
    }
}

pub struct VirtualGrid {
    id: ElementId,
    base: Stateful<Div>,
    scroll_handle: ScrollHandle,
    item_count: usize,
    card_width: Pixels,
    card_height: Pixels,
    content_height: Pixels,
    scroll_state: Rc<RefCell<VirtualGridScrollState>>,
    render: Box<
        dyn for<'a> Fn(
            Range<usize>,
            usize,
            &'a mut Window,
            &'a mut App,
        ) -> SmallVec<[AnyElement; 32]>,
    >,
    overscan: usize,
}

pub fn vgrid<R, V>(
    view: Entity<V>,
    id: impl Into<ElementId>,
    item_count: usize,
    card_width: Pixels,
    card_height: Pixels,
    scroll_handle: ScrollHandle,
    controller: &VirtualGridScrollController,
    f: impl 'static + Fn(&mut V, Range<usize>, usize, &mut Window, &mut Context<V>) -> Vec<R>,
) -> VirtualGrid
where
    R: IntoElement,
    V: Render,
{
    let id = id.into();

    let render = move |range: Range<usize>, cols: usize, window: &mut Window, cx: &mut App| {
        view.update(cx, |this, cx| {
            f(this, range, cols, window, cx)
                .into_iter()
                .map(gpui::IntoElement::into_any_element)
                .collect()
        })
    };

    let base = div()
        .id(id.clone())
        .size_full()
        .overflow_scroll()
        .track_scroll(&scroll_handle);

    VirtualGrid {
        id,
        base,
        scroll_handle,
        item_count,
        card_width,
        card_height,
        content_height: px(0.0),
        scroll_state: controller.state.clone(),
        render: Box::new(render),
        overscan: 1,
    }
}

pub struct FrameState {
    items: SmallVec<[AnyElement; 32]>,
}

impl IntoElement for VirtualGrid {
    type Element = Self;
    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for VirtualGrid {
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
            |style, window: &mut Window, cx| window.request_layout(style, None, cx),
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

        let available_width_px: f32 = bounds.size.width.into();
        let card_width_px: f32 = self.card_width.into();
        let card_height_px: f32 = self.card_height.into();

        let mut cols = (available_width_px / card_width_px).floor() as usize;
        if cols == 0 {
            cols = 1;
        }

        let rows = (self.item_count + cols - 1) / cols;

        self.content_height = px(rows as f32 * card_height_px);
        let mut logical_scroll = self.scroll_handle.offset().y;

        if let Some(deferred) = self.scroll_state.borrow_mut().deferred_scroll.take() {
            let target = deferred.item_index.min(self.item_count.saturating_sub(1));
            let target_row = target / cols;
            let item_top = target_row as f32 * card_height_px;
            let target_scroll = -item_top.max(0.0);
            self.scroll_handle
                .set_offset(point(px(0.0), px(target_scroll)));
            logical_scroll = px(target_scroll);
        }

        let max_scroll = (self.content_height - viewport_height).max(px(0.0));

        logical_scroll = logical_scroll.clamp(-max_scroll, px(0.0));

        self.scroll_handle
            .set_offset(point(px(0.0), logical_scroll));

        let visual_scroll = {
            let mut state = self.scroll_state.borrow_mut();
            state.smooth_scroll.set_target(logical_scroll);
            if state.smooth_scroll.update() {
                window.refresh();
            }
            state.smooth_scroll.current()
        };

        let visual_scroll_px: f32 = visual_scroll.into();
        let viewport_height_px: f32 = viewport_height.into();

        let mut start_row = ((-visual_scroll_px) / card_height_px).floor() as isize;
        let mut end_row =
            ((-visual_scroll_px + viewport_height_px) / card_height_px).ceil() as isize;

        if start_row < 0 {
            start_row = 0;
        }
        if end_row < 0 {
            end_row = 0;
        }

        let start_row_usize = start_row as usize;
        let mut end_row_usize = end_row as usize;
        end_row_usize = cmp::min(end_row_usize + self.overscan, rows);

        let visible_start_item = start_row_usize.saturating_mul(cols);
        let visible_end_item = cmp::min(end_row_usize.saturating_mul(cols), self.item_count);

        let items = (self.render)(visible_start_item..visible_end_item, cols, window, cx);

        let content_mask = ContentMask { bounds };

        window.with_content_mask(Some(content_mask), |window| {
            for (mut item, ix) in items.into_iter().zip(visible_start_item..visible_end_item) {
                let row = ix / cols;
                let col = ix % cols;

                let y = px(row as f32 * card_height_px) + visual_scroll;
                let cell_width = bounds.size.width / (cols as f32);
                let origin = bounds.origin + point(cell_width * (col as f32), y);

                item.layout_as_root(
                    size(
                        AvailableSpace::Definite(cell_width),
                        AvailableSpace::Definite(self.card_height),
                    ),
                    window,
                    cx,
                );
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
