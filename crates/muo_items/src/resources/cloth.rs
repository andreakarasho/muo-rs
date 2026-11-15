//! The tailoring resources (`Cloth`, `BoltOfCloth`, `UncutCloth`, `Cotton`, `Flax`, `Wool`, `Bone`, the yarns and
//! threads), `Bolt` and the fish (`Fish`, `BigFish`, `MagicFish`).
//!
//! `IDyable` / `IScissorable` / `ICarvable` are `muo_content::items::interfaces` entries (the dye tub, the scissors and
//! the knives look the item up there); `ICommodity` is [`crate::commodity`]. The spinning wheel and the loom are
//! `addons::devices`.

use muo_content::items::addon::component_addon;
use muo_content::items::clothing::tub_dyed_hue;
use muo_content::items::interfaces::{register_carvable, register_dyable, register_scissorable};
use muo_content::props::{Ctor, Kind, Param, Value, add_ctors};
use muo_content::utilities::types::{Numeric, PropType};
use muo_primitives::{TimeSpan, utility};
use muo_server::World;
use muo_server::ext::Ext;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, deps, ops};
use muo_server::mobile::StatType;
use muo_server::packets::message::MessageType;
use muo_server::property_list::PropertyList;
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::target::{self, TargetFlags, Targeted};
use muo_server::type_registry::{Phase, TypeRegistry};

use crate::addons::devices::{
    is_loom, is_spinning_wheel, loom_phase, loom_set_phase, spinning_wheel_begin_spin,
    spinning_wheel_spinning,
};
use crate::commodity;
use crate::simple::{hue, random_from, random_min_max};

fn make(w: &mut World, name: &str) -> Option<ItemId> {
    muo_content::factory::make_item(w, name)
}

fn make_amount(w: &mut World, name: &str, amount: i32) -> Option<ItemId> {
    muo_content::factory::make_item_amount(w, name, amount)
}

fn in_pack(w: &mut World, id: ItemId, from: MobileId) -> bool {
    let pack = muo_server::mobile::part1::props_b::backpack(w, from);
    ops::is_child_of(w, id, pack.map(EntityId::Item))
}

fn consume(w: &mut World, id: ItemId, n: i32) {
    if let Some(ty) = w.item(id).map(|i| i.ty) {
        (ty.consume)(w, id, n);
    }
}

fn new_stack(w: &mut World, ty: &'static ItemType, gid: i32, amount: i32) -> ItemId {
    let id = ops::new_item_with_id(w, ty, gid);
    ops::set_stackable(w, id, true);
    ops::set_amount(w, id, amount);
    id
}

/// `Dye(from, sender)` of the plain dyables.
fn dye_plain(w: &mut World, id: ItemId, _from: MobileId, tub: ItemId) -> bool {
    if ops::is_deleted(w, id) {
        return false;
    }
    let h = tub_dyed_hue(w, tub);
    hue(w, id, h);
    true
}

const INT: Kind = Kind::Plain(PropType::Numeric(Numeric::Int));
static AMOUNT_1: [Param; 1] = [Param {
    name: "amount",
    kind: INT,
    default: Some(Value::Int(1)),
    ty: "",
}];

/// A stackable `Item` with an `amount = 1` constructor, `ICommodity` optional, in the shape of most of the folder.
macro_rules! stackables {
    ($($name:ident, $ctor:ident = $full:literal, $gid:expr, weight $weight:expr $(, $f:ident : $v:expr)*;)*) => {$(
        #[doc = concat!("`", $full, "`.")]
        pub static $name: ItemType = ItemType {
            full_name: $full,
            parent: Some(&ITEM_BASE),
            construct: Some(|w| new_stack(w, &$name, $gid, 1)),
            default_weight: |_, _| $weight,
            $($f: $v,)*
            serialize: |w, id, out| {
                (ITEM_BASE.serialize)(w, id, out);
                out.write_i32(0);
            },
            deserialize: |w, id, r| {
                (ITEM_BASE.deserialize)(w, id, r)?;
                let _version = r.read_i32()?;
                Ok(())
            },
            ..ITEM_BASE
        };
        static $ctor: [Ctor; 1] = [Ctor {
            access: None,
            params: &AMOUNT_1,
            make: |w, v| EntityId::Item(new_stack(w, &$name, $gid, v[0].i32())),
        }];
    )*};
}

