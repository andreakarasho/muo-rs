//! The Arcane Circle, Attune Weapon, Essence of Wind, Gift of Life, Gift of Renewal, Immolating Weapon,
//! Thunderstorm and Word of Death spells.

use indexmap::IndexMap;
use muo_content::creature;
use muo_content::items::is_named;
use muo_content::items::weapon::BASE_RANGED;
use muo_content::items::weapon_hooks::{self, WeaponHooks};
use muo_content::misc::aos_helpers::{self, DamageOptions, DamageSplit};
use muo_content::player_mobile::buff::{BuffIcon, BuffInfo, add_buff, remove_buff};
use muo_content::spells::SpellInfo;
use muo_content::spells::helper::{self, SpellDamageSplit};
use muo_content::spells::hooks::hooks_mut;
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_content::spells::targeting::{SpellTargetKind, begin_spell_target};
use muo_gumps::m_z::resurrect;
use muo_io::text::TextDefinition;
use muo_primitives::{Point3D, TimeSpan, utility};
use muo_server::World;
use muo_server::effects::{EffectLayer, Targeted as FxTargeted, send_moving_particles_layer};
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::{deps, ops};
use muo_server::mobile::base as mb;
use muo_server::mobile::part1::combat;
use muo_server::mobile::part2::{effects, messages as mm};
use muo_server::skills::{self, SkillName};
use muo_server::target::{TargetFlags, Targeted};
use muo_server::timer::{self, TimerHandle};

use super::arcanist::{
    ARCANE_BASE, ARCANIST_SPELL, ArcaneType, check_resisted, find_arcane_focus, focus_level,
};
use super::items::{self, new_arcane_focus};
use crate::common::{is_player_mobile, mobile_of, mobiles_in_range, turn_to};

fn begin_mobile(
    w: &mut World,
    sp: SpellId,
    flags: TargetFlags,
    f: fn(&mut World, SpellId, Targeted),
) {
    begin_spell_target(w, sp, false, flags, false, SpellTargetKind::Mobile, f);
}

// ---- ArcaneCircleSpell ----

static ARCANE_CIRCLE: ArcaneType = ArcaneType {
    required_skill: 0.0,
    required_mana: 24,
    ..ARCANE_BASE
};

/// `ArcaneCircleSpell`: Myrshalee.
pub static ARCANE_CIRCLE_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Spellweaving.ArcaneCircleSpell",
    parent: Some(&ARCANIST_SPELL),
    info: SpellInfo::new("Arcane Circle", "Myrshalee", &[]).action(-1),
    cast_delay_base: |_, _| TimeSpan::from_millis(500),
    check_cast: circle_check_cast,
    on_cast: circle_on_cast,
    family: Some(&ARCANE_CIRCLE),
    ..ARCANIST_SPELL
};

fn circle_check_cast(w: &mut World, sp: SpellId) -> bool {
    let Some(caster) = spell::caster(w, sp) else {
        return false;
    };
    let (loc, map) = w
        .mobile(caster)
        .map(|c| (c.location, c.map))
        .unwrap_or_default();
    if !is_valid_location(w, loc, map) {
        // You must be standing on an arcane circle, pentagram or abattoir to use this spell.
        mm::send_localized_message(w, caster, 1072705, "", 0x3B2);
        return false;
    }

    if get_arcanists(w, caster).is_empty() {
        mm::send_localized_message(w, caster, 1080452, "", 0x3B2); // There are not enough spellweavers present to create an Arcane Focus.
        return false;
    }

    (ARCANIST_SPELL.check_cast)(w, sp)
}

fn circle_on_cast(w: &mut World, sp: SpellId) {
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };
    if spell::check_sequence(w, sp) {
        effects::fixed_particles(w, caster, 0x3779, 10, 20, 0x0, EffectLayer::Waist);
        effects::play_sound(w, caster, 0x5C0);

        let spell_weaving = skills::value(w, caster, SkillName::Spellweaving);
        let pool = get_arcanists(w, caster);

        let duration = TimeSpan::from_hours(1.max((spell_weaving / 24.0) as i64));

        let (loc, map) = w
            .mobile(caster)
            .map(|c| (c.location, c.map))
            .unwrap_or_default();
        let strength_bonus = (pool.len() as i32).min(if is_sanctuary(loc, map) { 6 } else { 5 });

        for m in pool {
            give_arcane_focus(w, m, duration, strength_bonus);
        }
    }

    spell::finish_sequence(w, sp);
}

fn is_sanctuary(p: Point3D, m: Option<muo_server::map::MapId>) -> bool {
    (m == Some(muo_server::map::MapId::TRAMMEL) || m == Some(muo_server::map::MapId::FELUCCA))
        && p.x == 6267
        && p.y == 131
}

