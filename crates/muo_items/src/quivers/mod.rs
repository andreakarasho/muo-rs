//! Quivers: `BaseQuiver` (a `Container` on `Layer.Cloak` that holds one kind of ammo and
//! lowers its weight) and the four elven quivers. State is `muo_content::items::ext_bags::QuiverData` in
//! `Item::ext` (so `items::attr` reaches the `AosAttributes`). The class constants (graphic, label, the
//! `AlterBowDamage` split) are template props and the constructors' statements template keys
//! (`Data/Templates/Items/quivers`). [`configure`] sets the hooks the weapon code calls (`quiver_of`, ..).

use muo_content::aos::{AosAttribute, AttrKind, Attrs};
use muo_content::items::clothing::ClothingQuality;
use muo_content::items::equip_hooks::CraftArgs;
use muo_content::items::ext_bags::QuiverData;
use muo_content::items::weapon_hooks::WeaponHooks;
use muo_content::items::{attr, remove_stat_mods};
use muo_io::{GenericReader, GenericWriter, IoError, Result};
use muo_primitives::TimeSpan;
use muo_server::World;
use muo_server::ext::Ext;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::container::{self, CONTAINER};
use muo_server::item::{ItemType, TotalType, ops};
use muo_server::layer::Layer;
use muo_server::mobile::StatType;
use muo_server::mods::StatMod;
use muo_server::property_list::PropertyList;
use muo_server::type_registry::{Phase, TypeRegistry};

pub fn data(w: &World, id: ItemId) -> Option<&QuiverData> {
    w.item(id)?.ext.get::<QuiverData>()
}

pub fn data_mut(w: &mut World, id: ItemId) -> Option<&mut QuiverData> {
    w.item_mut(id)?.ext.get_mut::<QuiverData>()
}

/// `item is BaseQuiver`.
pub fn is_quiver(w: &World, id: ItemId) -> bool {
    w.item(id).is_some_and(|i| i.ty.is(&BASE_QUIVER))
}

/// `LowerAmmoCost`.
pub fn lower_ammo_cost(w: &World, id: ItemId) -> i32 {
    data(w, id).map_or(0, |d| d.lower_ammo_cost)
}

/// `LowerAmmoCost` setter (`[InvalidateProperties]`).
pub fn set_lower_ammo_cost(w: &mut World, id: ItemId, value: i32) {
    if let Some(d) = data_mut(w, id)
        && d.lower_ammo_cost != value
    {
        d.lower_ammo_cost = value;
        ops::invalidate_properties(w, id);
    }
}

/// `WeightReduction` setter (`[InvalidateProperties]`).
pub fn set_weight_reduction(w: &mut World, id: ItemId, value: i32) {
    if let Some(d) = data_mut(w, id)
        && d.weight_reduction != value
    {
        d.weight_reduction = value;
        ops::invalidate_properties(w, id);
    }
}

/// `Capacity` setter (`[InvalidateProperties]`).
pub fn set_capacity(w: &mut World, id: ItemId, value: i32) {
    if let Some(d) = data_mut(w, id)
        && d.capacity != value
    {
        d.capacity = value;
        ops::invalidate_properties(w, id);
    }
}

/// `DamageIncrease` setter (`[InvalidateProperties]`).
pub fn set_damage_increase(w: &mut World, id: ItemId, value: i32) {
    if let Some(d) = data_mut(w, id)
        && d.damage_increase != value
    {
        d.damage_increase = value;
        ops::invalidate_properties(w, id);
    }
}

/// `Quality` setter.
pub fn set_quality(w: &mut World, id: ItemId, value: ClothingQuality) {
    if let Some(d) = data_mut(w, id)
        && d.quality != value as i32
    {
        d.quality = value as i32;
        ops::invalidate_properties(w, id);
    }
}

/// `Crafter` setter.
pub fn set_crafter(w: &mut World, id: ItemId, value: Option<String>) {
    if let Some(d) = data_mut(w, id)
        && d.crafter != value
    {
        d.crafter = value;
        ops::invalidate_properties(w, id);
    }
}

