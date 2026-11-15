//! `BaseJewel` (save version 5) and the jewelry base classes: `BaseBracelet`, `BaseEarrings`, `BaseNecklace`,
//! `BaseRing`.
//!
//! State is [`JewelData`] in `Item::ext`, the constants of a class are [`JewelClass`]. A leaf is one `jewel_leaf!`
//! call in `items/leaves.rs`. The layout has no save flags; version 4 and the pre-codegen versions below it have
//! the same fields as version 5 minus the gem count.

use muo_io::{GenericReader, GenericWriter, Result};
use muo_server::World;
use muo_server::ext::Ext;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::base;
use muo_server::item::ops;
use muo_server::item::{ITEM_BASE, ItemType};
use muo_server::layer::Layer;
use muo_server::property_list::PropertyList;

use super::equip_hooks::{CraftArgs, hooks, lower_text};
use super::equip_props as ep;
use crate::templates::item_props as ip;
use super::{core_uotd, parent_mobile, random_min_max, read_enum};
use crate::aos::{
    AosAttribute, AosElementAttribute, AttrKind, Attrs, add_attribute_properties,
    add_skill_bonus_properties,
};
use crate::core;
use crate::craft::{self, CraftResource};

int_enum! {
    /// `GemType`.
    pub enum GemType {
        None = 0, StarSapphire = 1, Emerald = 2, Sapphire = 3, Ruby = 4, Citrine = 5, Amethyst = 6,
        Tourmaline = 7, Amber = 8, Diamond = 9,
    }
}

/// The `override`s of a jewel class.
#[derive(Copy, Clone, Debug)]
pub struct JewelClass {
    /// The `layer` the abstract class passes to `BaseJewel(itemID, layer)`.
    pub layer: Layer,
    pub default_weight: Option<f64>,
    /// `LabelNumber` when overridden (0 keeps the gem or tiledata name).
    pub label_number: i32,
    /// `RequiredRaces` when overridden.
    pub required_races: Option<i32>,
    /// `BaseGemTypeNumber`.
    pub base_gem_type_number: i32,
    pub artifact_rarity: i32,
    pub init_min_hits: i32,
    pub init_max_hits: i32,
}

/// `BaseJewel`'s defaults.
pub static BASE_JEWEL_CLASS: JewelClass = JewelClass {
    layer: Layer::Invalid,
    default_weight: None,
    label_number: 0,
    required_races: None,
    base_gem_type_number: 0,
    artifact_rarity: 0,
    init_min_hits: 0,
    init_max_hits: 0,
};

/// Everything `BaseJewel` adds to `Item`.
#[derive(Clone, Debug)]
pub struct JewelData {
    pub attributes: Attrs,
    pub resistances: Attrs,
    pub skill_bonuses: Attrs,
    pub hit_points: i32,
    pub max_hit_points: i32,
    pub resource: CraftResource,
    pub gem_type: GemType,
    pub gem_count: i32,
}

impl JewelData {
    fn for_load() -> JewelData {
        JewelData {
            attributes: Attrs::default(),
            resistances: Attrs::default(),
            skill_bonuses: Attrs::default(),
            hit_points: 0,
            max_hit_points: 0,
            resource: CraftResource::None,
            gem_type: GemType::None,
            gem_count: 0,
        }
    }
}

/// The jewel state of an item.
pub fn data(w: &World, id: ItemId) -> Option<&JewelData> {
    w.item(id)?.ext.get()
}

/// The jewel state of an item, mutable.
pub fn data_mut(w: &mut World, id: ItemId) -> Option<&mut JewelData> {
    w.item_mut(id)?.ext.get_mut()
}

/// The constants of the item's class.
pub fn class(w: &World, id: ItemId) -> &'static JewelClass {
    w.item(id)
        .and_then(|i| i.ty.family::<JewelClass>())
        .unwrap_or(&BASE_JEWEL_CLASS)
}

/// `LabelNumber` when overridden: the class's template prop `labelNumber`, else its table field (0 keeps the
/// tiledata name).
pub fn class_label_number(w: &World, id: ItemId) -> i32 {
    crate::templates::item_props::int(w, id, "labelNumber", class(w, id).label_number)
}

