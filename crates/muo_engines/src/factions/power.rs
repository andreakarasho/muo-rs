//! The power faction items: `PowerFactionItem` (the drop on a kill, the use rules and the
//! punishment for a non-member), `GemOfEmpowerment`, `BloodRose` and `ClarityPotion`; and the `StrongholdRegion`.
//!
//! `StormsEye` and `UrnOfAscension` (their `Use`) are at the end of this file.

use muo_content::spells::helper as spell_helper;
use muo_io::{GenericReader, GenericWriter};
use muo_primitives::{Direction, MessageType, Point3D, TimeSpan, utility};
use muo_server::World;
use muo_server::effects;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, base, ops};
use muo_server::map::MapId;
use muo_server::mobile::AccessLevel;
use muo_server::mobile::StatType;
use muo_server::mobile::part2::{effects as fx, messages as mm};
use muo_server::mods::StatMod;
use muo_server::region::RegionId;
use muo_server::region::{self as regions, REGION_BASE, RegionType};
use muo_server::target::{self, TargetFlags};
use muo_server::timer;
use muo_server::type_registry::TypeRegistry;

use muo_content::hooks::rnd;
use muo_content::regions::BASE_REGION;

use super::tables::{FACTION_DEFS, STRONGHOLD_AREAS};
use super::{FACET, find, find_full, player_state, st_mut};

// ---- StrongholdRegion ----

/// `StrongholdRegion.Faction`.
pub struct StrongholdData {
    pub faction: usize,
}

/// `StrongholdRegion.OnMoveInto`.
fn stronghold_move_into(
    w: &mut World,
    id: RegionId,
    m: MobileId,
    d: Direction,
    new: Point3D,
    old: Point3D,
) -> bool {
    if !(REGION_BASE.on_move_into)(w, id, m, d, new, old) {
        return false;
    }
    let access = w.mobile(m).map_or(AccessLevel::Player, |x| x.access_level);
    if access >= AccessLevel::Counselor || regions::contains(w, id, old) {
        return true;
    }
    if crate::conpvp::duel_context_of(w, m).is_some() {
        mm::send_message(
            w,
            m,
            "You may not enter this area while participating in a duel or a tournament.",
        );
        return false;
    }
    find_full(w, m, true, true).is_some()
}

pub static STRONGHOLD_REGION: RegionType = RegionType {
    name: "StrongholdRegion",
    parent: Some(&BASE_REGION),
    on_move_into: stronghold_move_into,
    allow_housing: |_, _, _, _| false,
    ..BASE_REGION
};

/// `region.GetRegion<StrongholdRegion>()?.Faction`.
pub fn stronghold_region_faction(w: &World, region: RegionId) -> Option<usize> {
    let r = regions::get_region_of_type(w, region, &STRONGHOLD_REGION)?;
    w.region(r)?.ext.get::<StrongholdData>().map(|d| d.faction)
}

/// `Faction.Definition` setter: `new StrongholdRegion(this)` and `Register()`.
pub fn create_stronghold_regions(w: &mut World) {
    for f in 0..FACTION_DEFS.len() {
        let area: Vec<_> = STRONGHOLD_AREAS[f]
            .iter()
            .map(|&(x, y, width, height)| muo_primitives::Rectangle2D::new(x, y, width, height))
            .collect();
        let id = regions::new_region(
            w,
            &STRONGHOLD_REGION,
            Some(FACTION_DEFS[f].friendly_name),
            FACET,
            None,
            Some(regions::DEFAULT_PRIORITY),
            regions::convert_all_to_3d(&area),
        );
        if let Some(r) = w.region_mut(id) {
            let (x, y, z) = FACTION_DEFS[f].stronghold.faction_stone;
            r.go_location = Point3D::new(x, y, z);
            r.ext.insert(StrongholdData { faction: f });
        }
        regions::register(w, id);
        st_mut(w).stronghold_regions[f] = Some(id);
    }
}

// ---- PowerFactionItem ----

const ITEMS: [(i32, &str); 5] = [
    (30, "Server.GemOfEmpowerment"),
    (25, "Server.BloodRose"),
    (20, "Server.ClarityPotion"),
    (15, "Server.UrnOfAscension"),
    (10, "Server.StormsEye"),
];
const TOTAL_WEIGHT: i32 = 30 + 25 + 20 + 15 + 10;

