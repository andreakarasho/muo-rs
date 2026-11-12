//! The mining harvest system: ore and stone, and sand for glassblowing.
//!
//! The Witch Apprentice quest's grave and `TreasureMap.OnBeginDig` of `HarvestTarget` are hooks (`HarvestHooks`);
//! the elementals `GargoylesPickaxe` calls are made by name.

use muo_content::player_mobile::{self, PlayerFlag};
use muo_io::text::TextDefinition as Td;
use muo_primitives::{Point3D, TimeSpan, utility};
use muo_server::World;
use muo_server::context_menu_entry::ContextMenuEntry;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::deps;
use muo_server::skills::{self, SkillName};
use muo_server::target::Targeted;

use super::*;

const MOUNTAIN_CAVE_TILES: &[i32] = &[
    220, 231, 236, 247, 252, 236, 268, 279, 286, 297, 321, 324, 467, 474, 476, 487, 492, 495, 543,
    579, 581, 621, // 1010, 1010, // This is missing from the files in newer clients
    1741, 1757, 1771, 1790, 1801, 1824, 1831, 1854, 1861, 1884, 1981, 2004, 2028, 2033, 2100, 2105,
    0x3F39, 0x3F74, 0x3F82, 0x3F8F, 0x3F91, 0x3FCF,
];

const MOUNTAIN_CAVE_STATIC_TILES: &[i32] = &[0x053B, 0x054F];

const SAND_TILES: &[i32] = &[
    22, 62, 68, 75, 286, 301, 402, 402, 424, 427, 441, 465, 642, 645, 650, 657, 821, 828, 833, 836,
    845, 852, 857, 860, 951, 958, 967, 970, 1447, 1458, 1611, 1618, 1623, 1626, 1635, 1642, 1647,
    1650,
];

const OFFSETS: [i32; 16] = [-1, -1, -1, 0, -1, 1, 0, -1, 0, 1, 1, -1, 1, 0, 1, 1];

