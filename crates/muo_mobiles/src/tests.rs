//! Every class of this crate: names are unique, it constructs, and its save round-trips.

use std::collections::HashSet;

use muo_content::creature::speeds::{NpcSpeeds, SpeedClassEntry};
use muo_data::expansion::Expansion;
use muo_io::{SaveReader, SaveWriter};
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::mobile::MobileType;

use crate::{animals, monsters};

fn all() -> Vec<(&'static MobileType, &'static [&'static str])> {
    let mut v = monsters::leaves::ALL.to_vec();
    v.extend_from_slice(animals::leaves::ALL);
    v.extend(crate::classes::all().into_iter().map(|t| (t, &[][..])));
    v
}

fn world(era: Option<Expansion>) -> World {
    let mut w = World::new();
    // the creature template keys (`creatureSetup`, ...): a bare registry would skip them
    crate::template_keys::register(&mut w.types);
    w.core.expansion = era;
    // Every class takes the table's "Medium" speeds (Data/npc-speeds.json is not loaded in a test world).
    w.resources
        .get_mut::<NpcSpeeds>()
        .register(SpeedClassEntry {
            level: "Medium".into(),
            active: 0.2,
            passive: 0.4,
            active_move: 0.0,
            passive_move: 0.0,
            types: vec![],
        });
    w
}

fn save(w: &World, id: MobileId) -> Vec<u8> {
    let ty = w.mobile(id).unwrap().ty;
    let mut out = SaveWriter::new(true);
    (ty.serialize)(w, id, &mut out);
    out.into_vec()
}

#[test]
fn names_are_unique_and_registered() {
    let mut names = HashSet::new();
    let list = all();
    for (t, aliases) in &list {
        assert!(names.insert(t.full_name), "{} twice", t.full_name);
        for a in *aliases {
            assert!(names.insert(*a), "alias {a} twice");
        }
        assert!(t.full_name.starts_with("Server.Mobiles."));
    }
    let mut types = muo_server::type_registry::TypeRegistry::default();
    crate::register(&mut types);
    assert!(types.mobile_types().len() >= list.len());
}

fn enraged_ctor(name: &str) -> Option<fn(&mut World, MobileId) -> MobileId> {
    use crate::classes::enraged::*;
    Some(match name {
        "Server.Mobiles.EnragedRabbit" => new_enraged_rabbit,
        "Server.Mobiles.EnragedHart" => new_enraged_hart,
        "Server.Mobiles.EnragedHind" => new_enraged_hind,
        "Server.Mobiles.EnragedBlackBear" => new_enraged_black_bear,
        "Server.Mobiles.EnragedEagle" => new_enraged_eagle,
        _ => return None,
    })
}

#[test]
fn every_class_constructs_and_round_trips() {
    for era in [None, Some(Expansion::AOS), Some(Expansion::ML)] {
        for (ty, _) in all() {
            let mut w = world(era);
            let id = match ty.construct {
                Some(c) => c(&mut w),
                // The enraged animals take their summoner: `new EnragedHart(summoner)`.
                None => {
                    let Some(make) = enraged_ctor(ty.full_name) else {
                        continue;
                    };
                    let summoner = (monsters::leaves::LIZARDMAN.construct.unwrap())(&mut w);
                    make(&mut w, summoner)
                }
            };
            assert_eq!(w.mobile(id).unwrap().ty.full_name, ty.full_name);
            assert!(
                muo_content::creature::is_creature(&w, id),
                "{}",
                ty.full_name
            );
            let bytes = save(&w, id);

            let mut w2 = world(era);
            let other = w2.new_mobile(ty);
            let mut r = SaveReader::new(&bytes);
            (ty.deserialize)(&mut w2, other, &mut r)
                .unwrap_or_else(|e| panic!("{}: {e}", ty.full_name));
            assert_eq!(r.remaining(), 0, "{} is read to its end", ty.full_name);
            let again = save(&w2, other);
            // A worn item is saved as a reference, which a world without the items cannot resolve.
            if w.mobile(id).unwrap().items.is_empty() {
                assert_eq!(bytes.len(), again.len(), "{}", ty.full_name);
            }
        }
    }
}

#[test]
fn a_leaf_saves_its_version_after_the_parent() {
    // Lizardman: [SerializationGenerator(0, false)] over BaseCreature: one int version 0.
    let mut w = world(None);
    let id = (monsters::leaves::LIZARDMAN.construct.unwrap())(&mut w);
    let bytes = save(&w, id);
    let mut base = SaveWriter::new(true);
    (muo_content::creature::BASE_CREATURE.serialize)(&w, id, &mut base);
    let mut want = base.into_vec();
    want.extend([0, 0, 0, 0]);
    assert_eq!(bytes, want);
}

#[test]
fn lizardman_constructor_matches_the_expected_values() {
    let mut w = world(Some(Expansion::AOS));
    let id = (monsters::leaves::LIZARDMAN.construct.unwrap())(&mut w);
    let m = w.mobile(id).unwrap();
    assert!(matches!(m.body.0, 35 | 36));
    assert_eq!(
        (m.fame, m.karma, m.virtual_armor, m.base_sound_id),
        (1500, -1500, 28, 417)
    );
    assert!(
        (96..=120).contains(&m.str) && (86..=105).contains(&m.dex) && (36..=60).contains(&m.int)
    );
    let c = muo_content::creature::cd(&w, id).unwrap();
    assert_eq!((c.damage_min, c.damage_max), (5, 7));
    assert_eq!(c.physical_damage, 100);
}

#[test]
fn horse_has_a_mount_item_and_saves_like_basemount() {
    use muo_content::creature::mount;
    let mut w = world(None);
    let id = (animals::leaves::HORSE.construct.unwrap())(&mut w);
    let item = mount::internal_item(&w, id).expect("the mount item");
    assert_eq!(mount::mount_of_item(&w, item), Some(id));
    let (body, item_id) = {
        let m = w.mobile(id).unwrap();
        (m.body.0, w.item(item).unwrap().item_id)
    };
    let table = [
        (0xC8, 0x3E9F),
        (0xE2, 0x3EA0),
        (0xE4, 0x3EA1),
        (0xCC, 0x3EA2),
    ];
    assert!(table.contains(&(body, item_id)), "{body:x} {item_id:x}");
    // BaseCreature part, then version 0, _nextMountAbility, _rider, _internalItem, then Horse's version 0.
    let bytes = save(&w, id);
    let mut base = SaveWriter::new(true);
    (muo_content::creature::BASE_CREATURE.serialize)(&w, id, &mut base);
    let base = base.into_vec();
    assert_eq!(&bytes[..base.len()], &base[..]);
    let tail = &bytes[base.len()..];
    assert_eq!(&tail[..4], &[0, 0, 0, 0]);
    assert_eq!(tail.len(), 4 + 8 + 4 + 4 + 4);
    assert_eq!(&tail[12..16], &[0xFF; 4], "no rider");
    let mut want = SaveWriter::new(true);
    muo_server::serialization_ext::WriteEntityExt::write_item(&mut want, &w, Some(item));
    assert_eq!(&tail[16..20], &want.into_vec()[..]);
}

#[test]
fn riding_puts_the_mount_item_on_the_rider() {
    use muo_content::creature::mount;
    let mut w = world(None);
    let horse = (animals::leaves::HORSE.construct.unwrap())(&mut w);
    let rider = (monsters::leaves::LIZARDMAN.construct.unwrap())(&mut w);
    mount::set_rider(&mut w, horse, Some(rider));
    assert_eq!(mount::rider(&w, horse), Some(rider));
    assert_eq!(
        muo_server::mobile::part1::props_b::mount(&mut w, rider),
        None,
        "hooks are not configured in a bare world"
    );
    let item = mount::internal_item(&w, horse).unwrap();
    assert!(
        muo_server::mobile::part2::items::find_item_on_layer(
            &w,
            rider,
            muo_server::layer::Layer::Mount
        ) == Some(item)
    );
    mount::set_rider(&mut w, horse, None);
    assert_eq!(mount::rider(&w, horse), None);
}

#[test]
fn orc_constructs_with_its_speech_and_loot_table() {
    let mut w = world(Some(Expansion::AOS));
    let id = (monsters::leaves::ORC.construct.unwrap())(&mut w);
    let c = muo_content::creature::ctype(&w, id).unwrap();
    assert!((c.speech_type)(&w, id).is_some());
    assert_eq!(w.mobile(id).unwrap().body.0, 17);
    assert_eq!(
        w.mobile(id).unwrap().speech_hue,
        muo_content::misc::inhuman_speech::ORC.hue
    );
}

#[test]
fn golem_scalar_and_summoned_constructor_match_the_expected_values() {
    let mut w = world(Some(Expansion::AOS));
    let id = crate::classes::misc4::new_golem(&mut w, true, 0.5);
    let m = w.mobile(id).unwrap();
    assert_eq!((m.body.0, m.hue, m.fame, m.karma), (752, 2101, 10, 10));
    assert!((125..=175).contains(&m.str)); // (int)(251 * 0.5) ..= (int)(350 * 0.5)
    let c = muo_content::creature::cd(&w, id).unwrap();
    assert_eq!((c.damage_min, c.damage_max, c.control_slots), (6, 12, 3));
}

#[test]
fn hell_steed_and_grizzled_mare_share_the_set_stats() {
    let mut w = world(Some(Expansion::AOS));
    let ty = crate::monsters::leaves::ALL
        .iter()
        .chain(crate::animals::leaves::ALL.iter())
        .map(|(t, _)| *t);
    let mut found = 0;
    for t in ty.filter(|t| {
        t.full_name == "Server.Mobiles.HellSteed" || t.full_name == "Server.Mobiles.GrizzledMare"
    }) {
        let id = (t.construct.unwrap())(&mut w);
        let m = w.mobile(id).unwrap();
        assert!((201..=210).contains(&m.str), "{}", t.full_name);
        assert_eq!((m.fame, m.karma), (0, 0));
        found += 1;
    }
    assert_eq!(found, 2);
}

fn item_save(w: &World, id: muo_server::ids::ItemId) -> Vec<u8> {
    let ty = w.item(id).unwrap().ty;
    let mut out = SaveWriter::new(true);
    (ty.serialize)(w, id, &mut out);
    out.into_vec()
}

#[test]
fn plague_beast_saves_its_three_fields_after_the_parent() {
    let mut w = world(None);
    let id = (monsters::leaves::PLAGUE_BEAST.construct.unwrap())(&mut w);
    let bytes = save(&w, id);
    let mut base = SaveWriter::new(true);
    (muo_content::creature::BASE_CREATURE.serialize)(&w, id, &mut base);
    let want = base.into_vec();
    assert_eq!(&bytes[..want.len()], &want[..]);
    let tail = &bytes[want.len()..];
    // version 0, _hasMetalChest false, _totalDevoured 0, _devourGoal 15..=25
    assert_eq!(&tail[..4], &[0, 0, 0, 0]);
    assert_eq!(tail[4], 0);
    assert_eq!(&tail[5..9], &[0, 0, 0, 0]);
    let goal = i32::from_le_bytes(tail[9..13].try_into().unwrap());
    assert!((15..=25).contains(&goal), "{goal}");
    assert_eq!(tail.len(), 13);
}

#[test]
fn stained_ooze_round_trips_and_reads_its_corrosive_flag() {
    let mut w = world(None);
    let id = crate::classes::ilhenir::new_stained_ooze(&mut w, true);
    let bytes = item_save(&w, id);
    assert_eq!(bytes[bytes.len() - 5..], [0, 0, 0, 0, 1]);
    let mut w2 = world(None);
    let other = w2.new_item(&crate::classes::ilhenir::STAINED_OOZE);
    let mut r = SaveReader::new(&bytes);
    (crate::classes::ilhenir::STAINED_OOZE.deserialize)(&mut w2, other, &mut r).unwrap();
    assert_eq!(r.remaining(), 0);
    assert_eq!(item_save(&w2, other), bytes);
}

#[test]
fn revenants_are_never_saved_and_hold_their_target() {
    let mut w = world(None);
    let caster = (monsters::leaves::LIZARDMAN.construct.unwrap())(&mut w);
    let target = (monsters::leaves::LIZARDMAN.construct.unwrap())(&mut w);
    let rev = crate::classes::monsters5::new_revenant(
        &mut w,
        caster,
        target,
        muo_primitives::TimeSpan::from_secs(30),
    );
    let ty = w.mobile(rev).unwrap().ty;
    assert!((ty.skip_serialization)(&w, rev));
    let k = crate::classes::monsters5::new_khaldun_revenant(&mut w, target);
    let kty = w.mobile(k).unwrap().ty;
    assert!((kty.skip_serialization)(&w, k));
    assert_eq!(w.mobile(k).unwrap().hue, 0x41CE);
}

#[test]
fn changeling_saves_its_morph_target_and_paragons_take_their_hue() {
    let mut w = world(None);
    let id = (monsters::leaves::CHANGELING.construct.unwrap())(&mut w);
    let bytes = save(&w, id);
    // version 0 and a null mobile reference
    assert_eq!(&bytes[bytes.len() - 8..bytes.len() - 4], &[0, 0, 0, 0]);
    let guile = (monsters::leaves::GUILE.construct.unwrap())(&mut w);
    assert_eq!(w.mobile(guile).unwrap().hue, 0x3F);
    let irk = (monsters::leaves::IRK.construct.unwrap())(&mut w);
    assert_eq!(w.mobile(irk).unwrap().hue, 0x489);
}

#[test]
fn bake_kitsune_starts_undisguised() {
    let mut w = world(None);
    let id = (monsters::leaves::BAKE_KITSUNE.construct.unwrap())(&mut w);
    assert_eq!(w.mobile(id).unwrap().body_mod.0, 0);
    let bytes = save(&w, id);
    let mut w2 = world(None);
    let other = w2.new_mobile(w.mobile(id).unwrap().ty);
    let mut r = SaveReader::new(&bytes);
    (w2.mobile(other).unwrap().ty.deserialize)(&mut w2, other, &mut r).unwrap();
    assert_eq!(r.remaining(), 0);
}

#[test]
fn an_ethereal_mount_saves_version_four_with_its_flags_and_the_subclass_version() {
    use crate::classes::ethereal::*;
    let mut w = world(None);
    let id = (ETHEREAL_HORSE.construct.unwrap())(&mut w);
    let bytes = item_save(&w, id);
    // ..base.., 4, flags (Steps: 0 is not StepsMax), mounted id, regular id, steps, subclass version
    let tail = &bytes[bytes.len() - 21..];
    assert_eq!(&tail[..4], &[4, 0, 0, 0]);
    assert_eq!(tail[4], 0x08);
    assert_eq!(i32::from_le_bytes(tail[5..9].try_into().unwrap()), 0x3EAA);
    assert_eq!(i32::from_le_bytes(tail[9..13].try_into().unwrap()), 0x20DD);
    assert_eq!(&tail[13..], &[0u8; 8]); // steps 0, subclass version 0

    let mut w2 = world(None);
    let other = w2.new_item(&ETHEREAL_HORSE);
    let mut r = SaveReader::new(&bytes);
    (ETHEREAL_HORSE.deserialize)(&mut w2, other, &mut r).unwrap();
    assert_eq!(r.remaining(), 0);
    assert_eq!(item_save(&w2, other), bytes);

    // the encoded subclass version of the later classes
    let bear = (RIDEABLE_POLAR_BEAR.construct.unwrap())(&mut w);
    let b = item_save(&w, bear);
    assert_eq!(b.len(), bytes.len() - 3);
}

#[test]
fn ilhenir_and_meraktus_use_the_champion_lists() {
    let il = crate::npcs::special::champions::info(&crate::classes::ilhenir::ILHENIR).unwrap();
    assert!(il.unique.is_empty() && il.shared.len() == 4);
    let me = crate::npcs::special::champions::info(&crate::classes::ilhenir::MERAKTUS).unwrap();
    assert_eq!(me.unique, &["Server.Items.Subdue"]);
}

/// `IsParagon = true` in a constructor runs `Paragon.Convert` (a boss's speeds are divided by the buff, its hue is the
/// paragon one until the class sets its own), not just the flag.
#[test]
fn a_paragon_boss_is_converted_by_its_constructor() {
    let mut w = world(Some(Expansion::ML));
    crate::npcs::special::paragon::initialize(&mut w);
    let id = (crate::classes::bedlam::LADY_MARAI.construct.unwrap())(&mut w);
    let d = muo_content::creature::read(&w, id, (0.0, 0.0, false), |d| {
        (d.passive_speed, d.active_speed, d.is_paragon)
    });
    assert!(d.2);
    assert!(
        (d.0 - 0.4 / 1.2).abs() < 1e-9 && (d.1 - 0.2 / 1.2).abs() < 1e-9,
        "{d:?}"
    );
    assert_eq!(w.mobile(id).unwrap().hue, 0x21); // the class's own Hue, set after the conversion
}

/// The classes whose constructor is their `Data/Templates/Mobiles` template: (static name, full name) read from the
/// source, so a new templated constructor is covered without listing it here.
fn templated_classes() -> Vec<(String, String)> {
    let mut out = Vec::new();
    for text in [
        include_str!("monsters/leaves.rs"),
        include_str!("animals/leaves.rs"),
    ] {
        let parts = text
            .split("cprops::new_creature(w, &")
            .skip(1)
            .chain(text.split("cprops::new_mount(w, &").skip(1));
        for part in parts {
            let ident = &part[..part.find(')').unwrap()];
            let head = format!("\n    {ident}, {ident}_C = \"");
            let at = text.find(&head).unwrap_or_else(|| panic!("{ident}")) + head.len();
            let full = &text[at..at + text[at..].find('"').unwrap()];
            out.push((ident.to_owned(), full.to_owned()));
        }
    }
    out
}

#[test]
fn every_templated_class_has_its_template() {
    let classes = templated_classes();
    assert!(classes.len() >= 190, "{}", classes.len());
    let mut w = world(None);
    let table = muo_content::templates::table(&mut w);
    let types = all();
    for (ident, full) in &classes {
        assert!(
            table.mobiles.contains_key(full.as_str()),
            "{ident}: no mobile template for {full}"
        );
        let ty = types.iter().find(|(t, _)| t.full_name == full).unwrap().0;
        let id = (ty.construct.unwrap())(&mut w);
        assert_ne!(
            w.mobile(id).unwrap().body.0,
            0,
            "{full}: the template sets a body"
        );
    }
}