fn is_valid_location(
    w: &mut World,
    location: Point3D,
    map: Option<muo_server::map::MapId>,
) -> bool {
    let Some(map) = map else { return false };

    // Land tiles
    let lt = w.maps.land_tile(map, location.x, location.y);
    if is_valid_tile(i32::from(lt.id)) && i32::from(lt.z) == location.z {
        return true;
    }

    // Static tiles
    for t in muo_server::map::tiles::static_tiles(w, map, location.x, location.y) {
        let calc_height = w
            .maps
            .tile_data
            .item((t.id & 0x3FFF) as usize)
            .calc_height();
        if i32::from(t.z) + calc_height != location.z {
            continue;
        }

        if is_valid_tile(i32::from(t.id)) {
            return true;
        }
    }

    // Items
    for item in muo_server::map::enumerate::items_at(w, Some(map), location.x, location.y) {
        let Some(i) = w.item(item) else { continue };
        let calc_height = w
            .maps
            .tile_data
            .item((i.item_id & 0x3FFF) as usize)
            .calc_height();
        if i.location.z + calc_height == location.z && is_valid_tile(i.item_id) {
            return true;
        }
    }

    false
}

/// `IsValidTile(itemID)`.
pub fn is_valid_tile(item_id: i32) -> bool {
    matches!(
        item_id,
        0xFEA | 0x1216 | 0x307F | 0x1D10 | 0x1D0F | 0x1D1F | 0x1D12
    )
}

/// `GetArcanists(spellWeaving)` / `CheckArcanists()`: the players within 1 tile whose Spellweaving is within 20.
fn get_arcanists(w: &mut World, caster: MobileId) -> Vec<MobileId> {
    // OSI Verified: Even enemies/combatants count
    // Everyone gets the Arcane Focus, power capped elsewhere
    let spell_weaving = skills::value(w, caster, SkillName::Spellweaving);
    let (loc, map) = w
        .mobile(caster)
        .map(|c| (c.location, c.map))
        .unwrap_or_default();
    let mut pool = Vec::new();
    for m in mobiles_in_range(w, map, loc, 1) {
        if m != caster
            && is_player_mobile(w, m)
            && mb::can_be_beneficial_msg(w, caster, m, false)
            && (spell_weaving - skills::value(w, m, SkillName::Spellweaving)).abs() <= 20.0
        {
            pool.push(m);
        }
    }
    pool
}

fn give_arcane_focus(w: &mut World, to: MobileId, duration: TimeSpan, strength_bonus: i32) {
    match find_arcane_focus(w, to) {
        None => {
            let focus = new_arcane_focus(w, duration, strength_bonus);
            if muo_server::mobile::part2::items::place_in_backpack(w, to, focus) {
                items::send_time_remaining_message(w, focus, to);
                mm::send_localized_message(w, to, 1072740, "", 0x3B2); // An arcane focus appears in your backpack.
            } else {
                muo_server::item::base::delete(w, focus);
            }
        }
        Some(focus) => {
            // OSI renewal rules: the new one will override the old one, always.
            mm::send_localized_message(w, to, 1072828, "", 0x3B2); // Your arcane focus is renewed.
            items::reset_expiration(w, focus, duration);
            items::set_strength_bonus(w, focus, strength_bonus);
            ops::invalidate_properties(w, focus);
            items::send_time_remaining_message(w, focus, to);
        }
    }
}

// ---- AttuneWeaponSpell ----

static ATTUNE_WEAPON: ArcaneType = ArcaneType {
    required_skill: 0.0,
    required_mana: 24,
    ..ARCANE_BASE
};

/// `AttuneWeaponSpell`: Haeldril.
pub static ATTUNE_WEAPON_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Spellweaving.AttuneWeaponSpell",
    parent: Some(&ARCANIST_SPELL),
    info: SpellInfo::new("Attune Weapon", "Haeldril", &[]).action(-1),
    cast_delay_base: |_, _| TimeSpan::from_secs(1),
    check_cast: attune_check_cast,
    on_cast: attune_on_cast,
    family: Some(&ATTUNE_WEAPON),
    ..ARCANIST_SPELL
};

const ATTUNE_KEY: &str = "Server.Spells.Spellweaving.AttuneWeaponSpell";

#[derive(Default)]
struct AttuneTable(IndexMap<MobileId, TimerHandle>);

fn attune_table(w: &mut World) -> &mut IndexMap<MobileId, TimerHandle> {
    &mut w.resources.get_mut::<AttuneTable>().0
}

fn attune_check_cast(w: &mut World, sp: SpellId) -> bool {
    let Some(caster) = spell::caster(w, sp) else {
        return false;
    };
    if attune_table(w).contains_key(&caster) {
        mm::send_localized_message(w, caster, 501775, "", 0x3B2); // This spell is already in effect.
        return false;
    }

    if combat::can_begin_action(w, caster, ATTUNE_KEY) {
        return (ARCANIST_SPELL.check_cast)(w, sp);
    }

    mm::send_localized_message(w, caster, 1075124, "", 0x3B2); // You must wait before casting that spell again.
    false
}

