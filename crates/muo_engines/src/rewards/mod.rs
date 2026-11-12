//! Veteran rewards (`RewardSystem`, `RewardEntry`, `RewardList`, `RewardCategory`, `RewardNoticeGump`):
//! the reward tables, account age levels, `CheckIsUsableBy`/`GetRewardYear` (set as `clothing::special::RewardHooks`
//! for `RewardCloak`, `RewardRobe` and `RewardDress`) and the login notice.
//!
//! The Character Statue Maker (CharacterStatue, CharacterStatueMaker, CharacterStatuePlinth, its packets and gump) is
//! in `statue`; the reward item classes themselves (Items/Special/Veteran Rewards) are another folder's, so
//! `Construct` makes them by name and `RewardHooks` sets what only they know (`IsRewardItem` of a deed, a soulstone's
//! account).

pub mod gumps;
pub mod statue;

use muo_content::accounting;
use muo_content::gumps::base::{RelayInfo, StaticGumpLogic, StaticGumpObj};
use muo_content::gumps::builder::StaticGumpBuilder;
use muo_content::player_mobile as pmob;
use muo_data::expansion::Expansion;
use muo_primitives::TimeSpan;
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::mobile::part2::messages::send_localized_message_affix;
use muo_server::net_state::NetStateId;
use muo_server::type_registry::{DEFAULT_CALL_PRIORITY, Phase, TypeRegistry};

/// `RewardCategory`: the names in `m_Categories` order.
pub static CATEGORIES: [i32; 6] = [1049750, 1049752, 1049751, 1049753, 1049754, 1078596];

const MONSTER_STATUES: usize = 0;
const CLOAKS_AND_ROBES: usize = 1;
const ETHEREAL_STEEDS: usize = 2;
const SPECIAL_DYE_TUBS: usize = 3;
const HOUSE_ADD_ONS: usize = 4;
const MISCELLANEOUS: usize = 5;

/// `RewardEntry`: category, name cliloc, item class, required expansion and constructor arguments (ints and enum
/// numbers; the `RewardCloak`, `RewardRobe` and `RewardDress` ones are `(hue, label number)`).
pub struct Entry {
    pub category: usize,
    pub name: i32,
    pub item_type: &'static str,
    pub required_expansion: Expansion,
    pub args: &'static [i64],
}

const fn e(category: usize, name: i32, item_type: &'static str, args: &'static [i64]) -> Entry {
    Entry {
        category,
        name,
        item_type,
        required_expansion: Expansion::None,
        args,
    }
}

const fn ex(
    category: usize,
    name: i32,
    item_type: &'static str,
    expansion: Expansion,
    args: &'static [i64],
) -> Entry {
    Entry {
        category,
        name,
        item_type,
        required_expansion: expansion,
        args,
    }
}

// Hues of the cloak, robe and dress rewards.
const BRONZE: i64 = 0x972;
const COPPER: i64 = 0x96D;
const GOLDEN: i64 = 0x8A5;
const AGAPITE: i64 = 0x979;
const VERITE: i64 = 0x89F;
const VALORITE: i64 = 0x8AB;
const ICE_GREEN: i64 = 0x47F;
const ICE_BLUE: i64 = 0x482;
const DARK_GRAY: i64 = 0x497;
const FIRE: i64 = 0x489;
const ICE_WHITE: i64 = 0x47E;
const JET_BLACK: i64 = 0x001;
const PINK: i64 = 0x490;
const CRIMSON: i64 = 0x485;

const CLOAK: &str = "Server.Items.RewardCloak";
const ROBE: &str = "Server.Items.RewardRobe";
const DRESS: &str = "Server.Items.RewardDress";
const STATUETTE: &str = "Server.Items.MonsterStatuette";

use Expansion::{AOS, ML, SE};

