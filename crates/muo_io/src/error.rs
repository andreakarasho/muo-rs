//! Errors for the byte cursors, save reader/writer and persistence helpers: end of stream, out of range
//! arguments, invalid operations and other failures.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum IoError {
    #[error("Cannot read past the end of the buffer.")]
    EndOfStream,
    #[error("Buffer is full and resizing is disabled.")]
    BufferFull,
    #[error("Seek to position {0} is out of range.")]
    SeekOutOfRange(i64),
    #[error("invalid data: {0}")]
    InvalidData(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, IoError>;

/// `System.IO.SeekOrigin`.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SeekOrigin {
    Begin,
    Current,
    End,
}
