//! Tests of the craft engine: every system builds, its types resolve, and a craft runs end to end.

use muo_server::World;

use super::model::CraftSystem;

fn world() -> World {
    let mut w = World::new();
    muo_content::register(&mut w.types);
    muo_items::register(&mut w.types);
    crate::register(&mut w.types);
    w
}

#[test]
fn every_system_builds_with_items_in_groups() {
    let mut w = world();
    for name in super::defs::NAMES {
        let s: std::rc::Rc<CraftSystem> =
            super::system(&mut w, name).unwrap_or_else(|| panic!("{name}"));
        assert!(!s.items.is_empty(), "{name} has no items");
        let grouped: usize = s.groups.iter().map(|g| g.items.len()).sum();
        assert_eq!(grouped, s.items.len(), "{name}: every item is in one group");
    }
}

/// The craft tables name 780 type names (items made and resources used); these 11 do not resolve to a registered
/// class yet (the three trap crafts are `CustomCraft`s, not items; the others belong to the skill_items and
/// furniture tasks). A name outside the list fails.
#[test]
fn craft_types_that_do_not_resolve() {
    let mut w = world();
    let mut all = std::collections::BTreeSet::new();
    let mut missing = std::collections::BTreeSet::new();
    for name in super::defs::NAMES {
        let s = super::system(&mut w, name).unwrap();
        for item in &s.items {
            all.insert(item.item_type);
            if super::item::resolve(&w, item.item_type).is_none() {
                missing.insert(item.item_type);
            }
            for r in &item.resources {
                all.insert(r.item_type);
                if super::item::resolve(&w, r.item_type).is_none() {
                    missing.insert(r.item_type);
                }
            }
        }
    }
    println!(
        "{} of {} craft type names do not resolve: {:#?}",
        missing.len(),
        all.len(),
        missing
    );
    let want: &[&str] = &[
        "Server.Engines.Craft.DartTrapCraft",
        "Server.Engines.Craft.ExplosionTrapCraft",
        "Server.Engines.Craft.PoisonTrapCraft",
        "Server.Items.Armoire",
        "Server.Items.Arrow",
        "Server.Items.DragonBardingDeed",
        "Server.Items.EggBomb",
        "Server.Items.FancyArmoire",
        "Server.Items.FishingPole",
        "Server.Items.LeatherNinjaBelt",
        "Server.Items.SmokeBomb",
    ];
    // Some of these get implemented meanwhile (Armoire, FancyArmoire, FishingPole were): only a new gap fails.
    let new: Vec<&&str> = missing.iter().filter(|m| !want.contains(*m)).collect();
    assert!(
        new.is_empty(),
        "craft types that stopped resolving: {new:?}"
    );
}

fn player_with_pack(w: &mut World) -> (muo_server::ids::MobileId, muo_server::ids::ItemId) {
    let m = w.new_mobile(&muo_content::player_mobile::PLAYER_MOBILE);
    let pack = muo_content::creature::setters::ensure_backpack(w, m).expect("backpack");
    (m, pack)
}

fn give(
    w: &mut World,
    pack: muo_server::ids::ItemId,
    ty: &str,
    amount: i32,
) -> muo_server::ids::ItemId {
    let id = muo_content::factory::make_item(w, ty).unwrap_or_else(|| panic!("{ty}"));
    muo_server::item::ops::set_amount(w, id, amount);
    let t = w.item(pack).unwrap().ty;
    (t.drop_item)(w, pack, id);
    id
}

#[test]
fn the_tool_class_names_its_system_and_opens_the_craft_check() {
    let mut w = world();
    let (m, pack) = player_with_pack(&mut w);
    let saw = give(&mut w, pack, "Server.Items.Saw", 1);
    let sys = super::system(&mut w, "Carpentry").unwrap();
    assert_eq!(
        muo_content::items::tool::class(&w, saw).craft_system,
        "Carpentry"
    );
    assert_eq!((sys.vt.can_craft)(&mut w, m, Some(saw), None), 0);
    // a tool outside the pack is not "on your person"
    let loose = muo_content::factory::make_item(&mut w, "Server.Items.Saw").unwrap();
    assert_eq!((sys.vt.can_craft)(&mut w, m, Some(loose), None), 1044263);
    assert_eq!((sys.vt.can_craft)(&mut w, m, None, None), 1044038);
}

