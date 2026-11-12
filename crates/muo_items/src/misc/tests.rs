//! Every plain Misc class constructs, has a unique full name and round-trips its save bytes; the save bytes are
//! `Item`'s part followed by the version.

use std::collections::HashSet;

use muo_io::{GenericWriter, SaveReader, SaveWriter};
use muo_server::World;
use muo_server::item::{ITEM_BASE, ItemType};

fn save(w: &World, id: muo_server::ids::ItemId) -> Vec<u8> {
    let ty = w.item(id).unwrap().ty;
    let mut out = SaveWriter::new(true);
    (ty.serialize)(w, id, &mut out);
    out.into_vec()
}

#[test]
fn leaf_names_are_unique() {
    let mut seen = HashSet::new();
    for (t, aliases) in super::leaves::ALL {
        assert!(seen.insert(t.full_name), "{} twice", t.full_name);
        for a in *aliases {
            assert!(seen.insert(a), "alias {a} twice");
        }
    }
}

#[test]
fn leaves_construct_and_round_trip() {
    for (t, _) in super::leaves::ALL {
        round_trip(t);
    }
}

/// Constructs `t`, saves it, loads the bytes into a fresh item and saves again.
pub(crate) fn round_trip(t: &'static ItemType) {
    let mut w = World::new();
    let id = (t
        .construct
        .unwrap_or_else(|| panic!("{} has no constructor", t.full_name)))(&mut w);
    assert_eq!(w.item(id).unwrap().ty.full_name, t.full_name);
    let bytes = save(&w, id);
    // Item's part, then the version: 4 bytes of int 0, or 1 byte of encoded 0.
    let mut base = SaveWriter::new(true);
    (ITEM_BASE.serialize)(&w, id, &mut base);
    let base = base.into_vec();
    assert!(bytes.starts_with(&base), "{}", t.full_name);
    assert!(matches!(bytes.len() - base.len(), 1 | 4), "{}", t.full_name);
    assert!(
        bytes[base.len()..].iter().all(|b| *b == 0),
        "{}",
        t.full_name
    );

    let loaded = w.add_item(muo_server::item::Item::new_for_load(
        muo_primitives::Serial::new(0x4000_0100),
        t,
    ));
    let mut r = SaveReader::new(&bytes);
    (t.deserialize)(&mut w, loaded, &mut r).unwrap_or_else(|e| panic!("{}: {e}", t.full_name));
    assert_eq!(save(&w, loaded), bytes, "{}", t.full_name);
}

/// `new T(...)` for a class that has only `[Constructible]` overloads: the first of its table, with these arguments.
pub(crate) fn construct_any(w: &mut World, t: &'static ItemType) -> muo_server::ids::ItemId {
    use muo_content::props::Value;
    if let Some(c) = t.construct {
        return c(w);
    }
    super::basic::basic_configure(w);
    super::gadgets::gadgets_configure(w);
    super::moongates::moongates_tables(w);
    crate::maps::ctors_configure(w);
    crate::guilds::guilds_configure(w);
    let args: Vec<Value> = match t.full_name {
        "Server.Items.Static" | "Server.Items.LocalizedStatic" => vec![Value::Int(0x80)],
        "Server.Items.MorphItem" => vec![Value::Int(1), Value::Int(2), Value::Int(3)],
        "Server.Items.PresetMap" => vec![Value::Enum(0)],
        "Server.Items.Moonstone" => vec![Value::Enum(1)],
        "Server.Items.TreasureMap" => vec![
            Value::Int(1),
            Value::Map(Some(muo_server::map::MapId::FELUCCA)),
        ],
        "Server.Items.WarningItem" => vec![Value::Int(0x80), Value::Int(5), Value::Int(1000000)],
        "Server.Items.HintItem" => vec![
            Value::Int(0x80),
            Value::Int(5),
            Value::Int(1000000),
            Value::Int(1000001),
        ],
        _ => Vec::new(),
    };
    let ctors = muo_content::props::ctors_of(w, t.full_name)
        .unwrap_or_else(|| panic!("{} has no constructor", t.full_name));
    match (ctors[0].make)(w, &args) {
        muo_server::ids::EntityId::Item(i) => i,
        _ => panic!("{} is not an item", t.full_name),
    }
}

