//! `PackAnimal` and the overrides PackHorse and PackLlama share.

use muo_content::creature::data::read;
use muo_content::creature::{BASE_CREATURE, BASE_CREATURE_C, control};
use muo_server::World;
use muo_server::context_menu_entry::ContextMenuEntry;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::DeathMoveResult;
use muo_server::item::container::drop_item_default;
use muo_server::layer::Layer;
use muo_server::mobile::AccessLevel;
use muo_server::mobile::part2::items;

use super::ctors::si;

fn dead_pet(w: &World, animal: MobileId) -> bool {
    muo_content::creature::cd(w, animal).is_some_and(|d| d.is_dead_pet)
}

/// `var pack = Backpack; pack?.Delete(); pack = new StrongBackpack(); pack.Movable = false; AddItem(pack);`
pub fn replace_backpack(w: &mut World, id: MobileId) {
    if let Some(old) = items::find_item_on_layer(w, id, Layer::Backpack) {
        muo_server::item::base::delete(w, old);
    }
    if let Some(p) = muo_content::factory::make_item(w, &si("StrongBackpack")) {
        muo_server::item::ops::set_movable(w, p, false);
        items::add_item(w, id, p);
    }
}

/// `PackAnimal.CheckAccess(animal, from)`.
pub fn check_access(w: &World, animal: MobileId, from: MobileId) -> bool {
    if from == animal || muo_content::mx::access_level(w, from) >= AccessLevel::GameMaster {
        return true;
    }
    let controlled = muo_content::creature::cd(w, animal).is_some_and(|d| d.controlled);
    muo_content::mx::alive(w, from)
        && controlled
        && !dead_pet(w, animal)
        && (control::get_master(w, animal) == Some(from) || control::is_pet_friend(w, animal, from))
}

/// `PackAnimal.CombineBackpacks(animal)`.
pub fn combine_backpacks(w: &mut World, animal: MobileId) {
    if w.core_aos() {
        return;
    }
    if read(w, animal, false, |d| d.is_bonded) || dead_pet(w, animal) {
        return;
    }
    let Some(pack) = items::find_item_on_layer(w, animal, Layer::Backpack) else {
        return;
    };
    let Some(new_pack) = muo_content::factory::make_item(w, &si("Backpack")) else {
        return;
    };
    let children: Vec<ItemId> = w.item(pack).map(|p| p.items.clone()).unwrap_or_default();
    for &i in children.iter().rev() {
        drop_item_default(w, new_pack, i);
    }
    drop_item_default(w, pack, new_pack);
}

/// `PackAnimal.TryPackOpen(animal, from)`.
pub fn try_pack_open(w: &mut World, animal: MobileId, from: MobileId) {
    if dead_pet(w, animal) {
        return;
    }
    if let Some(item) = items::find_item_on_layer(w, animal, Layer::Backpack) {
        muo_server::mobile::base::use_item(w, from, item);
    }
}

/// `PackAnimal.GetContextMenuEntries(animal, from, ref list)`: `PackAnimalBackpackEntry(!animal.IsDeadPet)`.
pub fn context_menu_entries(
    w: &mut World,
    animal: MobileId,
    from: MobileId,
    list: &mut Vec<ContextMenuEntry>,
) {
    if check_access(w, animal, from) {
        let mut e = ContextMenuEntry::new(6145, 3);
        e.enabled = !dead_pet(w, animal);
        e.on_click = Some(std::rc::Rc::new(
            |w: &mut World, from: MobileId, target: EntityId| {
                if let EntityId::Mobile(bc) = target
                    && muo_content::creature::is_creature(w, bc)
                {
                    try_pack_open(w, bc, from);
                }
            },
        ));
        list.push(e);
    }
}

macro_rules! pack_animal_class {
    ($before:ident, $inv:ident, $snoop:ident, $drop:ident, $nl_drop:ident, $nl_lift:ident, $dbl:ident, $menu:ident) => {
        /// `if (!base.OnBeforeDeath()) return false; PackAnimal.CombineBackpacks(this); return true;`
        pub fn $before(w: &mut World, id: MobileId) -> bool {
            if !(BASE_CREATURE.on_before_death)(w, id) {
                return false;
            }
            combine_backpacks(w, id);
            true
        }
        /// `=> DeathMoveResult.MoveToCorpse`
        pub fn $inv(_w: &mut World, _id: MobileId, _item: ItemId) -> DeathMoveResult {
            DeathMoveResult::MoveToCorpse
        }
        /// `!PackAnimal.CheckAccess(this, from) && base.IsSnoop(from)`
        pub fn $snoop(w: &World, id: MobileId, from: MobileId) -> bool {
            !check_access(w, id, from) && (BASE_CREATURE.is_snoop)(w, id, from)
        }
        /// `if (CheckFeed(from, item)) return true; if (CheckAccess) { AddToBackpack(item); return true; }`
        pub fn $drop(w: &mut World, id: MobileId, from: MobileId, item: ItemId) -> bool {
            if (BASE_CREATURE_C.check_feed)(w, id, from, item) {
                return true;
            }
            if check_access(w, id, from) {
                items::add_to_backpack(w, id, item);
                return true;
            }
            (BASE_CREATURE.on_drag_drop)(w, id, from, item)
        }
        pub fn $nl_drop(
            w: &mut World,
            id: MobileId,
            from: MobileId,
            _item: ItemId,
            _target: ItemId,
        ) -> bool {
            check_access(w, id, from)
        }
        pub fn $nl_lift(w: &mut World, id: MobileId, from: MobileId, _item: ItemId) -> bool {
            check_access(w, id, from)
        }
        pub fn $dbl(w: &mut World, id: MobileId, from: MobileId) {
            try_pack_open(w, id, from);
        }
        pub fn $menu(
            w: &mut World,
            id: MobileId,
            from: MobileId,
            list: &mut Vec<ContextMenuEntry>,
        ) {
            (BASE_CREATURE.get_context_menu_entries)(w, id, from, list);
            context_menu_entries(w, id, from, list);
        }
    };
}
pack_animal_class!(
    pack_horse_on_before_death,
    pack_horse_get_inventory_move_result_for,
    pack_horse_is_snoop,
    pack_horse_on_drag_drop,
    pack_horse_check_nonlocal_drop,
    pack_horse_check_nonlocal_lift,
    pack_horse_on_double_click,
    pack_horse_get_context_menu_entries
);
pack_animal_class!(
    pack_llama_on_before_death,
    pack_llama_get_inventory_move_result_for,
    pack_llama_is_snoop,
    pack_llama_on_drag_drop,
    pack_llama_check_nonlocal_drop,
    pack_llama_check_nonlocal_lift,
    pack_llama_on_double_click,
    pack_llama_get_context_menu_entries
);
