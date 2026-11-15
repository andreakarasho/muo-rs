//! `TaxidermyKit`, `TrophyAddon` and `TrophyDeed`.

use muo_content::items::addon;
use muo_content::items::house_deps::{self, SimpleAddon};
use muo_io::{GenericReader, GenericWriter, Result};
use muo_primitives::{Direction, Point3D, TimeSpan};
use muo_server::World;
use muo_server::ext::Ext;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, base, deps, ops};
use muo_server::map::MapId;
use muo_server::property_list::PropertyList;
use muo_server::serialization_ext::ReadEntityExt;
use muo_server::skills::SkillName;
use muo_server::target::{self, TargetFlags, Targeted};
use muo_server::type_registry::{Phase, TypeRegistry};

use super::{read_version, write_version};

/// `TaxidermyKit.TrophyInfo`: the creature class, the north graphic and the deed and addon labels.
struct TrophyInfo {
    creature: &'static str,
    north_id: i32,
    deed_number: i32,
    addon_number: i32,
}

const fn t(
    creature: &'static str,
    north_id: i32,
    deed_number: i32,
    addon_number: i32,
) -> TrophyInfo {
    TrophyInfo {
        creature,
        north_id,
        deed_number,
        addon_number,
    }
}

static TABLE: [TrophyInfo; 7] = [
    t("Server.Mobiles.BrownBear", 0x1E60, 1041093, 1041107),
    t("Server.Mobiles.GreatHart", 0x1E61, 1041095, 1041109),
    t("Server.Items.BigFish", 0x1E62, 1041096, 1041110),
    t("Server.Mobiles.Gorilla", 0x1E63, 1041091, 1041105),
    t("Server.Mobiles.Orc", 0x1E64, 1041090, 1041104),
    t("Server.Mobiles.PolarBear", 0x1E65, 1041094, 1041108),
    t("Server.Mobiles.Troll", 0x1E66, 1041092, 1041106),
];

// ---- the shared data of the deed and the addon ----

/// The six `[SerializableField]`s of `TrophyAddon` and `TrophyDeed`.
#[derive(Clone, Debug, Default)]
pub struct TrophyData {
    pub hunter: Option<String>,
    pub animal_weight: i32,
    pub west_id: i32,
    pub north_id: i32,
    pub deed_number: i32,
    pub addon_number: i32,
}

pub fn data(w: &World, id: ItemId) -> Option<&TrophyData> {
    w.item(id)?.ext.get::<TrophyData>()
}

fn data_clone(w: &World, id: ItemId) -> TrophyData {
    data(w, id).cloned().unwrap_or_default()
}

fn set_data(w: &mut World, id: ItemId, d: TrophyData) {
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(d);
    }
}

fn trophy_serialize(w: &World, id: ItemId, out: &mut dyn GenericWriter) {
    (ITEM_BASE.serialize)(w, id, out);
    out.write_i32(2);
    let d = data_clone(w, id);
    out.write_opt_string(d.hunter.as_deref());
    out.write_i32(d.animal_weight);
    out.write_i32(d.west_id);
    out.write_i32(d.north_id);
    out.write_i32(d.deed_number);
    out.write_i32(d.addon_number);
}

fn trophy_deserialize(w: &mut World, id: ItemId, r: &mut dyn GenericReader) -> Result<()> {
    (ITEM_BASE.deserialize)(w, id, r)?;
    let version = r.read_i32()?;
    let mut d = TrophyData::default();
    if version < 2 {
        // Deserialize(reader, version): the crafter is a Mobile whose name is copied on a zero-delay timer.
        let crafter = r.read_mobile(w)?;
        d.animal_weight = r.read_i32()?;
        d.west_id = r.read_i32()?;
        d.north_id = r.read_i32()?;
        d.deed_number = r.read_i32()?;
        d.addon_number = r.read_i32()?;
        set_data(w, id, d);
        muo_server::timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 1, move |w| {
            let name = crafter.and_then(|m| muo_server::mobile::part1::props_b::raw_name(w, m));
            if let Some(x) = w.item_mut(id).and_then(|i| i.ext.get_mut::<TrophyData>()) {
                x.hunter = name;
            }
        });
    } else {
        d.hunter = r.read_string()?;
        d.animal_weight = r.read_i32()?;
        d.west_id = r.read_i32()?;
        d.north_id = r.read_i32()?;
        d.deed_number = r.read_i32()?;
        d.addon_number = r.read_i32()?;
        set_data(w, id, d);
    }
    Ok(())
}

