//! `Repair`: the repair target of a craft tool or of a `RepairDeed`.

use std::rc::Rc;

use muo_content::craft as rc;
use muo_content::items::{armor, clothing, is_named, weapon};
use muo_content::mx;
use muo_primitives::{TimeSpan, utility};
use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{deps, ops};
use muo_server::skills::{self, SkillName};
use muo_server::target::{self, Target, TargetFlags, Targeted};

use super::item::{self as craft_item, ConsumeType};
use super::model::*;

/// Where the repair comes from: a tool, or a repair contract of a given skill level.
#[derive(Copy, Clone)]
pub enum Repairer {
    Tool(Option<ItemId>),
    Deed { item: ItemId, skill_level: f64 },
}

/// `Repair.Do(from, craftSystem, tool)`.
pub fn do_repair(w: &mut World, from: MobileId, sys: &Rc<CraftSystem>, tool: Option<ItemId>) {
    start(w, from, sys, Repairer::Tool(tool));
}

/// `Repair.Do(from, craftSystem, deed)`.
pub fn do_repair_deed(w: &mut World, from: MobileId, sys: &Rc<CraftSystem>, deed: ItemId) {
    let level = rc::hooks(w)
        .and_then(|h| h.repair_deed_level)
        .map_or(0.0, |f| f(w, deed));
    start(
        w,
        from,
        sys,
        Repairer::Deed {
            item: deed,
            skill_level: level,
        },
    );
}

fn start(w: &mut World, from: MobileId, sys: &Rc<CraftSystem>, who: Repairer) {
    let mut t = Target::new(w, 2, false, TargetFlags::NONE);
    let sys = sys.clone();
    t.handlers.on_target = Some(Rc::new(
        move |w: &mut World, from: MobileId, targeted: Targeted| {
            on_target(w, from, &sys, who, targeted);
        },
    ));
    let id = target::add_target(w, t);
    target::set_mobile_target(w, from, Some(id));
    deps::send_localized_message(w, from, 1044276); // Target an item to repair.
}

fn skill_level(
    w: &mut World,
    from: MobileId,
    who: Repairer,
    skill: SkillName,
    use_value: bool,
) -> f64 {
    match who {
        Repairer::Deed { skill_level, .. } => skill_level,
        Repairer::Tool(_) => {
            if use_value {
                skills::value(w, from, skill)
            } else {
                skills::base(w, from, skill)
            }
        }
    }
}

fn check_weaken(
    w: &mut World,
    from: MobileId,
    who: Repairer,
    skill: SkillName,
    cur: i32,
    max: i32,
) -> bool {
    let level = skill_level(w, from, who, skill, true);
    let chance = 40 + (max - cur) - (level / 10.0) as i32;
    chance > utility::random(&mut w.rng, 100)
}

fn repair_difficulty(cur: i32, max: i32) -> i32 {
    (max - cur) * 1250 / max.max(1) - 250
}

fn check_repair_difficulty(
    w: &mut World,
    from: MobileId,
    who: Repairer,
    skill: SkillName,
    cur: i32,
    max: i32,
) -> bool {
    let difficulty = f64::from(repair_difficulty(cur, max)) * 0.1;
    if let Repairer::Deed {
        skill_level: value, ..
    } = who
    {
        let min_skill = difficulty - 25.0;
        let max_skill = difficulty + 25.0;
        if value < min_skill {
            return false; // Too difficult
        }
        if value >= max_skill {
            return true; // No challenge
        }
        let chance = (value - min_skill) / (max_skill - min_skill);
        return chance >= utility::random_double(&mut w.rng);
    }
    mx::check_skill(w, from, skill, difficulty - 25.0, difficulty + 25.0)
}

fn is_any(ty: &str, w: &World, names: &[&str]) -> bool {
    super::item::resolve(w, ty).is_some_and(|t| names.iter().any(|n| is_named(t, n)))
}

fn special_clothing(w: &World, sys: &CraftSystem, ty: &str) -> bool {
    // Clothing repairable but not craftable
    sys.name == "Tailoring"
        && is_any(
            ty,
            w,
            &[
                "Server.Items.BearMask",
                "Server.Items.DeerMask",
                "Server.Items.TheMostKnowledgePerson",
                "Server.Items.TheRobeOfBritanniaAri",
                "Server.Items.EmbroideredOakLeafCloak",
            ],
        )
}