fn res(req: f64, min: f64, max: f64, msg: i32, types: [&'static str; 3]) -> HarvestResource {
    HarvestResource::new(req, min, max, msg, &types)
}

/// `new Mining()`: the ore-and-stone definition (0) and the sand one (1).
pub fn create(w: &mut World) -> HarvestSystem {
    let ml = muo_content::core::ml(w);
    let mut ore = HarvestDefinition {
        bank_width: 8,
        bank_height: 8,
        min_total: 10,
        max_total: 34,
        min_respawn: TimeSpan::from_mins(10),
        max_respawn: TimeSpan::from_mins(20),
        skill: SkillName::Mining,
        land_tiles: MOUNTAIN_CAVE_TILES.to_vec(),
        static_tiles: MOUNTAIN_CAVE_STATIC_TILES.to_vec(),
        ranged_tiles: true,
        max_range: 2,
        consumed_per_harvest: 1,
        consumed_per_felucca_harvest: 2,
        effect_actions: vec![11],
        effect_sounds: vec![0x125, 0x126],
        effect_counts: vec![1],
        effect_delay: TimeSpan::from_millis(1600),
        effect_sound_delay: TimeSpan::from_millis(900),
        no_resources_message: Td::of_number(503040), // There is no metal here to mine.
        double_harvest_message: Td::of_number(503042), // Someone has gotten to the metal before you.
        timed_out_of_range_message: Td::of_number(503041), // You have moved too far away to continue mining.
        out_of_range_message: Td::of_number(500446),       // That is too far away.
        fail_message: Td::of_number(503043), // You loosen some rocks but fail to find any useable ore.
        pack_full_message: Td::of_number(1010481), // Your backpack is full, so the ore you mined is lost.
        tool_broke_message: Td::of_number(1044038), // You have worn out your tool!
        ..Default::default()
    };
    ore.resources = vec![
        res(
            0.0,
            0.0,
            100.0,
            1007072,
            ["Server.Items.IronOre", "Server.Items.Granite", ""],
        ),
        res(
            65.0,
            25.0,
            105.0,
            1007073,
            [
                "Server.Items.DullCopperOre",
                "Server.Items.DullCopperGranite",
                "Server.Mobiles.DullCopperElemental",
            ],
        ),
        res(
            70.0,
            30.0,
            110.0,
            1007074,
            [
                "Server.Items.ShadowIronOre",
                "Server.Items.ShadowIronGranite",
                "Server.Mobiles.ShadowIronElemental",
            ],
        ),
        res(
            75.0,
            35.0,
            115.0,
            1007075,
            [
                "Server.Items.CopperOre",
                "Server.Items.CopperGranite",
                "Server.Mobiles.CopperElemental",
            ],
        ),
        res(
            80.0,
            40.0,
            120.0,
            1007076,
            [
                "Server.Items.BronzeOre",
                "Server.Items.BronzeGranite",
                "Server.Mobiles.BronzeElemental",
            ],
        ),
        res(
            85.0,
            45.0,
            125.0,
            1007077,
            [
                "Server.Items.GoldOre",
                "Server.Items.GoldGranite",
                "Server.Mobiles.GoldenElemental",
            ],
        ),
        res(
            90.0,
            50.0,
            130.0,
            1007078,
            [
                "Server.Items.AgapiteOre",
                "Server.Items.AgapiteGranite",
                "Server.Mobiles.AgapiteElemental",
            ],
        ),
        res(
            95.0,
            55.0,
            135.0,
            1007079,
            [
                "Server.Items.VeriteOre",
                "Server.Items.VeriteGranite",
                "Server.Mobiles.VeriteElemental",
            ],
        ),
        res(
            99.0,
            59.0,
            139.0,
            1007080,
            [
                "Server.Items.ValoriteOre",
                "Server.Items.ValoriteGranite",
                "Server.Mobiles.ValoriteElemental",
            ],
        ),
    ];
    // Iron's resource has only the ore and the granite.
    ore.resources[0].types.truncate(2);
    let vein = |chance, fallback_chance, primary, fallback| HarvestVein {
        vein_chance: chance,
        chance_to_fallback: fallback_chance,
        primary,
        fallback,
    };
    ore.set_veins(vec![
        vein(496, 0.0, 0, None),    // Iron
        vein(112, 0.5, 1, Some(0)), // Dull Copper
        vein(98, 0.5, 2, Some(0)),  // Shadow Iron
        vein(84, 0.5, 3, Some(0)),  // Copper
        vein(70, 0.5, 4, Some(0)),  // Bronze
        vein(56, 0.5, 5, Some(0)),  // Gold
        vein(42, 0.5, 6, Some(0)),  // Agapite
        vein(28, 0.5, 7, Some(0)),  // Verite
        vein(14, 0.5, 8, Some(0)),  // Valorite
    ]);
    if ml {
        let b = |req, chance, msg: i32, ty: Option<&'static str>| BonusResource {
            req_skill: req,
            chance,
            message: if msg == 0 {
                Td::EMPTY
            } else {
                Td::of_number(msg)
            },
            ty,
        };
        ore.bonus_resources = Some(vec![
            b(0.0, 99.4, 0, None), // Nothing
            b(100.0, 0.1, 1072562, Some("Server.Items.BlueDiamond")),
            b(100.0, 0.1, 1072567, Some("Server.Items.DarkSapphire")),
            b(100.0, 0.1, 1072570, Some("Server.Items.EcruCitrine")),
            b(100.0, 0.1, 1072564, Some("Server.Items.FireRuby")),
            b(100.0, 0.1, 1072566, Some("Server.Items.PerfectEmerald")),
            b(100.0, 0.1, 1072568, Some("Server.Items.Turquoise")),
        ]);
    }
    ore.race_bonus = ml;
    ore.randomize_veins = ml;

    let mut sand = HarvestDefinition {
        bank_width: 8,
        bank_height: 8,
        min_total: 6,
        max_total: 12,
        min_respawn: TimeSpan::from_mins(10),
        max_respawn: TimeSpan::from_mins(20),
        skill: SkillName::Mining,
        land_tiles: SAND_TILES.to_vec(),
        static_tiles: Vec::new(),
        ranged_tiles: true,
        max_range: 2,
        consumed_per_harvest: 1,
        consumed_per_felucca_harvest: 1,
        effect_actions: vec![11],
        effect_sounds: vec![0x125, 0x126],
        effect_counts: vec![6],
        effect_delay: TimeSpan::from_millis(1600),
        effect_sound_delay: TimeSpan::from_millis(900),
        no_resources_message: Td::of_number(1044629),
        double_harvest_message: Td::of_number(1044629),
        timed_out_of_range_message: Td::of_number(503041),
        out_of_range_message: Td::of_number(500446),
        fail_message: Td::of_number(1044630), // You dig for a while but fail to find any of sufficient quality for glassblowing.
        pack_full_message: Td::of_number(1044632), // Your backpack can't hold the sand, and it is lost!
        tool_broke_message: Td::of_number(1044038),
        ..Default::default()
    };
    sand.resources = vec![HarvestResource::new(
        100.0,
        70.0,
        400.0,
        1044631,
        &["Server.Items.Sand"],
    )];
    sand.set_veins(vec![vein(1000, 0.0, 0, None)]);

    HarvestSystem {
        kind: Kind::Mining,
        name: "Mining",
        definitions: vec![ore, sand],
    }
}

