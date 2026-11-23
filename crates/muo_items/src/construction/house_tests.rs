//! A house placed through `HousePlacement` and built with the real door classes: its doors and sign exist.

use muo_multis::house::{self, access};
use muo_multis::placement::{self, HousePlacementResult};
use muo_multis::placement_entry;
use muo_multis::placement_tool::construct_house;
use muo_primitives::{Direction, Point3D};
use muo_server::World;
use muo_server::item::ops;
use muo_server::map::MapId;
use muo_server::mobile::{AccessLevel, MOBILE_BASE};

use super::doors;

#[test]
fn a_placed_house_builds_with_its_doors_and_sign() {
    let mut w = crate::test_world();
    doors::configure(&mut w);
    let from = w.new_mobile(&MOBILE_BASE);
    if let Some(m) = w.mobile_mut(from) {
        m.access_level = AccessLevel::GameMaster; // staff can place anywhere, so no map data is needed
        m.map = Some(MapId::FELUCCA);
    }
    let center = Point3D::new(1500, 1500, 0);
    let (res, to_move) = placement::check(&mut w, from, 0x64, center, Direction::SOUTH);
    assert_eq!(res, HousePlacementResult::Valid);
    assert!(to_move.is_empty());

    let entry = placement_entry::find(&w, "SmallOldHouse", 0x64).expect("entry");
    let h = construct_house(&mut w, entry, from).expect("house");
    ops::move_to_world_map(&mut w, h, center, Some(MapId::FELUCCA));

    let d = house::d(&w, h);
    assert_eq!(d.owner, Some(from));
    assert_eq!(d.doors.len(), 1, "SmallOldHouse has one door");
    let door = d.doors[0];
    assert!(doors::is_house_door(&w, door));
    assert!(doors::locked(&w, door) || doors::key_value(&w, door) != 0);
    // the door stands at the template's offset (0, 3, 7) from the house
    assert_eq!(w.item(door).unwrap().location, Point3D::new(1500, 1503, 7));
    let sign = d.sign.expect("sign");
    assert_eq!(w.item(sign).unwrap().location, Point3D::new(1502, 1504, 5));
    assert_eq!(access::get_houses(&w, from), vec![h]);
}

mod foundation {
    use muo_data::client::multi_data::{MultiComponentList, MultiTileEntry};
    use muo_data::tile_data::TileFlag;
    use muo_io::{SaveReader, SaveWriter};
    use muo_multis::foundation::{self as f, HOUSE_FOUNDATION};
    use muo_multis::house;
    use muo_primitives::Serial;

    use super::*;

    fn world() -> World {
        let mut w = crate::test_world();
        w.types.register_item(&crate::misc::basic::STATIC, &[]);
        doors::configure(&mut w);
        f::install(&mut w);
        // a 7x7 patch of floor as the foundation's multi
        let tiles: Vec<MultiTileEntry> = (-3..=3)
            .flat_map(|x| {
                (-3..=3).map(move |y| MultiTileEntry::new(0x31F4, x, y, 7, TileFlag::BACKGROUND))
            })
            .collect();
        w.maps
            .multi_data
            .set_components(0x13EC, MultiComponentList::from_entries(&tiles));
        w
    }

    fn save(w: &World, id: muo_server::ids::ItemId) -> Vec<u8> {
        let mut out = SaveWriter::new(true);
        (w.item(id).unwrap().ty.serialize)(w, id, &mut out);
        out.into_vec()
    }

    #[test]
    fn a_foundation_has_a_sign_a_hanger_and_an_empty_design() {
        let mut w = world();
        let owner = w.new_mobile(&MOBILE_BASE);
        let h = f::new_foundation(&mut w, owner, 0x13EC, 425, 3);
        assert!(f::is_foundation(&w, h));
        assert_eq!(house::components(&w, h).width(), 7);
        assert_eq!(
            house::components(&w, h).height(),
            8,
            "the stairs row is added"
        );
        let d = f::fd(&w, h).unwrap();
        assert!(d.sign_hanger.is_some(), "a Static 0xB98 hangs the sign");
        assert_eq!(w.item(d.sign_hanger.unwrap()).unwrap().item_id, 0xB98);
        assert!(house::d(&w, h).sign.is_some());
        assert_eq!(house::d(&w, h).max_lock_downs, 425);
        assert!(house::is_aos_rules(&w, h));
    }

