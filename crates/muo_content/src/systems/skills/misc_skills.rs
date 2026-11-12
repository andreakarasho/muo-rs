//! Begging, Spirit Speak, Detect Hidden, Remove Trap and Snooping.
//!
//! The faction trap branch of `RemoveTrap` (`BaseFactionTrap`, the removal kit, silver) runs through
//! [`SkillItemFns::disarm_faction_trap`]; `DetectHidden`'s area scan reveals `BaseFactionTrap`/`BaseTrap` and
//! `ShadowKnight` via [`SkillItemFns`]; the passive `TryDetectStealther` reads `ContentFeatureFlags.PassiveDetectHidden`
//! from [`DetectHiddenSettings`].

use indexmap::IndexMap;
use muo_primitives::{MessageType, Point3D, TimeSpan, utility};
use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::container as cont;
use muo_server::map::MapId;
use muo_server::mobile::part1::props_b;
use muo_server::mobile::part2::{effects, enumerators, messages as mm};
use muo_server::mobile::{AccessLevel, base as mb};
use muo_server::skills::{self, SkillName};
use muo_server::target::{self, Target, TargetCancelType, TargetFlags, Targeted};
use muo_server::timer;

use super::common::{
    TargetOptions, check_skill, check_target_skill, entity, item_is, local_msg, msg, msg_args,
    set_target,
};
use super::hooks::ops;
use super::skills_info::set_callback;
use crate::core;
use crate::creature;
use crate::misc::titles;
use crate::spells::spell::{self, SPELL_BASE, SpellId};
use crate::spells::{self, DisturbType, SpellInfo, SpellType};

fn tick(w: &World) -> i64 {
    w.tick_count
}

fn set_next_skill_time(w: &mut World, m: MobileId, t: i64) {
    if let Some(x) = w.mobile_mut(m) {
        x.next_skill_time = t;
    }
}

// ---- Begging ----------------------------------------------------------------------------------------------------

pub fn begging_on_use(w: &mut World, m: MobileId) -> TimeSpan {
    mb::revealing_action(w, m);

    set_target(
        w,
        m,
        12,
        TargetOptions {
            on_cancel: Some(begging_on_cancel),
            ..Default::default()
        },
        begging_on_target,
    );
    mb::revealing_action(w, m);

    msg(w, m, 500397); // To whom do you wish to grovel?

    TimeSpan::from_secs(30)
}

fn begging_on_cancel(w: &mut World, from: MobileId, _: TargetCancelType) {
    let t = tick(w);
    set_next_skill_time(w, from, t);
}

fn begging_on_target(w: &mut World, from: MobileId, targeted: Targeted) {
    let targ = match targeted {
        Targeted::Entity(EntityId::Mobile(m)) => Some(m),
        _ => None,
    };
    let Some(targ) = targ else {
        msg(w, from, 500399); // There is little chance of getting money from that!
        let t = tick(w);
        set_next_skill_time(w, from, t);
        return;
    };

    if w.mobile(targ).is_some_and(|m| m.player) {
        // We can't beg from players
        msg(w, from, 500398); // Perhaps just asking would work better.
    } else if !super::common::is_human(w, targ) {
        // Make sure the NPC is human
        msg(w, from, 500399); // There is little chance of getting money from that!
    } else if !crate::mx::in_range(w, from, targ, 2) {
        if !w.mobile(targ).is_some_and(|m| m.female) {
            msg(w, from, 500401); // You are too far away to beg from him.
        } else {
            msg(w, from, 500402); // You are too far away to beg from her.
        }
    } else if !core::ml(w) && props_b::mounted(w, from) {
        // If we're on a mount, who would give us money? TODO: guessed it's removed since ML
        msg(w, from, 500404); // They seem unwilling to give you any money.
    } else {
        mb::revealing_action(w, from);

        // Face each other
        let d = crate::mx::get_direction_to(w, from, targ);
        props_b::set_direction(w, from, d);
        let d = crate::mx::get_direction_to(w, targ, from);
        props_b::set_direction(w, targ, d);

        mb::animate(w, from, 32, 5, 1, true, false, 0); // Bow

        timer::start(w, TimeSpan::from_secs(2), TimeSpan::ZERO, 1, move |w| {
            begging_tick(w, from, targ)
        });
        return;
    }

    let t = tick(w);
    set_next_skill_time(w, from, t);
}