/// Round trip of a class with fields: construct, save, load, save again and compare.
pub(crate) fn round_trip_any(t: &'static ItemType) -> Vec<u8> {
    let mut w = World::new();
    let id = construct_any(&mut w, t);
    let bytes = save(&w, id);
    let loaded = w.add_item(muo_server::item::Item::new_for_load(
        muo_primitives::Serial::new(0x4000_0100),
        t,
    ));
    let mut r = SaveReader::new(&bytes);
    (t.deserialize)(&mut w, loaded, &mut r).unwrap_or_else(|e| panic!("{}: {e}", t.full_name));
    assert_eq!(r.remaining(), 0, "{}", t.full_name);
    assert_eq!(save(&w, loaded), bytes, "{}", t.full_name);
    bytes
}

#[test]
fn hand_written_classes_round_trip_and_are_unique() {
    let mut seen: HashSet<&str> = super::leaves::ALL
        .iter()
        .map(|(t, _)| t.full_name)
        .collect();
    for (t, _) in super::basic::ALL {
        assert!(seen.insert(t.full_name), "{} twice", t.full_name);
        // A blank MorphItem has ItemID 1, which Refresh (run when it loads) turns invisible.
        if t.full_name != "Server.Items.MorphItem" {
            round_trip_any(t);
        }
    }
}

fn item_part_len(t: &'static ItemType) -> usize {
    let mut w = World::new();
    let id = construct_any(&mut w, t);
    let mut base = SaveWriter::new(true);
    (ITEM_BASE.serialize)(&w, id, &mut base);
    base.into_vec().len()
}

#[test]
fn basic_save_layouts() {
    use super::basic::*;
    // MorphItem: version int, OutsideRange, InactiveItemId, ActiveItemId, InsideRange.
    let mut w = World::new();
    let id = morph_item_new(&mut w, &MORPH_ITEM, 0x1B2D, 0x1B2E, 2, 3);
    let mut out = SaveWriter::new(true);
    (MORPH_ITEM.serialize)(&w, id, &mut out);
    let b = out.into_vec();
    let mut base = SaveWriter::new(true);
    (ITEM_BASE.serialize)(&w, id, &mut base);
    let base_len = base.into_vec().len();
    let mut tail = vec![0u8; 4];
    for v in [3, 0x1B2D, 0x1B2E, 2] {
        tail.extend(i32::to_le_bytes(v));
    }
    assert_eq!(&b[base_len..], &tail[..]);
    // Guillotine and LocalizedStatic write an encoded version; LocalizedStatic then its Number.
    let b = round_trip_any(&GUILLOTINE);
    assert_eq!(b[item_part_len(&GUILLOTINE)..], [0u8]);
    let b = round_trip_any(&LOCALIZED_STATIC);
    let tail = &b[item_part_len(&LOCALIZED_STATIC)..];
    assert_eq!(tail[..5], [0, 0, 0, 0, 0]); // Static's int version, then LocalizedStatic's encoded one
    assert_eq!(tail.len(), 5 + 3); // Number 1020000 + 0x80 as a 3 byte encoded int
    // WayPoint: version, NextPoint (serial -1... a null entity is 0 serial as written by write_item).
    let b = round_trip_any(&WAY_POINT);
    assert_eq!(b.len() - item_part_len(&WAY_POINT), 4 + 4);
}

#[test]
fn teleporters_round_trip_with_the_generated_layout() {
    use super::teleporter::*;
    use muo_io::GenericWriter;
    use muo_server::serialization_ext::WriteEntityExt;
    let mut seen = HashSet::new();
    for t in ALL {
        assert!(seen.insert(t.full_name));
        round_trip_any(t);
    }
    // Teleporter: version 5, flags (Active), Delay, SoundID, PointDest, MapDest.
    let mut w = World::new();
    let id = (TELEPORTER.construct.unwrap())(&mut w);
    let mut expected = SaveWriter::new(true);
    (ITEM_BASE.serialize)(&w, id, &mut expected);
    expected.write_i32(5);
    expected.write_enum(4, 1);
    expected.write_time_span_ticks(0);
    expected.write_encoded_int(0);
    expected.write_point3d(muo_primitives::Point3D::ZERO);
    expected.write_map(None);
    assert_eq!(save(&w, id), expected.into_vec());
    assert!(get_flag(&w, id, flags::ACTIVE));
    set_flag(&mut w, id, flags::CREATURES, true);
    assert!(get_flag(&w, id, flags::CREATURES));
}