/// `m_Lists`: one list per reward year; `Entries[i]` unlocks at `RewardInterval * (i + 1)`.
pub static LISTS: [&[Entry]; 12] = [
    &[
        e(
            SPECIAL_DYE_TUBS,
            1006008,
            "Server.Items.RewardBlackDyeTub",
            &[],
        ),
        e(
            SPECIAL_DYE_TUBS,
            1006013,
            "Server.Items.FurnitureDyeTub",
            &[],
        ),
        e(SPECIAL_DYE_TUBS, 1006047, "Server.Items.SpecialDyeTub", &[]),
        e(CLOAKS_AND_ROBES, 1006009, CLOAK, &[BRONZE, 1041286]),
        e(CLOAKS_AND_ROBES, 1006010, ROBE, &[BRONZE, 1041287]),
        ex(CLOAKS_AND_ROBES, 1080366, DRESS, ML, &[BRONZE, 1080366]),
        e(CLOAKS_AND_ROBES, 1006011, CLOAK, &[COPPER, 1041288]),
        e(CLOAKS_AND_ROBES, 1006012, ROBE, &[COPPER, 1041289]),
        ex(CLOAKS_AND_ROBES, 1080367, DRESS, ML, &[COPPER, 1080367]),
        e(MONSTER_STATUES, 1006024, STATUETTE, &[0]),
        e(MONSTER_STATUES, 1006025, STATUETTE, &[1]),
        e(MONSTER_STATUES, 1006026, STATUETTE, &[2]),
        e(MONSTER_STATUES, 1006027, STATUETTE, &[3]),
        e(MONSTER_STATUES, 1006028, STATUETTE, &[4]),
        e(MONSTER_STATUES, 1006029, STATUETTE, &[5]),
        e(MONSTER_STATUES, 1006030, STATUETTE, &[6]),
        e(MONSTER_STATUES, 1006031, STATUETTE, &[7]),
        e(MONSTER_STATUES, 1006032, STATUETTE, &[8]),
        e(MONSTER_STATUES, 1006033, STATUETTE, &[9]),
        e(MONSTER_STATUES, 1006034, STATUETTE, &[10]),
        e(MONSTER_STATUES, 1006035, STATUETTE, &[11]),
        e(MONSTER_STATUES, 1006036, STATUETTE, &[12]),
        e(MONSTER_STATUES, 1006037, STATUETTE, &[13]),
        ex(
            HOUSE_ADD_ONS,
            1062692,
            "Server.Items.ContestMiniHouseDeed",
            AOS,
            &[20],
        ),
        ex(
            HOUSE_ADD_ONS,
            1072216,
            "Server.Items.ContestMiniHouseDeed",
            SE,
            &[21],
        ),
        ex(MISCELLANEOUS, 1076155, "Server.Items.RedSoulstone", ML, &[]),
        ex(
            MISCELLANEOUS,
            1080523,
            "Server.Items.CommodityDeedBox",
            ML,
            &[],
        ),
    ],
    &[
        e(SPECIAL_DYE_TUBS, 1006052, "Server.Items.LeatherDyeTub", &[]),
        e(CLOAKS_AND_ROBES, 1006014, CLOAK, &[AGAPITE, 1041290]),
        e(CLOAKS_AND_ROBES, 1006015, ROBE, &[AGAPITE, 1041291]),
        ex(CLOAKS_AND_ROBES, 1080369, DRESS, ML, &[AGAPITE, 1080369]),
        e(CLOAKS_AND_ROBES, 1006016, CLOAK, &[GOLDEN, 1041292]),
        e(CLOAKS_AND_ROBES, 1006017, ROBE, &[GOLDEN, 1041293]),
        ex(CLOAKS_AND_ROBES, 1080368, DRESS, ML, &[GOLDEN, 1080368]),
        e(HOUSE_ADD_ONS, 1006048, "Server.Items.BannerDeed", &[]),
        e(HOUSE_ADD_ONS, 1006049, "Server.Items.FlamingHeadDeed", &[]),
        ex(
            HOUSE_ADD_ONS,
            1080409,
            "Server.Items.MinotaurStatueDeed",
            ML,
            &[],
        ),
    ],
    &[
        e(CLOAKS_AND_ROBES, 1006020, CLOAK, &[VERITE, 1041294]),
        e(CLOAKS_AND_ROBES, 1006021, ROBE, &[VERITE, 1041295]),
        ex(CLOAKS_AND_ROBES, 1080370, DRESS, ML, &[VERITE, 1080370]),
        e(CLOAKS_AND_ROBES, 1006022, CLOAK, &[VALORITE, 1041296]),
        e(CLOAKS_AND_ROBES, 1006023, ROBE, &[VALORITE, 1041297]),
        ex(CLOAKS_AND_ROBES, 1080371, DRESS, ML, &[VALORITE, 1080371]),
        e(MONSTER_STATUES, 1006038, STATUETTE, &[14]),
        e(MONSTER_STATUES, 1006039, STATUETTE, &[15]),
        e(MONSTER_STATUES, 1006040, STATUETTE, &[16]),
        e(
            ETHEREAL_STEEDS,
            1006019,
            "Server.Mobiles.EtherealHorse",
            &[],
        ),
        e(
            ETHEREAL_STEEDS,
            1006050,
            "Server.Mobiles.EtherealOstard",
            &[],
        ),
        e(
            ETHEREAL_STEEDS,
            1006051,
            "Server.Mobiles.EtherealLlama",
            &[],
        ),
        ex(
            HOUSE_ADD_ONS,
            1080407,
            "Server.Items.PottedCactusDeed",
            ML,
            &[],
        ),
    ],
    &[
        e(
            SPECIAL_DYE_TUBS,
            1049740,
            "Server.Items.RunebookDyeTub",
            &[],
        ),
        e(CLOAKS_AND_ROBES, 1049725, CLOAK, &[DARK_GRAY, 1049757]),
        e(CLOAKS_AND_ROBES, 1049726, ROBE, &[DARK_GRAY, 1049756]),
        ex(CLOAKS_AND_ROBES, 1080374, DRESS, ML, &[DARK_GRAY, 1080374]),
        e(CLOAKS_AND_ROBES, 1049727, CLOAK, &[ICE_GREEN, 1049759]),
        e(CLOAKS_AND_ROBES, 1049728, ROBE, &[ICE_GREEN, 1049758]),
        ex(CLOAKS_AND_ROBES, 1080372, DRESS, ML, &[ICE_GREEN, 1080372]),
        e(CLOAKS_AND_ROBES, 1049729, CLOAK, &[ICE_BLUE, 1049761]),
        e(CLOAKS_AND_ROBES, 1049730, ROBE, &[ICE_BLUE, 1049760]),
        ex(CLOAKS_AND_ROBES, 1080373, DRESS, ML, &[ICE_BLUE, 1080373]),
        e(MONSTER_STATUES, 1049742, STATUETTE, &[17]),
        e(MONSTER_STATUES, 1049743, STATUETTE, &[18]),
        e(MONSTER_STATUES, 1049744, STATUETTE, &[19]),
        e(
            ETHEREAL_STEEDS,
            1049746,
            "Server.Mobiles.EtherealKirin",
            &[],
        ),
        e(
            ETHEREAL_STEEDS,
            1049745,
            "Server.Mobiles.EtherealUnicorn",
            &[],
        ),
        e(
            ETHEREAL_STEEDS,
            1049747,
            "Server.Mobiles.EtherealRidgeback",
            &[],
        ),
        e(
            HOUSE_ADD_ONS,
            1049737,
            "Server.Items.DecorativeShieldDeed",
            &[],
        ),
        e(
            HOUSE_ADD_ONS,
            1049738,
            "Server.Items.HangingSkeletonDeed",
            &[],
        ),
    ],
    &[
        e(
            SPECIAL_DYE_TUBS,
            1049741,
            "Server.Items.StatuetteDyeTub",
            &[],
        ),
        e(CLOAKS_AND_ROBES, 1049731, CLOAK, &[JET_BLACK, 1049763]),
        e(CLOAKS_AND_ROBES, 1049732, ROBE, &[JET_BLACK, 1049762]),
        ex(CLOAKS_AND_ROBES, 1080377, DRESS, ML, &[JET_BLACK, 1080377]),
        e(CLOAKS_AND_ROBES, 1049733, CLOAK, &[ICE_WHITE, 1049765]),
        e(CLOAKS_AND_ROBES, 1049734, ROBE, &[ICE_WHITE, 1049764]),
        ex(CLOAKS_AND_ROBES, 1080376, DRESS, ML, &[ICE_WHITE, 1080376]),
        e(CLOAKS_AND_ROBES, 1049735, CLOAK, &[FIRE, 1049767]),
        e(CLOAKS_AND_ROBES, 1049736, ROBE, &[FIRE, 1049766]),
        ex(CLOAKS_AND_ROBES, 1080375, DRESS, ML, &[FIRE, 1080375]),
        e(MONSTER_STATUES, 1049768, STATUETTE, &[20]),
        e(MONSTER_STATUES, 1049769, STATUETTE, &[21]),
        e(MONSTER_STATUES, 1049770, STATUETTE, &[22]),
        e(
            ETHEREAL_STEEDS,
            1049749,
            "Server.Mobiles.EtherealSwampDragon",
            &[],
        ),
        e(
            ETHEREAL_STEEDS,
            1049748,
            "Server.Mobiles.EtherealBeetle",
            &[],
        ),
        e(HOUSE_ADD_ONS, 1049739, "Server.Items.StoneAnkhDeed", &[]),
        ex(
            HOUSE_ADD_ONS,
            1080384,
            "Server.Items.BloodyPentagramDeed",
            ML,
            &[],
        ),
    ],
    &[
        // StatueType: Marble 0, Jade 1, Bronze 2
        ex(
            HOUSE_ADD_ONS,
            1076188,
            "Server.Items.CharacterStatueMaker",
            ML,
            &[1],
        ),
        ex(
            HOUSE_ADD_ONS,
            1076189,
            "Server.Items.CharacterStatueMaker",
            ML,
            &[0],
        ),
        ex(
            HOUSE_ADD_ONS,
            1076190,
            "Server.Items.CharacterStatueMaker",
            ML,
            &[2],
        ),
        ex(
            HOUSE_ADD_ONS,
            1080527,
            "Server.Items.RewardBrazierDeed",
            ML,
            &[],
        ),
    ],
    &[
        ex(HOUSE_ADD_ONS, 1076157, "Server.Items.CannonDeed", ML, &[]),
        ex(
            HOUSE_ADD_ONS,
            1080550,
            "Server.Items.TreeStumpDeed",
            ML,
            &[],
        ),
    ],
    &[ex(
        MISCELLANEOUS,
        1076158,
        "Server.Items.WeaponEngravingTool",
        ML,
        &[],
    )],
    &[
        ex(
            ETHEREAL_STEEDS,
            1076159,
            "Server.Mobiles.RideablePolarBear",
            ML,
            &[],
        ),
        ex(
            HOUSE_ADD_ONS,
            1080549,
            "Server.Items.WallBannerDeed",
            ML,
            &[],
        ),
    ],
    &[
        ex(MONSTER_STATUES, 1080520, STATUETTE, ML, &[25]),
        ex(MONSTER_STATUES, 1080521, STATUETTE, ML, &[26]),
        ex(CLOAKS_AND_ROBES, 1080382, CLOAK, ML, &[PINK, 1080382]),
        ex(CLOAKS_AND_ROBES, 1080380, ROBE, ML, &[PINK, 1080380]),
        ex(CLOAKS_AND_ROBES, 1080378, DRESS, ML, &[PINK, 1080378]),
        ex(CLOAKS_AND_ROBES, 1080383, CLOAK, ML, &[CRIMSON, 1080383]),
        ex(CLOAKS_AND_ROBES, 1080381, ROBE, ML, &[CRIMSON, 1080381]),
        ex(CLOAKS_AND_ROBES, 1080379, DRESS, ML, &[CRIMSON, 1080379]),
        ex(
            ETHEREAL_STEEDS,
            1080386,
            "Server.Mobiles.EtherealCuSidhe",
            ML,
            &[],
        ),
        ex(
            HOUSE_ADD_ONS,
            1080548,
            "Server.Items.MiningCartDeed",
            ML,
            &[],
        ),
        ex(
            HOUSE_ADD_ONS,
            1080397,
            "Server.Items.AnkhOfSacrificeDeed",
            ML,
            &[],
        ),
    ],
    &[ex(
        ETHEREAL_STEEDS,
        1113908,
        "Server.Mobiles.EtherealReptalon",
        ML,
        &[],
    )],
    &[ex(
        ETHEREAL_STEEDS,
        1113813,
        "Server.Mobiles.EtherealHiryu",
        ML,
        &[],
    )],
];

