use crate::audio::engine::{AudioEngine, PlaybackState};
use crate::controller::player::{
    AudioCommand, AudioEvent, Controller, Event, PlayerState, ResHandler, ScannerCommand,
    ScannerEvent,
};
use crate::scanner::{Scanner, ScannerState};
use crate::ui::assets::Assets;
use crate::ui::{image_cache::ImageCache, wiremann::Wiremann};
use crossbeam_channel::unbounded;
use gpui::*;
use gpui_component::*;
use std::sync::Arc;
use std::{thread, time::Duration};

pub fn run() {
    let (audio_cmd_tx, audio_cmd_rx) = unbounded::<AudioCommand>();
    let (audio_events_tx, audio_events_rx) = unbounded::<AudioEvent>();
    let (scanner_cmd_tx, scanner_cmd_rx) = unbounded::<ScannerCommand>();
    let (scanner_events_tx, scanner_events_rx) = unbounded::<ScannerEvent>();

    thread::spawn(move || {
        AudioEngine::run(audio_cmd_rx, audio_events_tx);
    });

    thread::spawn(move || {
        Scanner::run(scanner_cmd_rx, scanner_events_tx);
    });

    let controller = Controller::new(
        audio_cmd_tx,
        audio_events_rx,
        scanner_cmd_tx,
        scanner_events_rx,
        PlayerState::default(),
        ScannerState::default(),
    );

    let assets = Assets {};
    let app = Application::new().with_assets(assets.clone());

    app.run(move |cx| {
        gpui_component::init(cx);
        let bounds = Bounds::centered(None, size(px(1280.0), px(760.0)), cx);
        assets.load_fonts(cx).expect("Could not load fonts");

        cx.spawn(async move |cx| {
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    app_id: Some(String::from("wiremann")),
                    focus: true,
                    titlebar: Some(TitlebarOptions {
                        title: None,
                        appears_transparent: true,
                        ..Default::default()
                    }),
                    window_min_size: Some(gpui::Size {
                        width: px(800.0),
                        height: px(740.0),
                    }),
                    ..Default::default()
                },
                |window, cx| {
                    let controller_evt_clone = controller.clone();

                    cx.set_global(controller);

                    let view = cx.new(|cx| Wiremann::new(cx));

                    cx.new(|cx| {
                        let res_handler = cx.new(|_| ResHandler {});
                        let arc_res = Arc::new(res_handler.clone());
                        cx.spawn(async move |_, cx| {
                            let res_handler = arc_res.clone();
                            loop {
                                while let Ok(event) =
                                    controller_evt_clone.audio_events_rx.try_recv()
                                {
                                    res_handler.update(&mut cx.clone(), |res_handler, cx| {
                                        res_handler.handle(cx, Event::Audio(event));
                                    });
                                }
                                while let Ok(event) =
                                    controller_evt_clone.scanner_events_rx.try_recv()
                                {
                                    res_handler.update(&mut cx.clone(), |res_handler, cx| {
                                        res_handler.handle(cx, Event::Scanner(event));
                                    });
                                }
                                cx.background_executor()
                                    .timer(Duration::from_millis(50))
                                    .await;
                            }
                        })
                            .detach();

                        let playbar_view = view.clone();

                        cx.subscribe(&res_handler, move |_, _, event: &Event, cx| match event {
                            Event::Audio(audio_event) => match audio_event {
                                AudioEvent::PlayerStateChanged(state) => {
                                    cx.global_mut::<Controller>().player_state = state.clone();

                                    if state.state == PlaybackState::Playing {
                                        playbar_view.update(cx, |this, cx| {
                                            this.player_page.update(cx, |this, cx| {
                                                this.controlbar.update(cx, |this, cx| {
                                                    this.playback_slider_state.update(
                                                        cx,
                                                        |this, cx| {
                                                            if let Some(meta) = cx
                                                                .global::<Controller>()
                                                                .player_state
                                                                .meta
                                                                .clone()
                                                            {
                                                                this.set_value(
                                                                    secs_to_slider(
                                                                        state.position,
                                                                        meta.duration,
                                                                    ),
                                                                    cx,
                                                                );
                                                            }
                                                            cx.notify();
                                                        },
                                                    );
                                                });
                                            })
                                        })
                                    }
                                    cx.global::<Controller>().write_app_state_cache();
                                    cx.notify();
                                }
                                AudioEvent::ScannerStateChanged(state) => {
                                    if cx.global::<Controller>().scanner_state.queue_order
                                        != state.queue_order
                                    {
                                        playbar_view.update(cx, |this, cx| {
                                            this.player_page.update(cx, |this, cx| {
                                                this.queue.update(cx, |this, cx| {
                                                    this.queue_order.update(cx, |this, _| {
                                                        *this = state.queue_order.clone()
                                                    });
                                                    this.views.update(cx, |v, _| v.clear());
                                                    let this = this.clone();
                                                    cx.defer(move |cx| this.scroll_to_item(cx));
                                                })
                                            })
                                        })
                                    }

                                    playbar_view.update(cx, |this, cx| {
                                        this.player_page.update(cx, |this, cx| {
                                            this.queue.update(cx, |this, _| {
                                                this.tracks = Arc::new(
                                                    state.current_playlist.clone().unwrap().tracks,
                                                )
                                            })
                                        })
                                    });

                                    cx.global_mut::<Controller>().scanner_state = state.clone();
                                    cx.global::<Controller>().write_app_state_cache();
                                }
                                AudioEvent::TrackLoaded(path) => {
                                    playbar_view.update(cx, |this, cx| {
                                        this.player_page.update(cx, |this, cx| {
                                            this.queue.update(cx, |this, cx| {
                                                let controller = cx.global::<Controller>();

                                                let idx = if let Some(playlist) =
                                                    &controller.scanner_state.current_playlist
                                                {
                                                    if let Some(real_index) = playlist
                                                        .tracks
                                                        .iter()
                                                        .position(|t| &t.path == path)
                                                    {
                                                        controller
                                                            .scanner_state
                                                            .queue_order
                                                            .iter()
                                                            .position(|&i| i == real_index)
                                                            .unwrap_or(0)
                                                    } else {
                                                        0
                                                    }
                                                } else {
                                                    0
                                                };

                                                if !this.stop_auto_scroll.read(cx) {
                                                    this.scroll_handle.scroll_to_item(
                                                        idx,
                                                        ScrollStrategy::Nearest,
                                                    );
                                                }
                                            });
                                        })
                                    });
                                    cx.global::<Controller>().write_app_state_cache();
                                    cx.notify();
                                }
                                AudioEvent::TrackEnded => {
                                    let controller = cx.global::<Controller>();
                                    let current = controller.player_state.current.clone();

                                    if controller.player_state.repeat {
                                        if current.is_some() {
                                            controller.load(
                                                current.unwrap().to_string_lossy().to_string(),
                                            )
                                        }
                                    } else {
                                        controller.next()
                                    }

                                    if controller.player_state.state != PlaybackState::Playing {
                                        controller.play();
                                    }
                                }
                            },
                            Event::Scanner(scanner_event) => match scanner_event {
                                ScannerEvent::State(state) => {
                                    cx.global_mut::<Controller>()
                                        .set_scanner_state_in_engine(state.clone());
                                    cx.global::<Controller>().write_app_state_cache();
                                }
                                ScannerEvent::Thumbnail { path, image } => {
                                    cx.global_mut::<ImageCache>()
                                        .add(path.clone(), image.clone());
                                }
                                ScannerEvent::ClearImageCache => {
                                    cx.global_mut::<ImageCache>().clear()
                                }
                                ScannerEvent::AppStateCache(app_state_cache) => {
                                    let scanner_cmd_tx = cx.global::<Controller>().scanner_cmd_tx.clone();
                                    cx.global::<Controller>()
                                        .send_app_state_cache(app_state_cache.clone(), scanner_cmd_tx);
                                    cx.notify();
                                }
                            },
                        })
                            .detach();

                        Root::new(view, window, cx)
                    })
                },
            )?;

            Ok::<_, anyhow::Error>(())
        })
            .detach();
    });
}

fn secs_to_slider(pos: u64, duration: u64) -> f32 {
    if duration == 0 {
        0.0
    } else {
        (pos as f32 / duration as f32) * 100.0
    }
}