#[test]
fn old_teleporter_versions_load() {
    use super::teleporter::*;
    use muo_io::GenericWriter;
    use muo_server::serialization_ext::WriteEntityExt;
    let mut w = World::new();
    let src = (TELEPORTER.construct.unwrap())(&mut w);
    let mut out = SaveWriter::new(true);
    (ITEM_BASE.serialize)(&w, src, &mut out);
    out.write_i32(4); // version < 5: the flags are separate bools
    for b in [true, false, true, false] {
        out.write_bool(b); // CriminalCheck, CombatCheck, SourceEffect, DestEffect
    }
    out.write_time_span_ticks(50_000_000);
    out.write_encoded_int(0x1F6);
    out.write_bool(true); // Creatures
    out.write_bool(false); // Active
    out.write_point3d(muo_primitives::Point3D::new(1, 2, 3));
    out.write_map(Some(muo_server::map::MapId::TRAMMEL));
    let bytes = out.into_vec();
    let loaded = w.add_item(muo_server::item::Item::new_for_load(
        muo_primitives::Serial::new(0x4000_0500),
        &TELEPORTER,
    ));
    let mut r = SaveReader::new(&bytes);
    (TELEPORTER.deserialize)(&mut w, loaded, &mut r).unwrap();
    let d = data(&w, loaded).unwrap();
    assert_eq!(
        d.flags,
        flags::CRIMINAL_CHECK | flags::SOURCE_EFFECT | flags::CREATURES
    );
    assert_eq!(d.sound_id, 0x1F6);
    assert_eq!(d.map_dest, Some(muo_server::map::MapId::TRAMMEL));
    assert_eq!(d.delay.ticks(), 50_000_000);
}

#[test]
fn wait_teleporter_time_format() {
    use super::teleporter::format_time;
    use muo_primitives::TimeSpan;
    assert_eq!(format_time(TimeSpan::from_secs_f64(7200.0)), "2 hours");
    assert_eq!(format_time(TimeSpan::from_secs_f64(3600.0)), "1 hour");
    assert_eq!(format_time(TimeSpan::from_secs_f64(90.0)), "2 minutes"); // 1.5 rounds to even
    assert_eq!(format_time(TimeSpan::from_secs_f64(30.0)), "30 seconds");
    assert_eq!(format_time(TimeSpan::from_secs_f64(1.0)), "1 second");
}

#[test]
fn flippable_statics_agree_with_the_attribute_table() {
    use muo_content::items::flip::flip_ids;
    let lists: [&[(&'static ItemType, &'static [i32])]; 6] = [
        super::leaves::FLIPPABLE,
        super::small::FLIPPABLE,
        crate::maps::FLIPPABLE,
        crate::games::FLIPPABLE,
        crate::guilds::FLIPPABLE,
        super::basic::FLIPPABLE,
    ];
    for list in lists {
        for (ty, ids) in list {
            assert_eq!(flip_ids(ty), Some(*ids), "{}", ty.full_name);
        }
    }
}

// ---- gadgets, dyes, machines, decorator, bulletin boards, mahjong ----

/// Save, load into a fresh item of the same class, save again: the bytes agree and are all read.
fn round_trip_item(w: &mut World, id: muo_server::ids::ItemId) -> Vec<u8> {
    let t = w.item(id).unwrap().ty;
    let bytes = save(w, id);
    let loaded = w.add_item(muo_server::item::Item::new_for_load(
        muo_primitives::Serial::new(0x4000_0700),
        t,
    ));
    let mut r = SaveReader::new(&bytes);
    (t.deserialize)(w, loaded, &mut r).unwrap_or_else(|e| panic!("{}: {e}", t.full_name));
    assert_eq!(r.remaining(), 0, "{}", t.full_name);
    assert_eq!(save(w, loaded), bytes, "{}", t.full_name);
    bytes
}

