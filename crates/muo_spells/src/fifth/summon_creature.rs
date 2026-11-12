//! The Summon Creature spell.

use muo_content::core;
use muo_content::factory;
use muo_content::spells::helper;
use muo_content::spells::magery::{FIFTH_CIRCLE, MAGERY_SPELL};
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_content::spells::{SpellInfo, reagent};
use muo_data::expansion::Expansion;
use muo_primitives::{TimeSpan, utility};
use muo_server::World;
use muo_server::mobile::part2::messages as mm;
use muo_server::skills::{self, SkillName};

/// `SummonCreatureSpell`: Kal Xen.
pub static SUMMON_CREATURE_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Fifth.SummonCreatureSpell",
    parent: Some(&MAGERY_SPELL),
    info: SpellInfo::new(
        "Summon Creature",
        "Kal Xen",
        &[
            reagent::BLOODMOSS,
            reagent::MANDRAKE_ROOT,
            reagent::SPIDERS_SILK,
        ],
    )
    .allow_town(false),
    on_cast,
    check_cast,
    get_cast_delay,
    family: Some(&FIFTH_CIRCLE),
    ..MAGERY_SPELL
};

// NOTE: Creature list based on 1hr of summon/release on OSI.
const TYPES: [&str; 18] = [
    "Server.Mobiles.PolarBear",
    "Server.Mobiles.GrizzlyBear",
    "Server.Mobiles.BlackBear",
    "Server.Mobiles.Horse",
    "Server.Mobiles.Walrus",
    "Server.Mobiles.Chicken",
    "Server.Mobiles.Scorpion",
    "Server.Mobiles.GiantSerpent",
    "Server.Mobiles.Llama",
    "Server.Mobiles.Alligator",
    "Server.Mobiles.GreyWolf",
    "Server.Mobiles.Slime",
    "Server.Mobiles.Eagle",
    "Server.Mobiles.Gorilla",
    "Server.Mobiles.SnowLeopard",
    "Server.Mobiles.Pig",
    "Server.Mobiles.Hind",
    "Server.Mobiles.Rabbit",
];

fn check_cast(w: &mut World, sp: SpellId) -> bool {
    if !spell::base_check_cast(w, sp) {
        return false;
    }
    let Some(caster) = spell::caster(w, sp) else {
        return false;
    };
    let (followers, max) = w
        .mobile(caster)
        .map_or((0, 0), |c| (c.followers, c.followers_max));
    if followers + 2 > max {
        mm::send_localized_message(w, caster, 1049645, "", 0x3B2); // You have too many followers to summon that creature.
        return false;
    }
    true
}

fn on_cast(w: &mut World, sp: SpellId) {
    if spell::check_sequence(w, sp) {
        let Some(caster) = spell::caster(w, sp) else {
            return;
        };
        let pick = utility::random(&mut w.rng, TYPES.len() as i32) as usize;
        if let Some(creature) = factory::make_mobile(w, TYPES[pick.min(TYPES.len() - 1)]) {
            // creature.ControlSlots = 2;
            let magery = skills::value(w, caster, SkillName::Magery);
            let duration = if core::expansion(w) == Expansion::None {
                TimeSpan::from_secs_f64(magery)
            } else {
                TimeSpan::from_secs(i64::from(magery as i32) * 4)
            };
            helper::summon(w, creature, caster, 0x215, duration, false, false);
        }
    }

    spell::finish_sequence(w, sp);
}

fn get_cast_delay(w: &mut World, sp: SpellId) -> TimeSpan {
    let base = (MAGERY_SPELL.get_cast_delay)(w, sp);
    let mut delay = TimeSpan::from_ticks(base.ticks() * if core::aos(w) { 5 } else { 4 });

    // SA made everything 0.25 slower, but that is applied after the scalar
    // So remove 0.25 * 5 to compensate
    if core::sa(w) {
        delay -= TimeSpan::from_secs_f64(1.25);
    }

    delay
}
