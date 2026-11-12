//! The fishing harvest system.
//!
//! Shipwreck chests and items brought up by an SOS (SOS, ShipwreckedItem, SpecialFishingNet, TreasureMapChest.Fill)
//! and the Collector quest's pearl are `HarvestHooks` (set in `harvest::configure`); the treasure map and
//! message-in-a-bottle catches are made by name.

use muo_io::text::TextDefinition as Td;
use muo_primitives::{Point3D, TimeSpan, utility};
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::{deps, ops};
use muo_server::map::MapId;
use muo_server::skills::{self, SkillName};

use super::*;

const WATER_LAND_TILES: &[i32] = &[0x00A8, 0x00AB, 0x0136, 0x0137];
const WATER_STATIC_TILES: &[i32] = &[
    0x1797, 0x179C, 0x346E, 0x3485, 0x3490, 0x34AB, 0x34B5, 0x35D5,
];

const MAGIC_FISH: &[&str] = &[
    "Server.Items.PrizedFish",
    "Server.Items.WondrousFish",
    "Server.Items.TrulyRareFish",
    "Server.Items.PeculiarFish",
];
const FOOTWEAR: &[&str] = &[
    "Server.Items.Boots",
    "Server.Items.Shoes",
    "Server.Items.Sandals",
    "Server.Items.ThighBoots",
];

/// `MutateEntry`.
struct MutateEntry {
    req_skill: f64,
    min_skill: f64,
    max_skill: f64,
    deep_water: bool,
    expansion: Expansion,
    max_expansion: Option<Expansion>,
    types: &'static [&'static str],
}

use muo_data::expansion::Expansion;

const fn entry(
    req: f64,
    min: f64,
    max: f64,
    deep: bool,
    expansion: Expansion,
    max_expansion: Option<Expansion>,
    types: &'static [&'static str],
) -> MutateEntry {
    MutateEntry {
        req_skill: req,
        min_skill: min,
        max_skill: max,
        deep_water: deep,
        expansion,
        max_expansion,
        types,
    }
}

static MUTATE_TABLE: &[MutateEntry] = &[
    entry(
        80.0,
        80.0,
        4080.0,
        true,
        Expansion::UOR,
        None,
        &["Server.Items.SpecialFishingNet"],
    ),
    entry(
        80.0,
        80.0,
        4080.0,
        true,
        Expansion::UOR,
        None,
        &["Server.Items.BigFish"],
    ),
    // T2A-UOTD: needs 90; 0.25% at 90, 0.5% at 100.
    // LBR onward: needs 90; 0.42% at 90, 0.83% at 100.
    entry(
        90.0,
        80.0,
        4080.0,
        true,
        Expansion::T2A,
        Some(Expansion::UOTD),
        &["Server.Items.TreasureMap"],
    ),
    entry(
        90.0,
        80.0,
        2480.0,
        true,
        Expansion::LBR,
        None,
        &["Server.Items.TreasureMap"],
    ),
    // T2A-UOTD: needs 100; 0.5% at 100.
    // LBR onward: needs 100; 1.33% at 100.
    entry(
        100.0,
        80.0,
        4080.0,
        true,
        Expansion::T2A,
        Some(Expansion::UOTD),
        &["Server.Items.MessageInABottle"],
    ),
    entry(
        100.0,
        80.0,
        1580.0,
        true,
        Expansion::LBR,
        None,
        &["Server.Items.MessageInABottle"],
    ),
    // T2A-UOTD: needs 80; 2.0% at 80, 2.5% at 100.
    // LBR-SA: no minimum; 0% at 0, 5% at 100.
    // HS onward: needs 80; 4.0% at 80, 5.0% at 100.
    entry(
        80.0,
        0.0,
        4000.0,
        false,
        Expansion::T2A,
        Some(Expansion::UOTD),
        MAGIC_FISH,
    ),
    entry(
        0.0,
        0.0,
        2000.0,
        false,
        Expansion::LBR,
        Some(Expansion::SA),
        MAGIC_FISH,
    ),
    entry(80.0, 0.0, 2000.0, false, Expansion::HS, None, MAGIC_FISH),
    entry(
        0.0,
        103.5,
        -34.5,
        false,
        Expansion::T2A,
        Some(Expansion::UOTD),
        FOOTWEAR,
    ),
    // From LBR the two footwear rows make one curve: 75% at 0 skill, 10% at 100 and 5% at 120.
    entry(0.0, 100.0, -62.5, false, Expansion::LBR, None, FOOTWEAR),
    entry(0.0, 140.0, -260.0, false, Expansion::LBR, None, FOOTWEAR),
];