// ---- Cloth, UncutCloth, BoltOfCloth, Bone, Bolt ----

fn cloth_single_click(
    w: &mut World,
    id: ItemId,
    from: MobileId,
    one: i32,
    many: i32,
    kind: MessageType,
    amount_scale: i32,
) {
    let Some(i) = w.item(id) else { return };
    let number = if i.amount == 1 { one } else { many };
    let (serial, graphic, amount) = (i.serial, i.item_id, i.amount);
    let Some(ns) = deps::net_state_of(w, from) else {
        return;
    };
    muo_server::packets::message::send_message_localized(
        w,
        ns,
        serial,
        graphic,
        kind,
        0x3B2,
        3,
        number,
        "",
        &(amount * amount_scale).to_string(),
    );
}

stackables! {
    CLOTH, C_CLOTH = "Server.Items.Cloth", 0x1766, weight 0.1,
        on_single_click: |w, id, from| cloth_single_click(w, id, from, 1049124, 1049123, MessageType::REGULAR, 1);
    UNCUT_CLOTH, C_UNCUT_CLOTH = "Server.Items.UncutCloth", 0x1767, weight 0.1,
        on_single_click: |w, id, from| cloth_single_click(w, id, from, 1049124, 1049123, MessageType::REGULAR, 1);
    BOLT_OF_CLOTH, C_BOLT_OF_CLOTH = "Server.Items.BoltOfCloth", 0xF95, weight 5.0,
        on_single_click: |w, id, from| cloth_single_click(w, id, from, 1049122, 1049121, MessageType::LABEL, 50);
    BONE, C_BONE = "Server.Items.Bone", 0xF7E, weight 1.0;
    BOLT, C_BOLT = "Server.Items.Bolt", 0x1BFB, weight 0.1;
    ARROW, C_ARROW = "Server.Items.Arrow", 0xF3F, weight 0.1;
    COTTON, C_COTTON = "Server.Items.Cotton", 0xDF9, weight 4.0,
        on_double_click: |w, id, from| spin_double_click(w, id, from, false);
    FLAX, C_FLAX = "Server.Items.Flax", 0x1A9C, weight 1.0,
        label_number: |_, _| 1026812, // Flax Bundle
        can_stack_with: flax_can_stack_with,
        on_double_click: |w, id, from| spin_double_click(w, id, from, false);
    WOOL, C_WOOL = "Server.Items.Wool", 0xDF8, weight 4.0,
        on_double_click: |w, id, from| spin_double_click(w, id, from, false);
}

/// `Flax.CanStackWith(dropped)`.
fn flax_can_stack_with(w: &World, id: ItemId, dropped: ItemId) -> bool {
    let (Some(a), Some(b)) = (w.item(id), w.item(dropped)) else {
        return false;
    };
    b.stackable()
        && a.stackable()
        && std::ptr::eq(a.ty, b.ty)
        && matches!(b.item_id, 0x1A9C | 0x1A9D)
        && b.hue == a.hue
        && b.name == a.name
        && b.amount + a.amount <= 60000
        && dropped != id
}

