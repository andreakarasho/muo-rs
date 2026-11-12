//! The Animal Form spell, `AnimalFormEntry`, `AnimalFormGump`, `AnimalFormContext` and
//! `AnimalFormTimer`.

use indexmap::IndexMap;
use muo_content::core;
use muo_content::gumps::base::{DynamicGumpLogic, DynamicGumpObj, RelayInfo};
use muo_content::gumps::builder::DynamicGumpBuilder;
use muo_content::gumps::layout::GumpButtonType;
use muo_content::items::weapon_hooks::WeaponHooks;
use muo_content::player_mobile::hooks as pm_hooks;
use muo_content::spells::SpellInfo;
use muo_content::spells::helper;
use muo_content::spells::hooks::{hooks, hooks_mut};
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_content::text::add_html_text_builder;
use muo_gumps::m_z::common::draw_builder;
use muo_io::text::TextDefinition;
use muo_primitives::{Body, MessageType, TimeSpan, utility};
use muo_server::World;
use muo_server::effects::EffectLayer;
use muo_server::ids::MobileId;
use muo_server::mobile::base as mb;
use muo_server::mobile::part1::{combat, props_b};
use muo_server::mobile::part2::{effects, messages as mm, stats};
use muo_server::mods::{ModId, SkillMod};
use muo_server::net_state::NetStateId;
use muo_server::packets::movement::{SpeedControlSetting, send_speed_control};
use muo_server::skills::{self, SkillName};
use muo_server::timer::{self, TimerHandle};

use super::ninja::{NINJA_SPELL, NinjaType, required_mana, scale_mana_of};
use crate::common::{alive, is_player_mobile, revealing_action};

/// `BaseCreature.DefaultRangePerception`: how far the Reptalon breath reaches.
const REPTALON_RANGE: i32 = muo_content::creature::enums::DEFAULT_RANGE_PERCEPTION;

static ANIMAL_FORM: NinjaType = NinjaType {
    required_skill: 0.0,
    required_skill_pre_ml: None,
    required_mana: 10,
    required_mana_pre_ml: Some(0),
};

/// `AnimalForm`.
pub static ANIMAL_FORM_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Ninjitsu.AnimalForm",
    parent: Some(&NINJA_SPELL),
    info: SpellInfo::new("Animal Form", "", &[])
        .action(-1)
        .hand_effect(9002),
    cast_delay_base: |_, _| TimeSpan::from_secs(1),
    cast_recovery_base: 10,
    cast_recovery_base_pre_ml: Some(7),
    blocked_by_animal_form: false,
    check_cast,
    check_disturb: |_, _, _, _, _| false,
    on_begin_cast,
    check_fizzle: |_, _| true,
    on_cast,
    family: Some(&ANIMAL_FORM),
    ..NINJA_SPELL
};

/// `MorphResult`.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum MorphResult {
    Success,
    Fail,
    NoSkill,
}

/// `AnimalFormEntry`.
pub struct AnimalFormEntry {
    pub type_name: &'static str,
    pub name: i32,
    pub item_id: i32,
    pub hue: i32,
    pub tooltip: i32,
    pub req_skill: f64,
    pub body_mod: i32,
    pub hue_mod_min: i32,
    pub hue_mod_max: i32,
    pub stealth_bonus: bool,
    pub speed_boost: bool,
    pub stealing_bonus: bool,
}

#[allow(clippy::too_many_arguments)]
const fn e(
    type_name: &'static str,
    name: i32,
    item_id: i32,
    hue: i32,
    tooltip: i32,
    req_skill: f64,
    body_mod: i32,
    hue_mod_min: i32,
    hue_mod_max: i32,
    stealth_bonus: bool,
    speed_boost: bool,
    stealing_bonus: bool,
) -> AnimalFormEntry {
    AnimalFormEntry {
        type_name,
        name,
        item_id,
        hue,
        tooltip,
        req_skill,
        body_mod,
        hue_mod_min,
        hue_mod_max,
        stealth_bonus,
        speed_boost,
        stealing_bonus,
    }
}