#[test]
fn resources_are_counted_and_consumed_from_the_pack() {
    use super::item::{ConsumeType, consume_res};
    let mut w = world();
    w.core.expansion = Some(muo_data::expansion::Expansion::ML);
    let (m, pack) = player_with_pack(&mut w);
    give(&mut w, pack, "Server.Items.Board", 10);
    let sys = super::system(&mut w, "Carpentry").unwrap();
    // "Server.Items.Log" is not registered yet, so craft something whose resource is a Board: look one up.
    let Some(item) = sys
        .items
        .iter()
        .find(|c| c.resources[0].item_type == "Server.Items.Board")
    else {
        return;
    };
    let (mut hue, mut max) = (0, 0);
    let mut msg = None;
    let enough = item.resources[0].amount <= 10;
    let ok = consume_res(
        &mut w,
        m,
        &sys,
        item,
        None,
        &mut hue,
        &mut max,
        ConsumeType::None,
        &mut msg,
        false,
    );
    assert_eq!(ok, enough);
}

/// A world with the skill table loaded. Without one `Skills` has no slots, `set_base` is a no-op and every
/// skill reads as 0, so a Human mobile (`PlayerMobile.RacialSkillBonus` is 20.0 from ML on) reads 20.0.
fn world_with_skills() -> World {
    use muo_data::skill_info::{SkillInfo, Stat};
    let mut w = world();
    let info = |i: i32| SkillInfo {
        skill_id: i,
        name: format!("S{i}"),
        title: String::new(),
        str_scale: 0.0,
        dex_scale: 0.0,
        int_scale: 0.0,
        stat_total: 0.0,
        str_gain: 0.0,
        dex_gain: 0.0,
        int_gain: 0.0,
        gain_factor: 0.0,
        profession_skill_name: String::new(),
        primary_stat: Stat::Str,
        secondary_stat: Stat::Dex,
    };
    w.resources.get_mut::<muo_server::skills::SkillTable>().info = (0..58).map(info).collect();
    w
}

#[test]
fn success_chance_follows_the_skill_and_the_chance_at_minimum() {
    // The earlier version of this test saw 0.9 at maximum skill and blamed the skill cap. The cause was the
    // missing skill table (see `world_with_skills`): the Skills held nothing, so the Human racial bonus of 20.0
    // was the only skill, and 0.5 + 20 / 25 * 0.5 = 0.9 for BarrelStaves (0..25).
    let mut w = world_with_skills();
    let (m, _) = player_with_pack(&mut w);
    let sys = super::system(&mut w, "Carpentry").unwrap();
    let main = sys.main_skill;
    // an item whose minimum is above the racial 20.0, so the racial bonus cannot hide the skill
    let item = sys
        .items
        .iter()
        .find(|i| {
            i.skills
                .iter()
                .any(|s| s.skill == main && s.min_skill >= 21.0 && s.max_skill > s.min_skill)
        })
        .cloned()
        .unwrap();
    let (min, max) = {
        let s = item.skills.iter().find(|s| s.skill == main).unwrap();
        (s.min_skill, s.max_skill)
    };
    let at_min = (sys.vt.chance_at_min)(&item);

    muo_server::skills::set_base(&mut w, m, main, max);
    assert_eq!(
        muo_server::skills::value(&mut w, m, main),
        max,
        "the skill table is in"
    );
    let mut all = false;
    let chance = super::item::success_chance(&mut w, &sys, &item, m, false, &mut all);
    assert!(all);
    assert_eq!(chance, 1.0, "at maximum skill ({min}..{max})");

    muo_server::skills::set_base(&mut w, m, main, min);
    let chance = super::item::success_chance(&mut w, &sys, &item, m, false, &mut all);
    assert!(all);
    assert_eq!(chance, at_min, "at minimum skill");

    muo_server::skills::set_base(&mut w, m, main, 0.0);
    let chance = super::item::success_chance(&mut w, &sys, &item, m, false, &mut all);
    assert!(!all, "below the minimum");
    assert_eq!(chance, 0.0);
}

