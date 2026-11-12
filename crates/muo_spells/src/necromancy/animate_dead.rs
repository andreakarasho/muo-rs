//! The Animate Dead spell.
// The `MaabusCoffinComponent` / `DarkTidesQuest` branch of `Target` goes through `SpellHooks::animate_maabus_coffin`,
// which muo_engines sets (quests/dark_tides_mobs.rs).

use indexmap::IndexMap;
use muo_content::creature::{self, control};
use muo_content::factory;
use muo_content::items::corpse;
use muo_content::items::slayer;
use muo_content::items::slayer::SlayerName;
use muo_content::misc::aos_helpers;
use muo_content::spells::hooks::hooks;
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_content::spells::targeting::{SpellTargetKind, begin_spell_target};
use muo_content::spells::{SpellInfo, reagent};
use muo_primitives::{TimeSpan, utility};
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::map::MapId;
use muo_server::mobile::base as mb;
use muo_server::mobile::part1::props_b;
use muo_server::mobile::part2::messages as mm;
use muo_server::skills::{self, SkillName};
use muo_server::target::{TargetFlags, Targeted};
use muo_server::timer;

use super::necromancer::{NECRO_BASE, NECROMANCER_SPELL, NecroType};
use crate::common::{item_is, item_of, mobile_is};

static ANIMATE_DEAD: NecroType = NecroType {
    required_skill: 40.0,
    required_mana: 23,
    ..NECRO_BASE
};

/// `AnimateDeadSpell`: Uus Corp.
pub static ANIMATE_DEAD_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Necromancy.AnimateDeadSpell",
    parent: Some(&NECROMANCER_SPELL),
    target_range: Some(muo_content::spells::targeting::target_range),
    info: SpellInfo::new(
        "Animate Dead",
        "Uus Corp",
        &[reagent::GRAVE_DUST, reagent::DAEMON_BLOOD],
    )
    .action(203)
    .hand_effect(9031),
    cast_delay_base: |_, _| TimeSpan::from_millis(1500),
    on_cast,
    family: Some(&ANIMATE_DEAD),
    ..NECROMANCER_SPELL
};

/// `SummonEntry`: the requirement and the classes to pick one of.
struct SummonEntry(i32, &'static [&'static str]);

/// `CreatureGroup`: the classes (and their subclasses) it applies to, and its entries.
struct CreatureGroup {
    types: &'static [&'static str],
    entries: &'static [SummonEntry],
}

const fn dragons_entries() -> &'static [SummonEntry] {
    &[
        SummonEntry(18000, &["Server.Mobiles.SkeletalDragon"]),
        SummonEntry(10000, &["Server.Mobiles.FleshGolem"]),
        SummonEntry(5000, &["Server.Mobiles.Lich"]),
        SummonEntry(
            3000,
            &["Server.Mobiles.SkeletalKnight", "Server.Mobiles.BoneKnight"],
        ),
        SummonEntry(2000, &["Server.Mobiles.Mummy"]),
        SummonEntry(
            1000,
            &["Server.Mobiles.SkeletalMage", "Server.Mobiles.BoneMagi"],
        ),
        SummonEntry(0, &["Server.Mobiles.PatchworkSkeleton"]),
    ]
}

