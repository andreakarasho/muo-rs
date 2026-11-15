//! Tests of the `leaf_simple` folders: names are unique, every class with a constructor builds and its save is the
//! parent's part followed by the version, and a few constructors are checked field by field.

use std::collections::HashSet;

use muo_io::{SaveReader, SaveWriter};
use muo_server::World;
use muo_server::ids::ItemId;
use muo_server::item::{Item, ItemType, LootType};

fn save(w: &World, id: ItemId) -> Vec<u8> {
    let ty = w.item(id).unwrap().ty;
    let mut out = SaveWriter::new(true);
    (ty.serialize)(w, id, &mut out);
    out.into_vec()
}

#[test]
fn names_are_unique() {
    let mut seen = HashSet::new();
    let all = super::all();
    assert!(all.len() > 200, "{}", all.len());
    for (t, aliases) in &all {
        assert!(seen.insert(t.full_name), "{} twice", t.full_name);
        assert!(t.full_name.starts_with("Server.Items."), "{}", t.full_name);
        for a in *aliases {
            assert!(seen.insert(*a), "alias {a} twice");
        }
    }
    // registering twice adds nothing, and every class is in the registry
    let mut types = muo_server::type_registry::TypeRegistry::default();
    super::register(&mut types);
    super::register(&mut types);
    for (t, _) in &all {
        assert!(
            types.item_types().iter().any(|r| std::ptr::eq(*r, *t)),
            "{} not registered",
            t.full_name
        );
    }
}

/// The version after the parent's part: `[SerializationGenerator(0)]` is an encoded int, `(0, false)` an int.
fn version_len(bytes: usize, parent: usize) -> usize {
    bytes - parent
}

#[test]
fn every_class_constructs_and_round_trips() {
    let mut built = 0;
    for (t, _) in super::all() {
        let Some(construct) = t.construct else {
            continue;
        };
        let mut w = crate::test_world();
        let id = construct(&mut w);
        assert_eq!(w.item(id).unwrap().ty.full_name, t.full_name);
        let bytes = save(&w, id);

        // the parent's part, then the version
        let parent = t.parent.unwrap();
        let mut base = SaveWriter::new(true);
        (parent.serialize)(&w, id, &mut base);
        let base = base.into_vec();
        assert!(bytes.starts_with(&base), "{}", t.full_name);
        assert!(
            matches!(version_len(bytes.len(), base.len()), 1 | 4),
            "{}",
            t.full_name
        );
        assert!(
            bytes[base.len()..].iter().all(|b| *b == 0),
            "{}",
            t.full_name
        );

        let loaded = w.add_item(Item::new_for_load(
            muo_primitives::Serial::new(0x4000_0100),
            t,
        ));
        let mut r = SaveReader::new(&bytes);
        (t.deserialize)(&mut w, loaded, &mut r).unwrap_or_else(|e| panic!("{}: {e}", t.full_name));
        assert_eq!(r.remaining(), 0, "{} is read to its end", t.full_name);
        assert_eq!(save(&w, loaded), bytes, "{}", t.full_name);
        built += 1;
    }
    assert!(built > 200, "{built}");
}

fn make(ty: &'static ItemType) -> (World, ItemId) {
    let mut w = crate::test_world();
    let id = (ty.construct.unwrap())(&mut w);
    (w, id)
}

#[test]
fn decoration_artifact_constructor_and_rarity() {
    use crate::decoration_artifacts::DecoClass;
    use crate::decoration_artifacts::leaves::ACADEMIC_BOOKS_ARTIFACT;
    // `: base(0x1E25) => Hue = 2413`, ArtifactRarity 8, LabelNumber 1071202, DefaultWeight 10 from the base
    let (w, id) = make(&ACADEMIC_BOOKS_ARTIFACT);
    let item = w.item(id).unwrap();
    assert_eq!((item.item_id, item.hue), (0x1E25, 2413));
    assert_eq!(item.ty.family::<DecoClass>().unwrap().artifact_rarity, 8);
    assert_eq!((item.ty.label_number)(&w, id), 1071202);
    assert_eq!((item.ty.default_weight)(&w, id), 10.0);
    assert!((item.ty.force_show_properties)(&w, id));
    // [SerializationGenerator(0)]: encoded versions, one byte each for the base and the class
    let bytes = save(&w, id);
    let mut base = SaveWriter::new(true);
    (muo_server::item::ITEM_BASE.serialize)(&w, id, &mut base);
    assert_eq!(bytes.len(), base.into_vec().len() + 2);
}

