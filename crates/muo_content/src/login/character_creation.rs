//! Character creation and the `CharacterCreated` event.
//!
//! The items a new character gets are made by class name through the type registry (`factory::make_item`); a class
//! that no content crate registered yet is skipped (`EquipItem(m, null)`: nothing happens). Where the
//! constructor sets state beyond hue and amount (`new Spellbook(0x382A8C38ul)`, `new BagOfReagents(30)`,
//! `new RedBook(..)`, `new Pitcher(BeverageType.Water)`) the constructor's overload table supplies it.

use muo_data::expansion::MapSelectionFlags;
use muo_net::client_version::ClientFlags;
use muo_primitives::dotnet::InvariantCase;
use muo_primitives::{TimeSpan, utility};
use muo_server::World;
use muo_server::city_info::CityInfo;
use muo_server::iaccount::AccountId;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::LootType;
use muo_server::item::ops as iops;
use muo_server::layer::Layer;
use muo_server::map::MapId;
use muo_server::mobile::AccessLevel;
use muo_server::mobile::part1::{props_a, props_b};
use muo_server::mobile::part2::{items as mitems, stats as mstats};
use muo_server::net_state::NetStateId;
use muo_server::race::{
    ALLOW_ELVES_ONLY, ALLOW_GARGOYLES_ONLY, ALLOW_HUMAN_OR_ELVES, RaceId, Races,
};
use muo_server::skills::{self, SkillName};
use tracing::{error, info};

use super::profession_info::{ProfessionInfo, get_profession};
use crate::accounting;
use crate::factory::make_item;
use crate::misc::name_verification::validate_player_name;
use crate::network::events;

const FELUCCA: MapId = MapId(0);
const TRAMMEL: MapId = MapId(1);
const MALAS: MapId = MapId(3);
const TER_MUR: MapId = MapId(5);

/// `CharacterCreatedEventArgs`.
pub struct CharacterCreatedEventArgs {
    pub state: NetStateId,
    pub account: AccountId,
    pub mobile: Option<MobileId>,
    pub name: String,
    pub female: bool,
    pub hue: i32,
    pub stats: [u8; 3],
    pub city: CityInfo,
    /// `(SkillName, byte)[]`; the raw skill number is kept so an invalid one fails the allowed-skill test.
    pub skills: Vec<(i32, u8)>,
    pub shirt_hue: i32,
    pub pants_hue: i32,
    pub hair_id: i32,
    pub hair_hue: i32,
    pub beard_id: i32,
    pub beard_hue: i32,
    pub profession: i32,
    pub race: Option<RaceId>,
}

/// `CharacterCreation.CharacterCreatedEvent(e)`.
pub fn character_created_event(w: &mut World, e: &mut CharacterCreatedEventArgs) {
    events::invoke(w, e);
}

pub fn initialize(w: &mut World) {
    events::subscribe::<CharacterCreatedEventArgs>(w, on_character_created);
}

// Allowed skills that are not race or era specific
const ALLOWED_STARTING_SKILLS: &[SkillName] = &[
    SkillName::Alchemy,
    SkillName::Anatomy,
    SkillName::AnimalLore,
    SkillName::AnimalTaming,
    SkillName::Archery,
    SkillName::ArmsLore,
    SkillName::Begging,
    SkillName::Blacksmith,
    SkillName::Fletching,
    SkillName::Bushido,
    SkillName::Camping,
    SkillName::Carpentry,
    SkillName::Cartography,
    SkillName::Chivalry,
    SkillName::Cooking,
    SkillName::DetectHidden,
    SkillName::Discordance,
    SkillName::EvalInt,
    SkillName::Fencing,
    SkillName::Fishing,
    SkillName::Focus,
    SkillName::Forensics,
    SkillName::Healing,
    SkillName::Herding,
    SkillName::Hiding,
    SkillName::Imbuing,
    SkillName::Inscribe,
    SkillName::ItemID,
    SkillName::Lockpicking,
    SkillName::Lumberjacking,
    SkillName::Macing,
    SkillName::Magery,
    SkillName::Meditation,
    SkillName::Mining,
    SkillName::Musicianship,
    SkillName::Mysticism,
    SkillName::Necromancy,
    SkillName::Ninjitsu,
    SkillName::Parry,
    SkillName::Peacemaking,
    SkillName::Poisoning,
    SkillName::Provocation,
    SkillName::MagicResist,
    SkillName::Snooping,
    SkillName::SpiritSpeak,
    SkillName::Stealing,
    SkillName::Swords,
    SkillName::Tactics,
    SkillName::Tailoring,
    SkillName::TasteID,
    SkillName::Throwing,
    SkillName::Tinkering,
    SkillName::Tracking,
    SkillName::Veterinary,
    SkillName::Wrestling,
];
fn skill_allowed(s: SkillName) -> bool {
    ALLOWED_STARTING_SKILLS.contains(&s)
}

const BAD_START_MESSAGE_DELAY: TimeSpan = TimeSpan::from_millis(3500);

fn city(name: &str, building: &str, desc: i32, x: i32, y: i32, z: i32, map: MapId) -> CityInfo {
    CityInfo::new(name, building, desc, x, y, z, map)
}

pub fn old_haven_starting_cities() -> Vec<CityInfo> {
    vec![
        CityInfo::on_map("Haven", "The Bountiful Harvest Inn", 3677, 2625, 0, TRAMMEL),
        city(
            "Britain",
            "Sweet Dreams Inn",
            1075074,
            1496,
            1628,
            10,
            TRAMMEL,
        ),
        city(
            "Magincia",
            "The Great Horns Tavern",
            1075077,
            3734,
            2222,
            20,
            TRAMMEL,
        ),
    ]
}

pub fn felucca_starting_cities() -> Vec<CityInfo> {
    vec![
        city("Yew", "The Empath Abbey", 1075072, 633, 858, 0, FELUCCA),
        city("Minoc", "The Barnacle", 1075073, 2476, 413, 15, FELUCCA),
        city(
            "Britain",
            "Sweet Dreams Inn",
            1075074,
            1496,
            1628,
            10,
            FELUCCA,
        ),
        city(
            "Moonglow",
            "The Scholars Inn",
            1075075,
            4408,
            1168,
            0,
            FELUCCA,
        ),
        city(
            "Trinsic",
            "The Traveler's Inn",
            1075076,
            1845,
            2745,
            0,
            FELUCCA,
        ),
        city(
            "Magincia",
            "The Great Horns Tavern",
            1075077,
            3734,
            2222,
            20,
            FELUCCA,
        ),
        city(
            "Jhelom",
            "The Mercenary Inn",
            1075078,
            1374,
            3826,
            0,
            FELUCCA,
        ),
        city(
            "Skara Brae",
            "The Falconer's Inn",
            1075079,
            618,
            2234,
            0,
            FELUCCA,
        ),
        city("Vesper", "The Ironwood Inn", 1075080, 2771, 976, 0, FELUCCA),
    ]
}

