//! The T2A packet craft menus: `T2ACraftSystem`, `T2ACraftToolTarget`, `AlchemyMenu`, `BlacksmithMenu`,
//! `BowFletchingMenu`, `CarpentryMenu`, `CartographyMenu`, `InscriptionMenu`, `TailoringMenu`, `TinkeringMenu` and the parts of the two skill
//! handlers that open Cartography and Inscription (`T2ACraftSystem.ShowMenu` for a tool-less skill, so `tool` is null).
//!
//! The six category-tree menus share one engine ([`Def`], [`Ctx`]): a table per menu says its categories, questions,
//! entries and how a leaf crafts; the filtering, navigation and messages are shared by every menu.
//! Not implemented: `GemSelectTarget.OnTargetCancel` (it calls `ShowCraftMenu` without a message, which does nothing in
//! T2A) (`T2ACooking` is `muo_items::food::cooking`).

use std::rc::Rc;

use muo_content::items::jewel::GemType;
use muo_content::skills::SkillGumpHooks;
use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::deps;
use muo_server::menus::{ItemListEntry, Menu};
use muo_server::mobile::part2::{messages::send_ascii_message, speech::send_menu};
use muo_server::target::{TargetFlags, Targeted, begin_target};

use super::item::{
    craft, craft_item_id, derives, equivalent_types, hued_amount, resolve, success_chance,
};
use super::model::CraftSystem;
use super::state::{state, state_mut};
use super::{context_of, modify_context, system};

const BLANK_MAP: &str = "Server.Items.BlankMap";
const BLANK_SCROLL: &str = "Server.Items.BlankScroll";
const RUNEBOOK: &str = "Server.Items.Runebook";
const IRON_INGOT: &str = "Server.Items.IronIngot";
const NOTHING: &str = "You lack the skill and materials to craft anything.";
const NOTHING_IN_CATEGORY: &str =
    "You lack the skill and materials to craft anything in that category.";

fn short_name(full: &str) -> &str {
    full.rsplit('.').next().unwrap_or(full)
}

/// `FormatItemName`: lower case, a space before each capital after the first letter.
fn format_name(name: &str) -> String {
    let mut out = String::new();
    for (i, c) in name.chars().enumerate() {
        if i > 0 && c.is_uppercase() {
            out.push(' ');
        }
        out.extend(c.to_lowercase());
    }
    out
}

/// The craft item of `sys` whose class is called `short` (the trap crafts live in `Server.Engines.Craft`).
fn find(sys: &CraftSystem, short: &str) -> Option<usize> {
    sys.items
        .iter()
        .position(|c| short_name(c.item_type) == short)
}

/// `GetResourceAmount`: the equivalence table, else the type itself.
fn amount_of(w: &World, pack: ItemId, ty: &str) -> i32 {
    let names = equivalent_types(w, ty).unwrap_or_else(|| vec![ty.to_string()]);
    let types: Vec<_> = names.iter().filter_map(|n| resolve(w, n)).collect();
    muo_server::item::container::get_amount_types(w, pack, &types, true)
}

/// `pack.GetAmount(typeof(T))`.
fn plain_amount(w: &World, pack: ItemId, ty: &str) -> i32 {
    resolve(w, ty).map_or(0, |t| {
        muo_server::item::container::get_amount(w, pack, t, true)
    })
}

fn backpack(w: &World, from: MobileId) -> Option<ItemId> {
    deps::find_item_on_layer(w, from, muo_server::layer::Layer::Backpack)
}

/// `T2ACraftSystem.CanCraftItem(from, itemDef, system, selectedResourceType, hue)`: a selected resource counts only
/// that sub-resource, none counts any the skill allows; a hue >= 0 counts only same-hue stock of the first resource.
pub fn can_craft_item(
    w: &mut World,
    from: MobileId,
    sys: &CraftSystem,
    index: usize,
    selected: Option<&str>,
    hue: i32,
) -> bool {
    let Some(pack) = backpack(w, from) else {
        return false;
    };
    let item = sys.items[index].clone();
    let mut all = true;
    let chance = success_chance(w, sys, &item, from, false, &mut all);
    if !all || chance <= 0.0 {
        return false;
    }
    let main = muo_server::skills::value(w, from, sys.main_skill);
    // The skill gate of a selected sub-resource.
    let gated = |s: &str| {
        sys.sub_res
            .search_for(s)
            .is_some_and(|r| main < r.required_skill)
    };
    for (i, res) in item.resources.iter().enumerate() {
        let mut res_type = res.item_type;
        let base_sub = sys.sub_res.init && res_type == sys.sub_res.res_type;
        if hue >= 0 && i == 0 {
            if base_sub && let Some(sel) = selected {
                if gated(sel) {
                    return false;
                }
                res_type = sel;
            }
            if hued_amount(w, pack, res_type, hue) < res.amount {
                return false;
            }
            continue;
        }
        let ok = if base_sub {
            match selected {
                Some(sel) => !gated(sel) && amount_of(w, pack, sel) >= res.amount,
                // HasAnySufficientSubResource
                None => sys.sub_res.items.iter().any(|s| {
                    main >= s.required_skill && amount_of(w, pack, s.item_type) >= res.amount
                }),
            }
        } else {
            amount_of(w, pack, res_type) >= res.amount
        };
        if !ok {
            return false;
        }
    }
    true
}

/// `T2ACraftSystem.SetLastResourceIndex`.
fn set_last_resource_index(
    w: &mut World,
    from: MobileId,
    sys: &CraftSystem,
    selected: Option<&str>,
) {
    let Some(sel) = selected else { return };
    if !sys.sub_res.init {
        return;
    }
    if let Some(i) = sys.sub_res.items.iter().position(|s| s.item_type == sel) {
        modify_context(w, sys.name, from, |c| c.last_resource_index = i as i32);
    }
}

fn blank_scrolls(w: &World, from: MobileId) -> i32 {
    backpack(w, from).map_or(0, |pack| plain_amount(w, pack, BLANK_SCROLL))
}

/// `from.SendMenu(new ItemListMenu(question, entries))`; the response gets the mobile of the net state. An index
/// outside the entries does nothing.
fn open(
    w: &mut World,
    from: MobileId,
    question: &str,
    entries: Vec<ItemListEntry>,
    on_response: impl Fn(&mut World, MobileId, i32, &[ItemListEntry]) + 'static,
) {
    let mut menu = Menu::item_list(w, question, entries.clone());
    menu.on_response = Some(Rc::new(move |w, ns, index| {
        if index < 0 || index as usize >= entries.len() {
            return;
        }
        if let Some(m) = w.net_mobile(ns) {
            on_response(w, m, index, &entries);
        }
    }));
    send_menu(w, from, menu);
}

fn item_is(w: &World, it: ItemId, names: &[&str]) -> bool {
    names
        .iter()
        .any(|n| muo_content::factory::item_is(w, it, n))
}

// ---- ShowMenu and T2ACraftToolTarget ----

/// `T2ACraftSystem.HasMenu(system)`.
pub fn has_menu(name: &str) -> bool {
    matches!(
        name,
        "Blacksmithy"
            | "Alchemy"
            | "BowFletching"
            | "Carpentry"
            | "Cartography"
            | "Inscription"
            | "Tailoring"
            | "Tinkering"
    )
}

/// `from.Target = new T2ACraftToolTarget(tool, system)` and its message: what `BaseTool.OnDoubleClick` does when
/// `HasMenu`.
pub fn begin_tool_target(w: &mut World, from: MobileId, sys: &str, tool: ItemId) {
    let Some(name) = super::defs::NAMES.iter().copied().find(|n| *n == sys) else {
        return;
    };
    begin_target(
        w,
        from,
        2,
        false,
        TargetFlags::NONE,
        move |w, from, targeted| {
            tool_target(w, from, name, tool, targeted);
        },
    );
    send_ascii_message(
        w,
        from,
        "Target this tool to make last item, or any other target to begin crafting.",
    );
}

