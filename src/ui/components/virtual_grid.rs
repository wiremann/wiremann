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
    top_padding: Pixels,
    bottom_padding: Pixels,
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

    let base = div().id(id.clone()).size_full();

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
        top_padding: px(0.0),
        bottom_padding: px(0.0),
    }
}

impl VirtualGrid {
    fn row_at_position(&self, pos: f32, card_h: f32) -> isize {
        ((pos) / card_h).floor() as isize
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
        self.top_padding = viewport_height / 2.0;
        self.bottom_padding = viewport_height / 2.0;

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
            let item_center =
                (self.top_padding.to_f64() as f32 + item_top + (card_height_px / 2.0)) as f32;
            let centered = item_center - (viewport_height.to_f64() / 2.0) as f32;
            let new_scroll = -centered.max(0.0);
            self.scroll_handle
                .set_offset(point(px(0.0), px(new_scroll)));
            logical_scroll = px(new_scroll);
        }

        let padded_height = self.content_height + self.top_padding + self.bottom_padding;

        let max_scroll = (padded_height - viewport_height).max(px(0.0));

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
        let top_padding_px: f32 = self.top_padding.into();
        let viewport_height_px: f32 = viewport_height.into();

        let mut start_row =
            ((-visual_scroll_px - top_padding_px) / card_height_px).floor() as isize;
        let mut end_row = ((-visual_scroll_px - top_padding_px + viewport_height_px)
            / card_height_px)
            .ceil() as isize;

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

                let y = self.top_padding + px(row as f32 * card_height_px) + visual_scroll;
                let cell_width = bounds.size.width / (cols as f32);
                let origin = bounds.origin + point(cell_width * (col as f32), y);

                let available = Size {
                    width: gpui::AvailableSpace::Definite(cell_width),
                    height: gpui::AvailableSpace::Definite(self.card_height),
                };

                item.layout_as_root(
                    size(
                        gpui::AvailableSpace::Definite(cell_width),
                        gpui::AvailableSpace::Definite(self.card_height),
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
                height: padded_height,
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
