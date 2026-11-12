//! Tests of the three spells: their classes, and the casting sequence through the spell system.

use muo_content::spells::magery::{MageryType, circle};
use muo_content::spells::registry::{new_spell_by_id, new_spell_by_name, registry};
use muo_content::spells::spell::{self, SpellType};
use muo_content::spells::{DisturbType, SpellCircle, SpellState, reagent};
use muo_data::expansion::Expansion;
use muo_server::World;
use muo_server::mobile::MOBILE_BASE;

use crate::{
    configure, first::clumsy::CLUMSY_SPELL, fourth::recall::RECALL_SPELL,
    third::fireball::FIREBALL_SPELL,
};

fn world() -> World {
    let mut w = World::new();
    w.core.expansion = Some(Expansion::AOS);
    configure(&mut w);
    w
}

#[test]
fn registered_under_the_initializer_ids() {
    let mut w = world();
    assert_eq!(
        registry(&mut w).registry_number("Server.Spells.First.ClumsySpell"),
        0
    );
    assert_eq!(
        registry(&mut w).registry_number("Server.Spells.Third.FireballSpell"),
        17
    );
    assert_eq!(
        registry(&mut w).registry_number("Server.Spells.Fourth.RecallSpell"),
        31
    );
    let c = w.new_mobile(&MOBILE_BASE);
    assert!(new_spell_by_id(&mut w, 17, c, None).is_some());
    assert!(new_spell_by_name(&mut w, "Recall", c, None).is_some());
    assert!(new_spell_by_name(&mut w, "Fireball Spell", c, None).is_some());
    assert!(new_spell_by_name(&mut w, "Cureall", c, None).is_none());
}

#[test]
fn infos_and_circles_have_the_expected_values() {
    assert_eq!(
        (
            CLUMSY_SPELL.info.name,
            CLUMSY_SPELL.info.mantra,
            CLUMSY_SPELL.info.action
        ),
        ("Clumsy", "Uus Jux", 212)
    );
    assert_eq!(CLUMSY_SPELL.info.left_hand_effect, 9031);
    assert_eq!(
        CLUMSY_SPELL.info.reagents,
        [reagent::BLOODMOSS, reagent::NIGHTSHADE]
    );
    assert_eq!(FIREBALL_SPELL.info.reagents, [reagent::BLACK_PEARL]);
    assert_eq!(
        (
            FIREBALL_SPELL.info.action,
            FIREBALL_SPELL.info.right_hand_effect
        ),
        (203, 9041)
    );
    assert_eq!(RECALL_SPELL.info.reagents.len(), 3);
    assert_eq!(RECALL_SPELL.info.amounts_vec(), [1, 1, 1]);
    let circle_of = |t: &'static SpellType| t.family::<MageryType>().map(|m| m.circle);
    assert_eq!(circle_of(&CLUMSY_SPELL), Some(SpellCircle::First));
    assert_eq!(circle_of(&FIREBALL_SPELL), Some(SpellCircle::Third));
    assert_eq!(circle_of(&RECALL_SPELL), Some(SpellCircle::Fourth));
    assert!(FIREBALL_SPELL.delayed_damage && !CLUMSY_SPELL.delayed_damage);
    assert!(
        RECALL_SPELL
            .interface::<muo_content::spells::targeting::RecallType>()
            .is_some()
    );
}

#[test]
fn mana_and_cast_delay_follow_the_circle() {
    let mut w = world();
    let c = w.new_mobile(&MOBILE_BASE);
    let clumsy = spell::new_spell(&mut w, &CLUMSY_SPELL, c, None);
    let fireball = spell::new_spell(&mut w, &FIREBALL_SPELL, c, None);
    let recall = spell::new_spell(&mut w, &RECALL_SPELL, c, None);
    assert_eq!((CLUMSY_SPELL.get_mana)(&w, clumsy), 4);
    assert_eq!((FIREBALL_SPELL.get_mana)(&w, fireball), 9);
    assert_eq!((RECALL_SPELL.get_mana)(&w, recall), 11);
    assert_eq!(circle(&w, recall), Some(SpellCircle::Fourth));
    // CastDelayBase = (3 + circle) * 0.25 s, Third being 2
    assert_eq!(
        (FIREBALL_SPELL.cast_delay_base)(&w, fireball).total_milliseconds(),
        1250.0
    );
    // Not AOS: 0.5 + 0.25 * circle.
    w.core.expansion = Some(Expansion::UOR);
    assert_eq!(
        (FIREBALL_SPELL.get_cast_delay)(&mut w, fireball).total_milliseconds(),
        1000.0
    );
    assert_eq!(
        (CLUMSY_SPELL.get_cast_delay)(&mut w, clumsy).total_milliseconds(),
        500.0
    );
}

