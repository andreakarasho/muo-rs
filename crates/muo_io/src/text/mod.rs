//! Text helpers and encodings.
//!
//! Not provided: `OrdinalStringHelpers` / `InsensitiveStringHelpers` (thin wrappers over `str` comparisons; use std),
//! `ISelfInterpolatedStringHandler` (interpolated-string handler; use `format!`).

pub mod definition;
pub mod encoding;
pub mod helpers;
pub mod hex;

pub use definition::TextDefinition;
pub use encoding::TextEncoding;
