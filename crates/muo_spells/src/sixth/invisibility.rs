//! The Invisibility spell.

use indexmap::IndexMap;
use muo_content::player_mobile::buff::{BuffIcon, BuffInfo, add_buff, remove_buff};
use muo_content::spells::magery::{MAGERY_SPELL, SIXTH_CIRCLE};
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_content::spells::targeting::{SpellTargetKind, begin_spell_target};
use muo_content::spells::{SpellInfo, reagent};
use muo_primitives::{Point3D, TimeSpan};
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::item::effect_item;
use muo_server::mobile::base as mb;
use muo_server::mobile::part1::props_b;
use muo_server::mobile::part2::{effects, messages as mm};
use muo_server::skills::{self, SkillName};
use muo_server::target::{TargetFlags, Targeted};
use muo_server::timer::{self, TimerHandle};

use crate::common::{
    check_cast_sudden_death, is_player_mobile, mobile_is, mobile_of, revealing_action, turn_to,
};

/// `InvisibilitySpell`: An Lor Xen.
pub static INVISIBILITY_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Sixth.InvisibilitySpell",
    parent: Some(&MAGERY_SPELL),
    target_range: Some(muo_content::spells::targeting::target_range),
    info: SpellInfo::new(
        "Invisibility",
        "An Lor Xen",
        &[reagent::BLOODMOSS, reagent::NIGHTSHADE],
    )
    .action(206)
    .hand_effect(9002),
    on_cast,
    check_cast: check_cast_sudden_death,
    family: Some(&SIXTH_CIRCLE),
    ..MAGERY_SPELL
};

#[derive(Default)]
struct Table(IndexMap<MobileId, TimerHandle>);

fn table(w: &mut World) -> &mut IndexMap<MobileId, TimerHandle> {
    &mut w.resources.get_mut::<Table>().0
}

fn target(w: &mut World, sp: SpellId, targeted: Targeted) {
    let Some(m) = mobile_of(targeted) else {
        return;
    };
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };

    let vendor = mobile_is(w, m, "Server.Mobiles.BaseVendor")
        || mobile_is(w, m, "Server.Mobiles.PlayerVendor");
    let higher = match (w.mobile(m), w.mobile(caster)) {
        (Some(a), Some(b)) => a.access_level > b.access_level,
        _ => false,
    };
    if vendor || higher {
        mm::send_localized_message(w, caster, 501857, "", 0x3B2); // This spell won't work on that!
    } else if spell::check_b_sequence(w, sp, m, false) {
        turn_to(w, caster, m);

        let (loc, _) = w.mobile(m).map(|x| (x.location, x.map)).unwrap_or_default();
        let map = w.mobile(caster).and_then(|c| c.map);
        let e = effect_item::create_targeted(
            w,
            Point3D::new(loc.x, loc.y, loc.z + 16),
            map,
            effect_item::DEFAULT_DURATION,
        );
        muo_server::effects::send_location_particles(w, e, 0x376A, 10, 15, 5045);
        effects::play_sound(w, m, 0x3C4);

        props_b::set_hidden(w, m, true);
        mb::set_combatant(w, m, None);
        props_b::set_warmode(w, m, false);

        stop_timer(w, m);

        let duration = TimeSpan::from_secs_f64(1.2 * skills::value(w, caster, SkillName::Magery));

        if is_player_mobile(w, m) {
            remove_buff(w, m, BuffIcon::HidingAndOrStealth);
            add_buff(
                w,
                m,
                BuffInfo::new(BuffIcon::Invisibility, 1075825).duration(duration),
            ); // Invisibility/Invisible
        }

        let h = timer::start(w, duration, TimeSpan::ZERO, 1, move |w| {
            revealing_action(w, m);
            stop_timer(w, m);
        });
        table(w).insert(m, h);
    }
}

fn on_cast(w: &mut World, sp: SpellId) {
    begin_spell_target(
        w,
        sp,
        false,
        TargetFlags::BENEFICIAL,
        false,
        SpellTargetKind::Mobile,
        target,
    );
}

/// `HasTimer(m)`.
pub fn has_timer(w: &mut World, m: MobileId) -> bool {
    table(w).contains_key(&m)
}

/// `StopTimer(m)`.
pub fn stop_timer(w: &mut World, m: MobileId) {
    if let Some(mut t) = table(w).shift_remove(&m) {
        t.cancel(w);
    }
}
