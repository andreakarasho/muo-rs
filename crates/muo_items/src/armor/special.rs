//! The armor classes with logic or state of their own: `ElvenGlasses`, the
//! arcane gloves (`LeatherGloves`, `LeafGloves`), `ChaosShield`/`OrderShield` and `MetalKiteShield.Dye`;
//! and the constructors of the three sample leaves for their derived classes.

use muo_content::aos::{AosWeaponAttribute, AttrKind, Attrs};
use muo_content::core;
use muo_content::craft::CraftResource;
use muo_content::items::armor::{
    self, ArmorClass, ArmorMaterialType, ArmorMeditationAllowance, BASE_ARMOR, BASE_ARMOR_CLASS,
    BASE_SHIELD, BASE_SHIELD_CLASS, new_armor,
};
use muo_content::items::{attr, parent_mobile};
use muo_io::{GenericReader, Result};
use muo_server::World;
use muo_server::guild::{GuildId, GuildType};
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::{ItemType, LootType, ops};
use muo_server::mobile::AccessLevel;
use muo_server::property_list::PropertyList;
use muo_server::race::ALLOW_ELVES_ONLY;
use muo_server::type_registry::TypeRegistry;

use super::{hue, item_id};

// ---- constructors of the samples in muo_content::items::leaves ----

/// `new T()` of sample `of` for class `ty`: `base(graphic)` with the sample's `graphic` prop.
fn sample_new(w: &mut World, ty: &'static ItemType, of: &'static ItemType) -> ItemId {
    let g = muo_content::items::equip_props::graphic(w, of);
    new_armor(w, ty, g)
}

/// `PlateChest()`.
pub fn plate_chest_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    sample_new(w, ty, &muo_content::items::leaves::PLATE_CHEST)
}

/// `LeatherChest()`.
pub fn leather_chest_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    sample_new(w, ty, &muo_content::items::leaves::LEATHER_CHEST)
}

/// `ChainChest()`.
pub fn chain_chest_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    sample_new(w, ty, &muo_content::items::leaves::CHAIN_CHEST)
}

// ---- ElvenGlasses ----

/// `ElvenGlasses`' constants.
pub static ELVEN_GLASSES_CLASS: ArmorClass = ArmorClass {
    default_weight: Some(2.0),
    base_physical_resistance: 2,
    base_fire_resistance: 4,
    base_cold_resistance: 4,
    base_poison_resistance: 3,
    base_energy_resistance: 2,
    init_min_hits: 36,
    init_max_hits: 48,
    aos_str_req: 45,
    old_str_req: 40,
    armor_base: 30,
    material_type: ArmorMaterialType::Leather,
    default_resource: CraftResource::RegularLeather,
    def_med_allowance: ArmorMeditationAllowance::All,
    ..BASE_ARMOR_CLASS
};

/// `ElvenGlasses()`: `base(0x2FB8)`, `_weaponAttributes = new AosWeaponAttributes(this)` (`ArmorData` starts with an
/// empty bag).
pub fn elven_glasses_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    new_armor(w, ty, 0x2FB8)
}

/// `SaveFlag.WeaponAttributes`.
const ELVEN_WEAPON_ATTRIBUTES: u32 = 0x1;

/// `AppendChildNameProperties`: the cliloc of each hit effect, in display order.
const ELVEN_HIT_PROPERTIES: [(AosWeaponAttribute, i32); 15] = [
    (AosWeaponAttribute::HitColdArea, 1060416), // hit cold area ~1_val~%
    (AosWeaponAttribute::HitDispel, 1060417),   // hit dispel ~1_val~%
    (AosWeaponAttribute::HitEnergyArea, 1060418), // hit energy area ~1_val~%
    (AosWeaponAttribute::HitFireArea, 1060419), // hit fire area ~1_val~%
    (AosWeaponAttribute::HitFireball, 1060420), // hit fireball ~1_val~%
    (AosWeaponAttribute::HitHarm, 1060421),     // hit harm ~1_val~%
    (AosWeaponAttribute::HitLeechHits, 1060422), // hit life leech ~1_val~%
    (AosWeaponAttribute::HitLightning, 1060423), // hit lightning ~1_val~%
    (AosWeaponAttribute::HitLowerAttack, 1060424), // hit lower attack ~1_val~%
    (AosWeaponAttribute::HitLowerDefend, 1060425), // hit lower defense ~1_val~%
    (AosWeaponAttribute::HitMagicArrow, 1060426), // hit magic arrow ~1_val~%
    (AosWeaponAttribute::HitLeechMana, 1060427), // hit mana leech ~1_val~%
    (AosWeaponAttribute::HitPhysicalArea, 1060428), // hit physical area ~1_val~%
    (AosWeaponAttribute::HitPoisonArea, 1060429), // hit poison area ~1_val~%
    (AosWeaponAttribute::HitLeechStam, 1060430), // hit stamina leech ~1_val~%
];

