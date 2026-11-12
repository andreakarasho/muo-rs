//! Veteran rewards: AnkhOfSacrifice, MiningCart, DecorativeShield. See `vet.rs` for the
//! shared parts.
//!
//! The ankh's `AnkhResurrectGump` is [`SpecialHooks::ankh_resurrect_gump`] with [`ankh_response_prelude`] for the gump to
//! call; the component's `ResurrectEntry` tests `target is AnkhOfSacrificeAddon` against the component, so
//! it never does anything.

use std::rc::Rc;

use muo_content::gumps::base::{
    DynamicGumpLogic, DynamicGumpObj, RelayInfo, StaticGumpLogic, StaticGumpObj,
};
use muo_content::gumps::builder::{DynamicGumpBuilder, StaticGumpBuilder};
use muo_content::gumps::{GumpButtonType, send_gump_to_mobile};
use muo_content::items::addon::{
    self, ADDON_COMPONENT, AddonClass, BASE_ADDON, BASE_ADDON_CLASS, BASE_ADDON_DEED, DeedClass,
};
use muo_content::items::house_deps::{self, SimpleAddon};

use muo_primitives::{DateTime, Point3D, TimeSpan, utility};
use muo_server::World;
use muo_server::context_menu_entry::ContextMenuEntry;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, deps, ops};
use muo_server::map::MapId;
use muo_server::net_state::NetStateId;
use muo_server::packets::message::MessageType;
use muo_server::target::Targeted;
use muo_server::type_registry::TypeRegistry;

use super::hooks::hooks;
use super::util::{in_pack, msg, overhead};
use super::vet::*;
use crate::simple::leaf;

// ---- AnkhOfSacrifice ----

/// `AnkhOfSacrificeComponent.Resurrect(m, ankh)`.
pub fn ankh_resurrect(w: &mut World, m: MobileId, ankh: ItemId) {
    if !muo_content::mx::in_range_point(w, m, ops::get_world_location(w, ankh), 2) {
        msg(w, m, 500446); // That is too far away.
    } else if muo_content::mx::alive(w, m) {
        msg(w, m, 1060197); // You are not dead, and thus cannot be resurrected!
    } else if muo_content::player_mobile::data(w, m).is_some_and(|d| d.ankh_next_use > w.now) {
        let next = muo_content::player_mobile::data(w, m).map_or(w.now, |d| d.ankh_next_use);
        let delay = next.since(w.now);
        if delay.total_minutes() > 0.0 {
            // You must wait ~1_minutes~ minutes before you can use this item.
            muo_content::mx::msg_args(
                w,
                m,
                1079265,
                &format!("{}", delay.total_minutes().round_ties_even() as i64),
            );
        } else {
            // You must wait ~1_seconds~ seconds before you can use this item.
            muo_content::mx::msg_args(
                w,
                m,
                1079263,
                &format!("{}", delay.total_seconds().round_ties_even() as i64),
            );
        }
    } else if let Some(f) = hooks(w).and_then(|h| h.ankh_resurrect_gump) {
        f(w, m);
    }
}

/// `AnkhResurrectGump.OnResponse` before `base.OnResponse`: false when the response ends there (button not 1 or 2,
/// or no room to stand), else `AnkhNextUse = Now + 1 hour` and the base goes on.
pub fn ankh_response_prelude(w: &mut World, from: MobileId, button: i32) -> bool {
    if button != 1 && button != 2 {
        return false;
    }
    let (loc, map) = w
        .mobile(from)
        .map_or((Point3D::default(), None), |m| (m.location, m.map));
    let fits = map.is_some_and(|map| {
        muo_server::map::fit::can_fit(w, map, loc.x, loc.y, loc.z, 16, false, false, true)
    });
    if !fits {
        msg(w, from, 502391); // Thou can not be resurrected there!
        return false;
    }
    let until = w.now + TimeSpan::from_hours(1);
    if let Some(d) = muo_content::player_mobile::data_mut(w, from) {
        d.ankh_next_use = until;
    }
    true
}

