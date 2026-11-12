//! Tests of the common item classes (containers, corpses, gold, keys, food, resources). Save bytes are written out
//! field by field in each class's serialized layout.

use muo_io::{GenericWriter, SaveReader, SaveWriter};
use muo_primitives::{DateTime, Point3D, TimeSpan};
use muo_server::World;
use muo_server::ids::ItemId;
use muo_server::item::{ITEM_BASE, ItemType};

use crate::craft::CraftResource;
use crate::items::cont::{self, TrapType};
use crate::items::corpse;
use crate::items::food;
use crate::items::gold;
use crate::items::resource;

fn hex(s: &str) -> Vec<u8> {
    let s: String = s.split_whitespace().collect();
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}

fn make(w: &mut World, t: &ItemType) -> ItemId {
    (t.construct.unwrap())(w)
}

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

fn for_load(w: &mut World, serial: u32, ty: &'static ItemType) -> ItemId {
    w.add_item(muo_server::item::Item::new_for_load(
        muo_primitives::Serial::new(serial),
        ty,
    ))
}

fn load(w: &mut World, id: ItemId, bytes: &[u8]) -> muo_io::Result<()> {
    let ty = w.item(id).unwrap().ty;
    let mut r = SaveReader::new(bytes);
    (ty.deserialize)(w, id, &mut r)
}

/// `Container`'s part when nothing differs from the defaults: version 0 and an empty flag set.
const CONTAINER_DEFAULT: &str = "00000000 00";

#[test]
fn pouch_record() {
    let mut w = World::new();
    let id = make(&mut w, &cont::POUCH);
    // Container; TrappableContainer v3: level, power, type (enum, encoded); Pouch v0.
    let mut expected = item_part(&w, id);
    expected.extend(hex(&format!(
        "{CONTAINER_DEFAULT} 03000000 00000000 00000000 00 00000000"
    )));
    assert_eq!(save(&w, id), expected);
    assert_eq!((cont::POUCH.default_weight)(&w, id), 1.0);
}

#[test]
fn old_trappable_version_loads_the_same_fields() {
    let mut w = World::new();
    let id = for_load(&mut w, 0x4000_0010, &cont::POUCH);
    let src = make(&mut w, &cont::POUCH);
    let mut bytes = item_part(&w, src);
    // version 2 (< 3): Deserialize(reader, version) reads the same three fields.
    bytes.extend(hex(&format!(
        "{CONTAINER_DEFAULT} 02000000 05000000 07000000 02 00000000"
    )));
    // Construction gives the container a fresh ContExt when a pouch is constructed; the loader's item has none yet.
    if let Some(i) = w.item_mut(id) {
        i.ext = muo_server::ext::Ext::new(cont::ContExt::default());
        *i.container_mut() = Default::default();
    }
    load(&mut w, id, &bytes).unwrap();
    let d = cont::ext(&w, id).unwrap();
    assert_eq!(
        (d.trap_level, d.trap_power, d.trap_type),
        (5, 7, TrapType::ExplosionTrap)
    );
}

#[test]
fn wooden_box_record_and_roundtrip() {
    let mut w = World::new();
    let id = make(&mut w, &cont::WOODEN_BOX);
    assert_eq!(w.item(id).unwrap().item_id, 0x9AA);
    assert_eq!((cont::WOODEN_BOX.default_weight)(&w, id), 4.0);
    // LockableContainer v0: shipwrecked, trapOnLockpick, requiredSkill, maxLockLevel (100), keyValue, lockLevel, locked.
    let mut expected = item_part(&w, id);
    expected.extend(hex(&format!(
        "{CONTAINER_DEFAULT} 03000000 00000000 00000000 00 \
         00000000 00 00 00000000 64000000 00000000 00000000 00 00000000"
    )));
    assert_eq!(save(&w, id), expected);

    cont::ext_mut(&mut w, id).unwrap().key_value = 0xDEADBEEF;
    cont::set_locked(&mut w, id, true);
    let bytes = save(&w, id);
    let copy = for_load(&mut w, 0x4000_0011, &cont::WOODEN_BOX);
    if let Some(i) = w.item_mut(copy) {
        i.ext = muo_server::ext::Ext::new(cont::ContExt::default());
        *i.container_mut() = Default::default();
    }
    load(&mut w, copy, &bytes).unwrap();
    let d = cont::ext(&w, copy).unwrap();
    assert!(d.locked && d.key_value == 0xDEADBEEF && d.max_lock_level == 100);
    assert!(cont::locked(&w, copy));
}

