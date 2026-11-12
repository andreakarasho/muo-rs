//! Peacemaking, Discordance and Provocation, and the static parts of `BaseInstrument` they call
//! (`GetBardRange`, `CheckMusicianship`, `GetBaseDifficulty`).
//!
//! The instrument items (`BaseInstrument`: which one the bard plays, its sounds, uses, quality and slayer) are
//! `muo_items`'; [`InstrumentOps`] is what these skills ask of them (`skill_items::instruments` sets it).

use indexmap::IndexMap;
use muo_primitives::{TimeSpan, utility};
use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::mobile::base as mb;
use muo_server::mobile::part2::{enumerators, messages as mm};
use muo_server::mobile::{ResistanceType, StatType};
use muo_server::mods::{ResistanceMod, SkillMod, StatMod};
use muo_server::skills::{self, SkillName};
use muo_server::target::{TargetCancelType, Targeted};
use muo_server::timer::{self, TimerHandle};

use super::common::{
    TargetOptions, check_skill, check_target_skill, in_safe_zone, msg, set_target,
};
use super::skills_info::set_callback;
use crate::abilities::{MonsterAbilityType, get_ability};
use crate::core;
use crate::creature::enums::AIType;
use crate::creature::{self, cd, control, ctype, pets};
use crate::misc::titles;
use crate::mx;

/// What the bard skills ask of the instrument items. Every method says what the absent class answers.
pub trait InstrumentOps: Sync {
    /// `BaseInstrument.PickInstrument(from, callback)`: finds the instrument `from` plays (or targets one) and calls
    /// back with it. Absent: never calls back.
    fn pick_instrument(
        &self,
        _w: &mut World,
        _from: MobileId,
        _callback: fn(&mut World, MobileId, ItemId),
    ) {
    }
    /// `instrument.PlayInstrumentWell(from)`.
    fn play_well(&self, _w: &mut World, _instrument: ItemId, _from: MobileId) {}
    /// `instrument.PlayInstrumentBadly(from)`.
    fn play_badly(&self, _w: &mut World, _instrument: ItemId, _from: MobileId) {}
    /// `instrument.ConsumeUse(from)`.
    fn consume_use(&self, _w: &mut World, _instrument: ItemId, _from: MobileId) {}
    /// `instrument.GetDifficultyFor(targ)`. Absent: the base difficulty, no quality or slayer.
    fn difficulty_for(&self, w: &mut World, _instrument: ItemId, targ: MobileId) -> f64 {
        get_base_difficulty(w, targ)
    }
}

struct NoInstruments;
impl InstrumentOps for NoInstruments {}
static NO_INSTRUMENTS: NoInstruments = NoInstruments;

#[derive(Default)]
pub struct InstrumentHooks {
    pub ops: Option<&'static dyn InstrumentOps>,
}

fn ops(w: &World) -> &'static dyn InstrumentOps {
    w.resources
        .get::<InstrumentHooks>()
        .and_then(|h| h.ops)
        .unwrap_or(&NO_INSTRUMENTS)
}

fn tick(w: &World) -> i64 {
    w.tick_count
}

fn set_next_skill_time(w: &mut World, m: MobileId, t: i64) {
    if let Some(x) = w.mobile_mut(m) {
        x.next_skill_time = t;
    }
}

fn entity_mobile(t: Targeted) -> Option<MobileId> {
    match t {
        Targeted::Entity(EntityId::Mobile(m)) => Some(m),
        _ => None,
    }
}

/// `BaseInstrument.GetBardRange(bard, skill)`.
pub fn get_bard_range(w: &mut World, bard: MobileId, skill: SkillName) -> i32 {
    8 + (skills::value(w, bard, skill) / 15.0) as i32
}

/// `BaseInstrument.CheckMusicianship(m)`.
pub fn check_musicianship(w: &mut World, m: MobileId) -> bool {
    check_skill(w, m, SkillName::Musicianship, 0.0, 120.0);
    skills::value(w, m, SkillName::Musicianship) / 100.0 > utility::random_double(&mut w.rng)
}