/// `RewardSystem`'s configuration.
pub struct RewardState {
    pub enabled: bool,
    pub skill_cap_rewards: bool,
    pub reward_interval: TimeSpan,
}

impl Default for RewardState {
    fn default() -> Self {
        RewardState {
            enabled: true,
            skill_cap_rewards: true,
            reward_interval: TimeSpan::from_days(30),
        }
    }
}

/// Calls the reward system makes into classes another crate owns.
#[derive(Default)]
pub struct RewardHooks {
    /// `IRewardItem.IsRewardItem = true` for a constructed reward that is not a clothing reward (the deeds and
    /// statuettes keep their flag in their own state). Unset: the clothing flag is set.
    pub set_is_reward_item: Option<fn(&mut World, ItemId)>,
    /// `soulstone.Account = username` for a `RedSoulstone`.
    pub set_soulstone_account: Option<fn(&mut World, ItemId, String)>,
}

fn state(w: &World) -> RewardState {
    w.resources
        .get::<RewardStateCell>()
        .map_or_else(RewardState::default, |c| RewardState {
            enabled: c.0.enabled,
            skill_cap_rewards: c.0.skill_cap_rewards,
            reward_interval: c.0.reward_interval,
        })
}

struct RewardStateCell(RewardState);

pub fn enabled(w: &World) -> bool {
    state(w).enabled
}