fn trophy_properties(w: &World, id: ItemId, list: &mut PropertyList) {
    base::get_properties_base(w, id, list);
    let d = data_clone(w, id);
    if muo_content::core::se(w) && d.animal_weight >= 20 {
        if let Some(h) = d.hunter.as_deref().filter(|h| !h.trim().is_empty()) {
            list.add_str(1070857, h); // Caught by ~1_fisherman~
        }
        list.add_int(1070858, d.animal_weight); // ~1_weight~ stones
    }
}

// ---- TrophyDeed ----

/// `new TrophyDeed(westID, northID, deedNumber, addonNumber, hunter, animalWeight)`.
pub fn new_trophy_deed(
    w: &mut World,
    west_id: i32,
    north_id: i32,
    deed_number: i32,
    addon_number: i32,
    hunter: Option<String>,
    animal_weight: i32,
) -> ItemId {
    let id = crate::simple::from_template(w, &TROPHY_DEED, "Server.Items.TrophyDeed");
    set_data(
        w,
        id,
        TrophyData {
            hunter,
            animal_weight,
            west_id,
            north_id,
            deed_number,
            addon_number,
        },
    );
    id
}

fn deed_double_click(w: &mut World, id: ItemId, from: MobileId) {
    let pack = w.mobile(from).and_then(|m| m.backpack);
    if !ops::is_child_of(w, id, pack.map(EntityId::Item)) {
        deps::send_localized_message(w, from, 1042001); // That must be in your pack for you to use it.
        return;
    }
    let house = muo_multis::house::access::find_house_at_mobile(w, from);
    let Some(house) = house.filter(|&h| muo_multis::house::access::is_co_owner(w, h, from)) else {
        deps::send_localized_message(w, from, 502092); // You must be in your house to do this.
        return;
    };
    let (loc, map, dir) = match w.mobile(from) {
        Some(m) => (m.location, m.map, m.direction),
        None => return,
    };
    let mut north_wall = addon::is_wall(w, loc.x, loc.y - 1, loc.z, map);
    let mut west_wall = addon::is_wall(w, loc.x - 1, loc.y, loc.z, map);
    if north_wall && west_wall {
        let d = dir & Direction::MASK;
        if d == Direction::NORTH || d == Direction::SOUTH {
            west_wall = false;
        } else if d == Direction::EAST || d == Direction::WEST {
            north_wall = false;
        } else {
            muo_server::mobile::part2::messages::send_message(
                w,
                from,
                "Turn to face the wall on which to hang this trophy.",
            );
            return;
        }
    }
    let d = data_clone(w, id);
    let mut item_id = 0;
    if north_wall {
        item_id = d.north_id;
    } else if west_wall {
        item_id = d.west_id;
    } else {
        deps::send_localized_message(w, from, 1042626); // The trophy must be placed next to a wall.
    }
    if item_id > 0 {
        let a = new_trophy_addon(
            w,
            from,
            item_id,
            d.west_id,
            d.north_id,
            d.deed_number,
            d.addon_number,
            d.hunter,
            d.animal_weight,
        );
        addon::house_addons_add(w, house, a);
        base::delete(w, id);
    }
}

/// `Server.Items.TrophyDeed`: no `[Constructible]` constructor.
pub static TROPHY_DEED: ItemType = ItemType {
    full_name: "Server.Items.TrophyDeed",
    parent: Some(&ITEM_BASE),
    construct: None,
    label_number: |w, id| data(w, id).map_or(0, |d| d.deed_number),
    get_properties: trophy_properties,
    on_double_click: deed_double_click,
    serialize: trophy_serialize,
    deserialize: trophy_deserialize,
    ..ITEM_BASE
};

// ---- TrophyAddon ----

/// `new TrophyAddon(from, itemID, westID, northID, deedNumber, addonNumber, hunter, animalWeight)`.
#[allow(clippy::too_many_arguments)]
pub fn new_trophy_addon(
    w: &mut World,
    from: MobileId,
    item_id: i32,
    west_id: i32,
    north_id: i32,
    deed_number: i32,
    addon_number: i32,
    hunter: Option<String>,
    animal_weight: i32,
) -> ItemId {
    let id = ops::new_item_with_id(w, &TROPHY_ADDON, item_id);
    set_data(
        w,
        id,
        TrophyData {
            hunter,
            animal_weight,
            west_id,
            north_id,
            deed_number,
            addon_number,
        },
    );
    ops::set_movable(w, id, false);
    let (loc, map) = w
        .mobile(from)
        .map_or((Point3D::default(), None), |m| (m.location, m.map));
    ops::move_to_world_map(w, id, loc, map);
    id
}

