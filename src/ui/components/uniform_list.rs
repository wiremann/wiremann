use gpui::{App, Div, ElementId, Pixels, Styled, UniformList, Window, div, px, uniform_list};
use std::ops::Range;

pub fn uniform_list_with_scroll_past(
    id: impl Into<ElementId>,
    item_count: usize,
    scroll_past: Pixels,
    row_height: Pixels,
    render_items: impl 'static + Fn(Range<usize>, &mut Window, &mut App) -> Vec<Div>,
) -> UniformList {
    let scroll_past: f32 = scroll_past.max(gpui::px(0.0)).into();
    let row_height: f32 = row_height.into();
    let extra_rows = if item_count == 0 || row_height <= 0.0 {
        0
    } else {
        (scroll_past / row_height).ceil() as usize
    };

    uniform_list(
        id,
        item_count.saturating_add(extra_rows),
        move |range, window, cx| {
            let visible_item_end = range.end.min(item_count);
            let mut items = if range.start < visible_item_end {
                render_items(range.start..visible_item_end, window, cx)
            } else {
                Vec::new()
            };

            items.extend((range.start.max(item_count)..range.end).map(|_| div().h(px(row_height))));
            items
        },
    )
}
