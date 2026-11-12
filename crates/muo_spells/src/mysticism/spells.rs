//! The Animated Weapon, Bombard, Cleansing Winds, Eagle Strike, Hail Storm, Nether Cyclone and Stone Form spells.
// The `AnimatedWeapon` creature is `spellweaving/mobiles.rs`; the spell gives it its caster and level through
// `SpellHooks::init_animated_weapon`.

use indexmap::IndexMap;
use muo_content::core;
use muo_content::creature::control;
use muo_content::factory;
use muo_content::items::weapon_ability;
use muo_content::player_mobile::buff::{BuffIcon, BuffInfo, add_buff, remove_buff};
use muo_content::spells::helper::{self, Point3dRef, SpellDamageSplit};
use muo_content::spells::hooks::hooks;
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_content::spells::targeting::{SpellTargetKind, begin_spell_target};
use muo_content::spells::{SpellCircle, SpellInfo, reagent};
use muo_io::text::TextDefinition;
use muo_primitives::{Body, Point3D, TimeSpan, utility};
use muo_server::World;
use muo_server::effects::{EffectLayer, Targeted as FxTargeted, send_moving_particles_layer};
use muo_server::ids::{EntityId, MobileId};
use muo_server::item::deps;
use muo_server::mobile::part1::{combat, props_b};
use muo_server::mobile::part2::{effects, messages as mm};
use muo_server::mobile::{MobileDelta, ResistanceType, base as mb};
use muo_server::mods::{ModId, ResistanceMod};
use muo_server::skills::{self, SkillName};
use muo_server::target::{TargetFlags, Targeted};
use muo_server::timer;

use super::mystic::{MYSTIC_SPELL, MysticType, check_resisted, get_base_skill, mystic};
use super::spell_plague;
use crate::common::{is_player_mobile, mobile_of, mobiles_in_range, surface_top, turn_to};

// ---- AnimatedWeaponSpell ----

static ANIMATED_WEAPON: MysticType = mystic(SpellCircle::Fourth);

/// `AnimatedWeaponSpell`: In Jux Por Ylem.
pub static ANIMATED_WEAPON_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Mysticism.AnimatedWeaponSpell",
    parent: Some(&MYSTIC_SPELL),
    target_range: Some(muo_content::spells::targeting::target_range),
    info: SpellInfo::new(
        "Animated Weapon",
        "In Jux Por Ylem",
        &[
            reagent::BONE,
            reagent::BLACK_PEARL,
            reagent::MANDRAKE_ROOT,
            reagent::NIGHTSHADE,
        ],
    )
    .action(-1)
    .hand_effect(9002),
    on_cast: |w, sp| begin_point(w, sp, animated_weapon_target),
    family: Some(&ANIMATED_WEAPON),
    ..MYSTIC_SPELL
};

fn begin_point(w: &mut World, sp: SpellId, f: fn(&mut World, SpellId, Targeted)) {
    begin_spell_target(
        w,
        sp,
        true,
        TargetFlags::NONE,
        false,
        SpellTargetKind::Point,
        f,
    );
}

fn animated_weapon_target(w: &mut World, sp: SpellId, targeted: Targeted) {
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };
    let (followers, max) = w
        .mobile(caster)
        .map_or((0, 0), |c| (c.followers, c.followers_max));
    if followers + 4 > max {
        mm::send_localized_message(w, caster, 1049645, "", 0x3B2); // You have too many followers to summon that creature.
        return;
    }

    let map = w.mobile(caster).and_then(|c| c.map);
    let p = surface_top(w, targeted);

    let blocked = map.is_none()
        || (w.mobile(caster).is_some_and(|c| c.player)
            && !map
                .is_some_and(|m| muo_server::map::fit::can_spawn_mobile_at(w, m, p.x, p.y, p.z)));
    if blocked {
        mm::send_localized_message(w, caster, 501942, "", 0x3B2); // That location is blocked.
    } else if helper::check_town(w, Point3dRef::Point(p), caster) && spell::check_sequence(w, sp) {
        let ty = spell::spell_type(w, sp).expect("the spell is casting");
        let level =
            ((get_base_skill(w, caster) + (ty.get_damage_skill)(w, sp, caster)) / 2.0) as i32;
        let duration = TimeSpan::from_secs(i64::from(10 + level));

        if let Some(summon) = factory::make_mobile(w, "Server.Mobiles.AnimatedWeapon") {
            if let Some(f) = hooks(w).and_then(|h| h.init_animated_weapon) {
                f(w, summon, caster, level);
            }
            control::summon(w, summon, false, caster, p, 0x212, duration, None);
            effects::play_sound(w, summon, 0x64A);
            muo_server::effects::send_target_particles_full(
                w,
                summon,
                0x3728,
                10,
                10,
                0,
                0,
                0x13AA,
                EffectLayer(255),
                0,
            );
        }
    }
}