/// `Server.Items.ElvenGlasses` (`[SerializationGenerator(0, false)]`: version int 0, save flags, the weapon
/// attributes when not empty).
pub static ELVEN_GLASSES: ItemType = ItemType {
    full_name: "Server.Items.ElvenGlasses",
    parent: Some(&BASE_ARMOR),
    construct: Some(|w| elven_glasses_new(w, &ELVEN_GLASSES)),
    family: Some(&ELVEN_GLASSES_CLASS),
    append_child_name_properties: |w, id, list| {
        (BASE_ARMOR.append_child_name_properties)(w, id, list);
        for (a, number) in ELVEN_HIT_PROPERTIES {
            let prop = attr(w, id, AttrKind::Weapon, a as u32);
            if prop != 0 {
                list.add_int(number, prop);
            }
        }
    },
    serialize: |w, id, out| {
        (BASE_ARMOR.serialize)(w, id, out);
        out.write_i32(0);
        let bag = armor::data(w, id).map(|d| &d.weapon_attributes);
        let has = bag.is_some_and(|b| !b.is_empty());
        out.write_enum(4, i64::from(if has { ELVEN_WEAPON_ATTRIBUTES } else { 0 }));
        if let (true, Some(b)) = (has, bag) {
            b.serialize(out);
        }
    },
    deserialize: |w, id, r| {
        (BASE_ARMOR.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        let flags = r.read_enum(4)? as u32;
        let bag = if flags & ELVEN_WEAPON_ATTRIBUTES != 0 {
            Attrs::deserialize(r)?
        } else {
            Attrs::default()
        };
        if let Some(d) = armor::data_mut(w, id) {
            d.weapon_attributes = bag;
        }
        Ok(())
    },
    ..BASE_ARMOR
};

// ---- the arcane gloves ----

/// `IsArcane`.
pub fn is_arcane(w: &World, id: ItemId) -> bool {
    armor::data(w, id).is_some_and(|d| d.arcane_charges.1 > 0 && d.arcane_charges.0 >= 0)
}

/// The `Update()` of the gloves: the arcane graphic while charged, `normal` otherwise.
fn arcane_update(w: &mut World, id: ItemId, normal: i32) {
    let cur = armor::data(w, id).map_or(0, |d| d.arcane_charges.0);
    let current = w.item(id).map_or(0, |i| i.item_id);
    if is_arcane(w, id) {
        item_id(w, id, 0x26B0);
    } else if current == 0x26B0 {
        item_id(w, id, normal);
    }
    if is_arcane(w, id) && cur == 0 {
        hue(w, id, 0);
    }
}

/// `CurArcaneCharges` setter (`normal` is the graphic `Update` falls back to).
pub fn set_cur_arcane_charges(w: &mut World, id: ItemId, value: i32, normal: i32) {
    let Some(d) = armor::data_mut(w, id) else {
        return;
    };
    if d.arcane_charges.0 == value {
        return;
    }
    d.arcane_charges.0 = value;
    ops::invalidate_properties(w, id);
    arcane_update(w, id, normal);
}

/// `MaxArcaneCharges` setter.
pub fn set_max_arcane_charges(w: &mut World, id: ItemId, value: i32, normal: i32) {
    let Some(d) = armor::data_mut(w, id) else {
        return;
    };
    if d.arcane_charges.1 == value {
        return;
    }
    d.arcane_charges.1 = value;
    ops::invalidate_properties(w, id);
    arcane_update(w, id, normal);
}

/// `GetProperties`: "arcane charges: ~1_val~ / ~2_val~" while arcane.
fn arcane_properties(w: &World, id: ItemId, list: &mut PropertyList) {
    (BASE_ARMOR.get_properties)(w, id, list);
    if is_arcane(w, id)
        && let Some(d) = armor::data(w, id)
    {
        list.add_str(
            1061837,
            &format!("{}\t{}", d.arcane_charges.0, d.arcane_charges.1),
        );
    }
}

/// `OnSingleClick`.
fn arcane_single_click(w: &mut World, id: ItemId, from: MobileId) {
    (BASE_ARMOR.on_single_click)(w, id, from);
    if is_arcane(w, id)
        && let Some(d) = armor::data(w, id)
    {
        let args = format!("{}\t{}", d.arcane_charges.0, d.arcane_charges.1);
        ops::label_to_number(w, id, from, 1061837, &args);
    }
}

/// `Flip()`: swaps between the two graphics of the pair.
fn flip_pair(w: &mut World, id: ItemId, a: i32, b: i32) {
    let current = w.item(id).map_or(0, |i| i.item_id);
    if current == a {
        item_id(w, id, b);
    } else if current == b {
        item_id(w, id, a);
    }
}

/// `LeatherGloves.Flip()`.
pub fn leather_gloves_flip(w: &mut World, id: ItemId) {
    flip_pair(w, id, 0x13C6, 0x13CE);
}

/// `LeafGloves.Flip()`.
pub fn leaf_gloves_flip(w: &mut World, id: ItemId) {
    flip_pair(w, id, 0x2FC6, 0x317C);
}

/// The generated `Deserialize(reader, version)` target: `if (reader.ReadBool()) { cur = ReadInt; max = ReadInt }`.
fn arcane_load(
    w: &mut World,
    id: ItemId,
    r: &mut dyn GenericReader,
    current: i32,
    encoded: bool,
) -> Result<()> {
    let version = if encoded {
        r.read_encoded_int()?
    } else {
        r.read_i32()?
    };
    let charges = if version < current {
        if r.read_bool()? {
            (r.read_i32()?, r.read_i32()?)
        } else {
            (0, 0)
        }
    } else {
        (r.read_encoded_int()?, r.read_encoded_int()?)
    };
    if let Some(d) = armor::data_mut(w, id) {
        // The fields are assigned directly: a load does not run `Update()`.
        if version < current {
            if charges != (0, 0) {
                d.arcane_charges = charges;
            }
        } else {
            d.arcane_charges = charges;
        }
    }
    Ok(())
}

/// `LeatherGloves`' constants.
pub static LEATHER_GLOVES_CLASS: ArmorClass = ArmorClass {
    default_weight: Some(1.0),
    base_physical_resistance: 2,
    base_fire_resistance: 4,
    base_cold_resistance: 3,
    base_poison_resistance: 3,
    base_energy_resistance: 3,
    init_min_hits: 30,
    init_max_hits: 40,
    aos_str_req: 20,
    old_str_req: 10,
    armor_base: 13,
    material_type: ArmorMaterialType::Leather,
    default_resource: CraftResource::RegularLeather,
    def_med_allowance: ArmorMeditationAllowance::All,
    ..BASE_ARMOR_CLASS
};

/// `LeatherGloves()`: `base(0x13C6)`.
pub fn leather_gloves_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    new_armor(w, ty, 0x13C6)
}

