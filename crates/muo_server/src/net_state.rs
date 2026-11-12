//! `NetState`, the entity-aware half. The transport (sockets, buffers,
//! framing, throttles) is `muo_net::NetState`, owned by `muo_net::Network<World>` which lives BESIDE the
//! `World`, not in it.
//!
//! What the world needs of a connection is a mirror, [`NetStateData`], keyed by [`NetStateId`] (the transport's
//! slot id): era flags for packet builders, the attached mobile, menus, hue pickers, trades. Anything the game
//! sends goes through [`World::send`] into an outbox which `net_glue::pump` hands to the transport (before and
//! after each `slice`, then a flush), in order. A handler therefore never sends through the transport's
//! `NetState`; it gets `&mut World` and the id.
//!
//! shortcut: `cannot_send` is a mirror refreshed by the pump, so a transport-side close is seen by the world
//! at the next pump.

use indexmap::IndexMap;
use muo_net::client_version::{ClientFlags, ClientType, ClientVersion};
use muo_net::movement::MovementState;
use muo_net::protocol_changes::ProtocolChanges;

use crate::ids::MobileId;
use crate::world::World;

/// A `NetState` reference: the transport slot id (`muo_net::NetState::id`). Stale ids are impossible while
/// the disconnect event detaches the mobile (`Mobile::net_state`) before the slot is reused.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct NetStateId(pub u32);

/// A `SecureTrade` reference (`NetState.Trades`).
// The handle; the trade itself is `secure_trade::SecureTrade` in `World::trades`.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct TradeId(pub u32);

pub const HUE_PICKER_CAP: usize = 512;
pub const MENU_CAP: usize = 512;

/// What a packet builder reads of the client: `NetState.Version`, `ProtocolChanges`, `Flags` and their helpers.
#[derive(Clone, Debug, Default)]
pub struct NetCaps {
    pub version: Option<ClientVersion>,
    pub protocol_changes: ProtocolChanges,
    pub flags: ClientFlags,
}

macro_rules! change_getters {
    ($($name:ident => $flag:ident),* $(,)?) => {
        $(pub fn $name(&self) -> bool { self.has_protocol_changes(ProtocolChanges::$flag) })*
    };
}

impl NetCaps {
    /// `NetState.Version = v`: also sets the protocol changes.
    pub fn set_version(&mut self, v: Option<ClientVersion>) {
        self.protocol_changes = v
            .as_ref()
            .map(|v| v.protocol_changes())
            .unwrap_or_else(ProtocolChanges::empty);
        self.version = v;
    }

    pub fn has_flag(&self, flag: ClientFlags) -> bool {
        self.flags.intersects(flag)
    }

    pub fn has_protocol_changes(&self, changes: ProtocolChanges) -> bool {
        self.protocol_changes.intersects(changes)
    }

    change_getters! {
        new_spellbook => NEW_SPELLBOOK,
        damage_packet => DAMAGE_PACKET,
        buff_icon => BUFF_ICON,
        new_haven => NEW_HAVEN,
        container_grid_lines => CONTAINER_GRID_LINES,
        extended_supported_features => EXTENDED_SUPPORTED_FEATURES,
        stygian_abyss => STYGIAN_ABYSS,
        high_seas => HIGH_SEAS,
        new_character_list => NEW_CHARACTER_LIST,
        new_character_creation => NEW_CHARACTER_CREATION,
        extended_status => EXTENDED_STATUS,
        new_mobile_incoming => NEW_MOBILE_INCOMING,
        new_secure_trading => NEW_SECURE_TRADING,
    }

    /// `NetState.ExpansionInfo`: the newest table entry this client satisfies. Not cached in a field:
    /// the scan is cheap and the version can change, so it is recomputed.
    pub fn expansion_info<'t>(
        &self,
        table: &'t muo_data::expansion::ExpansionTable,
    ) -> &'t muo_data::expansion::ExpansionInfo {
        for info in table.table.iter().rev() {
            // KR sends same client flags as EC
            if self.is_kr_client() && info.client_flags.intersects(ClientFlags::TER_MUR) {
                continue;
            }
            let version_ok = (info.required_client.is_none() || !self.is_kr_client())
                && ClientVersion::compare(self.version.as_ref(), info.required_client.as_ref())
                    .is_ge();
            if version_ok || self.has_flag(info.client_flags) {
                return info;
            }
        }
        table.get_info(0)
    }

    /// `NetState.SupportsExpansion(info, checkCoreExpansion)`; `core_expansion` is `Core.Expansion`.
    pub fn supports_expansion(
        &self,
        table: &muo_data::expansion::ExpansionTable,
        info: &muo_data::expansion::ExpansionInfo,
        core_expansion: i32,
        check_core_expansion: bool,
    ) -> bool {
        (!check_core_expansion || core_expansion >= info.id)
            && self.expansion_info(table).id >= info.id
    }

    fn client_type(&self) -> Option<ClientType> {
        self.version.as_ref().map(|v| v.kind)
    }

    pub fn is_uotd_client(&self) -> bool {
        self.has_flag(ClientFlags::UOTD) || self.client_type() == Some(ClientType::Uotd)
    }
    pub fn is_kr_client(&self) -> bool {
        self.client_type() == Some(ClientType::Kr)
    }
    pub fn is_sa_client(&self) -> bool {
        self.client_type() == Some(ClientType::Sa)
    }
    pub fn is_enhanced_client(&self) -> bool {
        matches!(self.client_type(), Some(ClientType::Kr | ClientType::Sa))
    }
}