// ---- BombardSpell ----

static BOMBARD: MysticType = mystic(SpellCircle::Sixth);

/// `BombardSpell`: Corp Por Ylem.
pub static BOMBARD_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Mysticism.BombardSpell",
    parent: Some(&MYSTIC_SPELL),
    target_range: Some(muo_content::spells::targeting::target_range),
    info: SpellInfo::new(
        "Bombard",
        "Corp Por Ylem",
        &[
            reagent::BLOODMOSS,
            reagent::GARLIC,
            reagent::SULFUROUS_ASH,
            reagent::DRAGONS_BLOOD,
        ],
    )
    .action(230)
    .hand_effect(9022),
    delayed_damage: true,
    // AOSNoDelayedDamageStackingSelf
    delayed_damage_family: Some(&[]),
    on_cast: |w, sp| begin_mobile(w, sp, TargetFlags::HARMFUL, bombard_target),
    family: Some(&BOMBARD),
    ..MYSTIC_SPELL
};

fn begin_mobile(
    w: &mut World,
    sp: SpellId,
    flags: TargetFlags,
    f: fn(&mut World, SpellId, Targeted),
) {
    begin_spell_target(w, sp, false, flags, false, SpellTargetKind::Mobile, f);
}

fn bombard_target(w: &mut World, sp: SpellId, targeted: Targeted) {
    let Some(mut m) = mobile_of(targeted) else {
        return;
    };
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };
    if spell::check_h_sequence(w, sp, m) {
        turn_to(w, caster, m);

        if spell::has_delayed_damage_context(w, sp, m) {
            let ty = spell::spell_type(w, sp).expect("the spell is casting");
            (ty.do_hurt_fizzle)(w, sp);
            return;
        }

        let mut source = caster;
        if helper::check_reflect(w, 6, &mut source, &mut m) {
            let (src, tgt) = (source, m);
            timer::start(w, TimeSpan::from_millis(500), TimeSpan::ZERO, 1, move |w| {
                effects::moving_effect(
                    w,
                    src,
                    EntityId::Mobile(tgt),
                    0x1363,
                    12,
                    1,
                    false,
                    true,
                    0,
                    0,
                );
                effects::play_sound(w, src, 0x64B);
            });
        }

        effects::moving_effect(
            w,
            caster,
            EntityId::Mobile(m),
            0x1363,
            12,
            1,
            false,
            true,
            0,
            0,
        );
        effects::play_sound(w, caster, 0x64B);

        let d = f64::from(spell::get_new_aos_damage(w, sp, 40, 1, 5, true, Some(m)));
        helper::damage_aos_spell(
            w,
            sp,
            m,
            d,
            SpellDamageSplit {
                phys: 100,
                ..Default::default()
            },
        );

        let target = m;
        timer::start(
            w,
            TimeSpan::from_millis(1200),
            TimeSpan::ZERO,
            1,
            move |w| {
                if check_resisted(w, sp, target) {
                    return;
                }

                let ty = spell::spell_type(w, sp).expect("the spell exists");
                let damage_skill = (ty.get_damage_skill)(w, sp, caster);
                let resist = (ty.get_resist_skill)(w, sp, target);

                let secs = 0.max((damage_skill / 10.0 - resist / 10.0) as i32);

                if secs > 0 {
                    mb::paralyze(w, target, TimeSpan::from_secs(i64::from(secs)));
                }

                // Up to 12% chance by checking mysticism + imbuing/focus against resist
                let knock_back_chance = (get_base_skill(w, caster) + damage_skill - resist) / 20.0;
                if knock_back_chance > 0.0 && utility::random_double(&mut w.rng) < knock_back_chance
                {
                    let loc = w.mobile(target).map(|t| t.location).unwrap_or_default();
                    let dir = effects::get_direction_to_point(
                        w,
                        caster,
                        Some(muo_primitives::Point2D { x: loc.x, y: loc.y }),
                        false,
                    );
                    mb::move_dir(w, target, dir);
                }
            },
        );
    }
}

