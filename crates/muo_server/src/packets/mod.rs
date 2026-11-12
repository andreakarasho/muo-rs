//! Outgoing packets.
//!
//! Convention. A packet has a builder (`CreateX`) and a sender (`ns.SendX(..)`: check, build, `ns.Send`). Here:
//! - `create_x(..) -> Vec<u8>` builds the bytes; pure, takes values, or `&World` plus a `&NetCaps` when the
//!   bytes depend on entities or on the client's era (`NetState.StygianAbyss` etc.).
//! - `send_x(w: &mut World, ns: NetStateId, ..)` is `SendX`: `if w.cannot_send_packets(ns) { return }`, build,
//!   `w.send(ns, &bytes)`. The caps come from `w.net_state(ns)`.
//!
//! A packet sent to many clients is built once by the caller and sent with `w.send` per client.

use std::ops::{Deref, DerefMut};

use muo_io::SpanWriter;
use muo_primitives::Serial;

pub mod account;
pub mod combat;
pub mod container;
pub mod container_builder;
pub mod damage;
pub mod effect;
pub mod entity;
pub mod equipment;
pub mod item;
pub mod light;
pub mod map;
pub mod menu;
pub mod message;
pub mod mobile;
pub mod movement;
pub mod player;
pub mod secure_trade;
pub mod target;
pub mod vendor_buy;
pub mod vendor_sell;

/// A growable `SpanWriter` whose writes cannot fail (it resizes). Derefs to `SpanWriter` for the string writers
/// (`pk.write_big_uni_null(s).ok()`); the helpers below are the infallible ones.
pub struct Pk(SpanWriter);

impl Pk {
    pub fn new(capacity: usize) -> Pk {
        Pk(SpanWriter::new(capacity, true))
    }

    pub fn u8(&mut self, v: u8) -> &mut Self {
        let _ = self.0.write_u8(v);
        self
    }
    pub fn i8(&mut self, v: i8) -> &mut Self {
        let _ = self.0.write_i8(v);
        self
    }
    pub fn bool(&mut self, v: bool) -> &mut Self {
        let _ = self.0.write_bool(v);
        self
    }
    pub fn u16(&mut self, v: u16) -> &mut Self {
        let _ = self.0.write_u16(v);
        self
    }
    pub fn i16(&mut self, v: i16) -> &mut Self {
        let _ = self.0.write_i16(v);
        self
    }
    pub fn u32(&mut self, v: u32) -> &mut Self {
        let _ = self.0.write_u32(v);
        self
    }
    pub fn i32(&mut self, v: i32) -> &mut Self {
        let _ = self.0.write_i32(v);
        self
    }
    pub fn serial(&mut self, v: Serial) -> &mut Self {
        let _ = self.0.write_serial(v);
        self
    }
    pub fn bytes(&mut self, v: &[u8]) -> &mut Self {
        let _ = self.0.write_bytes(v);
        self
    }
    /// `writer.Clear(n)`: n zero bytes.
    pub fn zeros(&mut self, n: usize) -> &mut Self {
        let _ = self.0.clear(n);
        self
    }

    /// `writer.WritePacketLength()`: the big-endian total at bytes 1..3, then the bytes.
    pub fn finish_with_length(mut self) -> Vec<u8> {
        let written = self.0.bytes_written();
        muo_net::packet_handler::write_packet_length(self.0.raw_buffer_mut(), written);
        self.0.into_vec()
    }

    /// `writer.Span`.
    pub fn finish(self) -> Vec<u8> {
        self.0.into_vec()
    }
}

impl Deref for Pk {
    type Target = SpanWriter;
    fn deref(&self) -> &SpanWriter {
        &self.0
    }
}

impl DerefMut for Pk {
    fn deref_mut(&mut self) -> &mut SpanWriter {
        &mut self.0
    }
}