/// `ArtifactRarity`: the class's template prop `artifactRarity`, else its table field.
pub fn artifact_rarity(w: &World, id: ItemId) -> i32 {
    crate::templates::item_props::int(w, id, "artifactRarity", class(w, id).artifact_rarity)
}

/// `item is BaseJewel`.
pub fn is_jewel(w: &World, id: ItemId) -> bool {
    w.item(id).is_some_and(|i| i.ty.is(&BASE_JEWEL))
}

/// `new BaseJewel(itemID, layer)` for class `ty` (the layer is its class's); every leaf constructor starts here.
pub fn new_jewel(w: &mut World, ty: &'static ItemType, item_id: i32) -> ItemId {
    let id = ops::new_item_with_id(w, ty, item_id);
    let c = ty.family::<JewelClass>().unwrap_or(&BASE_JEWEL_CLASS);
    crate::templates::table(w); // the class props below read the loaded table
    let (min, max) = (
        ip::int(w, id, "initMinHits", c.init_min_hits),
        ip::int(w, id, "initMaxHits", c.init_max_hits),
    );
    let hits = random_min_max(w, min, max);
    let mut d = JewelData::for_load();
    d.resource = CraftResource::Iron;
    d.hit_points = hits;
    d.max_hit_points = hits;
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(d);
    }
    // Hue = CraftResources.GetHue(_resource)
    let hue = craft::hue(w, CraftResource::Iron);
    (ty.set_hue)(w, id, hue);
    (ty.set_layer)(w, id, c.layer);
    id
}

// ---- setters with side effects ----

/// `Resource` setter (`OnResourceChanged`: the hue follows).
pub fn set_resource(w: &mut World, id: ItemId, value: CraftResource) {
    let Some(d) = data_mut(w, id) else { return };
    if d.resource == value {
        return;
    }
    d.resource = value;
    if let Some(ty) = w.item(id).map(|i| i.ty) {
        let hue = craft::hue(w, value);
        (ty.set_hue)(w, id, hue);
    }
}

/// `GemType` setter.
pub fn set_gem_type(w: &mut World, id: ItemId, value: GemType) {
    if let Some(d) = data_mut(w, id)
        && d.gem_type != value
    {
        d.gem_type = value;
        ops::invalidate_properties(w, id);
    }
}

/// `GemCount` setter.
pub fn set_gem_count(w: &mut World, id: ItemId, value: i32) {
    if let Some(d) = data_mut(w, id) {
        d.gem_count = value;
    }
}

/// `MaxHitPoints` setter.
pub fn set_max_hit_points(w: &mut World, id: ItemId, value: i32) {
    if let Some(d) = data_mut(w, id)
        && d.max_hit_points != value
    {
        d.max_hit_points = value;
        ops::invalidate_properties(w, id);
    }
}

/// `HitPoints` setter: ignored without `MaxHitPoints`; below zero the jewel is destroyed.
pub fn set_hit_points(w: &mut World, id: ItemId, value: i32) {
    let Some(d) = data_mut(w, id) else { return };
    if value == d.hit_points || d.max_hit_points <= 0 {
        return;
    }
    d.hit_points = value;
    if value < 0 {
        if let Some(ty) = w.item(id).map(|i| i.ty) {
            (ty.delete)(w, id);
        }
    } else if let Some(d) = data_mut(w, id)
        && d.hit_points > d.max_hit_points
    {
        d.hit_points = d.max_hit_points;
    }
    ops::invalidate_properties(w, id);
}

fn resist(w: &World, id: ItemId, e: AosElementAttribute) -> i32 {
    super::attr(w, id, AttrKind::Element, e as u32)
}

// ---- crafting ----

