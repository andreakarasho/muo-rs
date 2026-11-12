//! The special addon classes with behavior: Heritage Items (Curtains, FruitTrees, Fountain,
//! Guillotine, HouseLadder, IronMaiden), Evil Home Decor Collection (BedOfNails, BoneCouch, HauntedMirror, the four
//! portraits, SacrificialAltar) and Holiday/GingerBreadHouseDeed. Vanity is a plain table (`addons.rs`).
//!
//! `[FlippableAddon]` classes (BedOfNails, BoneCouch, SacrificialAltar) register their `Flip(from, direction)` with
//! `muo_content::items::flip::register_addon_flip` in the `Configure` phase.
//!
//! `IDyable` of CurtainsComponent is listed for the dye tub.

use muo_content::gumps::base::{RelayInfo, StaticGumpLogic, StaticGumpObj};
use muo_content::gumps::builder::StaticGumpBuilder;
use muo_content::items::addon::{
    self, ADDON_COMPONENT, AddonClass, BASE_ADDON, BASE_ADDON_CLASS, BASE_ADDON_DEED, Comp,
    DeedClass,
};
use muo_content::items::addon_container::{
    self, BASE_ADDON_CONTAINER, BASE_ADDON_CONTAINER_CLASS, BASE_ADDON_CONTAINER_DEED, CComp,
    ContainerAddonClass, ContainerDeedClass,
};
use muo_content::mx;
use muo_content::props::{Ctor, Kind, Param, add_ctors};
use muo_content::utilities::types::{Numeric, PropType};
use muo_primitives::{Direction, Point3D, TimeSpan, utility};
use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ItemType, LootType, deps, messages, ops};
use muo_server::light_type::LightType;
use muo_server::map::MapId;
use muo_server::mobile::AccessLevel;
use muo_server::net_state::NetStateId;
use muo_server::packets::message::MessageType;
use muo_server::timer::{self, TimerHandle};
use muo_server::type_registry::{Phase, TypeRegistry};

use super::addons as sp;
use crate::addons::leaves::STONE_FOUNTAIN_ADDON;
use crate::simple::{item_id, leaf};

// ---- shared pieces ------------------------------------------------------------------------------------------

fn overhead(w: &mut World, from: MobileId, hue: i32, number: i32) {
    deps::local_overhead_message(w, from, MessageType::REGULAR, hue, number);
}

fn cur_id(w: &World, id: ItemId) -> i32 {
    w.item(id).map_or(0, |i| i.item_id)
}

fn loc_of(w: &World, id: ItemId) -> Point3D {
    w.item(id).map_or_else(Point3D::default, |i| i.location)
}

fn map_of(w: &World, id: ItemId) -> Option<MapId> {
    w.item(id).and_then(|i| i.map)
}

fn play(w: &mut World, id: ItemId, sound: i32) {
    let (loc, map) = (loc_of(w, id), map_of(w, id));
    muo_server::effects::play_sound(w, loc, map, sound);
}

/// Blood on the ground around `(cx, cy, cz)`: per spot `x`, `y`, a fit check (then the average z), a `Blood`.
fn spill_blood(w: &mut World, map: MapId, cx: i32, cy: i32, cz: i32, amount: i32) {
    for _ in 0..amount {
        let x = cx + utility::random_min_max(&mut w.rng, -1, 1);
        let y = cy + utility::random_min_max(&mut w.rng, -1, 1);
        let mut z = cz;
        if !muo_server::map::fit::can_fit(w, map, x, y, z, 1, false, false, true) {
            z = w.maps.get_average_z(map, x, y);
            if !muo_server::map::fit::can_fit(w, map, x, y, z, 1, false, false, true) {
                continue;
            }
        }
        let graphic = utility::random_min_max(&mut w.rng, 0x122C, 0x122F);
        let blood = crate::misc::basic::blood_new_with_id(w, &crate::misc::basic::BLOOD, graphic);
        ops::move_to_world_map(w, blood, Point3D::new(x, y, z), Some(map));
    }
}

/// The deed of a blessed addon: `LootType = LootType.Blessed`.
macro_rules! blessed_deed {
    ($ty:ident, $cls:ident, $full:literal, $addon:expr, $label:expr) => {
        static $cls: DeedClass = DeedClass {
            addon: $addon,
            select_direction: None,
        };
        leaf! {
            #[doc = concat!("`", $full, "`.")]
            $ty = $full : BASE_ADDON_DEED, enc;
            construct: Some(|w| {
                let id = addon::new_deed(w, &$ty);
                ops::set_loot_type(w, id, LootType::Blessed);
                id
            }),
            family: Some(&$cls),
            label_number: |_, _| $label,
        }
    };
}

// ---- Guillotine ---------------------------------------------------------------------------------------------

fn guillotine_deactivate(w: &mut World, c: ItemId) {
    let next = match cur_id(w, c) {
        0x1269 => 0x1260,
        0x1260 => 0x125E,
        0x1247 => 0x1246,
        0x1246 => 0x1230,
        other => other,
    };
    item_id(w, c, next);
}

/// Blood, a scream, the message and the damage the guillotine and the iron maiden share.
fn gore(
    w: &mut World,
    c: ItemId,
    from: MobileId,
    female_sound: (i32, i32),
    male_sound: (i32, i32),
) {
    let amount = utility::random_min_max(&mut w.rng, 3, 7);
    if let Some(map) = map_of(w, c) {
        let l = loc_of(w, c);
        spill_blood(w, map, l.x, l.y, l.z, amount);
    }
    let female = w.mobile(from).is_some_and(|m| m.female);
    let (lo, hi) = if female { female_sound } else { male_sound };
    let sound = utility::random_min_max(&mut w.rng, lo, hi);
    mx::play_sound(w, from, sound);
    overhead(w, from, 0, 501777); // Hmm... you suspect that if you used this again, it might hurt.
    let dmg = utility::dice(&mut w.rng, 2, 10, 5);
    muo_content::spells::helper::damage_full(w, None, TimeSpan::ZERO, from, None, f64::from(dmg));
}

fn guillotine_activate(w: &mut World, c: ItemId, from: MobileId) {
    let cur = cur_id(w, c);
    item_id(
        w,
        c,
        if matches!(cur, 0x125E | 0x1269 | 0x1260) {
            0x1269
        } else {
            0x1247
        },
    );
    gore(w, c, from, (0x150, 0x153), (0x15A, 0x15D));
    timer::start(
        w,
        TimeSpan::from_secs_f64(0.5),
        TimeSpan::from_secs_f64(0.5),
        2,
        move |w| guillotine_deactivate(w, c),
    );
}

fn guillotine_on_component_used(w: &mut World, a: ItemId, c: ItemId, from: MobileId) {
    let loc = loc_of(w, a);
    if !mx::in_range_point(w, from, loc, 2) {
        overhead(w, from, 0x3B2, 1019045); // I can't reach that.
        return;
    }
    if utility::random_bool(&mut w.rng) {
        muo_server::mobile::part1::lifecycle::set_location(w, from, loc);
        timer::start(
            w,
            TimeSpan::from_secs_f64(0.5),
            TimeSpan::ZERO,
            1,
            move |w| guillotine_activate(w, c, from),
        );
    } else {
        overhead(w, from, 0, 501777); // Hmm... you suspect that if you used this again, it might hurt.
    }
}

/// `GuillotineAddon()`.
pub fn guillotine_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &GUILLOTINE_ADDON);
    addon::add_components(
        w,
        id,
        &[(Comp::Custom(sp::guillotine_component_new), 0, 0, 0)],
    );
    id
}