static GROUPS: [CreatureGroup; 6] = [
    // Undead group--empty: the types are the Silver slayer entry's, filled in `find_group`
    CreatureGroup {
        types: &[],
        entries: &[],
    },
    // Insects
    CreatureGroup {
        types: &[
            "Server.Mobiles.DreadSpider",
            "Server.Mobiles.FrostSpider",
            "Server.Mobiles.GiantSpider",
            "Server.Mobiles.GiantBlackWidow",
            "Server.Mobiles.BlackSolenInfiltratorQueen",
            "Server.Mobiles.BlackSolenInfiltratorWarrior",
            "Server.Mobiles.BlackSolenQueen",
            "Server.Mobiles.BlackSolenWarrior",
            "Server.Mobiles.BlackSolenWorker",
            "Server.Mobiles.RedSolenInfiltratorQueen",
            "Server.Mobiles.RedSolenInfiltratorWarrior",
            "Server.Mobiles.RedSolenQueen",
            "Server.Mobiles.RedSolenWarrior",
            "Server.Mobiles.RedSolenWorker",
            "Server.Mobiles.TerathanAvenger",
            "Server.Mobiles.TerathanDrone",
            "Server.Mobiles.TerathanMatriarch",
            "Server.Mobiles.TerathanWarrior",
            // TODO: Giant beetle? Ant lion? Ophidians?
        ],
        entries: &[SummonEntry(0, &["Server.Mobiles.MoundOfMaggots"])],
    },
    // Mounts
    CreatureGroup {
        types: &[
            "Server.Mobiles.Horse",
            "Server.Mobiles.Nightmare",
            "Server.Mobiles.FireSteed",
            "Server.Mobiles.Kirin",
            "Server.Mobiles.Unicorn",
        ],
        entries: &[
            SummonEntry(10000, &["Server.Mobiles.HellSteed"]),
            SummonEntry(0, &["Server.Mobiles.SkeletalMount"]),
        ],
    },
    // Elementals
    CreatureGroup {
        types: &[
            "Server.Mobiles.BloodElemental",
            "Server.Mobiles.EarthElemental",
            "Server.Mobiles.SummonedEarthElemental",
            "Server.Mobiles.AgapiteElemental",
            "Server.Mobiles.BronzeElemental",
            "Server.Mobiles.CopperElemental",
            "Server.Mobiles.DullCopperElemental",
            "Server.Mobiles.GoldenElemental",
            "Server.Mobiles.ShadowIronElemental",
            "Server.Mobiles.ValoriteElemental",
            "Server.Mobiles.VeriteElemental",
            "Server.Mobiles.PoisonElemental",
            "Server.Mobiles.FireElemental",
            "Server.Mobiles.SummonedFireElemental",
            "Server.Mobiles.SnowElemental",
            "Server.Mobiles.AirElemental",
            "Server.Mobiles.SummonedAirElemental",
            "Server.Mobiles.WaterElemental",
            "Server.Mobiles.AcidElemental",
        ],
        entries: &[
            SummonEntry(5000, &["Server.Mobiles.WailingBanshee"]),
            SummonEntry(0, &["Server.Mobiles.Wraith"]),
        ],
    },
    // Dragons
    CreatureGroup {
        types: &[
            "Server.Mobiles.AncientWyrm",
            "Server.Mobiles.Dragon",
            "Server.Mobiles.GreaterDragon",
            "Server.Mobiles.SerpentineDragon",
            "Server.Mobiles.ShadowWyrm",
            "Server.Mobiles.SkeletalDragon",
            "Server.Mobiles.WhiteWyrm",
            "Server.Mobiles.Drake",
            "Server.Mobiles.Wyvern",
            "Server.Mobiles.LesserHiryu",
            "Server.Mobiles.Hiryu",
        ],
        entries: dragons_entries(),
    },
    // Default group
    CreatureGroup {
        types: &[],
        entries: &[
            SummonEntry(18000, &["Server.Mobiles.LichLord"]),
            SummonEntry(10000, &["Server.Mobiles.FleshGolem"]),
            SummonEntry(5000, &["Server.Mobiles.Lich"]),
            SummonEntry(
                3000,
                &["Server.Mobiles.SkeletalKnight", "Server.Mobiles.BoneKnight"],
            ),
            SummonEntry(2000, &["Server.Mobiles.Mummy"]),
            SummonEntry(
                1000,
                &["Server.Mobiles.SkeletalMage", "Server.Mobiles.BoneMagi"],
            ),
            SummonEntry(0, &["Server.Mobiles.PatchworkSkeleton"]),
        ],
    },
];

#[derive(Default)]
struct Table(IndexMap<MobileId, Vec<MobileId>>);

fn table(w: &mut World) -> &mut IndexMap<MobileId, Vec<MobileId>> {
    &mut w.resources.get_mut::<Table>().0
}

/// The `PlayerDeletedEvent` handler.
pub fn init(w: &mut World) {
    muo_content::player_mobile::subscribe_deleted(w, remove_effects);
    // `CreatureData` death/delete call these through `creature::death`: Unregister(master, summoned).
}

