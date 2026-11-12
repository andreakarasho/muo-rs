//! Detection of non-UO protocols (HTTP and the like) spoken on the game port.
//!
//! Identifies traffic that is positively some OTHER protocol (HTTP, TLS, SSH), rather than deciding whether
//! a connection is a good Ultima Online client. A client with encryption enabled when the shard expects
//! none sends noise of the right length, so "unreadable" cannot mean "hostile"; "speaks HTTP" safely can.

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ForeignProtocolKind {
    None,
    Http,
    Tls,
    Ssh,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ForeignProtocolMatch {
    None,
    /// A prefix matched, but more bytes are needed to be sure.
    Incomplete,
    Confirmed,
}

const REQUIRED_BYTES: usize = 8;
const MAX_TLS_RECORD_LENGTH: usize = 16384;

pub fn identify(buffer: &[u8]) -> (ForeignProtocolMatch, ForeignProtocolKind) {
    // Too little to match a prefix; the caller's own short-read handling covers it.
    if buffer.len() < 4 {
        return (ForeignProtocolMatch::None, ForeignProtocolKind::None);
    }

    let candidate = match_prefix(buffer);
    if candidate == ForeignProtocolKind::None {
        return (ForeignProtocolMatch::None, ForeignProtocolKind::None);
    }

    if buffer.len() < REQUIRED_BYTES {
        return (ForeignProtocolMatch::Incomplete, ForeignProtocolKind::None);
    }

    if !confirm(buffer, candidate) {
        return (ForeignProtocolMatch::None, ForeignProtocolKind::None);
    }

    (ForeignProtocolMatch::Confirmed, candidate)
}

fn match_prefix(buffer: &[u8]) -> ForeignProtocolKind {
    // TLS handshake record: content type 0x16, major version 3, minor version 0..4 (SSL 3.0 - TLS 1.3).
    if buffer[0] == 0x16 && buffer[1] == 0x03 && buffer[2] <= 0x04 {
        return ForeignProtocolKind::Tls;
    }

    if buffer.starts_with(b"SSH-") {
        return ForeignProtocolKind::Ssh;
    }

    // HTTP request methods. Four bytes only selects a candidate; `confirm` checks the request line.
    const METHODS: [&[u8; 4]; 9] = [
        b"GET ", b"POST", b"HEAD", b"PUT ", b"OPTI", b"DELE", b"CONN", b"TRAC", b"PATC",
    ];
    if METHODS.iter().any(|m| buffer.starts_with(*m)) {
        return ForeignProtocolKind::Http;
    }

    ForeignProtocolKind::None
}

fn confirm(buffer: &[u8], candidate: ForeignProtocolKind) -> bool {
    if candidate == ForeignProtocolKind::Tls {
        // A seed can collide with the 0x16 0x03 0x0? prefix (that is just the address 22.3.x.x), so
        // require a ClientHello inside a plausible record.
        let record_length = ((buffer[3] as usize) << 8) | buffer[4] as usize;
        return buffer[5] == 0x01 && (4..=MAX_TLS_RECORD_LENGTH).contains(&record_length);
    }

    // A UO client's fifth byte is a packet id (0x80, 0x91, 0xEF), none of them printable, so requiring
    // the request line to continue in ASCII lets a seed that spells "GET " fall through.
    buffer
        .iter()
        .take(16)
        .skip(4)
        .all(|&b| is_printable_ascii(b))
}

fn is_printable_ascii(b: u8) -> bool {
    matches!(b, 0x20..=0x7E | b'\r' | b'\n' | b'\t')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn http_requests_are_identified() {
        for req in [
            "GET / HTTP/1.1\r\n",
            "POST /a HTTP/1.1\r\n",
            "HEAD / HTTP/1.0\r\n",
            "OPTIONS * HTTP/1.1\r\n",
            "CONNECT host:443 HTTP/1.1\r\n",
            "DELETE /x HTTP/1.1\r\n",
        ] {
            assert_eq!(
                identify(req.as_bytes()),
                (ForeignProtocolMatch::Confirmed, ForeignProtocolKind::Http)
            );
        }
    }

    #[test]
    fn ssh_banner_is_identified() {
        assert_eq!(
            identify(b"SSH-2.0-OpenSSH_9.6"),
            (ForeignProtocolMatch::Confirmed, ForeignProtocolKind::Ssh)
        );
    }

    #[test]
    fn tls_client_hello_is_identified() {
        let hello = [0x16, 0x03, 0x03, 0x01, 0x00, 0x01, 0x00, 0x00, 0xFC];
        assert_eq!(
            identify(&hello),
            (ForeignProtocolMatch::Confirmed, ForeignProtocolKind::Tls)
        );
    }

    #[test]
    fn tls_prefix_without_a_client_hello_is_not_foreign() {
        // A seed can spell 0x16 0x03 0x0? -- the address 22.3.x.x. Byte five is a packet id.
        let seed_then_login = [0x16, 0x03, 0x03, 0x04, 0x00, 0x80, 0x00, 0x00];
        assert_eq!(
            identify(&seed_then_login),
            (ForeignProtocolMatch::None, ForeignProtocolKind::None)
        );
    }

    #[test]
    fn seed_that_spells_an_http_method_falls_through() {
        // Seed 0x47455420 spells "GET ". The next byte is the 0x80 login packet id, not printable.
        let seed_then_login = [b'G', b'E', b'T', b' ', 0x80, 0x00, 0x3A, 0x00];
        assert_eq!(identify(&seed_then_login).0, ForeignProtocolMatch::None);
    }

    #[test]
    fn ordinary_uo_openings_are_not_foreign() {
        for second in [0x80, 0x91, 0xEF] {
            let buffer = [0x7F, 0x00, 0x00, 0x01, second, 0x00, 0x00, 0x00];
            assert_eq!(identify(&buffer).0, ForeignProtocolMatch::None);
        }
    }

    #[test]
    fn encrypted_login_is_not_mistaken_for_a_foreign_protocol() {
        let buffer = [0x7F, 0x00, 0x00, 0x01, 0xC3, 0x9A, 0x04, 0xE1, 0x55, 0xB2];
        assert_eq!(identify(&buffer).0, ForeignProtocolMatch::None);
    }

    #[test]
    fn prefix_match_without_enough_bytes_waits() {
        // Framing is never assumed: "GET " split from its request line must wait.
        assert_eq!(identify(b"GET ").0, ForeignProtocolMatch::Incomplete);
        assert_eq!(identify(b"GET /").0, ForeignProtocolMatch::Incomplete);
    }

    #[test]
    fn too_few_bytes_to_match_a_prefix_is_not_foreign() {
        assert_eq!(identify(b"GE").0, ForeignProtocolMatch::None);
        assert_eq!(identify(&[]).0, ForeignProtocolMatch::None);
    }
}
