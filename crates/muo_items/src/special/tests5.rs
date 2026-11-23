//! Tests of vet5, holiday3, anniv2, solen and mutation: every class constructs and round-trips its save bytes, and the
//! golden bytes of the small ones.

use muo_io::{SaveReader, SaveWriter};
use muo_primitives::Point3D;
use muo_server::World;
use muo_server::ids::ItemId;
use muo_server::item::{Item, ItemType};

fn save(w: &World, id: ItemId) -> Vec<u8> {
    let ty = w.item(id).unwrap().ty;
    let mut out = SaveWriter::new(true);
    (ty.serialize)(w, id, &mut out);
    out.into_vec()
}

fn round_trip(make: impl Fn(&mut World) -> ItemId) -> Vec<u8> {
    round_trip_inner(make, true)
}

/// For an addon: its components are other items, so the second world cannot resolve them; only the read is checked.
fn reads_to_end(make: impl Fn(&mut World) -> ItemId) {
    round_trip_inner(make, false);
}

fn round_trip_inner(make: impl Fn(&mut World) -> ItemId, same_bytes: bool) -> Vec<u8> {
    let mut w = crate::test_world();
    let id = make(&mut w);
    let ty: &'static ItemType = w.item(id).unwrap().ty;
    let bytes = save(&w, id);
    let mut w2 = crate::test_world();
    let id2 = w2.add_item(Item::new_for_load(
        muo_primitives::Serial::new(0x4000_0001),
        ty,
    ));
    let mut r = SaveReader::new(&bytes);
    (ty.deserialize)(&mut w2, id2, &mut r).unwrap_or_else(|e| panic!("{}: {e}", ty.full_name));
    assert_eq!(r.remaining(), 0, "{} is read to its end", ty.full_name);
    if same_bytes {
        assert_eq!(save(&w2, id2), bytes, "{}", ty.full_name);
    }
    bytes
}

/// The bytes after the `Item` part: everything `ITEM_BASE.serialize` wrote is the same for a fresh item, so the tail
/// is what the class adds.
fn tail(w_make: impl Fn(&mut World) -> ItemId, parent: &ItemType) -> Vec<u8> {
    let mut w = crate::test_world();
    let id = w_make(&mut w);
    let all = save(&w, id);
    let mut w2 = crate::test_world();
    let id2 = w2.add_item(Item::new_for_load(
        muo_primitives::Serial::new(0x4000_0002),
        &muo_server::item::ITEM_BASE,
    ));
    let _ = (id2, parent);
    let base_len = {
        let mut out = SaveWriter::new(true);
        (muo_server::item::ITEM_BASE.serialize)(&w, id, &mut out);
        out.into_vec().len()
    };
    all[base_len..].to_vec()
}

#[test]
fn constructed_classes_round_trip() {
    use super::{anniv2, holiday3, mutation, solen, vet5};
    round_trip(|w| vet5::weapon_engraving_tool_new(w, 10));
    round_trip(vet5::cannon_deed_new);
    round_trip(vet5::wall_banner_deed_new);
    reads_to_end(|w| vet5::wall_banner_new(w, 7));
    reads_to_end(|w| vet5::cannon_addon_new(w, 2));
    round_trip(holiday3::holiday_timepiece_new);
    round_trip(holiday3::wrist_watch_new);
    round_trip(holiday3::holiday_tree_deed_new);
    // `_itemID` is 0 until music plays, and `AfterDeserialization` sets `ItemID = _itemID`: a fresh box loads as item 0.
    reads_to_end(anniv2::dawns_music_box_new);
    round_trip(|w| anniv2::dawns_music_gear_new_music(w, 18));
    round_trip(|w| anniv2::enhanced_bandage_new(w, 3));
    round_trip(|w| anniv2::fountain_of_life_new(w, 10));
    round_trip(|w| anniv2::fountain_of_life_deed_new(w, 7));
    round_trip(solen::bag_of_sending_new);
    round_trip(solen::ball_of_summoning_new);
    round_trip(solen::bracelet_of_binding_new);
    round_trip(mutation::plague_beast_mutation_core_new);
    round_trip(|w| mutation::plague_beast_component_new(w, 0x1CF0, 0x2B, true));
    round_trip(|w| mutation::plague_beast_vein_new(w, 0x1B1B, 0x2B));
    round_trip(mutation::plague_beast_rubble_organ_new);
    round_trip(mutation::plague_beast_backup_organ_new);
    round_trip(mutation::plague_beast_main_organ_new);
    round_trip(mutation::plague_beast_maiden_organ_new);
    round_trip(mutation::plague_beast_rock_organ_new);
    round_trip(mutation::backpack_new);
}

