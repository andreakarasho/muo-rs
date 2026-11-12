//! WeaponAbility and its 31 subclasses, and HitLower.
//!
//! A `WeaponAbility` is a singleton without state; here it is the [`Ability`] enum (the index in
//! `WeaponAbility.Abilities`) and each virtual is a `match`. The statics (`WeaponAbility.Table`, `BleedAttack._table`,
//! `Block._table`, ..) are [`AbilityState`] in `w.resources`; their timers capture ids.
//!
//! Systems outside the weapons (special moves, buffs, mounts, spells) are in [`super::weapon_hooks`].

use indexmap::{IndexMap, IndexSet};
use muo_primitives::{MessageType, Point3D, TimeSpan, utility};
use muo_server::World;
use muo_server::effects::{EffectLayer, Targeted};
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::mobile::MobileDelta;
use muo_server::mobile::ResistanceType;
use muo_server::mobile::base as mb;
use muo_server::mobile::part1::combat::in_los;
use muo_server::mobile::part1::props_b;
use muo_server::mobile::part2::{effects as fx, messages as msgs};
use muo_server::mods::{ModId, ResistanceMod};
use muo_server::skills::{self, SkillName};
use muo_server::timer::{self, TimerHandle};

use super::weapon::{self, Ability};
use super::weapon_combat as wc;
use super::weapon_hooks as hk;
use crate::aos::{AosAttribute, AosWeaponAttribute, AttrKind};
use crate::core;
use crate::misc::aos_helpers::{self, DamageOptions, DamageSplit};

/// `WeaponAbility.Abilities`, by index (0 is the null slot).
pub const ABILITIES: [Option<Ability>; 32] = {
    use Ability::*;
    [
        None,
        Some(ArmorIgnore),
        Some(BleedAttack),
        Some(ConcussionBlow),
        Some(CrushingBlow),
        Some(Disarm),
        Some(Dismount),
        Some(DoubleStrike),
        Some(InfectiousStrike),
        Some(MortalStrike),
        Some(MovingShot),
        Some(ParalyzingBlow),
        Some(ShadowStrike),
        Some(WhirlwindAttack),
        Some(RidingSwipe),
        Some(FrenziedWhirlwind),
        Some(Block),
        Some(DefenseMastery),
        Some(NerveStrike),
        Some(TalonStrike),
        Some(Feint),
        Some(DualWield),
        Some(DoubleShot),
        Some(ArmorPierce),
        Some(Bladeweave),
        Some(ForceArrow),
        Some(LightningArrow),
        Some(PsychicAttack),
        Some(SerpentArrow),
        Some(ForceOfNature),
        Some(InfusedThrow),
        Some(MysticArc),
    ]
};

/// A WorldLocation: where a defender was when it was hit (it may be dead or internalized afterwards).
pub type WorldLocation = (muo_primitives::Point3D, Option<muo_server::map::MapId>);

struct DefenseMasteryInfo {
    malus: i32,
    resist_mod: ModId,
    timer: TimerHandle,
}

struct PsychicInfo {
    spell_damage_malus: i32,
    mana_cost_malus: i32,
    done_increase: bool,
    timer: TimerHandle,
}

struct ForceArrowInfo {
    defender: MobileId,
    defense_chance_malus: i32,
    expires: muo_primitives::DateTime,
    timer: TimerHandle,
}

struct ForceOfNatureInfo {
    target: MobileId,
    hits: i32,
    last_hit: muo_primitives::DateTime,
    timer: TimerHandle,
}

/// The statics of the ability classes.
#[derive(Default)]
pub struct AbilityState {
    /// `WeaponAbility.Table`.
    table: IndexMap<MobileId, Ability>,
    /// `WeaponAbility._abilityCooldown`.
    cooldown: IndexMap<MobileId, TimerHandle>,
    bleed: IndexMap<MobileId, TimerHandle>,
    /// `Block._table`: the bonus and its timer.
    block: IndexMap<MobileId, (i32, TimerHandle)>,
    defense_mastery: IndexMap<MobileId, DefenseMasteryInfo>,
    /// `Bladeweave._newAttack`: the redirect and its cliloc.
    bladeweave: IndexMap<MobileId, (Ability, i32)>,
    /// `DualWield.Registry`: bonus swing speed.
    dual_wield: IndexMap<MobileId, (i32, TimerHandle)>,
    /// `Feint.Registry`, keyed by the defender (the feinting mobile's target): `(attacker, reduction, timer)`.
    feint: IndexMap<MobileId, (MobileId, i32, TimerHandle)>,
    force_arrow: IndexMap<MobileId, Vec<ForceArrowInfo>>,
    force_of_nature: IndexMap<MobileId, ForceOfNatureInfo>,
    frenzied: IndexMap<MobileId, TimerHandle>,
    mortal_strike: IndexMap<MobileId, TimerHandle>,
    paralyzing_immunity: IndexMap<MobileId, TimerHandle>,
    psychic: IndexMap<MobileId, PsychicInfo>,
    talon_strike: IndexSet<MobileId>,
    hit_lower_attack: IndexSet<MobileId>,
    hit_lower_defense: IndexSet<MobileId>,
}

fn st(w: &mut World) -> &mut AbilityState {
    w.resources.get_mut::<AbilityState>()
}

fn st_ref(w: &World) -> Option<&AbilityState> {
    w.resources.get::<AbilityState>()
}

fn msg(w: &mut World, m: MobileId, n: i32) {
    muo_server::item::deps::send_localized_message(w, m, n);
}

fn msg_args(w: &mut World, m: MobileId, n: i32, args: &str) {
    msgs::send_localized_message(w, m, n, args, 0x3B2);
}

fn val(w: &mut World, m: MobileId, s: SkillName) -> f64 {
    skills::value(w, m, s)
}

fn base_skill(w: &mut World, m: MobileId, s: SkillName) -> f64 {
    skills::base(w, m, s)
}

fn is_player(w: &World, m: MobileId) -> bool {
    w.mobile(m).is_some_and(|x| x.player)
}

fn mobile_is_named(w: &World, m: MobileId, full_name: &str) -> bool {
    let mut t = w.mobile(m).map(|x| x.ty);
    while let Some(c) = t {
        if c.full_name == full_name {
            return true;
        }
        t = c.parent;
    }
    false
}

fn alive(w: &World, m: MobileId) -> bool {
    w.mobile(m)
        .is_some_and(|x| !x.deleted && (x.ty.alive)(w, m))
}

fn deleted(w: &World, m: MobileId) -> bool {
    w.mobile(m).is_none_or(|x| x.deleted)
}

fn loc(w: &World, m: MobileId) -> muo_primitives::Point3D {
    w.mobile(m)
        .map_or(muo_primitives::Point3D::ZERO, |x| x.location)
}

/// `AOS.Damage(m, from, damage, ignoreArmor, phys, fire, cold, pois, nrgy, chaos, direct)`.
#[allow(clippy::too_many_arguments)] // AOS.Damage(m, from, damage, ignoreArmor, phys, fire, cold, pois, nrgy, chaos, direct)
fn aos_damage(
    w: &mut World,
    m: MobileId,
    from: Option<MobileId>,
    damage: i32,
    ignore_armor: bool,
    split: [i32; 5],
    chaos: i32,
    direct: i32,
) -> i32 {
    aos_helpers::damage(
        w,
        m,
        from,
        damage,
        DamageSplit {
            phys: split[0],
            fire: split[1],
            cold: split[2],
            pois: split[3],
            nrgy: split[4],
        },
        DamageOptions {
            ignore_armor,
            chaos,
            direct,
            ..Default::default()
        },
    )
}

fn weapon_of(w: &mut World, m: MobileId) -> Option<ItemId> {
    hk::weapon_item(w, m)
}

/// `weapon.PrimaryAbility`/`SecondaryAbility` of the wielded `BaseWeapon`.
fn weapon_abilities(w: &mut World, m: MobileId) -> Option<(Option<Ability>, Option<Ability>)> {
    let wp = weapon_of(w, m)?;
    let c = weapon::class(w, wp);
    Some((c.primary_ability, c.secondary_ability))
}

// ---------------------------------------------------------------------------------------------------------------
// WeaponAbility virtual properties
// ---------------------------------------------------------------------------------------------------------------

/// `BaseMana`.
pub fn base_mana(w: &World, a: Ability) -> i32 {
    use Ability::*;
    match a {
        ArmorIgnore | ArmorPierce | Bladeweave | BleedAttack | Block | DefenseMastery
        | DoubleShot | DoubleStrike | DualWield | Feint | FrenziedWhirlwind | MortalStrike
        | NerveStrike | ParalyzingBlow | PsychicAttack | RidingSwipe | TalonStrike => 30,
        ConcussionBlow | CrushingBlow | InfusedThrow => 25,
        Disarm | Dismount | ForceArrow | LightningArrow | MysticArc | ShadowStrike => 20,
        InfectiousStrike | MovingShot | WhirlwindAttack => 15,
        SerpentArrow => 25,
        ForceOfNature => {
            if core::sa(w) {
                35
            } else {
                40
            }
        }
    }
}

/// `AccuracyBonus`.
pub fn accuracy_bonus(a: Ability) -> i32 {
    if a == Ability::MovingShot { -25 } else { 0 }
}

/// `DamageScalar`.
pub fn damage_scalar(a: Ability) -> f64 {
    use Ability::*;
    match a {
        ArmorIgnore | DoubleStrike => 0.9,
        ArmorPierce | CrushingBlow => 1.5,
        ShadowStrike => 1.25,
        TalonStrike => 1.2,
        _ => 1.0,
    }
}

fn requires_se(a: Ability) -> bool {
    matches!(a, Ability::ArmorPierce | Ability::RidingSwipe)
}

fn validates_during_hit(a: Ability) -> bool {
    a != Ability::MovingShot
}

fn requires_tactics(w: &mut World, a: Ability, from: MobileId) -> bool {
    use Ability::*;
    match a {
        DoubleShot | FrenziedWhirlwind | ForceArrow | InfectiousStrike | ShadowStrike => false,
        Disarm => {
            !weapon_of(w, from).is_some_and(|wp| weapon::skill(w, wp) == SkillName::Wrestling)
        }
        ParalyzingBlow => {
            core::aos(w)
                && !weapon_of(w, from)
                    .is_some_and(|wp| weapon::skill(w, wp) == SkillName::Wrestling)
        }
        _ => true,
    }
}

fn requires_secondary_skill(a: Ability) -> bool {
    use Ability::*;
    matches!(
        a,
        ArmorPierce
            | Block
            | DefenseMastery
            | DoubleShot
            | DualWield
            | Feint
            | FrenziedWhirlwind
            | NerveStrike
            | RidingSwipe
            | SerpentArrow
            | ShadowStrike
            | TalonStrike
    )
}

fn required_secondary_skill(a: Ability) -> f64 {
    match a {
        Ability::SerpentArrow => 60.0,
        Ability::ShadowStrike => 80.0,
        _ => 50.0,
    }
}

fn secondary_skill_name(w: &mut World, a: Ability, from: MobileId) -> SkillName {
    use Ability::*;
    match a {
        DualWield | TalonStrike => SkillName::Ninjitsu,
        NerveStrike | RidingSwipe => SkillName::Bushido,
        SerpentArrow => SkillName::Poisoning,
        ShadowStrike => SkillName::Stealth,
        _ => {
            if base_skill(w, from, SkillName::Ninjitsu) > base_skill(w, from, SkillName::Bushido) {
                SkillName::Ninjitsu
            } else {
                SkillName::Bushido
            }
        }
    }
}

