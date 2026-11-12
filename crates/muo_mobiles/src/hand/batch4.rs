//! Betrayer, ExodusMinion/Overseer, LadyJennifyr, SirPatrick and the DawnsMusicGear drops (Juggernaut, Betrayer).

use muo_content::abilities::{self, MonsterAbility, MonsterAbilityTrigger, Weighted};
use muo_content::creature::{BASE_CREATURE, cd};
use muo_content::factory;
use muo_content::mx;
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::container::drop_item_default;
use muo_server::region::MusicName;

use super::ctors::pack;
use crate::common::rnd;

// ---- Dawn's Music Box (8th anniversary items) ----

pub use muo_items::special::anniv2::DawnsMusicRarity as DawnsRarity;

/// `DawnsMusicBox.RandomTrack(rarity)`: `GetTracks(rarity).RandomElement()`.
pub fn dawns_random_track(w: &mut World, rarity: DawnsRarity) -> MusicName {
    muo_items::special::anniv2::random_track(w, rarity)
}

/// `c.DropItem(new DawnsMusicGear(DawnsMusicBox.RandomTrack(rarity)))`.
fn drop_gear(w: &mut World, c: Option<ItemId>, rarity: DawnsRarity) {
    let track = dawns_random_track(w, rarity);
    if let Some(c) = c {
        let i = muo_items::special::anniv2::dawns_music_gear_new_music(w, track as i32);
        drop_item_default(w, c, i);
    }
}

/// The 5% DawnsMusicGear drop of Juggernaut and Betrayer.
pub fn dawns_gear_on_death(w: &mut World, id: MobileId, c: Option<ItemId>) {
    (BASE_CREATURE.on_death)(w, id, c);
    if rnd::double(w) < 0.05 {
        if !cd(w, id).is_some_and(|d| d.is_paragon) {
            let r = if rnd::double(w) < 0.75 {
                DawnsRarity::Common
            } else {
                DawnsRarity::Uncommon
            };
            drop_gear(w, c, r);
        } else {
            drop_gear(w, c, DawnsRarity::Rare);
        }
    }
}

// ---- Betrayer ----

pub fn betrayer_on_death(w: &mut World, id: MobileId, c: Option<ItemId>) {
    dawns_gear_on_death(w, id, c);
}

static BETRAYER_MEMBERS: [Weighted; 2] = [
    Weighted::new(1, &abilities::COLOSSAL_BLOW),
    Weighted::new(1, &abilities::POISON_GAS_AREA_ATTACK),
];
static BETRAYER_GROUP: MonsterAbility = abilities::group(
    &BETRAYER_MEMBERS,
    MonsterAbilityTrigger::GIVE_MELEE_DAMAGE.or(MonsterAbilityTrigger::COMBAT_ACTION),
);
static BETRAYER_ABILITIES: [&MonsterAbility; 1] = [&BETRAYER_GROUP];
pub fn betrayer_abilities(_w: &World, _id: MobileId) -> &'static [&'static MonsterAbility] {
    &BETRAYER_ABILITIES
}

// ---- ExodusMinion, ExodusOverseer ----

/// `PackItem` of one of PowerCrystal, ArcaneGem, ClockworkAssembly: `switch (Utility.Random(3))`.
pub fn exodus_minion_ctor1(w: &mut World, id: MobileId) {
    match rnd::below(w, 3) {
        0 => pack(w, id, "PowerCrystal"),
        1 => pack(w, id, "ArcaneGem"),
        2 => pack(w, id, "ClockworkAssembly"),
        _ => {}
    }
}

static EXODUS_ML: [&MonsterAbility; 1] = [&abilities::ENERGY_BOLT_COUNTER];
static EXODUS_PRE_ML: [&MonsterAbility; 1] = [&abilities::MAGICAL_BARRIER];
/// `Core.ML ? MonsterAbilities.EnergyBoltCounter : MonsterAbilities.MagicalBarrier` (chosen once, when the
/// static array is built).
pub fn exodus_abilities(w: &World, _id: MobileId) -> &'static [&'static MonsterAbility] {
    if w.core_ml() {
        &EXODUS_ML
    } else {
        &EXODUS_PRE_ML
    }
}

// ---- LadyJennifyr: `new FanningFire(0.10, -10, 35, 45)` ----

