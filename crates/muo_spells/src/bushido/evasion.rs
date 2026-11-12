//! The Evasion spell.

use indexmap::IndexMap;
use muo_content::core;
use muo_content::hooks::hooks_mut as content_hooks_mut;
use muo_content::items::armor::BASE_SHIELD;
use muo_content::items::weapon::{self, BASE_WEAPON};
use muo_content::items::weapon_combat::check_parry;
use muo_content::items::weapon_hooks::WeaponHooks;
use muo_content::spells::SpellInfo;
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_primitives::TimeSpan;
use muo_server::World;
use muo_server::effects::EffectLayer;
use muo_server::ids::{ItemId, MobileId};
use muo_server::layer::Layer;
use muo_server::mobile::part1::combat;
use muo_server::mobile::part2::{effects, items, messages as mm};
use muo_server::skills::{self, SkillName};
use muo_server::timer::{self, TimerHandle};

use super::samurai::{SAMURAI_SPELL, SamuraiType, on_cast_successful, on_effect_end};

static EVASION: SamuraiType = SamuraiType {
    required_skill: 60.0,
    required_mana: 10,
};

/// `Evasion`: Evasion.
pub static EVASION_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Bushido.Evasion",
    parent: Some(&SAMURAI_SPELL),
    info: SpellInfo::new("Evasion", "", &[])
        .action(-1)
        .hand_effect(9002),
    cast_delay_base: |_, _| TimeSpan::from_millis(250),
    on_cast,
    on_begin_cast,
    check_cast,
    family: Some(&EVASION),
    ..SAMURAI_SPELL
};

const KEY: &str = "Server.Spells.Bushido.Evasion";

#[derive(Default)]
struct Table(IndexMap<MobileId, TimerHandle>);

fn table(w: &mut World) -> &mut IndexMap<MobileId, TimerHandle> {
    &mut w.resources.get_mut::<Table>().0
}

/// Sets the weapon hooks and `Hooks::check_spell_evasion`.
pub fn init(w: &mut World) {
    let h = w.resources.get_mut::<WeaponHooks>();
    h.evasion_is_evading = Some(|w, m| {
        w.resources
            .get::<Table>()
            .is_some_and(|t| t.0.contains_key(&m))
    });
    h.evasion_parry_scalar = Some(get_parry_scalar);
    content_hooks_mut(w).check_spell_evasion = Some(check_spell_evasion);
}

fn is_base_weapon(w: &World, i: ItemId) -> bool {
    w.item(i).is_some_and(|x| x.ty.is(&BASE_WEAPON))
}

fn is_base_shield(w: &World, i: ItemId) -> bool {
    w.item(i).is_some_and(|x| x.ty.is(&BASE_SHIELD))
}

/// `caster.FindItemOnLayer<BaseWeapon>(OneHanded) ?? FindItemOnLayer<BaseWeapon>(TwoHanded)`.
fn equipped_weapon(w: &World, m: MobileId) -> Option<ItemId> {
    let one = items::find_item_on_layer(w, m, Layer::OneHanded).filter(|&i| is_base_weapon(w, i));
    one.or_else(|| {
        items::find_item_on_layer(w, m, Layer::TwoHanded).filter(|&i| is_base_weapon(w, i))
    })
}

fn check_cast(w: &mut World, sp: SpellId) -> bool {
    let Some(caster) = spell::caster(w, sp) else {
        return false;
    };
    verify_cast(w, caster, true) && (SAMURAI_SPELL.check_cast)(w, sp)
}

/// `VerifyCast(caster, messages)`.
pub fn verify_cast(w: &mut World, caster: MobileId, messages: bool) -> bool {
    let weap = equipped_weapon(w, caster);

    match weap {
        None => {
            let shield = items::find_item_on_layer(w, caster, Layer::TwoHanded)
                .is_some_and(|i| is_base_shield(w, i));
            if !shield {
                if messages {
                    // You must have a weapon or a shield equipped to use this ability!
                    mm::send_localized_message(w, caster, 1062944, "", 0x3B2);
                }

                return false;
            }
        }
        Some(wp) => {
            if core::ml(w) && skills::base(w, caster, weapon::skill(w, wp)) < 50.0 {
                if messages {
                    // Your skill with your equipped weapon must be 50 or higher to use Evasion.
                    mm::send_localized_message(w, caster, 1076206, "", 0x3B2);
                }

                return false;
            }
        }
    }

    if !combat::can_begin_action(w, caster, KEY) {
        if messages {
            mm::send_localized_message(w, caster, 501789, "", 0x3B2); // You must wait before trying again.
        }

        return false;
    }

    true
}