fn special_weapon(w: &World, sys: &CraftSystem, ty: &str) -> bool {
    // Weapons repairable but not craftable
    match sys.name {
        "Tinkering" => is_any(
            ty,
            w,
            &[
                "Server.Items.Cleaver",
                "Server.Items.Hatchet",
                "Server.Items.Pickaxe",
                "Server.Items.ButcherKnife",
                "Server.Items.SkinningKnife",
            ],
        ),
        "Carpentry" => is_any(
            ty,
            w,
            &[
                "Server.Items.Club",
                "Server.Items.BlackStaff",
                "Server.Items.MagicWand",
                "Server.Items.WildStaff",
            ],
        ),
        "Blacksmithy" => is_any(
            ty,
            w,
            &[
                "Server.Items.Pitchfork",
                "Server.Items.RadiantScimitar",
                "Server.Items.WarCleaver",
                "Server.Items.ElvenSpellblade",
                "Server.Items.AssassinSpike",
                "Server.Items.Leafblade",
                "Server.Items.RuneBlade",
                "Server.Items.ElvenMachete",
                "Server.Items.OrnateAxe",
                "Server.Items.DiamondMace",
            ],
        ),
        // TODO: Make these items craftable
        "BowFletching" => is_any(
            ty,
            w,
            &[
                "Server.Items.ElvenCompositeLongbow",
                "Server.Items.MagicalShortbow",
            ],
        ),
        _ => false,
    }
}

fn special_armor(w: &World, sys: &CraftSystem, ty: &str) -> bool {
    // Armor repairable but not craftable
    match sys.name {
        "Tailoring" => is_any(
            ty,
            w,
            &[
                "Server.Items.LeafTonlet",
                "Server.Items.LeafArms",
                "Server.Items.LeafChest",
                "Server.Items.LeafGloves",
                "Server.Items.LeafGorget",
                "Server.Items.LeafLegs",
                "Server.Items.HideChest",
                "Server.Items.HideGloves",
                "Server.Items.HideGorget",
                "Server.Items.HidePants",
                "Server.Items.HidePauldrons",
            ],
        ),
        "Carpentry" => is_any(
            ty,
            w,
            &[
                "Server.Items.WingedHelm",
                "Server.Items.RavenHelm",
                "Server.Items.VultureHelm",
                "Server.Items.WoodlandArms",
                "Server.Items.WoodlandChest",
                "Server.Items.WoodlandGloves",
                "Server.Items.WoodlandGorget",
                "Server.Items.WoodlandLegs",
            ],
        ),
        "Blacksmithy" => is_any(
            ty,
            w,
            &[
                "Server.Items.Circlet",
                "Server.Items.RoyalCirclet",
                "Server.Items.GemmedCirclet",
            ],
        ),
        _ => false,
    }
}

/// `SearchForSubclass(type)`: an entry whose class is `ty` or a base of it.
pub(super) fn craftable_subclass(w: &World, sys: &CraftSystem, ty: &str) -> bool {
    sys.items
        .iter()
        .any(|c| c.item_type == ty || derives_from(w, ty, c.item_type))
}

fn derives_from(w: &World, ty: &str, base: &str) -> bool {
    super::item::derives(w, ty, base)
}

/// Which kind of equipment a repair target is: its data accessors.
#[derive(Copy, Clone, Eq, PartialEq)]
enum Kind {
    Weapon,
    Armor,
    Clothing,
}

fn kind_of(w: &World, item: ItemId) -> Option<Kind> {
    let ty = w.item(item)?.ty;
    if ty.is(&weapon::BASE_WEAPON) {
        Some(Kind::Weapon)
    } else if ty.is(&armor::BASE_ARMOR) {
        Some(Kind::Armor)
    } else if ty.is(&clothing::BASE_CLOTHING) {
        Some(Kind::Clothing)
    } else {
        None
    }
}

fn hits(w: &World, item: ItemId, k: Kind) -> (i32, i32) {
    match k {
        Kind::Weapon => weapon::data(w, item).map_or((0, 0), |d| (d.hit_points, d.max_hit_points)),
        Kind::Armor => armor::data(w, item).map_or((0, 0), |d| (d.hit_points, d.max_hit_points)),
        Kind::Clothing => {
            clothing::data(w, item).map_or((0, 0), |d| (d.hit_points, d.max_hit_points))
        }
    }
}

fn set_hits(w: &mut World, item: ItemId, k: Kind, hp: i32, max: i32) {
    // MaxHitPoints first, so that HitPoints is clamped by the new maximum.
    match k {
        Kind::Weapon => {
            weapon::set_max_hit_points(w, item, max);
            weapon::set_hit_points(w, item, hp);
        }
        Kind::Armor => {
            armor::set_max_hit_points(w, item, max);
            armor::set_hit_points(w, item, hp);
        }
        Kind::Clothing => {
            clothing::set_max_hit_points(w, item, max);
            clothing::set_hit_points(w, item, hp);
        }
    }
}