fn is_player(w: &World, m: MobileId) -> bool {
    w.mobile(m)
        .is_some_and(|x| x.ty.is(&player_mobile::PLAYER_MOBILE))
}

fn flag(w: &World, m: MobileId, f: PlayerFlag) -> bool {
    is_player(w, m) && player_mobile::data(w, m).is_some_and(|d| d.flag(f))
}

fn mounted_or_morphed(w: &mut World, from: MobileId) -> bool {
    if muo_server::mobile::part1::props_b::mounted(w, from) {
        deps::send_localized_message(w, from, 501864); // You can't mine while riding.
        return true;
    }
    let human = w.mobile(from).is_some_and(|_| {
        w.resources
            .get::<muo_primitives::BodyTable>()
            .is_some_and(|t| t.is_human(muo_server::mobile::part1::props_b::body(w, from)))
    });
    if muo_server::mobile::part1::props_b::is_body_mod(w, from) && !human {
        deps::send_localized_message(w, from, 501865); // You can't mine while polymorphed.
        return true;
    }
    false
}

/// `CheckHarvest(from, tool)` beyond the base's tool check.
pub fn check_harvest(w: &mut World, from: MobileId) -> bool {
    !mounted_or_morphed(w, from)
}

/// `CheckHarvest(from, tool, def, toHarvest)` beyond the base's tool check.
pub fn check_harvest_def(w: &mut World, from: MobileId, di: usize, to: ToHarvest) -> bool {
    if di == 1
        && !(is_player(w, from)
            && skills::base(w, from, SkillName::Mining) >= 100.0
            && flag(w, from, PlayerFlag::SAND_MINING))
    {
        on_bad_harvest_target(w, from, to);
        return false;
    }
    !mounted_or_morphed(w, from)
}

pub fn on_bad_harvest_target(w: &mut World, from: MobileId, to: ToHarvest) {
    if matches!(to, ToHarvest::Land(_)) {
        deps::send_localized_message(w, from, 501862); // You can't mine there.
    } else {
        deps::send_localized_message(w, from, 501863); // You can't mine that.
    }
}

/// The special cases of `HarvestTarget.OnTarget` for mining; true if handled.
pub fn on_target_special(w: &mut World, from: MobileId, targeted: Targeted) -> bool {
    if let Targeted::Static(s) = targeted
        && matches!(s.item_id, 0xED3 | 0xEDF | 0xEE0 | 0xEE1 | 0xEE2 | 0xEE8)
        && is_player(w, from)
        && let Some(f) = super::state_hooks(w).and_then(|h| h.witch_apprentice_bones)
    {
        // `qs is not WitchApprenticeQuest` returns, the bones objective completes: the hook answers "consumed".
        if f(w, from) {
            return true;
        }
    }
    if let Targeted::Entity(EntityId::Item(i)) = targeted
        && muo_content::factory::item_is(w, i, "Server.Items.TreasureMap")
        && let Some(f) = super::state_hooks(w).and_then(|h| h.treasure_map_begin_dig)
    {
        f(w, i, from);
        return true;
    }
    false
}