/// `Ammo => Items.Count > 0 ? Items[0] : null`.
pub fn ammo(w: &World, id: ItemId) -> Option<ItemId> {
    w.item(id)?.items.first().copied()
}

/// `new BaseQuiver(itemID)` for class `ty`: `Capacity = 500; Layer = Layer.Cloak; DamageIncrease = 10`.
pub fn new_quiver(w: &mut World, ty: &'static ItemType, item_id: i32) -> ItemId {
    let id = container::new_container(w, ty, item_id);
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(QuiverData::default());
    }
    set_capacity(w, id, 500);
    (ty.set_layer)(w, id, Layer::Cloak);
    set_damage_increase(w, id, 10);
    id
}

/// `OnCraft(..)`: `Quality = (ClothingQuality)quality; if (makersMark) Crafter = from.RawName; return quality`.
pub fn on_craft(w: &mut World, id: ItemId, a: &CraftArgs) -> i32 {
    // shortcut: a quality number the craft system never passes (not 0..=2) reads as Regular.
    set_quality(
        w,
        id,
        ClothingQuality::from_i32(a.quality).unwrap_or(ClothingQuality::Regular),
    );
    if a.makers_mark {
        let name = muo_server::mobile::part1::props_b::raw_name(w, a.from);
        set_crafter(w, id, name);
    }
    a.quality
}

/// `InvalidateWeight()`: `if (RootParent is Mobile m) m.UpdateTotals()`.
pub fn invalidate_weight(w: &mut World, id: ItemId) {
    if let Some(EntityId::Mobile(m)) = ops::root_parent(w, id) {
        muo_server::mobile::base::update_totals(w, m);
    }
}

/// `Attributes.AddStatBonuses(to)` (`AosAttributes`).
// shortcut: the stat mods are named with `GetHashCode()` of the attribute bag but removes them by the item's
// serial, so they are never removed (the bug stays); the name here is the serial xor a constant, which is just as
// unmatched by `RemoveStatMod($"{serial}Str")`.
pub fn add_stat_bonuses(w: &mut World, id: ItemId, to: MobileId) {
    let (s, d, i) = (
        attr(w, id, AttrKind::Attributes, AosAttribute::BonusStr as u32),
        attr(w, id, AttrKind::Attributes, AosAttribute::BonusDex as u32),
        attr(w, id, AttrKind::Attributes, AosAttribute::BonusInt as u32),
    );
    if let (true, Some(serial), Some(ty)) = (
        s != 0 || d != 0 || i != 0,
        w.item(id).map(|i| i.serial),
        w.mobile(to).map(|m| m.ty),
    ) {
        let code = u32::from(serial) ^ 0x5F37_59DF;
        for (stat, suffix, bonus) in [
            (StatType::STR, "Str", s),
            (StatType::DEX, "Dex", d),
            (StatType::INT, "Int", i),
        ] {
            if bonus != 0 {
                let name = format!("{code}{suffix}");
                let sm = StatMod::new(w, stat, Some(&name), bonus, TimeSpan::ZERO, None);
                (ty.add_stat_mod)(w, to, sm);
            }
        }
    }
    if let Some(ty) = w.mobile(to).map(|m| m.ty) {
        (ty.check_stat_timers)(w, to);
    }
}

/// `Attributes.RemoveStatBonuses(from)`.
pub fn remove_stat_bonuses(w: &mut World, id: ItemId, from: MobileId) {
    remove_stat_mods(w, id, from);
    if let Some(ty) = w.mobile(from).map(|m| m.ty) {
        (ty.check_stat_timers)(w, from);
    }
}

/// `CheckType(item)`: the same class as the ammo held, or an `Arrow`/`Bolt` when empty.
pub fn check_type(w: &World, id: ItemId, item: ItemId) -> bool {
    let Some(ty) = w.item(item).map(|i| i.ty) else {
        return false;
    };
    match ammo(w, id).and_then(|a| w.item(a)).map(|a| a.ty) {
        Some(a) => std::ptr::eq(a, ty) || a.full_name == ty.full_name,
        None => ty.full_name == "Server.Items.Arrow" || ty.full_name == "Server.Items.Bolt",
    }
}

