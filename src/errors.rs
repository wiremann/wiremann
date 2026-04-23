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
    #[error("I/O Error occurred: `{0}`")]
    IoError(#[from] std::io::Error),
    #[error("Rodio Decoder Error occurred: `{0}`")]
    RodioDecoderError(#[from] rodio::decoder::DecoderError),
    #[error("Recv Error occurred: `{0}`")]
    RecvError(#[from] RecvError),
}

#[derive(Error, Debug)]
pub enum ScannerError {
    #[error("Failed to load folder: `{0}`")]
    LoadFolder(String),
    #[error("I/O Error occurred: `{0}`")]
    IoError(#[from] std::io::Error),
    #[error("Lofty Error occurred: `{0}`")]
    LoftyError(#[from] LoftyError),
    #[error("SystemTime Error occurred: `{0}`")]
    SystemTimeError(#[from] SystemTimeError),
    #[error("Recv Error occurred: `{0}`")]
    RecvError(#[from] RecvError),
}

#[derive(Error, Debug)]
pub enum ImageProcessorError {
    #[error("Image Error occurred: `{0}`")]
    ImageError(#[from] ImageError),
    #[error("Garb Resize Error occurred: `{0}`")]
    GarbSizeError(#[from] garb::SizeError),
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
    #[error("Bitcode Error occurred: `{0}`")]
    BitcodeError(#[from] bitcode::Error),
    #[error("RON Error occurred: `{0}`")]
    RonError(#[from] ron::Error),
    #[error("RON Spanned Error occurred: `{0}`")]
    RonSpannedError(#[from] ron::de::SpannedError),
}

#[derive(Error, Debug)]
pub enum SystemIntegrationError {
    #[error("Souvlaki error occurred: `{0}`")]
    SouvlakiError(#[from] souvlaki::Error),
    #[error("Garb Resize Error occurred: `{0}`")]
    GarbSizeError(#[from] garb::SizeError),
    #[error("I/O Error occurred: `{0}`")]
    IoError(#[from] std::io::Error),
    #[error("Image Error occurred: `{0}`")]
    ImageError(#[from] ImageError),
    #[error("SystemTime Error occurred: `{0}`")]
    SystemTimeError(#[from] SystemTimeError),
}

#[derive(Error, Debug)]
pub enum LyricsError {}
