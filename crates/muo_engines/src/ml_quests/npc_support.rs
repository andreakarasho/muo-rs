//! What the generated constructors of the quest givers (npcs_gen.rs) call: one function per member or
//! helper a constructor uses, named after it.

use muo_primitives::Body;
use muo_server::ids::{ItemId, MobileId};
use muo_server::mobile::part1::{props_a, props_b};
use muo_server::mobile::part2::{items, stats};
use muo_server::world::World;

pub use muo_content::creature::enums::*;
pub use muo_content::creature::setters::*;
pub use muo_server::skills::SkillName;

/// `creature_leaf!`: one concrete creature class (a `MobileType` plus its `CreatureType` table of overrides).
/// The save is "parent part + version int 0" (`encoded` for `[SerializationGenerator(0)]` without `false`).
macro_rules! creature_leaf {
    // `template`: the constructor and `DefaultName`/`IsInvulnerable`/`CanShout`/`CanTeach` are the class's mobile
    // template (`Data/Templates/Mobiles`) and its props.
    (
        $(#[$m:meta])*
        $name:ident, $class:ident = $full:literal : $parent:ident / $pclass:ident, template, encoded $enc:literal;
        $($rest:tt)*
    ) => {
        creature_leaf! {
            $(#[$m])*
            $name, $class = $full : $parent / $pclass, ctor (|w| muo_content::templates::new_creature(w, &$name)), encoded $enc;
            c {
                default_name: muo_content::templates::creature_props::default_name,
                is_invulnerable: |w, id| $crate::template_keys::mobile_bool(w, id, "isInvulnerable", $pclass.is_invulnerable),
                can_shout: |w, id| $crate::template_keys::mobile_bool(w, id, "canShout", $pclass.can_shout),
                can_teach: |w, id| $crate::template_keys::mobile_bool(w, id, "canTeach", $pclass.can_teach),
            }
            $($rest)*
        }
    };
    (
        $(#[$m:meta])*
        $name:ident, $class:ident = $full:literal : $parent:ident / $pclass:ident, ctor $ctor:expr, encoded $enc:literal;
        c { $($cf:ident : $cv:expr,)* }
        m { $($mf:ident : $mv:expr,)* }
    ) => {
        /// The creature-family overrides of the class.
        pub static $class: muo_content::creature::CreatureType =
            muo_content::creature::CreatureType { $($cf: $cv,)* ..$pclass };

        $(#[$m])*
        pub static $name: muo_server::mobile::MobileType = muo_server::mobile::MobileType {
            full_name: $full,
            parent: Some(&$parent),
            construct: Some($ctor),
            family: Some(&$class),
            serialize: |w, id, out| {
                ($parent.serialize)(w, id, out);
                if $enc {
                    out.write_encoded_int(0);
                } else {
                    out.write_i32(0);
                }
            },
            deserialize: |w, id, r| {
                ($parent.deserialize)(w, id, r)?;
                let _version = if $enc { r.read_encoded_int()? } else { r.read_i32()? };
                Ok(())
            },
            $($mf: $mv,)*
            ..$parent
        };
    };
}
pub(crate) use creature_leaf;

/// `Title = value`.
pub fn set_title(w: &mut World, id: MobileId, value: Option<String>) {
    props_a::set_title(w, id, value);
}

/// `Body = value`.
pub fn set_body(w: &mut World, id: MobileId, value: Body) {
    props_b::set_body(w, id, value);
}

/// `Female = value`.
pub fn set_female(w: &mut World, id: MobileId, value: bool) {
    props_b::set_female(w, id, value);
}

/// `Frozen = value`.
pub fn set_frozen(w: &mut World, id: MobileId, value: bool) {
    props_a::set_frozen(w, id, value);
}

/// `Hue = value`.
pub fn set_hue(w: &mut World, id: MobileId, value: i32) {
    muo_server::mobile::base::set_hue_base(w, id, value);
}

pub fn set_hair_item_id(w: &mut World, id: MobileId, value: i32) {
    props_b::set_hair_item_id(w, id, value);
}

pub fn set_facial_hair_item_id(w: &mut World, id: MobileId, value: i32) {
    props_b::set_facial_hair_item_id(w, id, value);
}

pub fn set_hair_hue(w: &mut World, id: MobileId, value: i32) {
    props_b::set_hair_hue(w, id, value);
}

pub fn set_facial_hair_hue(w: &mut World, id: MobileId, value: i32) {
    props_b::set_facial_hair_hue(w, id, value);
}

/// `HairHue`.
pub fn hair_hue(w: &World, id: MobileId) -> i32 {
    w.mobile(id).map_or(0, |m| m.hair_hue)
}

/// `Race = Race.<name>`.
pub fn set_race_named(w: &mut World, id: MobileId, name: &str) {
    let race = w
        .resources
        .get::<muo_server::race::Races>()
        .and_then(|r| r.try_parse(name));
    props_a::set_race(w, id, race);
}

/// `SetSpeed(active, passive, isPassive)`.
pub fn set_speed(w: &mut World, id: MobileId, active: f64, passive: f64, is_passive: bool) {
    muo_content::creature::speeds::set_speed(w, id, active, passive, is_passive);
}

/// `InitStats(str, dex, int)`.
pub fn init_stats(w: &mut World, id: MobileId, str: i32, dex: i32, intel: i32) {
    stats::init_stats(w, id, str, dex, intel);
}

/// `Race.RandomSkinHue()` of the mobile's race.
pub fn race_skin_hue(w: &mut World, id: MobileId) -> i32 {
    let race = w.mobile(id).and_then(|m| m.race);
    let ops = w
        .resources
        .get::<muo_server::race::Races>()
        .and_then(|r| race.and_then(|id| r.get(id)).or_else(|| r.default_race()))
        .map(|r| r.ops);
    match ops {
        Some(o) => (o.random_skin_hue)(&mut w.rng),
        None => 0,
    }
}

/// `Utility.AssignRandomHair(m)`: the item, then the race's random hue (that order of random calls).
pub fn assign_random_hair(w: &mut World, id: MobileId) {
    muo_content::vendor::assign_random_hair(w, id, 0);
    let hue = muo_content::vendor::race_random_hair_hue(w, id);
    props_b::set_hair_hue(w, id, hue);
}

/// `Utility.AssignRandomHair(m, hue)`.
pub fn assign_random_hair_hue(w: &mut World, id: MobileId, hue: i32) {
    muo_content::vendor::assign_random_hair(w, id, hue);
}

/// `Utility.AssignRandomFacialHair(m)`.
#[allow(dead_code)]
pub fn assign_random_facial_hair(w: &mut World, id: MobileId) {
    muo_content::vendor::assign_random_facial_hair(w, id, 0);
    let hue = muo_content::vendor::race_random_hair_hue(w, id);
    props_b::set_facial_hair_hue(w, id, hue);
}

/// `Utility.AssignRandomFacialHair(m, hue)`.
pub fn assign_random_facial_hair_hue(w: &mut World, id: MobileId, hue: i32) {
    muo_content::vendor::assign_random_facial_hair(w, id, hue);
}

/// `new T()` by short class name.
pub fn make_named(w: &mut World, name: &str) -> Option<ItemId> {
    match w.types.find_type_by_name(name, false, false)? {
        muo_server::type_registry::EntityType::Item(t) => (t.construct?)(w).into(),
        muo_server::type_registry::EntityType::Mobile(_) => None,
    }
}

/// `item.Hue = value`.
pub fn set_item_hue(w: &mut World, item: Option<ItemId>, hue: i32) {
    if let Some(i) = item
        && let Some(ty) = w.item(i).map(|i| i.ty)
    {
        (ty.set_hue)(w, i, hue);
    }
}

/// `AddItem(item)`.
pub fn add_item_id(w: &mut World, id: MobileId, item: Option<ItemId>) {
    if let Some(i) = item {
        items::add_item(w, id, i);
    }
}

/// `AddItem(new T(hue))`: the class's `(int hue)` constructor only sets `Hue`.
pub fn add_item_hue(w: &mut World, id: MobileId, name: &str, hue: Option<i32>) {
    let item = make_named(w, name);
    if let Some(h) = hue {
        set_item_hue(w, item, h);
    }
    add_item_id(w, id, item);
}

/// `AddItem(new T { ItemID = a, Hue = b })`.
pub fn add_item_props(
    w: &mut World,
    id: MobileId,
    name: &str,
    item_id: Option<i32>,
    hue: Option<i32>,
) {
    let item = make_named(w, name);
    if let (Some(i), Some(g)) = (item, item_id)
        && let Some(ty) = w.item(i).map(|i| i.ty)
    {
        (ty.set_item_id)(w, i, g);
    }
    if let Some(h) = hue {
        set_item_hue(w, item, h);
    }
    add_item_id(w, id, item);
}
