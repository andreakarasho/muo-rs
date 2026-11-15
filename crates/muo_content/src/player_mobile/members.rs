//! The `PlayerMobile` members that `player_mobile.rs` does not hold: equipment validation, movement,
//! the context menu, trades, equip checks, death and resurrection, speech, name and click handling, pets
//! (`AutoStablePets`), the Young death teleport, the macros and the mount block.

use std::collections::VecDeque;

use indexmap::IndexSet;
use muo_primitives::MessageType;
use muo_primitives::{DateTime, Direction, Point3D, TimeSpan};
use muo_server::World;
use muo_server::context_menu_entry::ContextMenuEntry;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::ops;
use muo_server::layer::Layer;
use muo_server::mobile::part1::props_b;
use muo_server::mobile::part2::{effects, messages};
use muo_server::mobile::{self, AccessLevel, MOBILE_BASE, MobileDelta, StatType};
use muo_server::property_list::PropertyList;
use muo_server::skills::SkillName;
use muo_server::timer::{self, TimerHandle};

use super::buff::{self, BuffIcon, BuffInfo};
use super::{BlockMountType, PlayerFlag, RACE_GARGOYLE, data, data_mut, hooks, tables, young};
use crate::misc::aos_helpers;

/// `PlayerMobile.MountBlock`.
#[derive(Debug)]
pub struct MountBlock {
    ty: BlockMountType,
    timer: TimerHandle,
}

/// `SpeechLogEntry`.
#[derive(Clone, Debug)]
pub struct SpeechLogEntry {
    pub from: MobileId,
    pub name: String,
    pub speech: String,
    pub created: DateTime,
}

pub fn mobile_ty(w: &World, id: MobileId) -> Option<&'static mobile::MobileType> {
    w.mobile(id).map(|m| m.ty)
}

/// `item.Name ?? $"#{item.LabelNumber}"`.
pub fn name_or_label(w: &World, item: ItemId) -> String {
    let Some(i) = w.item(item) else {
        return String::new();
    };
    i.name
        .clone()
        .unwrap_or_else(|| format!("#{}", (i.ty.label_number)(w, item)))
}

// ---- equipment ----

/// `ValidateEquipment`
pub fn validate_equipment(w: &mut World, id: MobileId) {
    let Some(m) = w.mobile(id) else { return };
    if data(w, id).is_some_and(|d| d.no_delta_recursion)
        || m.map.is_none()
        || m.map == Some(muo_server::map::MapId::INTERNAL)
    {
        return;
    }
    if let Some(d) = data_mut(w, id) {
        d.no_delta_recursion = true;
    }
    timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 1, move |w| {
        validate_equipment_sandbox(w, id)
    });
}

/// Whether the clothing's `RequiredRace` names a race other than the wearer's.
pub fn clothing_wrong_race(w: &World, item: ItemId, id: MobileId) -> bool {
    use crate::items::clothing;
    let Some(name) = clothing::class(w, item).required_race else {
        return false;
    };
    let Some(races) = w.resources.get::<muo_server::race::Races>() else {
        return true;
    };
    let required = races
        .races
        .iter()
        .flatten()
        .find(|r| r.name == name)
        .map(|r| r.id());
    required.is_none_or(|r| w.mobile(id).and_then(|m| m.race) != Some(r))
}

/// `ValidateEquipment_Sandbox`
pub fn validate_equipment_sandbox(w: &mut World, id: MobileId) {
    use crate::items::{armor, clothing, weapon};
    let end = |w: &mut World| {
        if let Some(d) = data_mut(w, id) {
            d.no_delta_recursion = false;
        }
    };
    let Some(m) = w.mobile(id) else { return end(w) };
    if m.map.is_none() || m.map == Some(muo_server::map::MapId::INTERNAL) {
        return end(w);
    }
    let ty = m.ty;
    let (str, dex, intel) = ((ty.str)(w, id), (ty.dex)(w, id), (ty.int)(w, id));
    let mut moved = false;
    let mut faction_item_count = 0;
    let ethic = hooks::ethic_of_mobile(w, id);
    let mut i = w.mobile(id).map_or(0, |m| m.items.len());
    while i > 0 {
        i -= 1;
        let Some(&item) = w.mobile(id).and_then(|m| m.items.get(i)) else {
            continue;
        };
        let item_ethic = hooks::ethic_of_item(w, item);
        if item_ethic.is_some() && item_ethic != ethic {
            mobile::part2::items::add_to_backpack(w, id, item);
            moved = true;
            continue;
        }
        let (female, access) = w
            .mobile(id)
            .map_or((false, AccessLevel::Player), |m| (m.female, m.access_level));
        if weapon::is_weapon(w, item) {
            let lower = weapon::lower_stat_req(w, item);
            let drop = dex < weapon::dex_requirement(w, item)
                || str < aos_helpers::scale(weapon::str_requirement(w, item), 100 - lower)
                || intel < weapon::int_requirement(w, item)
                || !muo_server::item::base::check_race(w, item, id, false);
            if drop {
                // You can no longer wield your ~1_WEAPON~
                messages::send_localized_message(w, id, 1062001, &name_or_label(w, item), 0x3B2);
                mobile::part2::items::add_to_backpack(w, id, item);
                moved = true;
            }
        } else if armor::is_armor(w, item) {
            let (male, fem) = armor::allow_wearer(w, item);
            let gender =
                access < AccessLevel::GameMaster && ((!male && !female) || (!fem && female));
            let drop = if gender || !muo_server::item::base::check_race(w, item, id, false) {
                true
            } else {
                let (str_bonus, str_req) = (
                    armor::compute_stat_bonus(w, item, StatType::STR),
                    armor::compute_stat_req(w, item, StatType::STR),
                );
                let (dex_bonus, dex_req) = (
                    armor::compute_stat_bonus(w, item, StatType::DEX),
                    armor::compute_stat_req(w, item, StatType::DEX),
                );
                let (int_bonus, int_req) = (
                    armor::compute_stat_bonus(w, item, StatType::INT),
                    armor::compute_stat_req(w, item, StatType::INT),
                );
                dex < dex_req
                    || dex + dex_bonus < 1
                    || str < str_req
                    || str + str_bonus < 1
                    || intel < int_req
                    || intel + int_bonus < 1
            };
            if drop {
                let name = name_or_label(w, item);
                if armor::is_shield(w, item) {
                    messages::send_localized_message(w, id, 1062003, &name, 0x3B2); // You can no longer equip your ~1_SHIELD~
                } else {
                    messages::send_localized_message(w, id, 1062002, &name, 0x3B2); // You can no longer wear your ~1_ARMOR~
                }
                mobile::part2::items::add_to_backpack(w, id, item);
                moved = true;
            }
        } else if clothing::is_clothing(w, item) {
            let (male, fem) = clothing::allow_wearer(w, item);
            let gender =
                access < AccessLevel::GameMaster && ((!male && !female) || (!fem && female));
            let drop = if gender || clothing_wrong_race(w, item, id) {
                true
            } else {
                let str_bonus = clothing::compute_stat_bonus(w, item, StatType::STR);
                let str_req = clothing::compute_stat_req(w, item, StatType::STR);
                str < str_req || str + str_bonus < 1
            };
            if drop {
                // You can no longer wear your ~1_ARMOR~
                messages::send_localized_message(w, id, 1062002, &name_or_label(w, item), 0x3B2);
                mobile::part2::items::add_to_backpack(w, id, item);
                moved = true;
            }
        }

        if let Some(item_faction) = hooks::faction_item_faction(w, item) {
            let our = hooks::faction_of(w, id);
            let mut drop = false;
            if our.is_none() || our != Some(item_faction) {
                drop = true;
            } else {
                faction_item_count += 1;
                if faction_item_count > hooks::faction_max_wearables(w, id) {
                    drop = true;
                }
            }
            if drop {
                mobile::part2::items::add_to_backpack(w, id, item);
                moved = true;
            }
        }
    }
    if moved {
        messages::send_localized_message(w, id, 500647, "", 0x3B2); // Some equipment has been moved to your backpack.
    }
    end(w);
}

/// `Delta`
pub fn delta(w: &mut World, id: MobileId, flag: MobileDelta) {
    (MOBILE_BASE.delta)(w, id, flag);
    if flag.contains(MobileDelta::STAT) {
        validate_equipment(w, id);
    }
}

/// `CheckEquip`
pub fn check_equip(w: &mut World, id: MobileId, item: ItemId) -> bool {
    let access = w.mobile(id).map_or(AccessLevel::Player, |m| m.access_level);
    if let Some(reason) = hooks::item_equip_blocked(w, id, item) {
        messages::send_message_hue(w, id, 0x22, &reason);
        return false;
    }
    let _ = access;
    if !(MOBILE_BASE.check_equip)(w, id, item) {
        return false;
    }
    if !hooks::duel_allow_item_equip(w, id, item) {
        return false;
    }
    if let Some(item_faction) = hooks::faction_item_faction(w, item) {
        let Some(ours) = hooks::faction_of(w, id) else {
            messages::send_localized_message(w, id, 1010371, "", 0x3B2); // You cannot equip a faction item!
            return false;
        };
        if ours != item_faction {
            messages::send_localized_message(w, id, 1010372, "", 0x3B2); // You cannot equip an opposing faction's item!
            return false;
        }
        let mut max_wearables = hooks::faction_max_wearables(w, id);
        let equipped_all = w.mobile(id).map_or_else(Vec::new, |m| m.items.clone());
        for equipped in equipped_all {
            if item != equipped && hooks::faction_item_faction(w, equipped).is_some() {
                max_wearables -= 1;
                if max_wearables == 0 {
                    messages::send_localized_message(w, id, 1010373, "", 0x3B2); // You do not have enough rank to equip more faction items!
                    return false;
                }
            }
        }
    }

    let (access, layer) = (access, w.item(item).map_or(Layer::Invalid, |i| i.layer));
    if access < AccessLevel::GameMaster && layer != Layer::Mount && props_b::has_trade(w, id) {
        let bounce = w.item(item).and_then(|i| i.bounce.clone());
        if let Some(bounce) = bounce {
            let pack = props_b::backpack(w, id);
            match bounce.parent {
                Some(EntityId::Item(parent)) => {
                    let in_pack = Some(parent) == pack
                        || ops::is_child_of(w, parent, pack.map(EntityId::Item));
                    if in_pack {
                        return true;
                    }
                }
                Some(EntityId::Mobile(m)) if m == id => return true,
                _ => {}
            }
        }
        messages::send_localized_message(w, id, 1004042, "", 0x3B2); // You can only equip what you are already carrying while you have a trade pending.
        return false;
    }
    true
}