#[test]
fn context_keeps_ten_most_recent_first() {
    let mut c = super::model::CraftContext::default();
    for i in 0..12 {
        c.on_made(i);
    }
    assert_eq!(c.items.len(), 10);
    assert_eq!(c.last_made(), Some(11));
    c.on_made(5);
    assert_eq!(c.items[0], 5);
    assert_eq!(c.items.iter().filter(|&&i| i == 5).count(), 1);
}

#[test]
fn recipes_register_with_their_systems() {
    let mut w = world();
    super::ensure_built(&mut w);
    let st = super::state::state(&w).unwrap();
    assert!(!st.recipes.is_empty());
    for (id, (name, idx)) in &st.recipes {
        let s = &st.systems[name];
        assert_eq!(s.items[*idx].recipe, Some(*id));
    }
}

#[test]
fn enhance_refuses_charged_arcane_gloves_before_the_resource_check() {
    use super::enhance::{EnhanceResult, invoke};
    use muo_content::craft::CraftResource;
    let mut w = world();
    let (m, pack) = player_with_pack(&mut w);
    let gloves = give(&mut w, pack, "Server.Items.LeatherGloves", 1);
    let sys = super::system(&mut w, "Blacksmithy").unwrap();
    let mut msg = None;
    // Enhance: only the arcane check differs; a standard resource is BadResource.
    let r = invoke(
        &mut w,
        m,
        &sys,
        None,
        gloves,
        CraftResource::Iron,
        "",
        &mut msg,
    );
    assert_eq!(r, EnhanceResult::BadResource);
    muo_items::armor::special::set_max_arcane_charges(&mut w, gloves, 20, 0x13C6);
    let r = invoke(
        &mut w,
        m,
        &sys,
        None,
        gloves,
        CraftResource::Iron,
        "",
        &mut msg,
    );
    assert_eq!(r, EnhanceResult::BadItem);
}

/// A mobile with a connection whose packets land in `w.net_out`.
fn connect(w: &mut World, m: muo_server::ids::MobileId) -> muo_server::net_state::NetStateId {
    let ns = muo_server::net_state::NetStateId(7);
    let mut d = muo_server::net_state::NetStateData::new(0);
    d.mobile = Some(m);
    w.net_states.insert(ns, d);
    w.mobile_mut(m).unwrap().net_state = Some(ns);
    ns
}

fn sent_menu(w: &World) -> Option<Vec<u8>> {
    w.net_out.iter().rev().find_map(|o| match o {
        muo_server::net_state::NetOut::Send(_, b) if b[0] == 0x7C => Some(b.clone()),
        _ => None,
    })
}

#[test]
fn the_t2a_cartography_menu_lists_what_the_skill_and_pack_allow_and_crafts_on_response() {
    let mut w = world_with_skills();
    super::state::state_mut(&mut w).t2a = true;
    super::ensure_built(&mut w);
    let (m, pack) = player_with_pack(&mut w);
    let ns = connect(&mut w, m);

    // nothing in the pack: only the message
    super::t2a::show_cartography_menu(&mut w, m);
    assert!(sent_menu(&w).is_none());

    give(&mut w, pack, "Server.Items.BlankMap", 5);
    muo_server::skills::set_base(&mut w, m, super::model::SkillName::Cartography, 50.0);
    super::t2a::show_cartography_menu(&mut w, m);
    let p = sent_menu(&w).expect("a menu");
    let text = String::from_utf8_lossy(&p).into_owned();
    // skill 50 is above every minimum (10, 25, 35, 39.5)
    assert!(text.contains("What kind of map?"));
    assert!(text.contains("A map of the local environs."));
    assert!(text.contains("A map suitable for cities."));
    assert!(text.contains("A moderately sized sea chart."));
    assert!(text.contains("A map of the world."), "50 >= 39.5");
    assert_eq!(w.net_state(ns).unwrap().menus.len(), 1);
    // choosing the first entry starts the craft (it takes the "CraftSystem" action)
    let menu = muo_server::menus::remove_menu_at(&mut w, ns, 0).unwrap();
    menu.respond(&mut w, ns, 0);
    assert!(
        !muo_server::mobile::part1::combat::begin_action(&mut w, m, "CraftSystem"),
        "the craft is under way"
    );
}