#[allow(clippy::unnecessary_unwrap)]
fn begging_tick(w: &mut World, from: MobileId, target: MobileId) {
    let their_pack = w.mobile(target).and_then(|m| m.backpack);

    let karma = w.mobile(from).map_or(0, |m| m.karma);
    let bad_karma_chance = 0.5 - f64::from(karma) / 8570.0;

    if their_pack.is_none() {
        msg(w, from, 500404); // They seem unwilling to give you any money.
    } else if karma < 0 && bad_karma_chance > utility::random_double(&mut w.rng) {
        // Thou dost not look trustworthy... no gold for thee today!
        let hue = w.mobile(target).map_or(0, |m| m.speech_hue);
        mm::public_overhead_message_number(w, target, MessageType::REGULAR, hue, 500406, "", true);
    } else if check_target_skill(
        w,
        from,
        SkillName::Begging,
        Targeted::Entity(EntityId::Mobile(target)),
        0.0,
        100.0,
    ) {
        let pack = their_pack.expect("checked above");
        let gold_ty = match w.types.find_type_by_full_name("Server.Items.Gold", false) {
            Some(muo_server::type_registry::EntityType::Item(t)) => Some(t),
            _ => None,
        };
        let mut to_consume = gold_ty.map_or(0, |t| cont::get_amount(w, pack, t, true) / 10);
        let fame = w.mobile(from).map_or(0, |m| m.fame);
        let max = (10 + fame / 2500).clamp(10, 14);

        if to_consume > max {
            to_consume = max;
        }

        let speech_hue = w.mobile(target).map_or(0, |m| m.speech_hue);
        if to_consume > 0 {
            let consumed = gold_ty.map_or(0, |t| cont::consume_up_to(w, pack, t, to_consume, true));

            if consumed > 0 {
                // I feel sorry for thee...
                mm::public_overhead_message_number(
                    w,
                    target,
                    MessageType::REGULAR,
                    speech_hue,
                    500405,
                    "",
                    true,
                );

                let gold = crate::items::gold::new_gold(w, consumed);

                effects_items_add_to_backpack(w, from, gold);
                let sound = muo_server::item::base::get_drop_sound(w, gold);
                effects::play_sound(w, from, sound);

                if karma > -3000 {
                    let mut to_lose = karma + 3000;

                    if to_lose > 40 {
                        to_lose = 40;
                    }

                    titles::award_karma(w, from, -to_lose, true);
                }
            } else {
                // I have not enough money to give thee any!
                mm::public_overhead_message_number(
                    w,
                    target,
                    MessageType::REGULAR,
                    speech_hue,
                    500407,
                    "",
                    true,
                );
            }
        } else {
            // I have not enough money to give thee any!
            mm::public_overhead_message_number(
                w,
                target,
                MessageType::REGULAR,
                speech_hue,
                500407,
                "",
                true,
            );
        }
    } else {
        msg(w, from, 500404); // They seem unwilling to give you any money.
    }

    const TARGETER_COOLDOWN: i64 = 30000; // 30s
    const SKILL_COOLDOWN: i64 = 10000; // 10s

    // Calculate how much time has passed since the targeter was opened
    let now = tick(w);
    let next = w.mobile(from).map_or(0, |m| m.next_skill_time);
    let ticks_since_targeter = now.wrapping_sub(next.wrapping_sub(TARGETER_COOLDOWN)) as i32;
    let remaining_cooldown = (SKILL_COOLDOWN as i32 - ticks_since_targeter).max(0);
    set_next_skill_time(w, from, now.wrapping_add(i64::from(remaining_cooldown)));
}

// ---- SpiritSpeak ------------------------------------------------------------------------------------------------

pub fn spirit_speak_on_use(w: &mut World, m: MobileId) -> TimeSpan {
    if core::aos(w) {
        let s = spell::new_spell(w, &SPIRIT_SPEAK_SPELL, m, None);
        spell::cast(w, s);

        if spell::is_casting(w, s) {
            return TimeSpan::from_secs(5);
        }

        return TimeSpan::ZERO;
    }

    mb::revealing_action(w, m);

    if check_skill(w, m, SkillName::SpiritSpeak, 0.0, 100.0) {
        if !w.mobile(m).is_some_and(|x| x.can_hear_ghosts) {
            let mut secs = skills::base(w, m, SkillName::SpiritSpeak) / 50.0;
            secs *= 90.0;
            if secs < 15.0 {
                secs = 15.0;
            }

            // The SpiritSpeakTimer starts at 2 minutes, then `t.Delay` is set to 15 seconds to 3 minutes.
            timer::start(
                w,
                TimeSpan::from_secs_f64(secs),
                TimeSpan::ZERO,
                1,
                move |w| {
                    if let Some(x) = w.mobile_mut(m) {
                        x.can_hear_ghosts = false;
                    }
                    msg(w, m, 502445); // You feel your contact with the neitherworld fading.
                },
            );
            if let Some(x) = w.mobile_mut(m) {
                x.can_hear_ghosts = true;
            }
        }

        effects::play_sound(w, m, 0x24A);
        msg(w, m, 502444); // You contact the neitherworld.
    } else {
        msg(w, m, 502443); // You fail to contact the neitherworld.
        if let Some(x) = w.mobile_mut(m) {
            x.can_hear_ghosts = false;
        }
    }

    TimeSpan::from_secs(1)
}

