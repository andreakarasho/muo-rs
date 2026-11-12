//! `NecromancerSpell` and `TransformationSpell`: the abstract bases of the Necromancy
//! spells. A class sets `family` to its [`NecroType`].

use muo_content::core;
use muo_content::misc::aos_helpers;
use muo_content::spells::SpellInfo;
use muo_content::spells::helper::{self, TransformationInfo};
use muo_content::spells::hooks::hooks;
use muo_content::spells::spell::{
    self, SPELL_BASE, SpellId, SpellType, base_check_cast, base_consume_reagents,
};
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::skills::SkillName;

/// What a Necromancy spell class provides.
pub struct NecroType {
    /// `RequiredSkill`.
    pub required_skill: f64,
    /// `RequiredMana`.
    pub required_mana: i32,
    /// For a `TransformationSpell`: the `ITransformationSpell` members, by caster (`Body` can depend on `Female`).
    pub transformation: Option<fn(&World, MobileId) -> &'static TransformationInfo>,
}

/// The default [`NecroType`] to copy with struct update.
pub const NECRO_BASE: NecroType = NecroType {
    required_skill: 0.0,
    required_mana: 0,
    transformation: None,
};

/// `NecromancerSpell`'s members of a spell.
pub fn family(w: &World, sp: SpellId) -> Option<&'static NecroType> {
    spell::spell_type(w, sp)?.family::<NecroType>()
}

fn compute_karma_award(w: &mut World, sp: SpellId) -> i32 {
    // TODO: Verify this formula being that Necro spells don't HAVE a circle.
    let Some(ty) = spell::spell_type(w, sp) else {
        return 0;
    };
    let secs = (ty.cast_delay_base)(w, sp).total_seconds();
    let mut karma = -(40 + (10.0 * (secs / ty.cast_delay_seconds_per_tick)) as i32);

    // Pub 36: "Added a new property called Increased Karma Loss which grants higher karma loss for casting
    // necromancy spells."
    if core::ml(w)
        && let Some(c) = spell::caster(w, sp)
    {
        let loss =
            aos_helpers::attribute_value(w, c, muo_content::aos::AosAttribute::IncreasedKarmaLoss);
        karma += aos_helpers::scale(karma, loss);
    }

    karma
}

fn get_cast_skills(w: &mut World, sp: SpellId) -> (f64, f64) {
    let min = family(w, sp).map_or(0.0, |f| f.required_skill);
    let max = if spell::scroll(w, sp).is_some() {
        min
    } else {
        min + 40.0
    };
    (min, max)
}

fn consume_reagents(w: &mut World, sp: SpellId) -> bool {
    if base_consume_reagents(w, sp) {
        return true;
    }
    let Some(c) = spell::caster(w, sp) else {
        return false;
    };
    hooks(w)
        .and_then(|h| h.arcane_gem_consume_charges)
        .is_some_and(|f| f(w, c, 1))
}

/// `NecromancerSpell`, abstract.
pub static NECROMANCER_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Necromancy.NecromancerSpell",
    parent: Some(&SPELL_BASE),
    info: SpellInfo::new("", "", &[]),
    cast_skill: SkillName::Necromancy,
    damage_skill: SkillName::SpiritSpeak,
    cast_delay_fast_scalar_needs_se: true,
    clear_hands_on_cast: false,
    compute_karma_award,
    get_cast_skills,
    consume_reagents,
    get_mana: |w, sp| family(w, sp).map_or(0, |f| f.required_mana),
    family: Some(&NECRO_BASE),
    ..SPELL_BASE
};

fn info_of(w: &World, sp: SpellId) -> Option<&'static TransformationInfo> {
    let caster = spell::caster(w, sp)?;
    let f = family(w, sp)?.transformation?;
    Some(f(w, caster))
}

fn transformation_check_cast(w: &mut World, sp: SpellId) -> bool {
    let Some(caster) = spell::caster(w, sp) else {
        return false;
    };
    helper::transformation_check_cast(w, caster) && base_check_cast(w, sp)
}

fn transformation_on_cast(w: &mut World, sp: SpellId) {
    if let (Some(caster), Some(info)) = (spell::caster(w, sp), info_of(w, sp)) {
        helper::transformation_on_cast(w, caster, sp, info);
    }
    spell::finish_sequence(w, sp);
}

/// `TransformationSpell`, abstract.
pub static TRANSFORMATION_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Necromancy.TransformationSpell",
    parent: Some(&NECROMANCER_SPELL),
    blocked_by_horrific_beast: false,
    check_cast: transformation_check_cast,
    on_cast: transformation_on_cast,
    ..NECROMANCER_SPELL
};
