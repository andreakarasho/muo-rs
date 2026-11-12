//! The Incognito spell.

use indexmap::IndexMap;
use muo_content::items::{armor, clothing};
use muo_content::misc::small::random_name;
use muo_content::player_mobile::buff::{BuffIcon, BuffInfo, add_buff, remove_buff};
use muo_content::spells::hooks::hooks;
use muo_content::spells::magery::{FIFTH_CIRCLE, MAGERY_SPELL};
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_content::spells::{SpellInfo, reagent};
use muo_primitives::TimeSpan;
use muo_server::World;
use muo_server::effects::EffectLayer;
use muo_server::ids::MobileId;
use muo_server::mobile::part1::{combat, props_b};
use muo_server::mobile::part2::{effects, messages as mm};
use muo_server::race::{self, Races};
use muo_server::skills::{self, SkillName};
use muo_server::timer::{self, TimerHandle};

use crate::common::is_player_mobile;

/// `IncognitoSpell`: Kal In Ex.
pub static INCOGNITO_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Fifth.IncognitoSpell",
    parent: Some(&MAGERY_SPELL),
    info: SpellInfo::new(
        "Incognito",
        "Kal In Ex",
        &[reagent::BLOODMOSS, reagent::GARLIC, reagent::NIGHTSHADE],
    )
    .action(206)
    .hand_effect(9002),
    on_cast,
    check_cast,
    family: Some(&FIFTH_CIRCLE),
    ..MAGERY_SPELL
};

/// `Server.Spells.Fifth.IncognitoSpell`, the `BeginAction<IncognitoSpell>()` key.
pub const KEY: &str = "Server.Spells.Fifth.IncognitoSpell";
/// `typeof(PolymorphSpell)`.
const POLYMORPH_KEY: &str = "Server.Spells.Seventh.PolymorphSpell";

#[derive(Default)]
struct Table(IndexMap<MobileId, TimerHandle>);

fn table(w: &mut World) -> &mut IndexMap<MobileId, TimerHandle> {
    &mut w.resources.get_mut::<Table>().0
}

/// The `PlayerDeathEvent` handler.
pub fn init(w: &mut World) {
    muo_content::player_mobile::subscribe_death(w, |w, m| {
        stop_timer(w, m);
        combat::end_action(w, m, KEY);
    });
}

fn sigil_exists_on(w: &World, m: MobileId) -> bool {
    hooks(w)
        .and_then(|h| h.sigil_exists_on)
        .is_some_and(|f| f(w, m))
}

fn check_cast(w: &mut World, sp: SpellId) -> bool {
    let Some(caster) = spell::caster(w, sp) else {
        return false;
    };
    if sigil_exists_on(w, caster) {
        mm::send_localized_message(w, caster, 1010445, "", 0x3B2); // You cannot incognito if you have a sigil
        return false;
    }

    if !combat::can_begin_action(w, caster, KEY) {
        mm::send_localized_message(w, caster, 1005559, "", 0x3B2); // This spell is already in effect.
        return false;
    }

    let body_mod = w.mobile(caster).map_or(0, |c| c.body_mod.body_id());
    if body_mod == 183 || body_mod == 184 {
        mm::send_localized_message(w, caster, 1042402, "", 0x3B2); // You cannot use incognito while wearing body paint
        return false;
    }

    true
}