pub fn trammel_starting_cities() -> Vec<CityInfo> {
    vec![
        city("Yew", "The Empath Abbey", 1075072, 633, 858, 0, TRAMMEL),
        city("Minoc", "The Barnacle", 1075073, 2476, 413, 15, TRAMMEL),
        city(
            "Moonglow",
            "The Scholars Inn",
            1075075,
            4408,
            1168,
            0,
            TRAMMEL,
        ),
        city(
            "Trinsic",
            "The Traveler's Inn",
            1075076,
            1845,
            2745,
            0,
            TRAMMEL,
        ),
        city(
            "Jhelom",
            "The Mercenary Inn",
            1075078,
            1374,
            3826,
            0,
            TRAMMEL,
        ),
        city(
            "Skara Brae",
            "The Falconer's Inn",
            1075079,
            618,
            2234,
            0,
            TRAMMEL,
        ),
        city("Vesper", "The Ironwood Inn", 1075080, 2771, 976, 0, TRAMMEL),
    ]
}

pub fn new_haven_starting_cities() -> Vec<CityInfo> {
    vec![
        city(
            "New Haven",
            "The Bountiful Harvest Inn",
            1150168,
            3503,
            2574,
            14,
            TRAMMEL,
        ),
        city(
            "Britain",
            "The Wayfarer's Inn",
            1075074,
            1602,
            1591,
            20,
            TRAMMEL,
        ),
        // Magincia removed because it burned down.
    ]
}

pub fn starting_cities_sa() -> Vec<CityInfo> {
    vec![city(
        "Royal City",
        "Royal City Inn",
        1150169,
        738,
        3486,
        -19,
        TER_MUR,
    )]
}

#[derive(Default)]
struct StartingCities(Option<Vec<CityInfo>>);

/// `GetStartingCities()`: built on first use and kept.
pub fn get_starting_cities(w: &mut World) -> Vec<CityInfo> {
    if let Some(c) = w.resources.get_mut::<StartingCities>().0.clone() {
        return c;
    }
    let c = construct_available_starting_cities(w);
    w.resources.get_mut::<StartingCities>().0 = Some(c.clone());
    c
}

fn construct_available_starting_cities(w: &World) -> Vec<CityInfo> {
    // `TileMatrix.Pre6000ClientSupport`
    let pre6000 = w
        .maps
        .tile_source
        .as_ref()
        .is_some_and(|t| t.options.pre6000_trammel);
    let available = w.core_expansion_info().map_or(
        MapSelectionFlags::FELUCCA | MapSelectionFlags::TRAMMEL,
        |i| i.map_selection_flags,
    );
    let trammel = available.contains(MapSelectionFlags::TRAMMEL);
    let ter_mur = available.contains(MapSelectionFlags::TER_MUR);

    if trammel {
        if pre6000 {
            return [old_haven_starting_cities(), trammel_starting_cities()].concat();
        }
        if ter_mur {
            return [
                new_haven_starting_cities(),
                trammel_starting_cities(),
                starting_cities_sa(),
            ]
            .concat();
        }
        return [new_haven_starting_cities(), trammel_starting_cities()].concat();
    }
    if available.contains(MapSelectionFlags::FELUCCA) {
        return felucca_starting_cities();
    }
    error!("No starting cities are available.");
    Vec::new()
}

// ---- making items ----

fn item(w: &mut World, class: &str) -> Option<ItemId> {
    make_item(w, &format!("Server.Items.{class}"))
}

/// `new Class(args..)` through the class's constructor overload table; the parameterless one when the class has no
/// table (or the overload is missing).
fn item_args(w: &mut World, class: &str, args: &[crate::props::Value]) -> Option<ItemId> {
    crate::factory::make_item_args(w, &format!("Server.Items.{class}"), args)
        .or_else(|| item(w, class))
}

/// `new BagOfReagents(amount) { LootType = Regular }` (or a necromancer's bag, `amount` None) with, before AOS,
/// its contents Newbied.
fn reagent_bag(
    w: &mut World,
    class: &str,
    amount: Option<i32>,
    newbie_contents: bool,
) -> Option<ItemId> {
    let args: Vec<crate::props::Value> = amount
        .map(|a| crate::props::Value::Int(i64::from(a)))
        .into_iter()
        .collect();
    let regs = item_args(w, class, &args)?;
    iops::set_loot_type(w, regs, LootType::Regular);
    if newbie_contents && !aos(w) {
        let children = w.item(regs).map(|i| i.items.clone()).unwrap_or_default();
        for c in children {
            iops::set_loot_type(w, c, LootType::Newbied);
        }
    }
    Some(regs)
}

fn item_hue(w: &mut World, class: &str, hue: i32) -> Option<ItemId> {
    let id = item(w, class)?;
    muo_server::item::base::set_hue(w, id, hue);
    Some(id)
}

fn item_amount(w: &mut World, class: &str, amount: i32) -> Option<ItemId> {
    let id = item(w, class)?;
    iops::set_amount(w, id, amount);
    Some(id)
}

fn necro_hue(w: &mut World, id: Option<ItemId>) -> Option<ItemId> {
    if let Some(i) = id {
        muo_server::item::base::set_hue(w, i, 0x2C3);
    }
    id
}

fn loot_type(w: &World, id: ItemId) -> LootType {
    w.item(id).map_or(LootType::Regular, |i| i.loot_type)
}

fn aos(w: &World) -> bool {
    crate::core::aos(w)
}

/// `EquipItem(m, item, mustEquip)`.
fn equip_item(w: &mut World, m: MobileId, item: Option<ItemId>) {
    equip_item_must(w, m, item, false);
}

fn equip_item_must(w: &mut World, m: MobileId, item: Option<ItemId>, must_equip: bool) {
    let Some(item) = item else { return };
    if !aos(w) && loot_type(w, item) == LootType::Regular {
        iops::set_loot_type(w, item, LootType::Newbied);
    }
    if muo_server::mobile::base::equip_item(w, m, item) {
        return;
    }
    let pack = props_b::backpack(w, m);
    match pack {
        Some(pack) if !must_equip => drop_item(w, pack, item),
        _ => delete_item(w, item),
    }
}

