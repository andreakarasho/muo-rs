//! Errors for reading client files and data tables. Corrupt or missing input is an error, never a panic.

use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DataError {
    #[error("{path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("Error loading file {file}. {reason}")]
    InvalidData { file: String, reason: String },
    #[error("Data: {0} was not found")]
    NotFound(String),
    #[error("{path}: {source}")]
    Json {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error(transparent)]
    Read(#[from] muo_io::IoError),
    #[error(transparent)]
    Std(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, DataError>;

impl DataError {
    pub fn invalid(file: impl Into<String>, reason: impl Into<String>) -> Self {
        DataError::InvalidData {
            file: file.into(),
            reason: reason.into(),
        }
    }

    pub(crate) fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        DataError::Io {
            path: path.into(),
            source,
        }
    }
}