/// `new Fishing()`.
pub fn create(w: &mut World) -> HarvestSystem {
    let mut fish = HarvestDefinition {
        bank_width: 8,
        bank_height: 8,
        min_total: 5,
        max_total: 15,
        min_respawn: TimeSpan::from_mins(10),
        max_respawn: TimeSpan::from_mins(20),
        skill: SkillName::Fishing,
        land_tiles: WATER_LAND_TILES.to_vec(),
        static_tiles: WATER_STATIC_TILES.to_vec(),
        ranged_tiles: true,
        max_range: 4,
        consumed_per_harvest: 1,
        consumed_per_felucca_harvest: 1,
        effect_actions: vec![12],
        effect_sounds: Vec::new(),
        effect_counts: vec![1],
        effect_delay: TimeSpan::ZERO,
        effect_sound_delay: TimeSpan::from_secs(8),
        no_resources_message: Td::of_number(503172), // The fish don't seem to be biting here.
        fail_message: Td::of_number(503171), // You fish a while, but fail to catch anything.
        timed_out_of_range_message: Td::of_number(500976), // You need to be closer to the water to fish!
        out_of_range_message: Td::of_number(500976),
        pack_full_message: Td::of_number(503176), // You do not have room in your backpack for a fish.
        tool_broke_message: Td::of_number(503174), // You broke your fishing pole.
        ..Default::default()
    };
    fish.resources = vec![HarvestResource::new(
        0.0,
        0.0,
        100.0,
        1043297,
        &["Server.Items.Fish"],
    )];
    fish.set_veins(vec![HarvestVein {
        vein_chance: 1000,
        chance_to_fallback: 0.0,
        primary: 0,
        fallback: None,
    }]);
    if muo_content::core::ml(w) {
        fish.bonus_resources = Some(vec![
            BonusResource {
                req_skill: 0.0,
                chance: 99.4,
                message: Td::EMPTY,
                ty: None,
            }, // set to same chance as mining ml gems
            BonusResource {
                req_skill: 80.0,
                chance: 0.6,
                message: Td::of_number(1072597),
                ty: Some("Server.Items.WhitePearl"),
            },
        ]);
    }
    HarvestSystem {
        kind: Kind::Fishing,
        name: "Fishing",
        definitions: vec![fish],
    }
}

/// `CheckHarvest` (both overloads) beyond the base's tool check.
pub fn check_harvest(w: &mut World, from: MobileId) -> bool {
    if muo_server::mobile::part1::props_b::mounted(w, from) {
        deps::send_localized_message(w, from, 500971); // You can't fish while riding!
        return false;
    }
    true
}

/// `SpecialFishingNet.FullValidation(map, x, y)`: every tile within 13 is deep water.
pub fn full_validation(w: &mut World, map: Option<MapId>, x: i32, y: i32) -> bool {
    let Some(map) = map else { return false };
    for ty in (y - 13)..=(y + 13) {
        for tx in (x - 13)..=(x + 13) {
            let id = i32::from(w.maps.land_tile(map, tx, ty).id);
            if !WATER_LAND_TILES.chunks(2).any(|p| id >= p[0] && id <= p[1]) {
                return false;
            }
        }
    }
    true
}

/// `MutateResource` for deep water from UO:R.
pub fn deep_water_resource(
    w: &mut World,
    map: Option<MapId>,
    loc: Point3D,
) -> Option<HarvestResource> {
    // From UO:R deep water needs 65 skill to catch anything, and a GM succeeds 70% of the time.
    if crate::harvest::core_uor(w) && full_validation(w, map, loc.x, loc.y) {
        return Some(HarvestResource::new(
            0.0,
            65.0,
            115.0,
            1043297,
            &["Server.Items.Fish"],
        ));
    }
    None
}

