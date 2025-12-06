use thiserror::Error;

#[derive(Error, Debug)]
pub enum OcrError {
    #[error("Ort error: {0}")]
    Ort(#[from] ort::Error),
    #[error("Io error")]
    Io(#[from] std::io::Error),
    #[error("Image error")]
    ImageError(#[from] image::ImageError),
    #[error("Session not initialized")]
    SessionNotInitialized,
    #[error("Custom error: {0}")]
    Custom(String),
}