/// `Server.Items.TaintedWool`: `base(0x101F)` then its own stack and amount.
pub static TAINTED_WOOL: ItemType = ItemType {
    full_name: "Server.Items.TaintedWool",
    parent: Some(&WOOL),
    construct: Some(|w| new_stack(w, &TAINTED_WOOL, 0x101F, 1)),
    default_weight: crate::class_props::default_weight,
    serialize: |w, id, out| {
        (WOOL.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (WOOL.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        Ok(())
    },
    ..WOOL
};
static C_TAINTED_WOOL: [Ctor; 1] = [Ctor {
    access: None,
    params: &AMOUNT_1,
    make: |w, v| EntityId::Item(new_stack(w, &TAINTED_WOOL, 0x101F, v[0].i32())),
}];

// ---- spinning ----

/// The `OnSpun` callbacks (`Wool`, `TaintedWool`, `Cotton`, `Flax`); the wheel passes the hue it was given.
fn wool_spun(w: &mut World, _wheel: ItemId, from: MobileId, h: i32) {
    if let Some(y) = make_amount(w, "Server.Items.DarkYarn", 3) {
        hue(w, y, h);
        muo_server::mobile::part2::items::add_to_backpack(w, from, y);
    }
    deps::send_localized_message(w, from, 1010576); // You put the balls of yarn in your backpack.
}

fn tainted_spun(w: &mut World, _wheel: ItemId, from: MobileId, h: i32) {
    if let Some(y) = make(w, "Server.Items.DarkYarn") {
        hue(w, y, h);
        muo_server::mobile::part2::items::add_to_backpack(w, from, y);
    }
    deps::send_localized_message(w, from, 1010574); // You put a ball of yarn in your backpack.
}

fn thread_spun(w: &mut World, _wheel: ItemId, from: MobileId, h: i32) {
    if let Some(y) = make_amount(w, "Server.Items.SpoolOfThread", 6) {
        hue(w, y, h);
        muo_server::mobile::part2::items::add_to_backpack(w, from, y);
    }
    deps::send_localized_message(w, from, 1010577); // You put the spools of thread in your backpack.
}

fn spin_double_click(w: &mut World, id: ItemId, from: MobileId, _unused: bool) {
    if in_pack(w, id, from) {
        deps::send_localized_message(w, from, 502655); // What spinning wheel do you wish to spin this on?
        target::begin_target(w, from, 3, false, TargetFlags::NONE, move |w, from, t| {
            pick_wheel(w, id, from, t)
        });
    } else {
        deps::send_localized_message(w, from, 1042001); // That must be in your pack for you to use it.
    }
}

fn pick_wheel(w: &mut World, id: ItemId, from: MobileId, t: Targeted) {
    if ops::is_deleted(w, id) {
        return;
    }
    let mut wheel = match t {
        Targeted::Entity(EntityId::Item(i)) => Some(i),
        _ => None,
    };
    if let Some(c) = wheel.and_then(|i| component_addon(w, i)) {
        wheel = Some(c);
    }
    let Some(wheel) = wheel.filter(|&i| is_spinning_wheel(w, i)) else {
        deps::send_localized_message(w, from, 502658); // Use that on a spinning wheel.
        return;
    };
    if !in_pack(w, id, from) {
        deps::send_localized_message(w, from, 1042001); // That must be in your pack for you to use it.
    } else if spinning_wheel_spinning(w, wheel) {
        deps::send_localized_message(w, from, 502656); // That spinning wheel is being used.
    } else {
        let (ty, h) = (w.item(id).map(|i| i.ty), w.item(id).map_or(0, |i| i.hue));
        let cb: fn(&mut World, ItemId, MobileId, i32) = match ty {
            Some(t) if t.is(&TAINTED_WOOL) => tainted_spun,
            Some(t) if t.is(&WOOL) => wool_spun,
            _ => thread_spun,
        };
        consume(w, id, 1);
        spinning_wheel_begin_spin(w, wheel, Some(cb), from, h);
    }
}

// ---- Cloth materials (yarns and threads) ----

/// `Server.Items.BaseClothMaterial`.
pub static BASE_CLOTH_MATERIAL: ItemType = ItemType {
    full_name: "Server.Items.BaseClothMaterial",
    construct: None,
    parent: Some(&ITEM_BASE),
    default_weight: crate::class_props::default_weight,
    on_double_click: |w, id, from| {
        if in_pack(w, id, from) {
            deps::send_localized_message(w, from, 500366); // Select a loom to use that on.
            target::begin_target(w, from, 3, false, TargetFlags::NONE, move |w, from, t| {
                pick_loom(w, id, from, t)
            });
        } else {
            deps::send_localized_message(w, from, 1042001); // That must be in your pack for you to use it.
        }
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        Ok(())
    },
    ..ITEM_BASE
};

fn pick_loom(w: &mut World, id: ItemId, from: MobileId, t: Targeted) {
    if ops::is_deleted(w, id) {
        return;
    }
    let targeted = match t {
        Targeted::Entity(EntityId::Item(i)) => Some(i),
        _ => None,
    };
    let mut loom = targeted;
    if let Some(c) = loom.and_then(|i| component_addon(w, i)) {
        loom = Some(c);
    }
    let Some(loom) = loom.filter(|&i| is_loom(w, i)) else {
        deps::send_localized_message(w, from, 500367); // Try using that on a loom.
        return;
    };
    if !in_pack(w, id, from) {
        deps::send_localized_message(w, from, 1042001); // That must be in your pack for you to use it.
        return;
    }
    let phase = loom_phase(w, loom);
    if phase < 4 {
        consume(w, id, 1);
        if let Some(item) = targeted {
            // `item.SendLocalizedMessageTo(from, 1010001 + loom.Phase++)`
            loom_set_phase(w, loom, phase + 1);
            muo_server::item::messages::send_localized_message_to(
                w,
                item,
                from,
                1010001 + phase,
                "",
            );
        }
    } else {
        let h = w.item(id).map_or(0, |i| i.hue);
        let create = make(w, "Server.Items.BoltOfCloth");
        if let Some(c) = create {
            hue(w, c, h);
        }
        consume(w, id, 1);
        loom_set_phase(w, loom, 0);
        deps::send_localized_message(w, from, 500368); // You create some cloth and put it in your backpack.
        if let Some(c) = create {
            muo_server::mobile::part2::items::add_to_backpack(w, from, c);
        }
    }
}

macro_rules! materials {
    ($($name:ident, $ctor:ident = $full:literal, $gid:expr;)*) => {$(
        #[doc = concat!("`", $full, "`.")]
        pub static $name: ItemType = ItemType {
            full_name: $full,
            parent: Some(&BASE_CLOTH_MATERIAL),
            construct: Some(|w| new_stack(w, &$name, $gid, 1)),
            serialize: |w, id, out| {
                (BASE_CLOTH_MATERIAL.serialize)(w, id, out);
                out.write_i32(0);
            },
            deserialize: |w, id, r| {
                (BASE_CLOTH_MATERIAL.deserialize)(w, id, r)?;
                let _version = r.read_i32()?;
                Ok(())
            },
            ..BASE_CLOTH_MATERIAL
        };
        static $ctor: [Ctor; 1] = [Ctor {
            access: None,
            params: &AMOUNT_1,
            make: |w, v| EntityId::Item(new_stack(w, &$name, $gid, v[0].i32())),
        }];
    )*};
}

materials! {
    DARK_YARN, C_DARK_YARN = "Server.Items.DarkYarn", 0xE1D;
    LIGHT_YARN, C_LIGHT_YARN = "Server.Items.LightYarn", 0xE1E;
    LIGHT_YARN_UNRAVELED, C_LIGHT_YARN_UNRAVELED = "Server.Items.LightYarnUnraveled", 0xE1F;
    SPOOL_OF_THREAD, C_SPOOL_OF_THREAD = "Server.Items.SpoolOfThread", 0xFA0;
}

// ---- scissors ----

fn scissor_into(w: &mut World, id: ItemId, from: MobileId, new_class: &str, per: i32) -> bool {
    if ops::is_deleted(w, id) || !deps::can_see(w, from, id) {
        return false;
    }
    if let Some(item) = make(w, new_class)
        && let Some(ty) = w.item(id).map(|i| i.ty)
    {
        (ty.scissor_helper)(w, id, from, item, per, true);
    }
    true
}

// ---- fish ----

/// `BigFish._fisher`.
#[derive(Clone, Copy, Debug, Default)]
pub struct FishData {
    pub fisher: Option<MobileId>,
}

/// `BigFish.Fisher`.
pub fn fisher(w: &World, id: ItemId) -> Option<MobileId> {
    w.item(id)
        .and_then(|i| i.ext.get::<FishData>())
        .and_then(|d| d.fisher)
}

/// `BigFish.Fisher = from`.
pub fn set_fisher(w: &mut World, id: ItemId, fisher: MobileId) {
    if let Some(d) = w.item_mut(id).and_then(|i| i.ext.get_mut::<FishData>()) {
        d.fisher = Some(fisher);
    }
}

fn big_fish_weight(w: &mut World) -> f64 {
    if muo_content::core::se(w) {
        f64::from(random_min_max(w, 3, 200))
    } else {
        100.0
    }
}

/// `Server.Items.BigFish`; `[SerializationGenerator(1, false)]`.
pub static BIG_FISH: ItemType = ItemType {
    full_name: "Server.Items.BigFish",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let id = ops::new_item_with_id(w, &BIG_FISH, 0x09CC);
        if let Some(i) = w.item_mut(id) {
            i.ext = Ext::new(FishData::default());
        }
        // TODO: find the correct formula.
        let weight = big_fish_weight(w);
        ops::set_weight(w, id, weight);
        let h = if utility::random_bool(&mut w.rng) {
            0x847
        } else {
            0x58C
        };
        hue(w, id, h);
        id
    }),
    label_number: crate::class_props::label_number, // a big fish
    get_properties: |w, id, list: &mut PropertyList| {
        (ITEM_BASE.get_properties)(w, id, list);
        let weight = ops::weight(w, id);
        if muo_content::core::se(w) && weight >= 20.0 {
            if let Some(f) = w
                .item(id)
                .and_then(|i| i.ext.get::<FishData>())
                .and_then(|d| d.fisher)
            {
                let name = w
                    .mobile(f)
                    .and_then(|m| (m.ty.name)(w, f))
                    .unwrap_or_default();
                list.add_str(1070857, &name); // Caught by ~1_fisherman~
            }
            list.add_str(1070858, &format!("{}", weight as i32)); // ~1_weight~ stones
        }
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(1);
        out.write_mobile(
            w,
            w.item(id)
                .and_then(|i| i.ext.get::<FishData>())
                .and_then(|d| d.fisher),
        );
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let version = r.read_i32()?;
        let fisher = if version < 1 {
            // Deserialize(reader, version): the weight is rolled again.
            let weight = f64::from(random_min_max(w, 3, 200));
            ops::set_weight(w, id, weight);
            None
        } else {
            r.read_mobile(w)?
        };
        if let Some(i) = w.item_mut(id) {
            i.ext = Ext::new(FishData { fisher });
        }
        Ok(())
    },
    ..ITEM_BASE
};