static GUILLOTINE_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(|w| {
        let id = addon::new_deed(w, &GUILLOTINE_DEED);
        ops::set_loot_type(w, id, LootType::Blessed);
        id
    }),
    on_component_used: guillotine_on_component_used,
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.GuillotineAddon`.
    GUILLOTINE_ADDON = "Server.Items.GuillotineAddon" : BASE_ADDON, enc;
    construct: Some(guillotine_addon_new),
    family: Some(&GUILLOTINE_ADDON_CLASS),
}
blessed_deed!(
    GUILLOTINE_DEED,
    GUILLOTINE_DEED_CLASS,
    "Server.Items.GuillotineDeed",
    |w, _| guillotine_addon_new(w),
    1024656
);

// ---- IronMaiden ---------------------------------------------------------------------------------------------

fn iron_maiden_activate(w: &mut World, c: ItemId, from: MobileId) {
    item_id(w, c, cur_id(w, c) + 1);
    if cur_id(w, c) < 0x124D {
        return;
    }
    gore(w, c, from, (0x150, 0x153), (0x15A, 0x15D));
    timer::start(
        w,
        TimeSpan::from_secs_f64(1.0),
        TimeSpan::ZERO,
        1,
        move |w| item_id(w, c, 0x1249),
    );
}

fn iron_maiden_on_component_used(w: &mut World, a: ItemId, c: ItemId, from: MobileId) {
    let loc = ops::get_world_location(w, a);
    if !mx::in_range_point(w, from, loc, 2) || !deps::in_los_point(w, from, loc) {
        overhead(w, from, 0x3B2, 1019045); // I can't reach that.
        return;
    }
    if utility::random_bool(&mut w.rng) {
        let own = loc_of(w, a);
        muo_server::mobile::part1::lifecycle::set_location(w, from, own);
        item_id(w, c, 0x124A);
        timer::start(
            w,
            TimeSpan::from_secs_f64(0.5),
            TimeSpan::from_secs_f64(0.5),
            3,
            move |w| iron_maiden_activate(w, c, from),
        );
    } else {
        overhead(w, from, 0, 501777); // Hmm... you suspect that if you used this again, it might hurt.
    }
}

/// `IronMaidenAddon()` (no `[Constructible]`).
pub fn iron_maiden_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &IRON_MAIDEN_ADDON);
    addon::add_components(w, id, &[(Comp::Localized(0x1249, 1076288), 0, 0, 0)]);
    id
}

static IRON_MAIDEN_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(|w| {
        let id = addon::new_deed(w, &IRON_MAIDEN_DEED);
        ops::set_loot_type(w, id, LootType::Blessed);
        id
    }),
    on_component_used: iron_maiden_on_component_used,
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.IronMaidenAddon`.
    IRON_MAIDEN_ADDON = "Server.Items.IronMaidenAddon" : BASE_ADDON, enc;
    family: Some(&IRON_MAIDEN_ADDON_CLASS),
}
blessed_deed!(
    IRON_MAIDEN_DEED,
    IRON_MAIDEN_DEED_CLASS,
    "Server.Items.IronMaidenDeed",
    |w, _| iron_maiden_addon_new(w),
    1076288
);

// ---- Fountain -----------------------------------------------------------------------------------------------

/// `FountainAddon()`: the `StoneFountainAddon` components.
pub fn fountain_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &FOUNTAIN_ADDON);
    addon::add_components(
        w,
        id,
        &[
            (Comp::Plain(5937), -2, 1, 0),
            (Comp::Plain(5938), -1, 1, 0),
            (Comp::Plain(5939), 0, 1, 0),
            (Comp::Plain(5940), 1, 1, 0),
            (Comp::Plain(5941), 1, 0, 0),
            (Comp::Plain(5942), 1, -1, 0),
            (Comp::Plain(5943), 1, -2, 0),
            (Comp::Plain(5944), 0, -2, 0),
            (Comp::Plain(5945), 0, -1, 0),
            (Comp::Plain(5946), 0, 0, 0),
            (Comp::Plain(5947), -1, 0, 0),
            (Comp::Plain(5948), -2, 0, 0),
            (Comp::Plain(5949), -2, -1, 0),
            (Comp::Plain(5950), -1, -1, 0),
            (Comp::Plain(5951), -1, -2, 0),
            (Comp::Plain(5953), -2, -2, 0),
        ],
    );
    id
}

static FOUNTAIN_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(|w| {
        let id = addon::new_deed(w, &FOUNTAIN_DEED);
        ops::set_loot_type(w, id, LootType::Blessed);
        id
    }),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.FountainAddon : StoneFountainAddon`.
    FOUNTAIN_ADDON = "Server.Items.FountainAddon" : STONE_FOUNTAIN_ADDON, enc;
    construct: Some(fountain_addon_new),
    family: Some(&FOUNTAIN_ADDON_CLASS),
}
blessed_deed!(
    FOUNTAIN_DEED,
    FOUNTAIN_DEED_CLASS,
    "Server.Items.FountainDeed",
    |w, _| fountain_addon_new(w),
    1076283
);

// ---- Curtains -----------------------------------------------------------------------------------------------

#[derive(Clone, Copy, Default)]
struct CurtainData {
    closed_id: i32,
}

pub fn closed_id(w: &World, c: ItemId) -> i32 {
    w.item(c)
        .and_then(|i| i.ext.get::<CurtainData>())
        .map_or(0, |d| d.closed_id)
}

pub fn set_closed_id(w: &mut World, c: ItemId, v: i32) {
    if let Some(i) = w.item_mut(c) {
        i.ext.insert(CurtainData { closed_id: v });
    }
}

fn curtains_on_double_click(w: &mut World, id: ItemId, from: MobileId) {
    (ADDON_COMPONENT.on_double_click)(w, id, from); // base.OnDoubleClick(from)
    let Some(a) = addon::component_addon(w, id) else {
        return;
    };
    let loc = loc_of(w, id);
    if !mx::in_range_point(w, from, loc, 1) {
        return;
    }
    for c in addon::components(w, a) {
        if w.item(c).is_some_and(|i| i.ty.is(&CURTAINS_COMPONENT)) {
            let (open, closed) = (cur_id(w, c), closed_id(w, c));
            item_id(w, c, closed);
            set_closed_id(w, c, open);
        } else {
            overhead(w, from, 0x3B2, 1019045); // I can't reach that.
        }
    }
}

/// `CurtainsComponent(itemID, closedID)`.
pub fn curtains_component_new(w: &mut World, item_id: i32, closed: i32) -> ItemId {
    let id = addon::new_component(w, &CURTAINS_COMPONENT, item_id);
    set_closed_id(w, id, closed);
    id
}

/// `Server.Items.CurtainsComponent` (`IDyable`).
pub static CURTAINS_COMPONENT: ItemType = ItemType {
    full_name: "Server.Items.CurtainsComponent",
    parent: Some(&ADDON_COMPONENT),
    construct: None,
    label_number: |_, _| 1076280, // Curtains
    display_weight: |_, _| false,
    on_double_click: curtains_on_double_click,
    serialize: |w, id, out| {
        (ADDON_COMPONENT.serialize)(w, id, out);
        out.write_encoded_int(0);
        out.write_i32(closed_id(w, id));
    },
    deserialize: |w, id, r| {
        (ADDON_COMPONENT.deserialize)(w, id, r)?;
        r.read_encoded_int()?;
        let v = r.read_i32()?;
        set_closed_id(w, id, v);
        Ok(())
    },
    ..ADDON_COMPONENT
};

/// `CurtainsAddon(east)`.
pub fn curtains_addon_new(w: &mut World, east: bool) -> ItemId {
    let id = addon::new_addon(w, &CURTAINS_ADDON);
    let rows: [(i32, i32, i32, i32); 4] = if east {
        [
            (0x3D9E, 0x3DA8, 0, -1),
            (0x3DAC, 0x3DAE, 0, 0),
            (0x3DA0, 0x3DA6, 0, 2),
            (0x3D9F, 0x3DA7, 0, 1),
        ]
    } else {
        [
            (0x3D9C, 0x3DAD, 0, 0),
            (0x3D9D, 0x3DA3, -1, 0),
            (0x3DA1, 0x3DA5, 2, 0),
            (0x3DAB, 0x3DA4, 1, 0),
        ]
    };
    for (open, closed, x, y) in rows {
        let c = curtains_component_new(w, open, closed);
        addon::add_component(w, id, c, x, y, 0);
    }
    id
}

static CURTAINS_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(|w| {
        let id = addon::new_deed(w, &CURTAINS_DEED);
        ops::set_loot_type(w, id, LootType::Blessed);
        id
    }),
    retain_deed_hue: true,
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.CurtainsAddon` (`[Constructible] CurtainsAddon(bool east)`: see the constructor table).
    CURTAINS_ADDON = "Server.Items.CurtainsAddon" : BASE_ADDON, enc;
    family: Some(&CURTAINS_ADDON_CLASS),
}