fn ss_cast_delay_base(_: &World, _: SpellId) -> TimeSpan {
    TimeSpan::from_secs(1)
}

fn ss_get_mana(_: &World, _: SpellId) -> i32 {
    0
}

fn ss_on_caster_hurt(w: &mut World, id: SpellId) {
    if spell::is_casting(w, id) {
        spell::disturb(w, id, DisturbType::Hurt, false, true);
    }
}

fn ss_consume_reagents(_: &mut World, _: SpellId) -> bool {
    true
}

fn ss_check_fizzle(_: &mut World, _: SpellId) -> bool {
    true
}

fn ss_on_disturb(w: &mut World, id: SpellId, t: DisturbType, message: bool) {
    if let Some(c) = spell::caster(w, id) {
        let now = tick(w);
        set_next_skill_time(w, c, now);
    }
    spell::base_on_disturb(w, id, t, message);
}

fn ss_check_disturb(_: &World, _: SpellId, t: DisturbType, _: bool, _: bool) -> bool {
    t != DisturbType::EquipRequest && t != DisturbType::UseRequest
}

fn ss_say_mantra(w: &mut World, id: SpellId) {
    let Some(c) = spell::caster(w, id) else {
        return;
    };
    // Anh Mi Sah Ko
    mm::public_overhead_message_number(w, c, MessageType::REGULAR, 0x3B2, 1062074, "", false);
    effects::play_sound(w, c, 0x24A);
}

fn ss_on_cast(w: &mut World, id: SpellId) {
    let Some(caster) = spell::caster(w, id) else {
        return;
    };

    // `Caster.GetItemsInRange<Corpse>(3)`: the first corpse nobody has channeled.
    let to_channel = enumerators::get_items_in_range(w, caster, 3)
        .into_iter()
        .find(|&i| item_is(w, i, "Server.Items.Corpse") && !ops(w).corpse_channeled(w, i));

    let mut min = 1 + (skills::value(w, caster, SkillName::SpiritSpeak) * 0.25) as i32;
    let max = min + 4;

    let (mana, number) = if to_channel.is_some() {
        (0, 1061287) // You channel energy from a nearby corpse to heal your wounds.
    } else {
        (10, 1061286) // You channel your own spiritual energy to heal your wounds.
    };

    if w.mobile(caster).map_or(0, |m| m.mana) < mana {
        msg(w, caster, 1061285); // You lack the mana required to use this skill.
    } else {
        let cap = skills::cap(w, caster, SkillName::SpiritSpeak);
        let _ = cap;
        check_skill(w, caster, SkillName::SpiritSpeak, 0.0, 120.0);

        let skill = skills::value(w, caster, SkillName::SpiritSpeak);

        if skill < 100.0 && utility::random_double(&mut w.rng) >= skill / 100.0 {
            msg(w, caster, 502443); // You fail your attempt at contacting the netherworld.
        } else {
            if let Some(c) = to_channel {
                ops(w).channel_corpse(w, c);
            }

            let cur = w.mobile(caster).map_or(0, |m| m.mana);
            muo_server::mobile::part1::props_b::set_mana(w, caster, cur - mana); // the Mana setter: delta, regen timer
            msg(w, caster, number);

            if min > max {
                min = max;
            }

            let heal = utility::random_min_max(&mut w.rng, min, max);
            let hits = w.mobile(caster).map_or(0, |m| m.hits);
            props_b::set_hits(w, caster, hits + heal);

            effects::fixed_particles_hue(
                w,
                caster,
                0x375A,
                1,
                15,
                9501,
                2100,
                4,
                muo_server::effects::EffectLayer::Waist,
            );
        }
    }

    spell::finish_sequence(w, id);
}

/// `SpiritSpeakSpell`: the AOS form of the skill, a spell with no mantra cost, no reagents and no fizzle.
pub static SPIRIT_SPEAK_SPELL: SpellType = SpellType {
    full_name: "Server.SkillHandlers.SpiritSpeak+SpiritSpeakSpell",
    parent: Some(&SPELL_BASE),
    info: SpellInfo::new("Spirit Speak", "", &[]).action(269),
    blocked_by_horrific_beast: false,
    clear_hands_on_cast: false,
    cast_delay_fast_scalar: 0.0,
    cast_delay_base: ss_cast_delay_base,
    check_next_spell_time: |_, _| false,
    get_mana: ss_get_mana,
    on_caster_hurt: ss_on_caster_hurt,
    consume_reagents: ss_consume_reagents,
    check_fizzle: ss_check_fizzle,
    on_disturb: ss_on_disturb,
    check_disturb: ss_check_disturb,
    say_mantra: ss_say_mantra,
    on_cast: ss_on_cast,
    ..SPELL_BASE
};

