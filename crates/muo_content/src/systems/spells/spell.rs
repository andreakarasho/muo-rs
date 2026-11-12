//! Spells: the casting state machine every spell runs on.
//!
//! A spell class is a `static` [`SpellType`] built from [`SPELL_BASE`] (or [`super::magery::MAGERY_SPELL`]) with struct
//! update: the `virtual` members are fn pointers taking the world and the [`SpellId`], and `base.Foo()` is a call of
//! the parent's `pub fn base_foo`. A spell instance is a [`Spell`] in the [`Spells`] resource; `Mobile.Spell` holds
//! its id. A finished spell is dropped two minutes after `FinishSequence` (targets and timers may still hold the id
//! and then find nothing).

use std::any::Any;

use indexmap::IndexMap;
use muo_primitives::{Direction, MessageType, TimeSpan};
use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};
pub use muo_server::interfaces::SpellId;

use muo_server::mobile::part2::{effects as fx, messages as mm, stats};
use muo_server::mobile::{MobileDelta, base as mb};
use muo_server::region::RegionId;
use muo_server::skills::{self, SkillName};
use muo_server::timer::{self, TimerHandle};

use super::hooks::hooks;
use super::types::{
    DEFAULT_CAST_SKILL, DEFAULT_DAMAGE_SKILL, DisturbType, SpellInfo, SpellState, reagent,
};
use crate::core;
use crate::misc::aos_helpers::{self, DamageOptions, DamageSplit};
use crate::misc::titles;

const NEXT_SPELL_DELAY: TimeSpan = TimeSpan::from_millis(750);
const ANIMATE_DELAY: TimeSpan = TimeSpan::from_millis(1500);

/// One `static` per spell class.
pub struct SpellType {
    /// `Server.Spells.First.ClumsySpell`.
    pub full_name: &'static str,
    pub parent: Option<&'static SpellType>,
    /// The `SpellInfo` the constructor passes.
    pub info: SpellInfo,
    /// `CastSkill` / `DamageSkill`.
    pub cast_skill: SkillName,
    pub damage_skill: SkillName,
    pub reveal_on_cast: bool,
    pub clear_hands_on_cast: bool,
    pub show_hand_movement: bool,
    pub delayed_damage: bool,
    /// `DelayedDamage => !Core.AOS` (instead of `delayed_damage`).
    pub delayed_damage_unless_aos: bool,
    /// `DelayedDamageSpellFamilyStacking`: `None` allows stacking, an empty list forbids it with itself only, a
    /// list names the other spells (full names) it shares a context with.
    pub delayed_damage_family: Option<&'static [&'static str]>,
    pub blocked_by_horrific_beast: bool,
    pub blocked_by_animal_form: bool,
    /// `IRangedSpell.TargetRange`: `Some` for a spell that implements `ITargetingSpell<T>` or `IRecallSpell`; the
    /// class overrides it with its own function (the default is `targeting::target_range`).
    pub target_range: Option<fn(&World) -> i32>,
    pub cast_recovery_base: i32,
    /// `Core.ML ? CastRecoveryBase : x`: the value before ML.
    pub cast_recovery_base_pre_ml: Option<i32>,
    pub cast_recovery_fast_scalar: i32,
    pub cast_recovery_per_second: i32,
    pub cast_recovery_minimum: i32,
    pub cast_delay_fast_scalar: f64,
    /// `CastDelayFastScalar => Core.SE ? base : 0`: the scalar applies from SE on only.
    pub cast_delay_fast_scalar_needs_se: bool,
    pub cast_delay_seconds_per_tick: f64,
    pub cast_delay_minimum: TimeSpan,

    // abstract: every concrete class sets these
    pub cast_delay_base: fn(&World, SpellId) -> TimeSpan,
    pub on_cast: fn(&mut World, SpellId),
    pub get_mana: fn(&World, SpellId) -> i32,

    pub blocks_movement: fn(&World, SpellId) -> bool,
    pub check_next_spell_time: fn(&World, SpellId) -> bool,
    pub on_caster_hurt: fn(&mut World, SpellId),
    pub on_caster_killed: fn(&mut World, SpellId),
    pub on_connection_changed: fn(&mut World, SpellId),
    pub on_caster_moving: fn(&mut World, SpellId, Direction) -> bool,
    pub on_caster_equipping: fn(&mut World, SpellId, ItemId) -> bool,
    pub on_caster_using_object: fn(&mut World, SpellId, EntityId) -> bool,
    pub on_cast_in_town: fn(&World, SpellId, RegionId) -> bool,
    pub finish_sequence: fn(&mut World, SpellId),
    pub get_new_aos_damage_single:
        fn(&mut World, SpellId, i32, i32, i32, bool, Option<MobileId>) -> i32,
    pub get_new_aos_damage: fn(&mut World, SpellId, i32, i32, i32, bool, bool, f64) -> i32,
    pub consume_reagents: fn(&mut World, SpellId) -> bool,
    pub get_inscribe_skill: fn(&mut World, MobileId) -> f64,
    pub get_inscribe_fixed: fn(&mut World, MobileId) -> i32,
    pub get_damage_fixed: fn(&mut World, SpellId, MobileId) -> i32,
    pub get_damage_skill: fn(&mut World, SpellId, MobileId) -> f64,
    pub get_resist_skill: fn(&mut World, SpellId, MobileId) -> f64,
    pub get_damage_scalar: fn(&mut World, SpellId, MobileId) -> f64,
    pub get_slayer_damage_scalar: fn(&mut World, SpellId, MobileId) -> f64,
    pub do_fizzle: fn(&mut World, SpellId),
    pub check_disturb: fn(&World, SpellId, DisturbType, bool, bool) -> bool,
    pub do_hurt_fizzle: fn(&mut World, SpellId),
    pub on_disturb: fn(&mut World, SpellId, DisturbType, bool),
    pub check_cast: fn(&mut World, SpellId) -> bool,
    pub say_mantra: fn(&mut World, SpellId),
    pub on_begin_cast: fn(&mut World, SpellId),
    pub get_cast_skills: fn(&mut World, SpellId) -> (f64, f64),
    pub check_fizzle: fn(&mut World, SpellId) -> bool,
    pub scale_mana: fn(&mut World, SpellId, i32) -> i32,
    pub get_disturb_recovery: fn(&mut World, SpellId) -> TimeSpan,
    pub get_cast_recovery: fn(&mut World, SpellId) -> TimeSpan,
    pub get_cast_delay: fn(&mut World, SpellId) -> TimeSpan,
    pub compute_karma_award: fn(&mut World, SpellId) -> i32,

    /// The family's own vtable (`MageryType` for `MagerySpell`..), inherited with `..PARENT`.
    pub family: Option<&'static (dyn Any + Send + Sync)>,
    /// The interfaces the class implements (`IRecallSpell`..): a vtable read by the code that treats it as one.
    pub interface: Option<&'static (dyn Any + Send + Sync)>,
}

impl SpellType {
    /// `DelayedDamage`.
    pub fn delayed_damage_in(&self, aos: bool) -> bool {
        if self.delayed_damage_unless_aos {
            !aos
        } else {
            self.delayed_damage
        }
    }

