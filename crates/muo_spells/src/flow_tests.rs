//! Real flows: a cast through the cast timer, the target, the damage timer and the packets; both eras for the
//! `Core.` switches. The reagent items and the rune are stand-in classes (muo_items is above this crate).

use muo_content::spells::registry::new_spell_by_name;
use muo_content::spells::spell::{self, SpellType};
use muo_content::spells::targeting::RecallSource;
use muo_data::expansion::Expansion;
use muo_primitives::{Point3D, TimeSpan};
use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType};
use muo_server::map::MapId;
use muo_server::mobile::base::move_to_world;
use muo_server::net_state::{NetOut, NetStateData, NetStateId};
use muo_server::skills::{self, SkillName};
use muo_server::target::Targeted;

use crate::configure;

fn skill_table(w: &mut World) {
    use muo_data::skill_info::{SkillInfo, Stat};
    let info = |i: i32| SkillInfo {
        skill_id: i,
        name: format!("S{i}"),
        title: String::new(),
        str_scale: 0.0,
        dex_scale: 0.0,
        int_scale: 0.0,
        stat_total: 0.0,
        str_gain: 0.0,
        dex_gain: 0.0,
        int_gain: 0.0,
        gain_factor: 0.0,
        profession_skill_name: String::new(),
        primary_stat: Stat::Str,
        secondary_stat: Stat::Dex,
    };
    w.resources.get_mut::<muo_server::skills::SkillTable>().info = (0..58).map(info).collect();
}

fn world_for(e: Expansion) -> World {
    let mut w = muo_server::map::test_support::world();
    skill_table(&mut w);
    w.maps.global_max_update_range = 24;
    muo_server::effects::set_particle_support_type(
        &mut w,
        muo_server::effects::ParticleSupportType::Full,
    );
    w.core.expansion = Some(e);
    muo_content::skills::skill_check::initialize(&mut w);
    let (bodies, _) = muo_primitives::BodyTable::parse(
        "400	Human
401	Human
",
    );
    w.resources.insert(bodies);
    configure(&mut w);
    // `RegenRates.Configure` sets the default rates (without it every regen timer ticks each millisecond).
    muo_content::misc::regen_rates::configure(&mut w);
    w
}

/// A player mobile on Felucca at `(x, 100, 0)` with full stats, with a net state when `ns` is given.
fn player(w: &mut World, x: i32, ns: Option<u32>) -> MobileId {
    let m = muo_content::player_mobile::new_player_mobile(w);
    if let Some(n) = ns {
        let id = NetStateId(n);
        w.net_states.insert(id, NetStateData::new(0));
        muo_server::mobile::part1::props_b::set_net_state(w, m, Some(id));
        w.net_states.get_mut(&id).unwrap().mobile = Some(m);
    }
    move_to_world(w, m, Point3D::new(x, 100, 0), MapId::FELUCCA);
    let mob = w.mobile_mut(m).unwrap();
    mob.player = true;
    mob.str = 100;
    mob.dex = 100;
    mob.int = 100;
    mob.hits = 100;
    mob.stam = 100;
    mob.mana = 100;
    m
}

/// `count` of the reagent class in the caster's backpack (made if needed); the stack.
fn give_reagents(w: &mut World, m: MobileId, class: &'static str, count: i32) -> ItemId {
    let pack = match w.mobile(m).and_then(|x| x.backpack) {
        Some(p) => p,
        None => {
            let p = muo_content::items::cont::new_creature_backpack(w, "backpack");
            muo_server::mobile::part2::items::add_item(w, m, p);
            p
        }
    };
    let ty: &'static ItemType = Box::leak(Box::new(ItemType {
        full_name: class,
        parent: Some(&ITEM_BASE),
        ..ITEM_BASE
    }));
    w.types.register_item(ty, &[]);
    let it = w.new_item(ty);
    muo_server::item::ops::set_stackable(w, it, true);
    muo_server::item::ops::set_amount(w, it, count);
    muo_server::item::base::add_item(w, pack, it);
    it
}

fn sent(w: &mut World) -> Vec<Vec<u8>> {
    std::mem::take(&mut w.net_out)
        .into_iter()
        .filter_map(|o| match o {
            NetOut::Send(_, b) => Some(b),
            _ => None,
        })
        .collect()
}