// ---- CleansingWindsSpell ----

static CLEANSING_WINDS: MysticType = mystic(SpellCircle::Sixth);

/// `CleansingWindsSpell`: In Vas Mani Hur.
pub static CLEANSING_WINDS_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Mysticism.CleansingWindsSpell",
    parent: Some(&MYSTIC_SPELL),
    target_range: Some(muo_content::spells::targeting::target_range),
    info: SpellInfo::new(
        "Cleansing Winds",
        "In Vas Mani Hur",
        &[
            reagent::GARLIC,
            reagent::GINSENG,
            reagent::MANDRAKE_ROOT,
            reagent::DRAGONS_BLOOD,
        ],
    )
    .action(230)
    .hand_effect(9022),
    on_cast: |w, sp| begin_mobile(w, sp, TargetFlags::BENEFICIAL, cleansing_winds_target),
    family: Some(&CLEANSING_WINDS),
    ..MYSTIC_SPELL
};

fn cleansing_winds_target(w: &mut World, sp: SpellId, targeted: Targeted) {
    let Some(m) = mobile_of(targeted) else {
        return;
    };
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };
    if spell::check_b_sequence(w, sp, m, false) {
        /* Soothing winds attempt to neutralize poisons, lift curses, and heal a valid target and up to 3 party
         * members.
         */
        effects::play_sound(w, caster, 0x64C);

        let primary_skill = get_base_skill(w, caster);
        let ty = spell::spell_type(w, sp).expect("the spell is casting");
        let secondary_skill = (ty.get_damage_skill)(w, sp, caster);

        let cure_chance = 10000 + ((primary_skill + secondary_skill) / 2.0 * 75.0) as i32;

        let mut pool = vec![m];

        let party_contains = hooks(w).and_then(|h| h.party_contains);
        let in_party = party_contains.is_some();
        if in_party {
            let (mloc, map) = w.mobile(m).map(|x| (x.location, x.map)).unwrap_or_default();
            for mob in mobiles_in_range(w, map, mloc, 2) {
                if mob != m
                    && party_contains.is_some_and(|f| f(w, caster, mob))
                    && mb::can_be_beneficial_msg(w, caster, mob, false)
                {
                    pool.push(mob);

                    if pool.len() == 4 {
                        break;
                    }
                }
            }
        }

        let to_heal = (((primary_skill + secondary_skill) / 4.0) as i32
            + utility::random_min_max(&mut w.rng, -3, 3))
            / pool.len() as i32;

        for target in pool {
            mb::do_beneficial(w, caster, target);

            effects::fixed_particles_hue(w, target, 0x3709, 1, 30, 9963, 13, 3, EffectLayer::Head);

            let (loc, map) = w
                .mobile(target)
                .map(|x| (x.location, x.map))
                .unwrap_or_default();
            let from = FxTargeted::point(Point3D::new(loc.x, loc.y, loc.z - 10), map);
            let to = FxTargeted::point(Point3D::new(loc.x, loc.y, loc.z + 50), map);
            send_moving_particles_layer(
                w,
                from,
                to,
                0x2255,
                1,
                0,
                false,
                false,
                13,
                3,
                9501,
                1,
                0,
                i32::from(EffectLayer::Head.0),
                0x100,
            );

            let mut to_heal_mod = to_heal;

            if w.mobile(target).is_some_and(|x| x.poison.is_some()) {
                let poison_level = increased_poison_level(w, target);
                let chance_to_cure = cure_chance - poison_level * 1750;

                if chance_to_cure > 10000
                    || (chance_to_cure > utility::random(&mut w.rng, 10000)
                        && mb::cure_poison(w, target, Some(caster)))
                {
                    to_heal_mod -= (f64::from(to_heal) * f64::from(poison_level) * 0.15) as i32;
                } else {
                    to_heal_mod = 0;
                }
            }

            if weapon_ability::is_wounded(w, target) {
                to_heal_mod = 0;
            }

            let curse_level = remove_curses(w, target);

            if to_heal_mod > 0 && curse_level > 0 {
                to_heal_mod -= curse_level * 3;
                to_heal_mod -= (f64::from(to_heal_mod) * (f64::from(curse_level) / 100.0)) as i32;
            }

            if to_heal_mod > 0 {
                helper::heal(w, to_heal_mod, target, Some(caster), true);
            }
        }
    }
}