/// `BaseInstrument.GetBaseDifficulty(targ)`.
pub fn get_base_difficulty(w: &mut World, targ: MobileId) -> f64 {
    let hits_max = f64::from(mx::hits_max(w, targ));
    let (stam_max, mana_max) = (mb::stam_max(w, targ), mb::mana_max(w, targ));
    let mut val = hits_max * if core::lbr(w) { 1.6 } else { 0.625 }
        + f64::from(stam_max)
        + f64::from(mana_max);

    val += f64::from(skills::total(w, targ)) / 10.0;

    if val > 700.0 {
        val = 700.0 + ((val - 700.0) * (3.0 / 11.0)) as i32 as f64;
    }

    if let Some(d) = cd(w, targ) {
        let (ai, paragon) = (d.current_ai, d.is_paragon);
        // IsMageryCreature
        if ai == AIType::AiMage && skills::base(w, targ, SkillName::Magery) > 5.0 {
            val += 100.0;
        }
        // IsFireBreathingCreature
        if get_ability(w, targ, MonsterAbilityType::FireBreath).is_some() {
            val += 100.0;
        }
        // IsPoisonImmune
        if ctype(w, targ).is_some_and(|c| (c.poison_immune)(w, targ).is_some()) {
            val += 100.0;
        }
        // GetPoisonLevel
        let level = ctype(w, targ)
            .and_then(|c| (c.hit_poison)(w, targ))
            .map_or(-1, |p| muo_server::poison::level(w, p));
        if super::common::mobile_is(w, targ, "Server.Mobiles.VampireBat")
            || super::common::mobile_is(w, targ, "Server.Mobiles.VampireBatFamiliar")
        {
            val += 100.0;
        }
        val += f64::from((level + 1) * 20);
        val /= 10.0;
        if paragon {
            val += 40.0;
        }
    } else {
        if super::common::mobile_is(w, targ, "Server.Mobiles.VampireBat")
            || super::common::mobile_is(w, targ, "Server.Mobiles.VampireBatFamiliar")
        {
            val += 100.0;
        }
        // GetPoisonLevel(null) is 0
        val /= 10.0;
    }

    if core::se(w) && val > 160.0 {
        val = 160.0;
    }

    val
}

fn in_backpack(w: &World, from: MobileId, instrument: ItemId) -> bool {
    let pack = w.mobile(from).and_then(|m| m.backpack).map(EntityId::Item);
    muo_server::item::ops::is_child_of(w, instrument, pack)
}

// ---- Peacemaking ------------------------------------------------------------------------------------------------

pub fn peacemaking_on_use(w: &mut World, m: MobileId) -> TimeSpan {
    mb::revealing_action(w, m);

    ops(w).pick_instrument(w, m, peacemaking_picked);

    TimeSpan::from_secs(1) // Cannot use another skill for 1 second
}

fn peacemaking_picked(w: &mut World, from: MobileId, instrument: ItemId) {
    mb::revealing_action(w, from);
    msg(w, from, 1049525); // Whom do you wish to calm?
    let range = get_bard_range(w, from, SkillName::Peacemaking);
    set_target(
        w,
        from,
        range,
        TargetOptions {
            on_cancel: Some(|w: &mut World, from: MobileId, _: TargetCancelType| {
                let t = tick(w);
                set_next_skill_time(w, from, t);
            }),
            ..Default::default()
        },
        move |w, from, targeted| peacemaking_on_target(w, from, targeted, instrument),
    );
    let t = tick(w) + 30000; // 30s timeout on the targeter
    set_next_skill_time(w, from, t);
}