fn drop_item(w: &mut World, pack: ItemId, item: ItemId) {
    if let Some(ty) = w.item(pack).map(|i| i.ty) {
        (ty.drop_item)(w, pack, item);
    }
}

fn delete_item(w: &mut World, item: ItemId) {
    if let Some(ty) = w.item(item).map(|i| i.ty) {
        (ty.delete)(w, item);
    }
}

/// `PackItem(m, item)`.
fn pack_item(w: &mut World, m: MobileId, item: Option<ItemId>) {
    let Some(item) = item else { return };
    if !aos(w) && loot_type(w, item) == LootType::Regular {
        iops::set_loot_type(w, item, LootType::Newbied);
    }
    match props_b::backpack(w, m) {
        Some(pack) => drop_item(w, pack, item),
        None => delete_item(w, item),
    }
}

fn add_backpack(w: &mut World, m: MobileId) {
    let mut pack = props_b::backpack(w, m);
    if pack.is_none() {
        pack = item(w, "Backpack");
        if let Some(p) = pack {
            iops::set_movable(w, p, false);
            mitems::add_item(w, m, p);
        }
    }
    let name = w.mobile(m).and_then(|mm| (mm.ty.name)(w, m));
    use crate::props::Value;
    let book = item_args(
        w,
        "RedBook",
        &[
            Value::Str("a book".into()),
            name.map_or(Value::Null, Value::Str),
            Value::Int(20),
            Value::Bool(true),
        ],
    );
    pack_item(w, m, book);
    let gold = item_amount(w, "Gold", 1000); // Starting gold can be customized here
    pack_item(w, m, gold);
    let dagger = item(w, "Dagger");
    pack_item(w, m, dagger);
    let candle = item(w, "Candle");
    pack_item(w, m, candle);
}

fn create_mobile(w: &mut World, a: AccountId) -> Option<MobileId> {
    if accounting::count(w, a) >= accounting::limit(w) {
        return None;
    }
    let len = accounting::account(w, a).map_or(0, |x| x.length());
    for i in 0..len {
        if accounting::mobile_at(w, a, i).is_none() {
            let m = crate::player_mobile::new_player_mobile(w);
            accounting::set_mobile_at(w, a, i, Some(m));
            return Some(m);
        }
    }
    None
}

fn on_character_created(w: &mut World, args: &mut CharacterCreatedEventArgs) {
    let profession = get_profession(w, args.profession);
    if profession.is_none() {
        args.profession = 0;
    }
    let state = args.state;
    if w.net_state(state).is_none() {
        return;
    }

    let Some(new_char) = create_mobile(w, args.account) else {
        let desc = w
            .net_state(state)
            .map(|d| d.description.clone())
            .unwrap_or_default();
        info!("Login: {desc}: Character creation failed, account full");
        return;
    };
    args.mobile = Some(new_char);

    let level =
        accounting::account(w, args.account).map_or(AccessLevel::Player, |a| a.access_level);
    props_a::set_player(w, new_char, true);
    props_a::set_access_level(w, new_char, level);
    props_b::set_female(w, new_char, args.female);
    let race_id = props_a::race(w, new_char);
    let clip = w
        .resources
        .get::<Races>()
        .and_then(|r| r.get(race_id))
        .map_or(args.hue & 0x3FFF, |r| {
            (r.ops.clip_skin_hue)(args.hue & 0x3FFF)
        });
    muo_server::mobile::base::set_hue(w, new_char, clip | 0x8000);
    props_a::set_hunger(w, new_char, 20);

    set_name(w, new_char, &args.name);
    add_backpack(w, new_char);

    if level == AccessLevel::Player {
        let (default_race, wanted_ok) = {
            let races = w.resources.get::<Races>();
            let wanted = args.race.and_then(|id| races.and_then(|r| r.get(id)));
            let ok = wanted.is_none_or(|r| w.expansion() >= r.required_expansion);
            (races.and_then(|r| r.default_race()).map(|r| r.id()), ok)
        };
        let race = if wanted_ok {
            args.race.or(default_race)
        } else {
            default_race
        };
        props_a::set_race(w, new_char, race);

        if crate::player_mobile::data(w, new_char).is_some()
            && accounting::is_young(w, args.account)
        {
            crate::player_mobile::set_young(w, new_char, true);
            // `newChar.BankBox.DropItem(new NewPlayerTicket { Owner = newChar })`
            // `newChar.BankBox` is evaluated (creating the box: the lower serial) before `new NewPlayerTicket`.
            let bank = props_b::bank_box(w, new_char);
            if let Some(ticket) = item(w, "NewPlayerTicket") {
                let e = muo_server::ids::EntityId::Item(ticket);
                if let Some(set) = crate::props::find_prop(w, e, "Owner").and_then(|p| p.set) {
                    let _ = set(
                        w,
                        e,
                        crate::props::Value::Entity(Some(muo_server::ids::EntityId::Mobile(
                            new_char,
                        ))),
                    );
                }
                match bank {
                    Some(bank) => drop_item(w, bank, ticket),
                    None => delete_item(w, ticket),
                }
            }
        }

        let stats = profession.as_ref().map_or(args.stats, |p| p.stats);
        set_stats(w, new_char, state, stats);
        let skills = profession.as_ref().map_or_else(
            || args.skills.clone(),
            |p| p.skills.iter().map(|(s, v)| (*s as i32, *v)).collect(),
        );
        set_skills(w, new_char, skills);
        give_profession_items(
            w,
            new_char,
            profession.as_ref(),
            args.shirt_hue,
            args.pants_hue,
        );

        let race = props_a::race(w, new_char);
        let hair_ok = race_op(w, race, |r| {
            (r.ops.validate_hair)(args.female, args.hair_id)
        });
        if hair_ok {
            props_b::set_hair_item_id(w, new_char, args.hair_id);
            let hue = race_op(w, race, |r| (r.ops.clip_hair_hue)(args.hair_hue & 0x3FFF));
            props_b::set_hair_hue(w, new_char, hue);
        }
        let beard_ok = race_op(w, race, |r| {
            (r.ops.validate_facial_hair)(args.female, args.beard_id)
        });
        if beard_ok {
            props_b::set_facial_hair_item_id(w, new_char, args.beard_id);
            let hue = race_op(w, race, |r| (r.ops.clip_hair_hue)(args.beard_hue & 0x3FFF));
            props_b::set_facial_hair_hue(w, new_char, hue);
        }
        if crate::systems::special_systems::test_center::enabled(w) {
            crate::systems::special_systems::test_center::fill_bankbox(w, new_char);
        }
    } else {
        muo_server::mobile::base::set_str(w, new_char, 100);
        muo_server::mobile::base::set_int(w, new_char, 100);
        muo_server::mobile::base::set_dex(w, new_char, 100);
        for i in 0..SkillName::COUNT as i32 {
            if let Some(s) = SkillName::from_i32(i) {
                skills::set_base_fixed(w, new_char, s, 1000);
            }
        }
        props_a::set_race(w, new_char, Some(RaceId(0)));
        props_a::set_blessed(w, new_char, true);
        let level = w
            .mobile(new_char)
            .map_or(AccessLevel::Player, |m| m.access_level);
        let robe = crate::hooks::hooks(w)
            .and_then(|h| h.new_staff_robe)
            .map(|f| f(w, level));
        add_item(w, new_char, robe);
    }

    let city = get_start_location(w, args);
    muo_server::mobile::base::move_to_world(w, new_char, city.location, city.map);

    let desc = w
        .net_state(state)
        .map(|d| d.description.clone())
        .unwrap_or_default();
    let (uname, cname, serial) = (
        accounting::account(w, args.account)
            .map(|a| a.username.clone())
            .unwrap_or_default(),
        w.mobile(new_char)
            .and_then(|m| (m.ty.name)(w, new_char))
            .unwrap_or_default(),
        w.mobile(new_char).map(|m| m.serial).unwrap_or_default(),
    );
    info!(
        "Login: {desc}: New character being created (account={uname}, character={cname}, serial={serial}, started.city={}, started.location={}, started.map={})",
        city.city,
        city.location,
        map_name(w, city.map)
    );

    start_welcome_timer(w, new_char);
}