/// `CheckResources`: an SOS in the pack whose location is within 60 tiles means there is always something to catch.
pub fn check_resources_sos(w: &mut World, from: MobileId) -> bool {
    can_fish_up_shipwreck(w, from)
        && super::state_hooks(w)
            .and_then(|h| h.sos_near)
            .is_some_and(|f| f(w, from).is_some())
}

fn can_fish_up_shipwreck(w: &mut World, from: MobileId) -> bool {
    // From UO:R the chest needs 65 skill, and below that an SOS changes nothing.
    !crate::harvest::core_uor(w) || skills::base(w, from, SkillName::Fishing) >= 65.0
}

pub fn on_harvest_started(w: &mut World, from: MobileId, to: ToHarvest) {
    if let Some((_, map, loc, _)) = super::get_harvest_details(w, from, to) {
        muo_server::timer::start(
            w,
            TimeSpan::from_millis(1500),
            TimeSpan::ZERO,
            1,
            move |w| {
                if muo_content::core::t2a(w) {
                    muo_server::mobile::base::revealing_action(w, from);
                }
                muo_server::effects::send_location_effect(w, loc, map, 0x352D, 16, 4, 0, 0);
                muo_server::effects::play_sound(w, loc, map, 0x364);
            },
        );
    }
}

/// `SpecialHarvest`: the Collector quest's rainbow pearl.
pub fn special_harvest(w: &mut World, from: MobileId) -> bool {
    super::state_hooks(w)
        .and_then(|h| h.collector_pearl)
        .is_some_and(|f| f(w, from))
}

/// `Construct(type, from)`.
pub fn construct(w: &mut World, ty: &'static str, from: MobileId) -> Option<ItemId> {
    let map = if mx::map(w, from) == Some(MapId::FELUCCA) {
        MapId::FELUCCA
    } else {
        MapId::TRAMMEL
    };
    if ty == "Server.Items.TreasureMap" || ty == "Server.Items.MessageInABottle" {
        let hook = super::state_hooks(w).and_then(|h| h.make_sea_item);
        if let Some(f) = hook {
            let level = if ty == "Server.Items.TreasureMap" {
                let young = w
                    .mobile(from)
                    .is_some_and(|m| m.ty.is(&muo_content::player_mobile::PLAYER_MOBILE))
                    && muo_content::player_mobile::data(w, from)
                        .is_some_and(|d| d.flag(muo_content::player_mobile::PlayerFlag::YOUNG));
                let in_haven = super::state_hooks(w)
                    .and_then(|h| h.in_haven_island)
                    .is_some_and(|f| f(w, from));
                if young && mx::map(w, from) == Some(MapId::TRAMMEL) && in_haven {
                    0
                } else {
                    1
                }
            } else {
                0
            };
            return f(w, ty, level, map);
        }
        return None;
    }
    // The pearl is a bonus on top of the catch, so an SOS never takes its place.
    if ty != "Server.Items.WhitePearl"
        && can_fish_up_shipwreck(w, from)
        && let Some(f) = super::state_hooks(w).and_then(|h| h.sos_catch)
        && let Some(item) = f(w, from)
    {
        return Some(item);
    }
    muo_content::factory::make_item(w, ty)
}

/// `MutateType`: the rare catches.
pub fn mutate_type(
    w: &mut World,
    ty: &'static str,
    from: MobileId,
    map: Option<MapId>,
    loc: Point3D,
) -> &'static str {
    let deep_water = full_validation(w, map, loc.x, loc.y);
    let skill_base = skills::base(w, from, SkillName::Fishing);
    let skill_value = skills::value(w, from, SkillName::Fishing);
    let expansion = muo_content::core::expansion(w);
    for e in MUTATE_TABLE {
        if expansion < e.expansion || e.max_expansion.is_some_and(|m| expansion > m) {
            continue;
        }
        if !deep_water && e.deep_water {
            continue;
        }
        if skill_base >= e.req_skill {
            let chance = (skill_value - e.min_skill) / (e.max_skill - e.min_skill);
            if chance > utility::random_double(&mut w.rng) {
                let i = utility::random(&mut w.rng, e.types.len() as i32) as usize;
                return e.types[i];
            }
        }
    }
    ty
}