/// `T2ACraftToolTarget.OnTarget`.
fn tool_target(
    w: &mut World,
    from: MobileId,
    name: &'static str,
    tool: ItemId,
    targeted: Targeted,
) {
    let item = match targeted {
        Targeted::Entity(EntityId::Item(i)) => Some(i),
        _ => None,
    };
    if item != Some(tool) {
        // Normal flow: the targeted object goes through so the resource selection can use it.
        show_menu(w, from, name, Some(tool), item);
        return;
    }
    let Some(sys) = system(w, name) else { return };
    // Make Last: repeat last crafted item
    let Some(last) = context_of(w, name, from).and_then(|c| c.last_made()) else {
        send_ascii_message(w, from, "You have not yet crafted anything.");
        show_menu(w, from, name, Some(tool), None);
        return;
    };
    let item = sys.items[last].clone();
    let num = (sys.vt.can_craft)(w, from, Some(tool), Some(item.item_type));
    if num > 0 {
        deps::send_localized_message(w, from, num);
        return;
    }
    let ctx = context_of(w, name, from).unwrap_or_default();
    let mut ty = last_resource(&sys, &item, &ctx);
    // T2A tinkering has no colored ingots; a stale index from the gump flow must not leak in
    if name == "Tinkering" {
        ty = Some(IRON_INGOT);
        set_last_resource_index(w, from, &sys, ty);
    }
    // Jewelry requires gem targeting: re-prompt instead of crafting directly
    if derives(w, item.item_type, "Server.Items.BaseJewel") {
        modify_context(w, name, from, |c| c.pending_gem = None);
        send_ascii_message(w, from, "Target the gemstone you wish to use.");
        gem_target(w, from, Some(tool), item.item_type, ty);
        return;
    }
    let hue = ctx.last_hue;
    craft(
        w,
        from,
        &sys,
        last,
        ty,
        Some(tool),
        (hue >= 0).then_some(hue),
    );
}

/// `itemDef.UseSubRes2 ? CraftSubRes2 : CraftSubRes` at the context's last index.
fn last_resource(
    sys: &CraftSystem,
    item: &super::model::CraftItem,
    ctx: &super::model::CraftContext,
) -> Option<&'static str> {
    let (col, index) = if item.use_sub_res2 {
        (&sys.sub_res2, ctx.last_resource_index2)
    } else {
        (&sys.sub_res, ctx.last_resource_index)
    };
    if index > -1 {
        col.items.get(index as usize).map(|s| s.item_type)
    } else {
        None
    }
}

/// `T2ACraftSystem.ShowMenu(from, craftSystem, tool, preTarget)`.
pub fn show_menu(
    w: &mut World,
    from: MobileId,
    name: &'static str,
    tool: Option<ItemId>,
    pre: Option<ItemId>,
) {
    if !super::state::t2a_enabled(w) {
        return;
    }
    // `preTarget == null` or one of the resource classes opens the menu; anything else asks for a resource.
    let ok = |w: &World, names: &[&str]| pre.is_none_or(|p| item_is(w, p, names));
    match name {
        "Blacksmithy" => smith_resource_selection(w, from, tool, pre),
        "Alchemy" => {
            const OK: [&str; 2] = ["Server.Items.BaseReagent", "Server.Items.Bottle"];
            if ok(w, &OK) {
                show_direct(w, from, &ALCHEMY, tool);
            } else {
                prompt_for_resource(
                    w,
                    from,
                    tool,
                    name,
                    "Target a reagent or empty bottle.",
                    |w, i| item_is(w, i, &OK),
                );
            }
        }
        "BowFletching" => {
            const OK: [&str; 4] = [
                "Server.Items.Log",
                "Server.Items.Board",
                "Server.Items.Feather",
                "Server.Items.Shaft",
            ];
            if ok(w, &OK) {
                show_direct(w, from, &BOW_FLETCHING, tool);
            } else {
                prompt_for_resource(
                    w,
                    from,
                    tool,
                    name,
                    "Target the wood or feathers you wish to use.",
                    |w, i| item_is(w, i, &OK),
                );
            }
        }
        "Carpentry" => {
            const OK: [&str; 2] = ["Server.Items.Log", "Server.Items.Board"];
            if ok(w, &OK) {
                show_direct(w, from, &CARPENTRY, tool);
            } else {
                prompt_for_resource(
                    w,
                    from,
                    tool,
                    name,
                    "Target the wood you wish to use.",
                    |w, i| item_is(w, i, &OK),
                );
            }
        }
        "Cartography" => {
            const OK: [&str; 1] = [BLANK_MAP];
            if ok(w, &OK) {
                show_cartography(w, from, tool);
            } else {
                prompt_for_resource(w, from, tool, name, "Target a blank map.", |w, i| {
                    item_is(w, i, &OK)
                });
            }
        }
        "Inscription" => {
            const OK: [&str; 3] = [
                BLANK_SCROLL,
                "Server.Items.BaseReagent",
                "Server.Items.RecallRune",
            ];
            if ok(w, &OK) {
                show_inscription(w, from, tool);
            } else {
                prompt_for_resource(
                    w,
                    from,
                    tool,
                    name,
                    "Target the blank scrolls you wish to use.",
                    |w, i| item_is(w, i, &OK),
                );
            }
        }
        "Tailoring" => tailoring_resource_selection(w, from, tool, pre),
        "Tinkering" => tinkering_resource_selection(w, from, tool, pre),
        _ => {}
    }
}

/// `ShowMenuDirect<T>`: the menu, or the message when it has no entries.
fn show_direct(w: &mut World, from: MobileId, def: &'static Def, tool: Option<ItemId>) {
    let c = Ctx {
        def,
        tool,
        hue: -1,
        sel: None,
    };
    show_category(w, from, &c, def.main, NOTHING);
}

/// `PromptForResource` and its `CraftResourceTarget` (range 12, asks again until the item is valid).
fn prompt_for_resource(
    w: &mut World,
    from: MobileId,
    tool: Option<ItemId>,
    name: &'static str,
    message: &'static str,
    valid: fn(&World, ItemId) -> bool,
) {
    send_ascii_message(w, from, message);
    prompt_target(w, from, tool, name, message, valid);
}

fn prompt_target(
    w: &mut World,
    from: MobileId,
    tool: Option<ItemId>,
    name: &'static str,
    message: &'static str,
    valid: fn(&World, ItemId) -> bool,
) {
    begin_target(
        w,
        from,
        12,
        false,
        TargetFlags::NONE,
        move |w, from, targeted| {
            if let Targeted::Entity(EntityId::Item(i)) = targeted
                && valid(w, i)
            {
                show_menu(w, from, name, tool, Some(i));
                return;
            }
            send_ascii_message(w, from, message);
            prompt_target(w, from, tool, name, message, valid);
        },
    );
}

// ---- the category-tree menus ----

/// One row of a navigation category: `ItemListEntry(name, itemId, 0, craftIndex)`.
struct Nav {
    name: &'static str,
    item_id: i32,
    index: i32,
}

/// How the entries of a leaf category are named.
#[derive(Copy, Clone)]
enum Labels {
    /// `FormatItemName` without the `Potion` suffix.
    Potion,
    /// Fixed names and item ids, parallel to the types.
    Fixed(&'static [&'static str], &'static [i32]),
    /// `name (amount resource)`; Blacksmith's 7033 shows as 7032.
    Res(&'static str),
    /// Carpentry: `name (amount wood[, amount ingots|cloth])`.
    Carpentry,
}

struct Leaf {
    types: &'static [&'static str],
    labels: Labels,
}

