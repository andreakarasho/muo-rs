//! The crafting engine (`CraftSystem`, `CraftItem`, the gumps, repair,
//! enhance, resmelt) and the eleven systems (`Def*`).
//!
//! Engines/Craft/T2A (the packet-based menus of pre-UOTD servers; `T2ACraftSystem.Enabled` is read from the
//! `t2aCraftMenus` setting, default `!Core.UOTD`) is `t2a.rs`; `T2ACooking` is `muo_items::food::cooking`.

use std::rc::Rc;

use muo_content::craft as rc;
use muo_content::items::equip_hooks::CraftArgs;
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::type_registry::{Phase, TypeRegistry};

pub mod defs;
pub mod enhance;
pub mod gumps;
pub mod item;
pub mod model;
pub mod repair;
pub mod repair_deed;
pub mod resmelt;
pub mod salvage_bag;
pub mod state;
pub mod t2a;

#[cfg(test)]
mod tests;

use model::{CraftContext, CraftSystem, SkillName};
use state::{state, state_mut};

/// `DefX.CraftSystem`: the system called `name` (built on first use).
pub fn system(w: &mut World, name: &str) -> Option<Rc<CraftSystem>> {
    ensure_built(w);
    state(w)?.systems.get(name).cloned()
}

/// `DefX.Initialize()` for every system, in the fixed system order.
pub fn ensure_built(w: &mut World) {
    if state(w).is_some_and(|s| s.built) {
        return;
    }
    let mut built = Vec::new();
    for name in defs::NAMES {
        if let Some(s) = defs::create(w, name) {
            built.push((name, s));
        }
    }
    let st = state_mut(w);
    st.built = true;
    for (name, s) in built {
        for (idx, item) in s.items.iter().enumerate() {
            if let Some(r) = item.recipe {
                st.recipes.insert(r, (name, idx));
            }
        }
        st.systems.insert(name, Rc::new(s));
    }
}

/// `CraftSystem.GetContext(m)`: a copy of the player's context (a fresh one if none yet); `None` for a deleted or
/// unknown mobile. Change it with [`modify_context`] or store it with [`set_context`].
pub fn context_of(w: &World, sys: &'static str, m: MobileId) -> Option<CraftContext> {
    if !w.mobile(m).is_some_and(|x| !x.deleted) {
        return None;
    }
    Some(
        state(w)
            .and_then(|s| s.contexts.get(&(sys, m)).cloned())
            .unwrap_or_default(),
    )
}

pub fn set_context(w: &mut World, sys: &'static str, m: MobileId, c: CraftContext) {
    state_mut(w).contexts.insert((sys, m), c);
}

pub fn modify_context(
    w: &mut World,
    sys: &'static str,
    m: MobileId,
    f: impl FnOnce(&mut CraftContext),
) {
    if let Some(mut c) = context_of(w, sys, m) {
        f(&mut c);
        set_context(w, sys, m, c);
    }
}

/// `item is ICraftable` then `OnCraft`.
pub fn on_craft(w: &mut World, id: ItemId, a: &CraftArgs) -> Option<i32> {
    let result = rc::on_craft(w, id, a);
    // BaseJewel.OnCraft: the pending gem is spent whether or not the gems were still there, so the next craft
    // starts fresh.
    if a.pending_gem
        .is_some_and(|(gem, count)| gem != muo_content::items::jewel::GemType::None && count > 0)
        && w.item(id)
            .is_some_and(|i| i.ty.is(&muo_content::items::jewel::BASE_JEWEL))
    {
        modify_context(w, a.craft_system, a.from, |c| c.pending_gem = None);
    }
    result
}

/// `item is IUsesRemaining`: its uses.
pub fn uses_remaining_api(w: &World, id: ItemId) -> Option<i32> {
    let ty = w.item(id)?.ty;
    if ty.is(&muo_content::items::tool::BASE_TOOL) {
        return Some(muo_content::items::tool::uses_remaining(w, id));
    }
    if ty.is(&muo_content::items::weapon::BASE_AXE)
        || ty.is(&muo_content::items::weapon::BASE_POLE_ARM)
    {
        return muo_content::items::weapon::data(w, id).map(|d| d.uses_remaining);
    }
    None
}

pub fn set_uses_remaining_api(w: &mut World, id: ItemId, value: i32) {
    let Some(ty) = w.item(id).map(|i| i.ty) else {
        return;
    };
    if ty.is(&muo_content::items::tool::BASE_TOOL) {
        muo_content::items::tool::set_uses_remaining(w, id, value);
    } else if ty.is(&muo_content::items::weapon::BASE_AXE)
        || ty.is(&muo_content::items::weapon::BASE_POLE_ARM)
    {
        // shortcut: the weapon's UsesRemaining setter is not exposed; the field is set directly.
        if let Some(d) = muo_content::items::weapon::data_mut(w, id) {
            d.uses_remaining = value;
        }
        muo_server::item::ops::invalidate_properties(w, id);
    }
}