#[test]
fn floors_pick_a_graphic_in_range_and_are_fixed() {
    use crate::construction::leaves::STONE_PAVERS_LIGHT;
    // BaseFloor: `base(Utility.Random(0x519, 4))`, `Movable = false`
    for _ in 0..20 {
        let (w, id) = make(&STONE_PAVERS_LIGHT);
        let item = w.item(id).unwrap();
        assert!(
            (0x519..0x519 + 4).contains(&item.item_id),
            "{:#x}",
            item.item_id
        );
        assert!(!item.flags.contains(muo_server::item::ImplFlag::MOVABLE));
    }
}

#[test]
fn loot_type_and_hue_statements_run() {
    // the first generated class whose constructor sets LootType.Blessed
    let blessed = super::all()
        .into_iter()
        .filter(|(t, _)| t.construct.is_some())
        .filter(|(t, _)| {
            let (w, id) = make(t);
            w.item(id).unwrap().loot_type == LootType::Blessed
        })
        .count();
    assert!(blessed > 0);
}

// ---- lights (BaseLight, BaseEquipableLight, Torch, Lantern) ----

mod lights {
    use super::*;
    use crate::lights::*;
    use muo_io::{GenericWriter, SaveReader};
    use muo_primitives::{DateTime, TimeSpan};

    fn item_base_part(w: &World, id: ItemId) -> Vec<u8> {
        let mut out = SaveWriter::new(true);
        (muo_server::item::ITEM_BASE.serialize)(w, id, &mut out);
        out.into_vec()
    }

    #[test]
    fn torch_constructor_and_ignite_douse() {
        let (mut w, id) = make(&TORCH);
        let item = w.item(id).unwrap();
        // base(0xF6B), Stackable, Light = Circle300, DefaultWeight 1
        assert_eq!(item.item_id, 0xF6B);
        assert!(item.flags.contains(muo_server::item::ImplFlag::STACKABLE));
        assert_eq!(
            item.light(),
            Some(muo_server::light_type::LightType::Circle300)
        );
        assert_eq!((item.ty.default_weight)(&w, id), 1.0);
        assert!(!burning(&w, id));
        assert_eq!(duration(&w, id), TimeSpan::ZERO); // Burnout is false
        // the base constructor made it two-handed
        assert_eq!((item.ty.layer)(&w, id), muo_server::layer::Layer::TwoHanded);

        ignite(&mut w, id);
        assert!(burning(&w, id));
        assert_eq!(w.item(id).unwrap().item_id, 0xA12); // LitItemID
        douse(&mut w, id);
        assert!(!burning(&w, id));
        assert_eq!(w.item(id).unwrap().item_id, 0xF6B); // UnlitItemID
    }

    #[test]
    fn burnt_out_light_uses_burnt_out_graphic_and_cannot_ignite() {
        // Lantern: LitItemID/UnlitItemID read ItemID; BurntOutItemID is 0 so the unlit graphic is used
        let (mut w, id) = make(&LANTERN);
        assert_eq!(w.item(id).unwrap().item_id, 0xA25);
        ignite(&mut w, id);
        assert_eq!(w.item(id).unwrap().item_id, 0xA22);
        burn(&mut w, id);
        assert!(burnt_out(&w, id) && !burning(&w, id));
        assert_eq!(w.item(id).unwrap().item_id, 0xA25);
        ignite(&mut w, id); // BurntOut: nothing happens
        assert!(!burning(&w, id));
    }