/// `Server.Items.LeatherGloves` (`[SerializationGenerator(2, false)]`, `[Flippable]` with `Flip()`).
pub static LEATHER_GLOVES: ItemType = ItemType {
    full_name: "Server.Items.LeatherGloves",
    parent: Some(&BASE_ARMOR),
    construct: Some(|w| leather_gloves_new(w, &LEATHER_GLOVES)),
    family: Some(&LEATHER_GLOVES_CLASS),
    get_properties: arcane_properties,
    on_single_click: arcane_single_click,
    serialize: |w, id, out| {
        (BASE_ARMOR.serialize)(w, id, out);
        out.write_i32(2);
        let (cur, max) = armor::data(w, id).map_or((0, 0), |d| d.arcane_charges);
        out.write_encoded_int(cur);
        out.write_encoded_int(max);
    },
    deserialize: |w, id, r| {
        (BASE_ARMOR.deserialize)(w, id, r)?;
        arcane_load(w, id, r, 2, false)
    },
    ..BASE_ARMOR
};

/// `LeafGloves`' constants.
pub static LEAF_GLOVES_CLASS: ArmorClass = ArmorClass {
    default_weight: Some(2.0),
    required_races: Some(ALLOW_ELVES_ONLY),
    base_physical_resistance: 2,
    base_fire_resistance: 3,
    base_cold_resistance: 2,
    base_poison_resistance: 4,
    base_energy_resistance: 4,
    init_min_hits: 30,
    init_max_hits: 40,
    aos_str_req: 10,
    old_str_req: 10,
    armor_base: 13,
    material_type: ArmorMaterialType::Leather,
    default_resource: CraftResource::RegularLeather,
    def_med_allowance: ArmorMeditationAllowance::All,
    ..BASE_ARMOR_CLASS
};