/// `PowerFactionItem.CheckSpawn(killer, victim)`.
pub fn check_spawn(w: &mut World, killer: MobileId, victim: MobileId) {
    let Some((f, ps)) = player_state(w, victim) else {
        return;
    };
    let chance = FACTION_DEFS[f].ranks[ps.rank].rank;
    if chance <= rnd::below(w, 100) {
        return;
    }
    let mut weight = rnd::below(w, TOTAL_WEIGHT);
    for (item_weight, name) in ITEMS {
        if weight < item_weight {
            if let Some(obj) = muo_content::factory::make_item(w, name) {
                muo_server::mobile::part2::items::add_to_backpack(w, killer, obj);
                if let Some(ns) = w.mobile(killer).and_then(|m| m.net_state) {
                    let _ = ns;
                    fx::send_sound(w, killer, 1470);
                }
                mm::local_overhead_message(
                    w,
                    killer,
                    MessageType::REGULAR,
                    2119,
                    false,
                    "You notice a strange item on the corpse, and decide to pick it up.",
                );
                // `faction-power-items.log`
                use std::io::Write;
                if let Ok(mut f) = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open("faction-power-items.log")
                {
                    let _ = writeln!(f, "{}\t{:?}\t{:?}\t{:?}", w.now, killer, victim, obj);
                }
            }
            break;
        }
        weight -= item_weight;
    }
}

fn power_on_double_click(w: &mut World, id: ItemId, from: MobileId) {
    let pack = w.mobile(from).and_then(|m| m.backpack);
    if !ops::is_child_of(w, id, pack.map(EntityId::Item)) {
        mm::send_localized_message(w, from, 1042038, "", 0x3B2); // You must have the object in your backpack to use it.
    } else if muo_content::player_mobile::data(w, from).is_some()
        && crate::conpvp::duel_context_of(w, from).is_some()
    {
        mm::send_message(w, from, "You can't use that.");
    } else if find(w, from).is_none() {
        mm::local_overhead_message(
            w,
            from,
            MessageType::REGULAR,
            2119,
            false,
            "The object vanishes from your hands as you touch it.",
        );
        timer::start(w, TimeSpan::from_secs(1), TimeSpan::ZERO, 1, move |w| {
            mm::local_overhead_message(
                w,
                from,
                MessageType::REGULAR,
                2118,
                false,
                "You feel a strange tingling sensation throughout your body.",
            );
        });
        timer::start(w, TimeSpan::from_secs(4), TimeSpan::ZERO, 1, move |w| {
            mm::local_overhead_message(
                w,
                from,
                MessageType::REGULAR,
                2118,
                false,
                "Your skin begins to burn.",
            );
        });
        destruction_timer(w, from);
        if let Some(t) = w.item(id).map(|i| i.ty) {
            (t.delete)(w, id);
        }
    } else if power_use(w, id, from) {
        muo_server::mobile::base::revealing_action(w, from);
        base::consume(w, id, 1);
    }
}

/// `DestructionTimer`: after 5 seconds, ten ticks 0.1 seconds apart.
fn destruction_timer(w: &mut World, mob: MobileId) {
    let screamed = std::rc::Rc::new(std::cell::Cell::new(false));
    timer::start(
        w,
        TimeSpan::from_secs(5),
        TimeSpan::from_millis(100),
        10,
        move |w| {
            if !muo_server::mobile::base::alive(w, mob) {
                return;
            }
            if !screamed.get() {
                screamed.set(true);
                let female = w.mobile(mob).is_some_and(|m| m.female);
                fx::play_sound(w, mob, if female { 814 } else { 1088 });
                mm::public_overhead_message(
                    w,
                    mob,
                    MessageType::REGULAR,
                    2118,
                    false,
                    "Aaaaah!",
                    true,
                    AccessLevel::Player,
                );
            }
            let amount = utility::dice(&mut w.rng, 2, 6, 0);
            muo_server::mobile::part2::damage::damage(w, mob, amount, None, true, false);
        },
    );
}