/// `RewardSystem.RewardInterval.TotalDays`.
pub fn state_interval_days(w: &World) -> f64 {
    interval_days(w)
}

fn interval_days(w: &World) -> f64 {
    state(w).reward_interval.total_days()
}

/// `list.Age`: `RewardInterval * (index + 1)` (the list index counts from 1).
fn list_age(w: &World, index: usize) -> TimeSpan {
    TimeSpan::from_days_f64(interval_days(w) * (index + 1) as f64)
}

fn account_age(w: &World, m: MobileId) -> Option<TimeSpan> {
    let acct = w.mobile(m)?.account?;
    Some(accounting::account_age(w, acct))
}

/// `HasAccess(mob, list, out ts)`.
pub fn has_access_list(w: &World, m: MobileId, list: usize) -> (bool, TimeSpan) {
    let Some(age) = account_age(w, m) else {
        return (false, TimeSpan::ZERO);
    };
    let ts = list_age(w, list) - age;
    (ts <= TimeSpan::ZERO, ts)
}

/// `HasAccess(mob, entry)`.
pub fn has_access_entry(w: &World, m: MobileId, list: usize, entry: &Entry) -> bool {
    w.core
        .expansion
        .is_some_and(|x| x >= entry.required_expansion)
        && has_access_list(w, m, list).0
}