    /// `DelayedDamageSpellFamilyStacking`: `Some(&[])` is `AOSNoDelayedDamageStackingSelf` (`Core.AOS ? [] : null`).
    pub fn delayed_damage_family_in(&self, aos: bool) -> Option<&'static [&'static str]> {
        self.delayed_damage_family.filter(|f| aos || !f.is_empty())
    }

    /// The family vtable of type `F`, if this class has one.
    pub fn family<F: Any>(&self) -> Option<&'static F> {
        self.family?.downcast_ref()
    }

    /// The interface vtable of type `F`, if this class implements it.
    pub fn interface<F: Any>(&self) -> Option<&'static F> {
        self.interface?.downcast_ref()
    }

    /// Type test (`is`): this class, or an ancestor, is `other`.
    pub fn is(&self, other: &'static SpellType) -> bool {
        let mut t = self;
        loop {
            if std::ptr::eq(t, other) {
                return true;
            }
            match t.parent {
                Some(p) => t = p,
                None => return false,
            }
        }
    }
}

/// A live spell.
pub struct Spell {
    pub id: SpellId,
    pub ty: &'static SpellType,
    pub caster: MobileId,
    pub scroll: Option<ItemId>,
    pub state: SpellState,
    pub start_cast_time: i64,
    anim_timer: TimerHandle,
    cast_timer: TimerHandle,
    /// What a spell class adds (a Recall's rune, a summon's creature..).
    pub ext: muo_server::Ext,
}

/// The `Spell` instances, and the delayed-damage contexts (`_contextTable`).
#[derive(Default)]
pub struct Spells {
    spells: IndexMap<SpellId, Spell>,
    next_id: u32,
    /// `_contextTable`: spell class name -> the contexts of the family sharing it.
    contexts: IndexMap<&'static str, usize>,
    context_sets: Vec<IndexMap<MobileId, TimerHandle>>,
}

pub fn spells(w: &mut World) -> &mut Spells {
    w.resources.get_mut::<Spells>()
}

impl Spells {
    pub fn get(&self, id: SpellId) -> Option<&Spell> {
        self.spells.get(&id)
    }

    pub fn get_mut(&mut self, id: SpellId) -> Option<&mut Spell> {
        self.spells.get_mut(&id)
    }
}

/// The spell, if it still exists.
pub fn spell(w: &World, id: SpellId) -> Option<&Spell> {
    w.resources.get::<Spells>()?.get(id)
}

/// The class of a spell.
pub fn spell_type(w: &World, id: SpellId) -> Option<&'static SpellType> {
    spell(w, id).map(|s| s.ty)
}

pub fn caster(w: &World, id: SpellId) -> Option<MobileId> {
    spell(w, id).map(|s| s.caster)
}

pub fn scroll(w: &World, id: SpellId) -> Option<ItemId> {
    spell(w, id).and_then(|s| s.scroll)
}

pub fn state(w: &World, id: SpellId) -> SpellState {
    spell(w, id).map_or(SpellState::None, |s| s.state)
}

pub fn set_state(w: &mut World, id: SpellId, state: SpellState) {
    if let Some(s) = spells(w).get_mut(id) {
        s.state = state;
    }
}

pub fn info(w: &World, id: SpellId) -> Option<&'static SpellInfo> {
    spell(w, id).map(|s| &s.ty.info)
}

pub fn start_cast_time(w: &World, id: SpellId) -> i64 {
    spell(w, id).map_or(0, |s| s.start_cast_time)
}

/// `Spell.IsCasting`.
pub fn is_casting(w: &World, id: SpellId) -> bool {
    state(w, id) == SpellState::Casting
}

/// `new T(caster, scroll)`: a spell of class `ty`.
pub fn new_spell(
    w: &mut World,
    ty: &'static SpellType,
    caster: MobileId,
    scroll: Option<ItemId>,
) -> SpellId {
    let s = spells(w);
    s.next_id += 1;
    let id = SpellId(s.next_id);
    s.spells.insert(
        id,
        Spell {
            id,
            ty,
            caster,
            scroll,
            state: SpellState::None,
            start_cast_time: 0,
            anim_timer: TimerHandle::default(),
            cast_timer: TimerHandle::default(),
            ext: muo_server::Ext::NONE,
        },
    );
    id
}

/// Drops a spell from the table.
pub fn remove_spell(w: &mut World, id: SpellId) {
    spells(w).spells.shift_remove(&id);
}

// ---- what kind of scroll ----------------------------------------------------------------------------------------

fn item_is(w: &World, id: ItemId, full_name: &str) -> bool {
    let Some(it) = w.item(id) else { return false };
    let mut t = Some(it.ty);
    while let Some(ty) = t {
        if ty.full_name == full_name {
            return true;
        }
        t = ty.parent;
    }
    false
}

/// `Scroll is BaseWand`.
pub fn scroll_is_wand(w: &World, id: SpellId) -> bool {
    scroll(w, id).is_some_and(|s| item_is(w, s, "Server.Items.BaseWand"))
}

fn skill_value(w: &mut World, m: MobileId, skill: SkillName) -> f64 {
    skills::value(w, m, skill)
}

fn body_is_human(w: &World, m: MobileId) -> bool {
    if w.mobile(m).is_none() {
        return false;
    }
    w.resources
        .get::<muo_primitives::BodyTable>()
        .is_some_and(|t| t.is_human(muo_server::mobile::part1::props_b::body(w, m)))
}

fn body_is_monster(w: &World, m: MobileId) -> bool {
    if w.mobile(m).is_none() {
        return false;
    }
    w.resources
        .get::<muo_primitives::BodyTable>()
        .is_some_and(|t| t.is_monster(muo_server::mobile::part1::props_b::body(w, m)))
}

fn is_player(w: &World, m: MobileId) -> bool {
    w.mobile(m).is_some_and(|m| m.player)
}

fn send(w: &mut World, to: MobileId, number: i32) {
    mm::send_localized_message(w, to, number, "", 0x3B2);
}

fn attribute(w: &World, m: MobileId, a: crate::aos::AosAttribute) -> i32 {
    aos_helpers::attribute_value(w, m, a)
}

// ---- the base virtuals ------------------------------------------------------------------------------------------

/// `BlocksMovement => IsCasting`.
pub fn base_blocks_movement(w: &World, id: SpellId) -> bool {
    is_casting(w, id)
}

/// `CheckNextSpellTime => Scroll is not BaseWand`.
pub fn base_check_next_spell_time(w: &World, id: SpellId) -> bool {
    !scroll_is_wand(w, id)
}

/// `OnCasterHurt()`: monsters and pets cannot be disturbed.
pub fn base_on_caster_hurt(w: &mut World, id: SpellId) {
    let Some(c) = caster(w, id) else { return };
    if is_player(w, c) && is_casting(w, id) {
        let d = hooks(w).and_then(|h| h.protection).and_then(|f| f(w, c));
        let r = |w: &mut World| muo_primitives::utility::random(&mut w.rng, 1000);
        let disturb_it = match d {
            None => true,
            Some(d) => d < 1000 && d < r(w),
        };
        if disturb_it {
            disturb(w, id, DisturbType::Hurt, false, true);
        }
    }
}

pub fn base_on_caster_killed(w: &mut World, id: SpellId) {
    disturb(w, id, DisturbType::Kill, true, false);
}

pub fn base_on_connection_changed(w: &mut World, id: SpellId) {
    finish_sequence(w, id);
}