static CURTAINS_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, deed| curtains_addon_new(w, addon::deed_east(w, deed)),
    select_direction: Some(1076581),
};
leaf! {
    /// `Server.Items.CurtainsDeed`.
    CURTAINS_DEED = "Server.Items.CurtainsDeed" : BASE_ADDON_DEED, enc;
    construct: Some(|w| {
        let id = addon::new_deed(w, &CURTAINS_DEED);
        ops::set_loot_type(w, id, LootType::Blessed);
        id
    }),
    family: Some(&CURTAINS_DEED_CLASS),
    on_double_click: addon::direction_deed_on_double_click,
    label_number: |_, _| 1076280,
}

// ---- HouseLadder --------------------------------------------------------------------------------------------

/// `HouseLadderAddon(type)`; a `type` outside `0..=7` makes an addon with no components.
pub fn house_ladder_addon_new(w: &mut World, kind: i32) -> ItemId {
    let id = addon::new_addon(w, &HOUSE_LADDER_ADDON);
    let l = |item, label, x, y, z| (Comp::Localized(item, label), x, y, z);
    let rows: Vec<(Comp, i32, i32, i32)> = match kind {
        0 => vec![
            l(0x3DB2, 1076791, 0, 0, 0),
            l(0x3F28, 1076791, 0, 1, 28),
            l(0x3DB4, 1076791, 0, 2, 20),
        ],
        1 => vec![
            l(0x3DB3, 1076791, 0, 0, 0),
            l(0x3F28, 1076791, 1, 0, 28),
            l(0x3DB5, 1076791, 2, 0, 20),
        ],
        2 => vec![
            l(0x2FDF, 1076791, 0, 0, 0),
            l(0x3F28, 1076791, 0, -1, 28),
            l(0x3DB6, 1076791, 0, -2, 20),
        ],
        3 => vec![
            l(0x2FDE, 1076791, 0, 0, 0),
            l(0x3F28, 1076791, -1, 0, 28),
            l(0x3DB7, 1076791, -2, 0, 20),
        ],
        4 => vec![l(0x3DB2, 1076287, 0, 0, 0)],
        5 => vec![l(0x3DB3, 1076287, 0, 0, 0)],
        6 => vec![l(0x2FDF, 1076287, 0, 0, 0)],
        7 => vec![l(0x2FDE, 1076287, 0, 0, 0)],
        _ => Vec::new(),
    };
    addon::add_components(w, id, &rows);
    id
}

#[derive(Clone, Copy, Default)]
struct LadderDeedData {
    kind: i32,
}

fn ladder_kind(w: &World, deed: ItemId) -> i32 {
    w.item(deed)
        .and_then(|i| i.ext.get::<LadderDeedData>())
        .map_or(0, |d| d.kind)
}

struct HouseLadderGump {
    deed: ItemId,
}

impl StaticGumpLogic for HouseLadderGump {
    const TYPE_NAME: &'static str = "Server.Items.HouseLadderDeed+InternalGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut StaticGumpBuilder) {
        b.add_page(0);
        b.add_background(0, 0, 273, 324, 0x13BE);
        b.add_image_tiled(10, 10, 253, 20, 0xA40);
        b.add_image_tiled(10, 40, 253, 244, 0xA40);
        b.add_image_tiled(10, 294, 253, 20, 0xA40);
        b.add_alpha_region(10, 10, 253, 304);
        b.add_button(10, 294, 0xFB1, 0xFB2, 0);
        b.add_html_localized_color(45, 296, 450, 20, 1060051, 0x7FFF, false, false); // CANCEL
        // Please select your ladder position.  <br>Use the ladders marked (castle) <br> for accessing the tops of keeps <br> and castles.
        b.add_html_localized_color(14, 12, 273, 20, 1076780, 0x7FFF, false, false);
        b.add_page(1);
        let labels = [
            1076794, 1076795, 1076792, 1076793, 1075386, 1075387, 1075389, 1075390,
        ];
        for (i, label) in labels.into_iter().enumerate() {
            let y = 49 + 24 * i as i32;
            b.add_button(19, y, 0x845, 0x846, i as i32 + 1);
            b.add_html_localized_color(44, y - 2, 213, 20, label, 0x7FFF, false, false);
        }
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        if ops::is_deleted(w, self.deed) || !(1..=8).contains(&info.button_id) {
            return;
        }
        if let Some(i) = w.item_mut(self.deed) {
            i.ext.insert(LadderDeedData {
                kind: info.button_id - 1,
            });
        }
        if let Some(m) = w.net_states.get(&sender).and_then(|n| n.mobile) {
            addon::deed_on_double_click(w, self.deed, m); // SendTarget: base.OnDoubleClick
        }
    }
}

fn ladder_deed_on_double_click(w: &mut World, id: ItemId, from: MobileId) {
    let backpack = w.mobile(from).and_then(|m| m.backpack).map(EntityId::Item);
    if ops::is_child_of(w, id, backpack) {
        let gump = StaticGumpObj::new(60, 36, HouseLadderGump { deed: id });
        muo_content::gumps::send_gump_to_mobile(w, from, Box::new(gump), true);
    } else {
        deps::send_localized_message(w, from, 1062334); // This item must be in your backpack to be used.
    }
}