#[allow(clippy::too_many_arguments)] // the signature of `Container.CheckHold`
fn check_hold(
    w: &mut World,
    id: ItemId,
    m: Option<MobileId>,
    item: ItemId,
    message: bool,
    check_items: bool,
    plus_items: i32,
    plus_weight: i32,
) -> bool {
    if !check_type(w, id, item) {
        if message && let Some(m) = m {
            muo_server::item::deps::send_localized_message(w, m, 1074836); // The container can not hold that type of object.
        }
        return false;
    }
    let capacity = data(w, id).map_or(0, |d| d.capacity);
    let item_amount = w.item(item).map_or(0, |i| i.amount);
    let count = w.item(id).map_or(0, |i| i.items.len()) as i32;
    let max_items = (w
        .item(id)
        .map_or(CONTAINER.default_max_items, |i| i.ty.default_max_items))(w, id);
    if count < max_items {
        return item_amount <= capacity
            && (CONTAINER.check_hold)(
                w,
                id,
                m,
                item,
                message,
                check_items,
                plus_items,
                plus_weight,
            );
    }
    if check_items {
        return false;
    }
    match ammo(w, id)
        .filter(|&a| !ops::is_deleted(w, a))
        .and_then(|a| w.item(a))
    {
        Some(a) => a.amount + item_amount <= capacity,
        None => false,
    }
}

/// The `AosAttribute` lines of `GetProperties`, in display order (night sight has no value).
const ATTRIBUTE_LINES: [(AosAttribute, i32); 21] = [
    (AosAttribute::DefendChance, 1060408),
    (AosAttribute::BonusDex, 1060409),
    (AosAttribute::EnhancePotions, 1060411),
    (AosAttribute::CastRecovery, 1060412),
    (AosAttribute::CastSpeed, 1060413),
    (AosAttribute::AttackChance, 1060415),
    (AosAttribute::BonusHits, 1060431),
    (AosAttribute::BonusInt, 1060432),
    (AosAttribute::LowerManaCost, 1060433),
    (AosAttribute::LowerRegCost, 1060434),
    (AosAttribute::Luck, 1060436),
    (AosAttribute::BonusMana, 1060439),
    (AosAttribute::RegenMana, 1060440),
    (AosAttribute::NightSight, 1060441),
    (AosAttribute::ReflectPhysical, 1060442),
    (AosAttribute::RegenStam, 1060443),
    (AosAttribute::RegenHits, 1060444),
    (AosAttribute::SpellDamage, 1060483),
    (AosAttribute::BonusStam, 1060484),
    (AosAttribute::BonusStr, 1060485),
    (AosAttribute::WeaponSpeed, 1060486),
];