fn ankh_component_entries(
    w: &mut World,
    id: ItemId,
    from: MobileId,
    list: &mut Vec<ContextMenuEntry>,
) {
    (ADDON_COMPONENT.get_context_menu_entries)(w, id, from, list);
    if let Some(k) = crate::construction::paired::karma_lock_entry(w, from) {
        list.push(k);
    }
    let mut e = ContextMenuEntry::new(6195, 2);
    // `target is AnkhOfSacrificeAddon { Deleted: false }`: the target is the component, never the addon.
    e.on_click = Some(Rc::new(|_, _, _| {}));
    list.push(e);
}

fn ankh_component_new(w: &mut World, item_id: i32) -> ItemId {
    addon::new_component(w, &ANKH_OF_SACRIFICE_COMPONENT, item_id)
}

leaf! {
    /// `Server.Items.AnkhOfSacrificeComponent`.
    ANKH_OF_SACRIFICE_COMPONENT = "Server.Items.AnkhOfSacrificeComponent" : ADDON_COMPONENT, v0;
    force_show_properties: |w, _| w.core.opl_enabled,
    label_number: |_, _| 1027772, // Ankh of Sacrifice
    get_context_menu_entries: ankh_component_entries,
}

/// `AnkhOfSacrificeAddon(east)`.
pub fn ankh_of_sacrifice_addon_new(w: &mut World, east: bool) -> ItemId {
    let id = addon::new_addon(w, &ANKH_OF_SACRIFICE_ADDON);
    let rows: [(i32, i32, i32); 6] = if east {
        [
            (0x1D98, 0, 0),
            (0x1D97, 0, 1),
            (0x1CD6, 1, 0),
            (0x1CD4, 1, 1),
            (0x1CD0, 2, 0),
            (0x1CCE, 2, 1),
        ]
    } else {
        [
            (0x1E5D, 0, 0),
            (0x1E5C, 1, 0),
            (0x1CD2, 0, 1),
            (0x1CD8, 1, 1),
            (0x1CCD, 0, 2),
            (0x1CCE, 1, 2),
        ]
    };
    for (item_id, x, y) in rows {
        let c = ankh_component_new(w, item_id);
        addon::add_component(w, id, c, x, y, 0);
    }
    id
}

fn ankh_deed_of(w: &mut World, a: ItemId) -> ItemId {
    let d = ankh_of_sacrifice_deed_new(w);
    let r = is_reward(w, a);
    set_reward(w, d, r);
    d
}

static ANKH_CLASS: AddonClass = AddonClass {
    deed_of: Some(ankh_deed_of),
    ..BASE_ADDON_CLASS
};

rtype! {
    /// `Server.Items.AnkhOfSacrificeAddon` (`AnkhOfSacrificeAddon(bool east)` only).
    ANKH_OF_SACRIFICE_ADDON = "Server.Items.AnkhOfSacrificeAddon" : BASE_ADDON;
    construct: None,
    family: Some(&ANKH_CLASS),
    handles_on_movement: |_, _| true,
    on_movement: |w, id, m, old| {
        let loc = w.item(id).map_or_else(Point3D::default, |i| i.location);
        let mloc = w.mobile(m).map_or_else(Point3D::default, |x| x.location);
        if !muo_content::mx::alive(w, m)
            && utility::in_range(loc.x, loc.y, mloc.x, mloc.y, 1)
            && !utility::in_range(loc.x, loc.y, old.x, old.y, 1)
            && w.mobile(m).is_some_and(|x| x.player)
        {
            ankh_resurrect(w, m, id);
        }
        (BASE_ADDON.on_movement)(w, id, m, old);
    },
}

/// `AnkhOfSacrificeDeed(isRewardItem = false)`.
pub fn ankh_of_sacrifice_deed_new(w: &mut World) -> ItemId {
    let id = addon::new_deed(w, &ANKH_OF_SACRIFICE_DEED);
    ops::set_loot_type(w, id, muo_server::item::LootType::Blessed);
    id
}