/// `Entries`.
pub static ENTRIES: [AnimalFormEntry; 16] = [
    e(
        "Kirin", 1029632, 9632, 0, 1070811, 100.0, 0x84, 0, 0, false, true, false,
    ),
    e(
        "Unicorn", 1018214, 9678, 0, 1070812, 100.0, 0x7A, 0, 0, false, true, false,
    ),
    e(
        "BakeKitsune",
        1030083,
        10083,
        0,
        1070810,
        82.5,
        0xF6,
        0,
        0,
        false,
        true,
        false,
    ),
    e(
        "GreyWolf", 1028482, 9681, 2309, 1070810, 82.5, 0x19, 0x8FD, 0x90E, false, true, false,
    ),
    e(
        "Llama", 1028438, 8438, 0, 1070809, 70.0, 0xDC, 0, 0, false, true, false,
    ),
    e(
        "ForestOstard",
        1018273,
        8503,
        2212,
        1070809,
        70.0,
        0xDB,
        0x899,
        0x8B0,
        false,
        true,
        false,
    ),
    e(
        "BullFrog", 1028496, 8496, 2003, 1070807, 50.0, 0x51, 0x7D1, 0x7D6, false, false, false,
    ),
    e(
        "GiantSerpent",
        1018114,
        9663,
        2009,
        1070808,
        50.0,
        0x15,
        0x7D1,
        0x7E2,
        false,
        false,
        false,
    ),
    e(
        "Dog", 1018280, 8476, 2309, 1070806, 40.0, 0xD9, 0x8FD, 0x90E, false, false, false,
    ),
    e(
        "Cat", 1018264, 8475, 2309, 1070806, 40.0, 0xC9, 0x8FD, 0x90E, false, false, false,
    ),
    e(
        "Rat", 1018294, 8483, 2309, 1070805, 20.0, 0xEE, 0x8FD, 0x90E, true, false, false,
    ),
    e(
        "Rabbit", 1028485, 8485, 2309, 1070805, 20.0, 0xCD, 0x8FD, 0x90E, true, false, false,
    ),
    e(
        "Squirrel", 1031671, 11671, 0, 0, 20.0, 0x116, 0, 0, false, false, false,
    ),
    e(
        "Ferret", 1031672, 11672, 0, 1075220, 40.0, 0x117, 0, 0, false, false, true,
    ),
    e(
        "CuSidhe", 1031670, 11670, 0, 1075221, 60.0, 0x115, 0, 0, false, false, false,
    ),
    e(
        "Reptalon", 1075202, 11669, 0, 1075222, 90.0, 0x114, 0, 0, false, false, false,
    ),
];

/// `AnimalFormContext`.
#[derive(Clone)]
struct Context {
    timer: TimerHandle,
    mod_: Option<ModId>,
    speed_boost: bool,
    type_name: &'static str,
    stealing_mod: Option<ModId>,
}

#[derive(Default)]
struct State {
    /// `_lastAnimalForms`.
    last: IndexMap<MobileId, i32>,
    /// `_table`.
    table: IndexMap<MobileId, Context>,
    /// `_wasMoving` of the casting spell, by caster.
    was_moving: IndexMap<MobileId, bool>,
}

fn state(w: &mut World) -> &mut State {
    w.resources.get_mut::<State>()
}

fn state_ref(w: &World) -> Option<&State> {
    w.resources.get::<State>()
}

/// Sets the hooks: the player's, the weapon's and the spell base's, the login event and the death/delete events.
pub fn init(w: &mut World) {
    hooks_mut(w).animal_form_under = Some(under_transformation);
    w.resources.get_mut::<WeaponHooks>().animal_form_under = Some(|w, m, form| match form {
        None => under_transformation(w, m),
        Some(t) => under_transformation_of(w, m, t),
    });
    let h = w.resources.get_mut::<pm_hooks::SystemHooks>();
    h.animal_form_under = Some(under_transformation);
    h.animal_form_under_type = Some(under_transformation_of);
    h.animal_form_speed_boost = Some(|w, m| {
        state_ref(w)
            .and_then(|s| s.table.get(&m))
            .is_some_and(|c| c.speed_boost)
    });
    h.animal_form_remove_context = Some(remove_context);
    h.animal_form_start_timer = Some(|_, _| {
        // `GetContext(m)?.Timer.Start()`: the timer runs from the moment it is made here.
    });

    muo_content::player_mobile::subscribe_deleted(w, |w, m| {
        remove_context(w, m);
        state(w).table.shift_remove(&m); // RemoveLastAnimalForm (it removes from `_table`)
    });
    muo_content::player_mobile::subscribe_death(w, remove_context);
    muo_content::network::events::subscribe_ranked::<muo_content::network::events::PlayerLogin>(
        w,
        23,
        |w, e| on_login(w, e.0),
    );
}

