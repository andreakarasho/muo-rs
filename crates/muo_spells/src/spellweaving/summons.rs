//! The Arcane Summon base and the Summon Fey, Summon Fiend and Nature's Fury spells.

use muo_content::creature::{self, control};
use muo_content::factory;
use muo_content::spells::SpellInfo;
use muo_content::spells::helper::{self, Point3dRef};
use muo_content::spells::hooks::hooks;
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_content::spells::targeting::{SpellTargetKind, begin_spell_target};
use muo_primitives::TimeSpan;
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::mobile::part2::messages as mm;
use muo_server::skills::{self, SkillName};
use muo_server::target::{TargetFlags, Targeted};
use muo_server::timer;

use super::arcanist::{ARCANE_BASE, ARCANIST_SPELL, ArcaneType, focus_level};
use crate::common::is_player_mobile;

fn ml_quest_flag(w: &World, m: MobileId, flag: &str) -> bool {
    hooks(w)
        .and_then(|h| h.ml_quest_flag)
        .is_none_or(|f| f(w, m, flag))
}

/// `ArcaneSummon<T>.CheckCast()`.
fn summon_check_cast(w: &mut World, sp: SpellId) -> bool {
    if !(ARCANIST_SPELL.check_cast)(w, sp) {
        return false;
    }
    let Some(caster) = spell::caster(w, sp) else {
        return false;
    };
    let (followers, max) = w
        .mobile(caster)
        .map_or((0, 0), |c| (c.followers, c.followers_max));
    if followers + 1 > max {
        mm::send_localized_message(w, caster, 1074270, "", 0x3B2); // You have too many followers to summon another one.
        return false;
    }
    true
}

/// `ArcaneSummon<T>.OnCast()`: `class` is the summoned creature and `sound` the summon sound.
fn summon_on_cast(w: &mut World, sp: SpellId, class: &str, sound: i32, flag: &str) {
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };

    // `CheckSequence()` of Fey and Fiend first checks the quest flag ("This is done after casting completes").
    if is_player_mobile(w, caster) && !ml_quest_flag(w, caster, flag) {
        let n = if flag == "SummonFey" {
            1074563
        } else {
            1074564
        };
        // You haven't forged a friendship with the fey and are unable to summon their aid. /
        // You haven't demonstrated mastery to summon a fiend.
        mm::send_localized_message(w, caster, n, "", 0x3B2);
        // `CheckSequence()` fails here without finishing the sequence.
        return;
    }

    if spell::check_sequence(w, sp) {
        let focus = focus_level(w, sp);
        let skill = skills::value(w, caster, SkillName::Spellweaving);
        let duration = TimeSpan::from_mins_f64(skill / 24.0 + f64::from(focus) * 2.0);

        let (followers, max) = w
            .mobile(caster)
            .map_or((0, 0), |c| (c.followers, c.followers_max));
        let summons = (1 + focus).min(max - followers);

        for _ in 0..summons {
            let Some(bc) = factory::make_mobile(w, class) else {
                break;
            };
            helper::summon(w, bc, caster, sound, duration, false, false);
        }

        spell::finish_sequence(w, sp);
    }
}

static SUMMON_FEY: ArcaneType = ArcaneType {
    required_skill: 38.0,
    required_mana: 10,
    ..ARCANE_BASE
};

/// `SummonFeySpell`: Alalithra.
pub static SUMMON_FEY_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Spellweaving.SummonFeySpell",
    parent: Some(&ARCANIST_SPELL),
    info: SpellInfo::new("Summon Fey", "Alalithra", &[]).action(-1),
    cast_delay_base: |_, _| TimeSpan::from_millis(1500),
    check_cast: summon_check_cast,
    on_cast: |w, sp| summon_on_cast(w, sp, "Server.Mobiles.ArcaneFey", 0x217, "SummonFey"),
    family: Some(&SUMMON_FEY),
    ..ARCANIST_SPELL
};

static SUMMON_FIEND: ArcaneType = ArcaneType {
    required_skill: 38.0,
    required_mana: 10,
    ..ARCANE_BASE
};

