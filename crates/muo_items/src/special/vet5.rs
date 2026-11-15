//! Veteran rewards `Cannon`, `WallBanner`, `WeaponEngravingTool` and the `IsRewardItem`
//! `[CommandProperty]` of the reward classes. See `vet.rs` for the shared parts.
//!
//! The tinker guildmaster's recharge entry calls [`engraving_tool_find`] and [`engraving_confirm_gump`].

use muo_content::gumps::base::{
    DynamicGumpLogic, DynamicGumpObj, RelayInfo, StaticGumpLogic, StaticGumpObj,
};
use muo_content::gumps::builder::{DynamicGumpBuilder, StaticGumpBuilder};
use muo_content::gumps::{GumpButtonType, send_gump_to_mobile};
use muo_content::items::addon::{
    self, ADDON_COMPONENT, ADDON_COMPONENT_CLASS, AddonClass, BASE_ADDON, BASE_ADDON_CLASS,
    BASE_ADDON_DEED, ComponentClass, DeedClass,
};
use muo_content::props::{self, Ctor, Kind, Param, PropDef, Value};
use muo_content::utilities::types::{Numeric, PropType};

use muo_primitives::{Point3D, utility};
use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, LootType, ops};
use muo_server::mobile::AccessLevel;
use muo_server::net_state::NetStateId;
use muo_server::skills::SkillName;
use muo_server::target::{self, TargetFlags, Targeted};
use muo_server::type_registry::TypeRegistry;

use super::hooks::hooks;
use super::util::{deleted, in_pack, msg};
use super::vet::*;
use crate::simple::leaf;

const INT: Kind = Kind::Plain(PropType::Numeric(Numeric::Int));

/// `[CommandProperty(GameMaster)] IsRewardItem` of an `IRewardItem` class.
static REWARD_PROPS: &[PropDef] = &[PropDef {
    name: "IsRewardItem",
    kind: Kind::Plain(PropType::Bool),
    read: AccessLevel::GameMaster,
    write: AccessLevel::GameMaster,
    get: |w, e| match e {
        EntityId::Item(i) => Value::Bool(is_reward(w, i)),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err("not an item".to_string());
        };
        set_reward(w, i, v.bool());
        Ok(())
    }),
}];

/// `[CommandProperty(GameMaster)] Charges` of the cannon addon.
static CHARGES_PROPS: &[PropDef] = &[PropDef {
    name: "Charges",
    kind: INT,
    read: AccessLevel::GameMaster,
    write: AccessLevel::GameMaster,
    get: |w, e| match e {
        EntityId::Item(i) => Value::Int(i64::from(charges(w, i))),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err("not an item".to_string());
        };
        set_charges(w, i, v.i32());
        Ok(())
    }),
}];

/// `[CommandProperty(GameMaster)] UsesRemaining` of the engraving tool.
static USES_PROPS: &[PropDef] = &[PropDef {
    name: "UsesRemaining",
    kind: INT,
    read: AccessLevel::GameMaster,
    write: AccessLevel::GameMaster,
    get: |w, e| match e {
        EntityId::Item(i) => Value::Int(i64::from(uses(w, i))),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err("not an item".to_string());
        };
        set_uses(w, i, v.i32());
        Ok(())
    }),
}];

/// `[CommandProperty(GameMaster)] East` of the wall banner.
static EAST_PROPS: &[PropDef] = &[PropDef {
    name: "East",
    kind: Kind::Plain(PropType::Bool),
    read: AccessLevel::GameMaster,
    write: AccessLevel::GameMaster,
    get: |w, e| match e {
        EntityId::Item(i) => Value::Bool(banner_east(w, i)),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err("not an item".to_string());
        };
        if let Some(it) = w.item_mut(i) {
            it.ext.insert(BannerEast(v.bool()));
        }
        ops::invalidate_properties(w, i);
        Ok(())
    }),
}];

/// `base(range, true, TargetFlags.None)` with an out-of-range override.
fn begin_range_target(
    w: &mut World,
    from: MobileId,
    range: i32,
    on_target: impl Fn(&mut World, MobileId, Targeted) + 'static,
    on_out_of_range: impl Fn(&mut World, MobileId, Targeted) + 'static,
) {
    let mut t = target::Target::new(w, range, true, TargetFlags::NONE);
    t.handlers.on_target = Some(std::rc::Rc::new(on_target));
    t.handlers.on_target_out_of_range = Some(std::rc::Rc::new(on_out_of_range));
    let tid = target::add_target(w, t);
    target::set_mobile_target(w, from, Some(tid));
}

// ---- Cannon ----

/// `CannonAddon._cannonDirection` and `_charges`; `CannonDeed._charges` is the same ext.
#[derive(Clone, Copy, Debug, Default)]
pub struct CannonData {
    pub direction: i32,
    pub charges: i32,
}

fn cannon(w: &World, id: ItemId) -> CannonData {
    w.item(id)
        .and_then(|i| i.ext.get::<CannonData>())
        .copied()
        .unwrap_or_default()
}

fn set_cannon_raw(w: &mut World, id: ItemId, d: CannonData) {
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(d);
    }
}

pub fn charges(w: &World, id: ItemId) -> i32 {
    cannon(w, id).charges
}

/// `Charges = value`: the addon invalidates its components' properties, the deed its own.
pub fn set_charges(w: &mut World, id: ItemId, value: i32) {
    let mut d = cannon(w, id);
    d.charges = value;
    set_cannon_raw(w, id, d);
    if addon::is_addon(w, id) {
        for c in addon::components(w, id) {
            ops::invalidate_properties(w, c);
        }
    } else {
        ops::invalidate_properties(w, id);
    }
}

/// `CannonAddonComponent(itemID)`.
pub fn cannon_addon_component_new(w: &mut World, item_id: i32) -> ItemId {
    let id = addon::new_component(w, &CANNON_ADDON_COMPONENT, item_id);
    ops::set_loot_type(w, id, LootType::Blessed);
    id
}