/// A menu's tables. A category is an `i32` (the `Category` enum of that menu).
struct Def {
    system: &'static str,
    main: i32,
    question: fn(i32) -> &'static str,
    nav: fn(i32) -> Option<&'static [Nav]>,
    leaf: fn(i32) -> Option<Leaf>,
    /// The category a navigation row leads to.
    child: fn(cat: i32, index: i32) -> i32,
    /// The rows at the start of the main category that are actions (Repair, Smelt), always listed.
    actions: i32,
    action: fn(&mut World, MobileId, &Ctx, i32),
    /// What a chosen leaf does.
    craft: fn(&mut World, MobileId, &Ctx, i32, &'static str),
    /// Blacksmith: the sub-resource is the context's last one, not a field of the menu.
    context_resource: bool,
}

/// The state of one open menu: its tool, the targeted cloth's hue (-1 none), the selected resource.
#[derive(Clone)]
struct Ctx {
    def: &'static Def,
    tool: Option<ItemId>,
    hue: i32,
    sel: Option<&'static str>,
}

fn no_action(_: &mut World, _: MobileId, _: &Ctx, _: i32) {}

fn same_child(_: i32, index: i32) -> i32 {
    index
}

/// `GetSelectedResourceType` of the Blacksmith menu, else the menu's own resource.
fn selected(w: &World, from: MobileId, c: &Ctx) -> Option<&'static str> {
    if !c.def.context_resource {
        return c.sel;
    }
    let ctx = context_of(w, c.def.system, from)?;
    let i = usize::try_from(ctx.last_resource_index).ok()?;
    let sys = state(w)?.systems.get(c.def.system)?;
    sys.sub_res.items.get(i).map(|s| s.item_type)
}

fn build_static(w: &mut World, sys: &CraftSystem, leaf: &Leaf) -> Vec<ItemListEntry> {
    let mut out = Vec::new();
    for (i, short) in leaf.types.iter().enumerate() {
        let Some(idx) = find(sys, short) else {
            continue;
        };
        let item = &sys.items[idx];
        let entry = match leaf.labels {
            Labels::Potion => {
                let n = short.strip_suffix("Potion").unwrap_or(short);
                ItemListEntry::new(&format_name(n), craft_item_id(w, item), 0, i as i32)
            }
            Labels::Fixed(names, graphics) => {
                ItemListEntry::new(names[i], graphics[i], 0, i as i32)
            }
            Labels::Res(r) => {
                let mut id = craft_item_id(w, item);
                if id == 7033 {
                    id = 7032;
                }
                let label = format!("{} ({} {r})", format_name(short), item.resources[0].amount);
                ItemListEntry::new(&label, id, 0, i as i32)
            }
            Labels::Carpentry => {
                let (name, res) = (format_name(short), &item.resources);
                let label = if res.len() > 1 {
                    let second = if res[1].item_type == IRON_INGOT {
                        "ingots"
                    } else {
                        "cloth"
                    };
                    format!(
                        "{name} ({} wood, {} {second})",
                        res[0].amount, res[1].amount
                    )
                } else {
                    format!("{name} ({} wood)", res[0].amount)
                };
                ItemListEntry::new(&label, craft_item_id(w, item), 0, i as i32)
            }
        };
        out.push(entry);
    }
    out
}

/// `AnyCraftableInCategory` over a leaf category, or over the leaves below a navigation category.
fn any_craftable(w: &mut World, from: MobileId, c: &Ctx, sys: &CraftSystem, cat: i32) -> bool {
    let sel = selected(w, from, c);
    if let Some(leaf) = (c.def.leaf)(cat) {
        return leaf
            .types
            .iter()
            .any(|t| find(sys, t).is_some_and(|i| can_craft_item(w, from, sys, i, sel, c.hue)));
    }
    let Some(nav) = (c.def.nav)(cat) else {
        return false;
    };
    nav.iter().any(|n| {
        let child = (c.def.child)(cat, n.index);
        child != cat && any_craftable(w, from, c, sys, child)
    })
}

/// `BuildFilteredEntries(from, category)`.
fn build_entries(w: &mut World, from: MobileId, c: &Ctx, cat: i32) -> Vec<ItemListEntry> {
    let Some(sys) = system(w, c.def.system) else {
        return Vec::new();
    };
    if let Some(nav) = (c.def.nav)(cat) {
        let mut out = Vec::new();
        for n in nav {
            let shown = (cat == c.def.main && n.index < c.def.actions)
                || any_craftable(w, from, c, &sys, (c.def.child)(cat, n.index));
            if shown {
                out.push(ItemListEntry::new(n.name, n.item_id, 0, n.index));
            }
        }
        return out;
    }
    let Some(leaf) = (c.def.leaf)(cat) else {
        return Vec::new();
    };
    let sel = selected(w, from, c);
    let statics = build_static(w, &sys, &leaf);
    statics
        .into_iter()
        .filter(|e| {
            leaf.types
                .get(e.craft_index as usize)
                .and_then(|t| find(&sys, t))
                .is_some_and(|i| can_craft_item(w, from, &sys, i, sel, c.hue))
        })
        .collect()
}

/// Builds the menu of `cat` and sends it, or says `empty` when it has no entries.
fn show_category(w: &mut World, from: MobileId, c: &Ctx, cat: i32, empty: &str) {
    let entries = build_entries(w, from, c, cat);
    if entries.is_empty() {
        send_ascii_message(w, from, empty);
        return;
    }
    let question = (c.def.question)(cat);
    let c = c.clone();
    open(
        w,
        from,
        question,
        entries,
        move |w, from, index, entries| respond(w, from, &c, cat, entries, index),
    );
}

/// `OnResponse(state, index)`.
fn respond(
    w: &mut World,
    from: MobileId,
    c: &Ctx,
    cat: i32,
    entries: &[ItemListEntry],
    index: i32,
) {
    let ci = entries[index as usize].craft_index;
    if (c.def.nav)(cat).is_some() {
        if cat == c.def.main && ci < c.def.actions {
            (c.def.action)(w, from, c, ci);
            return;
        }
        show_category(w, from, c, (c.def.child)(cat, ci), NOTHING_IN_CATEGORY);
        return;
    }
    let Some(leaf) = (c.def.leaf)(cat) else {
        return;
    };
    if ci >= 0
        && let Some(short) = leaf.types.get(ci as usize)
    {
        (c.def.craft)(w, from, c, cat, short);
    }
}

/// Craft of Bow Fletching, Carpentry, Blacksmithy and Tailoring: `CanCraft`, the context's last resource, `CreateItem`.
fn craft_last_resource(w: &mut World, from: MobileId, c: &Ctx, _cat: i32, short: &'static str) {
    let Some(sys) = system(w, c.def.system) else {
        return;
    };
    let Some(idx) = find(&sys, short) else { return };
    let item = &sys.items[idx];
    let num = (sys.vt.can_craft)(w, from, c.tool, Some(item.item_type));
    if num > 0 {
        deps::send_localized_message(w, from, num);
        return;
    }
    let ctx = context_of(w, c.def.system, from).unwrap_or_default();
    let res = last_resource(&sys, item, &ctx);
    if c.def.system != "Tailoring" {
        craft(w, from, &sys, idx, res, c.tool, None);
    } else if c.hue >= 0 {
        // Hue-aware crafting: only resources matching this hue are consumed
        modify_context(w, "Tailoring", from, |x| x.last_hue = c.hue);
        craft(w, from, &sys, idx, res, c.tool, Some(c.hue));
    } else {
        // Make Last picks the hue-aware overload from LastHue, so a leather craft must clear the cloth hue
        modify_context(w, "Tailoring", from, |x| x.last_hue = -1);
        craft(w, from, &sys, idx, res, c.tool, None);
    }
}

// ---- Alchemy ----

const ALCHEMY_MAIN: [Nav; 8] = [
    Nav {
        name: "Refresh",
        item_id: 0xF0B,
        index: 1,
    },
    Nav {
        name: "Agility",
        item_id: 0xF08,
        index: 2,
    },
    Nav {
        name: "Night Sight",
        item_id: 0xF06,
        index: 3,
    },
    Nav {
        name: "Heal",
        item_id: 0xF0C,
        index: 4,
    },
    Nav {
        name: "Strength",
        item_id: 0xF09,
        index: 5,
    },
    Nav {
        name: "Poison",
        item_id: 0xF0A,
        index: 6,
    },
    Nav {
        name: "Cure",
        item_id: 0xF07,
        index: 7,
    },
    Nav {
        name: "Explosion",
        item_id: 0xF0D,
        index: 8,
    },
];