// ---- DetectHidden -----------------------------------------------------------------------------------------------

const PASSIVE_DETECT_DEBOUNCE_MS: i64 = 3000; // 3 seconds
const DEBOUNCE_CLEANUP_INTERVAL_MS: i64 = 10000; // Run cleanup every 10 seconds
const DEBOUNCE_EXPIRY_MS: i64 = 10000; // Remove entries older than 10 seconds

/// The statics of `DetectHidden`.
pub struct DetectHiddenSettings {
    /// `ContentFeatureFlags.PassiveDetectHidden`.
    pub passive_detect_hidden: bool,
    /// (stealther, detector) -> last detection time.
    debounce: IndexMap<(MobileId, MobileId), i64>,
    last_cleanup_time: i64,
}

impl Default for DetectHiddenSettings {
    fn default() -> Self {
        DetectHiddenSettings {
            passive_detect_hidden: true, // ContentFeatureFlags.PassiveDetectHidden = true
            debounce: IndexMap::new(),
            last_cleanup_time: 0,
        }
    }
}

pub fn detect_hidden_on_use(w: &mut World, src: MobileId) -> TimeSpan {
    msg(w, src, 500819); // Where will you search?
    let mut t = Target::new(w, 12, true, TargetFlags::NONE);
    t.handlers.on_target = Some(std::rc::Rc::new(detect_hidden_on_target));
    t.handlers.on_target_cancel = Some(std::rc::Rc::new(
        |w: &mut World, from: MobileId, _: TargetCancelType| {
            let now = tick(w);
            set_next_skill_time(w, from, now);
        },
    ));
    let id = target::add_target(w, t);
    target::set_mobile_target(w, src, Some(id));

    TimeSpan::from_secs(30)
}

/// `ClearDebounceCache()`: for tests.
pub fn clear_debounce_cache(w: &mut World) {
    w.resources
        .get_mut::<DetectHiddenSettings>()
        .debounce
        .clear();
}

/// `TryDetectStealther(detector, stealther)`: passive detection when either party moves within range. True if the
/// stealther was revealed.
pub fn try_detect_stealther(w: &mut World, detector: MobileId, stealther: MobileId) -> bool {
    let enabled = w
        .resources
        .get_mut::<DetectHiddenSettings>()
        .passive_detect_hidden;
    let (hidden, st_map) = w
        .mobile(stealther)
        .map_or((false, None), |m| (m.hidden, m.map));
    let det_alive = w
        .mobile(detector)
        .is_some_and(|m| (m.ty.alive)(w, detector));
    if !enabled || stealther == detector || !hidden || !det_alive {
        return false;
    }

    // Felucca PvP only
    if st_map != Some(MapId::FELUCCA) {
        return false;
    }

    let detect_skill = skills::value(w, detector, SkillName::DetectHidden);
    if detect_skill <= 0.0 {
        return false;
    }

    let now = tick(w);

    // Debounce: skip pairs already checked recently (cheap lookup before expensive checks)
    let key = (stealther, detector);
    let st = w.resources.get_mut::<DetectHiddenSettings>();
    if st
        .debounce
        .get(&key)
        .is_some_and(|&last| now.wrapping_sub(last) < PASSIVE_DETECT_DEBOUNCE_MS)
    {
        return false;
    }

    // Periodic cleanup (amortized, not every call)
    if now.wrapping_sub(st.last_cleanup_time) > DEBOUNCE_CLEANUP_INTERVAL_MS {
        st.last_cleanup_time = now;
        st.debounce
            .retain(|_, &mut v| now.wrapping_sub(v) <= DEBOUNCE_EXPIRY_MS);
    }

    // Excludes blessed, dead, bonded pets, and region-based PvP rules
    if w.mobile(stealther)
        .is_some_and(|m| m.access_level > AccessLevel::Player)
        || !mb::can_be_harmful_msg(w, detector, stealther, false)
    {
        return false;
    }

    // Exclude party members
    if let Some(f) = spells::hooks::hooks(w).and_then(|h| h.party_contains)
        && f(w, stealther, detector)
    {
        return false;
    }

    // Exclude guild members and allies
    if let Some(g) = spells::hooks::hooks(w).and_then(|h| h.guild_of) {
        let (sg, dg) = (g(w, stealther), g(w, detector));
        if let (Some(sg), Some(dg)) = (sg, dg)
            && (sg == dg
                || spells::hooks::hooks(w)
                    .and_then(|h| h.guild_is_ally)
                    .is_some_and(|a| a(w, sg, dg)))
        {
            return false;
        }
    }

    let ss = detect_skill + f64::from(utility::random(&mut w.rng, 21)) - 10.0;
    let ts = skills::value(w, stealther, SkillName::Hiding)
        + f64::from(utility::random(&mut w.rng, 21))
        - 10.0;

    if ss >= ts {
        mb::revealing_action(w, stealther);
        msg(w, stealther, 500814); // You have been revealed!
        w.resources
            .get_mut::<DetectHiddenSettings>()
            .debounce
            .insert(key, now);
        return true;
    }

    false
}