fn peacemaking_on_target(w: &mut World, from: MobileId, targeted: Targeted, instrument: ItemId) {
    mb::revealing_action(w, from);

    let Some(targ) = entity_mobile(targeted) else {
        msg(w, from, 1049528); // You cannot calm that!
        return;
    };
    if in_safe_zone(w, from) {
        mm::send_message(w, from, "You may not use peacemaking in this area.");
    } else if in_safe_zone(w, targ) {
        mm::send_message(w, from, "You may not use peacemaking there.");
    } else if !in_backpack(w, from, instrument) {
        // The instrument you are trying to play is no longer in your backpack!
        msg(w, from, 1062488);
    } else {
        let t = tick(w) + 10000;
        set_next_skill_time(w, from, t);

        if targ == from {
            // Standard mode : reset combatants for everyone in the area

            if !check_musicianship(w, from) {
                msg(w, from, 500612); // You play poorly, and there is no effect.
                ops(w).play_badly(w, instrument, from);
                ops(w).consume_use(w, instrument, from);
            } else if !check_skill(w, from, SkillName::Peacemaking, 0.0, 120.0) {
                msg(w, from, 500613); // You attempt to calm everyone, but fail.
                ops(w).play_badly(w, instrument, from);
                ops(w).consume_use(w, instrument, from);
            } else {
                let t = tick(w) + 5000;
                set_next_skill_time(w, from, t);
                ops(w).play_well(w, instrument, from);
                ops(w).consume_use(w, instrument, from);

                if w.mobile(from).is_some_and(|m| m.map.is_some()) {
                    let range = get_bard_range(w, from, SkillName::Peacemaking);

                    let mut calmed = false;

                    for m in enumerators::get_mobiles_in_range(w, from, range) {
                        let bc = creature::is_creature(w, m);
                        let uncalmable = bc && ctype(w, m).is_some_and(|c| (c.uncalmable)(w, m));
                        let area_immune =
                            bc && ctype(w, m).is_some_and(|c| (c.area_peace_immune)(w, m));
                        if uncalmable
                            || area_immune
                            || m == from
                            || !mx::can_be_harmful_quiet(w, from, m)
                        {
                            continue;
                        }

                        calmed = true;

                        // You hear lovely music, and forget to continue battling!
                        msg(w, m, 500616);
                        mx::set_combatant(w, m, None);
                        mx::set_warmode(w, m, false);

                        if bc && !cd(w, m).is_some_and(|d| d.bard_pacified) {
                            let end = w.now + TimeSpan::from_secs(1);
                            pets::pacify(w, m, end);
                        }
                    }

                    if !calmed {
                        // You play hypnotic music, but there is nothing in range for you to calm.
                        msg(w, from, 1049648);
                    } else {
                        msg(w, from, 500615); // You play your hypnotic music, stopping the battle.
                    }
                }
            }
        } else {
            // Target mode : pacify a single target for a longer duration
            let bc = creature::is_creature(w, targ);

            if !mx::can_be_harmful_quiet(w, from, targ) {
                msg(w, from, 1049528);
            } else if bc && ctype(w, targ).is_some_and(|c| (c.uncalmable)(w, targ)) {
                msg(w, from, 1049526); // You have no chance of calming that creature.
            } else if bc && cd(w, targ).is_some_and(|d| d.bard_pacified) {
                msg(w, from, 1049527); // That creature is already being calmed.
            } else if !check_musicianship(w, from) {
                msg(w, from, 500612); // You play poorly, and there is no effect.
                let t = tick(w) + 5000;
                set_next_skill_time(w, from, t);
                ops(w).play_badly(w, instrument, from);
                ops(w).consume_use(w, instrument, from);
            } else {
                let mut diff = ops(w).difficulty_for(w, instrument, targ) - 10.0;
                let music = skills::value(w, from, SkillName::Musicianship);

                if music > 100.0 {
                    diff -= (music - 100.0) * 0.5;
                }

                if !check_target_skill(
                    w,
                    from,
                    SkillName::Peacemaking,
                    targeted,
                    diff - 25.0,
                    diff + 25.0,
                ) {
                    msg(w, from, 1049531); // You attempt to calm your target, but fail.
                    ops(w).play_badly(w, instrument, from);
                    ops(w).consume_use(w, instrument, from);
                } else {
                    ops(w).play_well(w, instrument, from);
                    ops(w).consume_use(w, instrument, from);

                    let t = tick(w) + 5000;
                    set_next_skill_time(w, from, t);
                    mx::set_combatant(w, targ, None);
                    mx::set_warmode(w, targ, false);

                    msg(w, from, 1049532); // You play hypnotic music, calming your target.
                    if bc {
                        // You play hypnotic music, calming your target.
                        let seconds = (100.0 - diff / 1.5).clamp(10.0, 120.0);

                        let end = w.now + TimeSpan::from_secs_f64(seconds);
                        pets::pacify(w, targ, end);
                    } else {
                        // You play hypnotic music, calming your target.
                        // You hear lovely music, and forget to continue battling!
                        msg(w, targ, 500616);
                    }
                }
            }
        }
    }
}

// ---- Discordance ------------------------------------------------------------------------------------------------

struct DiscordanceInfo {
    from: MobileId,
    creature: MobileId,
    effect: i32,
    ending: bool,
    end_time: muo_primitives::DateTime,
    timer: TimerHandle,
}

/// `Discordance.m_Table`.
#[derive(Default)]
pub struct DiscordanceTable(IndexMap<MobileId, DiscordanceInfo>);