static ALCHEMY: Def = Def {
    system: "Alchemy",
    main: 0,
    question: |c| {
        if c == 0 {
            "What kind of potion?"
        } else {
            "Which potion would you like to make?"
        }
    },
    nav: |c| (c == 0).then_some(&ALCHEMY_MAIN[..]),
    leaf: |c| {
        let types: &'static [&'static str] = match c {
            1 => &["RefreshPotion", "TotalRefreshPotion"],
            2 => &["AgilityPotion", "GreaterAgilityPotion"],
            3 => &["NightSightPotion"],
            4 => &["LesserHealPotion", "HealPotion", "GreaterHealPotion"],
            5 => &["StrengthPotion", "GreaterStrengthPotion"],
            6 => &[
                "LesserPoisonPotion",
                "PoisonPotion",
                "GreaterPoisonPotion",
                "DeadlyPoisonPotion",
            ],
            7 => &["LesserCurePotion", "CurePotion", "GreaterCurePotion"],
            8 => &[
                "LesserExplosionPotion",
                "ExplosionPotion",
                "GreaterExplosionPotion",
            ],
            _ => return None,
        };
        Some(Leaf {
            types,
            labels: Labels::Potion,
        })
    },
    child: same_child,
    actions: 0,
    action: no_action,
    craft: craft_potion,
    context_resource: false,
};

/// `AlchemyMenu.CraftPotion`.
fn craft_potion(w: &mut World, from: MobileId, c: &Ctx, _cat: i32, short: &'static str) {
    if backpack(w, from).map_or(0, |p| plain_amount(w, p, "Server.Items.Bottle")) == 0 {
        send_ascii_message(w, from, "You need an empty bottle to make a potion.");
        return;
    }
    let Some(sys) = system(w, "Alchemy") else {
        return;
    };
    let Some(idx) = find(&sys, short) else { return };
    let item = &sys.items[idx];
    let num = (sys.vt.can_craft)(w, from, c.tool, Some(item.item_type));
    if num > 0 {
        deps::send_localized_message(w, from, num);
        return;
    }
    craft(
        w,
        from,
        &sys,
        idx,
        Some(item.resources[0].item_type),
        c.tool,
        None,
    );
}

// ---- BowFletching ----

static BOW_FLETCHING: Def = Def {
    system: "BowFletching",
    main: 0,
    question: |_| "What would you like to make?",
    nav: |_| None,
    leaf: |_| {
        Some(Leaf {
            types: &[
                "Kindling",
                "Shaft",
                "Arrow",
                "Bolt",
                "Bow",
                "Crossbow",
                "HeavyCrossbow",
            ],
            labels: Labels::Fixed(
                &[
                    "Kindling",
                    "Shafts",
                    "Arrows",
                    "Bolts",
                    "Bow",
                    "Crossbow",
                    "Heavy Crossbow",
                ],
                &[0xDE1, 0x1BD4, 0xF3F, 0x1BFB, 0x13B2, 0xF50, 0x13FD],
            ),
        })
    },
    child: same_child,
    actions: 0,
    action: no_action,
    craft: craft_last_resource,
    context_resource: false,
};

// ---- Carpentry ----

const CARPENTRY_MAIN: [Nav; 6] = [
    Nav {
        name: "Furniture",
        item_id: 0xB57,
        index: 1,
    },
    Nav {
        name: "Containers",
        item_id: 0x9AA,
        index: 2,
    },
    Nav {
        name: "Weapons",
        item_id: 0xE89,
        index: 3,
    },
    Nav {
        name: "Instruments",
        item_id: 0xEB3,
        index: 4,
    },
    Nav {
        name: "Miscellaneous",
        item_id: 0xDC0,
        index: 5,
    },
    Nav {
        name: "Add-Ons",
        item_id: 0x14F0,
        index: 6,
    },
];

/// Carpentry categories: 0 Main, 1 Furniture, 2 Containers, 3 Weapons, 4 Instruments, 5 Misc, 6 Addons.
static CARPENTRY: Def = Def {
    system: "Carpentry",
    main: 0,
    question: |c| match c {
        1 => "What kind of furniture?",
        2 => "What kind of container?",
        3 => "What kind of weapon?",
        4 => "What kind of instrument?",
        6 => "What kind of add-on?",
        _ => "What would you like to make?",
    },
    nav: |c| (c == 0).then_some(&CARPENTRY_MAIN[..]),
    leaf: |c| {
        let types: &'static [&'static str] = match c {
            1 => &[
                "FootStool",
                "Stool",
                "BambooChair",
                "WoodenChair",
                "FancyWoodenChairCushion",
                "WoodenChairCushion",
                "WoodenBench",
                "WoodenThrone",
                "Throne",
                "Nightstand",
                "WritingTable",
                "YewWoodTable",
                "LargeTable",
            ],
            2 => &[
                "WoodenBox",
                "SmallCrate",
                "MediumCrate",
                "LargeCrate",
                "WoodenChest",
                "EmptyBookcase",
                "FancyArmoire",
                "Armoire",
                "Keg",
            ],
            3 => &[
                "ShepherdsCrook",
                "QuarterStaff",
                "GnarledStaff",
                "WoodenShield",
            ],
            4 => &[
                "LapHarp",
                "Harp",
                "Drums",
                "Lute",
                "Tambourine",
                "TambourineTassel",
            ],
            5 => &[
                "FishingPole",
                "BarrelStaves",
                "BarrelLid",
                "ShortMusicStand",
                "TallMusicStand",
                "Easel",
            ],
            6 => &[
                "SmallBedSouthDeed",
                "SmallBedEastDeed",
                "LargeBedSouthDeed",
                "LargeBedEastDeed",
                "DartBoardSouthDeed",
                "DartBoardEastDeed",
                "BallotBoxDeed",
                "PentagramDeed",
                "AbbatoirDeed",
                "SmallForgeDeed",
                "LargeForgeEastDeed",
                "LargeForgeSouthDeed",
                "AnvilEastDeed",
                "AnvilSouthDeed",
                "TrainingDummyEastDeed",
                "TrainingDummySouthDeed",
                "PickpocketDipEastDeed",
                "PickpocketDipSouthDeed",
                "Dressform",
                "SpinningWheelEastDeed",
                "SpinningWheelSouthDeed",
                "LoomEastDeed",
                "LoomSouthDeed",
                "StoneOvenEastDeed",
                "StoneOvenSouthDeed",
                "FlourMillEastDeed",
                "FlourMillSouthDeed",
                "WaterTroughEastDeed",
                "WaterTroughSouthDeed",
            ],
            _ => return None,
        };
        Some(Leaf {
            types,
            labels: Labels::Carpentry,
        })
    },
    child: same_child,
    actions: 0,
    action: no_action,
    craft: craft_last_resource,
    context_resource: false,
};

// ---- Tailoring ----

const TAILORING_MAIN: [Nav; 4] = [
    Nav {
        name: "Build Hats",
        item_id: 0x1718,
        index: 2,
    },
    Nav {
        name: "Build Shirts",
        item_id: 0x1517,
        index: 3,
    },
    Nav {
        name: "Build Pants",
        item_id: 0x1539,
        index: 4,
    },
    Nav {
        name: "Build Misc",
        item_id: 0x153D,
        index: 5,
    },
];

const TAILORING_LEATHER: [Nav; 4] = [
    Nav {
        name: "Build Shoes",
        item_id: 0x170f,
        index: 6,
    },
    Nav {
        name: "Build Leather Armor",
        item_id: 0x13cc,
        index: 7,
    },
    Nav {
        name: "Build Studded Armor",
        item_id: 0x13db,
        index: 8,
    },
    Nav {
        name: "Build Female Armor",
        item_id: 0x1c06,
        index: 9,
    },
];

