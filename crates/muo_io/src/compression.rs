//! `Deflate`: zlib-wrapped deflate into and out of caller buffers, with libdeflate's contract.
//!
//! The compressor is flate2's pure-Rust backend. Its output is a valid zlib stream any inflater (the UO client's)
//! reads, but it is not byte-identical to libdeflate's. Nothing persisted or compared depends on those bytes.

use flate2::{Compress, Compression, Decompress, FlushCompress, FlushDecompress, Status};

/// `LibDeflateCompressionLevel`: `Standard` is libdeflate's default (6), `Maximum` its best ratio.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Level {
    Standard,
    Maximum,
}

pub const STANDARD: Level = Level::Standard;
pub const MAXIMUM: Level = Level::Maximum;

/// `LibDeflateResult` without `Success`.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum UnpackError {
    BadData,
    ShortOutput,
    InsufficientSpace,
}

impl Level {
    fn compression(self) -> Compression {
        match self {
            Self::Standard => Compression::new(6),
            Self::Maximum => Compression::best(),
        }
    }

    /// `Pack`: compresses `src` into `dest` as a zlib stream and returns the length, or 0 when `dest` is too small
    /// (libdeflate's convention).
    pub fn pack(self, dest: &mut [u8], src: &[u8]) -> usize {
        let mut c = Compress::new(self.compression(), true);
        match c.compress(src, dest, FlushCompress::Finish) {
            Ok(Status::StreamEnd) => c.total_out() as usize,
            _ => 0,
        }
    }

    /// `Unpack`: inflates a zlib stream into `dest`, returning the number of bytes produced.
    pub fn unpack(self, dest: &mut [u8], src: &[u8]) -> Result<usize, UnpackError> {
        let mut d = Decompress::new(true);
        match d.decompress(src, dest, FlushDecompress::Finish) {
            Ok(Status::StreamEnd) => Ok(d.total_out() as usize),
            // output ran out before the stream ended
            Ok(_) if d.total_out() as usize == dest.len() => Err(UnpackError::InsufficientSpace),
            Ok(_) => Err(UnpackError::ShortOutput),
            Err(_) => Err(UnpackError::BadData),
        }
    }

    /// Upper bound of `pack` output for `len` input bytes (zlib's `compressBound`).
    pub fn compress_bound(len: usize) -> usize {
        len + (len >> 12) + (len >> 14) + (len >> 25) + 13 + 6
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let src: Vec<u8> = (0..5000u32).map(|i| (i % 7) as u8).collect();
        for level in [STANDARD, MAXIMUM] {
            let mut packed = vec![0u8; Level::compress_bound(src.len())];
            let n = level.pack(&mut packed, &src);
            assert!(n > 0 && n < src.len());
            assert_eq!(packed[0], 0x78, "zlib header");

            let mut out = vec![0u8; src.len()];
            assert_eq!(STANDARD.unpack(&mut out, &packed[..n]), Ok(src.len()));
            assert_eq!(out, src);
        }
    }

    #[test]
    fn known_zlib_stream_unpacks() {
        // zlib.compress(b"hello hello hello")
        let packed = [
            0x78, 0x9C, 0xCB, 0x48, 0xCD, 0xC9, 0xC9, 0x57, 0xC8, 0x40, 0x90, 0x00, 0x3A, 0x2E,
            0x06, 0x7D,
        ];
        let mut out = [0u8; 32];
        let n = STANDARD.unpack(&mut out, &packed).unwrap();
        assert_eq!(&out[..n], b"hello hello hello");
    }

    #[test]
    fn errors() {
        let mut out = [0u8; 4];
        assert_eq!(
            STANDARD.unpack(&mut out, &[1, 2, 3, 4, 5]),
            Err(UnpackError::BadData)
        );
        let mut small = [0u8; 2];
        assert_eq!(STANDARD.pack(&mut small, &[7u8; 500]), 0);

        let mut packed = vec![0u8; 100];
        let n = STANDARD.pack(&mut packed, &[9u8; 50]);
        let mut tiny = [0u8; 10];
        assert_eq!(
            STANDARD.unpack(&mut tiny, &packed[..n]),
            Err(UnpackError::InsufficientSpace)
        );
        let mut big = [0u8; 100];
        assert_eq!(
            STANDARD.unpack(&mut big, &packed[..n - 3]),
            Err(UnpackError::ShortOutput)
        );
    }
}
