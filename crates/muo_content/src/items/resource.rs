//! The resource bases: ingots, ore, scales, leathers, hides and granite, with their leaves.
//!
//! One state struct ([`ResourceData`]: the `CraftResource`) and one class table per base ([`ResClass`]); each leaf
//! is one row of `resource_leaves!`. Every base writes its version int then the resource as an enum; the older
//! versions read the resource as a plain int (`BaseOre` as a byte: "use this line instead if you are getting
//! world loading issues"). Version 0/1 saves therefore load, never refused.
//!
//! `BaseOre.OnDoubleClick` (smelting at a forge, combining piles) and `GetIngot()` are `ore_*` below.
//!
//! `ICommodity` is `muo_items::commodity`, `[Flippable]` is `flip_table`, and Items/Resources/{Arrows, Fishing,
//! Reagents} and Tailor's cloth are `muo_items::resources`. `IronOre(bool fixedSize)` is not `[Constructible]` and
//! nothing calls it, so it does not exist here.

use muo_io::{GenericReader, IoError, Result};
use muo_primitives::utility;
use muo_server::World;
use muo_server::ext::Ext;
use muo_server::ids::ItemId;
use muo_server::item::{ITEM_BASE, ItemType, base, ops};
use muo_server::mobile::AccessLevel;
use muo_server::type_registry::TypeRegistry;

use super::read_enum;
use crate::core;
use crate::craft::{self, CraftResource};

/// What every resource base adds.
#[derive(Clone, Copy, Debug)]
pub struct ResourceData {
    pub resource: CraftResource,
}

/// How a base picks `Stackable`.
#[derive(Copy, Clone)]
pub enum Stack {
    Yes,
    /// `Stackable = Core.ML`.
    Ml,
}

/// The constants of a resource base.
#[derive(Copy, Clone)]
pub struct ResClass {
    /// The graphic the constructor passes to `base(..)` (ore picks one at random).
    pub graphic: fn(&mut World) -> i32,
    pub weight: fn(&World) -> f64,
    /// `LabelNumber` for a resource.
    pub label: fn(CraftResource) -> i32,
    /// `AddNameProperty`'s "~1_NUMBER~ ~2_ITEMNAME~" item name (0: not overridden).
    pub name_number: i32,
    /// `GetProperties` adds the resource name when it is not standard.
    pub resource_property: bool,
    pub stack: Stack,
    /// `SerializationVersion`.
    pub version: i32,
    /// The legacy `Deserialize(reader, version)` reads the resource as a byte instead of an int.
    pub legacy_byte: bool,
    /// The oldest version whose legacy read has a resource to read (below it the resource is `RegularWood`: `Log`).
    pub legacy_from: i32,
}

fn data(w: &World, id: ItemId) -> Option<&ResourceData> {
    w.item(id)?.ext.get::<ResourceData>()
}

fn class(w: &World, id: ItemId) -> &'static ResClass {
    w.item(id)
        .and_then(|i| i.ty.family::<ResClass>())
        .unwrap_or(&INGOT_CLASS)
}

/// `Resource`.
pub fn resource(w: &World, id: ItemId) -> CraftResource {
    data(w, id).map_or(CraftResource::None, |d| d.resource)
}

/// `Resource = value` (`OnResourceChanged` sets the hue; `[InvalidateProperties]`).
pub fn set_resource(w: &mut World, id: ItemId, value: CraftResource) {
    let changed = data(w, id).is_some_and(|d| d.resource != value);
    if let Some(d) = w.item_mut(id).and_then(|i| i.ext.get_mut::<ResourceData>()) {
        d.resource = value;
    }
    if changed {
        let hue = craft::hue(w, value);
        if let Some(ty) = w.item(id).map(|i| i.ty) {
            (ty.set_hue)(w, id, hue);
        }
        ops::invalidate_properties(w, id);
    }
}