fn detect_hidden_on_target(w: &mut World, src: MobileId, targ: Targeted) {
    let mut found_anyone = false;
    let src_skill = skills::value(w, src, SkillName::DetectHidden);
    let mut range = (src_skill / 10.0) as i32;

    let container = entity(targ).and_then(|e| match e {
        EntityId::Item(i) => ops(w)
            .trappable_container(w, i)
            .filter(|&(trapped, _)| trapped)
            .map(|_| i),
        _ => None,
    });

    if let Some(container) = container {
        // Direct container targeting: show [trapped] if within detection range and skill check passes
        let loc = muo_server::item::ops::get_world_location(w, container);
        if mb::in_range_point3d(w, src, loc, range)
            && check_skill(w, src, SkillName::DetectHidden, 0.0, 100.0)
        {
            if let (Some(ns), Some(i)) =
                (w.mobile(src).and_then(|m| m.net_state), w.item(container))
            {
                let (serial, graphic) = (i.serial, i.item_id);
                muo_server::packets::message::send_message_localized(
                    w,
                    ns,
                    serial,
                    graphic,
                    muo_server::packets::message::MessageType::REGULAR,
                    0x3B2,
                    3,
                    500813,
                    "",
                    "",
                );
            }
            found_anyone = true;
        }
    } else if entity(targ).is_none_or(|e| matches!(e, EntityId::Mobile(m) if m == src)) {
        // Area scan only when targeting self or the ground
        let p = match targ {
            Targeted::Entity(EntityId::Mobile(m)) => {
                w.mobile(m).map(|x| x.location).unwrap_or_default()
            }
            Targeted::Land(l) => l.location,
            Targeted::Static(s) => s.location,
            _ => w.mobile(src).map(|x| x.location).unwrap_or_default(),
        };

        if !check_skill(w, src, SkillName::DetectHidden, 0.0, 100.0) {
            range /= 2;
        }

        let map = w.mobile(src).and_then(|m| m.map);
        let house = ops(w).find_house_at_point(w, p, map, 16);
        let in_house = house.is_some_and(|h| ops(w).house_is_friend(w, h, src));

        if in_house {
            range = 22;
        }

        if range > 0 {
            let nearby: Vec<MobileId> =
                muo_server::map::enumerate::mobiles_in_range(w, map, p.x, p.y, range).collect();
            for trg in nearby {
                if !w.mobile(trg).is_some_and(|m| m.hidden) || src == trg {
                    continue;
                }

                let ss = src_skill + f64::from(utility::random(&mut w.rng, 21)) - 10.0;
                let ts = skills::value(w, trg, SkillName::Hiding)
                    + f64::from(utility::random(&mut w.rng, 21))
                    - 10.0;

                let (src_level, trg_level) = (
                    w.mobile(src)
                        .map_or(AccessLevel::Player, |m| m.access_level),
                    w.mobile(trg)
                        .map_or(AccessLevel::Player, |m| m.access_level),
                );
                let inside = house.is_some_and(|h| ops(w).is_inside_mobile(w, h, trg));
                if src_level < trg_level || (ss < ts && (!in_house || !inside)) {
                    continue;
                }

                let tloc = w.mobile(trg).map(|m| m.location).unwrap_or_default();
                if super::common::mobile_is(w, trg, "Server.Mobiles.ShadowKnight")
                    && (tloc.x != p.x || tloc.y != p.y)
                {
                    continue;
                }

                mb::revealing_action(w, trg);
                msg(w, trg, 500814); // You have been revealed!
                found_anyone = true;
            }

            let items: Vec<ItemId> =
                muo_server::map::enumerate::items_in_range(w, map, p.x, p.y, range).collect();
            for item in items {
                if ops(w).is_faction_trap(w, item) {
                    if ops(w).detect_faction_trap(w, src, item) {
                        found_anyone = true;
                    }
                } else if ops(w).is_hidden_trap(w, item) {
                    // High Seas (Publish 79): Requires 75 Detect Hidden to detect dungeon traps
                    if core::hs(w) && src_skill < 75.0 {
                        continue;
                    }

                    ops(w).set_trap_visible(w, item, true);
                    timer::start(w, TimeSpan::from_secs(10), TimeSpan::ZERO, 1, move |w| {
                        if w.item(item).is_some_and(|i| !i.deleted()) {
                            ops(w).set_trap_visible(w, item, false);
                        }
                    });

                    found_anyone = true;
                }
            }
        }
    }

    if !found_anyone {
        msg(w, src, 500817); // You can see nothing hidden there.
    }

    // Calculate how much time has passed since the targeter was opened
    let now = tick(w);
    let next = w.mobile(src).map_or(0, |m| m.next_skill_time);
    let ticks_since_targeter = now.wrapping_sub(next.wrapping_sub(30000)) as i32;
    let remaining_cooldown = (10000 - ticks_since_targeter).max(0);
    set_next_skill_time(w, src, now.wrapping_add(i64::from(remaining_cooldown)));
}