#[test]
fn the_new_misc_classes_round_trip() {
    let mut seen = HashSet::new();
    let classes: Vec<&'static ItemType> = super::gadgets::ALL
        .iter()
        .chain(super::dyes::ALL)
        .chain(super::machines::ALL)
        .chain(super::moongates::ALL)
        .copied()
        .chain([&super::decorator::INTERIOR_DECORATOR])
        .chain(crate::bulletin::ALL.iter().copied())
        .chain([
            &crate::games::mahjong::MAHJONG_GAME,
            &super::player_bb::PLAYER_BB_SOUTH,
            &super::player_bb::PLAYER_BB_EAST,
        ])
        .collect();
    for t in classes {
        assert!(seen.insert(t.full_name), "{} twice", t.full_name);
        let mut w = World::new();
        let id = match t.full_name {
            // `GateTravelMoongate(Point3D target, Map map)` only (no [Constructible], no parameterless
            // constructor) and `SkipSerialization => true`: never created by `[add`, never saved.
            "Server.Spells.Seventh.GateTravelMoongate" => continue,
            "Server.Items.BaseWindChimes"
            | "Server.Items.PromotionalToken"
            | "Server.Items.BaseBulletinBoard"
            | "Server.Items.BulletinMessage"
            | "Server.Items.ControlPanel"
            | "Server.Items.MoonstoneGate" => continue,
            "Server.Items.ProjectedItem" => super::machines::projected_item_new(&mut w, 0x3779),
            _ => construct_any(&mut w, t),
        };
        round_trip_item(&mut w, id);
    }
}

#[test]
fn control_panel_path_reaches_the_far_corner_and_round_trips() {
    use super::machines::*;
    let mut w = World::new();
    let id = control_panel_new(&mut w, 4);
    panel_init_path(&mut w, id);
    let bytes = round_trip_item(&mut w, id);
    // AddonComponent part, encoded version 1, SideLength, then the encoded length and the points
    assert!(bytes.windows(4).any(|b| b == 4i32.to_le_bytes()));
    let panel_path = {
        let t = w.item(id).unwrap();
        t.ext.get::<PanelData>().unwrap().path.clone()
    };
    assert_eq!(panel_path.first().map(|p| (p.x, p.y)), Some((0, 0)));
    assert_eq!(panel_path.last().map(|p| (p.x, p.y)), Some((3, 3)));
    for pair in panel_path.windows(2) {
        assert_eq!(
            (pair[0].x - pair[1].x).abs() + (pair[0].y - pair[1].y).abs(),
            1
        );
    }
}

#[test]
fn warning_item_old_layout_loads() {
    use super::gadgets::*;
    let mut w = World::new();
    let id = muo_server::item::ops::new_item_with_id(&mut w, &WARNING_ITEM, 0x80);
    let mut out = SaveWriter::new(true);
    (ITEM_BASE.serialize)(&w, id, &mut out);
    out.write_i32(0); // version 0: string, int, range, reset delay
    out.write_opt_string(Some("hello"));
    out.write_i32(1000001);
    out.write_i32(7);
    out.write_time_span_ticks(0);
    let bytes = out.into_vec();
    let loaded = w.add_item(muo_server::item::Item::new_for_load(
        muo_primitives::Serial::new(0x4000_0701),
        &WARNING_ITEM,
    ));
    let mut r = SaveReader::new(&bytes);
    (WARNING_ITEM.deserialize)(&mut w, loaded, &mut r).unwrap();
    // a message number wins over the string
    let again = save(&w, loaded);
    assert!(!again.is_empty());
    let mut r2 = SaveReader::new(&again);
    let l2 = w.add_item(muo_server::item::Item::new_for_load(
        muo_primitives::Serial::new(0x4000_0702),
        &WARNING_ITEM,
    ));
    (WARNING_ITEM.deserialize)(&mut w, l2, &mut r2).unwrap();
    assert_eq!(save(&w, l2), again);
}

#[test]
fn bulletin_message_record() {
    use crate::bulletin::*;
    let mut w = World::new();
    let poster = w.new_mobile(&muo_server::mobile::MOBILE_BASE);
    let id = bulletin_message_new(
        &mut w,
        poster,
        None,
        "subject".into(),
        vec!["one".into(), "two".into()],
    );
    let bytes = round_trip_item(&mut w, id);
    // base item part, version 2, then the poster serial
    let base = {
        let mut o = SaveWriter::new(true);
        (ITEM_BASE.serialize)(&w, id, &mut o);
        o.into_vec().len()
    };
    assert_eq!(&bytes[base..base + 4], &2i32.to_le_bytes());
    assert_eq!(
        time_as_string(muo_primitives::DateTime::from_ymd(2024, 3, 5).unwrap()),
        "Mar 05, 2024"
    );
    assert_eq!(
        format_ts(muo_primitives::TimeSpan::from_secs(150)),
        "2 minutes and 30 seconds"
    );
    assert_eq!(
        format_ts(muo_primitives::TimeSpan::from_secs(1)),
        "1 second"
    );
}