/// `new BaseIngot/BaseLeather/.. (resource, amount)` of class `ty`.
pub fn new_resource(
    w: &mut World,
    ty: &'static ItemType,
    resource: CraftResource,
    amount: i32,
) -> ItemId {
    let c = ty.family::<ResClass>().unwrap_or(&INGOT_CLASS);
    let graphic = (c.graphic)(w);
    let id = ops::new_item_with_id(w, ty, graphic);
    match c.stack {
        Stack::Yes => ops::set_stackable(w, id, true),
        Stack::Ml => {
            let ml = core::ml(w);
            ops::set_stackable(w, id, ml);
        }
    }
    ops::set_amount(w, id, amount);
    let hue = craft::hue(w, resource);
    (ty.set_hue)(w, id, hue);
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(ResourceData { resource });
    }
    id
}

fn add_name_property(w: &World, id: ItemId, list: &mut muo_server::property_list::PropertyList) {
    let c = class(w, id);
    if c.name_number == 0 {
        return base::add_name_property_base(w, id, list);
    }
    let amount = w.item(id).map_or(1, |i| i.amount);
    if amount > 1 {
        // ~1_NUMBER~ ~2_ITEMNAME~
        list.add_str(1050039, &format!("{amount}\t#{}", c.name_number));
    } else {
        list.add(c.name_number);
    }
}

fn get_properties(w: &World, id: ItemId, list: &mut muo_server::property_list::PropertyList) {
    base::get_properties_base(w, id, list);
    if !class(w, id).resource_property {
        return;
    }
    let r = resource(w, id);
    if !craft::is_standard(r) {
        let num = craft::localization_number(r);
        if num > 0 {
            list.add(num);
        } else {
            list.add_text(craft::name(r));
        }
    }
}

fn serialize_base(w: &World, id: ItemId, out: &mut dyn muo_io::GenericWriter) {
    base::serialize_base(w, id, out);
    out.write_i32(class(w, id).version);
    out.write_enum(4, resource(w, id) as i64);
}

fn deserialize_base(w: &mut World, id: ItemId, r: &mut dyn GenericReader) -> Result<()> {
    base::deserialize_base(w, id, r)?;
    let c = class(w, id);
    let version = r.read_i32()?;
    let resource = if version < c.version && version < c.legacy_from {
        CraftResource::RegularWood
    } else if version < c.version {
        // Deserialize(reader, version)
        let v = if c.legacy_byte {
            i32::from(r.read_u8()?)
        } else {
            r.read_i32()?
        };
        CraftResource::from_i32(v)
            .ok_or_else(|| IoError::InvalidData(format!("CraftResource: no value {v}")))?
    } else {
        read_enum(r, CraftResource::from_i32, "CraftResource")?
    };
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(ResourceData { resource });
    }
    Ok(())
}

fn range_label(
    r: CraftResource,
    lo: CraftResource,
    hi: CraftResource,
    first: i32,
    other: i32,
) -> i32 {
    if (lo as i32..=hi as i32).contains(&(r as i32)) {
        first + (r as i32 - lo as i32)
    } else {
        other
    }
}

// ---- the bases ----

pub static INGOT_CLASS: ResClass = ResClass {
    graphic: |_| 0x1BF2,
    weight: |_| 0.1,
    label: |r| {
        range_label(
            r,
            CraftResource::DullCopper,
            CraftResource::Valorite,
            1042684,
            1042692,
        )
    },
    name_number: 1027154, // ingots
    resource_property: true,
    stack: Stack::Yes,
    version: 2,
    legacy_byte: false,
    legacy_from: 0,
};

pub static LEATHER_CLASS: ResClass = ResClass {
    graphic: |_| 0x1081,
    weight: |_| 1.0,
    label: |r| {
        range_label(
            r,
            CraftResource::SpinedLeather,
            CraftResource::BarbedLeather,
            1049684,
            1047022,
        )
    },
    name_number: 1024199, // cut leather
    ..INGOT_CLASS
};