/// `CheckContentForTrade`
pub fn check_content_for_trade(w: &World, item: ItemId) -> i32 {
    if hooks::item_is_trapped(w, item) {
        return 1004044; // You may not trade trapped items.
    }
    if hooks::item_is_stolen(w, item) {
        return 1004043; // You may not trade recently stolen items.
    }
    if crate::factory::item_is(w, item, "Server.Items.Container") {
        for sub in w.item(item).map_or_else(Vec::new, container_items) {
            let msg = check_content_for_trade(w, sub);
            if msg != 0 {
                return msg;
            }
        }
    }
    0
}

pub fn container_items(i: &muo_server::item::Item) -> Vec<ItemId> {
    i.items.clone()
}

pub fn check_hold(
    w: &mut World,
    owner: MobileId,
    item: ItemId,
    check_items: bool,
    plus_items: i32,
    plus_weight: i32,
) -> bool {
    let Some(pack) = props_b::backpack(w, owner) else {
        return false;
    };
    let Some(ty) = w.item(pack).map(|i| i.ty) else {
        return false;
    };
    (ty.check_hold)(
        w,
        pack,
        Some(owner),
        item,
        false,
        check_items,
        plus_items,
        plus_weight,
    )
}

/// `CheckTrade`
#[allow(clippy::too_many_arguments)]
pub fn check_trade(
    w: &mut World,
    id: MobileId,
    to: MobileId,
    item: ItemId,
    cont: ItemId,
    message: bool,
    check_items: bool,
    mut plus_items: i32,
    mut plus_weight: i32,
) -> bool {
    let has_cont = w.item(cont).is_some();
    let has_item = w.item(item).is_some();
    let mut msg_num = 0;
    if !has_cont {
        if w.mobile(to).is_some_and(|m| m.holding.is_some()) {
            msg_num = 1062727; // You cannot trade with someone who is dragging something.
        } else if props_b::has_trade(w, id) {
            msg_num = 1062781; // You are already trading with someone else!
        } else if props_b::has_trade(w, to) {
            msg_num = 1062779; // That person is already involved in a trade
        } else if data(w, to).is_some_and(|d| d.flag(PlayerFlag::REFUSE_TRADES)) {
            msg_num = 1154111; // ~1_NAME~ is refusing all trades.
        }
    }
    if msg_num == 0 && has_item {
        if has_cont {
            let (items, weight) = w.item(cont).map_or((0, 0), |c| {
                (
                    (c.ty.get_total)(w, cont, muo_server::item::TotalType::Items),
                    (c.ty.get_total)(w, cont, muo_server::item::TotalType::Weight),
                )
            });
            plus_items += items;
            plus_weight += weight;
        }
        if !check_hold(w, id, item, check_items, plus_items, plus_weight) {
            msg_num = 1004040; // You would not be able to hold this if the trade failed.
        } else if !check_hold(w, to, item, check_items, plus_items, plus_weight) {
            msg_num = 1004039; // The recipient of this trade would not be able to carry this.
        } else {
            msg_num = check_content_for_trade(w, item);
        }
    }
    if msg_num != 0 {
        if message {
            if msg_num == 1154111 {
                let name = w
                    .mobile(to)
                    .and_then(|m| (m.ty.name)(w, to))
                    .unwrap_or_default();
                messages::send_localized_message(w, id, msg_num, &name, 0x3B2);
            } else {
                messages::send_localized_message(w, id, msg_num, "", 0x3B2);
            }
        }
        return false;
    }
    true
}

/// `CheckNonlocalDrop`
pub fn check_nonlocal_drop(
    w: &mut World,
    id: MobileId,
    from: MobileId,
    item: ItemId,
    target: ItemId,
) -> bool {
    if !(MOBILE_BASE.check_nonlocal_drop)(w, id, from, item, target) {
        return false;
    }
    if w.mobile(from)
        .is_some_and(|m| m.access_level >= AccessLevel::GameMaster)
    {
        return true;
    }
    let pack = w.mobile(id).and_then(|m| m.backpack);
    let in_pack = |t: ItemId| Some(t) == pack || ops::is_child_of(w, t, pack.map(EntityId::Item));
    if from == id && props_b::has_trade(w, id) && in_pack(target) {
        if let Some(EntityId::Item(parent)) = w
            .item(item)
            .and_then(|i| i.bounce.as_ref())
            .and_then(|b| b.parent)
            && in_pack(parent)
        {
            return true;
        }
        messages::send_localized_message(w, id, 1004041, "", 0x3B2); // You can't do that while you have a trade pending.
        return false;
    }
    true
}

// ---- movement and location ----

/// `Move`
pub fn move_dir(w: &mut World, id: MobileId, d: Direction) -> bool {
    let alive = w.mobile(id).is_some_and(|m| (m.ty.alive)(w, id));
    if w.mobile(id).is_some_and(|m| m.net_state.is_some()) {
        let had_gump = hooks::resurrect_gump_close(w, id, alive);
        if !alive && had_gump {
            messages::send_localized_message(w, id, 500111, "", 0x3B2); // You are frozen and cannot move.
            return false;
        }
    }
    if !alive {
        hooks::ignore_movable_impassables(w, true);
    }
    let res = (MOBILE_BASE.move_dir)(w, id, d);
    hooks::ignore_movable_impassables(w, false);
    res
}

/// `CheckMovement`
pub fn check_movement(w: &mut World, id: MobileId, d: Direction) -> (bool, i32) {
    let Some(b) = hooks::design_bounds(w, id) else {
        return (MOBILE_BASE.check_movement)(w, id, d);
    };
    let Some(m) = w.mobile(id) else {
        return (false, 0);
    };
    let mut p = m.location;
    muo_server::movement::offset(d, &mut p, 1);
    let (new_x, new_y) = (p.x, p.y);
    let ok = new_x >= b.start_x
        && new_y >= b.start_y
        && new_x < b.end_x
        && new_y < b.end_y
        && m.map == b.map;
    (ok, b.z)
}

/// `AllowItemUse`
pub fn allow_item_use(w: &mut World, id: MobileId, item: ItemId) -> bool {
    if let Some(reason) = hooks::item_use_blocked(w, id, item) {
        messages::send_message_hue(w, id, 0x22, &reason);
        return false;
    }
    hooks::duel_allow_item_use(w, id, item) && hooks::design_check(w, id)
}

const ANIMAL_FORM_RESTRICTED_SKILLS: [SkillName; 13] = [
    SkillName::ArmsLore,
    SkillName::Begging,
    SkillName::Discordance,
    SkillName::Forensics,
    SkillName::Inscribe,
    SkillName::ItemID,
    SkillName::Meditation,
    SkillName::Peacemaking,
    SkillName::Provocation,
    SkillName::RemoveTrap,
    SkillName::SpiritSpeak,
    SkillName::Stealing,
    SkillName::TasteID,
];

/// `AllowSkillUse`
pub fn allow_skill_use(w: &mut World, id: MobileId, skill: SkillName) -> bool {
    if let Some(reason) = hooks::skill_blocked(w, id, skill) {
        messages::send_message_hue(w, id, 0x22, &reason);
        return false;
    }
    if hooks::animal_form_under(w, id) && ANIMAL_FORM_RESTRICTED_SKILLS.contains(&skill) {
        messages::send_localized_message(w, id, 1070771, "", 0x3B2); // You cannot use that skill in this form.
        return false;
    }
    hooks::duel_allow_skill_use(w, id, skill) && hooks::design_check(w, id)
}

/// `RecheckTownProtection`
pub fn recheck_town_protection(w: &mut World, id: MobileId) {
    if let Some(d) = data_mut(w, id) {
        d.next_protection_check = 10;
    }
    let is_protected = hooks::guarded_region_protects(w, id);
    let last = data(w, id).is_some_and(|d| d.last_protected_message);
    if is_protected != last {
        if is_protected {
            messages::send_localized_message(w, id, 500112, "", 0x3B2); // You are now under the protection of the town guards.
        } else {
            messages::send_localized_message(w, id, 500113, "", 0x3B2); // You have left the protection of the town guards.
        }
        if let Some(d) = data_mut(w, id) {
            d.last_protected_message = is_protected;
        }
    }
}

/// `MoveToWorld`
pub fn move_to_world(w: &mut World, id: MobileId, loc: Point3D, map: muo_server::map::MapId) {
    (MOBILE_BASE.move_to_world)(w, id, loc, map);
    recheck_town_protection(w, id);
}

