//! Spell targeting: targeting spells, recall spells and the spell targets.
//!
//! `ITargetingSpell<T>.Target(T)` is the `on_target` function the spell hands to [`begin_spell_target`]; the
//! `IRecallSpell` interface is a [`RecallType`] in the spell class's `interface` slot.

use std::rc::Rc;

use muo_primitives::{MessageType, Point3D};
use muo_server::World;
use muo_server::ids::{EntityId, MobileId};
use muo_server::map::MapId;
use muo_server::mobile::part2::messages as mm;
use muo_server::target::{self, Target, TargetFlags, Targeted};

use super::hooks::hooks;
use super::spell::{self, SpellId};
use crate::core;

/// `typeof(T)` of `SpellTarget<T>`: what the target accepts.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SpellTargetKind {
    /// `SpellTarget<Mobile>`.
    Mobile,
    /// `SpellTarget<Item>`.
    Item,
    /// `SpellTarget<IPoint3D>`: mobiles, items, static tiles (and land when ground is allowed).
    Point,
}

impl SpellTargetKind {
    fn accepts(self, t: Targeted) -> bool {
        matches!(
            (self, t),
            (Self::Point, _)
                | (Self::Mobile, Targeted::Entity(EntityId::Mobile(_)))
                | (Self::Item, Targeted::Entity(EntityId::Item(_)))
        )
    }
}

/// `IRangedSpell.TargetRange`: `Core.T2A ? 10 : 12`.
pub fn target_range(w: &World) -> i32 {
    if core::t2a(w) { 10 } else { 12 }
}

/// `spell.TargetRange` of an `IRangedSpell`.
pub fn spell_target_range(w: &World, sp: SpellId) -> i32 {
    spell::spell_type(w, sp)
        .and_then(|t| t.target_range)
        .map_or_else(|| target_range(w), |f| f(w))
}

/// `(spell as IRangedSpell)?.TargetRange`.
pub fn ranged_spell_range(w: &World, sp: SpellId) -> Option<i32> {
    spell::spell_type(w, sp)
        .and_then(|t| t.target_range)
        .map(|f| f(w))
}

/// `TargetRange => Core.T2A ? 15 : 18`.
pub fn field_range(w: &World) -> i32 {
    if core::t2a(w) { 15 } else { 18 }
}

fn accepts_mobile(t: &Targeted) -> bool {
    SpellTargetKind::Mobile.accepts(*t)
}
fn accepts_item(t: &Targeted) -> bool {
    SpellTargetKind::Item.accepts(*t)
}

/// `new SpellTarget<T>(spell, allowGround, flags, retryOnLos)` (not yet the caster's target).
fn build_spell_target(
    w: &mut World,
    sp: SpellId,
    allow_ground: bool,
    flags: TargetFlags,
    retry_on_los: bool,
    kind: SpellTargetKind,
    on_target: fn(&mut World, SpellId, Targeted),
) -> Target {
    let mut t = Target::new(w, spell_target_range(w, sp), allow_ground, flags);
    t.spell = Some(sp);
    // The `CanTarget` overrides: a target of the wrong kind ends silently, before range and sight are looked at.
    t.accepts = match kind {
        SpellTargetKind::Mobile => Some(accepts_mobile),
        SpellTargetKind::Item => Some(accepts_item),
        SpellTargetKind::Point => None,
    };
    // `_spell.Target(o as T)`: a land target given to a Mobile/Item spell is a null.
    t.handlers.on_target = Some(Rc::new(
        move |w: &mut World, _from: MobileId, targeted: Targeted| {
            if kind.accepts(targeted) {
                on_target(w, sp, targeted);
            }
        },
    ));
    t.handlers.on_target_finish = Some(Rc::new(move |w: &mut World, _from: MobileId| {
        spell::finish_sequence(w, sp)
    }));
    if retry_on_los {
        // OnTargetOutOfLOS: tell the player and offer the same target again for the time that is left.
        t.handlers.on_target_out_of_los = Some(Rc::new(
            move |w: &mut World, from: MobileId, _old: Targeted| {
                mm::send_localized_message(w, from, 501943, "", 0x3B2); // Target cannot be seen. Try again.
                // Invoke already cleared the caster's target; the one running is still stored under its spell.
                let timeout_time = w
                    .targets
                    .values()
                    .find(|t| t.spell == Some(sp))
                    .map_or(0, |t| t.timeout_time);
                let again = build_spell_target(w, sp, allow_ground, flags, true, kind, on_target);
                let id = target::add_target(w, again);
                target::set_mobile_target(w, from, Some(id));
                target::begin_timeout(w, id, from, timeout_time.wrapping_sub(w.tick_count));
            },
        ));
    }
    t
}

/// `new SpellTarget<T>(spell, allowGround, flags, retryOnLos)` set as the caster's target. A target of the wrong kind
/// does nothing (`CanTarget` returns false). `OnTargetFinish` finishes the sequence.
pub fn begin_spell_target(
    w: &mut World,
    sp: SpellId,
    allow_ground: bool,
    flags: TargetFlags,
    retry_on_los: bool,
    kind: SpellTargetKind,
    on_target: fn(&mut World, SpellId, Targeted),
) {
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };
    let t = build_spell_target(w, sp, allow_ground, flags, retry_on_los, kind, on_target);
    let id = target::add_target(w, t);
    target::set_mobile_target(w, caster, Some(id));
}