/// `OnCraft(..)` (not virtual): the resource, then the gem the recipe or the craft context names.
pub fn on_craft(w: &mut World, id: ItemId, a: &CraftArgs) -> i32 {
    set_resource(w, id, a.resource);
    if a.do_not_color
        && let Some(ty) = w.item(id).map(|i| i.ty)
    {
        (ty.set_hue)(w, id, 0);
    }
    if let Some(gem) = a.second_resource {
        set_gem_type(w, id, gem);
    }
    // T2A jewelry: the gem info comes from the craft context. The entire targeted gem stack is consumed and the
    // piece is named by that count.
    if let Some((gem, count)) = a.pending_gem
        && gem != GemType::None
        && count > 0
    {
        let consumed = hooks(w)
            .and_then(|h| h.consume_gems)
            .is_some_and(|f| f(w, a.from, gem, count));
        if consumed {
            set_gem_type(w, id, gem);
            set_gem_count(w, id, count);
        } else {
            // Gems were no longer available (or unknown type): craft a plain piece rather than naming it for gems
            // that were never consumed.
            muo_server::mobile::part2::messages::send_ascii_message(
                w,
                a.from,
                "You lack the gemstones to set into this piece.",
            );
        }
    }
    1
}

// ---- wearing ----

fn bonuses(w: &World, id: ItemId) -> (i32, i32, i32) {
    let a = |x: AosAttribute| super::attr(w, id, AttrKind::Attributes, x as u32);
    (
        a(AosAttribute::BonusStr),
        a(AosAttribute::BonusDex),
        a(AosAttribute::BonusInt),
    )
}

fn on_added(w: &mut World, id: ItemId, parent: EntityId) {
    if core::aos(w)
        && let EntityId::Mobile(from) = parent
    {
        super::skill_bonuses_add_to(w, id, from);
        let (s, d, i) = bonuses(w, id);
        super::add_stat_mods(w, id, from, s, d, i);
        if let Some(ty) = w.mobile(from).map(|m| m.ty) {
            (ty.check_stat_timers)(w, from);
        }
    }
}

fn on_removed(w: &mut World, id: ItemId, parent: EntityId) {
    if core::aos(w)
        && let EntityId::Mobile(from) = parent
    {
        super::skill_bonuses_remove(w, id);
        super::remove_stat_mods(w, id, from);
        if let Some(ty) = w.mobile(from).map(|m| m.ty) {
            (ty.check_stat_timers)(w, from);
        }
    }
}

fn on_after_duped(w: &mut World, id: ItemId, new_item: ItemId) {
    let Some(src) = data(w, id).cloned() else {
        return;
    };
    let hue = w.item(id).map_or(0, |i| i.hue);
    let Some(ty) = w.item(new_item).map(|i| i.ty) else {
        return;
    };
    let Some(d) = data_mut(w, new_item) else {
        return;
    };
    d.attributes = src.attributes.clone();
    d.resistances = src.resistances.clone();
    d.skill_bonuses = src.skill_bonuses.clone();
    // Set hue again because of resource
    (ty.set_hue)(w, new_item, hue);
}

fn label_number(w: &World, id: ItemId) -> i32 {
    let c = class(w, id);
    let n = class_label_number(w, id);
    if n != 0 {
        return n;
    }
    match data(w, id).map(|d| d.gem_type) {
        Some(g) if g != GemType::None => c.base_gem_type_number + g as i32 - 1,
        _ => base::label_number_base(w, id),
    }
}

fn get_properties(w: &World, id: ItemId, list: &mut PropertyList) {
    base::get_properties_base(w, id, list);
    let (Some(d), Some(ty)) = (data(w, id), w.item(id).map(|i| i.ty)) else {
        return;
    };
    let aos = core::aos(w);
    add_skill_bonus_properties(&d.skill_bonuses, list, aos);
    let rarity = artifact_rarity(w, id);
    if rarity > 0 {
        list.add_int(1061078, rarity); // artifact rarity ~1_val~
    }
    add_attribute_properties(&d.attributes, list, aos, core::ml(w), 0, 0, 0);
    (ty.add_resistance_properties)(w, id, list);
    if d.hit_points >= 0 && d.max_hit_points > 0 {
        // durability ~1_val~ / ~2_val~
        list.add_number_fmt(
            1060639,
            format_args!("{}\t{}", d.hit_points, d.max_hit_points),
        );
    }
}