/// `OnLogin(pm)`.
fn on_login(w: &mut World, pm: MobileId) {
    if state_ref(w)
        .and_then(|s| s.table.get(&pm))
        .is_some_and(|c| c.speed_boost)
        && let Some(ns) = w.mobile(pm).and_then(|m| m.net_state)
    {
        send_speed_control(w, ns, SpeedControlSetting::Mount);
    }
}

fn check_cast(w: &mut World, sp: SpellId) -> bool {
    let Some(caster) = spell::caster(w, sp) else {
        return false;
    };
    if !combat::can_begin_action(w, caster, "Server.Spells.Seventh.PolymorphSpell") {
        mm::send_localized_message(w, caster, 1061628, "", 0x3B2); // You can't do that while polymorphed.
        return false;
    }

    if helper::under_transformation(w, caster) {
        mm::send_localized_message(w, caster, 1063219, "", 0x3B2); // You cannot mimic an animal while in that form.
        return false;
    }

    if hooks(w)
        .and_then(|h| h.is_disguised)
        .is_some_and(|f| f(w, caster))
    {
        mm::send_localized_message(w, caster, 1061631, "", 0x3B2); // You can't do that while disguised.
        return false;
    }

    (NINJA_SPELL.check_cast)(w, sp)
}

fn caster_is_moving(w: &World, caster: MobileId) -> bool {
    w.mobile(caster).is_some_and(|c| {
        w.tick_count - c.last_move_time
            <= i64::from(mb::compute_movement_speed(w, caster, c.direction, false))
    })
}

fn on_begin_cast(w: &mut World, sp: SpellId) {
    (NINJA_SPELL.on_begin_cast)(w, sp);
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };
    effects::fixed_effect_hue(w, caster, 0x37C4, 10, 14, 4, 3);
    let moving = caster_is_moving(w, caster);
    state(w).was_moving.insert(caster, moving);
}

fn on_cast(w: &mut World, sp: SpellId) {
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };
    if !combat::can_begin_action(w, caster, "Server.Spells.Seventh.PolymorphSpell") {
        mm::send_localized_message(w, caster, 1061628, "", 0x3B2); // You can't do that while polymorphed.
    } else if helper::under_transformation(w, caster) {
        mm::send_localized_message(w, caster, 1063219, "", 0x3B2); // You cannot mimic an animal while in that form.
    } else if !combat::can_begin_action(w, caster, "Server.Spells.Fifth.IncognitoSpell")
        || (w.mobile(caster).is_some_and(|c| c.body_mod.body_id() != 0)
            && get_context(w, caster).is_none())
    {
        let ty = spell::spell_type(w, sp).expect("the spell is casting");
        (ty.do_fizzle)(w, sp);
    } else if spell::check_sequence(w, sp) {
        let context = get_context(w, caster);

        let mana = scale_mana_of(w, sp, if core::ml(w) { required_mana(w, sp) } else { 0 });

        if mana > w.mobile(caster).map_or(0, |c| c.mana) {
            // You must have at least ~1_MANA_REQUIREMENT~ Mana to use this ability.
            mm::send_localized_message(w, caster, 1060174, &mana.to_string(), 0x3B2);
        } else if let Some(context) = context {
            remove_context_of(w, caster, &context);
            let cur = w.mobile(caster).map_or(0, |c| c.mana);
            props_b::set_mana(w, caster, cur - mana);
        }
        // On OSI, casting while standing still always opens the selection menu; casting while moving
        // quick-transforms into the last selected form without the menu.
        else if is_player_mobile(w, caster)
            && !state(w).was_moving.get(&caster).copied().unwrap_or(false)
            && !caster_is_moving(w, caster)
        {
            open_gump(w, caster, sp);
        } else {
            let last = get_last_animal_form(w, caster);
            let result = morph(w, caster, last);

            if result == MorphResult::Success {
                effects::fixed_particles(w, caster, 0x3728, 10, 13, 2023, EffectLayer::Waist);
                let cur = w.mobile(caster).map_or(0, |c| c.mana);
                props_b::set_mana(w, caster, cur - mana);
            } else if result == MorphResult::Fail {
                let ty = spell::spell_type(w, sp).expect("the spell is casting");
                (ty.do_fizzle)(w, sp);
            }
            // MorphResult.NoSkill: Morph already messaged the requirement; consume no mana.
        }
    }

    spell::finish_sequence(w, sp);
}

