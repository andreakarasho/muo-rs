//! `BaseHouse`: who the house belongs to and who may enter (owner, co-owners, friends, access, bans) and the
//! commands that change those lists.

use muo_content::core;
use muo_primitives::Point3D;
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::mobile::AccessLevel;
use muo_server::mobile::MobileDelta;
use muo_server::mobile::part2::messages as mm;

use super::{
    HouseData, MAX_CO_OWNERS, SecureLevel, d, data, data_mut, deleted, is_aos_rules, is_house,
    location, map_of, max_bans, max_friends, owner, sign, statics, statics_ref,
};
use muo_content::items::house_deps::deps;

/// `from.SendLocalizedMessage(number)`.
pub fn msg(w: &mut World, to: MobileId, number: i32) {
    mm::send_localized_message(w, to, number, "", 0x3B2);
}

/// `from.SendLocalizedMessage(number, args)`.
pub fn msg_args(w: &mut World, to: MobileId, number: i32, args: &str) {
    mm::send_localized_message(w, to, number, args, 0x3B2);
}

/// The characters of the account of `m`, as `account[i]` reads them (a deleted character reads as null).
pub fn account_mobiles(w: &World, m: MobileId) -> Vec<MobileId> {
    let Some(a) = w.mobile(m).and_then(|mm| mm.account) else {
        return Vec::new();
    };
    let Some(acct) = muo_content::accounting::account(w, a) else {
        return Vec::new();
    };
    acct.mobiles
        .iter()
        .flatten()
        .copied()
        .filter(|&c| w.mobile(c).is_some_and(|x| !x.deleted))
        .collect()
}

/// `AccountHandler.CheckAccount(mobCheck, accCheck)`: is `mob_check` a character of `acc_check`'s account.
pub fn check_account(w: &World, mob_check: MobileId, acc_check: Option<MobileId>) -> bool {
    let Some(acc) = acc_check else { return false };
    account_mobiles(w, acc).contains(&mob_check)
}

fn access_level(w: &World, m: MobileId) -> AccessLevel {
    w.mobile(m).map_or(AccessLevel::Player, |x| x.access_level)
}

/// `Owner` setter.
pub fn set_owner(w: &mut World, house: ItemId, value: Option<MobileId>) {
    let Some(old) = data(w, house).map(|h| h.owner) else {
        return;
    };
    if old == value {
        return;
    }
    if let Some(o) = old {
        let t = &mut statics(w).table;
        if let Some(list) = t.get_mut(&o)
            && let Some(p) = list.iter().position(|&h| h == house)
        {
            list.remove(p);
            if list.is_empty() {
                t.shift_remove(&o);
            }
        }
        muo_server::mobile::base::delta(w, o, MobileDelta::NOTO);
    }
    if let Some(h) = data_mut(w, house) {
        h.owner = value;
    }
    if let Some(o) = value {
        statics(w).table.entry(o).or_default().push(house);
        muo_server::mobile::base::delta(w, o, MobileDelta::NOTO);
    }
    if let Some(s) = sign(w, house) {
        muo_server::item::ops::invalidate_properties(w, s);
    }
}

/// `Public` setter.
pub fn set_public(w: &mut World, house: ItemId, value: bool) {
    let Some(old) = data(w, house).map(|h| h.public) else {
        return;
    };
    if old != value {
        if let Some(h) = data_mut(w, house) {
            h.public = value;
        }
        if !value {
            super::doors::change_sign_type(w, house, 0xBD2); // Privatizing the house, change to brass sign
        }
        if let Some(s) = sign(w, house) {
            muo_server::item::ops::invalidate_properties(w, s);
        }
    }
}

/// `BaseHouse.GetHouses(m)`: the live houses `m` owns.
pub fn get_houses(w: &World, m: MobileId) -> Vec<ItemId> {
    let Some(list) = statics_ref(w).and_then(|s| s.table.get(&m)) else {
        return Vec::new();
    };
    list.iter()
        .copied()
        .filter(|&h| !deleted(w, h) && owner(w, h) == Some(m))
        .collect()
}

/// `BaseHouse.HasHouse(m)`.
pub fn has_house(w: &World, m: MobileId) -> bool {
    statics_ref(w)
        .and_then(|s| s.table.get(&m))
        .is_some_and(|l| l.iter().any(|&h| !deleted(w, h)))
}

/// `BaseHouse.HasAccountHouse(m)`.
pub fn has_account_house(w: &World, m: MobileId) -> bool {
    account_mobiles(w, m).into_iter().any(|c| has_house(w, c))
}

