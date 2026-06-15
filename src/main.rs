#![warn(clippy::pedantic)]
#![allow(
    clippy::unreadable_literal,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::missing_panics_doc,
    clippy::missing_errors_doc,
    clippy::type_complexity,
    clippy::too_many_lines,
    clippy::new_without_default
)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod app;
pub mod audio;
pub mod cacher;
pub mod controller;
pub mod errors;
pub mod image_processor;
pub mod logging;
pub mod lyrics_manager;
pub mod scanner;
pub mod system_integration;
pub mod ui;

use app::{ensure_app_paths, get_app_paths};
use errors::AppError;
use tracing::info;

fn main() -> Result<(), AppError> {
    let app_paths = get_app_paths();
    ensure_app_paths(&app_paths);

    let _log_guard = logging::init(&app_paths).expect("Failed to initialize logging");

    if cfg!(debug_assertions) {
        tracing::warn!("Running in debug mode, performance will be garbage");
    }

    info!("Starting Wiremann...");

    app::run(app_paths)
}