#[test]
fn a_cast_by_a_mobile_with_no_mana_does_not_start() {
    let mut w = world();
    let c = w.new_mobile(&MOBILE_BASE);
    let s = spell::new_spell(&mut w, &CLUMSY_SPELL, c, None);
    assert!(!spell::cast(&mut w, s));
    assert_eq!(spell::state(&w, s), SpellState::None);
    assert!(w.mobile(c).unwrap().spell.is_none());
}

#[test]
fn disturbing_a_spell_that_is_not_casting_is_a_no_op() {
    let mut w = world();
    let c = w.new_mobile(&MOBILE_BASE);
    let s = spell::new_spell(&mut w, &FIREBALL_SPELL, c, None);
    spell::disturb(&mut w, s, DisturbType::Hurt, true, false);
    assert_eq!(spell::state(&w, s), SpellState::None);
}

// ---- the rest of the spells ----

use muo_content::spells::registry::Registered;

fn world_for(e: Expansion) -> World {
    let mut w = World::new();
    w.core.expansion = Some(e);
    configure(&mut w);
    w
}

fn id_of(w: &mut World, full_name: &str) -> i32 {
    registry(w).registry_number(full_name)
}

#[test]
fn every_circle_registers_under_the_initializer_ids() {
    let mut w = world_for(Expansion::AOS);
    for (name, id) in [
        ("Server.Spells.First.CreateFoodSpell", 1),
        ("Server.Spells.First.ReactiveArmorSpell", 6),
        ("Server.Spells.Second.ProtectionSpell", 14),
        ("Server.Spells.Third.WallOfStoneSpell", 23),
        ("Server.Spells.Fourth.ManaDrainSpell", 30),
        ("Server.Spells.Fifth.SummonCreatureSpell", 39),
        ("Server.Spells.Sixth.RevealSpell", 47),
        ("Server.Spells.Seventh.PolymorphSpell", 55),
        ("Server.Spells.Eighth.WaterElementalSpell", 63),
        ("Server.Spells.Necromancy.AnimateDeadSpell", 100),
        ("Server.Spells.Necromancy.WraithFormSpell", 115),
        ("Server.Spells.Chivalry.CleanseByFireSpell", 200),
        ("Server.Spells.Chivalry.SacredJourneySpell", 209),
    ] {
        assert_eq!(id_of(&mut w, name), id, "{name}");
    }
}

#[test]
fn the_expansion_gates_the_schools_as_the_initializer_does() {
    // AOS: no Exorcism, Bushido, Ninjitsu, Spellweaving or Mysticism.
    let mut aos = world_for(Expansion::AOS);
    assert_eq!(
        id_of(&mut aos, "Server.Spells.Necromancy.ExorcismSpell"),
        -1
    );
    assert_eq!(id_of(&mut aos, "Server.Spells.Bushido.Confidence"), -1);

    let mut se = world_for(Expansion::SE);
    assert_eq!(
        id_of(&mut se, "Server.Spells.Necromancy.ExorcismSpell"),
        116
    );
    assert_eq!(id_of(&mut se, "Server.Spells.Bushido.Evasion"), 402);
    assert_eq!(id_of(&mut se, "Server.Spells.Ninjitsu.Shadowjump"), 506);
    assert_eq!(
        id_of(&mut se, "Server.Spells.Spellweaving.ArcaneCircleSpell"),
        -1
    );
    // The moves are in the same table, as special moves.
    assert!(matches!(
        registry(&mut se).type_of(400),
        Some(Registered::Special(m)) if m.full_name == "Server.Spells.Bushido.HonorableExecution"
    ));
    assert!(matches!(
        registry(&mut se).type_of(401),
        Some(Registered::Spell(_))
    ));

    let mut ml = world_for(Expansion::ML);
    assert_eq!(
        id_of(&mut ml, "Server.Spells.Spellweaving.GiftOfLifeSpell"),
        614
    );
    assert_eq!(id_of(&mut ml, "Server.Spells.Mysticism.StoneFormSpell"), -1);

    let mut sa = world_for(Expansion::SA);
    assert_eq!(
        id_of(&mut sa, "Server.Spells.Mysticism.NetherCycloneSpell"),
        691
    );
    assert_eq!(
        id_of(&mut sa, "Server.Spells.Mysticism.SpellPlagueSpell"),
        689
    );
}