static HOUSE_LADDER_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(|w| {
        let id = addon::new_deed(w, &HOUSE_LADDER_DEED);
        ops::set_loot_type(w, id, LootType::Blessed);
        id
    }),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.HouseLadderAddon` (`[Constructible] HouseLadderAddon(int type)`: see the constructor table).
    HOUSE_LADDER_ADDON = "Server.Items.HouseLadderAddon" : BASE_ADDON, enc;
    family: Some(&HOUSE_LADDER_ADDON_CLASS),
}

static HOUSE_LADDER_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, deed| house_ladder_addon_new(w, ladder_kind(w, deed)),
    select_direction: None,
};
leaf! {
    /// `Server.Items.HouseLadderDeed`.
    HOUSE_LADDER_DEED = "Server.Items.HouseLadderDeed" : BASE_ADDON_DEED, enc;
    construct: Some(|w| {
        let id = addon::new_deed(w, &HOUSE_LADDER_DEED);
        ops::set_loot_type(w, id, LootType::Blessed);
        id
    }),
    family: Some(&HOUSE_LADDER_DEED_CLASS),
    on_double_click: ladder_deed_on_double_click,
    label_number: |_, _| 1076287,
}

// ---- FruitTrees ---------------------------------------------------------------------------------------------

pub fn fruits(w: &World, id: ItemId) -> i32 {
    w.item(id)
        .and_then(|i| i.ext.get::<FruitData>())
        .map_or(0, |d| d.fruits)
}

#[derive(Clone, Copy, Default)]
pub struct FruitData {
    pub fruits: i32,
}

/// `Fruits = value`: not below 0.
pub fn set_fruits(w: &mut World, id: ItemId, value: i32) {
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(FruitData {
            fruits: value.max(0),
        });
    }
}

fn fruit_respawn(w: &mut World, id: ItemId) {
    // `_fruits = Utility.RandomMinMax(1, 4)` on the field: no clamp, no dirty mark.
    let n = utility::random_min_max(&mut w.rng, 1, 4);
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(FruitData { fruits: n });
    }
}

fn fruit_on_component_used(w: &mut World, a: ItemId, c: ItemId, from: MobileId) {
    let loc = loc_of(w, c);
    if !mx::in_range_point(w, from, loc, 2) {
        overhead(w, from, 0x3B2, 1019045); // I can't reach that.
        return;
    }
    if fruits(w, a) <= 0 {
        mx::msg(w, from, 501017); // There is no more fruit on this tree
        return;
    }
    let name = if w.item(a).is_some_and(|i| i.ty.is(&PEACH_TREE_ADDON)) {
        "Server.Items.Peach"
    } else {
        "Server.Items.Apple"
    };
    let Some(fruit) = muo_content::factory::make_item(w, name) else {
        return;
    };
    if !muo_server::mobile::part2::items::place_in_backpack(w, from, fruit) {
        if let Some(ty) = w.item(fruit).map(|i| i.ty) {
            (ty.delete)(w, fruit);
        }
        mx::msg(w, from, 501015); // There is no room in your backpack for the fruit.
    } else {
        let left = fruits(w, a) - 1;
        set_fruits(w, a, left);
        if left == 0 {
            timer::start(w, TimeSpan::from_mins(30), TimeSpan::ZERO, 1, move |w| {
                fruit_respawn(w, a)
            });
        }
        mx::msg(w, from, 501016); // You pick some fruit and put it in your backpack.
    }
}

static BASE_FRUIT_TREE_CLASS: AddonClass = AddonClass {
    on_component_used: fruit_on_component_used,
    ..BASE_ADDON_CLASS
};

/// `BaseFruitTreeAddon()`: the respawn timer.
fn fruit_tree_init(w: &mut World, id: ItemId) {
    timer::start(w, TimeSpan::from_mins(5), TimeSpan::ZERO, 1, move |w| {
        fruit_respawn(w, id)
    });
}

/// `Server.Items.BaseFruitTreeAddon` (abstract).
pub static BASE_FRUIT_TREE_ADDON: ItemType = ItemType {
    full_name: "Server.Items.BaseFruitTreeAddon",
    parent: Some(&BASE_ADDON),
    construct: None,
    family: Some(&BASE_FRUIT_TREE_CLASS),
    serialize: |w, id, out| {
        (BASE_ADDON.serialize)(w, id, out);
        out.write_encoded_int(0);
        out.write_i32(fruits(w, id));
    },
    deserialize: |w, id, r| {
        (BASE_ADDON.deserialize)(w, id, r)?;
        r.read_encoded_int()?;
        let n = r.read_i32()?;
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(FruitData { fruits: n });
        }
        if n == 0 {
            fruit_respawn(w, id); // [AfterDeserialization]
        }
        Ok(())
    },
    ..BASE_ADDON
};

macro_rules! fruit_tree {
    ($addon:ident, $acls:ident, $anew:ident, $deed:ident, $dcls:ident, $afull:literal, $dfull:literal,
     $c1:expr, $c2:expr, $label:expr) => {
        pub fn $anew(w: &mut World) -> ItemId {
            let id = addon::new_addon(w, &$addon);
            fruit_tree_init(w, id);
            addon::add_components(
                w,
                id,
                &[
                    (Comp::Localized($c1, $label), 0, 0, 0),
                    (Comp::Localized($c2, $label), 0, 0, 0),
                ],
            );
            id
        }
        static $acls: AddonClass = AddonClass {
            deed: Some(|w| {
                let id = addon::new_deed(w, &$deed);
                ops::set_loot_type(w, id, LootType::Blessed);
                id
            }),
            ..BASE_FRUIT_TREE_CLASS
        };
        leaf! {
            #[doc = concat!("`", $afull, "`.")]
            $addon = $afull : BASE_FRUIT_TREE_ADDON, enc;
            construct: Some($anew),
            family: Some(&$acls),
        }
        blessed_deed!($deed, $dcls, $dfull, |w, _| $anew(w), $label);
    };
}

fruit_tree!(
    APPLE_TREE_ADDON,
    APPLE_TREE_ADDON_CLASS,
    apple_tree_addon_new,
    APPLE_TREE_DEED,
    APPLE_TREE_DEED_CLASS,
    "Server.Items.AppleTreeAddon",
    "Server.Items.AppleTreeDeed",
    0xD98,
    0x3124,
    1076269
);
fruit_tree!(
    PEACH_TREE_ADDON,
    PEACH_TREE_ADDON_CLASS,
    peach_tree_addon_new,
    PEACH_TREE_DEED,
    PEACH_TREE_DEED_CLASS,
    "Server.Items.PeachTreeAddon",
    "Server.Items.PeachTreeDeed",
    0xD9C,
    0x3123,
    1076270
);

// ---- BedOfNails ---------------------------------------------------------------------------------------------

#[derive(Clone, Copy, Default)]
struct NailsData {
    timer: Option<TimerHandle>,
}

fn nails_tick_body(w: &mut World, m: MobileId, loc: &mut Point3D) -> bool {
    let (map, deleted, alive, mloc) = (
        mx::map(w, m),
        mx::deleted(w, m),
        mx::alive(w, m),
        mx::location(w, m),
    );
    match map {
        Some(map) if !deleted && alive && map != MapId::INTERNAL => {
            if *loc != mloc {
                let amount = utility::random_min_max(&mut w.rng, 0, 7);
                spill_blood(w, map, mloc.x, mloc.y, mloc.z, amount);
                *loc = mloc;
            }
            true
        }
        _ => false,
    }
}

fn bed_of_nails_on_move_over(w: &mut World, a: ItemId, m: MobileId) -> bool {
    let (alive, level, hidden, player, female) = match w.mobile(m) {
        Some(x) => (
            mx::alive(w, m),
            x.access_level,
            x.hidden,
            x.player,
            x.female,
        ),
        None => return true,
    };
    if alive && (level == AccessLevel::Player || !hidden) {
        if player {
            let (lo, hi) = if female {
                (0x53B, 0x53D)
            } else {
                (0x53E, 0x540)
            };
            let sound = utility::random_min_max(&mut w.rng, lo, hi);
            play(w, a, sound);
        }
        let running = w
            .item(a)
            .and_then(|i| i.ext.get::<NailsData>())
            .and_then(|d| d.timer)
            .is_some_and(|t| t.running(w));
        if !running {
            let mut loc = Point3D::ZERO;
            let t = timer::start(
                w,
                TimeSpan::ZERO,
                TimeSpan::from_secs_f64(1.0),
                5,
                move |w| {
                    if !nails_tick_body(w, m, &mut loc)
                        && let Some(h) = timer::current(w)
                    {
                        timer::stop(w, h);
                    }
                },
            );
            if let Some(i) = w.item_mut(a) {
                i.ext.insert(NailsData { timer: Some(t) });
            }
        }
    }
    true
}

fn bed_of_nails_component_on_move_over(w: &mut World, id: ItemId, m: MobileId) -> bool {
    let allow = muo_server::item::base::on_move_over_base(w, id, m);
    if allow
        && let Some(a) = addon::component_addon(w, id)
        && w.item(a).is_some_and(|i| i.ty.is(&BED_OF_NAILS_ADDON))
    {
        bed_of_nails_on_move_over(w, a, m);
    }
    allow
}

/// `BedOfNailsComponent(itemID)`.
pub fn bed_of_nails_component_new(w: &mut World, item_id: i32) -> ItemId {
    addon::new_component(w, &BED_OF_NAILS_COMPONENT, item_id)
}

leaf! {
    /// `Server.Items.BedOfNailsComponent`.
    BED_OF_NAILS_COMPONENT = "Server.Items.BedOfNailsComponent" : ADDON_COMPONENT, enc;
    label_number: |_, _| 1074801,
    on_move_over: bed_of_nails_component_on_move_over,
}

fn bed_of_nails_flip(w: &mut World, _from: MobileId, a: ItemId, direction: Direction) {
    let rows: &[(i32, i32, i32)] = if direction == Direction::EAST {
        &[(0x2A89, 0, 0), (0x2A8A, -1, 0)]
    } else if direction == Direction::SOUTH {
        &[(0x2A81, 0, 0), (0x2A82, 0, -1)]
    } else {
        return;
    };
    for &(item, x, y) in rows {
        let c = bed_of_nails_component_new(w, item);
        addon::add_component(w, a, c, x, y, 0);
    }
}

/// `BedOfNailsAddon()`.
pub fn bed_of_nails_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &BED_OF_NAILS_ADDON);
    ops::set_direction(w, id, Direction::SOUTH);
    addon::add_components(
        w,
        id,
        &[
            (Comp::Typed(bed_of_nails_component_new, 0x2A81), 0, 0, 0),
            (Comp::Typed(bed_of_nails_component_new, 0x2A82), 0, -1, 0),
        ],
    );
    id
}

static BED_OF_NAILS_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(|w| {
        let id = addon::new_deed(w, &BED_OF_NAILS_DEED);
        ops::set_loot_type(w, id, LootType::Blessed);
        id
    }),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.BedOfNailsAddon` (`[FlippableAddon(South, East)]`).
    BED_OF_NAILS_ADDON = "Server.Items.BedOfNailsAddon" : BASE_ADDON, enc;
    construct: Some(bed_of_nails_addon_new),
    family: Some(&BED_OF_NAILS_ADDON_CLASS),
    on_move_over: bed_of_nails_on_move_over,
}
blessed_deed!(
    BED_OF_NAILS_DEED,
    BED_OF_NAILS_DEED_CLASS,
    "Server.Items.BedOfNailsDeed",
    |w, _| bed_of_nails_addon_new(w),
    1074801
);