pub fn base_on_caster_moving(w: &mut World, id: SpellId, _d: Direction) -> bool {
    let ty = spell_type(w, id);
    if is_casting(w, id) && ty.is_some_and(|t| (t.blocks_movement)(w, id)) {
        if let Some(c) = caster(w, id) {
            send(w, c, 500111); // You are frozen and can not move.
        }
        return false;
    }
    true
}

pub fn base_on_caster_equipping(w: &mut World, id: SpellId, _item: ItemId) -> bool {
    if is_casting(w, id) {
        disturb(w, id, DisturbType::EquipRequest, true, false);
    }
    true
}

pub fn base_on_caster_using_object(w: &mut World, id: SpellId, _e: EntityId) -> bool {
    if state(w, id) == SpellState::Sequencing {
        disturb(w, id, DisturbType::UseRequest, true, false);
    }
    true
}

/// `OnCastInTown(Region r) => Info.AllowTown`.
pub fn base_on_cast_in_town(w: &World, id: SpellId, _r: RegionId) -> bool {
    info(w, id).is_some_and(|i| i.allow_town)
}

/// `FinishSequence()`.
pub fn base_finish_sequence(w: &mut World, id: SpellId) {
    set_state(w, id, SpellState::None);
    let Some(c) = caster(w, id) else { return };
    if w.mobile(c).and_then(|m| m.spell) == Some(id) {
        muo_server::mobile::part1::props_a::set_spell(w, c, None);
    }
    mb::delta(w, c, MobileDelta::FLAGS); // Remove paralyze
    // The object may still be referenced by a target or timer for a moment.
    timer::start(w, TimeSpan::from_mins(2), TimeSpan::ZERO, 1, move |w| {
        remove_spell(w, id)
    });
}

pub fn finish_sequence(w: &mut World, id: SpellId) {
    if let Some(ty) = spell_type(w, id) {
        (ty.finish_sequence)(w, id);
    }
}

// ---- delayed damage contexts ------------------------------------------------------------------------------------

/// `StartDelayedDamageContext(m, t)`.
pub fn start_delayed_damage_context(w: &mut World, id: SpellId, m: MobileId, t: TimerHandle) {
    let Some(ty) = spell_type(w, id) else { return };
    let Some(stacking) = ty.delayed_damage_family_in(core::aos(w)) else {
        return;
    }; // Sanity
    let sp = spells(w);
    let set = match sp.contexts.get(ty.full_name) {
        Some(&i) => i,
        None => {
            let i = sp.context_sets.len();
            sp.context_sets.push(IndexMap::new());
            sp.contexts.insert(ty.full_name, i);
            for other in stacking {
                sp.contexts.insert(other, i);
            }
            i
        }
    };
    let old = sp.context_sets[set].insert(m, t);
    if let Some(mut old) = old {
        old.cancel(w);
    }
}

/// `HasDelayedDamageContext(m)`.
pub fn has_delayed_damage_context(w: &World, id: SpellId, m: MobileId) -> bool {
    let (Some(ty), Some(sp)) = (spell_type(w, id), w.resources.get::<Spells>()) else {
        return false;
    };
    ty.delayed_damage_family_in(core::aos(w)).is_some()
        && sp
            .contexts
            .get(ty.full_name)
            .is_some_and(|&i| sp.context_sets[i].contains_key(&m))
}

/// `RemoveDelayedDamageContext(m)`.
pub fn remove_delayed_damage_context(w: &mut World, id: SpellId, m: MobileId) {
    let Some(ty) = spell_type(w, id) else { return };
    if ty.delayed_damage_family_in(core::aos(w)).is_none() {
        return; // Sanity
    }
    let sp = spells(w);
    let Some(&i) = sp.contexts.get(ty.full_name) else {
        return;
    };
    if let Some(mut t) = sp.context_sets[i].shift_remove(&m) {
        t.cancel(w);
    }
}

/// `HarmfulSpell(m)`.
pub fn harmful_spell(w: &mut World, id: SpellId, m: MobileId) {
    let Some(c) = caster(w, id) else { return };
    if let Some(cr) = crate::creature::ctype(w, m) {
        (cr.on_harmful_spell)(w, m, c);
    }
}

// ---- damage -----------------------------------------------------------------------------------------------------

/// `GetNewAosDamage(bonus, dice, sides, sdi, singleTarget)`.
pub fn get_new_aos_damage(
    w: &mut World,
    id: SpellId,
    bonus: i32,
    dice: i32,
    sides: i32,
    sdi: bool,
    single_target: Option<MobileId>,
) -> i32 {
    let Some(ty) = spell_type(w, id) else {
        return 0;
    };
    (ty.get_new_aos_damage_single)(w, id, bonus, dice, sides, sdi, single_target)
}

pub fn base_get_new_aos_damage_single(
    w: &mut World,
    id: SpellId,
    bonus: i32,
    dice: i32,
    sides: i32,
    sdi: bool,
    single_target: Option<MobileId>,
) -> i32 {
    let (Some(ty), Some(c)) = (spell_type(w, id), caster(w, id)) else {
        return 0;
    };
    match single_target {
        Some(t) => {
            let pvp = is_player(w, c) && is_player(w, t);
            let scalar = (ty.get_damage_scalar)(w, id, t);
            (ty.get_new_aos_damage)(w, id, bonus, dice, sides, pvp, sdi, scalar)
        }
        None => (ty.get_new_aos_damage)(w, id, bonus, dice, sides, false, sdi, 1.0),
    }
}

#[allow(clippy::too_many_arguments)] // one parameter per input of the check
pub fn base_get_new_aos_damage(
    w: &mut World,
    id: SpellId,
    bonus: i32,
    dice: i32,
    sides: i32,
    player_vs_player: bool,
    sdi: bool,
    scalar: f64,
) -> i32 {
    let (Some(ty), Some(c)) = (spell_type(w, id), caster(w, id)) else {
        return 0;
    };
    let mut damage = muo_primitives::utility::dice(&mut w.rng, dice, sides, bonus) * 100;

    let inscribe_skill = (ty.get_inscribe_fixed)(w, c);
    let inscribe_bonus = (inscribe_skill + 1000 * (inscribe_skill / 1000)) / 200;
    let mut damage_bonus = inscribe_bonus;

    let int_bonus = w.mobile(c).map_or(0, |m| (m.ty.int)(w, c)) / 10;
    damage_bonus += int_bonus;

    if sdi {
        let mut sdi_bonus = attribute(w, c, crate::aos::AosAttribute::SpellDamage);
        // PvP spell damage increase cap of 15% from an item's magic property
        if player_vs_player && sdi_bonus > 15 {
            sdi_bonus = 15;
        }
        damage_bonus += sdi_bonus;
    }

    if let Some(f) = hooks(w).and_then(|h| h.reaper_form_damage_bonus) {
        damage_bonus += f(w, c);
    }

    damage = aos_helpers::scale(damage, 100 + damage_bonus);

    let eval_skill = (ty.get_damage_fixed)(w, id, c);
    let eval_scale = 30 + 9 * eval_skill / 100;

    damage = aos_helpers::scale(damage, eval_scale);

    damage = aos_helpers::scale(damage, (scalar * 100.0) as i32);

    damage / 100
}

pub fn base_get_inscribe_skill(w: &mut World, m: MobileId) -> f64 {
    skill_value(w, m, SkillName::Inscribe)
}