fn required_by_slot(
    w: &mut World,
    a: Ability,
    from: MobileId,
    primary: f64,
    secondary: f64,
    bladeweave_counts: bool,
) -> f64 {
    if let Some((p, s)) = weapon_abilities(w, from) {
        let bw = |x: Option<Ability>| bladeweave_counts && x == Some(Ability::Bladeweave);
        if p == Some(a) || bw(p) {
            return primary;
        }
        if s == Some(a) || bw(s) {
            return secondary;
        }
    }
    200.0
}

/// `GetRequiredSkill(from)`.
fn required_skill(w: &mut World, a: Ability, from: MobileId) -> f64 {
    required_by_slot(w, a, from, 70.0, 90.0, true)
}

/// `GetRequiredTactics(from)`.
fn required_tactics(w: &mut World, a: Ability, from: MobileId) -> f64 {
    match a {
        Ability::CrushingBlow | Ability::Disarm => required_by_slot(w, a, from, 30.0, 60.0, false),
        _ => required_by_slot(w, a, from, 70.0, 90.0, true),
    }
}

fn skill_localization(skill: SkillName) -> i32 {
    match skill {
        // You need ~1_SKILL_REQUIREMENT~ Bushido or Ninjitsu skill to perform that attack!
        SkillName::Bushido | SkillName::Ninjitsu => 1063347,
        // You lack the required poisoning to perform that attack
        SkillName::Poisoning => 1060184,
        // You need ~1_SKILL_REQUIREMENT~ weapon and tactics skill to perform that attack
        _ => 1157351,
    }
}

fn double_string(v: f64) -> String {
    // `double.ToString()`: "70" for 70.0, "47.5" otherwise.
    format!("{v}")
}

/// `CalculateMana(from)`.
pub fn calculate_mana(w: &mut World, a: Ability, from: MobileId) -> i32 {
    let mut mana = base_mana(w, a);

    let mut skill_total = 0.0;
    for s in [
        SkillName::Swords,
        SkillName::Macing,
        SkillName::Fencing,
        SkillName::Archery,
        SkillName::Parry,
        SkillName::Lumberjacking,
        SkillName::Stealth,
        SkillName::Poisoning,
        SkillName::Bushido,
        SkillName::Ninjitsu,
    ] {
        skill_total += val(w, from, s);
    }

    if skill_total >= 300.0 {
        mana -= 10;
    } else if skill_total >= 200.0 {
        mana -= 5;
    }

    let mut scalar = hk::mind_rot_scalar(w, from).unwrap_or(1.0);

    // Lower Mana Cost = 40%
    let lmc = aos_helpers::attribute_value(w, from, AosAttribute::LowerManaCost).min(40);

    scalar -= f64::from(lmc) / 100.0;
    mana = (f64::from(mana) * scalar) as i32;

    // Using a special move within 3 seconds of the previous special move costs double mana
    if on_cooldown(w, from) {
        mana *= 2;
    }

    mana
}

fn on_cooldown(w: &World, m: MobileId) -> bool {
    st_ref(w).is_some_and(|s| s.cooldown.contains_key(&m))
}

/// `CheckWeaponSkill(from)`.
fn check_weapon_skill(w: &mut World, a: Ability, from: MobileId) -> bool {
    let Some(wp) = weapon_of(w, from) else {
        return false;
    };
    let wskill = weapon::skill(w, wp);
    let req_skill = required_skill(w, a, from);
    let req_tactics = core::ml(w) && requires_tactics(w, a, from);

    if req_tactics && base_skill(w, from, SkillName::Tactics) < required_tactics(w, a, from) {
        // You need ~1_SKILL_REQUIREMENT~ weapon and tactics skill to perform that attack
        msg_args(w, from, 1079308, &double_string(req_skill));
        return false;
    }

    let req_secondary = required_secondary_skill(a);
    let secondary = secondary_skill_name(w, a, from);

    if requires_secondary_skill(a) && base_skill(w, from, secondary) < req_secondary {
        let loc = skill_localization(secondary);
        if loc == 1060184 {
            msg(w, from, loc);
        } else {
            msg_args(w, from, loc, &double_string(req_secondary));
        }
        return false;
    }

    if base_skill(w, from, wskill) >= req_skill {
        return true;
    }

    // <UBWS>
    let use_best = super::attr(
        w,
        wp,
        AttrKind::Weapon,
        AosWeaponAttribute::UseBestSkill as u32,
    );
    if use_best > 0
        && (base_skill(w, from, SkillName::Swords) >= req_skill
            || base_skill(w, from, SkillName::Macing) >= req_skill
            || base_skill(w, from, SkillName::Fencing) >= req_skill)
    {
        return true;
    }

    // You need ~1_SKILL_REQUIREMENT~ weapon skill to perform that attack
    msg_args(w, from, 1060182, &double_string(req_skill));
    false
}

/// `CheckSkills(from)`.
fn check_skills(w: &mut World, a: Ability, from: MobileId) -> bool {
    match a {
        Ability::ParalyzingBlow => {
            if !check_weapon_skill(w, a, from) {
                return false;
            }
            let fists = weapon_of(w, from)
                .and_then(|i| w.item(i))
                .is_some_and(|i| super::is_named(i.ty, "Server.Items.Fists"));
            if core::aos(w) || !fists {
                return true;
            }
            if val(w, from, SkillName::Anatomy) >= 80.0 {
                return true;
            }
            msg(w, from, 1061811); // You lack the required anatomy skill to perform that attack!
            false
        }
        Ability::ShadowStrike => {
            if !check_weapon_skill(w, a, from) {
                return false;
            }
            if val(w, from, SkillName::Stealth) >= 80.0 {
                return true;
            }
            msg(w, from, 1060183); // You lack the required stealth to perform that attack
            false
        }
        _ => check_weapon_skill(w, a, from),
    }
}

/// `CheckMana(from, consume)`.
pub fn check_mana(w: &mut World, a: Ability, from: MobileId, consume: bool) -> bool {
    if crate::creature::vtable::ctype(w, from).is_some_and(|t| (t.has_mana_override)(w, from)) {
        return true;
    }

    let mana = calculate_mana(w, a, from);

    if w.mobile(from).map_or(0, |m| m.mana) < mana {
        // You need ~1_MANA_REQUIREMENT~ mana to perform that attack
        msg_args(w, from, 1060181, &mana.to_string());
        return false;
    }

    if consume {
        if !on_cooldown(w, from) {
            let h = timer::start(w, TimeSpan::from_secs(3), TimeSpan::ZERO, 1, move |w| {
                remove_cooldown(w, from);
            });
            st(w).cooldown.insert(from, h);
        }
        let cur = w.mobile(from).map_or(0, |m| m.mana);
        props_b::set_mana(w, from, cur - mana);
    }

    true
}

fn remove_cooldown(w: &mut World, m: MobileId) {
    if let Some(mut h) = st(w).cooldown.shift_remove(&m) {
        h.cancel(w);
    }
}

fn option_name(a: Ability) -> &'static str {
    use Ability::*;
    match a {
        ArmorIgnore => "Armor Ignore",
        ArmorPierce => "Armor Pierce",
        Bladeweave => "Bladeweave",
        BleedAttack => "Bleed Attack",
        Block => "Block",
        ConcussionBlow => "Concussion Blow",
        CrushingBlow => "Crushing Blow",
        DefenseMastery => "Defense Mastery",
        Disarm => "Disarm",
        Dismount => "Dismount",
        DoubleShot => "Double Shot",
        DoubleStrike => "Double Strike",
        DualWield => "Dual Wield",
        Feint => "Feint",
        ForceArrow => "Force Arrow",
        ForceOfNature => "Force of Nature",
        FrenziedWhirlwind => "Frenzied Whirlwind",
        InfectiousStrike => "Infectious Strike",
        InfusedThrow => "Infused Throw",
        LightningArrow => "Lightning Arrow",
        MortalStrike => "Mortal Strike",
        MovingShot => "Moving Shot",
        MysticArc => "Mystic Arc",
        NerveStrike => "Nerve Strike",
        ParalyzingBlow => "Paralyzing Blow",
        PsychicAttack => "Psychic Attack",
        RidingSwipe => "Riding Swipe",
        SerpentArrow => "Serpent Arrow",
        ShadowStrike => "Shadow Strike",
        TalonStrike => "Talon Strike",
        WhirlwindAttack => "Whirlwind Attack",
    }
}

/// `Validate(from)`.
pub fn validate(w: &mut World, a: Ability, from: MobileId) -> bool {
    if a == Ability::DoubleShot {
        // DoubleShot.Validate: base first, then the mounted check.
        if validate_base(w, a, from) {
            if props_b::mounted(w, from) {
                return true;
            }
            msg(w, from, 1070770); // You can only execute this attack while mounted!
            clear_current_ability(w, from);
        }
        return false;
    }
    validate_base(w, a, from)
}

fn validate_base(w: &mut World, a: Ability, from: MobileId) -> bool {
    if !is_player(w, from) {
        return true;
    }

    if w.mobile(from).and_then(|m| m.net_state).is_none() {
        return false;
    }

    if requires_se(a) && !hk::net_supports_se(w, from) {
        msg(w, from, 1063456); // You must upgrade to Samurai Empire in order to use that ability.
        return false;
    }

    if hk::honorable_execution_is_under_penalty(w, from) || hk::animal_form_under(w, from, None) {
        msg(w, from, 1063024); // You cannot perform this special move right now.
        return false;
    }

    if core::ml(w) && w.mobile(from).is_some_and(|m| m.spell.is_some()) {
        msg(w, from, 1063024); // You cannot perform this special move right now.
        return false;
    }

    if !hk::duel_allow_special_ability(w, from, option_name(a), true) {
        return false;
    }

    check_skills(w, a, from)
}

// ---------------------------------------------------------------------------------------------------------------
// Current ability
// ---------------------------------------------------------------------------------------------------------------

/// `IsWeaponAbility(m, a)`.
fn is_weapon_ability(w: &mut World, m: MobileId, a: Option<Ability>) -> bool {
    let Some(a) = a else { return true };
    if !is_player(w, m) {
        return true;
    }
    weapon_abilities(w, m).is_some_and(|(p, s)| p == Some(a) || s == Some(a))
}

/// `GetCurrentAbility(m)`.
pub fn current_ability(w: &mut World, m: MobileId) -> Option<Ability> {
    if !core::aos(w) {
        clear_current_ability(w, m);
        return None;
    }

    let a = st_ref(w).and_then(|s| s.table.get(&m).copied());

    if !is_weapon_ability(w, m, a) {
        clear_current_ability(w, m);
        return None;
    }

    if let Some(ab) = a
        && validates_during_hit(ab)
        && (!validate(w, ab, m) || !check_mana(w, ab, m, false))
    {
        clear_current_ability(w, m);
        return None;
    }

    a
}

/// `SetCurrentAbility(m, a)`.
pub fn set_current_ability(w: &mut World, m: MobileId, a: Option<Ability>) -> bool {
    if !core::aos(w) {
        clear_current_ability(w, m);
        return false;
    }

    if !is_weapon_ability(w, m, a) {
        clear_current_ability(w, m);
        return false;
    }

    if let Some(ab) = a
        && (!validate(w, ab, m) || !check_mana(w, ab, m, false))
    {
        clear_current_ability(w, m);
        return false;
    }

    match a {
        None => {
            st(w).table.shift_remove(&m);
        }
        Some(ab) => {
            hk::special_move_clear(w, m);
            st(w).table.insert(m, ab);
        }
    }

    true
}