fn map_name(w: &World, map: MapId) -> String {
    w.maps
        .get(map)
        .map_or_else(|| map.0.to_string(), |m| m.name().to_owned())
}

fn race_op<T: Default>(w: &World, race: RaceId, f: impl FnOnce(&muo_server::race::Race) -> T) -> T {
    w.resources
        .get::<Races>()
        .and_then(|r| r.get(race))
        .map(f)
        .unwrap_or_default()
}

fn add_item(w: &mut World, m: MobileId, item: Option<ItemId>) {
    if let Some(i) = item {
        mitems::add_item(w, m, i);
    }
}

fn start_welcome_timer(w: &mut World, m: MobileId) {
    super::welcome_timer::start(w, m);
}

fn get_start_location(w: &mut World, args: &CharacterCreatedEventArgs) -> CityInfo {
    let available = w.core_expansion_info().map_or(
        MapSelectionFlags::FELUCCA | MapSelectionFlags::TRAMMEL,
        |i| i.map_selection_flags,
    );
    let m = args.mobile.expect("the character was just made");

    if w.mobile(m)
        .is_some_and(|mm| mm.access_level > AccessLevel::Player)
        && available.contains(MapSelectionFlags::FELUCCA)
    {
        return CityInfo::on_map("Green Acres", "Green Acres", 5445, 1153, 0, FELUCCA);
    }

    if crate::core::sa(w) {
        return args.city.clone();
    }

    let flags = w
        .net_state(args.state)
        .map_or(ClientFlags::empty(), |d| d.caps.flags);
    if let Some(profession) = get_profession(w, args.profession) {
        let cities = get_starting_cities(w);
        let first = cities.first().cloned().unwrap_or_else(|| args.city.clone());
        match profession.name.lower_inv().as_str() {
            "necromancer" => {
                if flags.contains(ClientFlags::MALAS)
                    && available.contains(MapSelectionFlags::MALAS)
                {
                    return CityInfo::on_map("Umbra", "Mardoth's Tower", 2114, 1301, -50, MALAS);
                }
                // Unfortunately you are playing on a *NON-Age-Of-Shadows* game installation and cannot be
                // transported to Malas. You will not be able to take your new player quest in Malas without an
                // AOS client. You are now being taken to the city of Haven on the Trammel facet.
                bad_start_message(w, m, 1062205);
                return first;
            }
            "paladin" => return first,
            "samurai" => {
                let accessible = flags.contains(ClientFlags::TOKUNO)
                    && flags.contains(ClientFlags::MALAS)
                    && available.contains(MapSelectionFlags::MALAS | MapSelectionFlags::TOKUNO);
                if accessible {
                    return CityInfo::on_map("Samurai DE", "Haoti's Grounds", 368, 780, -1, MALAS);
                }
                // Unfortunately you are playing on a *NON-Samurai-Empire* game installation and cannot be
                // transported to Tokuno. ... You are now being taken to the city of Haven on the Trammel facet.
                bad_start_message(w, m, 1063487);
                return first;
            }
            "ninja" => {
                let accessible = flags.contains(ClientFlags::TOKUNO)
                    && flags.contains(ClientFlags::MALAS)
                    && available.contains(MapSelectionFlags::MALAS | MapSelectionFlags::TOKUNO);
                if accessible {
                    return CityInfo::on_map("Ninja DE", "Enimo's Residence", 414, 823, -1, MALAS);
                }
                bad_start_message(w, m, 1063487);
                return first;
            }
            _ => {}
        }
    }
    args.city.clone()
}

fn bad_start_message(w: &mut World, m: MobileId, number: i32) {
    muo_server::timer::start(w, BAD_START_MESSAGE_DELAY, TimeSpan::ZERO, 1, move |w| {
        muo_server::mobile::part2::messages::send_localized_message(w, m, number, "", 0x3B2);
    });
}

fn set_stats(w: &mut World, m: MobileId, state: NetStateId, stats: [u8; 3]) {
    let new_creation = w
        .net_state(state)
        .is_some_and(|d| d.caps.new_character_creation());
    let max_stats = if new_creation { 90 } else { 80 };
    let (mut str, mut dex, mut intel) = (
        i32::from(stats[0]),
        i32::from(stats[1]),
        i32::from(stats[2]),
    );
    if !(10..=60).contains(&str)
        || !(10..=60).contains(&dex)
        || !(10..=60).contains(&intel)
        || str + dex + intel != max_stats
    {
        str = 10;
        dex = 10;
        intel = 10;
    }
    mstats::init_stats(w, m, str, dex, intel);
}

fn set_name(w: &mut World, m: MobileId, name: &str) {
    let mut name = name.trim().to_owned();
    if !validate_player_name(&name) {
        name = "Generic Player".to_owned();
    }
    props_b::set_raw_name(w, m, Some(name));
}