/// `GetLastAnimalForm(m)`.
pub fn get_last_animal_form(w: &mut World, m: MobileId) -> i32 {
    state(w).last.get(&m).copied().unwrap_or(-1)
}

/// `CanSelectEntry(m, entry)`: the Ninjitsu skill and the form talisman.
pub fn can_select_entry(w: &mut World, m: MobileId, entry: &AnimalFormEntry) -> bool {
    skills::value(w, m, SkillName::Ninjitsu) >= entry.req_skill
        && entry_enabled(w, m, entry.type_name)
}

/// `BaseFormTalisman.EntryEnabled(m, type)`. Unset: enabled.
fn entry_enabled(w: &World, m: MobileId, type_name: &str) -> bool {
    hooks(w)
        .and_then(|h| h.form_talisman_entry_enabled)
        .is_none_or(|f| f(w, m, type_name))
}

/// `Morph(m, entryID)`.
pub fn morph(w: &mut World, m: MobileId, entry_id: i32) -> MorphResult {
    if entry_id < 0 || entry_id as usize >= ENTRIES.len() {
        return MorphResult::Fail;
    }

    let entry = &ENTRIES[entry_id as usize];

    state(w).last.insert(m, entry_id); // On OSI, it's the last /attempted/ one not the last succeeded one

    let ninjitsu = skills::value(w, m, SkillName::Ninjitsu);
    if ninjitsu < entry.req_skill {
        // You need at least ~1_SKILL_REQUIREMENT~ ~2_SKILL_NAME~ skill to use that ability.
        mm::send_localized_message(
            w,
            m,
            1063013,
            &format!("{:.1}\t{:?}\t ", entry.req_skill, SkillName::Ninjitsu),
            0x3B2,
        );
        return MorphResult::NoSkill;
    }

    /* On OSI,it seems you can only gain starting at '0' using Animal form. */
    if ninjitsu < entry.req_skill + 37.5 {
        let chance = (ninjitsu - entry.req_skill) / 37.5;

        if chance < utility::random_double(&mut w.rng) {
            return MorphResult::Fail;
        }
    }

    stats::check_skill(w, m, SkillName::Ninjitsu, 0.0, 37.5);

    if !entry_enabled(w, m, entry.type_name) {
        return MorphResult::Success; // Still consumes mana, just no effect
    }

    if let Some(f) = w
        .resources
        .get::<WeaponHooks>()
        .and_then(|h| h.base_mount_dismount)
    {
        f(w, m);
    }

    let body_mod = entry.body_mod;
    let hue_mod = utility::random_min_max(&mut w.rng, entry.hue_mod_min, entry.hue_mod_max);

    props_b::set_body_mod(w, m, Body::new(body_mod));
    props_b::set_hue_mod(w, m, hue_mod);

    if entry.speed_boost
        && let Some(ns) = w.mobile(m).and_then(|x| x.net_state)
    {
        send_speed_control(w, ns, SpeedControlSetting::Mount);
    }

    // TODO: Determine if transform spell skill mods to a generic location like stat mods
    let mut mod_id = None;
    if entry.stealth_bonus {
        let mut sm =
            SkillMod::new_default(w, SkillName::Stealth, Some("StealthAnimalForm"), true, 20.0);
        sm.obey_cap = true;
        mod_id = Some(sm.id);
        mb::add_skill_mod(w, m, sm);
    }

    let mut stealing_mod = None;
    if entry.stealing_bonus {
        let mut sm = SkillMod::new_default(
            w,
            SkillName::Stealing,
            Some("StealingAnimalForm"),
            true,
            10.0,
        );
        sm.obey_cap = true;
        stealing_mod = Some(sm.id);
        mb::add_skill_mod(w, m, sm);
    }

    let timer = start_form_timer(w, m, body_mod, hue_mod);
    add_context(
        w,
        m,
        Context {
            timer,
            mod_: mod_id,
            speed_boost: entry.speed_boost,
            type_name: entry.type_name,
            stealing_mod,
        },
    );

    mb::check_stat_timers(w, m);

    MorphResult::Success
}

