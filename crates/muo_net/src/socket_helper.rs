//! Socket address helpers.
//!
//! Sockets are mio streams, so the two lookups are `peer_addr`/`local_addr`; the `sockaddr` parsing is kept
//! for callers that have raw bytes (and documents the layouts).

use mio::net::TcpStream;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

// Address family values found in a raw `sockaddr`.
const ADDRESS_FAMILY_INTER_NETWORK: i32 = 2;
const ADDRESS_FAMILY_INTER_NETWORK_V6: i32 = 23;

/// The remote IP address of a connected stream, or `None` if it cannot be read.
pub fn get_remote_address(stream: &TcpStream) -> Option<IpAddr> {
    stream.peer_addr().ok().map(|a| a.ip())
}

/// The local endpoint a client connected to, or `None` if it cannot be read.
pub fn get_local_end_point(stream: &TcpStream) -> Option<SocketAddr> {
    stream.local_addr().ok()
}

fn family(buffer: &[u8], bsd: bool) -> Option<i32> {
    if buffer.len() < 2 {
        return None;
    }
    // BSD/macOS: sin_len (1 byte) + sin_family (1 byte). Linux: sa_family (2 bytes).
    Some(if bsd {
        buffer[1] as i32
    } else {
        i16::from_le_bytes([buffer[0], buffer[1]]) as i32
    })
}

/// `ParseSockAddr`: the address inside a `sockaddr_in`/`sockaddr_in6` image.
pub fn parse_sock_addr(buffer: &[u8], bsd: bool) -> Option<IpAddr> {
    let fam = family(buffer, bsd)?;

    if fam == ADDRESS_FAMILY_INTER_NETWORK && buffer.len() >= 8 {
        // IPv4: family (2) + port (2) + addr (4)
        let b: [u8; 4] = buffer[4..8].try_into().ok()?;
        return Some(IpAddr::V4(Ipv4Addr::from(b)));
    }

    if fam == ADDRESS_FAMILY_INTER_NETWORK_V6 && buffer.len() >= 28 {
        // IPv6: family (2) + port (2) + flowinfo (4) + addr (16) + scope (4)
        let b: [u8; 16] = buffer[8..24].try_into().ok()?;
        return Some(IpAddr::V6(Ipv6Addr::from(b)));
    }

    None
}

/// `ParseSockAddrEndPoint`: address and port (network byte order at offset 2).
pub fn parse_sock_addr_end_point(buffer: &[u8], bsd: bool) -> Option<SocketAddr> {
    if buffer.len() < 4 {
        return None;
    }
    let port = ((buffer[2] as u16) << 8) | buffer[3] as u16;
    parse_sock_addr(buffer, bsd).map(|ip| SocketAddr::new(ip, port))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_v4_on_both_layouts() {
        let linux = [2, 0, 0x0A, 0x21, 127, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0];
        let bsd = [16, 2, 0x0A, 0x21, 127, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0];
        assert_eq!(
            parse_sock_addr(&linux, false),
            Some("127.0.0.1".parse().unwrap())
        );
        assert_eq!(
            parse_sock_addr(&bsd, true),
            Some("127.0.0.1".parse().unwrap())
        );
        assert_eq!(
            parse_sock_addr_end_point(&linux, false),
            Some("127.0.0.1:2593".parse().unwrap())
        );
    }

    #[test]
    fn parses_v6_and_rejects_short_or_unknown() {
        let mut v6 = [0u8; 28];
        v6[0] = 23;
        v6[3] = 80;
        v6[23] = 1; // ::1
        assert_eq!(
            parse_sock_addr_end_point(&v6, false),
            Some("[::1]:80".parse().unwrap())
        );
        assert_eq!(parse_sock_addr(&v6[..27], false), None);
        assert_eq!(parse_sock_addr(&[9, 0, 0, 0, 1, 2, 3, 4], false), None);
        assert_eq!(parse_sock_addr(&[], false), None);
        assert_eq!(parse_sock_addr_end_point(&[2, 0, 0], false), None);
    }
}
