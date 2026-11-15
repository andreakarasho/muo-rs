//! The last equipment class tables and constructors moved to `Data/Templates/Items`: the samples, `Fists`,
//! `ProspectorsTool`, `FireworksWand`, the wands, quivers, talismans, decoration artifacts and the special
//! jewels/clothing. Each converted class keeps its values in its template.

use muo_content::items::equip_props::LAYERS;
use muo_content::items::weapon::{self, Ability, WeaponAnimation, WeaponType};
use muo_content::templates::item_steps::Step;
use muo_content::templates::{self, Kind, props};
use muo_data::expansion::Expansion;
use muo_server::skills::SkillName;
use serde_json::Value;

/// (class, prop or key its template must hold).
const CONVERTED: &[(&str, &str)] = &[
    ("Server.Items.Longsword", "graphic"),
    ("Server.Items.Katana", "primaryAbility"),
    ("Server.Items.Bow", "resource"),
    ("Server.Items.PlateChest", "materialType"),
    ("Server.Items.LeatherChest", "defMedAllowance"),
    ("Server.Items.ChainChest", "graphic"),
    ("Server.Items.Cap", "baseColdResistance"),
    ("Server.Items.FancyShirt", "graphic"),
    ("Server.Items.GoldRing", "graphic"),
    ("Server.Items.Necklace", "graphic"),
    ("Server.Items.Fists", "defSkill"),
    ("Server.Items.ProspectorsTool", "usesRemaining"),
    ("Server.Items.FireworksWand", "weaponCharges"),
    ("Server.Items.BaseWand", "aosSpeed"),
    ("Server.Items.FireballWand", "wandSpell"),
    ("Server.Items.BaseQuiver", "gumpId"),
    ("Server.Items.ElvenQuiver", "quiverWeightReduction"),
    ("Server.Items.QuiverOfFire", "bowDamageSplit"),
    ("Server.Items.QuiverOfElements", "bowDamageSplit"),
    ("Server.Items.QuiverOfRage", "quiverDamageIncrease"),
    ("Server.Items.QuiverOfInfinity", "quiverLowerAmmoCost"),
    ("Server.Items.BaseTalisman", "labelNumber"),
    ("Server.Items.RunedSwitch", "itemId"),
    ("Server.Items.HollowPrism", "itemId"),
    ("Server.Items.BaseFormTalisman", "layer"),
    ("Server.Items.FerretFormTalisman", "talismanForm"),
    ("Server.Items.AcademicBooksArtifact", "artifactRarity"),
    ("Server.Items.TowerLanternArtifact", "artifactRarity"),
    ("Server.Items.LuckyNecklace", "hue"),
    ("Server.Items.EarringsOfProtection", "layer"),
    ("Server.Items.BraceletOfBinding", "hue"),
    ("Server.Items.LeatherGlovesOfMining", "graphic"),
    ("Server.Items.RobeOfTheEclipse", "hue"),
    ("Server.Items.RobeOfTheEquinox", "hue"),
    ("Server.Items.PadsOfTheCuSidhe", "hue"),
    ("Server.Items.BoneMachete", "physicalResistance"),
    ("Server.Items.GargishLeatherWingArmor", "energyResistance"),
    ("Server.Items.ResilientBracer", "physicalResistance"),
    ("Server.Items.WeddingRing", "defaultName"),
    ("Server.Items.TribalSpear", "defaultName"),
    ("Server.Items.OrcishKinMask", "defaultName"),
    ("Server.Items.ThrowingDagger", "defaultName"),
    ("Server.Items.Beads", "itemId"),
];

#[test]
fn converted_equipment_classes_hold_their_values_in_templates() {
    let t = templates::table(&mut crate::test_world());
    for (name, key) in CONVERTED {
        let tm = t
            .get(Kind::Item, name)
            .unwrap_or_else(|| panic!("{name}: no template"));
        let has = tm.props.contains_key(*key) || tm.item_steps.iter().any(|s| sets(s, key));
        assert!(has, "{name}: no \"{key}\"");
    }
    for tm in t.items.values() {
        if let Some(Value::String(l)) = tm.props.get("layer") {
            assert!(LAYERS.get(l).is_some(), "layer {l}");
        }
    }
}