fn get_properties(w: &World, id: ItemId, list: &mut PropertyList) {
    (CONTAINER.get_properties)(w, id, list);
    let Some(d) = data(w, id) else { return };
    if let Some(c) = &d.crafter {
        list.add_str(1050043, c); // crafted by ~1_NAME~
    }
    if d.quality == ClothingQuality::Exceptional as i32 {
        list.add(1063341); // exceptional
    }
    let ammo_item = ammo(w, id);
    match ammo_item.and_then(|a| w.item(a)) {
        None => list.add_str(1075265, &format!("0\t{}", d.capacity)), // Ammo: ~1_QUANTITY~/~2_CAPACITY~ arrows
        Some(a) if muo_content::items::is_named(a.ty, "Server.Items.Arrow") => {
            list.add_str(1075265, &format!("{}\t{}", a.amount, d.capacity));
        }
        Some(a) if muo_content::items::is_named(a.ty, "Server.Items.Bolt") => {
            list.add_str(1075266, &format!("{}\t{}", a.amount, d.capacity)); // Ammo: ~1_QUANTITY~/~2_CAPACITY~ bolts
        }
        Some(_) => {}
    }
    if d.damage_increase != 0 {
        list.add_int(1074762, d.damage_increase); // Damage modifier: ~1_PERCENT~%
    }
    let mut split = [0; 7];
    alter_bow_damage(w, id, &mut split);
    let [phys, fire, cold, pois, nrgy, chaos, direct] = split;
    for (v, number) in [
        (phys, 1060403),   // physical damage ~1_val~%
        (fire, 1060405),   // fire damage ~1_val~%
        (cold, 1060404),   // cold damage ~1_val~%
        (pois, 1060406),   // poison damage ~1_val~%
        (nrgy, 1060407),   // energy damage ~1_val
        (chaos, 1072846),  // chaos damage ~1_val~%
        (direct, 1079978), // Direct Damage: ~1_PERCENT~%
    ] {
        if v != 0 {
            list.add_int(number, v);
        }
    }
    list.add(1075085); // Requirement: Mondain's Legacy
    for (a, number) in ATTRIBUTE_LINES {
        let v = attr(w, id, AttrKind::Attributes, a as u32);
        if v != 0 {
            if a == AosAttribute::NightSight {
                list.add(number);
            } else {
                list.add_int(number, v);
            }
        }
    }
    if d.lower_ammo_cost > 0 {
        list.add_int(1075208, d.lower_ammo_cost); // Lower Ammo Cost ~1_Percentage~%
    }
    let weight = ammo_item.map_or(0.0, |a| {
        ops::weight(w, a) + f64::from(w.item(a).map_or(0, |i| i.amount))
    });
    let count = w.item(id).map_or(0, |i| i.items.len());
    let ty = w.item(id).map(|i| i.ty);
    let max_items = ty.map_or(1, |t| (t.default_max_items)(w, id));
    let max_weight = ty.map_or(50, |t| (t.default_max_weight)(w, id));
    // Contents: ~1_COUNT~/~2_MAXCOUNT items, ~3_WEIGHT~/~4_MAXWEIGHT~ stones
    list.add_str(
        1072241,
        &format!("{count}\t{max_items}\t{}\t{max_weight}", weight as i32),
    );
    if d.weight_reduction != 0 {
        list.add_int(1072210, d.weight_reduction); // Weight reduction: ~1_PERCENTAGE~%
    }
}

mod save_flag {
    pub const ATTRIBUTES: u32 = 0x1;
    pub const LOWER_AMMO_COST: u32 = 0x2;
    pub const WEIGHT_REDUCTION: u32 = 0x4;
    pub const DAMAGE_INCREASE: u32 = 0x8;
    pub const CRAFTER: u32 = 0x10;
    pub const QUALITY: u32 = 0x20;
    pub const CAPACITY: u32 = 0x40;
}

fn serialize(w: &World, id: ItemId, out: &mut dyn GenericWriter) {
    (CONTAINER.serialize)(w, id, out);
    out.write_i32(1); // [SerializationGenerator(1, false)]
    let Some(d) = data(w, id) else {
        out.write_enum(4, 0);
        return;
    };
    let mut flags = 0;
    for (on, f) in [
        (!d.attributes.is_empty(), save_flag::ATTRIBUTES),
        (d.lower_ammo_cost != 0, save_flag::LOWER_AMMO_COST),
        (d.weight_reduction != 0, save_flag::WEIGHT_REDUCTION),
        (d.damage_increase != 0, save_flag::DAMAGE_INCREASE),
        (
            d.crafter.as_ref().is_some_and(|c| !c.is_empty()),
            save_flag::CRAFTER,
        ),
        (
            d.quality != ClothingQuality::Regular as i32,
            save_flag::QUALITY,
        ),
        (d.capacity != 0, save_flag::CAPACITY),
    ] {
        if on {
            flags |= f;
        }
    }
    out.write_enum(4, i64::from(flags));
    if flags & save_flag::ATTRIBUTES != 0 {
        d.attributes.serialize(out);
    }
    if flags & save_flag::LOWER_AMMO_COST != 0 {
        out.write_i32(d.lower_ammo_cost);
    }
    if flags & save_flag::WEIGHT_REDUCTION != 0 {
        out.write_i32(d.weight_reduction);
    }
    if flags & save_flag::DAMAGE_INCREASE != 0 {
        out.write_i32(d.damage_increase);
    }
    if flags & save_flag::CRAFTER != 0 {
        out.write_string(d.crafter.as_deref().unwrap_or(""));
    }
    if flags & save_flag::QUALITY != 0 {
        out.write_enum(4, i64::from(d.quality));
    }
    if flags & save_flag::CAPACITY != 0 {
        out.write_i32(d.capacity);
    }
}