fn table(w: &mut World) -> &mut DiscordanceTable {
    w.resources.get_mut::<DiscordanceTable>()
}

/// `Discordance.GetEffect(targ, ref effect)`.
pub fn discordance_effect(w: &World, targ: MobileId) -> Option<i32> {
    w.resources
        .get::<DiscordanceTable>()
        .and_then(|t| t.0.get(&targ))
        .map(|i| i.effect)
}

pub fn discordance_on_use(w: &mut World, m: MobileId) -> TimeSpan {
    mb::revealing_action(w, m);

    ops(w).pick_instrument(w, m, discordance_picked);

    TimeSpan::from_secs(1) // Cannot use another skill for 1 second
}

fn discordance_picked(w: &mut World, from: MobileId, instrument: ItemId) {
    mb::revealing_action(w, from);
    msg(w, from, 1049541); // Choose the target for your song of discordance.
    let range = get_bard_range(w, from, SkillName::Discordance);
    set_target(
        w,
        from,
        range,
        TargetOptions::default(),
        move |w, from, targeted| discordance_on_target(w, from, targeted, instrument),
    );
    let t = tick(w) + 6000;
    set_next_skill_time(w, from, t);
}

fn process_discordance(w: &mut World, targ: MobileId) {
    let Some(info) = table(w).0.get(&targ) else {
        return;
    };
    let from = info.from;
    let mut ends = false;

    // According to uoherald bard must remain alive, visible, and
    // within range of the target or the effect ends in 15 seconds.
    if !mx::alive(w, targ)
        || mx::deleted(w, targ)
        || !mx::alive(w, from)
        || w.mobile(from).is_some_and(|m| m.hidden)
    {
        ends = true;
    } else {
        let range = creature::misc::distance_sqrt(w, targ, from) as i32;
        let max_range = get_bard_range(w, from, SkillName::Discordance);

        if mx::map(w, from) != mx::map(w, targ) || range > max_range {
            ends = true;
        }
    }

    let now = w.now;
    let info = table(w).0.get_mut(&targ).expect("checked");
    if ends && info.ending && info.end_time < now {
        let mut t = info.timer;
        t.cancel(w);

        // info.Clear()
        mb::remove_resistance_mod_by_name(w, targ, "Discordance");
        mb::remove_stat_mod_by_name(w, targ, "Discordance");
        mb::remove_skill_mod_by_name(w, targ, "Discordance");
        table(w).0.shift_remove(&targ);
    } else {
        if ends && !info.ending {
            info.ending = true;
            info.end_time = now + TimeSpan::from_secs(15);
        } else if !ends {
            info.ending = false;
            info.end_time = now;
        }

        mx::fixed_effect(w, targ, 0x376A, 1, 32);
    }
}

