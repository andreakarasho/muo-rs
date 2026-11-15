//! The weapon classes with code of their own: `ThrowingDagger` (an `Item`, not a weapon), `FireworksWand` (a
//! field), `Fists` (the default weapon), and the targets/overrides of `JukaBow`, `ShepherdsCrook`,
//! `ButchersWarCleaver` and `BoneMachete`. The numbers of these classes are in the generated files next to this
//! one; this file has the functions they point to.

use muo_content::creature::data::{CreatureData, cd};
use muo_content::items::equip_hooks::{CraftArgs, lower_text};
use muo_content::items::weapon::{self, BASE_MELEE_WEAPON, BASE_MELEE_WEAPON_CLASS, WeaponQuality};
use muo_content::items::weapon_bases;
use muo_content::items::weapon_combat as wc;
use muo_content::items::weapon_hooks::{self as hk, WeaponHooks};
use muo_content::items::{core_uor, core_uotd, is_named};
use muo_content::misc::aos_helpers::{self, DamageOptions, DamageSplit};
use muo_content::{core, mx};

use muo_primitives::MessageType;
use muo_primitives::{Point2D, Point3D, TimeSpan, utility};
use muo_server::effects;
use muo_server::ids::{EntityId, MobileId};
use muo_server::item::{ITEM_BASE, base as item_base, deps, ops};
use muo_server::map::MapId;
use muo_server::mobile::base as mb;
use muo_server::mobile::part1::{combat, props_b};
use muo_server::mobile::part2::{effects as mfx, messages as msgs, stats};
use muo_server::property_list::PropertyList;
use muo_server::skills;
use muo_server::target::{self, TargetFlags, Targeted};
use muo_server::timer;
use muo_server::type_registry::Phase;

use super::*;

// ---------------------------------------------------------------------------------------------------------------
// ThrowingDagger: an `Item` with a `Flippable(0xF52, 0xF51)`.
// ---------------------------------------------------------------------------------------------------------------

/// `[Flippable(0xF52, 0xF51)]` of `ThrowingDagger` (it is an `Item`, so it has no `WeaponClass`).
pub const THROWING_DAGGER_FLIP: (i32, i32) = (0xF52, 0xF51);

/// `ThrowingDagger()`: `base(0xF52)`, `Layer = Layer.OneHanded`.
pub fn throwing_dagger_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.ThrowingDagger")
}

/// `Server.Items.ThrowingDagger` (`[SerializationGenerator(0, false)]`).
pub static THROWING_DAGGER: ItemType = ItemType {
    full_name: "Server.Items.ThrowingDagger",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| throwing_dagger_new(w, &THROWING_DAGGER)),
    default_weight: crate::class_props::default_weight,
    default_name: crate::class_props::default_name,
    on_double_click: throwing_dagger_double_click,
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

/// `from.Items.Contains(item)`: the item is worn by `from`.
fn worn_by(w: &World, item: ItemId, from: MobileId) -> bool {
    w.item(item)
        .is_some_and(|i| i.parent == Some(EntityId::Mobile(from)))
}

fn throwing_dagger_double_click(w: &mut World, id: ItemId, from: MobileId) {
    if worn_by(w, id, from) {
        // InternalTarget: range 10, no ground, Harmful.
        target::begin_target(
            w,
            from,
            10,
            false,
            TargetFlags::HARMFUL,
            move |w, from, targeted| {
                throwing_dagger_on_target(w, id, from, targeted);
            },
        );
    } else {
        mx::send_message(w, from, "You must be holding that weapon to use it.");
    }
}