fn attune_on_cast(w: &mut World, sp: SpellId) {
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };
    if spell::check_sequence(w, sp) {
        effects::play_sound(w, caster, 0x5C3);
        effects::fixed_particles_hue(
            w,
            caster,
            0x3728,
            1,
            13,
            0x26B8,
            0x455,
            7,
            EffectLayer::Waist,
        );
        effects::fixed_particles_hue(
            w,
            caster,
            0x3779,
            1,
            15,
            0x251E,
            0x3F,
            7,
            EffectLayer::Waist,
        );

        let skill = skills::value(w, caster, SkillName::Spellweaving);
        let focus = focus_level(w, sp);

        let damage_absorb = (18.0 + (skill - 10.0) / 10.0 * 3.0 + f64::from(focus) * 6.0) as i32;
        if let Some(c) = w.mobile_mut(caster) {
            c.melee_damage_absorb = damage_absorb;
        }

        let duration = TimeSpan::from_secs(60 + i64::from(focus) * 12);

        let h = timer::start(w, duration, TimeSpan::ZERO, 1, move |w| {
            do_expire(w, caster, true)
        });
        attune_table(w).insert(caster, h);

        combat::begin_action(w, caster, ATTUNE_KEY);

        if is_player_mobile(w, caster) {
            add_buff(
                w,
                caster,
                BuffInfo::new(BuffIcon::AttuneWeapon, 1075798)
                    .duration(duration)
                    .args(TextDefinition::of_text(damage_absorb.to_string())),
            );
        }
    }

    spell::finish_sequence(w, sp);
}

/// `TryAbsorb(defender, ref damage)`: the damage left.
pub fn try_absorb(w: &mut World, defender: MobileId, damage: i32) -> i32 {
    let absorb = w.mobile(defender).map_or(0, |d| d.melee_damage_absorb);
    if damage == 0 || !is_absorbing(w, defender) || absorb <= 0 {
        return damage;
    }

    let absorbed = damage.min(absorb);
    let left = damage - absorbed;

    let remaining = absorb - absorbed;
    if let Some(d) = w.mobile_mut(defender) {
        d.melee_damage_absorb = remaining;
    }

    // ~1_damage~ point(s) of damage have been absorbed. A total of ~2_remaining~ point(s) of shielding remain.
    mm::send_localized_message(
        w,
        defender,
        1075127,
        &format!("{absorbed}\t{remaining}"),
        0x3B2,
    );

    if remaining <= 0 {
        stop_absorbing(w, defender, true);
    }

    left
}

/// `IsAbsorbing(m)`.
pub fn is_absorbing(w: &mut World, m: MobileId) -> bool {
    attune_table(w).contains_key(&m)
}

/// `StopAbsorbing(m, message)`.
pub fn stop_absorbing(w: &mut World, m: MobileId, message: bool) {
    if attune_table(w).contains_key(&m) {
        do_expire(w, m, message);
    }
}

fn do_expire(w: &mut World, m: MobileId, message: bool) {
    if let Some(mut t) = attune_table(w).shift_remove(&m) {
        t.cancel(w);
    }
    if let Some(x) = w.mobile_mut(m) {
        x.melee_damage_absorb = 0;
    }

    if message {
        mm::send_localized_message(w, m, 1075126, "", 0x3B2); // Your attunement fades.
        effects::play_sound(w, m, 0x1F8);
    }

    timer::start(w, TimeSpan::from_secs(120), TimeSpan::ZERO, 1, move |w| {
        combat::end_action(w, m, ATTUNE_KEY)
    });

    if is_player_mobile(w, m) {
        remove_buff(w, m, BuffIcon::AttuneWeapon);
    }
}

// ---- EssenceOfWindSpell ----

static ESSENCE_OF_WIND: ArcaneType = ArcaneType {
    required_skill: 52.0,
    required_mana: 40,
    ..ARCANE_BASE
};

/// `EssenceOfWindSpell`: Anathrae.
pub static ESSENCE_OF_WIND_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Spellweaving.EssenceOfWindSpell",
    parent: Some(&ARCANIST_SPELL),
    info: SpellInfo::new("Essence of Wind", "Anathrae", &[]).action(-1),
    cast_delay_base: |_, _| TimeSpan::from_secs(3),
    on_cast: wind_on_cast,
    family: Some(&ESSENCE_OF_WIND),
    ..ARCANIST_SPELL
};

struct Wind {
    timer: TimerHandle,
    fc_malus: i32,
    ssi_malus: i32,
}

#[derive(Default)]
struct WindTable(IndexMap<MobileId, Wind>);

fn wind_table(w: &mut World) -> &mut IndexMap<MobileId, Wind> {
    &mut w.resources.get_mut::<WindTable>().0
}

fn wind_ref(w: &World, m: MobileId) -> Option<&Wind> {
    w.resources.get::<WindTable>()?.0.get(&m)
}

