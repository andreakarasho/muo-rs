//! The Polymorph spell.

use indexmap::IndexMap;
use muo_content::core;
use muo_content::items::{armor, clothing};
use muo_content::spells::helper;
use muo_content::spells::hooks::hooks;
use muo_content::spells::magery::{MAGERY_SPELL, SEVENTH_CIRCLE};
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_content::spells::{SpellInfo, reagent};
use muo_data::expansion::Expansion;
use muo_primitives::{Body, TimeSpan};
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::mobile::part1::{combat, props_b};
use muo_server::mobile::part2::messages as mm;
use muo_server::skills::{self, SkillName};
use muo_server::timer::{self, TimerHandle};

use crate::common::random_skin_hue;
use crate::fifth::incognito;

/// `PolymorphSpell`: Vas Ylem Rel.
pub static POLYMORPH_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Seventh.PolymorphSpell",
    parent: Some(&MAGERY_SPELL),
    info: SpellInfo::new(
        "Polymorph",
        "Vas Ylem Rel",
        &[
            reagent::BLOODMOSS,
            reagent::SPIDERS_SILK,
            reagent::MANDRAKE_ROOT,
        ],
    )
    .action(221)
    .hand_effect(9002),
    on_cast,
    check_cast,
    family: Some(&SEVENTH_CIRCLE),
    ..MAGERY_SPELL
};

/// The `BeginAction<PolymorphSpell>()` key.
pub const KEY: &str = "Server.Spells.Seventh.PolymorphSpell";

/// `m_NewBody`.
#[derive(Clone, Copy, Default)]
struct NewBody(i32);

#[derive(Default)]
struct Table(IndexMap<MobileId, TimerHandle>);

fn table(w: &mut World) -> &mut IndexMap<MobileId, TimerHandle> {
    &mut w.resources.get_mut::<Table>().0
}

/// `new PolymorphSpell(caster, scroll, body)`.
pub fn new_polymorph_spell(
    w: &mut World,
    caster: MobileId,
    scroll: Option<ItemId>,
    body: i32,
) -> SpellId {
    let id = spell::new_spell(w, &POLYMORPH_SPELL, caster, scroll);
    if let Some(s) = spell::spells(w).get_mut(id) {
        s.ext = muo_server::Ext::new(NewBody(body));
    }
    id
}

fn new_body(w: &World, sp: SpellId) -> i32 {
    spell::spell(w, sp)
        .and_then(|s| s.ext.get::<NewBody>().copied())
        .map_or(0, |b| b.0)
}

/// Sets the `PlayerDeathEvent` handler and the gump's `polymorph_cast` hook.
pub fn init(w: &mut World) {
    muo_content::player_mobile::subscribe_death(w, |w, m| {
        stop_timer(w, m);
        combat::end_action(w, m, KEY);
    });
    muo_gumps::m_z::hooks::hooks_mut(w).polymorph_cast = Some(|w, caster, scroll, body| {
        let sp = new_polymorph_spell(w, caster, scroll, body);
        spell::cast(w, sp);
    });
}

fn sigil_exists_on(w: &World, m: MobileId) -> bool {
    hooks(w)
        .and_then(|h| h.sigil_exists_on)
        .is_some_and(|f| f(w, m))
}

fn body_mod(w: &World, m: MobileId) -> i32 {
    w.mobile(m).map_or(0, |x| x.body_mod.body_id())
}

fn check_cast(w: &mut World, sp: SpellId) -> bool {
    let Some(caster) = spell::caster(w, sp) else {
        return false;
    };
    if sigil_exists_on(w, caster) {
        mm::send_localized_message(w, caster, 1010521, "", 0x3B2); // You cannot polymorph while you have a Town Sigil
        return false;
    }

    if helper::under_transformation(w, caster) {
        mm::send_localized_message(w, caster, 1061633, "", 0x3B2); // You cannot polymorph while in that form.
        return false;
    }

    if hooks(w)
        .and_then(|h| h.is_disguised)
        .is_some_and(|f| f(w, caster))
    {
        mm::send_localized_message(w, caster, 502167, "", 0x3B2); // You cannot polymorph while disguised.
        return false;
    }

    let bm = body_mod(w, caster);
    if bm == 183 || bm == 184 {
        mm::send_localized_message(w, caster, 1042512, "", 0x3B2); // You cannot polymorph while wearing body paint
        return false;
    }

    if !combat::can_begin_action(w, caster, KEY) {
        if core::ml(w) {
            end_polymorph(w, caster);
        } else {
            mm::send_localized_message(w, caster, 1005559, "", 0x3B2); // This spell is already in effect.
        }

        return false;
    }

    if new_body(w, sp) == 0 {
        let scroll = spell::scroll(w, sp);
        let gump: Box<dyn muo_content::gumps::Gump> = if core::se(w) {
            Box::new(muo_gumps::m_z::simple::new_new_polymorph(scroll))
        } else {
            Box::new(muo_gumps::m_z::simple::new_polymorph(scroll))
        };
        muo_content::gumps::system::send_gump_to_mobile(w, caster, gump, false);
        return false;
    }

    true
}