/// Moves the clock `ms` forward one millisecond at a time and turns the timer wheel; `watch` runs after each step.
fn advance_with(w: &mut World, ms: i64, mut watch: impl FnMut(&mut World)) {
    let end = w.tick_count + ms;
    while w.tick_count < end {
        w.tick_count += 1;
        w.now += TimeSpan::from_millis(1);
        muo_server::timer::slice(w);
        watch(w);
    }
}

fn advance(w: &mut World, ms: i64) {
    advance_with(w, ms, |_| {});
}

fn pending_target(w: &World, m: MobileId) -> Option<muo_server::target::TargetId> {
    w.mobile(m).and_then(|x| x.target)
}

/// Casts `name` as `caster` and waits for the cast delay to give the target cursor.
fn cast_and_wait_for_cursor(w: &mut World, caster: MobileId, name: &str) -> spell::SpellId {
    let sp = new_spell_by_name(w, name, caster, None).expect("registered");
    assert!(spell::cast(w, sp), "{name} starts");
    assert_eq!(w.mobile(caster).unwrap().spell, Some(sp));
    assert!(pending_target(w, caster).is_none(), "still casting");
    advance(w, 3_000);
    assert!(pending_target(w, caster).is_some(), "the cursor is out");
    sp
}

/// The lowest `hits` the mobile has in the next `ms` (damage is regenerated by the same clock).
fn lowest_hits_while(w: &mut World, victim: MobileId, ms: i64) -> i32 {
    let mut lowest = w.mobile(victim).unwrap().hits;
    advance_with(w, ms, |w| {
        lowest = lowest.min(w.mobile(victim).unwrap().hits);
    });
    lowest
}

// ---- Fireball, end to end ----

#[test]
fn a_fireball_spends_mana_and_reagent_asks_for_a_target_and_burns_after_the_missile() {
    let mut w = world_for(Expansion::AOS);
    let caster = player(&mut w, 100, Some(1));
    let victim = player(&mut w, 103, None);
    skills::set_base(&mut w, caster, SkillName::Magery, 120.0);
    skills::set_base(&mut w, caster, SkillName::EvalInt, 120.0);
    let pearls = give_reagents(&mut w, caster, "Server.Items.BlackPearl", 20);
    sent(&mut w);

    let sp = cast_and_wait_for_cursor(&mut w, caster, "Fireball");
    let packets = sent(&mut w);
    assert!(packets.iter().any(|p| p[0] == 0x6C), "the 0x6C cursor");
    // The mana and the reagent are paid when the target is chosen (CheckSequence).
    assert_eq!(w.mobile(caster).unwrap().mana, 100);

    let t = pending_target(&w, caster).unwrap();
    muo_server::target::invoke(
        &mut w,
        t,
        caster,
        Targeted::Entity(EntityId::Mobile(victim)),
    );
    assert_eq!(spell::state(&w, sp), muo_content::spells::SpellState::None);
    assert_eq!(
        w.mobile(caster).unwrap().mana,
        91,
        "9 mana for the third circle"
    );
    assert_eq!(w.item(pearls).unwrap().amount, 19);
    let at_hit = sent(&mut w);
    assert!(
        at_hit.iter().any(|p| matches!(p[0], 0xC0 | 0x70 | 0xC7)),
        "the missile flies: {:?}",
        at_hit.iter().map(|p| p[0]).collect::<Vec<_>>()
    );
    // (the sound 0x54 is not captured by these net states: send_sound_effect needs the viewer in the sector)
    // Nothing is hurt before the missile lands (AOS delay: 1.25 s).
    assert_eq!(lowest_hits_while(&mut w, victim, 1_000), 100);
    let lowest = lowest_hits_while(&mut w, victim, 1_000);
    assert!(lowest < 100, "the victim burned: {lowest}");
    assert!(
        lowest >= 100 - 60,
        "AOS fireball damage is bounded: {lowest}"
    );
    assert_eq!(w.mobile(caster).unwrap().mana, 91, "no more mana");
}

