mod events;
mod paths;
mod window;
mod workers;

use crate::cacher::Cacher;
use crate::image_processor::ImageProcessor;
use crate::lyrics_manager::LyricsManager;
use crate::system_integration::SystemIntegration;
use crate::{
    audio::Audio,
    controller::{Controller, state::AppState},
    errors::AppError,
    scanner::Scanner,
    ui::{assets::Assets, res_handler::ResHandler, wiremann::Wiremann},
};
pub use paths::*;

use gpui::{AppContext, Application, Result};
use raw_window_handle::HasWindowHandle;
use std::sync::Arc;
use tracing::info;

use events::{spawn_event_loop, subscribe_controller_events};
use window::build_window_options;
use workers::{WorkerConfig, calculate_worker_config, spawn_worker};

static ICON_PNG: &[u8] = include_bytes!("../../assets/logos/logo.png");

pub fn run(app_paths: AppPaths) -> Result<(), AppError> {
    let assets = Assets {};

    Application::new()
        .with_assets(assets.clone())
        .run(move |cx| {
            assets.load_fonts(cx).expect("Could not load fonts");

            let WorkerConfig {
                metadata: metadata_workers,
                thumbnail: thumbnail_workers,
                cacher: cacher_workers,
            } = calculate_worker_config();

            let app_icon = gpui::WindowIcon::from_png_bytes(ICON_PNG).ok();
            let window_options = build_window_options(app_icon, cx);

            info!("Spawning application window...");

            cx.open_window(window_options, |window, cx| {
                info!("Initializing engines...");

                let (mut audio, audio_tx, audio_rx) = Audio::new();

                let (mut scanner, scanner_tx, scanner_rx) = Scanner::new(app_paths.clone());

                let (cacher, cacher_tx, cacher_rx) = Cacher::new(app_paths.clone());

                let (mut image_processor, image_processor_tx, image_processor_rx) =
                    ImageProcessor::new(app_paths.clone());

                let raw_window_handle = window.window_handle().ok().map(|this| this.as_raw());

                let (mut system_integration, system_integration_tx, system_integration_rx) =
                    SystemIntegration::new(raw_window_handle, app_paths);

                let (mut lyrics_manager, lyrics_manager_tx, lyrics_manager_rx) =
                    LyricsManager::new();

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
                    system_integration_tx,
                    system_integration_rx,
                    lyrics_manager_tx,
                    lyrics_manager_rx,
                );

                spawn_worker("audio", move || audio.run());

                spawn_worker("scanner", move || scanner.run(metadata_workers));

                spawn_worker("cacher", move || cacher.run(cacher_workers));

                spawn_worker("image processor", move || {
                    image_processor.run(thumbnail_workers)
                });

                spawn_worker("system integration", move || system_integration.run());

                spawn_worker("lyrics manager", move || lyrics_manager.run());

                cx.set_global(controller.clone());

                let view = cx.new(Wiremann::new);

                let res_handler = cx.new(|_| ResHandler {});
                let arc_res = Arc::new(res_handler.clone());

                info!("Spawning event loop...");
                spawn_event_loop(cx, controller.clone(), arc_res.clone());

                subscribe_controller_events(cx, &res_handler, controller.clone(), view.clone());

                view
            })
            .expect("Application panicked.");

            cx.activate(true);
        });

    Ok(())
}
