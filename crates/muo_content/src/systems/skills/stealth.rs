//! The Hiding, Stealth and Meditation skills.

use muo_primitives::{MessageType, Point3D, TimeSpan, utility};
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::layer::Layer;
use muo_server::mobile::base as mb;
use muo_server::mobile::part1::{props_a, props_b};
use muo_server::mobile::part2::{effects, items as mitems, messages as mm};
use muo_server::skills::{self, SkillName};
use muo_server::target;

use super::common::{check_skill, item_is, msg};
use super::hooks::ops;
use super::skills_info::set_callback;
use crate::core;
use crate::misc::regen_rates::RegenHooks;
use crate::player_mobile::{self, PLAYER_MOBILE, buff};

// ---- Hiding -----------------------------------------------------------------------------------------------------

/// `Hiding.CombatOverride`.
#[derive(Default)]
pub struct HidingSettings {
    pub combat_override: bool,
}

pub fn hiding_on_use(w: &mut World, m: MobileId) -> TimeSpan {
    if w.mobile(m).is_some_and(|x| x.spell.is_some()) {
        msg(w, m, 501238); // You are busy doing something else and cannot hide.
        return TimeSpan::from_secs(1);
    }

    if core::ml(w) && w.mobile(m).is_some_and(|x| x.target.is_some()) {
        target::cancel_mobile_target(w, m);
    }

    let mut bonus = 0.0;

    let mut house = ops(w).find_house_at(w, m);

    if house.is_some_and(|h| ops(w).house_is_friend(w, h, m)) {
        bonus = 100.0;
    } else if !core::aos(w) {
        let (loc, map) = w.mobile(m).map(|x| (x.location, x.map)).unwrap_or_default();
        if house.is_none() {
            for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                house = ops(w).find_house_at_point(
                    w,
                    Point3D::new(loc.x + dx, loc.y + dy, 127),
                    map,
                    16,
                );
                if house.is_some() {
                    break;
                }
            }
        }

        if house.is_some() {
            bonus = 50.0;
        }
    }

    // int range = 18 - (int)(m.Skills.Hiding.Value / 10);
    // Cap of 18 not OSI-exact, intentional difference
    let range = (((100.0 - skills::value(w, m, SkillName::Hiding)) / 2.0) as i32 + 8).min(18);

    let combat_override = w.resources.get_mut::<HidingSettings>().combat_override;
    let combatant = w.mobile(m).and_then(|x| x.combatant);
    let mut bad_combat = !combat_override
        && combatant.is_some_and(|c| {
            let cloc = w.mobile(c).map(|x| x.location).unwrap_or_default();
            mb::in_range_point3d(w, m, cloc, range)
        })
        && combatant.is_some_and(|c| crate::mx::in_los(w, c, m));
    let mut ok = !bad_combat;

    if ok {
        if !combat_override {
            for check in crate::mx::mobiles_in_range(w, m, range) {
                if crate::mx::in_los(w, check, m)
                    && w.mobile(check).and_then(|x| x.combatant) == Some(m)
                {
                    bad_combat = true;
                    break;
                }
            }
        }

        ok = !bad_combat && check_skill(w, m, SkillName::Hiding, 0.0 - bonus, 100.0 - bonus);
    }

    if bad_combat {
        mb::revealing_action(w, m);

        mm::local_overhead_message_number(w, m, MessageType::REGULAR, 0x22, 501237, ""); // You can't seem to hide right now.

        return TimeSpan::from_secs(1);
    }

    if ok {
        props_b::set_hidden(w, m, true);
        props_b::set_warmode(w, m, false);
        mm::local_overhead_message_number(w, m, MessageType::REGULAR, 0x1F4, 501240, ""); // You have hidden yourself well.
        ops(w).stop_invisibility_timer(w, m);
    } else {
        mb::revealing_action(w, m);
        mm::local_overhead_message_number(w, m, MessageType::REGULAR, 0x22, 501241, ""); // You can't seem to hide here.
    }

    TimeSpan::from_secs(10)
}

// ---- Stealth ----------------------------------------------------------------------------------------------------

/// `Stealth.HidingRequirement`.
pub fn hiding_requirement(w: &World) -> f64 {
    if core::ml(w) {
        30.0
    } else if core::se(w) {
        50.0
    } else {
        80.0
    }
}