leaf! {
    /// `Server.Items.CannonAddonComponent`.
    CANNON_ADDON_COMPONENT = "Server.Items.CannonAddonComponent" : ADDON_COMPONENT, v0;
    label_number: crate::class_props::label_number, // Decorative Cannon
    get_properties: |w, id, list| {
        (ADDON_COMPONENT.get_properties)(w, id, list);
        if let Some(a) = addon::component_addon(w, id).filter(|&a| w.item(a).is_some_and(|i| i.ty.is(&CANNON_ADDON))) {
            if is_reward(w, a) {
                list.add(1076223); // 7th Year Veteran Reward
            }
            list.add_int(1076207, charges(w, a)); // Remaining Charges: ~1_val~
        }
    },
}

const CANNON_EFFECTS: [i32; 4] = [0x36B0, 0x3728, 0x3709, 0x36FE];

/// `CannonAddon(direction)`; `direction` is a `CannonDirection` (North, East, South, West).
pub fn cannon_addon_new(w: &mut World, direction: i32) -> ItemId {
    let id = addon::new_addon(w, &CANNON_ADDON);
    set_cannon_raw(
        w,
        id,
        CannonData {
            direction,
            charges: 0,
        },
    );
    let rows: [(i32, i32, i32); 3] = match direction {
        0 => [(0xE8D, 0, 0), (0xE8C, 0, 1), (0xE8B, 0, 2)],
        1 => [(0xE96, 0, 0), (0xE95, -1, 0), (0xE94, -2, 0)],
        2 => [(0xE91, 0, 0), (0xE92, 0, -1), (0xE93, 0, -2)],
        _ => [(0xE8E, 0, 0), (0xE8F, 1, 0), (0xE90, 2, 0)],
    };
    for (g, x, y) in rows {
        let c = cannon_addon_component_new(w, g);
        addon::add_component(w, id, c, x, y, 0);
    }
    id
}

/// `CannonAddon.Validate(keg)`.
fn cannon_validate(w: &World, keg: Option<ItemId>) -> i32 {
    use crate::skill_items::potion_keg::{held, keg_type};
    use crate::skill_items::potions::fx;
    let Some(keg) = keg.filter(|&k| w.item(k).is_some_and(|i| !i.deleted())) else {
        return 0;
    };
    if held(w, keg) != 100 {
        return 0;
    }
    match keg_type(w, keg) {
        fx::EXPLOSION_LESSER => 5,
        fx::EXPLOSION => 10,
        fx::EXPLOSION_GREATER => 15,
        _ => 0,
    }
}

/// `CannonAddon.Fill(from, keg)`.
fn cannon_fill(w: &mut World, a: ItemId, from: MobileId, keg: ItemId) {
    let n = cannon_validate(w, Some(keg));
    set_charges(w, a, n);
    if n > 0 {
        super::util::delete(w, keg);
        msg(w, from, 1076199); // Your cannon is recharged.
    } else {
        msg(w, from, 1076198); // You do not have a full keg of explosion potions needed to recharge the cannon.
    }
}

/// `CannonAddon.DoFireEffect(target)`.
fn cannon_fire(w: &mut World, a: ItemId, target: Point3D) {
    let Some(map) = w.item(a).and_then(|i| i.map) else {
        return;
    };
    let sound = *utility::random_element(&mut w.rng, &[0x11B, 0x11C, 0x11D]).unwrap_or(&0x11B);
    muo_server::effects::play_sound(w, target, Some(map), sound);
    let e = *utility::random_element(&mut w.rng, &CANNON_EFFECTS).unwrap_or(&0x36B0);
    muo_server::effects::send_location_effect(w, target, Some(map), e, 16, 1, 0, 0);
    let mut count = utility::random(&mut w.rng, 3);
    while count > 0 {
        let x = target.x + utility::random_min_max(&mut w.rng, -1, 1);
        let y = target.y + utility::random_min_max(&mut w.rng, -1, 1);
        let e = *utility::random_element(&mut w.rng, &CANNON_EFFECTS).unwrap_or(&0x36B0);
        muo_server::effects::send_location_effect(
            w,
            Point3D::new(x, y, target.z),
            Some(map),
            e,
            16,
            1,
            0,
            0,
        );
        count -= 1;
    }
    let c = charges(w, a);
    set_charges(w, a, c - 1);
}

/// `CannonAddon.InternalTarget.OnTarget`.
fn cannon_target(w: &mut World, a: ItemId, from: MobileId, t: Targeted) {
    if deleted(w, a) {
        return;
    }
    let Some(p) = target_point(w, t) else { return };
    if !muo_server::item::deps::in_los_point(w, from, p) {
        msg(w, from, 1049630); // You cannot see that target!
        return;
    }
    let loc = w.item(a).map_or_else(Point3D::default, |i| i.location);
    if utility::in_range(p.x, p.y, loc.x, loc.y, 2) {
        msg(w, from, 1076215); // Cannon must be aimed farther away.
        return;
    }
    let (x, y) = (p.x - loc.x, p.y - loc.y);
    let allow = match cannon(w, a).direction {
        0 => y < 0 && x.abs() <= -y / 3,
        1 => x > 0 && y.abs() <= x / 3,
        2 => y > 0 && x.abs() <= y / 3,
        3 => x < 0 && y.abs() <= -x / 3,
        _ => false,
    };
    if allow && utility::in_range(p.x, p.y, loc.x, loc.y, 14) {
        cannon_fire(w, a, p);
    } else {
        msg(w, from, 1076203); // Target out of range.
    }
}

/// `CannonAddon.OnComponentUsed(c, from)`.
fn cannon_component_used(w: &mut World, a: ItemId, _c: ItemId, from: MobileId) {
    let loc = w.item(a).map_or_else(Point3D::default, |i| i.location);
    if !muo_content::mx::in_range_point(w, from, loc, 2) {
        msg(w, from, 1076766); // That is too far away.
        return;
    }
    if charges(w, a) > 0 {
        begin_range_target(
            w,
            from,
            12,
            move |w, from, t| cannon_target(w, a, from, t),
            |w, from, _| msg(w, from, 1076203), // Target out of range.
        );
    } else if let Some(pack) = muo_server::mobile::part1::props_b::backpack(w, from) {
        let keg = muo_content::factory::find_item_by_type(w, pack, "Server.Items.PotionKeg");
        if cannon_validate(w, keg) > 0 {
            if let Some(keg) = keg {
                cannon_gump(w, from, a, keg);
            }
        } else {
            msg(w, from, 1076198); // You do not have a full keg of explosion potions needed to recharge the cannon.
        }
    }
}