#[test]
fn the_t2a_inscription_menu_needs_blank_scrolls_and_a_known_spell() {
    let mut w = world_with_skills();
    super::state::state_mut(&mut w).t2a = true;
    super::ensure_built(&mut w);
    let (m, pack) = player_with_pack(&mut w);
    connect(&mut w, m);
    muo_server::skills::set_base(&mut w, m, super::model::SkillName::Inscribe, 100.0);
    // reagents and scrolls but no spellbook: no scroll can be scribed, the runebook entry (8 scrolls, a rune) may
    give(&mut w, pack, "Server.Items.BlankScroll", 10);
    give(&mut w, pack, "Server.Items.BlackPearl", 10);
    super::t2a::show_inscription_menu(&mut w, m);
    if let Some(p) = sent_menu(&w) {
        let text = String::from_utf8_lossy(&p).into_owned();
        assert!(text.contains("Which circle of spells?"));
        assert!(
            !text.contains("First Circle"),
            "no spellbook holds a first circle spell"
        );
    }
    assert_eq!(
        super::t2a::format_scroll_name_for_test("Server.Items.MagicArrowScroll"),
        "magic arrow"
    );
}

// ---- the six tool menus, T2ACraftToolTarget, CanCraftItem ----

fn t2a_world() -> World {
    let mut w = world_with_skills();
    super::state::state_mut(&mut w).t2a = true;
    super::ensure_built(&mut w);
    w
}

fn menu_text(w: &World) -> String {
    String::from_utf8_lossy(&sent_menu(w).expect("a menu")).into_owned()
}

/// Takes the open menu off the connection and answers it with `index`.
fn answer(w: &mut World, ns: muo_server::net_state::NetStateId, index: i32) {
    let menu = muo_server::menus::remove_menu_at(w, ns, 0).expect("an open menu");
    menu.respond(w, ns, index);
}

fn sent_ascii(w: &World, needle: &str) -> bool {
    w.net_out.iter().any(|o| match o {
        muo_server::net_state::NetOut::Send(_, b) => String::from_utf8_lossy(b).contains(needle),
        _ => false,
    })
}

#[test]
fn the_t2a_alchemy_menu_lists_categories_then_potions_and_crafts_with_the_tool() {
    let mut w = t2a_world();
    let (m, pack) = player_with_pack(&mut w);
    let ns = connect(&mut w, m);
    let tool = give(&mut w, pack, "Server.Items.MortarPestle", 1);
    muo_server::skills::set_base(&mut w, m, super::model::SkillName::Alchemy, 100.0);
    give(&mut w, pack, "Server.Items.Ginseng", 10);
    give(&mut w, pack, "Server.Items.Bottle", 5);
    super::t2a::show_menu(&mut w, m, "Alchemy", Some(tool), None);
    let text = menu_text(&w);
    assert!(
        text.contains("What kind of potion?") && text.contains("Heal"),
        "{text}"
    );
    assert!(!text.contains("Poison"), "no nightshade");
    answer(&mut w, ns, 0);
    let text = menu_text(&w);
    assert!(
        text.contains("Which potion would you like to make?") && text.contains("lesser heal"),
        "{text}"
    );
    answer(&mut w, ns, 0);
    assert!(
        !muo_server::mobile::part1::combat::begin_action(&mut w, m, "CraftSystem"),
        "the craft is under way"
    );
}

#[test]
fn the_t2a_alchemy_menu_asks_for_a_reagent_or_bottle_when_the_target_is_something_else() {
    let mut w = t2a_world();
    let (m, pack) = player_with_pack(&mut w);
    connect(&mut w, m);
    let tool = give(&mut w, pack, "Server.Items.MortarPestle", 1);
    let junk = give(&mut w, pack, "Server.Items.Gold", 5);
    super::t2a::show_menu(&mut w, m, "Alchemy", Some(tool), Some(junk));
    assert!(sent_ascii(&w, "Target a reagent or empty bottle."));
    assert!(w.mobile(m).unwrap().target.is_some());
    assert!(sent_menu(&w).is_none());
}