/// Tailoring categories: 0 Main, 1 LeatherMain, 2 Hats, 3 Shirts, 4 Pants, 5 Misc, 6 Footwear, 7 Leather, 8 Studded,
/// 9 Female.
static TAILORING: Def = Def {
    system: "Tailoring",
    main: 0,
    question: |c| match c {
        2 => "What kind of hat?",
        3 => "What kind of shirt?",
        4 => "What kind of pants?",
        6 => "What kind of footwear?",
        7 => "What kind of leather armor?",
        8 => "What kind of studded armor?",
        9 => "What kind of female leather?",
        _ => "What would you like to make?",
    },
    nav: |c| match c {
        0 => Some(&TAILORING_MAIN[..]),
        1 => Some(&TAILORING_LEATHER[..]),
        _ => None,
    },
    leaf: |c| {
        let (types, r): (&'static [&'static str], &'static str) = match c {
            2 => (
                &[
                    "SkullCap",
                    "Bandana",
                    "FloppyHat",
                    "Cap",
                    "WideBrimHat",
                    "StrawHat",
                    "TallStrawHat",
                    "WizardsHat",
                    "Bonnet",
                    "FeatheredHat",
                    "TricorneHat",
                    "JesterHat",
                ],
                "cloth",
            ),
            3 => (
                &[
                    "Doublet",
                    "Shirt",
                    "FancyShirt",
                    "Tunic",
                    "Surcoat",
                    "PlainDress",
                ],
                "cloth",
            ),
            4 => (&["ShortPants", "LongPants", "Kilt"], "cloth"),
            5 => (
                &[
                    "Skirt",
                    "Cloak",
                    "Robe",
                    "JesterSuit",
                    "FancyDress",
                    "BodySash",
                    "HalfApron",
                    "FullApron",
                ],
                "cloth",
            ),
            6 => (&["Sandals", "Shoes", "Boots", "ThighBoots"], "leather"),
            7 => (
                &[
                    "LeatherChest",
                    "LeatherGorget",
                    "LeatherGloves",
                    "LeatherCap",
                    "LeatherArms",
                    "LeatherLegs",
                ],
                "leather",
            ),
            8 => (
                &[
                    "StuddedChest",
                    "StuddedGorget",
                    "StuddedGloves",
                    "StuddedArms",
                    "StuddedLegs",
                ],
                "leather",
            ),
            9 => (
                &[
                    "FemaleLeatherChest",
                    "FemaleStuddedChest",
                    "LeatherBustierArms",
                    "StuddedBustierArms",
                    "FemalePlateChest",
                    "LeatherShorts",
                    "LeatherSkirt",
                ],
                "leather",
            ),
            _ => return None,
        };
        Some(Leaf {
            types,
            labels: Labels::Res(r),
        })
    },
    child: same_child,
    actions: 0,
    action: no_action,
    craft: craft_last_resource,
    context_resource: false,
};

fn tailoring_open(w: &mut World, from: MobileId, tool: Option<ItemId>, cat: i32, hue: i32) {
    let c = Ctx {
        def: &TAILORING,
        tool,
        hue,
        sel: None,
    };
    show_category(w, from, &c, cat, NOTHING);
}

/// `TailoringMenu.TrySelectResource`.
fn tailoring_try_select(w: &mut World, from: MobileId, tool: Option<ItemId>, item: ItemId) -> bool {
    if item_is(w, item, &["Server.Items.Cloth", "Server.Items.UncutCloth"]) {
        let hue = w.item(item).map_or(0, |i| i.hue);
        tailoring_open(w, from, tool, 0, hue);
        return true;
    }
    if item_is(w, item, &["Server.Items.Leather", "Server.Items.Hides"]) {
        tailoring_open(w, from, tool, 1, -1);
        return true;
    }
    false
}

/// `TailoringMenu.ResourceSelection` and its `ResourceSelectTarget`.
fn tailoring_resource_selection(
    w: &mut World,
    from: MobileId,
    tool: Option<ItemId>,
    pre: Option<ItemId>,
) {
    if let Some(p) = pre
        && tailoring_try_select(w, from, tool, p)
    {
        return;
    }
    send_ascii_message(
        w,
        from,
        "Select the resource you wish to use (cloth, leather, or hides).",
    );
    tailoring_target(w, from, tool);
}

fn tailoring_target(w: &mut World, from: MobileId, tool: Option<ItemId>) {
    begin_target(
        w,
        from,
        12,
        false,
        TargetFlags::NONE,
        move |w, from, targeted| {
            if let Targeted::Entity(EntityId::Item(i)) = targeted
                && tailoring_try_select(w, from, tool, i)
            {
                return;
            }
            send_ascii_message(
                w,
                from,
                "That is not a valid resource. Please select cloth, leather, or hides.",
            );
            tailoring_target(w, from, tool);
        },
    );
}

// ---- Blacksmith ----

// Main's category rows are offset by the two actions (Repair, Smelt).
const SMITH_MAIN: [Nav; 5] = [
    Nav {
        name: "Repair",
        item_id: 0x0FAF,
        index: 0,
    },
    Nav {
        name: "Smelt",
        item_id: 0x0FB1,
        index: 1,
    },
    Nav {
        name: "Build Armor",
        item_id: 0x13EC,
        index: 3 + 2,
    },
    Nav {
        name: "Build Shield",
        item_id: 0x1B74,
        index: 1 + 2,
    },
    Nav {
        name: "Build Weapons",
        item_id: 0xF45,
        index: 2 + 2,
    },
];

const SMITH_WEAPONS: [Nav; 4] = [
    Nav {
        name: "Build Blades",
        item_id: 0xF61,
        index: 4,
    },
    Nav {
        name: "Build Axes",
        item_id: 0x13FB,
        index: 5,
    },
    Nav {
        name: "Build Pole Arms",
        item_id: 0xF4D,
        index: 7,
    },
    Nav {
        name: "Build Bludgeoning Weapons",
        item_id: 0x1407,
        index: 6,
    },
];

const SMITH_ARMOR: [Nav; 3] = [
    Nav {
        name: "Build Ring Armor",
        item_id: 0x13EC,
        index: 10,
    },
    Nav {
        name: "Build Chain Armor",
        item_id: 0x13BF,
        index: 9,
    },
    Nav {
        name: "Build Plate Armor",
        item_id: 0x1415,
        index: 8,
    },
];

/// Blacksmith categories: 0 Main, 1 Shields, 2 Weapons, 3 Armor, 4 Blades, 5 Axes, 6 Maces, 7 Polearms, 8 Platemail,
/// 9 Chainmail, 10 Ringmail.
static BLACKSMITH: Def = Def {
    system: "Blacksmithy",
    main: 0,
    question: |c| match c {
        0 => "What would you like to do?",
        3 => "What kind of armor?",
        1 => "What kind of shield?",
        2 => "What kind of weapon?",
        4 => "What kind of blade?",
        5 => "What kind of axe?",
        7 => "What kind of pole arm?",
        6 => "What kind of bludgeoning weapon?",
        10 => "What kind of ring armor?",
        9 => "What kind of chain armor?",
        8 => "What kind of plate armor?",
        _ => "What would you like to make?",
    },
    nav: |c| match c {
        0 => Some(&SMITH_MAIN[..]),
        2 => Some(&SMITH_WEAPONS[..]),
        3 => Some(&SMITH_ARMOR[..]),
        _ => None,
    },
    leaf: |c| {
        let types: &'static [&'static str] = match c {
            1 => &[
                "Buckler",
                "BronzeShield",
                "HeaterShield",
                "MetalShield",
                "MetalKiteShield",
                "WoodenKiteShield",
            ],
            4 => &[
                "Broadsword",
                "Cutlass",
                "Dagger",
                "Katana",
                "Kryss",
                "Longsword",
                "Scimitar",
                "VikingSword",
            ],
            5 => &[
                "Axe",
                "BattleAxe",
                "DoubleAxe",
                "ExecutionersAxe",
                "LargeBattleAxe",
                "TwoHandedAxe",
                "WarAxe",
            ],
            6 => &["HammerPick", "Mace", "Maul", "WarMace", "WarHammer"],
            7 => &["Bardiche", "Halberd", "WarFork", "ShortSpear", "Spear"],
            8 => &[
                "Bascinet",
                "CloseHelm",
                "Helmet",
                "NorseHelm",
                "PlateHelm",
                "PlateArms",
                "PlateGloves",
                "PlateGorget",
                "PlateLegs",
                "PlateChest",
                "FemalePlateChest",
            ],
            9 => &["ChainCoif", "ChainLegs", "ChainChest"],
            10 => &[
                "RingmailGloves",
                "RingmailLegs",
                "RingmailArms",
                "RingmailChest",
            ],
            _ => return None,
        };
        Some(Leaf {
            types,
            labels: Labels::Res("ingots"),
        })
    },
    child: |cat, index| if cat == 0 { index - 2 } else { index },
    actions: 2,
    action: smith_action,
    craft: craft_last_resource,
    context_resource: true,
};

