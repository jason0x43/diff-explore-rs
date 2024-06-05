use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Notify error: {0}")]
    NotifyError(#[from] notify::Error),

    #[error("IO error: {0}")]
    IoError(#[from] io::Error),
}