static LADY_JENNIFYR_FANNING_FIRE: MonsterAbility = MonsterAbility {
    chance: 0.10,
    ..abilities::FANNING_FIRE
};
static LADY_JENNIFYR_ABILITIES: [&MonsterAbility; 1] = [&LADY_JENNIFYR_FANNING_FIRE];
pub fn lady_jennifyr_abilities(_w: &World, _id: MobileId) -> &'static [&'static MonsterAbility] {
    &LADY_JENNIFYR_ABILITIES
}

// ---- SirPatrick: `SirPatrickDrainLife : DrainLifeAreaAttack` (damage 14-30, other particles and sound) ----

static SIR_PATRICK_DRAIN_LIFE: MonsterAbility = MonsterAbility {
    name: "SirPatrickDrainLife",
    do_effect_target: |w, _, s, d| {
        muo_server::mobile::part2::effects::fixed_particles_hue(
            w,
            d,
            0x374A,
            10,
            15,
            5013,
            0x455,
            0,
            muo_server::effects::EffectLayer::Waist,
        );
        mx::play_sound(w, d, 0x1EA);
        // DrainLife(source, defender)
        let to_drain = rnd::min_max(w, 14, 30);
        let h = mx::hits(w, s);
        muo_server::mobile::part1::props_b::set_hits(w, s, h + to_drain);
        mx::do_harmful(w, s, d);
        mx::damage(w, d, to_drain, Some(s));
    },
    ..abilities::DRAIN_LIFE_AREA_ATTACK
};
static SIR_PATRICK_ABILITIES: [&MonsterAbility; 1] = [&SIR_PATRICK_DRAIN_LIFE];
pub fn sir_patrick_abilities(_w: &World, _id: MobileId) -> &'static [&'static MonsterAbility] {
    &SIR_PATRICK_ABILITIES
}

// ---- HellSteed.SetStats(this) ----

use crate::common::*;

pub fn hell_steed_ctor1(w: &mut World, id: MobileId) {
    set_str_range(w, id, 201, 210);
    set_dex_range(w, id, 101, 110);
    set_int_range(w, id, 101, 115);
    set_hits_range(w, id, 201, 220);
    set_damage(w, id, 20, 24);
    set_damage_type(w, id, ResistanceType::Physical, 25);
    set_damage_type(w, id, ResistanceType::Fire, 75);
    set_resistance_range(w, id, ResistanceType::Physical, 60, 70);
    set_resistance(w, id, ResistanceType::Fire, 90);
    set_resistance(w, id, ResistanceType::Poison, 100);
    set_skill_range(w, id, SkillName::MagicResist, 90.1, 110.0);
    set_skill(w, id, SkillName::Tactics, 50.0);
    set_skill_range(w, id, SkillName::Wrestling, 90.1, 110.0);
    fame(w, id, 0);
    karma(w, id, 0);
}

// ---- Ferret ----

/// `m_CanTalk` (not saved; a ferret without the state can talk, like one after `AfterDeserialization`).
#[derive(Clone, Copy)]
struct CanTalk(bool);

fn can_talk(w: &World, id: MobileId) -> bool {
    w.mobile(id)
        .and_then(|m| m.ext.get::<CanTalk>())
        .is_none_or(|c| c.0)
}
fn set_can_talk(w: &mut World, id: MobileId, v: bool) {
    if let Some(m) = w.mobile_mut(id) {
        m.ext.insert(CanTalk(v));
    }
}

/// `m_CanTalk = true;`
pub fn ferret_ctor1(w: &mut World, id: MobileId) {
    set_can_talk(w, id, true);
}

/// `OnMovement(m, oldLocation)`: another living ferret within 3 tiles is talked to.
pub fn ferret_on_movement(w: &mut World, id: MobileId, m: MobileId, _old: muo_primitives::Point3D) {
    if factory::mobile_is(w, m, "Server.Mobiles.Ferret")
        && mx::in_range(w, m, id, 3)
        && mx::alive(w, m)
    {
        ferret_talk(w, id, Some(m));
    }
}