/// `Stealth.ArmorTable`: rows by armor material, columns by body position (Gorget, Glove, Helmet, Arms, Legs, Chest,
/// Shield).
pub const ARMOR_TABLE: [[i32; 7]; 13] = [
    /* Cloth */ [0, 0, 0, 0, 0, 0, 0],
    /* Leather */ [0, 0, 0, 0, 0, 0, 0],
    /* Studded */ [2, 2, 0, 4, 6, 10, 0],
    /* Bone */ [0, 5, 10, 10, 15, 25, 0],
    /* Spined */ [0, 0, 0, 0, 0, 0, 0],
    /* Horned */ [0, 0, 0, 0, 0, 0, 0],
    /* Barbed */ [0, 0, 0, 0, 0, 0, 0],
    /* Ring */ [0, 5, 0, 10, 15, 25, 0],
    /* Chain */ [0, 0, 10, 0, 15, 25, 0],
    /* Plate */ [5, 5, 10, 10, 15, 25, 0],
    /* Dragon */ [5, 5, 10, 10, 15, 25, 0],
    /* Wood */ [5, 5, 10, 10, 15, 25, 0],
    /* Stone */ [5, 5, 10, 10, 15, 25, 0],
];

/// `GetArmorRating(m)`.
pub fn get_armor_rating(w: &mut World, m: MobileId) -> i32 {
    if !core::aos(w) {
        return mb::armor_rating(w, m) as i32;
    }

    let mut ar = 0;
    for (material_type, body_position, mage_armor) in ops(w).worn_armor(w, m) {
        let (Ok(mt), Ok(bp)) = (
            usize::try_from(material_type),
            usize::try_from(body_position),
        ) else {
            continue;
        };
        if mt >= ARMOR_TABLE.len() || bp >= ARMOR_TABLE[0].len() {
            continue;
        }

        if !mage_armor {
            ar += ARMOR_TABLE[mt][bp];
        }
    }

    ar
}

pub fn stealth_on_use(w: &mut World, m: MobileId) -> TimeSpan {
    if !w.mobile(m).is_some_and(|x| x.hidden) {
        msg(w, m, 502725); // You must hide first
    } else if skills::base(w, m, SkillName::Hiding) < hiding_requirement(w) {
        msg(w, m, 502726); // You are not hidden well enough.  Become better at hiding.
        mb::revealing_action(w, m);
    } else if !muo_server::mobile::part1::combat::can_begin_action(w, m, "Stealth") {
        msg(w, m, 1063086); // You cannot use this skill right now.
        mb::revealing_action(w, m);
    } else {
        let armor_rating = get_armor_rating(w, m);
        let aos = core::aos(w);

        if armor_rating >= if aos { 42 } else { 26 } {
            // I have a hunch '42' was chosen cause someone's a fan of DNA
            msg(w, m, 502727); // You could not hope to move quietly wearing this much armor.
            mb::revealing_action(w, m);
        } else if check_skill(
            w,
            m,
            SkillName::Stealth,
            -20.0 + f64::from(armor_rating) * 2.0,
            (if aos { 60.0 } else { 80.0 }) + f64::from(armor_rating) * 2.0,
        ) {
            let steps = ((skills::value(w, m, SkillName::Stealth) / if aos { 5.0 } else { 10.0 })
                as i32)
                .max(1);
            if let Some(x) = w.mobile_mut(m) {
                x.allowed_stealth_steps = steps;
            }

            if let Some(d) = player_mobile::data_mut(w, m) {
                d.is_stealthing = true;
            }

            msg(w, m, 502730); // You begin to move quietly.

            return TimeSpan::from_secs(10);
        } else {
            msg(w, m, 502731); // You fail in your attempt to move unnoticed.
            mb::revealing_action(w, m);
        }
    }

    TimeSpan::from_secs(10)
}

// ---- Meditation -------------------------------------------------------------------------------------------------

/// `CheckOkayHolding(item)`.
pub fn check_okay_holding(w: &World, item: Option<ItemId>) -> bool {
    let Some(i) = item else { return true };
    item_is(w, i, "Server.Items.Spellbook")
        || item_is(w, i, "Server.Items.Runebook")
        || (core::aos(w) && ops(w).okay_to_hold_while_meditating(w, i))
}

fn armor_offset(w: &World, m: MobileId) -> f64 {
    w.resources
        .get::<RegenHooks>()
        .and_then(|h| h.armor_offset)
        .map_or(0.0, |f| f(w, m))
}