/// `LeafGloves()`: `base(0x2FC6)`.
pub fn leaf_gloves_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    new_armor(w, ty, 0x2FC6)
}

/// `Server.Items.LeafGloves` (`[SerializationGenerator(1)]`: encoded version).
pub static LEAF_GLOVES: ItemType = ItemType {
    full_name: "Server.Items.LeafGloves",
    parent: Some(&BASE_ARMOR),
    construct: Some(|w| leaf_gloves_new(w, &LEAF_GLOVES)),
    family: Some(&LEAF_GLOVES_CLASS),
    get_properties: arcane_properties,
    on_single_click: arcane_single_click,
    serialize: |w, id, out| {
        (BASE_ARMOR.serialize)(w, id, out);
        out.write_encoded_int(1);
        let (cur, max) = armor::data(w, id).map_or((0, 0), |d| d.arcane_charges);
        out.write_encoded_int(cur);
        out.write_encoded_int(max);
    },
    deserialize: |w, id, r| {
        (BASE_ARMOR.deserialize)(w, id, r)?;
        arcane_load(w, id, r, 1, true)
    },
    ..BASE_ARMOR
};

// ---- shields ----

/// What `ChaosShield` and `OrderShield` need from the guild system (`Guild.Type`): set by the
/// owning crate. Unset, no guild is Chaos or Order, as when the guild is not a `Guild`.
#[derive(Default)]
pub struct ShieldGuildHooks {
    pub guild_type: Option<fn(&World, GuildId) -> Option<GuildType>>,
}

/// `Validate(m)` of both shields; `want` is the guild type that may carry the shield.
fn validate(w: &mut World, id: ItemId, m: Option<MobileId>, want: GuildType) -> bool {
    if core::aos(w) {
        return true;
    }
    let Some(m) = m else { return true };
    let Some(mob) = w.mobile(m) else { return true };
    if !(mob.player && mob.access_level == AccessLevel::Player) {
        return true;
    }
    let guild = mob.guild;
    let ok = guild.is_some_and(|g| {
        w.resources
            .get::<ShieldGuildHooks>()
            .and_then(|h| h.guild_type)
            .and_then(|f| f(w, g))
            == Some(want)
    });
    if ok {
        return true;
    }
    muo_content::mx::fixed_effect(w, m, 0x3728, 10, 13);
    if let Some(ty) = w.item(id).map(|i| i.ty) {
        (ty.delete)(w, id);
    }
    false
}

fn chaos_validate(w: &mut World, id: ItemId, m: Option<MobileId>) -> bool {
    validate(w, id, m, GuildType::Chaos)
}

fn order_validate(w: &mut World, id: ItemId, m: Option<MobileId>) -> bool {
    validate(w, id, m, GuildType::Order)
}

/// `ChaosShield()` / `OrderShield()` after `base(..)`: `if (!Core.AOS) LootType = LootType.Newbied;`.
fn newbied_before_aos(w: &mut World, id: ItemId) {
    if !core::aos(w) {
        ops::set_loot_type(w, id, LootType::Newbied);
    }
}

