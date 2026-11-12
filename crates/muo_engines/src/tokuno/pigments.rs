//! `BasePigmentsOfTokuno` and its pigment classes (`PigmentsOfTokuno`, `LesserPigmentsOfTokuno`,
//! `MetalPigmentsOfTokuno`).
//!
//! `Label` is not serialized: a pigment loaded from a save has no label line (the deserializer assigns the
//! fields, not the properties, so `OnTypeChanged` does not run either).

use muo_content::craft;
use muo_content::items::{armor, clothing, weapon};
use muo_content::props::{Ctor, Kind, Param, Value, add_ctors};
use muo_content::utilities::types::{Numeric, PropType};
use muo_server::World;
use muo_server::ext::Ext;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, base, ops};
use muo_server::target::{TargetFlags, Targeted};
use muo_server::type_registry::{DEFAULT_CALL_PRIORITY, Phase, TypeRegistry};

use super::{in_names, lists};
use crate::util::hue;

/// What `BasePigmentsOfTokuno` and its subclasses hold.
#[derive(Clone, Debug, Default)]
pub struct PigmentData {
    /// `_usesRemaining`.
    pub uses: i32,
    /// `Label` as a cliloc number (always a number); 0 is the default `TextDefinition`.
    pub label: i32,
    /// `_type` (`PigmentType` or `LesserPigmentType`).
    pub kind: i32,
}

/// `PigmentsOfTokuno._table`: hue and label per `PigmentType`.
static TABLE: [(i32, i32); 20] = [
    (0, -1),
    (0x501, 1070987),
    (0x486, 1070988),
    (0x4F2, 1070989),
    (0x47E, 1070990),
    (0x48F, 1070991),
    (0x455, 1070992),
    (0x21, 1070993),
    (0x58C, 1070994),
    (0x66C, 1070995),
    (0x54F, 1070996),
    (0x96A, 1079579),
    (0x96B, 1079580),
    (0x972, 1079581),
    (0x977, 1079582),
    (0x97C, 1079583),
    (0x96C, 1079584),
    (0x975, 1079585),
    (0x97B, 1079586),
    (0x97E, 1079587),
];

/// `LesserPigmentsOfTokuno._table`.
static LESSER_TABLE: [(i32, i32); 11] = [
    (0, -1),
    (0x02E, 1071458),
    (0x4B9, 1071455),
    (0x005, 1071459),
    (0x3E9, 1071451),
    (0x227, 1071457),
    (0x1C8, 1071454),
    (0x24F, 1071456),
    (0x145, 1071450),
    (0x3F0, 1071452),
    (0x41A, 1071453),
];

/// `PigmentType.FadedCoal`.
const FADED_COAL: i32 = 10;

/// `PigmentsOfTokuno.GetInfo(type)`: `[hue, label]`, `None`'s for an unknown value.
pub fn get_info(kind: i32) -> (i32, i32) {
    TABLE
        .get(usize::try_from(kind).unwrap_or(usize::MAX))
        .copied()
        .unwrap_or(TABLE[0])
}

/// `LesserPigmentsOfTokuno.GetInfo(type)`.
pub fn lesser_get_info(kind: i32) -> (i32, i32) {
    LESSER_TABLE
        .get(usize::try_from(kind).unwrap_or(usize::MAX))
        .copied()
        .unwrap_or(LESSER_TABLE[0])
}

pub fn data(w: &World, id: ItemId) -> Option<&PigmentData> {
    w.item(id)?.ext.get::<PigmentData>()
}

fn data_mut(w: &mut World, id: ItemId) -> Option<&mut PigmentData> {
    w.item_mut(id)?.ext.get_mut::<PigmentData>()
}

/// `PigmentsOfTokuno.Type`.
pub fn pigment_type(w: &World, id: ItemId) -> i32 {
    data(w, id).map_or(0, |d| d.kind)
}

/// `BasePigmentsOfTokuno(uses)`: `base(0xEFF)`.
pub fn base_new(w: &mut World, ty: &'static ItemType, uses: i32) -> ItemId {
    let id = ops::new_item_with_id(w, ty, 0xEFF);
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(PigmentData {
            uses,
            label: 0,
            kind: 0,
        });
    }
    id
}