/// `IsOwner(m)`.
pub fn is_owner(w: &World, house: ItemId, m: MobileId) -> bool {
    let o = owner(w, house);
    o == Some(m)
        || access_level(w, m) >= AccessLevel::GameMaster
        || (is_aos_rules(w, house) && check_account(w, m, o))
}

/// `IsCoOwner(m)`.
pub fn is_co_owner(w: &World, house: ItemId, m: MobileId) -> bool {
    let Some(h) = data(w, house) else {
        return false;
    };
    is_owner(w, house, m)
        || h.co_owners.contains(&m)
        || (!is_aos_rules(w, house) && check_account(w, m, h.owner))
}

/// `IsGuildMember(m)`.
pub fn is_guild_member(w: &World, house: ItemId, m: MobileId) -> bool {
    let og = owner(w, house)
        .and_then(|o| w.mobile(o))
        .and_then(|o| o.guild);
    og.is_some() && w.mobile(m).and_then(|x| x.guild) == og
}

/// `IsFriend(m)`.
pub fn is_friend(w: &World, house: ItemId, m: MobileId) -> bool {
    data(w, house).is_some_and(|h| h.initialized)
        && (is_co_owner(w, house, m) || d(w, house).friends.contains(&m))
}

/// `IsBanned(m)`.
pub fn is_banned(w: &World, house: ItemId, m: MobileId) -> bool {
    let Some(h) = data(w, house) else {
        return false;
    };
    if Some(m) == h.owner || access_level(w, m) > AccessLevel::Player {
        return false;
    }
    let their = w.mobile(m).and_then(|x| x.account);
    h.bans.iter().any(|&c| {
        c == m
            || w.mobile(c)
                .and_then(|x| x.account)
                .is_some_and(|a| Some(a) == their)
    })
}

/// `HasAccess(m)`.
pub fn has_access(w: &World, house: ItemId, m: MobileId) -> bool {
    let Some(h) = data(w, house) else {
        return false;
    };
    if access_level(w, m) > AccessLevel::Player || is_friend(w, house, m) || h.access.contains(&m) {
        return true;
    }
    let Some((no_restrictions, master)) = creature_master(w, m) else {
        return false;
    };
    if no_restrictions {
        return true;
    }
    match master {
        Some(mm) => {
            access_level(w, mm) > AccessLevel::Player
                || is_friend(w, house, mm)
                || h.access.contains(&mm)
        }
        None => false,
    }
}

/// For a `BaseCreature`: `(NoHouseRestrictions, GetMaster())`; None for anything else.
fn creature_master(w: &World, m: MobileId) -> Option<(bool, Option<MobileId>)> {
    let flags = super::region_ops::creature_house_flags(w, m)?;
    Some((
        flags.no_house_restrictions,
        muo_content::creature::control::get_master(w, m),
    ))
}

/// `IsCombatRestricted(m)`.
pub fn is_combat_restricted(w: &World, house: ItemId, m: MobileId) -> bool {
    let Some(mob) = w.mobile(m) else { return false };
    if !mob.player || mob.access_level >= AccessLevel::GameMaster || !is_aos_rules(w, house) {
        return false;
    }
    if owner(w, house)
        .and_then(|o| w.mobile(o))
        .is_some_and(|o| o.access_level >= AccessLevel::GameMaster)
    {
        return false;
    }
    let delay = muo_content::regions::house::COMBAT_HEAT_DELAY;
    for info in &mob.aggressed {
        let Some(def) = w.mobile(info.defender) else {
            continue;
        };
        if def.player
            && muo_server::mobile::base::alive(w, info.defender)
            && w.now - info.last_combat_time < delay
        {
            let (Some(ag), Some(dg)) = (mob.guild, def.guild) else {
                return true;
            };
            if dg != ag
                && !deps(w)
                    .and_then(|x| x.guild_is_enemy)
                    .is_some_and(|f| f(w, dg, ag))
            {
                return true;
            }
        }
    }
    false
}

/// `HasSecureAccess(m, level)`.
pub fn has_secure_access(w: &World, house: ItemId, m: MobileId, level: SecureLevel) -> bool {
    if access_level(w, m) >= AccessLevel::GameMaster {
        return true;
    }
    if is_combat_restricted(w, house, m) {
        return false;
    }
    match level {
        SecureLevel::Owner => is_owner(w, house, m),
        SecureLevel::CoOwners => is_co_owner(w, house, m),
        SecureLevel::Friends => is_friend(w, house, m),
        SecureLevel::Anyone => true,
        SecureLevel::Guild => is_guild_member(w, house, m),
    }
}