#[test]
fn mahjong_game_record() {
    use crate::games::mahjong::*;
    let mut w = World::new();
    let id = mahjong_game_new(&mut w, &MAHJONG_GAME);
    let bytes = round_trip_item(&mut w, id);
    let d = w.item(id).unwrap().ext.get::<GameData>().unwrap().clone();
    assert_eq!(d.tiles.len(), 136);
    // four of each of the 34 tile types
    for t in 1..=34 {
        assert_eq!(d.tiles.iter().filter(|x| x.value == t).count(), 4);
    }
    assert!(d.dices.0 >= 1 && d.dices.0 <= 6 && d.dices.1 >= 1 && d.dices.1 <= 6);
    assert_eq!(d.tiles[0].position.x, 165);
    assert_eq!(d.tiles[17].stack_level, 1);
    assert!(bytes.len() > 136 * 20);
    // a tile on the top of its pile can move; one under another cannot
    assert!(get_stack_level(&d, &d.tiles[0].dimensions()) >= 0);
}

#[test]
fn constructor_tables_replace_the_stand_ins() {
    use muo_content::props::{Value, ctors_of};
    let mut w = World::new();
    super::basic::basic_configure(&mut w);
    super::gadgets::gadgets_configure(&mut w);
    crate::maps::ctors_configure(&mut w);
    crate::guilds::guilds_configure(&mut w);
    // declaration order: Static(itemID) then Static(itemID, count)
    let t = ctors_of(&w, "Server.Items.Static").unwrap();
    assert_eq!(t.len(), 2);
    assert_eq!((t[0].params.len(), t[1].params.len()), (1, 2));
    let id = match (t[0].make)(&mut w, &[Value::Int(0x1B72)]) {
        muo_server::ids::EntityId::Item(i) => i,
        _ => panic!(),
    };
    assert_eq!(w.item(id).unwrap().item_id, 0x1B72);
    assert!(!w.item(id).unwrap().movable());
    // MorphItem(a, b, range) passes the range twice
    let m = ctors_of(&w, "Server.Items.MorphItem").unwrap();
    assert_eq!(m.len(), 2);
    // Guildstone has no [Constructible] constructor
    assert!(ctors_of(&w, "Server.Items.Guildstone").is_none());
    assert!(crate::guilds::GUILDSTONE.construct.is_none());
    assert!(ctors_of(&w, "Server.Items.TreasureMap").is_some());
    assert!(ctors_of(&w, "Server.Items.PresetMap").is_some());
}

#[test]
fn unholy_bone_is_carvable_and_a_loaded_one_spawns() {
    use muo_server::ids::EntityId;
    let mut w = World::new();
    super::gadgets::gadgets_configure(&mut w);
    let m = w.new_mobile(&muo_server::mobile::MOBILE_BASE);
    let bone = (super::gadgets::UNHOLY_BONE.construct.unwrap())(&mut w);
    // BladedItemTarget/HarvestTarget reach it through ICarvable.
    assert!(muo_content::items::interfaces::carve(&mut w, bone, m, bone));
    let _ = EntityId::Item(bone);
    // A loaded bone gets its SpawnTimer back ([AfterDeserialization]).
    let t = &super::gadgets::UNHOLY_BONE;
    let mut w2 = World::new();
    let b = (t.construct.unwrap())(&mut w2);
    let bytes = save(&w2, b);
    let loaded = w2.add_item(muo_server::item::Item::new_for_load(
        muo_primitives::Serial::new(0x4000_0100),
        t,
    ));
    (t.deserialize)(&mut w2, loaded, &mut SaveReader::new(&bytes)).unwrap();
    assert!(
        w2.item(loaded)
            .unwrap()
            .ext
            .get::<super::gadgets::BoneData>()
            .is_some_and(|d| d.timer.is_some())
    );
}