/// The item's `Use(from)`, by class.
fn power_use(w: &mut World, id: ItemId, from: MobileId) -> bool {
    let name = w.item(id).map_or("", |i| i.ty.full_name);
    match name {
        "Server.GemOfEmpowerment" => {
            if super::clear_skill_loss(w, from) {
                mm::local_overhead_message(
                    w,
                    from,
                    MessageType::REGULAR,
                    2219,
                    false,
                    "The gem shatters as you invoke its power.",
                );
                fx::play_sound(w, from, 909);
                fx::fixed_effect(w, from, 0x373A, 10, 30);
                fx::play_sound(w, from, 0x209);
                return true;
            }
            false
        }
        "Server.BloodRose" => {
            if muo_server::mobile::base::get_stat_mod(w, from, "blood-rose", false).is_none() {
                let s = 0x3A + rnd::below(w, 3);
                fx::play_sound(w, from, s);
                let human = muo_content::creature::misc::body_is_human(w, from);
                if human && !muo_server::mobile::part1::props_b::mounted(w, from) {
                    muo_server::mobile::base::animate(w, from, 34, 5, 1, true, false, 0);
                }
                let amount = utility::dice(&mut w.rng, 3, 3, 3);
                let time = rnd::min_max(w, 5, 30);
                fx::fixed_particles(
                    w,
                    from,
                    0x373A,
                    10,
                    15,
                    5018,
                    muo_server::effects::EffectLayer::Waist,
                );
                fx::play_sound(w, from, 0x1EE);
                let m = StatMod::new(
                    w,
                    StatType::ALL,
                    Some("blood-rose"),
                    amount,
                    TimeSpan::from_mins(time as i64),
                    None,
                );
                muo_server::mobile::base::add_stat_mod(w, from, m);
                return true;
            }
            mm::send_localized_message(w, from, 1062927, "", 0x3B2); // You have eaten one of these recently and eating another would provide no benefit.
            false
        }
        "Server.ClarityPotion" => {
            if !muo_server::mobile::part1::combat::begin_action(w, from, "Server.ClarityPotion") {
                return false;
            }
            let amount = utility::dice(&mut w.rng, 3, 3, 3);
            let time = rnd::min_max(w, 5, 30);
            fx::play_sound(w, from, 0x2D6);
            if muo_content::creature::misc::body_is_human(w, from) {
                muo_server::mobile::base::animate(w, from, 34, 5, 1, true, false, 0);
            }
            fx::fixed_particles(
                w,
                from,
                0x375A,
                10,
                15,
                5011,
                muo_server::effects::EffectLayer::Head,
            );
            fx::play_sound(w, from, 0x1EB);
            if let Some(offset) =
                muo_server::mobile::base::get_stat_mod(w, from, "Concussion", false)
                    .and_then(|i| w.mobile(from).map(|m| m.stat_mods[i].offset))
            {
                muo_server::mobile::base::remove_stat_mod_by_name(w, from, "Concussion");
                let mana = w.mobile(from).map_or(0, |m| m.mana);
                muo_server::mobile::part1::props_b::set_mana(w, from, mana - offset);
            }
            fx::play_sound(w, from, 0x1EE);
            let m = StatMod::new(
                w,
                StatType::INT,
                Some("clarity-potion"),
                amount,
                TimeSpan::from_mins(time as i64),
                None,
            );
            muo_server::mobile::base::add_stat_mod(w, from, m);
            timer::start(
                w,
                TimeSpan::from_mins(time as i64),
                TimeSpan::ZERO,
                1,
                move |w| {
                    muo_server::mobile::part1::combat::end_action(w, from, "Server.ClarityPotion");
                },
            );
            true
        }
        "Server.StormsEye" => storms_eye_use(w, id, from),
        "Server.UrnOfAscension" => urn_use(w, from),
        _ => false,
    }
}