pub fn base_get_inscribe_fixed(w: &mut World, m: MobileId) -> i32 {
    skills::fixed(w, m, SkillName::Inscribe)
}

pub fn base_get_damage_fixed(w: &mut World, id: SpellId, m: MobileId) -> i32 {
    let skill = spell_type(w, id).map_or(DEFAULT_DAMAGE_SKILL, |t| t.damage_skill);
    skills::fixed(w, m, skill)
}

pub fn base_get_damage_skill(w: &mut World, id: SpellId, m: MobileId) -> f64 {
    let skill = spell_type(w, id).map_or(DEFAULT_DAMAGE_SKILL, |t| t.damage_skill);
    skill_value(w, m, skill)
}

pub fn base_get_resist_skill(w: &mut World, _id: SpellId, m: MobileId) -> f64 {
    skill_value(w, m, SkillName::MagicResist)
}

/// `GetDamageScalar(target)`.
pub fn base_get_damage_scalar(w: &mut World, id: SpellId, target: MobileId) -> f64 {
    let (Some(ty), Some(c)) = (spell_type(w, id), caster(w, id)) else {
        return 1.0;
    };
    let mut scalar = 1.0;

    if !core::aos(w) {
        // EvalInt stuff for AoS is handled elsewhere
        let caster_ei = skill_value(w, c, ty.damage_skill);
        let target_rs = skill_value(w, target, SkillName::MagicResist);

        if caster_ei > target_rs {
            scalar = 1.0 + (caster_ei - target_rs) / 500.0;
        } else {
            scalar = 1.0 + (caster_ei - target_rs) / 200.0;
        }

        // magery damage bonus, -25% at 0 skill, +0% at 100 skill, +5% at 120 skill
        scalar += (skill_value(w, c, ty.cast_skill) - 100.0) / 400.0;

        if !is_player(w, target) && !body_is_human(w, target) {
            scalar *= 2.0; // Double magery damage to monsters/animals if not AOS
        }
    }

    if let Some(cr) = crate::creature::ctype(w, target) {
        (cr.alter_damage_scalar_from)(w, target, c, &mut scalar);
    }
    if let Some(cr) = crate::creature::ctype(w, c) {
        (cr.alter_damage_scalar_to)(w, c, target, &mut scalar);
    }

    if core::se(w) {
        scalar *= (ty.get_slayer_damage_scalar)(w, id, target);
    }

    if let Some(rid) = w.mobile(target).map(|m| m.region)
        && let Some(rt) = w.region(rid).map(|r| r.ty)
    {
        (rt.spell_damage_scalar)(w, rid, c, target, &mut scalar);
    }

    // Only single target spells an be evaded
    if crate::hooks::hooks(w)
        .and_then(|h| h.check_spell_evasion)
        .is_some_and(|f| f(w, target))
    {
        scalar = 0.0;
    }

    scalar
}

pub fn base_get_slayer_damage_scalar(w: &mut World, id: SpellId, defender: MobileId) -> f64 {
    let Some(c) = caster(w, id) else { return 1.0 };
    hooks(w)
        .and_then(|h| h.slayer_damage_scalar)
        .map_or(1.0, |f| f(w, c, defender))
}

// ---- reagents ---------------------------------------------------------------------------------------------------

/// `ConsumeReagents()`: true when the scroll, a non-player, lower reagent cost, a duel, or the backpack covers it.
pub fn base_consume_reagents(w: &mut World, id: SpellId) -> bool {
    let (Some(ty), Some(c)) = (spell_type(w, id), caster(w, id)) else {
        return false;
    };
    if scroll(w, id).is_some() || !is_player(w, c) {
        return true;
    }
    let lrc = attribute(w, c, crate::aos::AosAttribute::LowerRegCost);
    if lrc > muo_primitives::utility::random(&mut w.rng, 100) {
        return true;
    }
    if hooks(w)
        .and_then(|h| h.duel_is_free_consume)
        .is_some_and(|f| f(w, c))
    {
        return true;
    }
    let Some(pack) = w.mobile(c).and_then(|m| m.backpack) else {
        return false;
    };
    let types: Vec<_> = ty
        .info
        .reagents
        .iter()
        .filter_map(|n| match w.types.find_type_by_full_name(n, false)? {
            muo_server::type_registry::EntityType::Item(t) => Some(t),
            _ => None,
        })
        .collect();
    if types.len() != ty.info.reagents.len() {
        // A reagent class isn't registered: the backpack cannot have it.
        return false;
    }
    let amounts = ty.info.amounts_vec();
    muo_server::item::container::consume_total_types(w, pack, &types, &amounts, true, None) == -1
}

// ---- fizzle and disturb -----------------------------------------------------------------------------------------

/// `DoFizzle()`.
pub fn base_do_fizzle(w: &mut World, id: SpellId) {
    let Some(c) = caster(w, id) else { return };
    mm::local_overhead_message_number(w, c, MessageType::REGULAR, 0x3B2, 502632, ""); // The spell fizzles.

    if is_player(w, c) {
        if core::aos(w) {
            fx::fixed_particles(
                w,
                c,
                0x3735,
                1,
                30,
                9503,
                muo_server::effects::EffectLayer::Waist,
            );
        } else {
            fx::fixed_effect(w, c, 0x3735, 6, 30);
        }
        fx::play_sound(w, c, 0x5C);
    }
}

/// `CheckDisturb(type, firstCircle, resistable) => !(resistable && Scroll is BaseWand)`.
pub fn base_check_disturb(
    w: &World,
    id: SpellId,
    _t: DisturbType,
    _first_circle: bool,
    resistable: bool,
) -> bool {
    !(resistable && scroll_is_wand(w, id))
}

/// `Disturb(type, firstCircle = true, resistable = false)`.
pub fn disturb(w: &mut World, id: SpellId, t: DisturbType, first_circle: bool, resistable: bool) {
    let Some(ty) = spell_type(w, id) else { return };
    if !(ty.check_disturb)(w, id, t, first_circle, resistable) {
        return;
    }
    let st = state(w, id);
    let is_first_circle_magery = ty
        .family::<super::magery::MageryType>()
        .is_some_and(|m| m.circle == super::types::SpellCircle::First);
    if st == SpellState::None || (!first_circle && !core::aos(w) && is_first_circle_magery) {
        return;
    }

    let Some(c) = caster(w, id) else { return };
    let was_casting = is_casting(w, id); // Copy SpellState before resetting it to none
    set_state(w, id, SpellState::None);
    muo_server::mobile::part1::props_a::set_spell(w, c, None);

    (ty.on_disturb)(w, id, t, was_casting);

    if was_casting {
        let (mut ct, mut at) = spells(w)
            .get_mut(id)
            .map_or((TimerHandle::default(), TimerHandle::default()), |s| {
                (s.cast_timer, s.anim_timer)
            });
        timer::stop(w, ct);
        timer::stop(w, at);
        ct.cancel(w);
        at.cancel(w);
        let recovery = (ty.get_disturb_recovery)(w, id);
        let now = w.tick_count;
        if let Some(m) = w.mobile_mut(c) {
            m.next_spell_time = now.wrapping_add(recovery.total_milliseconds() as i32 as i64);
        }
    } else {
        muo_server::target::cancel_mobile_target(w, c);
    }

    if core::aos(w) && is_player(w, c) && t == DisturbType::Hurt {
        (ty.do_hurt_fizzle)(w, id);
    }

    mb::delta(w, c, MobileDelta::FLAGS); // Remove paralyze
}