/// `Label = value` (invalidates the properties).
fn set_label(w: &mut World, id: ItemId, label: i32) {
    if let Some(d) = data_mut(w, id) {
        d.label = label;
    }
    ops::invalidate_properties(w, id);
}

/// `UsesRemaining = value`.
pub fn set_uses_remaining(w: &mut World, id: ItemId, value: i32) {
    let changed = data_mut(w, id).is_some_and(|d| {
        let c = d.uses != value;
        d.uses = value;
        c
    });
    if changed {
        ops::invalidate_properties(w, id);
    }
}

pub fn uses_remaining(w: &World, id: ItemId) -> i32 {
    data(w, id).map_or(0, |d| d.uses)
}

/// The `Type` setter of `PigmentsOfTokuno`: `OnTypeChanged` when the value changes.
pub fn set_type(w: &mut World, id: ItemId, value: i32) {
    let old = pigment_type(w, id);
    if old == value {
        return;
    }
    if let Some(d) = data_mut(w, id) {
        d.kind = value;
    }
    let (h, label) = get_info_or_none(value, TABLE.len(), get_info);
    hue(w, id, h);
    set_label(w, id, label);
}

/// The `Type` setter of `LesserPigmentsOfTokuno`.
pub fn set_lesser_type(w: &mut World, id: ItemId, value: i32) {
    let old = pigment_type(w, id);
    if old == value {
        return;
    }
    if let Some(d) = data_mut(w, id) {
        d.kind = value;
    }
    let (h, label) = get_info_or_none(value, LESSER_TABLE.len(), lesser_get_info);
    hue(w, id, h);
    set_label(w, id, label);
}

/// `OnTypeChanged`: `v >= 0 && v < _table.Length` ? the row : `Hue = 0; Label = -1`.
fn get_info_or_none(v: i32, len: usize, info: fn(i32) -> (i32, i32)) -> (i32, i32) {
    if v >= 0 && (v as usize) < len {
        info(v)
    } else {
        (0, -1)
    }
}

/// `PigmentsOfTokuno(type, uses)`.
pub fn pigments_new(w: &mut World, kind: i32, uses: Option<i32>) -> ItemId {
    let uses = uses.unwrap_or(if kind == 0 || kind >= FADED_COAL {
        10
    } else {
        50
    });
    let id = base_new(w, &PIGMENTS_OF_TOKUNO, uses);
    set_type(w, id, kind);
    id
}

/// `LesserPigmentsOfTokuno(type)`: `base(1)`.
pub fn lesser_new(w: &mut World, kind: i32) -> ItemId {
    let id = base_new(w, &LESSER_PIGMENTS_OF_TOKUNO, 1);
    set_lesser_type(w, id, kind);
    id
}

/// `MetalPigmentsOfTokuno()`: `base(1)`, `RandomHue(); Label = -1`.
pub fn metal_new(w: &mut World) -> ItemId {
    let id = base_new(w, &METAL_PIGMENTS_OF_TOKUNO, 1);
    metal_random_hue(w, id);
    set_label(w, id, -1);
    id
}

/// `MetalPigmentsOfTokuno.RandomHue()`.
pub fn metal_random_hue(w: &mut World, id: ItemId) {
    let a = muo_content::hooks::rnd::from(w, 0, 30);
    hue(w, id, if a != 0 { a + 0x960 } else { 0 });
}

fn base_get_properties(w: &World, id: ItemId, list: &mut muo_server::property_list::PropertyList) {
    (ITEM_BASE.get_properties)(w, id, list);
    let d = data(w, id).cloned().unwrap_or_default();
    if d.label > 0 {
        list.add(d.label);
    }
    list.add_int(1060584, d.uses); // uses remaining: ~1_val~
}

