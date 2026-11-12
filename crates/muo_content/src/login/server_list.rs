//! ServerList directs connecting clients depending on both the address they are connecting from and the address
//! and port they are connecting to. If both ends of a connection are private addresses, the client is sent to the
//! local private address; a public client of a private listener gets the configured or detected public address.
//!
//! shortcut: `HasPublicIPAddress` asks only the interface the system would route to the internet through (std has
//! no interface list), not every adapter. A host with a public address on a second adapter only is then
//! looked up on api.ipify.org needlessly.

use std::net::{IpAddr, SocketAddr};

use muo_primitives::net::is_private_network;
use muo_server::World;
use tracing::{error, info};

use crate::network::events;
use crate::network::gateway_server::ServerListEventArgs;

#[derive(Default)]
pub struct ServerListState {
    public_address: Option<IpAddr>,
    pub address: Option<String>,
    pub server_name: String,
    pub auto_detect: bool,
    use_server_listing_address_config: bool,
}

fn state(w: &mut World) -> &mut ServerListState {
    w.resources.get_mut::<ServerListState>()
}

pub fn server_name(w: &mut World) -> String {
    state(w).server_name.clone()
}

/// `ServerList.Configure`.
pub fn configure(w: &mut World) {
    let address: String = w
        .config
        .get_or_update_nullable_setting("serverListing.address", None)
        .unwrap_or_default()
        .unwrap_or_default();
    let auto = w
        .config
        .get_or_update_setting("serverListing.autoDetect", true)
        .unwrap_or(true);
    let name = w
        .config
        .get_or_update_setting("serverListing.serverName", "muo".to_owned())
        .unwrap_or_else(|_| "muo".to_owned());
    let s = state(w);
    s.address = (!address.is_empty()).then_some(address);
    s.auto_detect = auto;
    s.server_name = name;
}

/// `ServerList.Initialize`.
pub fn initialize(w: &mut World) {
    events::subscribe::<ServerListEventArgs>(w, on_server_list_event);
    let address = state(w).address.clone();
    match address {
        None => {
            if state(w).auto_detect && !has_public_ip_address() {
                match find_public_address() {
                    Some(ip) => {
                        state(w).public_address = Some(ip);
                        info!("Auto-detected public IP address ({ip})");
                    }
                    None => error!(
                        "Could not auto-detect public IP address. Users will not be able to connect!"
                    ),
                }
            }
        }
        Some(a) => {
            if let Some(ip) = resolve(&a) {
                let s = state(w);
                s.public_address = Some(ip);
                s.use_server_listing_address_config = true;
                info!("Server listing address set from config: {ip}");
            }
        }
    }
}

/// `Resolve(addr)`: an address literal, else the last address DNS returns.
fn resolve(addr: &str) -> Option<IpAddr> {
    if let Ok(ip) = muo_primitives::dotnet::parse_ip_address(addr).ok_or(()) {
        return Some(ip);
    }
    use std::net::ToSocketAddrs;
    (addr, 0)
        .to_socket_addrs()
        .ok()
        .and_then(|mut it| it.next_back())
        .map(|s| s.ip())
}

/// `HasPublicIPAddress`: is the address this host reaches the internet from public (see the module's shortcut)?
fn has_public_ip_address() -> bool {
    let local = std::net::UdpSocket::bind("0.0.0.0:0").and_then(|s| {
        s.connect("192.0.2.1:9")?; // no packet is sent
        s.local_addr()
    });
    matches!(local, Ok(a) if !a.ip().is_loopback() && a.ip().is_ipv4() && !is_private_network(a.ip()))
}

const IPIFY_URL: &str = "https://api.ipify.org";

/// `FindPublicAddress`: three tries of one second each.
fn find_public_address() -> Option<IpAddr> {
    const COUNT: usize = 3;
    for _ in 0..COUNT {
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .tls_config(crate::http_tls::tls_config())
            .timeout_global(Some(std::time::Duration::from_secs(1)))
            .build()
            .into();
        if let Ok(mut r) = agent.get(IPIFY_URL).call()
            && let Ok(text) = r.body_mut().read_to_string()
            && let Ok(ip) = muo_primitives::dotnet::parse_ip_address(text.trim()).ok_or(())
        {
            return Some(ip);
        }
    }
    tracing::warn!(
        "Attempted to get a public IP address {COUNT} times from {IPIFY_URL} and failed."
    );
    None
}

/// `OnServerListEvent`.
pub fn on_server_list_event(w: &mut World, e: &mut ServerListEventArgs) {
    let Some(local) = e.local_end_point else {
        return;
    };
    let (mut local_address, local_port) = (local.ip(), local.port());
    let s = w.resources.get_mut::<ServerListState>();
    if s.use_server_listing_address_config {
        if let Some(p) = s.public_address {
            local_address = p;
        }
    } else if is_private_network(local_address) {
        // Check if client is from a public network
        if !is_private_network(e.address)
            && let Some(p) = s.public_address
        {
            local_address = p;
        }
    }
    let name = s.server_name.clone();
    e.add_server(&name, SocketAddr::new(local_address, local_port));
}