fn discordance_on_target(w: &mut World, from: MobileId, target: Targeted, instrument: ItemId) {
    mb::revealing_action(w, from);
    let t = tick(w) + 1000;
    set_next_skill_time(w, from, t);

    if !in_backpack(w, from, instrument) {
        // The instrument you are trying to play is no longer in your backpack!
        msg(w, from, 1062488);
        return;
    }
    let Some(targ) = entity_mobile(target) else {
        msg(w, from, 1049535); // A song of discord would have no effect on that.
        return;
    };

    let bc = creature::is_creature(w, targ);
    let master_is_from = control::control_master(w, targ) == Some(from);
    if targ == from
        || bc
            && (ctype(w, targ).is_some_and(|c| (c.bard_immune)(w, targ))
                || !mx::can_be_harmful_quiet(w, from, targ))
            && !master_is_from
    {
        msg(w, from, 1049535); // A song of discord would have no effect on that.
    } else if table(w).0.contains_key(&targ) {
        // Already discorded
        msg(w, from, 1049537); // Your target is already in discord.
    } else if !w.mobile(targ).is_some_and(|m| m.player) {
        let mut diff = ops(w).difficulty_for(w, instrument, targ) - 10.0;
        let music = skills::value(w, from, SkillName::Musicianship);

        if music > 100.0 {
            diff -= (music - 100.0) * 0.5;
        }

        if !check_musicianship(w, from) {
            msg(w, from, 500612); // You play poorly, and there is no effect.
            ops(w).play_badly(w, instrument, from);
            ops(w).consume_use(w, instrument, from);
        } else if check_target_skill(
            w,
            from,
            SkillName::Discordance,
            target,
            diff - 25.0,
            diff + 25.0,
        ) {
            msg(w, from, 1049539); // You play the song surpressing your targets strength
            ops(w).play_well(w, instrument, from);
            ops(w).consume_use(w, instrument, from);

            let effect: i32;
            let scalar: f64;

            if core::aos(w) {
                let discord = skills::value(w, from, SkillName::Discordance);

                let mut e = if discord > 100.0 {
                    -20 + ((discord - 100.0) / -2.5) as i32
                } else {
                    (discord / -5.0) as i32
                };

                if core::se(w) && get_base_difficulty(w, targ) >= 160.0 {
                    e /= 2;
                }
                effect = e;

                scalar = f64::from(effect) * 0.01;

                for ty in [
                    ResistanceType::Physical,
                    ResistanceType::Fire,
                    ResistanceType::Cold,
                    ResistanceType::Poison,
                    ResistanceType::Energy,
                ] {
                    let m = ResistanceMod::new(w, ty, Some("Discordance"), effect, None);
                    mb::add_resistance_mod(w, targ, m);
                }
            } else {
                effect = (skills::value(w, from, SkillName::Discordance) / -5.0) as i32;
                scalar = f64::from(effect) * 0.01;

                let (s, i, d) = w.mobile(targ).map_or((0, 0, 0), |m| (m.str, m.int, m.dex));
                for (ty, raw) in [(StatType::STR, s), (StatType::INT, i), (StatType::DEX, d)] {
                    let m = StatMod::new(
                        w,
                        ty,
                        Some("Discordance"),
                        (f64::from(raw) * scalar) as i32,
                        TimeSpan::ZERO,
                        None,
                    );
                    mb::add_stat_mod(w, targ, m);
                }
            }

            let len = w
                .resources
                .get::<muo_server::skills::SkillTable>()
                .map_or(0, |t| t.info.len());
            for i in 0..len {
                let Some(name) = SkillName::from_i32(i as i32) else {
                    continue;
                };
                let v = skills::value(w, targ, name);
                if v > 0.0 {
                    let m = SkillMod::new_default(w, name, Some("Discordance"), true, v * scalar);
                    mb::add_skill_mod(w, targ, m);
                }
            }

            let handle = timer::start(
                w,
                TimeSpan::ZERO,
                TimeSpan::from_millis(1250),
                0,
                move |w| process_discordance(w, targ),
            );
            let now = w.now;
            table(w).0.insert(
                targ,
                DiscordanceInfo {
                    from,
                    creature: targ,
                    effect: effect.abs(),
                    ending: false,
                    end_time: now,
                    timer: handle,
                },
            );
        } else {
            msg(w, from, 1049540); // You fail to disrupt your target
            ops(w).play_badly(w, instrument, from);
            ops(w).consume_use(w, instrument, from);
        }

        let t = tick(w) + 12000;
        set_next_skill_time(w, from, t);
    } else {
        ops(w).play_badly(w, instrument, from);
    }
}

// ---- Provocation ------------------------------------------------------------------------------------------------

pub fn provocation_on_use(w: &mut World, m: MobileId) -> TimeSpan {
    mb::revealing_action(w, m);

    ops(w).pick_instrument(w, m, provocation_picked);

    TimeSpan::from_secs(1) // Cannot use another skill for 1 second
}

fn provocation_picked(w: &mut World, from: MobileId, instrument: ItemId) {
    mb::revealing_action(w, from);
    msg(w, from, 501587); // Whom do you wish to incite?
    let range = get_bard_range(w, from, SkillName::Provocation);
    set_target(
        w,
        from,
        range,
        TargetOptions::default(),
        move |w, from, targeted| provocation_first(w, from, targeted, instrument),
    );
}

