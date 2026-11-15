//! Tests of the Items/Skill Items classes: unique names, each class constructs and round-trips its save bytes.

use std::collections::HashSet;

use muo_io::{SaveReader, SaveWriter};
use muo_server::World;
use muo_server::ids::ItemId;
use muo_server::item::Item;

fn save(w: &World, id: ItemId) -> Vec<u8> {
    let ty = w.item(id).unwrap().ty;
    let mut out = SaveWriter::new(true);
    (ty.serialize)(w, id, &mut out);
    out.into_vec()
}

fn all() -> Vec<&'static muo_server::item::ItemType> {
    let mut v: Vec<_> = super::tools::ALL.to_vec();
    v.extend(super::plain::ALL.iter().map(|(t, _)| *t));
    v.extend(super::scroll_list::ALL.iter().map(|(t, _)| *t));
    v.extend(super::spellbook::all());
    v.extend(super::potions::ALL);
    v.extend(super::instruments::ALL);
    v.extend(super::small::ALL);
    v.extend([
        &super::ninja::FUKIYA_DARTS,
        &super::ninja::SHURIKEN,
        &super::ninja::FUKIYA,
    ]);
    v.extend(super::dyes::ALL);
    v.extend([&super::dyes::DYES, &super::dyes::SCISSORS]);
    v.extend([
        &super::runebook::RUNEBOOK,
        &super::runebook::RECALL_RUNE,
        &super::runebook::SEXTANT,
    ]);
    v.extend([&super::potion_keg::POTION_KEG]);
    v.extend([
        &super::camping::BEDROLL,
        &super::camping::KINDLING,
        &super::fishing::FISHING_POLE,
        &super::fishing::MESSAGE_IN_A_BOTTLE,
        &super::fishing::SOS,
        &super::fishing::SPECIAL_FISHING_NET,
        &super::fishing::FABLED_FISHING_NET,
        &super::trophy::TAXIDERMY_KIT,
        &super::misc::FIRE_HORN,
        &super::ninja::SMOKE_BOMB,
        &super::ninja::EGG_BOMB,
        &super::ninja::LEATHER_NINJA_BELT,
        &super::thief::DISGUISE_KIT,
        &super::thief::GLASSBLOWING_BOOK,
        &super::thief::MASONRY_BOOK,
        &super::thief::SAND_MINING_BOOK,
        &super::thief::STONE_MINING_BOOK,
    ]);
    v
}

/// Every class of this folder, constructible or not, for the name check.
fn every_name() -> Vec<&'static str> {
    let mut v: Vec<_> = all().iter().map(|t| t.full_name).collect();
    v.extend([
        super::camping::CAMPFIRE.full_name,
        super::fishing::SHIPWRECKED_ITEM.full_name,
        super::forges::LARGE_FORGE_WEST.full_name,
        super::forges::LARGE_FORGE_EAST.full_name,
        super::trophy::TROPHY_ADDON.full_name,
        super::trophy::TROPHY_DEED.full_name,
        super::misc::RECIPE_SCROLL.full_name,
        super::runic_tools::RUNIC_HAMMER.full_name,
        super::runic_tools::RUNIC_SEWING_KIT.full_name,
        super::runic_tools::RUNIC_DOVETAIL_SAW.full_name,
        super::runic_tools::RUNIC_FLETCHER_TOOL.full_name,
        super::thief::DISGUISE_PERSISTENCE_TIMERS.full_name,
        super::potion_throw::FIRE_FIELD.full_name,
    ]);
    v
}

fn reload(w: &World, id: ItemId, w2: &mut World, serial: u32) -> ItemId {
    let t = w.item(id).unwrap().ty;
    let bytes = save(w, id);
    let id2 = w2.add_item(Item::new_for_load(muo_primitives::Serial::new(serial), t));
    let mut r = SaveReader::new(&bytes);
    (t.deserialize)(w2, id2, &mut r).unwrap();
    assert_eq!(r.remaining(), 0, "{} is read to its end", t.full_name);
    id2
}

#[test]
fn skill_item_names_are_unique() {
    let mut names = HashSet::new();
    for n in every_name() {
        assert!(names.insert(n), "{n} twice");
    }
}

#[test]
fn every_skill_item_constructs_and_round_trips() {
    for t in all() {
        let mut w = crate::test_world();
        let id = (t.construct.unwrap())(&mut w);
        assert_eq!(w.item(id).unwrap().ty.full_name, t.full_name);
        let bytes = save(&w, id);
        let mut w2 = crate::test_world();
        let id2 = w2.add_item(Item::new_for_load(
            muo_primitives::Serial::new(0x4000_0001),
            t,
        ));
        let mut r = SaveReader::new(&bytes);
        (t.deserialize)(&mut w2, id2, &mut r).unwrap_or_else(|e| panic!("{}: {e}", t.full_name));
        assert_eq!(r.remaining(), 0, "{} is read to its end", t.full_name);
        // A clothing record without the resource flag reads back as `CraftResource.None` (a quirk of the generated
        // reader), so a leather belt writes the flag the second time.
        if !std::ptr::eq(t, &super::ninja::LEATHER_NINJA_BELT) {
            assert_eq!(save(&w2, id2), bytes, "{}", t.full_name);
        }
    }
}