/// `Give(m, item, placeAtFeet)`: a treasure map, bottle or net comes with a sea serpent instead. Returns
/// `(handled, result)`; when not handled the caller does the base `Give`.
pub fn give(w: &mut World, m: MobileId, item: ItemId, place_at_feet: bool) -> (bool, bool) {
    let name = w.item(item).map_or("", |i| i.ty.full_name);
    for n in [
        "Server.Items.TreasureMap",
        "Server.Items.MessageInABottle",
        "Server.Items.SpecialFishingNet",
    ] {
        if muo_content::items::is_named(
            w.item(item)
                .map(|i| i.ty)
                .unwrap_or(&muo_server::item::ITEM_BASE),
            n,
        ) {
            surface_serpent(w, m, Some(item));
            return (true, true); // we don't want to give the item to the player, it's on the serpent
        }
    }
    let at_feet = place_at_feet
        || [
            "Server.Items.BigFish",
            "Server.Items.WoodenChest",
            "Server.Items.MetalGoldenChest",
        ]
        .iter()
        .any(|n| muo_content::factory::item_is(w, item, n));
    let _ = name;
    let _ = at_feet;
    // The base Give with the widened `placeAtFeet`.
    (true, base_give(w, m, item, at_feet))
}

fn base_give(w: &mut World, m: MobileId, item: ItemId, place_at_feet: bool) -> bool {
    if muo_server::mobile::part2::items::place_in_backpack(w, m, item) {
        return true;
    }
    if !place_at_feet {
        return false;
    }
    let Some(map) = mx::map(w, m) else {
        return false;
    };
    for i in muo_server::mobile::part2::enumerators::get_items_at(w, m) {
        if muo_server::item::base::stack_with(w, i, Some(m), i, false) {
            return true;
        }
    }
    let loc = mx::location(w, m);
    ops::move_to_world_map(w, item, loc, Some(map));
    true
}

/// `SurfaceSerpent(m)`: a sea serpent rises with the item in its pack.
fn surface_serpent(w: &mut World, m: MobileId, item: Option<ItemId>) {
    let name = if utility::random_double(&mut w.rng) < 0.1 {
        "Server.Mobiles.DeepSeaSerpent"
    } else {
        "Server.Mobiles.SeaSerpent"
    };
    let Some(serp) = muo_content::factory::make_mobile(w, name) else {
        // No serpent class: the player keeps the catch.
        if let Some(item) = item {
            base_give(w, m, item, true);
        }
        return;
    };
    let loc = mx::location(w, m);
    let map = mx::map(w, m);
    let (mut x, mut y) = (loc.x, loc.y);
    if let Some(map) = map {
        for _ in 0..20 {
            let tx = loc.x - 10 + utility::random(&mut w.rng, 21);
            let ty = loc.y - 10 + utility::random(&mut w.rng, 21);
            let t = w.maps.land_tile(map, tx, ty);
            let id = i32::from(t.id);
            if t.z == -5
                && ((0xA8..=0xAB).contains(&id) || (0x136..=0x137).contains(&id))
                && !super::state_hooks(w)
                    .and_then(|h| h.check_multi)
                    .is_some_and(|f| f(w, Point3D::new(tx, ty, -5), map))
            {
                x = tx;
                y = ty;
                break;
            }
        }
        muo_server::mobile::part1::lifecycle::move_to_world(w, serp, Point3D::new(x, y, -5), map);
        // `serp.Home = serp.Location; serp.RangeHome = 10;` (10 is the creature default)
        muo_content::creature::control::set_home(w, serp, Point3D::new(x, y, -5));
    }
    // serp.Home = serp.Location; serp.RangeHome = 10;
    let at = mx::location(w, serp);
    muo_content::creature::control::set_home(w, serp, at);
    if let Some(d) = muo_content::creature::data::cd_mut(w, serp) {
        d.range_home = 10;
    }
    // serp.PackItem(item)
    muo_content::creature::setters::pack_item(w, serp, item);
    mx::set_combatant(w, serp, Some(m));
    muo_server::timer::start(w, TimeSpan::from_mins(5), TimeSpan::ZERO, 1, move |w| {
        // Damage entries last 2 minutes and clear at full health, so a serpent nobody is fighting goes back to deep water.
        if !mx::deleted(w, serp)
            && super::state_hooks(w)
                .and_then(|h| h.has_recent_damager)
                .is_none_or(|f| !f(w, serp))
            && let Some(t) = w.mobile(serp).map(|x| x.ty)
        {
            (t.delete)(w, serp);
        }
    });
    deps::send_localized_message(w, m, 503170); // Uh oh! That doesn't look like a fish!
}