/// `BigFish.Carve(from, item)`.
fn big_fish_carve(w: &mut World, id: ItemId, from: MobileId, _item: ItemId) {
    let weight = ops::weight(w, id) as i32;
    let steaks = if muo_content::core::se(w) {
        4.max(weight / 5)
    } else {
        16.max(weight) / 4
    };
    if let Some(s) = make(w, "Server.Items.RawFishSteak")
        && let Some(ty) = w.item(id).map(|i| i.ty)
    {
        (ty.scissor_helper)(w, id, from, s, steaks, false);
    }
}

/// `Server.Items.Fish`: `base(Utility.Random(0x09CC, 4))`.
pub static FISH: ItemType = ItemType {
    full_name: "Server.Items.Fish",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| new_fish(w, 1)),
    default_weight: crate::class_props::default_weight,
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        Ok(())
    },
    ..ITEM_BASE
};

fn new_fish(w: &mut World, amount: i32) -> ItemId {
    let gid = random_from(w, 0x09CC, 4);
    new_stack(w, &FISH, gid, amount)
}

static C_FISH: [Ctor; 1] = [Ctor {
    access: None,
    params: &AMOUNT_1,
    make: |w, v| EntityId::Item(new_fish(w, v[0].i32())),
}];

// ---- BaseMagicFish ----