// ---- RemoveTrap -------------------------------------------------------------------------------------------------

pub fn remove_trap_on_use(w: &mut World, m: MobileId) -> TimeSpan {
    if skills::value(w, m, SkillName::Lockpicking) < 50.0 {
        msg(w, m, 502366); // You do not know enough about locks.  Become better at picking locks.
    } else if skills::value(w, m, SkillName::DetectHidden) < 50.0 {
        msg(w, m, 502367); // You are not perceptive enough.  Become better at detect hidden.
    } else {
        set_target(w, m, 2, TargetOptions::default(), remove_trap_on_target);
        msg(w, m, 502368); // Which trap will you attempt to disarm?
    }

    TimeSpan::from_secs(10) // 10 second delay before being able to re-use a skill
}

fn remove_trap_on_target(w: &mut World, from: MobileId, targeted: Targeted) {
    if let Targeted::Entity(EntityId::Mobile(_)) = targeted {
        msg(w, from, 502816); // You feel that such an action would be inappropriate
        return;
    }
    let Targeted::Entity(EntityId::Item(item)) = targeted else {
        msg(w, from, 502373); // That doesn't appear to be trapped
        return;
    };

    if let Some((trapped, power)) = ops(w).trappable_container(w, item) {
        if !trapped {
            msg(w, from, 502373); // That doesn't appear to be trapped
            return;
        }

        if muo_server::item::ops::root_parent(w, item) == Some(EntityId::Mobile(from)) {
            let loc = muo_server::item::ops::get_world_location(w, item);
            let d = effects::get_direction_to(w, from, loc.x, loc.y, false);
            props_b::set_direction(w, from, d);
        }

        effects::play_sound(w, from, 0x241);

        if check_target_skill(
            w,
            from,
            SkillName::RemoveTrap,
            targeted,
            f64::from(power),
            f64::from(power + 30),
        ) {
            ops(w).disarm_trappable_container(w, item);
            msg(w, from, 502377); // You successfully render the trap harmless
        } else {
            msg(w, from, 502372); // You fail to disarm the trap... but you don't set it off
        }
    } else if ops(w).is_faction_trap(w, item) {
        ops(w).disarm_faction_trap(w, from, item);
    } else {
        msg(w, from, 502373); // That doesn't appear to be trapped
    }
}

// ---- Snooping ---------------------------------------------------------------------------------------------------

/// `CheckSnoopAllowed(from, to)`.
pub fn check_snoop_allowed(w: &mut World, from: MobileId, to: MobileId) -> bool {
    let map = w.mobile(from).and_then(|m| m.map);

    if w.mobile(to).is_some_and(|m| m.player) {
        return mb::can_be_harmful_full(w, from, to, false, true); // normal restrictions
    }

    let harmful_restrictions = map.and_then(|m| w.maps.get(m)).is_some_and(|m| {
        m.rules
            .contains(muo_server::map::MapRules::HARMFUL_RESTRICTIONS)
    });
    if !harmful_restrictions {
        return true; // felucca you can snoop anybody
    }

    let region = w.mobile(to).map(|m| m.region);
    let guarded = region.and_then(|r| crate::regions::get_guarded(w, r));
    let guards_disabled = guarded.is_some_and(|g| crate::regions::guarded::is_disabled(w, g));
    if !guards_disabled {
        return true; // not in town? we can snoop any npc
    }

    !super::common::is_human(w, to)
        || (creature::is_creature(w, to)
            && creature::ctype(w, to)
                .is_some_and(|c| (c.always_attackable)(w, to) || (c.always_murderer)(w, to)))
}

