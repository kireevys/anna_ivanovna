use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("config file not found: {path}")]
    FileNotFound { path: PathBuf },

    #[error("failed to parse config {path}: {source}")]
    ParseError {
        path: PathBuf,
        source: serde_json::Error,
    },

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("home directory not found")]
    HomeDirNotFound,
}