fn wind_on_cast(w: &mut World, sp: SpellId) {
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };
    if spell::check_sequence(w, sp) {
        effects::play_sound(w, caster, 0x5C6);

        let focus = focus_level(w, sp);
        let damage = 25 + focus;

        let skill = skills::value(w, caster, SkillName::Spellweaving);
        let duration = TimeSpan::from_secs((skill / 24.0) as i64 + i64::from(focus));

        let fc_malus = focus + 1;
        let ssi_malus = 2 * (focus + 1);

        let (loc, map) = w
            .mobile(caster)
            .map(|c| (c.location, c.map))
            .unwrap_or_default();
        let mut queue = Vec::new();
        for m in mobiles_in_range(w, map, loc, 5 + focus) {
            if caster == m
                || !deps::in_los_mobile(w, caster, m)
                || !helper::valid_indirect_target(w, caster, m)
                || !mb::can_be_harmful_msg(w, caster, m, false)
            {
                continue;
            }

            queue.push(m);
        }

        for m in queue {
            mb::do_harmful(w, caster, m, false);

            helper::damage_aos_spell(
                w,
                sp,
                m,
                f64::from(damage),
                SpellDamageSplit {
                    cold: 100,
                    ..Default::default()
                },
            );

            if check_resisted(w, sp, m) {
                continue;
            }

            let h = timer::start(w, duration, TimeSpan::ZERO, 1, move |w| wind_expire(w, m));
            wind_table(w).insert(
                m,
                Wind {
                    timer: h,
                    fc_malus,
                    ssi_malus,
                },
            );

            if is_player_mobile(w, m) {
                add_buff(
                    w,
                    m,
                    BuffInfo::new(BuffIcon::EssenceOfWind, 1075802)
                        .duration(duration)
                        .args(TextDefinition::of_text(format!("{fc_malus}\t{ssi_malus}"))),
                );
            }
        }
    }

    spell::finish_sequence(w, sp);
}

/// `IsDebuffed(m)`.
pub fn is_debuffed(w: &World, m: MobileId) -> bool {
    wind_ref(w, m).is_some()
}

/// `GetFCMalus(m)`.
pub fn get_fc_malus(w: &World, m: MobileId) -> i32 {
    wind_ref(w, m).map_or(0, |t| t.fc_malus)
}

/// `GetSSIMalus(m)`.
pub fn get_ssi_malus(w: &World, m: MobileId) -> i32 {
    wind_ref(w, m).map_or(0, |t| t.ssi_malus)
}

/// `StopDebuffing(m)`.
pub fn stop_debuffing(w: &mut World, m: MobileId) {
    if wind_table(w).contains_key(&m) {
        wind_expire(w, m);
    }
}

fn wind_expire(w: &mut World, m: MobileId) {
    if let Some(mut t) = wind_table(w).shift_remove(&m) {
        t.timer.cancel(w);
    }
    if is_player_mobile(w, m) {
        remove_buff(w, m, BuffIcon::EssenceOfWind);
    }
}

// ---- GiftOfLifeSpell ----

static GIFT_OF_LIFE: ArcaneType = ArcaneType {
    required_skill: 38.0,
    required_mana: 70,
    ..ARCANE_BASE
};

/// `GiftOfLifeSpell`: Illorae.
pub static GIFT_OF_LIFE_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Spellweaving.GiftOfLifeSpell",
    parent: Some(&ARCANIST_SPELL),
    target_range: Some(muo_content::spells::targeting::target_range),
    info: SpellInfo::new("Gift of Life", "Illorae", &[]).action(-1),
    cast_delay_base: |_, _| TimeSpan::from_secs(4),
    on_cast: |w, sp| begin_mobile(w, sp, TargetFlags::BENEFICIAL, life_target),
    family: Some(&GIFT_OF_LIFE),
    ..ARCANIST_SPELL
};

struct Life {
    timer: TimerHandle,
    /// `Spell.HitsScalar`, read when the spell was cast.
    hits_scalar: f64,
}

#[derive(Default)]
struct LifeTable(IndexMap<MobileId, Life>);

fn life_table(w: &mut World) -> &mut IndexMap<MobileId, Life> {
    &mut w.resources.get_mut::<LifeTable>().0
}

fn life_target(w: &mut World, sp: SpellId, targeted: Targeted) {
    let Some(m) = mobile_of(targeted) else {
        return;
    };
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };
    let dead = mb::is_dead_bonded_pet(w, m) || !super::super::common::alive(w, m);
    let bonded_pet_of_caster =
        creature::cd(w, m).is_some_and(|d| d.is_bonded && d.master == Some(caster));

    if dead {
        // As per Osi: Nothing happens.
    } else if m != caster && !bonded_pet_of_caster {
        mm::send_localized_message(w, caster, 1072077, "", 0x3B2); // You may only cast this spell on yourself or a bonded pet.
    } else if life_table(w).contains_key(&m) {
        mm::send_localized_message(w, caster, 501775, "", 0x3B2); // This spell is already in effect.
    } else if spell::check_b_sequence(w, sp, m, false) {
        if caster == m {
            mm::send_localized_message(w, caster, 1074774, "", 0x3B2); // You weave powerful magic, protecting yourself from death.
        } else {
            mm::send_localized_message(w, caster, 1074775, "", 0x3B2); // You weave powerful magic, protecting your pet from death.
            turn_to(w, caster, m);
        }

        effects::play_sound(w, m, 0x244);
        effects::fixed_particles_hue(w, m, 0x3709, 1, 30, 0x26ED, 5, 2, EffectLayer::Waist);
        effects::fixed_particles_hue(w, m, 0x376A, 1, 30, 0x251E, 5, 3, EffectLayer::Waist);

        let skill = skills::value(w, caster, SkillName::Spellweaving);
        let focus = focus_level(w, sp);
        let duration = TimeSpan::from_mins(i64::from((skill / 24.0) as i32 * 2 + focus));
        let hits_scalar = (skill / 2.4 + f64::from(focus)) / 100.0;

        let h = timer::start(w, duration, TimeSpan::ZERO, 1, move |w| life_expire(w, m));
        life_table(w).insert(
            m,
            Life {
                timer: h,
                hits_scalar,
            },
        );

        if is_player_mobile(w, m) {
            add_buff(
                w,
                m,
                BuffInfo::with_secondary(BuffIcon::GiftOfLife, 1031615, 1075807)
                    .duration(duration)
                    .retain_through_death(),
            );
        }
    }
}