/// `Container_Snoop(cont, from)`: `Container.SnoopHandler`.
pub fn container_snoop(w: &mut World, cont_id: ItemId, from: MobileId) {
    let from_level = w
        .mobile(from)
        .map_or(AccessLevel::Player, |m| m.access_level);
    let loc = muo_server::item::ops::get_world_location(w, cont_id);
    if from_level <= AccessLevel::Player && !mb::in_range_point3d(w, from, loc, 1) {
        msg(w, from, 500446); // That is too far away.
        return;
    }

    let root = match muo_server::item::ops::root_parent(w, cont_id) {
        Some(EntityId::Mobile(m)) => Some(m),
        _ => None,
    };

    if root.is_some_and(|r| !w.mobile(r).is_some_and(|m| (m.ty.alive)(w, r))) {
        return;
    }

    let root_level = root.and_then(|r| w.mobile(r)).map(|m| m.access_level);
    let denied = match root {
        Some(r) => root_level > Some(AccessLevel::Player) || !check_snoop_allowed(w, from, r),
        // `CheckSnoopAllowed(from, null)` has no answer; a container nobody carries cannot be snooped.
        None => true,
    };
    if denied {
        msg(w, from, 1001018); // You cannot perform negative acts on your target.
        return;
    }

    if from_level == AccessLevel::Player {
        let snooping = skills::value(w, from, SkillName::Snooping);
        if let Some(root) = root
            && snooping < 100.0
            && snooping < utility::random_double(&mut w.rng) * 100.0
        {
            let map = w.mobile(from).and_then(|m| m.map);
            if map.is_some() {
                let name = |w: &World, m: MobileId| {
                    w.mobile(m)
                        .and_then(|x| (x.ty.name)(w, m))
                        .unwrap_or_default()
                };
                let message = format!(
                    "You notice {} attempting to peek into {}'s belongings.",
                    name(w, from),
                    name(w, root)
                );
                let floc = w.mobile(from).map(|m| m.location).unwrap_or_default();

                let clients: Vec<_> =
                    muo_server::map::enumerate::clients_in_range(w, map, floc.x, floc.y, 8)
                        .collect();
                for ns in clients {
                    let Some(m) = w.net_mobile(ns) else { continue };
                    if m != from {
                        mm::send_message(w, m, &message);
                    }
                }
            }
        }

        titles::award_karma(w, from, -4, true);
    }

    if from_level > AccessLevel::Player
        || check_target_skill(
            w,
            from,
            SkillName::Snooping,
            Targeted::Entity(EntityId::Item(cont_id)),
            0.0,
            100.0,
        )
    {
        if ops(w).execute_trap(w, cont_id, from) {
            return;
        }

        ops(w).display_container_to(w, cont_id, from);
    } else {
        msg(w, from, 500210); // You failed to peek into the container.

        if skills::value(w, from, SkillName::Hiding) / 2.0
            < utility::random_double(&mut w.rng) * 100.0
        {
            mb::revealing_action(w, from);
        }
    }
}

pub fn initialize(w: &mut World) {
    set_callback(w, SkillName::Begging, begging_on_use);
    set_callback(w, SkillName::SpiritSpeak, spirit_speak_on_use);
    set_callback(w, SkillName::DetectHidden, detect_hidden_on_use);
    set_callback(w, SkillName::RemoveTrap, remove_trap_on_use);
}

#[allow(unused)]
fn _unused() {
    let _ = (local_msg, msg_args, Point3D::default());
    let _ = titles::award_karma;
}

fn effects_items_add_to_backpack(w: &mut World, m: MobileId, item: ItemId) {
    muo_server::mobile::part2::items::add_to_backpack(w, m, item);
}

#[cfg(test)]
mod tests {
    use super::*;
    use muo_server::mobile::MOBILE_BASE;

    #[test]
    fn detect_hidden_opens_a_ground_target_and_waits_thirty_seconds() {
        let mut w = World::new();
        let m = w.new_mobile(&MOBILE_BASE);
        assert_eq!(detect_hidden_on_use(&mut w, m), TimeSpan::from_secs(30));
        let t = w.mobile(m).unwrap().target.unwrap();
        assert_eq!(
            (w.targets[&t].range, w.targets[&t].allow_ground),
            (12, true)
        );
    }

    // ---- Skills/DetectHiddenTests: all coordinates at Y=500 on Felucca ----