#[test]
fn before_aos_a_fireball_is_old_damage_and_lands_after_half_a_second() {
    let mut w = world_for(Expansion::UOR);
    let caster = player(&mut w, 100, Some(1));
    let victim = player(&mut w, 103, None);
    skills::set_base(&mut w, caster, SkillName::Magery, 120.0);
    skills::set_base(&mut w, caster, SkillName::EvalInt, 120.0);
    give_reagents(&mut w, caster, "Server.Items.BlackPearl", 20);
    let sp = cast_and_wait_for_cursor(&mut w, caster, "Fireball");
    let t = pending_target(&w, caster).unwrap();
    muo_server::target::invoke(
        &mut w,
        t,
        caster,
        Targeted::Entity(EntityId::Mobile(victim)),
    );
    assert_eq!(spell::state(&w, sp), muo_content::spells::SpellState::None);
    assert_eq!(
        lowest_hits_while(&mut w, victim, 400),
        100,
        "the old delay is half a second"
    );
    let lowest = lowest_hits_while(&mut w, victim, 1_000);
    assert!((55..100).contains(&lowest), "old-era damage {lowest}");
}

// ---- Recall with a rune ----

#[test]
fn recall_to_a_marked_rune_takes_the_caster_there() {
    let mut w = world_for(Expansion::AOS);
    let caster = player(&mut w, 100, Some(1));
    skills::set_base(&mut w, caster, SkillName::Magery, 120.0);
    give_reagents(&mut w, caster, "Server.Items.BlackPearl", 5);
    give_reagents(&mut w, caster, "Server.Items.Bloodmoss", 5);
    give_reagents(&mut w, caster, "Server.Items.MandrakeRoot", 5);
    let rune = w.new_item(Box::leak(Box::new(ItemType {
        full_name: "Server.Items.RecallRune",
        parent: Some(&ITEM_BASE),
        ..ITEM_BASE
    })));
    muo_server::item::ops::move_to_world_map(
        &mut w,
        rune,
        Point3D::new(100, 101, 0),
        Some(MapId::FELUCCA),
    );
    // The rune is marked for (300, 300, 0): the items crate's resolver, stood in for.
    muo_content::spells::hooks::hooks_mut(&mut w).resolve_recall_source = Some(|_, e, _| match e {
        EntityId::Item(_) => RecallSource::Location {
            loc: Point3D::new(300, 300, 0),
            map: Some(MapId::FELUCCA),
            check_multi: true,
        },
        _ => RecallSource::Invalid,
    });
    sent(&mut w);

    let sp = cast_and_wait_for_cursor(&mut w, caster, "Recall");
    let t = pending_target(&w, caster).unwrap();
    muo_server::target::invoke(&mut w, t, caster, Targeted::Entity(EntityId::Item(rune)));
    let m = w.mobile(caster).unwrap();
    assert_eq!((m.location.x, m.location.y), (300, 300), "recalled");
    assert_eq!(m.map, Some(MapId::FELUCCA));
    assert_eq!(m.mana, 89, "fourth circle costs 11");
    assert_eq!(spell::state(&w, sp), muo_content::spells::SpellState::None);
    let packets = sent(&mut w);
    assert!(!packets.is_empty(), "the caster is told of the move");
}

// ---- A Necromancy transformation ----

#[test]
fn lich_form_changes_the_body_ticks_and_ends_with_remove_context() {
    use muo_content::spells::helper;
    let mut w = world_for(Expansion::AOS);
    let caster = player(&mut w, 100, Some(1));
    skills::set_base(&mut w, caster, SkillName::Necromancy, 120.0);
    skills::set_base(&mut w, caster, SkillName::SpiritSpeak, 120.0);
    for r in ["GraveDust", "DaemonBlood", "NoxCrystal"] {
        let name: &'static str = Box::leak(format!("Server.Items.{r}").into_boxed_str());
        give_reagents(&mut w, caster, name, 5);
    }
    let body_before = w.mobile(caster).unwrap().body_mod;
    sent(&mut w);

    let sp = new_spell_by_name(&mut w, "Lich Form", caster, None).expect("registered");
    assert!(spell::cast(&mut w, sp));
    advance(&mut w, 5_000);
    assert_eq!(w.mobile(caster).unwrap().body_mod.0, 749, "the lich body");
    assert!(helper::under_transformation_of(
        &w,
        caster,
        "Server.Spells.Necromancy.LichFormSpell"
    ));
    assert!(
        w.mobile(caster).unwrap().mana <= 100,
        "mana regenerates on this clock"
    );
    // The tick drains a hit point every 2.5 s.
    let lowest = lowest_hits_while(&mut w, caster, 3_000);
    assert!(lowest < 100, "the lich tick: {lowest}");
    assert!(!sent(&mut w).is_empty());

    helper::remove_context(&mut w, caster, true);
    assert_eq!(
        w.mobile(caster).unwrap().body_mod,
        body_before,
        "the body is back"
    );
    assert!(!helper::under_transformation_of(
        &w,
        caster,
        "Server.Spells.Necromancy.LichFormSpell"
    ));
}

