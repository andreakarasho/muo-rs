//! Byte cursors, text encodings, compression and save reader/writer traits and implementations.

pub mod buffers;
pub mod compression;
pub mod error;
pub mod serialization;
pub mod text;

pub use buffers::{SpanReader, SpanWriter};
pub use error::{IoError, Result, SeekOrigin};
pub use serialization::{
    GenericReader, GenericSerializable, GenericWriter, SaveReader, SaveWorker, SaveWriter,
};
pub use text::{TextDefinition, TextEncoding};