/// `ValidateSkills`: may blank an entry that is not allowed in this era or for this race.
fn validate_skills(w: &World, race_flag: i32, skills: &mut [(i32, u8)]) -> bool {
    let core_aos = w.core_aos();
    let core_se = w.core_se();
    let core_sa = crate::core::sa(w);
    let mut total = 0;
    for i in 0..skills.len() {
        let (raw, value) = skills[i];
        let Some(name) = SkillName::from_i32(raw).filter(|n| skill_allowed(*n)) else {
            return false;
        };
        if value > 50 {
            return false;
        }

        // Note: Change to Alchemy @ 0 skill if something invalid is chosen.
        // To avoid this, modify the client to only show the skills allowed by your shard.
        let blank = match name {
            SkillName::Necromancy | SkillName::Chivalry | SkillName::Focus => !core_aos,
            SkillName::Ninjitsu | SkillName::Bushido => !core_se,
            SkillName::Throwing | SkillName::Imbuing if !core_sa => true,
            SkillName::Archery => race_flag == ALLOW_GARGOYLES_ONLY,
            SkillName::Throwing => race_flag != ALLOW_GARGOYLES_ONLY,
            _ => false,
        };
        if blank {
            skills[i] = (0, 0);
        }

        total += i32::from(value);

        // Do not allow a skill to be listed twice
        if skills[i + 1..]
            .iter()
            .any(|&(name_check, value_check)| value_check > 0 && name_check == raw)
        {
            return false;
        }
    }
    total == 100 || total == 120
}

fn set_skills(w: &mut World, m: MobileId, mut list: Vec<(i32, u8)>) {
    let race = props_a::race(w, m);
    let flag = race_op(w, race, |r| r.race_flag);
    if !validate_skills(w, flag, &mut list) {
        return;
    }
    for (raw, value) in list {
        if value == 0 {
            continue;
        }
        let Some(name) = SkillName::from_i32(raw) else {
            continue;
        };
        skills::set_base_fixed(w, m, name, i32::from(value) * 10);
        add_skill_items(w, m, name);
    }
}

// ---- the items of a profession ----

fn race_info(w: &mut World, m: MobileId) -> (i32, bool, bool, bool, bool) {
    let race = props_a::race(w, m);
    let flag = race_op(w, race, |r| r.race_flag);
    let female = w.mobile(m).is_some_and(|mm| mm.female);
    let elf = flag == ALLOW_ELVES_ONLY;
    let gargoyle = flag == ALLOW_GARGOYLES_ONLY;
    (
        flag,
        female,
        elf,
        gargoyle,
        flag & ALLOW_HUMAN_OR_ELVES != 0,
    )
}

fn gargish(w: &mut World, female: bool, kind: &str, hue: Option<i32>) -> Option<ItemId> {
    let id = item(
        w,
        &format!("{kind}{}", if female { "Type2" } else { "Type1" }),
    )?;
    if let Some(h) = hue {
        muo_server::item::base::set_hue(w, id, h);
    }
    Some(id)
}

