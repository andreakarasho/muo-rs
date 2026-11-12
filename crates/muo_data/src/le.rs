//! Little-endian reads over `std::io::Read`, in the manner of a `BinaryReader`.

use std::io::Read;

macro_rules! le_read {
    ($name:ident, $ty:ty) => {
        pub fn $name<R: Read + ?Sized>(r: &mut R) -> std::io::Result<$ty> {
            let mut b = [0u8; std::mem::size_of::<$ty>()];
            r.read_exact(&mut b)?;
            Ok(<$ty>::from_le_bytes(b))
        }
    };
}

le_read!(i16, i16);
le_read!(u16, u16);
le_read!(i32, i32);
le_read!(u32, u32);
le_read!(i64, i64);
le_read!(u64, u64);

/// Reads up to `buf.len()` bytes, stopping at end of input; returns the count (`Stream.Read` in a loop).
pub fn read_fully<R: Read + ?Sized>(r: &mut R, buf: &mut [u8]) -> std::io::Result<usize> {
    let mut n = 0;
    while n < buf.len() {
        match r.read(&mut buf[n..]) {
            Ok(0) => break,
            Ok(k) => n += k,
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {}
            Err(e) => return Err(e),
        }
    }
    Ok(n)
}