#[test]
fn locked_container_refuses_to_show_content() {
    let mut w = World::new();
    let id = make(&mut w, &cont::LARGE_CRATE);
    assert!((cont::LARGE_CRATE.displays_content)(&w, id));
    cont::set_locked(&mut w, id, true);
    assert!(!(cont::LARGE_CRATE.displays_content)(&w, id));
}

#[test]
fn treasure_chest_record() {
    let mut w = World::new();
    let id = make(&mut w, &cont::WOODEN_TREASURE_CHEST);
    let item = w.item(id).unwrap();
    assert_eq!(item.item_id, 0xE43);
    // Level 2: lock level and required skill 20, locked, holds one pile of gold in 300..=600.
    assert_eq!(item.items.len(), 1);
    let gold_amount = w.item(item.items[0]).unwrap().amount;
    assert!((300..=600).contains(&gold_amount));
    assert_eq!(
        ((cont::WOODEN_TREASURE_CHEST.default_name)(&w, id)).as_deref(),
        Some("a locked treasure chest")
    );

    let mut expected = item_part(&w, id);
    expected.extend(hex(&format!(
        "{CONTAINER_DEFAULT} 03000000 00000000 00000000 00 \
         00000000 00 00 14000000 64000000 00000000 14000000 01 \
         01000000 01 00bca06501000000 0068c46108000000 00000000"
    )));
    assert_eq!(save(&w, id), expected);
}

#[test]
fn treasure_chest_version_0_reads_bytes_and_minutes() {
    let mut w = World::new();
    let src = make(&mut w, &cont::METAL_TREASURE_CHEST);
    let mut bytes = item_part(&w, src);
    bytes.extend(hex(&format!(
        "{CONTAINER_DEFAULT} 03000000 00000000 00000000 00 \
         00000000 00 00 00000000 64000000 00000000 00000000 01 \
         00000000 03 0500 0a00 00000000"
    )));
    let id = for_load(&mut w, 0x4000_0012, &cont::METAL_TREASURE_CHEST);
    if let Some(i) = w.item_mut(id) {
        i.ext = muo_server::ext::Ext::new(cont::ContExt::default());
        *i.container_mut() = Default::default();
    }
    load(&mut w, id, &bytes).unwrap();
    let d = cont::ext(&w, id).unwrap();
    assert_eq!(d.level, cont::TreasureLevel::Level4);
    assert_eq!(
        (d.min_spawn_time, d.max_spawn_time),
        (TimeSpan::from_mins(5), TimeSpan::from_mins(10))
    );
}

#[test]
fn backpack_and_bags() {
    let mut w = World::new();
    let pack = make(&mut w, &cont::BACKPACK);
    assert_eq!(w.item(pack).unwrap().item_id, 0xE75);
    assert_eq!((cont::BACKPACK.default_weight)(&w, pack), 3.0);
    assert_eq!(
        w.item(pack).unwrap().layer,
        muo_server::layer::Layer::Backpack
    );
    let bag = make(&mut w, &cont::BAG);
    assert_eq!((cont::BAG.default_weight)(&w, bag), 2.0);
    let mut expected = item_part(&w, bag);
    expected.extend(hex(&format!("{CONTAINER_DEFAULT} 00000000")));
    assert_eq!(save(&w, bag), expected);
    let strong = make(&mut w, &cont::STRONG_BACKPACK);
    assert_eq!((cont::STRONG_BACKPACK.default_max_weight)(&w, strong), 1600);
}