fn add_context(w: &mut World, m: MobileId, context: Context) {
    let check = context.type_name == "BakeKitsune" || context.type_name == "GreyWolf";
    state(w).table.insert(m, context);

    if check {
        mb::check_stat_timers(w, m);
    }
}

/// The animal form part of `AosSkillBonuses.CheckCancelMorph`: the form ends when Ninjitsu fell under its requirement.
pub fn check_cancel_morph(w: &mut World, m: MobileId) {
    let Some(c) = get_context(w, m) else { return };
    if let Some(e) = ENTRIES.iter().find(|e| e.type_name == c.type_name)
        && skills::value(w, m, SkillName::Ninjitsu) < e.req_skill
    {
        remove_context(w, m);
    }
}

/// `RemoveContext(m)`.
pub fn remove_context(w: &mut World, m: MobileId) {
    if let Some(context) = get_context(w, m) {
        remove_context_of(w, m, &context);
    }
}

/// `RemoveContext(m, context)`.
fn remove_context_of(w: &mut World, m: MobileId, context: &Context) {
    state(w).table.shift_remove(&m);

    if context.speed_boost
        && let Some(ns) = w.mobile(m).and_then(|x| x.net_state)
    {
        send_speed_control(w, ns, SpeedControlSetting::Disable);
    }

    if let Some(id) = context.mod_ {
        mb::remove_skill_mod(w, m, id);
    }
    if let Some(id) = context.stealing_mod {
        mb::remove_skill_mod(w, m, id);
    }

    props_b::set_hue_mod(w, m, -1);
    props_b::set_body_mod(w, m, Body::new(0));

    effects::fixed_particles(w, m, 0x3728, 10, 13, 2023, EffectLayer::Waist);

    let mut t = context.timer;
    t.cancel(w);
}

fn get_context(w: &mut World, m: MobileId) -> Option<Context> {
    state(w).table.get(&m).cloned()
}

/// `UnderTransformation(m)`.
pub fn under_transformation(w: &World, m: MobileId) -> bool {
    state_ref(w).is_some_and(|s| s.table.contains_key(&m))
}

/// `UnderTransformation(m, type)`: the form's class name.
pub fn under_transformation_of(w: &World, m: MobileId, type_name: &str) -> bool {
    state_ref(w)
        .and_then(|s| s.table.get(&m))
        .is_some_and(|c| c.type_name == type_name)
}

// ---- AnimalFormTimer ----