fn increased_poison_level(w: &World, target: MobileId) -> i32 {
    let Some(p) = w.mobile(target).and_then(|x| x.poison) else {
        return 0;
    };
    w.resources
        .get::<muo_server::poison::Poisons>()
        .and_then(|ps| {
            let inc = ps.increase_level(Some(p))?;
            ps.get_poison_by_index(i32::from(inc.0)).map(|x| x.level)
        })
        .unwrap_or(0)
}

/// `RemoveCurses(m)`: the curse level removed.
pub fn remove_curses(w: &mut World, m: MobileId) -> i32 {
    let mut curse_level = 0;

    // if (SleepSpell.EndSleep(m)) { curseLevel += 2; }

    if crate::necromancy::evil_omen::end_effect(w, m) {
        curse_level += 1;
    }

    if crate::necromancy::strangle::remove_curse(w, m) {
        curse_level += 2;
    }

    if crate::necromancy::corpse_skin::remove_curse(w, m) {
        curse_level += 3;
    }

    if crate::fourth::curse::remove_effect(w, m) {
        curse_level += 4;
    }

    if crate::necromancy::blood_oath::remove_curse(w, m) {
        curse_level += 3;
    }

    if crate::necromancy::mind_rot::clear_mind_rot_scalar(w, m) {
        curse_level += 2;
    }

    if spell_plague::remove_effect(w, m) {
        curse_level += 4;
    }

    crate::chivalry::remove_stat_curses(w, m);

    if weapon_ability::end_wound(w, m) {
        curse_level += 2;
    }

    if is_player_mobile(w, m) {
        for icon in [
            BuffIcon::Clumsy,
            BuffIcon::FeebleMind,
            BuffIcon::Weaken,
            BuffIcon::Curse,
            BuffIcon::MassCurse,
            BuffIcon::MortalStrike,
            BuffIcon::CorpseSkin,
            BuffIcon::Strangle,
            BuffIcon::EvilOmen,
        ] {
            remove_buff(w, m, icon);
        }
    }

    curse_level
}

// ---- EagleStrikeSpell ----

static EAGLE_STRIKE: MysticType = mystic(SpellCircle::Third);

/// `EagleStrikeSpell`: Kal Por Xen.
pub static EAGLE_STRIKE_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Mysticism.EagleStrikeSpell",
    parent: Some(&MYSTIC_SPELL),
    target_range: Some(muo_content::spells::targeting::target_range),
    info: SpellInfo::new(
        "Eagle Strike",
        "Kal Por Xen",
        &[
            reagent::BLOODMOSS,
            reagent::BONE,
            reagent::SPIDERS_SILK,
            reagent::MANDRAKE_ROOT,
        ],
    )
    .action(-1)
    .hand_effect(9002),
    on_cast: |w, sp| begin_mobile(w, sp, TargetFlags::HARMFUL, eagle_strike_target),
    family: Some(&EAGLE_STRIKE),
    ..MYSTIC_SPELL
};