/// `CannonAddon.CannonGump`; `charges` is `Validate(keg)` when the gump is built.
struct CannonGump {
    cannon: ItemId,
    keg: ItemId,
    charges: i32,
}

fn cannon_gump(w: &mut World, from: MobileId, a: ItemId, keg: ItemId) {
    if super::util::net_state(w, from).is_none() || deleted(w, a) || deleted(w, keg) {
        return;
    }
    let charges = cannon_validate(w, Some(keg));
    send_gump_to_mobile(
        w,
        from,
        Box::new(DynamicGumpObj::new(
            50,
            50,
            CannonGump {
                cannon: a,
                keg,
                charges,
            },
        )),
        false,
    );
}

impl DynamicGumpLogic for CannonGump {
    const TYPE_NAME: &'static str = "Server.Items.CannonAddon+CannonGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        b.add_page(0);
        b.add_background(0, 0, 291, 133, 0x13BE);
        b.add_image_tiled(5, 5, 280, 100, 0xA40);
        // You will need a full keg of explosion potions to recharge the cannon.  Your keg will provide ~1_CHARGES~ charges.
        b.add_html_localized_args(
            9,
            9,
            272,
            100,
            1076196,
            &self.charges.to_string(),
            0x7FFF,
            false,
            false,
        );
        b.add_button(5, 107, 0xFB1, 0xFB2, 0);
        b.add_html_localized_color(40, 109, 100, 20, 1060051, 0x7FFF, false, false); // CANCEL
        b.add_button(160, 107, 0xFB7, 0xFB8, 1);
        b.add_html_localized_color(195, 109, 120, 20, 1076197, 0x7FFF, false, false); // Recharge
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        if !deleted(w, self.cannon)
            && info.button_id == 1
            && let Some(m) = mobile_of(w, sender)
        {
            cannon_fill(w, self.cannon, m, self.keg);
        }
    }
}

fn cannon_deed_of(w: &mut World, a: ItemId) -> ItemId {
    let d = cannon_deed_new(w);
    set_cannon_raw(
        w,
        d,
        CannonData {
            direction: 0,
            charges: charges(w, a),
        },
    );
    set_reward(w, d, is_reward(w, a));
    d
}

static CANNON_CLASS: AddonClass = AddonClass {
    deed_of: Some(cannon_deed_of),
    on_component_used: cannon_component_used,
    ..BASE_ADDON_CLASS
};

/// `Server.Items.CannonAddon` (`[SerializationGenerator(1)]`; `CannonAddon(direction)` only).
pub static CANNON_ADDON: ItemType = ItemType {
    full_name: "Server.Items.CannonAddon",
    parent: Some(&BASE_ADDON),
    construct: None,
    family: Some(&CANNON_CLASS),
    serialize: |w, id, out| {
        (BASE_ADDON.serialize)(w, id, out);
        out.write_encoded_int(1);
        let d = cannon(w, id);
        out.write_enum(4, i64::from(d.direction));
        out.write_i32(d.charges);
        out.write_bool(is_reward(w, id));
    },
    deserialize: |w, id, r| {
        (BASE_ADDON.deserialize)(w, id, r)?;
        let version = r.read_encoded_int()?;
        let direction = if version < 1 {
            r.read_i32()?
        } else {
            r.read_enum(4)? as i32
        };
        let charges = r.read_i32()?;
        let reward = r.read_bool()?;
        set_cannon_raw(w, id, CannonData { direction, charges });
        set_reward_raw(w, id, reward);
        Ok(())
    },
    ..BASE_ADDON
};

/// `CannonDeed()`.
pub fn cannon_deed_new(w: &mut World) -> ItemId {
    let id = addon::new_deed(w, &CANNON_DEED);
    ops::set_loot_type(w, id, LootType::Blessed);
    id
}

static CANNON_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, deed| {
        let direction = deed_state(w, deed).statue; // `_direction`
        let a = cannon_addon_new(w, direction);
        set_charges(w, a, charges(w, deed));
        set_reward(w, a, is_reward(w, deed));
        a
    },
    select_direction: None,
};

/// `Server.Items.CannonDeed` (`IRewardOption`: [`super::vet::reward_options`]).
pub static CANNON_DEED: ItemType = ItemType {
    full_name: "Server.Items.CannonDeed",
    parent: Some(&BASE_ADDON_DEED),
    construct: Some(cannon_deed_new),
    family: Some(&CANNON_DEED_CLASS),
    label_number: crate::class_props::label_number, // A deed for a cannon
    get_properties: |w, id, list| {
        (BASE_ADDON_DEED.get_properties)(w, id, list);
        if is_reward(w, id) {
            list.add(1076223); // 7th Year Veteran Reward
        }
        list.add_int(1076207, charges(w, id)); // Remaining Charges: ~1_val~
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
            msg(w, from, 1042038); // You must have the object in your backpack to use it.
        }
    },
    serialize: |w, id, out| {
        (BASE_ADDON_DEED.serialize)(w, id, out);
        out.write_encoded_int(0);
        out.write_i32(charges(w, id));
        out.write_bool(is_reward(w, id));
    },
    deserialize: |w, id, r| {
        (BASE_ADDON_DEED.deserialize)(w, id, r)?;
        let _version = r.read_encoded_int()?;
        let c = r.read_i32()?;
        let reward = r.read_bool()?;
        set_cannon_raw(
            w,
            id,
            CannonData {
                direction: 0,
                charges: c,
            },
        );
        set_reward_raw(w, id, reward);
        Ok(())
    },
    ..BASE_ADDON_DEED
};

// ---- WallBanner ----

/// `WallBanner._east`.
#[derive(Clone, Copy, Debug, Default)]
struct BannerEast(bool);

/// `WallBannerDeed._bannerId`.
#[derive(Clone, Copy, Debug, Default)]
struct BannerId(i32);

fn banner_east(w: &World, id: ItemId) -> bool {
    w.item(id)
        .and_then(|i| i.ext.get::<BannerEast>())
        .is_some_and(|e| e.0)
}

/// `WallBannerComponent.East`.
fn component_east(w: &World, c: ItemId) -> bool {
    addon::component_addon(w, c).is_some_and(|a| banner_east(w, a))
}

