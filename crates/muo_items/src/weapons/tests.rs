//! Tests of the weapon leaves. The expected version encoding of each class is fixed by its save format:
//! `true` is `WriteEncodedInt(SerializationVersion)` (`[SerializationGenerator(0)]`), `false` is
//! `Write(SerializationVersion)` (`[SerializationGenerator(0, false)]`). `FireworksWand` also writes `_charges`.

use muo_content::items::weapon::{self, BASE_WEAPON};
use muo_content::items::weapon_combat as wc;
use muo_data::expansion::Expansion;
use muo_io::{SaveReader, SaveWriter};
use muo_server::item::LootType;
use muo_server::type_registry::EntityType;

use super::*;

/// Every class of this directory plus the three samples: (full name, version is an encoded int).
const CLASSES: &[(&str, bool)] = &[
    ("Server.Items.AxeOfTheHeavens", false),
    ("Server.Items.BladeOfInsanity", false),
    ("Server.Items.BladeOfTheRighteous", false),
    ("Server.Items.BoneCrusher", false),
    ("Server.Items.BreathOfTheDead", false),
    ("Server.Items.Frostbringer", false),
    ("Server.Items.LegacyOfTheDreadLord", false),
    ("Server.Items.SerpentsFang", false),
    ("Server.Items.StaffOfTheMagi", false),
    ("Server.Items.TheBeserkersMaul", false),
    ("Server.Items.TheDragonSlayer", false),
    ("Server.Items.TheDryadBow", false),
    ("Server.Items.TheTaskmaster", false),
    ("Server.Items.TitansHammer", false),
    ("Server.Items.ZyronicClaw", false),
    ("Server.Items.Axe", false),
    ("Server.Items.BattleAxe", false),
    ("Server.Items.DoubleAxe", false),
    ("Server.Items.DualShortAxes", true),
    ("Server.Items.ExecutionersAxe", false),
    ("Server.Items.GuardianAxe", true),
    ("Server.Items.Hatchet", false),
    ("Server.Items.HeavyOrnateAxe", true),
    ("Server.Items.LargeBattleAxe", false),
    ("Server.Items.Pickaxe", false),
    ("Server.Items.SingingAxe", true),
    ("Server.Items.ThunderingAxe", true),
    ("Server.Items.TwoHandedAxe", false),
    ("Server.Items.WarAxe", false),
    ("Server.Items.ButcherKnife", false),
    ("Server.Items.Cleaver", false),
    ("Server.Items.Dagger", false),
    ("Server.Items.SkinningKnife", false),
    ("Server.Items.BlightGrippedLongbow", true),
    ("Server.Items.ColdForgedBlade", true),
    ("Server.Items.FaerieFire", true),
    ("Server.Items.LuminousRuneBlade", true),
    ("Server.Items.MischiefMaker", true),
    ("Server.Items.OverseerSunderedBlade", true),
    ("Server.Items.PhantomStaff", true),
    ("Server.Items.RuneCarvingKnife", true),
    ("Server.Items.ShardThrasher", true),
    ("Server.Items.SilvanisFeywoodBow", true),
    ("Server.Items.TheNightReaper", false),
    ("Server.Items.AssassinSpike", true),
    ("Server.Items.ButchersWarCleaver", true),
    ("Server.Items.DiamondMace", true),
    ("Server.Items.ElvenCompositeLongbow", true),
    ("Server.Items.ElvenMachete", true),
    ("Server.Items.ElvenSpellblade", true),
    ("Server.Items.Leafblade", true),
    ("Server.Items.MagicalShortbow", true),
    ("Server.Items.OrnateAxe", true),
    ("Server.Items.RadiantScimitar", true),
    ("Server.Items.RuneBlade", true),
    ("Server.Items.WarCleaver", true),
    ("Server.Items.WildStaff", true),
    ("Server.Items.Club", false),
    ("Server.Items.DiscMace", true),
    ("Server.Items.EmeraldMace", true),
    ("Server.Items.HammerPick", false),
    ("Server.Items.Mace", false),
    ("Server.Items.MagicWand", false),
    ("Server.Items.Maul", false),
    ("Server.Items.RubyMace", true),
    ("Server.Items.SapphireMace", true),
    ("Server.Items.Scepter", false),
    ("Server.Items.SilverEtchedMace", true),
    ("Server.Items.WarHammer", false),
    ("Server.Items.WarMace", false),
    ("Server.Items.Bardiche", false),
    ("Server.Items.Halberd", false),
    ("Server.Items.Scythe", false),
    ("Server.Items.AssassinsShortbow", true),
    ("Server.Items.BarbedLongbow", true),
    ("Server.Items.Bow", false),
    ("Server.Items.CompositeBow", false),
    ("Server.Items.Crossbow", false),
    ("Server.Items.FrozenLongbow", true),
    ("Server.Items.HeavyCrossbow", false),
    ("Server.Items.JukaBow", false),
    ("Server.Items.LightweightShortbow", true),
    ("Server.Items.LongbowOfMight", true),
    ("Server.Items.MysticalShortbow", true),
    ("Server.Items.RangersShortbow", true),
    ("Server.Items.RepeatingCrossbow", false),
    ("Server.Items.SlayerLongbow", true),
    ("Server.Items.Bokuto", false),
    ("Server.Items.Daisho", false),
    ("Server.Items.Kama", false),
    ("Server.Items.Lajatang", false),
    ("Server.Items.NoDachi", false),
    ("Server.Items.Nunchaku", false),
    ("Server.Items.Sai", false),
    ("Server.Items.Tekagi", false),
    ("Server.Items.Tessen", false),
    ("Server.Items.Tetsubo", false),
    ("Server.Items.Wakizashi", false),
    ("Server.Items.Yumi", false),
    ("Server.Items.BladedStaff", false),
    ("Server.Items.DoubleBladedStaff", false),
    ("Server.Items.DualPointedSpear", true),
    ("Server.Items.Pike", false),
    ("Server.Items.Pitchfork", false),
    ("Server.Items.ShortSpear", false),
    ("Server.Items.Spear", false),
    ("Server.Items.TribalSpear", false),
    ("Server.Items.WarFork", false),
    ("Server.Items.BlackStaff", false),
    ("Server.Items.GlacialStaff", false),
    ("Server.Items.GlassStaff", true),
    ("Server.Items.GnarledStaff", false),
    ("Server.Items.QuarterStaff", false),
    ("Server.Items.SerpentstoneStaff", true),
    ("Server.Items.ShepherdsCrook", false),
    ("Server.Items.AdventurersMachete", true),
    ("Server.Items.BloodBlade", true),
    ("Server.Items.BoneHarvester", false),
    ("Server.Items.BoneMachete", true),
    ("Server.Items.Broadsword", false),
    ("Server.Items.ChargedAssassinSpike", true),
    ("Server.Items.CorruptedRuneBlade", true),
    ("Server.Items.CrescentBlade", false),
    ("Server.Items.Cutlass", false),
    ("Server.Items.DarkglowScimitar", true),
    ("Server.Items.DiseasedMachete", true),
    ("Server.Items.DreadSword", true),
    ("Server.Items.FierySpellblade", true),
    ("Server.Items.GargishTalwar", true),
    ("Server.Items.GlassSword", true),
    ("Server.Items.IcyScimitar", true),
    ("Server.Items.IcySpellblade", true),
    ("Server.Items.Katana", false),
    ("Server.Items.KnightsWarCleaver", true),
    ("Server.Items.Kryss", false),
    ("Server.Items.Lance", false),
    ("Server.Items.LeafbladeOfEase", true),
    ("Server.Items.Longsword", false),
    ("Server.Items.Luckblade", true),
    ("Server.Items.MacheteOfDefense", true),
    ("Server.Items.MagekillerAssassinSpike", true),
    ("Server.Items.MagekillerLeafblade", true),
    ("Server.Items.MagesRuneBlade", true),
    ("Server.Items.OrcishMachete", true),
    ("Server.Items.RuneBladeOfKnowledge", true),
    ("Server.Items.Runesabre", true),
    ("Server.Items.Scimitar", false),
    ("Server.Items.SerratedWarCleaver", true),
    ("Server.Items.SpellbladeOfDefense", true),
    ("Server.Items.ThinLongsword", false),
    ("Server.Items.TrueAssassinSpike", true),
    ("Server.Items.TrueLeafblade", true),
    ("Server.Items.TrueRadiantScimitar", true),
    ("Server.Items.TrueSpellblade", true),
    ("Server.Items.TrueWarCleaver", true),
    ("Server.Items.TwinklingScimitar", true),
    ("Server.Items.VikingSword", false),
    ("Server.Items.WoundingAssassinSpike", true),
    ("Server.Items.AbyssReaver", true),
    ("Server.Items.BansheesCall", true),
    ("Server.Items.Boomerang", true),
    ("Server.Items.Cyclone", true),
    ("Server.Items.RaptorClaw", true),
    ("Server.Items.SoulGlaive", true),
    ("Server.Items.StoneSlithClaw", true),
    ("Server.Items.StormCaller", true),
    ("Server.Items.ValkyriesGlaive", true),
    ("Server.Items.WindOfCorruption", true),
    ("Server.Items.AncientWildStaff", true),
    ("Server.Items.ArcanistsWildStaff", true),
    ("Server.Items.HardenedWildStaff", true),
    ("Server.Items.ThornedWildStaff", true),
    ("Server.Items.ThrowingDagger", false),
    ("Server.Items.FireworksWand", false),
    ("Server.Items.Fists", false),
];