/// `ClearCurrentAbility(m)`.
pub fn clear_current_ability(w: &mut World, m: MobileId) {
    st(w).table.shift_remove(&m);

    if core::aos(w)
        && let Some(ns) = w.mobile(m).and_then(|x| x.net_state)
    {
        send_clear_weapon_ability(w, ns);
    }
}

/// `SendClearWeaponAbility()`.
pub fn send_clear_weapon_ability(w: &mut World, ns: muo_server::net_state::NetStateId) {
    if !w.cannot_send_packets(ns) {
        w.send(ns, &[0xBF, 0x00, 0x5, 0x00, 0x21]);
    }
}

/// `WeaponAbilityPackets.SetAbility`: the 0xBF/0x19 packet's body for a mobile.
pub fn set_ability_packet(w: &mut World, m: MobileId, index: i32) {
    if index >= 1 && (index as usize) < ABILITIES.len() {
        if props_b::mounted(w, m) && index == 6 {
            // dismount
            clear_current_ability(w, m);
            msg(w, m, 1061283); // You cannot perform that attack while mounted!
            return;
        }
        set_current_ability(w, m, ABILITIES[index as usize]);
    } else {
        clear_current_ability(w, m);
    }
}

/// `WeaponAbilityPackets.Configure`: the encoded packet 0xBF/0x19.
pub fn configure_packets(w: &mut World) {
    fn set_ability(w: &mut World, ns: &mut muo_net::net_state::NetState, _e: u32, body: &[u8]) {
        let id = muo_server::net_glue::ensure_net_state(w, ns);
        let Some(m) = w.net_state(id).and_then(|d| d.mobile) else {
            return;
        };
        let Ok(index) = muo_server::encoded_reader::EncodedReader::new(body).read_i32() else {
            return;
        };
        set_ability_packet(w, m, index);
    }
    w.resources
        .get_mut::<crate::network::IncomingRegistrations>()
        .0
        .push(crate::network::Registration::Encoded {
            id: 0x19,
            in_game: true,
            on_receive: set_ability,
        });
}

/// `SendToggleSpecialAbility(abilityId, active)`.
pub fn send_toggle_special_ability(
    w: &mut World,
    ns: muo_server::net_state::NetStateId,
    ability_id: i16,
    active: bool,
) {
    if w.cannot_send_packets(ns) {
        return;
    }
    let mut b = vec![0xBF, 0, 8, 0, 0x25];
    b[1..3].copy_from_slice(&8_u16.to_be_bytes());
    b[3..5].copy_from_slice(&0x25_i16.to_be_bytes());
    b.extend_from_slice(&ability_id.to_be_bytes());
    b.push(u8::from(active));
    w.send(ns, &b);
}

// ---------------------------------------------------------------------------------------------------------------
// OnBeforeSwing / OnBeforeDamage / OnHit / OnMiss
// ---------------------------------------------------------------------------------------------------------------

/// `OnBeforeSwing(attacker, defender)`.
pub fn on_before_swing(w: &mut World, a: Ability, attacker: MobileId, defender: MobileId) -> bool {
    match a {
        Ability::Bladeweave => {
            if !validate(w, a, attacker) || !check_mana(w, a, attacker, false) {
                return false;
            }
            // Bladeweave is only from 90 fighting and tactics skill, so you need only to check bushido/ninjitsu
            // value over 50 to perform block and feint.
            let required_bushido =
                val(w, attacker, SkillName::Bushido).max(val(w, attacker, SkillName::Ninjitsu));
            let ran = utility::random(&mut w.rng, if required_bushido >= 50.0 { 9 } else { 7 });
            let redirect = bladeweave_redirect(ran);
            st(w).bladeweave.insert(attacker, redirect);
            on_before_swing(w, redirect.0, attacker, defender)
        }
        Ability::MovingShot => validate(w, a, attacker) && check_mana(w, a, attacker, false),
        Ability::NerveStrike | Ability::ParalyzingBlow => {
            if mb::paralyzed(w, defender) {
                msg(w, attacker, 1061923); // The target is already frozen.
                return false;
            }
            true
        }
        _ => true,
    }
}

fn bladeweave_redirect(n: i32) -> (Ability, i32) {
    use Ability::*;
    match n {
        0 => (ArmorIgnore, 1028838),
        1 => (BleedAttack, 1028839),
        2 => (ConcussionBlow, 1028840),
        3 => (CrushingBlow, 1028841),
        4 => (DoubleStrike, 1028844),
        5 => (MortalStrike, 1028846),
        6 => (ParalyzingBlow, 1028848),
        7 => (Block, 1028853),
        _ => (Feint, 1028857),
    }
}

/// `Bladeweave.BladeWeaving(attacker, out a)`.
pub fn blade_weaving(w: &World, attacker: MobileId) -> Option<Ability> {
    st_ref(w)?.bladeweave.get(&attacker).map(|b| b.0)
}

/// `OnBeforeDamage(attacker, defender)`.
pub fn on_before_damage(w: &mut World, a: Ability, attacker: MobileId, defender: MobileId) -> bool {
    match a {
        Ability::Bladeweave => match st_ref(w).and_then(|s| s.bladeweave.get(&attacker).copied()) {
            Some((n, _)) => on_before_damage(w, n, attacker, defender),
            None => true,
        },
        Ability::ConcussionBlow => {
            if !validate(w, a, attacker) || !check_mana(w, a, attacker, true) {
                return false;
            }
            clear_current_ability(w, attacker);

            msg(w, attacker, 1060165); // You have delivered a concussion!
            msg(w, defender, 1060166); // You feel disoriented!

            fx::play_sound(w, defender, 0x213);
            fx::fixed_particles_hue(w, defender, 0x377A, 1, 32, 9949, 1153, 0, EffectLayer::Head);

            let (dl, dm) = (loc(w, defender), w.mobile(defender).and_then(|m| m.map));
            muo_server::effects::send_moving_particles_layer(
                w,
                Targeted::point(Point3D::new(dl.x, dl.y, dl.z + 10), dm),
                Targeted::point(Point3D::new(dl.x, dl.y, dl.z + 20), dm),
                0x36FE,
                1,
                0,
                false,
                false,
                1133,
                3,
                9501,
                1,
                0,
                i32::from(EffectLayer::Waist.0),
                0x100,
            );

            let mut damage = 10; // Base damage is 10.

            let (hits, hits_max) = (
                w.mobile(defender).map_or(0, |m| m.hits),
                mb::hits_max(w, defender),
            );
            if hits_max > 0 {
                let hits_percent = f64::from(hits) / f64::from(hits_max) * 100.0;
                let mut mana_percent = 0.0;
                let (mana, mana_max) = (
                    w.mobile(defender).map_or(0, |m| m.mana),
                    w.mobile(defender)
                        .map_or(0, |m| (m.ty.mana_max)(w, defender)),
                );
                if mana_max > 0 {
                    mana_percent = f64::from(mana) / f64::from(mana_max) * 100.0;
                }
                damage += (((hits_percent - mana_percent).abs() / 4.0) as i32).min(20);
            }

            // Total damage is 10 + (0~20) = 10~30, physical, non-resistable.
            mb::damage(w, defender, damage, Some(attacker), true, false);
            true
        }
        _ => true,
    }
}

/// `OnMiss(attacker, defender)`.
pub fn on_miss(w: &mut World, a: Ability, attacker: MobileId, defender: MobileId) {
    match a {
        Ability::Bladeweave => {
            if let Some((n, _)) = st_ref(w).and_then(|s| s.bladeweave.get(&attacker).copied()) {
                on_miss(w, n, attacker, defender)
            }
            st(w).bladeweave.shift_remove(&attacker);
        }
        Ability::DoubleShot => double_shot_use(w, attacker, defender),
        Ability::MovingShot => {
            // Validates in OnSwing for accuracy scalar
            clear_current_ability(w, attacker);
            if check_mana(w, a, attacker, true) {
                msg(w, attacker, 1060089); // You fail to execute your special move
            }
        }
        _ => {}
    }
}

fn begin(w: &mut World, a: Ability, attacker: MobileId) -> bool {
    if !validate(w, a, attacker) || !check_mana(w, a, attacker, true) {
        return false;
    }
    clear_current_ability(w, attacker);
    true
}