fn is_player_vendor(w: &World, m: MobileId) -> bool {
    muo_content::factory::mobile_is(w, m, "Server.Mobiles.PlayerVendor")
}

fn creature_no_restrictions(w: &World, m: MobileId) -> bool {
    super::region_ops::creature_house_flags(w, m).is_some_and(|f| f.no_house_restrictions)
}

fn name_of(w: &World, m: MobileId) -> String {
    muo_server::mobile::base::name(w, m).unwrap_or_default()
}

/// `Kick(from, targ)`.
pub fn kick(w: &mut World, house: ItemId, from: MobileId, targ: MobileId) {
    if !is_friend(w, house, from) || !data(w, house).is_some_and(|h| h.initialized) {
        return;
    }
    if access_level(w, targ) > AccessLevel::Player && access_level(w, from) <= access_level(w, targ)
    {
        msg(w, from, 501346); // Uh oh...a bigger boot may be required!
    } else if is_friend(w, house, targ) && !core::ml(w) {
        msg(w, from, 501348); // You cannot eject a friend of the house!
    } else if is_player_vendor(w, targ) {
        msg(w, from, 501351); // You cannot eject a vendor.
    } else if !is_inside_mobile(w, house, targ) {
        msg(w, from, 501352); // You may not eject someone who is not in your house!
    } else if creature_no_restrictions(w, targ) {
        msg(w, from, 501347); // You cannot eject that from the house!
    } else {
        let ban = super::ban_location(w, house);
        move_mobile_to_world(w, targ, ban, map_of(w, house));
        let n = name_of(w, targ);
        msg_args(w, from, 1042840, &n); // ~1_PLAYER NAME~ has been ejected from this house.
        // You have been ejected from this house. If you persist in entering, you may be banned from the house.
        msg(w, targ, 501341);
    }
}

/// `targ.MoveToWorld(location, map)` for a mobile; no map is Map.Internal (nothing moves).
pub fn move_mobile_to_world(
    w: &mut World,
    m: MobileId,
    p: Point3D,
    map: Option<muo_server::map::MapId>,
) {
    if let Some(map) = map {
        muo_server::mobile::base::move_to_world(w, m, p, map);
    }
}

/// `IsInside(Mobile m)`.
pub fn is_inside_mobile(w: &World, house: ItemId, m: MobileId) -> bool {
    let Some(mob) = w.mobile(m) else { return false };
    !mob.deleted
        && mob.map == map_of(w, house)
        && super::storage::is_inside_point(w, house, mob.location, 16)
}

/// `IsInside(Item item)`.
pub fn is_inside_item(w: &World, house: ItemId, item: ItemId) -> bool {
    let Some(i) = w.item(item) else { return false };
    !i.deleted() && i.map == map_of(w, house) && {
        let h = muo_server::item::ops::item_data(w, item).height as i32;
        super::storage::is_inside_point(w, house, i.location, h)
    }
}

/// `RemoveAccess(from, targ)`.
pub fn remove_access(w: &mut World, house: ItemId, from: MobileId, targ: MobileId) {
    if !is_friend(w, house, from) || !data(w, house).is_some_and(|h| h.initialized) {
        return;
    }
    let removed =
        data_mut(w, house).is_some_and(|h| match h.access.iter().position(|&m| m == targ) {
            Some(p) => {
                h.access.remove(p);
                true
            }
            None => false,
        });
    if removed {
        if !has_access(w, house, targ) && is_inside_mobile(w, house, targ) {
            let ban = super::ban_location(w, house);
            muo_server::mobile::base::set_location(w, targ, ban, false);
            msg(w, targ, 1060734); // Your access to this house has been revoked.
        }
        msg(w, from, 1050051); // The invitation has been revoked.
    }
}

/// `RemoveBan(from, targ)`.
pub fn remove_ban(w: &mut World, house: ItemId, from: MobileId, targ: MobileId) {
    if !is_co_owner(w, house, from) || !data(w, house).is_some_and(|h| h.initialized) {
        return;
    }
    if let Some(h) = data_mut(w, house)
        && let Some(p) = h.bans.iter().position(|&m| m == targ)
    {
        h.bans.remove(p);
        msg(w, from, 501297); // The ban is lifted.
    }
}