static BANNER_COMPONENT_CLASS: ComponentClass = ComponentClass {
    needs_wall: true,
    wall_position_of: Some(|w, id| {
        if component_east(w, id) {
            Point3D::new(-1, 0, 0)
        } else {
            Point3D::new(0, -1, 0)
        }
    }),
    ..ADDON_COMPONENT_CLASS
};

/// `WallBannerComponent(itemID)`.
pub fn wall_banner_component_new(w: &mut World, item_id: i32) -> ItemId {
    addon::new_component(w, &WALL_BANNER_COMPONENT, item_id)
}

leaf! {
    /// `Server.Items.WallBannerComponent` (`IDyable`).
    WALL_BANNER_COMPONENT = "Server.Items.WallBannerComponent" : ADDON_COMPONENT, enc;
    family: Some(&BANNER_COMPONENT_CLASS),
}

/// `WallBanner`'s `case 1..30` rows: `(itemID, x, y)`.
const BANNERS: [&[(i32, i32, i32)]; 30] = [
    &[(0x161F, 0, 0), (0x161E, 0, 1), (0x161D, 0, 2)],
    &[(0x1586, 0, 0), (0x1587, 1, 0), (0x1588, 2, 0)],
    &[(0x1622, 0, 0), (0x1621, 0, 1), (0x1620, 0, 2)],
    &[(0x1589, 0, 0), (0x158A, 1, 0), (0x158B, 2, 0)],
    &[(0x1625, 0, 0), (0x1624, 0, 1), (0x1623, 0, 2)],
    &[(0x158C, 0, 0), (0x158D, 1, 0), (0x158E, 2, 0)],
    &[(0x1628, 0, 0), (0x1627, 0, 1), (0x1626, 0, 2)],
    &[(0x1590, 0, 0), (0x1591, 1, 0), (0x158F, 2, 0)],
    &[(0x162A, 0, 0), (0x1629, 0, 1), (0x1626, 0, 2)],
    &[(0x1592, 0, 0), (0x1593, 1, 0), (0x158F, 2, 0)],
    &[(0x162D, 0, 0), (0x162C, 0, 1), (0x162B, 0, 2)],
    &[(0x1594, 0, 0), (0x1595, 1, 0), (0x1596, 2, 0)],
    &[(0x1632, 0, 0), (0x1631, 0, 1), (0x162E, 0, 2)],
    &[(0x1598, 0, 0), (0x159B, 1, 0), (0x159C, 2, 0)],
    &[(0x1633, 0, 0), (0x1630, 0, 1), (0x162F, 0, 2)],
    &[(0x1599, 0, 0), (0x159A, 1, 0), (0x159D, 2, 0)],
    &[(0x1610, 0, 0), (0x160F, 0, 1)],
    &[(0x15A0, 0, 0), (0x15A1, 1, 0)],
    &[(0x1612, 0, 0), (0x1611, 0, 1)],
    &[(0x15A2, 0, 0), (0x15A3, 1, 0)],
    &[(0x1614, 0, 0), (0x1613, 0, 1)],
    &[(0x15A4, 0, 0), (0x15A5, 1, 0)],
    &[(0x1616, 0, 0), (0x1615, 0, 1)],
    &[(0x15A6, 0, 0), (0x15A7, 1, 0)],
    &[(0x1618, 0, 0), (0x1617, 0, 1)],
    &[(0x15A8, 0, 0), (0x15A9, 1, 0)],
    &[(0x161A, 0, 0), (0x1619, 0, 1)],
    &[(0x15AA, 0, 0), (0x15AB, 1, 0)],
    &[(0x161C, 0, 0), (0x161B, 0, 1)],
    &[(0x15AC, 0, 0), (0x15AD, 1, 0)],
];

/// `WallBanner(bannerID)`.
pub fn wall_banner_new(w: &mut World, banner_id: i32) -> ItemId {
    let id = addon::new_addon(w, &WALL_BANNER);
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(BannerEast(false));
    }
    if let Some(rows) = usize::try_from(banner_id - 1)
        .ok()
        .and_then(|n| BANNERS.get(n))
    {
        for &(g, x, y) in *rows {
            let c = wall_banner_component_new(w, g);
            addon::add_component(w, id, c, x, y, 0);
        }
    }
    id
}

fn banner_deed_of(w: &mut World, a: ItemId) -> ItemId {
    let d = wall_banner_deed_new(w);
    set_reward(w, d, is_reward(w, a));
    d
}

static BANNER_CLASS: AddonClass = AddonClass {
    deed_of: Some(banner_deed_of),
    ..BASE_ADDON_CLASS
};

/// `Server.Items.WallBanner` (`WallBanner(int bannerID)` only).
pub static WALL_BANNER: ItemType = ItemType {
    full_name: "Server.Items.WallBanner",
    parent: Some(&BASE_ADDON),
    construct: None,
    family: Some(&BANNER_CLASS),
    serialize: |w, id, out| {
        (BASE_ADDON.serialize)(w, id, out);
        out.write_encoded_int(0);
        out.write_bool(banner_east(w, id));
        out.write_bool(is_reward(w, id));
    },
    deserialize: |w, id, r| {
        (BASE_ADDON.deserialize)(w, id, r)?;
        let _version = r.read_encoded_int()?;
        let east = r.read_bool()?;
        let reward = r.read_bool()?;
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(BannerEast(east));
        }
        set_reward_raw(w, id, reward);
        Ok(())
    },
    ..BASE_ADDON
};

/// `WallBannerDeed()`.
pub fn wall_banner_deed_new(w: &mut World) -> ItemId {
    let id = addon::new_deed(w, &WALL_BANNER_DEED);
    ops::set_loot_type(w, id, LootType::Blessed);
    id
}

static BANNER_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, deed| {
        let n = w
            .item(deed)
            .and_then(|i| i.ext.get::<BannerId>())
            .map_or(0, |b| b.0);
        let a = wall_banner_new(w, n);
        set_reward(w, a, is_reward(w, deed));
        a
    },
    select_direction: None,
};