/// `HasAccess(mob, category)`.
pub fn has_access_category(w: &World, m: MobileId, category: usize) -> bool {
    LISTS.iter().enumerate().any(|(i, l)| {
        l.iter()
            .any(|e| e.category == category && has_access_entry(w, m, i, e))
    })
}

/// `GetRewardLevel(mob)`.
pub fn get_reward_level(w: &World, m: MobileId) -> i32 {
    match account_age(w, m) {
        Some(age) => ((age.total_days() / interval_days(w)) as i32).max(0),
        None => 0,
    }
}

/// `HasHalfLevel(mob)`.
pub fn has_half_level(w: &World, m: MobileId) -> bool {
    account_age(w, m).is_some_and(|age| age.total_days() / interval_days(w) >= 0.5)
}

/// `ComputeRewardInfo(mob, out cur, out max, out level)`.
pub fn compute_reward_info(w: &World, m: MobileId) -> (i32, i32, i32) {
    let Some(acct) = w.mobile(m).and_then(|x| x.account) else {
        return (0, 0, 0);
    };
    let level = get_reward_level(w, m);
    if level == 0 {
        return (0, 0, 0);
    }
    let cur = accounting::account(w, acct)
        .and_then(|a| a.get_tag("numRewardsChosen"))
        .filter(|t| !t.is_empty())
        .map_or(0, |t| t.trim().parse::<i32>().unwrap_or(0));
    let max = if level >= 6 {
        9 + (level - 6) * 2
    } else {
        2 + level
    };
    (cur, max, level)
}

/// `ConsumeRewardPoint(mob)`.
pub fn consume_reward_point(w: &mut World, m: MobileId) -> bool {
    let (cur, max, _) = compute_reward_info(w, m);
    if cur >= max {
        return false;
    }
    let Some(acct) = w.mobile(m).and_then(|x| x.account) else {
        return false;
    };
    if let Some(a) = accounting::account_mut(w, acct) {
        a.set_tag("numRewardsChosen", &(cur + 1).to_string());
    }
    true
}

/// The entry of `item_type` whose arguments are `args` (`None` matches an entry without arguments), as
/// `CheckIsUsableBy` and `GetRewardYear` walk the lists: `(list index, entry)`.
fn find_entry(item_type: &str, args: Option<&[i64]>) -> Option<(usize, &'static Entry)> {
    for (i, list) in LISTS.iter().enumerate() {
        for entry in list.iter() {
            if entry.item_type != item_type {
                continue;
            }
            match args {
                None if entry.args.is_empty() => return Some((i, entry)),
                Some(a) if a.len() == entry.args.len() && a == entry.args => {
                    return Some((i, entry));
                }
                _ => {}
            }
        }
    }
    None
}