/// `OnHit(attacker, defender, damage, worldLocation)`.
pub fn on_hit(
    w: &mut World,
    a: Ability,
    attacker: MobileId,
    defender: MobileId,
    // `OnHit(.., damage, ..)`: no ability reads it; the Bladeweave recursion only passes it on.
    _damage: i32,
    at: WorldLocation,
) {
    use Ability::*;
    match a {
        ArmorIgnore => {
            if !begin(w, a, attacker) {
                return;
            }
            msg(w, attacker, 1060076); // Your attack penetrates their armor!
            msg(w, defender, 1060077); // The blow penetrated your armor!
            fx::play_sound(w, defender, 0x56);
            fx::fixed_particles(w, defender, 0x3728, 200, 25, 9942, EffectLayer::Waist);
        }
        ArmorPierce => {
            if !begin(w, a, attacker) {
                return;
            }
            msg(w, attacker, 1063350); // You pierce your opponent's armor!
            msg(w, defender, 1063351); // Your attacker pierced your armor!
            fx::fixed_particles(w, defender, 0x3728, 1, 26, 0x26D6, EffectLayer::Waist);
        }
        Bladeweave => {
            if check_mana(w, a, attacker, false) {
                if let Some((n, cliloc)) =
                    st_ref(w).and_then(|s| s.bladeweave.get(&attacker).copied())
                {
                    msg_args(w, attacker, 1072841, &format!("#{cliloc}"));
                    on_hit(w, n, attacker, defender, _damage, at);
                }
                st(w).bladeweave.shift_remove(&attacker);
                clear_current_ability(w, attacker);
            }
        }
        BleedAttack => bleed_on_hit(w, a, attacker, defender),
        Block => {
            if !begin(w, a, attacker) {
                return;
            }
            msg(w, attacker, 1063345); // You block an attack!
            msg(w, defender, 1063346); // Your attack was blocked!
            fx::fixed_particles_hue(
                w,
                attacker,
                0x37C4,
                1,
                16,
                0x251D,
                0x39D,
                0x3,
                EffectLayer::RightHand,
            );
            let m = val(w, attacker, SkillName::Bushido).max(val(w, attacker, SkillName::Ninjitsu));
            let bonus = (10.0 * ((m - 50.0) / 70.0 + 5.0)) as i32;
            begin_block(w, attacker, bonus);
        }
        ConcussionBlow => {}
        CrushingBlow => {
            if !begin(w, a, attacker) {
                return;
            }
            msg(w, attacker, 1060090); // You have delivered a crushing blow!
            msg(w, defender, 1060091); // You take extra damage from the crushing attack!
            fx::play_sound(w, defender, 0x1E1);
            fx::fixed_particles(w, defender, 0, 1, 0, 9946, EffectLayer::Head);
            let (dl, dm) = (loc(w, defender), w.mobile(defender).and_then(|m| m.map));
            muo_server::effects::send_moving_particles_layer(
                w,
                Targeted::point(Point3D::new(dl.x, dl.y, dl.z + 50), dm),
                Targeted::point(Point3D::new(dl.x, dl.y, dl.z + 20), dm),
                0xFB4,
                1,
                0,
                false,
                false,
                0,
                3,
                9501,
                1,
                0,
                i32::from(EffectLayer::Head.0),
                0x100,
            );
        }
        DefenseMastery => {
            if !begin(w, a, attacker) {
                return;
            }
            msg(w, attacker, 1063353); // You perform a masterful defense!
            fx::fixed_particles_hue(
                w,
                attacker,
                0x375A,
                1,
                17,
                0x7F2,
                0x3E8,
                0x3,
                EffectLayer::Waist,
            );
            let m = val(w, attacker, SkillName::Bushido).max(val(w, attacker, SkillName::Ninjitsu));
            let modifier = (30.0 * ((m - 50.0) / 70.0)) as i32;

            end_defense(w, attacker);

            let rm = ResistanceMod::new(
                w,
                ResistanceType::Physical,
                Some("PhysicalResistDefenseMastery"),
                50 + modifier,
                None,
            );
            let mod_id = rm.id;
            mb::add_resistance_mod(w, attacker, rm);
            let timer = timer::start(w, TimeSpan::from_secs(3), TimeSpan::ZERO, 1, move |w| {
                end_defense(w, attacker)
            });
            st(w).defense_mastery.insert(
                attacker,
                DefenseMasteryInfo {
                    malus: 80 - modifier,
                    resist_mod: mod_id,
                    timer,
                },
            );
            mb::delta(w, attacker, MobileDelta::WEAPON_DAMAGE);
        }
        Disarm => disarm_on_hit(w, a, attacker, defender),
        Dismount => dismount_on_hit(w, a, attacker, defender),
        DoubleShot => double_shot_use(w, attacker, defender),
        DoubleStrike => double_strike_on_hit(w, a, attacker, defender),
        DualWield => {
            if !validate(w, a, attacker) || !check_mana(w, a, attacker, true) {
                return;
            }
            if let Some((_, mut h)) = st(w).dual_wield.shift_remove(&attacker) {
                h.cancel(w);
            }
            clear_current_ability(w, attacker);
            msg(w, attacker, 1063362); // You dually wield for increased speed!
            fx::fixed_particles_hue(
                w,
                attacker,
                0x3779,
                1,
                15,
                0x7F6,
                0x3E8,
                3,
                EffectLayer::LeftHand,
            );
            let bonus = (20.0 + 3.0 * (val(w, attacker, SkillName::Ninjitsu) - 50.0) / 7.0) as i32; // 20-50 % increase
            let h = timer::start(w, TimeSpan::from_secs(6), TimeSpan::ZERO, 1, move |w| {
                st(w).dual_wield.shift_remove(&attacker);
            });
            st(w).dual_wield.insert(attacker, (bonus, h));
        }
        Feint => {
            if !validate(w, a, attacker) || !check_mana(w, a, attacker, true) {
                return;
            }
            if let Some((_, _, mut h)) = st(w).feint.shift_remove(&defender) {
                h.cancel(w);
            }
            clear_current_ability(w, attacker);
            msg(w, attacker, 1063360); // You baffle your target with a feint!
            msg(w, defender, 1063361); // You were deceived by an attacker's feint!
            fx::fixed_particles_hue(
                w,
                attacker,
                0x3728,
                1,
                13,
                0x7F3,
                0x962,
                0,
                EffectLayer::Waist,
            );
            let skill =
                val(w, attacker, SkillName::Ninjitsu).max(val(w, attacker, SkillName::Bushido));
            // 20-50 % decrease in damage taken for 6 seconds
            let reduction = (20.0 + 3.0 * (skill - 50.0) / 7.0) as i32;
            let h = timer::start(w, TimeSpan::from_secs(6), TimeSpan::ZERO, 1, move |w| {
                // FeintTimer.OnTick: `Registry.Remove(_attacker)`; the table is keyed by the mobile hit
                // (`Registry.Add(defender, ..)`), so removing the attacker's key is a no-op (a known bug, kept).
                st(w).feint.shift_remove(&attacker);
            });
            st(w).feint.insert(defender, (attacker, reduction, h));
        }
        ForceArrow => force_arrow_on_hit(w, a, attacker, defender),
        ForceOfNature => {
            if !begin(w, a, attacker) {
                return;
            }
            force_of_nature_remove(w, attacker);
            msg(w, attacker, 1074374); // You attack your enemy with the force of nature!
            msg(w, defender, 1074375); // You are assaulted with great force!
            fx::play_sound(w, defender, 0x22F);
            fx::fixed_particles_hue(w, defender, 0x36CB, 1, 9, 9911, 67, 5, EffectLayer::Head);
            fx::fixed_particles_hue(w, defender, 0x374A, 1, 17, 9502, 1108, 4, EffectLayer::None);
            if core::sa(w) {
                let d = utility::random_from(&mut w.rng, 15, 20);
                aos_damage(w, defender, Some(attacker), d, false, [0; 5], 0, 100);
            }
            let secs = if core::sa(w) { 10 } else { 360 };
            let now = w.now;
            let h = timer::start(w, TimeSpan::from_secs(secs), TimeSpan::ZERO, 1, move |w| {
                force_of_nature_tick(w, attacker);
            });
            st(w).force_of_nature.insert(
                attacker,
                ForceOfNatureInfo {
                    target: defender,
                    hits: 1,
                    last_hit: now,
                    timer: h,
                },
            );
        }
        FrenziedWhirlwind => frenzied_on_hit(w, a, attacker, defender),
        InfectiousStrike => infectious_on_hit(w, a, attacker, defender),
        InfusedThrow => {
            if !check_mana(w, a, attacker, true) {
                return;
            }
            clear_current_ability(w, attacker);
            msg(w, attacker, 1149563); // The infused projectile strikes a target!
            msg(w, defender, 1149564); // You are struck by the infused projectile and take damage!
            aos_damage(w, defender, Some(attacker), 15, false, [0; 5], 100, 0);
            let defender_mounted = props_b::mounted(w, defender)
                || w.mobile(defender).is_some_and(|m| m.flying)
                || hk::animal_form_under(w, defender, None);
            let attacker_free =
                !props_b::mounted(w, attacker) && !w.mobile(attacker).is_some_and(|m| m.flying);
            if defender_mounted
                && attacker_free
                && !mobile_is_named(w, defender, "Server.Mobiles.ChaosDragoon")
                && !mobile_is_named(w, defender, "Server.Mobiles.ChaosDragoonElite")
            {
                fx::play_sound(w, defender, 0x140);
                fx::fixed_particles(w, defender, 0x3728, 10, 15, 9955, EffectLayer::Waist);
                do_dismount(w, attacker, defender, TimeSpan::from_secs(10), 0); // InfusedThrow: the default BlockMountType.Dazed
            } else {
                fx::fixed_effect(w, defender, 0x376A, 9, 32);
                fx::play_sound(w, defender, 0x204);
                let secs = if is_player(w, defender) { 3.0 } else { 6.0 };
                mb::paralyze(w, defender, TimeSpan::from_secs_f64(secs));
            }
        }
        LightningArrow => lightning_arrow_on_hit(w, a, attacker, defender, at),
        MortalStrike => {
            if !begin(w, a, attacker) {
                return;
            }
            msg(w, attacker, 1060086); // You deliver a mortal wound!
            msg(w, defender, 1060087); // You have been mortally wounded!
            fx::play_sound(w, defender, 0x1E1);
            fx::fixed_particles_hue(
                w,
                defender,
                0x37B9,
                244,
                25,
                9944,
                31,
                0,
                EffectLayer::Waist,
            );
            // Do not reset timer if one is already in place.
            if !is_wounded(w, defender) {
                let d = if is_player(w, defender) { 6 } else { 12 };
                begin_wound(w, defender, TimeSpan::from_secs(d));
            }
        }
        MovingShot => {
            // Validates in OnSwing for accuracy scalar
            clear_current_ability(w, attacker);
            if check_mana(w, a, attacker, true) {
                msg(w, attacker, 1060216); // Your shot was successful
            }
        }
        MysticArc => mystic_arc_on_hit(w, a, attacker, defender),
        NerveStrike => nerve_strike_on_hit(w, a, attacker, defender),
        ParalyzingBlow => {
            if !begin(w, a, attacker) {
                return;
            }
            if is_paralysis_immune(w, defender) {
                // Intentionally going after Mana consumption
                msg(w, attacker, 1070804); // Your target resists paralysis.
                msg(w, defender, 1070813); // You resist paralysis.
                return;
            }
            fx::fixed_effect(w, defender, 0x376A, 9, 32);
            fx::play_sound(w, defender, 0x204);
            msg(w, attacker, 1060163); // You deliver a paralyzing blow!
            msg(w, defender, 1060164); // The attack has temporarily paralyzed you!

            let duration = if is_player(w, defender) {
                PLAYER_FREEZE
            } else {
                NPC_FREEZE
            };
            if core::aos(w) {
                mb::paralyze(w, defender, duration);
            } else {
                mb::freeze(w, defender, duration);
            }
            begin_immunity(w, defender, duration + FREEZE_DELAY);
        }
        PsychicAttack => psychic_on_hit(w, a, attacker, defender),
        RidingSwipe => riding_swipe_on_hit(w, a, attacker, defender),
        SerpentArrow => {
            if !begin(w, a, attacker) {
                return;
            }
            let level = if mb::in_range_point3d(w, attacker, loc(w, defender), 2) {
                let t =
                    val(w, attacker, SkillName::Archery) + val(w, attacker, SkillName::Poisoning);
                if t > 199.8 {
                    3
                } else if t > 170.2 {
                    2
                } else if t > 130.2 {
                    1
                } else {
                    0
                }
            } else {
                0
            };
            msg(w, defender, 1112369); // You have been poisoned by a lethal arrow!
            if let Some(p) = crate::misc::poison_kinds::get_poison_by_level(w, level) {
                mb::apply_poison(w, defender, Some(attacker), p);
            }
            fx::fixed_particles(w, defender, 0x374A, 10, 15, 5021, EffectLayer::Waist);
            fx::play_sound(w, defender, 0x474);
        }
        ShadowStrike => {
            if !begin(w, a, attacker) {
                return;
            }
            msg(w, attacker, 1060078); // You strike and hide in the shadows!
            msg(w, defender, 1060079); // You are dazed by the attack and your attacker vanishes!
            let (al, am) = (loc(w, attacker), w.mobile(attacker).and_then(|m| m.map));
            let item = muo_server::item::effect_item::create(
                w,
                al,
                am,
                muo_server::item::effect_item::DEFAULT_DURATION,
            );
            muo_server::effects::send_location_particles(w, item, 0x376A, 8, 12, 9943);
            fx::play_sound(w, attacker, 0x482);
            fx::fixed_effect(w, defender, 0x37BE, 20, 25);
            mb::set_combatant(w, attacker, None);
            props_b::set_warmode(w, attacker, false);
            props_b::set_hidden(w, attacker, true);
        }
        TalonStrike => {
            if st_ref(w).is_some_and(|s| s.talon_strike.contains(&defender))
                || !validate(w, a, attacker)
                || !check_mana(w, a, attacker, true)
            {
                return;
            }
            clear_current_ability(w, attacker);
            msg(w, attacker, 1063358); // You deliver a talon strike!
            msg(w, defender, 1063359); // Your attacker delivers a talon strike!
            fx::fixed_particles_hue(
                w,
                defender,
                0x373A,
                1,
                17,
                0x26BC,
                0x662,
                0,
                EffectLayer::Waist,
            );
            let total = (10.0 * (val(w, attacker, SkillName::Ninjitsu) - 50.0) / 70.0 + 5.0) as i32; // 5 - 15 damage
            dot_timer(w, attacker, defender, total, DotKind::Talon);
            st(w).talon_strike.insert(defender);
        }
        WhirlwindAttack => whirlwind_on_hit(w, a, attacker, defender),
    }
}

