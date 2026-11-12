//! The Test Center server mode (`testCenter.enable`): the speech
//! commands `set <stat|skill> <value>` and `help`, the filled bank box of a new character, and `TCHelpGump`.
//!
//! The items of the bank box are made by class name, like the supply bags.
//!
//! The items with their own constructor arguments are [`Special`] (potion kegs, runic tools, treasure maps,
//! spellbooks, runebooks, reagent bags, messages in a bottle, power scrolls, moonstones, a dye tub with its hue):
//! they are made through the class's constructor table, or by `muo_items` ([`TestCenterHooks`]).

use muo_primitives::dotnet::IgnoreCase;
use muo_primitives::{Point3D, Serial, utility};
use muo_server::World;
use muo_server::commands::CommandSystem;
use muo_server::events::SpeechEventArgs;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::{LootType, ops};
use muo_server::mobile::part1::props_b;
use muo_server::mobile::part2::messages as mm;
use muo_server::net_state::NetStateId;
use muo_server::packets::message::MessageType;
use muo_server::skills::{self, SkillName};
use muo_server::type_registry::{Phase, TypeRegistry};

use super::supply_bags::{Fill, make_filled};
use crate::aos::{AosAttribute, AttrKind};
use crate::core;
use crate::gumps::base::{LegacyGumpLogic, LegacyGumpObj, RelayInfo};
use crate::gumps::send_gump_to_mobile;
use crate::items::gold::new_bank_check;

/// `TestCenter.Enabled`.
#[derive(Default)]
pub struct TestCenterState {
    pub enabled: bool,
}

/// `TestCenter.Enabled`.
pub fn enabled(w: &World) -> bool {
    w.resources
        .get::<TestCenterState>()
        .is_some_and(|s| s.enabled)
}