// ---- hooks ----

fn hook_can_craft(
    w: &mut World,
    system_name: &str,
    from: MobileId,
    tool: ItemId,
    ty: Option<&str>,
) -> i32 {
    match system(w, system_name) {
        Some(s) => (s.vt.can_craft)(w, from, Some(tool), ty),
        None => 0,
    }
}

fn hook_show_menu(w: &mut World, system_name: &str, from: MobileId, tool: ItemId) {
    // `T2ACraftSystem.Enabled && HasMenu(system)`: the tool asks for a target first.
    if state::t2a_enabled(w) && t2a::has_menu(system_name) {
        t2a::begin_tool_target(w, from, system_name, tool);
        return;
    }
    if let Some(s) = system(w, system_name) {
        item::show_craft_menu(w, from, &s, Some(tool), None);
    }
}

fn hook_do_not_color(w: &World, system_name: &str, m: MobileId) -> bool {
    state(w)
        .and_then(|s| {
            s.contexts
                .iter()
                .find(|((n, mm), _)| *n == system_name && *mm == m)
                .map(|(_, c)| c.do_not_color)
        })
        .unwrap_or(false)
}

fn hook_repair_deed(w: &mut World, from: MobileId, deed: ItemId, skill_type: i32) {
    let name = match skill_type {
        1 => "Tailoring",
        2 => "Tinkering",
        3 => "Carpentry",
        4 => "BowFletching",
        _ => "Blacksmithy",
    };
    if let Some(s) = system(w, name) {
        repair::do_repair_deed(w, from, &s, deed);
    }
}

fn hook_retains_color(ty: &'static muo_server::item::ItemType) -> bool {
    const COLORED: &[&str] = &[
        "Server.Items.BaseWeapon",
        "Server.Items.BaseArmor",
        "Server.Items.BaseClothing",
        "Server.Items.BaseJewel",
        "Server.Items.DragonBardingDeed",
    ];
    if muo_content::items::is_named(ty, "Server.Items.OrcHelm") {
        return false;
    }
    COLORED.iter().any(|n| muo_content::items::is_named(ty, n))
}

/// The tailoring half of `Scissor`: the resource a cut-up item gives back and the recipe's first resource amount.
fn hook_tailoring_scissor(
    w: &mut World,
    ty: &'static muo_server::item::ItemType,
    resource: rc::CraftResource,
    clothing_fallback: bool,
) -> Option<(ItemId, i32)> {
    let sys = system(w, "Tailoring")?;
    let item = &sys.items[sys.search_for(ty.full_name)?];
    if item.resources.len() != 1 || item.resources[0].amount < 2 {
        return None;
    }
    let res_type = match rc::resource_types(resource).first() {
        Some(t) => *t,
        // BaseClothing: `(resourceType ?? resource.ItemType)`; BaseArmor has no fallback (the null info throws).
        None if clothing_fallback => item.resources[0].item_type,
        None => return None,
    };
    let res = muo_content::factory::make_item(w, res_type)?;
    Some((res, item.resources[0].amount))
}

/// `from.Backpack.ConsumeTotal(gemItemType, count)` for T2A jewelry.
fn hook_consume_gems(
    w: &mut World,
    from: MobileId,
    gem: muo_content::items::jewel::GemType,
    count: i32,
) -> bool {
    let name = match gem {
        muo_content::items::jewel::GemType::StarSapphire => "Server.Items.StarSapphire",
        muo_content::items::jewel::GemType::Emerald => "Server.Items.Emerald",
        muo_content::items::jewel::GemType::Sapphire => "Server.Items.Sapphire",
        muo_content::items::jewel::GemType::Ruby => "Server.Items.Ruby",
        muo_content::items::jewel::GemType::Citrine => "Server.Items.Citrine",
        muo_content::items::jewel::GemType::Amethyst => "Server.Items.Amethyst",
        muo_content::items::jewel::GemType::Tourmaline => "Server.Items.Tourmaline",
        muo_content::items::jewel::GemType::Amber => "Server.Items.Amber",
        muo_content::items::jewel::GemType::Diamond => "Server.Items.Diamond",
        muo_content::items::jewel::GemType::None => return false,
    };
    let (Some(pack), Some(ty)) = (
        muo_server::item::deps::find_item_on_layer(w, from, muo_server::layer::Layer::Backpack),
        item::resolve(w, name),
    ) else {
        return false;
    };
    muo_server::item::container::consume_total(w, pack, ty, count, true, None)
}