fn on_cast(w: &mut World, sp: SpellId) {
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };
    if sigil_exists_on(w, caster) {
        mm::send_localized_message(w, caster, 1010445, "", 0x3B2); // You cannot incognito if you have a sigil
        return;
    }

    if !combat::can_begin_action(w, caster, KEY) {
        mm::send_localized_message(w, caster, 1005559, "", 0x3B2); // This spell is already in effect.
        return;
    }

    let body_mod = w.mobile(caster).map_or(0, |c| c.body_mod.body_id());
    if body_mod == 183 || body_mod == 184 {
        mm::send_localized_message(w, caster, 1042402, "", 0x3B2); // You cannot use incognito while wearing body paint
        return;
    }

    if hooks(w)
        .and_then(|h| h.is_disguised)
        .is_some_and(|f| f(w, caster))
    {
        mm::send_localized_message(w, caster, 1061631, "", 0x3B2); // You can't do that while disguised.
        return;
    }

    let is_body_mod = w.mobile(caster).is_some_and(|c| c.body_mod.body_id() != 0);
    if !combat::can_begin_action(w, caster, POLYMORPH_KEY) || is_body_mod {
        let ty = spell::spell_type(w, sp).expect("the spell is casting");
        (ty.do_fizzle)(w, sp);
        return;
    }

    if spell::check_sequence(w, sp) {
        if combat::begin_action(w, caster, KEY) {
            if let Some(f) = hooks(w).and_then(|h| h.disguise_stop_timer) {
                f(w, caster);
            }

            let (race_id, female) = w
                .mobile(caster)
                .map_or((None, false), |c| (c.race, c.female));
            let ops = race_id.and_then(|r| w.resources.get::<Races>()?.get(r).map(|r| r.ops));
            if let Some(ops) = ops {
                let hue = (ops.random_skin_hue)(&mut w.rng);
                props_b::set_hue_mod(w, caster, hue);
            }
            let name = random_name(w, if female { "female" } else { "male" });
            props_b::set_name_mod(w, caster, Some(name));

            if is_player_mobile(w, caster)
                && let Some(r) = race_id
            {
                set_random_hair(w, caster, r);
            }

            effects::fixed_particles(w, caster, 0x373A, 10, 15, 5036, EffectLayer::Head);
            effects::play_sound(w, caster, 0x3BD);

            armor::validate_mobile(w, caster);
            clothing::validate_mobile(w, caster);

            stop_timer(w, caster);

            let secs = (6.0 * skills::value(w, caster, SkillName::Magery) / 5.0).min(144.0);
            let duration = TimeSpan::from_secs_f64(secs);
            let h = timer::start(w, duration, TimeSpan::ZERO, 1, move |w| {
                end_incognito(w, caster)
            });
            table(w).insert(caster, h);

            if is_player_mobile(w, caster) {
                add_buff(
                    w,
                    caster,
                    BuffInfo::new(BuffIcon::Incognito, 1075819).duration(duration),
                );
            }
        } else {
            mm::send_localized_message(w, caster, 1079022, "", 0x3B2); // You're already incognitoed!
        }
    }

    spell::finish_sequence(w, sp);
}

/// `pm.SetHairMods(Race.RandomHair, Race.RandomFacialHair)` then the random hair hues.
fn set_random_hair(w: &mut World, pm: MobileId, race_id: race::RaceId) {
    let Some(r) = w
        .resources
        .get::<Races>()
        .and_then(|rs| rs.get(race_id))
        .map(|r| r.ops)
    else {
        return;
    };
    let female = w.mobile(pm).is_some_and(|m| m.female);
    let hair = (r.random_hair)(&mut w.rng, female);
    let beard = (r.random_facial_hair)(&mut w.rng, female);
    muo_content::player_mobile::set_hair_mods(w, pm, hair, beard);
    let hh = (r.random_hair_hue)(&mut w.rng);
    props_b::set_hair_hue(w, pm, hh);
    let fh = (r.random_hair_hue)(&mut w.rng);
    props_b::set_facial_hair_hue(w, pm, fh);
}

/// `StopTimer(m)`.
pub fn stop_timer(w: &mut World, m: MobileId) {
    if let Some(mut t) = table(w).shift_remove(&m) {
        t.cancel(w);
    }
    if is_player_mobile(w, m) {
        remove_buff(w, m, BuffIcon::Incognito);
    }
}

/// `EndIncognito(m)`.
pub fn end_incognito(w: &mut World, m: MobileId) {
    if combat::can_begin_action(w, m, KEY) {
        return;
    }

    if is_player_mobile(w, m) {
        muo_content::player_mobile::set_hair_mods(w, m, -1, -1);
    }
    props_b::set_body_mod(w, m, muo_primitives::Body::new(0));
    props_b::set_hue_mod(w, m, -1);
    props_b::set_name_mod(w, m, None);
    combat::end_action(w, m, KEY);

    armor::validate_mobile(w, m);
    clothing::validate_mobile(w, m);

    stop_timer(w, m);
}
