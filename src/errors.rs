use crossbeam_channel::RecvError;
use image::ImageError;
use lofty::error::LoftyError;
use rodio::source::SeekError;
use std::time::SystemTimeError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("anyhow Error occurred: `{0}`")]
    AnyHowError(#[from] anyhow::Error),
    #[error("Controller Error occurred: `{0}`")]
    ControllerError(#[from] ControllerError),
}

#[derive(Error, Debug)]
pub enum AudioError {
    #[error("Failed to load audio file: `{0}`")]
    LoadFile(String),
    #[error("Error occurred while seeking: `{0}`")]
    SeekError(#[from] SeekError),
}

#[derive(Error, Debug)]
pub enum ScannerError {
    #[error("Failed to load folder: `{0}`")]
    LoadFolder(String),
    #[error("I/O Error occurred: `{0}`")]
    IoError(#[from] std::io::Error),
    #[error("Image Error occurred: `{0}`")]
    ImageError(#[from] ImageError),
    #[error("Lofty Error occurred: `{0}`")]
    LoftyError(#[from] LoftyError),
    #[error("SystemTime Error occurred: `{0}`")]
    SystemTimeError(#[from] SystemTimeError),
    #[error("Recv Error occurred: `{0}`")]
    RecvError(#[from] RecvError),
}

#[derive(Error, Debug)]
pub enum ControllerError {
    #[error("Scanner Error occurred: `{0}`")]
    ScannerError(#[from] ScannerError),
}

#[derive(Error, Debug)]
pub enum CacherError {
    #[error("Recv Error occurred: `{0}`")]
    RecvError(#[from] RecvError),
    #[error("I/O Error occurred: `{0}`")]
    IoError(#[from] std::io::Error),
}