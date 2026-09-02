use gpui::{ElementId, Entity, IntoElement, Pixels, ScrollHandle, div, px};
use gpui::Styled;
use std::{ops::Range, rc::Rc};

#[derive(Clone)]
pub struct VirtualGridScrollController {
    list: gpui::VirtualListScrollController,
}

impl VirtualGridScrollController {
    pub fn new() -> Self {
        Self {
            list: gpui::VirtualListScrollController::new(),
        }
    }

    pub fn scroll_to_item(&self, item_index: usize) {
        let _ = self.list.scroll_to_item(item_index);
    }
}

pub fn vgrid<R, V>(
    view: Entity<V>,
    id: impl Into<ElementId>,
    item_count: usize,
    min_card_width: Pixels,
    footer_height: Pixels,
    vertical_padding: Pixels,
    scroll_handle: ScrollHandle,
    controller: &VirtualGridScrollController,
    f: impl 'static
        + Fn(
            &mut V,
            Range<usize>,
            usize,
            &mut gpui::core::window::Window,
            &mut gpui::Context<V>,
        ) -> Vec<R>,
) -> gpui::VirtualList
where
    R: IntoElement + 'static,
    V: 'static,
{
    let columns = 4;
    let row_count = item_count.div_ceil(columns);
    let row_height = min_card_width + footer_height + vertical_padding + vertical_padding;
    let heights = Rc::new(vec![row_height; row_count]);

    gpui::vlist(
        view,
        id,
        heights,
        scroll_handle,
        controller.list.clone(),
        move |view, mut rows, window, cx| {
            let Some(row) = rows.next() else {
                return Vec::new();
            };
            let start = row * columns;
            let end = (start + columns).min(item_count);
            let items = f(view, start..end, columns, window, cx);
            if items.is_empty() {
                Vec::new()
            } else {
                vec![
                    div()
                        .h(px(row_height.value()))
                        .flex()
                        .children(items),
                ]
            }
        },
    )
}