fn smith_action(w: &mut World, from: MobileId, c: &Ctx, index: i32) {
    let Some(sys) = system(w, "Blacksmithy") else {
        return;
    };
    if index == 0 {
        super::repair::do_repair(w, from, &sys, c.tool);
    } else {
        super::resmelt::do_resmelt(w, from, &sys, c.tool);
    }
}

/// The `afterSelect` of `ShowMenu` for Blacksmithy.
fn smith_after(w: &mut World, from: MobileId, tool: Option<ItemId>) {
    let c = Ctx {
        def: &BLACKSMITH,
        tool,
        hue: -1,
        sel: None,
    };
    show_category(w, from, &c, 0, NOTHING);
}

/// `BlacksmithMenu.ResourceSelection(from, tool, afterSelect, preTarget)`.
fn smith_resource_selection(
    w: &mut World,
    from: MobileId,
    tool: Option<ItemId>,
    pre: Option<ItemId>,
) {
    let Some(sys) = system(w, "Blacksmithy") else {
        return;
    };
    if let Some(p) = pre {
        // Validate preTarget first: reject invalid targets before any auto-selection
        if smith_try_select(w, from, &sys, p) {
            smith_after(w, from, tool);
            return;
        }
        let ty = w.item(p).map_or("", |i| i.ty.full_name);
        if item_is(w, p, &["Server.Items.BaseArmor", "Server.Items.BaseWeapon"])
            && super::repair::craftable_subclass(w, &sys, ty)
        {
            // Repairable/smeltable item: auto-select the default resource and open the menu
            smith_default_resource(w, from, &sys);
            smith_after(w, from, tool);
            return;
        }
        deps::send_message(w, from, "Target the ingots you wish to use.");
        smith_target(w, from, tool);
        return;
    }
    // No target (make-last failure path): auto-select if only one ingot type is available
    let pack = backpack(w, from);
    let mut count = 0;
    let mut last = -1;
    for (i, r) in sys.sub_res.items.iter().enumerate() {
        if pack.map_or(0, |p| plain_amount(w, p, r.item_type)) > 0 {
            count += 1;
            last = i as i32;
        }
    }
    if count <= 1 {
        if last != -1 {
            modify_context(w, "Blacksmithy", from, |c| c.last_resource_index = last);
        }
        smith_after(w, from, tool);
    } else {
        deps::send_message(w, from, "Target the ingots you wish to use.");
        smith_target(w, from, tool);
    }
}

/// `TrySelectResource`: the item is exactly one of the ingot classes.
fn smith_try_select(w: &mut World, from: MobileId, sys: &CraftSystem, item: ItemId) -> bool {
    let ty = w.item(item).map_or("", |i| i.ty.full_name);
    let Some(i) = sys.sub_res.items.iter().position(|r| r.item_type == ty) else {
        return false;
    };
    modify_context(w, "Blacksmithy", from, |c| c.last_resource_index = i as i32);
    true
}

/// `SelectDefaultResource`: iron if the pack has some, else the first ingot type it has.
fn smith_default_resource(w: &mut World, from: MobileId, sys: &CraftSystem) {
    let pack = backpack(w, from);
    let mut first = -1;
    for (i, r) in sys.sub_res.items.iter().enumerate() {
        if pack.map_or(0, |p| plain_amount(w, p, r.item_type)) > 0 {
            if r.item_type == IRON_INGOT {
                modify_context(w, "Blacksmithy", from, |c| c.last_resource_index = i as i32);
                return;
            }
            if first == -1 {
                first = i as i32;
            }
        }
    }
    if first != -1 {
        modify_context(w, "Blacksmithy", from, |c| c.last_resource_index = first);
    }
}

/// `BlacksmithResourceTarget` (range 2).
fn smith_target(w: &mut World, from: MobileId, tool: Option<ItemId>) {
    begin_target(
        w,
        from,
        2,
        false,
        TargetFlags::NONE,
        move |w, from, targeted| {
            if let Targeted::Entity(EntityId::Item(i)) = targeted
                && let Some(sys) = system(w, "Blacksmithy")
                && smith_try_select(w, from, &sys, i)
            {
                smith_after(w, from, tool);
                return;
            }
            deps::send_message(w, from, "That is not a valid ingot.");
            smith_target(w, from, tool);
        },
    );
}

// ---- Tinkering ----

const TINKERING_MAIN: [Nav; 7] = [
    Nav {
        name: "Wooden Items",
        item_id: 0x1BDD,
        index: 1,
    },
    Nav {
        name: "Tools",
        item_id: 0x1EB8,
        index: 2,
    },
    Nav {
        name: "Parts",
        item_id: 0x1053,
        index: 3,
    },
    Nav {
        name: "Utensils",
        item_id: 0x9D7,
        index: 4,
    },
    Nav {
        name: "Traps",
        item_id: 0x1BFC,
        index: 5,
    },
    Nav {
        name: "Miscellaneous",
        item_id: 0xA25,
        index: 6,
    },
    Nav {
        name: "Jewelry",
        item_id: 0x1088,
        index: 7,
    },
];

const TINKERING_JEWELRY: [Nav; 3] = [
    Nav {
        name: "Necklaces",
        item_id: 0x1088,
        index: 8,
    },
    Nav {
        name: "Earrings",
        item_id: 0x1087,
        index: 9,
    },
    Nav {
        name: "Rings",
        item_id: 0x108a,
        index: 10,
    },
];

/// Tinkering categories: 0 Main, 1 Wood, 2 Tools, 3 Parts, 4 Utensils, 5 Traps, 6 Misc, 7 Jewelry, 8 Necklaces,
/// 9 Earrings, 10 Rings, 11 Keg.
static TINKERING: Def = Def {
    system: "Tinkering",
    main: 0,
    question: |c| match c {
        1 => "What kind of wooden item?",
        2 => "What kind of tool?",
        3 => "What kind of part?",
        4 => "What kind of utensil?",
        5 => "What kind of trap?",
        7 => "What kind of jewelry?",
        8 => "What kind of necklace?",
        9 => "What kind of earrings?",
        10 => "What kind of ring?",
        _ => "What would you like to make?",
    },
    nav: |c| match c {
        0 => Some(&TINKERING_MAIN[..]),
        7 => Some(&TINKERING_JEWELRY[..]),
        _ => None,
    },
    leaf: |c| {
        let (types, r): (&'static [&'static str], &'static str) = match c {
            1 => (
                &[
                    "JointingPlane",
                    "MouldingPlane",
                    "SmoothingPlane",
                    "ClockFrame",
                    "Axle",
                    "RollingPin",
                ],
                "logs",
            ),
            2 => (
                &[
                    "SewingKit",
                    "TinkerTools",
                    "DrawKnife",
                    "Froe",
                    "Inshave",
                    "Scorp",
                    "Scissors",
                    "Tongs",
                    "DovetailSaw",
                    "Saw",
                    "Hammer",
                    "SmithHammer",
                    "SledgeHammer",
                    "Shovel",
                    "MortarPestle",
                    "Hatchet",
                    "Pickaxe",
                    "Lockpick",
                ],
                "ingots",
            ),
            3 => (
                &[
                    "Gears",
                    "Springs",
                    "Hinge",
                    "ClockParts",
                    "SextantParts",
                    "BarrelTap",
                    "BarrelHoops",
                    "AxleGears",
                ],
                "ingots",
            ),
            4 => (
                &[
                    "ButcherKnife",
                    "Plate",
                    "Cleaver",
                    "KnifeLeft",
                    "KnifeRight",
                    "SkinningKnife",
                    "ForkLeft",
                    "ForkRight",
                    "SpoonLeft",
                    "SpoonRight",
                    "Goblet",
                    "PewterMug",
                ],
                "ingots",
            ),
            5 => (
                &["DartTrapCraft", "ExplosionTrapCraft", "PoisonTrapCraft"],
                "ingots",
            ),
            6 => (
                &[
                    "KeyRing",
                    "Key",
                    "Scales",
                    "Spyglass",
                    "Lantern",
                    "HeatingStand",
                    "Globe",
                    "Candelabra",
                    "Sextant",
                    "ClockRight",
                    "ClockLeft",
                ],
                "ingots",
            ),
            8 => (&["GoldNecklace", "SilverNecklace"], "ingots"),
            9 => (&["GoldEarrings", "SilverEarrings"], "ingots"),
            10 => (&["GoldRing", "SilverRing", "WeddingRing"], "ingots"),
            11 => (&["PotionKeg"], "kegs"),
            _ => return None,
        };
        Some(Leaf {
            types,
            labels: Labels::Res(r),
        })
    },
    child: same_child,
    actions: 0,
    action: no_action,
    craft: craft_tinkering,
    context_resource: false,
};