// ---- BoneCouch ----------------------------------------------------------------------------------------------

fn bone_couch_flip(w: &mut World, _from: MobileId, a: ItemId, direction: Direction) {
    let rows: &[(i32, i32, i32)] = if direction == Direction::EAST {
        &[(0x2A80, 0, 0), (0x2A7F, 0, 1)]
    } else if direction == Direction::SOUTH {
        &[(0x2A5A, 0, 0), (0x2A5B, -1, 0)]
    } else {
        return;
    };
    for &(item, x, y) in rows {
        let c = sp::bone_couch_component_new(w, item);
        addon::add_component(w, a, c, x, y, 0);
    }
}

/// `BoneCouchAddon()`.
pub fn bone_couch_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &BONE_COUCH_ADDON);
    ops::set_direction(w, id, Direction::SOUTH);
    addon::add_components(
        w,
        id,
        &[
            (Comp::Typed(sp::bone_couch_component_new, 0x2A5A), 0, 0, 0),
            (Comp::Typed(sp::bone_couch_component_new, 0x2A5B), -1, 0, 0),
        ],
    );
    id
}

static BONE_COUCH_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(|w| {
        let id = addon::new_deed(w, &BONE_COUCH_DEED);
        ops::set_loot_type(w, id, LootType::Blessed);
        id
    }),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.BoneCouchAddon` (`[FlippableAddon(South, East)]`).
    BONE_COUCH_ADDON = "Server.Items.BoneCouchAddon" : BASE_ADDON, enc;
    construct: Some(bone_couch_addon_new),
    family: Some(&BONE_COUCH_ADDON_CLASS),
}
blessed_deed!(
    BONE_COUCH_DEED,
    BONE_COUCH_DEED_CLASS,
    "Server.Items.BoneCouchDeed",
    |w, _| bone_couch_addon_new(w),
    1074477
);

// ---- HauntedMirror ------------------------------------------------------------------------------------------

fn in_range2(a: Point3D, b: Point3D) -> bool {
    utility::in_range(a.x, a.y, b.x, b.y, 2)
}

/// The mobile test the decor components share: alive, a player, and visible unless staff.
fn watches(w: &World, m: MobileId) -> bool {
    mx::alive(w, m)
        && w.mobile(m)
            .is_some_and(|x| x.player && (x.access_level == AccessLevel::Player || !x.hidden))
}

fn haunted_mirror_on_movement(w: &mut World, id: ItemId, m: MobileId, old: Point3D) {
    if !watches(w, m) {
        return;
    }
    let loc = loc_of(w, id);
    let (in_old, in_new) = (in_range2(old, loc), in_range2(mx::location(w, m), loc));
    let cur = cur_id(w, id);
    if !in_old && in_new {
        if matches!(cur, 0x2A7B | 0x2A7D) {
            let sound = utility::random_min_max(&mut w.rng, 0x551, 0x553);
            play(w, id, sound);
            item_id(w, id, cur + 1);
        }
    } else if in_old && !in_new && matches!(cur, 0x2A7C | 0x2A7E) {
        item_id(w, id, cur - 1);
    }
}

/// `HaunterMirrorComponent()`.
pub fn haunter_mirror_component_new(w: &mut World) -> ItemId {
    addon::new_component(w, &HAUNTER_MIRROR_COMPONENT, 0x2A7B)
}

leaf! {
    /// `Server.Items.HaunterMirrorComponent` (`[Flippable(0x2A7B, 0x2A7D)]`).
    HAUNTER_MIRROR_COMPONENT = "Server.Items.HaunterMirrorComponent" : ADDON_COMPONENT, enc;
    construct: None,
    make: Some(haunter_mirror_component_new),
    label_number: |_, _| 1074800,
    handles_on_movement: |_, _| true,
    on_movement: haunted_mirror_on_movement,
}

/// `HaunterMirrorAddon()`.
pub fn haunter_mirror_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &HAUNTER_MIRROR_ADDON);
    addon::add_components(
        w,
        id,
        &[(Comp::Custom(haunter_mirror_component_new), 0, 0, 0)],
    );
    id
}

static HAUNTER_MIRROR_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(|w| {
        let id = addon::new_deed(w, &HAUNTER_MIRROR_DEED);
        ops::set_loot_type(w, id, LootType::Blessed);
        id
    }),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.HaunterMirrorAddon`.
    HAUNTER_MIRROR_ADDON = "Server.Items.HaunterMirrorAddon" : BASE_ADDON, enc;
    construct: Some(haunter_mirror_addon_new),
    family: Some(&HAUNTER_MIRROR_ADDON_CLASS),
}
blessed_deed!(
    HAUNTER_MIRROR_DEED,
    HAUNTER_MIRROR_DEED_CLASS,
    "Server.Items.HaunterMirrorDeed",
    |w, _| haunter_mirror_addon_new(w),
    1074800
);

// ---- Portraits ----------------------------------------------------------------------------------------------

/// The `OnDoubleClick` of the portraits: a sound within 2 tiles, else "I can't reach that."
fn portrait_sound(w: &mut World, id: ItemId, from: MobileId, lo: i32, hi: i32) {
    let loc = loc_of(w, id);
    if in_range2(loc, mx::location(w, from)) {
        let sound = utility::random_min_max(&mut w.rng, lo, hi);
        play(w, id, sound);
    } else {
        overhead(w, from, 0x3B2, 1019045); // I can't reach that.
    }
}

#[derive(Clone, Copy, Default)]
struct PortraitTimer {
    timer: Option<TimerHandle>,
}

fn set_portrait_timer(w: &mut World, id: ItemId, t: Option<TimerHandle>) {
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(PortraitTimer { timer: t });
    }
}

/// `OnAfterDelete`: the base's, then `_timer?.Stop()`.
fn portrait_on_after_delete(w: &mut World, id: ItemId) {
    (ADDON_COMPONENT.on_after_delete)(w, id);
    if let Some(t) = w
        .item(id)
        .and_then(|i| i.ext.get::<PortraitTimer>())
        .and_then(|d| d.timer)
    {
        timer::stop(w, t);
    }
}

/// A repeating-timer portrait component (Disturbing, Unsettling): the timer starts on construction and on load.
macro_rules! leaf_with_timer {
    ($ty:ident, $full:literal, $new:ident, $label:expr, $change:path) => {
        #[doc = concat!("`", $full, "`.")]
        pub static $ty: ItemType = ItemType {
            full_name: $full,
            parent: Some(&ADDON_COMPONENT),
            construct: None,
            make: Some($new),
            label_number: |_, _| $label,
            on_double_click: |w, id, from| portrait_sound(w, id, from, 0x567, 0x568),
            on_after_delete: portrait_on_after_delete,
            serialize: |w, id, out| {
                (ADDON_COMPONENT.serialize)(w, id, out);
                out.write_encoded_int(0);
            },
            deserialize: |w, id, r| {
                (ADDON_COMPONENT.deserialize)(w, id, r)?;
                r.read_encoded_int()?;
                start_repeating(w, id, $change); // [AfterDeserialization]
                Ok(())
            },
            ..ADDON_COMPONENT
        };
    };
}