/// `Recipe.Recipes[id]`: the system and the craft item of a recipe.
fn recipe_item(w: &World, id: i32) -> Option<(Rc<CraftSystem>, model::CraftItem)> {
    let st = state(w)?;
    let (name, index) = *st.recipes.get(&id)?;
    let sys = st.systems.get(name)?.clone();
    let item = sys.items.get(index)?.clone();
    Some((sys, item))
}

/// The `RecipeScroll` hooks: `Recipe.TextDefinition`, `GetSuccessChance` of the scroll's recipe, and the scroll
/// the ML quests' satchels give.
fn configure_recipes(w: &mut World) {
    use muo_items::skill_items::misc::{Learn, RecipeHooks};
    let h = w.resources.get_mut::<RecipeHooks>();
    h.text = Some(|w, id| recipe_item(w, id).map(|(_, item)| item.name));
    h.learn = Some(|w, pm, id| {
        let Some((sys, item)) = recipe_item(w, id) else {
            return Learn::Unskilled;
        };
        let mut all_required = true;
        let chance = item::success_chance(w, &sys, &item, pm, false, &mut all_required);
        if all_required && chance >= 0.0 {
            Learn::Learned
        } else {
            Learn::Unskilled
        }
    });
    // `DuelContext.IsFreeConsume(m)` for `Bandage`.
    w.resources
        .get_mut::<muo_items::skill_items::bandage::BandageHooks>()
        .is_free_consume = Some(crate::conpvp::is_free_consume);
    w.resources
        .get_mut::<crate::ml_quests::defs::special::QuestItemHooks>()
        .recipe_scroll = Some(|w, id| Some(muo_items::skill_items::misc::new_recipe_scroll(w, id)));
}

/// Sets what the items wait for: `CraftItem.RetainsColor`, the craft menu, the runic tool.
fn configure(w: &mut World) {
    // `T2ACooking.CraftRange`: the skill range of the cooking craft entry of a type.
    w.resources
        .get_mut::<muo_items::food::cooking::CookingHooks>()
        .craft_range = Some(|w, class| {
        let sys = system(w, "Cooking")?;
        let item = sys.items.iter().find(|c| c.item_type == class)?;
        item.skills.first().map(|s| (s.min_skill, s.max_skill))
    });
    let h = rc::hooks_mut(w);
    h.can_craft = Some(hook_can_craft);
    h.show_craft_menu = Some(hook_show_menu);
    h.do_not_color = Some(hook_do_not_color);
    h.repair_do_deed = Some(hook_repair_deed);
    configure_commands(w);
    repair_deed::configure(w);
    t2a::configure(w);
    configure_recipes(w);
    let e = muo_content::items::equip_hooks::hooks_mut(w);
    e.retains_color = Some(hook_retains_color);
    e.tailoring_scissor = Some(hook_tailoring_scissor);
    e.consume_gems = Some(hook_consume_gems);
    // IHasQuantity: the water containers (BaseWaterContainer) and the beverages (BaseBeverage, with its Content).
    let st = state_mut(w);
    st.quantity = Some(state::QuantityApi {
        type_is_quantity: |t| {
            t.family::<muo_items::special::water::WaterClass>()
                .is_some()
                || t.is(&muo_items::food::beverage::BASE_BEVERAGE)
                || t.is(&muo_items::food::cooking::SACK_FLOUR)
        },
        quantity: |w, id| {
            let ty = w.item(id)?.ty;
            if ty.is(&muo_items::food::beverage::BASE_BEVERAGE) {
                use muo_items::food::beverage as bev;
                let c = match bev::content(w, id) {
                    bev::BeverageType::Ale => model::BeverageType::Ale,
                    bev::BeverageType::Cider => model::BeverageType::Cider,
                    bev::BeverageType::Liquor => model::BeverageType::Liquor,
                    bev::BeverageType::Milk => model::BeverageType::Milk,
                    bev::BeverageType::Wine => model::BeverageType::Wine,
                    bev::BeverageType::Water => model::BeverageType::Water,
                };
                return Some((bev::quantity(w, id), Some(c)));
            }
            if ty.is(&muo_items::food::cooking::SACK_FLOUR) {
                return Some((muo_items::food::cooking::flour_quantity(w, id), None));
            }
            ty.family::<muo_items::special::water::WaterClass>()?;
            Some((muo_items::special::water::quantity(w, id), None))
        },
        set_quantity: |w, id, v| {
            if w.item(id)
                .is_some_and(|i| i.ty.is(&muo_items::food::beverage::BASE_BEVERAGE))
            {
                muo_items::food::beverage::set_quantity(w, id, v);
            } else if w
                .item(id)
                .is_some_and(|i| i.ty.is(&muo_items::food::cooking::SACK_FLOUR))
            {
                muo_items::food::cooking::set_flour_quantity(w, id, v);
            } else {
                muo_items::special::water::set_quantity(w, id, v);
            }
        },
    });
    st.deed_resource = Some(muo_items::last::deeds::barding_resource);
    st.in_faction = Some(|w, m| crate::factions::find(w, m).is_some());
    st.talisman = Some(|w, m| {
        let t = muo_items::talismans::talisman_of(w, m)?;
        let d = muo_items::talismans::data(w, t)?;
        Some((
            SkillName::from_i32(d.skill)?,
            d.success_bonus,
            d.exceptional_bonus,
        ))
    });
    st.spell_id_of = Some(|w, scroll_type| {
        // `typeItem.CreateEntityInstance<SpellScroll>()`, `scroll.SpellID`, `scroll.Delete()`.
        let scroll = muo_content::factory::make_item(w, scroll_type)?;
        let sid = muo_items::skill_items::scrolls::spell_id(w, scroll);
        if let Some(ty) = w.item(scroll).map(|i| i.ty) {
            (ty.delete)(w, scroll);
        }
        sid
    });
    st.has_spell = Some(|w, from, sid| {
        use muo_items::skill_items::spellbook;
        spellbook::find(w, from, sid).is_some_and(|b| spellbook::has_spell(w, b, sid))
    });
    st.spell_known = Some(|w, from, scroll_type| {
        // `DefInscription.CanCraft` makes a scroll each time, unlike the T2A menu.
        let (Some(id_of), Some(has)) = (
            state(w).and_then(|s| s.spell_id_of),
            state(w).and_then(|s| s.has_spell),
        ) else {
            return false;
        };
        id_of(w, scroll_type).is_some_and(|s| has(w, from, s))
    });
}