/// `SetLocation`
pub fn set_location(w: &mut World, id: MobileId, loc: Point3D, is_teleport: bool) {
    let (access, z) = w
        .mobile(id)
        .map_or((AccessLevel::Player, 0), |m| (m.access_level, m.location.z));
    if !is_teleport && access == AccessLevel::Player {
        // moving, not teleporting
        let z_drop = z - loc.z;
        if z_drop > 20 {
            // we fell more than one story: deal some damage; does not kill, disrupt, etc
            if let Some(m) = w.mobile(id) {
                let hits = m.hits - (z_drop / 20 * 10 - 5);
                mobile::part2::deps::set_hits(w, id, hits);
            }
        }
    }
    (MOBILE_BASE.set_location)(w, id, loc, is_teleport);
    let check = if let Some(d) = data_mut(w, id) {
        d.next_protection_check -= 1;
        d.next_protection_check == 0
    } else {
        false
    };
    if is_teleport || check {
        recheck_town_protection(w, id);
    }
}

/// `OnLocationChange`
pub fn on_location_change(w: &mut World, id: MobileId, _old: Point3D) {
    if let Some(ty) = mobile_ty(w, id) {
        (ty.check_light_levels)(w, id, false);
    }
    hooks::duel_on_location_changed(w, id);
    let Some(b) = hooks::design_bounds(w, id) else {
        return;
    };
    if w.resources.get::<RecursionGuard>().is_some_and(|g| g.0) {
        return;
    }
    w.resources.get_mut::<RecursionGuard>().0 = true;
    let (x, y, z, map) = w.mobile(id).map_or((0, 0, 0, None), |m| {
        (m.location.x, m.location.y, m.location.z, m.map)
    });
    if x >= b.start_x && y >= b.start_y && x < b.end_x && y < b.end_y && map == b.map {
        if z != b.z {
            mobile::part1::lifecycle::set_location(w, id, Point3D::new(x, y, b.z));
        }
        w.resources.get_mut::<RecursionGuard>().0 = false;
        return;
    }
    mobile::part1::lifecycle::set_location(
        w,
        id,
        Point3D::new(b.foundation_x, b.foundation_y, b.z),
    );
    mobile::part2::deps::set_map(w, id, b.map);
    w.resources.get_mut::<RecursionGuard>().0 = false;
}

/// `m_NoRecursion` (static).
#[derive(Default)]
struct RecursionGuard(bool);

/// `OnMoveOver`
pub fn on_move_over(w: &mut World, id: MobileId, m: MobileId) -> bool {
    if let Some(uncontrolled) = hooks::creature_uncontrolled(w, m)
        && uncontrolled
    {
        let alive = |w: &World, x: MobileId| w.mobile(x).is_some_and(|o| (o.ty.alive)(w, x));
        let dead_bonded =
            |w: &World, x: MobileId| w.mobile(x).is_some_and(|o| (o.ty.is_dead_bonded_pet)(w, x));
        let me = w.mobile(id);
        return !alive(w, id)
            || !alive(w, m)
            || dead_bonded(w, id)
            || dead_bonded(w, m)
            || me.is_some_and(|me| me.hidden && me.access_level > AccessLevel::Player);
    }
    if hooks::in_safe_zone(w, id)
        && data(w, m).is_some()
        && hooks::duel_state(w, m)
            .is_none_or(|d| !d.has_player || !d.started || d.finished || d.eliminated)
    {
        return true;
    }
    (MOBILE_BASE.on_move_over)(w, id, m)
}

/// `OnMapChange`
pub fn on_map_change(w: &mut World, id: MobileId, old_map: Option<muo_server::map::MapId>) {
    let facet = hooks::faction_facet(w);
    let map = w.mobile(id).and_then(|m| m.map);
    if facet.is_some() && ((map != facet && old_map == facet) || (map == facet && old_map != facet))
    {
        mobile::part2::properties::invalidate_properties(w, id);
    }
    hooks::duel_on_map_changed(w, id);
    let Some(b) = hooks::design_bounds(w, id) else {
        return;
    };
    if w.resources.get::<RecursionGuard>().is_some_and(|g| g.0) {
        return;
    }
    w.resources.get_mut::<RecursionGuard>().0 = true;
    if map != b.map {
        mobile::part2::deps::set_map(w, id, b.map);
    }
    w.resources.get_mut::<RecursionGuard>().0 = false;
}

/// `OnMove`
pub fn on_move(w: &mut World, id: MobileId, d: Direction) -> bool {
    if !crate::core::se(w) {
        return (MOBILE_BASE.on_move)(w, id, d);
    }
    let Some(m) = w.mobile(id) else { return true };
    if m.access_level != AccessLevel::Player {
        return true;
    }
    if m.hidden && hooks::design_bounds(w, id).is_none() {
        // Hidden & NOT customizing a house
        let mounted = props_b::mounted(w, id);
        if !mounted && muo_server::skills::value_ref(w, id, SkillName::Stealth) >= 25.0 {
            let running = d.is_running();
            let steps = w.mobile(id).map_or(0, |m| m.allowed_stealth_steps);
            if running {
                let left = steps - 2;
                if let Some(m) = w.mobile_mut(id) {
                    m.allowed_stealth_steps = left;
                }
                if left <= 0 {
                    revealing_action(w, id);
                }
            } else {
                if let Some(m) = w.mobile_mut(id) {
                    m.allowed_stealth_steps = steps - 1;
                }
                if steps <= 0 {
                    hooks::stealth_on_use(w, id);
                }
            }
        } else {
            revealing_action(w, id);
        }
    }
    true
}

/// `OnMovement`
pub fn on_movement(w: &mut World, id: MobileId, m: MobileId, old: Point3D) {
    (MOBILE_BASE.on_movement)(w, id, m, old);
    // Passive detect hidden: either party moving within range can trigger detection
    let (Some(me), Some(other)) = (w.mobile(id), w.mobile(m)) else {
        return;
    };
    if data(w, m).is_some() && in_range_pt(me.location, other.location, 4) {
        if other.hidden && me.access_level == AccessLevel::Player {
            hooks::detect_hidden_try(w, id, m);
        } else if me.hidden && other.access_level == AccessLevel::Player {
            hooks::detect_hidden_try(w, m, id);
        }
    }
}

// ---- visibility, harm and help ----

/// `RevealingAction`
pub fn revealing_action(w: &mut World, id: MobileId) {
    if hooks::design_bounds(w, id).is_some() {
        return;
    }
    hooks::invisibility_stop_timer(w, id);
    (MOBILE_BASE.revealing_action)(w, id);
    if let Some(d) = data_mut(w, id) {
        d.is_stealthing = false; // IsStealthing should be moved to Server.Mobiles
    }
}

/// `OnHiddenChanged`
pub fn on_hidden_changed(w: &mut World, id: MobileId) {
    (MOBILE_BASE.on_hidden_changed)(w, id);
    // Always remove, default to the hiding icon EXCEPT in the invis spell where it's explicitly set
    buff::remove_buff(w, id, BuffIcon::Invisibility);
    if !w.mobile(id).is_some_and(|m| m.hidden) {
        buff::remove_buff(w, id, BuffIcon::HidingAndOrStealth);
    } else {
        // Hidden/Stealthing & You Are Hidden
        buff::add_buff(w, id, BuffInfo::new(BuffIcon::HidingAndOrStealth, 1075655));
    }
}

/// `OnSubItemAdded`
pub fn on_sub_item_added(w: &mut World, id: MobileId, item: ItemId) {
    let access = w.mobile(id).map_or(AccessLevel::Player, |m| m.access_level);
    let pack = props_b::backpack(w, id);
    if access < AccessLevel::GameMaster && ops::is_child_of(w, item, pack.map(EntityId::Item)) {
        let max_weight = crate::misc::stamina_system::get_max_weight(w, id);
        let cur_weight =
            mobile::part1::props_b::body_weight(w) + mobile::part1::props_a::total_weight(w, id);
        if cur_weight > max_weight {
            messages::send_localized_message_affix(
                w,
                id,
                1019035,
                true,
                &format!(" : {cur_weight} / {max_weight}"),
                "",
                0,
            );
        }
    }
    (MOBILE_BASE.on_sub_item_added)(w, id, item);
}

/// `CanBeHarmful`
pub fn can_be_harmful_full(
    w: &mut World,
    id: MobileId,
    target: MobileId,
    message: bool,
    ignore_our_blessedness: bool,
) -> bool {
    if hooks::design_bounds(w, id).is_some() || hooks::design_bounds(w, target).is_some() {
        return false;
    }
    if hooks::cannot_be_harmed(w, target) {
        if message {
            let (name, title) = w.mobile(target).map_or((String::new(), None), |m| {
                ((m.ty.name)(w, target).unwrap_or_default(), m.title.clone())
            });
            match title {
                None => messages::send_message(w, id, &format!("{name} cannot be harmed.")),
                Some(t) => messages::send_message(w, id, &format!("{name} {t} cannot be harmed.")),
            }
        }
        return false;
    }
    (MOBILE_BASE.can_be_harmful_full)(w, id, target, message, ignore_our_blessedness)
}

/// `CanBeBeneficial`
pub fn can_be_beneficial_full(
    w: &mut World,
    id: MobileId,
    target: MobileId,
    message: bool,
    allow_dead: bool,
) -> bool {
    if hooks::design_bounds(w, id).is_some() || hooks::design_bounds(w, target).is_some() {
        return false;
    }
    (MOBILE_BASE.can_be_beneficial_full)(w, id, target, message, allow_dead)
}

/// `CheckContextMenuDisplay`
pub fn check_context_menu_display(w: &World, id: MobileId, _target: EntityId) -> bool {
    hooks::design_bounds(w, id).is_none()
}