/// `StormsEye.Use(user)`: target a spot, throw the eye there, then `OnDelay` half a second later.
fn storms_eye_use(w: &mut World, eye: ItemId, user: MobileId) -> bool {
    if !w.item(eye).is_some_and(|i| i.movable()) {
        return false;
    }
    target::begin_target(w, user, 12, true, TargetFlags::NONE, move |w, from, obj| {
        let movable = w.item(eye).is_some_and(|i| i.movable() && !i.deleted());
        if !movable {
            return;
        }
        let pt = spell_helper::get_surface_top(w, spell_helper::Point3dRef::from(obj));
        let origin = spell_helper::point_of(w, pt);
        let facet = w.mobile(from).and_then(|m| m.map);
        let Some(map) = facet else { return };
        if !muo_server::map::fit::can_fit(
            w, map, origin.x, origin.y, origin.z, 16, false, false, true,
        ) {
            return;
        }
        ops::set_movable(w, eye, false);
        let (item_id, hue) = w.item(eye).map_or((0, 0), |i| (i.item_id, i.hue));
        effects::send_moving_effect_to_point(
            w,
            from,
            origin,
            item_id & 0x3FFF,
            7,
            0,
            false,
            false,
            hue - 1,
            0,
        );
        timer::start(w, TimeSpan::from_millis(500), TimeSpan::ZERO, 1, move |w| {
            storms_eye_on_delay(w, from, eye, origin, map)
        });
    });
    false
}

/// `StormsEye.OnDelay`.
fn storms_eye_on_delay(w: &mut World, from: MobileId, eye: ItemId, origin: Point3D, facet: MapId) {
    if let Some(t) = w.item(eye).map(|i| i.ty) {
        (t.delete)(w, eye);
    }
    effects::play_sound(w, origin, Some(facet), 530);
    effects::play_sound(w, origin, Some(facet), 263);
    effects::send_location_effect(w, origin, Some(facet), 14284, 96, 1, 0, 2);
    timer::start(w, TimeSpan::from_secs(1), TimeSpan::ZERO, 1, move |w| {
        storms_eye_on_hit(w, from, origin, facet)
    });
}

/// `StormsEye.OnHit`. It enqueues `from` (not the mobile it tested) for every faction member in range that
/// `from` could harm, so the caster is the one struck, once per such mobile; that stays.
fn storms_eye_on_hit(w: &mut World, from: MobileId, origin: Point3D, facet: MapId) {
    let spot = Point3D::new(origin.x, origin.y, origin.z + 1);
    let mut queue = Vec::new();
    let near: Vec<MobileId> =
        muo_server::map::enumerate::mobiles_in_range(w, Some(facet), origin.x, origin.y, 12)
            .collect();
    for m in near {
        if muo_server::mobile::base::can_be_harmful_msg(w, from, m, false)
            && muo_server::mobile::part1::combat::in_los_point(w, m, spot)
            && find(w, m).is_some()
        {
            queue.push(from);
        }
    }
    for mob in queue {
        let mut damage = muo_content::mx::hits(w, mob) * 6 / 10;
        if !muo_content::mx::player(w, mob) && damage < 10 {
            damage = 10;
        } else if damage > 75 {
            damage = 75;
        }
        let above =
            effects::Targeted::point(Point3D::new(origin.x, origin.y, origin.z + 4), Some(facet));
        effects::send_moving_effect(w, above, mob, 14068, 1, 32, false, false, 1111, 2);
        muo_server::mobile::base::do_harmful(w, from, mob, false);
        for delay in [500, 700, 1000] {
            spell_helper::damage_aos(
                w,
                None,
                TimeSpan::from_millis(delay),
                mob,
                Some(from),
                f64::from(damage) / 3.0,
                spell_helper::SpellDamageSplit {
                    nrgy: 100,
                    ..Default::default()
                },
            );
        }
        timer::start(w, TimeSpan::from_millis(500), TimeSpan::ZERO, 1, move |w| {
            fx::play_sound(w, mob, 0x1FB)
        });
    }
}