// ---- The Core. switches, both eras ----

#[test]
fn the_info_switches_follow_the_era() {
    use crate::fifth::mind_blast::MIND_BLAST_SPELL;
    use crate::fourth::arch_protection::ARCH_PROTECTION_SPELL;
    use crate::second::harm::HARM_SPELL;
    let ty =
        |t: &'static SpellType, aos: bool| (t.info.action_in(aos), t.info.hand_effects_in(aos).0);
    assert_eq!(ty(&HARM_SPELL, true), (212, 9001));
    assert_eq!(ty(&HARM_SPELL, false), (212, 9041));
    assert_eq!(ty(&ARCH_PROTECTION_SPELL, true), (239, 9011));
    assert_eq!(ty(&ARCH_PROTECTION_SPELL, false), (215, 9011));
    assert_eq!(ty(&MIND_BLAST_SPELL, true), (218, 9002));
    assert_eq!(ty(&MIND_BLAST_SPELL, false), (218, 9032));
}

#[test]
fn a_casting_harm_shows_the_hand_effect_of_the_era() {
    for (e, effect) in [(Expansion::AOS, 9001), (Expansion::UOR, 9041)] {
        let mut w = world_for(e);
        let caster = player(&mut w, 100, Some(1));
        muo_server::mobile::part1::props_b::set_body(&mut w, caster, muo_primitives::Body(0x190));
        skills::set_base(&mut w, caster, SkillName::Magery, 120.0);
        give_reagents(&mut w, caster, "Server.Items.Nightshade", 5);
        give_reagents(&mut w, caster, "Server.Items.SpidersSilk", 5);
        sent(&mut w);
        let sp = new_spell_by_name(&mut w, "Harm", caster, None).expect("registered");
        assert!(spell::cast(&mut w, sp));
        advance(&mut w, 400);
        // The hand effect is a fixed particle effect on each hand, its id inside the effect packet.
        let packets = sent(&mut w);
        let found = packets.iter().any(|p| {
            matches!(p[0], 0xC0 | 0xC7 | 0x70)
                && p.windows(2)
                    .any(|b| i16::from_be_bytes([b[0], b[1]]) as i32 == effect)
        });
        assert!(
            found,
            "{e:?} shows {effect}: {:?}",
            packets.iter().map(|p| p[0]).collect::<Vec<_>>()
        );
    }
}

#[test]
fn delayed_damage_of_mind_blast_and_earthquake_is_for_the_old_era_only() {
    use crate::eighth::earthquake::EARTHQUAKE_SPELL;
    use crate::fifth::mind_blast::MIND_BLAST_SPELL;
    for t in [&MIND_BLAST_SPELL, &EARTHQUAKE_SPELL] {
        assert!(!t.delayed_damage_in(true), "{}", t.full_name);
        assert!(t.delayed_damage_in(false), "{}", t.full_name);
    }
}

#[test]
fn the_old_era_has_no_delayed_damage_family_for_magic_arrow_and_explosion() {
    use crate::first::magic_arrow::MAGIC_ARROW_SPELL;
    use crate::sixth::explosion::EXPLOSION_SPELL;
    for t in [&MAGIC_ARROW_SPELL, &EXPLOSION_SPELL] {
        assert!(
            t.delayed_damage_family_in(true).is_some(),
            "{}",
            t.full_name
        );
        assert!(
            t.delayed_damage_family_in(false).is_none(),
            "{}",
            t.full_name
        );
    }
}