/// `SendSuccessTo(from, item, resource)` of fishing.
pub fn send_success_to(w: &mut World, from: MobileId, item: ItemId) {
    let is = |w: &World, n: &str| muo_content::factory::item_is(w, item, n);
    if is(w, "Server.Items.BigFish") {
        deps::send_localized_message(w, from, 1042635); // Your fishing pole bends as you pull a big fish from the depths!
        if let Some(f) = super::state_hooks(w).and_then(|h| h.set_fisher) {
            f(w, item, from);
        }
    } else if is(w, "Server.Items.WoodenChest") || is(w, "Server.Items.MetalGoldenChest") {
        deps::send_localized_message(w, from, 503175); // You pull up a heavy chest from the depths of the ocean!
    } else {
        let data = w.item(item).map(|i| i.item_id).unwrap_or(0);
        let tile = w.maps.tile_data.items.get(data as usize).cloned();
        let tile_name = tile.as_ref().map(|t| t.name.clone()).unwrap_or_default();
        let (number, name) = if is(w, "Server.Items.BaseMagicFish") {
            (1008124, "a mess of small fish".to_string())
        } else if is(w, "Server.Items.Fish") || is(w, "Server.Items.BaseShoes") {
            (1008124, tile_name)
        } else if is(w, "Server.Items.TreasureMap") {
            (1008125, "a sodden piece of parchment".to_string())
        } else if is(w, "Server.Items.MessageInABottle") {
            (1008125, "a bottle, with a message in it".to_string())
        } else if is(w, "Server.Items.SpecialFishingNet") {
            (1008125, "a special fishing net".to_string()) // TODO: this is just a guess
        } else {
            use muo_data::tile_data::TileFlag;
            let flags = tile.as_ref().map_or(TileFlag::empty(), |t| t.flags);
            let n = if flags.contains(TileFlag::ARTICLE_A) {
                format!("a {tile_name}")
            } else if flags.contains(TileFlag::ARTICLE_AN) {
                format!("an {tile_name}")
            } else {
                tile_name
            };
            (1043297, n)
        };
        let high_seas = w
            .mobile(from)
            .and_then(|m| m.net_state)
            .and_then(|ns| w.net_state(ns))
            .is_some_and(|n| n.caps.high_seas());
        if number == 1043297 || high_seas {
            muo_server::mobile::part2::messages::send_localized_message(
                w, from, number, &name, 0x3B2,
            );
        } else {
            muo_server::mobile::part2::messages::send_localized_message_affix(
                w, from, number, true, &name, "", 0,
            );
        }
    }
}

/// `OnHarvestFinished`: from T2A the player is revealed, and deep water sometimes yields a serpent.
pub fn on_harvest_finished(w: &mut World, from: MobileId, to: ToHarvest) {
    if !muo_content::core::t2a(w) {
        return;
    }
    muo_server::mobile::base::revealing_action(w, from);
    // From T2A, 1% of casts in deep water at 80 skill or more bring up an empty sea serpent, whatever the catch.
    if skills::base(w, from, SkillName::Fishing) >= 80.0
        && utility::random_double(&mut w.rng) < 0.01
        && let Some((_, map, loc, _)) = super::get_harvest_details(w, from, to)
        && full_validation(w, map, loc.x, loc.y)
    {
        // SurfaceSerpent(from) with no item: the serpent only.
        surface_serpent(w, from, None);
    }
}