#[test]
fn gold_numbers() {
    let mut w = World::new();
    w.core.expansion = Some(muo_data::expansion::Expansion::UOR);
    let g = gold::new_gold(&mut w, 1);
    assert_eq!(w.item(g).unwrap().item_id, 0xEED);
    assert_eq!((gold::GOLD.get_drop_sound)(&w, g), 0x2E4);
    muo_server::item::ops::set_amount(&mut w, g, 5);
    assert_eq!((gold::GOLD.get_drop_sound)(&w, g), 0x2E5);
    muo_server::item::ops::set_amount(&mut w, g, 6);
    assert_eq!((gold::GOLD.get_drop_sound)(&w, g), 0x2E6);
    assert_eq!((gold::GOLD.default_weight)(&w, g), 0.02);
    assert_eq!(gold::n0(1234567), "1,234,567");
    assert_eq!(gold::n0(12), "12");
}

#[test]
fn bank_check_record() {
    let mut w = World::new();
    let id = gold::new_bank_check(&mut w, 100);
    let item = w.item(id).unwrap();
    assert_eq!((item.item_id, item.hue), (0x14F0, 0x34));
    let mut expected = item_part(&w, id);
    expected.extend(hex("00000000 64000000"));
    assert_eq!(save(&w, id), expected);
    let copy = for_load(&mut w, 0x4000_0013, &gold::BANK_CHECK);
    load(&mut w, copy, &expected).unwrap();
    assert_eq!(gold::worth(&w, copy), 100);
}

#[test]
fn key_roundtrip_and_ring() {
    let mut w = World::new();
    let key = gold::new_key(&mut w, 0x1234);
    assert_eq!(w.item(key).unwrap().item_id, 0x1010);
    gold::set_key_description(&mut w, key, Some("front door".into()));
    let bytes = save(&w, key);
    let copy = for_load(&mut w, 0x4000_0014, &gold::KEY);
    load(&mut w, copy, &bytes).unwrap();
    assert_eq!(gold::key_value(&w, copy), 0x1234);

    let ring = gold::new_key_ring(&mut w);
    assert_eq!(w.item(ring).unwrap().item_id, 0x1011);
    gold::ring_add(&mut w, ring, key);
    assert_eq!(w.item(ring).unwrap().item_id, 0x1769);
    assert!(gold::ring_contains_key(&w, ring, 0x1234));
    assert!(!gold::ring_contains_key(&w, ring, 0x4321));
    // KeyRing v1: encoded version, encoded count, the keys.
    let ring_bytes = save(&w, ring);
    let tail = &ring_bytes[ring_bytes.len() - 6..];
    assert_eq!(tail[0], 1);
    assert_eq!(tail[1], 1);
    gold::ring_remove_key(&mut w, ring, 0x1234);
    assert_eq!(w.item(ring).unwrap().item_id, 0x1011);
}

#[test]
fn food_record_and_stacking() {
    let mut w = World::new();
    let id = make(&mut w, &food::BREAD_LOAF);
    assert_eq!(
        (w.item(id).unwrap().item_id, food::fill_factor(&w, id)),
        (0x103B, 3)
    );
    let cake = make(&mut w, &food::CAKE);
    assert!(
        !w.item(cake)
            .unwrap()
            .flags
            .contains(muo_server::item::ImplFlag::STACKABLE)
    );
    let bytes = save(&w, id);
    let copy = for_load(&mut w, 0x4000_0015, &food::BREAD_LOAF);
    load(&mut w, copy, &bytes).unwrap();
    assert_eq!(food::fill_factor(&w, copy), 3);
    // the leaf version follows Food's part
    assert_eq!(&bytes[bytes.len() - 4..], &[0, 0, 0, 0]);
}