fn start_form_timer(w: &mut World, from: MobileId, body: i32, hue: i32) -> TimerHandle {
    let mut counter = 0i32;
    let mut last_target: Option<MobileId> = None;
    timer::start(
        w,
        TimeSpan::from_secs(1),
        TimeSpan::from_secs(1),
        0,
        move |w| {
            let gone = w.mobile(from).is_none_or(|m| {
                m.deleted
                    || !(m.ty.alive)(w, from)
                    || muo_server::mobile::part1::props_b::body(w, from).body_id() != body
                    || (m.ty.hue)(w, from) != hue
            });
            if gone {
                remove_context(w, from);
                return;
            }

            if body == 0x115 {
                // Cu Sidhe
                // Once every 8 seconds
                let c = counter;
                counter += 1;
                if c < 8 {
                    return;
                }

                let (hits, hits_max) =
                    (w.mobile(from).map_or(0, |m| m.hits), mb::hits_max(w, from));
                if hits < hits_max {
                    if let Some(f) = hooks(w).and_then(|h| h.consume_bandage)
                        && f(w, from)
                    {
                        let heal = utility::random_min_max(&mut w.rng, 20, 50);
                        props_b::set_hits(w, from, hits + heal);
                    }
                } else if w.mobile(from).is_none_or(|m| {
                    m.map.is_none() || m.map == Some(muo_server::map::MapId::INTERNAL)
                }) {
                    // Logged out
                    if let Some(h) = state(w).table.get(&from).map(|c| c.timer) {
                        let mut h = h;
                        h.cancel(w);
                    }
                }

                counter = 0;
            } else if body == 0x114 {
                // Reptalon
                // Logged out
                if w.mobile(from).is_none_or(|m| {
                    m.map.is_none() || m.map == Some(muo_server::map::MapId::INTERNAL)
                }) {
                    if let Some(h) = state(w).table.get(&from).map(|c| c.timer) {
                        let mut h = h;
                        h.cancel(w);
                    }
                    counter = 0;
                    return;
                }

                let combatant = w.mobile(from).and_then(|m| m.combatant);
                if combatant.is_some() && combatant != last_target {
                    counter = 1;
                    last_target = combatant;
                } else if w.mobile(from).is_some_and(|m| m.warmode)
                    && let Some(t) = last_target.filter(|&t| {
                        alive(w, t)
                            && w.mobile(t).is_some_and(|x| !x.deleted)
                            && w.mobile(t).map(|x| x.map) == w.mobile(from).map(|x| x.map)
                    })
                    && {
                        let c = counter;
                        counter -= 1;
                        c <= 0
                    }
                {
                    let in_range = w
                        .mobile(from)
                        .map(|m| m.location)
                        .is_some_and(|l| mb::in_range_point3d(w, t, l, REPTALON_RANGE));
                    if mb::can_be_harmful(w, from, t)
                        && in_range
                        && muo_server::item::deps::in_los_mobile(w, from, t)
                    {
                        let tl = w.mobile(t).map(|x| x.location).unwrap_or_default();
                        let d = effects::get_direction_to_point(
                            w,
                            from,
                            Some(muo_primitives::Point2D { x: tl.x, y: tl.y }),
                            false,
                        );
                        effects::set_direction(w, from, d);
                        mb::freeze(w, from, TimeSpan::from_secs(1));
                        effects::play_sound(w, from, 0x16A);

                        timer::start(
                            w,
                            TimeSpan::from_millis(1300),
                            TimeSpan::ZERO,
                            1,
                            move |w| breath_effect_callback(w, from, t),
                        );
                    }

                    counter = (super::super::common::distance_sqrt(w, from, t) as i32).min(10);
                }
            }
        },
    )
}

fn breath_effect_callback(w: &mut World, from: MobileId, target: MobileId) {
    if mb::can_be_harmful(w, from, target) {
        revealing_action(w, from);
        effects::play_sound(w, from, 0x227);
        muo_server::effects::send_moving_effect(w, from, target, 0x36D4, 5, 0, false, false, 0, 0);

        timer::start(w, TimeSpan::from_secs(1), TimeSpan::ZERO, 1, move |w| {
            breath_damage_callback(w, from, target)
        });
    }
}

fn breath_damage_callback(w: &mut World, from: MobileId, target: MobileId) {
    if mb::can_be_harmful(w, from, target) {
        revealing_action(w, from);
        mb::do_harmful(w, from, target, false);
        let player = w.mobile(target).is_some_and(|t| t.player);
        muo_content::misc::aos_helpers::damage(
            w,
            target,
            Some(from),
            20,
            muo_content::misc::aos_helpers::DamageSplit {
                fire: 100,
                ..Default::default()
            },
            muo_content::misc::aos_helpers::DamageOptions {
                ignore_armor: !player,
                ..Default::default()
            },
        );
    }
}

// ---- AnimalFormGump ----

/// `AnimalFormGump`.
pub struct AnimalFormGump {
    caster: MobileId,
    spell: SpellId,
    enabled: [bool; 16],
    /// `ItemBounds.Bounds[entry.ItemID]` as (x, y, width, height), read when the gump is made.
    bounds: [(i32, i32, i32, i32); 16],
}