/// What a magic fish overrides: `Bonus`, `Type`, `Apply`.
pub struct MagicFishClass {
    pub apply: fn(&mut World, ItemId, MobileId) -> bool,
}

fn apply_stat(w: &mut World, from: MobileId, stat: StatType, bonus: i32) -> bool {
    let duration = if muo_content::core::se(w) {
        TimeSpan::from_secs_f64(12.0)
    } else {
        TimeSpan::from_mins(1)
    };
    let applied = muo_content::spells::helper::add_stat_offset(w, from, stat, bonus, duration);
    if !applied {
        deps::send_localized_message(w, from, 502173); // You are already under a similar effect.
    }
    applied
}

fn fish_bonus(w: &World) -> i32 {
    if muo_content::core::se(w) { 1 } else { 5 }
}

/// `Server.Items.BaseMagicFish` (abstract).
pub static BASE_MAGIC_FISH: ItemType = ItemType {
    full_name: "Server.Items.BaseMagicFish",
    construct: None,
    parent: Some(&ITEM_BASE),
    default_weight: crate::class_props::default_weight,
    on_double_click: |w, id, from| {
        if !in_pack(w, id, from) {
            deps::send_localized_message(w, from, 1042001); // That must be in your pack for you to use it.
            return;
        }
        let Some(c) = w.item(id).and_then(|i| i.ty.family::<MagicFishClass>()) else {
            return;
        };
        if (c.apply)(w, id, from) {
            muo_server::mobile::part2::effects::fixed_effect(w, from, 0x375A, 10, 15);
            muo_server::mobile::part2::effects::play_sound(w, from, 0x1E7);
            deps::local_overhead_message(w, from, MessageType::REGULAR, 0x3B2, 501774); // You swallow the fish whole!
            if let Some(ty) = w.item(id).map(|i| i.ty) {
                (ty.delete)(w, id);
            }
        }
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        Ok(())
    },
    ..ITEM_BASE
};