#[test]
fn resource_bases_and_old_versions() {
    let mut w = World::new();
    let ingot = make(&mut w, &resource::DULL_COPPER_INGOT);
    assert_eq!(w.item(ingot).unwrap().hue, 0x973);
    assert_eq!(
        (resource::DULL_COPPER_INGOT.label_number)(&w, ingot),
        1042684
    );
    let iron = make(&mut w, &resource::IRON_INGOT);
    assert_eq!((resource::IRON_INGOT.label_number)(&w, iron), 1042692);

    let mut expected = item_part(&w, ingot);
    expected.extend(hex("02000000 02 00000000"));
    assert_eq!(save(&w, ingot), expected);

    // version 1 (< 2) stores the resource as a plain int; the leaf version int follows.
    let copy = for_load(&mut w, 0x4000_0016, &resource::DULL_COPPER_INGOT);
    let mut old = item_part(&w, ingot);
    old.extend(hex("01000000 03000000 00000000"));
    load(&mut w, copy, &old).unwrap();
    assert_eq!(resource::resource(&w, copy), CraftResource::ShadowIron);

    // BaseOre's legacy read is a single byte.
    let ore = for_load(&mut w, 0x4000_0017, &resource::GOLD_ORE);
    let mut old_ore = item_part(&w, ingot);
    old_ore.extend(hex("01000000 06 00000000"));
    load(&mut w, ore, &old_ore).unwrap();
    assert_eq!(resource::resource(&w, ore), CraftResource::Gold);

    let scales = make(&mut w, &resource::RED_SCALES);
    assert_eq!((resource::RED_SCALES.label_number)(&w, scales), 1053139);
    let granite = make(&mut w, &resource::GRANITE);
    assert_eq!(w.item(granite).unwrap().item_id, 0x1779);
}

#[test]
fn corpse_versions_load() {
    let mut w = World::new();
    w.now = DateTime::from_ymd_hms(2024, 1, 1, 0, 0, 0).unwrap();
    let probe = make(&mut w, &cont::BAG);
    let head = item_part(&w, probe);

    // Version 17: delta times, hair as four ints, Murderer folded into the flags.
    let mut rec = head.clone();
    rec.extend(hex(CONTAINER_DEFAULT));
    let mut out = SaveWriter::new(true);
    out.write_i32(17);
    out.write_encoded_int(0); // RestoreEquip
    out.write_enum(4, i64::from(corpse::flag::CARVED)); // Flags
    out.write_delta_time_ticks(w.now.ticks() - 600_000_000, w.now.ticks()); // TimeOfDeath
    out.write_encoded_int(0); // RestoreTable
    out.write_delta_time_ticks(DateTime::MIN.ticks(), w.now.ticks()); // no decay timer
    out.write_encoded_int(0); // Looters
    out.write_serial(muo_primitives::Serial::MINUS_ONE); // Killer
    out.write_encoded_int(0); // Aggressors
    out.write_serial(muo_primitives::Serial::MINUS_ONE); // Owner
    out.write_opt_string(Some("a corpse"));
    out.write_enum(4, 0); // AccessLevel
    out.write_serial(muo_primitives::Serial::MINUS_ONE); // Guild
    out.write_encoded_int(0); // EquipItems
    for v in [0x203B, 0x44, 0, 0] {
        out.write_i32(v);
    }
    rec.extend(out.into_vec());

    let id = for_load(&mut w, 0x4000_0020, &corpse::CORPSE);
    if let Some(i) = w.item_mut(id) {
        *i.container_mut() = Default::default();
    }
    load(&mut w, id, &rec).unwrap();
    let d = corpse::data(&w, id).unwrap();
    assert_eq!(
        (d.flags, d.hair_item_id, d.hair_hue),
        (corpse::flag::CARVED, 0x203B, 0x44)
    );
    assert_eq!(d.corpse_name.as_deref(), Some("a corpse"));
    assert!(d.decay_timer == muo_server::timer::TimerHandle::default());

    // Version 13: Kills >= 5 makes the owner a murderer; no hair at all.
    let mut rec = head.clone();
    rec.extend(hex(CONTAINER_DEFAULT));
    let mut out = SaveWriter::new(true);
    out.write_i32(13);
    out.write_encoded_int(0);
    out.write_enum(4, 0);
    out.write_delta_time_ticks(w.now.ticks(), w.now.ticks());
    out.write_encoded_int(0);
    out.write_delta_time_ticks(DateTime::MIN.ticks(), w.now.ticks());
    out.write_encoded_int(0);
    out.write_serial(muo_primitives::Serial::MINUS_ONE);
    out.write_encoded_int(0);
    out.write_serial(muo_primitives::Serial::MINUS_ONE);
    out.write_opt_string(None);
    out.write_enum(4, 0);
    out.write_serial(muo_primitives::Serial::MINUS_ONE);
    out.write_i32(5); // Kills
    out.write_encoded_int(0);
    rec.extend(out.into_vec());
    let id = for_load(&mut w, 0x4000_0021, &corpse::CORPSE);
    if let Some(i) = w.item_mut(id) {
        *i.container_mut() = Default::default();
    }
    load(&mut w, id, &rec).unwrap();
    assert!(corpse::get_flag(&w, id, corpse::flag::MURDERER));
    // 13-16 always started a decay timer, even for MinValue.
    assert!(corpse::data(&w, id).unwrap().decay_timer != muo_server::timer::TimerHandle::default());
    let _ = Point3D::new(0, 0, 0);
}

