//! Tests of the faction items and tables: save layouts against the generated serializers, and the order of the lists.

use muo_io::{SaveReader, SaveWriter};
use muo_server::World;
use muo_server::ids::ItemId;
use muo_server::item::ItemType;

use super::items::{self, FACTION_STONE, SIGIL, TOWN_STONE};
use super::tables::{FACTION_DEFS, TOWN_DEFS};
use super::traps;

fn bytes(w: &World, id: ItemId) -> Vec<u8> {
    let ty = w.item(id).expect("item").ty;
    let mut out = SaveWriter::new(true);
    (ty.serialize)(w, id, &mut out);
    out.as_slice().to_vec()
}

fn load(w: &mut World, ty: &'static ItemType, data: &[u8]) -> ItemId {
    let id = muo_server::item::ops::new_item_with_id(w, ty, 0x1);
    let mut r = SaveReader::new(data);
    (ty.deserialize)(w, id, &mut r).expect("deserialize");
    assert_eq!(r.remaining(), 0, "all bytes read");
    id
}

#[test]
fn pvp_factions_lists_are_in_the_compiled_type_order() {
    // The faction and town order saves refer to by index.
    let factions: Vec<_> = FACTION_DEFS.iter().map(|f| f.friendly_name).collect();
    assert_eq!(
        factions,
        [
            "Council of Mages",
            "Minax",
            "Shadowlords",
            "True Britannians"
        ]
    );
    let towns: Vec<_> = TOWN_DEFS.iter().map(|t| t.friendly_name).collect();
    assert_eq!(
        towns,
        [
            "Britain",
            "Magincia",
            "Minoc",
            "Moonglow",
            "Skara Brae",
            "Trinsic",
            "Vesper",
            "Yew"
        ]
    );
    assert_eq!(super::TRUE_BRITANNIANS, 3);
}

#[test]
fn pvp_factions_faction_stone_layout_and_round_trip() {
    let mut w = World::new();
    let id = items::new_faction_stone(&mut w, Some(1)); // Minax
    let b = bytes(&w, id);
    // ... BaseSystemController (version 1, label number), FactionStone version 0, faction reference index + 1.
    let mut tail = Vec::new();
    tail.extend_from_slice(&1i32.to_le_bytes());
    tail.extend_from_slice(&FACTION_DEFS[1].faction_stone_name.to_le_bytes());
    tail.extend_from_slice(&0i32.to_le_bytes());
    tail.push(2); // encoded int 2
    assert!(b.ends_with(&tail), "{b:?}");
    let back = load(&mut w, &FACTION_STONE, &b);
    assert_eq!(items::stone_faction(&w, back), Some(1));
}

#[test]
fn pvp_factions_town_stone_round_trip() {
    let mut w = World::new();
    let id = items::new_town_stone(&mut w, Some(5)); // Trinsic
    let b = bytes(&w, id);
    let back = load(&mut w, &TOWN_STONE, &b);
    assert_eq!(items::town_stone_town(&w, back), Some(5));
}

#[test]
fn pvp_factions_sigil_and_monolith_round_trip() {
    let mut w = World::new();
    let sigil = items::new_sigil(&mut w, Some(2));
    let mono = items::new_monolith(&mut w, true, Some(2), Some(0));
    items::monolith_set_sigil(&mut w, mono, Some(sigil));
    assert_eq!(
        items::sigil(&w, sigil).and_then(|d| d.last_monolith),
        Some(mono)
    );

    let b = bytes(&w, sigil);
    let back = load(&mut w, &SIGIL, &b);
    let d = items::sigil(&w, back).expect("sigil data");
    assert_eq!((d.town, d.corrupted, d.corrupting), (Some(2), None, None));
    assert_eq!(d.last_monolith, Some(mono));

    let b = bytes(&w, mono);
    let back = load(&mut w, &items::STRONGHOLD_MONOLITH, &b);
    assert_eq!(items::monolith_town(&w, back), Some(2));
    assert_eq!(items::monolith_faction(&w, back), Some(0));
    assert_eq!(items::monolith_sigil(&w, back), Some(sigil));
}