#[test]
fn necromancer_spells_use_fast_cast_only_from_se() {
    let mut w = world_for(Expansion::SE);
    let c = player(&mut w, 100, None);
    // Two points of fast cast, through the Essence of Wind malus (which is subtracted).
    muo_content::spells::hooks::hooks_mut(&mut w).essence_of_wind_fc_malus = Some(|_, _| -2);
    let sp = new_spell_by_name(&mut w, "Pain Spike", c, None).expect("registered");
    let ty = spell::spell_type(&w, sp).unwrap();
    let se = (ty.get_cast_delay)(&mut w, sp);
    w.core.expansion = Some(Expansion::AOS);
    let aos = (ty.get_cast_delay)(&mut w, sp);
    assert!(
        se < aos,
        "SE honors fast cast ({se:?}) and AOS does not ({aos:?})"
    );
    assert_eq!(
        aos,
        (ty.cast_delay_base)(&w, sp),
        "AOS: the plain base delay"
    );
}

#[test]
fn ninjitsu_values_switch_on_ml() {
    use crate::ninjitsu::animal_form::ANIMAL_FORM_SPELL;
    use crate::ninjitsu::mirror_image::MIRROR_IMAGE_SPELL;
    use crate::ninjitsu::moves::{BACKSTAB, FOCUS_ATTACK, SURPRISE_ATTACK};
    assert_eq!(
        (
            BACKSTAB.required_skill_in(true),
            BACKSTAB.required_skill_in(false)
        ),
        (40.0, 20.0)
    );
    assert_eq!(
        (
            SURPRISE_ATTACK.required_skill_in(true),
            SURPRISE_ATTACK.required_skill_in(false)
        ),
        (60.0, 30.0)
    );
    assert_eq!(
        (
            FOCUS_ATTACK.base_mana_in(true),
            FOCUS_ATTACK.base_mana_in(false)
        ),
        (10, 20)
    );
    assert_eq!(
        (
            FOCUS_ATTACK.required_skill_in(true),
            FOCUS_ATTACK.required_skill_in(false)
        ),
        (30.0, 60.0)
    );

    let mut w = world_for(Expansion::ML);
    let c = player(&mut w, 100, None);
    let mirror = spell::new_spell(&mut w, &MIRROR_IMAGE_SPELL, c, None);
    let animal = spell::new_spell(&mut w, &ANIMAL_FORM_SPELL, c, None);
    assert_eq!(crate::ninjitsu::ninja::required_skill(&w, mirror), 20.0);
    assert_eq!(crate::ninjitsu::ninja::required_mana(&w, animal), 10);
    w.core.expansion = Some(Expansion::SE);
    assert_eq!(crate::ninjitsu::ninja::required_skill(&w, mirror), 40.0);
    assert_eq!(crate::ninjitsu::ninja::required_mana(&w, animal), 0);
    // Animal Form recovers in 10 from ML and in the ninja base of 7 before.
    assert_eq!(ANIMAL_FORM_SPELL.cast_recovery_base, 10);
    assert_eq!(ANIMAL_FORM_SPELL.cast_recovery_base_pre_ml, Some(7));
}

#[test]
fn field_spells_have_their_own_target_range() {
    use crate::fourth::fire_field::FIRE_FIELD_SPELL;
    use crate::third::wall_of_stone::WALL_OF_STONE_SPELL;
    let w = world_for(Expansion::AOS);
    let range = |t: &'static SpellType| t.target_range.map(|f| f(&w));
    assert_eq!(range(&WALL_OF_STONE_SPELL), Some(15));
    assert_eq!(range(&FIRE_FIELD_SPELL), Some(15));
    assert_eq!(
        range(&crate::eighth::resurrection::RESURRECTION_SPELL),
        Some(1)
    );
    assert_eq!(
        range(&crate::ninjitsu::shadow_jump::SHADOWJUMP_SPELL),
        Some(11)
    );
    assert_eq!(
        range(&crate::first::night_sight::NIGHT_SIGHT_SPELL),
        Some(12)
    );
    assert_eq!(range(&crate::third::fireball::FIREBALL_SPELL), Some(10));
    assert_eq!(
        range(&crate::first::create_food::CREATE_FOOD_SPELL),
        None,
        "not a targeting spell"
    );
}

