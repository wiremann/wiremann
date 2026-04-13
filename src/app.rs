use crate::cacher::Cacher;
use crate::image_processor::ImageProcessor;
use crate::worker_config::{WorkerConfig, calculate_worker_config};
use crate::{
    audio::Audio,
    controller::{Controller, state::AppState},
    errors::AppError,
    scanner::Scanner,
    ui::{
        assets::Assets,
        res_handler::{Event, ResHandler},
        wiremann::Wiremann,
    },
};
use gpui::{AppContext, Bounds, Result, TitlebarOptions, WindowBounds, WindowOptions, px, size};
use gpui_platform_gpui_unofficial::application;
use std::{
    fs,
    path::PathBuf,
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

#[derive(Clone)]
pub struct AppPaths {
    pub cache: PathBuf,
    pub config: PathBuf,
    pub data: PathBuf,
}

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
            metadata: metadata_workers,
            thumbnail: thumbnail_workers,
            cacher: cacher_workers,
        } = calculate_worker_config();

        let app_paths = get_app_paths();
        ensure_app_paths(&app_paths);

        let (mut audio, audio_tx, audio_rx) = Audio::new();
        let (mut scanner, scanner_tx, scanner_rx) = Scanner::new(app_paths.clone());
        let (cacher, cacher_tx, cacher_rx) = Cacher::new(app_paths.clone());
        let (mut image_processor, image_processor_tx, image_processor_rx) =
            ImageProcessor::new(app_paths);

        let controller = Controller::new(
            cx.new(|_| AppState::default()),
            audio_tx,
            audio_rx,
            scanner_tx,
            scanner_rx,
            cacher_tx,
            cacher_rx,
            image_processor_tx,
            image_processor_rx,
        );

        thread::spawn(move || {
            if let Err(e) = audio.run() {
                eprintln!("Audio thread crashed with error: {e:?}");
            }
        });

        thread::spawn(move || {
            if let Err(e) = scanner.run(metadata_workers) {
                eprintln!("Scanner thread crashed with error: {e:?}");
            }
        });

        thread::spawn(move || {
            if let Err(e) = cacher.run(cacher_workers) {
                eprintln!("Cacher thread crashed with error: {e:?}");
            }
        });

        thread::spawn(move || {
            if let Err(e) = image_processor.run(thumbnail_workers) {
                eprintln!("Image processor thread crashed with error: {e:?}");
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

                        while let Ok(e) = controller.image_processor_rx.try_recv() {
                            arc_res.update(cx, |res_handler, cx| {
                                res_handler.handle(cx, Event::ImageProcessor(e));
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
                            .timer(Duration::from_millis(64))
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
                        Event::ImageProcessor(event) => {
                            controller_clone.handle_image_processor_event(cx, event, &view_clone)
                        }
                    } {
                        eprintln!("controller error: {e:?}");
                    }
                })
                .detach();

                view
            },
        )
        .expect("Application panicked.");

        cx.activate(true);
    });

    Ok(())
}

fn get_app_paths() -> AppPaths {
    let project_dir = directories::ProjectDirs::from("app", "wiremann", "wiremann")
        .expect("Couldn't get application paths");

    let cache = project_dir.cache_dir().to_path_buf();
    let config = project_dir.config_dir().to_path_buf();
    let data = project_dir.data_dir().to_path_buf();

    AppPaths {
        cache,
        config,
        data,
    }
}

fn ensure_app_paths(app_paths: &AppPaths) {
    fs::create_dir_all(app_paths.cache.as_path()).expect("failed to create cache directory");
    fs::create_dir_all(app_paths.config.as_path()).expect("failed to create cache directory");
    fs::create_dir_all(app_paths.data.as_path()).expect("failed to create cache directory");
}