// CreepyPortrait

fn creepy_portrait_on_movement(w: &mut World, id: ItemId, m: MobileId, old: Point3D) {
    if !watches(w, m) {
        return;
    }
    let loc = loc_of(w, id);
    let (in_old, in_new) = (in_range2(old, loc), in_range2(mx::location(w, m), loc));
    let cur = cur_id(w, id);
    if !in_old && in_new {
        if matches!(cur, 0x2A69 | 0x2A6D) {
            item_id(w, id, cur_id(w, id) + 1);
            timer::start(
                w,
                TimeSpan::from_secs_f64(0.5),
                TimeSpan::from_secs_f64(0.5),
                2,
                move |w| item_id(w, id, cur_id(w, id) + 1),
            );
        }
    } else if in_old && !in_new && matches!(cur, 0x2A6C | 0x2A70) {
        item_id(w, id, cur_id(w, id) - 1);
        timer::start(
            w,
            TimeSpan::from_secs_f64(0.5),
            TimeSpan::from_secs_f64(0.5),
            2,
            move |w| item_id(w, id, cur_id(w, id) - 1),
        );
    }
}

/// `CreepyPortraitComponent()`.
pub fn creepy_portrait_component_new(w: &mut World) -> ItemId {
    addon::new_component(w, &CREEPY_PORTRAIT_COMPONENT, 0x2A69)
}

leaf! {
    /// `Server.Items.CreepyPortraitComponent` (`[Flippable(0x2A69, 0x2A6D)]`).
    CREEPY_PORTRAIT_COMPONENT = "Server.Items.CreepyPortraitComponent" : ADDON_COMPONENT, enc;
    construct: None,
    make: Some(creepy_portrait_component_new),
    label_number: |_, _| 1074481,
    handles_on_movement: |_, _| true,
    on_movement: creepy_portrait_on_movement,
    on_double_click: |w, id, from| portrait_sound(w, id, from, 0x565, 0x566),
}

// DisturbingPortrait

fn disturbing_change(w: &mut World, id: ItemId) {
    let cur = cur_id(w, id);
    let next = if cur < 0x2A61 {
        utility::random_min_max(&mut w.rng, 0x2A5D, 0x2A60)
    } else {
        utility::random_min_max(&mut w.rng, 0x2A61, 0x2A64)
    };
    item_id(w, id, next);
}

fn start_repeating(w: &mut World, id: ItemId, change: fn(&mut World, ItemId)) {
    let t = timer::start(
        w,
        TimeSpan::from_mins(3),
        TimeSpan::from_mins(3),
        0,
        move |w| change(w, id),
    );
    set_portrait_timer(w, id, Some(t));
}

/// `DisturbingPortraitComponent()`.
pub fn disturbing_portrait_component_new(w: &mut World) -> ItemId {
    let id = addon::new_component(w, &DISTURBING_PORTRAIT_COMPONENT, 0x2A5D);
    start_repeating(w, id, disturbing_change);
    id
}

leaf_with_timer!(
    DISTURBING_PORTRAIT_COMPONENT,
    "Server.Items.DisturbingPortraitComponent",
    disturbing_portrait_component_new,
    1074479,
    disturbing_change
);

// UnsettlingPortrait

fn unsettling_change(w: &mut World, id: ItemId) {
    let cur = cur_id(w, id);
    let next = match cur {
        0x2A65 | 0x2A67 => cur + 1,
        0x2A66 | 0x2A68 => cur - 1,
        other => other,
    };
    if next != cur {
        item_id(w, id, next);
    }
}

/// `UnsettlingPortraitComponent()`.
pub fn unsettling_portrait_component_new(w: &mut World) -> ItemId {
    let id = addon::new_component(w, &UNSETTLING_PORTRAIT_COMPONENT, 0x2A65);
    start_repeating(w, id, unsettling_change);
    id
}

leaf_with_timer!(
    UNSETTLING_PORTRAIT_COMPONENT,
    "Server.Items.UnsettlingPortraitComponent",
    unsettling_portrait_component_new,
    1074480,
    unsettling_change
);

// AwesomeDisturbingPortrait

/// `Clock.GetTime(map, x, y, out hours, ..)`.
pub fn clock_hours(w: &World, map: Option<MapId>, x: i32) -> i32 {
    crate::skill_items::small::get_time(w, map, x).0
}

fn awesome_facing_south(w: &World, id: ItemId) -> bool {
    cur_id(w, id) < 0x2A61
}

fn awesome_update_image(w: &mut World, id: ItemId) {
    let hours = clock_hours(w, map_of(w, id), loc_of(w, id).x);
    let table: [i32; 7] = if awesome_facing_south(w, id) {
        [0x2A60, 0x2A5F, 0x2A5E, 0x2A5D, 0x2A5E, 0x2A5F, 0x2A60]
    } else {
        [0x2A64, 0x2A63, 0x2A62, 0x2A61, 0x2A62, 0x2A63, 0x2A64]
    };
    // the first bound above `hours`: `if (hours < 4) .. else if (hours < 6) ..`
    let i = [4, 6, 8, 16, 18, 20]
        .iter()
        .position(|&bound| hours < bound)
        .unwrap_or(6);
    item_id(w, id, table[i]);
}

fn start_awesome_timer(w: &mut World, id: ItemId, delay: TimeSpan) {
    let t = timer::start(w, delay, TimeSpan::from_mins(10), 0, move |w| {
        if !ops::is_deleted(w, id) {
            awesome_update_image(w, id);
        }
    });
    set_portrait_timer(w, id, Some(t));
}

fn awesome_on_double_click(w: &mut World, id: ItemId, from: MobileId) {
    let loc = loc_of(w, id);
    if in_range2(loc, mx::location(w, from)) {
        let hours = clock_hours(w, map_of(w, id), loc.x);
        if !(4..=20).contains(&hours) {
            play(w, id, 0x569);
        }
        awesome_update_image(w, id);
    } else {
        overhead(w, from, 0x3B2, 1019045); // I can't reach that.
    }
}

/// `AwesomeDisturbingPortraitComponent()`.
pub fn awesome_disturbing_portrait_component_new(w: &mut World) -> ItemId {
    let id = addon::new_component(w, &AWESOME_DISTURBING_PORTRAIT_COMPONENT, 0x2A5D);
    start_awesome_timer(w, id, TimeSpan::from_secs_f64(1.0));
    id
}

/// `Server.Items.AwesomeDisturbingPortraitComponent` (`[Flippable(0x2A5D, 0x2A61)]`).
pub static AWESOME_DISTURBING_PORTRAIT_COMPONENT: ItemType = ItemType {
    full_name: "Server.Items.AwesomeDisturbingPortraitComponent",
    parent: Some(&ADDON_COMPONENT),
    construct: None,
    make: Some(awesome_disturbing_portrait_component_new),
    label_number: |_, _| 1074479,
    on_double_click: awesome_on_double_click,
    on_after_delete: portrait_on_after_delete,
    serialize: |w, id, out| {
        (ADDON_COMPONENT.serialize)(w, id, out);
        out.write_encoded_int(0);
    },
    deserialize: |w, id, r| {
        (ADDON_COMPONENT.deserialize)(w, id, r)?;
        r.read_encoded_int()?;
        // [AfterDeserialization]: randomized so they don't all go off together.
        let ms = 8 * utility::random(&mut w.rng, 256);
        start_awesome_timer(w, id, TimeSpan::from_millis(i64::from(ms)));
        Ok(())
    },
    ..ADDON_COMPONENT
};