#[test]
fn pvp_factions_trap_deed_and_kit_layouts() {
    let mut w = World::new();
    let kit =
        muo_server::item::ops::new_item_with_id(&mut w, &traps::FACTION_TRAP_REMOVAL_KIT, 7867);
    w.item_mut(kit)
        .expect("kit")
        .ext
        .insert(traps::KitData { charges: 25 });
    let b = bytes(&w, kit);
    // version 0, then _charges as an encoded int
    assert!(b.ends_with(&[0, 0, 0, 0, 25]), "{b:?}");
    let back = load(&mut w, &traps::FACTION_TRAP_REMOVAL_KIT, &b);
    assert_eq!(
        w.item(back)
            .and_then(|i| i.ext.get::<traps::KitData>())
            .map(|k| k.charges),
        Some(25)
    );
}

#[test]
fn pvp_factions_storms_eye_asks_for_a_target_and_the_urn_needs_a_dead_faction_mate() {
    use muo_server::mobile::MOBILE_BASE;
    let mut w = World::new();
    muo_content::register(&mut w.types);
    muo_items::register(&mut w.types);
    let m = w.new_mobile(&MOBILE_BASE);
    muo_server::mobile::base::move_to_world(
        &mut w,
        m,
        muo_primitives::Point3D::new(100, 100, 0),
        muo_server::map::MapId::FELUCCA,
    );

    // Use() answers false (the eye is consumed by the target callback) and leaves a target pending.
    let eye = (super::power::STORMS_EYE.make.unwrap())(&mut w);
    assert!(w.item(eye).unwrap().movable());
    assert!(!super::power::power_use_for_test(&mut w, eye, m));
    assert!(w.mobile(m).unwrap().target.is_some());

    // A stone-still eye cannot be used at all.
    muo_server::target::cancel_mobile_target(&mut w, m);
    muo_server::item::ops::set_movable(&mut w, eye, false);
    assert!(!super::power::power_use_for_test(&mut w, eye, m));
    assert!(w.mobile(m).unwrap().target.is_none());

    // The urn: nobody dead in sight, nothing happens and it is not used up.
    let urn = (super::power::URN_OF_ASCENSION.make.unwrap())(&mut w);
    assert!(!super::power::power_use_for_test(&mut w, urn, m));
}

#[test]
fn pvp_factions_vendors_can_be_walked_through_from_ml_on() {
    use muo_data::expansion::Expansion;
    let mut w = World::new();
    muo_content::register(&mut w.types);
    muo_items::register(&mut w.types);
    let walker = w.new_mobile(&muo_server::mobile::MOBILE_BASE);
    let vendor = (super::mobiles::FACTION_BOARD_VENDOR.make.unwrap())(&mut w);
    let ty = w.mobile(vendor).unwrap().ty;
    w.core.expansion = Some(Expansion::ML);
    assert!((ty.on_move_over)(&mut w, vendor, walker));
    w.core.expansion = Some(Expansion::AOS);
    let base = (muo_content::vendor::BASE_VENDOR.on_move_over)(&mut w, vendor, walker);
    assert_eq!(
        (ty.on_move_over)(&mut w, vendor, walker),
        base,
        "before ML the base rule applies"
    );
}

#[test]
fn pvp_factions_guard_leaves_no_corpse_and_no_click_title() {
    let mut w = World::new();
    muo_content::register(&mut w.types);
    muo_items::register(&mut w.types);
    muo_mobiles::register(&mut w.types);
    crate::register(&mut w.types);
    let g =
        muo_content::factory::make_mobile(&mut w, "Server.Factions.FactionKnight").expect("guard");
    let ty = w.mobile(g).expect("guard").ty;
    assert!(!(ty.click_title)(&w, g));
    let c = muo_content::factory::make_item(&mut w, "Server.Items.Backpack").expect("container");
    (ty.on_death)(&mut w, g, Some(c));
    assert!(
        w.item(c).is_none_or(|i| i.deleted()),
        "the corpse is deleted"
    );
}

#[test]
fn sigil_delete_override_runs_once_and_returns_home() {
    use muo_server::item::{base, ops};
    let mut w = World::new();
    let sigil = items::new_sigil(&mut w, Some(2));
    let mono = items::new_monolith(&mut w, true, Some(2), Some(0));
    items::monolith_set_sigil(&mut w, mono, Some(sigil));
    // `Delete()` sends the sigil home while its monolith lives
    base::delete(&mut w, sigil);
    assert!(!ops::is_deleted(&w, sigil));
    // no monolith left: the base delete runs, once, and `OnAfterDelete` drops it from `Sigils`
    base::delete(&mut w, mono);
    base::delete(&mut w, sigil);
    assert!(ops::is_deleted(&w, sigil));
    assert!(!super::st(&w).is_some_and(|s| s.sigils.contains(&sigil)));
}