// ---- recall ----------------------------------------------------------------------------------------------------

/// What a recall target resolved to (`RecallSpellTarget.OnTarget`).
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum RecallSource {
    /// A rune that isn't marked.
    RuneNotMarked,
    /// A runebook with no default entry.
    RunebookNoDefault,
    /// A marked rune, runebook entry, boat key (`check_multi` false) or house raffle plot.
    Location {
        loc: Point3D,
        map: Option<MapId>,
        check_multi: bool,
    },
    /// Anything else.
    Invalid,
}

/// `IRecallSpell`: a spell that recalls from a rune, runebook, boat key or house plot.
pub struct RecallType {
    /// `Effect(loc, map, checkMulti)`.
    pub effect: fn(&mut World, SpellId, Point3D, Option<MapId>, bool),
}

/// `new RecallSpellTarget(spell, toBoat)` as the caster's target.
pub fn begin_recall_target(w: &mut World, sp: SpellId, to_boat: bool) {
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };
    mm::local_overhead_message_number(w, caster, MessageType::REGULAR, 0x3B2, 501029, ""); // Select Marked item.

    let mut t = Target::new(w, spell_target_range(w, sp), false, TargetFlags::NONE);
    t.handlers.on_target = Some(Rc::new(
        move |w: &mut World, from: MobileId, targeted: Targeted| {
            recall_on_target(w, sp, from, targeted, to_boat);
        },
    ));
    // OnNonlocalTarget does nothing.
    t.handlers.on_nonlocal_target = Some(Rc::new(|_: &mut World, _: MobileId, _: Targeted| {}));
    t.handlers.on_target_finish = Some(Rc::new(move |w: &mut World, _from: MobileId| {
        spell::finish_sequence(w, sp)
    }));
    let id = target::add_target(w, t);
    target::set_mobile_target(w, caster, Some(id));
}

fn recall_on_target(w: &mut World, sp: SpellId, from: MobileId, targeted: Targeted, to_boat: bool) {
    let source = match (targeted, hooks(w).and_then(|h| h.resolve_recall_source)) {
        (Targeted::Entity(e), Some(f)) => f(w, e, to_boat),
        _ => RecallSource::Invalid,
    };
    let effect = spell::spell_type(w, sp)
        .and_then(|t| t.interface::<RecallType>())
        .map(|r| r.effect);

    match source {
        RecallSource::RuneNotMarked => mm::send_localized_message(w, from, 501805, "", 0x3B2), // That rune is not yet marked.
        RecallSource::RunebookNoDefault => mm::send_localized_message(w, from, 502354, "", 0x3B2), // Target is not marked.
        RecallSource::Location {
            loc,
            map,
            check_multi,
        } => {
            if let Some(effect) = effect {
                effect(w, sp, loc, map, check_multi);
            }
        }
        RecallSource::Invalid => {
            mm::local_overhead_message_number(w, from, MessageType::REGULAR, 0x3B2, 502357, ""); // I can not recall from that object.
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spells::spell::{SPELL_BASE, SpellType};
    use muo_data::expansion::Expansion;
    use muo_server::mobile::MOBILE_BASE;

    static TEST: SpellType = SpellType {
        full_name: "Test",
        parent: Some(&SPELL_BASE),
        ..SPELL_BASE
    };

    #[test]
    fn range_follows_core_t2a() {
        // `Core.T2A` is `Expansion >= T2A`, so every real era gets 10; the 12 is only for a world before T2A.
        let mut w = World::new();
        w.core.expansion = Some(Expansion::T2A);
        assert_eq!(target_range(&w), 10);
        w.core.expansion = Some(Expansion::AOS);
        assert_eq!(target_range(&w), 10);
        w.core.expansion = Some(Expansion::None);
        assert_eq!(target_range(&w), 12);
    }

    #[test]
    fn a_spell_target_is_the_casters_target_with_the_era_range() {
        let mut w = World::new();
        w.core.expansion = Some(Expansion::AOS);
        let c = w.new_mobile(&MOBILE_BASE);
        let sp = spell::new_spell(&mut w, &TEST, c, None);
        begin_spell_target(
            &mut w,
            sp,
            false,
            TargetFlags::HARMFUL,
            false,
            SpellTargetKind::Mobile,
            |_, _, _| {},
        );
        let t = w.mobile(c).unwrap().target.unwrap();
        assert_eq!(w.targets[&t].range, 10);
        assert_eq!(w.targets[&t].flags, TargetFlags::HARMFUL);
    }

    #[test]
    fn kinds_filter_what_can_be_targeted() {
        let m = Targeted::Entity(EntityId::Mobile(MobileId(muo_primitives::Serial::new(1))));
        let i = Targeted::Entity(EntityId::Item(muo_server::ids::ItemId(
            muo_primitives::Serial::new(0x40000001),
        )));
        assert!(SpellTargetKind::Mobile.accepts(m) && !SpellTargetKind::Mobile.accepts(i));
        assert!(SpellTargetKind::Item.accepts(i) && !SpellTargetKind::Item.accepts(m));
        assert!(SpellTargetKind::Point.accepts(m) && SpellTargetKind::Point.accepts(i));
    }
}