rtype! {
    /// `Server.Items.WallBannerDeed`.
    WALL_BANNER_DEED = "Server.Items.WallBannerDeed" : BASE_ADDON_DEED;
    construct: Some(wall_banner_deed_new),
    family: Some(&BANNER_DEED_CLASS),
    label_number: crate::class_props::label_number, // Wall Banner Deed
    get_properties: |w, id, list| {
        (BASE_ADDON_DEED.get_properties)(w, id, list);
        if is_reward(w, id) {
            list.add(1076225); // 9th Year Veteran Reward
        }
    },
    on_double_click: |w, id, from| {
        if reward_blocked(w, from, id) {
            return;
        }
        if in_pack(w, id, from) {
            if super::util::net_state(w, from).is_some() && !deleted(w, id) {
                send_gump_to_mobile(w, from, Box::new(StaticGumpObj::new(150, 50, WallBannerGump { deed: id })), false);
            }
        } else {
            msg(w, from, 1042038); // You must have the object in your backpack to use it.
        }
    },
}

/// `WallBannerDeed.WallBannerGump`.
struct WallBannerGump {
    deed: ItemId,
}

impl StaticGumpLogic for WallBannerGump {
    const TYPE_NAME: &'static str = "Server.Items.WallBannerDeed+WallBannerGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut StaticGumpBuilder) {
        b.add_background(25, 0, 500, 265, 0xA28);
        b.add_label(70, 12, 0x3E3, "Choose a Wall Banner:");
        b.add_page(1);
        b.add_item(55, 110, 0x161D, 0);
        b.add_item(75, 90, 0x161E, 0);
        b.add_item(95, 70, 0x161F, 0);
        b.add_button(70, 50, 0x845, 0x846, 1);
        b.add_item(105, 70, 0x1586, 0);
        b.add_item(125, 90, 0x1587, 0);
        b.add_item(145, 110, 0x1588, 0);
        b.add_button(145, 50, 0x845, 0x846, 2);
        b.add_item(200, 110, 0x1620, 0);
        b.add_item(220, 90, 0x1621, 0);
        b.add_item(240, 70, 0x1622, 0);
        b.add_button(220, 50, 0x845, 0x846, 3);
        b.add_item(250, 70, 0x1589, 0);
        b.add_item(270, 90, 0x158A, 0);
        b.add_item(290, 110, 0x158B, 0);
        b.add_button(300, 50, 0x845, 0x846, 4);
        b.add_item(350, 110, 0x1623, 0);
        b.add_item(370, 90, 0x1624, 0);
        b.add_item(390, 70, 0x1625, 0);
        b.add_button(365, 50, 0x845, 0x846, 5);
        b.add_item(400, 70, 0x158C, 0);
        b.add_item(420, 90, 0x158D, 0);
        b.add_item(440, 110, 0x158E, 0);
        b.add_button(445, 50, 0x845, 0x846, 6);
        b.add_button_type(455, 205, 0x8B0, 0x8B0, 0, GumpButtonType::Page, 2);
        b.add_page(2);
        b.add_item(52, 110, 0x1626, 0);
        b.add_item(72, 90, 0x1627, 0);
        b.add_item(95, 70, 0x1628, 0);
        b.add_button(70, 50, 0x845, 0x846, 7);
        b.add_item(105, 70, 0x1590, 0);
        b.add_item(125, 90, 0x1591, 0);
        b.add_item(145, 110, 0x158F, 0);
        b.add_button(145, 50, 0x845, 0x846, 8);
        b.add_item(197, 110, 0x1626, 0);
        b.add_item(217, 90, 0x1629, 0);
        b.add_item(240, 70, 0x162A, 0);
        b.add_button(220, 50, 0x845, 0x846, 9);
        b.add_item(250, 70, 0x1592, 0);
        b.add_item(270, 90, 0x1593, 0);
        b.add_item(290, 110, 0x158F, 0);
        b.add_button(300, 50, 0x845, 0x846, 10);
        b.add_item(340, 110, 0x162B, 0);
        b.add_item(363, 90, 0x162C, 0);
        b.add_item(385, 70, 0x162D, 0);
        b.add_button(365, 50, 0x845, 0x846, 11);
        b.add_item(395, 70, 0x1594, 0);
        b.add_item(417, 90, 0x1595, 0);
        b.add_item(439, 111, 0x1596, 0);
        b.add_button(445, 50, 0x845, 0x846, 12);
        b.add_button_type(70, 205, 0x8AF, 0x8AF, 0, GumpButtonType::Page, 1);
        b.add_button_type(455, 205, 0x8B0, 0x8B0, 0, GumpButtonType::Page, 3);
        b.add_page(3);
        b.add_item(55, 110, 0x162E, 0);
        b.add_item(75, 93, 0x1631, 0);
        b.add_item(95, 70, 0x1632, 0);
        b.add_button(70, 50, 0x845, 0x846, 13);
        b.add_item(118, 70, 0x1598, 0);
        b.add_item(138, 94, 0x159B, 0);
        b.add_item(159, 113, 0x159C, 0);
        b.add_button(160, 50, 0x845, 0x846, 14);
        b.add_item(219, 111, 0x162F, 0);
        b.add_item(238, 94, 0x1630, 0);
        b.add_item(258, 70, 0x1633, 0);
        b.add_button(240, 50, 0x845, 0x846, 15);
        b.add_item(279, 70, 0x1599, 0);
        b.add_item(298, 93, 0x159A, 0);
        b.add_item(319, 113, 0x159D, 0);
        b.add_button(320, 50, 0x845, 0x846, 16);
        b.add_item(380, 90, 0x160F, 0);
        b.add_item(400, 70, 0x1610, 0);
        b.add_button(390, 50, 0x845, 0x846, 17);
        b.add_item(420, 70, 0x15A0, 0);
        b.add_item(440, 90, 0x15A1, 0);
        b.add_button(455, 50, 0x845, 0x846, 18);
        b.add_button_type(70, 205, 0x8AF, 0x8AF, 0, GumpButtonType::Page, 2);
        b.add_button_type(455, 205, 0x8B0, 0x8B0, 0, GumpButtonType::Page, 4);
        b.add_page(4);
        b.add_item(55, 90, 0x1611, 0);
        b.add_item(75, 70, 0x1612, 0);
        b.add_button(70, 50, 0x845, 0x846, 19);
        b.add_item(105, 70, 0x15A2, 0);
        b.add_item(125, 90, 0x15A3, 0);
        b.add_button(145, 50, 0x845, 0x846, 20);
        b.add_item(200, 84, 0x1613, 0);
        b.add_item(220, 70, 0x1614, 0);
        b.add_button(215, 50, 0x845, 0x846, 21);
        b.add_item(250, 70, 0x15A4, 0);
        b.add_item(270, 84, 0x15A5, 0);
        b.add_button(290, 50, 0x845, 0x846, 22);
        b.add_item(350, 90, 0x1615, 0);
        b.add_item(370, 70, 0x1616, 0);
        b.add_button(365, 50, 0x845, 0x846, 23);
        b.add_item(400, 70, 0x15A6, 0);
        b.add_item(420, 90, 0x15A7, 0);
        b.add_button(445, 50, 0x845, 0x846, 24);
        b.add_button_type(70, 205, 0x8AF, 0x8AF, 0, GumpButtonType::Page, 3);
        b.add_button_type(455, 205, 0x8B0, 0x8B0, 0, GumpButtonType::Page, 5);
        b.add_page(5);
        b.add_item(55, 90, 0x1617, 0);
        b.add_item(77, 70, 0x1618, 0);
        b.add_button(70, 50, 0x845, 0x846, 25);
        b.add_item(105, 70, 0x15A8, 0);
        b.add_item(127, 90, 0x15A9, 0);
        b.add_button(145, 50, 0x845, 0x846, 26);
        b.add_item(200, 90, 0x1619, 0);
        b.add_item(222, 70, 0x161A, 0);
        b.add_button(220, 50, 0x845, 0x846, 27);
        b.add_item(250, 70, 0x15AA, 0);
        b.add_item(272, 90, 0x15AB, 0);
        b.add_button(300, 50, 0x845, 0x846, 28);
        b.add_item(350, 90, 0x161B, 0);
        b.add_item(372, 70, 0x161C, 0);
        b.add_button(365, 50, 0x845, 0x846, 29);
        b.add_item(400, 70, 0x15AC, 0);
        b.add_item(422, 90, 0x15AD, 0);
        b.add_button(445, 50, 0x845, 0x846, 30);
        b.add_button_type(70, 205, 0x8AF, 0x8AF, 0, GumpButtonType::Page, 4);
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        if deleted(w, self.deed) || info.button_id <= 0 || info.button_id >= 31 {
            return;
        }
        let Some(m) = mobile_of(w, sender) else {
            return;
        };
        // Use(m, bannerID)
        if let Some(i) = w.item_mut(self.deed) {
            i.ext.insert(BannerId(info.button_id));
        }
        addon::deed_on_double_click(w, self.deed, m);
    }
}

