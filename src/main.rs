#![warn(clippy::pedantic)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
pub mod app;
pub mod audio;
mod cacher;
pub mod controller;
pub mod errors;
pub mod image_processor;
pub mod library;
pub mod lyrics;
pub mod scanner;
pub mod system_integration;
pub mod ui;
mod worker_config;

use errors::AppError;

fn main() -> Result<(), AppError> {
    if cfg!(debug_assertions) {
        eprintln!("WARNING: running in debug mode — performance will be garbage");
    }

    app::run()
}