fn on_target(
    w: &mut World,
    from: MobileId,
    sys: &Rc<CraftSystem>,
    who: Repairer,
    targeted: Targeted,
) {
    let tool = match who {
        Repairer::Tool(t) => t,
        Repairer::Deed { .. } => None,
    };
    let using_deed = matches!(who, Repairer::Deed { .. });
    if let Repairer::Deed { item, .. } = who {
        // m_Deed.Check(from)
        let ok = rc::hooks(w)
            .and_then(|h| h.repair_deed_check)
            .is_none_or(|f| f(w, item, from));
        if !ok {
            return;
        }
    }
    let mut to_delete = false;
    let target_type = match targeted {
        Targeted::Entity(EntityId::Item(i)) => w.item(i).map(|x| x.ty.full_name),
        Targeted::Entity(EntityId::Mobile(m)) => w.mobile(m).map(|x| x.ty.full_name),
        _ => None,
    };
    let number: i32;

    if (sys.vt.can_craft)(w, from, tool, target_type) == 1044267 {
        number = 1044282; // You must be near a forge and and anvil to repair items. * Yes, there are two and's *
    } else if sys.name == "Tinkering"
        && let Targeted::Entity(EntityId::Mobile(g)) = targeted
        && muo_content::factory::mobile_is(w, g, "Server.Mobiles.Golem")
    {
        (number, to_delete) = repair_golem(w, from, g, who);
    } else if let Targeted::Entity(EntityId::Item(item)) = targeted
        && let Some(kind) = kind_of(w, item)
    {
        (number, to_delete) = repair_equipment(w, from, sys, who, item, kind, using_deed);
    } else if let Targeted::Entity(EntityId::Item(item)) = targeted
        && !using_deed
        && muo_content::factory::item_is(w, item, "Server.Items.BlankScroll")
    {
        let skill = sys.main_skill;
        if skills::value(w, from, skill) >= 50.0 {
            muo_server::item::base::consume(w, item, 1);
            let level = skills::value(w, from, skill);
            let kind = repair_type_for(sys.name);
            if let Some(f) = rc::hooks(w).and_then(|h| h.repair_deed_create)
                && let Some(deed) = f(w, kind, level, from)
            {
                muo_server::mobile::part2::items::add_to_backpack(w, from, deed);
            }
            number = 500442; // You create the item and put it in your backpack.
        } else {
            number = 1047005; // You must be at least apprentice level to create a repair service contract.
        }
    } else if let Targeted::Entity(EntityId::Item(_)) = targeted {
        number = if using_deed { 1061136 } else { 1044277 }; // That item cannot be repaired. // .. with this type of repair contract.
    } else {
        number = 500426; // You can't repair that.
    }

    if !using_deed {
        if super::state::t2a_enabled(w) {
            deps::send_localized_message(w, from, number);
            craft_item::show_craft_menu(w, from, sys, tool, None);
        } else {
            craft_item::show_craft_menu(w, from, sys, tool, Some(Td::of_number(number)));
        }
    } else {
        deps::send_localized_message(w, from, number);
        if to_delete
            && let Repairer::Deed { item, .. } = who
            && let Some(t) = w.item(item).map(|i| i.ty)
        {
            (t.delete)(w, item);
        }
    }
}

/// `RepairDeed.GetTypeFor(system)`: the `RepairSkillType` index.
pub fn repair_type_for(system: &str) -> i32 {
    match system {
        "Blacksmithy" => 0,
        "Tailoring" => 1,
        "Tinkering" => 2,
        "Carpentry" => 3,
        "BowFletching" => 4,
        _ => 0,
    }
}