/// `OnItemAdded`
pub fn on_item_added(w: &mut World, id: MobileId, item: ItemId) {
    (MOBILE_BASE.on_item_added)(w, id, item);
    after_item_change(w, id, item);
}

/// `OnItemRemoved`
pub fn on_item_removed(w: &mut World, id: MobileId, item: ItemId) {
    (MOBILE_BASE.on_item_removed)(w, id, item);
    after_item_change(w, id, item);
}

pub fn after_item_change(w: &mut World, id: MobileId, item: ItemId) {
    let ty = mobile_ty(w, id);
    if (crate::items::armor::is_armor(w, item) || crate::items::weapon::is_weapon(w, item))
        && let Some(ty) = ty
    {
        (ty.check_stat_timers)(w, id);
    }
    if w.mobile(id).is_some_and(|m| m.net_state.is_some())
        && let Some(ty) = ty
    {
        (ty.check_light_levels)(w, id, false);
    }
}

/// `OnBeneficialAction`
pub fn on_beneficial_action(w: &mut World, id: MobileId, target: MobileId, is_criminal: bool) {
    hooks::honor_on_source_beneficial_action(w, id, target);
    (MOBILE_BASE.on_beneficial_action)(w, id, target, is_criminal);
}

/// `IsHarmfulCriminal`
pub fn is_harmful_criminal(w: &mut World, id: MobileId, target: MobileId) -> bool {
    if hooks::stealing_classic_mode(w) && data(w, target).is_some_and(|d| !d.perma_flags.is_empty())
    {
        if muo_server::notoriety::compute(w, id, target) == muo_server::notoriety::INNOCENT {
            muo_server::mobile::base::delta(w, target, muo_server::mobile::MobileDelta::NOTO);
        }
        return false;
    }
    if hooks::creature_initial_innocent_uncontrolled(w, target) {
        return false;
    }
    if crate::core::ml(w) && hooks::creature_controlled_by(w, target, id) {
        return false;
    }
    (MOBILE_BASE.is_harmful_criminal)(w, id, target)
}

// ---- seeing ----

/// `CanSee(Mobile)`
pub fn can_see_mobile(w: &mut World, id: MobileId, m: MobileId) -> bool {
    hooks::statue_requested_animation(w, m, id);
    if data(w, m).is_some_and(|d| d.visibility_list.contains(&id)) {
        return true;
    }
    if let Some(duel) = hooks::duel_state(w, id)
        && !duel.finished
        && duel.tournament
        && !duel.eliminated
    {
        let owner = hooks::creature_master(w, m).unwrap_or(m);
        if w.mobile(m)
            .is_some_and(|x| x.access_level == AccessLevel::Player)
            && data(w, owner).is_some()
            && !hooks::duel_same_context(w, owner, id)
        {
            return false;
        }
    }
    (MOBILE_BASE.can_see_mobile)(w, id, m)
}

/// `CanSee(Item)`
pub fn can_see_item(w: &mut World, id: MobileId, item: ItemId) -> bool {
    !hooks::design_item_hidden(w, id, item) && (MOBILE_BASE.can_see_item)(w, id, item)
}

/// `CheckedAnimate`
#[allow(clippy::too_many_arguments)]
pub fn checked_animate(
    w: &mut World,
    id: MobileId,
    action: i32,
    frames: i32,
    repeat: i32,
    forward: bool,
    rep: bool,
    delay: i32,
) {
    if !props_b::mounted(w, id) {
        mobile::base::animate(w, id, action, frames, repeat, forward, rep, delay);
    }
}

// ---- interaction ----

/// `DisruptiveAction`
pub fn disruptive_action(w: &mut World, id: MobileId) {
    if w.mobile(id).is_some_and(|m| m.meditating) {
        buff::remove_buff(w, id, BuffIcon::ActiveMeditation);
    }
    (MOBILE_BASE.disruptive_action)(w, id);
}

/// `OnDoubleClick`
pub fn on_double_click(w: &mut World, id: MobileId, from: MobileId) {
    if id == from && !w.mobile(id).is_some_and(|m| m.warmode) {
        let mounted = props_b::mount(w, id).is_some();
        if mounted && !hooks::design_check(w, id) {
            return;
        }
    }
    (MOBILE_BASE.on_double_click)(w, id, from);
}

/// `DisplayPaperdollTo`
pub fn display_paperdoll_to(w: &mut World, id: MobileId, to: MobileId) {
    if hooks::design_check(w, id) {
        (MOBILE_BASE.display_paperdoll_to)(w, id, to);
    }
}

/// `OnSingleClick`
pub fn on_single_click(w: &mut World, id: MobileId, from: MobileId) {
    let in_facet = hooks::faction_facet(w).is_some()
        && w.mobile(id).and_then(|m| m.map) == hooks::faction_facet(w);
    if in_facet && let Some((text, ascii, hue)) = hooks::faction_single_click_label(w, id, from) {
        let ns = w.mobile(from).and_then(|m| m.net_state);
        messages::private_overhead_message(w, id, MessageType::LABEL, hue, ascii, &text, ns);
    }
    (MOBILE_BASE.on_single_click)(w, id, from);
}

/// `GetProperties`
pub fn get_properties(w: &World, id: MobileId, list: &mut PropertyList) {
    (MOBILE_BASE.get_properties)(w, id, list);
    let in_facet = hooks::faction_facet(w).is_some()
        && w.mobile(id).and_then(|m| m.map) == hooks::faction_facet(w);
    if in_facet {
        hooks::faction_add_properties(w, id, list);
    }
    // TODO: Add the Titles Menu for later Eras
    if data(w, id).is_some_and(|d| d.flag(PlayerFlag::DISPLAY_CHAMPION_TITLE)) {
        let label = tables::champion_title_label(w, id);
        if label > 0 {
            list.add(label);
        }
    }
    if crate::core::ml(w)
        && let Some(followers) = data(w, id).and_then(|d| d.all_followers.as_ref())
        && followers.iter().any(|f| hooks::pet_is_guarding(w, *f))
    {
        list.add(501129); // guarded
    }
}

/// `OnSpeech`
pub fn on_speech(w: &mut World, id: MobileId, e: &mut muo_server::speech_event::SpeechEventArgs) {
    let has_net = w.mobile(id).is_some_and(|m| m.net_state.is_some());
    if has_net && data(w, id).is_some() {
        let now = w.now;
        let from = e.mobile;
        let name = w
            .mobile(from)
            .and_then(|m| (m.ty.name)(w, from))
            .unwrap_or_default();
        if let Some(d) = data_mut(w, id) {
            let log = d.speech_log.get_or_insert_with(VecDeque::new);
            // SpeechLog.Clean(): entries older than 20 minutes go.
            while log
                .front()
                .is_some_and(|x| now - x.created > TimeSpan::from_mins(20))
            {
                log.pop_front();
            }
            log.push_back(SpeechLogEntry {
                from,
                name,
                speech: e.speech.clone(),
                created: now,
            });
        }
    }
}

// ---- context menu ----

pub fn callback_entry(number: i32, f: impl Fn(&mut World, MobileId) + 'static) -> ContextMenuEntry {
    let mut e = ContextMenuEntry::new(number, -1);
    e.on_click = Some(std::rc::Rc::new(
        move |w: &mut World, from: MobileId, _target: EntityId| f(w, from),
    ));
    e
}

/// `GetContextMenuEntries`
pub fn get_context_menu_entries(
    w: &mut World,
    id: MobileId,
    from: MobileId,
    list: &mut Vec<ContextMenuEntry>,
) {
    (MOBILE_BASE.get_context_menu_entries)(w, id, from, list);
    let alive = w.mobile(id).is_some_and(|m| (m.ty.alive)(w, id));
    let house = hooks::house_at(w, id, from);
    if from == id {
        if alive
            && let Some(pack) = w.mobile(id).and_then(|m| m.backpack)
            && (MOBILE_BASE.can_see_item)(w, id, pack)
        {
            list.push(crate::systems::context_menus::entries::open_backpack());
        }
        hooks::quest_context_menu_entries(w, id, list);
        if alive {
            if hooks::insurance_enabled(w) {
                if crate::core::sa(w) {
                    list.push(callback_entry(1114299, |w, m| {
                        hooks::insurance_open_menu(w, m)
                    })); // Open Item Insurance Menu
                }
                list.push(callback_entry(6201, hooks::insurance_toggle)); // Toggle Item Insurance
                if !crate::core::sa(w) {
                    if data(w, id).is_some_and(|d| d.flag(PlayerFlag::AUTO_RENEW_INSURANCE)) {
                        // Cancel Renewing Inventory Insurance
                        list.push(callback_entry(6202, |w, m| {
                            hooks::insurance_cancel_renew(w, m)
                        }));
                    } else {
                        // Auto Renew Inventory Insurance
                        list.push(callback_entry(6200, |w, m| {
                            hooks::insurance_auto_renew(w, m)
                        }));
                    }
                }
            }
            if hooks::ml_quest_enabled(w) {
                list.push(callback_entry(6169, toggle_quest_item)); // Toggle Quest Item
            }
        }
        if let Some(h) = house {
            if alive && h.internalized_vendors > 0 && h.is_owner {
                list.push(callback_entry(6204, get_vendor));
            }
            if h.is_aos_rules && !hooks::in_safe_zone(w, id) {
                // Dueling
                list.push(callback_entry(6207, leave_house));
            }
        }
        if hooks::justice_is_protected(w, id) {
            list.push(callback_entry(6157, cancel_protection));
        }
        if alive {
            list.push(callback_entry(6210, toggle_champion_title_display));
        }
        if crate::core::hs(w) {
            let extended = w
                .mobile(from)
                .and_then(|m| m.net_state)
                .and_then(|ns| w.net_state(ns))
                .is_some_and(|d| d.caps.extended_status());
            if extended {
                // Allow Trades / Refuse Trades
                let refuse = data(w, id).is_some_and(|d| d.flag(PlayerFlag::REFUSE_TRADES));
                list.push(callback_entry(
                    if refuse { 1154112 } else { 1154113 },
                    toggle_trades,
                ));
            }
        }
    } else {
        let in_range_2 = w
            .mobile(from)
            .zip(w.mobile(id))
            .is_some_and(|(a, b)| in_range_pt(a.location, b.location, 2));
        if crate::core::tol(w) && in_range_2 {
            let target = id;
            list.push(callback_entry(1077728, move |w, from| {
                // Trade
                if let Some(ty) = mobile_ty(w, target) {
                    (ty.open_trade)(w, target, from, None);
                }
            }));
        }
        if alive && crate::core::aos(w) {
            let their_party = w.mobile(from).and_then(|m| m.party);
            let our_party = w.mobile(id).and_then(|m| m.party);
            let leader_is_from = |w: &World, p| {
                crate::hooks::world_hooks(w)
                    .and_then(|h| h.party_members)
                    .is_some_and(|f| f(w, from).first() == Some(&from) && p)
            };
            if their_party.is_none() && our_party.is_none() {
                list.push(crate::systems::context_menus::entries::add_to_party());
            } else if their_party.is_some() && leader_is_from(w, true) {
                if our_party.is_none() {
                    list.push(crate::systems::context_menus::entries::add_to_party());
                } else if our_party == their_party {
                    list.push(remove_from_party_entry());
                }
            }
        }
        if let Some(h) = house
            && alive
            && crate::core::aos(w)
            && h.is_aos_rules
            && h.is_friend
        {
            list.push(crate::systems::context_menus::entries::eject_player());
        }
    }
}