fn deserialize(w: &mut World, id: ItemId, r: &mut dyn GenericReader) -> Result<()> {
    (CONTAINER.deserialize)(w, id, r)?;
    let version = r.read_i32()?;
    // `_quality` is `QualityDefaultValue()` (Regular) in the current layout, the enum's default (Low) before it.
    let quality = if version < 1 {
        ClothingQuality::Low
    } else {
        ClothingQuality::Regular
    };
    let mut d = QuiverData {
        quality: quality as i32,
        ..QuiverData::default()
    };
    // Both paths read the same flags: `Deserialize(reader, version)` of version 0 is the old `flags` layout, which
    // is the same save flags with the crafter as a mobile reference (below).
    let flags = if version < 1 {
        r.read_encoded_int()? as u32
    } else {
        r.read_enum(4)? as u32
    };
    if flags & save_flag::ATTRIBUTES != 0 {
        d.attributes = Attrs::deserialize(r)?;
    }
    if version < 1 {
        // The pre-codegen layout: the flags above use 0x1 attributes, 0x4 lower ammo cost, 0x8 weight reduction,
        // 0x10 crafter (a mobile), 0x20 quality, 0x40 capacity and 0x80 damage increase.
        if flags & 0x4 != 0 {
            d.lower_ammo_cost = r.read_i32()?;
        }
        if flags & 0x8 != 0 {
            d.weight_reduction = r.read_i32()?;
        }
        if flags & 0x80 != 0 {
            d.damage_increase = r.read_i32()?;
        }
        if flags & 0x10 != 0 {
            // `Timer.DelayCall((item, crafter) => item._crafter = crafter?.RawName, this, reader.ReadEntity<Mobile>())`
            use muo_server::serialization_ext::ReadEntityExt;
            if let Some(m) = r.read_mobile(w)? {
                let id2 = id;
                muo_server::timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 1, move |w| {
                    let name = muo_server::mobile::part1::props_b::raw_name(w, m);
                    if let Some(d) = data_mut(w, id2) {
                        d.crafter = name;
                    }
                });
            }
        }
        if flags & 0x20 != 0 {
            d.quality = r.read_i32()?;
        }
        if flags & 0x40 != 0 {
            d.capacity = r.read_i32()?;
        }
    } else {
        if flags & save_flag::LOWER_AMMO_COST != 0 {
            d.lower_ammo_cost = r.read_i32()?;
        }
        if flags & save_flag::WEIGHT_REDUCTION != 0 {
            d.weight_reduction = r.read_i32()?;
        }
        if flags & save_flag::DAMAGE_INCREASE != 0 {
            d.damage_increase = r.read_i32()?;
        }
        if flags & save_flag::CRAFTER != 0 {
            d.crafter = r.read_string()?;
        }
        if flags & save_flag::QUALITY != 0 {
            let v = r.read_enum(4)? as i32;
            ClothingQuality::from_i32(v)
                .ok_or_else(|| IoError::InvalidData(format!("ClothingQuality: no value {v}")))?;
            d.quality = v;
        }
        if flags & save_flag::CAPACITY != 0 {
            d.capacity = r.read_i32()?;
        }
    }
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(d);
    }
    Ok(())
}

