//! Calls the A to L gumps make into items, houses and guilds that other crates own, as `fn` pointers in
//! `w.resources` set by the owning module's `register`. An unset hook behaves as if the system were absent or
//! inert (noted per field).

use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::net_state::NetStateId;

type FactionFindFn = fn(&World, MobileId) -> Option<(u32, String)>;
#[derive(Default)]
pub struct AlHooks {
    /// `BaseImprisonedMobile.Summon` of the statuette. Unset: none (the gump does nothing).
    pub imprisoned_summon: Option<fn(&mut World, ItemId) -> Option<MobileId>>,
    /// `BaseImprisonedMobile.Release(from, summon)`. Unset: nothing.
    pub imprisoned_release: Option<fn(&mut World, ItemId, MobileId, MobileId)>,
    /// `vendor.House = house` of a `PlayerVendor` or `PlayerBarkeeper`. Unset: nothing.
    pub set_vendor_house: Option<fn(&mut World, MobileId, Option<ItemId>)>,
    /// `securable.Level = level` of an `ISecurable` item (a `SecureLevel` byte). Unset: nothing changes.
    pub securable_set_level: Option<fn(&mut World, ItemId, u8)>,
    /// `PlayerState.Find(m)?.IsLeaving == true` (Factions). Unset: false.
    pub faction_is_leaving: Option<fn(&World, MobileId) -> bool>,
    /// `new GuildTeleporter(guildstone)`: the new item. Unset: none, the guildstone stays put.
    pub new_guild_teleporter: Option<fn(&mut World, ItemId) -> Option<ItemId>>,
    /// `from.SendGump(new GuildInfoGump(from, guild))` (the new guild system).
    pub open_guild_info_gump: Option<fn(&mut World, MobileId, muo_server::guild::GuildId)>,
    /// `NetState.Assistant` of a connection (the transport's field). Unset: none.
    pub client_assistant: Option<fn(&World, NetStateId) -> Option<String>>,
    /// `BaseCommand.IsAccessible(from, focus)`. Unset: true.
    pub is_accessible: Option<fn(&World, MobileId, MobileId) -> bool>,
    /// `CommandLogging.WriteLine(from, text)`. Unset: nothing is logged.
    pub command_log: Option<fn(&mut World, MobileId, &str)>,
    /// `from.SendGump(new PropertiesGump(from, focus))` (`m_z::props`). Unset: nothing.
    pub open_properties_gump: Option<fn(&mut World, MobileId, MobileId)>,
    /// `from.SendGump(new SkillsGump(from, focus))` (`m_z::skills`). Unset: nothing.
    pub open_skills_gump: Option<fn(&mut World, MobileId, MobileId)>,
    /// `NetState.GetNetworkStats()` (the transport owns the counters). Unset: all zero.
    pub network_stats: Option<fn(&World) -> Option<muo_net::network_stats::NetworkStats>>,
    /// `BanChannel.Retract(address)` (the transport owns the channel). Unset: nothing.
    pub ban_retract: Option<fn(&mut World, std::net::IpAddr)>,
    /// `Faction.Find(m)`: the faction (identity, `ToString()`) of a mobile. Unset: none.
    pub faction_find: Option<FactionFindFn>,
    /// `HouseFoundation.BeginCustomize(from)`. Unset: nothing.
    pub foundation_begin_customize: Option<fn(&mut World, ItemId, MobileId)>,
    /// `foundation.Signpost != null`. Unset: false.
    pub foundation_has_signpost: Option<fn(&World, ItemId) -> bool>,
    /// `foundation.SignHanger.ItemID = id`. Unset: nothing.
    pub foundation_set_sign_hanger: Option<fn(&mut World, ItemId, i32)>,
    /// `foundation.Type = t` with the backup, design and current states re-applied (`FoundationType` as a number:
    /// Stone 0, DarkWood 1, LightWood 2, Dungeon 3, Brick 4, ElvenGrey 5, ElvenNatural 6, Crystal 7, Shadow 8).
    pub foundation_set_type: Option<fn(&mut World, ItemId, i32)>,
    /// `foundation.SignpostGraphic = id; foundation.CheckSignpost()`.
    pub foundation_set_signpost_graphic: Option<fn(&mut World, ItemId, i32)>,
    /// `house.HasAddonContainers`. Unset: false.
    pub has_addon_containers: Option<fn(&World, ItemId) -> bool>,
}

pub fn hooks(w: &World) -> Option<&AlHooks> {
    w.resources.get::<AlHooks>()
}

pub fn hooks_mut(w: &mut World) -> &mut AlHooks {
    w.resources.get_mut::<AlHooks>()
}

/// Configure phase: sets the hooks whose targets this crate (or a crate below it) owns. The rest are set by the
/// crates that own them: `muo_engines::special_wiring` (factions, `GuildTeleporter`, the imprisoned mobiles).
/// `client_assistant`, `network_stats` and `ban_retract` reach the transport through mirrors in the `World`.
pub fn configure(w: &mut World) {
    use super::common::send;
    use muo_server::ids::EntityId;
    use muo_server::target::Targeted;

    let h = hooks_mut(w);
    // The transport lives beside the world: its numbers are the `NetStatsSnapshot` and `NetStateData::assistant`
    // mirrors `NetworkHooks` keeps, and `BanChannel.Retract` goes out as a `NetOut`.
    h.network_stats = Some(|w| {
        w.resources
            .get::<muo_server::net_glue::NetStatsSnapshot>()
            .map(|s| s.stats.clone())
    });
    h.client_assistant = Some(|w, ns| w.net_state(ns).and_then(|d| d.assistant.clone()));
    h.ban_retract = Some(|w, ip| {
        w.net_out
            .push(muo_server::net_state::NetOut::BanRetract(ip))
    });
    h.command_log = Some(muo_commands::logging::write_line);
    h.is_accessible = Some(|w, from, focus| {
        muo_commands::generic::is_accessible(
            w,
            from,
            Some(Targeted::Entity(EntityId::Mobile(focus))),
        )
    });
    h.securable_set_level = Some(|w, item, level| {
        muo_content::items::house_deps::set_securable_level(w, item, level);
    });
    h.open_guild_info_gump = Some(super::new_guilds::open_info_gump);
    h.open_properties_gump = Some(|w, from, focus| {
        let g = crate::m_z::props::properties::new(
            w,
            from,
            crate::m_z::props::host::PropObj::Entity(EntityId::Mobile(focus)),
        );
        send(w, from, g);
    });
    // CommandHooks: the gumps the commands open (`[go`, `[client`, `[skills`, the ban duration of `[ban`).
    let c = w.resources.get_mut::<muo_commands::hooks::CommandHooks>();
    c.go_gump = Some(super::go::display_to);
    c.interface_gump = Some(super::interface::open);
    c.client_gump = Some(|w, from, targ| {
        let state = w.mobile(targ).and_then(|m| m.net_state);
        let g = super::client::new(w, from, state, "");
        send(w, from, g);
    });
    c.skills_gump = Some(|w, from, targ| {
        let g = crate::m_z::skills::new_skills(w, from, targ, None);
        send(w, from, g);
    });
    c.ban_duration_gump = Some(|w, from, targ| {
        if let Some(a) = w.mobile(targ).and_then(|m| m.account) {
            send(w, from, super::ban_duration::new_one(a));
        }
    });
    let h = hooks_mut(w);
    h.open_skills_gump = Some(|w, from, focus| {
        let g = crate::m_z::skills::new_skills(w, from, focus, None);
        send(w, from, g);
    });
}