macro_rules! magic_fish {
    ($($name:ident, $class:ident = $full:literal, $hue:expr, $label:expr, $stat:expr;)*) => {$(
        static $class: MagicFishClass = MagicFishClass { apply: |w, _, from| {
            let bonus = fish_bonus(w);
            apply_stat(w, from, $stat, bonus)
        } };
        #[doc = concat!("`", $full, "`.")]
        pub static $name: ItemType = ItemType {
            full_name: $full,
            parent: Some(&BASE_MAGIC_FISH),
            construct: Some(|w| {
                let id = ops::new_item_with_id(w, &$name, 0xDD6);
                hue(w, id, $hue);
                id
            }),
            family: Some(&$class),
            label_number: |_, _| $label,
            serialize: |w, id, out| {
                (BASE_MAGIC_FISH.serialize)(w, id, out);
                out.write_i32(0);
            },
            deserialize: |w, id, r| {
                (BASE_MAGIC_FISH.deserialize)(w, id, r)?;
                let _version = r.read_i32()?;
                Ok(())
            },
            ..BASE_MAGIC_FISH
        };
    )*};
}

magic_fish! {
    PRIZED_FISH, PRIZED_C = "Server.Items.PrizedFish", 51, 1041073, StatType::INT;
    WONDROUS_FISH, WONDROUS_C = "Server.Items.WondrousFish", 86, 1041074, StatType::DEX;
    TRULY_RARE_FISH, TRULY_RARE_C = "Server.Items.TrulyRareFish", 76, 1041075, StatType::STR;
}

static PECULIAR_C: MagicFishClass = MagicFishClass {
    apply: |w, _, from| {
        // `from.Stam += 10`
        let s = w.mobile(from).map_or(0, |m| m.stam);
        muo_server::mobile::part1::props_b::set_stam(w, from, s + 10);
        true
    },
};

