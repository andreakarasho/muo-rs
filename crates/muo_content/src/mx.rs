//! Short names for the `Mobile` calls the creature code makes most. Each is the `Mobile` member of the same name.

use muo_primitives::{Direction, MessageType, Point3D, TimeSpan};
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::map::MapId;
use muo_server::mobile::part1::{combat, props_a, props_b};
use muo_server::mobile::part2::{effects, enumerators, messages, speech, stats};
use muo_server::mobile::{AccessLevel, base};
use muo_server::world::World;

/// `Mobile.Alive` (virtual).
pub fn alive(w: &World, id: MobileId) -> bool {
    w.mobile(id).is_some_and(|m| (m.ty.alive)(w, id))
}

pub fn deleted(w: &World, id: MobileId) -> bool {
    w.mobile(id).is_none_or(|m| m.deleted)
}

/// `source is { Alive: true, Deleted: false }`.
pub fn alive_and_present(w: &World, id: MobileId) -> bool {
    !deleted(w, id) && alive(w, id)
}

pub fn location(w: &World, id: MobileId) -> Point3D {
    w.mobile(id).map_or(Point3D::ZERO, |m| m.location)
}

pub fn map(w: &World, id: MobileId) -> Option<MapId> {
    w.mobile(id).and_then(|m| m.map)
}

pub fn combatant(w: &World, id: MobileId) -> Option<MobileId> {
    let ty = w.mobile(id)?.ty;
    (ty.combatant)(w, id)
}

pub fn hits(w: &World, id: MobileId) -> i32 {
    w.mobile(id).map_or(0, |m| m.hits)
}

pub fn hits_max(w: &World, id: MobileId) -> i32 {
    w.mobile(id).map_or(0, |m| (m.ty.hits_max)(w, id))
}

pub fn player(w: &World, id: MobileId) -> bool {
    w.mobile(id).is_some_and(|m| m.player)
}

pub fn access_level(w: &World, id: MobileId) -> AccessLevel {
    w.mobile(id).map_or(AccessLevel::Player, |m| m.access_level)
}

/// `SendLocalizedMessage(number)`.
pub fn msg(w: &mut World, id: MobileId, number: i32) {
    messages::send_localized_message(w, id, number, "", 0x3B2);
}

/// `SendLocalizedMessage(number, args)`.
pub fn msg_args(w: &mut World, id: MobileId, number: i32, args: &str) {
    messages::send_localized_message(w, id, number, args, 0x3B2);
}

pub fn send_message(w: &mut World, id: MobileId, text: &str) {
    messages::send_message(w, id, text);
}

pub fn play_sound(w: &mut World, id: MobileId, sound: i32) {
    effects::play_sound(w, id, sound);
}

pub fn fixed_effect(w: &mut World, id: MobileId, item_id: i32, speed: i32, duration: i32) {
    effects::fixed_effect(w, id, item_id, speed, duration);
}

#[allow(clippy::too_many_arguments)] // one parameter per input of the call
pub fn animate(
    w: &mut World,
    id: MobileId,
    action: i32,
    frames: i32,
    repeat: i32,
    forward: bool,
    rep: bool,
    delay: i32,
) {
    base::animate(w, id, action, frames, repeat, forward, rep, delay);
}

/// `CanBeHarmful(target)` (message = true, ignoreOurBlessedness = false).
pub fn can_be_harmful(w: &mut World, id: MobileId, target: MobileId) -> bool {
    base::can_be_harmful(w, id, target)
}

pub fn can_be_harmful_quiet(w: &mut World, id: MobileId, target: MobileId) -> bool {
    base::can_be_harmful_msg(w, id, target, false)
}

pub fn do_harmful(w: &mut World, id: MobileId, target: MobileId) {
    base::do_harmful_base(w, id, target, false);
}

pub fn in_los(w: &mut World, id: MobileId, target: MobileId) -> bool {
    combat::in_los(w, id, target)
}

/// `InRange(other, range)` between two mobiles.
pub fn in_range(w: &World, a: MobileId, b: MobileId, range: i32) -> bool {
    match (w.mobile(a), w.mobile(b)) {
        (Some(x), Some(y)) => {
            muo_primitives::utility::in_range_2d(x.location.into(), y.location.into(), range)
        }
        _ => false,
    }
}

pub fn in_range_point(w: &World, a: MobileId, p: Point3D, range: i32) -> bool {
    w.mobile(a)
        .is_some_and(|x| muo_primitives::utility::in_range_2d(x.location.into(), p.into(), range))
}

/// `GetMobilesInRange(range)`.
pub fn mobiles_in_range(w: &World, id: MobileId, range: i32) -> Vec<MobileId> {
    enumerators::get_mobiles_in_range(w, id, range)
}

pub fn get_direction_to(w: &World, a: MobileId, b: MobileId) -> Direction {
    let p = location(w, b);
    effects::get_direction_to(w, a, p.x, p.y, false)
}

pub fn set_direction(w: &mut World, id: MobileId, d: Direction) {
    effects::set_direction(w, id, d);
}

pub fn set_frozen(w: &mut World, id: MobileId, v: bool) {
    props_a::set_frozen(w, id, v);
}

/// `Combatant = v`: virtual (BaseCreature refuses a guard's friendly target, BaseFamiliar its caster's enemies).
pub fn set_combatant(w: &mut World, id: MobileId, v: Option<MobileId>) {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.set_combatant)(w, id, v),
        None => base::set_combatant_base(w, id, v),
    }
}

pub fn set_warmode(w: &mut World, id: MobileId, v: bool) {
    props_b::set_warmode(w, id, v);
}

pub fn damage(w: &mut World, id: MobileId, amount: i32, from: Option<MobileId>) {
    base::damage_base(w, id, amount, from, true, false);
}

pub fn paralyze(w: &mut World, id: MobileId, d: TimeSpan) {
    base::paralyze(w, id, d);
}

pub fn say_to(w: &mut World, id: MobileId, to: MobileId, text: &str) {
    speech::say_to(w, id, to, text);
}

pub fn local_overhead(w: &mut World, id: MobileId, hue: i32, text: &str) {
    messages::local_overhead_message(w, id, MessageType::REGULAR, hue, false, text);
}

pub fn check_skill(
    w: &mut World,
    id: MobileId,
    skill: muo_server::skills::SkillName,
    min: f64,
    max: f64,
) -> bool {
    stats::check_skill(w, id, skill, min, max)
}

pub fn entity(id: MobileId) -> EntityId {
    EntityId::Mobile(id)
}

pub fn item_entity(id: ItemId) -> EntityId {
    EntityId::Item(id)
}

pub fn use_skill(w: &mut World, id: MobileId, skill: muo_server::skills::SkillName) -> bool {
    base::use_skill(w, id, skill)
}