    #[test]
    fn burning_setter_sets_true_whatever_the_value() {
        // `if (_burning != value) { _burning = true; DoTimer(_duration); }`
        let (mut w, id) = make(&TORCH);
        set_burning(&mut w, id, true);
        assert!(burning(&w, id));
        let (mut w, id) = make(&TORCH);
        set_burning(&mut w, id, false); // equal: nothing
        assert!(!burning(&w, id));
    }

    #[test]
    fn save_layout_and_timer_round_trip() {
        let (mut w, id) = make(&TORCH);
        // idle: item part, BaseLight version 2 and four fields, the anchored time MinValue, then two int versions
        let mut want = SaveWriter::new(true);
        want.write_i32(2);
        want.write_bool(false);
        want.write_bool(false);
        want.write_time_span_ticks(0);
        want.write_bool(false);
        want.write_anchored_time_ticks(DateTime::MIN.ticks());
        want.write_i32(0); // BaseEquipableLight
        want.write_i32(0); // Torch
        let mut expect = item_base_part(&w, id);
        expect.extend(want.into_vec());
        assert_eq!(save(&w, id), expect);

        // a burning light with 5 minutes left starts its timer when loaded
        set_duration(&mut w, id, TimeSpan::from_mins(5));
        ignite(&mut w, id);
        assert!(data(&w, id).unwrap().burn_timer != muo_server::timer::TimerHandle::default());
        let left = duration(&w, id);
        assert!(
            left > TimeSpan::from_secs(299) && left <= TimeSpan::from_mins(5),
            "{left}"
        );
        let bytes = save(&w, id);
        let loaded = w.add_item(Item::new_for_load(
            muo_primitives::Serial::new(0x4000_0200),
            &TORCH,
        ));
        let mut r = SaveReader::new(&bytes);
        (TORCH.deserialize)(&mut w, loaded, &mut r).unwrap();
        assert_eq!(r.remaining(), 0);
        let d = data(&w, loaded).unwrap();
        assert!(d.burning && d.burn_timer != muo_server::timer::TimerHandle::default());
        assert!(d.duration > TimeSpan::from_secs(299) && d.duration <= TimeSpan::from_mins(5));
    }

    #[test]
    fn version_0_reads_the_delta_time_only_for_a_burning_light() {
        let (mut w, id) = make(&TORCH);
        let mut out = SaveWriter::new(true);
        out.write_i32(0); // BaseLight version 0: Deserialize(reader, version)
        out.write_bool(true); // burnt out
        out.write_bool(false); // burning
        out.write_time_span_ticks(1234);
        out.write_bool(true); // protected
        out.write_i32(0); // BaseEquipableLight
        out.write_i32(0); // Torch
        let mut bytes = item_base_part(&w, id);
        bytes.extend(out.into_vec());
        let loaded = w.add_item(Item::new_for_load(
            muo_primitives::Serial::new(0x4000_0201),
            &TORCH,
        ));
        let mut r = SaveReader::new(&bytes);
        (TORCH.deserialize)(&mut w, loaded, &mut r).unwrap();
        assert_eq!(r.remaining(), 0);
        let d = data(&w, loaded).unwrap();
        assert!(d.burnt_out && !d.burning && d.protected);
        assert_eq!(d.duration.ticks(), 1234);
    }
}

// ---- artifacts derived from weapons, armor and jewels ----

mod artifacts {
    use super::*;
    use muo_content::aos::{
        AosArmorAttribute, AosAttribute, AosElementAttribute, AosWeaponAttribute, AttrKind,
    };
    use muo_content::items::attr;
    use muo_content::items::weapon::{self};
    use muo_data::expansion::Expansion;

    fn aos() -> World {
        let mut w = crate::test_world();
        w.core.expansion = Some(Expansion::AOS);
        w
    }