/// The death handlers (`CreatureDeathEvent` and `PlayerDeathEvent`).
fn life_on_death(w: &mut World, m: MobileId) {
    if life_table(w).contains_key(&m) {
        let secs = utility::random_min_max(&mut w.rng, 2, 4);
        timer::start(
            w,
            TimeSpan::from_secs(i64::from(secs)),
            TimeSpan::ZERO,
            1,
            move |w| handle_death_callback(w, m),
        );
    }
}

fn handle_death_callback(w: &mut World, m: MobileId) {
    let Some(hits_scalar) = life_table(w).get(&m).map(|l| l.hits_scalar) else {
        return;
    };

    if mb::is_dead_bonded_pet(w, m) && creature::is_creature(w, m) {
        // The pet resurrection gump goes to its master, else to a friend, through the content hook.
        if let Some(f) = muo_content::spells::hooks::hooks(w).and_then(|h| h.pet_resurrect_gump) {
            f(w, m, hits_scalar);
        }
    } else {
        let gump = Box::new(resurrect::new_with_scalar(Some(m), hits_scalar));
        muo_content::gumps::system::send_gump_to_mobile(w, m, gump, false);
    }

    // Per OSI, buff is removed when gump sent, irregardless of online status or acceptance
    life_expire(w, m);
}

fn life_expire(w: &mut World, m: MobileId) {
    if let Some(mut l) = life_table(w).shift_remove(&m) {
        l.timer.cancel(w);
    }
    mm::send_localized_message(w, m, 1074776, "", 0x3B2); // You are no longer protected with Gift of Life.
    if is_player_mobile(w, m) {
        remove_buff(w, m, BuffIcon::GiftOfLife);
    }
}

// ---- GiftOfRenewalSpell ----

static GIFT_OF_RENEWAL: ArcaneType = ArcaneType {
    required_skill: 0.0,
    required_mana: 24,
    ..ARCANE_BASE
};

/// `GiftOfRenewalSpell`: Olorisstra.
pub static GIFT_OF_RENEWAL_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Spellweaving.GiftOfRenewalSpell",
    parent: Some(&ARCANIST_SPELL),
    target_range: Some(muo_content::spells::targeting::target_range),
    info: SpellInfo::new("Gift of Renewal", "Olorisstra", &[]).action(-1),
    cast_delay_base: |_, _| TimeSpan::from_secs(3),
    on_cast: |w, sp| begin_mobile(w, sp, TargetFlags::BENEFICIAL, renewal_target),
    family: Some(&GIFT_OF_RENEWAL),
    ..ARCANIST_SPELL
};

const RENEWAL_KEY: &str = "Server.Spells.Spellweaving.GiftOfRenewalSpell";

struct Renewal {
    timer: TimerHandle,
    caster: MobileId,
}

#[derive(Default)]
struct RenewalTable(IndexMap<MobileId, Renewal>);

fn renewal_table(w: &mut World) -> &mut IndexMap<MobileId, Renewal> {
    &mut w.resources.get_mut::<RenewalTable>().0
}

fn renewal_target(w: &mut World, sp: SpellId, targeted: Targeted) {
    let Some(m) = mobile_of(targeted) else {
        return;
    };
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };
    if renewal_table(w).contains_key(&m) {
        mm::send_localized_message(w, caster, 501775, "", 0x3B2); // This spell is already in effect.
    } else if !combat::can_begin_action(w, caster, RENEWAL_KEY) {
        mm::send_localized_message(w, caster, 501789, "", 0x3B2); // You must wait before trying again.
    } else if spell::check_b_sequence(w, sp, m, false) {
        turn_to(w, caster, m);

        effects::fixed_effect(w, caster, 0x374A, 10, 20);
        effects::play_sound(w, caster, 0x5C9);

        if w.mobile(m).is_some_and(|x| x.poison.is_some()) {
            mb::cure_poison(w, m, Some(m));
        } else {
            let skill = skills::value(w, caster, SkillName::Spellweaving);
            let focus = focus_level(w, sp);
            let hits_per_round = 5 + (skill / 24.0) as i32 + focus;
            let duration = 30 + focus * 10;

            let mut ticks = 0;
            let count = duration / 2;
            let h = timer::start(
                w,
                TimeSpan::from_secs(2),
                TimeSpan::from_secs(2),
                count,
                move |w| {
                    ticks += 1;
                    renewal_tick(w, caster, m, hits_per_round, ticks >= count);
                },
            );
            renewal_table(w).insert(m, Renewal { timer: h, caster });

            combat::begin_action(w, caster, RENEWAL_KEY);

            if is_player_mobile(w, m) {
                add_buff(
                    w,
                    m,
                    BuffInfo::with_secondary(BuffIcon::GiftOfRenewal, 1031602, 1075797)
                        .duration(TimeSpan::from_secs(i64::from(duration)))
                        .args(TextDefinition::of_text(hits_per_round.to_string())),
                );
            }
        }
    }
}