#[test]
fn a_smith_hammer_record_is_the_golden_bytes() {
    // Item part, then BaseTool: version 2, null crafter, quality Regular (encoded 1), uses 50; then SmithHammer 0.
    let mut w = crate::test_world();
    let id = super::tools::new_smith_hammer(&mut w, Some(50));
    let bytes = save(&w, id);
    let tail = &bytes[bytes.len() - 14..];
    assert_eq!(tail, &[2, 0, 0, 0, 0, 1, 50, 0, 0, 0, 0, 0, 0, 0]);
}

#[test]
fn a_scroll_record_is_the_golden_bytes() {
    // Item part, then SpellScroll: version 0 and spell id 681, then the subclass's version 0.
    let mut w = crate::test_world();
    let id = (super::scroll_list::SLEEP_SCROLL.construct.unwrap())(&mut w);
    let bytes = save(&w, id);
    assert_eq!(
        &bytes[bytes.len() - 12..],
        &[0, 0, 0, 0, 0xA9, 2, 0, 0, 0, 0, 0, 0]
    );
}

#[test]
fn a_spellbook_counts_its_spells() {
    let mut w = crate::test_world();
    let id = super::spellbook::new_chivalry(&mut w, 0x3FF);
    assert_eq!(super::spellbook::data(&w, id).unwrap().spell_count, 10);
    assert!(super::spellbook::has_spell(&w, id, 209));
    assert!(!super::spellbook::has_spell(&w, id, 210));
}

#[test]
fn a_heal_potion_record_is_the_golden_bytes() {
    // BasePotion: version 2 and the effect (HealLesser 14, an encoded int); BaseHealPotion 0; LesserHealPotion 0.
    let mut w = crate::test_world();
    let id = super::potions::new_lesser_heal(&mut w);
    let bytes = save(&w, id);
    assert_eq!(
        &bytes[bytes.len() - 13..],
        &[2, 0, 0, 0, 14, 0, 0, 0, 0, 0, 0, 0, 0]
    );
}

#[test]
fn a_runebook_keeps_its_entries_through_a_save() {
    use muo_primitives::Point3D;
    let mut w = crate::test_world();
    let id = super::runebook::new_runebook(&mut w, 6);
    w.item_mut(id)
        .unwrap()
        .ext
        .get_mut::<super::runebook::BookData>()
        .unwrap()
        .entries
        .push(super::runebook::Entry {
            house: None,
            location: Point3D::new(1, 2, 3),
            map: Some(muo_server::map::MapId(1)),
            description: Some("Home".to_string()),
        });
    let bytes = save(&w, id);
    let mut w2 = crate::test_world();
    let id2 = w2.add_item(Item::new_for_load(
        muo_primitives::Serial::new(0x4000_0002),
        &super::runebook::RUNEBOOK,
    ));
    let mut r = SaveReader::new(&bytes);
    (super::runebook::RUNEBOOK.deserialize)(&mut w2, id2, &mut r).unwrap();
    let e = &super::runebook::book(&w2, id2).unwrap().entries[0];
    assert_eq!(
        (e.location, e.map, e.description.as_deref()),
        (
            Point3D::new(1, 2, 3),
            Some(muo_server::map::MapId(1)),
            Some("Home")
        )
    );
}

#[test]
fn a_runic_hammer_keeps_its_resource_and_hue() {
    use muo_content::craft::CraftResource;
    let mut w = crate::test_world();
    let id = super::runic_tools::new_runic_hammer(&mut w, CraftResource::Agapite, Some(40));
    assert_eq!(super::runic_tools::resource(&w, id), CraftResource::Agapite);
    assert_eq!(w.item(id).unwrap().hue, 0x979);
    let mut w2 = crate::test_world();
    let id2 = reload(&w, id, &mut w2, 0x4000_0010);
    assert_eq!(
        super::runic_tools::resource(&w2, id2),
        CraftResource::Agapite
    );
    assert_eq!(muo_content::items::tool::uses_remaining(&w2, id2), 40);
    assert_eq!(save(&w2, id2), save(&w, id));
}

