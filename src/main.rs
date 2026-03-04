#![warn(clippy::pedantic)]

pub mod app;
pub mod audio;
mod cacher;
pub mod controller;
pub mod errors;
pub mod library;
mod queue;
pub mod scanner;
pub mod ui;
mod worker_config;

use errors::AppError;

fn main() -> Result<(), AppError> {
    app::run()
}