static ANKH_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, deed| {
        let east = deed_state(w, deed).east;
        let a = ankh_of_sacrifice_addon_new(w, east);
        let r = is_reward(w, deed);
        set_reward(w, a, r);
        a
    },
    select_direction: None,
};

rtype! {
    /// `Server.Items.AnkhOfSacrificeDeed`.
    ANKH_OF_SACRIFICE_DEED = "Server.Items.AnkhOfSacrificeDeed" : BASE_ADDON_DEED;
    construct: Some(ankh_of_sacrifice_deed_new),
    family: Some(&ANKH_DEED_CLASS),
    label_number: |_, _| 1080397, // Deed For An Ankh Of Sacrifice
    on_double_click: |w, id, from| {
        if reward_blocked(w, from, id) {
            return;
        }
        if in_pack(w, id, from) {
            if let Some(f) = hooks(w).and_then(|h| h.reward_option_gump) {
                f(w, from, id);
            }
        } else {
            msg(w, from, 1062334); // This item must be in your backpack to be used.
        }
    },
    get_properties: |w, id, list| {
        (BASE_ADDON_DEED.get_properties)(w, id, list);
        if is_reward(w, id) {
            list.add(1080457); // 10th Year Veteran Reward
        }
    },
}

// ---- MiningCart ----

/// `MiningCart._cartType`, `_gems`, `_ore` and `_lastResourceTime`.
#[derive(Clone, Copy, Debug)]
pub struct CartData {
    pub cart_type: i32,
    pub gems: i32,
    pub ore: i32,
    pub last_resource_time: DateTime,
}

fn cart(w: &World, id: ItemId) -> CartData {
    w.item(id)
        .and_then(|i| i.ext.get::<CartData>())
        .copied()
        .unwrap_or(CartData {
            cart_type: 100,
            gems: 0,
            ore: 0,
            last_resource_time: DateTime::MIN,
        })
}

fn set_cart(w: &mut World, id: ItemId, d: CartData) {
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(d);
    }
}

/// `MiningCart.GiveResources()`.
pub fn give_resources(w: &mut World, id: ItemId) {
    let mut d = cart(w, id);
    let amount = w.now.since(d.last_resource_time).days();
    if amount <= 0 {
        return;
    }
    match d.cart_type {
        100 | 101 => d.ore = 100.min(d.ore + amount * 10),
        102 | 103 => d.gems = 50.min(d.gems + amount * 5),
        _ => {}
    }
    d.last_resource_time += TimeSpan::from_days(i64::from(amount));
    set_cart(w, id, d);
}

/// `MiningCart(type)`.
pub fn mining_cart_new(w: &mut World, kind: i32) -> ItemId {
    let id = addon::new_addon(w, &MINING_CART);
    let plain = |w: &mut World, id: ItemId, rows: &[(i32, i32, i32, i32)]| {
        for &(g, x, y, z) in rows {
            let c = addon::new_component(w, &ADDON_COMPONENT, g);
            addon::add_component(w, id, c, x, y, z);
        }
    };
    let gem_body = |w: &mut World, id: ItemId, rows: &[(i32, i32, i32)]| {
        for &(g, x, y) in rows {
            let c =
                addon::new_localized_component(w, &addon::LOCALIZED_ADDON_COMPONENT, g, 1080388);
            addon::add_component(w, id, c, x, y, 0);
        }
    };
    match kind {
        100 => plain(
            w,
            id,
            &[(0x1A83, 0, 0, 0), (0x1A82, 0, 1, 0), (0x1A86, 0, -1, 0)],
        ),
        101 => plain(
            w,
            id,
            &[(0x1A88, 0, 0, 0), (0x1A87, 1, 0, 0), (0x1A8B, -1, 0, 0)],
        ),
        102 => {
            gem_body(w, id, &[(0x1A83, 0, 0), (0x1A82, 0, 1), (0x1A86, 0, -1)]);
            plain(
                w,
                id,
                &[
                    (0xF2C, 0, 0, 6),
                    (0xF1D, 0, 0, 5),
                    (0xF2B, 0, 0, 2),
                    (0xF21, 0, 0, 1),
                    (0xF22, 0, 0, 4),
                    (0xF2F, 0, 0, 5),
                    (0xF26, 0, 0, 6),
                    (0xF27, 0, 0, 3),
                    (0xF29, 0, 0, 0),
                ],
            );
        }
        103 => {
            gem_body(w, id, &[(0x1A88, 0, 0), (0x1A87, 1, 0), (0x1A8B, -1, 0)]);
            plain(
                w,
                id,
                &[
                    (0xF2E, 0, 0, 6),
                    (0xF12, 0, 0, 3),
                    (0xF29, 0, 0, 1),
                    (0xF24, 0, 0, 5),
                    (0xF21, 0, 0, 1),
                    (0xF2B, 0, 0, 3),
                    (0xF2F, 0, 0, 4),
                    (0xF23, 0, 0, 3),
                    (0xF27, 0, 0, 3),
                ],
            );
        }
        _ => {}
    }
    let now = w.now;
    set_cart(
        w,
        id,
        CartData {
            cart_type: kind,
            gems: 0,
            ore: 0,
            last_resource_time: now,
        },
    );
    id
}