// ---------------------------------------------------------------------------------------------------------------
// BleedAttack
// ---------------------------------------------------------------------------------------------------------------

fn bleed_on_hit(w: &mut World, a: Ability, attacker: MobileId, defender: MobileId) {
    if !begin(w, a, attacker) {
        return;
    }

    // Necromancers under Lich or Wraith Form are immune to Bleed Attacks.
    let immune = hk::bleed_immune_form(w, defender)
        || crate::creature::vtable::ctype(w, defender)
            .is_some_and(|t| (t.bleed_immune)(w, defender));
    if immune {
        msg(w, attacker, 1062052); // Your target is not affected by the bleed attack!
        return;
    }

    msg(w, attacker, 1060159); // Your target is bleeding!
    msg(w, defender, 1060160); // You are bleeding!

    if w.mobile(defender)
        .is_some_and(|m| m.ty.is(&crate::player_mobile::PLAYER_MOBILE))
    {
        msgs::local_overhead_message_number(w, defender, MessageType::REGULAR, 0x21, 1060757, ""); // You are bleeding profusely
        let name = props_b::name(w, defender).unwrap_or_default();
        msgs::nonlocal_overhead_message_number(
            w,
            defender,
            MessageType::REGULAR,
            0x21,
            1060758,
            &name,
        );
        // ~1_NAME~ is bleeding profusely
    }

    fx::play_sound(w, defender, 0x133);
    fx::fixed_particles_hue(
        w,
        defender,
        0x377A,
        244,
        25,
        9950,
        31,
        0,
        EffectLayer::Waist,
    );

    begin_bleed(w, defender, attacker);
}

/// `BleedAttack.IsBleeding(m)`.
pub fn is_bleeding(w: &World, m: MobileId) -> bool {
    st_ref(w).is_some_and(|s| s.bleed.contains_key(&m))
}

/// `BleedAttack.BeginBleed(m, from)`.
pub fn begin_bleed(w: &mut World, m: MobileId, from: MobileId) {
    if let Some(mut h) = st(w).bleed.shift_remove(&m) {
        h.cancel(w);
    }
    let mut index = 0;
    let h = timer::start(
        w,
        TimeSpan::from_secs(2),
        TimeSpan::from_secs(2),
        5,
        move |w| {
            do_bleed(w, m, from, 5 - index);
            if index == 4 {
                end_bleed(w, m, true);
            }
            index += 1;
        },
    );
    st(w).bleed.insert(m, h);
}

/// `BleedAttack.DoBleed(m, from, level)`.
pub fn do_bleed(w: &mut World, m: MobileId, from: MobileId, level: i32) {
    if alive(w, m) {
        let mut damage = super::random_min_max(w, level, level * 2);
        if !is_player(w, m) {
            damage *= 2;
        }
        fx::play_sound(w, m, 0x133);
        mb::damage(w, m, damage, Some(from), true, false);

        if let Some(blood) = crate::factory::make_item(w, "Server.Items.Blood") {
            let graphic = utility::random_from(&mut w.rng, 0x122A, 5);
            if let Some(ty) = w.item(blood).map(|i| i.ty) {
                (ty.set_item_id)(w, blood, graphic);
            }
            let (l, map) = w
                .mobile(m)
                .map_or((muo_primitives::Point3D::ZERO, None), |x| {
                    (x.location, x.map)
                });
            muo_server::item::ops::move_to_world_map(w, blood, l, map);
        }
    } else {
        end_bleed(w, m, false);
    }
}

/// `BleedAttack.EndBleed(m, message)`.
pub fn end_bleed(w: &mut World, m: MobileId, message: bool) {
    let Some(mut h) = st(w).bleed.shift_remove(&m) else {
        return;
    };
    h.cancel(w);
    if message {
        msg(w, m, 1060167); // The bleeding wounds have healed, you are no longer bleeding!
    }
}

// ---------------------------------------------------------------------------------------------------------------
// Block / DefenseMastery / DualWield / Feint / Bladeweave tables
// ---------------------------------------------------------------------------------------------------------------

fn begin_block(w: &mut World, m: MobileId, bonus: i32) {
    end_block(w, m);
    let h = timer::start(w, TimeSpan::from_secs(6), TimeSpan::ZERO, 1, move |w| {
        end_block(w, m)
    });
    st(w).block.insert(m, (bonus, h));
}

fn end_block(w: &mut World, m: MobileId) {
    if let Some((_, mut h)) = st(w).block.shift_remove(&m) {
        h.cancel(w);
    }
}

/// `Block.GetBonus(targ, ref bonus)`.
pub fn block_bonus(w: &World, m: MobileId) -> Option<i32> {
    st_ref(w)?.block.get(&m).map(|b| b.0)
}

fn end_defense(w: &mut World, m: MobileId) {
    let Some(mut info) = st(w).defense_mastery.shift_remove(&m) else {
        return;
    };
    mb::remove_resistance_mod(w, m, info.resist_mod);
    info.timer.cancel(w);
    // No message is sent to the player.
    mb::delta(w, m, MobileDelta::WEAPON_DAMAGE);
}

/// `DefenseMastery.GetMalus(targ, ref damageMalus)`.
pub fn defense_mastery_malus(w: &World, m: MobileId) -> Option<i32> {
    st_ref(w)?.defense_mastery.get(&m).map(|i| i.malus)
}

/// `DualWield.Registry[m].BonusSwingSpeed`.
pub fn dual_wield_swing_bonus(w: &World, m: MobileId) -> Option<i32> {
    st_ref(w)?.dual_wield.get(&m).map(|b| b.0)
}

/// `Feint.GetDamageReduction(attacker, defender, out damageReduction)`: `Registry.TryGetValue(attacker, ..) &&
/// timer.Defender == defender`. `Registry` is keyed by the mobile the feint hit (`Registry.Add(defender, timer)`) and
/// `timer.Defender` is that same mobile, so the test only holds when `attacker == defender`: the reduction is never
/// applied in a real fight. That behavior is kept.
pub fn feint_damage_reduction(w: &World, attacker: MobileId, defender: MobileId) -> Option<i32> {
    let (_, reduction, _) = st_ref(w)?.feint.get(&attacker)?;
    (attacker == defender).then_some(*reduction)
}

// ---------------------------------------------------------------------------------------------------------------
// Disarm / Dismount / DoubleShot / DoubleStrike
// ---------------------------------------------------------------------------------------------------------------

fn disarm_on_hit(w: &mut World, a: Ability, attacker: MobileId, defender: MobileId) {
    if !begin(w, a, attacker) {
        return;
    }

    use muo_server::layer::Layer;
    let find =
        |w: &World, l: Layer| muo_server::mobile::part2::items::find_item_on_layer(w, defender, l);
    let movable =
        |w: &World, i: Option<ItemId>| i.and_then(|i| w.item(i)).is_some_and(|x| x.movable());

    let mut to_disarm = find(w, Layer::OneHanded);
    if !movable(w, to_disarm) {
        to_disarm = find(w, Layer::TwoHanded);
    }

    let pack = props_b::backpack(w, defender);
    let immovable = to_disarm.is_some() && !movable(w, to_disarm);

    if pack.is_none() || immovable {
        msg(w, attacker, 1004001); // You cannot disarm your opponent.
    } else if (!core::ml(w) && to_disarm.is_none())
        || to_disarm.is_some_and(|i| {
            w.item(i).is_some_and(|x| {
                x.ty.is(&super::armor::BASE_SHIELD)
                    || super::is_named(x.ty, "Server.Items.Spellbook")
            })
        })
    {
        msg(w, attacker, 1060849); // Your target is already unarmed!
    } else {
        msg(w, attacker, 1060092); // You disarm their weapon!
        msg(w, defender, 1060093); // Your weapon has been disarmed!
        fx::play_sound(w, defender, 0x3B9);
        fx::fixed_particles(w, defender, 0x37BE, 232, 25, 9948, EffectLayer::LeftHand);
        // `pack.DropItem(toDisarm)`: with nothing in either hand (ML) the item is null and nothing moves.
        if let (Some(pack), Some(item)) = (pack, to_disarm) {
            muo_server::item::container::drop_item_default(w, pack, item);
        }
        if w.mobile(defender)
            .is_some_and(|m| m.ty.is(&crate::player_mobile::PLAYER_MOBILE))
        {
            hk::add_buff(w, defender, 0x3F, 1075637, BLOCK_EQUIP_DURATION); // BuffIcon.NoRearm
        }
        wc::block_equip(w, defender, BLOCK_EQUIP_DURATION);
    }
}

/// `Disarm.BlockEquipDuration`.
pub const BLOCK_EQUIP_DURATION: TimeSpan = TimeSpan::from_secs(5);

fn dismount_on_hit(w: &mut World, a: Ability, attacker: MobileId, defender: MobileId) {
    if !begin(w, a, attacker) {
        return;
    }

    if mobile_is_named(w, defender, "Server.Mobiles.ChaosDragoon")
        || mobile_is_named(w, defender, "Server.Mobiles.ChaosDragoonElite")
    {
        return;
    }

    let flying = |w: &World, m: MobileId| w.mobile(m).is_some_and(|x| x.flying);
    if props_b::mounted(w, attacker) || flying(w, attacker) {
        let lance = |w: &mut World, m: MobileId| {
            weapon_of(w, m)
                .and_then(|i| w.item(i))
                .is_some_and(|i| super::is_named(i.ty, "Server.Items.Lance"))
        };
        if !lance(w, attacker)
            || (!props_b::mounted(w, defender) && !flying(w, defender) && !lance(w, defender))
        {
            msg(w, attacker, 1061283); // You cannot perform that attack while mounted!
            return;
        }
    }

    let mount = props_b::mount(w, defender);
    if mount.is_none() && !hk::animal_form_under(w, defender, None) {
        msg(w, attacker, 1060848); // This attack only works on mounted targets
        return;
    }

    if core::ml(w)
        && mobile_is_named(w, attacker, "Server.Mobiles.LesserHiryu")
        && utility::random_double(&mut w.rng) < 0.8
    {
        return; // Lesser Hiryu have an 80% chance of missing this attack
    }

    do_dismount(w, attacker, defender, TimeSpan::from_secs(10), 0);

    if !props_b::mounted(w, attacker) {
        let d = super::random_min_max(w, 15, 25);
        aos_damage(
            w,
            defender,
            Some(attacker),
            d,
            false,
            [100, 0, 0, 0, 0],
            0,
            0,
        );
    }
}

