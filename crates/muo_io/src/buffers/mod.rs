//! The byte cursors.
//!
//! Not provided: `STArrayPool`, `PooledArraySpanFormattable`, `ValueStringBuilder` (pooling; use `Vec`/`String`),
//! `RawInterpolatedStringHandler` (interpolated-string handler; use `format!` then `write_ascii`/`write_latin1`).

pub mod span_reader;
pub mod span_writer;

pub use span_reader::SpanReader;
pub use span_writer::SpanWriter;