/// `CheckSpellEvasion(defender)`.
pub fn check_spell_evasion(w: &mut World, defender: MobileId) -> bool {
    let weap = equipped_weapon(w, defender);

    if core::ml(w) {
        if let Some(s) = w.mobile(defender).and_then(|d| d.spell)
            && spell::is_casting(w, s)
        {
            return false;
        }

        if let Some(wp) = weap {
            if skills::base(w, defender, weapon::skill(w, wp)) < 50.0 {
                return false;
            }
        } else if !items::find_item_on_layer(w, defender, Layer::TwoHanded)
            .is_some_and(|i| is_base_shield(w, i))
        {
            return false;
        }
    }

    if is_evading(w, defender) && check_parry(w, defender) {
        mm::emote(w, defender, "*evades*"); // Yes.  Eew.  Blame OSI.
        effects::fixed_effect(w, defender, 0x37B9, 10, 16);
        return true;
    }

    false
}

fn on_begin_cast(w: &mut World, sp: SpellId) {
    (SAMURAI_SPELL.on_begin_cast)(w, sp);
    if let Some(caster) = spell::caster(w, sp) {
        effects::fixed_effect_hue(w, caster, 0x37C4, 10, 7, 4, 3);
    }
}

fn on_cast(w: &mut World, sp: SpellId) {
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };
    if spell::check_sequence(w, sp) {
        mm::send_localized_message(w, caster, 1063120, "", 0x3B2); // You feel that you might be able to deflect any attack!

        effects::fixed_particles_hue(
            w,
            caster,
            0x376A,
            1,
            20,
            0x7F5,
            0x960,
            3,
            EffectLayer::Waist,
        );
        effects::play_sound(w, caster, 0x51B);

        on_cast_successful(w, sp);

        begin_evasion(w, caster);

        combat::begin_action(w, caster, KEY);
        timer::start(w, TimeSpan::from_secs(20), TimeSpan::ZERO, 1, move |w| {
            combat::end_action(w, caster, KEY)
        });
    }

    spell::finish_sequence(w, sp);
}

/// `IsEvading(m)`.
pub fn is_evading(w: &mut World, m: MobileId) -> bool {
    table(w).contains_key(&m)
}

/// `GetEvadeDuration(m)`.
pub fn get_evade_duration(w: &mut World, m: MobileId) -> TimeSpan {
    /* Evasion duration now scales with Bushido skill
     *
     * If the player has higher than GM Bushido, and GM Tactics and Anatomy, they get a 1 second bonus
     * Evasion duration range:
     * o 3-6 seconds w/o tactics/anatomy
     * o 6-7 seconds w/ GM+ Bushido and GM tactics/anatomy
     */
    if !core::ml(w) {
        return TimeSpan::from_secs(8);
    }

    let bushido = skills::value(w, m, SkillName::Bushido);
    let mut seconds = 3.0;

    if bushido > 60.0 {
        seconds += (bushido - 60.0) / 20.0;
    }

    // Bushido being HIGHER than 100 for bonus is intended
    if skills::value(w, m, SkillName::Anatomy) >= 100.0
        && skills::value(w, m, SkillName::Tactics) >= 100.0
        && bushido > 100.0
    {
        seconds += 1.0;
    }

    TimeSpan::from_secs(seconds as i64)
}

/// `GetParryScalar(m)`.
pub fn get_parry_scalar(w: &World, m: MobileId) -> f64 {
    /* Evasion modifier to parry now scales with Bushido skill
     *
     * If the player has higher than GM Bushido, and at least GM Tactics and Anatomy, they get a bonus to their
     * evasion modifier (10% bonus to the evasion modifier to parry NOT 10% to the final parry chance)
     *
     * Bonus modifier to parry range: (these are the ranges for the evasion modifier)
     * o 16-40% bonus w/o tactics/anatomy
     * o 42-50% bonus w/ GM+ bushido and GM tactics/anatomy
     */
    if !core::ml(w) {
        return 1.5;
    }

    let bushido = skills::value_ref(w, m, SkillName::Bushido);
    let mut bonus = 0.0;

    if bushido >= 60.0 {
        bonus += (bushido - 60.0) * 0.004 + 0.16;
    }

    // Bushido being HIGHER than 100 for bonus is intended
    if skills::value_ref(w, m, SkillName::Anatomy) >= 100.0
        && skills::value_ref(w, m, SkillName::Tactics) >= 100.0
        && bushido > 100.0
    {
        bonus += 0.10;
    }

    1.0 + bonus
}

/// `BeginEvasion(m)`.
pub fn begin_evasion(w: &mut World, m: MobileId) {
    stop_evasion_timer(w, m);

    let d = get_evade_duration(w, m);
    let h = timer::start(w, d, TimeSpan::ZERO, 1, move |w| {
        end_evasion(w, m);
        mm::send_localized_message(w, m, 1063121, "", 0x3B2); // You no longer feel that you could deflect any attack.
    });
    table(w).insert(m, h);
}

fn stop_evasion_timer(w: &mut World, m: MobileId) -> bool {
    if let Some(mut t) = table(w).shift_remove(&m) {
        t.cancel(w);
        return true;
    }
    false
}

/// `EndEvasion(m)`.
pub fn end_evasion(w: &mut World, m: MobileId) {
    if stop_evasion_timer(w, m) {
        on_effect_end(w, m, KEY);
    }
}