fn throwing_dagger_on_target(w: &mut World, dagger: ItemId, from: MobileId, targeted: Targeted) {
    if w.item(dagger).is_none_or(|i| i.deleted()) {
        return;
    }
    if !worn_by(w, dagger, from) {
        mx::send_message(w, from, "You must be holding that weapon to use it.");
        return;
    }
    let Targeted::Entity(EntityId::Mobile(m)) = targeted else {
        return;
    };
    if m == from || !mb::harmful_check(w, from, m) {
        return;
    }
    let to = mx::get_direction_to(w, from, m);
    mx::set_direction(w, from, to);
    let mounted = props_b::mounted(w, from);
    mx::animate(w, from, if mounted { 26 } else { 9 }, 7, 1, true, false, 0);

    let (loc, map) = (mx::location(w, m), mx::map(w, m));
    let dex = mb::dex(w, m);
    if utility::random_double(&mut w.rng) >= (f64::from(dex) / 100.0).sqrt() * 0.8 {
        mfx::moving_effect(
            w,
            from,
            EntityId::Mobile(m),
            0x1BFE,
            7,
            1,
            false,
            false,
            0x481,
            0,
        );
        let str = mb::str(w, from);
        let amount = utility::random_from(&mut w.rng, 5, str / 10);
        aos_helpers::damage(
            w,
            m,
            Some(from),
            amount,
            DamageSplit {
                phys: 100,
                fire: 0,
                cold: 0,
                pois: 0,
                nrgy: 0,
            },
            DamageOptions::default(),
        );
        ops::move_to_world_map(w, dagger, loc, map);
    } else {
        let mut p = loc;
        muo_server::movement::offset(to, &mut p, 1);
        p.x += utility::random_from(&mut w.rng, -1, 3);
        p.y += utility::random_from(&mut w.rng, -1, 3);
        ops::move_to_world_map(w, dagger, p, map);
        mfx::moving_effect(
            w,
            from,
            EntityId::Item(dagger),
            0x1BFE,
            7,
            1,
            false,
            false,
            0x481,
            0,
        );
        mx::send_message(w, from, "You miss.");
    }
}

// ---------------------------------------------------------------------------------------------------------------
// FireworksWand: `Charges`, a `[SerializableField(0)]` that lives in `WeaponData`.
// ---------------------------------------------------------------------------------------------------------------

/// `FireworksWand()`: `MagicWand()`, then its template (`Charges = 100; LootType = LootType.Blessed;`).
pub fn fireworks_wand_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = magic_wand_new(w, ty);
    muo_content::templates::apply_item_as(w, id, "Server.Items.FireworksWand");
    id
}

/// `Charges`.
pub fn charges(w: &World, id: ItemId) -> i32 {
    weapon::data(w, id).map_or(0, |d| d.charges)
}

/// `Charges = value` (`[InvalidateProperties]`).
pub fn set_charges(w: &mut World, id: ItemId, value: i32) {
    if let Some(d) = weapon::data_mut(w, id)
        && d.charges != value
    {
        d.charges = value;
        ops::invalidate_properties(w, id);
    }
}

