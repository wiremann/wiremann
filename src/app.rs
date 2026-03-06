use crate::cacher::Cacher;
use crate::worker_config::{calculate_worker_config, WorkerConfig};
use crate::{
    audio::Audio,
    controller::{state::AppState, Controller},
    errors::AppError,
    scanner::Scanner,
    ui::{
        assets::Assets,
        res_handler::{Event, ResHandler},
        wiremann::Wiremann,
    },
};
use gpui::{
    px, size, AppContext, Bounds, Result, TitlebarOptions, WindowBounds, WindowOptions,
};
use gpui_platform::application;
use std::{
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

#[allow(
    clippy::too_many_lines,
    clippy::missing_panics_doc,
    clippy::missing_errors_doc
)]
pub fn run() -> Result<(), AppError> {
    let assets = Assets {};

    application().with_assets(assets.clone()).run(move |cx| {
        let bounds = Bounds::centered(None, size(px(1280.0), px(760.0)), cx);
        assets.load_fonts(cx).expect("Could not load fonts");

        let WorkerConfig {
            metadata,
            thumbnail,
            cacher: cacher_workers,
        } = calculate_worker_config();

        let (mut audio, audio_tx, audio_rx) = Audio::new();
        let (mut scanner, scanner_tx, scanner_rx) = Scanner::new();
        let (cacher, cacher_tx, cacher_rx) = Cacher::new();

        let controller = Controller::new(
            cx.new(|_| AppState::default()),
            audio_tx,
            audio_rx,
            scanner_tx,
            scanner_rx,
            cacher_tx,
            cacher_rx,
        );

        thread::spawn(move || {
            if let Err(e) = audio.run() {
                eprintln!("Audio thread crashed with error: {e:?}");
            }
        });

        thread::spawn(move || {
            if let Err(e) = scanner.run(metadata, thumbnail) {
                eprintln!("Scanner thread crashed with error: {e:?}");
            }
        });

        thread::spawn(move || {
            if let Err(e) = cacher.run(cacher_workers) {
                eprintln!("Cacher thread crashed with error: {e:?}");
            }
        });

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
            |_, cx| {
                cx.set_global(controller.clone());

                let view = cx.new(Wiremann::new);

                let res_handler = cx.new(|_| ResHandler {});
                let arc_res = Arc::new(res_handler.clone());
                let mut controller_clone = controller.clone();

                cx.spawn(async move |cx| {
                    let mut last_pos_request = Instant::now();
                    let mut last_track_ended_request = Instant::now();

                    loop {
                        while let Ok(e) = controller.audio_rx.try_recv() {
                            arc_res.update(cx, |res_handler, cx| {
                                res_handler.handle(cx, Event::Audio(e));
                            });
                        }

                        while let Ok(e) = controller.scanner_rx.try_recv() {
                            arc_res.update(cx, |res_handler, cx| {
                                res_handler.handle(cx, Event::Scanner(e));
                            });
                        }

                        while let Ok(e) = controller.cacher_rx.try_recv() {
                            arc_res.update(cx, |res_handler, cx| {
                                res_handler.handle(cx, Event::Cacher(e));
                            });
                        }

                        if last_pos_request.elapsed() >= Duration::from_millis(256) {
                            controller.get_pos();

                            last_pos_request = Instant::now();
                        }

                        if last_track_ended_request.elapsed() >= Duration::from_millis(512) {
                            controller.check_track_ended();

                            last_track_ended_request = Instant::now();
                        }

                        cx.background_executor()
                            .timer(Duration::from_millis(16))
                            .await;
                    }
                })
                    .detach();

                let view_clone = view.clone();

                cx.subscribe(&res_handler, move |_, event, cx| {
                    if let Err(e) = match event {
                        Event::Audio(event) => {
                            controller_clone.handle_audio_event(cx, event, &view_clone)
                        }

                        Event::Scanner(event) => {
                            controller_clone.handle_scanner_event(cx, event, &view_clone)
                        }

                        Event::Cacher(event) => {
                            controller_clone.handle_cacher_event(cx, event, &view_clone)
                        }
                    } {
                        eprintln!("controller error: {e:?}");
                    }
                })
                    .detach();

                view
            },
        ).expect("Application panicked.");

        cx.activate(true);
    });

    Ok(())
}