fn provocation_first(w: &mut World, from: MobileId, targeted: Targeted, instrument: ItemId) {
    mb::revealing_action(w, from);

    let creature = entity_mobile(targeted).filter(|&c| creature::is_creature(w, c));
    match creature {
        Some(creature) if mb::can_be_harmful(w, from, creature) => {
            if !in_backpack(w, from, instrument) {
                // The instrument you are trying to play is no longer in your backpack!
                msg(w, from, 1062488);
            } else if muo_server::notoriety::compute(w, from, creature)
                == muo_server::notoriety::INNOCENT
            {
                // Think not that I fail to see what thou art doing!
                mm::say_number(w, creature, 501591, "");

                titles::award_karma(w, from, -75, true);
            } else if cd(w, creature).is_some_and(|d| d.controlled) {
                msg(w, from, 501590); // They are too loyal to their master to be provoked.
            } else if cd(w, creature).is_some_and(|d| d.is_paragon)
                && get_base_difficulty(w, creature) >= 160.0
            {
                msg(w, from, 1049446); // You have no chance of provoking those creatures.
            } else {
                mb::revealing_action(w, from);
                ops(w).play_well(w, instrument, from);

                // You play your music and your target becomes angered.  Whom do you wish them to attack?
                msg(w, from, 1008085);
                let range = get_bard_range(w, from, SkillName::Provocation);
                set_target(
                    w,
                    from,
                    range,
                    TargetOptions::default(),
                    move |w, from, targeted| {
                        provocation_second(w, from, targeted, instrument, creature)
                    },
                );
            }
        }
        _ => msg(w, from, 501589), // You can't incite that!
    }
}

fn provocation_second(
    w: &mut World,
    from: MobileId,
    targeted: Targeted,
    instrument: ItemId,
    first: MobileId,
) {
    mb::revealing_action(w, from);

    let Some(creature) = entity_mobile(targeted).filter(|&c| creature::is_creature(w, c)) else {
        msg(w, from, 501589); // You can't incite that!
        return;
    };

    let unprovokable = |w: &World, c: MobileId| ctype(w, c).is_some_and(|x| (x.unprovokable)(w, c));
    if !in_backpack(w, from, instrument) {
        // The instrument you are trying to play is no longer in your backpack!
        msg(w, from, 1062488);
    } else if unprovokable(w, first)
        || (unprovokable(w, creature)
            && !super::common::mobile_is(w, creature, "Server.Mobiles.DemonKnight"))
    {
        msg(w, from, 1049446); // You have no chance of provoking those creatures.
    } else if mx::map(w, first) != mx::map(w, creature) || {
        let range = get_bard_range(w, from, SkillName::Provocation);
        !mx::in_range(w, first, creature, range)
    } {
        // The creatures you are trying to provoke are too far away from each other for your music to have an effect.
        msg(w, from, 1049450);
    } else if first != creature {
        let t = tick(w) + 10000;
        set_next_skill_time(w, from, t);

        let mut diff = (ops(w).difficulty_for(w, instrument, first)
            + ops(w).difficulty_for(w, instrument, creature))
            * 0.5
            - 5.0;
        let music = skills::value(w, from, SkillName::Musicianship);

        if music > 100.0 {
            diff -= (music - 100.0) * 0.5;
        }

        if mb::can_be_harmful(w, from, first) && mb::can_be_harmful(w, from, creature) {
            if !check_musicianship(w, from) {
                let t = tick(w) + 5000;
                set_next_skill_time(w, from, t);
                msg(w, from, 500612); // You play poorly, and there is no effect.
                ops(w).play_badly(w, instrument, from);
                ops(w).consume_use(w, instrument, from);
            } else if !check_target_skill(
                w,
                from,
                SkillName::Provocation,
                targeted,
                diff - 25.0,
                diff + 25.0,
            ) {
                let t = tick(w) + 5000;
                set_next_skill_time(w, from, t);
                msg(w, from, 501599); // Your music fails to incite enough anger.
                ops(w).play_badly(w, instrument, from);
                ops(w).consume_use(w, instrument, from);
            } else {
                msg(w, from, 501602); // Your music succeeds, as you start a fight.
                ops(w).play_well(w, instrument, from);
                ops(w).consume_use(w, instrument, from);
                pets::provoke(w, first, from, creature, true);
            }
        }
    } else {
        msg(w, from, 501593); // You can't tell someone to attack themselves!
    }
}

pub fn initialize(w: &mut World) {
    set_callback(w, SkillName::Peacemaking, peacemaking_on_use);
    set_callback(w, SkillName::Discordance, discordance_on_use);
    set_callback(w, SkillName::Provocation, provocation_on_use);
}

/// Sets the `ContentHooks` this file answers: `BaseInstrument.GetBaseDifficulty`.
pub fn configure(w: &mut World) {
    crate::hooks::hooks_mut(w).instrument_base_difficulty = Some(get_base_difficulty);
}

#[allow(unused)]
fn _unused(i: &DiscordanceInfo) -> MobileId {
    i.creature
}