fn repair_golem(w: &mut World, from: MobileId, g: MobileId, who: Repairer) -> (i32, bool) {
    let (hits, hits_max) = w
        .mobile(g)
        .map_or((0, 0), |m| (m.hits, (m.ty.hits_max)(w, g)));
    let mut damage = hits_max - hits;
    let dead_bonded = w.mobile(g).is_some_and(|m| (m.ty.is_dead_bonded_pet)(w, g));
    if dead_bonded {
        return (500426, false); // You can't repair that.
    }
    if damage <= 0 {
        return (500423, false); // That is already in full repair.
    }
    let skill_value = match who {
        Repairer::Deed { skill_level, .. } => skill_level,
        Repairer::Tool(_) => skills::value(w, from, SkillName::Tinkering),
    };
    if skill_value < 60.0 {
        return (1044153, false); // You don't have the required skills to attempt this item.
    }
    if !muo_server::mobile::part1::combat::can_begin_action(w, from, "Golem") {
        return (501789, false); // You must wait before trying again.
    }
    if damage > (skill_value * 0.3) as i32 {
        damage = (skill_value * 0.3) as i32;
    }
    damage += 30;
    if !mx::check_skill(w, from, SkillName::Tinkering, 0.0, 100.0) {
        damage /= 2;
    }
    let Some(pack) = deps::find_item_on_layer(w, from, muo_server::layer::Layer::Backpack) else {
        return (1044037, false); // You do not have sufficient metal to make that.
    };
    let Some(ingot) = craft_item::resolve(w, "Server.Items.IronIngot") else {
        return (1044037, false);
    };
    let v = muo_server::item::container::consume_up_to(w, pack, ingot, (damage + 4) / 5, true);
    if v > 0 {
        muo_server::mobile::part1::props_b::set_hits(w, g, hits + v * 5);
        muo_server::mobile::part1::combat::begin_action(w, from, "Golem");
        muo_server::timer::start(w, TimeSpan::from_secs(12), TimeSpan::ZERO, 1, move |w| {
            muo_server::mobile::part1::combat::end_action(w, from, "Golem");
        });
        (1044279, true) // You repair the item.
    } else {
        (1044037, false) // You do not have sufficient metal to make that.
    }
}

fn repair_equipment(
    w: &mut World,
    from: MobileId,
    sys: &Rc<CraftSystem>,
    who: Repairer,
    item: ItemId,
    kind: Kind,
    using_deed: bool,
) -> (i32, bool) {
    let skill = sys.main_skill;
    let ty = w.item(item).map_or("", |i| i.ty.full_name);
    let mut to_weaken = 0;
    if muo_content::core::aos(w) {
        to_weaken = 1;
    } else if skill != SkillName::Tailoring {
        let level = skill_level(w, from, who, skill, false);
        to_weaken = if level >= 90.0 {
            1
        } else if level >= 70.0 {
            2
        } else {
            3
        };
    }

    let repairable = craftable_subclass(w, sys, ty)
        || match kind {
            Kind::Weapon => special_weapon(w, sys, ty),
            Kind::Armor => special_armor(w, sys, ty),
            Kind::Clothing => {
                special_clothing(w, sys, ty)
                    || is_any(
                        ty,
                        w,
                        &["Server.Items.TribalMask", "Server.Items.HornedTribalMask"],
                    )
            }
        };
    if !repairable {
        return (if using_deed { 1061136 } else { 1044277 }, false); // That item cannot be repaired.
    }

    let pack = deps::find_item_on_layer(w, from, muo_server::layer::Layer::Backpack);
    let in_pack = pack.is_some_and(|p| ops::is_child_of(w, item, Some(EntityId::Item(p))));
    let held = w
        .item(item)
        .is_some_and(|i| i.parent == Some(EntityId::Mobile(from)));
    if !in_pack && (!muo_content::core::ml(w) || !held) {
        return (1044275, false); // The item must be in your backpack to repair it.
    }
    if kind == Kind::Weapon
        && !muo_content::core::aos(w)
        && weapon::data(w, item).is_some_and(|d| d.poison_charges != 0)
    {
        return (1005012, false); // You cannot repair an item while a caustic substance is on it.
    }
    let (hp, max) = hits(w, item, kind);
    if max <= 0 || hp == max {
        return (1044281, false); // That item is in full repair
    }
    if max <= to_weaken {
        return (1044278, false); // That item has been repaired many times, and will break if repairs are attempted again.
    }

    let (mut hp, mut max) = (hp, max);
    if check_weaken(w, from, who, skill, hp, max) {
        max -= to_weaken;
        hp = (hp - to_weaken).max(0);
        set_hits(w, item, kind, hp, max);
    }
    let number;
    if check_repair_difficulty(w, from, who, skill, hp, max) {
        number = 1044279; // You repair the item.
        (sys.vt.play_craft_effect)(w, from);
        set_hits(w, item, kind, max, max);
    } else {
        number = if using_deed { 1061137 } else { 1044280 }; // You fail to repair the item. [And the contract is destroyed]
        (sys.vt.play_craft_effect)(w, from);
    }
    let _ = ConsumeType::All;
    (number, true)
}