#[test]
fn corpse_roundtrip_v19() {
    let mut w = World::new();
    w.now = DateTime::from_ymd_hms(2024, 1, 1, 0, 0, 0).unwrap();
    let id = for_load(&mut w, 0x4000_0030, &corpse::CORPSE);
    if let Some(i) = w.item_mut(id) {
        *i.container_mut() = Default::default();
        i.ext = muo_server::ext::Ext::new(corpse::CorpseData::default());
    }
    let now = w.now;
    {
        let d = corpse::data_mut(&mut w, id).unwrap();
        d.flags = corpse::flag::NO_BONES | corpse::flag::CRIMINAL;
        d.corpse_name = Some("a dead orc".into());
        d.hair_item_id = 0x2044;
        d.hair_hue = 0x12;
        d.time_of_death = now;
    }
    let bytes = save(&w, id);
    let copy = for_load(&mut w, 0x4000_0031, &corpse::CORPSE);
    load(&mut w, copy, &bytes).unwrap();
    let (a, b) = (
        corpse::data(&w, id).unwrap(),
        corpse::data(&w, copy).unwrap(),
    );
    assert_eq!(
        (a.flags, a.hair_item_id, &a.corpse_name),
        (b.flags, b.hair_item_id, &b.corpse_name)
    );
    assert_eq!(save(&w, copy)[..], bytes[..]);
}

#[test]
fn common_classes_register() {
    let mut types = muo_server::type_registry::TypeRegistry::new();
    crate::items::common::register(&mut types);
    for name in [
        "Server.Items.Backpack",
        "Server.Items.Pouch",
        "Server.Items.Corpse",
        "Server.Items.Gold",
        "Server.Items.BankCheck",
        "Server.Items.Key",
        "Server.Items.KeyRing",
        "Server.Items.BreadLoaf",
        "Server.Items.IronIngot",
    ] {
        assert!(
            types.find_type_by_full_name(name, false).is_some(),
            "{name}"
        );
    }
}

#[test]
fn key_unlocking_a_magic_locked_chest_resets_the_lock_level() {
    let mut w = World::new();
    let m = w.new_mobile(&muo_server::mobile::MOBILE_BASE);
    let chest = cont::new_lockable(&mut w, &cont::LOCKABLE_CONTAINER, 0xE41);
    {
        let d = cont::ext_mut(&mut w, chest).unwrap();
        d.lock_level = -255;
        d.required_skill = 40;
    }
    cont::set_locked(&mut w, chest, true);
    let key = gold::new_key(&mut w, 7);
    cont::ext_mut(&mut w, chest).unwrap().key_value = 7;
    assert!(gold::use_on(&mut w, key, m, chest));
    assert_eq!(cont::ext(&w, chest).unwrap().lock_level, 30);
}