/// `Talk(Ferret to)`.
pub fn ferret_talk(w: &mut World, id: MobileId, to: Option<MobileId>) {
    use muo_primitives::TimeSpan;
    const VOCABULARY: [&str; 3] = ["dook", "dook dook", "dook dook dook!"];
    if !can_talk(w, id) {
        return;
    }
    if let Some(to) = to {
        // QuestSystem.FocusTo(this, to)
        if rnd::bool(w) {
            mx::animate(w, id, 17, 7, 1, true, false, 0);
        } else {
            let a = 32 + rnd::below(w, 3);
            mx::animate(w, id, a, 7, 1, true, false, 0);
        }
        let d = mx::get_direction_to(w, id, to);
        mx::set_direction(w, id, d);
    }
    let text = VOCABULARY[rnd::below(w, 3) as usize];
    muo_server::mobile::part2::messages::say_text(w, id, text);
    if let Some(to) = to
        && rnd::bool(w)
    {
        let secs = rnd::min_max(w, 5, 8);
        muo_server::timer::start(
            w,
            TimeSpan::from_secs(i64::from(secs)),
            TimeSpan::ZERO,
            1,
            move |w| {
                if !mx::deleted(w, to) {
                    ferret_talk(w, to, None);
                }
            },
        );
    }
    set_can_talk(w, id, false);
    let secs = rnd::min_max(w, 20, 30);
    muo_server::timer::start(
        w,
        TimeSpan::from_secs(i64::from(secs)),
        TimeSpan::ZERO,
        1,
        move |w| {
            if !mx::deleted(w, id) {
                set_can_talk(w, id, true);
            }
        },
    );
}

// ---- OrcBrute ----

/// `OnDamagedBySpell(caster, damage)`: anyone but the brute itself brings an orc lord.
pub fn orc_brute_on_damaged_by_spell(w: &mut World, id: MobileId, caster: MobileId, _damage: i32) {
    if caster == id {
        return;
    }
    orc_brute_spawn_orc_lord(w, id, caster);
}

/// `SpawnOrcLord(target)`: at most 10 orcish lords within 10 tiles; a `SpawnedOrcishLord` appears near the target
/// and attacks it.
pub fn orc_brute_spawn_orc_lord(w: &mut World, id: MobileId, target: MobileId) {
    let Some(map) = mx::map(w, target).filter(|&m| m != muo_server::map::MapId::INTERNAL) else {
        return;
    };
    let mut count = 0;
    for m in muo_server::mobile::part2::enumerators::get_mobiles_in_range(w, id, 10) {
        if factory::mobile_is(w, m, "Server.Mobiles.OrcishLord") {
            count += 1;
            if count == 10 {
                return;
            }
        }
    }
    let here = mx::location(w, target);
    let location =
        muo_server::map::fit::get_random_nearby_location(w, map, here, 2, 0, 10, 16, false, false);
    let Some(orc) = factory::make_mobile(w, "Server.Mobiles.SpawnedOrcishLord") else {
        return;
    };
    let team = cd(w, id).map_or(0, |d| d.team);
    muo_content::creature::control::set_team(w, orc, team);
    muo_content::creature::control::set_home(w, orc, location);
    if let Some(d) = muo_content::creature::data::cd_mut(w, orc) {
        d.range_home = 10;
    }
    mx::set_combatant(w, orc, Some(target));
    muo_server::mobile::base::move_to_world(w, orc, location, map);
}

// ---- SandVortex ----

/// `m_NextAttack` (not saved).
#[derive(Clone, Copy)]
struct NextSandAttack(muo_primitives::DateTime);

/// `OnActionCombat`: a sand attack on the combatant every 10-20 seconds.
pub fn sand_vortex_on_action_combat(w: &mut World, id: MobileId) {
    use muo_primitives::{DateTime, TimeSpan};
    let Some(c) = mx::combatant(w, id) else {
        return;
    };
    if mx::deleted(w, c)
        || mx::map(w, c) != mx::map(w, id)
        || !mx::in_range(w, c, id, 12)
        || !mx::can_be_harmful(w, id, c)
        || !mx::in_los(w, id, c)
    {
        return;
    }
    let next = w
        .mobile(id)
        .and_then(|m| m.ext.get::<NextSandAttack>())
        .map_or(DateTime::MIN, |n| n.0);
    if w.now >= next {
        // SandAttack(combatant)
        mx::do_harmful(w, id, c);
        muo_server::mobile::part2::effects::fixed_particles_hue(
            w,
            c,
            0x36B0,
            10,
            25,
            9540,
            2413,
            0,
            muo_server::effects::EffectLayer::Waist,
        );
        muo_server::timer::start(w, TimeSpan::from_secs(1), TimeSpan::ZERO, 1, move |w| {
            mx::play_sound(w, c, 0x4CF);
            let amount = rnd::min_max(w, 1, 40);
            muo_content::misc::aos_helpers::damage(
                w,
                c,
                Some(id),
                amount,
                muo_content::misc::aos_helpers::DamageSplit {
                    phys: 90,
                    fire: 10,
                    cold: 0,
                    pois: 0,
                    nrgy: 0,
                },
                Default::default(),
            );
        });
        let delay = TimeSpan::from_secs_f64(10.0 + 10.0 * rnd::double(w));
        let at = w.now + delay;
        if let Some(m) = w.mobile_mut(id) {
            m.ext.insert(NextSandAttack(at));
        }
    }
}