pub fn base_do_hurt_fizzle(w: &mut World, id: SpellId) {
    let Some(c) = caster(w, id) else { return };
    fx::fixed_effect(w, c, 0x3735, 6, 30);
    fx::play_sound(w, c, 0x5C);
}

pub fn base_on_disturb(w: &mut World, id: SpellId, _t: DisturbType, message: bool) {
    if message && let Some(c) = caster(w, id) {
        send(w, c, 500641); // Your concentration is disturbed, thus ruining thy spell.
    }
}

pub fn base_check_cast(_w: &mut World, _id: SpellId) -> bool {
    true
}

/// `SayMantra()`.
pub fn base_say_mantra(w: &mut World, id: SpellId) {
    if scroll_is_wand(w, id) {
        return;
    }
    let (Some(c), Some(i)) = (caster(w, id), info(w, id)) else {
        return;
    };
    if !i.mantra.is_empty() && is_player(w, c) {
        let hue = w.mobile(c).map_or(0, |m| m.speech_hue);
        mm::public_overhead_message(
            w,
            c,
            MessageType::SPELL,
            hue,
            true,
            i.mantra,
            false,
            muo_server::mobile::AccessLevel::Player,
        );
    }
}

// ---- cast -------------------------------------------------------------------------------------------------------

/// `Cast()`: true when the spell started casting.
pub fn cast(w: &mut World, id: SpellId) -> bool {
    let (Some(ty), Some(c)) = (spell_type(w, id), caster(w, id)) else {
        return false;
    };
    let tick = w.tick_count;
    if let Some(s) = spells(w).get_mut(id) {
        s.start_cast_time = tick;
    }

    if core::aos(w)
        && let Some(cur) = w.mobile(c).and_then(|m| m.spell)
        && state(w, cur) == SpellState::Sequencing
    {
        disturb(w, cur, DisturbType::NewCast, true, false);
    }

    if !stats::check_alive(w, c, true) {
        return false;
    }

    let current = w.mobile(c).and_then(|m| m.spell);
    let is_casting_now = current.is_some_and(|s| is_casting(w, s));
    let is_wand = scroll_is_wand(w, id);
    let under_horrific = ty.blocked_by_horrific_beast
        && super::helper::under_transformation_of(
            w,
            c,
            "Server.Spells.Necromancy.HorrificBeastSpell",
        );
    let animal_form = hooks(w)
        .and_then(|h| h.animal_form_under)
        .is_some_and(|f| f(w, c));
    let (paralyzed, frozen) = w
        .mobile(c)
        .map_or((false, false), |m| (m.paralyzed, m.frozen));

    if is_casting_now {
        if is_wand {
            send(w, c, 502643); // You can not cast a spell while frozen.
        } else {
            send(w, c, 502642); // You are already casting a spell.
        }
    } else if under_horrific || (ty.blocked_by_animal_form && animal_form) {
        send(w, c, 1061091); // You cannot cast that spell in this form.
    } else if !is_wand && (paralyzed || frozen) {
        send(w, c, 502643); // You can not cast a spell while frozen.
    } else if (ty.check_next_spell_time)(w, id)
        && w.tick_count
            .wrapping_sub(w.mobile(c).map_or(0, |m| m.next_spell_time))
            < 0
    {
        send(w, c, 502644); // You have not yet recovered from casting a spell.
    } else if crate::player_mobile::data(w, c).is_some_and(|d| d.peaced_until > w.now) {
        send(w, c, 1072060); // You cannot cast a spell while calmed.
    } else if is_player(w, c)
        && hooks(w)
            .and_then(|h| h.duel_allow_spell_cast)
            .is_some_and(|f| !f(w, c, id))
    {
        // The duel context says no and says why itself.
    } else if let Some(reason) = blocked_reason(w, c, ty) {
        let text = reason.unwrap_or_else(|| "This spell is temporarily disabled.".to_string());
        mm::send_message_hue(w, c, 0x22, &text);
    } else {
        let mana = (ty.get_mana)(w, id);
        let required_mana = (ty.scale_mana)(w, id, mana);

        let have_mana = w.mobile(c).map_or(0, |m| m.mana);
        if have_mana >= required_mana {
            let spell_free = w.mobile(c).is_some_and(|m| m.spell.is_none());
            if spell_free
                && mb::check_spell_cast(w, c, id)
                && (ty.check_cast)(w, id)
                && begin_spell_cast_in_region(w, c, id)
            {
                set_state(w, id, SpellState::Casting);
                muo_server::mobile::part1::props_a::set_spell(w, c, Some(id));

                if !is_wand && ty.reveal_on_cast {
                    mb::revealing_action(w, c);
                }

                (ty.say_mantra)(w, id);

                let cast_delay = (ty.get_cast_delay)(w, id);

                if ty.show_hand_movement
                    && (body_is_human(w, c) || (is_player(w, c) && body_is_monster(w, c)))
                {
                    let count =
                        (cast_delay.total_seconds() / ANIMATE_DELAY.total_seconds()).ceil() as i32;

                    if count != 0 {
                        let h = timer::create(w, TimeSpan::ZERO, ANIMATE_DELAY, count, move |w| {
                            anim_tick(w, id)
                        });
                        if let Some(s) = spells(w).get_mut(id) {
                            s.anim_timer = h;
                        }
                        timer::begin(w, h);
                    }

                    let (left_hand, right_hand) = ty.info.hand_effects_in(core::aos(w));
                    if left_hand > 0 {
                        fx::fixed_particles(
                            w,
                            c,
                            0,
                            10,
                            5,
                            left_hand,
                            muo_server::effects::EffectLayer::LeftHand,
                        );
                    }

                    if right_hand > 0 {
                        fx::fixed_particles(
                            w,
                            c,
                            0,
                            10,
                            5,
                            right_hand,
                            muo_server::effects::EffectLayer::RightHand,
                        );
                    }
                }

                if ty.clear_hands_on_cast {
                    mb::clear_hands(w, c);
                }

                if core::ml(w)
                    && let Some(f) = hooks(w).and_then(|h| h.clear_current_weapon_ability)
                {
                    f(w, c);
                }

                mb::delta(w, c, MobileDelta::FLAGS); // Start paralyze

                if cast_delay > TimeSpan::ZERO {
                    let h = timer::create(w, cast_delay, TimeSpan::ZERO, 1, move |w| {
                        cast_timer_tick(w, id)
                    });
                    if let Some(s) = spells(w).get_mut(id) {
                        s.cast_timer = h;
                    }
                    (ty.on_begin_cast)(w, id);
                    timer::begin(w, h);
                } else {
                    (ty.on_begin_cast)(w, id);
                    cast_timer_tick(w, id);
                }

                return true;
            }
        } else if insufficient_mana_has_amount(w, c) {
            // Insufficient mana. You must have at least ~1_MANA_REQUIREMENT~ Mana to use this spell.
            mm::local_overhead_message_number(
                w,
                c,
                MessageType::REGULAR,
                0x22,
                502625,
                &required_mana.to_string(),
            );
        } else {
            mm::local_overhead_message_number(w, c, MessageType::REGULAR, 0x22, 502625, ""); // Insufficient mana
        }
    }

    false
}