    #[test]
    fn the_foundation_record_is_version_5_then_the_house_and_round_trips() {
        let mut w = world();
        let owner = w.new_mobile(&MOBILE_BASE);
        let h = f::new_foundation(&mut w, owner, 0x13EC, 425, 3);
        // a house at (0, 0, 0) on no map deletes itself when read
        ops::move_to_world_map(&mut w, h, Point3D::new(1500, 1500, 0), Some(MapId::FELUCCA));
        let bytes = save(&w, h);
        assert_eq!(
            bytes[0..4],
            5_i32.to_le_bytes(),
            "HouseFoundation writes its version before the house part"
        );
        let id = w.add_item(muo_server::item::Item::new_for_load(
            Serial::new(0x4000_0700),
            &HOUSE_FOUNDATION,
        ));
        let mut r = SaveReader::new(&bytes);
        (HOUSE_FOUNDATION.deserialize)(&mut w, id, &mut r).unwrap();
        assert_eq!(r.remaining(), 0);
        assert_eq!(save(&w, id), bytes);
        assert_eq!(house::components(&w, id).width(), 7);
    }

    #[test]
    fn customizing_and_committing_replaces_the_visible_design() {
        let mut w = world();
        let owner = w.new_mobile(&MOBILE_BASE);
        if let Some(m) = w.mobile_mut(owner) {
            m.access_level = AccessLevel::GameMaster; // no bank is charged
            m.map = Some(MapId::FELUCCA);
        }
        let h = f::new_foundation(&mut w, owner, 0x13EC, 425, 3);
        ops::move_to_world_map(&mut w, h, Point3D::new(1500, 1500, 0), Some(MapId::FELUCCA));
        let before = f::current_state(&mut w, h).components.list.len();

        f::begin_customize(&mut w, h, owner);
        assert!(f::find_context(&w, owner).is_some());
        assert_eq!(f::customizer(&w, h), Some(owner));
        assert!(f::design_bounds(&w, owner).is_some());
        assert!(
            !house::is_active(&w, h),
            "a house being customized is not active"
        );

        // the client builds a wall piece on the second floor
        f::edit(&mut w, h, |d, td| {
            let mut design = d.design.take().unwrap();
            design.components.add(0x6, 0, 0, 27, td);
            d.revise(&mut design);
            d.design = Some(design);
        });
        f::end_confirm_commit(&mut w, h, owner);
        assert!(f::find_context(&w, owner).is_none());
        assert_eq!(f::customizer(&w, h), None);
        assert_eq!(
            f::current_state(&mut w, h).components.list.len(),
            before + 1
        );
        assert!(house::is_active(&w, h));
    }

    #[test]
    fn the_design_packet_is_0xd8_and_sized() {
        let mut w = world();
        let owner = w.new_mobile(&MOBILE_BASE);
        let h = f::new_foundation(&mut w, owner, 0x13EC, 425, 3);
        let st = f::current_state(&mut w, h);
        let p = muo_multis::house_packets::create_house_design_state_detailed(
            &w,
            f::serial_of(&w, h),
            7,
            &st.components,
        );
        assert_eq!(p[0], 0xD8);
        assert_eq!(u16::from_be_bytes([p[1], p[2]]) as usize, p.len());
        assert_eq!(p[3], 0x03, "compression type");
        assert_eq!(p[9..13], 7_i32.to_be_bytes(), "the revision");
        assert_eq!(
            i16::from_be_bytes([p[13], p[14]]) as usize,
            st.components.list.len()
        );
    }

    #[test]
    fn stairs_and_fixtures_are_told_apart() {
        let mut dir = 0;
        assert!(f::is_stair(0x70A, &mut dir) && dir == 0);
        assert!(f::is_stair(0x70D, &mut dir) && dir == 3);
        assert!(f::is_stair_block(0x709) && !f::is_stair_block(0x70A));
        assert!(
            f::design_is_fixture(0x675)
                && f::design_is_fixture(0x181D)
                && !f::design_is_fixture(0x6F5)
        );
    }
}
