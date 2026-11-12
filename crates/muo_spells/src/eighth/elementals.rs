//! Air, Earth, Fire and Water Elemental and Summon Daemon: five summon spells that differ in text, hand effect,
//! followers needed and the creature.

use muo_content::core;
use muo_content::factory;
use muo_content::misc::titles;
use muo_content::spells::helper;
use muo_content::spells::magery::{EIGHTH_CIRCLE, MAGERY_SPELL};
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_content::spells::{SpellInfo, reagent};
use muo_data::expansion::Expansion;
use muo_primitives::TimeSpan;
use muo_server::World;
use muo_server::mobile::part2::messages as mm;
use muo_server::skills::{self, SkillName};

/// `if (Caster.Followers + n > Caster.FollowersMax) { SendLocalizedMessage(1049645); return false; }`.
fn check_followers(w: &mut World, sp: SpellId, n: i32) -> bool {
    if !spell::base_check_cast(w, sp) {
        return false;
    }
    let Some(caster) = spell::caster(w, sp) else {
        return false;
    };
    let (followers, max) = w
        .mobile(caster)
        .map_or((0, 0), |c| (c.followers, c.followers_max));
    if followers + n > max {
        mm::send_localized_message(w, caster, 1049645, "", 0x3B2); // You have too many followers to summon that creature.
        return false;
    }
    true
}

/// The `OnCast` of the elementals and the daemon.
fn summon(w: &mut World, sp: SpellId, class: &str, sound: i32, karma: i32) {
    if spell::check_sequence(w, sp)
        && let Some(caster) = spell::caster(w, sp)
    {
        let magery = skills::value(w, caster, SkillName::Magery);
        let duration = if core::expansion(w) == Expansion::None {
            TimeSpan::from_secs_f64(magery)
        } else {
            // T2A -> Current
            TimeSpan::from_secs_f64(4.0 * magery.max(5.0))
        };

        if let Some(c) = factory::make_mobile(w, class) {
            helper::summon(w, c, caster, sound, duration, false, false);
        }
        if karma != 0 {
            titles::award_karma(w, caster, karma, true);
        }
    }
    spell::finish_sequence(w, sp);
}

/// `AirElementalSpell`: Kal Vas Xen Hur.
pub static AIR_ELEMENTAL_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Eighth.AirElementalSpell",
    parent: Some(&MAGERY_SPELL),
    info: SpellInfo::new(
        "Air Elemental",
        "Kal Vas Xen Hur",
        &[
            reagent::BLOODMOSS,
            reagent::MANDRAKE_ROOT,
            reagent::SPIDERS_SILK,
        ],
    )
    .action(269)
    .hand_effect(9010)
    .allow_town(false),
    on_cast: |w, sp| summon(w, sp, "Server.Mobiles.SummonedAirElemental", 0x217, 0),
    check_cast: |w, sp| check_followers(w, sp, 2),
    family: Some(&EIGHTH_CIRCLE),
    ..MAGERY_SPELL
};

/// `EarthElementalSpell`: Kal Vas Xen Ylem.
pub static EARTH_ELEMENTAL_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Eighth.EarthElementalSpell",
    parent: Some(&MAGERY_SPELL),
    info: SpellInfo::new(
        "Earth Elemental",
        "Kal Vas Xen Ylem",
        &[
            reagent::BLOODMOSS,
            reagent::MANDRAKE_ROOT,
            reagent::SPIDERS_SILK,
        ],
    )
    .action(269)
    .hand_effect(9020)
    .allow_town(false),
    on_cast: |w, sp| summon(w, sp, "Server.Mobiles.SummonedEarthElemental", 0x217, 0),
    check_cast: |w, sp| check_followers(w, sp, 2),
    family: Some(&EIGHTH_CIRCLE),
    ..MAGERY_SPELL
};

/// `FireElementalSpell`: Kal Vas Xen Flam.
pub static FIRE_ELEMENTAL_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Eighth.FireElementalSpell",
    parent: Some(&MAGERY_SPELL),
    info: SpellInfo::new(
        "Fire Elemental",
        "Kal Vas Xen Flam",
        &[
            reagent::BLOODMOSS,
            reagent::MANDRAKE_ROOT,
            reagent::SPIDERS_SILK,
            reagent::SULFUROUS_ASH,
        ],
    )
    .action(269)
    .hand_effect(9050)
    .allow_town(false),
    on_cast: |w, sp| summon(w, sp, "Server.Mobiles.SummonedFireElemental", 0x217, 0),
    check_cast: |w, sp| check_followers(w, sp, 4),
    family: Some(&EIGHTH_CIRCLE),
    ..MAGERY_SPELL
};

/// `WaterElementalSpell`: Kal Vas Xen An Flam.
pub static WATER_ELEMENTAL_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Eighth.WaterElementalSpell",
    parent: Some(&MAGERY_SPELL),
    info: SpellInfo::new(
        "Water Elemental",
        "Kal Vas Xen An Flam",
        &[
            reagent::BLOODMOSS,
            reagent::MANDRAKE_ROOT,
            reagent::SPIDERS_SILK,
        ],
    )
    .action(269)
    .hand_effect(9070)
    .allow_town(false),
    on_cast: |w, sp| summon(w, sp, "Server.Mobiles.SummonedWaterElemental", 0x217, 0),
    check_cast: |w, sp| check_followers(w, sp, 3),
    family: Some(&EIGHTH_CIRCLE),
    ..MAGERY_SPELL
};

fn daemon_followers(w: &mut World, sp: SpellId) -> bool {
    check_followers(w, sp, if core::se(w) { 4 } else { 5 })
}

/// `SummonDaemonSpell`: Kal Vas Xen Corp.
pub static SUMMON_DAEMON_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Eighth.SummonDaemonSpell",
    parent: Some(&MAGERY_SPELL),
    info: SpellInfo::new(
        "Summon Daemon",
        "Kal Vas Xen Corp",
        &[
            reagent::BLOODMOSS,
            reagent::MANDRAKE_ROOT,
            reagent::SPIDERS_SILK,
            reagent::SULFUROUS_ASH,
        ],
    )
    .action(269)
    .hand_effect(9050)
    .allow_town(false),
    on_cast: |w, sp| summon(w, sp, "Server.Mobiles.SummonedDaemon", 0x216, -70),
    check_cast: daemon_followers,
    family: Some(&EIGHTH_CIRCLE),
    ..MAGERY_SPELL
};