fn renewal_tick(w: &mut World, caster: MobileId, m: MobileId, hits_per_round: i32, last: bool) {
    // Last tick will change running to false
    if last {
        stop_effect(w, m);
        effects::play_sound(w, m, 0x455);
        mm::send_localized_message(w, m, 1075071, "", 0x3B2); // The Gift of Renewal has faded.
        return;
    }

    if !renewal_table(w).contains_key(&m) {
        return;
    }

    if !super::super::common::alive(w, m) {
        stop_effect(w, m);
        return;
    }

    let (hits, hits_max) = (w.mobile(m).map_or(0, |x| x.hits), mb::hits_max(w, m));
    if hits >= hits_max {
        return;
    }

    helper::heal(w, hits_per_round, m, Some(caster), true);
    effects::fixed_particles(w, m, 0x376A, 9, 32, 5005, EffectLayer::Waist);
}

/// `StopEffect(m)`.
pub fn stop_effect(w: &mut World, m: MobileId) -> bool {
    if is_player_mobile(w, m) {
        remove_buff(w, m, BuffIcon::GiftOfRenewal);
    }

    if let Some(mut r) = renewal_table(w).shift_remove(&m) {
        r.timer.cancel(w);
        let caster = r.caster;
        timer::start(w, TimeSpan::from_secs(60), TimeSpan::ZERO, 1, move |w| {
            combat::end_action(w, caster, RENEWAL_KEY)
        });
        return true;
    }

    false
}

// ---- ImmolatingWeaponSpell ----

static IMMOLATING_WEAPON: ArcaneType = ArcaneType {
    required_skill: 10.0,
    required_mana: 32,
    ..ARCANE_BASE
};

/// `ImmolatingWeaponSpell`: Thalshara.
pub static IMMOLATING_WEAPON_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Spellweaving.ImmolatingWeaponSpell",
    parent: Some(&ARCANIST_SPELL),
    info: SpellInfo::new("Immolating Weapon", "Thalshara", &[]).action(-1),
    cast_delay_base: |_, _| TimeSpan::from_secs(1),
    check_cast: immolating_check_cast,
    on_cast: immolating_on_cast,
    family: Some(&IMMOLATING_WEAPON),
    ..ARCANIST_SPELL
};

struct Immolating {
    timer: TimerHandle,
    caster: MobileId,
    damage: i32,
}

#[derive(Default)]
struct ImmolatingTable(IndexMap<ItemId, Immolating>);

fn immolating_table(w: &mut World) -> &mut IndexMap<ItemId, Immolating> {
    &mut w.resources.get_mut::<ImmolatingTable>().0
}

fn melee_weapon(w: &mut World, caster: MobileId) -> Option<ItemId> {
    weapon_hooks::weapon_item(w, caster).filter(|&i| {
        w.item(i)
            .is_some_and(|x| !is_named(x.ty, "Server.Items.Fists") && !x.ty.is(&BASE_RANGED))
    })
}

fn immolating_check_cast(w: &mut World, sp: SpellId) -> bool {
    let Some(caster) = spell::caster(w, sp) else {
        return false;
    };
    if melee_weapon(w, caster).is_none() {
        mm::send_localized_message(w, caster, 1060179, "", 0x3B2); // You must be wielding a weapon to use this ability!
        return false;
    }

    (ARCANIST_SPELL.check_cast)(w, sp)
}

fn immolating_on_cast(w: &mut World, sp: SpellId) {
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };
    let Some(wp) = melee_weapon(w, caster) else {
        mm::send_localized_message(w, caster, 1060179, "", 0x3B2); // You must be wielding a weapon to use this ability!
        spell::finish_sequence(w, sp);
        return;
    };

    if spell::check_sequence(w, sp) {
        effects::play_sound(w, caster, 0x5CA);
        effects::fixed_particles(w, caster, 0x36BD, 20, 10, 5044, EffectLayer::Head);

        if !is_immolating(w, wp) {
            // On OSI, the effect is not re-applied
            let skill = skills::value(w, caster, SkillName::Spellweaving);
            let focus = focus_level(w, sp);
            let duration = 10 + (skill / 24.0) as i64 + i64::from(focus);
            let damage = 5 + (skill / 24.0) as i32 + focus;

            let h = timer::start(
                w,
                TimeSpan::from_secs(duration),
                TimeSpan::ZERO,
                1,
                move |w| stop_immolating(w, wp),
            );
            immolating_table(w).insert(
                wp,
                Immolating {
                    timer: h,
                    caster,
                    damage,
                },
            );

            ops::invalidate_properties(w, wp);
        }
    }

    spell::finish_sequence(w, sp);
}