/// `Caster.NetState?.IsKRClient != true && Caster.NetState?.Version >= ClientVersion.Version70654`.
fn insufficient_mana_has_amount(w: &World, c: MobileId) -> bool {
    let Some(d) = w
        .mobile(c)
        .and_then(|m| m.net_state)
        .and_then(|ns| w.net_state(ns))
    else {
        return false;
    };
    !d.caps.is_kr_client()
        && d.caps
            .version
            .as_ref()
            .is_some_and(|v| v.ge(&muo_net::client_version::ClientVersion::v70654()))
}

/// `Caster is PlayerMobile { AccessLevel: < Administrator } && FeatureFlagManager.IsSpellBlocked(GetType())`.
fn blocked_reason(w: &World, c: MobileId, ty: &'static SpellType) -> Option<Option<String>> {
    let m = w.mobile(c)?;
    if !m.ty.is(&crate::player_mobile::PLAYER_MOBILE)
        || m.access_level >= muo_server::mobile::AccessLevel::Administrator
    {
        return None;
    }
    hooks(w)
        .and_then(|h| h.spell_blocked_reason)
        .and_then(|f| f(w, ty.full_name))
}

/// `Caster.Region.OnBeginSpellCast(Caster, this)`.
fn begin_spell_cast_in_region(w: &mut World, c: MobileId, id: SpellId) -> bool {
    let Some(rid) = w.mobile(c).map(|m| m.region) else {
        return true;
    };
    match w.region(rid).map(|r| r.ty) {
        Some(rt) => (rt.on_begin_spell_cast)(w, rid, c, id),
        None => true,
    }
}

fn anim_tick(w: &mut World, id: SpellId) {
    let (Some(c), Some(ty)) = (caster(w, id), spell_type(w, id)) else {
        return;
    };
    if state(w, id) != SpellState::Casting || w.mobile(c).and_then(|m| m.spell) != Some(id) {
        if let Some(h) = spells(w).get_mut(id).map(|s| s.anim_timer) {
            timer::stop(w, h);
        }
        return;
    }

    let action = ty.info.action_in(core::aos(w));
    if !muo_server::mobile::part1::props_b::mounted(w, c) && action >= 0 {
        if body_is_human(w, c) {
            mb::animate(w, c, action, 7, 1, true, false, 0);
        } else if is_player(w, c) && body_is_monster(w, c) {
            mb::animate(w, c, 12, 7, 1, true, false, 0);
        }
    }
}

fn cast_timer_tick(w: &mut World, id: SpellId) {
    let Some(c) = caster(w, id) else { return };
    let Some(ty) = spell_type(w, id) else { return };
    if state(w, id) == SpellState::Casting && w.mobile(c).and_then(|m| m.spell) == Some(id) {
        set_state(w, id, SpellState::Sequencing);
        if let Some(s) = spells(w).get_mut(id) {
            s.cast_timer = TimerHandle::default();
        }
        if let Some(f) = w.mobile(c).map(|m| m.ty.on_spell_cast) {
            f(w, c, id);
        }
        if let Some(rid) = w.mobile(c).map(|m| m.region)
            && let Some(rt) = w.region(rid).map(|r| r.ty)
        {
            (rt.on_spell_cast)(w, rid, c, id);
        }
        let recovery = (ty.get_cast_recovery)(w, id);
        let now = w.tick_count;
        if let Some(m) = w.mobile_mut(c) {
            m.next_spell_time = now.wrapping_add(recovery.total_milliseconds() as i32 as i64);
        }

        mb::delta(w, c, MobileDelta::FLAGS); // Update paralyze

        let original_target = w.mobile(c).and_then(|m| m.target);

        (ty.on_cast)(w, id);

        let target = w.mobile(c).and_then(|m| m.target);
        if is_player(w, c)
            && target != original_target
            && let Some(t) = target
        {
            muo_server::target::begin_timeout(w, t, c, 30000); // 30 seconds
        }

        if let Some(s) = spells(w).get_mut(id) {
            s.cast_timer = TimerHandle::default();
        }
    }
}

pub fn base_on_begin_cast(_w: &mut World, _id: SpellId) {}

/// `GetCastSkills(out min, out max)`: "intended but not required for overriding".
pub fn base_get_cast_skills(_w: &mut World, _id: SpellId) -> (f64, f64) {
    (0.0, 0.0)
}

/// `CheckFizzle()`.
pub fn base_check_fizzle(w: &mut World, id: SpellId) -> bool {
    if scroll_is_wand(w, id) {
        return true;
    }
    let (Some(ty), Some(c)) = (spell_type(w, id), caster(w, id)) else {
        return false;
    };
    let (min_skill, max_skill) = (ty.get_cast_skills)(w, id);

    if ty.damage_skill != ty.cast_skill {
        let cap = skills::cap(w, c, ty.damage_skill);
        stats::check_skill(w, c, ty.damage_skill, 0.0, cap);
    }

    stats::check_skill(w, c, ty.cast_skill, min_skill, max_skill)
}

/// `ScaleMana(mana)`.
pub fn base_scale_mana(w: &mut World, id: SpellId, mana: i32) -> i32 {
    let Some(c) = caster(w, id) else { return mana };
    let mut scalar = hooks(w)
        .and_then(|h| h.mind_rot_scalar)
        .and_then(|f| f(w, c))
        .unwrap_or(1.0);

    // Lower Mana Cost = 40%
    let lmc = attribute(w, c, crate::aos::AosAttribute::LowerManaCost).min(40);
    scalar -= f64::from(lmc) / 100.0;

    (f64::from(mana) * scalar) as i32
}

/// `ScaleMana` as `Spell.ScaleMana(GetMana())`.
pub fn scaled_mana(w: &mut World, id: SpellId) -> i32 {
    let Some(ty) = spell_type(w, id) else {
        return 0;
    };
    let mana = (ty.get_mana)(w, id);
    (ty.scale_mana)(w, id, mana)
}

pub fn base_get_disturb_recovery(w: &mut World, id: SpellId) -> TimeSpan {
    if core::aos(w) {
        return TimeSpan::ZERO;
    }
    let Some(ty) = spell_type(w, id) else {
        return TimeSpan::ZERO;
    };
    let delay = (ty.get_cast_delay)(w, id).total_seconds();
    let elapsed = w.tick_count.wrapping_sub(start_cast_time(w, id)) as f64 / 1000.0;
    let d = (1.0 - (elapsed / delay).sqrt()).max(0.2);
    TimeSpan::from_secs_f64(d)
}

pub fn base_get_cast_recovery(w: &mut World, id: SpellId) -> TimeSpan {
    if !core::aos(w) {
        return NEXT_SPELL_DELAY;
    }
    let (Some(ty), Some(c)) = (spell_type(w, id), caster(w, id)) else {
        return NEXT_SPELL_DELAY;
    };

    let malus = hooks(w)
        .and_then(|h| h.thunderstorm_cast_recovery_malus)
        .map_or(0, |f| f(w, c));
    let fcr = attribute(w, c, crate::aos::AosAttribute::CastRecovery) - malus;

    let fcr_delay = -(ty.cast_recovery_fast_scalar * fcr);

    let base = ty
        .cast_recovery_base_pre_ml
        .filter(|_| !core::ml(w))
        .unwrap_or(ty.cast_recovery_base);
    let mut delay = base + fcr_delay;

    if delay < ty.cast_recovery_minimum {
        delay = ty.cast_recovery_minimum;
    }

    TimeSpan::from_secs_f64(f64::from(delay) / f64::from(ty.cast_recovery_per_second))
}