pub(super) fn registry() -> TypeRegistry {
    let mut t = TypeRegistry::new();
    muo_content::items::leaves::register(&mut t);
    register(&mut t);
    t
}

pub(super) fn find(types: &TypeRegistry, name: &str) -> &'static ItemType {
    match types.find_type_by_full_name(name, false) {
        Some(EntityType::Item(t)) => t,
        _ => panic!("{name} is not registered"),
    }
}

fn save(w: &World, id: ItemId) -> Vec<u8> {
    let ty = w.item(id).unwrap().ty;
    let mut out = SaveWriter::new(true);
    (ty.serialize)(w, id, &mut out);
    out.into_vec()
}

fn make(w: &mut World, ty: &'static ItemType) -> ItemId {
    match ty.construct {
        Some(c) => c(w),
        None => fists_new(w, ty), // Fists is not [Constructible]
    }
}

fn new(w: &mut World, ty: &'static ItemType) -> ItemId {
    (ty.construct.unwrap())(w)
}

fn era(w: &mut World, e: Expansion) {
    w.core.expansion = Some(e);
}

#[test]
fn registered_names_are_unique_and_complete() {
    let types = registry();
    let names: Vec<&str> = types.item_types().iter().map(|t| t.full_name).collect();
    let mut sorted = names.clone();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(sorted.len(), names.len(), "a class is registered twice");
    for (name, _) in CLASSES {
        assert!(names.contains(name), "{name} is not registered");
    }
    assert_eq!(
        names
            .iter()
            .filter(|n| CLASSES.iter().any(|(c, _)| c == *n))
            .count(),
        CLASSES.len()
    );
    // [TypeAlias("Server.Items.SerpentStoneStaff")]
    assert!(std::ptr::eq(
        find(&types, "Server.Items.SerpentStoneStaff"),
        &SERPENTSTONE_STAFF
    ));
}