fn on_single_click(w: &mut World, id: ItemId, from: MobileId) {
    if !core_uotd(w) {
        on_single_click_pre_uotd(w, id, from);
        return;
    }
    base::on_single_click_base(w, id, from);
}

/// `GetGemName(type, plural)`.
fn gem_name(t: GemType, plural: bool) -> &'static str {
    match (t, plural) {
        (GemType::StarSapphire, true) => "star sapphires",
        (GemType::StarSapphire, false) => "a star sapphire",
        (GemType::Emerald, true) => "emeralds",
        (GemType::Emerald, false) => "an emerald",
        (GemType::Sapphire, true) => "sapphires",
        (GemType::Sapphire, false) => "a sapphire",
        (GemType::Ruby, true) => "rubies",
        (GemType::Ruby, false) => "a ruby",
        (GemType::Citrine, true) => "citrines",
        (GemType::Citrine, false) => "a citrine",
        (GemType::Amethyst, true) => "amethysts",
        (GemType::Amethyst, false) => "an amethyst",
        (GemType::Tourmaline, true) => "tourmalines",
        (GemType::Tourmaline, false) => "a tourmaline",
        (GemType::Amber, true) => "ambers",
        (GemType::Amber, false) => "an amber",
        (GemType::Diamond, true) => "diamonds",
        (GemType::Diamond, false) => "a diamond",
        (_, true) => "gems",
        (_, false) => "a gem",
    }
}

/// `OnSingleClickPreUOTD(from)`.
pub fn on_single_click_pre_uotd(w: &mut World, id: ItemId, from: MobileId) {
    let Some((d, ty)) = data(w, id).cloned().zip(w.item(id).map(|i| i.ty)) else {
        return;
    };
    let plural = d.gem_count > 1;
    let name = if super::is_named(ty, "Server.Items.WeddingRing") {
        format!("a {}", ops::name(w, id).unwrap_or_default())
    } else {
        ops::name(w, id).unwrap_or_else(|| {
            let article_an = ops::item_data(w, id)
                .flags
                .contains(muo_data::tile_data::TileFlag::ARTICLE_AN);
            let text = lower_text(w, (ty.label_number)(w, id));
            format!("{} {text}", if article_an { "an" } else { "a" })
        })
    };
    let text = if d.gem_type != GemType::None && d.gem_count > 0 {
        let gem = gem_name(d.gem_type, plural);
        if plural {
            format!("{name} with {} {gem}", d.gem_count)
        } else {
            format!("{name} with {gem}")
        }
    } else {
        name
    };
    ops::label_to(w, id, from, &text);
}

// ---- save layout (BaseJewel version 5) ----

const SERIALIZATION_VERSION: i32 = 5;

fn serialize(w: &World, id: ItemId, writer: &mut dyn GenericWriter) {
    (ITEM_BASE.serialize)(w, id, writer);
    let Some(d) = data(w, id) else { return };
    writer.write_i32(SERIALIZATION_VERSION);
    writer.write_encoded_int(d.max_hit_points);
    writer.write_encoded_int(d.hit_points);
    writer.write_enum(4, d.resource as i64);
    writer.write_enum(4, d.gem_type as i64);
    d.attributes.serialize(writer);
    d.resistances.serialize(writer);
    d.skill_bonuses.serialize(writer);
    writer.write_encoded_int(d.gem_count);
}

fn deserialize(w: &mut World, id: ItemId, reader: &mut dyn GenericReader) -> Result<()> {
    (ITEM_BASE.deserialize)(w, id, reader)?;
    let version = reader.read_i32()?;
    let mut d = JewelData::for_load();
    d.max_hit_points = reader.read_encoded_int()?;
    d.hit_points = reader.read_encoded_int()?;
    d.resource = read_enum(reader, CraftResource::from_i32, "CraftResource")?;
    d.gem_type = read_enum(reader, GemType::from_i32, "GemType")?;
    d.attributes = Attrs::deserialize(reader)?;
    d.resistances = Attrs::deserialize(reader)?;
    d.skill_bonuses = Attrs::deserialize(reader)?;
    if version >= SERIALIZATION_VERSION {
        d.gem_count = reader.read_encoded_int()?;
    }
    // Versions 4 and below (`V4Content`, the pre-codegen `Deserialize(reader, version)`) are the same without the
    // gem count, which defaults to 0.
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(d);
    }
    after_deserialization(w, id);
    Ok(())
}