/// `Server.Items.PeculiarFish`.
pub static PECULIAR_FISH: ItemType = ItemType {
    full_name: "Server.Items.PeculiarFish",
    parent: Some(&BASE_MAGIC_FISH),
    construct: Some(|w| {
        let id = ops::new_item_with_id(w, &PECULIAR_FISH, 0xDD6);
        hue(w, id, 66);
        id
    }),
    family: Some(&PECULIAR_C),
    label_number: crate::class_props::label_number, // highly peculiar fish
    serialize: |w, id, out| {
        (BASE_MAGIC_FISH.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (BASE_MAGIC_FISH.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        Ok(())
    },
    ..BASE_MAGIC_FISH
};

/// `Hides.Scissor` (and Spined/Horned/Barbed): AOS wants the hides in the backpack.
fn scissor_hides(w: &mut World, id: ItemId, from: MobileId, leather: &str) -> bool {
    if ops::is_deleted(w, id) || !deps::can_see(w, from, id) {
        return false;
    }
    let pack = muo_server::mobile::part1::props_b::backpack(w, from);
    if muo_content::core::aos(w) && !ops::is_child_of(w, id, pack.map(EntityId::Item)) {
        deps::send_localized_message(w, from, 502437); // Items you wish to cut must be in your backpack
        return false;
    }
    scissor_into(w, id, from, leather, 1)
}

fn configure(w: &mut World) {
    use muo_content::items::resource::{BARBED_HIDES, HIDES, HORNED_HIDES, SPINED_HIDES};
    register_scissorable(w, &HIDES, |w, id, from, _| {
        scissor_hides(w, id, from, "Server.Items.Leather")
    });
    register_scissorable(w, &SPINED_HIDES, |w, id, from, _| {
        scissor_hides(w, id, from, "Server.Items.SpinedLeather")
    });
    register_scissorable(w, &HORNED_HIDES, |w, id, from, _| {
        scissor_hides(w, id, from, "Server.Items.HornedLeather")
    });
    register_scissorable(w, &BARBED_HIDES, |w, id, from, _| {
        scissor_hides(w, id, from, "Server.Items.BarbedLeather")
    });
    for t in [
        &CLOTH,
        &UNCUT_CLOTH,
        &BOLT_OF_CLOTH,
        &COTTON,
        &WOOL,
        &BASE_CLOTH_MATERIAL,
    ] {
        register_dyable(w, t, dye_plain);
    }
    register_scissorable(w, &CLOTH, |w, id, from, _| {
        scissor_into(w, id, from, "Server.Items.Bandage", 1)
    });
    register_scissorable(w, &UNCUT_CLOTH, |w, id, from, _| {
        scissor_into(w, id, from, "Server.Items.Bandage", 1)
    });
    register_scissorable(w, &BOLT_OF_CLOTH, |w, id, from, _| {
        scissor_into(w, id, from, "Server.Items.Cloth", 50)
    });
    register_carvable(w, &BIG_FISH, big_fish_carve);
    register_carvable(w, &FISH, |w, id, from, _| {
        if let Some(s) = make(w, "Server.Items.RawFishSteak")
            && let Some(ty) = w.item(id).map(|i| i.ty)
        {
            (ty.scissor_helper)(w, id, from, s, 4, true);
        }
    });
    for t in [&CLOTH, &UNCUT_CLOTH, &BOLT_OF_CLOTH, &BONE, &BOLT, &ARROW] {
        commodity::register(w, t, |_| true);
    }
    let tables: [(&str, &'static [Ctor]); 15] = [
        ("Server.Items.Cloth", &C_CLOTH),
        ("Server.Items.UncutCloth", &C_UNCUT_CLOTH),
        ("Server.Items.BoltOfCloth", &C_BOLT_OF_CLOTH),
        ("Server.Items.Bone", &C_BONE),
        ("Server.Items.Bolt", &C_BOLT),
        ("Server.Items.Arrow", &C_ARROW),
        ("Server.Items.Cotton", &C_COTTON),
        ("Server.Items.Flax", &C_FLAX),
        ("Server.Items.Wool", &C_WOOL),
        ("Server.Items.TaintedWool", &C_TAINTED_WOOL),
        ("Server.Items.DarkYarn", &C_DARK_YARN),
        ("Server.Items.LightYarn", &C_LIGHT_YARN),
        ("Server.Items.LightYarnUnraveled", &C_LIGHT_YARN_UNRAVELED),
        ("Server.Items.SpoolOfThread", &C_SPOOL_OF_THREAD),
        ("Server.Items.Fish", &C_FISH),
    ];
    for (name, t) in tables {
        add_ctors(w, name, t);
    }
}

/// Registers the classes.
pub fn register(types: &mut TypeRegistry) {
    for t in [
        &CLOTH,
        &UNCUT_CLOTH,
        &BOLT_OF_CLOTH,
        &BONE,
        &BOLT,
        &ARROW,
        &COTTON,
        &FLAX,
        &WOOL,
        &TAINTED_WOOL,
        &BASE_CLOTH_MATERIAL,
        &DARK_YARN,
        &LIGHT_YARN,
        &LIGHT_YARN_UNRAVELED,
        &SPOOL_OF_THREAD,
        &BIG_FISH,
        &FISH,
        &BASE_MAGIC_FISH,
        &PRIZED_FISH,
        &WONDROUS_FISH,
        &TRULY_RARE_FISH,
        &PECULIAR_FISH,
    ] {
        types.register_item(t, &[]);
    }
    types.register_phase_default(Phase::Configure, "Server.Items.Cloth.Configure", configure);
}

#[cfg(test)]
mod tests {
    use super::*;
    use muo_io::{GenericWriter, SaveReader, SaveWriter};

    fn save(w: &World, id: ItemId) -> Vec<u8> {
        let mut out = SaveWriter::new(true);
        (w.item(id).unwrap().ty.serialize)(w, id, &mut out);
        out.into_vec()
    }

    #[test]
    fn stackables_construct_with_their_graphic_and_round_trip() {
        for t in [
            &CLOTH,
            &BOLT_OF_CLOTH,
            &WOOL,
            &TAINTED_WOOL,
            &DARK_YARN,
            &SPOOL_OF_THREAD,
            &BOLT,
            &FLAX,
        ] {
            let mut w = World::new();
            let id = (t.construct.unwrap())(&mut w);
            assert!(w.item(id).unwrap().stackable(), "{}", t.full_name);
            let bytes = save(&w, id);
            let id2 = w.add_item(muo_server::item::Item::new_for_load(
                muo_primitives::Serial::new(0x4000_0100),
                t,
            ));
            let mut r = SaveReader::new(&bytes);
            (t.deserialize)(&mut w, id2, &mut r).unwrap();
            assert_eq!(r.remaining(), 0, "{}", t.full_name);
            assert_eq!(save(&w, id2), bytes, "{}", t.full_name);
        }
        let mut w = World::new();
        let wool = (TAINTED_WOOL.construct.unwrap())(&mut w);
        assert_eq!(w.item(wool).unwrap().item_id, 0x101F);
    }

    #[test]
    fn hides_and_oil_cloth_are_scissorable() {
        use muo_content::items::{
            interfaces::is_scissorable,
            resource::{BARBED_HIDES, HIDES, HORNED_HIDES, SPINED_HIDES},
        };
        let mut w = World::new();
        configure(&mut w);
        crate::misc::gadgets::gadgets_configure(&mut w);
        for t in [
            &HIDES,
            &SPINED_HIDES,
            &HORNED_HIDES,
            &BARBED_HIDES,
            &crate::misc::gadgets::OIL_CLOTH,
        ] {
            let id = (t.construct.unwrap())(&mut w);
            assert!(is_scissorable(&w, id), "{}", t.full_name);
        }
    }

    #[test]
    fn magic_fish_hue_and_label() {
        let mut w = World::new();
        let id = (PECULIAR_FISH.construct.unwrap())(&mut w);
        assert_eq!(
            (w.item(id).unwrap().item_id, w.item(id).unwrap().hue),
            (0xDD6, 66)
        );
        assert_eq!((PECULIAR_FISH.label_number)(&w, id), 1041076);
    }

    #[test]
    fn big_fish_saves_the_fisher_after_version_1() {
        let mut w = World::new();
        let id = (BIG_FISH.construct.unwrap())(&mut w);
        let mut want = SaveWriter::new(true);
        (ITEM_BASE.serialize)(&w, id, &mut want);
        want.write_i32(1);
        want.write_mobile(&w, None);
        assert_eq!(save(&w, id), want.into_vec());
    }
}
