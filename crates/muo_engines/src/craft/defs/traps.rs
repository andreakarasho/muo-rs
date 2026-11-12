//! The `CustomCraft` classes of Tinkering (`TrapCraft` and its three trap kinds) and `CustomCraft` itself:
//! crafting that targets something instead of making an item.

use std::rc::Rc;

use muo_content::items::cont::{self, LOCKABLE_CONTAINER, TrapType};
use muo_content::mx;
use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::deps;
use muo_server::skills::{self, SkillName};
use muo_server::target::{self, Target, TargetCancelType, TargetFlags, Targeted};

use crate::craft::model::*;
use crate::craft::{item as craft_item, state::t2a_enabled};

/// A `TrapCraft` in progress: the trap it sets and the container it was aimed at.
#[derive(Clone, Debug)]
pub struct TrapCraft {
    pub trap_type: TrapType,
    pub container: Option<ItemId>,
}

/// `typeof(CustomCraft).IsAssignableFrom(type)`: the trap each craft class sets.
pub fn custom_craft(item_type: &str) -> Option<TrapType> {
    match item_type {
        "Server.Engines.Craft.DartTrapCraft" => Some(TrapType::DartTrap),
        "Server.Engines.Craft.PoisonTrapCraft" => Some(TrapType::PoisonTrap),
        "Server.Engines.Craft.ExplosionTrapCraft" => Some(TrapType::ExplosionTrap),
        _ => None,
    }
}

fn verify(w: &mut World, from: MobileId, container: Option<ItemId>) -> i32 {
    let Some(c) = container.filter(|c| w.item(*c).is_some_and(|i| i.ty.is(&LOCKABLE_CONTAINER)))
    else {
        return 1005638; // You can only trap lockable chests.
    };
    let Some(e) = cont::ext(w, c) else {
        return 1005638;
    };
    if e.key_value == 0 {
        return 1005638; // You can only trap lockable chests.
    }
    let trap_type = e.trap_type;

    let (cmap, cloc) = (
        w.item(c).and_then(|i| i.map),
        muo_server::item::ops::get_world_location(w, c),
    );
    if mx::map(w, from) != cmap || !mx::in_range_point(w, from, cloc, 2) {
        return 500446; // That is too far away.
    }
    if !w.item(c).is_some_and(|i| i.movable()) {
        return 502944; // You cannot trap this item because it is locked down.
    }
    let ty = w.item(c).map(|i| i.ty);
    if let Some(ty) = ty
        && !(ty.is_accessible_to)(w, c, from)
    {
        return 502946; // That belongs to someone else.
    }
    if cont::locked(w, c) {
        return 502943; // You can only trap an unlocked object.
    }
    if trap_type != TrapType::None {
        return 502945; // You can only place one trap on an object at a time.
    }
    0
}

/// `EndCraftAction()`: ask what to trap.
#[allow(clippy::too_many_arguments)]
pub fn end_craft_action(
    w: &mut World,
    from: MobileId,
    sys: &Rc<CraftSystem>,
    index: usize,
    trap_type: TrapType,
    type_res: Option<&'static str>,
    tool: Option<ItemId>,
    quality: i32,
) {
    deps::send_localized_message(w, from, 502921); // What would you like to set a trap on?
    let mut t = Target::new(w, -1, false, TargetFlags::NONE);
    let (sys1, sys2) = (sys.clone(), sys.clone());
    t.handlers.on_target = Some(Rc::new(
        move |w: &mut World, from: MobileId, targeted: Targeted| {
            let c = match targeted {
                Targeted::Entity(EntityId::Item(i)) => Some(i),
                _ => None,
            };
            let message = verify(w, from, c);
            if message > 0 {
                failure(w, from, &sys1, tool, message);
                return;
            }
            let trap = TrapCraft {
                trap_type,
                container: c,
            };
            craft_item::complete_craft(
                w,
                from,
                &sys1,
                index,
                quality,
                false,
                type_res,
                tool,
                None,
                Some(&trap),
            );
        },
    ));
    t.handlers.on_target_cancel = Some(Rc::new(
        move |w: &mut World, from: MobileId, cancel: TargetCancelType| {
            if cancel == TargetCancelType::Canceled {
                failure(w, from, &sys2, tool, 0);
            }
        },
    ));
    let id = target::add_target(w, t);
    target::set_mobile_target(w, from, Some(id));
}

fn failure(
    w: &mut World,
    from: MobileId,
    sys: &Rc<CraftSystem>,
    tool: Option<ItemId>,
    message: i32,
) {
    if craft_item::tool_usable(w, tool) {
        craft_item::show_craft_menu(w, from, sys, tool, Some(Td::of_number(message)));
    } else if message > 0 {
        deps::send_localized_message(w, from, message);
    }
    let _ = t2a_enabled(w);
}

/// `CompleteCraft(out message)`: sets the trap; no item is made.
pub fn complete(w: &mut World, from: MobileId, c: &TrapCraft, message: &mut i32) -> Option<ItemId> {
    *message = verify(w, from, c.container);
    if *message == 0 {
        let tinkering = skills::value(w, from, SkillName::Tinkering);
        let trap_level = (tinkering / 10.0) as i32;
        if let Some(e) = c.container.and_then(|i| cont::ext_mut(w, i)) {
            e.trap_type = c.trap_type;
            e.trap_power = trap_level * 9;
            e.trap_level = trap_level;
            e.trap_on_lockpick = true;
        }
        *message = 1005639; // Trap is disabled until you lock the chest.
    }
    None
}