fn cart_deed_of(w: &mut World, a: ItemId) -> ItemId {
    give_resources(w, a);
    let d = mining_cart_deed_new(w);
    set_reward(w, d, is_reward(w, a));
    let c = cart(w, a);
    let mut s = deed_state(w, d);
    s.gems = c.gems;
    s.ore = c.ore;
    if let Some(i) = w.item_mut(d) {
        i.ext.insert(s);
    }
    d
}

fn cart_component_used(w: &mut World, a: ItemId, _c: ItemId, from: MobileId) {
    let house = muo_multis::house::access::find_house_at_item(w, a);
    let loc = ops::get_world_location(w, a);
    let from_z = w.mobile(from).map_or(0, |m| m.location.z);
    let z = w.item(a).map_or(0, |i| i.location.z);
    if !muo_content::mx::in_range_point(w, from, loc, 2)
        || !deps::in_los_item(w, from, a)
        || !(from_z - z > -3 && from_z - z < 3)
    {
        overhead(w, from, 0x3B2, 1019045); // I can't reach that.
        return;
    }
    if !house.is_some_and(|h| {
        muo_multis::house::access::has_secure_access(
            w,
            h,
            from,
            muo_multis::house::SecureLevel::Friends,
        )
    }) {
        msg(w, from, 1061637); // You are not allowed to access this.
        return;
    }
    give_resources(w, a);
    let d = cart(w, a);
    match d.cart_type {
        100 | 101 => {
            if d.ore > 0 {
                let name = match utility::random(&mut w.rng, 9) {
                    0 => "Server.Items.IronIngot",
                    1 => "Server.Items.DullCopperIngot",
                    2 => "Server.Items.ShadowIronIngot",
                    3 => "Server.Items.CopperIngot",
                    4 => "Server.Items.BronzeIngot",
                    5 => "Server.Items.GoldIngot",
                    6 => "Server.Items.AgapiteIngot",
                    7 => "Server.Items.VeriteIngot",
                    _ => "Server.Items.ValoriteIngot",
                };
                give(w, a, from, name, d.ore.min(10), 1094724, false); // Ore: ~1_COUNT~
            } else {
                msg(w, from, 1094725); // There are no more resources available at this time.
            }
        }
        102 | 103 => {
            if d.gems > 0 {
                let name = match utility::random(&mut w.rng, 15) {
                    0 => "Server.Items.Amber",
                    1 => "Server.Items.Amethyst",
                    2 => "Server.Items.Citrine",
                    3 => "Server.Items.Diamond",
                    4 => "Server.Items.Emerald",
                    5 => "Server.Items.Ruby",
                    6 => "Server.Items.Sapphire",
                    7 => "Server.Items.StarSapphire",
                    8 => "Server.Items.Tourmaline",
                    9 => "Server.Items.PerfectEmerald",
                    10 => "Server.Items.DarkSapphire",
                    11 => "Server.Items.Turquoise",
                    12 => "Server.Items.EcruCitrine",
                    13 => "Server.Items.FireRuby",
                    _ => "Server.Items.BlueDiamond",
                };
                give(w, a, from, name, d.gems.min(5), 1094723, true); // Gems: ~1_COUNT~
            } else {
                msg(w, from, 1094725); // There are no more resources available at this time.
            }
        }
        _ => {}
    }
}