// ---- WeaponEngravingTool ----

/// `WeaponEngravingTool._usesRemaining`.
#[derive(Clone, Copy, Debug, Default)]
struct Uses(i32);

fn uses(w: &World, id: ItemId) -> i32 {
    w.item(id)
        .and_then(|i| i.ext.get::<Uses>())
        .map_or(0, |u| u.0)
}

fn set_uses_raw(w: &mut World, id: ItemId, n: i32) {
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(Uses(n));
    }
}

/// `UsesRemaining = n` (`[InvalidateProperties]`).
fn set_uses(w: &mut World, id: ItemId, n: i32) {
    if uses(w, id) != n {
        set_uses_raw(w, id, n);
        ops::invalidate_properties(w, id);
    }
}

/// `WeaponEngravingTool(uses = 10)`.
pub fn weapon_engraving_tool_new(w: &mut World, uses: i32) -> ItemId {
    let id = crate::simple::from_template(w, &WEAPON_ENGRAVING_TOOL, "Server.Items.WeaponEngravingTool");
    set_uses_raw(w, id, uses);
    id
}

/// `WeaponEngravingTool.Find(from)`.
pub fn engraving_tool_find(w: &mut World, from: MobileId) -> Option<ItemId> {
    let pack = muo_server::mobile::part1::props_b::backpack(w, from)?;
    muo_content::factory::find_item_by_type(w, pack, "Server.Items.WeaponEngravingTool")
}

/// `WeaponEngravingTool.Recharge(from, guildmaster)`.
pub fn engraving_recharge(
    w: &mut World,
    tool: ItemId,
    from: MobileId,
    guildmaster: Option<MobileId>,
) {
    use muo_server::mobile::part2::messages::say_number;
    let Some(pack) = muo_server::mobile::part1::props_b::backpack(w, from) else {
        return;
    };
    let diamond = muo_content::factory::find_item_by_type(w, pack, "Server.Items.BlueDiamond");
    if let Some(gm) = guildmaster {
        if uses(w, tool) > 0 {
            say_number(w, gm, 1076164, ""); // I can only help with this if you are carrying an engraving tool that needs repair.s
        } else if let (Some(d), true) = (
            diamond,
            muo_content::vendor::banker::withdraw(w, from, 100000),
        ) {
            muo_server::item::base::consume(w, d, 1);
            set_uses(w, tool, 10);
            say_number(w, gm, 1076165, ""); // Your weapon engraver should be good as new!
        } else {
            say_number(w, gm, 1076167, ""); // You need a 100,000 gold and a blue diamond to recharge the weapon engraver.
        }
        return;
    }
    let tinkering = muo_server::skills::value(w, from, SkillName::Tinkering);
    if tinkering == 0.0 {
        // Since you have no tinkering skill, you will need to find an NPC tinkerer to repair this for you.
        msg(w, from, 1076179);
        return;
    }
    if tinkering < 75.0 {
        // Your tinkering skill is too low to fix this yourself.  An NPC tinkerer can help you repair this for a fee.
        msg(w, from, 1076178);
        return;
    }
    let Some(diamond) = diamond else {
        msg(w, from, 1076166); // You do not have a blue diamond needed to recharge the engraving tool.
        return;
    };
    muo_server::item::base::consume(w, diamond, 1);
    if utility::random_double(&mut w.rng) < tinkering / 100.0 {
        set_uses(w, tool, 10);
        msg(w, from, 1076165); // Your weapon engraver should be good as new! ?????
    } else {
        msg(w, from, 1076175); // You cracked the diamond attempting to fix the weapon engraver.
    }
}