#[test]
fn every_class_constructs_and_round_trips_its_record() {
    let types = registry();
    let mut serial = 0x4000_0100;
    for (name, encoded) in CLASSES {
        let ty = find(&types, name);
        let mut w = World::new();
        let id = make(&mut w, ty);
        assert_eq!(w.item(id).unwrap().ty.full_name, *name);
        assert!(
            ty.is(&BASE_WEAPON) || *name == "Server.Items.ThrowingDagger",
            "{name}"
        );

        let bytes = save(&w, id);
        let tail: &[u8] = match *name {
            "Server.Items.FireworksWand" => &[0, 0, 0, 0, 100, 0, 0, 0],
            _ if *encoded => &[0],
            _ => &[0, 0, 0, 0],
        };
        assert!(
            bytes.ends_with(tail),
            "{name}: the record ends with its own version"
        );

        // and it loads back into a fresh instance of the class, to the last byte
        serial += 1;
        let mut w2 = World::new();
        let other = w2.add_item(muo_server::item::Item::new_for_load(
            muo_primitives::Serial::new(serial),
            ty,
        ));
        let mut r = SaveReader::new(&bytes);
        (ty.deserialize)(&mut w2, other, &mut r).unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!(r.remaining(), 0, "{name}: the record is read to its end");
        let again = save(&w2, other);
        assert_eq!(again.len(), bytes.len(), "{name}");
        // The bytes after the Item part: everything the leaf chain wrote.
        let item_part = {
            let mut out = SaveWriter::new(true);
            (muo_server::item::ITEM_BASE.serialize)(&w, id, &mut out);
            out.into_vec().len()
        };
        assert_eq!(again[item_part..], bytes[item_part..], "{name}");
    }
}