/// The tail of `OnComponentUsed`: the stack goes to the pack, the cart loses it and says what is left.
fn give(
    w: &mut World,
    cart_id: ItemId,
    from: MobileId,
    name: &str,
    amount: i32,
    label: i32,
    gems: bool,
) {
    let Some(item) = muo_content::factory::make_item(w, name) else {
        return;
    };
    ops::set_amount(w, item, amount);
    if !muo_server::mobile::part2::items::place_in_backpack(w, from, item) {
        super::util::delete(w, item);
        msg(w, from, 1078837); // Your backpack is full! Please make room and try again.
    } else {
        muo_server::item::messages::public_overhead_message_number(
            w,
            cart_id,
            MessageType::REGULAR,
            0,
            label,
            &amount.to_string(),
        );
        let mut d = cart(w, cart_id);
        if gems {
            d.gems -= amount;
        } else {
            d.ore -= amount;
        }
        set_cart(w, cart_id, d);
    }
}

static CART_CLASS: AddonClass = AddonClass {
    deed_of: Some(cart_deed_of),
    on_component_used: cart_component_used,
    ..BASE_ADDON_CLASS
};

/// `Server.Items.MiningCart` (`[SerializationGenerator(3)]`; `MiningCart(MiningCartType type)` only).
pub static MINING_CART: ItemType = ItemType {
    full_name: "Server.Items.MiningCart",
    parent: Some(&BASE_ADDON),
    construct: None,
    family: Some(&CART_CLASS),
    serialize: |w, id, out| {
        (BASE_ADDON.serialize)(w, id, out);
        out.write_encoded_int(3);
        let d = cart(w, id);
        out.write_enum(4, i64::from(d.cart_type));
        out.write_bool(is_reward(w, id));
        out.write_i32(d.gems);
        out.write_i32(d.ore);
        out.write_date_time_ticks(d.last_resource_time.ticks());
    },
    deserialize: |w, id, r| {
        (BASE_ADDON.deserialize)(w, id, r)?;
        let version = r.read_encoded_int()?;
        let cart_type = if version < 3 {
            r.read_i32()?
        } else {
            r.read_enum(4)? as i32
        };
        let reward = r.read_bool()?;
        let gems = r.read_i32()?;
        let ore = r.read_i32()?;
        let last_resource_time = DateTime::from_ticks(r.read_date_time_ticks()?);
        set_reward_raw(w, id, reward);
        set_cart(
            w,
            id,
            CartData {
                cart_type,
                gems,
                ore,
                last_resource_time,
            },
        );
        give_resources(w, id); // [AfterDeserialization]
        Ok(())
    },
    ..BASE_ADDON
};

/// `MiningCartDeed()`.
pub fn mining_cart_deed_new(w: &mut World) -> ItemId {
    let id = addon::new_deed(w, &MINING_CART_DEED);
    ops::set_loot_type(w, id, muo_server::item::LootType::Blessed);
    id
}

static CART_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, deed| {
        let s = deed_state(w, deed);
        let a = mining_cart_new(w, s.statue);
        set_reward(w, a, is_reward(w, deed));
        let mut d = cart(w, a);
        d.gems = s.gems;
        d.ore = s.ore;
        set_cart(w, a, d);
        a
    },
    select_direction: None,
};