/// `Deed`.
fn addon_deed(w: &mut World, id: ItemId) -> ItemId {
    let d = data_clone(w, id);
    new_trophy_deed(
        w,
        d.west_id,
        d.north_id,
        d.deed_number,
        d.addon_number,
        d.hunter,
        d.animal_weight,
    )
}

fn addon_could_fit(w: &mut World, id: ItemId, p: Point3D, map: MapId) -> bool {
    let height = i32::from(ops::item_data(w, id).height);
    if !muo_server::map::fit::can_fit_point(w, map, p, height) {
        return false;
    }
    let north_id = data(w, id).map_or(0, |d| d.north_id);
    if w.item(id).is_some_and(|i| i.item_id == north_id) {
        addon::is_wall(w, p.x, p.y - 1, p.z, Some(map)) // North wall
    } else {
        addon::is_wall(w, p.x - 1, p.y, p.z, Some(map)) // West wall
    }
}

/// `FixMovingCrate()`.
fn fix_moving_crate(w: &mut World, id: ItemId) {
    if w.item(id).is_none_or(|i| i.deleted()) {
        return;
    }
    if w.item(id).is_some_and(|i| i.movable()) || ops::is_locked_down(w, id) {
        let deed = addon_deed(w, id);
        let (parent, loc, map) = match w.item(id) {
            Some(i) => (i.parent, i.location, i.map),
            None => return,
        };
        if let Some(EntityId::Item(p)) = parent {
            if let Some(pt) = w.item(p).map(|i| i.ty) {
                (pt.add_item)(w, p, deed);
            }
            ops::set_location(w, deed, loc);
        } else {
            ops::move_to_world_map(w, deed, loc, map);
        }
        base::delete(w, id);
    }
}

fn addon_double_click(w: &mut World, id: ItemId, from: MobileId) {
    let house = muo_multis::house::access::find_house_at_item(w, id);
    if !house.is_some_and(|h| muo_multis::house::access::is_co_owner(w, h, from)) {
        return;
    }
    let loc = ops::get_world_location(w, id);
    if deps::mobile_in_range(w, from, loc, 1) {
        let deed = addon_deed(w, id);
        muo_server::mobile::part2::items::add_to_backpack(w, from, deed);
        base::delete(w, id);
    } else {
        deps::send_localized_message(w, from, 500295); // You are too far away to do that.
    }
}

/// `Server.Items.TrophyAddon`: no `[Constructible]` constructor.
pub static TROPHY_ADDON: ItemType = ItemType {
    full_name: "Server.Items.TrophyAddon",
    parent: Some(&ITEM_BASE),
    construct: None,
    label_number: |w, id| data(w, id).map_or(0, |d| d.addon_number),
    get_properties: trophy_properties,
    on_double_click: addon_double_click,
    serialize: trophy_serialize,
    deserialize: |w, id, r| {
        trophy_deserialize(w, id, r)?;
        muo_server::timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 1, move |w| {
            fix_moving_crate(w, id)
        }); // [AfterDeserialization(false)]
        Ok(())
    },
    ..ITEM_BASE
};

// ---- TaxidermyKit ----

