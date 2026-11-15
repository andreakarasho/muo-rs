//! The weapon classes whose constructor statements and `LabelNumber`/`ArtifactRarity` live in
//! `Data/Templates/Items/weapons`: each still has its template, and the class reads the moved values.

use muo_content::items::weapon;
use muo_content::templates::{self, Kind};
use muo_server::World;

use super::tests::{find, registry};

/// (class, its constructor is its template, `labelNumber` prop or 0, `artifactRarity` prop or 0).
const MOVED: &[(&str, bool, i32, i32)] = &[
    ("Server.Engines.Quests.Doom.ChylothStaff", true, 1041111, 0),
    ("Server.Items.AbyssReaver", false, 1112694, 0),
    ("Server.Items.AdventurersMachete", true, 1073533, 0),
    ("Server.Items.AncientWildStaff", true, 1073550, 0),
    ("Server.Items.ArcanistsWildStaff", true, 1073549, 0),
    ("Server.Items.AssassinsShortbow", true, 1073512, 0),
    ("Server.Items.AxeOfTheHeavens", true, 1061106, 11),
    ("Server.Items.BansheesCall", false, 1113529, 0),
    ("Server.Items.BarbedLongbow", true, 1073505, 0),
    ("Server.Items.BattleAxe", true, 0, 0),
    ("Server.Items.BladeOfInsanity", true, 1061088, 11),
    ("Server.Items.BladeOfTheRighteous", false, 1061107, 10),
    ("Server.Items.BlightGrippedLongbow", true, 1072907, 0),
    ("Server.Items.BoneCrusher", true, 1061596, 11),
    ("Server.Items.BoneMachete", true, 0, 0),
    ("Server.Items.Boomerang", true, 0, 0),
    ("Server.Items.BreathOfTheDead", true, 1061109, 11),
    ("Server.Items.ButchersWarCleaver", false, 1073526, 0),
    ("Server.Items.ChargedAssassinSpike", true, 1073518, 0),
    ("Server.Items.ColdForgedBlade", false, 1072916, 0),
    ("Server.Items.CorruptedRuneBlade", true, 1073540, 0),
    ("Server.Items.Cyclone", true, 0, 0),
    ("Server.Items.Daisho", true, 0, 0),
    ("Server.Items.DarkglowScimitar", true, 1073542, 0),
    ("Server.Items.DiseasedMachete", true, 1073536, 0),
    ("Server.Items.ElvenSpellblade", true, 0, 0),
    ("Server.Items.EmeraldMace", true, 1073530, 0),
    ("Server.Items.FaerieFire", false, 1072908, 0),
    ("Server.Items.FierySpellblade", true, 1073515, 0),
    ("Server.Items.Frostbringer", true, 1061111, 11),
    ("Server.Items.FrozenLongbow", true, 1073507, 0),
    ("Server.Items.GargoylesPickaxe", false, 1041281, 0),
    ("Server.Items.GlacialStaff", false, 1017413, 0),
    ("Server.Items.GuardianAxe", true, 1073545, 0),
    ("Server.Items.HammerPick", true, 0, 0),
    ("Server.Items.HardenedWildStaff", true, 1073552, 0),
    ("Server.Items.HeavyOrnateAxe", true, 1073548, 0),
    ("Server.Items.IcyScimitar", true, 1073543, 0),
    ("Server.Items.IcySpellblade", true, 1073514, 0),
    ("Server.Items.Kama", true, 0, 0),
    ("Server.Items.KnightsWarCleaver", true, 1073525, 0),
    ("Server.Items.Lajatang", true, 0, 0),
    ("Server.Items.LeafbladeOfEase", true, 1073524, 0),
    ("Server.Items.LegacyOfTheDreadLord", true, 1060860, 10),
    ("Server.Items.LightweightShortbow", false, 1073510, 0),
    ("Server.Items.LongbowOfMight", true, 1073508, 0),
    ("Server.Items.Luckblade", true, 1073522, 0),
    ("Server.Items.LuminousRuneBlade", false, 1072922, 0),
    ("Server.Items.MacheteOfDefense", true, 1073535, 0),
    ("Server.Items.MagekillerAssassinSpike", true, 1073519, 0),
    ("Server.Items.MagekillerLeafblade", true, 1073523, 0),
    ("Server.Items.MagesRuneBlade", true, 1073538, 0),
    ("Server.Items.MischiefMaker", false, 1072910, 0),
    ("Server.Items.MysticalShortbow", true, 1073511, 0),
    ("Server.Items.NoDachi", true, 0, 0),
    ("Server.Items.OrcishMachete", true, 1073534, 0),
    ("Server.Items.OrnateAxe", true, 0, 0),
    ("Server.Items.OverseerSunderedBlade", false, 1072920, 0),
    ("Server.Items.PhantomStaff", true, 1072919, 0),
    ("Server.Items.ProspectorsTool", false, 1049065, 0),
    ("Server.Items.RangersShortbow", true, 1073509, 0),
    ("Server.Items.RaptorClaw", false, 1112394, 0),
    ("Server.Items.RubyMace", true, 1073529, 0),
    ("Server.Items.RuneBlade", true, 0, 0),
    ("Server.Items.RuneBladeOfKnowledge", true, 1073539, 0),
    ("Server.Items.RuneCarvingKnife", true, 1072915, 0),
    ("Server.Items.Runesabre", true, 1073537, 0),
    ("Server.Items.Sai", true, 0, 0),
    ("Server.Items.SapphireMace", true, 1073531, 0),
    ("Server.Items.SerpentsFang", true, 1061601, 11),
    ("Server.Items.SerratedWarCleaver", true, 1073527, 0),
    ("Server.Items.ShardThrasher", true, 1072918, 0),
    ("Server.Items.SilvanisFeywoodBow", true, 1072955, 0),
    ("Server.Items.SilverEtchedMace", false, 1073532, 0),
    ("Server.Items.SingingAxe", true, 1073546, 0),
    ("Server.Items.SlayerLongbow", false, 1073506, 0),
    ("Server.Items.SoulGlaive", true, 0, 0),
    ("Server.Items.SpellbladeOfDefense", true, 1073516, 0),
    ("Server.Items.StaffOfTheMagi", true, 1061600, 11),
    ("Server.Items.StoneSlithClaw", false, 1112393, 0),
    ("Server.Items.StormCaller", true, 1113530, 0),
    ("Server.Items.SturdyPickaxe", false, 1045126, 0),
    ("Server.Items.Tekagi", true, 0, 0),
    ("Server.Items.Tessen", true, 0, 0),
    ("Server.Items.Tetsubo", true, 0, 0),
    ("Server.Items.TheBeserkersMaul", true, 1061108, 11),
    ("Server.Items.TheDragonSlayer", false, 1061248, 11),
    ("Server.Items.TheDryadBow", false, 1061090, 11),
    ("Server.Items.TheNightReaper", false, 1072912, 0),
    ("Server.Items.TheTaskmaster", true, 1061110, 10),
    ("Server.Items.ThornedWildStaff", true, 1073551, 0),
    ("Server.Items.ThunderingAxe", true, 1073547, 0),
    ("Server.Items.TitansHammer", true, 1060024, 10),
    ("Server.Items.TribalSpear", true, 0, 0),
    ("Server.Items.TrueAssassinSpike", true, 1073517, 0),
    ("Server.Items.TrueLeafblade", true, 1073521, 0),
    ("Server.Items.TrueRadiantScimitar", true, 1073541, 0),
    ("Server.Items.TrueSpellblade", true, 1073513, 0),
    ("Server.Items.TrueWarCleaver", true, 1073528, 0),
    ("Server.Items.TwinklingScimitar", true, 1073544, 0),
    ("Server.Items.ValkyriesGlaive", false, 1113531, 5),
    ("Server.Items.Wakizashi", true, 0, 0),
    ("Server.Items.WarHammer", true, 0, 0),
    ("Server.Items.WindOfCorruption", false, 1150358, 0),
    ("Server.Items.WoundingAssassinSpike", true, 1073520, 0),
    ("Server.Items.ZyronicClaw", false, 1061593, 10),
];

#[test]
fn every_moved_class_has_its_template_and_props() {
    let types = registry();
    let mut w = World::new();
    let t = templates::table(&mut w);
    for &(name, steps, label, rarity) in MOVED {
        let tmpl = t
            .get(Kind::Item, name)
            .unwrap_or_else(|| panic!("{name}: no template"));
        assert_eq!(
            !tmpl.item_steps.is_empty(),
            steps,
            "{name}: constructor statements"
        );
        let prop = |k: &str| tmpl.props.get(k).and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        assert_eq!(prop("labelNumber"), label, "{name}: labelNumber");
        assert_eq!(prop("artifactRarity"), rarity, "{name}: artifactRarity");

        let ty = find(&types, name);
        let id = (ty.construct.unwrap())(&mut w);
        if label != 0 {
            assert_eq!((ty.label_number)(&w, id), label, "{name}");
        }
        assert_eq!(weapon::artifact_rarity(&w, id), rarity, "{name}");
    }
}
