//! `SaveReader`: the buffer, unmanaged and file readers as one reader over a byte slice.
//!
//! shortcut: files are read whole instead of memory mapped (`BinaryFileReader`); upgrade only if load memory
//! matters (a mapped slice can be passed to `SaveReader::new` unchanged).

use super::reader::GenericReader;
use crate::error::{IoError, Result, SeekOrigin};
use std::collections::HashMap;
use std::path::Path;

/// `SerializedTypes.db` contents: type name hash to full name.
pub type TypesDb = HashMap<u64, String>;

pub struct SaveReader<'a> {
    buffer: &'a [u8],
    position: usize,
    use_prefixes: bool,
    anchored_time_shift_ticks: i64,
    types_db: Option<&'a TypesDb>,
}

impl<'a> SaveReader<'a> {
    /// Strings carry a presence flag (the format all current saves use).
    pub fn new(buffer: &'a [u8]) -> Self {
        Self {
            buffer,
            position: 0,
            use_prefixes: true,
            anchored_time_shift_ticks: 0,
            types_db: None,
        }
    }

    /// `BinaryFileReader(path, usePrefixes: false)`: strings have no presence flag.
    pub fn without_prefixes(buffer: &'a [u8]) -> Self {
        Self {
            use_prefixes: false,
            ..Self::new(buffer)
        }
    }

    pub fn with_types_db(mut self, types_db: &'a TypesDb) -> Self {
        self.types_db = Some(types_db);
        self
    }

    pub fn with_anchored_time_shift_ticks(mut self, ticks: i64) -> Self {
        self.anchored_time_shift_ticks = ticks;
        self
    }

    pub fn set_anchored_time_shift_ticks(&mut self, ticks: i64) {
        self.anchored_time_shift_ticks = ticks;
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    pub fn remaining(&self) -> usize {
        self.buffer.len() - self.position
    }

    /// The full name recorded for a type hash in `SerializedTypes.db`, if one was loaded.
    pub fn type_name_for_hash(&self, hash: u64) -> Option<&'a str> {
        self.types_db?.get(&hash).map(String::as_str)
    }
}

impl GenericReader for SaveReader<'_> {
    fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
        if buf.len() > self.remaining() {
            return Err(IoError::EndOfStream);
        }
        buf.copy_from_slice(&self.buffer[self.position..self.position + buf.len()]);
        self.position += buf.len();
        Ok(())
    }

    fn position(&self) -> u64 {
        self.position as u64
    }

    fn seek(&mut self, offset: i64, origin: SeekOrigin) -> Result<u64> {
        let base = match origin {
            SeekOrigin::Begin => 0,
            SeekOrigin::Current => self.position as i64,
            SeekOrigin::End => self.buffer.len() as i64,
        };
        // a negative target clamps to 0; a target past the end can only fail on the next read, so reject it here
        let target = base.saturating_add(offset).max(0);
        if target > self.buffer.len() as i64 {
            return Err(IoError::SeekOutOfRange(target));
        }
        self.position = target as usize;
        Ok(self.position as u64)
    }

    fn read_string(&mut self) -> Result<Option<String>> {
        if !self.use_prefixes || self.read_bool()? {
            self.read_string_raw().map(Some)
        } else {
            Ok(None)
        }
    }

    fn anchored_time_shift_ticks(&self) -> i64 {
        self.anchored_time_shift_ticks
    }
}

/// Reads a whole save file; a missing file is `Ok(None)`.
pub fn read_file_if_exists(path: &Path) -> Result<Option<Vec<u8>>> {
    match std::fs::read(path) {
        Ok(b) => Ok(Some(b)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.into()),
    }
}

#[cfg(test)]
mod type_flag_tests {
    use super::*;

    #[test]
    fn unknown_type_flag_reads_as_null() {
        let mut r = SaveReader::new(&[7, 1]);
        assert!(r.read_type().unwrap().is_none());
        assert_eq!(r.read_u8().unwrap(), 1);
    }
}