/// `CorpseTarget.OnTarget`.
fn on_target(w: &mut World, kit: ItemId, from: MobileId, targeted: Targeted) {
    if w.item(kit).is_none_or(|i| i.deleted()) {
        return;
    }
    let pack = w.mobile(from).and_then(|m| m.backpack);
    if !ops::is_child_of(w, kit, pack.map(EntityId::Item)) {
        deps::send_localized_message(w, from, 1042001); // That must be in your pack for you to use it.
        return;
    }
    if muo_server::skills::base(w, from, SkillName::Carpentry) < 90.0 {
        deps::send_localized_message(w, from, 1042603); // You would not understand how to use the kit.
        return;
    }
    let item = match targeted {
        Targeted::Entity(EntityId::Item(i)) => Some(i),
        _ => None,
    };
    let corpse = item.filter(|&i| muo_content::items::corpse::is_corpse(w, i));
    let fish = item.filter(|&i| {
        w.item(i)
            .is_some_and(|x| std::ptr::eq(x.ty, &crate::resources::cloth::BIG_FISH))
    });
    if corpse.is_none() && fish.is_none() {
        deps::send_localized_message(w, from, 1042600); // That is not a corpse!
        return;
    }
    if corpse.is_some_and(|c| {
        muo_content::items::corpse::get_flag(
            w,
            c,
            muo_content::items::corpse::flag::VISITED_BY_TAXIDERMIST,
        )
    }) {
        deps::send_localized_message(w, from, 1042596); // That corpse seems to have been visited by a taxidermist already.
        return;
    }
    // `corpse?.Owner ?? targeted`: the type of the creature, else of the targeted object.
    let owner = corpse.and_then(|c| muo_content::items::corpse::owner(w, c));
    let type_name = match (owner, item) {
        (Some(o), _) => w.mobile(o).map(|m| m.ty.full_name),
        (None, Some(i)) => w.item(i).map(|x| x.ty.full_name),
        _ => None,
    };
    let Some(info) = type_name.and_then(|n| TABLE.iter().find(|t| t.creature == n)) else {
        deps::send_localized_message(w, from, 1042599); // That does not look like something you want hanging on a wall.
        return;
    };
    let board = super::item_type(w, "Server.Items.Board");
    let enough = match (pack, board) {
        (Some(p), Some(b)) => muo_server::item::container::consume_total(w, p, b, 10, true, None),
        _ => false,
    };
    if !enough {
        deps::send_localized_message(w, from, 1042598); // You do not have enough boards.
        return;
    }
    deps::send_localized_message(w, from, 1042278); // You review the corpse and find it worthy of a trophy.
    deps::send_localized_message(w, from, 1042602); // You use your kit up making the trophy.
    let (hunter, weight) = match fish {
        Some(f) => {
            let hunter = crate::resources::cloth::fisher(w, f)
                .and_then(|m| muo_server::mobile::part1::props_b::raw_name(w, m));
            (hunter, ops::weight(w, f) as i32)
        }
        None => (None, 0),
    };
    if let Some(f) = fish {
        base::consume(w, f, 1);
    }
    let deed = new_trophy_deed(
        w,
        info.north_id + 7,
        info.north_id,
        info.deed_number,
        info.addon_number,
        hunter,
        weight,
    );
    muo_server::mobile::part2::items::add_to_backpack(w, from, deed);
    if let Some(c) = corpse {
        muo_content::items::corpse::set_flag(
            w,
            c,
            muo_content::items::corpse::flag::VISITED_BY_TAXIDERMIST,
            true,
        );
    }
    base::delete(w, kit);
}

fn kit_double_click(w: &mut World, id: ItemId, from: MobileId) {
    let pack = w.mobile(from).and_then(|m| m.backpack);
    if !ops::is_child_of(w, id, pack.map(EntityId::Item)) {
        deps::send_localized_message(w, from, 1042001); // That must be in your pack for you to use it.
    } else if muo_server::skills::base(w, from, SkillName::Carpentry) < 90.0 {
        deps::send_localized_message(w, from, 1042594); // You do not understand how to use this.
    } else {
        deps::send_localized_message(w, from, 1042595); // Target the corpse to make a trophy out of.
        target::begin_target(w, from, 3, false, TargetFlags::NONE, move |w, from, t| {
            on_target(w, id, from, t)
        });
    }
}

/// `Server.Items.TaxidermyKit`.
pub static TAXIDERMY_KIT: ItemType = ItemType {
    full_name: "Server.Items.TaxidermyKit",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| crate::simple::from_template(w, &TAXIDERMY_KIT, "Server.Items.TaxidermyKit")),
    default_weight: crate::class_props::default_weight,
    label_number: crate::class_props::label_number, // a taxidermy kit
    on_double_click: kit_double_click,
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        write_version(out, false);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        read_version(r, false)?;
        Ok(())
    },
    ..ITEM_BASE
};

fn configure(w: &mut World) {
    house_deps::deps_mut(w).simple_addons.push(SimpleAddon {
        is: |w, id| w.item(id).is_some_and(|i| i.ty.is(&TROPHY_ADDON)),
        deed: addon_deed,
        could_fit: addon_could_fit,
    });
}

/// Registers the taxidermy classes.
pub fn register(types: &mut TypeRegistry) {
    for t in [&TAXIDERMY_KIT, &TROPHY_ADDON, &TROPHY_DEED] {
        types.register_item(t, &[]);
    }
    types.register_phase_default(
        Phase::Configure,
        "Server.Items.TrophyAddon.Configure",
        configure,
    );
}