/// The gump with the entries enabled for the caster (`CanSelectEntry`), read when it is made.
pub fn open_gump(w: &mut World, caster: MobileId, sp: SpellId) {
    let mut enabled = [false; 16];
    for (i, entry) in ENTRIES.iter().enumerate() {
        enabled[i] = can_select_entry(w, caster, entry);
    }
    muo_server::item::bounds::ensure_loaded(w);
    let mut bounds = [(0, 0, 0, 0); 16];
    for (i, entry) in ENTRIES.iter().enumerate() {
        let r = muo_server::item::bounds::bounds(w, entry.item_id as usize);
        bounds[i] = (r.x(), r.y(), r.width(), r.height());
    }
    let g = DynamicGumpObj::new(
        50,
        50,
        AnimalFormGump {
            caster,
            spell: sp,
            enabled,
            bounds,
        },
    );
    muo_content::gumps::system::send_gump_to_mobile(w, caster, Box::new(g), true);
}

impl DynamicGumpLogic for AnimalFormGump {
    const TYPE_NAME: &'static str = "Server.Spells.Ninjitsu.AnimalForm+AnimalFormGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        // TODO: Convert this for ML to the BaseImageTileButtonsGump
        b.add_page(0);

        b.add_background(0, 0, 520, 404, 0x13BE);
        b.add_image_tiled(10, 10, 500, 20, 0xA40);
        b.add_image_tiled(10, 40, 500, 324, 0xA40);
        b.add_image_tiled(10, 374, 500, 20, 0xA40);
        b.add_alpha_region(10, 10, 500, 384);

        b.add_html_localized_color(14, 12, 500, 20, 1063394, 0x7FFF, false, false); // <center>Polymorph Selection Menu</center>

        b.add_button(10, 374, 0xFB1, 0xFB2, 0);
        b.add_html_localized_color(45, 376, 450, 20, 1011012, 0x7FFF, false, false); // CANCEL

        let mut current = 0;

        for (i, entry) in ENTRIES.iter().enumerate() {
            let enabled = self.enabled[i];

            let page = current / 10 + 1;
            let pos = current % 10;

            if pos == 0 {
                if page > 1 {
                    b.add_button_type(400, 374, 0xFA5, 0xFA7, 0, GumpButtonType::Page, page);
                    b.add_html_localized_color(440, 376, 60, 20, 1043353, 0x7FFF, false, false); // Next
                }

                b.add_page(page);

                if page > 1 {
                    b.add_button_type(300, 374, 0xFAE, 0xFB0, 0, GumpButtonType::Page, 1);
                    b.add_html_localized_color(340, 376, 60, 20, 1011393, 0x7FFF, false, false); // Back
                }
            }

            if !enabled {
                continue;
            }

            let y = (pos / 2) * 64 + 44;
            let x = if pos % 2 == 0 { 14 } else { 264 };

            let (bx, by, bw, bh) = self.bounds[i];
            b.add_image_tiled_button(
                x,
                y,
                0x918,
                0x919,
                i as i32 + 1,
                GumpButtonType::Reply,
                0,
                entry.item_id,
                entry.hue,
                40 - bw / 2 - bx,
                30 - bh / 2 - by,
                entry.tooltip,
            );

            let name = TextDefinition::of_number(entry.name);
            draw_builder(
                b,
                add_html_text_builder(Some(&name), false, false, 0x7FFF, -1),
                x + 84,
                y,
                250,
                60,
            );

            current += 1;
        }
    }

    fn on_response(&mut self, w: &mut World, _sender: NetStateId, info: &RelayInfo) {
        let caster = self.caster;
        let entry_id = info.button_id - 1;

        if entry_id < 0 || entry_id as usize >= ENTRIES.len() {
            return;
        }

        let mana = scale_mana_of(w, self.spell, required_mana(w, self.spell));
        let entry = &ENTRIES[entry_id as usize];

        if !entry_enabled(w, caster, entry.type_name) {
            return;
        }

        let blocked = muo_content::player_mobile::data(w, caster)
            .is_some_and(|d| d.mount_block.is_some())
            || hooks(w)
                .and_then(|h| h.duel_allow_spell_cast)
                .is_some_and(|f| !f(w, caster, self.spell));

        if mana > w.mobile(caster).map_or(0, |c| c.mana) {
            // You must have at least ~1_MANA_REQUIREMENT~ Mana to use this ability.
            mm::send_localized_message(w, caster, 1060174, &mana.to_string(), 0x3B2);
        } else if is_player_mobile(w, caster) && blocked {
            mm::send_localized_message(w, caster, 1063108, "", 0x3B2); // You cannot use this ability right now.
        } else {
            let result = morph(w, caster, entry_id);

            if result == MorphResult::Success {
                effects::fixed_particles(w, caster, 0x3728, 10, 13, 2023, EffectLayer::Waist);
                let cur = w.mobile(caster).map_or(0, |c| c.mana);
                props_b::set_mana(w, caster, cur - mana);
            } else if result == MorphResult::Fail {
                mm::local_overhead_message_number(
                    w,
                    caster,
                    MessageType::REGULAR,
                    0x3B2,
                    502632,
                    "",
                ); // The spell fizzles.
                effects::fixed_particles(w, caster, 0x3735, 1, 30, 9503, EffectLayer::Waist);
                effects::play_sound(w, caster, 0x5C);
            }
            // MorphResult.NoSkill: Morph already messaged the requirement; consume no mana.
        }
    }
}

