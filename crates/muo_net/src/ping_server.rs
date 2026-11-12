//! The ping server.
//!
//! A UDP echo: every datagram received on the ping port is sent straight back, so clients can measure
//! latency. The game loop calls [`PingServer::service`]
//! once per tick and the sockets are nonblocking.

use mio::net::UdpSocket;
use std::io::ErrorKind;
use std::net::SocketAddr;
use tracing::{info, warn};

/// `pingServer.*` settings: `enabled` (default true), `port` (default 12000), `maxConnections` (default 2048).
#[derive(Copy, Clone, Debug)]
pub struct PingConfig {
    pub enabled: bool,
    pub port: u16,
    pub max_queued: i32,
}

impl Default for PingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            port: 12000,
            max_queued: 2048,
        }
    }
}

#[derive(Default)]
pub struct PingServer {
    pub config: PingConfig,
    listeners: Vec<UdpSocket>,
}

impl PingServer {
    pub fn new(config: PingConfig) -> Self {
        Self {
            config,
            listeners: Vec::new(),
        }
    }

    pub fn enabled(&self) -> bool {
        self.config.enabled
    }

    pub fn port(&self) -> u16 {
        self.config.port
    }

    /// Binds one socket per server listener address, on the ping port. Returns the bound addresses.
    pub fn start(&mut self, server_listeners: &[SocketAddr]) -> Vec<SocketAddr> {
        if !self.config.enabled {
            return Vec::new();
        }

        let mut bound = Vec::new();
        for server in server_listeners {
            let mut ipep = *server;
            ipep.set_port(self.config.port);

            if let Some(socket) = Self::create_listener(ipep) {
                // shortcut: a wildcard address is logged as itself, not expanded to each NIC address
                // (`NetState.GetListeningAddresses` needs interface enumeration, as for the TCP listener)
                if let Ok(local) = socket.local_addr()
                    && !bound.contains(&local)
                {
                    bound.push(local);
                }
                self.listeners.push(socket);
            }
        }
        // after every socket is bound, once per address
        for local in &bound {
            info!("Listening: {}:{} (Pings)", local.ip(), local.port());
        }
        bound
    }

    pub fn create_listener(ipep: SocketAddr) -> Option<UdpSocket> {
        match UdpSocket::bind(ipep) {
            Ok(s) => Some(s),
            Err(e) => {
                match e.kind() {
                    ErrorKind::AddrInUse => warn!(
                        "Ping Listener: {}:{}: Failed (In Use)",
                        ipep.ip(),
                        ipep.port()
                    ),
                    ErrorKind::AddrNotAvailable => {
                        warn!(
                            "Ping Listener {}:{}: Failed (Unavailable)",
                            ipep.ip(),
                            ipep.port()
                        )
                    }
                    _ => warn!("Ping Listener Exception: {e}"),
                }
                None
            }
        }
    }

    /// Echoes every datagram waiting on every listener. Errors on a single datagram are ignored.
    pub fn service(&mut self) {
        let mut buf = [0u8; 65536];
        for socket in &self.listeners {
            // Bounded so a flood cannot stall the tick: at most `max_queued` datagrams per listener
            for _ in 0..self.config.max_queued.max(1) {
                match socket.recv_from(&mut buf) {
                    Ok((n, from)) => {
                        let _ = socket.send_to(&buf[..n], from);
                    }
                    Err(e) if e.kind() == ErrorKind::WouldBlock => break,
                    Err(_) => continue,
                }
            }
        }
    }

    pub fn listener_addrs(&self) -> Vec<SocketAddr> {
        self.listeners
            .iter()
            .filter_map(|l| l.local_addr().ok())
            .collect()
    }

    pub fn shutdown(&mut self) {
        self.listeners.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    #[test]
    fn echoes_datagrams() {
        let mut ps = PingServer::new(PingConfig {
            enabled: true,
            port: 0,
            max_queued: 16,
        });
        let bound = ps.start(&["127.0.0.1:0".parse().unwrap()]);
        assert_eq!(bound.len(), 1);

        let client = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
        client
            .set_read_timeout(Some(Duration::from_millis(50)))
            .unwrap();
        client.send_to(&[1, 2, 3, 4, 5], bound[0]).unwrap();

        let deadline = Instant::now() + Duration::from_secs(5);
        let mut buf = [0u8; 16];
        loop {
            ps.service();
            if let Ok((n, _)) = client.recv_from(&mut buf) {
                assert_eq!(&buf[..n], &[1, 2, 3, 4, 5]);
                break;
            }
            assert!(Instant::now() < deadline, "no echo");
        }
        ps.shutdown();
    }

    #[test]
    fn disabled_binds_nothing() {
        let mut ps = PingServer::new(PingConfig {
            enabled: false,
            ..Default::default()
        });
        assert!(ps.start(&["127.0.0.1:0".parse().unwrap()]).is_empty());
    }
}