pub static HIDES_CLASS: ResClass = ResClass {
    graphic: |_| 0x1079,
    weight: |_| 5.0,
    label: |r| {
        range_label(
            r,
            CraftResource::SpinedLeather,
            CraftResource::BarbedLeather,
            1049687,
            1047023,
        )
    },
    name_number: 1024216, // pile of hides
    ..INGOT_CLASS
};

pub static SCALES_CLASS: ResClass = ResClass {
    graphic: |_| 0x26B4,
    weight: |_| 0.1,
    label: |_| 1053139, // dragon scales
    name_number: 0,
    resource_property: false,
    version: 1,
    ..INGOT_CLASS
};

pub static GRANITE_CLASS: ResClass = ResClass {
    graphic: |_| 0x1779,
    weight: |w| if core::ml(w) { 1.0 } else { 10.0 },
    label: |_| 1044607, // high quality granite
    name_number: 0,
    stack: Stack::Ml,
    ..INGOT_CLASS
};

/// `Log`: version 2 (version 1 wrote the resource as an int).
pub static LOG_CLASS: ResClass = ResClass {
    graphic: |_| 0x1BDD,
    weight: |_| 2.0,
    label: |_| 0,
    name_number: 0,
    legacy_from: 1,
    ..INGOT_CLASS
};

/// `Board`: version 4; every older version wrote an int.
pub static BOARD_CLASS: ResClass = ResClass {
    graphic: |_| 0x1BD7,
    weight: |_| -1.0, // the tiledata weight
    label: |_| 0,
    name_number: 0,
    version: 4,
    ..INGOT_CLASS
};

fn random_ore_size(w: &mut World) -> i32 {
    let d = utility::random_double(&mut w.rng);
    if d < 0.125 {
        0x19B7 // Small
    } else if d < 0.1875 {
        0x19B8 // Medium clump
    } else if d < 0.25 {
        0x19BA // Medium
    } else {
        0x19B9 // Large
    }
}

pub static ORE_CLASS: ResClass = ResClass {
    graphic: random_ore_size,
    // BaseOre does not override DefaultWeight: the tiledata weight of the chosen graphic stands (see below).
    weight: |_| -1.0,
    label: |r| {
        range_label(
            r,
            CraftResource::DullCopper,
            CraftResource::Valorite,
            1042845,
            1042853,
        )
    },
    name_number: 1026583, // ore
    legacy_byte: true,
    ..INGOT_CLASS
};