pub fn base_get_cast_delay(w: &mut World, id: SpellId) -> TimeSpan {
    let (Some(ty), Some(c)) = (spell_type(w, id), caster(w, id)) else {
        return TimeSpan::ZERO;
    };
    if scroll_is_wand(w, id) {
        return if core::ml(w) {
            (ty.cast_delay_base)(w, id)
        } else {
            TimeSpan::ZERO
        }; // TODO: Should FC apply to wands?
    }

    // Faster casting cap of 2 (if not using the protection spell)
    // Faster casting cap of 0 (if using the protection spell)
    // Paladin spells are subject to a faster casting cap of 4
    // Paladins with magery of 70.0 or above are subject to a faster casting cap of 2
    let mut fc_max = 4;

    if matches!(ty.cast_skill, SkillName::Magery | SkillName::Necromancy)
        || (ty.cast_skill == SkillName::Chivalry && skill_value(w, c, SkillName::Magery) >= 70.0)
    {
        fc_max = 2;
    }

    let mut fc = attribute(w, c, crate::aos::AosAttribute::CastSpeed).min(fc_max);

    if hooks(w)
        .and_then(|h| h.protection)
        .is_some_and(|f| f(w, c).is_some())
    {
        fc -= 2;
    }

    if let Some(f) = hooks(w).and_then(|h| h.essence_of_wind_fc_malus) {
        fc -= f(w, c);
    }

    if core::sa(w) {
        // At some point OSI added 0.25s to every spell. This makes the minimum 0.5s
        // Note: This is done after multiplying for summon creature & blade spirits.
        fc -= 1;
    }

    let fast_scalar = if ty.cast_delay_fast_scalar_needs_se && !core::se(w) {
        0.0
    } else {
        ty.cast_delay_fast_scalar
    };
    let fc_delay =
        TimeSpan::from_secs_f64(-(fast_scalar * f64::from(fc) * ty.cast_delay_seconds_per_tick));

    let sum = (ty.cast_delay_base)(w, id) + fc_delay;
    if sum > ty.cast_delay_minimum {
        sum
    } else {
        ty.cast_delay_minimum
    }
}

pub fn base_compute_karma_award(_w: &mut World, _id: SpellId) -> i32 {
    0
}

// ---- sequences --------------------------------------------------------------------------------------------------