fn target(w: &mut World, sp: SpellId, targeted: Targeted) {
    let Some(item) = item_of(targeted) else {
        return;
    };
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };

    if let Some(f) = hooks(w).and_then(|h| h.animate_maabus_coffin)
        && f(w, item, caster)
    {
        return;
    }

    if !item_is(w, item, "Server.Items.Corpse") {
        mm::send_localized_message(w, caster, 1061084, "", 0x3B2); // You cannot animate that.
    } else {
        let c = item;
        let owner = corpse::owner(w, c);
        let item_id = w.item(c).map_or(0, |i| i.item_id);
        let owner_fame = owner.and_then(|o| w.mobile(o)).map_or(0, |o| o.fame);
        let owner_is_pm = owner.is_some_and(|o| crate::common::is_player_mobile(w, o));
        let summoned_or_bonded = owner.is_some_and(|o| {
            creature::is_creature(w, o)
                && (creature::cd(w, o).is_some_and(|d| d.summoned)
                    || creature::cd(w, o).is_some_and(|d| d.is_bonded))
        });

        if item_id != 0x2006
            || corpse::animated(w, c)
            || owner_is_pm
            || owner.is_none()
            || owner_fame < 100
            || summoned_or_bonded
        {
            mm::send_localized_message(w, caster, 1061085, "", 0x3B2); // There's not enough life force there to animate.
        } else if let Some(owner) = owner
            && let Some(group) = find_group(w, owner)
        {
            let demon_knight = mobile_is_exact(w, owner, "Server.Mobiles.DemonKnight");
            if group.entries.is_empty() || demon_knight {
                mm::send_localized_message(w, caster, 1061086, "", 0x3B2); // You cannot animate undead remains.
            } else if spell::check_sequence(w, sp) {
                let p = muo_server::item::ops::get_world_location(w, c);
                let map = w.item(c).and_then(|i| i.map);
                if let Some(map) = map {
                    muo_server::effects::play_sound(w, p, Some(map), 0x1FB);
                    let e = muo_server::item::effect_item::create_targeted(
                        w,
                        p,
                        Some(map),
                        muo_server::item::effect_item::DEFAULT_DURATION,
                    );
                    muo_server::effects::send_location_particles_full(
                        w, e, 0x3789, 1, 40, 0x3F, 3, 9907, 0,
                    );

                    let gi = GROUPS
                        .iter()
                        .position(|g| std::ptr::eq(g, group))
                        .unwrap_or(GROUPS.len() - 1);
                    timer::start(w, TimeSpan::from_secs(2), TimeSpan::ZERO, 1, move |w| {
                        summon_delay_callback(w, caster, c, p, map, gi)
                    });
                }
            }
        }
    }

    spell::finish_sequence(w, sp);
}

fn mobile_is_exact(w: &World, m: MobileId, name: &str) -> bool {
    w.mobile(m).is_some_and(|x| x.ty.full_name == name)
}

fn on_cast(w: &mut World, sp: SpellId) {
    begin_spell_target(
        w,
        sp,
        false,
        TargetFlags::NONE,
        false,
        SpellTargetKind::Item,
        target,
    );
    if let Some(caster) = spell::caster(w, sp) {
        mm::send_localized_message(w, caster, 1061083, "", 0x3B2); // Animate what corpse?
    }
}

/// `RemoveEffects(caster)`.
pub fn remove_effects(w: &mut World, caster: MobileId) {
    if let Some(list) = table(w).shift_remove(&caster) {
        for m in list {
            if let Some(ty) = w.mobile(m).map(|x| x.ty) {
                (ty.delete)(w, m);
            }
        }
    }
}

fn find_group(w: &World, owner: MobileId) -> Option<&'static CreatureGroup> {
    for (i, group) in GROUPS.iter().enumerate() {
        // The first group takes the types of the Silver slayer entry; the last has none and takes everything.
        let contains = if i == 0 {
            slayer::entry_by_name(w, SlayerName::Silver)
                .is_some_and(|(e, _)| e.types.iter().any(|t| mobile_is(w, owner, t)))
        } else {
            group.types.is_empty() || group.types.iter().any(|t| mobile_is(w, owner, t))
        };
        if contains {
            return Some(group);
        }
    }
    None
}

/// `Unregister(master, summoned)`.
pub fn unregister(w: &mut World, master: Option<MobileId>, summoned: MobileId) {
    let Some(master) = master else { return };
    let Some(list) = table(w).get_mut(&master) else {
        return;
    };
    if let Some(i) = list.iter().position(|&m| m == summoned) {
        list.remove(i);
        if list.is_empty() {
            table(w).shift_remove(&master);
        }
    }
}