fn give_profession_items(
    w: &mut World,
    m: MobileId,
    profession: Option<&ProfessionInfo>,
    shirt_hue: i32,
    pants_hue: i32,
) {
    let (_, female, elf, gargoyle, _) = race_info(w, m);
    let name = profession.map(|p| p.name.lower_inv());

    macro_rules! eq {
        ($class:expr) => {{
            let i = item(w, $class);
            equip_item(w, m, i);
        }};
        ($class:expr, hue $h:expr) => {{
            let i = item_hue(w, $class, $h);
            equip_item(w, m, i);
        }};
    }
    macro_rules! eq_necro {
        ($class:expr) => {{
            let i = item(w, $class);
            let i = necro_hue(w, i);
            equip_item(w, m, i);
        }};
    }
    macro_rules! eq_gargish {
        ($kind:expr) => {{
            let i = gargish(w, female, $kind, None);
            equip_item(w, m, i);
        }};
    }
    macro_rules! eq_gargish_necro {
        ($kind:expr) => {{
            let i = gargish(w, female, $kind, Some(0x2C3));
            equip_item(w, m, i);
        }};
    }

    match name.as_deref() {
        Some("necromancer") => {
            let regs = reagent_bag(w, "BagOfNecroReagents", None, true);
            pack_item(w, m, regs);

            eq!("BoneHelm");
            if elf {
                eq!("ElvenMachete");
                eq_necro!("LeafChest");
                eq_necro!("LeafArms");
                eq_necro!("LeafGloves");
                eq_necro!("LeafGorget");
                eq_necro!("LeafLegs");
                eq!("ElvenBoots");
            } else if gargoyle {
                eq!("GlassSword");
                eq_gargish_necro!("GargishLeatherChest");
                eq_gargish_necro!("GargishLeatherArms");
                eq_gargish_necro!("GargishLeatherKilt");
                eq_gargish_necro!("GargishLeatherLegs");
            } else {
                eq!("BoneHarvester");
                eq_necro!("LeatherChest");
                eq_necro!("LeatherArms");
                eq_necro!("LeatherGloves");
                eq_necro!("LeatherGorget");
                eq_necro!("LeatherLegs");
                eq_necro!("Skirt");
                eq!("Sandals", hue 0x8FD);
            }

            // animate dead, evil omen, pain spike, summon familiar, wraith form
            let book = item_args(
                w,
                "NecromancerSpellbook",
                &[crate::props::Value::UInt(0x8981)],
            );
            pack_item(w, m, book);
            return;
        }
        Some("paladin") => {
            if elf {
                eq!("ElvenMachete");
                eq!("WingedHelm");
                eq!("LeafGorget");
                eq!("LeafArms");
                eq!("LeafChest");
                eq!("LeafLegs");
                eq!("LeafGloves");
                eq!("ElvenBoots"); // Verify hue
            } else if gargoyle {
                eq!("GlassSword");
                eq_gargish!("GargishStoneChest");
                eq_gargish!("GargishStoneArms");
                eq_gargish!("GargishStoneKilt");
                eq_gargish!("GargishStoneLegs");
            } else {
                eq!("Broadsword");
                eq!("Helmet");
                eq!("PlateGorget");
                eq!("RingmailArms");
                eq!("RingmailChest");
                eq!("RingmailLegs");
                eq!("RingmailGloves");
                eq!("ThighBoots", hue 0x748);
                eq!("Cloak", hue 0xCF);
                eq!("BodySash", hue 0xCF);
            }
            let book = item(w, "BookOfChivalry");
            pack_item(w, m, book);
            return;
        }
        Some("samurai") => {
            if elf {
                eq!("RavenHelm");
                eq!("HakamaShita", hue 0x2C3);
                eq!("Hakama", hue 0x2C3);
                eq!("SamuraiTabi", hue 0x2C3);
                eq!("TattsukeHakama", hue 0x22D);
                eq!("Bokuto");
            } else if gargoyle {
                eq!("GargishTalwar");
                eq_gargish!("GargishLeatherChest");
                eq_gargish!("GargishLeatherArms");
                eq_gargish!("GargishLeatherKilt");
                eq_gargish!("GargishLeatherLegs");
            } else {
                eq!("LeatherJingasa");
                eq!("HakamaShita", hue 0x2C3);
                eq!("Hakama", hue 0x2C3);
                eq!("SamuraiTabi", hue 0x2C3);
                eq!("TattsukeHakama", hue 0x22D);
                eq!("Bokuto");
            }
            let scissors = item(w, "Scissors");
            pack_item(w, m, scissors);
            let bandage = item_amount(w, "Bandage", 50);
            pack_item(w, m, bandage);
            let book = item(w, "BookOfBushido");
            pack_item(w, m, book);
            return;
        }
        Some("ninja") => {
            // TODO: Verify that's ALL the hues for that above.
            const HUES: [i32; 7] = [0x1A8, 0xEC, 0x99, 0x90, 0xB5, 0x336, 0x89];
            let hue = |w: &mut World| HUES[utility::random(&mut w.rng, HUES.len() as i32) as usize];
            if elf {
                eq!("AssassinSpike");
                let h = hue(w);
                eq!("TattsukeHakama", hue h);
                eq!("HakamaShita", hue 0x2C3);
                eq!("NinjaTabi", hue 0x2C3);
                eq!("Kasa");
            } else if gargoyle {
                eq!("DualPointedSpear");
                eq_gargish!("GargishLeatherChest");
                eq_gargish!("GargishLeatherArms");
                eq_gargish!("GargishLeatherKilt");
                eq_gargish!("GargishLeatherLegs");
            } else {
                eq!("Tekagi");
                let h = hue(w);
                eq!("TattsukeHakama", hue h);
                eq!("HakamaShita", hue 0x2C3);
                eq!("NinjaTabi", hue 0x2C3);
                eq!("Kasa");
            }
            for _ in 0..5 {
                let b = item(w, "SmokeBomb");
                pack_item(w, m, b);
            }
            let book = item(w, "BookOfNinjitsu");
            pack_item(w, m, book);
            return;
        }
        Some("swordsman" | "fencer" | "warrior" | "mace fighter") => {
            if elf {
                eq!("Circlet");
                eq!("HideGorget");
                eq!("HideChest");
                eq!("HidePauldrons");
                eq!("HideGloves");
                eq!("HidePants");
                eq!("ElvenBoots");
            } else if gargoyle {
                eq_gargish!("GargishLeatherChest");
                eq_gargish!("GargishLeatherArms");
                eq_gargish!("GargishLeatherKilt");
                eq_gargish!("GargishLeatherLegs");
            } else {
                eq!("Bascinet");
                eq!("StuddedGorget");
                eq!("StuddedChest");
                eq!("StuddedArms");
                eq!("StuddedGloves");
                eq!("StuddedLegs");
                eq!("ThighBoots");
            }
        }
        _ => {}
    }

    add_shirt(w, m, shirt_hue);
    add_pants(w, m, pants_hue);
    add_shoes(w, m);

    // All elves get a wild staff
    if elf {
        eq!("WildStaff");
    }
}

fn add_shirt(w: &mut World, m: MobileId, shirt_hue: i32) {
    let hue = utility::clip_dyed_hue(shirt_hue & 0x3FFF);
    let (_, female, elf, gargoyle, _) = race_info(w, m);
    let shirt = if elf {
        item_hue(w, "ElvenShirt", hue)
    } else if gargoyle {
        gargish(w, female, "GargishClothChest", Some(hue))
    } else {
        // Humans
        match utility::random(&mut w.rng, 3) {
            0 => item_hue(w, "Shirt", hue),
            1 => item_hue(w, "FancyShirt", hue),
            _ => item_hue(w, "Doublet", hue),
        }
    };
    equip_item(w, m, shirt);
}

fn add_pants(w: &mut World, m: MobileId, pants_hue: i32) {
    let hue = utility::clip_dyed_hue(pants_hue & 0x3FFF);
    let (_, female, elf, gargoyle, _) = race_info(w, m);
    let pants = if elf {
        item_hue(w, "ElvenPants", hue)
    } else if gargoyle {
        gargish(w, female, "GargishClothLegs", Some(hue))
    } else {
        // Humans
        match (utility::random_bool(&mut w.rng), female) {
            (true, true) => item_hue(w, "Skirt", hue),
            (true, false) => item_hue(w, "LongPants", hue),
            (false, true) => item_hue(w, "Kilt", hue),
            (false, false) => item_hue(w, "ShortPants", hue),
        }
    };
    equip_item(w, m, pants);
}

fn add_shoes(w: &mut World, m: MobileId) {
    let (_, _, elf, gargoyle, _) = race_info(w, m);
    if elf {
        let i = item(w, "ElvenBoots");
        equip_item(w, m, i);
    } else if !gargoyle {
        // Race.Human
        let hue = utility::random_yellow_hue(&mut w.rng);
        let i = item_hue(w, "Shoes", hue);
        equip_item(w, m, i);
    }
}

fn pack_instrument(w: &mut World, m: MobileId) {
    let class = match utility::random(&mut w.rng, 6) {
        0 => "Drums",
        1 => "Harp",
        2 => "LapHarp",
        3 => "Lute",
        4 => "Tambourine",
        _ => "TambourineTassel",
    };
    let i = item(w, class);
    pack_item(w, m, i);
}

fn pack_scroll(w: &mut World, m: MobileId, circle: i32) {
    const SCROLLS: [&str; 24] = [
        "ClumsyScroll",
        "CreateFoodScroll",
        "FeeblemindScroll",
        "HealScroll",
        "MagicArrowScroll",
        "NightSightScroll",
        "ReactiveArmorScroll",
        "WeakenScroll",
        "AgilityScroll",
        "CunningScroll",
        "CureScroll",
        "HarmScroll",
        "MagicTrapScroll",
        "MagicUnTrapScroll",
        "ProtectionScroll",
        "StrengthScroll",
        "BlessScroll",
        "FireballScroll",
        "MagicLockScroll",
        "PoisonScroll",
        "TelekinesisScroll",
        "TeleportScroll",
        "UnlockScroll",
        "WallOfStoneScroll",
    ];
    let n = (utility::random(&mut w.rng, 8) * (circle + 1)) as usize;
    let i = item(w, SCROLLS[n.min(23)]);
    pack_item(w, m, i);
}