/// `Server.Items.MiningCartDeed` (`Gems` and `Ore` are `[SerializableField(1)]` and `(2)`).
pub static MINING_CART_DEED: ItemType = ItemType {
    full_name: "Server.Items.MiningCartDeed",
    parent: Some(&BASE_ADDON_DEED),
    construct: Some(mining_cart_deed_new),
    family: Some(&CART_DEED_CLASS),
    label_number: |_, _| 1080385, // deed for a mining cart decoration
    get_properties: |w, id, list| {
        (BASE_ADDON_DEED.get_properties)(w, id, list);
        if is_reward(w, id) {
            list.add(1080457); // 10th Year Veteran Reward
        }
    },
    on_double_click: |w, id, from| {
        if reward_blocked(w, from, id) {
            return;
        }
        if in_pack(w, id, from) {
            if let Some(f) = hooks(w).and_then(|h| h.reward_option_gump) {
                f(w, from, id);
            }
        } else {
            msg(w, from, 1062334); // This item must be in your backpack to be used.
        }
    },
    serialize: |w, id, out| {
        (BASE_ADDON_DEED.serialize)(w, id, out);
        out.write_encoded_int(0);
        out.write_bool(is_reward(w, id));
        let s = deed_state(w, id);
        out.write_i32(s.gems);
        out.write_i32(s.ore);
    },
    deserialize: |w, id, r| {
        (BASE_ADDON_DEED.deserialize)(w, id, r)?;
        let _version = r.read_encoded_int()?;
        let reward = r.read_bool()?;
        let gems = r.read_i32()?;
        let ore = r.read_i32()?;
        set_reward_raw(w, id, reward);
        let mut s = deed_state(w, id);
        s.gems = gems;
        s.ore = ore;
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(s);
        }
        Ok(())
    },
    ..BASE_ADDON_DEED
};

// ---- DecorativeShield ----

/// `DecorativeShield(itemID)`.
pub fn decorative_shield_new(w: &mut World, item_id: i32) -> ItemId {
    let id = ops::new_item_with_id(w, &DECORATIVE_SHIELD, item_id);
    ops::set_movable(w, id, false);
    id
}

fn shield_facing_south(item_id: i32) -> bool {
    if item_id < 0x1582 {
        return item_id & 1 == 0;
    }
    item_id <= 0x1585
}

fn shield_could_fit(w: &mut World, id: ItemId, p: Point3D, map: MapId) -> bool {
    let (item_id, height) = {
        let Some(i) = w.item(id) else { return false };
        (i.item_id, item_height(w, i.item_id))
    };
    muo_server::map::fit::can_fit_item(w, map, p.x, p.y, p.z, height)
        && (shield_facing_south(item_id) && addon::is_wall(w, p.x, p.y - 1, p.z, Some(map))
            || addon::is_wall(w, p.x - 1, p.y, p.z, Some(map)))
}

fn shield_deed(w: &mut World, id: ItemId) -> ItemId {
    let d = decorative_shield_deed_new(w);
    let r = is_reward(w, id);
    set_reward(w, d, r);
    d
}

rtype! {
    /// `Server.Items.DecorativeShield` (`IAddon`; `DecorativeShield(int itemID = 0x156C)`).
    DECORATIVE_SHIELD = "Server.Items.DecorativeShield" : ITEM_BASE;
    construct: Some(|w| decorative_shield_new(w, 0x156C)),
    force_show_properties: |w, _| w.core.opl_enabled,
    get_properties: |w, id, list| {
        (ITEM_BASE.get_properties)(w, id, list);
        if deps::core_ml(w) && is_reward(w, id) {
            list.add(1076220); // 4th Year Veteran Reward
        }
    },
    on_double_click: |w, id, from| {
        let loc = w.item(id).map_or_else(Point3D::default, |i| i.location);
        if !muo_content::mx::in_range_point(w, from, loc, 2) {
            overhead(w, from, 0x3B2, 1019045); // I can't reach that.
            return;
        }
        let house = muo_multis::house::access::find_house_at_item(w, id);
        if house.is_some_and(|h| muo_multis::house::access::is_owner(w, h, from)) {
            if let Some(f) = hooks(w).and_then(|h| h.reward_demolition_gump) {
                f(w, from, id, 1049783); // Do you wish to re-deed this decoration?
            }
        } else {
            // You can only re-deed this decoration if you are the house owner or originally placed the decoration.
            msg(w, from, 1049784);
        }
    },
}