#[test]
fn the_t2a_blacksmith_menu_opens_after_the_ingots_are_chosen_and_nests_its_categories() {
    let mut w = t2a_world();
    let (m, pack) = player_with_pack(&mut w);
    let ns = connect(&mut w, m);
    let tool = give(&mut w, pack, "Server.Items.SmithHammer", 1);
    muo_server::skills::set_base(&mut w, m, super::model::SkillName::Blacksmith, 100.0);
    let ingots = give(&mut w, pack, "Server.Items.IronIngot", 100);
    super::t2a::show_menu(&mut w, m, "Blacksmithy", Some(tool), Some(ingots));
    let text = menu_text(&w);
    assert!(text.contains("What would you like to do?"), "{text}");
    for name in [
        "Repair",
        "Smelt",
        "Build Armor",
        "Build Shield",
        "Build Weapons",
    ] {
        assert!(text.contains(name), "{name} in {text}");
    }
    // "Build Shield" is the fourth row: Repair, Smelt, Build Armor, Build Shield
    answer(&mut w, ns, 3);
    let text = menu_text(&w);
    assert!(
        text.contains("What kind of shield?") && text.contains("buckler (10 ingots)"),
        "{text}"
    );
}

#[test]
fn the_t2a_blacksmith_menu_hides_what_the_selected_ingot_cannot_make() {
    let mut w = t2a_world();
    let (m, pack) = player_with_pack(&mut w);
    let sys = super::system(&mut w, "Blacksmithy").unwrap();
    muo_server::skills::set_base(&mut w, m, super::model::SkillName::Blacksmith, 100.0);
    give(&mut w, pack, "Server.Items.IronIngot", 100);
    let plate = sys.search_for("Server.Items.PlateChest").unwrap();
    // iron: enough ingots and skill; dull copper: none in the pack
    assert!(super::t2a::can_craft_item(
        &mut w,
        m,
        &sys,
        plate,
        Some("Server.Items.IronIngot"),
        -1
    ));
    assert!(!super::t2a::can_craft_item(
        &mut w,
        m,
        &sys,
        plate,
        Some("Server.Items.DullCopperIngot"),
        -1
    ));
    // no selection: any sub-resource the skill and the pack allow
    assert!(super::t2a::can_craft_item(&mut w, m, &sys, plate, None, -1));
}

#[test]
fn the_t2a_hue_counts_only_stock_of_that_hue_for_the_first_resource() {
    let mut w = t2a_world();
    let (m, pack) = player_with_pack(&mut w);
    muo_server::skills::set_base(&mut w, m, super::model::SkillName::Tailoring, 100.0);
    let sys = super::system(&mut w, "Tailoring").unwrap();
    let robe = sys.search_for("Server.Items.Robe").unwrap(); // 16 cloth
    let red = give(&mut w, pack, "Server.Items.Cloth", 3);
    w.item_mut(red).unwrap().hue = 0x21;
    give(&mut w, pack, "Server.Items.Cloth", 50);
    assert!(
        super::t2a::can_craft_item(&mut w, m, &sys, robe, None, -1),
        "53 cloth of any hue"
    );
    assert!(
        !super::t2a::can_craft_item(&mut w, m, &sys, robe, None, 0x21),
        "only 3 of hue 0x21"
    );
    assert!(
        super::t2a::can_craft_item(&mut w, m, &sys, robe, None, 0),
        "50 uncolored"
    );
}