fn eagle_strike_target(w: &mut World, sp: SpellId, targeted: Targeted) {
    let Some(mut m) = mobile_of(targeted) else {
        return;
    };
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };
    if spell::check_h_sequence(w, sp, m) {
        turn_to(w, caster, m);

        if core::sa(w) && spell::has_delayed_damage_context(w, sp, m) {
            let ty = spell::spell_type(w, sp).expect("the spell is casting");
            (ty.do_hurt_fizzle)(w, sp);
            return;
        }

        let mut source = caster;
        if helper::check_reflect(w, 2, &mut source, &mut m) {
            let (src, tgt) = (source, m);
            timer::start(w, TimeSpan::from_millis(500), TimeSpan::ZERO, 1, move |w| {
                /* Conjures a magical eagle that assaults the Target with its talons, dealing energy damage. */
                effects::moving_effect(
                    w,
                    src,
                    EntityId::Mobile(tgt),
                    0x407A,
                    8,
                    1,
                    false,
                    true,
                    0,
                    0,
                );
                effects::play_sound(w, src, 0x2EE);
            });
        }

        effects::moving_particles(
            w,
            caster,
            EntityId::Mobile(m),
            0x407A,
            7,
            0,
            false,
            true,
            0xBBE,
            0xFA6,
            0xFFFF,
            0,
        );
        effects::play_sound(w, caster, 0x2EE);

        let to = m;
        timer::start(w, TimeSpan::from_secs(1), TimeSpan::ZERO, 1, move |w| {
            let damage = f64::from(spell::get_new_aos_damage(w, sp, 19, 1, 5, true, Some(to)));
            helper::damage_aos_spell(
                w,
                sp,
                to,
                damage,
                SpellDamageSplit {
                    nrgy: 100,
                    ..Default::default()
                },
            );
            effects::play_sound(w, to, 0x64D);
        });
    }
}

// ---- HailStormSpell and NetherCycloneSpell ----

static HAIL_STORM: MysticType = MysticType {
    circle: SpellCircle::Seventh,
    mana: Some(50),
};

/// `HailStormSpell`: Kal Des Ylem.
pub static HAIL_STORM_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Mysticism.HailStormSpell",
    parent: Some(&MYSTIC_SPELL),
    target_range: Some(muo_content::spells::targeting::target_range),
    info: SpellInfo::new(
        "Hail Storm",
        "Kal Des Ylem",
        &[
            reagent::DRAGONS_BLOOD,
            reagent::BLOODMOSS,
            reagent::BLACK_PEARL,
            reagent::MANDRAKE_ROOT,
        ],
    )
    .action(-1)
    .hand_effect(9002),
    on_cast: |w, sp| begin_point(w, sp, hail_storm_target),
    family: Some(&HAIL_STORM),
    ..MYSTIC_SPELL
};

static NETHER_CYCLONE: MysticType = mystic(SpellCircle::Eighth);

/// `NetherCycloneSpell`: Grav Hur.
pub static NETHER_CYCLONE_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Mysticism.NetherCycloneSpell",
    parent: Some(&MYSTIC_SPELL),
    target_range: Some(muo_content::spells::targeting::target_range),
    info: SpellInfo::new(
        "Nether Cyclone",
        "Grav Hur",
        &[
            reagent::MANDRAKE_ROOT,
            reagent::NIGHTSHADE,
            reagent::SULFUROUS_ASH,
            reagent::BLOODMOSS,
        ],
    )
    .action(-1)
    .hand_effect(9002),
    on_cast: |w, sp| begin_point(w, sp, nether_cyclone_target),
    family: Some(&NETHER_CYCLONE),
    ..MYSTIC_SPELL
};