/// The template step is key `key`.
fn sets(s: &Step, key: &str) -> bool {
    match s {
        Step::ItemId(_) => key == "itemId",
        Step::Hue(_) => key == "hue",
        Step::Layer(_) => key == "layer",
        Step::Ext(name, ..) => *name == key,
        _ => false,
    }
}

/// `Fists` is not constructible, so the snapshot of every class does not see it: its stats in each era are the
/// values of the old class table.
#[test]
fn fists_keep_their_stats() {
    for (era, dmg, speed) in [
        (Expansion::LBR, (1, 8), 30.0),
        (Expansion::AOS, (1, 4), 50.0),
        (Expansion::ML, (1, 4), 2.5),
    ] {
        let mut w = crate::test_world();
        w.core.expansion = Some(era);
        let id = crate::weapons::special::fists_new(&mut w, &crate::weapons::special::FISTS);
        let i = w.item(id).unwrap();
        assert_eq!((i.item_id, i.visible(), i.movable()), (0, false, false));
        assert_eq!(
            weapon::abilities(&w, id),
            (Some(Ability::Disarm), Some(Ability::ParalyzingBlow))
        );
        assert_eq!(
            (weapon::min_damage(&w, id), weapon::max_damage(&w, id)),
            dmg,
            "{era:?}"
        );
        assert_eq!(f64::from(weapon::speed(&w, id)), speed, "{era:?}");
        assert_eq!(weapon::str_requirement(&w, id), 0);
        assert_eq!(
            (weapon::hit_sound(&w, id), weapon::miss_sound(&w, id)),
            (-1, -1)
        );
        assert_eq!(weapon::skill(&w, id), SkillName::Wrestling);
        assert_eq!(weapon::weapon_type(&w, id), WeaponType::Fists);
        assert_eq!(weapon::animation(&w, id), WeaponAnimation::Wrestle);
    }
}

/// The spell each wand casts (`OnWandUse`), now its `wandSpell` prop; `IDWand` targets.
#[test]
fn wands_cast_their_spells() {
    let mut w = crate::test_world();
    for (ty, spell) in [
        (
            &crate::wands::CLUMSY_WAND,
            Some("Server.Spells.First.ClumsySpell"),
        ),
        (
            &crate::wands::FEEBLE_WAND,
            Some("Server.Spells.First.FeeblemindSpell"),
        ),
        (
            &crate::wands::FIREBALL_WAND,
            Some("Server.Spells.Third.FireballSpell"),
        ),
        (
            &crate::wands::GREATER_HEAL_WAND,
            Some("Server.Spells.Fourth.GreaterHealSpell"),
        ),
        (
            &crate::wands::HARM_WAND,
            Some("Server.Spells.Second.HarmSpell"),
        ),
        (
            &crate::wands::HEAL_WAND,
            Some("Server.Spells.First.HealSpell"),
        ),
        (&crate::wands::ID_WAND, None),
        (
            &crate::wands::LIGHTNING_WAND,
            Some("Server.Spells.Fourth.LightningSpell"),
        ),
        (
            &crate::wands::MAGIC_ARROW_WAND,
            Some("Server.Spells.First.MagicArrowSpell"),
        ),
        (
            &crate::wands::MANA_DRAIN_WAND,
            Some("Server.Spells.Fourth.ManaDrainSpell"),
        ),
        (
            &crate::wands::WEAKNESS_WAND,
            Some("Server.Spells.First.WeakenSpell"),
        ),
    ] {
        let id = (ty.construct.unwrap())(&mut w);
        let got = props::item(&w, id, "wandSpell").and_then(Value::as_str);
        assert_eq!(got, spell, "{}", ty.full_name);
    }
}