/// Animal Form tests.
#[cfg(test)]
mod behavior_tests {
    use super::*;
    use muo_server::mobile::MOBILE_BASE;

    fn index_of(type_name: &str) -> i32 {
        ENTRIES
            .iter()
            .position(|e| e.type_name == type_name)
            .expect("an animal form") as i32
    }

    fn new_mobile_with_ninjitsu(skill: f64) -> (World, MobileId) {
        let mut w = World::new();
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
        w.resources.get_mut::<muo_server::skills::SkillTable>().info = (0..58).map(info).collect();
        let m = w.new_mobile(&MOBILE_BASE);
        stats::default_mobile_init(&mut w, m);
        skills::set_base(&mut w, m, SkillName::Ninjitsu, skill);
        (w, m)
    }

    // Issue #2452: with only 30 Ninjitsu the gump showed every form (including Dog, which needs 40) because the
    // selectable check compared Skill.Fixed (value x10) against the raw ReqSkill.
    #[test]
    fn can_select_entry_form_above_skill_is_not_selectable() {
        let (mut w, m) = new_mobile_with_ninjitsu(30.0);

        assert!(!can_select_entry(
            &mut w,
            m,
            &ENTRIES[index_of("Dog") as usize]
        )); // ReqSkill 40
        assert!(!can_select_entry(
            &mut w,
            m,
            &ENTRIES[index_of("Cat") as usize]
        )); // ReqSkill 40
    }

    #[test]
    fn can_select_entry_form_at_or_below_skill_is_selectable() {
        let (mut w, m) = new_mobile_with_ninjitsu(40.0);

        assert!(can_select_entry(
            &mut w,
            m,
            &ENTRIES[index_of("Dog") as usize]
        )); // ReqSkill 40 (== boundary)
        assert!(can_select_entry(
            &mut w,
            m,
            &ENTRIES[index_of("Rat") as usize]
        )); // ReqSkill 20
        assert!(can_select_entry(
            &mut w,
            m,
            &ENTRIES[index_of("Rabbit") as usize]
        )); // ReqSkill 20
    }

    // Issue #2452: attempting a form above your skill must report NoSkill (so the caller can refrain from charging
    // mana) and must not actually transform the caster.
    #[test]
    fn morph_insufficient_skill_returns_no_skill_and_does_not_transform() {
        let (mut w, m) = new_mobile_with_ninjitsu(30.0);

        let result = morph(&mut w, m, index_of("Dog")); // needs 40

        assert_eq!(result, MorphResult::NoSkill);
        assert!(get_context(&mut w, m).is_none());
        assert_eq!(w.mobile(m).unwrap().body_mod.0, 0);
    }

    // The Reptalon breath reaches DefaultRangePerception (16), not 18.
    #[test]
    fn reptalon_breath_range_is_default_range_perception() {
        assert_eq!(REPTALON_RANGE, 16);
    }
}
