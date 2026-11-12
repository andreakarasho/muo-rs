//! The gateway server: the shard entries of the login server list.

use std::net::{IpAddr, SocketAddr};

use muo_net::server_info::ServerInfo;
use muo_server::World;
use muo_server::net_state::NetStateId;

use super::events;

/// `GatewayServer.ServerListEventArgs`.
pub struct ServerListEventArgs {
    pub state: NetStateId,
    /// `NetState.LocalEndPoint` and `NetState.Address` (the transport's, which handlers cannot reach).
    pub local_end_point: Option<SocketAddr>,
    pub address: IpAddr,
    /// `Account.Username`.
    pub account: Option<String>,
    pub rejected: bool,
    pub servers: Vec<ServerInfo>,
}

impl ServerListEventArgs {
    pub fn new(
        state: NetStateId,
        local_end_point: Option<SocketAddr>,
        address: IpAddr,
        account: Option<String>,
    ) -> Self {
        Self {
            state,
            local_end_point,
            address,
            account,
            rejected: false,
            servers: Vec::new(),
        }
    }

    /// `AddServer(name, address)`: 0% full, the local time zone.
    pub fn add_server(&mut self, name: &str, address: SocketAddr) {
        self.add_server_full(name, 0, local_utc_offset_hours(), address);
    }

    /// `AddServer(name, fullPercent, tz, address)`; the time zone is its UTC offset in hours. A true IPv6 address
    /// cannot be listed (`ServerInfo` refuses it and the server list event rejects the login).
    pub fn add_server_full(
        &mut self,
        name: &str,
        full_percent: i32,
        utc_offset_hours: i32,
        address: SocketAddr,
    ) {
        match ServerInfo::new(name, full_percent, utc_offset_hours, address) {
            Ok(s) => self.servers.push(s),
            Err(e) => {
                tracing::warn!("Unhandled exception at server list: {e}");
                self.rejected = true;
            }
        }
    }
}

/// `TimeZoneInfo.Local.GetUtcOffset(DateTime.Now).Hours`: whole hours, truncated toward zero like `TimeSpan.Hours`.
pub fn local_utc_offset_hours() -> i32 {
    use chrono::Offset;
    chrono::Local::now().offset().fix().local_minus_utc() / 3600
}

/// `GatewayServer.ServerListEvent(e)`.
pub fn server_list_event(w: &mut World, e: &mut ServerListEventArgs) {
    events::invoke(w, e);
}