/// `DecorativeShieldDeed.GetWestItemID(east)`.
pub fn shield_west_item_id(east: i32) -> i32 {
    match east {
        0x1582 => 0x1635,
        0x1583 => 0x1634,
        0x1584 => 0x1637,
        0x1585 => 0x1636,
        _ => east + 1,
    }
}

/// `DecorativeShieldDeed()`.
pub fn decorative_shield_deed_new(w: &mut World) -> ItemId {
    let id = ops::new_item_with_id(w, &DECORATIVE_SHIELD_DEED, 0x14F0);
    ops::set_loot_type(w, id, muo_server::item::LootType::Blessed);
    id
}

const SHIELD_START: i32 = 0x156C;
const SHIELD_END: i32 = 0x1585;

/// `DecorativeShieldDeed.DecorativeShieldGump`.
struct ShieldGump {
    deed: ItemId,
}

impl StaticGumpLogic for ShieldGump {
    const TYPE_NAME: &'static str = "Server.Items.DecorativeShieldDeed+DecorativeShieldGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut StaticGumpBuilder) {
        b.add_page(0);
        b.add_background(25, 0, 500, 230, 0xA28);
        let mut item_id = SHIELD_START;
        for i in 1..=2 {
            b.add_page(i);
            for j in 0..(9 - i) {
                b.add_item(40 + j * 60, 70, item_id, 0);
                b.add_button(60 + j * 60, 50, 0x845, 0x846, item_id);
                item_id += if item_id < 0x1582 { 2 } else { 1 };
            }
            if i == 1 {
                b.add_button_type(455, 198, 0x8B0, 0x8B0, 0, GumpButtonType::Page, 2);
            } else {
                b.add_button_type(70, 198, 0x8AF, 0x8AF, 0, GumpButtonType::Page, 1);
            }
        }
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let b = info.button_id;
        if ops::is_deleted(w, self.deed)
            || !(SHIELD_START..=SHIELD_END).contains(&b)
            || ((b & 1 != 0 || b >= 0x1582) && !(0x1582..=0x1585).contains(&b))
        {
            return;
        }
        let Some(m) = w.net_states.get(&sender).and_then(|n| n.mobile) else {
            return;
        };
        msg(w, m, 1049780); // Where would you like to place this decoration?
        let deed = self.deed;
        begin_target(w, m, move |w, from, t| {
            shield_on_target(w, deed, b, from, t)
        });
    }
}

fn shield_on_target(w: &mut World, deed: ItemId, item_id: i32, from: MobileId, targeted: Targeted) {
    if ops::is_deleted(w, deed) {
        return;
    }
    if !in_pack(w, deed, from) {
        msg(w, from, 1042038); // You must have the object in your backpack to use it.
        return;
    }
    if owned_house_at_mobile(w, from).is_none() {
        msg(w, from, 502092); // You must be in your house to do this.
        return;
    }
    let Some(p) = target_point(w, targeted) else {
        return;
    };
    let Some(map) = w.mobile(from).and_then(|m| m.map) else {
        return;
    };
    let height = item_height(w, item_id);
    if !muo_server::map::fit::can_fit_point(w, map, p, height) {
        msg(w, from, 500269); // You cannot build that there.
        return;
    }
    let Some(house) = owned_house_at_point(w, from, p, map, height) else {
        msg(w, from, 1042036); // That location is not in your house.
        return;
    };
    let north = addon::is_wall(w, p.x, p.y - 1, p.z, Some(map));
    let west = addon::is_wall(w, p.x - 1, p.y, p.z, Some(map));
    if north && west {
        let g = ShieldFacingGump {
            deed,
            item_id,
            location: p,
            house,
        };
        if super::util::net_state(w, from).is_some() {
            send_gump_to_mobile(w, from, Box::new(DynamicGumpObj::new(150, 50, g)), false);
        }
    } else if north || west {
        let shield = decorative_shield_new(
            w,
            if west {
                shield_west_item_id(item_id)
            } else {
                item_id
            },
        );
        addon::house_addons_add(w, house, shield);
        set_reward(w, shield, is_reward(w, deed));
        ops::move_to_world_map(w, shield, p, Some(map));
        super::util::delete(w, deed);
    } else {
        msg(w, from, 1049781); // This decoration must be placed next to a wall.
    }
}