/// `CheckIsUsableBy(from, item, args)` for an item of class `item_type` (`isRelaxedRules`: it is a `DyeTub` or a
/// `MonsterStatuette`).
pub fn check_is_usable_by(
    w: &mut World,
    from: MobileId,
    item_type: &str,
    relaxed_rules: bool,
    args: Option<&[i64]>,
) -> bool {
    let Some((i, _)) = find_entry(item_type, args) else {
        return true;
    }; // no entry?
    let (access, ts) = has_access_list(w, from, i);
    if (relaxed_rules && i == 0) || access {
        return true;
    }
    // Your account is not old enough to use this item. Months until you can use this item :
    let months = (ts.total_days() / 30.0).ceil();
    send_localized_message_affix(w, from, 1008126, true, &format!("{months}"), "", 0x3B2);
    false
}

/// `GetRewardYear(item, args)`.
pub fn get_reward_year(item_type: &str, args: Option<&[i64]>) -> i32 {
    find_entry(item_type, args).map_or(0, |(i, _)| i as i32 + 1)
}

/// `GetRewardYearLabel(item, args)`.
pub fn get_reward_year_label(item_type: &str, args: Option<&[i64]>) -> i32 {
    let level = get_reward_year(item_type, args);
    1076216
        + if level < 10 {
            level
        } else if level < 12 {
            level - 9 + 4240
        } else {
            level - 11 + 37585
        }
}

/// The `(hue, number)` arguments `RewardCloak`/`RewardRobe`/`RewardDress` pass.
fn reward_args(w: &World, item: ItemId) -> Option<(&'static str, Vec<i64>)> {
    let it = w.item(item)?;
    if it.ty.is(&muo_items::special::vet2::CONTEST_MINI_HOUSE_DEED) {
        // new object[] { Type }
        return Some((
            it.ty.full_name,
            vec![i64::from(muo_items::special::vet2::mini_type(w, item))],
        ));
    }
    if muo_content::items::is_named(it.ty, "Server.Mobiles.EtherealMount") {
        // RewardSystem.GetRewardYearLabel(this, Array.Empty<object>())
        return Some((it.ty.full_name, vec![]));
    }
    let number = muo_content::items::clothing::data(w, item).map_or(0, |d| d.number);
    Some((it.ty.full_name, vec![i64::from(it.hue), i64::from(number)]))
}

fn is_relaxed(w: &World, item: ItemId) -> bool {
    muo_content::factory::item_is(w, item, "Server.Items.DyeTub")
        || muo_content::factory::item_is(w, item, "Server.Items.MonsterStatuette")
}

/// `OnLogin(pm)`.
pub fn on_login(w: &mut World, pm: MobileId) {
    if !enabled(w) || !muo_content::mx::alive(w, pm) {
        return;
    }
    let (cur, max, mut level) = compute_reward_info(w, pm);
    let cap = muo_server::mobile::part1::props_a::skills_cap(w, pm);
    if matches!(cap, 7000 | 7050 | 7100 | 7150 | 7200) {
        level = level.clamp(0, 4);
        let new_cap = if state(w).skill_cap_rewards {
            7000 + level * 50
        } else {
            7000
        };
        muo_server::mobile::part1::props_a::set_skills_cap(w, pm, new_cap);
    }
    let has_stat_reward =
        pmob::data(w, pm).is_some_and(|d| d.flag(pmob::PlayerFlag::HAS_STAT_REWARD));
    if w.core_ml() && !has_stat_reward && has_half_level(w, pm) {
        if let Some(d) = pmob::data_mut(w, pm) {
            d.set_flag(pmob::PlayerFlag::HAS_STAT_REWARD, true);
        }
        let cap = w.mobile(pm).map_or(0, |m| m.stat_cap);
        muo_server::mobile::part1::props_b::set_stat_cap(w, pm, cap + 5);
    }
    if cur < max {
        display_notice(w, pm);
    }
}

// ---- RewardNoticeGump ----

/// `RewardNoticeGump`.
pub struct RewardNoticeGump;