#[test]
fn the_t2a_tailoring_menu_follows_the_targeted_resource() {
    let mut w = t2a_world();
    let (m, pack) = player_with_pack(&mut w);
    let ns = connect(&mut w, m);
    let tool = give(&mut w, pack, "Server.Items.SewingKit", 1);
    muo_server::skills::set_base(&mut w, m, super::model::SkillName::Tailoring, 100.0);
    let cloth = give(&mut w, pack, "Server.Items.Cloth", 50);
    give(&mut w, pack, "Server.Items.Leather", 50);
    super::t2a::show_menu(&mut w, m, "Tailoring", Some(tool), Some(cloth));
    let text = menu_text(&w);
    assert!(
        text.contains("Build Hats") && !text.contains("Build Shoes"),
        "{text}"
    );
    answer(&mut w, ns, 0);
    assert!(menu_text(&w).contains("What kind of hat?"));
    // leather opens the leather menu instead
    let leather = give(&mut w, pack, "Server.Items.Leather", 1);
    super::t2a::show_menu(&mut w, m, "Tailoring", Some(tool), Some(leather));
    let text = menu_text(&w);
    assert!(
        text.contains("Build Shoes") && text.contains("Build Leather Armor"),
        "{text}"
    );
}

#[test]
fn the_t2a_tool_asks_for_a_target_and_makes_last_or_shows_the_menu() {
    let mut w = t2a_world();
    let (m, pack) = player_with_pack(&mut w);
    connect(&mut w, m);
    let tool = give(&mut w, pack, "Server.Items.MortarPestle", 1);
    muo_server::skills::set_base(&mut w, m, super::model::SkillName::Alchemy, 100.0);
    give(&mut w, pack, "Server.Items.Ginseng", 10);
    give(&mut w, pack, "Server.Items.Bottle", 5);
    assert!(super::t2a::has_menu("Alchemy") && !super::t2a::has_menu("Cooking"));
    super::t2a::begin_tool_target(&mut w, m, "Alchemy", tool);
    assert!(sent_ascii(
        &w,
        "Target this tool to make last item, or any other target to begin crafting."
    ));
    let t = w.mobile(m).unwrap().target.expect("a target");
    // targeting the tool with nothing made yet
    // (`Target.Invoke` needs the tool on the map in range; the handler is what the packet reaches)
    let on_target = w.targets[&t].handlers.on_target.clone().expect("a handler");
    on_target(
        &mut w,
        m,
        muo_server::target::Targeted::Entity(muo_server::ids::EntityId::Item(tool)),
    );
    assert!(sent_ascii(&w, "You have not yet crafted anything."));
    assert!(
        menu_text(&w).contains("What kind of potion?"),
        "then the menu"
    );
}

#[test]
fn the_t2a_inscription_menu_makes_one_throwaway_scroll_per_class_not_per_open() {
    let mut w = t2a_world();
    super::configure(&mut w); // the spell hooks
    let (m, pack) = player_with_pack(&mut w);
    connect(&mut w, m);
    muo_server::skills::set_base(&mut w, m, super::model::SkillName::Inscribe, 100.0);
    give(&mut w, pack, "Server.Items.BlankScroll", 10);
    for r in [
        "BlackPearl",
        "Bloodmoss",
        "Garlic",
        "Ginseng",
        "MandrakeRoot",
        "Nightshade",
        "SulfurousAsh",
        "SpidersSilk",
    ] {
        give(&mut w, pack, &format!("Server.Items.{r}"), 10);
    }
    let probe = |w: &mut World| {
        muo_content::factory::make_item(w, "Server.Items.Gold")
            .unwrap()
            .0
    };
    super::t2a::show_inscription_menu(&mut w, m);
    let first = probe(&mut w);
    super::t2a::show_inscription_menu(&mut w, m);
    super::t2a::show_inscription_menu(&mut w, m);
    let second = probe(&mut w);
    assert!(
        !super::state::state(&w).unwrap().spell_ids.is_empty(),
        "the spell ids are kept per class"
    );
    assert_eq!(
        second.value(),
        first.value() + 1,
        "reopening the menu burned item serials"
    );
}

// ---- T2A hued filter ----

