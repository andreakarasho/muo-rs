//! Maps: names, construction, save layout (`MapItem`:
//! version 1, `Rect2D`, width, height, protected, encoded pin count, pins, facet, editable).

use muo_io::{GenericWriter, SaveReader, SaveWriter};
use muo_primitives::Point2D;
use muo_server::World;
use muo_server::ids::ItemId;
use muo_server::item::{ITEM_BASE, Item, ItemType};

use super::*;

fn save(w: &World, id: ItemId) -> Vec<u8> {
    let ty = w.item(id).unwrap().ty;
    let mut out = SaveWriter::new(true);
    (ty.serialize)(w, id, &mut out);
    out.into_vec()
}

fn item_part(w: &World, id: ItemId) -> Vec<u8> {
    let mut out = SaveWriter::new(true);
    (ITEM_BASE.serialize)(w, id, &mut out);
    out.into_vec()
}

fn reload(w: &mut World, t: &'static ItemType, bytes: &[u8]) -> ItemId {
    let id = w.add_item(Item::new_for_load(Serial::new(0x4000_0300), t));
    let mut r = SaveReader::new(bytes);
    (t.deserialize)(w, id, &mut r).unwrap_or_else(|e| panic!("{}: {e}", t.full_name));
    assert_eq!(r.remaining(), 0, "{}", t.full_name);
    id
}

fn all() -> Vec<&'static ItemType> {
    ALL.iter()
        .copied()
        .chain([&treasure::TREASURE_MAP])
        .collect()
}

#[test]
fn names_are_unique_and_every_map_round_trips() {
    let mut seen = std::collections::HashSet::new();
    for t in all() {
        assert!(seen.insert(t.full_name), "{} twice", t.full_name);
        let mut w = World::new();
        let id = crate::misc::tests::construct_any(&mut w, t);
        assert_eq!(w.item(id).unwrap().ty.full_name, t.full_name);
        let bytes = save(&w, id);
        let again = reload(&mut w, t, &bytes);
        assert_eq!(save(&w, again), bytes, "{}", t.full_name);
    }
}

#[test]
fn city_map_record() {
    let mut w = World::new();
    let id = (CITY_MAP.construct.unwrap())(&mut w);
    let d = data(&w, id).unwrap().clone();
    assert_eq!((d.width, d.height), (400, 400));
    assert_eq!(
        (
            d.bounds.x(),
            d.bounds.y(),
            d.bounds.width(),
            d.bounds.height()
        ),
        (0, 0, 5119, 4095)
    );
    let mut expected = item_part(&w, id);
    expected.extend([1, 0, 0, 0]); // MapItem version
    let mut rect = SaveWriter::new(true);
    rect.write_rect2d(d.bounds);
    expected.extend(rect.into_vec());
    expected.extend(400i32.to_le_bytes());
    expected.extend(400i32.to_le_bytes());
    expected.push(0); // protected
    expected.push(0); // pin count (encoded)
    let mut facet = SaveWriter::new(true);
    facet.write_map(None);
    expected.extend(facet.into_vec());
    expected.push(0); // editable
    expected.extend([0, 0, 0, 0]); // CityMap version
    assert_eq!(save(&w, id), expected);
}

#[test]
fn pins_follow_the_edge_cases() {
    let mut w = World::new();
    let id = (MAP_ITEM.construct.unwrap())(&mut w);
    add_pin(&mut w, id, 1, 2);
    add_pin(&mut w, id, 3, 4);
    remove_pin(&mut w, id, 0); // `index > 0`: the first pin is never removed
    assert_eq!(data(&w, id).unwrap().pins.len(), 2);
    remove_pin(&mut w, id, 1);
    assert_eq!(data(&w, id).unwrap().pins, vec![Point2D::new(1, 2)]);
    insert_pin(&mut w, id, 9, 5, 6); // out of range appends
    assert_eq!(data(&w, id).unwrap().pins[1], Point2D::new(5, 6));
    assert_eq!(validate_xy(&w, id, -4, 999), (0, 199));
    // ConvertToMap scales Y by Width
    data_mut(&mut w, id).unwrap().bounds = Rectangle2D::new(100, 100, 400, 200);
    assert_eq!(convert_to_map(&w, id, 300, 200), (100, 100)); // (200 * 200 / 400, 100 * 200 / 200)
}

#[test]
fn preset_map_table() {
    let mut w = World::new();
    let id = preset_map_new(&mut w, &PRESET_MAP, 20);
    let d = data(&w, id).unwrap();
    assert_eq!(d.preset_label, 1041204);
    assert_eq!((d.bounds.width(), d.bounds.height()), (5199, 4095));
    assert_eq!((PRESET_MAP.label_number)(&w, id), 1041204);
    let blank = preset_map_new(&mut w, &PRESET_MAP, 99);
    assert_eq!(data(&w, blank).unwrap().preset_label, 0);
}

#[test]
fn old_version_loads_without_facet() {
    let mut w = World::new();
    let src = (MAP_ITEM.construct.unwrap())(&mut w);
    let mut bytes = item_part(&w, src);
    bytes.extend([0, 0, 0, 0]); // version 0
    let mut rect = SaveWriter::new(true);
    rect.write_rect2d(Rectangle2D::new(0, 0, 0, 0));
    bytes.extend(rect.into_vec());
    bytes.extend(200i32.to_le_bytes());
    bytes.extend(100i32.to_le_bytes());
    bytes.push(1); // protected
    bytes.extend(1i32.to_le_bytes()); // count is not encoded in version 0
    let mut pin = SaveWriter::new(true);
    pin.write_point2d(Point2D::new(3, 4));
    bytes.extend(pin.into_vec());
    let id = reload(&mut w, &MAP_ITEM, &bytes);
    let d = data(&w, id).unwrap();
    assert_eq!(
        (d.width, d.height, d.protected, d.facet),
        (200, 100, true, None)
    );
    assert_eq!(d.pins, vec![Point2D::new(3, 4)]);
}