/// `RewardNoticeGump.DisplayTo(from)`.
pub fn display_notice(w: &mut World, from: MobileId) {
    if w.mobile(from).and_then(|m| m.net_state).is_none() {
        return;
    }
    let g = StaticGumpObj::new(0, 0, RewardNoticeGump);
    muo_content::gumps::send_gump_to_mobile(w, from, Box::new(g), true);
}

impl StaticGumpLogic for RewardNoticeGump {
    const TYPE_NAME: &'static str = "Server.Engines.VeteranRewards.RewardNoticeGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut StaticGumpBuilder) {
        b.add_page(0);
        b.add_background(10, 10, 500, 135, 2600);
        /* You have reward items available.
         * Click 'ok' below to get the selection menu or 'cancel' to be prompted upon your next login. */
        b.add_html_localized(52, 35, 420, 55, 1006046, true, true);
        b.add_button(60, 95, 4005, 4007, 1);
        b.add_html_localized(95, 96, 150, 35, 1006044, false, false); // Ok
        b.add_button(285, 95, 4017, 4019, 0);
        b.add_html_localized(320, 96, 150, 35, 1006045, false, false); // Cancel
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        if info.button_id == 1
            && let Some(m) = w.net_states.get(&sender).and_then(|n| n.mobile)
        {
            gumps::display_choice(w, m);
        }
    }
}

// ---- setup ----

/// `Configure()`: the settings, and the `RewardHooks` the reward clothing asks.
pub fn configure(w: &mut World) {
    // `GetOrUpdateSetting`: written back on first boot
    let enabled = w
        .config
        .get_or_update_setting("vetRewards.enable", true)
        .unwrap_or(true);
    let skill_cap_rewards = w
        .config
        .get_or_update_setting("vetRewards.skillCapRewards", true)
        .unwrap_or(true);
    let reward_interval = w
        .config
        .get_or_update_setting("vetRewards.rewardInterval", TimeSpan::from_days(30))
        .unwrap_or(TimeSpan::from_days(30));
    w.resources.insert(RewardStateCell(RewardState {
        enabled,
        skill_cap_rewards,
        reward_interval,
    }));

    w.resources
        .get_mut::<muo_content::player_mobile::hooks::SystemHooks>()
        .statue_requested_animation = Some(|w, m, viewer| {
        if statue::statue(w, m).is_some() {
            statue::on_requested_animation(w, m, viewer);
        }
    });

    let h = w
        .resources
        .get_mut::<muo_items::clothing::special::RewardHooks>();
    h.reward_level = Some(get_reward_level);
    h.year_label = Some(|w, item| {
        let (t, a) = reward_args(w, item)?;
        Some(get_reward_year_label(t, Some(&a)))
    });
    h.check_is_usable_by = Some(|w, from, item| {
        let Some((t, a)) = reward_args(w, item) else {
            return true;
        };
        let relaxed = is_relaxed(w, item);
        check_is_usable_by(w, from, t, relaxed, Some(&a))
    });

    // `IRewardItem.IsRewardItem = true` of a constructed reward: the statue makers keep it in their own state, the
    // clothing rewards in the clothing data (the default when this hook is unset).
    w.resources.get_mut::<RewardHooks>().set_is_reward_item = Some(|w, item| {
        if w.item(item)
            .is_some_and(|i| i.ty.is(&statue::CHARACTER_STATUE_MAKER))
        {
            statue::set_maker_reward(w, item, true);
        } else if w
            .item(item)
            .is_some_and(|i| muo_content::items::is_named(i.ty, "Server.Mobiles.EtherealMount"))
        {
            muo_mobiles::classes::ethereal::set_is_reward_item(w, item, true);
        } else if let Some(d) = muo_content::items::clothing::data_mut(w, item) {
            d.is_reward_item = true;
        }
    });

    w.resources.get_mut::<RewardHooks>().set_soulstone_account = Some(|w, item, username| {
        muo_items::special::soulstone::set_account(w, item, Some(username))
    });

    muo_content::network::events::subscribe_ranked::<muo_content::network::events::PlayerLogin>(
        w,
        10,
        |w, e| on_login(w, e.0),
    );
}

pub fn register(types: &mut TypeRegistry) {
    statue::register(types);
    types.register_phase(
        Phase::Configure,
        "RewardSystem.Configure",
        DEFAULT_CALL_PRIORITY,
        configure,
    );
}