/// `DecorativeShieldDeed.InternalTarget.DecorativeShieldFacingGump`.
struct ShieldFacingGump {
    deed: ItemId,
    item_id: i32,
    location: Point3D,
    house: ItemId,
}

impl DynamicGumpLogic for ShieldFacingGump {
    const TYPE_NAME: &'static str =
        "Server.Items.DecorativeShieldDeed+InternalTarget+DecorativeShieldFacingGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        b.add_page(0);
        b.add_background(0, 0, 300, 150, 0xA28);
        b.add_item(90, 30, shield_west_item_id(self.item_id), 0);
        b.add_item(180, 30, self.item_id, 0);
        b.add_button(50, 35, 0x867, 0x869, 2); // East
        b.add_button(145, 35, 0x867, 0x869, 1); // South
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        if ops::is_deleted(w, self.deed) {
            return;
        }
        let Some(m) = w.net_states.get(&sender).and_then(|n| n.mobile) else {
            return;
        };
        let map = w.mobile(m).and_then(|x| x.map);
        let shield = match info.button_id {
            2 => decorative_shield_new(w, shield_west_item_id(self.item_id)),
            1 => decorative_shield_new(w, self.item_id),
            _ => return,
        };
        addon::house_addons_add(w, self.house, shield);
        set_reward(w, shield, is_reward(w, self.deed));
        ops::move_to_world_map(w, shield, self.location, map);
        super::util::delete(w, self.deed);
    }
}

rtype! {
    /// `Server.Items.DecorativeShieldDeed`.
    DECORATIVE_SHIELD_DEED = "Server.Items.DecorativeShieldDeed" : ITEM_BASE;
    construct: Some(decorative_shield_deed_new),
    default_weight: |_, _| 1.0,
    label_number: |_, _| 1049771, // deed for a decorative shield wall hanging
    get_properties: |w, id, list| {
        (ITEM_BASE.get_properties)(w, id, list);
        if is_reward(w, id) {
            list.add(1076220); // 4th Year Veteran Reward
        }
    },
    on_double_click: |w, id, from| {
        if reward_blocked(w, from, id) {
            return;
        }
        if in_pack(w, id, from) {
            if super::util::net_state(w, from).is_some() {
                send_gump_to_mobile(w, from, Box::new(StaticGumpObj::new(150, 50, ShieldGump { deed: id })), false);
            }
        } else {
            msg(w, from, 1042038); // You must have the object in your backpack to use it.
        }
    },
}

pub fn register(types: &mut TypeRegistry) {
    for t in [
        &ANKH_OF_SACRIFICE_COMPONENT,
        &ANKH_OF_SACRIFICE_ADDON,
        &ANKH_OF_SACRIFICE_DEED,
        &MINING_CART,
        &MINING_CART_DEED,
        &DECORATIVE_SHIELD,
        &DECORATIVE_SHIELD_DEED,
    ] {
        types.register_item(t, &[]);
    }
}

pub(super) fn configure(w: &mut World) {
    house_deps::deps_mut(w).simple_addons.push(SimpleAddon {
        is: |w, id| w.item(id).is_some_and(|i| i.ty.is(&DECORATIVE_SHIELD)),
        deed: shield_deed,
        could_fit: shield_could_fit,
    });
}

#[allow(dead_code)]
fn unused(_: EntityId) {}