// ---- The creatures the spells make ----

#[test]
fn the_spell_creatures_construct_with_their_stats_and_save_a_version_zero() {
    use crate::spellweaving::mobiles::{ANIMATED_WEAPON, ARCANE_FEY, ARCANE_FIEND, NATURE_FURY};
    let mut w = world_for(Expansion::ML);
    let fey = (ARCANE_FEY.construct.unwrap())(&mut w);
    let m = w.mobile(fey).unwrap();
    assert_eq!(
        (m.body.0, m.base_sound_id, m.str, m.dex, m.int),
        (128, 0x467, 20, 150, 125)
    );
    let fiend = (ARCANE_FIEND.construct.unwrap())(&mut w);
    assert_eq!(w.mobile(fiend).unwrap().body.0, 74);
    let fury = (NATURE_FURY.construct.unwrap())(&mut w);
    let m = w.mobile(fury).unwrap();
    assert_eq!((m.body.0, m.hue, m.hits), (0x33, 0x4001, 80));
    let c = muo_content::creature::ctype(&w, fury).unwrap();
    assert!(
        (c.delete_corpse_on_death)(&w, fury),
        "AOS or later deletes the corpse"
    );
    w.core.expansion = Some(Expansion::UOR);
    assert!(
        !(c.delete_corpse_on_death)(&w, fury),
        "before AOS the corpse stays"
    );

    // The Animated Weapon takes its level and the caster's skills through the hook.
    let caster = player(&mut w, 100, None);
    skills::set_base(&mut w, caster, SkillName::Anatomy, 80.0);
    let weapon = (ANIMATED_WEAPON.make.unwrap())(&mut w);
    crate::spellweaving::mobiles::init_animated_weapon(&mut w, weapon, caster, 100);
    let m = w.mobile(weapon).unwrap();
    assert_eq!((m.body.0, m.str, m.dex, m.int), (692, 110, 110, 10));
    assert_eq!(skills::base(&mut w, weapon, SkillName::Anatomy), 40.0);
    assert_eq!(skills::base(&mut w, weapon, SkillName::Wrestling), 100.0);
    let (lo, hi) = muo_content::creature::cd(&w, weapon)
        .map(|d| (d.damage_min, d.damage_max))
        .unwrap();
    assert_eq!((lo, hi), (12, 15), "level 100: SetDamage(12, 15)");

    let mut out = muo_io::SaveWriter::new(true);
    (ARCANE_FEY.serialize)(&w, fey, &mut out);
    assert!(!out.into_vec().is_empty());
}

#[test]
fn a_mirror_image_clone_copies_the_caster_follows_it_and_vanishes_when_hurt() {
    use crate::ninjitsu::clone::new_clone;
    use crate::ninjitsu::mirror_image::has_clone;
    let mut w = world_for(Expansion::SE);
    let caster = player(&mut w, 100, Some(1));
    skills::set_base(&mut w, caster, SkillName::Ninjitsu, 80.0);
    if let Some(m) = w.mobile_mut(caster) {
        m.name = Some("Tester".to_string());
        m.female = true;
    }
    muo_server::mobile::base::set_hue(&mut w, caster, 0x3E8);

    let clone = new_clone(&mut w, caster);
    muo_server::mobile::base::move_to_world(
        &mut w,
        clone,
        Point3D::new(100, 100, 0),
        MapId::FELUCCA,
    );
    let m = w.mobile(clone).unwrap();
    assert_eq!(
        (m.name.as_deref(), m.female, m.hue),
        (Some("Tester"), true, 0x3E8)
    );
    assert!(has_clone(&mut w, caster));
    let d = muo_content::creature::cd(&w, clone).unwrap();
    assert!(d.summoned && d.master == Some(caster));
    assert_eq!(
        d.control_order,
        muo_content::creature::enums::OrderType::Follow
    );
    assert_eq!(skills::base(&mut w, clone, SkillName::Ninjitsu), 80.0);

    // Its AI is CloneAI, whose Think only follows (it always answers true).
    assert_eq!(
        muo_content::ai::class_of(&w, clone).map(|c| c.name),
        Some("CloneAI")
    );
    assert!((crate::ninjitsu::clone::CLONE_AI.think)(&mut w, clone));

    // Hurting it deletes it, and the count goes with it.
    muo_server::mobile::base::damage(&mut w, clone, 1, Some(caster), true, false);
    assert!(w.mobile(clone).is_none_or(|m| m.deleted));
    // RemoveClone leaves the key with a count of zero, so HasClone stays true.
    assert!(has_clone(&mut w, caster));
}