fn world_loc(w: &World, targeted: Targeted) -> Point3D {
    match targeted {
        Targeted::Entity(EntityId::Item(i)) => muo_server::item::ops::get_world_location(w, i),
        other => helper::point_of(w, Point3dRef::from(other)),
    }
}

fn storm_effect(
    w: &mut World,
    p: Point3D,
    map: Option<muo_server::map::MapId>,
    item_id: i32,
    hue: i32,
) {
    muo_server::effects::play_sound(w, p, map, 0x64F);
    for (a, b, c, d) in [
        (-1, 1, -1, 1),
        (-2, 0, -3, -1),
        (-3, -1, -1, 1),
        (1, 3, -1, 1),
        (-1, 1, 1, 3),
    ] {
        let (x, y, z) = (p.x, p.y, p.z + 18);
        let pts = [
            (Point3D::new(x + a, y + c, z), Point3D::new(x + a, y + c, z)),
            (Point3D::new(x + b, y + c, z), Point3D::new(x + b, y + c, z)),
            (Point3D::new(x + b, y + d, z), Point3D::new(x + b, y + d, z)),
            (Point3D::new(x + a, y + d, z), Point3D::new(x + a, y + d, z)),
            (Point3D::new(x + b, y + c, z), Point3D::new(x + a, y + c, z)),
            (Point3D::new(x + b, y + d, z), Point3D::new(x + b, y + c, z)),
            (Point3D::new(x + a, y + d, z), Point3D::new(x + b, y + d, z)),
            (Point3D::new(x + a, y + c, z), Point3D::new(x + a, y + d, z)),
        ];
        for (orig, dest) in pts {
            muo_server::effects::send_moving_effect_origin(
                w, p, map, item_id, orig, dest, 0, 0, false, false, hue, 0x4,
            );
        }
    }
}

fn storm_targets(
    w: &mut World,
    caster: MobileId,
    loc: Point3D,
    with_los_check_first: bool,
) -> (Vec<MobileId>, bool) {
    let map = w.mobile(caster).and_then(|c| c.map);
    let mut pool = Vec::new();
    let mut pvp = false;
    for m in mobiles_in_range(w, map, loc, 2) {
        if m == caster {
            continue;
        }
        let ok = helper::valid_indirect_target(w, caster, m)
            && mb::can_be_harmful_msg(w, caster, m, false)
            && mb::can_see_mobile(w, caster, m);
        if !ok {
            continue;
        }
        if with_los_check_first && !deps::in_los_mobile(w, caster, m) {
            continue;
        }
        pool.push(m);
        if w.mobile(m).is_some_and(|x| x.player) {
            pvp = true;
        }
    }
    (pool, pvp)
}

fn hail_storm_target(w: &mut World, sp: SpellId, targeted: Targeted) {
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };
    let loc = world_loc(w, targeted);
    if helper::check_town_at(w, loc, caster) && spell::check_sequence(w, sp) {
        /* Summons a storm of hailstones that strikes all Targets within a radius around the Target's Location,
         * dealing cold damage.
         */
        helper::turn(w, caster, Some(Point3dRef::from(targeted)));

        let map = w.mobile(caster).and_then(|c| c.map);
        if map.is_some() {
            storm_effect(w, loc, map, 0x36D4, 0x63);
            let (pool, pvp) = storm_targets(w, caster, loc, true);

            let damage = f64::from(spell::get_new_aos_damage(w, sp, 51, 1, 5, pvp, None));

            for m in pool {
                mb::do_harmful(w, caster, m, false);
                helper::damage_aos_spell(
                    w,
                    sp,
                    m,
                    damage,
                    SpellDamageSplit {
                        cold: 100,
                        ..Default::default()
                    },
                );
            }
        }
    }
}