macro_rules! portrait_addon {
    ($addon:ident, $acls:ident, $anew:ident, $deed:ident, $dcls:ident, $afull:literal, $dfull:literal, $comp:path, $label:expr) => {
        pub fn $anew(w: &mut World) -> ItemId {
            let id = addon::new_addon(w, &$addon);
            addon::add_components(w, id, &[(Comp::Custom($comp), 0, 0, 0)]);
            id
        }
        static $acls: AddonClass = AddonClass {
            deed: Some(|w| {
                let id = addon::new_deed(w, &$deed);
                ops::set_loot_type(w, id, LootType::Blessed);
                id
            }),
            ..BASE_ADDON_CLASS
        };
        leaf! {
            #[doc = concat!("`", $afull, "`.")]
            $addon = $afull : BASE_ADDON, enc;
            construct: Some($anew),
            family: Some(&$acls),
        }
        blessed_deed!($deed, $dcls, $dfull, |w, _| $anew(w), $label);
    };
}

portrait_addon!(
    CREEPY_PORTRAIT_ADDON,
    CREEPY_PORTRAIT_ADDON_CLASS,
    creepy_portrait_addon_new,
    CREEPY_PORTRAIT_DEED,
    CREEPY_PORTRAIT_DEED_CLASS,
    "Server.Items.CreepyPortraitAddon",
    "Server.Items.CreepyPortraitDeed",
    creepy_portrait_component_new,
    1074481
);
portrait_addon!(
    DISTURBING_PORTRAIT_ADDON,
    DISTURBING_PORTRAIT_ADDON_CLASS,
    disturbing_portrait_addon_new,
    DISTURBING_PORTRAIT_DEED,
    DISTURBING_PORTRAIT_DEED_CLASS,
    "Server.Items.DisturbingPortraitAddon",
    "Server.Items.DisturbingPortraitDeed",
    disturbing_portrait_component_new,
    1074479
);
portrait_addon!(
    UNSETTLING_PORTRAIT_ADDON,
    UNSETTLING_PORTRAIT_ADDON_CLASS,
    unsettling_portrait_addon_new,
    UNSETTLING_PORTRAIT_DEED,
    UNSETTLING_PORTRAIT_DEED_CLASS,
    "Server.Items.UnsettlingPortraitAddon",
    "Server.Items.UnsettlingPortraitDeed",
    unsettling_portrait_component_new,
    1074480
);
portrait_addon!(
    AWESOME_DISTURBING_PORTRAIT_ADDON,
    AWESOME_DISTURBING_PORTRAIT_ADDON_CLASS,
    awesome_disturbing_portrait_addon_new,
    AWESOME_DISTURBING_PORTRAIT_DEED,
    AWESOME_DISTURBING_PORTRAIT_DEED_CLASS,
    "Server.Items.AwesomeDisturbingPortraitAddon",
    "Server.Items.AwesomeDisturbingPortraitDeed",
    awesome_disturbing_portrait_component_new,
    1074479
);

// ---- SacrificialAltar ---------------------------------------------------------------------------------------

#[derive(Clone, Copy, Default)]
struct AltarData {
    timer: Option<TimerHandle>,
}

fn altar_timer(w: &World, id: ItemId) -> Option<TimerHandle> {
    w.item(id)
        .and_then(|i| i.ext.get::<AltarData>())
        .and_then(|d| d.timer)
}

fn set_altar_timer(w: &mut World, id: ItemId, t: Option<TimerHandle>) {
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(AltarData { timer: t });
    }
}

fn start_altar_timer(w: &mut World, id: ItemId) {
    let t = timer::start(w, TimeSpan::from_mins(3), TimeSpan::ZERO, 1, move |w| {
        altar_empty(w, id)
    });
    set_altar_timer(w, id, Some(t));
}

/// `SacrificialAltarAddon.Empty()`.
pub fn altar_empty(w: &mut World, id: ItemId) {
    let items = w.item(id).map(|i| i.items.clone()).unwrap_or_default();
    if !items.is_empty() {
        let mut location = loc_of(w, id);
        location.z += 10;
        let map = map_of(w, id);
        muo_server::effects::send_location_effect(w, location, map, 0x3709, 10, 10, 0x356, 0);
        muo_server::effects::play_sound(w, location, map, 0x32E);
        for item in items.into_iter().rev() {
            if let Some(ty) = w.item(item).map(|i| i.ty) {
                (ty.delete)(w, item);
            }
        }
    }
    if let Some(t) = altar_timer(w, id) {
        timer::stop(w, t);
    }
    set_altar_timer(w, id, None);
}

fn altar_invalidate_contents(w: &mut World, id: ItemId, from: MobileId) {
    if ops::get_total(w, id, muo_server::item::TotalType::Items) >= 50 {
        messages::send_localized_message_to(w, id, from, 501478, ""); // The trash is full!  Emptying!
        altar_empty(w, id);
    } else {
        messages::send_localized_message_to(w, id, from, 1010442, ""); // The item will be deleted in three minutes
        // `_timer.Stop(); _timer.Start()`, or a new `EmptyTimer`: the three minutes begin again.
        if let Some(t) = altar_timer(w, id) {
            timer::stop(w, t);
        }
        start_altar_timer(w, id);
    }
}

fn altar_flip(w: &mut World, _from: MobileId, a: ItemId, direction: Direction) {
    if direction == Direction::EAST {
        item_id(w, a, 0x2A9C);
        addon_container::add_components(w, a, &[(CComp::Localized(0x2A9D, 1074818), 0, -1, 0)]);
    } else if direction == Direction::SOUTH {
        item_id(w, a, 0x2A9B);
        addon_container::add_components(w, a, &[(CComp::Localized(0x2A9A, 1074818), 1, 0, 0)]);
    }
}

/// `SacrificialAltarAddon()`.
pub fn sacrificial_altar_addon_new(w: &mut World) -> ItemId {
    let id = addon_container::new_addon_container(w, &SACRIFICIAL_ALTAR_ADDON, 0x2A9B);
    ops::set_direction(w, id, Direction::SOUTH);
    addon_container::add_components(w, id, &[(CComp::Localized(0x2A9A, 1074818), 1, 0, 0)]);
    id
}

static SACRIFICIAL_ALTAR_ADDON_CLASS: ContainerAddonClass = ContainerAddonClass {
    deed: Some(|w| {
        let id = addon_container::new_deed(w, &SACRIFICIAL_ALTAR_DEED);
        ops::set_loot_type(w, id, LootType::Blessed);
        id
    }),
    ..BASE_ADDON_CONTAINER_CLASS
};

/// `Server.Items.SacrificialAltarAddon` (`[FlippableAddon(South, East)]`).
pub static SACRIFICIAL_ALTAR_ADDON: ItemType = ItemType {
    full_name: "Server.Items.SacrificialAltarAddon",
    parent: Some(&BASE_ADDON_CONTAINER),
    construct: Some(sacrificial_altar_addon_new),
    family: Some(&SACRIFICIAL_ALTAR_ADDON_CLASS),
    label_number: |_, _| 1074818,
    default_max_weight: |_, _| 0,
    default_gump_id: |_, _| 0x107,
    default_drop_sound: |_, _| 0x42,
    on_drag_drop: |w, id, from, dropped| {
        if (BASE_ADDON_CONTAINER.on_drag_drop)(w, id, from, dropped) {
            altar_invalidate_contents(w, id, from);
            return true;
        }
        false
    },
    on_drag_drop_into: |w, id, from, item, p| {
        if (BASE_ADDON_CONTAINER.on_drag_drop_into)(w, id, from, item, p) {
            altar_invalidate_contents(w, id, from);
            return true;
        }
        false
    },
    serialize: |w, id, out| {
        (BASE_ADDON_CONTAINER.serialize)(w, id, out);
        out.write_encoded_int(0);
    },
    deserialize: |w, id, r| {
        (BASE_ADDON_CONTAINER.deserialize)(w, id, r)?;
        r.read_encoded_int()?;
        // [AfterDeserialization(false)]: Items is not filled yet, so the check runs on the next tick.
        timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 1, move |w| {
            if w.item(id).is_some_and(|i| !i.items.is_empty()) {
                start_altar_timer(w, id);
            }
        });
        Ok(())
    },
    ..BASE_ADDON_CONTAINER
};