/// The leaf branches of `TinkeringMenu.OnResponse`.
fn craft_tinkering(w: &mut World, from: MobileId, c: &Ctx, cat: i32, short: &'static str) {
    let Some(sys) = system(w, "Tinkering") else {
        return;
    };
    let Some(idx) = find(&sys, short) else { return };
    let item_type = sys.items[idx].item_type;
    if (8..=10).contains(&cat) {
        // Jewelry leaf categories: clear stale gem state from a previous attempt, then ask for the gem
        modify_context(w, "Tinkering", from, |x| x.pending_gem = None);
        send_ascii_message(w, from, "Target the gemstone you wish to use.");
        gem_target(w, from, c.tool, item_type, c.sel);
        return;
    }
    let Some(sel) = c.sel else { return };
    // Persist the selected resource index so make-last remembers it
    set_last_resource_index(w, from, &sys, Some(sel));
    craft(w, from, &sys, idx, Some(sel), c.tool, None);
}

/// `TinkeringMenu.GemSelectTarget` (range 12).
fn gem_target(
    w: &mut World,
    from: MobileId,
    tool: Option<ItemId>,
    item_type: &'static str,
    sel: Option<&'static str>,
) {
    begin_target(
        w,
        from,
        12,
        false,
        TargetFlags::NONE,
        move |w, from, targeted| {
            let gem_item = match targeted {
                Targeted::Entity(EntityId::Item(i)) => i,
                _ => {
                    send_ascii_message(w, from, "That is not a gemstone.");
                    return;
                }
            };
            let Some(gem) = gem_type_of(w, gem_item) else {
                send_ascii_message(w, from, "That is not a gemstone.");
                return;
            };
            let amount = w.item(gem_item).map_or(0, |i| i.amount);
            if amount < 1 {
                send_ascii_message(w, from, "That gemstone stack is empty.");
                return;
            }
            let Some(sys) = system(w, "Tinkering") else {
                return;
            };
            let Some(idx) = sys.search_for(item_type) else {
                return;
            };
            // Store pending gem info in the craft context
            modify_context(w, "Tinkering", from, |x| {
                x.pending_gem = Some((gem, amount))
            });
            set_last_resource_index(w, from, &sys, sel);
            craft(w, from, &sys, idx, sel, tool, None);
        },
    );
}

/// `BaseJewel.GetGemType(item)`.
fn gem_type_of(w: &World, item: ItemId) -> Option<GemType> {
    const GEMS: [(&str, GemType); 9] = [
        ("Server.Items.StarSapphire", GemType::StarSapphire),
        ("Server.Items.Emerald", GemType::Emerald),
        ("Server.Items.Sapphire", GemType::Sapphire),
        ("Server.Items.Ruby", GemType::Ruby),
        ("Server.Items.Citrine", GemType::Citrine),
        ("Server.Items.Amethyst", GemType::Amethyst),
        ("Server.Items.Tourmaline", GemType::Tourmaline),
        ("Server.Items.Amber", GemType::Amber),
        ("Server.Items.Diamond", GemType::Diamond),
    ];
    GEMS.iter()
        .find(|(n, _)| muo_content::factory::item_is(w, item, n))
        .map(|(_, g)| *g)
}

fn tinkering_open(
    w: &mut World,
    from: MobileId,
    tool: Option<ItemId>,
    cat: i32,
    sel: &'static str,
) {
    let c = Ctx {
        def: &TINKERING,
        tool,
        hue: -1,
        sel: Some(sel),
    };
    show_category(w, from, &c, cat, NOTHING);
}

/// `TinkeringMenu.TrySelectResource`.
fn tinkering_try_select(w: &mut World, from: MobileId, tool: Option<ItemId>, item: ItemId) -> bool {
    if item_is(w, item, &["Server.Items.Log", "Server.Items.Board"]) {
        tinkering_open(w, from, tool, 1, "Server.Items.Log");
        return true;
    }
    if item_is(w, item, &["Server.Items.BaseIngot"]) && !item_is(w, item, &[IRON_INGOT]) {
        // T2A tinkering predates colored ingots; metal items are always made from iron.
        send_ascii_message(
            w,
            from,
            "Tinkering can only use iron ingots. Please select iron ingots or wood.",
        );
        tinkering_target(w, from, tool);
        return true;
    }
    if item_is(w, item, &[IRON_INGOT]) {
        tinkering_open(w, from, tool, 0, IRON_INGOT);
        return true;
    }
    if item_is(w, item, &["Server.Items.Keg"]) {
        tinkering_open(w, from, tool, 11, "Server.Items.Keg");
        return true;
    }
    false
}

/// `TinkeringMenu.ResourceSelection` and its `ResourceSelectTarget`.
fn tinkering_resource_selection(
    w: &mut World,
    from: MobileId,
    tool: Option<ItemId>,
    pre: Option<ItemId>,
) {
    if let Some(p) = pre
        && tinkering_try_select(w, from, tool, p)
    {
        return;
    }
    send_ascii_message(
        w,
        from,
        "Select the resource you wish to use (wood or ingots).",
    );
    tinkering_target(w, from, tool);
}

fn tinkering_target(w: &mut World, from: MobileId, tool: Option<ItemId>) {
    begin_target(
        w,
        from,
        12,
        false,
        TargetFlags::NONE,
        move |w, from, targeted| {
            if let Targeted::Entity(EntityId::Item(i)) = targeted
                && tinkering_try_select(w, from, tool, i)
            {
                return;
            }
            send_ascii_message(
                w,
                from,
                "That is not a valid resource. Please select wood or ingots.",
            );
            tinkering_target(w, from, tool);
        },
    );
}

// ---- CartographyMenu ----

/// `T2ACraftSystem.ShowMenu(from, DefCartography.CraftSystem, null)`.
pub fn show_cartography_menu(w: &mut World, from: MobileId) {
    show_cartography(w, from, None);
}

fn show_cartography(w: &mut World, from: MobileId, tool: Option<ItemId>) {
    if !super::state::t2a_enabled(w) {
        return;
    }
    let Some(sys) = system(w, "Cartography") else {
        return;
    };
    let mut entries = Vec::new();
    for (i, item) in sys.items.iter().enumerate() {
        let name = match i {
            0 => "A map of the local environs.",
            1 => "A map suitable for cities.",
            2 => "A moderately sized sea chart.",
            3 => "A map of the world.",
            _ => short_name(item.item_type),
        };
        if can_craft_item(w, from, &sys, i, None, -1) {
            entries.push(ItemListEntry::new(name, 6511 + i as i32, 0, i as i32));
        }
    }
    if entries.is_empty() {
        send_ascii_message(w, from, NOTHING);
        return;
    }
    open(
        w,
        from,
        "What kind of map?",
        entries,
        move |w, from, index, entries| {
            let Some(sys) = system(w, "Cartography") else {
                return;
            };
            let ci = entries[index as usize].craft_index;
            if ci < 0 || ci as usize >= sys.items.len() {
                return;
            }
            let num = (sys.vt.can_craft)(w, from, tool, Some(sys.items[ci as usize].item_type));
            if num > 0 {
                deps::send_localized_message(w, from, num);
                return;
            }
            craft(w, from, &sys, ci as usize, Some(BLANK_MAP), tool, None);
        },
    );
}