/// `CancelProtection`
pub fn cancel_protection(w: &mut World, id: MobileId) {
    if let Some(prot) = hooks::justice_cancel_protection(w, id) {
        let name = w
            .mobile(id)
            .and_then(|m| (m.ty.name)(w, id))
            .unwrap_or_default();
        let prot_name = w
            .mobile(prot)
            .and_then(|m| (m.ty.name)(w, prot))
            .unwrap_or_default();
        let args = format!("{name}\t{prot_name}");
        // The protective relationship between ~1_PLAYER1~ and ~2_PLAYER2~ has been ended.
        messages::send_localized_message(w, prot, 1049371, &args, 0x3B2);
        messages::send_localized_message(w, id, 1049371, &args, 0x3B2);
    }
}

/// `ToggleTrades`
pub fn toggle_trades(w: &mut World, id: MobileId) {
    if let Some(d) = data_mut(w, id) {
        let v = d.flag(PlayerFlag::REFUSE_TRADES);
        d.set_flag(PlayerFlag::REFUSE_TRADES, !v);
    }
}

/// `GetVendor`
pub fn get_vendor(w: &mut World, id: MobileId) {
    let alive = mobile::part2::stats::check_alive(w, id, true);
    let house = hooks::house_at(w, id, id);
    if alive
        && house.is_some_and(|h| h.is_owner && h.internalized_vendors > 0)
        && w.mobile(id).is_some_and(|m| m.net_state.is_some())
    {
        hooks::house_reclaim_vendor_gump(w, id);
    }
}

/// `LeaveHouse`
pub fn leave_house(w: &mut World, id: MobileId) {
    if let Some(h) = hooks::house_at(w, id, id) {
        mobile::part1::lifecycle::set_location(w, id, h.ban_location);
    }
}

/// `ToggleQuestItem`
pub fn toggle_quest_item(w: &mut World, id: MobileId) {
    if !mobile::part2::stats::check_alive(w, id, true) {
        return;
    }
    toggle_quest_item_target(w, id);
}

/// `ToggleQuestItemTarget`
pub fn toggle_quest_item_target(w: &mut World, id: MobileId) {
    if w.mobile(id).is_some_and(|m| m.net_state.is_some()) {
        hooks::ml_quest_close_gumps(w, id);
    }
    muo_server::target::begin_target(
        w,
        id,
        -1,
        false,
        muo_server::target::TargetFlags::NONE,
        toggle_quest_item_callback,
    );
    messages::send_localized_message(w, id, 1072352, "", 0x3B2); // Target the item you wish to toggle Quest Item status on <ESC> to cancel
}

/// `ToggleQuestItem_Callback`
pub fn toggle_quest_item_callback(
    w: &mut World,
    from: MobileId,
    obj: muo_server::target::Targeted,
) {
    if !mobile::part2::stats::check_alive(w, from, true) {
        return;
    }
    let muo_server::target::Targeted::Entity(EntityId::Item(item)) = obj else {
        return;
    };
    let pack = w.mobile(from).and_then(|m| m.backpack);
    let parent = w.item(item).and_then(|i| i.parent);
    if pack.is_none() || parent != pack.map(EntityId::Item) {
        // An item must be in your backpack (and not in a container within) to be toggled as a quest item.
        messages::send_localized_message(w, from, 1074769, "", 0x3B2);
    } else if w.item(item).is_some_and(|i| i.quest_item()) {
        ops::set_quest_item(w, item, false);
        messages::send_localized_message(w, from, 1072354, "", 0x3B2); // You remove Quest Item status from the item
    } else if hooks::ml_quest_mark_item(w, from, item) {
        messages::send_localized_message(w, from, 1072353, "", 0x3B2); // You set the item to Quest Item status
    } else {
        // That item does not match any of your quest criteria
        messages::send_localized_message(w, from, 1072355, "", 0x23);
    }
    toggle_quest_item_target(w, from);
}

/// `ToggleChampionTitleDisplay`
pub fn toggle_champion_title_display(w: &mut World, id: MobileId) {
    if !mobile::part2::stats::check_alive(w, id, true) {
        return;
    }
    let on = data(w, id).is_some_and(|d| d.flag(PlayerFlag::DISPLAY_CHAMPION_TITLE));
    if on {
        messages::send_localized_message(w, id, 1062419, "", 0x23); // You have chosen to hide your monster kill title.
    } else {
        messages::send_localized_message(w, id, 1062418, "", 0x23); // You have chosen to display your monster kill title.
    }
    if let Some(d) = data_mut(w, id) {
        d.set_flag(PlayerFlag::DISPLAY_CHAMPION_TITLE, !on);
    }
    mobile::part2::properties::invalidate_properties(w, id);
}

// ---- damage, death and resurrection ----

/// `OnDamage`
pub fn on_damage(
    w: &mut World,
    id: MobileId,
    amount: i32,
    from: Option<MobileId>,
    will_kill: bool,
) {
    let disrupt_threshold = if !crate::core::aos(w) {
        0
    } else if from.and_then(|f| w.mobile(f)).is_some_and(|f| f.player) {
        18
    } else {
        25
    };
    if amount > disrupt_threshold {
        hooks::bandage_slip(w, id);
    }
    hooks::confidence_stop_regenerating(w, id);
    crate::misc::stamina_system::fatigue_on_damage(w, id, amount);
    hooks::honor_on_damaged(w, id, from, amount);
    (MOBILE_BASE.on_damage)(w, id, amount, from, will_kill);
}

/// `Damage`
pub fn damage(
    w: &mut World,
    id: MobileId,
    amount: i32,
    from: Option<MobileId>,
    inform_mount: bool,
    ignore_evil_omen: bool,
) {
    let mut damage_bonus = 1.0;
    let evil_omen = crate::hooks::hooks(w)
        .and_then(|h| h.evil_omen_end_effect)
        .is_some_and(|f| f(w, id));
    if evil_omen && !ignore_evil_omen {
        damage_bonus += 0.25;
    }
    let mut has_blood_oath = false;
    if let Some(f) = from {
        let protection = hooks::talisman_protection(w, id, f);
        if protection != 0 {
            damage_bonus -= f64::from(protection) / 100.0;
        }
        // Is the attacker attacking the blood oath caster?
        let oath = crate::hooks::hooks(w)
            .and_then(|h| h.blood_oath_of)
            .and_then(|g| g(w, f));
        if oath == Some(id) {
            has_blood_oath = true;
            damage_bonus += 0.2;
        }
    }
    (MOBILE_BASE.damage)(
        w,
        id,
        (f64::from(amount) * damage_bonus) as i32,
        from,
        inform_mount,
        false,
    );

    // If the blood oath caster will die then damage is not reflected back to the attacker
    let dead_pet = w
        .mobile(id)
        .is_some_and(|m| (m.ty.is_dead_bonded_pet)(w, id));
    let alive = w
        .mobile(id)
        .is_some_and(|m| !m.deleted && (m.ty.alive)(w, id));
    if has_blood_oath
        && alive
        && !dead_pet
        && let Some(f) = from
    {
        // Reflect the attacker's original damage back to them, attributed to the caster.
        let reflected = crate::hooks::hooks(w)
            .and_then(|h| h.blood_oath_reflected)
            .map_or(amount, |g| g(amount, 0.0, false));
        if let Some(ty) = mobile_ty(w, f) {
            (ty.damage)(w, f, reflected, Some(id), true, false);
        }
    }
}

/// `Resurrect`
pub fn resurrect(w: &mut World, id: MobileId) {
    let was_alive = w.mobile(id).is_some_and(|m| (m.ty.alive)(w, id));
    (MOBILE_BASE.resurrect)(w, id);
    let alive = w.mobile(id).is_some_and(|m| (m.ty.alive)(w, id));
    if alive
        && !was_alive
        && let Some(robe) = crate::factory::make_item(w, "Server.Items.DeathRobe")
        && !mobile::base::equip_item(w, id, robe)
    {
        muo_server::item::base::delete(w, robe);
    }
}

