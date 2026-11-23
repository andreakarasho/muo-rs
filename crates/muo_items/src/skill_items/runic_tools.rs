//! `BaseRunicTool`, `RunicHammer`, `RunicSewingKit`, `RunicDovetailSaw` and `RunicFletcherTool`.
//!
//! `BaseRunicTool`'s static `ApplyAttributesTo` for loot is `muo_content::items::runic`; here are the tool itself
//! (its `Resource`) and the instance overloads `ApplyAttributesTo(BaseWeapon)` / `ApplyAttributesTo(BaseArmor)`
//! that a runic tool's crafting calls through `EquipHooks::runic_apply_attributes`.

use muo_content::craft::{self, CraftResource};
use muo_content::items::equip_hooks;
use muo_content::items::runic;
use muo_content::items::tool::{self, BASE_TOOL, BASE_TOOL_CLASS, ToolClass};
use muo_content::items::weapon_hooks;
use muo_content::props::{Ctor, Kind, Param, Value, add_ctors};
use muo_content::utilities::types::{Numeric, PropType};
use muo_io::{GenericReader, GenericWriter, Result};
use muo_primitives::utility;
use muo_server::World;
use muo_server::ids::{EntityId, ItemId};
use muo_server::item::{ItemType, ops};
use muo_server::layer::Layer;
use muo_server::property_list::PropertyList;
use muo_server::type_registry::{Phase, TypeRegistry};

use super::{read_version, write_version};

/// `BaseRunicTool._resource`.
#[derive(Clone, Copy, Debug)]
struct RunicData {
    resource: CraftResource,
}

/// `Resource`.
pub fn resource(w: &World, id: ItemId) -> CraftResource {
    w.item(id)
        .and_then(|i| i.ext.get::<RunicData>())
        .map_or(CraftResource::None, |d| d.resource)
}

/// `Resource = value`: `Hue = CraftResources.GetHue(value)` and the properties are rebuilt.
pub fn set_resource(w: &mut World, id: ItemId, value: CraftResource) {
    if resource(w, id) == value {
        return;
    }
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(RunicData { resource: value });
    }
    ops::invalidate_properties(w, id);
    let hue = craft::hue(w, value);
    super::hue(w, id, hue);
}

/// `BaseRunicTool.GetIndex`: `CraftResources.GetIndex(Resource)`.
fn index(w: &World, id: ItemId) -> i32 {
    let r = resource(w, id);
    let start = craft::start(r);
    if start == CraftResource::None {
        0
    } else {
        r as i32 - start as i32
    }
}

/// `new RunicX(resource)` / `new RunicX(resource, uses)` around `base(resource[, uses], itemID)`.
fn new_runic(
    w: &mut World,
    ty: &'static ItemType,
    item_id: i32,
    r: CraftResource,
    uses: Option<i32>,
) -> ItemId {
    let id = match uses {
        Some(u) => tool::new_tool(w, ty, item_id, u),
        None => tool::new_tool_random(w, ty, item_id),
    };
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(RunicData { resource: r });
    }
    id
}

/// The base: `[SerializationGenerator(1, false)]` with the resource at version 1 (an int before).
pub static BASE_RUNIC_TOOL: ItemType = ItemType {
    full_name: "Server.Items.BaseRunicTool",
    parent: Some(&BASE_TOOL),
    construct: None,
    serialize: |w, id, out: &mut dyn GenericWriter| {
        (BASE_TOOL.serialize)(w, id, out);
        out.write_i32(1);
        out.write_enum(4, resource(w, id) as i64);
    },
    deserialize: |w, id, r: &mut dyn GenericReader| -> Result<()> {
        (BASE_TOOL.deserialize)(w, id, r)?;
        let version = r.read_i32()?;
        let res = if version < 1 {
            // Deserialize(reader, version): `(CraftResource)reader.ReadInt()`
            CraftResource::from_i32(r.read_i32()?).unwrap_or(CraftResource::None)
        } else {
            muo_content::items::read_enum(r, CraftResource::from_i32, "CraftResource")?
        };
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(RunicData { resource: res });
        }
        Ok(())
    },
    ..BASE_TOOL
};