/// `CheckSequence()`.
pub fn check_sequence(w: &mut World, id: SpellId) -> bool {
    let (Some(ty), Some(c)) = (spell_type(w, id), caster(w, id)) else {
        return false;
    };
    let mana = scaled_mana(w, id);

    let (deleted, alive, cur_spell) = w.mobile(c).map_or((true, false, None), |m| {
        (m.deleted, (m.ty.alive)(w, c), m.spell)
    });
    let scroll = scroll(w, id);

    let scroll_bad = scroll.is_some_and(|s| {
        let Some(it) = w.item(s) else { return true };
        let is_runebook = item_is(w, s, "Server.Items.Runebook");
        if is_runebook {
            return false;
        }
        let root_is_caster = muo_server::item::ops::root_parent(w, s) == Some(EntityId::Mobile(c));
        let wand_bad = item_is(w, s, "Server.Items.BaseWand")
            && (hooks(w)
                .and_then(|h| h.wand_charges)
                .is_some_and(|f| f(w, s) <= 0)
                || it.parent != Some(EntityId::Mobile(c)));
        it.amount <= 0 || it.deleted() || !root_is_caster || wand_bad
    });

    if deleted
        || !alive
        || cur_spell != Some(id)
        || state(w, id) != SpellState::Sequencing
        || scroll_bad
    {
        (ty.do_fizzle)(w, id);
    } else if !(ty.consume_reagents)(w, id) {
        mm::local_overhead_message_number(w, c, MessageType::REGULAR, 0x22, 502630, ""); // More reagents are needed for this spell.
    } else if w.mobile(c).map_or(0, |m| m.mana) < mana {
        mm::local_overhead_message_number(w, c, MessageType::REGULAR, 0x22, 502625, ""); // Insufficient mana for this spell.
    } else if core::aos(w) && w.mobile(c).is_some_and(|m| m.frozen || m.paralyzed) {
        send(w, c, 502646); // You cannot cast a spell while frozen.
        (ty.do_fizzle)(w, id);
    } else if crate::player_mobile::data(w, c).is_some_and(|d| d.peaced_until > w.now) {
        send(w, c, 1072060); // You cannot cast a spell while calmed.
        (ty.do_fizzle)(w, id);
    } else if (ty.check_fizzle)(w, id) {
        let cur = w.mobile(c).map_or(0, |m| m.mana);
        muo_server::mobile::part1::props_b::set_mana(w, c, cur - mana); // the Mana setter: delta, regen timer

        if let Some(s) = scroll {
            if item_is(w, s, "Server.Items.SpellScroll") {
                muo_server::item::base::consume(w, s, 1);
            } else if item_is(w, s, "Server.Items.BaseWand") {
                if let Some(f) = hooks(w).and_then(|h| h.wand_consume_charge) {
                    f(w, s, c);
                }
                mb::revealing_action(w, c);
            }
        }

        if let Some(s) = scroll.filter(|&s| item_is(w, s, "Server.Items.BaseWand")) {
            let movable = w.item(s).is_some_and(|i| i.movable());
            muo_server::item::ops::set_movable(w, s, false);
            if ty.clear_hands_on_cast {
                mb::clear_hands(w, c);
            }
            muo_server::item::ops::set_movable(w, s, movable);
        } else if ty.clear_hands_on_cast {
            mb::clear_hands(w, c);
        }

        let karma = (ty.compute_karma_award)(w, id);

        if karma != 0 {
            titles::award_karma(w, c, karma, true);
        }

        if super::helper::under_transformation_of(
            w,
            c,
            "Server.Spells.Necromancy.VampiricEmbraceSpell",
        ) {
            let garlic = ty.info.reagents.contains(&reagent::GARLIC);
            if garlic {
                send(w, c, 1061651); // The garlic burns you!
                let amount = muo_primitives::utility::random_min_max(&mut w.rng, 17, 23);
                aos_helpers::damage(
                    w,
                    c,
                    None, // AOS.Damage(m, damage, ..) has no `from`
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
            }
        }

        return true;
    } else {
        (ty.do_fizzle)(w, id);
    }

    false
}

/// `CheckBSequence(target, allowDead = false)`.
pub fn check_b_sequence(w: &mut World, id: SpellId, target: MobileId, allow_dead: bool) -> bool {
    let Some(c) = caster(w, id) else { return false };
    let alive = w.mobile(target).is_some_and(|m| (m.ty.alive)(w, target));
    if !alive && !allow_dead {
        send(w, c, 501857); // This spell won't work on that!
        return false;
    }

    let young = |w: &World, m: MobileId| {
        crate::player_mobile::data(w, m).is_some() && crate::player_mobile::young(w, m)
    };
    let target_is_pm = w
        .mobile(target)
        .is_some_and(|m| m.ty.is(&crate::player_mobile::PLAYER_MOBILE));
    let caster_is_pm = w
        .mobile(c)
        .is_some_and(|m| m.ty.is(&crate::player_mobile::PLAYER_MOBILE));
    if caster_is_pm && young(w, c) && target_is_pm && !young(w, target) {
        send(w, c, 500278); // As a young player, you may not cast beneficial spells onto older players.
        return false;
    }

    if mb::can_be_beneficial_full(w, c, target, true, allow_dead) && check_sequence(w, id) {
        mb::do_beneficial(w, c, target);

        if c != target {
            let karma = w.mobile(target).map_or(0, |m| m.karma);
            titles::award_karma(w, c, karma / 5, true);
        }

        return true;
    }

    false
}

/// `CheckHSequence(target)`.
pub fn check_h_sequence(w: &mut World, id: SpellId, target: MobileId) -> bool {
    let Some(c) = caster(w, id) else { return false };
    let alive = w.mobile(target).is_some_and(|m| (m.ty.alive)(w, target));
    if !alive {
        send(w, c, 501857); // This spell won't work on that!
        return false;
    }

    if mb::can_be_harmful(w, c, target) && check_sequence(w, id) {
        mb::do_harmful(w, c, target, false);
        return true;
    }

    false
}

// ---- the base class ---------------------------------------------------------------------------------------------

fn abstract_cast_delay(_: &World, _: SpellId) -> TimeSpan {
    TimeSpan::ZERO
}

fn abstract_on_cast(_: &mut World, _: SpellId) {}

fn abstract_get_mana(_: &World, _: SpellId) -> i32 {
    0
}

/// `Spell` itself, abstract: concrete classes inherit from it with `..SPELL_BASE` and set `full_name`, `info`,
/// `cast_delay_base`, `on_cast` and `get_mana`.
pub static SPELL_BASE: SpellType = SpellType {
    full_name: "Server.Spells.Spell",
    parent: None,
    info: SpellInfo::new("", "", &[]),
    cast_skill: DEFAULT_CAST_SKILL,
    damage_skill: DEFAULT_DAMAGE_SKILL,
    reveal_on_cast: true,
    clear_hands_on_cast: true,
    show_hand_movement: true,
    delayed_damage: false,
    delayed_damage_unless_aos: false,
    delayed_damage_family: None,
    blocked_by_horrific_beast: true,
    blocked_by_animal_form: true,
    target_range: None,
    cast_recovery_base: 6,
    cast_recovery_base_pre_ml: None,
    cast_recovery_fast_scalar: 1,
    cast_recovery_per_second: 4,
    cast_recovery_minimum: 0,
    cast_delay_fast_scalar: 1.0,
    cast_delay_fast_scalar_needs_se: false,
    cast_delay_seconds_per_tick: 0.25,
    cast_delay_minimum: TimeSpan::from_millis(250),
    cast_delay_base: abstract_cast_delay,
    on_cast: abstract_on_cast,
    get_mana: abstract_get_mana,
    blocks_movement: base_blocks_movement,
    check_next_spell_time: base_check_next_spell_time,
    on_caster_hurt: base_on_caster_hurt,
    on_caster_killed: base_on_caster_killed,
    on_connection_changed: base_on_connection_changed,
    on_caster_moving: base_on_caster_moving,
    on_caster_equipping: base_on_caster_equipping,
    on_caster_using_object: base_on_caster_using_object,
    on_cast_in_town: base_on_cast_in_town,
    finish_sequence: base_finish_sequence,
    get_new_aos_damage_single: base_get_new_aos_damage_single,
    get_new_aos_damage: base_get_new_aos_damage,
    consume_reagents: base_consume_reagents,
    get_inscribe_skill: base_get_inscribe_skill,
    get_inscribe_fixed: base_get_inscribe_fixed,
    get_damage_fixed: base_get_damage_fixed,
    get_damage_skill: base_get_damage_skill,
    get_resist_skill: base_get_resist_skill,
    get_damage_scalar: base_get_damage_scalar,
    get_slayer_damage_scalar: base_get_slayer_damage_scalar,
    do_fizzle: base_do_fizzle,
    check_disturb: base_check_disturb,
    do_hurt_fizzle: base_do_hurt_fizzle,
    on_disturb: base_on_disturb,
    check_cast: base_check_cast,
    say_mantra: base_say_mantra,
    on_begin_cast: base_on_begin_cast,
    get_cast_skills: base_get_cast_skills,
    check_fizzle: base_check_fizzle,
    scale_mana: base_scale_mana,
    get_disturb_recovery: base_get_disturb_recovery,
    get_cast_recovery: base_get_cast_recovery,
    get_cast_delay: base_get_cast_delay,
    compute_karma_award: base_compute_karma_award,
    family: None,
    interface: None,
};

// ---- Mobile -> Spell glue (muo_server's deps) -------------------------------------------------------------------

/// `m_Spell?.OnCasterHurt()`.
pub fn caster_hurt(w: &mut World, m: MobileId) {
    if let Some(s) = w.mobile(m).and_then(|m| m.spell)
        && let Some(ty) = spell_type(w, s)
    {
        (ty.on_caster_hurt)(w, s);
    }
}

/// `m_Spell?.OnCasterKilled()`.
pub fn caster_killed(w: &mut World, m: MobileId) {
    if let Some(s) = w.mobile(m).and_then(|m| m.spell)
        && let Some(ty) = spell_type(w, s)
    {
        (ty.on_caster_killed)(w, s);
    }
}

/// `m_Spell?.OnConnectionChanged()`.
pub fn connection_changed(w: &mut World, m: MobileId) {
    if let Some(s) = w.mobile(m).and_then(|m| m.spell)
        && let Some(ty) = spell_type(w, s)
    {
        (ty.on_connection_changed)(w, s);
    }
}

/// `m_Spell?.OnCasterUsingObject(o) != false`: true lets the use go on.
pub fn caster_using_object(w: &mut World, m: MobileId, o: EntityId) -> bool {
    let Some(s) = w.mobile(m).and_then(|m| m.spell) else {
        return true;
    };
    match spell_type(w, s) {
        Some(ty) => (ty.on_caster_using_object)(w, s, o),
        None => true,
    }
}

/// `m_Spell?.OnCasterEquipping(item) != false`.
pub fn caster_equipping(w: &mut World, m: MobileId, item: ItemId) -> bool {
    let Some(s) = w.mobile(m).and_then(|m| m.spell) else {
        return true;
    };
    match spell_type(w, s) {
        Some(ty) => (ty.on_caster_equipping)(w, s, item),
        None => true,
    }
}

/// `spell.BlocksMovement`.
pub fn spell_blocks_movement(w: &World, s: SpellId) -> bool {
    spell_type(w, s).is_some_and(|ty| (ty.blocks_movement)(w, s))
}

/// `m_Spell?.OnCasterMoving(d)`: false stops the move.
pub fn caster_moving(w: &mut World, m: MobileId, d: Direction) -> bool {
    let Some(s) = w.mobile(m).and_then(|m| m.spell) else {
        return true;
    };
    match spell_type(w, s) {
        Some(ty) => (ty.on_caster_moving)(w, s, d),
        None => true,
    }
}