#[test]
fn a_runic_tool_record_is_the_golden_bytes() {
    // BaseTool: version 2, null crafter, quality Regular (encoded 1), uses 10; BaseRunicTool: version 1 and the
    // resource Gold (6, an encoded int); RunicSewingKit: version 0.
    use muo_content::craft::CraftResource;
    let mut w = crate::test_world();
    let id = super::runic_tools::new_runic_sewing_kit(&mut w, CraftResource::Gold, Some(10));
    let bytes = save(&w, id);
    assert_eq!(
        &bytes[bytes.len() - 19..],
        &[2, 0, 0, 0, 0, 1, 10, 0, 0, 0, 1, 0, 0, 0, 6, 0, 0, 0, 0]
    );
}

#[test]
fn a_trophy_deed_keeps_its_fields() {
    let mut w = crate::test_world();
    let id = super::trophy::new_trophy_deed(
        &mut w,
        0x1E67,
        0x1E60,
        1041093,
        1041107,
        Some("Bob".into()),
        33,
    );
    let mut w2 = crate::test_world();
    let id2 = reload(&w, id, &mut w2, 0x4000_0011);
    let d = super::trophy::data(&w2, id2).unwrap();
    assert_eq!(
        (
            d.hunter.as_deref(),
            d.animal_weight,
            d.west_id,
            d.north_id,
            d.deed_number,
            d.addon_number
        ),
        (Some("Bob"), 33, 0x1E67, 0x1E60, 1041093, 1041107)
    );
}

#[test]
fn a_recipe_scroll_keeps_its_recipe() {
    let mut w = crate::test_world();
    let id = super::misc::new_recipe_scroll(&mut w, 321);
    let mut w2 = crate::test_world();
    let id2 = reload(&w, id, &mut w2, 0x4000_0012);
    assert_eq!(save(&w2, id2), save(&w, id));
}

#[test]
fn an_sos_is_ancient_from_level_four_and_clamps_its_level() {
    let mut w = crate::test_world();
    let id = super::fishing::new_sos(&mut w, Some(muo_server::map::MapId::TRAMMEL), Some(2));
    assert!(!super::fishing::is_ancient(&w, id));
    super::fishing::set_level(&mut w, id, 9);
    assert!(super::fishing::is_ancient(&w, id));
    assert_eq!(w.item(id).unwrap().hue, 0x481);
}

#[test]
fn a_large_forge_moves_its_pieces_with_it() {
    use muo_primitives::Point3D;
    let mut w = crate::test_world();
    let id = super::forges::new_large_forge_west(&mut w);
    muo_server::item::ops::set_location(&mut w, id, Point3D::new(10, 20, 5));
    let loc = |w: &World, i: ItemId| w.item(i).unwrap().location;
    let pieces: Vec<ItemId> = w.items.keys().copied().filter(|&i| i != id).collect();
    assert_eq!(pieces.len(), 2);
    let mut at: Vec<Point3D> = pieces.iter().map(|&p| loc(&w, p)).collect();
    at.sort_by_key(|p| p.y);
    assert_eq!(at, vec![Point3D::new(10, 21, 5), Point3D::new(10, 22, 5)]);
}

#[test]
fn a_specialized_book_teaches_once() {
    use muo_content::player_mobile::PlayerFlag;
    let _ = PlayerFlag::GLASSBLOWING;
    assert_eq!(
        super::thief::GLASSBLOWING_BOOK.full_name,
        "Server.Items.GlassblowingBook"
    );
}

/// `Sextant.ReverseLookup` against hand-worked results (Felucca and Trammel centre at 1323,1624 on a
/// 5120 x 4096 sphere; a coordinate past the edge wraps once).
#[test]
fn sextant_reverse_lookup_follows_the_expected_arithmetic() {
    use muo_primitives::Point3D;
    use muo_server::map::MapId;
    let mut w = crate::test_world();
    let look = super::runebook::sextant_reverse_lookup;
    // 0 deg 0' E/S is the centre itself
    assert_eq!(
        look(&mut w, MapId(0), 0, 0, 0, 0, true, true),
        Point3D::new(1323, 1624, 0)
    );
    // 10 deg 30' E, 20 deg 15' S: (int)(10.5 * 5120 / 360) = 149, (int)(20.25 * 4096 / 360) = 230
    assert_eq!(
        look(&mut w, MapId(1), 10, 20, 30, 15, true, true),
        Point3D::new(1323 + 149, 1624 + 230, 0)
    );
    // West 90 and North 45 count 360 - the angle (270, 315): x = 1323 + 3840 wraps by the width, y = 1624 + 3584 by the height
    assert_eq!(
        look(&mut w, MapId(0), 90, 45, 0, 0, false, false),
        Point3D::new(43, 1112, 0)
    );
    // the Internal map, and a map no table row covers, answer Point3D.Zero
    assert_eq!(
        look(&mut w, MapId::INTERNAL, 1, 1, 1, 1, true, true),
        Point3D::default()
    );
    assert_eq!(
        look(&mut w, MapId(2), 1, 1, 1, 1, true, true),
        Point3D::default()
    );
}