/// `UrnOfAscension.Use(from)`: resurrects the dead faction mates in sight (house friends only inside a house).
fn urn_use(w: &mut World, from: MobileId) -> bool {
    let ours = find(w, from);
    let mut used = false;
    for mob in muo_server::mobile::part2::enumerators::get_mobiles_in_range(w, from, 8) {
        if muo_content::mx::player(w, mob)
            && !muo_server::mobile::base::alive(w, mob)
            && muo_server::mobile::part1::combat::in_los(w, from, mob)
        {
            if find(w, mob) != ours {
                continue;
            }
            let house = muo_multis::house::access::find_house_at_mobile(w, mob);
            // `house?.IsFriend(from) != false || house.IsFriend(mob)`
            if house.is_none_or(|h| {
                muo_multis::house::access::is_friend(w, h, from)
                    || muo_multis::house::access::is_friend(w, h, mob)
            }) {
                super::clear_skill_loss(w, mob);
                let g = muo_gumps::m_z::resurrect::new(
                    Some(from),
                    muo_gumps::m_z::resurrect::ResurrectMessage::Generic,
                    false,
                    0.0,
                    0,
                );
                muo_content::gumps::system::send_gump_to_mobile(w, mob, Box::new(g), false);
                used = true;
            }
        }
    }
    if used {
        mm::local_overhead_message(
            w,
            from,
            MessageType::REGULAR,
            2219,
            false,
            "The urn shatters as you invoke its power.",
        );
        fx::play_sound(w, from, 64);
        if let Some(m) = w.mobile(from) {
            let (loc, map) = (m.location, m.map);
            effects::play_sound(w, loc, map, 1481);
        }
    }
    used
}

#[cfg(test)]
pub(crate) fn power_use_for_test(w: &mut World, id: ItemId, from: MobileId) -> bool {
    power_use(w, id, from)
}

macro_rules! power_item {
    ($name:ident, $full:literal, $ctor:ident, $item_id:expr, $hue:expr, $default_name:literal) => {
        fn $ctor(w: &mut World) -> ItemId {
            let id = ops::new_item_with_id(w, &$name, $item_id);
            let hue: Option<i32> = $hue;
            if let Some(h) = hue {
                base::set_hue(w, id, h);
            }
            id
        }
        pub static $name: ItemType = ItemType {
            full_name: $full,
            parent: Some(&ITEM_BASE),
            construct: None, // no parameterless `[Constructible]`
            make: Some($ctor),
            default_name: |_, _| Some($default_name.to_owned()),
            on_double_click: power_on_double_click,
            serialize: |w, id, out: &mut dyn GenericWriter| {
                (ITEM_BASE.serialize)(w, id, out);
                out.write_i32(0); // PowerFactionItem
                out.write_i32(0); // the class
            },
            deserialize: |w, id, r: &mut dyn GenericReader| {
                (ITEM_BASE.deserialize)(w, id, r)?;
                let _ = r.read_i32()?;
                let _ = r.read_i32()?;
                Ok(())
            },
            ..ITEM_BASE
        };
    };
}

power_item!(
    GEM_OF_EMPOWERMENT,
    "Server.GemOfEmpowerment",
    gem_new,
    7955,
    Some(1154),
    "gem of empowerment"
);
power_item!(
    CLARITY_POTION,
    "Server.ClarityPotion",
    clarity_new,
    3628,
    Some(1154),
    "clarity potion"
);
power_item!(
    STORMS_EYE,
    "Server.StormsEye",
    storms_new,
    3967,
    Some(1165),
    "storms eye"
);
power_item!(
    URN_OF_ASCENSION,
    "Server.UrnOfAscension",
    urn_new,
    9246,
    None,
    "urn of ascension"
);

fn rose_new(w: &mut World) -> ItemId {
    let item_id = if rnd::bool(w) { 6378 } else { 9035 };
    let id = ops::new_item_with_id(w, &BLOOD_ROSE, item_id);
    base::set_hue(w, id, 2118);
    id
}

pub static BLOOD_ROSE: ItemType = ItemType {
    full_name: "Server.BloodRose",
    construct: None,
    make: Some(rose_new),
    default_name: |_, _| Some("blood rose".to_owned()),
    ..GEM_OF_EMPOWERMENT
};

pub fn register(types: &mut TypeRegistry) {
    for t in [
        &GEM_OF_EMPOWERMENT,
        &BLOOD_ROSE,
        &CLARITY_POTION,
        &STORMS_EYE,
        &URN_OF_ASCENSION,
    ] {
        types.register_item(t, &[]);
    }
}
