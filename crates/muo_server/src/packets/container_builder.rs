//! The 0xF7 container packet, a header (id, total length,
//! packet count) followed by whole packets. The pooled `GetSpan`/`Advance` pair is `push` of finished packets.

pub const MIN_PACKET_LENGTH: usize = 5;

pub struct PacketContainerBuilder {
    bytes: Vec<u8>,
    count: u16,
}

impl Default for PacketContainerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl PacketContainerBuilder {
    pub fn new() -> Self {
        // Packet ID, then length + count patched by `finish`.
        PacketContainerBuilder {
            bytes: vec![0xF7, 0, 0, 0, 0],
            count: 0,
        }
    }

    /// `GetSpan` + write + `Advance`: appends one packet.
    pub fn push(&mut self, packet: &[u8]) {
        self.bytes.extend_from_slice(packet);
        self.count = self.count.wrapping_add(1);
    }

    /// `Length`.
    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bytes.len() == MIN_PACKET_LENGTH
    }

    /// `Finalize()`: the bytes with the length (truncated to a ushort) and the count written.
    pub fn finish(mut self) -> Vec<u8> {
        let len = self.bytes.len() as u16;
        self.bytes[1..3].copy_from_slice(&len.to_be_bytes());
        self.bytes[3..5].copy_from_slice(&self.count.to_be_bytes());
        self.bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_length_and_count() {
        let mut c = PacketContainerBuilder::new();
        c.push(&[0x15, 1, 2, 3, 4, 5, 6, 7, 8]);
        c.push(&[0x54, 1]);
        assert_eq!(c.len(), 16);
        let b = c.finish();
        assert_eq!(b[..5], [0xF7, 0, 16, 0, 2]);
        assert_eq!(b.len(), 16);
    }

    #[test]
    fn empty_container() {
        let c = PacketContainerBuilder::new();
        assert!(c.is_empty());
        assert_eq!(c.finish(), [0xF7, 0, 5, 0, 0]);
    }
}