/// `MutateVein`: a GargoylesPickaxe finds the next richer vein.
pub fn mutate_vein(w: &World, sys: &HarvestSystem, tool: ItemId, di: usize, vein: usize) -> usize {
    if di == 0
        && muo_content::factory::item_is(w, tool, "Server.Items.GargoylesPickaxe")
        && vein + 1 < sys.definitions[0].veins.len()
    {
        return vein + 1;
    }
    vein
}

/// `GetResourceType` for the ore definition: ore, or (sometimes) stone for a miner with the stone-mining reward.
pub fn get_resource_type(
    w: &mut World,
    from: MobileId,
    di: usize,
    resource: &HarvestResource,
) -> Option<&'static str> {
    if di != 0 {
        return None;
    }
    if skills::base(w, from, SkillName::Mining) >= 100.0
        && flag(w, from, PlayerFlag::STONE_MINING)
        && flag(w, from, PlayerFlag::TOGGLE_MINING_STONE)
        && utility::random_double(&mut w.rng) < 0.1
    {
        return resource.types.get(1).copied();
    }
    resource.types.first().copied()
}

/// `OnHarvestFinished`: a GargoylesPickaxe may call an elemental.
#[allow(clippy::too_many_arguments)]
pub fn on_harvest_finished(
    w: &mut World,
    from: MobileId,
    tool: ItemId,
    di: usize,
    vein: HarvestVein,
    resource: &HarvestResource,
    res_index: Option<usize>,
) {
    if !muo_content::factory::item_is(w, tool, "Server.Items.GargoylesPickaxe")
        || di != 0
        || utility::random_double(&mut w.rng) >= 0.1
    {
        return;
    }
    if res_index != Some(vein.primary) || resource.types.len() < 3 {
        return;
    }
    let Some(map) = mx::map(w, from) else { return };
    let Some(spawned) = muo_content::factory::make_mobile(w, resource.types[2]) else {
        return;
    };
    let loc = mx::location(w, from);
    let offset = utility::random(&mut w.rng, 8) * 2;
    for i in (0..OFFSETS.len() as i32).step_by(2) {
        let x = loc.x + OFFSETS[((offset + i) as usize) % OFFSETS.len()];
        let y = loc.y + OFFSETS[((offset + i + 1) as usize) % OFFSETS.len()];
        if spawn_at(w, spawned, from, map, Point3D::new(x, y, loc.z), true) {
            return;
        }
        let z = w.maps.get_average_z(map, x, y);
        if (z - loc.z).abs() < 10 && spawn_at(w, spawned, from, map, Point3D::new(x, y, z), true) {
            return;
        }
    }
    spawn_at(w, spawned, from, map, loc, false);
}

fn spawn_at(
    w: &mut World,
    bc: MobileId,
    target: MobileId,
    map: muo_server::map::MapId,
    p: Point3D,
    check: bool,
) -> bool {
    if check && !muo_server::map::fit::can_spawn_mobile_at(w, map, p.x, p.y, p.z) {
        return false;
    }
    if let Some(t) = w.mobile(bc).map(|m| m.ty) {
        (t.on_before_spawn)(w, bc, p, map);
    }
    muo_server::mobile::part1::lifecycle::move_to_world(w, bc, p, map);
    mx::set_combatant(w, bc, Some(target));
    true
}