/// `Register(master, summoned)`.
pub fn register(w: &mut World, master: Option<MobileId>, summoned: MobileId) {
    let Some(master) = master else { return };

    let current: Vec<MobileId> = table(w).get(&master).cloned().unwrap_or_default();
    let mut alive_list: Vec<MobileId> = current
        .into_iter()
        .filter(|&m| w.mobile(m).is_some_and(|x| !x.deleted))
        .collect();
    alive_list.push(summoned);
    let (len, first) = (alive_list.len(), alive_list[0]);
    table(w).insert(master, alive_list);

    if len > 3 {
        let to_kill = first;
        unregister(w, Some(master), to_kill);
        mb::kill(w, to_kill);
    }

    timer::start(
        w,
        TimeSpan::from_millis(1650),
        TimeSpan::from_millis(1650),
        0,
        move |w| summoned_damage(w, summoned),
    );
}

fn summoned_damage(w: &mut World, mob: MobileId) {
    let hits = w.mobile(mob).map_or(0, |x| x.hits);
    if hits > 0 {
        props_b::set_hits(w, mob, hits - 1);
    } else {
        mb::kill(w, mob);
    }
}

fn summon_delay_callback(
    w: &mut World,
    caster: MobileId,
    corpse_id: ItemId,
    loc: muo_primitives::Point3D,
    map: MapId,
    group: usize,
) {
    if corpse::animated(w, corpse_id)
        || muo_server::item::ops::is_deleted(w, corpse_id)
        || w.mobile(caster).is_none_or(|c| c.deleted)
    {
        return;
    }

    let Some(owner) = corpse::owner(w, corpse_id) else {
        return;
    };

    let necromancy = skills::value(w, caster, SkillName::Necromancy);
    let spirit_speak = skills::value(w, caster, SkillName::SpiritSpeak);

    let mut caster_ability = (necromancy * 30.0) as i32 + (spirit_speak * 70.0) as i32;
    let fame = w.mobile(owner).map_or(0, |o| o.fame);
    caster_ability = (caster_ability / 10 * 18).clamp(0, fame.max(0));

    let mut to_summon: Option<&'static str> = None;
    for entry in GROUPS[group].entries {
        if to_summon.is_some() {
            break;
        }
        if caster_ability < entry.0 {
            continue;
        }
        let i = utility::random(&mut w.rng, entry.1.len() as i32) as usize;
        to_summon = Some(entry.1[i.min(entry.1.len() - 1)]);
    }

    let Some(to_summon) = to_summon else {
        return;
    };

    let Some(summoned) = factory::make_mobile(w, to_summon) else {
        return;
    };

    if creature::is_creature(w, summoned) {
        // to be sure
        creature::edit(w, summoned, |d| d.tamable_raw = false);
        let mount = mobile_is(w, summoned, "Server.Mobiles.BaseMount");
        creature::edit(w, summoned, |d| d.control_slots = i32::from(mount));
        let snd = w
            .mobile(summoned)
            .map(|x| x.ty)
            .map_or(0, |x| (x.get_anger_sound)(w, summoned));
        muo_server::effects::play_sound(w, loc, Some(map), snd);
        control::summon(
            w,
            summoned,
            false,
            caster,
            loc,
            0x28,
            TimeSpan::from_days(1),
            None,
        );
    }

    if mobile_is(w, summoned, "Server.Mobiles.SkeletalDragon") {
        scale(w, summoned, 50); // lose 50% hp and strength
    }

    muo_server::mobile::part1::props_a::set_fame(w, summoned, 0);
    muo_server::mobile::part1::props_a::set_karma(w, summoned, -1500);
    mb::move_to_world(w, summoned, loc, map);

    muo_server::item::base::set_hue(w, corpse_id, 1109);
    corpse::set_flag(w, corpse_id, corpse::flag::ANIMATED, true);

    register(w, Some(caster), summoned);
}

/// `Scale(bc, scalar)`.
pub fn scale(w: &mut World, bc: MobileId, scalar: i32) {
    let to_scale = w.mobile(bc).map_or(0, |x| x.str);
    props_b::set_raw_str(w, bc, aos_helpers::scale(to_scale, scalar));

    let seed = creature::cd(w, bc).map_or(0, |d| d.hits_max_seed);
    if seed > 0 {
        creature::edit(w, bc, |d| {
            d.hits_max_seed = aos_helpers::scale(seed, scalar)
        });
    }

    let hits = w.mobile(bc).map_or(0, |x| x.hits);
    props_b::set_hits(w, bc, hits); // refresh hits
}