/// `Server.Items.BaseQuiver` (a concrete class, but with no `[Constructible]` constructor).
pub static BASE_QUIVER: ItemType = ItemType {
    full_name: "Server.Items.BaseQuiver",
    parent: Some(&CONTAINER),
    construct: None,
    default_gump_id: crate::class_props::default_gump_id,
    default_max_items: crate::class_props::default_max_items,
    default_max_weight: crate::class_props::default_max_weight,
    default_weight: crate::class_props::default_weight,
    on_after_duped: |w, id, new_item| {
        // `quiver.Attributes = new AosAttributes(newItem, Attributes)`
        let bag = data(w, id).map(|d| d.attributes.clone());
        if let (Some(bag), Some(d)) = (bag, data_mut(w, new_item)) {
            d.attributes = bag;
        }
    },
    update_total: |w, id, sender, ty, delta| {
        ops::invalidate_properties(w, id);
        (CONTAINER.update_total)(w, id, sender, ty, delta);
    },
    get_total: |w, id, ty| {
        let mut total = (CONTAINER.get_total)(w, id, ty);
        if ty == TotalType::Weight {
            let reduction = data(w, id).map_or(0, |d| d.weight_reduction);
            total -= total * reduction / 100;
        }
        total
    },
    check_hold,
    add_item: |w, id, item| {
        (CONTAINER.add_item)(w, id, item);
        invalidate_weight(w, id);
    },
    remove_item: |w, id, item| {
        (CONTAINER.remove_item)(w, id, item);
        invalidate_weight(w, id);
    },
    // `OnAdded`/`OnRemoved` do not call the base.
    on_added: |w, id, parent| {
        if let EntityId::Mobile(m) = parent {
            add_stat_bonuses(w, id, m);
        }
    },
    on_removed: |w, id, parent| {
        if let EntityId::Mobile(m) = parent {
            remove_stat_bonuses(w, id, m);
        }
    },
    get_properties,
    serialize,
    deserialize,
    ..CONTAINER
};

/// `new T()` of a quiver class: `BaseQuiver(graphic)` with the `graphic` prop of `ty`, then the template of `name`
/// (its own class, or the parent whose constructor a derived class runs).
pub fn quiver_from_template(w: &mut World, ty: &'static ItemType, name: &str) -> ItemId {
    let g = muo_content::items::equip_props::graphic(w, ty);
    let id = new_quiver(w, ty, g);
    muo_content::templates::apply_item_as(w, id, name);
    id
}

/// `ElvenQuiver()`: its template (`WeightReduction = 30`).
pub fn elven_quiver_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    quiver_from_template(w, ty, "Server.Items.ElvenQuiver")
}

/// `Server.Items.ElvenQuiver` (`[Flippable(0x2FB7, 0x3171)]`, `LabelNumber => 1032657`).
pub static ELVEN_QUIVER: ItemType = ItemType {
    full_name: "Server.Items.ElvenQuiver",
    parent: Some(&BASE_QUIVER),
    construct: Some(|w| elven_quiver_new(w, &ELVEN_QUIVER)),
    label_number: crate::class_props::label_number, // elven quiver
    serialize: |w, id, out| {
        (BASE_QUIVER.serialize)(w, id, out);
        out.write_encoded_int(0);
    },
    deserialize: |w, id, r| {
        (BASE_QUIVER.deserialize)(w, id, r)?;
        let _version = r.read_encoded_int()?;
        Ok(())
    },
    ..BASE_QUIVER
};

/// `[Flippable(a, b)]` of the elven quivers.
pub const ELVEN_QUIVER_FLIPPABLE: (i32, i32) = (0x2FB7, 0x3171);