/// `WeaponEngravingToolConfirmGump.DisplayTo(from, engraver, guildmaster)`.
pub fn engraving_confirm_gump(
    w: &mut World,
    from: MobileId,
    tool: ItemId,
    guildmaster: Option<MobileId>,
) {
    if super::util::net_state(w, from).is_none() || deleted(w, tool) {
        return;
    }
    let g = EngravingConfirmGump { tool, guildmaster };
    send_gump_to_mobile(w, from, Box::new(DynamicGumpObj::new(200, 200, g)), false);
}

struct EngravingConfirmGump {
    tool: ItemId,
    guildmaster: Option<MobileId>,
}

impl DynamicGumpLogic for EngravingConfirmGump {
    const TYPE_NAME: &'static str =
        "Server.Items.WeaponEngravingTool+WeaponEngravingToolConfirmGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        b.set_no_close();
        b.add_page(0);
        b.add_background(0, 0, 291, 133, 0x13BE);
        b.add_image_tiled(5, 5, 280, 100, 0xA40);
        if self.guildmaster.is_some() {
            // It will cost you 100,000 gold and a blue diamond to recharge your weapon engraver with 10 charges.
            b.add_html_localized_color(9, 9, 272, 100, 1076169, 0x7FFF, false, false);
            b.add_html_localized_color(195, 109, 120, 20, 1076172, 0x7FFF, false, false); // Recharge it
        } else {
            // You will need a blue diamond to repair the tip of the engraver.  A successful repair will give the engraver 10 charges.
            b.add_html_localized_color(9, 9, 272, 100, 1076176, 0x7FFF, false, false);
            b.add_html_localized_color(195, 109, 120, 20, 1076177, 0x7FFF, false, false); // Replace the tip.
        }
        b.add_button(160, 107, 0xFB7, 0xFB8, 1);
        b.add_button(5, 107, 0xFB1, 0xFB2, 0);
        b.add_html_localized_color(40, 109, 100, 20, 1060051, 0x7FFF, false, false); // CANCEL
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        if deleted(w, self.tool) || info.button_id != 1 {
            return;
        }
        if let Some(m) = mobile_of(w, sender) {
            engraving_recharge(w, self.tool, m, self.guildmaster);
        }
    }
}

/// `WeaponEngravingToolGump`.
struct EngravingGump {
    tool: ItemId,
    target: ItemId,
}

impl StaticGumpLogic for EngravingGump {
    const TYPE_NAME: &'static str = "Server.Items.WeaponEngravingTool+WeaponEngravingToolGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut StaticGumpBuilder) {
        b.add_page(0);
        b.add_background(50, 50, 400, 300, 0xA28);
        b.add_html_localized_color(50, 70, 400, 20, 1072359, 0x0, false, false); // <CENTER>Engraving Tool</CENTER>
        // Please enter the text to add to the selected object. Leave the text area blank to remove any existing text.  Removing text does not use a charge.
        b.add_html_localized_color(75, 95, 350, 145, 1076229, 0x0, true, true);
        b.add_button(125, 300, 0x81A, 0x81B, 1);
        b.add_button(320, 300, 0x819, 0x818, 0);
        b.add_image_tiled(75, 245, 350, 40, 0xDB0);
        b.add_image_tiled(76, 245, 350, 2, 0x23C5);
        b.add_image_tiled(75, 245, 2, 40, 0x23C3);
        b.add_image_tiled(75, 285, 350, 2, 0x23C5);
        b.add_image_tiled(425, 245, 2, 42, 0x23C3);
        b.add_text_entry(75, 245, 350, 40, 0x0, 2, "");
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        if deleted(w, self.tool) || deleted(w, self.target) {
            return;
        }
        let Some(m) = mobile_of(w, sender) else {
            return;
        };
        if info.button_id != 1 {
            msg(w, m, 1072363); // The object was not engraved.
            return;
        }
        let Some(relay) = info.get_text_entry(2) else {
            return;
        };
        if relay.is_empty() {
            if let Some(d) = muo_content::items::weapon::data_mut(w, self.target) {
                d.engraved_text = None;
            }
            msg(w, m, 1072362); // You remove the engraving from the object.
        } else {
            let text: String = relay.chars().take(64).collect();
            if let Some(d) = muo_content::items::weapon::data_mut(w, self.target) {
                d.engraved_text = Some(utility::fix_html(&text));
            }
            msg(w, m, 1072361); // You engraved the object.
            ops::invalidate_properties(w, self.target);
            let n = uses(w, self.tool);
            set_uses(w, self.tool, n - 1);
            ops::invalidate_properties(w, self.tool);
        }
    }
}

/// `Server.Items.WeaponEngravingTool` (`[SerializationGenerator(0, false)]`).
pub static WEAPON_ENGRAVING_TOOL: ItemType = ItemType {
    full_name: "Server.Items.WeaponEngravingTool",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| weapon_engraving_tool_new(w, 10)),
    default_weight: crate::class_props::default_weight,
    label_number: crate::class_props::label_number, // Weapon Engraving Tool
    get_properties: |w, id, list| {
        (ITEM_BASE.get_properties)(w, id, list);
        if is_reward(w, id) {
            list.add(1076224); // 8th Year Veteran Reward
        }
        list.add_int(1060584, uses(w, id)); // uses remaining: ~1_val~
    },
    on_double_click: |w, id, from| {
        if reward_blocked(w, from, id) {
            return;
        }
        if uses(w, id) > 0 {
            msg(w, from, 1072357); // Select an object to engrave.
            begin_target(w, from, move |w, from, t| {
                if deleted(w, id) {
                    return;
                }
                match t {
                    Targeted::Entity(EntityId::Item(item))
                        if muo_content::items::weapon::is_weapon(w, item) =>
                    {
                        if super::util::net_state(w, from).is_some() && !deleted(w, item) {
                            let g = EngravingGump {
                                tool: id,
                                target: item,
                            };
                            send_gump_to_mobile(
                                w,
                                from,
                                Box::new(StaticGumpObj::new(0, 0, g)),
                                false,
                            );
                        }
                    }
                    _ => msg(w, from, 1072309), // The selected item cannot be engraved by this engraving tool.
                }
            });
            return;
        }
        let tinkering = muo_server::skills::value(w, from, SkillName::Tinkering);
        if tinkering == 0.0 {
            // Since you have no tinkering skill, you will need to find an NPC tinkerer to repair this for you.
            msg(w, from, 1076179);
            return;
        }
        if tinkering < 75.0 {
            // Your tinkering skill is too low to fix this yourself.  An NPC tinkerer can help you repair this for a fee.
            msg(w, from, 1076178);
            return;
        }
        let pack = muo_server::mobile::part1::props_b::backpack(w, from);
        if pack
            .and_then(|p| muo_content::factory::find_item_by_type(w, p, "Server.Items.BlueDiamond"))
            .is_none()
        {
            msg(w, from, 1076166); // You do not have a blue diamond needed to recharge the engraving tool.
            return;
        }
        msg(w, from, 1076163); // There are no charges left on this engraving tool.
        engraving_confirm_gump(w, from, id, None);
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(0);
        out.write_i32(uses(w, id));
        out.write_bool(is_reward(w, id));
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        let n = r.read_i32()?;
        let reward = r.read_bool()?;
        set_uses_raw(w, id, n);
        set_reward_raw(w, id, reward);
        Ok(())
    },
    ..ITEM_BASE
};

