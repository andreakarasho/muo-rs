//! A shard entry of the server list.

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

/// A shard listed in the login server list (0xA8).
#[derive(Clone, Debug)]
pub struct ServerInfo {
    pub name: String,
    pub full_percent: i32,
    /// Whole hours of the UTC offset (`TimeZoneInfo.GetUtcOffset(now).Hours`).
    pub time_zone: i32,
    address: SocketAddr,
    raw_address: u32,
}

impl ServerInfo {
    /// `utc_offset_hours` is `tz.GetUtcOffset(DateTime.Now).Hours`; the caller resolves the time zone.
    pub fn new(
        name: impl Into<String>,
        full_percent: i32,
        utc_offset_hours: i32,
        address: SocketAddr,
    ) -> Result<Self, String> {
        let raw_address = Self::raw(address.ip())?;
        Ok(Self {
            name: name.into(),
            full_percent,
            time_zone: utc_offset_hours,
            address,
            raw_address,
        })
    }

    /// The four address bytes read little-endian, as the client expects them (`MapToIPv4`). UO doesn't
    /// support IPv6 servers, so a true IPv6 address is an error.
    fn raw(ip: IpAddr) -> Result<u32, String> {
        let v4: Ipv4Addr = match ip {
            IpAddr::V4(a) => a,
            IpAddr::V6(a) => a
                .to_ipv4_mapped()
                .ok_or("IP Address could not be serialized to an integer")?,
        };
        Ok(u32::from_le_bytes(v4.octets()))
    }

    pub fn address(&self) -> SocketAddr {
        self.address
    }

    pub fn set_address(&mut self, address: SocketAddr) -> Result<(), String> {
        self.raw_address = Self::raw(address.ip())?;
        self.address = address;
        Ok(())
    }

    pub fn raw_address(&self) -> u32 {
        self.raw_address
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_address_is_little_endian_of_the_octets() {
        let s = ServerInfo::new("Test", 0, -5, "192.168.1.2:2593".parse().unwrap()).unwrap();
        assert_eq!(s.raw_address(), 0x0201_A8C0);
        assert_eq!(s.time_zone, -5);
    }

    #[test]
    fn mapped_v6_is_accepted_and_real_v6_rejected() {
        let s = ServerInfo::new("t", 0, 0, "[::ffff:10.0.0.1]:1".parse().unwrap()).unwrap();
        assert_eq!(s.raw_address(), u32::from_le_bytes([10, 0, 0, 1]));
        assert!(ServerInfo::new("t", 0, 0, "[2001:db8::1]:1".parse().unwrap()).is_err());
    }

    #[test]
    fn set_address_keeps_state_on_failure() {
        let mut s = ServerInfo::new("t", 0, 0, "1.2.3.4:1".parse().unwrap()).unwrap();
        assert!(s.set_address("[2001:db8::1]:1".parse().unwrap()).is_err());
        assert_eq!(s.raw_address(), u32::from_le_bytes([1, 2, 3, 4]));
    }
}