/// `Server.Items.FireworksWand`: MagicWand's record, then its own version (an int) and `_charges`.
pub static FIREWORKS_WAND: ItemType = ItemType {
    full_name: "Server.Items.FireworksWand",
    parent: Some(&MAGIC_WAND),
    construct: Some(|w| fireworks_wand_new(w, &FIREWORKS_WAND)),
    serialize: |w, id, out| {
        (MAGIC_WAND.serialize)(w, id, out);
        out.write_i32(0);
        out.write_i32(charges(w, id));
    },
    deserialize: |w, id, r| {
        (MAGIC_WAND.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        let c = r.read_i32()?;
        if let Some(d) = weapon::data_mut(w, id) {
            d.charges = c;
        }
        Ok(())
    },
    add_name_properties: |w, id, list| {
        (MAGIC_WAND.add_name_properties)(w, id, list);
        list.add_str(1060741, &charges(w, id).to_string()); // charges: ~1_val~
    },
    on_double_click: |w, id, from| begin_launch(w, id, from, true),
    on_single_click: |w, id, from| {
        // `OnSingleClickPreUOTD` is the override; from UOTD on the weapon's own OnSingleClick runs.
        if core_uotd(w) {
            (MAGIC_WAND.on_single_click)(w, id, from);
        } else {
            let text = ops::name(w, id).unwrap_or_else(|| {
                let n = (FIREWORKS_WAND.label_number)(w, id);
                lower_text(w, n)
            });
            ops::label_to(w, id, from, &text);
        }
    },
    ..MAGIC_WAND
};

/// `BeginLaunch(from, useCharges)`.
pub fn begin_launch(w: &mut World, id: ItemId, from: MobileId, use_charges: bool) {
    let map = mx::map(w, from);
    if map.is_none_or(|m| m == MapId::INTERNAL) {
        return;
    }
    if use_charges {
        let c = charges(w, id);
        if c > 0 {
            set_charges(w, id, c - 1);
        } else {
            mx::msg(w, from, 502412); // There are no charges left on that item.
            return;
        }
    }
    mx::msg(w, from, 502615); // You launch a firework!

    let our = ops::get_world_location(w, id);
    let start = Point3D::new(our.x, our.y, our.z + 10);
    let dx = utility::random_min_max(&mut w.rng, -2, 2);
    let dy = utility::random_min_max(&mut w.rng, -2, 2);
    let end = Point3D::new(start.x + dx, start.y + dy, start.z + 32);
    effects::send_moving_effect_map(w, map, 0x36E4, start, end, 5, 0, false, false, 0, 0);
    timer::start(w, TimeSpan::from_secs(1), TimeSpan::ZERO, 1, move |w| {
        finish_launch(w, end, map)
    });
}

fn finish_launch(w: &mut World, end: Point3D, map: Option<MapId>) {
    let r = utility::random(&mut w.rng, 40);
    let mut hue = if r < 8 {
        0x66D
    } else if r < 10 {
        0x482
    } else if r < 12 {
        0x47E
    } else if r < 16 {
        0x480
    } else if r < 20 {
        0x47F
    } else {
        0
    };
    if utility::random_bool(&mut w.rng) {
        const HUES: [i32; 5] = [0x47E, 0x47F, 0x480, 0x482, 0x66D];
        hue = HUES[utility::random(&mut w.rng, 5) as usize];
    }
    const MODES: [i32; 6] = [0, 2, 3, 4, 5, 7];
    let render_mode = MODES[utility::random(&mut w.rng, 6) as usize];
    let sound = utility::random_from(&mut w.rng, 0x11B, 4);
    effects::play_sound(w, end, map, sound);
    let fx = 0x373A + 0x10 * utility::random(&mut w.rng, 4);
    effects::send_location_effect(w, end, map, fx, 16, 10, hue, render_mode);
}

// ---------------------------------------------------------------------------------------------------------------
// JukaBow
// ---------------------------------------------------------------------------------------------------------------

/// `IsModified`.
pub fn juka_bow_is_modified(w: &World, id: ItemId) -> bool {
    w.item(id).is_some_and(|i| i.hue == 0x453)
}

fn in_backpack(w: &mut World, id: ItemId, from: MobileId) -> bool {
    let pack = props_b::backpack(w, from);
    ops::is_child_of(w, id, pack.map(EntityId::Item))
}

pub(super) fn juka_bow_on_double_click(w: &mut World, id: ItemId, from: MobileId) {
    if juka_bow_is_modified(w, id) {
        mx::send_message(w, from, "That has already been modified.");
    } else if !in_backpack(w, id, from) {
        mx::send_message(w, from, "This must be in your backpack to modify it.");
    } else if skills::base(w, from, SkillName::Fletching) < 100.0 {
        mx::send_message(
            w,
            from,
            "Only a grandmaster bowcrafter can modify this weapon.",
        );
    } else {
        target::begin_target(
            w,
            from,
            2,
            false,
            TargetFlags::NONE,
            move |w, from, targ| {
                juka_bow_on_target_gears(w, id, from, targ);
            },
        );
        mx::send_message(w, from, "Select the gears you wish to use.");
    }
}

/// `OnTargetGears(from, targ)`.
pub fn juka_bow_on_target_gears(w: &mut World, id: ItemId, from: MobileId, targ: Targeted) {
    let gears = match targ {
        Targeted::Entity(EntityId::Item(g))
            if w.item(g)
                .is_some_and(|i| is_named(i.ty, "Server.Items.Gears")) =>
        {
            Some(g)
        }
        _ => None,
    };
    let gears = gears.filter(|&g| in_backpack(w, g, from));
    let Some(g) = gears else {
        // Apparently gears that aren't in your backpack aren't really gears at all. :-(
        mx::send_message(w, from, "Those are not gears.");
        return;
    };
    if juka_bow_is_modified(w, id) {
        mx::send_message(w, from, "That has already been modified.");
    } else if !in_backpack(w, id, from) {
        mx::send_message(w, from, "This must be in your backpack to modify it.");
    } else if skills::base(w, from, SkillName::Fletching) < 100.0 {
        mx::send_message(
            w,
            from,
            "Only a grandmaster bowcrafter can modify this weapon.",
        );
    } else {
        item_base::consume(w, g, 1);
        ctor::set_hue(w, id, 0x453);
        let s = utility::random_from(&mut w.rng, 2, 25);
        if let Some(d) = weapon::data_mut(w, id) {
            d.slayer = s;
        }
        ops::invalidate_properties(w, id);
        mx::send_message(w, from, "You modify it.");
    }
}

// ---------------------------------------------------------------------------------------------------------------
// ShepherdsCrook
// ---------------------------------------------------------------------------------------------------------------

/// The champion-spawn creatures a crook can herd (`m_ChampTamables`).
const CHAMP_TAMABLES: [&str; 17] = [
    "Server.Mobiles.StrongMongbat",
    "Server.Mobiles.Imp",
    "Server.Mobiles.Scorpion",
    "Server.Mobiles.GiantSpider",
    "Server.Mobiles.Snake",
    "Server.Mobiles.LavaLizard",
    "Server.Mobiles.Drake",
    "Server.Mobiles.Dragon",
    "Server.Mobiles.Kirin",
    "Server.Mobiles.Unicorn",
    "Server.Mobiles.GiantRat",
    "Server.Mobiles.Slime",
    "Server.Mobiles.DireWolf",
    "Server.Mobiles.HellHound",
    "Server.Mobiles.DeathwatchBeetle",
    "Server.Mobiles.LesserHiryu",
    "Server.Mobiles.Hiryu",
];

/// What the champion spawn engine answers for `IsHerdable`, set by the owning crate. Unset: no region is one.
#[derive(Default)]
pub struct HerdingHooks {
    /// `Region.Find(bc.Home, map) is ChampionSpawnRegion region && region.Spawn?.IsChampionSpawn(bc) == true`.
    pub in_champion_spawn_region_of: Option<fn(&World, MobileId) -> bool>,
}

pub(super) fn shepherds_crook_on_double_click(w: &mut World, _id: ItemId, from: MobileId) {
    mx::msg(w, from, 502464); // Target the animal you wish to herd.
    target::begin_target(w, from, 10, false, TargetFlags::NONE, herding_target);
}

/// `HerdingTarget.IsHerdable(bc)`.
fn is_herdable(w: &World, bc: MobileId) -> bool {
    let Some(d) = cd(w, bc) else { return false };
    if d.is_paragon {
        return false;
    }
    if d.tamable() {
        return true;
    }
    let owned = w
        .resources
        .get::<HerdingHooks>()
        .and_then(|h| h.in_champion_spawn_region_of)
        .is_some_and(|f| f(w, bc));
    if !owned {
        return false;
    }
    // `type == t`: the exact class, not a subclass.
    w.mobile(bc)
        .is_some_and(|m| CHAMP_TAMABLES.contains(&m.ty.full_name))
}

fn herding_target(w: &mut World, from: MobileId, targ: Targeted) {
    let Targeted::Entity(EntityId::Mobile(bc)) = targ else {
        mx::msg(w, from, 502472); // You don't seem to be able to persuade that to move.
        return;
    };
    if cd(w, bc).is_none() {
        mx::msg(w, from, 502472);
        return;
    }
    if !is_herdable(w, bc) {
        mx::msg(w, from, 502468); // That is not a herdable animal.
        return;
    }
    if cd(w, bc).is_some_and(|d| d.controlled) {
        let ns = deps::net_state_of(w, from);
        msgs::private_overhead_message_number(
            w,
            bc,
            MessageType::REGULAR,
            0x3B2,
            502467, // That animal looks tame already.
            "",
            ns,
        );
    } else {
        mx::msg(w, from, 502475); // Click where you wish the animal to go.
        target::begin_target(
            w,
            from,
            10,
            true,
            TargetFlags::NONE,
            move |w, from, targ| {
                herding_destination(w, bc, from, targ);
            },
        );
    }
}

fn herding_destination(w: &mut World, bc: MobileId, from: MobileId, targ: Targeted) {
    // `targ is not IPoint2D p`: an entity, a land tile or a static all are.
    let p = match targ {
        Targeted::Entity(EntityId::Mobile(m)) => mx::location(w, m),
        Targeted::Entity(EntityId::Item(i)) => item_base::location(w, i), // raw X/Y
        Targeted::Land(l) => l.location,
        Targeted::Static(s) => s.location,
    };
    let tame = cd(w, bc).map_or(0.0, |d| d.min_tame_skill);
    let min = tame - 30.0;
    let max = tame + 30.0 + f64::from(utility::random(&mut w.rng, 10));

    if max <= skills::value(w, from, SkillName::Herding) {
        let ns = deps::net_state_of(w, from);
        msgs::private_overhead_message_number(
            w,
            bc,
            MessageType::REGULAR,
            0x3B2,
            502471, // That wasn't even challenging.
            "",
            ns,
        );
    }
    // the target object is the creature (`m_Creature`), not the clicked spot
    let creature = Targeted::Entity(EntityId::Mobile(bc));
    if stats::check_target_skill(w, from, SkillName::Herding, creature, min, max) {
        if let Some(d) = w
            .mobile_mut(bc)
            .and_then(|m| m.ext.get_mut::<CreatureData>())
        {
            d.target_location = Some(Point2D::new(p.x, p.y));
        }
        mx::msg(w, from, 502479); // The animal walks where it was instructed to.
    } else {
        mx::msg(w, from, 502472); // You don't seem to be able to persuade that to move.
    }
}

// ---------------------------------------------------------------------------------------------------------------
// ButchersWarCleaver and BoneMachete
// ---------------------------------------------------------------------------------------------------------------

pub(super) fn butchers_war_cleaver_append_child_name_properties(
    w: &World,
    id: ItemId,
    list: &mut PropertyList,
) {
    (WAR_CLEAVER.append_child_name_properties)(w, id, list);
    list.add(1072512); // Bovine Slayer
}

/// `BoneMachete.OnTicketUsed(from)` (`ITicket`, Engines/MLQuests): the machete snaps (`Delete()`) a quarter of the
/// time.
pub fn bone_machete_on_ticket_used(w: &mut World, id: ItemId, from: MobileId) {
    if utility::random_double(&mut w.rng) < 0.25 {
        // Your bone handled machete snaps in half as you force your way through the poisonous undergrowth.
        mx::msg(w, from, 1075007);
        if let Some(ty) = w.item(id).map(|i| i.ty) {
            (ty.delete)(w, id);
        }
    } else {
        // Your bone handled machete has grown dull but you still manage to force your way past the venomous branches.
        mx::msg(w, from, 1075008);
    }
}

// ---------------------------------------------------------------------------------------------------------------
// Fists: `Mobile.DefaultWeapon`
// ---------------------------------------------------------------------------------------------------------------

/// The `Fists` singleton (`Mobile.DefaultWeapon`), in `w.resources`.
pub struct FistsSingleton(pub ItemId);

/// `Fists()`: `base(graphic)`, then its template (`Visible = false; Movable = false;`; `Quality = Regular` is what
/// `new_weapon` already set).
pub fn fists_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let g = muo_content::items::equip_props::graphic(w, ty);
    let id = new_weapon(w, ty, g);
    muo_content::templates::apply_item_as(w, id, "Server.Items.Fists");
    id
}