fn nether_cyclone_target(w: &mut World, sp: SpellId, targeted: Targeted) {
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };
    let loc = world_loc(w, targeted);
    if helper::check_town_at(w, loc, caster) && spell::check_sequence(w, sp) {
        /* Summons a gale of lethal winds that strikes all Targets within a radius around the Target's Location,
         * dealing chaos damage. In addition to inflicting damage, each Target of the Nether Cyclone temporarily
         * loses a percentage of mana and stamina.
         */
        helper::turn(w, caster, Some(Point3dRef::from(targeted)));

        let map = w.mobile(caster).and_then(|c| c.map);
        if map.is_some() {
            storm_effect(w, loc, map, 0x375A, 0x49A);
            let (pool, pvp) = storm_targets(w, caster, loc, true);

            let damage = f64::from(spell::get_new_aos_damage(w, sp, 51, 1, 5, pvp, None));
            let ty = spell::spell_type(w, sp).expect("the spell is casting");
            let reduction =
                (get_base_skill(w, caster) + (ty.get_damage_skill)(w, sp, caster)) / 1200.0;

            for m in pool {
                mb::do_harmful(w, caster, m, false);
                helper::damage_aos_spell(
                    w,
                    sp,
                    m,
                    damage,
                    SpellDamageSplit {
                        chaos: 100,
                        ..Default::default()
                    },
                );

                let resisted_reduction =
                    reduction - skills::value(w, m, SkillName::MagicResist) / 800.0;
                let (stam, stam_max) = (w.mobile(m).map_or(0, |x| x.stam), mb::stam_max(w, m));
                props_b::set_stam(
                    w,
                    m,
                    stam - (f64::from(stam_max) * resisted_reduction) as i32,
                );
                let (mana, mana_max) = (w.mobile(m).map_or(0, |x| x.mana), mb::mana_max(w, m));
                props_b::set_mana(
                    w,
                    m,
                    mana - (f64::from(mana_max) * resisted_reduction) as i32,
                );
            }
        }
    }
}

// ---- StoneFormSpell ----

static STONE_FORM: MysticType = mystic(SpellCircle::Fourth);

/// `StoneFormSpell`: In Rel Ylem.
pub static STONE_FORM_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Mysticism.StoneFormSpell",
    parent: Some(&MYSTIC_SPELL),
    info: SpellInfo::new(
        "Stone Form",
        "In Rel Ylem",
        &[reagent::BLOODMOSS, reagent::FERTILE_DIRT, reagent::GARLIC],
    )
    .action(-1)
    .hand_effect(9002),
    on_cast: stone_form_cast,
    check_cast: stone_form_check_cast,
    family: Some(&STONE_FORM),
    ..MYSTIC_SPELL
};

#[derive(Default)]
struct StoneTable(IndexMap<MobileId, Vec<ModId>>);

fn stone_table(w: &mut World) -> &mut IndexMap<MobileId, Vec<ModId>> {
    &mut w.resources.get_mut::<StoneTable>().0
}

/// The `PlayerDeletedEvent` and `PlayerDeathEvent` handler.
pub fn init(w: &mut World) {
    muo_content::player_mobile::subscribe_deleted(w, remove_stone_effects);
    muo_content::player_mobile::subscribe_death(w, remove_stone_effects);
    muo_content::player_mobile::subscribe_death(w, |w, m| {
        spell_plague::remove_effect(w, m);
    });
}

/// `UnderEffect(m)`.
pub fn stone_form_under_effect(w: &mut World, m: MobileId) -> bool {
    stone_table(w).contains_key(&m)
}

fn sigil(w: &World, m: MobileId) -> bool {
    hooks(w)
        .and_then(|h| h.sigil_exists_on)
        .is_some_and(|f| f(w, m))
}