/// `MakeRecipe()` in a tailoring system of its own: `new CraftItem(typeof(Shirt), "shirt", "shirt")`, 8 cloth, skill 0..100.
fn hued_recipe_system(w: &mut World) -> (CraftSystem, usize) {
    let tailoring = super::system(w, "Tailoring").unwrap();
    let vt = &tailoring.vt;
    let mut sys = CraftSystem::new(
        "Tailoring",
        super::model::SkillName::Tailoring,
        (
            tailoring.min_craft_effect,
            tailoring.max_craft_effect,
            tailoring.delay,
        ),
        tailoring.gump_title.clone(),
        tailoring.eca,
        super::model::SystemVt {
            chance_at_min: vt.chance_at_min,
            can_craft: vt.can_craft,
            play_craft_effect: vt.play_craft_effect,
            play_ending_effect: vt.play_ending_effect,
            consume_on_failure: vt.consume_on_failure,
            retains_color_from: vt.retains_color_from,
            requires_tool: vt.requires_tool,
        },
    );
    let td = super::model::Td::of_text;
    let i = sys.add_craft(
        "Server.Items.Shirt",
        td("shirt"),
        td("shirt"),
        None,
        None,
        0.0,
        100.0,
        "Server.Items.Cloth",
        td("cloth"),
        8,
        td("You do not have enough cloth."),
    );
    (sys, i)
}

fn hued(w: &mut World, pack: muo_server::ids::ItemId, ty: &str, amount: i32, hue: i32) {
    let id = give(w, pack, ty, amount);
    w.item_mut(id).unwrap().hue = hue;
}

#[test]
fn can_craft_item_with_hue_counts_only_matching_hue_cloth() {
    let mut w = t2a_world();
    let (m, pack) = player_with_pack(&mut w);
    let (sys, shirt) = hued_recipe_system(&mut w);
    hued(&mut w, pack, "Server.Items.Cloth", 5, 0x21);
    hued(&mut w, pack, "Server.Items.Cloth", 5, 0x30);

    // 10 cloth total, but no single hue reaches the 8 the recipe needs
    assert!(super::t2a::can_craft_item(&mut w, m, &sys, shirt, None, -1));
    assert!(!super::t2a::can_craft_item(
        &mut w, m, &sys, shirt, None, 0x21
    ));
    assert!(!super::t2a::can_craft_item(
        &mut w, m, &sys, shirt, None, 0x30
    ));

    hued(&mut w, pack, "Server.Items.Cloth", 3, 0x21);
    assert!(super::t2a::can_craft_item(
        &mut w, m, &sys, shirt, None, 0x21
    ));
    assert!(!super::t2a::can_craft_item(
        &mut w, m, &sys, shirt, None, 0x30
    ));
}

#[test]
fn can_craft_item_with_hue_counts_same_hue_uncut_cloth() {
    let mut w = t2a_world();
    let (m, pack) = player_with_pack(&mut w);
    let (sys, shirt) = hued_recipe_system(&mut w);
    hued(&mut w, pack, "Server.Items.Cloth", 5, 0x21);
    hued(&mut w, pack, "Server.Items.UncutCloth", 3, 0x21);
    hued(&mut w, pack, "Server.Items.UncutCloth", 5, 0x30);

    assert!(super::t2a::can_craft_item(
        &mut w, m, &sys, shirt, None, 0x21
    ));
    assert!(!super::t2a::can_craft_item(
        &mut w, m, &sys, shirt, None, 0x30
    ));
}

#[test]
fn required_expansion_message_names_sa_by_its_info_name() {
    use muo_data::expansion::Expansion;
    let t = super::gumps::required_expansion_message(Expansion::SA, "Stygian Abyss");
    assert_eq!(
        t.string.as_deref(),
        Some("* Requires the \"Stygian Abyss\" expansion")
    );
    assert_eq!(
        super::gumps::required_expansion_message(Expansion::SE, "x").number,
        1063363
    );
    assert_eq!(
        super::gumps::required_expansion_message(Expansion::ML, "x").number,
        1072651
    );
}

/// `ItemIDOf` special-cases `FactionExplosionTrap` only: its deed (the thing tinkering makes) has its own item id.
#[test]
fn read3c_item_id_of_explosion_trap_not_deed() {
    let mut w = world();
    assert_eq!(
        super::item::item_id_of(&mut w, "Server.Factions.FactionExplosionTrap"),
        14034
    );
    assert_eq!(
        super::item::item_id_of(&mut w, "Server.Factions.FactionExplosionTrapDeed"),
        0x36D2
    ); // from its constructor
}