fn leaf_serialize(w: &World, id: ItemId, out: &mut dyn GenericWriter) {
    (BASE_RUNIC_TOOL.serialize)(w, id, out);
    write_version(out, false);
}

fn leaf_deserialize(w: &mut World, id: ItemId, r: &mut dyn GenericReader) -> Result<()> {
    (BASE_RUNIC_TOOL.deserialize)(w, id, r)?;
    read_version(r, false)?;
    Ok(())
}

/// The name line every runic tool adds after the base name: nothing for a standard resource, else the metal.
fn add_resource_name(w: &World, id: ItemId, list: &mut PropertyList) {
    let r = resource(w, id);
    if !craft::is_standard(r) {
        let num = craft::localization_number(r);
        if num > 0 {
            list.add_localized(num);
        } else {
            list.add_text(craft::name(r));
        }
    }
}

// ---- RunicHammer ----

static HAMMER_CLASS: ToolClass = ToolClass {
    craft_system: "Blacksmithy",
    ..BASE_TOOL_CLASS
};

/// `new RunicHammer(resource[, uses])`.
pub fn new_runic_hammer(w: &mut World, r: CraftResource, uses: Option<i32>) -> ItemId {
    let id = new_runic(w, &RUNIC_HAMMER, 0x13E3, r, uses);
    (RUNIC_HAMMER.set_layer)(w, id, Layer::OneHanded);
    let h = craft::hue(w, r);
    super::hue(w, id, h);
    id
}

/// `Server.Items.RunicHammer`.
pub static RUNIC_HAMMER: ItemType = ItemType {
    full_name: "Server.Items.RunicHammer",
    parent: Some(&BASE_RUNIC_TOOL),
    construct: None,
    family: Some(&HAMMER_CLASS),
    default_weight: crate::class_props::default_weight,
    label_number: |w, id| {
        let i = index(w, id);
        if (1..=8).contains(&i) {
            1049019 + i
        } else {
            1045128
        } // runic smithy hammer
    },
    add_name_properties: |w, id, list| {
        muo_server::item::base::add_name_properties_base(w, id, list);
        if (1..=8).contains(&index(w, id)) {
            return;
        }
        add_resource_name(w, id, list);
    },
    serialize: leaf_serialize,
    deserialize: leaf_deserialize,
    ..BASE_RUNIC_TOOL
};

// ---- RunicSewingKit ----

static SEWING_CLASS: ToolClass = ToolClass {
    craft_system: "Tailoring",
    ..BASE_TOOL_CLASS
};

/// `new RunicSewingKit(resource[, uses])`.
pub fn new_runic_sewing_kit(w: &mut World, r: CraftResource, uses: Option<i32>) -> ItemId {
    let id = new_runic(w, &RUNIC_SEWING_KIT, 0xF9D, r, uses);
    let h = craft::hue(w, r);
    super::hue(w, id, h);
    id
}

/// `~1_LEATHER_TYPE~ runic sewing kit` for the leather of the kit: `(args, is a cliloc number)`.
fn sewing_leather(w: &World, id: ItemId) -> (String, bool) {
    let r = resource(w, id);
    if craft::is_standard(r) {
        return (" ".to_owned(), false);
    }
    let num = craft::localization_number(r);
    if num > 0 {
        (format!("#{num}"), true)
    } else {
        (craft::name(r).to_owned(), false)
    }
}