pub fn meditation_on_use(w: &mut World, m: MobileId) -> TimeSpan {
    mb::revealing_action(w, m);

    if w.mobile(m).is_some_and(|x| x.target.is_some()) {
        msg(w, m, 501845); // You are busy doing something else and cannot focus.

        return TimeSpan::from_secs(5);
    }

    let aos = core::aos(w);
    let (hits, hits_max, mana, mana_max) = w.mobile(m).map_or((0, 1, 0, 1), |x| {
        (x.hits, mb::hits_max(w, m), x.mana, mb::mana_max(w, m))
    });

    if !aos && hits < hits_max / 10 {
        // Less than 10% health
        msg(w, m, 501849); // The mind is strong but the body is weak.

        return TimeSpan::from_secs(5);
    }

    if mana >= mana_max {
        msg(w, m, 501846); // You are at peace.

        return TimeSpan::from_secs(if aos { 10 } else { 5 });
    }

    if aos && armor_offset(w, m) > 0.0 {
        msg(w, m, 500135); // Regenerative forces cannot penetrate your armor!

        return TimeSpan::from_secs(10);
    }

    let one_handed = mitems::find_item_on_layer(w, m, Layer::OneHanded);
    let two_handed = mitems::find_item_on_layer(w, m, Layer::TwoHanded);

    let is_player = w.mobile(m).is_some_and(|x| x.player);
    if aos && is_player {
        for hand in [one_handed, two_handed] {
            if !check_okay_holding(w, hand)
                && let Some(h) = hand
            {
                mitems::add_to_backpack(w, m, h);
            }
        }
    } else if !check_okay_holding(w, one_handed) || !check_okay_holding(w, two_handed) {
        msg(w, m, 502626); // Your hands must be free to cast spells or meditate.

        return TimeSpan::from_millis(2500);
    }

    let skill_val = skills::value(w, m, SkillName::Meditation);
    let chance = (50.0 + (skill_val - f64::from(mana_max - mana)) * 2.0) / 100.0;

    if chance > utility::random_double(&mut w.rng) {
        check_skill(w, m, SkillName::Meditation, 0.0, 100.0);

        msg(w, m, 501851); // You enter a meditative trance.
        if let Some(x) = w.mobile_mut(m) {
            x.meditating = true;
        }
        if w.mobile(m).is_some_and(|x| x.ty.is(&PLAYER_MOBILE)) {
            buff::add_buff(
                w,
                m,
                buff::BuffInfo::new(buff::BuffIcon::ActiveMeditation, 1075657),
            );
        }

        if is_player || super::common::is_human(w, m) {
            effects::play_sound(w, m, 0xF9);
        }
    } else {
        msg(w, m, 501850); // You cannot focus your concentration.
    }

    TimeSpan::from_secs(10)
}

pub fn initialize(w: &mut World) {
    set_callback(w, SkillName::Hiding, hiding_on_use);
    set_callback(w, SkillName::Meditation, meditation_on_use);
    set_callback(w, SkillName::Stealth, stealth_on_use);
}

#[allow(unused)]
fn _unused(_: MessageType) {
    let _ = props_a::set_spell;
}

#[cfg(test)]
mod tests {
    use super::*;
    use muo_data::expansion::Expansion;
    use muo_server::mobile::MOBILE_BASE;

    #[test]
    fn hiding_requirement_by_era() {
        let mut w = World::new();
        w.core.expansion = Some(Expansion::AOS);
        assert_eq!(hiding_requirement(&w), 80.0);
        w.core.expansion = Some(Expansion::SE);
        assert_eq!(hiding_requirement(&w), 50.0);
        w.core.expansion = Some(Expansion::ML);
        assert_eq!(hiding_requirement(&w), 30.0);
    }

    #[test]
    fn armor_table_shape() {
        // Studded chest 10, plate legs 15, cloth nothing, shields never count.
        assert_eq!(ARMOR_TABLE[2][5], 10);
        assert_eq!(ARMOR_TABLE[9][4], 15);
        assert!(ARMOR_TABLE[0].iter().all(|&v| v == 0));
        assert!(ARMOR_TABLE.iter().all(|row| row[6] == 0));
    }

    #[test]
    fn stealth_requires_hiding_first() {
        let mut w = World::new();
        let m = w.new_mobile(&MOBILE_BASE);
        assert_eq!(stealth_on_use(&mut w, m), TimeSpan::from_secs(10));
    }

    #[test]
    fn meditation_at_full_mana_is_at_peace() {
        let mut w = World::new();
        w.core.expansion = Some(Expansion::AOS);
        let m = w.new_mobile(&MOBILE_BASE);
        // mana 0 of a ManaMax of Int: a fresh mobile has int 0 -> mana_max 0 -> at peace.
        assert_eq!(meditation_on_use(&mut w, m), TimeSpan::from_secs(10));
        w.core.expansion = Some(Expansion::T2A);
        assert_eq!(meditation_on_use(&mut w, m), TimeSpan::from_secs(5));
    }

    #[test]
    fn holding_nothing_is_fine() {
        let w = World::new();
        assert!(check_okay_holding(&w, None));
    }
}