    #[test]
    fn weapon_artifact_constructor_and_damage_types() {
        use crate::minor_artifacts::leaves::ARCTIC_DEATH_DEALER;
        let mut w = aos();
        let id = (ARCTIC_DEATH_DEALER.construct.unwrap())(&mut w);
        assert_eq!(w.item(id).unwrap().hue, 0x480);
        let a = |k, m: u32| attr(&w, id, k, m);
        assert_eq!(a(AttrKind::Weapon, AosWeaponAttribute::HitHarm as u32), 33);
        assert_eq!(
            a(AttrKind::Weapon, AosWeaponAttribute::HitLowerAttack as u32),
            40
        );
        assert_eq!(
            a(AttrKind::Weapon, AosWeaponAttribute::ResistColdBonus as u32),
            10
        );
        assert_eq!(
            a(AttrKind::Attributes, AosAttribute::WeaponSpeed as u32),
            20
        );
        assert_eq!(
            a(AttrKind::Attributes, AosAttribute::WeaponDamage as u32),
            40
        );
        let d = weapon::data(&w, id).unwrap();
        assert_eq!((d.hit_points, d.max_hit_points), (255, 255)); // InitMinHits/InitMaxHits 255
        assert_eq!((ARCTIC_DEATH_DEALER.label_number)(&w, id), 1063481);
        // GetDamageTypes: cold = 50; phys = 50; the rest 0
        let t = muo_content::items::weapon_combat::get_damage_types(&w, id, None);
        assert_eq!(
            (t.phys, t.fire, t.cold, t.pois, t.nrgy, t.chaos, t.direct),
            (50, 0, 50, 0, 0, 0, 0)
        );
    }

    #[test]
    fn armor_artifact_constructor_and_resistances() {
        use crate::minor_artifacts::leaves::HEART_OF_THE_LION;
        let mut w = aos();
        let id = (HEART_OF_THE_LION.construct.unwrap())(&mut w);
        assert_eq!(w.item(id).unwrap().hue, 0x501);
        assert_eq!(
            attr(&w, id, AttrKind::Attributes, AosAttribute::Luck as u32),
            95
        );
        assert_eq!(
            attr(
                &w,
                id,
                AttrKind::Attributes,
                AosAttribute::DefendChance as u32
            ),
            15
        );
        assert_eq!(
            attr(
                &w,
                id,
                AttrKind::Armor,
                AosArmorAttribute::LowerStatReq as u32
            ),
            100
        );
        assert_eq!(
            attr(&w, id, AttrKind::Armor, AosArmorAttribute::MageArmor as u32),
            1
        );
        assert_eq!((HEART_OF_THE_LION.physical_resistance)(&w, id), 15);
        assert_eq!((HEART_OF_THE_LION.fire_resistance)(&w, id), 10);
    }

    #[test]
    fn jewel_artifact_constructor() {
        use crate::minor_artifacts::leaves::ALCHEMISTS_BAUBLE;
        let mut w = aos();
        let id = (ALCHEMISTS_BAUBLE.construct.unwrap())(&mut w);
        assert_eq!(w.item(id).unwrap().hue, 0x290);
        assert_eq!(
            attr(
                &w,
                id,
                AttrKind::Attributes,
                AosAttribute::EnhancePotions as u32
            ),
            30
        );
        assert_eq!(
            attr(
                &w,
                id,
                AttrKind::Attributes,
                AosAttribute::LowerRegCost as u32
            ),
            20
        );
        assert_eq!(
            attr(
                &w,
                id,
                AttrKind::Element,
                AosElementAttribute::Poison as u32
            ),
            10
        );
        assert_eq!((ALCHEMISTS_BAUBLE.label_number)(&w, id), 1070638);
    }
}

#[test]
fn no_class_of_these_folders_is_defined_twice_in_the_crate() {
    let mut types = muo_server::type_registry::TypeRegistry::default();
    crate::register(&mut types);
    let mut seen = HashSet::new();
    for t in types.item_types() {
        assert!(
            seen.insert(t.full_name),
            "{} is defined twice in muo_items",
            t.full_name
        );
    }
}