/// `Server.Items.RunicSewingKit`.
pub static RUNIC_SEWING_KIT: ItemType = ItemType {
    full_name: "Server.Items.RunicSewingKit",
    parent: Some(&BASE_RUNIC_TOOL),
    construct: None,
    family: Some(&SEWING_CLASS),
    default_weight: crate::class_props::default_weight,
    add_name_property: |w, id, list| {
        let r = resource(w, id);
        if craft::is_standard(r) {
            list.add_str(1061119, " "); // ~1_LEATHER_TYPE~ runic sewing kit
            return;
        }
        let num = craft::localization_number(r);
        if num > 0 {
            list.add_localized_number(1061119, num);
        } else {
            list.add_str(1061119, craft::name(r));
        }
    },
    on_single_click: |w, id, from| {
        let (args, _) = sewing_leather(w, id);
        ops::label_to_number(w, id, from, 1061119, &args); // ~1_LEATHER_TYPE~ runic sewing kit
    },
    serialize: leaf_serialize,
    deserialize: leaf_deserialize,
    ..BASE_RUNIC_TOOL
};

// ---- RunicDovetailSaw ----

static DOVETAIL_CLASS: ToolClass = ToolClass {
    craft_system: "Carpentry",
    ..BASE_TOOL_CLASS
};

/// `new RunicDovetailSaw(resource[, uses])`.
pub fn new_runic_dovetail_saw(w: &mut World, r: CraftResource, uses: Option<i32>) -> ItemId {
    let id = new_runic(w, &RUNIC_DOVETAIL_SAW, 0x1028, r, uses);
    let h = craft::hue(w, r);
    super::hue(w, id, h);
    id
}

/// `Server.Items.RunicDovetailSaw`.
pub static RUNIC_DOVETAIL_SAW: ItemType = ItemType {
    full_name: "Server.Items.RunicDovetailSaw",
    parent: Some(&BASE_RUNIC_TOOL),
    construct: None,
    family: Some(&DOVETAIL_CLASS),
    default_weight: crate::class_props::default_weight,
    label_number: |w, id| {
        let i = index(w, id);
        if (1..=6).contains(&i) {
            1072633 + i
        } else {
            1024137
        } // dovetail saw
    },
    serialize: leaf_serialize,
    deserialize: leaf_deserialize,
    ..BASE_RUNIC_TOOL
};

// ---- RunicFletcherTool ----

static FLETCHER_CLASS: ToolClass = ToolClass {
    craft_system: "BowFletching",
    ..BASE_TOOL_CLASS
};

/// `new RunicFletcherTool(resource[, uses])`.
pub fn new_runic_fletcher_tool(w: &mut World, r: CraftResource, uses: Option<i32>) -> ItemId {
    let id = new_runic(w, &RUNIC_FLETCHER_TOOL, 0x1022, r, uses);
    let h = craft::hue(w, r);
    super::hue(w, id, h);
    id
}

/// `Server.Items.RunicFletcherTool`.
pub static RUNIC_FLETCHER_TOOL: ItemType = ItemType {
    full_name: "Server.Items.RunicFletcherTool",
    parent: Some(&BASE_RUNIC_TOOL),
    construct: None,
    family: Some(&FLETCHER_CLASS),
    default_weight: crate::class_props::default_weight,
    label_number: |w, id| {
        let i = index(w, id);
        if (1..=6).contains(&i) {
            1072627 + i
        } else {
            1044559
        } // Fletcher's Tools
    },
    serialize: leaf_serialize,
    deserialize: leaf_deserialize,
    ..BASE_RUNIC_TOOL
};

// ---- ApplyAttributesTo(item) ----

/// `tool.ApplyAttributesTo(weapon)` / `ApplyAttributesTo(armor)`: the attribute count and intensity come from the
/// tool's resource, and the roll is that of a runic tool. Any other item is left alone.
pub fn apply_attributes_to(w: &mut World, tool_id: ItemId, item: ItemId) {
    let r = resource(w, tool_id);
    let ml = muo_content::core::ml(w);
    let Some((min_attr, max_attr, min, max)) = craft::runic_info(r, ml) else {
        return;
    };
    let count = utility::random_min_max(&mut w.rng, min_attr, max_attr);
    if muo_content::factory::item_is(w, item, "Server.Items.BaseWeapon") {
        runic::apply_to_weapon(w, item, true, 0, count, min, max);
    } else if muo_content::factory::item_is(w, item, "Server.Items.BaseArmor") {
        runic::apply_to_armor(w, item, true, 0, count, min, max);
    }
}