// ---- RaiJu ----

/// `OnGaveMeleeAttack`: a 10% Lightning Fist freezes the defender for 4 seconds (no stacking).
pub fn rai_ju_on_gave_melee_attack(w: &mut World, id: MobileId, defender: MobileId, damage: i32) {
    use muo_primitives::TimeSpan;
    (muo_content::creature::BASE_CREATURE_C.on_gave_melee_attack)(w, id, defender, damage);
    if rnd::double(w) < 0.9 || tables(w).rai_ju.contains(&defender) {
        return;
    }
    mx::fixed_effect(w, defender, 0x37B9, 10, 5);
    mx::msg(w, defender, 1070839); // The creature attacks with stunning force!
    mx::set_frozen(w, defender, true);
    muo_server::timer::start(w, TimeSpan::from_secs(4), TimeSpan::ZERO, 1, move |w| {
        mx::msg(w, defender, 1005603); // You can move again!
        mx::set_frozen(w, defender, false);
        tables(w).rai_ju.remove(&defender);
    });
    tables(w).rai_ju.insert(defender);
}

// ---- Spellbinder ----

/// `SpellbinderAI : MageAI` (`GetRandomDamageSpell() => GetRandomCurseSpell()`, which MageAI checks by name).
pub static SPELLBINDER_AI: muo_content::ai::AiClass = muo_content::ai::AiClass {
    name: "SpellbinderAI",
    ..muo_content::ai::mage::MAGE_AI
};
pub fn spellbinder_forced_ai(
    _w: &World,
    _id: MobileId,
) -> Option<&'static muo_content::ai::AiClass> {
    Some(&SPELLBINDER_AI)
}

// ---- DeathwatchBeetle ----

/// `AlterMeleeDamageTo(to, ref damage)`: half the time, with mana to spare, a crushing hit does 50% more.
pub fn deathwatch_beetle_alter_melee_damage_to(
    w: &mut World,
    id: MobileId,
    to: MobileId,
    damage: &mut i32,
) {
    if rnd::bool(w) && w.mobile(id).is_some_and(|m| m.mana > 14) {
        *damage += *damage / 2;
        mx::msg(w, to, 1060091); // You take extra damage from the crushing attack!
        mx::play_sound(w, to, 0x1E1);
        muo_server::mobile::part2::effects::fixed_particles_hue(
            w,
            to,
            0x377A,
            1,
            32,
            0x26da,
            0,
            0,
            muo_server::effects::EffectLayer::Head,
        );
        let mana = w.mobile(id).map_or(0, |m| m.mana);
        muo_server::mobile::part1::props_b::set_mana(w, id, mana - 15);
    }
}

/// `OnDamage`: without a combatant in reach nothing happens (not even the base); else a 10% poison spit.
pub fn deathwatch_beetle_on_damage(
    w: &mut World,
    id: MobileId,
    amount: i32,
    from: Option<MobileId>,
    will_kill: bool,
) {
    let Some(c) = mx::combatant(w, id) else {
        return;
    };
    if mx::deleted(w, c)
        || mx::map(w, c) != mx::map(w, id)
        || !mx::in_range(w, c, id, 12)
        || !mx::can_be_harmful(w, id, c)
        || !mx::in_los(w, id, c)
    {
        return;
    }
    if rnd::below(w, 10) == 0 {
        // PoisonAttack(combatant)
        mx::do_harmful(w, id, c);
        muo_server::mobile::part2::effects::moving_particles_layer(
            w,
            id,
            mx::entity(c),
            0x36D4,
            1,
            0,
            false,
            false,
            0x3F,
            0,
            0x1F73,
            1,
            0,
            muo_server::effects::EffectLayer::None,
            0x100,
        );
        muo_server::mobile::base::apply_poison(w, c, Some(id), muo_server::poison::PoisonId(1));
        let name = w
            .mobile(id)
            .and_then(|m| (m.ty.name)(w, id))
            .unwrap_or_default();
        mx::msg_args(w, c, 1070821, &name); // %s spits a poisonous substance at you!
    }
    (BASE_CREATURE.on_damage)(w, id, amount, from, will_kill);
}