/// `FindItems_Callback`
pub fn find_items_callback(w: &World, id: MobileId, item: ItemId) -> bool {
    let Some(i) = w.item(item) else { return false };
    let pack = w.mobile(id).and_then(|m| m.backpack);
    !i.deleted()
        && pack.map(EntityId::Item) != i.parent
        && (i.loot_type == muo_server::item::LootType::Blessed
            || (hooks::insurance_feature_enabled(w) && i.insured()))
}

/// `OnBeforeDeath`
pub fn on_before_death(w: &mut World, id: MobileId) -> bool {
    if let Some(ns) = w.mobile(id).and_then(|m| m.net_state) {
        mobile::part2::deps::cancel_all_trades(w, ns);
    }
    mobile::part2::items::drop_holding(w, id);
    // During AOS+, insured/blessed items are moved out of their child containers and put directly into the backpack.
    if crate::core::aos(w)
        && let Some(pack) = w.mobile(id).and_then(|m| m.backpack)
        && w.item(pack).is_some_and(|p| !p.deleted())
    {
        let moving = muo_server::item::enumerable::find_items(w, pack, true, |w, i| {
            find_items_callback(w, id, i)
        });
        for item in moving {
            if let Some(ty) = w.item(pack).map(|p| p.ty) {
                (ty.add_item)(w, pack, item);
            }
        }
    }
    let snapshot = w.mobile(id).map(|m| m.items.clone());
    if let Some(d) = data_mut(w, id) {
        d.equip_snapshot = snapshot;
    }
    hooks::insurance_before_death(w, id);
    hooks::honor_on_killed(w, id);
    (MOBILE_BASE.on_before_death)(w, id)
}

/// `OnDeath`
pub fn on_death(w: &mut World, id: MobileId, c: Option<ItemId>) {
    (MOBILE_BASE.on_death)(w, id, c);
    if let Some(d) = data_mut(w, id) {
        d.equip_snapshot = None;
    }
    props_b::set_hue_mod(w, id, -1);
    props_b::set_name_mod(w, id, None);
    super::set_savage_paint_expiration(w, id, TimeSpan::ZERO);
    super::set_hair_mods(w, id, -1, -1);

    if w.mobile(id).is_some_and(|m| m.flying) {
        props_b::set_flying(w, id, false);
        buff::remove_buff(w, id, BuffIcon::Fly);
    }

    if data(w, id).is_some_and(|d| !d.perma_flags.is_empty()) {
        if let Some(d) = data_mut(w, id) {
            d.perma_flags.clear();
        }
        if let Some(corpse) = c {
            hooks::corpse_set_criminal(w, corpse);
        }
        if hooks::stealing_classic_mode(w)
            && let Some(ty) = mobile_ty(w, id)
        {
            (ty.set_criminal)(w, id, true);
        }
    }

    let murderer = w.mobile(id).is_some_and(|m| (m.ty.murderer)(w, id));
    let next_award = data(w, id).map_or(DateTime::MAX, |d| d.next_just_award);
    if murderer && w.now >= next_award {
        let mut m = mobile::part2::damage::find_most_recent_damager(w, id, false);
        if let Some(killer) = m
            && let Some(master) = hooks::creature_master(w, killer)
        {
            m = Some(master);
        }
        if let Some(pm) = m
            && pm != id
            && data(w, pm).is_some()
        {
            let game_time = super::game_time(w, id);
            let mut points = (game_time.total_seconds() * 4.0).sqrt() as i32;
            points *= 5;
            points += (f64::from(muo_server::skills::total(w, id)) / 250.0).powi(2) as i32;
            let (awarded, gained_path) = hooks::virtue_award_justice(w, pm, points);
            if awarded {
                if gained_path {
                    messages::send_localized_message(w, pm, 1049367, "", 0x3B2); // You have gained a path in Justice!
                } else {
                    messages::send_localized_message(w, pm, 1049363, "", 0x3B2); // You have gained in Justice.
                }
                effects::fixed_particles(
                    w,
                    pm,
                    0x375A,
                    9,
                    20,
                    5027,
                    muo_server::effects::EffectLayer::Waist,
                );
                effects::play_sound(w, pm, 0x1F7);
                let next = w.now + TimeSpan::from_mins_f64(f64::from(points) / 3.0);
                if let Some(d) = data_mut(w, id) {
                    d.next_just_award = next;
                }
            }
        }
    }

    hooks::insurance_on_death(w, id);

    let mut killer = mobile::part2::damage::find_most_recent_damager(w, id, true);
    if let Some(k) = killer
        && let Some(master) = hooks::creature_master(w, k)
    {
        killer = Some(master);
    }

    let duel = hooks::duel_state(w, id);
    if young(w, id) && duel.is_none() && young_death_teleport(w, id) {
        timer::start(
            w,
            TimeSpan::from_millis(2500),
            TimeSpan::ZERO,
            1,
            move |w| send_young_death_notice(w, id),
        );
    }

    if duel.is_none_or(|d| !d.registered || !d.started || d.eliminated) {
        hooks::faction_handle_death(w, id, killer);
    }
    hooks::guild_handle_death(w, id, killer);

    let doomed: Vec<BuffIcon> = data(w, id).map_or_else(Vec::new, |d| {
        d.buff_table
            .values()
            .filter(|b| !b.retain_through_death)
            .map(|b| b.id)
            .collect()
    });
    for icon in doomed {
        buff::remove_buff(w, id, icon);
    }

    super::player_death_event(w, id);
}

/// `MutateSpeech`
pub fn mutate_speech(
    w: &mut World,
    id: MobileId,
    hears: &mut Vec<MobileId>,
    text: &mut String,
    context: &mut mobile::SpeechContext,
) -> bool {
    if w.mobile(id).is_some_and(|m| (m.ty.alive)(w, id)) {
        return false;
    }
    if crate::core::ml(w) && muo_server::skills::value_ref(w, id, SkillName::SpiritSpeak) >= 100.0 {
        return false;
    }
    if crate::core::aos(w) {
        for h in hears.iter() {
            if *h != id && muo_server::skills::value_ref(w, *h, SkillName::SpiritSpeak) >= 100.0 {
                return false;
            }
        }
    }
    (MOBILE_BASE.mutate_speech)(w, id, hears, text, context)
}

/// `DoSpeech`
pub fn do_speech(
    w: &mut World,
    id: MobileId,
    text: &str,
    keywords: &[i32],
    ty: MessageType,
    hue: i32,
) {
    if crate::core::se(w) && (ty == MessageType::GUILD || ty == MessageType::ALLIANCE) {
        let guild = w.mobile(id).and_then(|m| m.guild);
        if guild.is_none() {
            messages::send_localized_message(w, id, 1063142, "", 0x3B2); // You are not in a guild!
        } else if ty == MessageType::ALLIANCE {
            if hooks::guild_in_alliance(w, id) {
                hooks::guild_alliance_chat(w, id, text);
                send_to_staff_message(w, id, &format!("[Alliance]: {text}"));
                if let Some(d) = data_mut(w, id) {
                    d.alliance_message_hue = hue;
                }
            } else {
                messages::send_localized_message(w, id, 1071020, "", 0x3B2); // You are not in an alliance!
            }
        } else {
            if let Some(d) = data_mut(w, id) {
                d.guild_message_hue = hue;
            }
            hooks::guild_chat(w, id, text);
            send_to_staff_message(w, id, &format!("[Guild]: {text}"));
        }
    } else {
        (MOBILE_BASE.do_speech)(w, id, text, keywords, ty, hue);
    }
}

/// `SendToStaffMessage`
pub fn send_to_staff_message(w: &mut World, from: MobileId, text: &str) {
    let Some(f) = w.mobile(from) else { return };
    let (serial, body, hue, language, name, level) = (
        f.serial,
        muo_server::mobile::part1::props_b::body(w, from),
        f.speech_hue,
        f.language.clone(),
        (f.ty.name)(w, from).unwrap_or_default(),
        f.access_level,
    );
    for ns in mobile::part2::enumerators::get_clients_in_range(w, from, 8) {
        let Some(mob) = w.net_state(ns).and_then(|d| d.mobile) else {
            continue;
        };
        let Some(access) = w.mobile(mob).map(|m| m.access_level) else {
            continue;
        };
        if access >= AccessLevel::GameMaster && access > level {
            muo_server::packets::message::send_message(
                w,
                ns,
                serial,
                body.0,
                muo_server::packets::message::MessageType::REGULAR,
                hue,
                3,
                false,
                language.as_deref(),
                &name,
                text,
            );
        }
    }
}

// ---- deletion ----

/// `OnAfterDelete`
pub fn on_after_delete(w: &mut World, id: MobileId) {
    (MOBILE_BASE.on_after_delete)(w, id);
    hooks::faction_remove_member(w, id);
    super::player_deleted_event(w, id);
}

/// `OnDelete`
pub fn on_delete(w: &mut World, id: MobileId) {
    hooks::honor_cancel(w, id);
    hooks::ammo_recovery_forget(w, id);

    let stabled = data_mut(w, id).and_then(|d| d.stabled.take());
    for s in stabled.into_iter().flatten() {
        if let Some(ty) = mobile_ty(w, s) {
            (ty.delete)(w, s);
        }
    }
    let followers: Vec<MobileId> = data(w, id)
        .and_then(|d| d.all_followers.as_ref())
        .map_or_else(Vec::new, |f| f.iter().copied().collect());
    // Releasing or deleting a follower removes it from _allFollowers: act on a copy.
    for f in followers {
        // Escorts and hirelings notice a deleted master and walk off on their own.
        match hooks::pet_goes_with_owner(w, f) {
            Some(true) => {
                if let Some(ty) = mobile_ty(w, f) {
                    (ty.delete)(w, f);
                }
            }
            // An unbonded pet goes wild and despawns on the abandoned-pet timer.
            Some(false) => hooks::pet_release(w, f),
            None => {}
        }
    }
}