fn pack_tinker_part(w: &mut World, m: MobileId) {
    let class = match utility::random(&mut w.rng, 4) {
        0 => "Axle",
        1 => "Gears",
        2 => "Hinge",
        _ => "Springs",
    };
    let i = item(w, class);
    pack_item(w, m, i);
}

fn robe(w: &mut World, race_flag: i32, female: bool, hue: i32) -> Option<ItemId> {
    if race_flag == ALLOW_ELVES_ONLY {
        item_hue(
            w,
            if female {
                "FemaleElvenRobe"
            } else {
                "MaleElvenRobe"
            },
            hue,
        )
    } else {
        item_hue(w, "Robe", hue)
    }
}

fn swords_weapon(w: &mut World, race_flag: i32) -> Option<ItemId> {
    item(
        w,
        match race_flag {
            ALLOW_ELVES_ONLY => "RuneBlade",
            ALLOW_GARGOYLES_ONLY => "DreadSword",
            _ => "Katana",
        },
    )
}

fn macing_weapon(w: &mut World, race_flag: i32) -> Option<ItemId> {
    item(
        w,
        match race_flag {
            ALLOW_ELVES_ONLY => "DiamondMace",
            ALLOW_GARGOYLES_ONLY => "DiscMace",
            _ => "Club",
        },
    )
}

fn fencing_weapon(w: &mut World, race_flag: i32) -> Option<ItemId> {
    item(
        w,
        match race_flag {
            ALLOW_ELVES_ONLY => "Leafblade",
            ALLOW_GARGOYLES_ONLY => "BloodBlade",
            _ => "Kryss",
        },
    )
}

fn ranged_weapon(w: &mut World, race_flag: i32) -> Option<ItemId> {
    item(
        w,
        match race_flag {
            ALLOW_ELVES_ONLY => "ElvenCompositeLongbow",
            ALLOW_GARGOYLES_ONLY => "SerpentstoneStaff",
            _ => "Bow",
        },
    )
}

fn find_layer(w: &World, m: MobileId, layer: Layer) -> Option<ItemId> {
    mitems::find_item_on_layer(w, m, layer)
}