fn stone_form_check_cast(w: &mut World, sp: SpellId) -> bool {
    let Some(caster) = spell::caster(w, sp) else {
        return false;
    };
    if sigil(w, caster) {
        mm::send_localized_message(w, caster, 1061632, "", 0x3B2); // You can't do that while carrying the sigil.
        return false;
    }

    if !combat::can_begin_action(w, caster, "Server.Spells.Seventh.PolymorphSpell") {
        mm::send_localized_message(w, caster, 1061628, "", 0x3B2); // You can't do that while polymorphed.
        return false;
    }

    if hooks(w)
        .and_then(|h| h.animal_form_under)
        .is_some_and(|f| f(w, caster))
    {
        mm::send_localized_message(w, caster, 1063218, "", 0x3B2); // You cannot use that ability in this form.
        return false;
    }

    if w.mobile(caster).is_some_and(|c| c.flying) {
        mm::send_localized_message(w, caster, 1113415, "", 0x3B2); // You cannot use this ability while flying.
        return false;
    }

    (MYSTIC_SPELL.check_cast)(w, sp)
}

fn stone_form_cast(w: &mut World, sp: SpellId) {
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };
    let body_mod = w.mobile(caster).is_some_and(|c| c.body_mod.body_id() != 0);
    if sigil(w, caster) {
        mm::send_localized_message(w, caster, 1061632, "", 0x3B2); // You can't do that while carrying the sigil.
    } else if !combat::can_begin_action(w, caster, "Server.Spells.Seventh.PolymorphSpell") {
        mm::send_localized_message(w, caster, 1061628, "", 0x3B2); // You can't do that while polymorphed.
    } else if !combat::can_begin_action(w, caster, "Server.Spells.Fifth.IncognitoSpell")
        || (body_mod && !stone_form_under_effect(w, caster))
    {
        mm::send_localized_message(w, caster, 1063218, "", 0x3B2); // You cannot use that ability in this form.
    } else if spell::check_sequence(w, sp) {
        if stone_form_under_effect(w, caster) {
            remove_stone_effects(w, caster);
            effects::play_sound(w, caster, 0xFA);
            mb::delta(w, caster, MobileDelta::RESISTANCES);
        } else {
            if let Some(mount) = props_b::mount(w, caster) {
                muo_content::creature::mount::set_rider(w, mount, None);
            }

            props_b::set_body_mod(w, caster, Body::new(0x2C1));
            props_b::set_hue_mod(w, caster, 0);

            let ty = spell::spell_type(w, sp).expect("the spell is casting");
            let sum = get_base_skill(w, caster) + (ty.get_damage_skill)(w, sp, caster);
            let offset = (sum / 24.0) as i32;

            let defs = [
                (ResistanceType::Physical, "PhysicalResistStoneFormSpell"),
                (ResistanceType::Fire, "FireResistStoneFormSpell"),
                (ResistanceType::Cold, "ColdResistStoneFormSpell"),
                (ResistanceType::Poison, "PoisonResistStoneFormSpell"),
                (ResistanceType::Energy, "EnergyResistStoneFormSpell"),
            ];
            let mut ids = Vec::new();
            for (t, name) in defs {
                let m = ResistanceMod::new(w, t, Some(name), offset, None);
                ids.push(m.id);
                mb::add_resistance_mod(w, caster, m);
            }
            stone_table(w).insert(caster, ids);

            effects::play_sound(w, caster, 0x65A);
            mb::delta(w, caster, MobileDelta::RESISTANCES);

            let damage_bonus = (sum / 12.0) as i32;
            let resist_cap = (sum / 48.0) as i32;

            if is_player_mobile(w, caster) {
                add_buff(
                    w,
                    caster,
                    BuffInfo::with_secondary(BuffIcon::StoneForm, 1080145, 1080146).args(
                        TextDefinition::of_text(format!(
                            "-10\t-2\t{offset}\t{resist_cap}\t{damage_bonus}"
                        )),
                    ),
                );
            }
        }
    }

    spell::finish_sequence(w, sp);
}

/// `RemoveEffects(m)`.
pub fn remove_stone_effects(w: &mut World, m: MobileId) {
    let Some(mods) = stone_table(w).shift_remove(&m) else {
        return;
    };
    for id in mods {
        mb::remove_resistance_mod(w, m, id);
    }

    props_b::set_body_mod(w, m, Body::new(0));
    props_b::set_hue_mod(w, m, -1);
    remove_buff(w, m, BuffIcon::StoneForm);
}