/// `ChaosShield`' constants.
pub static CHAOS_SHIELD_CLASS: ArmorClass = ArmorClass {
    default_weight: Some(4.0),
    base_physical_resistance: 1,
    base_fire_resistance: 0,
    base_cold_resistance: 0,
    base_poison_resistance: 0,
    base_energy_resistance: 0,
    init_min_hits: 100,
    init_max_hits: 125,
    aos_str_req: 95,
    armor_base: 32,
    ..BASE_SHIELD_CLASS
};

/// `ChaosShield()`: `base(0x1BC3)`.
pub fn chaos_shield_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = new_armor(w, ty, 0x1BC3);
    newbied_before_aos(w, id);
    id
}

/// `Server.Items.ChaosShield`.
pub static CHAOS_SHIELD: ItemType = ItemType {
    full_name: "Server.Items.ChaosShield",
    parent: Some(&BASE_SHIELD),
    construct: Some(|w| chaos_shield_new(w, &CHAOS_SHIELD)),
    family: Some(&CHAOS_SHIELD_CLASS),
    // `Validate(from) && base.OnEquip(from)`
    on_equip: |w, id, from| {
        chaos_validate(w, id, Some(from)) && (BASE_SHIELD.on_equip)(w, id, from)
    },
    on_single_click: |w, id, from| {
        let parent = parent_mobile(w, id);
        if chaos_validate(w, id, parent) {
            (BASE_SHIELD.on_single_click)(w, id, from);
        }
    },
    serialize: |w, id, out| {
        (BASE_SHIELD.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (BASE_SHIELD.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        Ok(())
    },
    ..BASE_SHIELD
};

/// `OrderShield`' constants.
pub static ORDER_SHIELD_CLASS: ArmorClass = ArmorClass {
    default_weight: Some(7.0),
    base_physical_resistance: 1,
    base_fire_resistance: 0,
    base_cold_resistance: 0,
    base_poison_resistance: 0,
    base_energy_resistance: 0,
    init_min_hits: 100,
    init_max_hits: 125,
    aos_str_req: 95,
    armor_base: 30,
    ..BASE_SHIELD_CLASS
};

/// `OrderShield()`: `base(0x1BC4)`.
pub fn order_shield_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = new_armor(w, ty, 0x1BC4);
    newbied_before_aos(w, id);
    id
}

/// `Server.Items.OrderShield`.
pub static ORDER_SHIELD: ItemType = ItemType {
    full_name: "Server.Items.OrderShield",
    parent: Some(&BASE_SHIELD),
    construct: Some(|w| order_shield_new(w, &ORDER_SHIELD)),
    family: Some(&ORDER_SHIELD_CLASS),
    on_equip: |w, id, from| {
        order_validate(w, id, Some(from)) && (BASE_SHIELD.on_equip)(w, id, from)
    },
    on_single_click: |w, id, from| {
        let parent = parent_mobile(w, id);
        if order_validate(w, id, parent) {
            (BASE_SHIELD.on_single_click)(w, id, from);
        }
    },
    serialize: |w, id, out| {
        (BASE_SHIELD.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (BASE_SHIELD.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        Ok(())
    },
    ..BASE_SHIELD
};

/// `MetalKiteShield.Dye` (`IDyable`): `if (Deleted) return false; Hue = sender.DyedHue; return true;`.
pub fn metal_kite_shield_dye(w: &mut World, id: ItemId, dyed_hue: i32) -> bool {
    if w.item(id).is_none_or(|i| i.deleted()) {
        return false;
    }
    hue(w, id, dyed_hue);
    true
}

/// Registers the classes of this file.
pub fn register(types: &mut TypeRegistry) {
    for t in [
        &ELVEN_GLASSES,
        &LEATHER_GLOVES,
        &LEAF_GLOVES,
        &CHAOS_SHIELD,
        &ORDER_SHIELD,
    ] {
        types.register_item(t, &[]);
    }
    types.register_phase_default(
        muo_server::type_registry::Phase::Configure,
        "Server.Items.MetalKiteShield.Dye",
        |w| {
            muo_content::items::interfaces::register_dyable(
                w,
                &super::leaves::METAL_KITE_SHIELD,
                |w, id, _, tub| {
                    let h = muo_content::items::clothing::tub_dyed_hue(w, tub);
                    metal_kite_shield_dye(w, id, h)
                },
            );
        },
    );
}