#[test]
fn every_spell_class_the_mage_and_healer_ai_names_resolves() {
    let mut w = world_for(Expansion::ML);
    let c = player(&mut w, 100, None);
    for name in [
        "BlessSpell",
        "BloodOathSpell",
        "ClumsySpell",
        "CorpseSkinSpell",
        "CureSpell",
        "CurseSpell",
        "DispelSpell",
        "EnergyBoltSpell",
        "EvilOmenSpell",
        "ExplosionSpell",
        "FeeblemindSpell",
        "FireballSpell",
        "FlameStrikeSpell",
        "GreaterHealSpell",
        "HarmSpell",
        "HealSpell",
        "InvisibilitySpell",
        "LightningSpell",
        "MagicArrowSpell",
        "ManaDrainSpell",
        "ManaVampireSpell",
        "MindBlastSpell",
        "MindRotSpell",
        "PainSpikeSpell",
        "ParalyzeSpell",
        "PoisonSpell",
        "PoisonStrikeSpell",
        "RevealSpell",
        "StrangleSpell",
        "TeleportSpell",
        "VengefulSpiritSpell",
        "WeakenSpell",
        "WitherSpell",
    ] {
        assert!(new_spell_by_name(&mut w, name, c, None).is_some(), "{name}");
    }
}

#[test]
fn a_mage_ai_range_is_the_spells_target_range_when_it_is_ranged() {
    use muo_content::spells::targeting::ranged_spell_range;
    let mut w = world_for(Expansion::AOS);
    let c = player(&mut w, 100, None);
    let fireball = new_spell_by_name(&mut w, "Fireball", c, None).unwrap();
    let wall = new_spell_by_name(&mut w, "Wall of Stone", c, None).unwrap();
    let food = new_spell_by_name(&mut w, "Create Food", c, None).unwrap();
    assert_eq!(ranged_spell_range(&w, fireball), Some(10));
    assert_eq!(ranged_spell_range(&w, wall), Some(15));
    assert_eq!(
        ranged_spell_range(&w, food),
        None,
        "not an IRangedSpell: RangePerception"
    );
}

#[test]
fn the_weapon_hooks_this_crate_owns_reach_the_buff_table_and_the_bard() {
    use muo_content::items::weapon_hooks as hk;
    use muo_content::player_mobile::buff::BuffConfig;
    let mut w = world_for(Expansion::ML);
    w.resources.get_mut::<BuffConfig>().enabled = true;
    let m = player(&mut w, 100, None);
    // 0x3F is BuffIcon.NoRearm: the weapon abilities add it for the length of a lock.
    hk::add_buff(&mut w, m, 0x3F, 1075637, TimeSpan::from_secs(10));
    let has = |w: &World| {
        muo_content::player_mobile::data(w, m).is_some_and(|d| d.buff_table.contains_key(&0x3F))
    };
    assert!(has(&w), "added");
    advance(&mut w, 10_100);
    assert!(!has(&w), "the buff ran out with its timer");
    hk::add_buff(&mut w, m, 0x3F, 1075637, TimeSpan::ZERO);
    assert!(has(&w));
    hk::remove_buff(&mut w, m, 0x3F);
    assert!(!has(&w), "removed");
    assert_eq!(
        hk::discordance_effect(&w, m),
        None,
        "no song of discordance on a plain player"
    );
    assert!(
        muo_content::spells::hooks::hooks(&w)
            .unwrap()
            .consume_bandage
            .is_some()
    );
}