/// The four quivers that tint an elven quiver: `ElvenQuiver()`, then their template (`Hue = hue`); the label and
/// the damage split their `AlterBowDamage` sets are props.
macro_rules! elemental_quiver {
    ($($(#[$m:meta])* $name:ident, $new:ident = $full:literal;)*) => {$(
        /// `new T()`: `ElvenQuiver()` then the class's template.
        pub fn $new(w: &mut World, ty: &'static ItemType) -> ItemId {
            let id = elven_quiver_new(w, ty);
            muo_content::templates::apply_item_as(w, id, $full);
            id
        }

        $(#[$m])*
        pub static $name: ItemType = ItemType {
            full_name: $full,
            parent: Some(&ELVEN_QUIVER),
            construct: Some(|w| $new(w, &$name)),
            label_number: crate::class_props::label_number,
            serialize: |w, id, out| {
                (ELVEN_QUIVER.serialize)(w, id, out);
                out.write_encoded_int(0);
            },
            deserialize: |w, id, r| {
                (ELVEN_QUIVER.deserialize)(w, id, r)?;
                let _version = r.read_encoded_int()?;
                Ok(())
            },
            ..ELVEN_QUIVER
        };
    )*};
}

elemental_quiver! {
    /// `Server.Items.QuiverOfFire`.
    QUIVER_OF_FIRE, quiver_of_fire_new = "Server.Items.QuiverOfFire";
    /// `Server.Items.QuiverOfIce`.
    QUIVER_OF_ICE, quiver_of_ice_new = "Server.Items.QuiverOfIce";
    /// `Server.Items.QuiverOfBlight`.
    QUIVER_OF_BLIGHT, quiver_of_blight_new = "Server.Items.QuiverOfBlight";
    /// `Server.Items.QuiverOfLightning`.
    QUIVER_OF_LIGHTNING, quiver_of_lightning_new = "Server.Items.QuiverOfLightning";
}

// ---- hooks the weapon code calls ----

/// `AlterBowDamage(ref phys, ref fire, ref cold, ref pois, ref nrgy, ref chaos, ref direct)`: the class's
/// `bowDamageSplit` prop (those seven, in that order) replaces the split; without one (`BaseQuiver`) it is kept.
pub fn alter_bow_damage(w: &World, q: ItemId, s: &mut [i32; 7]) {
    if let Some(serde_json::Value::Array(a)) =
        muo_content::templates::props::item(w, q, "bowDamageSplit")
        && a.len() == 7
    {
        for (d, v) in s.iter_mut().zip(a) {
            *d = v.as_i64().and_then(|n| i32::try_from(n).ok()).unwrap_or(0);
        }
    }
}

fn quiver_of(w: &World, m: MobileId) -> Option<ItemId> {
    muo_server::mobile::part2::items::find_item_on_layer(w, m, Layer::Cloak)
        .filter(|&i| is_quiver(w, i))
}

/// Sets the `WeaponHooks` that read the quiver on `Layer.Cloak`.
fn configure(w: &mut World) {
    // `BaseQuiver : ICraftable`: the craft system asks every craftable class in turn.
    muo_content::craft::hooks_mut(w)
        .craftables
        .push(|w, id, a| {
            w.item(id)
                .is_some_and(|i| i.ty.is(&BASE_QUIVER))
                .then(|| on_craft(w, id, a))
        });
    let h = w.resources.get_mut::<WeaponHooks>();
    h.quiver_of = Some(quiver_of);
    h.quiver_lower_ammo_cost = Some(lower_ammo_cost);
    h.quiver_invalidate_weight = Some(invalidate_weight);
    h.quiver_alter_bow_damage = Some(|w, attacker, split| {
        let mut s = split;
        if let Some(q) = quiver_of(w, attacker) {
            alter_bow_damage(w, q, &mut s);
        }
        s
    });
}

/// Registers the classes of this file.
pub fn register(types: &mut TypeRegistry) {
    for t in [
        &BASE_QUIVER,
        &ELVEN_QUIVER,
        &QUIVER_OF_FIRE,
        &QUIVER_OF_ICE,
        &QUIVER_OF_BLIGHT,
        &QUIVER_OF_LIGHTNING,
    ] {
        types.register_item(t, &[]);
    }
    types.register_phase_default(
        Phase::Configure,
        "Server.Items.BaseQuiver.Configure",
        configure,
    );
}