/// `Dismount.DoDismount(attacker, defender, delay, type)`; `kind` is `BlockMountType` (0 = Dazed).
pub fn do_dismount(
    w: &mut World,
    attacker: MobileId,
    defender: MobileId,
    delay: TimeSpan,
    kind: i32,
) {
    msg(w, attacker, 1060082); // The force of your attack has dislodged them from their mount!

    if props_b::mounted(w, attacker) {
        msg(w, defender, 1062315); // You fall off your mount!
    } else {
        msg(w, defender, 1060083); // You fall off of your mount and take damage!
    }

    fx::play_sound(w, defender, 0x140);
    fx::fixed_particles(w, defender, 0x3728, 10, 15, 9955, EffectLayer::Waist);

    let is_pm = |w: &World, m: MobileId| {
        w.mobile(m)
            .is_some_and(|x| x.ty.is(&crate::player_mobile::PLAYER_MOBILE))
    };
    if is_pm(w, defender) {
        let name = props_b::name(w, attacker).unwrap_or_default();
        if hk::animal_form_under(w, defender, None) {
            msg_args(w, defender, 1114066, &name); // ~1_NAME~ knocked you out of animal form!
        } else if w.mobile(defender).is_some_and(|m| m.flying) {
            msg_args(w, defender, 1113590, &name); // You have been grounded by ~1_NAME~!
        } else if props_b::mounted(w, defender) {
            msg(w, defender, 1040023); // You have been knocked off of your mount!
        }
        hk::set_mount_block(w, defender, kind, delay, true);
    } else {
        hk::base_mount_dismount(w, defender);
    }

    const DISMOUNT_RECOVERY: i32 = 1;
    let remount = TimeSpan::from_secs(10);
    if is_pm(w, attacker) {
        hk::set_mount_block(w, attacker, DISMOUNT_RECOVERY, remount, true);
    } else if core::ml(w)
        && crate::creature::data::cd(w, attacker)
            .and_then(|c| c.master)
            .is_some_and(|master| is_pm(w, master))
        && let Some(master) = crate::creature::data::cd(w, attacker).and_then(|c| c.master)
    {
        hk::set_mount_block(w, master, DISMOUNT_RECOVERY, remount, false);
    }
}

fn double_shot_use(w: &mut World, attacker: MobileId, defender: MobileId) {
    let a = Ability::DoubleShot;
    if !validate(w, a, attacker) || !check_mana(w, a, attacker, true) {
        return;
    }
    let Some(wp) = weapon_of(w, attacker) else {
        return; // sanity
    };
    clear_current_ability(w, attacker);
    msg(w, attacker, 1063348); // You launch two shots at once!
    msg(w, defender, 1063349); // You're attacked with a barrage of shots!
    fx::fixed_particles(w, defender, 0x37B9, 1, 19, 0x251D, EffectLayer::Waist);
    wc::swing(w, wp, attacker, defender, 1.0);
}

fn double_strike_on_hit(w: &mut World, a: Ability, attacker: MobileId, defender: MobileId) {
    if !begin(w, a, attacker) {
        return;
    }
    msg(w, attacker, 1060084); // You attack with lightning speed!
    msg(w, defender, 1060085); // Your attacker strikes with lightning speed!
    fx::play_sound(w, defender, 0x3BB);
    fx::fixed_effect(w, defender, 0x37B9, 244, 25);

    // Swing again: if no combatant, wrong map, one of us is a ghost, or cannot see, or deleted, then stop combat
    let same_map = w.mobile(defender).map(|m| m.map) == w.mobile(attacker).map(|m| m.map);
    if deleted(w, defender)
        || deleted(w, attacker)
        || !same_map
        || !alive(w, defender)
        || !alive(w, attacker)
        || !mb::can_see_mobile(w, attacker, defender)
    {
        mb::set_combatant(w, attacker, None);
        return;
    }

    let Some(wp) = weapon_of(w, attacker) else {
        return;
    };
    let range = weapon::max_range(w, wp);
    if !(mb::in_range_point3d(w, attacker, loc(w, defender), range)
        && in_los(w, attacker, defender))
    {
        return;
    }

    w.resources.get_mut::<wc::DoubleStrikeFlag>().0 = true;
    mb::revealing_action(w, attacker);
    let ms = wc::swing(w, wp, attacker, defender, 1.0).total_milliseconds() as i64;
    let tick = w.tick_count;
    if let Some(m) = w.mobile_mut(attacker) {
        m.next_combat_time = tick + i64::from(ms as i32);
    }
    w.resources.get_mut::<wc::DoubleStrikeFlag>().0 = false;
}

// ---------------------------------------------------------------------------------------------------------------
// ForceArrow / ForceOfNature
// ---------------------------------------------------------------------------------------------------------------

fn force_arrow_on_hit(w: &mut World, a: Ability, attacker: MobileId, defender: MobileId) {
    if !begin(w, a, attacker) {
        return;
    }
    msg(w, attacker, 1074381); // You fire an arrow of pure force.
    msg(w, defender, 1074382); // You are struck by a force arrow!

    if 0.4 > utility::random_double(&mut w.rng) {
        mb::set_combatant(w, defender, None);
        props_b::set_warmode(w, defender, false);
    }

    let existing = st_ref(w)
        .and_then(|s| s.force_arrow.get(&attacker))
        .and_then(|l| l.iter().position(|i| i.defender == defender));
    match existing {
        None => begin_force_arrow(w, attacker, defender),
        Some(i) => {
            // info.Timer.Running: the timer is live; IncreaseExpiration()
            let running = st_ref(w)
                .and_then(|s| s.force_arrow.get(&attacker))
                .map(|l| l[i].timer.running(w))
                == Some(true);
            if running && let Some(info) = st(w).force_arrow.get_mut(&attacker).map(|l| &mut l[i]) {
                info.expires += TimeSpan::from_secs(2);
                info.defense_chance_malus += 5;
                let malus = info.defense_chance_malus;
                add_force_arrow_buff(w, defender, malus);
            }
        }
    }

    // defender.Spell is Spell { IsCasting: true } -> spell.Disturb(DisturbType.Hurt, false, true)
    if let Some(sp) = w.mobile(defender).and_then(|m| m.spell)
        && crate::spells::spell::is_casting(w, sp)
    {
        crate::spells::spell::disturb(w, sp, crate::spells::types::DisturbType::Hurt, false, true);
    }
}

/// `ForceArrow.BeginForceArrow(attacker, defender)`.
pub fn begin_force_arrow(w: &mut World, attacker: MobileId, defender: MobileId) {
    let expires = w.now + TimeSpan::from_secs(10);
    let h = timer::start(
        w,
        TimeSpan::from_secs(1),
        TimeSpan::from_secs(1),
        0,
        move |w| {
            let now = w.now;
            let expired = st_ref(w)
                .and_then(|s| s.force_arrow.get(&attacker))
                .and_then(|l| l.iter().find(|i| i.defender == defender))
                .is_some_and(|i| i.expires < now);
            if expired {
                end_force_arrow(w, attacker, defender);
            }
        },
    );
    st(w)
        .force_arrow
        .entry(attacker)
        .or_default()
        .push(ForceArrowInfo {
            defender,
            defense_chance_malus: 10,
            expires,
            timer: h,
        });
    add_force_arrow_buff(w, defender, 10);
}

/// `(defender as PlayerMobile)?.AddBuff(new BuffInfo(BuffIcon.ForceArrow, 1151285, 1151286, args: malus))`.
fn add_force_arrow_buff(w: &mut World, defender: MobileId, malus: i32) {
    use crate::player_mobile::buff::{BuffIcon, BuffInfo, add_buff};
    add_buff(
        w,
        defender,
        BuffInfo::with_secondary(BuffIcon::ForceArrow, 1151285, 1151286)
            .args(muo_io::text::TextDefinition::of_text(malus.to_string())),
    );
}

/// `ForceArrow.EndForceArrow(info)`.
pub fn end_force_arrow(w: &mut World, attacker: MobileId, defender: MobileId) {
    let mut removed = None;
    if let Some(list) = st(w).force_arrow.get_mut(&attacker) {
        if let Some(i) = list.iter().position(|i| i.defender == defender) {
            removed = Some(list.remove(i));
        }
        if list.is_empty() {
            st(w).force_arrow.shift_remove(&attacker);
        }
    }
    if let Some(mut info) = removed {
        info.timer.cancel(w);
        hk::remove_buff(
            w,
            defender,
            i32::from(crate::player_mobile::buff::BuffIcon::ForceArrow.0),
        ); // BuffIcon.ForceArrow = 0x434
    }
}

/// `ForceArrow.GetInfo(attacker, defender)?.DefenseChanceMalus` for the info whose defender is `defender`.
pub fn force_arrow_defense_malus(w: &World, attacker: MobileId, defender: MobileId) -> Option<i32> {
    st_ref(w)?
        .force_arrow
        .get(&attacker)?
        .iter()
        .find(|i| i.defender == defender)
        .map(|i| i.defense_chance_malus)
}

/// `ForceOfNature.Remove(m)`.
pub fn force_of_nature_remove(w: &mut World, m: MobileId) {
    if let Some(mut info) = st(w).force_of_nature.shift_remove(&m) {
        info.timer.cancel(w);
    }
}

fn force_of_nature_tick(w: &mut World, from: MobileId) {
    let Some((target, last_hit)) = st_ref(w)
        .and_then(|s| s.force_of_nature.get(&from))
        .map(|i| (i.target, i.last_hit))
    else {
        return;
    };
    let far = {
        let (a, b) = (loc(w, from), loc(w, target));
        let (dx, dy) = (f64::from(a.x - b.x), f64::from(a.y - b.y));
        (dx * dx + dy * dy).sqrt() > 10.0
    };
    let same_map = w.mobile(target).map(|m| m.map) == w.mobile(from).map(|m| m.map);
    if !core::sa(w)
        || !alive(w, from)
        || !alive(w, target)
        || !same_map
        || far
        || last_hit + TimeSpan::from_secs(20) < w.now
    {
        force_of_nature_remove(w, from);
    }
}

/// `ForceOfNature.OnHit(from, target)`.
pub fn force_of_nature_on_hit(w: &mut World, from: MobileId, target: MobileId) {
    let Some(info) = st_ref(w).and_then(|s| s.force_of_nature.get(&from)) else {
        return;
    };
    if info.target != target {
        return;
    }
    if core::sa(w) {
        let now = w.now;
        let mut stun = false;
        if let Some(i) = st(w).force_of_nature.get_mut(&from) {
            i.hits += 1;
            i.last_hit = now;
            if i.hits % 12 == 0 {
                stun = true;
                i.hits = 0;
            }
        }
        if stun {
            let duration = if val(w, target, SkillName::MagicResist) >= 90.0 {
                1
            } else {
                2
            };
            mb::paralyze(w, target, TimeSpan::from_secs(duration));
            fx::fixed_effect(w, target, 0x376A, 9, 32);
            fx::play_sound(w, target, 0x204);
            msg(w, from, 1004013); // You successfully stun your opponent!
            msg(w, target, 1004014); // You have been stunned!
        }
    } else {
        // TODO: What is the effect?
        let d = utility::random_from(&mut w.rng, 25, 10);
        aos_damage(w, from, Some(from), d, false, [0, 0, 0, 100, 0], 0, 0);
    }
}

/// `ForceOfNature.GetDamageScalar(from, target)`.
pub fn force_of_nature_damage_scalar(w: &mut World, from: MobileId, target: MobileId) -> f64 {
    let Some(info) = st_ref(w).and_then(|s| s.force_of_nature.get(&from)) else {
        return 1.0;
    };
    if info.target != target {
        return 1.0;
    }
    if core::sa(w) {
        let bonus = 100.min(50.max(mb::str(w, from) - 50));
        return f64::from(100 + bonus) / 100.0;
    }
    1.65
}

