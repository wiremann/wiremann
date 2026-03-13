use std::rc::Rc;

use crate::{
    controller::Controller,
    ui::{
        components::image_cache::ImageCache,
        theme::Theme,
    },
};

use crate::ui::components::virtual_list::vlist;
use gpui::{
    div, px, App, AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled,
    UniformListScrollHandle, Window,
};

#[derive(Clone)]
pub struct LibraryPage {
    scroll_handle: UniformListScrollHandle,
    show_playlists: Entity<bool>,
    test_heights: Rc<Vec<gpui::Pixels>>,
}

impl LibraryPage {
    pub fn new(cx: &mut App) -> Self {
        let scroll_handle = UniformListScrollHandle::new();
        let show_playlists = cx.new(|_| true);

        let mut heights = Vec::new();

        for i in 0..1000 {
            if i % 20 == 0 {
                heights.push(px(120.0));
            } else {
                heights.push(px(32.0));
            }
        }

        let test_heights = Rc::new(heights);

        LibraryPage {
            scroll_handle,
            show_playlists,
            test_heights,
        }
    }

    fn render_test_row(&mut self, ix: usize) -> impl IntoElement {
        println!("rendering row {}", ix);

        let height = self.test_heights[ix];

        div()
            .h(height)
            .w_full()
            .border_1()
            .items_center()
            .pl(px(10.0))
            .child(format!("Row {}", ix))
    }
}

impl Render for LibraryPage {
    #[allow(clippy::too_many_lines)]
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>();

        let controller = cx.global::<Controller>().clone();
        let state = controller.state.read(cx);
        let _thumbnail = cx.global::<ImageCache>().current.clone();
        let _scroll_handle = self.scroll_handle.clone();
        let _show_playlists = self.show_playlists.clone();

        let _current = if let Some(id) = state.playback.current {
            state.library.tracks.get(&id)
        } else {
            None
        };

        let heights = self.test_heights.clone();

        div()
            .size_full()
            .bg(theme.bg_main)
            .child(
                vlist(cx.entity(), "library-test", heights, |this, range, _, _| {
                    range
                        .map(|i| this.render_test_row(i))
                        .collect::<Vec<_>>()
                }),
            )
    }
}