/// An item of the bank box whose constructor takes more than an amount.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Special {
    /// `new PotionKeg { Held = 100, Type = PotionEffect.<effect>, Hue = hue }`.
    PotionKeg { effect: &'static str, hue: i32 },
    /// `new RunicHammer(CraftResource.<resource>, uses)`.
    RunicHammer { resource: &'static str, uses: i32 },
    /// `new RunicSewingKit(CraftResource.<resource>, uses)`.
    RunicSewingKit { resource: &'static str, uses: i32 },
    /// `new TreasureMap(level, Map.Trammel)`.
    TreasureMap { level: i32 },
    /// `new Spellbook(content)` (also `new Spellbook { Content = content }`).
    Spellbook { content: u64 },
    /// `new NecromancerSpellbook(content)`.
    NecromancerSpellbook { content: u64 },
    /// `new Runebook(10)` with `CurCharges = MaxCharges`.
    Runebook,
    /// `new BagOfReagents(amount) { Hue = hue }` (`hue` 0: not set).
    BagOfReagents { amount: i32, hue: i32 },
    /// `new BagOfNecroReagents(amount) { Hue = hue }`.
    BagOfNecroReagents { amount: i32, hue: i32 },
    /// `new BagOfAllReagents(amount)`.
    BagOfAllReagents { amount: i32 },
    /// `new MessageInABottle(felucca ? Map.Felucca : Map.Trammel, 4)`.
    MessageInABottle { felucca: bool },
    /// `new PowerScroll(skill, value)`.
    PowerScroll { skill: SkillName, value: f64 },
    /// `new StatCapScroll(value)`.
    StatCapScroll { value: i32 },
    /// `new Moonstone(felucca ? MoonstoneType.Felucca : MoonstoneType.Trammel)`.
    Moonstone { felucca: bool },
    /// `new DyeTub { DyedHue = hue, Redyable = false }`.
    UnredyableDyeTub { hue: i32 },
}

/// `new T(args..)` through the class's constructor table, the arguments parsed as `Add` parses them.
fn build(w: &mut World, class: &str, args: &[String]) -> Option<ItemId> {
    use crate::props::{ctors_of, parse_ctor_args};
    let full = format!("Server.Items.{class}");
    let ctors = ctors_of(w, &full)?;
    for c in ctors {
        let required = c.params.iter().take_while(|p| p.default.is_none()).count();
        if args.len() < required || args.len() > c.params.len() {
            continue;
        }
        let values = parse_ctor_args(w, c.params, args)?;
        if let muo_server::ids::EntityId::Item(id) = (c.make)(w, &values) {
            return Some(id);
        }
    }
    None
}

/// What only `muo_items` can make: the kegs, the runebook and the tub (state without a property table).
#[derive(Default)]
pub struct TestCenterHooks {
    pub make_special: Option<fn(&mut World, Special) -> Option<ItemId>>,
}

/// The item of a [`Special`].
fn make_special(w: &mut World, s: Special) -> Option<ItemId> {
    if matches!(
        s,
        Special::PotionKeg { .. } | Special::Runebook | Special::UnredyableDyeTub { .. }
    ) {
        let f = w
            .resources
            .get::<TestCenterHooks>()
            .and_then(|h| h.make_special)?;
        return f(w, s);
    }
    let map = |felucca: bool| if felucca { "Felucca" } else { "Trammel" };
    match s {
        Special::RunicHammer { resource, uses } => {
            build(w, "RunicHammer", &[resource.to_owned(), uses.to_string()])
        }
        Special::RunicSewingKit { resource, uses } => build(
            w,
            "RunicSewingKit",
            &[resource.to_owned(), uses.to_string()],
        ),
        Special::TreasureMap { level } => {
            build(w, "TreasureMap", &[level.to_string(), "Trammel".to_owned()])
        }
        Special::Spellbook { content } => build(w, "Spellbook", &[content.to_string()]),
        Special::NecromancerSpellbook { content } => {
            build(w, "NecromancerSpellbook", &[content.to_string()])
        }
        Special::BagOfReagents { amount, hue } | Special::BagOfNecroReagents { amount, hue } => {
            let class = if matches!(s, Special::BagOfReagents { .. }) {
                "BagOfReagents"
            } else {
                "BagOfNecroReagents"
            };
            let bag = build(w, class, &[amount.to_string()])?;
            if hue != 0
                && let Some(ty) = w.item(bag).map(|i| i.ty)
            {
                (ty.set_hue)(w, bag, hue);
            }
            Some(bag)
        }
        Special::BagOfAllReagents { amount } => build(w, "BagOfAllReagents", &[amount.to_string()]),
        Special::MessageInABottle { felucca } => build(
            w,
            "MessageInABottle",
            &[map(felucca).to_owned(), "4".to_owned()],
        ),
        Special::PowerScroll { skill, value } => {
            build(w, "PowerScroll", &[format!("{skill:?}"), value.to_string()])
        }
        Special::StatCapScroll { value } => build(w, "StatCapScroll", &[value.to_string()]),
        Special::Moonstone { felucca } => build(w, "Moonstone", &[map(felucca).to_owned()]),
        Special::PotionKeg { .. } | Special::Runebook | Special::UnredyableDyeTub { .. } => None,
    }
}

// ---- Configure / Initialize / speech ---------------------------------------------------------------------------------

/// `TestCenter.Configure()`.
pub fn configure(w: &mut World) {
    let e = w
        .config
        .get_or_update_setting("testCenter.enable", false)
        .unwrap_or(false);
    w.resources.get_mut::<TestCenterState>().enabled = e;
    w.resources
        .get_mut::<crate::player_mobile::hooks::SystemHooks>()
        .test_center_enabled = Some(enabled);
    // `AccountHandler.RestrictDeletion = !TestCenter.Enabled`
    w.resources
        .get_mut::<crate::accounting::handler::HandlerState>()
        .restrict_deletion = !e;
}

/// `TestCenter.Initialize()`.
pub fn initialize(w: &mut World) {
    // Register our speech handler
    if enabled(w) {
        w.events.speech.subscribe(on_speech);
    }
}

fn starts_with_insensitive(s: &str, prefix: &str) -> bool {
    s.len() >= prefix.len() && s.as_bytes()[..prefix.len()].eq_ignore_ascii_case(prefix.as_bytes())
}

/// `EventSink_Speech`.
fn on_speech(w: &mut World, args: &mut SpeechEventArgs) {
    if args.handled {
        return;
    }

    if starts_with_insensitive(&args.speech, "set") {
        let from = args.mobile;

        let mut tokens = args.speech.split(' ');
        if tokens.next().is_none() {
            return;
        }

        let name = tokens.next().unwrap_or("");
        let value_str = tokens.next().unwrap_or("");
        if value_str.is_empty() {
            return;
        }

        // `catch { // ignored }`
        let Ok(value) = value_str.trim().parse::<f64>() else {
            return;
        };

        if name.eq_ic("str") {
            change_stat(w, from, Stat::Str, value as i32);
        } else if name.eq_ic("dex") {
            change_stat(w, from, Stat::Dex, value as i32);
        } else if name.eq_ic("int") {
            change_stat(w, from, Stat::Int, value as i32);
        } else {
            change_skill(w, from, name, value);
        }
    } else if args.speech.eq_ic("help") {
        let g = LegacyGumpObj::new(40, 40, TcHelpGump);
        send_gump_to_mobile(w, args.mobile, Box::new(tc_help_layout(g)), false);
        args.handled = true;
    }
}

#[derive(Copy, Clone)]
enum Stat {
    Str,
    Dex,
    Int,
}

/// `ChangeStrength`, `ChangeDexterity` and `ChangeIntelligence`.
fn change_stat(w: &mut World, from: MobileId, which: Stat, value: i32) {
    if !(10..=125).contains(&value) {
        mm::send_localized_message(w, from, 1005628, "", 0x3B2); // Stats range between 10 and 125.
        return;
    }
    let Some((s, d, i, cap)) = w.mobile(from).map(|m| (m.str, m.dex, m.int, m.stat_cap)) else {
        return;
    };
    let total = match which {
        Stat::Str => value + d + i,
        Stat::Dex => s + value + i,
        Stat::Int => s + d + value,
    };
    if total > cap {
        // You can not exceed the stat cap.  Try setting another stat lower first.
        mm::send_localized_message(w, from, 1005629, "", 0x3B2);
    } else {
        match which {
            Stat::Str => props_b::set_raw_str(w, from, value),
            Stat::Dex => props_b::set_raw_dex(w, from, value),
            Stat::Int => props_b::set_raw_int(w, from, value),
        }
        mm::send_localized_message(w, from, 1005630, "", 0x3B2); // Your stats have been adjusted.
    }
}

/// `Enum.TryParse(name, true, out SkillName index)`: a name (any case) or a number.
fn parse_skill_name(name: &str) -> Option<i32> {
    if let Ok(n) = name.trim().parse::<i32>() {
        return Some(n);
    }
    (0..SkillName::COUNT as i32)
        .find(|&i| SkillName::from_i32(i).is_some_and(|s| format!("{s:?}").eq_ic(name)))
}

/// `ChangeSkill(from, name, value)`.
fn change_skill(w: &mut World, from: MobileId, name: &str, value: f64) {
    let (se, aos) = (core::se(w), core::aos(w));
    let index = parse_skill_name(name);
    let invalid = index.is_none_or(|i| (!se && i > 51) || (!aos && i > 48));
    let skill = index
        .filter(|_| !invalid)
        .and_then(SkillName::from_i32)
        .filter(|&s| {
            // `Skills[index]` is null past the table.
            let len = w
                .resources
                .get::<muo_server::skills::SkillTable>()
                .map_or(0, |t| t.info.len());
            (s as usize) < len
        });
    let Some(skill) = skill else {
        mm::send_localized_message(w, from, 1005631, "", 0x3B2); // You have specified an invalid skill to set.
        return;
    };

    let cap = skills::cap(w, from, skill);
    if value < 0.0 || value > cap {
        let info_name = w
            .resources
            .get::<muo_server::skills::SkillTable>()
            .and_then(|t| t.info.get(skill as usize))
            .map(|i| i.name.clone())
            .unwrap_or_default();
        mm::send_message(
            w,
            from,
            &format!("Your skill in {info_name} is capped at {cap:.1}."),
        );
        return;
    }

    let new_fixed = (value * 10.0) as i32;
    let old_fixed = skills::base_fixed(w, from, skill);
    let (total, owner_cap) = w
        .mobile(from)
        .and_then(|m| m.skills.as_ref())
        .map_or((0, 0), |s| (s.total, s.cap));
    if total - old_fixed + new_fixed > owner_cap {
        mm::send_message(
            w,
            from,
            "You can not exceed the skill cap.  Try setting another skill lower first.",
        );
    } else {
        skills::set_base_fixed(w, from, skill, new_fixed);
    }
}

// ---- the bank box ---------------------------------------------------------------------------------------------------

/// `MakeNewbie(item)`.
fn make_newbie(w: &mut World, item: ItemId) -> ItemId {
    if !core::aos(w) {
        ops::set_loot_type(w, item, LootType::Newbied);
    }
    item
}

/// `PlaceItemIn(parent, x, y, item)`.
fn place_item_in(w: &mut World, parent: ItemId, x: i32, y: i32, item: ItemId) {
    if let Some(ty) = w.item(parent).map(|i| i.ty) {
        (ty.add_item)(w, parent, item);
    }
    ops::set_location(w, item, Point3D::new(x, y, 0));
}

/// `parent.DropItem(item)`.
fn drop_item(w: &mut World, parent: ItemId, item: ItemId) {
    if let Some(ty) = w.item(parent).map(|i| i.ty) {
        (ty.drop_item)(w, parent, item);
    }
}

/// What a placed item is.
#[derive(Copy, Clone)]
enum It {
    /// A class by short name (`Server.Items.<name>`).
    P(&'static str, Fill),
    S(Special),
    /// `new BankCheck(worth)`.
    Check(i32),
}

fn make_it(w: &mut World, it: It) -> Option<ItemId> {
    match it {
        It::P(name, fill) => make_filled(w, &format!("Server.Items.{name}"), fill),
        It::S(Special::MessageInABottle { .. }) => {
            let felucca = !utility::random_bool(&mut w.rng); // `RandomBool() ? Map.Trammel : Map.Felucca`
            make_special(w, Special::MessageInABottle { felucca })
        }
        It::S(s) => make_special(w, s),
        It::Check(worth) => Some(new_bank_check(w, worth)),
    }
}

const fn p(name: &'static str) -> It {
    It::P(name, Fill::One)
}
const fn amt(name: &'static str, n: i32) -> It {
    It::P(name, Fill::Amount(n))
}
const fn uses(name: &'static str, n: i32) -> It {
    It::P(name, Fill::Uses(n))
}
const fn keg(effect: &'static str, hue: i32) -> It {
    It::S(Special::PotionKeg { effect, hue })
}
const fn hammer(resource: &'static str) -> It {
    It::S(Special::RunicHammer {
        resource,
        uses: 1000,
    })
}
const fn sewing(resource: &'static str) -> It {
    It::S(Special::RunicSewingKit {
        resource,
        uses: 1000,
    })
}
const fn map(level: i32) -> It {
    It::S(Special::TreasureMap { level })
}

/// The class of a container the AOS bank holds.
#[derive(Copy, Clone)]
enum Cont {
    /// `new WoodenBox { ItemID = 0xE7D, Hue = 0x489 }`.
    MoneyBox,
    Backpack {
        name: &'static str,
        hue: i32,
    },
    Bag {
        name: &'static str,
        hue: i32,
    },
}

struct Section {
    cont: Cont,
    /// Where the container goes in the bank.
    at: (i32, i32),
    items: &'static [(i32, i32, It)],
}

const MONEY: &[(i32, i32, It)] = &[
    (16, 51, It::Check(500000)),
    (28, 51, It::Check(250000)),
    (40, 51, It::Check(100000)),
    (52, 51, It::Check(100000)),
    (64, 51, It::Check(50000)),
    (16, 115, amt("Silver", 9000)),
    (34, 115, amt("Gold", 60000)),
];

const KEGS: &[(i32, i32, It)] = &[
    (45, 149, keg("CureGreater", 0x2D)),
    (69, 149, keg("HealGreater", 0x499)),
    (93, 149, keg("PoisonDeadly", 0x46)),
    (117, 149, keg("RefreshTotal", 0x21)),
    (141, 149, keg("ExplosionGreater", 0x74)),
    (93, 82, amt("Bottle", 1000)),
];

const TOOLS: &[(i32, i32, It)] = &[
    (30, 35, uses("TinkerTools", 1000)),
    (60, 35, p("HousePlacementTool")),
    (90, 35, uses("DovetailSaw", 1000)),
    (30, 68, p("Scissors")),
    (45, 68, uses("MortarPestle", 1000)),
    (75, 68, uses("ScribesPen", 1000)),
    (90, 68, uses("SmithHammer", 1000)),
    (30, 118, p("TwoHandedAxe")),
    (60, 118, uses("FletcherTools", 1000)),
    (90, 118, uses("SewingKit", 1000)),
    (36, 51, hammer("DullCopper")),
    (42, 51, hammer("ShadowIron")),
    (48, 51, hammer("Copper")),
    (54, 51, hammer("Bronze")),
    (61, 51, hammer("Gold")),
    (67, 51, hammer("Agapite")),
    (73, 51, hammer("Verite")),
    (79, 51, hammer("Valorite")),
    (36, 55, sewing("SpinedLeather")),
    (42, 55, sewing("HornedLeather")),
    (48, 55, sewing("BarbedLeather")),
];

const AMMO: &[(i32, i32, It)] = &[(48, 76, amt("Arrow", 5000)), (72, 76, amt("Bolt", 5000))];

const MAPS: &[(i32, i32, It)] = &[
    (30, 35, map(1)),
    (45, 35, map(2)),
    (60, 35, map(3)),
    (75, 35, map(4)),
    (90, 35, map(5)),
    (90, 35, map(6)),
    (30, 50, map(1)),
    (45, 50, map(2)),
    (60, 50, map(3)),
    (75, 50, map(4)),
    (90, 50, map(5)),
    (90, 50, map(6)),
    (55, 100, uses("Lockpick", 30)),
    (60, 100, p("Pickaxe")),
];

const RAW: &[(i32, i32, It)] = &[
    (92, 60, amt("BarbedLeather", 5000)),
    (92, 68, amt("HornedLeather", 5000)),
    (92, 76, amt("SpinedLeather", 5000)),
    (92, 84, amt("Leather", 5000)),
    (30, 118, amt("Cloth", 5000)),
    (30, 84, amt("Board", 5000)),
    (57, 80, amt("BlankScroll", 500)),
    (30, 35, amt("DullCopperIngot", 5000)),
    (37, 35, amt("ShadowIronIngot", 5000)),
    (44, 35, amt("CopperIngot", 5000)),
    (51, 35, amt("BronzeIngot", 5000)),
    (58, 35, amt("GoldIngot", 5000)),
    (65, 35, amt("AgapiteIngot", 5000)),
    (72, 35, amt("VeriteIngot", 5000)),
    (79, 35, amt("ValoriteIngot", 5000)),
    (86, 35, amt("IronIngot", 5000)),
    (30, 59, amt("RedScales", 5000)),
    (36, 59, amt("YellowScales", 5000)),
    (42, 59, amt("BlackScales", 5000)),
    (48, 59, amt("GreenScales", 5000)),
    (54, 59, amt("WhiteScales", 5000)),
    (60, 59, amt("BlueScales", 5000)),
];

const SPELLS: &[(i32, i32, It)] = &[
    (45, 105, It::S(Special::Spellbook { content: u64::MAX })),
    (
        65,
        105,
        It::S(Special::NecromancerSpellbook { content: 0xFFFF }),
    ),
    (85, 105, p("BookOfChivalry")),
    (105, 105, p("BookOfBushido")),
    (125, 105, p("BookOfNinjitsu")),
    (145, 105, It::S(Special::Runebook)),
    (
        45,
        150,
        It::S(Special::BagOfReagents {
            amount: 150,
            hue: 0x2D,
        }),
    ),
    (
        65,
        150,
        It::S(Special::BagOfNecroReagents {
            amount: 150,
            hue: 0x488,
        }),
    ),
    (140, 150, It::S(Special::BagOfAllReagents { amount: 500 })),
    (45, 75, p("RecallRune")),
    (55, 75, p("RecallRune")),
    (65, 75, p("RecallRune")),
    (75, 75, p("RecallRune")),
    (85, 75, p("RecallRune")),
    (95, 75, p("RecallRune")),
    (105, 75, p("RecallRune")),
    (115, 75, p("RecallRune")),
    (125, 75, p("RecallRune")),
    (141, 74, p("FireHorn")),
];

const ETHEREALS: &[(i32, i32, It)] = &[
    (45, 66, p("EtherealHorse")),
    (69, 82, p("EtherealOstard")),
    (93, 99, p("EtherealLlama")),
    (117, 115, p("EtherealKirin")),
    (45, 132, p("EtherealUnicorn")),
    (69, 66, p("EtherealRidgeback")),
    (93, 82, p("EtherealSwampDragon")),
    (117, 99, p("EtherealBeetle")),
];

const ARTIFACTS_1: &[(i32, i32, It)] = &[
    (45, 66, p("TitansHammer")),
    (69, 82, p("InquisitorsResolution")),
    (93, 99, p("BladeOfTheRighteous")),
    (117, 115, p("ZyronicClaw")),
];

const ARTIFACTS_2: &[(i32, i32, It)] = &[
    (45, 66, p("GauntletsOfNobility")),
    (69, 82, p("MidnightBracers")),
    (93, 99, p("VoiceOfTheFallenKing")),
    (117, 115, p("OrnateCrownOfTheHarrower")),
    (45, 132, p("HelmOfInsight")),
    (69, 66, p("HolyKnightsBreastplate")),
    (93, 82, p("ArmorOfFortune")),
    (117, 99, p("TunicOfFire")),
    (45, 115, p("LeggingsOfBane")),
    (69, 132, p("ArcaneShield")),
    (93, 66, p("Aegis")),
    (117, 82, p("RingOfTheVile")),
    (45, 99, p("BraceletOfHealth")),
    (69, 115, p("RingOfTheElements")),
    (93, 132, p("OrnamentOfTheMagician")),
    (117, 66, p("DivineCountenance")),
    (45, 82, p("JackalsCollar")),
    (69, 99, p("HuntersHeaddress")),
    (93, 115, p("HatOfTheMagi")),
    (117, 132, p("ShadowDancerLeggings")),
    (45, 66, p("SpiritOfTheTotem")),
    (69, 82, p("BladeOfInsanity")),
    (93, 99, p("AxeOfTheHeavens")),
    (117, 115, p("TheBeserkersMaul")),
    (45, 132, p("Frostbringer")),
    (69, 66, p("BreathOfTheDead")),
    (93, 82, p("TheDragonSlayer")),
    (117, 99, p("BoneCrusher")),
    (45, 115, p("StaffOfTheMagi")),
    (69, 132, p("SerpentsFang")),
    (93, 66, p("LegacyOfTheDreadLord")),
    (117, 82, p("TheTaskmaster")),
    (45, 99, p("TheDryadBow")),
];

const MINOR_ARTIFACTS: &[(i32, i32, It)] = &[
    (45, 66, p("LunaLance")),
    (69, 82, p("VioletCourage")),
    (93, 99, p("CavortingClub")),
    (117, 115, p("CaptainQuacklebushsCutlass")),
    (45, 132, p("NightsKiss")),
    (69, 66, p("ShipModelOfTheHMSCape")),
    (93, 82, p("AdmiralsHeartyRum")),
    (117, 99, p("CandelabraOfSouls")),
    (45, 115, p("IolosLute")),
    (69, 132, p("GwennosHarp")),
    (93, 66, p("ArcticDeathDealer")),
    (117, 82, p("EnchantedTitanLegBone")),
    (45, 99, p("NoxRangersHeavyCrossbow")),
    (69, 115, p("BlazeOfDeath")),
    (93, 132, p("DreadPirateHat")),
    (117, 66, p("BurglarsBandana")),
    (45, 82, p("GoldBricks")),
    (69, 99, p("AlchemistsBauble")),
    (93, 115, p("PhillipsWoodenSteed")),
    (117, 132, p("PolarBearMask")),
    (45, 66, p("BowOfTheJukaKing")),
    (69, 82, p("GlovesOfThePugilist")),
    (93, 99, p("OrcishVisage")),
    (117, 115, p("StaffOfPower")),
    (45, 132, p("ShieldOfInvulnerability")),
    (69, 66, p("HeartOfTheLion")),
    (93, 82, p("ColdBlood")),
    (117, 99, p("GhostShipAnchor")),
    (45, 115, p("SeahorseStatuette")),
    (69, 132, p("WrathOfTheDryad")),
    (93, 66, p("PixieSwatter")),
];

const TOKUNO: &[(i32, i32, It)] = &[
    (42, 70, p("Exiler")),
    (38, 53, p("HanzosBow")),
    (45, 40, p("TheDestroyer")),
    (92, 80, p("DragonNunchaku")),
    (42, 56, p("PeasantsBokuto")),
    (44, 71, p("TomeOfEnlightenment")),
    (35, 35, p("ChestOfHeirlooms")),
    (29, 0, p("HonorableSwords")),
    (49, 85, p("AncientUrn")),
    (51, 58, p("FluteOfRenewal")),
    (70, 51, p("PigmentsOfTokuno")),
    (40, 79, p("AncientSamuraiDo")),
    (51, 61, p("LegsOfStability")),
    (88, 78, p("GlovesOfTheSun")),
    (55, 62, p("AncientFarmersKasa")),
    (55, 83, p("ArmsOfTacticalExcellence")),
    (50, 85, p("DaimyosHelm")),
    (52, 78, p("BlackLotusHood")),
    (52, 79, p("DemonForks")),
    (33, 49, p("PilferedDancerFans")),
];

const BOWS: &[(i32, i32, It)] = &[
    (31, 84, p("Bow")),
    (78, 74, p("CompositeBow")),
    (53, 71, p("Crossbow")),
    (56, 39, p("HeavyCrossbow")),
    (82, 72, p("RepeatingCrossbow")),
    (49, 45, p("Yumi")),
];

fn make_container(w: &mut World, c: Cont) -> Option<ItemId> {
    match c {
        Cont::MoneyBox => {
            let b = crate::factory::make_item(w, "Server.Items.WoodenBox")?;
            muo_server::item::base::set_item_id(w, b, 0xE7D);
            muo_server::item::base::set_hue(w, b, 0x489);
            Some(b)
        }
        Cont::Backpack { name, hue } | Cont::Bag { name, hue } => {
            let class = if matches!(c, Cont::Backpack { .. }) {
                "Server.Items.Backpack"
            } else {
                "Server.Items.Bag"
            };
            let b = crate::factory::make_item(w, class)?;
            if hue != 0 {
                muo_server::item::base::set_hue(w, b, hue);
            }
            if !name.is_empty() {
                ops::set_name(w, b, Some(name.to_string()));
            }
            Some(b)
        }
    }
}

fn fill_section(w: &mut World, bank: ItemId, s: &Section) -> Option<ItemId> {
    let cont = make_container(w, s.cont)?;
    for &(x, y, it) in s.items {
        let newbie = matches!(it, It::S(Special::PotionKeg { .. }));
        if let Some(item) = make_it(w, it) {
            let item = if newbie { make_newbie(w, item) } else { item };
            place_item_in(w, cont, x, y, item);
        }
    }
    place_item_in(w, bank, s.at.0, s.at.1, cont);
    Some(cont)
}

/// `PowerScroll.Skills`.
pub fn power_scroll_skills(w: &World) -> Vec<SkillName> {
    use SkillName::*;
    let mut v = vec![
        Blacksmith,
        Tailoring,
        Swords,
        Fencing,
        Macing,
        Archery,
        Wrestling,
        Parry,
        Tactics,
        Anatomy,
        Healing,
        Magery,
        Meditation,
        EvalInt,
        MagicResist,
        AnimalTaming,
        AnimalLore,
        Veterinary,
        Musicianship,
        Provocation,
        Discordance,
        Peacemaking,
    ];
    if core::aos(w) {
        v.extend([Chivalry, Focus, Necromancy, Stealing, Stealth, SpiritSpeak]);
    }
    if core::se(w) {
        v.extend([Ninjitsu, Bushido]);
    }
    if core::ml(w) {
        v.push(Spellweaving);
    }
    v
}

fn fill_bank_aos(w: &mut World, m: MobileId) {
    let Some(bank) = props_b::bank_box(w, m) else {
        return;
    };

    // The new AOS bankboxes don't have powerscrolls, they are automatically 'applied':
    for s in power_scroll_skills(w) {
        skills::set_cap(w, m, s, 120.0);
    }

    props_b::set_stat_cap(w, m, 250); // the StatCap setter sends the stat update

    let bp = |name, hue| Cont::Backpack { name, hue };
    let bag = |name, hue| Cont::Bag { name, hue };
    let sections = [
        Section {
            cont: Cont::MoneyBox,
            at: (18, 169),
            items: MONEY,
        },
        Section {
            cont: bp("Various Potion Kegs", 0),
            at: (53, 169),
            items: KEGS,
        },
        Section {
            cont: bag("Tool Bag", 0),
            at: (118, 169),
            items: TOOLS,
        },
        Section {
            cont: bag("Bag Of Archery Ammo", 0),
            at: (118, 124),
            items: AMMO,
        },
        Section {
            cont: bag("Bag Of Treasure Maps", 0),
            at: (98, 124),
            items: MAPS,
        },
        Section {
            cont: bag("Raw Materials Bag", 0x835),
            at: (98, 169),
            items: RAW,
        },
        Section {
            cont: bp("Spell Casting Stuff", 0x480),
            at: (78, 169),
            items: SPELLS,
        },
        Section {
            cont: bp("Bag Of Ethy's!", 0x490),
            at: (38, 124),
            items: ETHEREALS,
        },
        Section {
            cont: bp("Bag of Artifacts", 0x48F),
            at: (58, 124),
            items: ARTIFACTS_1,
        },
        Section {
            cont: bp("Bag of Artifacts", 0x48F),
            at: (78, 124),
            items: ARTIFACTS_2,
        },
    ];
    for s in &sections {
        fill_section(w, bank, s);
    }

    // The minor artifacts, then the ten messages in a bottle.
    if let Some(cont) = make_container(w, bp("Bag of Minor Artifacts", 0x48F)) {
        for &(x, y, it) in MINOR_ARTIFACTS {
            if let Some(item) = make_it(w, it) {
                place_item_in(w, cont, x, y, item);
            }
        }
        for _ in 0..10 {
            if let Some(item) = make_it(w, It::S(Special::MessageInABottle { felucca: false })) {
                place_item_in(w, cont, 117, 128, item);
            }
        }
        place_item_in(w, bank, 18, 124, cont);
    }

    if core::se(w) {
        fill_section(
            w,
            bank,
            &Section {
                cont: bag("Tokuno Minor Artifacts", 0x501),
                at: (58, 124),
                items: TOKUNO,
            },
        );
    }

    if core::se(w) {
        // This bag came only after SE.
        if let Some(cont) = fill_section(
            w,
            bank,
            &Section {
                cont: bag("Bag of Bows", 0),
                at: (108, 135),
                items: BOWS,
            },
        ) {
            let items = w.item(cont).map(|i| i.items.clone()).unwrap_or_default();
            for it in items {
                if w.item(it)
                    .is_some_and(|i| i.ty.is(&crate::items::weapon::BASE_RANGED))
                {
                    crate::items::set_attr(
                        w,
                        it,
                        AttrKind::Attributes,
                        AosAttribute::WeaponSpeed as u32,
                        35,
                    );
                    crate::items::set_attr(
                        w,
                        it,
                        AttrKind::Attributes,
                        AosAttribute::WeaponDamage as u32,
                        35,
                    );
                }
            }
        }
    }
}

/// `TestCenter.FillBankbox(m)`.
pub fn fill_bankbox(w: &mut World, m: MobileId) {
    if core::aos(w) {
        fill_bank_aos(w, m);
        return;
    }

    let Some(bank) = props_b::bank_box(w, m) else {
        return;
    };

    let check = new_bank_check(w, 1_000_000);
    drop_item(w, bank, check);

    // Full spellbook
    if let Some(book) = make_special(w, Special::Spellbook { content: u64::MAX }) {
        drop_item(w, bank, book);
    }

    for felucca in [true, false] {
        if let Some(bag) = crate::factory::make_item(w, "Server.Items.Bag") {
            for _ in 0..5 {
                if let Some(stone) = make_special(w, Special::Moonstone { felucca }) {
                    drop_item(w, bag, stone);
                }
            }
            // Felucca moonstones, then Trammel moonstones
            drop_item(w, bank, bag);
        }
    }

    // Treasure maps
    for level in 1..=5 {
        if let Some(m) = make_special(w, Special::TreasureMap { level }) {
            drop_item(w, bank, m);
        }
    }

    // Bag containing 50 of each reagent
    if let Some(bag) = make_special(w, Special::BagOfReagents { amount: 50, hue: 0 }) {
        drop_item(w, bank, bag);
    }

    let newbie_drop = |w: &mut World, it: It| {
        if let Some(item) = make_it(w, it) {
            let item = make_newbie(w, item);
            drop_item(w, bank, item);
        }
    };
    let plain_drop = |w: &mut World, it: It| {
        if let Some(item) = make_it(w, it) {
            drop_item(w, bank, item);
        }
    };

    // Craft tools
    newbie_drop(w, p("Scissors"));
    for n in [
        "SewingKit",
        "SmithHammer",
        "FletcherTools",
        "DovetailSaw",
        "MortarPestle",
        "ScribesPen",
        "TinkerTools",
    ] {
        newbie_drop(w, uses(n, 1000));
    }

    // A few dye tubs
    plain_drop(w, p("Dyes"));
    plain_drop(w, p("DyeTub"));
    plain_drop(w, p("DyeTub"));
    plain_drop(w, p("BlackDyeTub"));
    if let Some(tub) = make_special(w, Special::UnredyableDyeTub { hue: 0x485 }) {
        drop_item(w, bank, tub);
    }

    // Some food
    newbie_drop(w, amt("Apple", 1000));

    // Resources
    for n in [
        "Feather",
        "BoltOfCloth",
        "BlankScroll",
        "Hides",
        "Bandage",
        "Bottle",
        "Log",
    ] {
        newbie_drop(w, amt(n, 1000));
    }
    for n in [
        "IronIngot",
        "DullCopperIngot",
        "ShadowIronIngot",
        "CopperIngot",
        "BronzeIngot",
        "GoldIngot",
        "AgapiteIngot",
        "VeriteIngot",
        "ValoriteIngot",
    ] {
        newbie_drop(w, amt(n, 5000));
    }

    // Reagents
    for n in [
        "BlackPearl",
        "Bloodmoss",
        "Garlic",
        "Ginseng",
        "MandrakeRoot",
        "Nightshade",
        "SulfurousAsh",
        "SpidersSilk",
    ] {
        newbie_drop(w, amt(n, 1000));
    }

    // Some extra starting gold
    newbie_drop(w, amt("Gold", 9000));

    // 5 blank recall runes
    for _ in 0..5 {
        newbie_drop(w, p("RecallRune"));
    }

    // AddPowerScrolls
    if let Some(bag) = crate::factory::make_item(w, "Server.Items.Bag") {
        for skill in power_scroll_skills(w) {
            if let Some(s) = make_special(
                w,
                Special::PowerScroll {
                    skill,
                    value: 120.0,
                },
            ) {
                drop_item(w, bag, s);
            }
        }
        if let Some(s) = make_special(w, Special::StatCapScroll { value: 250 }) {
            drop_item(w, bag, s);
        }
        drop_item(w, bank, bag);
    }
}

// ---- TCHelpGump -----------------------------------------------------------------------------------------------------

pub struct TcHelpGump;

fn tc_help_layout(mut g: LegacyGumpObj<TcHelpGump>) -> LegacyGumpObj<TcHelpGump> {
    let l = &mut g.layout;
    l.add_page(0);
    l.add_background(0, 0, 160, 120, 5054);

    l.add_button(10, 10, 0xFB7, 0xFB9, 1);
    l.add_label(45, 10, 0x34, "muo");

    l.add_button(10, 35, 0xFB7, 0xFB9, 2);
    l.add_label(45, 35, 0x34, "List of skills");

    l.add_button(10, 60, 0xFB7, 0xFB9, 3);
    l.add_label(45, 60, 0x34, "Command list");

    l.add_button(10, 85, 0xFB1, 0xFB3, 0);
    l.add_label(45, 85, 0x34, "Close");
    g
}

impl LegacyGumpLogic for TcHelpGump {
    const TYPE_NAME: &'static str = "Server.Misc.TestCenter+TCHelpGump";

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let mobile = w.net_states.get(&sender).and_then(|n| n.mobile);
        match info.button_id {
            1 => muo_server::packets::player::send_launch_browser(w, sender, ""),
            2 => {
                // List of skills
                let mut strings: Vec<String> = (0..SkillName::COUNT as i32)
                    .filter_map(SkillName::from_i32)
                    .map(|s| format!("{s:?}"))
                    .collect();
                // `Array.Sort(strings)`: the culture comparer
                muo_primitives::dotnet::net_sort(&mut strings, |a, b| {
                    muo_primitives::dotnet::string_compare(a, b)
                });

                let mut sb = String::new();
                if let Some(first) = strings.first() {
                    sb.push_str(first);
                }

                for v in strings.iter().skip(1) {
                    if sb.len() + 1 + v.len() >= 256 {
                        send_system_label(w, sender, &sb);
                        sb.clear();
                        sb.push_str(v);
                    } else {
                        sb.push(' ');
                        sb.push_str(v);
                    }
                }

                if !sb.is_empty() {
                    send_system_label(w, sender, &sb);
                }
            }
            3 => {
                // Command list
                if let Some(m) = mobile {
                    let prefix = w.resources.get_mut::<CommandSystem>().prefix.clone();
                    mm::send_ascii_message_hue(
                        w,
                        m,
                        0x482,
                        &format!("The command prefix is \"{prefix}\""),
                    );
                    // `CommandHandlers.Help_OnCommand(new CommandEventArgs(m, "help", "", []))`
                    let entry = w
                        .resources
                        .get_mut::<CommandSystem>()
                        .entries
                        .get("help")
                        .cloned();
                    if let Some(entry) = entry {
                        let mut e = muo_server::commands::CommandEventArgs::new(
                            m,
                            "help".to_string(),
                            String::new(),
                            Vec::new(),
                        );
                        (entry.handler)(w, &mut e);
                    }
                }
            }
            _ => {}
        }
    }
}

/// `sender.SendMessage(Serial.MinusOne, -1, MessageType.Label, 0x35, 3, true, null, "System", text)`.
fn send_system_label(w: &mut World, ns: NetStateId, text: &str) {
    muo_server::packets::message::send_message(
        w,
        ns,
        Serial::MINUS_ONE,
        -1,
        MessageType::LABEL,
        0x35,
        3,
        true,
        None,
        "System",
        text,
    );
}

pub fn register(t: &mut TypeRegistry) {
    t.register_phase_default(
        Phase::Configure,
        "Server.Misc.TestCenter.Configure",
        configure,
    );
    t.register_phase_default(
        Phase::Initialize,
        "Server.Misc.TestCenter.Initialize",
        initialize,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use muo_server::mobile::MOBILE_BASE;

    #[test]
    fn skill_names_parse_in_any_case_and_as_numbers() {
        assert_eq!(parse_skill_name("magery"), Some(25));
        assert_eq!(parse_skill_name("MagicResist"), Some(26));
        assert_eq!(parse_skill_name("7"), Some(7));
        assert_eq!(parse_skill_name("nonsense"), None);
    }

    #[test]
    fn set_str_checks_the_range_and_the_cap() {
        let mut w = World::new();
        let m = w.new_mobile(&MOBILE_BASE);
        {
            let x = w.mobile_mut(m).unwrap();
            x.str = 50;
            x.dex = 50;
            x.int = 50;
            x.stat_cap = 225;
        }
        change_stat(&mut w, m, Stat::Str, 9);
        assert_eq!(w.mobile(m).unwrap().str, 50, "below 10 is refused");
        change_stat(&mut w, m, Stat::Str, 126);
        assert_eq!(w.mobile(m).unwrap().str, 50, "above 125 is refused");
        change_stat(&mut w, m, Stat::Str, 125);
        assert_eq!(
            w.mobile(m).unwrap().str,
            125,
            "125 + 50 + 50 is exactly the cap"
        );
        change_stat(&mut w, m, Stat::Dex, 51);
        assert_eq!(
            w.mobile(m).unwrap().dex,
            50,
            "one point over the cap is refused"
        );
    }

    #[test]
    fn the_power_scroll_list_grows_with_the_expansion() {
        let mut w = World::new();
        w.core.expansion = Some(muo_data::expansion::Expansion::T2A);
        assert_eq!(power_scroll_skills(&w).len(), 22);
        w.core.expansion = Some(muo_data::expansion::Expansion::ML);
        assert_eq!(power_scroll_skills(&w).len(), 22 + 6 + 2 + 1);
    }
}
