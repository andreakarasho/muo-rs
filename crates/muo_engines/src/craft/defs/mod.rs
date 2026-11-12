//! The eleven craft systems. `InitCraftList` is `*_table.rs` (generated); here is what each class overrides (`CanCraft`, the effects, the chance at minimum).

use muo_content::items::tool;
use muo_content::mx;
use muo_io::text::TextDefinition as Td;
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::deps;
use muo_server::skills::{self, SkillName};

use super::item::derives;
use super::model::*;

pub mod hand;
pub mod traps;

pub mod alchemy_table;
pub mod blacksmithy_table;
pub mod bow_fletching_table;
pub mod carpentry_table;
pub mod cartography_table;
pub mod cooking_table;
pub mod glassblowing_table;
pub mod inscription_table;
pub mod masonry_table;
pub mod tailoring_table;
pub mod tinkering_table;

/// The names of the systems, in the order their `Initialize()` ran.
pub const NAMES: [&str; 11] = [
    "Alchemy",
    "Blacksmithy",
    "BowFletching",
    "Carpentry",
    "Cartography",
    "Cooking",
    "Glassblowing",
    "Inscription",
    "Masonry",
    "Tailoring",
    "Tinkering",
];

/// `new Def<name>()` with its `InitCraftList()`.
pub fn create(w: &World, name: &str) -> Option<CraftSystem> {
    use CraftEca::*;
    let (n, skill, title, eca, vt): (&'static str, SkillName, i32, CraftEca, SystemVt) = match name
    {
        "Alchemy" => (
            "Alchemy",
            SkillName::Alchemy,
            1044001,
            ChanceMinusSixty,
            vt(alchemy_vt),
        ),
        "Blacksmithy" => (
            "Blacksmithy",
            SkillName::Blacksmith,
            1044002,
            ChanceMinusSixtyToFortyFive,
            vt(blacksmithy_vt),
        ),
        "BowFletching" => (
            "BowFletching",
            SkillName::Fletching,
            1044006,
            FiftyPercentChanceMinusTenPercent,
            vt(bow_vt),
        ),
        "Carpentry" => (
            "Carpentry",
            SkillName::Carpentry,
            1044004,
            ChanceMinusSixty,
            vt(carpentry_vt),
        ),
        "Cartography" => (
            "Cartography",
            SkillName::Cartography,
            1044008,
            ChanceMinusSixty,
            vt(cartography_vt),
        ),
        "Cooking" => (
            "Cooking",
            SkillName::Cooking,
            1044003,
            ChanceMinusSixtyToFortyFive,
            vt(cooking_vt),
        ),
        "Glassblowing" => (
            "Glassblowing",
            SkillName::Alchemy,
            1044622,
            ChanceMinusSixty,
            vt(glassblowing_vt),
        ),
        "Inscription" => (
            "Inscription",
            SkillName::Inscribe,
            1044009,
            ChanceMinusSixty,
            vt(inscription_vt),
        ),
        "Masonry" => (
            "Masonry",
            SkillName::Carpentry,
            1044500,
            ChanceMinusSixty,
            vt(masonry_vt),
        ),
        "Tailoring" => (
            "Tailoring",
            SkillName::Tailoring,
            1044005,
            ChanceMinusSixtyToFortyFive,
            vt(tailoring_vt),
        ),
        "Tinkering" => (
            "Tinkering",
            SkillName::Tinkering,
            1044007,
            ChanceMinusSixty,
            vt(tinkering_vt),
        ),
        _ => return None,
    };
    let mut s = CraftSystem::new(n, skill, (1, 1, 1.25), Td::of_number(title), eca, vt);
    match n {
        "Alchemy" => alchemy_table::init(w, &mut s),
        "Blacksmithy" => blacksmithy_table::init(w, &mut s),
        "BowFletching" => bow_fletching_table::init(w, &mut s),
        "Carpentry" => carpentry_table::init(w, &mut s),
        "Cartography" => cartography_table::init(w, &mut s),
        "Cooking" => cooking_table::init(w, &mut s),
        "Glassblowing" => glassblowing_table::init(w, &mut s),
        "Inscription" => inscription_table::init(w, &mut s),
        "Masonry" => masonry_table::init(w, &mut s),
        "Tailoring" => tailoring_table::init(w, &mut s),
        _ => tinkering_table::init(w, &mut s),
    }
    Some(s)
}

// ---- shared pieces ----

/// `tool?.Deleted != false || tool.UsesRemaining < 0`.
fn worn_out(w: &World, t: Option<ItemId>) -> bool {
    match t {
        Some(t) => w.item(t).is_none_or(|i| i.deleted()) || tool::uses_remaining(w, t) < 0,
        None => true,
    }
}

/// The `CanCraft` most systems share: a usable tool on your person.
fn can_craft_plain(w: &mut World, from: MobileId, t: Option<ItemId>, _ty: Option<&str>) -> i32 {
    if worn_out(w, t) {
        return 1044038; // You have worn out your tool!
    }
    if !tool::check_accessible(w, t.unwrap_or(ItemId(muo_primitives::Serial::ZERO)), from) {
        return 1044263; // The tool must be on your person to use.
    }
    0
}

fn effect_none(_w: &mut World, _from: MobileId) {}

fn consume_on_failure_always(
    _w: &mut World,
    _from: MobileId,
    _ty: &str,
    _item: &CraftItem,
) -> bool {
    true
}

fn retains_color_never(_w: &World, _item: &CraftItem, _ty: &str) -> bool {
    false
}

fn requires_tool_always(_w: &World) -> bool {
    true
}

fn chance_zero(_item: &CraftItem) -> f64 {
    0.0
}

fn chance_half(_item: &CraftItem) -> f64 {
    0.5
}

/// The common `PlayEndingEffect`: tool broken, failure, quality.
fn ending_standard(w: &mut World, from: MobileId, e: Ending, _item: &CraftItem) -> i32 {
    if e.tool_broken {
        deps::send_localized_message(w, from, 1044038); // You have worn out your tool
    }
    if e.failed {
        if e.lost_material {
            return 1044043; // You failed to create the item, and some of your materials are lost.
        }
        return 1044157; // You failed to create the item, but no materials were lost.
    }
    quality_message(e)
}

fn quality_message(e: Ending) -> i32 {
    if e.quality == 0 {
        return 502785; // You were barely able to make this item.  It's quality is below average.
    }
    if e.makers_mark && e.quality == 2 {
        return 1044156; // You create an exceptional quality item and affix your maker's mark.
    }
    if e.quality == 2 {
        return 1044155; // You create an exceptional quality item.
    }
    1044154 // You create the item.
}

fn play_sound(w: &mut World, from: MobileId, sound: i32) {
    mx::play_sound(w, from, sound);
}

fn base_vt() -> SystemVt {
    SystemVt {
        chance_at_min: chance_zero,
        can_craft: can_craft_plain,
        play_craft_effect: effect_none,
        play_ending_effect: ending_standard,
        consume_on_failure: consume_on_failure_always,
        retains_color_from: retains_color_never,
        requires_tool: requires_tool_always,
    }
}

fn vt(f: fn() -> SystemVt) -> SystemVt {
    f()
}

// ---- Alchemy ----

fn alchemy_vt() -> SystemVt {
    SystemVt {
        play_craft_effect: |w, from| play_sound(w, from, 0x242),
        play_ending_effect: |w, from, e, item| {
            if e.tool_broken {
                deps::send_localized_message(w, from, 1044038); // You have worn out your tool
            }
            let potion = derives(w, item.item_type, "Server.Items.BasePotion");
            if e.failed {
                if potion {
                    if let Some(b) = muo_content::factory::make_item(w, "Server.Items.Bottle") {
                        muo_server::mobile::part2::items::add_to_backpack(w, from, b);
                    }
                    return 500287; // You fail to create a useful potion.
                }
                return 1044043; // You failed to create the item, and some of your materials are lost.
            }
            play_sound(w, from, 0x240); // Sound of a filling bottle
            if potion {
                if e.quality == -1 {
                    return 1048136; // You create the potion and pour it into a keg.
                }
                return 500279; // You pour the potion into a bottle...
            }
            1044154 // You create the item.
        },
        ..base_vt()
    }
}

// ---- Blacksmithy ----

/// `ForgeAttribute` / `AnvilAttribute`: the item classes carrying them.
const ANVILS: &[&str] = &[
    "Server.Items.Anvil",
    "Server.Items.AnvilComponent",
    "Server.Items.ColoredAnvil",
];
const FORGES: &[&str] = &[
    "Server.Items.Forge",
    "Server.Items.ForgeComponent",
    "Server.Items.LargeForgeWest",
    "Server.Items.LargeForgeEast",
    "Server.Items.LargeForgeWest+InternalItem",
    "Server.Items.LargeForgeWest+InternalItem2",
    "Server.Items.LargeForgeEast+InternalItem",
    "Server.Items.LargeForgeEast+InternalItem2",
    "Server.Items.ShrineOfWisdomComponent",
];

/// `DefBlacksmithy.CheckAnvilAndForge(from, range, out anvil, out forge)`.
pub fn check_anvil_and_forge(w: &mut World, from: MobileId, range: i32) -> (bool, bool) {
    let (mut anvil, mut forge) = (false, false);
    let Some(map) = mx::map(w, from) else {
        return (false, false);
    };
    let loc = mx::location(w, from);

    for it in muo_server::mobile::part2::enumerators::get_items_in_range(w, from, range) {
        let Some((ty, id, z)) = w.item(it).map(|i| (i.ty, i.item_id, i.location.z)) else {
            continue;
        };
        let is_anvil = ANVILS.iter().any(|n| muo_content::items::is_named(ty, n))
            || matches!(id, 4015 | 4016 | 11733 | 11734);
        let is_forge = FORGES.iter().any(|n| muo_content::items::is_named(ty, n))
            || id == 4017
            || (6522..=6569).contains(&id)
            || id == 11736;
        if is_anvil || is_forge {
            if loc.z + 16 < z || z + 16 < loc.z || !mx_in_los_item(w, from, it) {
                continue;
            }
            anvil = anvil || is_anvil;
            forge = forge || is_forge;
            if anvil && forge {
                break;
            }
        }
    }

    let mut x = -range;
    while (!anvil || !forge) && x <= range {
        let mut y = -range;
        while (!anvil || !forge) && y <= range {
            for tile in muo_server::map::tiles::static_and_multi_tiles(w, map, loc.x + x, loc.y + y)
            {
                let id = i32::from(tile.id);
                let is_anvil = matches!(id, 4015 | 4016 | 11733 | 11734);
                let is_forge = id == 4017 || (6522..=6569).contains(&id) || id == 11736;
                if is_anvil || is_forge {
                    let tz = i32::from(tile.z);
                    let height = w
                        .maps
                        .tile_data
                        .items
                        .get(id as usize)
                        .map_or(0, |d| i32::from(d.height));
                    let p = muo_primitives::Point3D::new(loc.x + x, loc.y + y, tz + height / 2 + 1);
                    if loc.z + 16 < tz
                        || tz + 16 < loc.z
                        || !muo_server::mobile::part1::combat::in_los_point(w, from, p)
                    {
                        continue;
                    }
                    anvil = anvil || is_anvil;
                    forge = forge || is_forge;
                }
            }
            y += 1;
        }
        x += 1;
    }
    (anvil, forge)
}

fn mx_in_los_item(w: &mut World, from: MobileId, item: ItemId) -> bool {
    muo_server::item::deps::in_los_item(w, from, item)
}

fn blacksmithy_vt() -> SystemVt {
    SystemVt {
        can_craft: |w, from, t, _ty| {
            if worn_out(w, t) {
                return 1044038; // You have worn out your tool!
            }
            let t = t.unwrap_or(ItemId(muo_primitives::Serial::ZERO));
            if !tool::check_tool(w, t, from) {
                return 1048146; // If you have a tool equipped, you must use that tool.
            }
            if !tool::check_accessible(w, t, from) {
                return 1044263; // The tool must be on your person to use.
            }
            let (anvil, forge) = check_anvil_and_forge(w, from, 2);
            if anvil && forge {
                return 0;
            }
            1044267 // You must be near an anvil and a forge to smith items.
        },
        play_craft_effect: |w, from| play_sound(w, from, 0x2A),
        ..base_vt()
    }
}

// ---- BowFletching, Carpentry ----

fn bow_vt() -> SystemVt {
    SystemVt {
        chance_at_min: chance_half,
        play_craft_effect: |w, from| play_sound(w, from, 0x55),
        ..base_vt()
    }
}

fn carpentry_vt() -> SystemVt {
    SystemVt {
        chance_at_min: chance_half,
        play_craft_effect: |w, from| play_sound(w, from, 0x23D),
        ..base_vt()
    }
}

// ---- Cartography ----

fn cartography_vt() -> SystemVt {
    SystemVt {
        requires_tool: |w| !super::state::t2a_enabled(w),
        can_craft: |w, from, t, ty| {
            if !super::state::t2a_enabled(w) {
                return can_craft_plain(w, from, t, ty);
            }
            0
        },
        play_craft_effect: |w, from| play_sound(w, from, 0x249),
        ..base_vt()
    }
}

// ---- Cooking ----

fn cooking_vt() -> SystemVt {
    base_vt()
}

// ---- Glassblowing ----

fn glassblowing_vt() -> SystemVt {
    SystemVt {
        chance_at_min: |item| {
            if item.item_type == "Server.Items.HollowPrism" {
                0.5
            } else {
                0.0
            }
        },
        can_craft: |w, from, t, _ty| {
            if worn_out(w, t) {
                return 1044038; // You have worn out your tool!
            }
            let t = t.unwrap_or(ItemId(muo_primitives::Serial::ZERO));
            if !tool::check_tool(w, t, from) {
                return 1048146; // If you have a tool equipped, you must use that tool.
            }
            if !(player_flag(
                w,
                from,
                muo_content::player_mobile::PlayerFlag::GLASSBLOWING,
            ) && skills::base(w, from, SkillName::Alchemy) >= 100.0)
            {
                return 1044634; // You havent learned glassblowing.
            }
            if !tool::check_accessible(w, t, from) {
                return 1044263; // The tool must be on your person to use.
            }
            let (_, forge) = check_anvil_and_forge(w, from, 2);
            if forge { 0 } else { 1044628 } // You must be near a forge to blow glass.
        },
        play_craft_effect: |w, from| play_sound(w, from, 0x2B), // bellows
        play_ending_effect: |w, from, e, _item| {
            if e.tool_broken {
                deps::send_localized_message(w, from, 1044038); // You have worn out your tool
            }
            if e.failed {
                if e.lost_material {
                    return 1044043; // You failed to create the item, and some of your materials are lost.
                }
                return 1044157; // You failed to create the item, but no materials were lost.
            }
            play_sound(w, from, 0x41); // glass breaking
            quality_message(e)
        },
        ..base_vt()
    }
}

fn player_flag(w: &World, m: MobileId, flag: muo_content::player_mobile::PlayerFlag) -> bool {
    w.mobile(m)
        .is_some_and(|x| x.ty.is(&muo_content::player_mobile::PLAYER_MOBILE))
        && muo_content::player_mobile::data(w, m).is_some_and(|d| d.flag(flag))
}

// ---- Inscription ----

fn inscription_vt() -> SystemVt {
    SystemVt {
        requires_tool: |w| !super::state::t2a_enabled(w),
        can_craft: |w, from, t, ty| {
            if !super::state::t2a_enabled(w) {
                if worn_out(w, t) {
                    return 1044038; // You have worn out your tool!
                }
                if !tool::check_accessible(
                    w,
                    t.unwrap_or(ItemId(muo_primitives::Serial::ZERO)),
                    from,
                ) {
                    return 1044263; // The tool must be on your person to use.
                }
            }
            if let Some(ty) = ty
                && derives(w, ty, "Server.Items.SpellScroll")
            {
                let known = super::state::state(w)
                    .and_then(|s| s.spell_known)
                    .is_some_and(|f| f(w, from, ty));
                return if known { 0 } else { 1042404 }; // null : You don't have that spell!
            }
            0
        },
        play_craft_effect: |w, from| play_sound(w, from, 0x249),
        play_ending_effect: |w, from, e, item| {
            if e.tool_broken {
                deps::send_localized_message(w, from, 1044038); // You have worn out your tool
            }
            if !derives(w, item.item_type, "Server.Items.SpellScroll") {
                // not a scroll
                if e.failed {
                    if e.lost_material {
                        return 1044043;
                    }
                    return 1044157;
                }
                return quality_message(e);
            }
            if e.failed {
                return 501630; // You fail to inscribe the scroll, and the scroll is ruined.
            }
            501629 // You inscribe the spell and put the scroll in your backpack.
        },
        ..base_vt()
    }
}

// ---- Masonry ----

fn masonry_vt() -> SystemVt {
    SystemVt {
        retains_color_from: |_, _, _| true,
        can_craft: |w, from, t, _ty| {
            if worn_out(w, t) {
                return 1044038; // You have worn out your tool!
            }
            let t = t.unwrap_or(ItemId(muo_primitives::Serial::ZERO));
            if !tool::check_tool(w, t, from) {
                return 1048146; // If you have a tool equipped, you must use that tool.
            }
            if !(player_flag(w, from, muo_content::player_mobile::PlayerFlag::MASONRY)
                && skills::base(w, from, SkillName::Carpentry) >= 100.0)
            {
                return 1044633; // You haven't learned stonecraft.
            }
            if !tool::check_accessible(w, t, from) {
                return 1044263; // The tool must be on your person to use.
            }
            0
        },
        ..base_vt()
    }
}

// ---- Tailoring ----

const TAILOR_COLORABLES: &[&str] = &[
    "Server.Items.GozaMatEastDeed",
    "Server.Items.GozaMatSouthDeed",
    "Server.Items.SquareGozaMatEastDeed",
    "Server.Items.SquareGozaMatSouthDeed",
    "Server.Items.BrocadeGozaMatEastDeed",
    "Server.Items.BrocadeGozaMatSouthDeed",
    "Server.Items.BrocadeSquareGozaMatEastDeed",
    "Server.Items.BrocadeSquareGozaMatSouthDeed",
];

fn tailoring_vt() -> SystemVt {
    SystemVt {
        chance_at_min: chance_half,
        retains_color_from: |_, item, ty| {
            if ty != "Server.Items.Cloth" && ty != "Server.Items.UncutCloth" {
                return false;
            }
            TAILOR_COLORABLES.contains(&item.item_type)
        },
        play_craft_effect: |w, from| play_sound(w, from, 0x248),
        ..base_vt()
    }
}

// ---- Tinkering ----

const TINKER_COLORABLES: &[&str] = &[
    "Server.Items.ForkLeft",
    "Server.Items.ForkRight",
    "Server.Items.SpoonLeft",
    "Server.Items.SpoonRight",
    "Server.Items.KnifeLeft",
    "Server.Items.KnifeRight",
    "Server.Items.Plate",
    "Server.Items.Goblet",
    "Server.Items.PewterMug",
    "Server.Items.KeyRing",
    "Server.Items.Candelabra",
    "Server.Items.Scales",
    "Server.Items.Key",
    "Server.Items.Globe",
    "Server.Items.Spyglass",
    "Server.Items.Lantern",
    "Server.Items.HeatingStand",
];

fn tinkering_vt() -> SystemVt {
    SystemVt {
        chance_at_min: |item| {
            if matches!(item.name_number(), 1044258 | 1046445) {
                0.5
            } else {
                0.0
            }
        },
        can_craft: |w, from, t, ty| {
            let n = can_craft_plain(w, from, t, ty);
            if n != 0 {
                return n;
            }
            if let Some(ty) = ty
                && (derives(w, ty, "Server.Factions.BaseFactionTrapDeed")
                    && ty != "Server.Factions.BaseFactionTrapDeed"
                    || ty == "Server.Factions.FactionTrapRemovalKit")
                && !super::state::faction_member(w, from)
            {
                return 1044573; // You have to be in a faction to do that.
            }
            0
        },
        retains_color_from: |w, item, ty| {
            if !muo_content::items::core_uotd(w)
                || !(derives(w, ty, "Server.Items.BaseIngot") && ty != "Server.Items.BaseIngot")
            {
                return false;
            }
            TINKER_COLORABLES.contains(&item.item_type)
        },
        consume_on_failure: |_w, _from, ty, _item| ty != "Server.Factions.Silver",
        ..base_vt()
    }
}