/// One entry of the outbox. Order is the order the game produced them.
#[derive(Debug)]
pub enum NetOut {
    Send(NetStateId, Vec<u8>),
    /// `NetState.Disconnect(reason)`.
    Disconnect(NetStateId, String),
    /// `NetState.BlockAllPackets = false` (the transport's flag, which game code cannot reach).
    UnblockPackets(NetStateId),
    /// `NetState.PacketLogging = on` (the transport's flag).
    SetPacketLogging(NetStateId, bool),
    /// `NetState.Account = ...` on the transport, for a login that completes after the packet handler returned.
    SetAccount(NetStateId, Option<String>),
    /// `BanChannel.Retract(address)` (the transport owns the channel).
    BanRetract(std::net::IpAddr),
}

/// The world's side of one connection (the entity-aware fields of `NetState`).
#[derive(Debug)]
pub struct NetStateData {
    pub caps: NetCaps,
    /// `NetState.Mobile`.
    pub mobile: Option<MobileId>,
    /// `NetState.Account`: the username of the transport's `account` field.
    pub account: Option<String>,
    /// The movement throttle's per-connection state, including `NetState.Sequence`. Authoritative here (the
    /// transport's copy is unused) because `Mobile.Move` reads and writes it.
    pub movement: MovementState,
    /// `NetState.CityInfo` (the character list's start cities).
    pub city_info: Vec<crate::city_info::CityInfo>,
    /// `NetState.ServerInfo` (the server list sent at login).
    pub server_info: Vec<muo_net::server_info::ServerInfo>,
    /// `NetState.Trades`; `None` once empty.
    pub trades: Option<Vec<TradeId>>,
    /// `NetState.Menus` (empty is none). See `menus.rs`.
    pub menus: Vec<crate::menus::Menu>,
    /// `NetState.HuePickers` (empty is none). See `hue_picker.rs`.
    pub hue_pickers: Vec<crate::hue_picker::HuePicker>,
    /// Mirror of the transport's `CannotSendPackets`.
    pub cannot_send: bool,
    pub address: Option<std::net::IpAddr>,
    /// `NetState.LocalEndPoint`.
    pub local_end_point: Option<std::net::SocketAddr>,
    /// `NetState.ToString()`.
    pub description: String,
    /// Mirror of the transport's `NetState.Assistant`.
    pub assistant: Option<String>,
}

impl Default for NetStateData {
    fn default() -> Self {
        Self::new(0)
    }
}

impl NetStateData {
    pub fn new(now: i64) -> NetStateData {
        NetStateData {
            caps: NetCaps::default(),
            mobile: None,
            account: None,
            movement: MovementState::new(now),
            city_info: Vec::new(),
            server_info: Vec::new(),
            trades: None,
            menus: Vec::new(),
            hue_pickers: Vec::new(),
            cannot_send: false,
            address: None,
            local_end_point: None,
            description: String::new(),
            assistant: None,
        }
    }
}

impl World {
    pub fn net_state(&self, ns: NetStateId) -> Option<&NetStateData> {
        self.net_states.get(&ns)
    }

    pub fn net_state_mut(&mut self, ns: NetStateId) -> Option<&mut NetStateData> {
        self.net_states.get_mut(&ns)
    }

    /// `ns == null || ns.CannotSendPackets()`.
    pub fn cannot_send_packets(&self, ns: NetStateId) -> bool {
        self.net_states.get(&ns).is_none_or(|d| d.cannot_send)
    }

    /// `NetState.Send(span)`.
    pub fn send(&mut self, ns: NetStateId, bytes: &[u8]) {
        if bytes.is_empty() || self.cannot_send_packets(ns) {
            return;
        }
        self.net_out.push(NetOut::Send(ns, bytes.to_vec()));
    }

    /// `NetState.Disconnect(reason)`.
    pub fn disconnect(&mut self, ns: NetStateId, reason: &str) {
        // `CannotSendPackets` does not look at a queued disconnect: sends up to the handoff still go out.
        if self.net_states.contains_key(&ns) {
            self.net_out
                .push(NetOut::Disconnect(ns, reason.to_string()));
        }
    }

    /// `NetState.PacketLogging = on`.
    pub fn set_packet_logging(&mut self, ns: NetStateId, on: bool) {
        if self.net_states.contains_key(&ns) {
            self.net_out.push(NetOut::SetPacketLogging(ns, on));
        }
    }

    /// `NetState.Mobile` of a connection.
    pub fn net_mobile(&self, ns: NetStateId) -> Option<MobileId> {
        self.net_states.get(&ns)?.mobile
    }
}

pub type NetStates = IndexMap<NetStateId, NetStateData>;