/// `[AfterDeserialization]`.
fn after_deserialization(w: &mut World, id: ItemId) {
    let Some(m) = parent_mobile(w, id) else {
        return;
    };
    if core::aos(w) {
        super::skill_bonuses_add_to(w, id, m);
    }
    let (s, d, i) = bonuses(w, id);
    if s != 0 || d != 0 || i != 0 {
        super::add_stat_mods(w, id, m, s, d, i);
    }
    if let Some(ty) = w.mobile(m).map(|m| m.ty) {
        (ty.check_stat_timers)(w, m);
    }
}

// ---- class chain ----

/// `Server.Items.BaseJewel`.
pub static BASE_JEWEL: ItemType = ItemType {
    full_name: "Server.Items.BaseJewel",
    parent: Some(&ITEM_BASE),
    construct: None,
    family: Some(&BASE_JEWEL_CLASS),
    default_weight: |w, id| {
        ep::float_opt(w, id, "defaultWeight", class(w, id).default_weight)
            .unwrap_or_else(|| base::default_weight_base(w, id))
    },
    label_number,
    required_races: |w, id| {
        ep::required_races(w, id, class(w, id).required_races)
            .unwrap_or_else(|| base::required_races_base(w, id))
    },
    physical_resistance: |w, id| resist(w, id, AosElementAttribute::Physical),
    fire_resistance: |w, id| resist(w, id, AosElementAttribute::Fire),
    cold_resistance: |w, id| resist(w, id, AosElementAttribute::Cold),
    poison_resistance: |w, id| resist(w, id, AosElementAttribute::Poison),
    energy_resistance: |w, id| resist(w, id, AosElementAttribute::Energy),
    on_after_duped,
    on_added,
    on_removed,
    get_properties,
    on_single_click,
    serialize,
    deserialize,
    ..ITEM_BASE
};

/// Declares an abstract jewel class: its layer and `BaseGemTypeNumber`, `[SerializationGenerator(0, false)]`.
macro_rules! jewel_base {
    ($(#[$m:meta])* $name:ident, $class:ident = $full:literal, $layer:expr, $gem:expr) => {
        /// The class table.
        pub static $class: JewelClass =
            JewelClass { layer: $layer, base_gem_type_number: $gem, ..BASE_JEWEL_CLASS };
        $(#[$m])*
        pub static $name: ItemType = ItemType {
            full_name: $full,
            parent: Some(&BASE_JEWEL),
            family: Some(&$class),
            serialize: |w, id, out| {
                (BASE_JEWEL.serialize)(w, id, out);
                out.write_i32(0);
            },
            deserialize: |w, id, r| {
                (BASE_JEWEL.deserialize)(w, id, r)?;
                let _version = r.read_i32()?;
                Ok(())
            },
            ..BASE_JEWEL
        };
    };
}

jewel_base!(
    /// `Server.Items.BaseBracelet` (star sapphire bracelet is 1044221).
    BASE_BRACELET, BASE_BRACELET_CLASS = "Server.Items.BaseBracelet", Layer::Bracelet, 1044221
);
jewel_base!(
    /// `Server.Items.BaseEarrings` (star sapphire earrings is 1044203).
    BASE_EARRINGS, BASE_EARRINGS_CLASS = "Server.Items.BaseEarrings", Layer::Earrings, 1044203
);
jewel_base!(
    /// `Server.Items.BaseNecklace` (star sapphire necklace is 1044241).
    BASE_NECKLACE, BASE_NECKLACE_CLASS = "Server.Items.BaseNecklace", Layer::Neck, 1044241
);
jewel_base!(
    /// `Server.Items.BaseRing` (star sapphire ring is 1044176).
    BASE_RING, BASE_RING_CLASS = "Server.Items.BaseRing", Layer::Ring, 1044176
);