fn add_skill_items(w: &mut World, m: MobileId, skill: SkillName) {
    let (race_flag, female, elf, gargoyle, elf_or_human) = race_info(w, m);
    let human = !elf && !gargoyle;

    macro_rules! pack {
        ($class:expr) => {{
            let i = item(w, $class);
            pack_item(w, m, i);
        }};
        ($class:expr, amount $n:expr) => {{
            let i = item_amount(w, $class, $n);
            pack_item(w, m, i);
        }};
    }
    macro_rules! eq {
        ($class:expr) => {{
            let i = item(w, $class);
            equip_item(w, m, i);
        }};
        ($class:expr, hue $h:expr) => {{
            let i = item_hue(w, $class, $h);
            equip_item(w, m, i);
        }};
    }
    macro_rules! eq_robe {
        ($hue:expr) => {{
            let h = $hue;
            let i = robe(w, race_flag, female, h);
            equip_item(w, m, i);
        }};
    }

    match skill {
        SkillName::Alchemy => {
            pack!("Bottle", amount 4);
            pack!("MortarPestle");
            eq_robe!(utility::random_pink_hue(&mut w.rng));
        }
        SkillName::Anatomy => {
            pack!("Bandage", amount 3);
            eq_robe!(utility::random_yellow_hue(&mut w.rng));
        }
        SkillName::AnimalLore => {
            if elf {
                eq!("WildStaff");
            } else {
                eq!("ShepherdsCrook");
            }
            eq_robe!(utility::random_green_hue(&mut w.rng));
        }
        SkillName::AnimalTaming => {
            if human {
                eq!("ShepherdsCrook");
            }
        }
        SkillName::Archery => {
            pack!("Arrow", amount 25);
            let i = ranged_weapon(w, race_flag);
            equip_item(w, m, i);
        }
        SkillName::ArmsLore => {
            let i = match utility::random(&mut w.rng, 3) {
                0 => fencing_weapon(w, race_flag),
                1 => macing_weapon(w, race_flag),
                _ => swords_weapon(w, race_flag),
            };
            equip_item(w, m, i);
        }
        SkillName::Begging => {
            let class = match race_flag {
                ALLOW_ELVES_ONLY => "WildStaff",
                ALLOW_GARGOYLES_ONLY => "GlassStaff",
                _ => "GnarledStaff",
            };
            eq!(class);
        }
        SkillName::Blacksmith => {
            pack!("Tongs");
            pack!("Pickaxe");
            pack!("Pickaxe");
            pack!("IronIngot", amount 50);
            let h = utility::random_yellow_hue(&mut w.rng);
            eq!("HalfApron", hue h);
        }
        SkillName::Bushido => {
            if elf_or_human {
                // Delete pants
                if let Some(p) = find_layer(w, m, Layer::OuterLegs) {
                    delete_item(w, p);
                }
                eq!("Hakama");
                eq!("Kasa");
            }
            eq!("BookOfBushido");
        }
        SkillName::Fletching => {
            pack!("Board", amount 14);
            pack!("Feather", amount 5);
            pack!("Shaft", amount 5);
        }
        SkillName::Camping => {
            pack!("Bedroll");
            pack!("Kindling", amount 5);
        }
        SkillName::Carpentry => {
            pack!("Board", amount 10);
            pack!("Saw");
            if elf_or_human {
                let h = utility::random_yellow_hue(&mut w.rng);
                eq!("HalfApron", hue h);
            }
        }
        SkillName::Cartography => {
            for _ in 0..4 {
                pack!("BlankMap");
            }
            pack!("Sextant");
        }
        SkillName::Cooking => {
            pack!("Kindling", amount 2);
            pack!("RawLambLeg");
            pack!("RawChickenLeg");
            pack!("RawFishSteak");
            pack!("SackFlour");
            let pitcher = item_args(w, "Pitcher", &[crate::props::Value::Enum(5)]); // BeverageType.Water
            pack_item(w, m, pitcher);
        }
        SkillName::Chivalry => {
            pack!("BookOfChivalry");
        }
        SkillName::DetectHidden => {
            if elf_or_human {
                eq!("Cloak", hue 0x455);
            }
        }
        SkillName::Discordance => pack_instrument(w, m),
        SkillName::Fencing => {
            let i = fencing_weapon(w, race_flag);
            equip_item(w, m, i);
        }
        SkillName::Fishing => {
            eq!("FishingPole");
            let hue = utility::random_yellow_hue(&mut w.rng);
            if elf {
                eq!("Circlet", hue hue);
            } else if human {
                eq!("FloppyHat", hue hue);
            }
        }
        SkillName::Healing => {
            pack!("Bandage", amount 50);
            pack!("Scissors");
        }
        SkillName::Herding => {
            eq!("ShepherdsCrook");
        }
        SkillName::Hiding => {
            if elf_or_human {
                eq!("Cloak", hue 0x455);
            }
        }
        SkillName::Inscribe => {
            pack!("BlankScroll", amount 2);
            pack!("BlueBook");
        }
        SkillName::ItemID => {
            let class = match race_flag {
                ALLOW_ELVES_ONLY => "WildStaff",
                ALLOW_GARGOYLES_ONLY => "SerpentstoneStaff",
                _ => "GnarledStaff",
            };
            eq!(class);
        }
        SkillName::Lockpicking => {
            pack!("Lockpick", amount 20);
        }
        SkillName::Lumberjacking => {
            if elf_or_human {
                eq!("Hatchet");
            } else {
                eq!("DualShortAxes");
            }
        }
        SkillName::Macing => {
            let i = macing_weapon(w, race_flag);
            equip_item(w, m, i);
        }
        SkillName::Magery => {
            let regs = reagent_bag(w, "BagOfReagents", Some(30), true);
            pack_item(w, m, regs);
            pack_scroll(w, m, 0);
            pack_scroll(w, m, 1);
            pack_scroll(w, m, 2);

            let book = item_args(w, "Spellbook", &[crate::props::Value::UInt(0x382A_8C38)]);
            equip_item(w, m, book);
            eq_robe!(utility::random_blue_hue(&mut w.rng));

            if elf {
                eq!("Circlet");
            } else if human {
                eq!("WizardsHat");
            }
        }
        SkillName::Mining => {
            pack!("Pickaxe");
        }
        SkillName::Musicianship => pack_instrument(w, m),
        SkillName::Necromancy => {
            if crate::core::ml(w) {
                let regs = reagent_bag(w, "BagOfNecroReagents", None, false);
                pack_item(w, m, regs);
            }
        }
        SkillName::Ninjitsu => {
            if elf_or_human {
                // Delete pants
                if let Some(p) = find_layer(w, m, Layer::OuterLegs) {
                    delete_item(w, p);
                }
                eq!("Hakama", hue 0x2C3); // Only ninjas get the hued one.
                eq!("Kasa");
            }
            eq!("BookOfNinjitsu");
        }
        SkillName::Parry => {
            eq!(if race_flag == ALLOW_GARGOYLES_ONLY {
                "GargishWoodenShield"
            } else {
                "WoodenShield"
            });
        }
        SkillName::Peacemaking => pack_instrument(w, m),
        SkillName::Poisoning => {
            pack!("LesserPoisonPotion");
            pack!("LesserPoisonPotion");
        }
        SkillName::Provocation => pack_instrument(w, m),
        SkillName::Stealing | SkillName::Snooping => {
            pack!("Lockpick", amount 20);
        }
        SkillName::SpiritSpeak => {
            eq!("Cloak", hue 0x455);
        }
        SkillName::Tactics | SkillName::Swords => {
            let i = swords_weapon(w, race_flag);
            equip_item(w, m, i);
        }
        SkillName::Tailoring => {
            pack!("BoltOfCloth");
            pack!("SewingKit");
        }
        SkillName::Tinkering => {
            if !aos(w) {
                pack_tinker_part(w, m);
                pack_tinker_part(w, m);
                pack_tinker_part(w, m);
            }
            pack!("TinkerTools");
        }
        SkillName::Tracking => {
            if elf_or_human {
                // Delete shoes
                if let Some(s) = find_layer(w, m, Layer::Shoes) {
                    delete_item(w, s);
                }
                let hue = utility::random_yellow_hue(&mut w.rng);
                let i = item_hue(w, if elf { "ElvenBoots" } else { "Boots" }, hue);
                equip_item(w, m, i);
            }
            eq!("SkinningKnife");
        }
        SkillName::Veterinary => {
            pack!("Bandage", amount 5);
            pack!("Scissors");
        }
        SkillName::Wrestling => {
            let i = match race_flag {
                ALLOW_ELVES_ONLY => item(w, "LeafGloves"),
                ALLOW_GARGOYLES_ONLY => gargish(w, female, "GargishLeatherArms", None),
                _ => item(w, "LeatherGloves"),
            };
            equip_item(w, m, i);
        }
        SkillName::Throwing => {
            if gargoyle {
                eq!("Boomerang");
            }
        }
        SkillName::Mysticism => {
            // PackItem(new MysticBook(0xAB));
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skill_validation_rules() {
        let w = World::new();
        // 3 skills at 40 + 0 (old client) is 120: valid; a listed-twice skill is not
        let mut s = vec![
            (SkillName::Alchemy as i32, 40),
            (SkillName::Anatomy as i32, 40),
            (SkillName::Magery as i32, 40),
        ];
        assert!(validate_skills(&w, 0x1, &mut s));
        let mut dup = vec![
            (SkillName::Alchemy as i32, 50),
            (SkillName::Alchemy as i32, 50),
        ];
        assert!(!validate_skills(&w, 0x1, &mut dup));
        let mut big = vec![
            (SkillName::Alchemy as i32, 51),
            (SkillName::Anatomy as i32, 49),
        ];
        assert!(!validate_skills(&w, 0x1, &mut big));
        // Mining at 100 total with Archery for gargoyles is blanked but still counts
        let mut garg = vec![
            (SkillName::Archery as i32, 50),
            (SkillName::Mining as i32, 50),
        ];
        assert!(validate_skills(&w, ALLOW_GARGOYLES_ONLY, &mut garg));
        assert_eq!(garg[0], (0, 0));
    }

    #[test]
    fn felucca_and_trammel_lists() {
        assert_eq!(felucca_starting_cities().len(), 9);
        assert_eq!(trammel_starting_cities().len(), 7);
        assert_eq!(new_haven_starting_cities()[0].city, "New Haven");
    }
}
