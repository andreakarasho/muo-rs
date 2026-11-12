//! Outgoing packets that read more of the world than the
//! builders in `muo_server::packets` take as arguments (the connection's account, the core expansion, the maps).

use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::map::MapId;
use muo_server::net_state::NetStateId;
use muo_server::packets::{account, combat, map as map_pk, mobile as mobile_pk, player};

use crate::accounting;
use crate::accounting::handler::account_chars;

/// `ns.Account` as an id.
pub fn state_account(w: &World, ns: NetStateId) -> Option<muo_server::iaccount::AccountId> {
    let name = w.net_state(ns)?.account.clone()?;
    accounting::get_account(w, &name)
}

/// `SendSupportedFeature()`.
pub fn send_supported_feature(w: &mut World, ns: NetStateId) {
    account::send_supported_feature_of_core(w, ns);
}

/// `SendCharacterList()`: the account's characters and the cities to start in.
pub fn send_character_list(w: &mut World, ns: NetStateId) {
    let Some(a) = state_account(w, ns) else {
        return;
    };
    let chars = account_chars(w, a);
    let Some(info) = w.core_expansion_info().cloned() else {
        return;
    };
    let cities = w
        .net_state(ns)
        .map(|d| d.city_info.clone())
        .unwrap_or_default();
    account::send_character_list(w, ns, Some(&chars), &cities, &info);
}

/// `SendAccountLoginAck()`.
pub fn send_account_login_ack(w: &mut World, ns: NetStateId) {
    let servers = w
        .net_state(ns)
        .map(|d| d.server_info.clone())
        .unwrap_or_default();
    account::send_account_login_ack(w, ns, &servers);
}

/// `SendMapPatches()`.
pub fn send_map_patches(w: &mut World, ns: NetStateId) {
    map_pk::send_map_patches_of_world(w, ns);
}

/// `SendLoginConfirmation(m)`.
pub fn send_login_confirmation(w: &mut World, ns: NetStateId, m: MobileId) {
    let sizes: Vec<(u8, (i32, i32))> = w
        .maps
        .all_maps
        .iter()
        .filter_map(|id| w.maps.get(*id).map(|mm| (id.0, (mm.width, mm.height))))
        .collect();
    let felucca = sizes
        .iter()
        .find(|(i, _)| *i == 0)
        .map_or((7168, 4096), |s| s.1);
    account::send_login_confirmation(w, ns, m, |map: Option<MapId>| {
        map.and_then(|id| sizes.iter().find(|(i, _)| *i == id.0))
            .map_or(felucca, |s| s.1)
    });
}

/// The sequence of `IncomingAccountPackets.DoLogin`.
pub fn do_login(w: &mut World, ns: NetStateId, m: MobileId) {
    use muo_server::mobile::base as mb;
    use muo_server::mobile::part2::effects::send_everything;

    send_login_confirmation(w, ns, m);
    let map = w.mobile(m).and_then(|mm| mm.map);
    map_pk::send_map_change(w, ns, map);
    send_map_patches(w, ns);
    let season = mb::get_season(w, m) as u8;
    player::send_season_change(w, ns, season, true);
    send_supported_feature(w, ns);
    muo_server::net_glue::reset_movement_state(w, ns);

    mobile_pk::send_mobile_update(w, ns, m);
    mobile_pk::send_mobile_update(w, ns, m);

    if let Some(ty) = w.mobile(m).map(|mm| mm.ty) {
        (ty.check_light_levels)(w, m, true); // virtual: PlayerMobile's sends 0x4F/0x4E
    }

    mobile_pk::send_mobile_update(w, ns, m);
    mobile_pk::send_mobile_incoming(w, ns, m, m);
    mobile_pk::send_mobile_status(w, ns, m, m);
    let warmode = w.mobile(m).is_some_and(|mm| mm.warmode);
    combat::send_set_war_mode(w, ns, warmode);

    send_everything(w, m);

    send_supported_feature(w, ns);
    mobile_pk::send_mobile_update(w, ns, m);
    mobile_pk::send_mobile_status(w, ns, m, m);
    combat::send_set_war_mode(w, ns, warmode);
    mobile_pk::send_mobile_incoming(w, ns, m, m);

    account::send_login_complete(w, ns);
    player::send_current_time(w, ns);
    player::send_season_change(w, ns, season, true);
    map_pk::send_map_change(w, ns, map);

    let region = w.mobile(m).map(|mm| mm.region);
    if let Some(music) = region.and_then(|r| w.region(r)).map(|r| r.music) {
        player::send_play_music(w, ns, music);
    }

    if w.mobile(m)
        .is_some_and(|mm| mm.ty.is(&crate::player_mobile::PLAYER_MOBILE))
    {
        super::events::player_login_event(w, m);
    }
}