/// `Recipe.Configure`: `LearnAllRecipes` and `ForgetAllRecipes` (a target on a player).
fn recipe_command(w: &mut World, e: &mut muo_server::commands::CommandEventArgs, learn: bool) {
    use muo_server::item::deps::send_message;
    use muo_server::target::{TargetFlags, Targeted, begin_target};
    let m = e.mobile;
    send_message(
        w,
        m,
        if learn {
            "Target a player to teach them all of the recipes."
        } else {
            "Target a player to have them forget all of the recipes they've learned."
        },
    );
    begin_target(
        w,
        m,
        -1,
        false,
        TargetFlags::NONE,
        move |w, from, targeted| {
            let player = match targeted {
                Targeted::Entity(muo_server::ids::EntityId::Mobile(p))
                    if muo_content::player_mobile::data(w, p).is_some() =>
                {
                    Some(p)
                }
                _ => None,
            };
            let Some(p) = player else {
                send_message(w, from, "That is not a player!");
                return;
            };
            if learn {
                let ids: Vec<i32> = state(w)
                    .map(|s| s.recipes.keys().copied().collect())
                    .unwrap_or_default();
                for id in ids {
                    muo_content::player_mobile::acquire_recipe(w, p, id);
                }
                send_message(w, from, "You teach them all of the recipes.");
            } else {
                muo_content::player_mobile::reset_recipes(w, p);
                send_message(w, from, "They forget all their recipes.");
            }
        },
    );
}

fn configure_commands(w: &mut World) {
    use muo_server::commands::CommandSystem;
    use muo_server::mobile::AccessLevel;
    let learn: muo_server::commands::CommandEventHandler =
        Rc::new(|w, e| recipe_command(w, e, true));
    let forget: muo_server::commands::CommandEventHandler =
        Rc::new(|w, e| recipe_command(w, e, false));
    CommandSystem::register(w, "LearnAllRecipes", AccessLevel::GameMaster, &[], learn);
    CommandSystem::register(w, "ForgetAllRecipes", AccessLevel::GameMaster, &[], forget);
}

/// `DefX.Initialize()` of every system.
fn initialize(w: &mut World) {
    // ExpansionConfiguration: T2ACraftSystem.Enabled = GetSetting("t2aCraftMenus", !Core.UOTD)
    let default = w.expansion() < muo_data::expansion::Expansion::UOTD;
    state_mut(w).t2a = w.config.get_setting("t2aCraftMenus", default);
    ensure_built(w);
}

pub fn register(types: &mut TypeRegistry) {
    repair_deed::register(types);
    salvage_bag::register(types);
    types.register_phase_default(
        Phase::Configure,
        "Server.Engines.Craft.Configure",
        configure,
    );
    types.register_phase_default(
        Phase::Initialize,
        "Server.Engines.Craft.Initialize",
        initialize,
    );
}

#[cfg(test)]
mod t2a_jewel_tests;