const INT: Kind = Kind::Plain(PropType::Numeric(Numeric::Int));
const RESOURCES: Kind = Kind::Plain(PropType::Enum(&[
    ("None", 0),
    ("Iron", 1),
    ("DullCopper", 2),
    ("ShadowIron", 3),
    ("Copper", 4),
    ("Bronze", 5),
    ("Gold", 6),
    ("Agapite", 7),
    ("Verite", 8),
    ("Valorite", 9),
    ("RegularLeather", 101),
    ("SpinedLeather", 102),
    ("HornedLeather", 103),
    ("BarbedLeather", 104),
    ("RedScales", 201),
    ("YellowScales", 202),
    ("BlackScales", 203),
    ("GreenScales", 204),
    ("WhiteScales", 205),
    ("BlueScales", 206),
    ("RegularWood", 301),
    ("OakWood", 302),
    ("AshWood", 303),
    ("YewWood", 304),
    ("Heartwood", 305),
    ("Bloodwood", 306),
    ("Frostwood", 307),
]));

macro_rules! ctors {
    ($($name:ident = $new:ident;)*) => {$(
        static $name: [Ctor; 2] = [
            Ctor {
                access: None,
                params: &[Param { name: "resource", kind: RESOURCES, default: None, ty: "CraftResource" }],
                make: |w, v| EntityId::Item($new(w, to_resource(&v[0]), None)),
            },
            Ctor {
                access: None,
                params: &[
                    Param { name: "resource", kind: RESOURCES, default: None, ty: "CraftResource" },
                    Param { name: "uses", kind: INT, default: None, ty: "" },
                ],
                make: |w, v| EntityId::Item($new(w, to_resource(&v[0]), Some(v[1].i32()))),
            },
        ];
    )*};
}

fn to_resource(v: &Value) -> CraftResource {
    CraftResource::from_i32(v.i32()).unwrap_or(CraftResource::None)
}

ctors! {
    HAMMER_CTORS = new_runic_hammer;
    SEWING_CTORS = new_runic_sewing_kit;
    DOVETAIL_CTORS = new_runic_dovetail_saw;
    FLETCHER_CTORS = new_runic_fletcher_tool;
}

fn configure(w: &mut World) {
    add_ctors(w, "Server.Items.RunicHammer", &HAMMER_CTORS);
    add_ctors(w, "Server.Items.RunicSewingKit", &SEWING_CTORS);
    add_ctors(w, "Server.Items.RunicDovetailSaw", &DOVETAIL_CTORS);
    add_ctors(w, "Server.Items.RunicFletcherTool", &FLETCHER_CTORS);
    equip_hooks::hooks_mut(w).runic_apply_attributes = Some(apply_attributes_to);
    w.resources
        .get_mut::<weapon_hooks::WeaponHooks>()
        .runic_tool_resource = Some(resource);
    // `BaseRunicTool.ApplyAttributesTo(spellbook, true, 0, propertyCount, minIntensity, maxIntensity)`.
    w.resources
        .get_mut::<super::spellbook::SpellbookHooks>()
        .apply_runic =
        Some(|w, id, count, min, max| runic::apply_to_spellbook(w, id, true, 0, count, min, max));
}

/// Registers the runic tools.
pub fn register(types: &mut TypeRegistry) {
    for t in [
        &RUNIC_HAMMER,
        &RUNIC_SEWING_KIT,
        &RUNIC_DOVETAIL_SAW,
        &RUNIC_FLETCHER_TOOL,
    ] {
        types.register_item(t, &[]);
    }
    types.register_phase_default(
        Phase::Configure,
        "Server.Items.BaseRunicTool.Configure",
        configure,
    );
}