// ---------------------------------------------------------------------------------------------------------------
// Frenzied Whirlwind / Talon Strike (damage over time)
// ---------------------------------------------------------------------------------------------------------------

#[derive(Copy, Clone, PartialEq, Eq)]
enum DotKind {
    Frenzied,
    Talon,
}

fn dot_timer(
    w: &mut World,
    attacker: MobileId,
    defender: MobileId,
    total_damage: i32,
    kind: DotKind,
) {
    let damage_per_tick = f64::from(total_damage) / 12.0 + 0.01;
    let mut remaining = f64::from(total_damage);
    let mut to_do = 0.0_f64;
    let h = timer::start(
        w,
        TimeSpan::ZERO,
        TimeSpan::from_millis(250),
        12,
        move |w| {
            let finish = |w: &mut World| match kind {
                DotKind::Frenzied => {
                    if let Some(mut h) = st(w).frenzied.shift_remove(&defender) {
                        h.cancel(w);
                    }
                }
                DotKind::Talon => {
                    st(w).talon_strike.shift_remove(&defender);
                }
            };
            if !alive(w, defender) || remaining <= 0.0 {
                finish(w);
                return;
            }

            remaining -= damage_per_tick;
            to_do += damage_per_tick;

            if remaining <= 0.0 && to_do < 1.0 {
                to_do = 1.0; // Confirm this 'round up' at the end
            }

            let damage = to_do as i32;
            if damage > 0 {
                match kind {
                    DotKind::Frenzied => {
                        mb::damage(w, defender, damage, Some(attacker), true, false)
                    }
                    DotKind::Talon => {
                        // Don't show damage, don't disrupt
                        let h = w.mobile(defender).map_or(0, |m| m.hits);
                        props_b::set_hits(w, defender, h - damage);
                    }
                }
                to_do -= f64::from(damage);
            }

            if !alive(w, defender) || remaining <= 0.0 {
                finish(w);
            }
        },
    );
    if kind == DotKind::Frenzied {
        st(w).frenzied.insert(defender, h);
    }
}

fn nearby_victims(
    w: &mut World,
    attacker: MobileId,
    defender: MobileId,
    range: i32,
    weapon_range: i32,
) -> Vec<MobileId> {
    let mut v = Vec::new();
    let amap = w.mobile(attacker).map(|m| m.map);
    for m in muo_server::mobile::part2::enumerators::get_mobiles_in_range(w, attacker, range) {
        if !deleted(w, m)
            && m != defender
            && m != attacker
            && hk::spell_helper_valid_indirect_target(w, attacker, m)
            && w.mobile(m).map(|x| x.map) == amap
            && alive(w, m)
            && mb::can_see_mobile(w, attacker, m)
            && mb::can_be_harmful(w, attacker, m)
            && mb::in_range_point3d(w, attacker, loc(w, m), weapon_range)
            && in_los(w, attacker, m)
        {
            v.push(m);
        }
    }
    v
}

fn frenzied_on_hit(w: &mut World, a: Ability, attacker: MobileId, defender: MobileId) {
    if !begin(w, a, attacker) {
        return;
    }
    if w.mobile(attacker).and_then(|m| m.map).is_none() {
        return;
    }
    let Some(wp) = weapon_of(w, attacker) else {
        return;
    };
    let range = weapon::max_range(w, wp);

    // 5-15 damage
    let m = val(w, attacker, SkillName::Bushido).max(val(w, attacker, SkillName::Ninjitsu));
    let amount = (10.0 * ((m - 50.0) / 70.0 + 5.0)) as i32;

    let mut did_effect = false;
    for v in nearby_victims(w, attacker, defender, 1, range) {
        mb::do_harmful(w, attacker, v, true);
        if let Some(mut h) = st(w).frenzied.shift_remove(&v) {
            h.cancel(w);
        }
        dot_timer(w, attacker, v, amount, DotKind::Frenzied);
        did_effect = true;
    }

    if did_effect {
        fx::fixed_effect(w, attacker, 0x3728, 10, 15);
        fx::play_sound(w, attacker, 0x2A1);
    }

    timer::start(w, TimeSpan::from_secs(2), TimeSpan::ZERO, 1, move |w| {
        fx::fixed_effect(w, attacker, 0x3728, 10, 15);
        fx::play_sound(w, attacker, 0x2A1);
    });
}

// ---------------------------------------------------------------------------------------------------------------
// InfectiousStrike / LightningArrow / MortalStrike / MysticArc / NerveStrike / ParalyzingBlow / PsychicAttack
// ---------------------------------------------------------------------------------------------------------------

fn infectious_on_hit(w: &mut World, a: Ability, attacker: MobileId, defender: MobileId) {
    if !begin(w, a, attacker) {
        return;
    }
    let Some(wp) = weapon_of(w, attacker) else {
        return;
    };
    let (poison, charges) = weapon::data(w, wp).map_or((None, 0), |d| (d.poison, d.poison_charges));
    let Some(mut p) = poison.filter(|_| charges > 0) else {
        // Your weapon must have a dose of poison to perform an infectious strike!
        msg(w, attacker, 1061141);
        return;
    };
    weapon::set_poison_charges(w, wp, charges - 1);

    let poisoning_skill = val(w, attacker, SkillName::Poisoning);

    // Infectious strike special move now uses poisoning skill to help determine potency
    let family = muo_server::poison::find(w, p)
        .map_or(muo_server::poison::PoisonFamily::Standard, |x| x.family);
    let max_level = match family {
        muo_server::poison::PoisonFamily::Darkglow => ((poisoning_skill / 33.3) as i32).min(3),
        muo_server::poison::PoisonFamily::Parasitic => ((poisoning_skill / 25.0) as i32).min(4),
        _ => ((poisoning_skill / 20.0) as i32).max(0),
    };

    if muo_server::poison::level(w, p) > max_level
        && let Some(np) =
            crate::misc::poison_kinds::get_poison_by_family_and_level(w, family, max_level)
    {
        p = np;
    }

    // Check if the poison can be increased.
    let new_poison = w
        .resources
        .get::<muo_server::poison::Poisons>()
        .and_then(|ps| ps.increase_level(Some(p)));
    if let Some(np) = new_poison
        && np != p
        && poisoning_skill / 100.0 > utility::random_double(&mut w.rng)
    {
        p = np;
        msg(w, attacker, 1060080); // Your precise strike has increased the level of the poison by 1
        msg(w, defender, 1060081); // The poison seems extra effective!
    }

    fx::play_sound(w, defender, 0xDD);
    fx::fixed_particles_hue(
        w,
        defender,
        0x3728,
        244,
        25,
        9941,
        1266,
        0,
        EffectLayer::Waist,
    );

    if mb::apply_poison(w, defender, Some(attacker), p)
        != muo_server::mobile::ApplyPoisonResult::Immune
    {
        let dn = props_b::name(w, defender).unwrap_or_default();
        let an = props_b::name(w, attacker).unwrap_or_default();
        msgs::send_localized_message_affix(w, attacker, 1008096, true, &dn, "", 0x3B2); // You have poisoned your target :
        msgs::send_localized_message_affix(w, defender, 1008097, false, &an, "", 0x3B2); //  : poisoned you!
    }
}

fn lightning_arrow_on_hit(
    w: &mut World,
    a: Ability,
    attacker: MobileId,
    defender: MobileId,
    at: WorldLocation,
) {
    if !validate(w, a, attacker) {
        return;
    }
    clear_current_ability(w, attacker);

    if w.mobile(attacker).and_then(|m| m.map).is_none() {
        return;
    }
    let Some(wp) = weapon_of(w, attacker) else {
        return;
    };
    if !check_mana(w, a, attacker, true) {
        return;
    }
    let range = weapon::max_range(w, wp);

    let mut list = Vec::new();
    if let Some(map) = at.1 {
        let ms: Vec<MobileId> =
            muo_server::map::enumerate::mobiles_in_range(w, Some(map), at.0.x, at.0.y, 5).collect();
        for m in ms {
            if m != defender
                && m != attacker
                && hk::spell_helper_valid_indirect_target(w, attacker, m)
                && !deleted(w, m)
                && alive(w, m)
                && mb::can_see_mobile(w, attacker, m)
                && mb::can_be_harmful(w, attacker, m)
                && mb::in_range_point3d(w, attacker, loc(w, m), range)
                && in_los(w, attacker, m)
            {
                list.push(m);
            }
        }
    }

    // Defender might be already dead/internalized
    if !deleted(w, defender) && alive(w, defender) {
        fx::bolt_effect(w, defender, 0);
        let d = super::random_min_max(w, 29, 40);
        aos_damage(
            w,
            defender,
            Some(attacker),
            d,
            false,
            [0, 0, 0, 0, 100],
            0,
            0,
        );
    } else {
        muo_server::effects::send_bolt_effect(w, Targeted::point(at.0, at.1), true, 0);
    }

    let count = list.len().min(2);
    if count > 0 {
        // list.Shuffle()
        muo_primitives::random::Rng::shuffle(&mut w.rng, &mut list[..]);
        for &m in list.iter().take(count) {
            fx::bolt_effect(w, m, 0);
            let d = super::random_min_max(w, 29, 40);
            aos_damage(w, m, Some(attacker), d, false, [0, 0, 0, 0, 100], 0, 0);
        }
    }
}

/// `MortalStrike.IsWounded(m)`.
pub fn is_wounded(w: &World, m: MobileId) -> bool {
    st_ref(w).is_some_and(|s| s.mortal_strike.contains_key(&m))
}

/// `MortalStrike.BeginWound(m, duration)`.
pub fn begin_wound(w: &mut World, m: MobileId, duration: TimeSpan) {
    if let Some(mut h) = st(w).mortal_strike.shift_remove(&m) {
        h.cancel(w);
    }
    let h = timer::start(w, duration, TimeSpan::ZERO, 1, move |w| {
        end_wound(w, m);
    });
    st(w).mortal_strike.insert(m, h);
    props_b::set_yellow_healthbar(w, m, true);
}

/// `MortalStrike.EndWound(m)`.
pub fn end_wound(w: &mut World, m: MobileId) -> bool {
    if let Some(mut h) = st(w).mortal_strike.shift_remove(&m) {
        h.cancel(w);
        props_b::set_yellow_healthbar(w, m, false);
        msg(w, m, 1060208); // You are no longer mortally wounded.
        true
    } else {
        false
    }
}

fn mystic_arc_on_hit(w: &mut World, a: Ability, attacker: MobileId, defender: MobileId) {
    // `!CheckMana(attacker, true) && defender != null`: defender is never null here.
    if !check_mana(w, a, attacker, true) {
        return;
    }
    let Some(wp) = weapon_of(w, attacker) else {
        return;
    };
    if !w.item(wp).is_some_and(|i| i.ty.is(&weapon::BASE_THROWN)) {
        return;
    }
    let range = weapon::max_range(w, wp);

    let mut queue = Vec::new();
    for m in muo_server::mobile::part2::enumerators::get_mobiles_in_range(w, attacker, range) {
        if m == defender || mb::combatant(w, m) != Some(attacker) {
            continue;
        }
        queue.push(m);
    }

    let target = if queue.is_empty() {
        None
    } else {
        let i = utility::random(&mut w.rng, queue.len() as i32) as usize;
        Some(queue[i])
    };

    aos_damage(w, defender, Some(attacker), 15, false, [0; 5], 100, 0);

    if let Some(t) = target {
        let graphic = w.item(wp).map_or(0, |i| i.item_id);
        fx::moving_effect_plain(
            w,
            defender,
            EntityId::Mobile(t),
            graphic,
            18,
            1,
            false,
            false,
        );
        timer::start(w, TimeSpan::from_millis(300), TimeSpan::ZERO, 1, move |w| {
            mystic_arc_throw_again(w, attacker, t);
        });
    }
    clear_current_ability(w, attacker);
}