#[test]
fn every_magery_circle_has_its_mana_and_a_unique_class() {
    use muo_content::spells::registry::Registered;
    let mut w = world_for(Expansion::AOS);
    let c = w.new_mobile(&MOBILE_BASE);
    let mut names = std::collections::HashSet::new();
    for id in 0..64 {
        let Some(Registered::Spell(t)) = registry(&mut w).type_of(id) else {
            panic!("{id} is not registered");
        };
        assert!(names.insert(t.full_name), "{} twice", t.full_name);
        let sp = spell::new_spell(&mut w, t, c, None);
        assert_eq!(
            (t.get_mana)(&w, sp),
            [4, 6, 9, 11, 14, 20, 40, 50][(id / 8) as usize],
            "{}",
            t.full_name
        );
    }
}

#[test]
fn blood_oath_formulas() {
    use crate::necromancy::blood_oath::{compute_reflected_damage, get_duration_seconds};
    assert_eq!(get_duration_seconds(100.0, 60.0), 13.0);
    assert_eq!(compute_reflected_damage(100, 80.0, false), 100);
    // (80 * 0.5 + 10) / 100 = 50% resisted
    assert_eq!(compute_reflected_damage(100, 80.0, true), 50);
}

#[test]
fn animal_form_entries_match_the_table() {
    use crate::ninjitsu::animal_form::ENTRIES;
    assert_eq!(ENTRIES.len(), 16);
    assert_eq!(
        (
            ENTRIES[0].type_name,
            ENTRIES[0].req_skill,
            ENTRIES[0].body_mod
        ),
        ("Kirin", 100.0, 0x84)
    );
    assert!(ENTRIES[10].stealth_bonus && !ENTRIES[10].speed_boost);
    assert!(ENTRIES[13].stealing_bonus);
}

#[test]
fn the_items_construct_and_round_trip() {
    use muo_io::{SaveReader, SaveWriter};
    use muo_server::item::Item;

    fn save(w: &World, id: muo_server::ids::ItemId) -> Vec<u8> {
        let ty = w.item(id).unwrap().ty;
        let mut out = SaveWriter::new(true);
        (ty.serialize)(w, id, &mut out);
        out.into_vec()
    }

    let mut w = world();
    let focus = (crate::spellweaving::items::ARCANE_FOCUS.construct.unwrap())(&mut w);
    assert_eq!(crate::spellweaving::items::strength_bonus(&w, focus), 1);
    let bytes = save(&w, focus);
    let mut w2 = world();
    let id2 = w2.add_item(Item::new_for_load(
        muo_primitives::Serial::new(0x4000_0001),
        &crate::spellweaving::items::ARCANE_FOCUS,
    ));
    let mut r = SaveReader::new(&bytes);
    (crate::spellweaving::items::ARCANE_FOCUS.deserialize)(&mut w2, id2, &mut r).unwrap();
    assert_eq!(r.remaining(), 0);
    assert_eq!(crate::spellweaving::items::strength_bonus(&w2, id2), 1);
    assert_eq!(save(&w2, id2), bytes);
}

#[test]
fn the_field_items_save_their_version_one_layout() {
    use muo_io::SaveWriter;
    // Version 1 of FireFieldItem: base item, version, damage, caster serial (zero), anchored end.
    let mut w = world();
    let id = w.new_item(&crate::fourth::fire_field::FIRE_FIELD_ITEM);
    let mut out = SaveWriter::new(true);
    (crate::fourth::fire_field::FIRE_FIELD_ITEM.serialize)(&w, id, &mut out);
    let mut paralyze = SaveWriter::new(true);
    let p = w.new_item(&crate::sixth::paralyze_field::PARALYZE_FIELD);
    (crate::sixth::paralyze_field::PARALYZE_FIELD.serialize)(&w, p, &mut paralyze);
    // The fire field writes one int more (the damage) than the paralyze field.
    assert_eq!(out.into_vec().len(), paralyze.into_vec().len() + 4);
}