    fn detect_world() -> World {
        use muo_server::skills::SkillTable;
        let mut w = muo_server::map::test_support::world();
        let info = |i: i32| muo_data::skill_info::SkillInfo {
            skill_id: i,
            name: format!("S{i}"),
            title: String::new(),
            str_scale: 0.0,
            dex_scale: 0.0,
            int_scale: 0.0,
            stat_total: 0.0,
            str_gain: 0.0,
            dex_gain: 0.0,
            int_gain: 0.0,
            gain_factor: 0.0,
            profession_skill_name: String::new(),
            primary_stat: muo_data::skill_info::Stat::Str,
            secondary_stat: muo_data::skill_info::Stat::Dex,
        };
        w.resources.get_mut::<SkillTable>().info = (0..58).map(info).collect();
        w
    }

    fn create_player(w: &mut World, x: i32) -> MobileId {
        let m = crate::player_mobile::new_player_mobile(w);
        muo_server::mobile::part1::lifecycle::move_to_world(
            w,
            m,
            muo_primitives::Point3D::new(x, 500, 0),
            MapId::FELUCCA,
        );
        m
    }

    /// `PredictableRandom(10)`, then the detector and the hidden stealther next to it.
    fn detect(detect_fixed: i32, hiding_fixed: i32, x: i32, elf: bool) -> (bool, bool) {
        let mut w = detect_world();
        w.rng.set_predictable(Some(10));
        clear_debounce_cache(&mut w);
        let detector = create_player(&mut w, x);
        let stealther = create_player(&mut w, x + 1);
        if elf {
            // Humans get a 20.0 racial bonus (Jack of All Trades)
            muo_server::mobile::part1::props_a::set_race(
                &mut w,
                detector,
                Some(crate::player_mobile::RACE_ELF),
            );
        }
        skills::set_base_fixed(&mut w, detector, SkillName::DetectHidden, detect_fixed);
        skills::set_base_fixed(&mut w, stealther, SkillName::Hiding, hiding_fixed);
        muo_server::mobile::part1::props_b::set_hidden(&mut w, stealther, true);

        let result = try_detect_stealther(&mut w, detector, stealther);
        (result, w.mobile(stealther).unwrap().hidden)
    }

    // A detector with higher Detect Hidden than the stealther's Hiding reveals them: with PredictableRandom both
    // rolls get the same offset, so detection succeeds when detectSkill >= hiding.
    #[test]
    fn try_detect_stealther_reveals_when_detect_skill_exceeds_hiding() {
        assert_eq!(detect(600, 500, 1000, false), (true, false));
    }

    // When detect skill exactly equals hiding, detection succeeds (ss >= ts).
    #[test]
    fn try_detect_stealther_reveals_when_skills_are_equal() {
        assert_eq!(detect(500, 500, 1200, false), (true, false));
    }

    #[test]
    fn try_detect_stealther_does_not_reveal_when_hiding_exceeds_detect_skill() {
        assert_eq!(detect(500, 600, 1400, false), (false, true));
    }

    // A detector with zero Detect Hidden skill never passively detects anyone (an Elf: Humans get the bonus).
    #[test]
    fn try_detect_stealther_does_not_reveal_when_detector_has_no_skill() {
        assert_eq!(detect(0, 0, 1600, true), (false, true));
    }

    #[test]
    fn passive_detection_is_off_unless_the_flag_is_set() {
        let mut w = World::new();
        let a = w.new_mobile(&MOBILE_BASE);
        let b = w.new_mobile(&MOBILE_BASE);
        w.mobile_mut(b).unwrap().hidden = true;
        assert!(!try_detect_stealther(&mut w, a, b));
        clear_debounce_cache(&mut w);
    }

    #[test]
    fn begging_and_remove_trap_set_their_targets() {
        let mut w = World::new();
        let m = w.new_mobile(&MOBILE_BASE);
        assert_eq!(begging_on_use(&mut w, m), TimeSpan::from_secs(30));
        assert_eq!(w.targets[&w.mobile(m).unwrap().target.unwrap()].range, 12);
        // RemoveTrap needs 50 Lockpicking and Detect Hidden first.
        assert_eq!(remove_trap_on_use(&mut w, m), TimeSpan::from_secs(10));
    }

    #[test]
    fn the_spirit_speak_spell_is_not_a_magery_spell() {
        assert!(SPIRIT_SPEAK_SPELL.is(&SPELL_BASE));
        assert!(
            SPIRIT_SPEAK_SPELL
                .family::<crate::spells::magery::MageryType>()
                .is_none()
        );
        assert!(
            !SPIRIT_SPEAK_SPELL.clear_hands_on_cast
                && !SPIRIT_SPEAK_SPELL.blocked_by_horrific_beast
        );
        assert_eq!(SPIRIT_SPEAK_SPELL.info.action, 269);
    }
}