fn on_cast(w: &mut World, sp: SpellId) {
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };

    if sigil_exists_on(w, caster) {
        mm::send_localized_message(w, caster, 1010521, "", 0x3B2); // You cannot polymorph while you have a Town Sigil
        return;
    }

    if !combat::can_begin_action(w, caster, KEY) {
        if core::ml(w) {
            end_polymorph(w, caster);
        } else {
            mm::send_localized_message(w, caster, 1005559, "", 0x3B2); // This spell is already in effect.
        }

        return;
    }

    if helper::under_transformation(w, caster) {
        mm::send_localized_message(w, caster, 1061633, "", 0x3B2); // You cannot polymorph while in that form.
        return;
    }

    if hooks(w)
        .and_then(|h| h.is_disguised)
        .is_some_and(|f| f(w, caster))
    {
        mm::send_localized_message(w, caster, 502167, "", 0x3B2); // You cannot polymorph while disguised.
        return;
    }

    let bm = body_mod(w, caster);
    if bm == 183 || bm == 184 {
        mm::send_localized_message(w, caster, 1042512, "", 0x3B2); // You cannot polymorph while wearing body paint
        return;
    }

    if !combat::can_begin_action(w, caster, incognito::KEY) || bm != 0 {
        let ty = spell::spell_type(w, sp).expect("the spell is casting");
        (ty.do_fizzle)(w, sp);
        return;
    }

    if spell::check_sequence(w, sp) {
        if !combat::begin_action(w, caster, KEY) {
            mm::send_localized_message(w, caster, 1005559, "", 0x3B2); // This spell is already in effect.
            return;
        }

        let nb = new_body(w, sp);
        if nb != 0 {
            let human = w
                .resources
                .get::<muo_primitives::BodyTable>()
                .is_some_and(|t| t.is_human(Body::new(nb)));
            if !human {
                let mt = props_b::mount(w, caster);
                if let Some(mt) = mt {
                    muo_content::creature::mount::set_rider(w, mt, None);
                }
            }

            props_b::set_body_mod(w, caster, Body::new(nb));

            let hue = if nb == 400 || nb == 401 {
                random_skin_hue(w, caster).unwrap_or(0)
            } else {
                0
            };
            props_b::set_hue_mod(w, caster, hue);

            armor::validate_mobile(w, caster);
            clothing::validate_mobile(w, caster);

            if !core::ml(w) {
                stop_timer(w, caster);

                let magery = skills::value(w, caster, SkillName::Magery);
                let duration = if core::expansion(w) == Expansion::None {
                    TimeSpan::from_secs_f64(6.0 * magery / 5.0)
                } else {
                    TimeSpan::from_secs_f64(magery.max(120.0))
                };

                let h = timer::start(w, duration, TimeSpan::ZERO, 1, move |w| {
                    end_polymorph(w, caster)
                });
                table(w).insert(caster, h);
            }
        }
    }

    spell::finish_sequence(w, sp);
}

/// `StopTimer(m)`.
pub fn stop_timer(w: &mut World, m: MobileId) {
    if let Some(mut t) = table(w).shift_remove(&m) {
        t.cancel(w);
    }
}

/// `EndPolymorph(m)`.
pub fn end_polymorph(w: &mut World, m: MobileId) {
    if combat::can_begin_action(w, m, KEY) {
        return;
    }

    props_b::set_body_mod(w, m, Body::new(0));
    props_b::set_hue_mod(w, m, -1);
    props_b::set_name_mod(w, m, None);
    combat::end_action(w, m, KEY);

    armor::validate_mobile(w, m);
    clothing::validate_mobile(w, m);

    stop_timer(w, m);
}