macro_rules! resource_base {
    ($(#[$m:meta])* $name:ident = $full:literal, $class:ident $(, $f:ident : $v:expr)*) => {
        $(#[$m])*
        pub static $name: ItemType = ItemType {
            full_name: $full,
            parent: Some(&ITEM_BASE),
            family: Some(&$class),
            default_weight: |w, id| {
                let x = (class(w, id).weight)(w);
                if x < 0.0 { base::default_weight_base(w, id) } else { x }
            },
            label_number: |w, id| match (class(w, id).label)(resource(w, id)) {
                0 => base::label_number_base(w, id),
                n => n,
            },
            add_name_property,
            get_properties,
            serialize: serialize_base,
            deserialize: deserialize_base,
            $($f: $v,)*
            ..ITEM_BASE
        };
    };
}

resource_base!(
    /// `Server.Items.BaseIngot`.
    BASE_INGOT = "Server.Items.BaseIngot", INGOT_CLASS, construct: None
);
resource_base!(
    /// `Server.Items.BaseLeather`.
    BASE_LEATHER = "Server.Items.BaseLeather", LEATHER_CLASS, construct: None
);
resource_base!(
    /// `Server.Items.BaseHides`.
    BASE_HIDES = "Server.Items.BaseHides", HIDES_CLASS, construct: None
);
resource_base!(
    /// `Server.Items.BaseScales`.
    BASE_SCALES = "Server.Items.BaseScales", SCALES_CLASS, construct: None
);
resource_base!(
    /// `Server.Items.BaseGranite`.
    BASE_GRANITE = "Server.Items.BaseGranite", GRANITE_CLASS, construct: None
);
resource_base!(
    /// `Server.Items.BaseOre`.
    BASE_ORE = "Server.Items.BaseOre", ORE_CLASS, construct: None,
    on_double_click: ore_on_double_click,
    can_stack_with: |w, id, dropped| {
        base::can_stack_with_base(w, id, dropped)
            && w.item(dropped).is_some_and(|d| d.ty.is(&BASE_ORE))
            && resource(w, dropped) == resource(w, id)
    }
);

resource_base!(
    /// `Server.Items.Log`.
    LOG = "Server.Items.Log", LOG_CLASS,
    construct: Some(|w| new_resource(w, &LOG, CraftResource::RegularWood, 1))
);
resource_base!(
    /// `Server.Items.Board`.
    BOARD = "Server.Items.Board", BOARD_CLASS,
    construct: Some(|w| new_resource(w, &BOARD, CraftResource::RegularWood, 1))
);

macro_rules! resource_leaves {
    ($parent:ident, amount: $amount:tt; $($name:ident = $full:literal, $res:ident;)+) => {
        $(
            pub static $name: ItemType = ItemType {
                full_name: $full,
                parent: Some(&$parent),
                construct: Some(|w| new_resource(w, &$name, CraftResource::$res, resource_leaves!(@amount $amount))),
                serialize: |w, id, out| {
                    ($parent.serialize)(w, id, out);
                    out.write_i32(0);
                },
                deserialize: |w, id, r| {
                    ($parent.deserialize)(w, id, r)?;
                    let _version = r.read_i32()?;
                    Ok(())
                },
                ..$parent
            };
        )+
    };
    (@amount one) => { 1 };
    (@amount default) => { 1 };
}

resource_leaves! { BASE_INGOT, amount: default;
    IRON_INGOT = "Server.Items.IronIngot", Iron;
    DULL_COPPER_INGOT = "Server.Items.DullCopperIngot", DullCopper;
    SHADOW_IRON_INGOT = "Server.Items.ShadowIronIngot", ShadowIron;
    COPPER_INGOT = "Server.Items.CopperIngot", Copper;
    BRONZE_INGOT = "Server.Items.BronzeIngot", Bronze;
    GOLD_INGOT = "Server.Items.GoldIngot", Gold;
    AGAPITE_INGOT = "Server.Items.AgapiteIngot", Agapite;
    VERITE_INGOT = "Server.Items.VeriteIngot", Verite;
    VALORITE_INGOT = "Server.Items.ValoriteIngot", Valorite;
}

resource_leaves! { BASE_ORE, amount: default;
    IRON_ORE = "Server.Items.IronOre", Iron;
    DULL_COPPER_ORE = "Server.Items.DullCopperOre", DullCopper;
    SHADOW_IRON_ORE = "Server.Items.ShadowIronOre", ShadowIron;
    COPPER_ORE = "Server.Items.CopperOre", Copper;
    BRONZE_ORE = "Server.Items.BronzeOre", Bronze;
    GOLD_ORE = "Server.Items.GoldOre", Gold;
    AGAPITE_ORE = "Server.Items.AgapiteOre", Agapite;
    VERITE_ORE = "Server.Items.VeriteOre", Verite;
    VALORITE_ORE = "Server.Items.ValoriteOre", Valorite;
}

resource_leaves! { LOG, amount: default;
    HEARTWOOD_LOG = "Server.Items.HeartwoodLog", Heartwood;
    BLOODWOOD_LOG = "Server.Items.BloodwoodLog", Bloodwood;
    FROSTWOOD_LOG = "Server.Items.FrostwoodLog", Frostwood;
    OAK_LOG = "Server.Items.OakLog", OakWood;
    ASH_LOG = "Server.Items.AshLog", AshWood;
    YEW_LOG = "Server.Items.YewLog", YewWood;
}

resource_leaves! { BOARD, amount: default;
    HEARTWOOD_BOARD = "Server.Items.HeartwoodBoard", Heartwood;
    BLOODWOOD_BOARD = "Server.Items.BloodwoodBoard", Bloodwood;
    FROSTWOOD_BOARD = "Server.Items.FrostwoodBoard", Frostwood;
    OAK_BOARD = "Server.Items.OakBoard", OakWood;
    ASH_BOARD = "Server.Items.AshBoard", AshWood;
    YEW_BOARD = "Server.Items.YewBoard", YewWood;
}

resource_leaves! { BASE_LEATHER, amount: default;
    LEATHER = "Server.Items.Leather", RegularLeather;
    SPINED_LEATHER = "Server.Items.SpinedLeather", SpinedLeather;
    HORNED_LEATHER = "Server.Items.HornedLeather", HornedLeather;
    BARBED_LEATHER = "Server.Items.BarbedLeather", BarbedLeather;
}

resource_leaves! { BASE_HIDES, amount: default;
    HIDES = "Server.Items.Hides", RegularLeather;
    SPINED_HIDES = "Server.Items.SpinedHides", SpinedLeather;
    HORNED_HIDES = "Server.Items.HornedHides", HornedLeather;
    BARBED_HIDES = "Server.Items.BarbedHides", BarbedLeather;
}

resource_leaves! { BASE_SCALES, amount: default;
    RED_SCALES = "Server.Items.RedScales", RedScales;
    YELLOW_SCALES = "Server.Items.YellowScales", YellowScales;
    BLACK_SCALES = "Server.Items.BlackScales", BlackScales;
    GREEN_SCALES = "Server.Items.GreenScales", GreenScales;
    WHITE_SCALES = "Server.Items.WhiteScales", WhiteScales;
    BLUE_SCALES = "Server.Items.BlueScales", BlueScales;
}

resource_leaves! { BASE_GRANITE, amount: one;
    GRANITE = "Server.Items.Granite", Iron;
    DULL_COPPER_GRANITE = "Server.Items.DullCopperGranite", DullCopper;
    SHADOW_IRON_GRANITE = "Server.Items.ShadowIronGranite", ShadowIron;
    COPPER_GRANITE = "Server.Items.CopperGranite", Copper;
    BRONZE_GRANITE = "Server.Items.BronzeGranite", Bronze;
    GOLD_GRANITE = "Server.Items.GoldGranite", Gold;
    AGAPITE_GRANITE = "Server.Items.AgapiteGranite", Agapite;
    VERITE_GRANITE = "Server.Items.VeriteGranite", Verite;
    VALORITE_GRANITE = "Server.Items.ValoriteGranite", Valorite;
}

/// `Resource` of the resource classes (`BaseIngot`, `BaseOre`, `BaseScales`, `BaseHides`, `BaseLeather`,
/// `BaseGranite`, `Log`, `Board`): the setter is `OnResourceChanged`, the hue and the invalidation.
static RESOURCE_PROPS: &[crate::props::PropDef] = &[crate::props::PropDef {
    name: "Resource",
    kind: crate::props::Kind::Plain(crate::utilities::types::PropType::Enum(
        super::prop_tables::RESOURCES,
    )),
    read: AccessLevel::GameMaster,
    write: AccessLevel::GameMaster,
    get: |w, e| match e {
        muo_server::ids::EntityId::Item(i) if data(w, i).is_some() => {
            crate::props::Value::Enum(resource(w, i) as i64)
        }
        _ => crate::props::Value::Null,
    },
    set: Some(|w, e, v| {
        if let (muo_server::ids::EntityId::Item(i), Some(r)) = (e, CraftResource::from_i32(v.i32()))
        {
            set_resource(w, i, r);
        }
        Ok(())
    }),
}];

fn configure_props(w: &mut World) {
    for class in [
        "Server.Items.BaseIngot",
        "Server.Items.BaseOre",
        "Server.Items.BaseScales",
        "Server.Items.BaseHides",
        "Server.Items.BaseLeather",
        "Server.Items.BaseGranite",
        "Server.Items.Log",
        "Server.Items.Board",
    ] {
        crate::props::add(w, class, RESOURCE_PROPS);
    }
}

pub fn register(types: &mut TypeRegistry) {
    for ty in [
        &IRON_INGOT,
        &DULL_COPPER_INGOT,
        &SHADOW_IRON_INGOT,
        &COPPER_INGOT,
        &BRONZE_INGOT,
        &GOLD_INGOT,
        &AGAPITE_INGOT,
        &VERITE_INGOT,
        &VALORITE_INGOT,
        &IRON_ORE,
        &DULL_COPPER_ORE,
        &SHADOW_IRON_ORE,
        &COPPER_ORE,
        &BRONZE_ORE,
        &GOLD_ORE,
        &AGAPITE_ORE,
        &VERITE_ORE,
        &VALORITE_ORE,
        &LEATHER,
        &SPINED_LEATHER,
        &HORNED_LEATHER,
        &BARBED_LEATHER,
        &HIDES,
        &SPINED_HIDES,
        &HORNED_HIDES,
        &BARBED_HIDES,
        &RED_SCALES,
        &YELLOW_SCALES,
        &BLACK_SCALES,
        &GREEN_SCALES,
        &WHITE_SCALES,
        &BLUE_SCALES,
        &GRANITE,
        &DULL_COPPER_GRANITE,
        &SHADOW_IRON_GRANITE,
        &COPPER_GRANITE,
        &BRONZE_GRANITE,
        &GOLD_GRANITE,
        &AGAPITE_GRANITE,
        &VERITE_GRANITE,
        &VALORITE_GRANITE,
        &LOG,
        &HEARTWOOD_LOG,
        &BLOODWOOD_LOG,
        &FROSTWOOD_LOG,
        &OAK_LOG,
        &ASH_LOG,
        &YEW_LOG,
        &BOARD,
        &HEARTWOOD_BOARD,
        &BLOODWOOD_BOARD,
        &FROSTWOOD_BOARD,
        &OAK_BOARD,
        &ASH_BOARD,
        &YEW_BOARD,
    ] {
        types.register_item(ty, &[]);
    }
    types.register_phase_default(
        muo_server::type_registry::Phase::Configure,
        "Server.Items.BaseResource.Properties",
        configure_props,
    );
}

// ---- BaseOre.OnDoubleClick ----

/// `GetIngot()` of an ore leaf: a new ingot of the ore's own metal.
pub fn ore_get_ingot(w: &mut World, ore: ItemId) -> Option<ItemId> {
    let ingot = *craft::resource_types(resource(w, ore)).first()?;
    crate::factory::make_item(w, ingot)
}

fn ore_double_click_target(w: &mut World, ore: ItemId, from: muo_server::ids::MobileId) {
    use muo_server::ids::EntityId;
    use muo_server::item::deps;
    use muo_server::target::{self, Target, TargetFlags, Targeted};
    let mut t = Target::new(w, 2, false, TargetFlags::NONE);
    t.handlers.on_target = Some(std::rc::Rc::new(
        move |w: &mut World, from, targeted: Targeted| {
            if w.item(ore).is_none_or(|i| i.deleted()) {
                return;
            }
            let loc = ops::get_world_location(w, ore);
            if !crate::mx::in_range_point(w, from, loc, 2) {
                deps::send_localized_message(w, from, 501976); // The ore is too far away.
                return;
            }
            if let Targeted::Entity(EntityId::Item(other)) = targeted
                && w.item(other).is_some_and(|i| i.ty.is(&BASE_ORE))
            {
                ore_on_target_ore(w, ore, from, other);
                return;
            }
            if ore_is_forge(w, targeted) {
                ore_smelt(w, ore, from, targeted);
            }
        },
    ));
    let id = target::add_target(w, t);
    target::set_mobile_target(w, from, Some(id));
}

/// `InternalTarget.IsForge(obj)`.
fn ore_is_forge(w: &World, targeted: muo_server::target::Targeted) -> bool {
    use muo_server::ids::EntityId;
    use muo_server::target::Targeted;
    if let Targeted::Entity(EntityId::Mobile(m)) = targeted
        && core::ml(w)
        && w.mobile(m).is_some_and(|x| (x.ty.is_dead_bonded_pet)(w, m))
    {
        return false;
    }
    let class_is_forge = match targeted {
        Targeted::Entity(EntityId::Item(i)) => w.item(i).is_some_and(|x| {
            craft::FORGE_CLASSES
                .iter()
                .any(|n| super::is_named(x.ty, n))
        }),
        Targeted::Entity(EntityId::Mobile(m)) => w
            .mobile(m)
            .is_some_and(|x| craft::FORGE_CLASSES.contains(&x.ty.full_name)),
        _ => false,
    };
    if class_is_forge {
        return true;
    }
    let item_id = match targeted {
        Targeted::Entity(EntityId::Item(i)) => w.item(i).map_or(0, |x| x.item_id),
        Targeted::Static(s) => s.item_id,
        _ => 0,
    };
    item_id == 4017 || (6522..=6569).contains(&item_id) || item_id == 11736
}

fn ore_smelt(
    w: &mut World,
    ore: ItemId,
    from: muo_server::ids::MobileId,
    targeted: muo_server::target::Targeted,
) {
    use muo_server::item::deps;
    use muo_server::skills::{self, SkillName};
    let difficulty = match resource(w, ore) {
        CraftResource::DullCopper => 65.0,
        CraftResource::ShadowIron => 70.0,
        CraftResource::Copper => 75.0,
        CraftResource::Bronze => 80.0,
        CraftResource::Gold => 85.0,
        CraftResource::Agapite => 90.0,
        CraftResource::Verite => 95.0,
        CraftResource::Valorite => 99.0,
        _ => 50.0,
    };
    let (min_skill, max_skill) = (difficulty - 25.0, difficulty + 25.0);
    if difficulty > 50.0 && difficulty > skills::value(w, from, SkillName::Mining) {
        deps::send_localized_message(w, from, 501986); // You have no idea how to smelt this strange ore!
        return;
    }
    let (graphic, amount) = w.item(ore).map_or((0, 0), |i| (i.item_id, i.amount));
    if graphic == 0x19B7 && amount < 2 {
        // There is not enough metal-bearing ore in this pile to make an ingot.
        deps::send_localized_message(w, from, 501987);
        return;
    }
    let ty = w.item(ore).map(|i| i.ty);
    if crate::systems::skills::common::check_target_skill(
        w,
        from,
        SkillName::Mining,
        targeted,
        min_skill,
        max_skill,
    ) {
        let mut to_consume = amount;
        if to_consume <= 0 {
            deps::send_localized_message(w, from, 501987);
            return;
        }
        if to_consume > 30000 {
            to_consume = 30000;
        }
        let ingot_amount;
        if graphic == 0x19B7 {
            ingot_amount = to_consume / 2;
            if to_consume % 2 != 0 {
                to_consume -= 1;
            }
        } else if graphic == 0x19B9 {
            ingot_amount = to_consume * 2;
        } else {
            ingot_amount = to_consume;
        }
        let Some(ingot) = ore_get_ingot(w, ore) else {
            return;
        };
        ops::set_amount(w, ingot, ingot_amount);
        base::consume(w, ore, to_consume);
        muo_server::mobile::part2::items::add_to_backpack(w, from, ingot);
        // You smelt the ore removing the impurities and put the metal in your backpack.
        deps::send_localized_message(w, from, 501988);
    } else {
        if amount < 2 {
            let g = if graphic == 0x19B9 { 0x19B8 } else { 0x19B7 };
            if let Some(ty) = ty {
                (ty.set_item_id)(w, ore, g);
            }
        } else {
            ops::set_amount(w, ore, amount / 2);
        }
        // You burn away the impurities but are left with less useable metal.
        deps::send_localized_message(w, from, 501990);
    }
}

/// `OnTargetOre(from, ore)`: combining two piles.
fn ore_on_target_ore(w: &mut World, source: ItemId, from: muo_server::ids::MobileId, ore: ItemId) {
    use muo_server::ids::EntityId;
    use muo_server::item::deps;
    if !w.item(ore).is_some_and(|i| i.movable()) {
        return;
    }
    if source == ore {
        deps::send_localized_message(w, from, 501972); // Select another pile or ore with which to combine this.
        ore_double_click_target(w, ore, from);
        return;
    }
    if resource(w, ore) != resource(w, source) {
        deps::send_localized_message(w, from, 501979); // You cannot combine ores of different metals.
        return;
    }
    let size_worth = |graphic: i32| match graphic {
        0x19B9 => 8,
        0x19B7 => 2,
        _ => 4,
    };
    let (ore_id, ore_amount) = w.item(ore).map_or((0, 0), |i| (i.item_id, i.amount));
    let (src_id, src_amount) = w.item(source).map_or((0, 0), |i| (i.item_id, i.amount));
    let mut worth = ore_amount * size_worth(ore_id);
    worth += src_amount * size_worth(src_id);

    let mut plus_weight = 0;
    let mut new_id = ore_id;
    let (Some(ore_ty), Some(src_ty)) = (w.item(ore).map(|i| i.ty), w.item(source).map(|i| i.ty))
    else {
        return;
    };
    if (ore_ty.default_weight)(w, ore) != (src_ty.default_weight)(w, source) {
        if ore_id == 0x19B7 || src_id == 0x19B7 {
            new_id = 0x19B7;
        } else if ore_id == 0x19B9 {
            new_id = src_id;
            plus_weight = ore_amount * 2;
        } else {
            plus_weight = src_amount * 2;
        }
    }

    if (ore_id == 0x19B9 && worth > 120000)
        || ((ore_id == 0x19B8 || ore_id == 0x19BA) && worth > 60000)
        || (ore_id == 0x19B7 && worth > 30000)
    {
        deps::send_localized_message(w, from, 1062844); // There is too much ore to combine.
        return;
    }

    if let Some(EntityId::Mobile(m)) = ops::root_parent(w, ore)
        && let Some(pack) = muo_server::mobile::part1::props_b::backpack(w, m)
    {
        let total = w
            .item(pack)
            .and_then(|i| i.ext.get::<muo_server::item::container::ContainerState>())
            .map_or(0, |c| c.total_weight);
        if plus_weight + total > muo_server::item::container::max_weight(w, pack) {
            deps::send_localized_message(w, from, 501978); // The weight is too great to combine in a container.
            return;
        }
    }

    (ore_ty.set_item_id)(w, ore, new_id);
    let new_amount = match new_id {
        0x19B9 => worth / 8,
        0x19B7 => worth / 2,
        _ => worth / 4,
    };
    ops::set_amount(w, ore, new_amount);
    (src_ty.delete)(w, source);
}

fn ore_on_double_click(w: &mut World, id: ItemId, from: muo_server::ids::MobileId) {
    use muo_server::ids::EntityId;
    use muo_server::item::deps;
    if !w.item(id).is_some_and(|i| i.movable()) {
        return;
    }
    let in_creature = matches!(ops::root_parent(w, id), Some(EntityId::Mobile(m))
        if w.mobile(m).is_some_and(|x| x.ty.is(&crate::creature::vtable::BASE_CREATURE)));
    if in_creature {
        deps::send_localized_message(w, from, 500447); // That is not accessible
    } else if crate::mx::in_range_point(w, from, ops::get_world_location(w, id), 2) {
        // Select the forge on which to smelt the ore, or another pile of ore with which to combine it.
        deps::send_localized_message(w, from, 501971);
        ore_double_click_target(w, id, from);
    } else {
        deps::send_localized_message(w, from, 501976); // The ore is too far away.
    }
}
