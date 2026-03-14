use crate::cacher::ImageKind;
use crate::library::{ImageId, TrackId};
use crate::ui::components::image_cache::ImageCache;
use crate::ui::theme::Theme;
use crate::{controller::Controller, library::Track};
use ahash::AHashMap;
use gpui::prelude::FluentBuilder;
use gpui::{div, img, px, uniform_list, App, AppContext, Context, Entity, InteractiveElement, IntoElement, ObjectFit, ParentElement, Render, ScrollStrategy, StatefulInteractiveElement, Styled, StyledImage, UniformListScrollHandle, Window};
use std::path::PathBuf;
use std::sync::Arc;

const THUMBNAIL_MARGIN: usize = 16;

struct ItemData {
    id: TrackId,
    title: String,
    artist: String,
    image_id: Option<ImageId>,
}

#[allow(unused)]
pub struct Item {
    data: ItemData,
    idx: usize,
}

impl Item {
    pub fn new(cx: &mut App, track: Arc<Track>, idx: usize) -> Entity<Self> {
        cx.new(move |_| {
            let track = track.clone();

            let data = ItemData {
                id: track.id,
                title: track.title.clone(),
                artist: track.artist.clone(),
                image_id: track.image_id.clone(),
            };

            Self { data, idx }
        })
    }
}

impl Render for Item {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let thumbnail = self.data.image_id.and_then(|id| {
            cx.global_mut::<ImageCache>().get(&id)
        });

        let theme = cx.global::<Theme>();
        let state = cx.global::<Controller>().state.read(cx);

        let is_current = Some(&self.data.id) == state.playback.current.as_ref();

        let current = if let Some(id) = state.playback.current {
            state.library.tracks.get(&id)
        } else {
            None
        };

        let path = if let Some(track) = current {
            track.path.clone()
        } else {
            PathBuf::new()
        };
        div()
            .id(format!("track_item_{}", path.to_string_lossy()))
            .h(px(64.))
            .w_full()
            .flex()
            .items_center()
            .p_3()
            .gap_4()
            .mb_2()
            .rounded_lg()
            .hover(|d| d.bg(theme.white_05))
            .when(is_current, |d| d.bg(theme.accent_15))
            .child(match thumbnail {
                Some(image) => div().size_12().flex_shrink_0().child(
                    img(image.clone())
                        .object_fit(ObjectFit::Contain)
                        .size_full()
                        .rounded_md(),
                ),
                None => div().size_12().flex_shrink_0(),
            })
            .child(
                div()
                    .flex_col()
                    .flex_1()
                    .child(
                        div()
                            .text_base()
                            .truncate()
                            .text_color(if is_current {
                                theme.accent
                            } else {
                                theme.text_primary
                            })
                            .child(self.data.title.clone()),
                    )
                    .child(
                        div()
                            .text_sm()
                            .truncate()
                            .text_color(theme.text_muted)
                            .child(self.data.artist.clone()),
                    ),
            )
    }
}

#[derive(Clone)]
pub struct Queue {
    pub views: Entity<AHashMap<TrackId, Entity<Item>>>,
    pub scroll_handle: UniformListScrollHandle,
    pub stop_auto_scroll: Entity<bool>,

    last_tracks: Vec<TrackId>,
    last_order: Vec<usize>,
    last_current: Option<TrackId>,
}

impl Queue {
    pub fn new(cx: &mut App, scroll_handle: UniformListScrollHandle) -> Entity<Self> {
        cx.new(|cx| Self {
            views: cx.new(|_| AHashMap::new()),
            scroll_handle,
            stop_auto_scroll: cx.new(|_| false),

            last_tracks: vec![],
            last_order: vec![],
            last_current: None,
        })
    }

    fn get_or_create_item(
        views: &Entity<AHashMap<TrackId, Entity<Item>>>,
        track: Arc<Track>,
        cx: &mut App,
    ) -> Entity<Item> {
        let key = track.id;
        views.update(cx, |this, cx| {
            this.entry(key)
                .or_insert_with(|| Item::new(cx, track, 0))
                .clone()
        })
    }

    pub fn scroll_to_item(&self, cx: &mut App) {
        let controller = cx.global::<Controller>();
        let state = controller.state.read(cx);

        let idx = if let Some(current) = &state.playback.current {
            state
                .queue
                .order
                .iter()
                .position(|&i| state.queue.tracks[i] == *current)
                .unwrap_or(0)
        } else {
            0
        };

        if !self.stop_auto_scroll.read(cx) {
            self.scroll_handle
                .scroll_to_item(idx, ScrollStrategy::Nearest);
        }
    }
}

impl Render for Queue {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let views = self.views.clone();
        let stop_auto_scroll = self.stop_auto_scroll.clone();

        let state = cx.global::<Controller>().state.read(cx).clone();

        let tracks = state.queue.tracks.clone();
        let order = state.queue.order.clone();
        let current = state.playback.current;

        let queue_changed = self.last_tracks != tracks || self.last_order != order;
        let current_changed = self.last_current != current;

        if queue_changed {
            self.views.update(cx, |map, _| map.clear());

            self.last_tracks.clone_from(&tracks);
            self.last_order.clone_from(&order);
        }

        if (queue_changed || current_changed) && !self.stop_auto_scroll.read(cx) {
            let this = self.clone();
            cx.defer(move |cx| {
                this.scroll_to_item(cx);
            });

            self.last_current = current;
        }

        let tracks = self.last_tracks.clone();
        let queue_order = self.last_order.clone();
        let len = queue_order.len();
        let scroll_handle = self.scroll_handle.clone();

        div()
            .id("queue_container")
            .on_hover(move |hovered, _, cx| stop_auto_scroll.update(cx, |this, _| *this = *hovered))
            .size_full()
            .child(
                uniform_list("queue", len, move |range, _, cx| {
                    let visible_tracks: Vec<TrackId> = range
                        .clone()
                        .map(|i| {
                            let real_index = &tracks[queue_order[i]];
                            if let Some(track) = state.library.tracks.get(real_index) {
                                track.id
                            } else {
                                TrackId::default()
                            }
                        })
                        .collect();

                    let start = range.start.saturating_sub(THUMBNAIL_MARGIN);
                    let end = (range.end + THUMBNAIL_MARGIN).min(len);

                    let ids = {
                        let state = cx.global::<Controller>().state.read(cx);

                        (start..end)
                            .filter_map(|i| {
                                let track_id = tracks[queue_order[i]];
                                state.library
                                    .tracks
                                    .get(&track_id)
                                    .and_then(|track| track.image_id)
                            })
                            .collect::<Vec<_>>()
                    };

                    let tx = cx.global::<Controller>().cacher_tx.clone();
                    cx.global_mut::<ImageCache>().request(ids, &tx, ImageKind::Thumbnail);

                    views.update(cx, |map, _| {
                        map.retain(|id, _| visible_tracks.contains(id));
                    });

                    range
                        .map(|i| {
                            let real_index = &tracks[queue_order[i]];
                            if let Some(track) = state.library.tracks.get(real_index) {
                                let path = track.path.clone();

                                div()
                                    .id(format!("track_{}", path.to_string_lossy()))
                                    .child(Queue::get_or_create_item(&views, track.clone(), cx))
                                    .on_click(move |_, _, cx| {
                                        cx.global::<Controller>().load_audio(path.clone());
                                    })
                            } else {
                                div()
                                    .id("undefined")
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .child("Track not loaded...")
                            }
                        })
                        .collect()
                })
                    .w_full()
                    .h_full()
                    .flex()
                    .flex_col()
                    .track_scroll(&scroll_handle),
            )
    }
}