#[test]
fn golden_bytes() {
    use super::{anniv2, vet5};
    // [SerializationGenerator(0, false)]: int version 0, UsesRemaining, IsRewardItem
    assert_eq!(
        tail(
            |w| vet5::weapon_engraving_tool_new(w, 10),
            &muo_server::item::ITEM_BASE
        ),
        [0, 0, 0, 0, 10, 0, 0, 0, 0]
    );
    // [SerializationGenerator(0)]: encoded version, Charges, IsRewardItem
    assert!(tail(vet5::cannon_deed_new, &muo_server::item::ITEM_BASE).len() >= 6);
    // [SerializationGenerator(2)]: encoded version, Music (enum int)
    assert_eq!(
        tail(
            |w| anniv2::dawns_music_gear_new_music(w, 18),
            &muo_server::item::ITEM_BASE
        ),
        [2, 18]
    );
}

#[test]
fn track_tables() {
    use super::anniv2;
    assert_eq!(
        anniv2::get_tracks(anniv2::DawnsMusicRarity::Common).len(),
        40
    );
    assert_eq!(
        anniv2::get_tracks(anniv2::DawnsMusicRarity::Uncommon).len(),
        12
    );
    assert_eq!(anniv2::get_tracks(anniv2::DawnsMusicRarity::Rare).len(), 3);
    for r in [
        anniv2::DawnsMusicRarity::Common,
        anniv2::DawnsMusicRarity::Uncommon,
        anniv2::DawnsMusicRarity::Rare,
    ] {
        for t in anniv2::get_tracks(r) {
            assert_eq!(anniv2::get_info(*t as i32).map(|i| i.1), Some(r));
        }
    }
    let _ = Point3D::default();
}

#[test]
fn fixed_organs_lay_out_their_components() {
    use super::mutation;
    type Make = fn(&mut muo_server::World) -> muo_server::ids::ItemId;
    type Layout = &'static [(i32, i32, i32, i32)];
    let backup: Layout = &[
        (0x1B1B, 0x42, 16, 39),
        (0x1B1B, 0x42, 39, 49),
        (0x1B1B, 0x42, 39, 48),
        (0x1B1B, 0x42, 44, 42),
        (0x1CF2, 0x42, 20, 34),
        (0x135F, 0x42, 47, 58),
        (0x1360, 0x42, 70, 68),
    ];
    let main: Layout = &[
        (0x1B1B, 0x42, -36, -2),
        (0x1FB3, 0x42, -42, 0),
        (0x9DF, 0x42, -53, -7),
        (0x1B1C, 0x54, 29, 9),
        (0x1D06, 0x54, 18, -2),
        (0x9DF, 0x54, 36, -1),
        (0x1D10, 0x2B, -36, 47),
        (0x1B1C, 0x2B, -24, 62),
        (0x9DF, 0x2B, -41, 74),
        (0x1B1B, 0x60, 39, 56),
        (0x1FB4, 0x60, 34, 52),
        (0x9DF, 0x60, 45, 71),
        (0x1351, 0x15, 23, 0),
        (0x134F, 0x15, -22, 0),
        (0x1350, 0x15, 0, 0),
    ];
    let cases: [(Make, Layout); 2] = [
        (mutation::plague_beast_backup_organ_new, backup),
        (mutation::plague_beast_main_organ_new, main),
    ];
    for (make, want) in cases {
        let mut w = crate::test_world();
        let o = make(&mut w);
        let at = muo_primitives::Point3D::new(1000, 1000, 5);
        muo_server::item::ops::set_location(&mut w, o, at);
        // `Initialize()` runs on the next tick
        for _ in 0..4 {
            w.tick_count += muo_server::timer::TICK_RATE;
            muo_server::timer::slice(&mut w);
        }
        let comps = w
            .item(o)
            .unwrap()
            .ext
            .get::<mutation::Pb>()
            .unwrap()
            .components
            .clone();
        let got: Vec<_> = comps
            .iter()
            .map(|c| {
                let i = w.item(c.unwrap()).unwrap();
                (i.item_id, i.hue, i.location.x - at.x, i.location.y - at.y)
            })
            .collect();
        assert_eq!(got, want);
    }
}