fn mystic_arc_throw_again(w: &mut World, attacker: MobileId, target: MobileId) {
    let Some(wp) = weapon_of(w, attacker) else {
        return;
    };
    if !w.item(wp).is_some_and(|i| i.ty.is(&weapon::BASE_THROWN)) {
        return;
    }
    if current_ability(w, attacker) == Some(Ability::MysticArc) {
        clear_current_ability(w, attacker);
    }
    if wc::check_hit(w, wp, attacker, target) {
        wc::on_hit(w, wp, attacker, target, 0.0);
        aos_damage(
            w,
            target,
            Some(attacker),
            15,
            false,
            [0, 0, 0, 0, 100],
            0,
            0,
        );
    }
}

/// `ParalyzingBlow.FreezeDelayDuration`.
const FREEZE_DELAY: TimeSpan = TimeSpan::from_secs(8);
const PLAYER_FREEZE: TimeSpan = TimeSpan::from_secs(3);
const NPC_FREEZE: TimeSpan = TimeSpan::from_secs(6);

/// `ParalyzingBlow.IsImmune(m)`.
pub fn is_paralysis_immune(w: &World, m: MobileId) -> bool {
    st_ref(w).is_some_and(|s| s.paralyzing_immunity.contains_key(&m))
}

/// `ParalyzingBlow.BeginImmunity(m, duration)`.
pub fn begin_immunity(w: &mut World, m: MobileId, duration: TimeSpan) {
    end_immunity(w, m);
    let h = timer::start(w, duration, TimeSpan::ZERO, 1, move |w| end_immunity(w, m));
    st(w).paralyzing_immunity.insert(m, h);
}

/// `ParalyzingBlow.EndImmunity(m)`.
pub fn end_immunity(w: &mut World, m: MobileId) {
    if let Some(mut h) = st(w).paralyzing_immunity.shift_remove(&m) {
        h.cancel(w);
    }
}

fn nerve_strike_on_hit(w: &mut World, a: Ability, attacker: MobileId, defender: MobileId) {
    if !begin(w, a, attacker) {
        return;
    }
    let cantpara = is_paralysis_immune(w, defender);
    if cantpara {
        msg(w, attacker, 1070804); // Your target resists paralysis.
        msg(w, defender, 1070813); // You resist paralysis.
    } else {
        msg(w, attacker, 1063356); // You cripple your target with a nerve strike!
        msg(w, defender, 1063357); // Your attacker dealt a crippling nerve strike!
    }

    fx::play_sound(w, attacker, 0x204);
    fx::fixed_effect(w, defender, 0x376A, 9, 32);
    fx::fixed_particles_hue(w, defender, 0x37C4, 1, 8, 0x13AF, 0, 0, EffectLayer::Waist);

    let bushido = val(w, attacker, SkillName::Bushido);
    if core::ml(w) {
        let r = utility::random(&mut w.rng, 10);
        aos_damage(
            w,
            defender,
            Some(attacker),
            (15.0 * (bushido - 50.0) / 70.0 + f64::from(r)) as i32,
            true,
            [100, 0, 0, 0, 0],
            0,
            0,
        ); // 0-25
        if !cantpara
            && (150.0 / 7.0 + 4.0 * bushido / 7.0) / 100.0 > utility::random_double(&mut w.rng)
        {
            mb::paralyze(w, defender, TimeSpan::from_secs(2));
            begin_immunity(w, defender, FREEZE_DELAY);
        }
    } else if !cantpara {
        aos_damage(
            w,
            defender,
            Some(attacker),
            (15.0 * (bushido - 50.0) / 70.0 + 10.0) as i32,
            true,
            [100, 0, 0, 0, 0],
            0,
            0,
        ); // 10-25
        mb::freeze(w, defender, TimeSpan::from_secs(2));
        begin_immunity(w, defender, FREEZE_DELAY);
    }
}

fn psychic_on_hit(w: &mut World, a: Ability, attacker: MobileId, defender: MobileId) {
    if !begin(w, a, attacker) {
        return;
    }
    msg(w, attacker, 1074383); // Your shot sends forth a wave of psychic energy.
    msg(w, defender, 1074384); // Your mind is attacked by psychic force!
    fx::fixed_particles(w, defender, 0x3789, 10, 25, 5032, EffectLayer::Head);
    fx::play_sound(w, defender, 0x1F8);

    if let Some(info) = st(w).psychic.get_mut(&defender) {
        if !info.done_increase {
            info.spell_damage_malus *= 2;
            info.mana_cost_malus *= 2;
            info.done_increase = true;
        }
    } else {
        let h = timer::start(w, TimeSpan::from_secs(10), TimeSpan::ZERO, 1, move |w| {
            remove_psychic_effects(w, defender);
        });
        st(w).psychic.insert(
            defender,
            PsychicInfo {
                spell_damage_malus: 15,
                mana_cost_malus: 15,
                done_increase: false,
                timer: h,
            },
        );
    }

    let (dmg, mana) = st(w)
        .psychic
        .get(&defender)
        .map_or((15, 15), |i| (i.spell_damage_malus, i.mana_cost_malus));
    {
        use crate::player_mobile::buff::{BuffIcon, BuffInfo, add_buff};
        add_buff(
            w,
            defender,
            BuffInfo::with_secondary(BuffIcon::PsychicAttack, 1151296, 1151297).args(
                muo_io::text::TextDefinition::of_text(format!("{dmg}	{mana}")),
            ),
        );
    }
}

fn remove_psychic_effects(w: &mut World, defender: MobileId) {
    hk::remove_buff(
        w,
        defender,
        i32::from(crate::player_mobile::buff::BuffIcon::PsychicAttack.0),
    ); // 0x439
    if let Some(mut info) = st(w).psychic.shift_remove(&defender) {
        info.timer.cancel(w);
    }
    msg(w, defender, 1150292); // You recover from the effects of the psychic attack.
}

/// `PsychicAttack.Registry[m].SpellDamageMalus`.
pub fn psychic_attack_spell_damage_malus(w: &World, m: MobileId) -> Option<i32> {
    st_ref(w)?.psychic.get(&m).map(|i| i.spell_damage_malus)
}

/// `PsychicAttack.Registry[m].ManaCostMalus`.
pub fn psychic_attack_mana_cost_malus(w: &World, m: MobileId) -> Option<i32> {
    st_ref(w)?.psychic.get(&m).map(|i| i.mana_cost_malus)
}

fn riding_swipe_on_hit(w: &mut World, a: Ability, attacker: MobileId, defender: MobileId) {
    if !props_b::mounted(w, defender) {
        msg(w, attacker, 1060848); // This attack only works on mounted targets
        clear_current_ability(w, attacker);
        return;
    }

    if !begin(w, a, attacker) {
        return;
    }

    let bushido = val(w, attacker, SkillName::Bushido);
    if !props_b::mounted(w, attacker) {
        let mount = props_b::mount(w, defender);
        hk::base_mount_dismount(w, defender);
        if let Some(mount) = mount {
            // Ethy mounts don't take damage
            let amount = 10 + (10.0 * (bushido - 50.0) / 70.0 + 5.0) as i32;
            // The mount just takes damage, there's no flagging as if it was attacking the mount directly
            aos_damage(w, mount, None, amount, false, [100, 0, 0, 0, 0], 0, 0);
        }
    } else {
        let amount = 10 + (10.0 * (bushido - 50.0) / 70.0 + 5.0) as i32;
        aos_damage(
            w,
            defender,
            Some(attacker),
            amount,
            false,
            [100, 0, 0, 0, 0],
            0,
            0,
        );
        if is_paralysis_immune(w, defender) {
            // Does it still do damage?
            msg(w, attacker, 1070804); // Your target resists paralysis.
            msg(w, defender, 1070813); // You resist paralysis.
        } else {
            mb::paralyze(w, defender, TimeSpan::from_secs(3));
            begin_immunity(w, defender, FREEZE_DELAY);
        }
    }
}

fn whirlwind_on_hit(w: &mut World, a: Ability, attacker: MobileId, defender: MobileId) {
    if !begin(w, a, attacker) {
        return;
    }
    if w.mobile(attacker).and_then(|m| m.map).is_none() {
        return;
    }
    let Some(wp) = weapon_of(w, attacker) else {
        return;
    };

    fx::fixed_effect(w, attacker, 0x3728, 10, 15);
    fx::play_sound(w, attacker, 0x2A1);

    let range = weapon::max_range(w, wp);
    let queue = nearby_victims(w, attacker, defender, 1, range);

    if queue.is_empty() {
        return;
    }

    let bushido = val(w, attacker, SkillName::Bushido);
    let mut damage_bonus = 1.0 + (queue.len() as f64 * bushido / 60.0).powi(2) / 100.0;

    if damage_bonus > 2.0 {
        damage_bonus = 2.0;
    }

    mb::revealing_action(w, attacker);

    for m in queue {
        msg(w, attacker, 1060161); // The whirling attack strikes a target!
        msg(w, m, 1060162); // You are struck by the whirling attack and take damage!
        wc::on_hit(w, wp, attacker, m, damage_bonus);
    }
}

// ---------------------------------------------------------------------------------------------------------------
// HitLower
// ---------------------------------------------------------------------------------------------------------------

/// `HitLower.IsUnderAttackEffect(m)`.
pub fn hit_lower_is_under_attack_effect(w: &World, m: MobileId) -> bool {
    st_ref(w).is_some_and(|s| s.hit_lower_attack.contains(&m))
}

/// `HitLower.ApplyAttack(m)`.
pub fn hit_lower_apply_attack(w: &mut World, m: MobileId) -> bool {
    if hit_lower_is_under_attack_effect(w, m) {
        return false;
    }
    st(w).hit_lower_attack.insert(m);
    timer::start(w, TimeSpan::from_secs(10), TimeSpan::ZERO, 1, move |w| {
        st(w).hit_lower_attack.shift_remove(&m);
        msg(w, m, 1062320); // Your attack chance has returned to normal.
    });
    msg(w, m, 1062319); // Your attack chance has been reduced!
    true
}

/// `HitLower.IsUnderDefenseEffect(m)`.
pub fn hit_lower_is_under_defense_effect(w: &World, m: MobileId) -> bool {
    st_ref(w).is_some_and(|s| s.hit_lower_defense.contains(&m))
}

/// `HitLower.ApplyDefense(m)`.
pub fn hit_lower_apply_defense(w: &mut World, m: MobileId) -> bool {
    if hit_lower_is_under_defense_effect(w, m) {
        return false;
    }
    st(w).hit_lower_defense.insert(m);
    timer::start(w, TimeSpan::from_secs(8), TimeSpan::ZERO, 1, move |w| {
        st(w).hit_lower_defense.shift_remove(&m);
        msg(w, m, 1062321); // Your defense chance has returned to normal.
    });
    msg(w, m, 1062318); // Your defense chance has been reduced!
    true
}