/// `BaseHarvestTool.AddContextMenuEntries(from, item, ref list, system)` for mining.
pub fn add_context_menu_entries(
    w: &mut World,
    from: MobileId,
    tool: ItemId,
    list: &mut Vec<ContextMenuEntry>,
) {
    let is_mining = super::tool_system(w, tool) == Some(Kind::Mining);
    if !is_mining || !is_player(w, from) {
        return;
    }
    let pack = deps::find_item_on_layer(w, from, muo_server::layer::Layer::Backpack);
    let in_pack =
        pack.is_some_and(|p| muo_server::item::ops::is_child_of(w, tool, Some(EntityId::Item(p))));
    let held = w
        .item(tool)
        .is_some_and(|i| i.parent == Some(EntityId::Mobile(from)));
    if !in_pack && !held {
        return;
    }
    let toggle = flag(w, from, PlayerFlag::TOGGLE_MINING_STONE);
    let mut header = ContextMenuEntry::new(if toggle { 6179 } else { 6178 }, -1);
    header.color = 0x421F;
    list.push(header);
    let stone_mining = flag(w, from, PlayerFlag::STONE_MINING)
        && skills::value_ref(w, from, SkillName::Mining) >= 100.0;
    list.push(toggle_entry(false, toggle, 6176));
    list.push(toggle_entry(true, !toggle && stone_mining, 6177));
}

fn toggle_entry(value: bool, enabled: bool, number: i32) -> ContextMenuEntry {
    let mut e = ContextMenuEntry::new(number, -1);
    e.enabled = enabled;
    e.on_click = Some(std::rc::Rc::new(
        move |w: &mut World, from: MobileId, _target: EntityId| {
            if !is_player(w, from) {
                return;
            }
            let old = flag(w, from, PlayerFlag::TOGGLE_MINING_STONE);
            let set = |w: &mut World, v: bool| {
                if let Some(d) = player_mobile::data_mut(w, from) {
                    d.set_flag(PlayerFlag::TOGGLE_MINING_STONE, v);
                }
            };
            if value {
                if old {
                    deps::send_localized_message(w, from, 1054023); // You are already set to mine both ore and stone!
                } else if !flag(w, from, PlayerFlag::STONE_MINING)
                    || skills::base(w, from, SkillName::Mining) < 100.0
                {
                    // You have not learned how to mine stone or you do not have enough skill!
                    deps::send_localized_message(w, from, 1054024);
                } else {
                    set(w, true);
                    deps::send_localized_message(w, from, 1054022); // You are now set to mine both ore and stone.
                }
            } else if old {
                set(w, false);
                deps::send_localized_message(w, from, 1054020); // You are now set to mine only ore.
            } else {
                deps::send_localized_message(w, from, 1054021); // You are already set to mine only ore!
            }
        },
    ));
    e
}

/// `ProspectorsTool.Prospect(from, toProspect)` after the pack check: the system's side of it.
pub fn prospect(
    w: &mut World,
    from: MobileId,
    tool: ItemId,
    to: Targeted,
) -> muo_items::weapons::pickaxes::Prospect {
    use muo_items::weapons::pickaxes::Prospect;
    let sys = system(w, Kind::Mining);
    let to = to_harvest_of(w, to);
    let Some((tile, map, loc, land)) = get_harvest_details(w, from, to) else {
        return Prospect::Cannot;
    };
    let _ = tool;
    let Some(di) = get_definition(&sys, tile, land) else {
        return Prospect::Cannot;
    };
    let def = &sys.definitions[di];
    if def.veins.len() <= 1 {
        return Prospect::Cannot;
    }
    let Some(k) = bank_key(w, sys.kind, di, def, map, loc.x, loc.y) else {
        return Prospect::Cannot;
    };
    let (vein, default_vein) = (bank_vein(w, def, k), {
        check_respawn(w, def, k);
        bank_mut(w, k).default_vein
    });
    let (Some(vein), Some(default_vein)) = (vein, default_vein) else {
        return Prospect::Cannot;
    };
    if vein != default_vein {
        return Prospect::Already;
    }
    if vein + 1 >= def.veins.len() {
        return Prospect::Valorite;
    }
    check_respawn(w, def, k);
    bank_mut(w, k).vein = Some(vein + 1);
    Prospect::Improved(vein as i32)
}