/// `IsImmolating(weapon)`.
pub fn is_immolating(w: &mut World, wp: ItemId) -> bool {
    immolating_table(w).contains_key(&wp)
}

/// `GetImmolatingDamage(weapon)`.
pub fn get_immolating_damage(w: &mut World, wp: ItemId) -> i32 {
    immolating_table(w).get(&wp).map_or(0, |e| e.damage)
}

/// `DoEffect(weapon, target)`.
pub fn do_effect(w: &mut World, wp: ItemId, target: MobileId) {
    if let Some(mut e) = immolating_table(w).shift_remove(&wp) {
        e.timer.cancel(w);
        let (caster, damage) = (e.caster, e.damage);
        timer::start(w, TimeSpan::from_millis(250), TimeSpan::ZERO, 1, move |w| {
            aos_helpers::damage(
                w,
                target,
                Some(caster),
                damage,
                DamageSplit {
                    fire: 100,
                    ..Default::default()
                },
                DamageOptions::default(),
            );
        });
    }
}

/// `StopImmolating(weapon)`.
pub fn stop_immolating(w: &mut World, wp: ItemId) {
    if let Some(mut e) = immolating_table(w).shift_remove(&wp) {
        effects::play_sound(w, e.caster, 0x27);
        e.timer.cancel(w);
        ops::invalidate_properties(w, wp);
    }
}

// ---- ThunderstormSpell ----

static THUNDERSTORM: ArcaneType = ArcaneType {
    required_skill: 10.0,
    required_mana: 32,
    ..ARCANE_BASE
};

/// `ThunderstormSpell`: Erelonia.
pub static THUNDERSTORM_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Spellweaving.ThunderstormSpell",
    parent: Some(&ARCANIST_SPELL),
    info: SpellInfo::new("Thunderstorm", "Erelonia", &[]).action(-1),
    cast_delay_base: |_, _| TimeSpan::from_millis(1500),
    on_cast: thunder_on_cast,
    family: Some(&THUNDERSTORM),
    ..ARCANIST_SPELL
};

#[derive(Default)]
struct ThunderTable(IndexMap<MobileId, TimerHandle>);

fn thunder_table(w: &mut World) -> &mut IndexMap<MobileId, TimerHandle> {
    &mut w.resources.get_mut::<ThunderTable>().0
}

fn thunder_on_cast(w: &mut World, sp: SpellId) {
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };
    if spell::check_sequence(w, sp) {
        effects::play_sound(w, caster, 0x5CE);

        let skill = skills::value(w, caster, SkillName::Spellweaving);
        let focus = focus_level(w, sp);
        let damage = 11.max(10 + (skill / 24.0) as i32) + focus;

        let sdi_bonus =
            aos_helpers::attribute_value(w, caster, muo_content::aos::AosAttribute::SpellDamage);
        let pvm_damage = damage * (100 + sdi_bonus) / 100;
        let pvp_damage = damage * (100 + sdi_bonus.min(15)) / 100;

        let duration = TimeSpan::from_secs(i64::from(5 + focus));

        let (loc, map) = w
            .mobile(caster)
            .map(|c| (c.location, c.map))
            .unwrap_or_default();
        let mut queue = Vec::new();
        for m in mobiles_in_range(w, map, loc, 2 + focus) {
            if caster == m
                || !helper::valid_indirect_target(w, caster, m)
                || !mb::can_be_harmful_msg(w, caster, m, false)
                || !deps::in_los_mobile(w, caster, m)
            {
                continue;
            }

            queue.push(m);
        }

        for m in queue {
            mb::do_harmful(w, caster, m, false);

            let old_spell = w.mobile(m).and_then(|x| x.spell);
            let pvp =
                w.mobile(m).is_some_and(|x| x.player) && w.mobile(caster).is_some_and(|x| x.player);
            helper::damage_aos_spell(
                w,
                sp,
                m,
                f64::from(if pvp { pvp_damage } else { pvm_damage }),
                SpellDamageSplit {
                    nrgy: 100,
                    ..Default::default()
                },
            );

            let cur = w.mobile(m).and_then(|x| x.spell);
            if old_spell.is_none() || old_spell == cur || check_resisted(w, sp, m) {
                continue;
            }

            thunder_stop_timer(w, m);

            let h = timer::start(w, duration, TimeSpan::ZERO, 1, move |w| {
                do_thunder_expire(w, m)
            });
            thunder_table(w).insert(m, h);

            if is_player_mobile(w, m) {
                add_buff(
                    w,
                    m,
                    BuffInfo::new(BuffIcon::Thunderstorm, 1075800)
                        .duration(duration)
                        .args(TextDefinition::of_text(
                            get_cast_recovery_malus_value(w, m).to_string(),
                        )),
                );
            }
        }
    }

    spell::finish_sequence(w, sp);
}