// ---- Young ----

/// `CheckYoungProtection`, up to the message: whether `from` is held back by the Young status.
pub fn young_protects_from(w: &World, id: MobileId, from: MobileId) -> bool {
    young(w, id)
        && !hooks::region_not_young_protected(w, id)
        && !hooks::creature_ignores_young_protection(w, from)
        && !hooks::quest_ignore_young_protection(w, id, from)
}

/// `CheckYoungProtection`
pub fn check_young_protection(w: &mut World, id: MobileId, from: MobileId) -> bool {
    if !young_protects_from(w, id, from) {
        return false;
    }
    let now = w.now;
    if data(w, id).is_some_and(|d| now - d.last_young_message > TimeSpan::from_mins(1)) {
        if let Some(d) = data_mut(w, id) {
            d.last_young_message = now;
        }
        // A monster looks at you menacingly but does not attack.
        // You would be under attack now if not for your status as a new citizen of Britannia.
        messages::send_localized_message(w, id, 1019067, "", 0x3B2);
    }
    true
}

const TRAMMEL_DEATH_DESTINATIONS: [(i32, i32, i32); 15] = [
    (1481, 1612, 20),
    (2708, 2153, 0),
    (2249, 1230, 0),
    (5197, 3994, 37),
    (1412, 3793, 0),
    (3688, 2232, 20),
    (2578, 604, 0),
    (4397, 1089, 0),
    (5741, 3218, -2),
    (2996, 3441, 15),
    (624, 2225, 0),
    (1916, 2814, 0),
    (2929, 854, 0),
    (545, 967, 0),
    (3665, 2587, 0),
];
const ILSHENAR_DEATH_DESTINATIONS: [(i32, i32, i32); 9] = [
    (1216, 468, -13),
    (723, 1367, -60),
    (745, 725, -28),
    (281, 1017, 0),
    (986, 1011, -32),
    (1175, 1287, -30),
    (1533, 1341, -3),
    (529, 217, -44),
    (1722, 219, 96),
];
const MALAS_DEATH_DESTINATIONS: [(i32, i32, i32); 2] = [(2079, 1376, -70), (944, 519, -71)];
const TOKUNO_DEATH_DESTINATIONS: [(i32, i32, i32); 3] =
    [(1166, 801, 27), (782, 1228, 25), (268, 624, 15)];

/// `YoungDeathTeleport`
pub fn young_death_teleport(w: &mut World, id: MobileId) -> bool {
    use muo_server::region::is_part_of_name;
    let Some(m) = w.mobile(id) else { return false };
    let region = m.region;
    if hooks::in_jail_region(w, id)
        || is_part_of_name(w, region, "Samurai start location", true)
        || is_part_of_name(w, region, "Ninja start location", true)
        || is_part_of_name(w, region, "Ninja cave", true)
    {
        return false;
    }
    let (loc, map) = match hooks::dungeon_entrance(w, id) {
        Some((entrance, map)) => (entrance, map),
        None => (m.location, m.map),
    };
    use muo_server::map::MapId;
    let list: &[(i32, i32, i32)] = match map {
        Some(MapId::TRAMMEL) => &TRAMMEL_DEATH_DESTINATIONS,
        Some(MapId::ILSHENAR) => &ILSHENAR_DEATH_DESTINATIONS,
        Some(MapId::MALAS) => &MALAS_DEATH_DESTINATIONS,
        Some(MapId::TOKUNO) => &TOKUNO_DEATH_DESTINATIONS,
        _ => return false,
    };
    let mut dest = Point3D::new(0, 0, 0);
    let mut sq_distance = i32::MAX;
    for &(x, y, z) in list {
        let (width, height) = (loc.x - x, loc.y - y);
        let cur = width * width + height * height;
        if cur < sq_distance {
            dest = Point3D::new(x, y, z);
            sq_distance = cur;
        }
    }
    let Some(map) = map else { return false };
    if let Some(ty) = mobile_ty(w, id) {
        (ty.move_to_world)(w, id, dest, map);
    }
    true
}

/// `SendYoungDeathNotice`
pub fn send_young_death_notice(w: &mut World, id: MobileId) {
    if w.mobile(id).is_some_and(|m| m.net_state.is_some()) {
        hooks::young_death_notice_gump(w, id);
    }
}

// ---- flying ----

/// `ToggleFlying`
pub fn toggle_flying(w: &mut World, id: MobileId) {
    if super::race_of(w, id) != RACE_GARGOYLE {
        return;
    }
    if w.mobile(id).is_some_and(|m| m.flying) {
        mobile::base::freeze(w, id, TimeSpan::from_secs(1));
        mobile::base::animate(w, id, 61, 10, 1, true, false, 0);
        props_b::set_flying(w, id, false);
        buff::remove_buff(w, id, BuffIcon::Fly);
        messages::send_message(w, id, "You have landed.");
        hooks::dismount(w, id);
        return;
    }
    let ty = mount_block_reason(w, id);
    let alive = w.mobile(id).is_some_and(|m| (m.ty.alive)(w, id));
    let (body_mod, hits) = w.mobile(id).map_or((0, 0), |m| (m.body_mod.0, m.hits));
    let is_body_mod = body_mod != 0;
    if !alive {
        messages::send_localized_message(w, id, 1113082, "", 0x3B2); // You may not fly while dead.
    } else if is_body_mod && !(body_mod == 666 || body_mod == 667) {
        messages::send_localized_message(w, id, 1112453, "", 0x3B2); // You can't fly in your current form!
    } else if ty != BlockMountType::None {
        match ty {
            BlockMountType::Dazed => messages::send_localized_message(w, id, 1112457, "", 0x3B2), // You are still too dazed to fly.
            BlockMountType::BolaRecovery => {
                messages::send_localized_message(w, id, 1112455, "", 0x3B2)
            } // You cannot fly while recovering from a bola throw.
            BlockMountType::DismountRecovery => {
                messages::send_localized_message(w, id, 1112456, "", 0x3B2)
            } // You cannot fly while recovering from a dismount maneuver.
            BlockMountType::None => {}
        }
    } else if hits < 25 {
        // TODO confirm
        messages::send_localized_message(w, id, 1112454, "", 0x3B2); // You must heal before flying.
    } else {
        // `Flying` is false here: stop a running FlySpell and cast a new one (no message?).
        hooks::fly_cast(w, id);
    }
}

// ---- dropped items ----

/// `OnDroppedItemToWorld`
pub fn on_dropped_item_to_world(
    w: &mut World,
    id: MobileId,
    item: ItemId,
    location: Point3D,
) -> bool {
    if !(MOBILE_BASE.on_dropped_item_to_world)(w, id, item, location) {
        return false;
    }
    let map = w.mobile(id).and_then(|m| m.map);
    if crate::core::aos(w) {
        for m in muo_server::map::enumerate::mobiles_at(w, map, location.x, location.y) {
            if let Some(o) = w.mobile(m)
                && o.location.z >= location.z
                && o.location.z < location.z + 16
                && (!o.hidden || o.access_level == AccessLevel::Player)
            {
                return false;
            }
        }
    }
    let Some(bi) = w.item(item).and_then(|i| i.bounce.clone()) else {
        return true;
    };
    if let Some(ids) = hooks::furniture_flip_ids(w, item) {
        let old_world_loc = bi.world_loc;
        if old_world_loc.x != location.x || old_world_loc.y != location.y {
            let dir = super::get_direction4(old_world_loc, location);
            let cur = w.item(item).map_or(0, |i| i.item_id);
            let new_id = match ids.len() {
                2 => match dir {
                    d if d == Direction::NORTH || d == Direction::SOUTH => ids[0],
                    d if d == Direction::EAST || d == Direction::WEST => ids[1],
                    _ => cur,
                },
                4 => match dir {
                    d if d == Direction::SOUTH => ids[0],
                    d if d == Direction::EAST => ids[1],
                    d if d == Direction::NORTH => ids[2],
                    d if d == Direction::WEST => ids[3],
                    _ => cur,
                },
                _ => cur,
            };
            muo_server::item::base::set_item_id(w, item, new_id);
        }
    }
    true
}

// ---- mount block ----

/// `SetMountBlock`
pub fn set_mount_block(
    w: &mut World,
    id: MobileId,
    ty: BlockMountType,
    duration: Option<TimeSpan>,
    dismount: bool,
) {
    if dismount {
        if props_b::mount(w, id).is_some() {
            hooks::dismount(w, id);
        } else if hooks::animal_form_under(w, id) {
            hooks::animal_form_remove_context(w, id);
        }
    }
    let now = w.now;
    let replace = match data(w, id).and_then(|d| d.mount_block.as_ref()) {
        None => true,
        Some(b) => {
            !check_block(w, b)
                || block_expiration(w, b)
                    < duration.map_or(DateTime::MAX, |d| {
                        now.checked_add(d).unwrap_or(DateTime::MAX)
                    })
        }
    };
    if replace {
        remove_mount_block(w, id);
        let timer = match duration {
            Some(d) => timer::start(w, d, TimeSpan::ZERO, 1, move |w| remove_mount_block(w, id)),
            None => TimerHandle::default(),
        };
        if let Some(d) = data_mut(w, id) {
            d.mount_block = Some(MountBlock { ty, timer });
        }
    }
}