fn on_double_click(w: &mut World, id: ItemId, from: MobileId) {
    let loc = ops::get_world_location(w, id);
    if base::is_accessible_to(w, id, from) && muo_content::mx::in_range_point(w, from, loc, 3) {
        muo_content::mx::msg(w, from, 1070929); // Select the artifact or enhanced magic item to dye.
        muo_server::target::begin_target(
            w,
            from,
            3,
            false,
            TargetFlags::NONE,
            move |w, from, t| internal_callback(w, id, from, t),
        );
    } else {
        muo_content::mx::msg(w, from, 502436); // That is not accessible.
    }
}

fn internal_callback(w: &mut World, id: ItemId, from: MobileId, targeted: Targeted) {
    let loc = ops::get_world_location(w, id);
    if ops::is_deleted(w, id)
        || uses_remaining(w, id) <= 0
        || !muo_content::mx::in_range_point(w, from, loc, 3)
        || !base::is_accessible_to(w, id, from)
    {
        return;
    }
    let Targeted::Entity(EntityId::Item(i)) = targeted else {
        muo_content::mx::msg(w, from, 1070931); // You can only dye artifacts and enhanced magic items with this tub.
        return;
    };
    let Some(item) = w.item(i) else {
        muo_content::mx::msg(w, from, 1070931);
        return;
    };
    let (ity, worn, quest) = (
        item.ty,
        item.parent == Some(EntityId::Mobile(from)),
        item.quest_item(),
    );
    let iloc = ops::get_world_location(w, i);
    if !muo_content::mx::in_range_point(w, from, iloc, 3) || !base::is_accessible_to(w, id, from) {
        muo_content::mx::msg(w, from, 502436); // That is not accessible.
    } else if worn {
        muo_content::mx::msg(w, from, 1070930); // Can't dye artifacts or enhanced magic items that are being worn.
    } else if ops::is_locked_down(w, i) {
        // You may not dye artifacts and enhanced magic items which are locked down.
        muo_content::mx::msg(w, from, 1070932);
    } else if quest {
        muo_content::mx::msg(w, from, 1151836); // You may not dye toggled quest items.
    } else if in_names(
        ity,
        &[
            "Server.Items.MetalPigmentsOfTokuno",
            "Server.Items.LesserPigmentsOfTokuno",
            "Server.Items.PigmentsOfTokuno",
        ],
    ) {
        muo_content::mx::msg(w, from, 1042417); // You cannot dye that.
    } else if !is_valid_item(w, i) {
        // You can only dye artifacts and enhanced magic items with this tub.
        muo_content::mx::msg(w, from, 1070931);
    } else {
        // On OSI there is no hue check to see if it's already hued, and no message on success.
        let h = w.item(id).map_or(0, |x| x.hue);
        hue(w, i, h);
        let left = uses_remaining(w, id) - 1;
        set_uses_remaining(w, id, left);
        if left <= 0
            && let Some(t) = w.item(id).map(|x| x.ty)
        {
            (t.delete)(w, id);
        }
        muo_content::mx::play_sound(w, from, 0x23E); // As per OSI TC1
    }
}

/// `BasePigmentsOfTokuno.IsValidItem(i)`.
pub fn is_valid_item(w: &World, i: ItemId) -> bool {
    let Some(item) = w.item(i) else { return false };
    let ty = item.ty;
    if in_names(ty, &["Server.Items.BasePigmentsOfTokuno"]) {
        return false;
    }
    let resource = if let Some(d) = weapon::data(w, i) {
        d.resource
    } else if let Some(d) = armor::data(w, i) {
        d.resource
    } else if let Some(d) = clothing::data(w, i) {
        d.resource
    } else {
        craft::CraftResource::None
    };
    if !craft::is_standard(resource) {
        return true;
    }
    let greater = super::gumps::greater_artifacts(super::reward_era(w));
    in_names(ty, lists::TOKUNO_DYABLE)
        || in_names(ty, lists::LESSER_ARTIFACTS_TOTAL)
        || in_names(ty, &greater)
        || in_names(ty, lists::DEMON_KNIGHT_RARITY_10)
        || in_names(ty, lists::DEMON_KNIGHT_RARITY_11)
        || in_names(ty, lists::MONDAINS_LEGACY)
        || in_names(ty, lists::STEALABLE)
        || in_names(ty, lists::PARAGON)
        || in_names(ty, lists::LEVIATHAN)
        || in_names(ty, lists::TREASURE_MAP_CHEST)
        || in_names(ty, lists::REPLICAS)
        || in_names(ty, lists::DYABLE_HERITAGE_ITEMS)
        || in_names(ty, lists::GLASSES)
}