// ---- InscriptionMenu ----

/// `Category.Runebook`; the circles are 1..=8.
const RUNEBOOK_CATEGORY: i32 = 9;

/// `FormatScrollName`.
fn format_scroll_name(full: &str) -> String {
    let name = short_name(full);
    format_name(name.strip_suffix("Scroll").unwrap_or(name))
}

#[cfg(test)]
pub fn format_scroll_name_for_test(full: &str) -> String {
    format_scroll_name(full)
}

/// `HasSpellInBook`: the spell id of a scroll class is made once (by a throwaway scroll) and kept per class, as
/// `_spellIds` does; building one scroll per menu open would burn item serials.
fn has_spell_in_book(w: &mut World, from: MobileId, scroll_type: &str) -> bool {
    let Some(st) = state(w) else { return false };
    let (id_of, has_spell) = (st.spell_id_of, st.has_spell);
    let id = match st.spell_ids.get(scroll_type).copied() {
        Some(id) => id,
        None => {
            let Some(id) = id_of.and_then(|f| f(w, scroll_type)) else {
                return false;
            };
            state_mut(w).spell_ids.insert(scroll_type.to_string(), id);
            id
        }
    };
    has_spell.is_some_and(|f| f(w, from, id))
}

fn can_scribe_scroll(w: &mut World, from: MobileId, sys: &CraftSystem, index: usize) -> bool {
    can_craft_item(w, from, sys, index, None, -1)
        && blank_scrolls(w, from) != 0
        && has_spell_in_book(w, from, sys.items[index].item_type)
}

/// `T2ACraftSystem.ShowMenu(from, DefInscription.CraftSystem, null)`.
pub fn show_inscription_menu(w: &mut World, from: MobileId) {
    show_inscription(w, from, None);
}

fn show_inscription(w: &mut World, from: MobileId, tool: Option<ItemId>) {
    if !super::state::t2a_enabled(w) {
        return;
    }
    let Some(sys) = system(w, "Inscription") else {
        return;
    };
    let mut entries = Vec::new();
    if sys
        .search_for(RUNEBOOK)
        .is_some_and(|i| can_craft_item(w, from, &sys, i, None, -1))
    {
        entries.push(ItemListEntry::new("Runebook", 0xEFA, 0, RUNEBOOK_CATEGORY));
    }
    const NAMES: [&str; 8] = [
        "First", "Second", "Third", "Fourth", "Fifth", "Sixth", "Seventh", "Eighth",
    ];
    for (c, n) in NAMES.iter().enumerate() {
        if (0..8).any(|j| can_scribe_scroll(w, from, &sys, c * 8 + j)) {
            entries.push(ItemListEntry::new(
                &format!("{n} Circle"),
                8384 + c as i32,
                0,
                c as i32 + 1,
            ));
        }
    }
    if entries.is_empty() {
        send_ascii_message(w, from, NOTHING);
        return;
    }
    open(
        w,
        from,
        "Which circle of spells?",
        entries,
        move |w, from, index, entries| {
            main_response(w, from, index, entries, tool);
        },
    );
}

fn main_response(
    w: &mut World,
    from: MobileId,
    index: i32,
    entries: &[ItemListEntry],
    tool: Option<ItemId>,
) {
    if blank_scrolls(w, from) == 0 {
        send_ascii_message(
            w,
            from,
            "You do not have enough blank scrolls to make that.",
        );
        return;
    }
    let category = entries[index as usize].craft_index;
    if category == RUNEBOOK_CATEGORY {
        craft_runebook_with(w, from, tool);
        return;
    }
    let Some(sys) = system(w, "Inscription") else {
        return;
    };
    let circle = (category - 1) as usize;
    let mut list = Vec::new();
    for i in 0..8 {
        if can_scribe_scroll(w, from, &sys, circle * 8 + i) {
            let item = &sys.items[circle * 8 + i];
            list.push(ItemListEntry::new(
                &format_scroll_name(item.item_type),
                8320 + (circle * 8 + i) as i32,
                0,
                i as i32,
            ));
        }
    }
    if list.is_empty() {
        send_ascii_message(
            w,
            from,
            "You lack the skill and materials to scribe anything in that circle.",
        );
        return;
    }
    open(
        w,
        from,
        "Which spell would you like to scribe?",
        list,
        move |w, from, index, entries| {
            if blank_scrolls(w, from) == 0 {
                send_ascii_message(
                    w,
                    from,
                    "You do not have enough blank scrolls to make that.",
                );
                return;
            }
            craft_scroll(
                w,
                from,
                circle,
                entries[index as usize].craft_index as usize,
                tool,
            );
        },
    );
}

fn craft_scroll(
    w: &mut World,
    from: MobileId,
    circle: usize,
    scroll_index: usize,
    tool: Option<ItemId>,
) {
    let Some(sys) = system(w, "Inscription") else {
        return;
    };
    let index = circle * 8 + scroll_index;
    let item = &sys.items[index];
    if !has_spell_in_book(w, from, item.item_type) {
        send_ascii_message(w, from, "You do not have that spell in your spellbook.");
        return;
    }
    let ctx = context_of(w, sys.name, from).unwrap_or_default();
    let ty = last_resource(&sys, item, &ctx);
    craft(w, from, &sys, index, ty, tool, None);
}

/// `CraftRunebook` (and the `RecallRune` branch of `Inscribe.T2AInscribeTarget`).
pub fn craft_runebook(w: &mut World, from: MobileId) {
    craft_runebook_with(w, from, None);
}

fn craft_runebook_with(w: &mut World, from: MobileId, tool: Option<ItemId>) {
    let Some(sys) = system(w, "Inscription") else {
        return;
    };
    let Some(index) = sys.search_for(RUNEBOOK) else {
        return;
    };
    let num = (sys.vt.can_craft)(w, from, tool, Some(sys.items[index].item_type));
    if num > 0 {
        deps::send_localized_message(w, from, num);
        return;
    }
    craft(w, from, &sys, index, None, tool, None);
}

/// Sets the skill hooks the two handlers wait for.
pub fn configure(w: &mut World) {
    let h = w.resources.get_mut::<SkillGumpHooks>();
    h.show_cartography_menu = Some(show_cartography_menu);
    h.show_inscription_menu = Some(show_inscription_menu);
    h.craft_runebook = Some(craft_runebook);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every class a menu lists is a craft item of its system: `BuildStaticEntries` skips a missing one, so a
    /// misspelled name here would silently drop an entry.
    #[test]
    fn every_menu_entry_is_a_craft_item_of_its_system() {
        let mut w = World::new();
        muo_content::register(&mut w.types);
        muo_items::register(&mut w.types);
        crate::register(&mut w.types);
        super::super::state::state_mut(&mut w).t2a = true; // the silver jewelry is not in the T2A table
        let mut missing = Vec::new();
        for def in [
            &ALCHEMY,
            &BOW_FLETCHING,
            &CARPENTRY,
            &TAILORING,
            &BLACKSMITH,
            &TINKERING,
        ] {
            let sys = system(&mut w, def.system).unwrap();
            for cat in 0..12 {
                if let Some(leaf) = (def.leaf)(cat) {
                    missing.extend(
                        leaf.types
                            .iter()
                            .filter(|t| find(&sys, t).is_none())
                            .map(|t| format!("{}: {t}", def.system)),
                    );
                    if let Labels::Fixed(names, graphics) = leaf.labels {
                        assert!(
                            names.len() == leaf.types.len() && graphics.len() == leaf.types.len()
                        );
                    }
                }
            }
        }
        // DefTailoring has no FemalePlateChest either: that entry is skipped too
        assert_eq!(
            missing,
            ["Tailoring: FemalePlateChest"],
            "not in the craft tables"
        );
    }
}