/// `Fists` overrides that are code (the stats are its template's props).
pub static FISTS_CLASS: WeaponClass = WeaponClass {
    on_swing: fists_on_swing,
    get_defend_skill_value: fists_defend_skill_value,
    ..BASE_MELEE_WEAPON_CLASS
};

/// `Server.Items.Fists`. Not `[Constructible]`, and `SkipSerialization`, so it is never saved.
pub static FISTS: ItemType = ItemType {
    full_name: "Server.Items.Fists",
    parent: Some(&BASE_MELEE_WEAPON),
    construct: None,
    family: Some(&FISTS_CLASS),
    skip_serialization: |_, _| true,
    serialize: |w, id, out| {
        (BASE_MELEE_WEAPON.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (BASE_MELEE_WEAPON.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        Ok(())
    },
    ..BASE_MELEE_WEAPON
};

/// `Fists.Initialize()`: `Mobile.DefaultWeapon = new Fists()`.
fn fists_initialize(w: &mut World) {
    let id = fists_new(w, &FISTS);
    w.resources.insert(FistsSingleton(id));
    w.resources.get_mut::<WeaponHooks>().fists_item =
        Some(|w| w.resources.get::<FistsSingleton>().map(|f| f.0));
}

fn fists_defend_skill_value(
    w: &mut World,
    _id: ItemId,
    _attacker: MobileId,
    defender: MobileId,
) -> f64 {
    let wres = skills::value(w, defender, SkillName::Wrestling);
    if !core::lbr(w) {
        return wres;
    }
    let anat = skills::value(w, defender, SkillName::Anatomy);
    let eval = skills::value(w, defender, SkillName::EvalInt);
    let incr = ((anat + eval + 20.0) * 0.5).min(120.0);
    if wres > incr { wres } else { incr }
}

fn fists_on_swing(
    w: &mut World,
    id: ItemId,
    attacker: MobileId,
    defender: MobileId,
    _bonus: f64,
) -> TimeSpan {
    if !core::aos(w) {
        check_pre_aos_moves(w, attacker, defender);
    }
    wc::swing_base(w, id, attacker, defender, 1.0) // base.OnSwing(attacker, defender)
}

/// `BeginAction<Fists>()`'s key.
const FISTS_KEY: &str = "Server.Items.Fists";

fn check_pre_aos_moves(w: &mut World, attacker: MobileId, defender: MobileId) {
    if !core_uor(w) || !combat::can_begin_action(w, attacker, FISTS_KEY) {
        return;
    }
    if w.mobile(attacker).is_some_and(|m| m.stun_ready) {
        if skills::value(w, attacker, SkillName::Anatomy) < 80.0
            || skills::value(w, attacker, SkillName::Wrestling) < 80.0
        {
            mx::msg(w, attacker, 1004008); // You are not skilled enough to stun your opponent.
            set_stun_ready(w, attacker, false);
            return;
        }
        if stam(w, attacker) < 15 {
            mx::msg(w, attacker, 1004009); // You are too fatigued to attempt anything.
            return;
        }
        add_stam(w, attacker, -15);
        if check_move(w, attacker, SkillName::Anatomy) {
            start_move_delay(w, attacker);
            set_stun_ready(w, attacker, false);
            mx::msg(w, attacker, 1004013); // You successfully stun your opponent!
            mx::msg(w, defender, 1004014); // You have been stunned!
            mb::freeze(w, defender, TimeSpan::from_secs(4));
        } else {
            mx::msg(w, attacker, 1004010); // You failed in your attempt to stun.
            mx::msg(w, defender, 1004011); // Your opponent tried to stun you and failed.
        }
        return;
    }
    if !w.mobile(attacker).is_some_and(|m| m.disarm_ready) {
        return;
    }
    let humanoid = w.mobile(defender).is_some_and(|m| {
        m.player
            || w.resources
                .get::<muo_primitives::BodyTable>()
                .is_some_and(|t| t.is_human(muo_server::mobile::part1::props_b::body(w, defender)))
    });
    if !humanoid {
        mx::msg(w, attacker, 1004001); // You cannot disarm your opponent.
        return;
    }
    if skills::value(w, attacker, SkillName::ArmsLore) < 80.0
        || skills::value(w, attacker, SkillName::Wrestling) < 80.0
    {
        mx::msg(w, attacker, 1004002); // You are not skilled enough to disarm your opponent.
        set_disarm_ready(w, attacker, false);
        return;
    }
    if stam(w, attacker) < 15 {
        mx::msg(w, attacker, 1004003); // You are too fatigued to attempt anything.
        return;
    }
    let movable = |w: &World, i: ItemId| w.item(i).is_some_and(|i| i.movable());
    let mut to_disarm = deps::find_item_on_layer(w, defender, Layer::OneHanded);
    if !to_disarm.is_some_and(|i| movable(w, i)) {
        to_disarm = deps::find_item_on_layer(w, defender, Layer::TwoHanded);
    }
    let pack = props_b::backpack(w, defender);
    let (Some(pack), Some(to_disarm)) = (pack, to_disarm.filter(|&i| movable(w, i))) else {
        mx::msg(w, attacker, 1004001); // You cannot disarm your opponent.
        return;
    };
    if check_move(w, attacker, SkillName::ArmsLore) {
        start_move_delay(w, attacker);
        add_stam(w, attacker, -15);
        set_disarm_ready(w, attacker, false);
        mx::msg(w, attacker, 1004006); // You successfully disarm your opponent!
        mx::msg(w, defender, 1004007); // You have been disarmed!
        if let Some(ty) = w.item(pack).map(|i| i.ty) {
            (ty.drop_item)(w, pack, to_disarm);
        }
    } else {
        add_stam(w, attacker, -15);
        mx::msg(w, attacker, 1004004); // You failed in your attempt to disarm.
        mx::msg(w, defender, 1004005); // Your opponent tried to disarm you but failed.
    }
}

fn stam(w: &World, m: MobileId) -> i32 {
    w.mobile(m).map_or(0, |m| m.stam)
}

/// `m.Stam += delta`.
fn add_stam(w: &mut World, m: MobileId, delta: i32) {
    let v = stam(w, m) + delta;
    props_b::set_stam(w, m, v);
}

fn set_stun_ready(w: &mut World, m: MobileId, v: bool) {
    if let Some(m) = w.mobile_mut(m) {
        m.stun_ready = v;
    }
}

fn set_disarm_ready(w: &mut World, m: MobileId, v: bool) {
    if let Some(m) = w.mobile_mut(m) {
        m.disarm_ready = v;
    }
}

/// `CheckMove(m, other)`: 40% chance at 80, 80; 50% at 100, 100; 60% at 120, 120.
fn check_move(w: &mut World, m: MobileId, other: SkillName) -> bool {
    let wres = skills::value(w, m, SkillName::Wrestling);
    let scnd = skills::value(w, m, other);
    let chance = (wres + scnd) / 400.0;
    chance >= utility::random_double(&mut w.rng)
}

/// `StartMoveDelay(m)`: `MoveDelayTimer`, 10 seconds of `BeginAction<Fists>`.
fn start_move_delay(w: &mut World, m: MobileId) {
    combat::begin_action(w, m, FISTS_KEY);
    timer::start(w, TimeSpan::from_secs(10), TimeSpan::ZERO, 1, move |w| {
        combat::end_action(w, m, FISTS_KEY);
    });
}

/// `HasFreeHands(m)`.
fn has_free_hands(w: &World, m: MobileId) -> bool {
    let one_free = match deps::find_item_on_layer(w, m, Layer::OneHanded) {
        None => true,
        Some(i) => w
            .item(i)
            .is_some_and(|i| is_named(i.ty, "Server.Items.Spellbook")),
    };
    one_free && deps::find_item_on_layer(w, m, Layer::TwoHanded).is_none()
}

/// `Fists.DisarmRequest(m)`.
pub fn disarm_request(w: &mut World, m: MobileId) {
    if core::aos(w) || !core_uor(w) {
        return;
    }
    if !hk::duel_allow_special_ability(w, m, "Disarm", true) {
        return;
    }
    let arms = skills::value(w, m, SkillName::ArmsLore);
    let wres = skills::value(w, m, SkillName::Wrestling);
    if !has_free_hands(w, m) {
        mx::msg(w, m, 1004029); // You must have your hands free to attempt to disarm your opponent.
        set_disarm_ready(w, m, false);
    } else if arms >= 80.0 && wres >= 80.0 {
        mb::disruptive_action(w, m);
        let v = !w.mobile(m).is_some_and(|m| m.disarm_ready);
        set_disarm_ready(w, m, v);
        mx::msg(w, m, if v { 1019013 } else { 1019014 });
    } else {
        mx::msg(w, m, 1004002); // You are not skilled enough to disarm your opponent.
        set_disarm_ready(w, m, false);
    }
}

/// `Fists.StunRequest(m)`.
pub fn stun_request(w: &mut World, m: MobileId) {
    if core::aos(w) || !core_uor(w) || !hk::duel_allow_special_ability(w, m, "Stun", true) {
        return;
    }
    let anat = skills::value(w, m, SkillName::Anatomy);
    let wres = skills::value(w, m, SkillName::Wrestling);
    if !has_free_hands(w, m) {
        mx::msg(w, m, 1004031); // You must have your hands free to attempt to stun your opponent.
        set_stun_ready(w, m, false);
    } else if anat >= 80.0 && wres >= 80.0 {
        mb::disruptive_action(w, m);
        let v = !w.mobile(m).is_some_and(|m| m.stun_ready);
        set_stun_ready(w, m, v);
        mx::msg(w, m, if v { 1019011 } else { 1019012 });
    } else {
        mx::msg(w, m, 1004008); // You are not skilled enough to stun your opponent.
        set_stun_ready(w, m, false);
    }
}

// ---------------------------------------------------------------------------------------------------------------
// Pickaxe
// ---------------------------------------------------------------------------------------------------------------

/// `Pickaxe.OnCraft(..)`: the base crafting, then more uses for a better pickaxe.
pub(super) fn pickaxe_on_craft(w: &mut World, id: ItemId, a: &CraftArgs) -> i32 {
    let result = weapon_bases::on_craft_base(w, id, a);
    let quality = weapon::data(w, id).map_or(WeaponQuality::Regular, |d| d.quality);
    let uor = core_uor(w);
    if let Some(d) = weapon::data_mut(w, id) {
        if uor {
            if quality == WeaponQuality::Exceptional {
                d.uses_remaining += 50;
            }
        } else if quality != WeaponQuality::Regular {
            d.uses_remaining +=
                (f64::from(d.uses_remaining) * f64::from(quality as i32 - 1) * 0.2) as i32;
        }
    }
    ops::invalidate_properties(w, id); // UsesRemaining is [InvalidateProperties]
    result
}

/// Registers the hand-written classes.
pub fn register(types: &mut TypeRegistry) {
    types.register_item(&THROWING_DAGGER, &[]);
    types.register_item(&FIREWORKS_WAND, &[]);
    types.register_item(&FISTS, &[]);
    types.register_phase_default(
        Phase::Initialize,
        "Server.Items.Fists.Initialize",
        fists_initialize,
    );
}