fn base_serialize(w: &World, id: ItemId, out: &mut dyn muo_io::GenericWriter) {
    (ITEM_BASE.serialize)(w, id, out);
    out.write_i32(0);
    out.write_encoded_int(uses_remaining(w, id));
}

fn base_deserialize(
    w: &mut World,
    id: ItemId,
    r: &mut dyn muo_io::GenericReader,
) -> muo_io::Result<()> {
    (ITEM_BASE.deserialize)(w, id, r)?;
    let _version = r.read_i32()?;
    let uses = r.read_encoded_int()?;
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(PigmentData {
            uses,
            label: 0,
            kind: 0,
        });
    }
    Ok(())
}

/// `BasePigmentsOfTokuno` (abstract).
pub static BASE_PIGMENTS_OF_TOKUNO: ItemType = ItemType {
    full_name: "Server.Items.BasePigmentsOfTokuno",
    parent: Some(&ITEM_BASE),
    construct: None,
    default_weight: |_, _| 1.0,
    label_number: |_, _| 1070933, // Pigments of Tokuno
    get_properties: base_get_properties,
    on_double_click,
    serialize: base_serialize,
    deserialize: base_deserialize,
    ..ITEM_BASE
};

fn type_serialize(
    parent: &'static ItemType,
    w: &World,
    id: ItemId,
    out: &mut dyn muo_io::GenericWriter,
) {
    (parent.serialize)(w, id, out);
    out.write_i32(0);
    out.write_enum(4, i64::from(pigment_type(w, id)));
}

fn type_deserialize(
    parent: &'static ItemType,
    w: &mut World,
    id: ItemId,
    r: &mut dyn muo_io::GenericReader,
) -> muo_io::Result<()> {
    (parent.deserialize)(w, id, r)?;
    let _version = r.read_i32()?;
    let kind = r.read_enum(4)? as i32;
    if let Some(d) = data_mut(w, id) {
        d.kind = kind;
    }
    Ok(())
}

/// `[SerializationGenerator(0, false)] PigmentsOfTokuno`.
pub static PIGMENTS_OF_TOKUNO: ItemType = ItemType {
    full_name: "Server.Items.PigmentsOfTokuno",
    parent: Some(&BASE_PIGMENTS_OF_TOKUNO),
    construct: Some(|w| pigments_new(w, 0, None)),
    label_number: |_, _| 1070933, // Pigments of Tokuno
    serialize: |w, id, out| type_serialize(&BASE_PIGMENTS_OF_TOKUNO, w, id, out),
    deserialize: |w, id, r| type_deserialize(&BASE_PIGMENTS_OF_TOKUNO, w, id, r),
    ..BASE_PIGMENTS_OF_TOKUNO
};

/// `[SerializationGenerator(0, false)] LesserPigmentsOfTokuno`.
pub static LESSER_PIGMENTS_OF_TOKUNO: ItemType = ItemType {
    full_name: "Server.Items.LesserPigmentsOfTokuno",
    parent: Some(&BASE_PIGMENTS_OF_TOKUNO),
    // `this((LesserPigmentType)Utility.Random(0, 11))`
    construct: Some(|w| {
        let kind = muo_content::hooks::rnd::from(w, 0, 11);
        lesser_new(w, kind)
    }),
    serialize: |w, id, out| type_serialize(&BASE_PIGMENTS_OF_TOKUNO, w, id, out),
    deserialize: |w, id, r| type_deserialize(&BASE_PIGMENTS_OF_TOKUNO, w, id, r),
    ..BASE_PIGMENTS_OF_TOKUNO
};