static SACRIFICIAL_ALTAR_DEED_CLASS: ContainerDeedClass = ContainerDeedClass {
    addon: |w, _| sacrificial_altar_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.SacrificialAltarDeed`.
    SACRIFICIAL_ALTAR_DEED = "Server.Items.SacrificialAltarDeed" : BASE_ADDON_CONTAINER_DEED, enc;
    construct: Some(|w| {
        let id = addon_container::new_deed(w, &SACRIFICIAL_ALTAR_DEED);
        ops::set_loot_type(w, id, LootType::Blessed);
        id
    }),
    family: Some(&SACRIFICIAL_ALTAR_DEED_CLASS),
    label_number: |_, _| 1074818,
}

// ---- GingerBreadHouse ---------------------------------------------------------------------------------------

/// `GingerBreadHouseAddon()` (no `[Constructible]`).
pub fn ginger_bread_house_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &GINGER_BREAD_HOUSE_ADDON);
    for i in 0x2be5..0x2be8 {
        let c = addon::new_localized_component(w, &addon::LOCALIZED_ADDON_COMPONENT, i, 1077395); // Gingerbread House
        ops::set_light(w, c, LightType::SouthSmall);
        addon::add_component(
            w,
            id,
            c,
            if i == 0x2be5 { -1 } else { 0 },
            if i == 0x2be7 { -1 } else { 0 },
            0,
        );
    }
    id
}

static GINGER_BREAD_HOUSE_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(ginger_bread_house_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.GingerBreadHouseAddon`.
    GINGER_BREAD_HOUSE_ADDON = "Server.Items.GingerBreadHouseAddon" : BASE_ADDON, v0;
    family: Some(&GINGER_BREAD_HOUSE_ADDON_CLASS),
}

/// `GingerBreadHouseDeed()`.
pub fn ginger_bread_house_deed_new(w: &mut World) -> ItemId {
    let id = addon::new_deed(w, &GINGER_BREAD_HOUSE_DEED);
    ops::set_loot_type(w, id, LootType::Blessed);
    id
}

static GINGER_BREAD_HOUSE_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _| ginger_bread_house_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.GingerBreadHouseDeed`.
    GINGER_BREAD_HOUSE_DEED = "Server.Items.GingerBreadHouseDeed" : BASE_ADDON_DEED, v0;
    construct: Some(ginger_bread_house_deed_new),
    family: Some(&GINGER_BREAD_HOUSE_DEED_CLASS),
    default_weight: |_, _| 1.0,
    label_number: |_, _| 1077394,
}

// ---- registry -----------------------------------------------------------------------------------------------

pub static ALL: &[(&ItemType, &[&str])] = &[
    (&GUILLOTINE_ADDON, &[]),
    (&GUILLOTINE_DEED, &[]),
    (&IRON_MAIDEN_ADDON, &[]),
    (&IRON_MAIDEN_DEED, &[]),
    (&FOUNTAIN_ADDON, &[]),
    (&FOUNTAIN_DEED, &[]),
    (&CURTAINS_COMPONENT, &[]),
    (&CURTAINS_ADDON, &[]),
    (&CURTAINS_DEED, &[]),
    (&HOUSE_LADDER_ADDON, &[]),
    (&HOUSE_LADDER_DEED, &[]),
    (&BASE_FRUIT_TREE_ADDON, &[]),
    (&APPLE_TREE_ADDON, &[]),
    (&APPLE_TREE_DEED, &[]),
    (&PEACH_TREE_ADDON, &[]),
    (&PEACH_TREE_DEED, &[]),
    (&BED_OF_NAILS_COMPONENT, &[]),
    (&BED_OF_NAILS_ADDON, &[]),
    (&BED_OF_NAILS_DEED, &[]),
    (&BONE_COUCH_ADDON, &[]),
    (&BONE_COUCH_DEED, &[]),
    (&HAUNTER_MIRROR_COMPONENT, &[]),
    (&HAUNTER_MIRROR_ADDON, &[]),
    (&HAUNTER_MIRROR_DEED, &[]),
    (&CREEPY_PORTRAIT_COMPONENT, &[]),
    (&CREEPY_PORTRAIT_ADDON, &[]),
    (&CREEPY_PORTRAIT_DEED, &[]),
    (&DISTURBING_PORTRAIT_COMPONENT, &[]),
    (&DISTURBING_PORTRAIT_ADDON, &[]),
    (&DISTURBING_PORTRAIT_DEED, &[]),
    (&UNSETTLING_PORTRAIT_COMPONENT, &[]),
    (&UNSETTLING_PORTRAIT_ADDON, &[]),
    (&UNSETTLING_PORTRAIT_DEED, &[]),
    (&AWESOME_DISTURBING_PORTRAIT_COMPONENT, &[]),
    (&AWESOME_DISTURBING_PORTRAIT_ADDON, &[]),
    (&AWESOME_DISTURBING_PORTRAIT_DEED, &[]),
    (&SACRIFICIAL_ALTAR_ADDON, &[]),
    (&SACRIFICIAL_ALTAR_DEED, &[]),
    (&GINGER_BREAD_HOUSE_ADDON, &[]),
    (&GINGER_BREAD_HOUSE_DEED, &[]),
];

const BOOL: Kind = Kind::Plain(PropType::Bool);
const INT: Kind = Kind::Plain(PropType::Numeric(Numeric::Int));

static CURTAINS_CTORS: [Ctor; 1] = [Ctor {
    access: None,
    params: &[Param {
        name: "east",
        kind: BOOL,
        default: None,
        ty: "",
    }],
    make: |w, v| EntityId::Item(curtains_addon_new(w, v[0].bool())),
}];
static HOUSE_LADDER_CTORS: [Ctor; 1] = [Ctor {
    access: None,
    params: &[Param {
        name: "type",
        kind: INT,
        default: None,
        ty: "",
    }],
    make: |w, v| EntityId::Item(house_ladder_addon_new(w, v[0].i32())),
}];
static VANITY_CTORS: [Ctor; 1] = [Ctor {
    access: None,
    params: &[Param {
        name: "east",
        kind: BOOL,
        default: None,
        ty: "",
    }],
    make: |w, v| EntityId::Item(sp::vanity_addon_new(w, v[0].bool())),
}];

fn configure(w: &mut World) {
    use muo_content::items::flip::register_addon_flip;
    const SOUTH_EAST: &[Direction] = &[Direction::SOUTH, Direction::EAST];
    register_addon_flip(
        w,
        "Server.Items.BedOfNailsAddon",
        SOUTH_EAST,
        bed_of_nails_flip,
    );
    register_addon_flip(
        w,
        "Server.Items.BoneCouchAddon",
        SOUTH_EAST,
        bone_couch_flip,
    );
    register_addon_flip(
        w,
        "Server.Items.SacrificialAltarAddon",
        SOUTH_EAST,
        altar_flip,
    );
    muo_content::items::interfaces::register_dyable(w, &CURTAINS_COMPONENT, |w, id, _from, tub| {
        if ops::is_deleted(w, id) {
            return false;
        }
        let h = muo_content::items::clothing::tub_dyed_hue(w, tub);
        crate::simple::hue(w, id, h);
        true
    });
    add_ctors(w, "Server.Items.CurtainsAddon", &CURTAINS_CTORS);
    add_ctors(w, "Server.Items.HouseLadderAddon", &HOUSE_LADDER_CTORS);
    add_ctors(w, "Server.Items.VanityAddon", &VANITY_CTORS);
}

pub fn register(types: &mut TypeRegistry) {
    for (t, aliases) in ALL {
        types.register_item(t, aliases);
    }
    types.register_phase_default(
        Phase::Configure,
        "Server.Items.SpecialAddons.Configure",
        configure,
    );
}

#[cfg(test)]
pub(crate) fn configure_for_tests(w: &mut World) {
    configure(w);
}

#[cfg(test)]
pub(crate) fn portrait_timer_for_tests(w: &World, id: ItemId) -> Option<TimerHandle> {
    w.item(id)
        .and_then(|i| i.ext.get::<PortraitTimer>())
        .and_then(|d| d.timer)
}