/// `MountBlock.CheckBlock()`.
pub fn check_block(w: &World, b: &MountBlock) -> bool {
    b.timer == TimerHandle::default() || b.timer.running(w)
}

/// `MountBlock.Expiration` (`_timerToken.Next`).
pub fn block_expiration(w: &World, b: &MountBlock) -> DateTime {
    if b.timer == TimerHandle::default() {
        DateTime::MIN
    } else {
        b.timer.next(w)
    }
}

/// `MountBlock.RemoveBlock`.
pub fn remove_mount_block(w: &mut World, id: MobileId) {
    let block = data_mut(w, id).and_then(|d| d.mount_block.take());
    if let Some(mut b) = block {
        b.timer.cancel(w);
    }
}

/// `MountBlockReason`
pub fn mount_block_reason(w: &World, id: MobileId) -> BlockMountType {
    data(w, id)
        .and_then(|d| d.mount_block.as_ref())
        .filter(|b| check_block(w, b))
        .map_or(BlockMountType::None, |b| b.ty)
}

// ---- pets ----

/// `CheckPets`
pub fn check_pets(w: &mut World) {
    let players: Vec<MobileId> = w
        .mobiles
        .keys()
        .copied()
        .filter(|m| !w.detaching.contains(&m.0))
        .collect();
    for pm in players {
        let Some(d) = data(w, pm) else { continue };
        let followers = d.all_followers.as_ref().map_or(0, IndexSet::len);
        if followers == 0 {
            continue;
        }
        let mut auto_stabled = d.auto_stabled.as_ref().map_or(0, IndexSet::len);
        if props_b::mounted(w, pm) && !mount_is_ethereal(w, pm) {
            auto_stabled += 1;
        }
        if followers > auto_stabled {
            auto_stable_pets(w, pm);
        }
    }
}

pub fn mount_is_ethereal(w: &mut World, id: MobileId) -> bool {
    props_b::mount(w, id)
        .is_some_and(|m| crate::factory::mobile_is(w, m, "Server.Mobiles.EtherealMount"))
        || w.mobile(id)
            .and_then(|m| m.mount_item)
            .is_some_and(|i| crate::factory::item_is(w, i, "Server.Mobiles.EtherealMount"))
}

/// `AutoStablePets`
pub fn auto_stable_pets(w: &mut World, id: MobileId) {
    let followers: Vec<MobileId> = data(w, id)
        .and_then(|d| d.all_followers.as_ref())
        .map_or_else(Vec::new, |f| f.iter().copied().collect());
    if !crate::core::se(w) || followers.is_empty() {
        return;
    }
    for follower in followers {
        let Some(pet) = hooks::pet_info(w, follower) else {
            continue;
        };
        if !pet.controlled || !pet.has_control_master {
            continue;
        }
        if pet.summoned {
            if pet.map != w.mobile(id).and_then(|m| m.map) {
                hooks::pet_dismiss_summoned(w, follower);
            }
            continue;
        }
        if pet.has_rider || pet.pack_animal_with_items || pet.escortable {
            continue;
        }
        hooks::pet_auto_stable(w, follower, id);
        if let Some(d) = data_mut(w, id) {
            d.stabled.get_or_insert_with(IndexSet::new).insert(follower);
            d.auto_stabled
                .get_or_insert_with(IndexSet::new)
                .insert(follower);
        }
    }
}

/// `ClaimAutoStabledPets`
pub fn claim_auto_stabled_pets(w: &mut World, id: MobileId) {
    let auto: Vec<MobileId> = data(w, id)
        .and_then(|d| d.auto_stabled.as_ref())
        .map_or_else(Vec::new, |s| s.iter().copied().collect());
    if !crate::core::se(w) || auto.is_empty() {
        return;
    }
    if !w.mobile(id).is_some_and(|m| (m.ty.alive)(w, id)) {
        // Your pet was unable to join you while you are a ghost.  Please re-login once you have ressurected to claim your pets.
        messages::send_localized_message(w, id, 1076251, "", 0x3B2);
        return;
    }
    for stabled in auto {
        let Some(pet) = hooks::pet_info(w, stabled) else {
            continue;
        };
        if pet.deleted {
            hooks::pet_clear_stabled(w, stabled);
            if let Some(d) = data_mut(w, id) {
                d.stabled.as_mut().map(|s| s.shift_remove(&stabled));
            }
            continue;
        }
        let (followers, max) = w
            .mobile(id)
            .map_or((0, 0), |m| (m.followers, m.followers_max));
        if followers + pet.control_slots <= max {
            hooks::pet_claim(w, stabled, id);
            if let Some(d) = data_mut(w, id) {
                d.stabled.as_mut().map(|s| s.shift_remove(&stabled));
            }
        } else {
            // ~1_NAME~ remained in the stables because you have too many followers.
            let name = w
                .mobile(stabled)
                .and_then(|m| (m.ty.name)(w, stabled))
                .unwrap_or_default();
            messages::send_localized_message(w, id, 1049612, &name, 0x3B2);
        }
    }
    if let Some(d) = data_mut(w, id) {
        d.auto_stabled = None;
    }
}

// ---- macros ----

/// `TargetedSkillUse`
pub fn targeted_skill_use(w: &mut World, from: MobileId, target: Option<EntityId>, skill_id: i32) {
    let Some(target) = target else { return };
    if let Some(m) = w.mobile_mut(from) {
        m.target_locked = true;
    }
    if skill_id == 35 {
        hooks::animal_taming_disable_message(w, true);
    }
    if let Some(ty) = mobile_ty(w, from)
        && (ty.use_skill_id)(w, from, skill_id)
        && let Some(t) = w.mobile(from).and_then(|m| m.target)
    {
        muo_server::target::invoke(w, t, from, muo_server::target::Targeted::Entity(target));
    }
    if skill_id == 35 {
        hooks::animal_taming_disable_message(w, false);
    }
    if let Some(m) = w.mobile_mut(from) {
        m.target_locked = false;
    }
}

/// `EquipMacro`
pub fn equip_macro(w: &mut World, m: MobileId, list: &[muo_primitives::Serial]) {
    if data(w, m).is_none() || !w.mobile(m).is_some_and(|x| (x.ty.alive)(w, m)) {
        return;
    }
    let Some(pack) = props_b::backpack(w, m) else {
        return;
    };
    for serial in list {
        let found = w
            .item(pack)
            .and_then(|p| p.items.iter().copied().find(|i| i.0 == *serial));
        let Some(item) = found else { continue };
        let layer = w.item(item).map_or(Layer::Invalid, |i| i.layer);
        let to_move = mobile::part2::items::find_item_on_layer(w, m, layer);
        if let Some(to_move) = to_move {
            ops::internalize(w, to_move);
            if !mobile::base::equip_item(w, m, item) {
                mobile::base::equip_item(w, m, to_move);
            } else if let Some(ty) = w.item(pack).map(|p| p.ty) {
                (ty.drop_item)(w, pack, to_move);
            }
        } else {
            mobile::base::equip_item(w, m, item);
        }
    }
}

/// `UnequipMacro`
pub fn unequip_macro(w: &mut World, m: MobileId, layers: &[Layer]) {
    if data(w, m).is_none() || !w.mobile(m).is_some_and(|x| (x.ty.alive)(w, m)) {
        return;
    }
    let Some(pack) = props_b::backpack(w, m) else {
        return;
    };
    let eq = w.mobile(m).map_or_else(Vec::new, |x| x.items.clone());
    for item in eq.into_iter().rev() {
        if w.item(item).is_some_and(|i| layers.contains(&i.layer)) {
            mobile::part2::deps::container_try_drop_item(w, pack, m, item, false);
        }
    }
}

// ---- the enemy of one type ----

/// `EnemyOfOneType.set`
pub fn set_enemy_of_one_type(
    w: &mut World,
    id: MobileId,
    value: Option<&'static mobile::MobileType>,
) {
    let old = data(w, id).and_then(|d| d.enemy_of_one_type);
    let same = match (old, value) {
        (None, None) => true,
        (Some(a), Some(b)) => std::ptr::eq(a, b),
        _ => false,
    };
    if same {
        return;
    }
    if let Some(d) = data_mut(w, id) {
        d.enemy_of_one_type = value;
    }
    // DeltaEnemies: resend the movement of every mobile of the old or new type within 18 tiles.
    for m in mobile::part2::enumerators::get_mobiles_in_range(w, id, 18) {
        let Some(t) = w.mobile(m).map(|x| x.ty) else {
            continue;
        };
        let is = |x: Option<&'static mobile::MobileType>| x.is_some_and(|x| std::ptr::eq(x, t));
        if (is(old) || is(value))
            && let Some(ns) = w.mobile(m).and_then(|x| x.net_state)
        {
            muo_server::packets::mobile::send_mobile_moving_notoriety(w, ns, id, m);
        }
    }
}

/// `Utility.InRange(a, b, range)`.
fn in_range_pt(a: Point3D, b: Point3D, range: i32) -> bool {
    muo_primitives::utility::in_range(a.x, a.y, b.x, b.y, range)
}

/// `RemoveFromPartyEntry`: `Party.Get(from)` removes the target (the party engine's `party_remove`).
fn remove_from_party_entry() -> ContextMenuEntry {
    let mut e = ContextMenuEntry::new(198, 12);
    e.on_click = Some(std::rc::Rc::new(
        |w: &mut World, from: MobileId, target: EntityId| {
            if let EntityId::Mobile(target) = target {
                hooks::party_remove_member(w, from, target);
            }
        },
    ));
    e
}