/// `[SerializationGenerator(0, false)] MetalPigmentsOfTokuno`.
pub static METAL_PIGMENTS_OF_TOKUNO: ItemType = ItemType {
    full_name: "Server.Items.MetalPigmentsOfTokuno",
    parent: Some(&BASE_PIGMENTS_OF_TOKUNO),
    construct: Some(metal_new),
    serialize: |w, id, out| {
        (BASE_PIGMENTS_OF_TOKUNO.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (BASE_PIGMENTS_OF_TOKUNO.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        Ok(())
    },
    ..BASE_PIGMENTS_OF_TOKUNO
};

const PIGMENT_NAMES: &[(&str, i64)] = &[
    ("None", 0),
    ("ParagonGold", 1),
    ("VioletCouragePurple", 2),
    ("InvulnerabilityBlue", 3),
    ("LunaWhite", 4),
    ("DryadGreen", 5),
    ("ShadowDancerBlack", 6),
    ("BerserkerRed", 7),
    ("NoxGreen", 8),
    ("RumRed", 9),
    ("FireOrange", 10),
    ("FadedCoal", 11),
    ("Coal", 12),
    ("FadedGold", 13),
    ("StormBronze", 14),
    ("Rose", 15),
    ("MidnightCoal", 16),
    ("FadedBronze", 17),
    ("FadedRose", 18),
    ("DeepRose", 19),
];

const LESSER_NAMES: &[(&str, i64)] = &[
    ("None", 0),
    ("PaleOrange", 1),
    ("FreshRose", 2),
    ("ChaosBlue", 3),
    ("Silver", 4),
    ("NobleGold", 5),
    ("LightGreen", 6),
    ("PaleBlue", 7),
    ("FreshPlum", 8),
    ("DeepBrown", 9),
    ("BurntBrown", 10),
];

static TYPE_PARAM: [Param; 1] = [Param {
    name: "type",
    kind: Kind::Plain(PropType::Enum(PIGMENT_NAMES)),
    default: Some(Value::Enum(0)),
    ty: "PigmentType",
}];
static TYPE_USES_PARAMS: [Param; 2] = [
    Param {
        name: "type",
        kind: Kind::Plain(PropType::Enum(PIGMENT_NAMES)),
        default: None,
        ty: "PigmentType",
    },
    Param {
        name: "uses",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        default: None,
        ty: "",
    },
];
static LESSER_PARAMS: [Param; 1] = [Param {
    name: "type",
    kind: Kind::Plain(PropType::Enum(LESSER_NAMES)),
    default: None,
    ty: "LesserPigmentType",
}];

/// `PigmentsOfTokuno(type = None)` and `(type, uses)`, in declaration order.
static PIGMENT_CTORS: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &TYPE_PARAM,
        make: |w, v| EntityId::Item(pigments_new(w, v[0].i32(), None)),
    },
    Ctor {
        access: None,
        params: &TYPE_USES_PARAMS,
        make: |w, v| EntityId::Item(pigments_new(w, v[0].i32(), Some(v[1].i32()))),
    },
];
/// `LesserPigmentsOfTokuno()` and `(type)`.
static LESSER_CTORS: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[],
        make: |w, _| EntityId::Item(lesser_default(w)),
    },
    Ctor {
        access: None,
        params: &LESSER_PARAMS,
        make: |w, v| EntityId::Item(lesser_new(w, v[0].i32())),
    },
];

fn lesser_default(w: &mut World) -> ItemId {
    let kind = muo_content::hooks::rnd::from(w, 0, 11);
    lesser_new(w, kind)
}

pub fn register(types: &mut TypeRegistry) {
    types.register_item(&BASE_PIGMENTS_OF_TOKUNO, &[]);
    types.register_item(&PIGMENTS_OF_TOKUNO, &[]);
    types.register_item(&LESSER_PIGMENTS_OF_TOKUNO, &[]);
    types.register_item(&METAL_PIGMENTS_OF_TOKUNO, &[]);
    types.register_phase(
        Phase::Configure,
        "BasePigmentsOfTokuno.Ctors",
        DEFAULT_CALL_PRIORITY,
        |w| {
            add_ctors(w, "Server.Items.PigmentsOfTokuno", &PIGMENT_CTORS);
            add_ctors(w, "Server.Items.LesserPigmentsOfTokuno", &LESSER_CTORS);
        },
    );
}

#[allow(dead_code)]
fn _unused(_: MobileId) {}
