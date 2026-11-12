//! The Exorcism spell.

use muo_content::core;
use muo_content::spells::helper::is_any_t2a;
use muo_content::spells::hooks::hooks;
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_content::spells::{SpellInfo, reagent};
use muo_primitives::{Point3D, TimeSpan};
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::map::MapId;
use muo_server::mobile::base as mb;
use muo_server::mobile::part1::{lifecycle, props_b};
use muo_server::mobile::part2::messages as mm;
use muo_server::region as regions;
use muo_server::skills::{self, SkillName};

use super::necromancer::{NECRO_BASE, NECROMANCER_SPELL, NecroType};
use crate::common::{distance_sqrt_to, mobiles_in_range};

static EXORCISM: NecroType = NecroType {
    required_skill: 80.0,
    required_mana: 40,
    ..NECRO_BASE
};

/// `ExorcismSpell`: Ort Corp Grav.
pub static EXORCISM_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Necromancy.ExorcismSpell",
    parent: Some(&NECROMANCER_SPELL),
    info: SpellInfo::new(
        "Exorcism",
        "Ort Corp Grav",
        &[reagent::NOX_CRYSTAL, reagent::GRAVE_DUST],
    )
    .action(203)
    .hand_effect(9031),
    cast_delay_base: |_, _| TimeSpan::from_secs(2),
    delayed_damage: false,
    on_cast,
    check_cast,
    compute_karma_award: |_, _| 0,
    family: Some(&EXORCISM),
    ..NECROMANCER_SPELL
};

const BRITANNIA_LOCS: [Point3D; 9] = [
    Point3D::new(1470, 843, 0),
    Point3D::new(1857, 865, -1),
    Point3D::new(4220, 563, 36),
    Point3D::new(1732, 3528, 0),
    Point3D::new(1300, 644, 8),
    Point3D::new(3355, 302, 9),
    Point3D::new(1606, 2490, 5),
    Point3D::new(2500, 3931, 3),
    Point3D::new(4264, 3707, 0),
];

const ILLSH_LOCS: [Point3D; 7] = [
    Point3D::new(1222, 474, -17),
    Point3D::new(718, 1360, -60),
    Point3D::new(297, 1014, -19),
    Point3D::new(986, 1006, -36),
    Point3D::new(1180, 1288, -30),
    Point3D::new(1538, 1341, -3),
    Point3D::new(528, 223, -38),
];

const MALAS_LOCS: [Point3D; 1] = [Point3D::new(976, 517, -30)];

const TOKUNO_LOCS: [Point3D; 3] = [
    Point3D::new(710, 1162, 25),
    Point3D::new(1034, 515, 18),
    Point3D::new(295, 712, 55),
];

fn range(w: &World) -> i32 {
    if core::ml(w) { 48 } else { 18 }
}

fn check_cast(w: &mut World, sp: SpellId) -> bool {
    let Some(caster) = spell::caster(w, sp) else {
        return false;
    };
    if skills::value(w, caster, SkillName::SpiritSpeak) < 100.0 {
        mm::send_localized_message(w, caster, 1072112, "", 0x3B2); // You must have GM Spirit Speak to use this spell
        return false;
    }

    (NECROMANCER_SPELL.check_cast)(w, sp)
}

fn on_cast(w: &mut World, sp: SpellId) {
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };
    // `Caster.Region.GetRegion<ChampionSpawnRegion>()` and `r.Spawn`: the spawn's location and map.
    let spawn = hooks(w)
        .and_then(|h| h.champion_spawn_of)
        .and_then(|f| f(w, caster));
    let in_range = spawn.is_some_and(|(loc, _)| {
        w.mobile(caster).map(|c| c.location).is_some()
            && mb::in_range_point3d(w, caster, loc, range(w))
    });

    let Some((spawn_loc, spawn_map)) = spawn.filter(|_| in_range) else {
        mm::send_localized_message(w, caster, 1072111, "", 0x3B2); // You are not in a valid exorcism region.
        spell::finish_sequence(w, sp);
        return;
    };

    if spell::check_sequence(w, sp) && w.mobile(caster).and_then(|c| c.map).is_some() {
        // Cannot move a mobile while iterating mobiles in range, so use a queue
        let mut queue = Vec::new();
        for m in mobiles_in_range(w, Some(spawn_map), spawn_loc, range(w)) {
            if is_valid_target(w, caster, m) {
                queue.push(m);
            }
        }

        for m in queue {
            // Surprisingly, no sparkle type effects
            let p = get_nearest_shrine(w, m);
            lifecycle::set_location(w, m, p);
        }
    }

    spell::finish_sequence(w, sp);
}

fn is_valid_target(w: &mut World, caster: MobileId, m: MobileId) -> bool {
    let (player, is_alive, mloc, map, corpse) = match w.mobile(m) {
        Some(x) => (x.player, (x.ty.alive)(w, m), x.location, x.map, x.corpse),
        None => return false,
    };
    if !player || is_alive {
        return false;
    }

    if let (Some(c), Some(map)) = (corpse, map) {
        let c_deleted = muo_server::item::ops::is_deleted(w, c);
        let (cloc, cmap) = w.item(c).map(|i| (i.location, i.map)).unwrap_or_default();
        if !c_deleted && cmap == Some(map) {
            if is_any_t2a(map, cloc) && is_any_t2a(map, mloc) {
                return false; // Same Map, both in T2A, ie, same 'sub server'.
            }

            let mr = props_b::region(w, m);
            let m_dungeon = muo_content::regions::is_part_of_dungeon(w, mr);
            let cr = regions::find(w, cloc, Some(map));
            let c_dungeon = muo_content::regions::is_part_of_dungeon(w, cr);
            if m_dungeon == c_dungeon {
                return false; // Same Map, both in Dungeon region OR They're both NOT in a dungeon region.
            }

            // Just an approximation cause RunUO doesn't divide up the world the same way OSI does ;p
        }
    }

    if hooks(w)
        .and_then(|h| h.party_contains)
        .is_some_and(|f| f(w, m, caster))
    {
        return false;
    }

    if let Some(f) = hooks(w).and_then(|h| h.guild_of) {
        let (mg, cg) = (f(w, m), f(w, caster));
        if let (Some(mg), Some(cg)) = (mg, cg) {
            let ally = hooks(w)
                .and_then(|h| h.guild_is_ally)
                .is_some_and(|a| a(w, mg, cg));
            if ally || mg == cg {
                return false;
            }
        }
    }

    // `var f = Faction.Find(m); return m.Map != Faction.Facet || f == null || f != Faction.Find(Caster);`
    !hooks(w)
        .and_then(|h| h.faction_same_as_caster)
        .is_some_and(|f| f(w, m, caster))
}

fn get_nearest_shrine(w: &World, m: MobileId) -> Point3D {
    let map = w.mobile(m).and_then(|x| x.map);
    let loc_list: &[Point3D] = if map == Some(MapId::FELUCCA) || map == Some(MapId::TRAMMEL) {
        &BRITANNIA_LOCS
    } else if map == Some(MapId::ILSHENAR) {
        &ILLSH_LOCS
    } else if map == Some(MapId::TOKUNO) {
        &TOKUNO_LOCS
    } else if map == Some(MapId::MALAS) {
        &MALAS_LOCS
    } else {
        &[]
    };

    let mut closest = Point3D::ZERO;
    let mut min_dist = f64::MAX;

    for &p in loc_list {
        let dist = distance_sqrt_to(w, m, p);
        if min_dist > dist {
            closest = p;
            min_dist = dist;
        }
    }

    closest
}