// ---- registration ----

fn mobile_of(w: &World, sender: NetStateId) -> Option<MobileId> {
    w.net_states.get(&sender).and_then(|n| n.mobile)
}

pub fn register(types: &mut TypeRegistry) {
    for t in [
        &CANNON_ADDON_COMPONENT,
        &CANNON_ADDON,
        &CANNON_DEED,
        &WALL_BANNER_COMPONENT,
        &WALL_BANNER,
        &WALL_BANNER_DEED,
        &WEAPON_ENGRAVING_TOOL,
    ] {
        types.register_item(t, &[]);
    }
}

/// `CannonAddon(direction)`: `[add CannonAddon North`.
static CANNON_DIRECTIONS: [(&str, i64); 4] = [("North", 0), ("East", 1), ("South", 2), ("West", 3)];

static CANNON_CTORS: [Ctor; 1] = [Ctor {
    access: None,
    params: &[Param {
        name: "direction",
        kind: Kind::Plain(PropType::Enum(&CANNON_DIRECTIONS)),
        default: None,
        ty: "CannonDirection",
    }],
    make: |w, v| EntityId::Item(cannon_addon_new(w, v[0].i64() as i32)),
}];

/// Every `IRewardItem` class of the folder (the base-most one of each chain).
const REWARD_CLASSES: [&str; 28] = [
    "Server.Items.BloodyPentagramAddon",
    "Server.Items.BloodyPentagramDeed",
    "Server.Items.StoneAnkh",
    "Server.Items.StoneAnkhDeed",
    "Server.Items.MinotaurStatue",
    "Server.Items.MinotaurStatueDeed",
    "Server.Items.TreeStump",
    "Server.Items.TreeStumpDeed",
    "Server.Items.RewardBrazier",
    "Server.Items.RewardBrazierDeed",
    "Server.Items.RewardPottedCactus",
    "Server.Items.PottedCactusDeed",
    "Server.Items.Banner",
    "Server.Items.BannerDeed",
    "Server.Items.FlamingHead",
    "Server.Items.FlamingHeadDeed",
    "Server.Items.MiniHouseAddon",
    "Server.Items.MiniHouseDeed",
    "Server.Items.AnkhOfSacrificeAddon",
    "Server.Items.AnkhOfSacrificeDeed",
    "Server.Items.MiningCart",
    "Server.Items.MiningCartDeed",
    "Server.Items.DecorativeShield",
    "Server.Items.DecorativeShieldDeed",
    "Server.Items.HangingSkeleton",
    "Server.Items.HangingSkeletonDeed",
    "Server.Items.CannonAddon",
    "Server.Items.CannonDeed",
];

const REWARD_CLASSES_2: [&str; 3] = [
    "Server.Items.WallBanner",
    "Server.Items.WallBannerDeed",
    "Server.Items.WeaponEngravingTool",
];

/// `TinkerGuildmaster.RechargeEntry` (6271, 6).
fn recharge_entry(
    w: &mut World,
    _vendor: MobileId,
    from: MobileId,
) -> muo_server::context_menu_entry::ContextMenuEntry {
    let mut e = muo_server::context_menu_entry::ContextMenuEntry::new(6271, 6);
    e.enabled = engraving_tool_find(w, from).is_some();
    e.on_click = Some(std::rc::Rc::new(
        move |w: &mut World, from: MobileId, target: EntityId| {
            use muo_server::mobile::part2::messages::say_number;
            let EntityId::Mobile(vendor) = target else {
                return;
            };
            if !w.core_ml() || deleted_mobile(w, vendor) {
                return;
            }
            let tool = engraving_tool_find(w, from);
            match tool.filter(|&t| uses(w, t) <= 0) {
                // I can only help with this if you are carrying an engraving tool that needs repair.
                None => say_number(w, vendor, 1076164, ""),
                Some(t) if muo_content::vendor::banker::get_balance(w, from) >= 100000 => {
                    engraving_confirm_gump(w, from, t, Some(vendor));
                }
                // You need a 100,000 gold and a blue diamond to recharge the weapon engraver.
                Some(_) => say_number(w, vendor, 1076167, ""),
            }
        },
    ));
    e
}

fn deleted_mobile(w: &World, m: MobileId) -> bool {
    w.mobile(m).is_none_or(|x| x.deleted)
}

pub(super) fn configure(w: &mut World) {
    w.resources
        .get_mut::<muo_content::vendor::VendorHooks>()
        .recharge_entry = Some(recharge_entry);
    for c in REWARD_CLASSES.iter().chain(REWARD_CLASSES_2.iter()) {
        props::add(w, c, REWARD_PROPS);
    }
    props::add(w, "Server.Items.CannonAddon", CHARGES_PROPS);
    props::add(w, "Server.Items.WallBanner", EAST_PROPS);
    props::add(w, "Server.Items.WeaponEngravingTool", USES_PROPS);
    props::add_ctors(w, "Server.Items.CannonAddon", &CANNON_CTORS);
    muo_content::items::interfaces::register_dyable(
        w,
        &WALL_BANNER_COMPONENT,
        |w, id, _from, tub| {
            if deleted(w, id) {
                return false;
            }
            if let Some(a) = addon::component_addon(w, id) {
                let h = muo_content::items::clothing::tub_dyed_hue(w, tub);
                crate::simple::hue(w, a, h);
            }
            true
        },
    );
}