/// `Ban(from, targ)`.
pub fn ban(w: &mut World, house: ItemId, from: MobileId, targ: MobileId) {
    if !is_friend(w, house, from) || !data(w, house).is_some_and(|h| h.initialized) {
        return;
    }
    let n_bans = d_len(w, house, |h| h.bans.len());
    if access_level(w, targ) > AccessLevel::Player && access_level(w, from) <= access_level(w, targ)
    {
        msg(w, from, 501354); // Uh oh...a bigger boot may be required.
    } else if is_friend(w, house, targ) {
        msg(w, from, 501348); // You cannot eject a friend of the house!
    } else if is_player_vendor(w, targ) {
        msg(w, from, 501351); // You cannot eject a vendor.
    } else if n_bans >= max_bans(w) {
        msg(w, from, 501355); // The ban limit for this house has been reached!
    } else if is_banned(w, house, targ) {
        msg(w, from, 501356); // This person is already banned!
    } else if !is_inside_mobile(w, house, targ) {
        msg(w, from, 501352); // You may not eject someone who is not in your house!
    } else if !data(w, house).is_some_and(|h| h.public) && is_aos_rules(w, house) {
        // You cannot ban someone from a private house.  Revoke their access instead.
        msg(w, from, 1062521);
    } else if creature_no_restrictions(w, targ) {
        msg(w, from, 1062040); // You cannot ban that.
    } else {
        if let Some(h) = data_mut(w, house) {
            h.bans.push(targ);
        }
        let n = name_of(w, targ);
        msg_args(w, from, 1042839, &n); // ~1_PLAYER_NAME~ has been banned from this house.
        msg(w, targ, 501340); // You have been banned from this house.
        let ban = super::ban_location(w, house);
        move_mobile_to_world(w, targ, ban, map_of(w, house));
    }
}

fn d_len(w: &World, house: ItemId, f: impl Fn(&HouseData) -> usize) -> usize {
    data(w, house).map_or(0, f)
}

/// `GrantAccess(from, targ)`.
pub fn grant_access(w: &mut World, house: ItemId, from: MobileId, targ: MobileId) {
    if !is_friend(w, house, from) || !data(w, house).is_some_and(|h| h.initialized) {
        return;
    }
    if has_access(w, house, targ) {
        msg(w, from, 1060729); // That person already has access to this house.
    } else if !w.mobile(targ).is_some_and(|m| m.player) {
        msg(w, from, 1060712); // That is not a player.
    } else if is_banned(w, house, targ) {
        msg(w, from, 501367); // This person is banned!  Unban them first.
    } else {
        if let Some(h) = data_mut(w, house) {
            h.access.push(targ);
        }
        msg(w, targ, 1060735); // You have been granted access to this house.
    }
}

/// `AddCoOwner(from, targ)`.
pub fn add_co_owner(w: &mut World, house: ItemId, from: MobileId, targ: MobileId) {
    if !is_owner(w, house, from) || !data(w, house).is_some_and(|h| h.initialized) {
        return;
    }
    let h = d(w, house);
    if is_owner(w, house, targ) {
        msg(w, from, 501360); // This person is already the house owner!
    } else if h.friends.contains(&targ) {
        msg(w, from, 501361); // This person is a friend of the house. Remove them first.
    } else if !w.mobile(targ).is_some_and(|m| m.player) {
        msg(w, from, 501362); // That can't be a co-owner of the house.
    } else if !core::aos(w) && has_account_house(w, targ) {
        msg(w, from, 501364); // That person is already a house owner.
    } else if is_banned(w, house, targ) {
        msg(w, from, 501367); // This person is banned!  Unban them first.
    } else if h.co_owners.len() >= MAX_CO_OWNERS {
        msg(w, from, 501368); // Your co-owner list is full!
    } else if h.co_owners.contains(&targ) {
        msg(w, from, 501369); // This person is already on your co-owner list!
    } else {
        if let Some(h) = data_mut(w, house) {
            h.co_owners.push(targ);
        }
        muo_server::mobile::base::delta(w, targ, MobileDelta::NOTO);
        msg(w, targ, 501343); // You have been made a co-owner of this house.
    }
}

