//! `Resmelt`: smelting a crafted weapon or armor back into ingots.

use std::rc::Rc;

use muo_content::craft::{self as rc, CraftResource, CraftResourceType};
use muo_content::items::{armor, equip_hooks, weapon};
use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::deps;
use muo_server::skills::{self, SkillName};
use muo_server::target::{self, Target, TargetFlags, Targeted};

use super::item::{self as craft_item};
use super::model::*;

/// `SmeltResult`.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SmeltResult {
    Success,
    Invalid,
    NoSkill,
}

/// `Resmelt.Do(from, craftSystem, tool)`.
pub fn do_resmelt(w: &mut World, from: MobileId, sys: &Rc<CraftSystem>, tool: Option<ItemId>) {
    let num = (sys.vt.can_craft)(w, from, tool, None);
    if num > 0 && num != 1044267 {
        craft_item::show_craft_menu(w, from, sys, tool, Some(Td::of_number(num)));
        return;
    }
    let sys2 = sys.clone();
    let mut t = Target::new(w, 2, false, TargetFlags::NONE);
    t.handlers.on_target = Some(Rc::new(
        move |w: &mut World, from: MobileId, targeted: Targeted| {
            on_target(w, from, &sys2, tool, targeted);
        },
    ));
    let id = target::add_target(w, t);
    target::set_mobile_target(w, from, Some(id));
    deps::send_localized_message(w, from, 1044273); // Target an item to recycle.
}

/// The `Resmelt(from, item, resource)` of the target.
fn smelt(
    w: &mut World,
    sys: &CraftSystem,
    from: MobileId,
    item: ItemId,
    resource: CraftResource,
) -> SmeltResult {
    if equip_hooks::hooks(w)
        .and_then(|h| h.ethic_is_imbued)
        .is_some_and(|f| f(w, item))
    {
        return SmeltResult::Invalid;
    }
    if rc::resource_type(resource) != CraftResourceType::Metal {
        return SmeltResult::Invalid;
    }
    let types = rc::resource_types(resource);
    if types.is_empty() {
        return SmeltResult::Invalid;
    }
    let Some(ty) = w.item(item).map(|i| i.ty.full_name) else {
        return SmeltResult::Invalid;
    };
    let Some(ci) = sys.search_for(ty) else {
        return SmeltResult::Invalid;
    };
    let Some(craft_resource) = sys.items[ci].resources.first() else {
        return SmeltResult::Invalid;
    };
    if craft_resource.amount < 2 {
        return SmeltResult::Invalid; // Not enough metal to resmelt
    }

    let difficulty = match resource {
        CraftResource::DullCopper => 65.0,
        CraftResource::ShadowIron => 70.0,
        CraftResource::Copper => 75.0,
        CraftResource::Bronze => 80.0,
        CraftResource::Gold => 85.0,
        CraftResource::Agapite => 90.0,
        CraftResource::Verite => 95.0,
        CraftResource::Valorite => 99.0,
        _ => 0.0,
    };
    if difficulty > skills::value(w, from, SkillName::Mining) {
        return SmeltResult::NoSkill;
    }

    let Some(ingot) = muo_content::factory::make_item(w, types[0]) else {
        return SmeltResult::Invalid;
    };
    let player_constructed = w.item(item).is_some_and(|i| i.player_constructed);
    let deed = muo_content::factory::item_is(w, item, "Server.Items.DragonBardingDeed");
    // `item is DragonBardingDeed || item is BaseArmor armor && armor.PlayerConstructed || ..`
    let amount = if deed || player_constructed {
        craft_resource.amount / 2
    } else {
        1
    };
    muo_server::item::ops::set_amount(w, ingot, amount);

    if let Some(t) = w.item(item).map(|i| i.ty) {
        (t.delete)(w, item);
    }
    muo_server::mobile::part2::items::add_to_backpack(w, from, ingot);

    muo_server::mobile::part2::effects::play_sound(w, from, 0x2A);
    muo_server::mobile::part2::effects::play_sound(w, from, 0x240);
    SmeltResult::Success
}

fn on_target(
    w: &mut World,
    from: MobileId,
    sys: &Rc<CraftSystem>,
    tool: Option<ItemId>,
    targeted: Targeted,
) {
    let mut num = (sys.vt.can_craft)(w, from, tool, None);
    if num > 0 {
        if num == 1044267 {
            let (anvil, forge) = super::defs::check_anvil_and_forge(w, from, 2);
            if !anvil {
                num = 1044266; // You must be near an anvil
            } else if !forge {
                num = 1044265; // You must be near a forge.
            }
        }
        craft_item::show_craft_menu(w, from, sys, tool, Some(Td::of_number(num)));
        return;
    }

    let mut result = SmeltResult::Invalid;
    let mut store_bought = false;
    if let Targeted::Entity(EntityId::Item(item)) = targeted
        && let Some(ty) = w.item(item).map(|i| i.ty)
    {
        if ty.is(&armor::BASE_ARMOR) {
            let resource = armor::data(w, item).map_or(CraftResource::None, |d| d.resource);
            store_bought = !w.item(item).is_some_and(|i| i.player_constructed);
            result = smelt(w, sys, from, item, resource);
        } else if ty.is(&weapon::BASE_WEAPON) {
            let resource = weapon::data(w, item).map_or(CraftResource::None, |d| d.resource);
            store_bought = !w.item(item).is_some_and(|i| i.player_constructed);
            result = smelt(w, sys, from, item, resource);
        } else if super::item::derives(w, ty.full_name, "Server.Items.DragonBardingDeed") {
            let resource = super::state::state(w)
                .and_then(|s| s.deed_resource)
                .map_or(CraftResource::None, |f| f(w, item));
            result = smelt(w, sys, from, item, resource);
        }
    }

    let message = match result {
        SmeltResult::Invalid => 1044272,
        SmeltResult::NoSkill => 1044269,
        SmeltResult::Success => {
            if store_bought {
                500418
            } else {
                1044270
            }
        }
    };
    craft_item::show_craft_menu(w, from, sys, tool, Some(Td::of_number(message)));
}