#[test]
fn stats_follow_the_era() {
    let mut w = World::new();
    let axe = new(&mut w, &AXE);
    era(&mut w, Expansion::LBR);
    assert_eq!(
        (
            weapon::str_requirement(&w, axe),
            weapon::min_damage(&w, axe),
            weapon::max_damage(&w, axe)
        ),
        (35, 6, 33)
    );
    era(&mut w, Expansion::AOS);
    assert_eq!(
        (
            weapon::str_requirement(&w, axe),
            weapon::min_damage(&w, axe),
            weapon::max_damage(&w, axe)
        ),
        (35, 14, 16)
    );
    assert_eq!(weapon::speed(&w, axe), 37.0);
    era(&mut w, Expansion::ML);
    assert_eq!(weapon::speed(&w, axe), 3.0);
    assert_eq!(
        weapon::class(&w, axe).primary_ability,
        Some(Ability::CrushingBlow)
    );
    assert_eq!(weapon::class(&w, axe).flippable, Some((0xF49, 0xF4A)));

    let xbow = new(&mut w, &CROSSBOW);
    assert_eq!(weapon::max_range(&w, xbow), 8);
    assert_eq!(weapon::class(&w, xbow).ammo_type, Some("Server.Items.Bolt"));
    let lb = new(&mut w, &ELVEN_COMPOSITE_LONGBOW);
    assert_eq!(weapon::class(&w, lb).ammo_type, Some("Server.Items.Arrow"));
    assert_eq!(weapon::max_range(&w, lb), 10);
    let juka = new(&mut w, &JUKA_BOW);
    assert_eq!(weapon::dex_requirement(&w, juka), 80);
}