/// `SummonFiendSpell`: Nylisstra.
pub static SUMMON_FIEND_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Spellweaving.SummonFiendSpell",
    parent: Some(&ARCANIST_SPELL),
    info: SpellInfo::new("Summon Fiend", "Nylisstra", &[]).action(-1),
    cast_delay_base: |_, _| TimeSpan::from_secs(2),
    check_cast: summon_check_cast,
    on_cast: |w, sp| summon_on_cast(w, sp, "Server.Mobiles.ArcaneFiend", 0x216, "SummonFiend"),
    family: Some(&SUMMON_FIEND),
    ..ARCANIST_SPELL
};

static NATURE_FURY: ArcaneType = ArcaneType {
    required_skill: 0.0,
    required_mana: 24,
    ..ARCANE_BASE
};

/// `NatureFurySpell`: Rauvvrae.
pub static NATURE_FURY_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Spellweaving.NatureFurySpell",
    parent: Some(&ARCANIST_SPELL),
    target_range: Some(muo_content::spells::targeting::target_range),
    info: SpellInfo::new("Nature's Fury", "Rauvvrae", &[])
        .action(-1)
        .allow_town(false),
    cast_delay_base: |_, _| TimeSpan::from_millis(1500),
    check_cast: fury_check_cast,
    on_cast: |w, sp| {
        begin_spell_target(
            w,
            sp,
            true,
            TargetFlags::NONE,
            false,
            SpellTargetKind::Point,
            fury_target,
        )
    },
    family: Some(&NATURE_FURY),
    ..ARCANIST_SPELL
};

fn fury_check_cast(w: &mut World, sp: SpellId) -> bool {
    if !(ARCANIST_SPELL.check_cast)(w, sp) {
        return false;
    }
    let Some(caster) = spell::caster(w, sp) else {
        return false;
    };
    let (followers, max) = w
        .mobile(caster)
        .map_or((0, 0), |c| (c.followers, c.followers_max));
    if followers + 1 > max {
        mm::send_localized_message(w, caster, 1049645, "", 0x3B2); // You have too many followers to summon that creature.
        return false;
    }
    true
}

fn fury_target(w: &mut World, sp: SpellId, targeted: Targeted) {
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };
    // `new Point3D(point)`: the raw point, not the surface top.
    let p = helper::point_of(w, Point3dRef::from(targeted));
    let map = w.mobile(caster).and_then(|c| c.map);
    let Some(map_id) = map else {
        return;
    };

    // `Region.Find(p, map).GetRegion<HouseRegion>()?.House?.IsFriend(Caster) == false`
    if hooks(w)
        .and_then(|h| h.house_blocks_nature_fury)
        .is_some_and(|f| f(w, caster, p, map_id))
    {
        return;
    }

    if !muo_server::map::fit::can_spawn_mobile_at(w, map_id, p.x, p.y, p.z) {
        mm::send_localized_message(w, caster, 501942, "", 0x3B2); // That location is blocked.
    } else if helper::check_town(w, Point3dRef::Point(p), caster) && spell::check_sequence(w, sp) {
        let focus = focus_level(w, sp);
        let skill = skills::value(w, caster, SkillName::Spellweaving);
        let duration = TimeSpan::from_secs_f64(skill / 24.0 + 25.0 + f64::from(focus) * 2.0);

        if let Some(nf) = factory::make_mobile(w, "Server.Mobiles.NatureFury") {
            control::summon(w, nf, false, caster, p, 0x5CB, duration, None);

            // InternalTimer: every 5 seconds, until dead or `DamageMin > 20`, the damage grows by one.
            let h = timer::start(
                w,
                TimeSpan::from_secs(5),
                TimeSpan::from_secs(5),
                0,
                move |w| {
                    let gone =
                        w.mobile(nf).is_none_or(|m| m.deleted) || !crate::common::alive(w, nf);
                    let min = creature::cd(w, nf).map_or(0, |d| d.damage_min);
                    if gone || min > 20 {
                        if let Some(mut t) = w.resources.get_mut::<FuryTimers>().0.shift_remove(&nf)
                        {
                            t.cancel(w);
                        }
                    } else {
                        creature::edit(w, nf, |d| {
                            d.damage_min += 1;
                            d.damage_max += 1;
                        });
                    }
                },
            );
            w.resources.get_mut::<FuryTimers>().0.insert(nf, h);
        }
    }
}

#[derive(Default)]
struct FuryTimers(indexmap::IndexMap<MobileId, muo_server::timer::TimerHandle>);