fn get_cast_recovery_malus_value(w: &World, m: MobileId) -> i32 {
    if w.resources
        .get::<ThunderTable>()
        .is_some_and(|t| t.0.contains_key(&m))
    {
        6
    } else {
        0
    }
}

fn thunder_stop_timer(w: &mut World, m: MobileId) {
    if let Some(mut t) = thunder_table(w).shift_remove(&m) {
        t.cancel(w);
    }
}

/// `DoExpire(m)`.
pub fn do_thunder_expire(w: &mut World, m: MobileId) {
    thunder_stop_timer(w, m);
    if is_player_mobile(w, m) {
        remove_buff(w, m, BuffIcon::Thunderstorm);
    }
}

// ---- WordOfDeathSpell ----

static WORD_OF_DEATH: ArcaneType = ArcaneType {
    required_skill: 80.0,
    required_mana: 50,
    ..ARCANE_BASE
};

/// `WordOfDeathSpell`: Nyraxle.
pub static WORD_OF_DEATH_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Spellweaving.WordOfDeathSpell",
    parent: Some(&ARCANIST_SPELL),
    target_range: Some(muo_content::spells::targeting::target_range),
    info: SpellInfo::new("Word of Death", "Nyraxle", &[]).action(-1),
    cast_delay_base: |_, _| TimeSpan::from_millis(3500),
    on_cast: |w, sp| begin_mobile(w, sp, TargetFlags::HARMFUL, word_target),
    family: Some(&WORD_OF_DEATH),
    ..ARCANIST_SPELL
};

fn word_target(w: &mut World, sp: SpellId, targeted: Targeted) {
    let Some(m) = mobile_of(targeted) else {
        return;
    };
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };
    if spell::check_h_sequence(w, sp, m) {
        let (mloc, map) = w.mobile(m).map(|x| (x.location, x.map)).unwrap_or_default();
        let loc = Point3D::new(mloc.x, mloc.y, mloc.z + 50);

        effects::play_sound(w, m, 0x211);
        effects::fixed_particles_hue(w, m, 0x3779, 1, 30, 0x26EC, 0x3, 0x3, EffectLayer::Waist);

        send_moving_particles_layer(
            w,
            FxTargeted::point(loc, map),
            FxTargeted::point(mloc, map),
            0xF5F,
            1,
            0,
            true,
            false,
            0x21,
            0x3F,
            0x251D,
            0,
            0,
            i32::from(EffectLayer::Head.0),
            0,
        );

        let focus = focus_level(w, sp);
        let percentage = 0.05 * f64::from(focus);

        let (hits, hits_max, player) = (
            w.mobile(m).map_or(0, |x| x.hits),
            mb::hits_max(w, m),
            w.mobile(m).is_some_and(|x| x.player),
        );

        let damage: i32 = if !player && f64::from(hits) / f64::from(hits_max) < percentage {
            300
        } else {
            let sw = skills::value(w, caster, SkillName::Spellweaving) as i32;
            let min_damage = sw / 5;
            let max_damage = sw / 3;

            let mut d = utility::random_min_max(&mut w.rng, min_damage, max_damage);

            let mut damage_bonus = aos_helpers::attribute_value(
                w,
                caster,
                muo_content::aos::AosAttribute::SpellDamage,
            );

            if player && damage_bonus > 15 {
                damage_bonus = 15;
            }

            d *= damage_bonus + 100;
            d /= 100;
            d
        };

        helper::damage_aos_spell(
            w,
            sp,
            m,
            f64::from(damage),
            SpellDamageSplit {
                chaos: 100,
                ..Default::default()
            },
        );
    }
}

/// Sets the weapon hooks, `SpellHooks::essence_of_wind_fc_malus`/`thunderstorm_cast_recovery_malus` and the events.
pub fn init(w: &mut World) {
    let h = w.resources.get_mut::<WeaponHooks>();
    h.attune_weapon_try_absorb = Some(try_absorb);
    h.essence_of_wind_ssi_malus = Some(|w, m| {
        if is_debuffed(w, m) {
            get_ssi_malus(w, m)
        } else {
            0
        }
    });
    h.immolating_is = Some(|w, i| {
        w.resources
            .get::<ImmolatingTable>()
            .is_some_and(|t| t.0.contains_key(&i))
    });
    h.immolating_do_effect = Some(do_effect);
    h.immolating_stop = Some(stop_immolating);

    let s = hooks_mut(w);
    s.essence_of_wind_fc_malus = Some(|w, m| {
        if is_debuffed(w, m) {
            get_fc_malus(w, m)
        } else {
            0
        }
    });
    s.thunderstorm_cast_recovery_malus = Some(get_cast_recovery_malus_value);

    muo_content::player_mobile::subscribe_death(w, life_on_death);
    muo_content::creature::events::subscribe_death(w, life_on_death);
}