#[test]
fn constructors_run_their_effects() {
    let mut w = World::new();
    era(&mut w, Expansion::ML);
    // Hue = ..; attributes; Slayer; the leaf's own InitMinHits/InitMaxHits (not Longsword's 31..110)
    let id = new(&mut w, &BLADE_OF_THE_RIGHTEOUS);
    assert_eq!(w.item(id).unwrap().hue, 0x47E);
    assert_eq!(
        weapon::data(&w, id).unwrap().slayer,
        SlayerName::Exorcism as i32
    );
    assert_eq!(
        muo_content::items::attr(
            &w,
            id,
            AttrKind::Attributes,
            AosAttribute::WeaponDamage as u32
        ),
        50
    );
    let d = weapon::data(&w, id).unwrap();
    assert_eq!((d.hit_points, d.max_hit_points), (255, 255));
    // ItemID = ..
    let id = new(&mut w, &BONE_CRUSHER);
    assert_eq!(w.item(id).unwrap().item_id, 0x1406);
    // Layer, Resource
    let id = new(&mut w, &ELVEN_SPELLBLADE);
    assert_eq!(w.item(id).unwrap().layer, Layer::TwoHanded);
    let id = new(&mut w, &YUMI);
    assert_eq!(
        weapon::data(&w, id).unwrap().resource,
        CraftResource::RegularWood
    );
    // a leaf of a leaf runs the parent's constructor for its own type
    let id = new(&mut w, &COLD_FORGED_BLADE);
    assert_eq!(w.item(id).unwrap().layer, Layer::TwoHanded);
    assert_eq!(
        w.item(id).unwrap().ty.full_name,
        "Server.Items.ColdForgedBlade"
    );
    // GetDamageTypes
    let t = wc::get_damage_types(&w, id, None);
    assert_eq!((t.phys, t.cold, t.fire), (0, 100, 0));
    let id = new(&mut w, &ZYRONIC_CLAW);
    let t = wc::get_damage_types(&w, id, None);
    assert_eq!(
        (t.phys, t.fire, t.cold, t.pois, t.nrgy, t.chaos, t.direct),
        (20, 20, 20, 20, 20, 0, 0)
    );
    // elemental damages
    let id = new(&mut w, &STORM_CALLER);
    assert_eq!(
        muo_content::items::attr(&w, id, AttrKind::Element, AosElementAttribute::Cold as u32),
        20
    );
    // UsesRemaining and the harvest system
    let id = new(&mut w, &PICKAXE);
    let d = weapon::data(&w, id).unwrap();
    assert_eq!((d.uses_remaining, d.show_uses_remaining), (50, true));
    assert_eq!(weapon::class(&w, id).harvest_system, Some("Mining"));
    let id = new(&mut w, &WAR_AXE);
    assert_eq!(weapon::class(&w, id).harvest_system, None);
    // Race.AllowGargoylesOnly, DefaultName, VirtualDamageBonus, resistances
    let id = new(&mut w, &BLOOD_BLADE);
    assert_eq!((BLOOD_BLADE.required_races)(&w, id), ALLOW_GARGOYLES_ONLY);
    let id = new(&mut w, &TRIBAL_SPEAR);
    assert_eq!(w.item(id).unwrap().hue, 837);
    assert_eq!(weapon::class(&w, id).virtual_damage_bonus, 25);
    assert_eq!(
        (TRIBAL_SPEAR.default_name)(&w, id).as_deref(),
        Some("a tribal spear")
    );
    let id = new(&mut w, &BONE_MACHETE);
    assert_eq!(w.item(id).unwrap().item_id, 0x20E);
    assert_eq!((BONE_MACHETE.fire_resistance)(&w, id), 1);
    assert_eq!(weapon::class(&w, id).primary_ability, None);
    // the wand
    let id = new(&mut w, &FIREWORKS_WAND);
    assert_eq!(charges(&w, id), 100);
    assert_eq!(w.item(id).unwrap().loot_type, LootType::Blessed);
    // ThrowingDagger is an Item
    let id = new(&mut w, &THROWING_DAGGER);
    let i = w.item(id).unwrap();
    assert_eq!((i.item_id, i.layer), (0xF52, Layer::OneHanded));
    assert_eq!((THROWING_DAGGER.default_weight)(&w, id), 1.0);
}

#[test]
fn fireworks_wand_record_has_its_charges() {
    let mut w = World::new();
    let id = new(&mut w, &FIREWORKS_WAND);
    set_charges(&mut w, id, 7);
    assert!(save(&w, id).ends_with(&[0, 0, 0, 0, 7, 0, 0, 0]));
}

#[test]
fn fists_absorb_damage_is_the_melee_weapons() {
    // Fists derive BaseMeleeWeapon: Attune Weapon / pre-AOS Reactive Armor must see unarmed hits.
    let c = special::FISTS.family::<weapon::WeaponClass>().unwrap();
    assert!(c.absorb_damage as usize == weapon::BASE_MELEE_WEAPON_CLASS.absorb_damage as usize);
    assert!(c.absorb_damage as usize != weapon::BASE_WEAPON_CLASS.absorb_damage as usize);
}