/// `RemoveCoOwner(from, targ)`.
pub fn remove_co_owner(w: &mut World, house: ItemId, from: MobileId, targ: MobileId) {
    if !is_owner(w, house, from) || !data(w, house).is_some_and(|h| h.initialized) {
        return;
    }
    let removed =
        data_mut(w, house).is_some_and(|h| match h.co_owners.iter().position(|&m| m == targ) {
            Some(p) => {
                h.co_owners.remove(p);
                true
            }
            None => false,
        });
    if !removed {
        return;
    }
    muo_server::mobile::base::delta(w, targ, MobileDelta::NOTO);
    msg(w, from, 501299); // Co-owner removed from list.
    msg(w, targ, 501300); // You have been removed as a house co-owner.

    let secures = d(w, house).secures;
    for (i, info) in secures.iter().enumerate() {
        let is_mine = deps(w)
            .and_then(|x| x.strong_box_owner)
            .and_then(|f| f(w, info.item))
            .is_some_and(|o| o == Some(targ));
        if is_mine {
            let b = info.item;
            muo_server::item::ops::set_locked_down(w, b, false);
            muo_server::item::ops::set_secure(w, b, false);
            if let Some(h) = data_mut(w, house) {
                h.secures.remove(i);
            }
            if let Some(f) = deps(w).and_then(|x| x.strong_box_destroy) {
                f(w, b);
            }
            break;
        }
    }
}

/// `AddFriend(from, targ)`.
pub fn add_friend(w: &mut World, house: ItemId, from: MobileId, targ: MobileId) {
    if !is_co_owner(w, house, from) || !data(w, house).is_some_and(|h| h.initialized) {
        return;
    }
    let h = d(w, house);
    if is_owner(w, house, targ) {
        msg(w, from, 501370); // This person is already an owner of the house!
    } else if h.co_owners.contains(&targ) {
        msg(w, from, 501369); // This person is already on your co-owner list!
    } else if !w.mobile(targ).is_some_and(|m| m.player) {
        msg(w, from, 501371); // That can't be a friend of the house.
    } else if is_banned(w, house, targ) {
        msg(w, from, 501374); // This person is banned!  Unban them first.
    } else if h.friends.len() >= max_friends(w) {
        msg(w, from, 501375); // Your friends list is full!
    } else if h.friends.contains(&targ) {
        msg(w, from, 501376); // This person is already on your friends list!
    } else {
        if let Some(h) = data_mut(w, house) {
            h.friends.push(targ);
        }
        muo_server::mobile::base::delta(w, targ, MobileDelta::NOTO);
        msg(w, targ, 501337); // You have been made a friend of this house.
    }
}

/// `RemoveFriend(from, targ)`.
pub fn remove_friend(w: &mut World, house: ItemId, from: MobileId, targ: MobileId) {
    if !is_co_owner(w, house, from) || !data(w, house).is_some_and(|h| h.initialized) {
        return;
    }
    let removed =
        data_mut(w, house).is_some_and(|h| match h.friends.iter().position(|&m| m == targ) {
            Some(p) => {
                h.friends.remove(p);
                true
            }
            None => false,
        });
    if removed {
        muo_server::mobile::base::delta(w, targ, MobileDelta::NOTO);
        msg(w, from, 501298); // Friend removed from list.
        msg(w, targ, 1060751); // You are no longer a friend of this house.
    }
}

/// `FindHouseAt(Point3D loc, Map map, int height)`.
pub fn find_house_at_point(
    w: &World,
    loc: Point3D,
    map: Option<muo_server::map::MapId>,
    height: i32,
) -> Option<ItemId> {
    let map = map.filter(|&m| m != muo_server::map::MapId::INTERNAL)?;
    muo_server::map::enumerate::multis_in_sector(w, Some(map), loc.x, loc.y)
        .filter(|&h| is_house(w, h))
        .find(|&h| super::storage::is_inside_point(w, h, loc, height))
}

/// `FindHouseAt(Mobile m)`.
pub fn find_house_at_mobile(w: &World, m: MobileId) -> Option<ItemId> {
    let mob = w.mobile(m).filter(|x| !x.deleted)?;
    find_house_at_point(w, mob.location, mob.map, 16)
}

/// `FindHouseAt(Item item)`.
pub fn find_house_at_item(w: &World, item: ItemId) -> Option<ItemId> {
    let i = w.item(item).filter(|x| !x.deleted())?;
    let loc = muo_server::item::ops::get_world_location(w, item);
    let h = muo_server::item::ops::item_data(w, item).height as i32;
    find_house_at_point(w, loc, i.map, h)
}

/// Where the house's own location is (for callers outside the module).
pub fn house_location(w: &World, house: ItemId) -> Point3D {
    location(w, house)
}
