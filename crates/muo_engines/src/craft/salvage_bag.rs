//! `SalvageBag`: a bag that resmelts the metal armor and weapons in it and cuts up the
//! cloth and leather, through the Blacksmithy craft tables and the scissors. Its place is here (not in muo_items)
//! because it reads `DefBlacksmithy.CraftSystem`.

use muo_content::craft::{CraftResource, CraftResourceType, resource_type, resource_types};
use muo_content::items::cont::{BAG, new_base_container};
use muo_content::items::interfaces;
use muo_content::items::{armor, is_named, weapon};
use muo_primitives::utility;
use muo_server::World;
use muo_server::context_menu_entry::ContextMenuEntry;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ItemType, base, deps, ops};
use muo_server::skills::SkillName;
use muo_server::type_registry::TypeRegistry;
use std::rc::Rc;

/// `SalvageBag(hue)`.
pub fn salvage_bag_new_hue(w: &mut World, hue: i32) -> ItemId {
    let id = new_base_container(w, &SALVAGE_BAG, 0xE76);
    muo_items::simple::hue(w, id, hue);
    id
}

/// `SalvageBag()`: `this(Utility.RandomBlueHue())`.
pub fn salvage_bag_new(w: &mut World) -> ItemId {
    let hue = utility::random_blue_hue(&mut w.rng);
    salvage_bag_new_hue(w, hue)
}

fn children(w: &World, id: ItemId) -> Vec<ItemId> {
    w.item(id).map(|i| i.items.clone()).unwrap_or_default()
}

fn is_a(w: &World, item: ItemId, name: &str) -> bool {
    w.item(item).is_some_and(|i| is_named(i.ty, name))
}

fn metal_armor_or_weapon(w: &World, item: ItemId) -> Option<CraftResource> {
    let i = w.item(item)?;
    if i.ty.is(&weapon::BASE_WEAPON) {
        weapon::data(w, item).map(|d| d.resource)
    } else if i.ty.is(&armor::BASE_ARMOR) {
        armor::data(w, item).map(|d| d.resource)
    } else {
        None
    }
}

/// `Resmeltables()`: the `foreach` returns on the first item, so only that one is looked at.
fn resmeltables(w: &World, id: ItemId) -> bool {
    let Some(&i) = children(w, id).first() else {
        return false;
    };
    !ops::is_deleted(w, i)
        && (metal_armor_or_weapon(w, i)
            .is_some_and(|r| resource_type(r) == CraftResourceType::Metal)
            || is_a(w, i, "Server.Items.DragonBardingDeed"))
}

/// `Scissorables()`.
fn scissorables(w: &World, id: ItemId) -> bool {
    children(w, id).into_iter().any(|i| {
        if !interfaces::is_scissorable(w, i) || ops::is_deleted(w, i) {
            return false;
        }
        is_a(w, i, "Server.Items.BaseClothing")
            || is_a(w, i, "Server.Items.Cloth")
            || is_a(w, i, "Server.Items.BoltOfCloth")
            || is_a(w, i, "Server.Items.Hides")
            || is_a(w, i, "Server.Items.BonePile")
            || (is_a(w, i, "Server.Items.BaseArmor")
                && metal_armor_or_weapon(w, i)
                    .is_some_and(|r| resource_type(r) == CraftResourceType::Leather))
    })
}

/// `Resmelt(from, item, resource)`; `failure` is `m_Failure`.
fn resmelt(
    w: &mut World,
    bag: ItemId,
    from: MobileId,
    item: ItemId,
    resource: CraftResource,
    failure: &mut bool,
) -> bool {
    let _ = bag;
    if resource_type(resource) != CraftResourceType::Metal {
        return false;
    }
    let Some(&ingot_type) = resource_types(resource).first() else {
        return false;
    };
    let Some(ty) = w.item(item).map(|i| i.ty.full_name) else {
        return false;
    };
    let Some(system) = crate::craft::system(w, "Blacksmithy") else {
        return false;
    };
    let Some(idx) = system.search_for(ty) else {
        return false;
    };
    let Some(craft_resource) = system.items[idx].resources.first() else {
        return false;
    };
    if craft_resource.amount < 2 {
        return false; // Not enough metal to resmelt
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
    let Some(ingot) = muo_content::factory::make_item(w, ingot_type) else {
        return false;
    };
    let mining_value = muo_server::skills::value_ref(w, from, SkillName::Mining);
    if w.item(item).is_some_and(|i| i.player_constructed) {
        let mining = mining_value.min(100.0);
        let amount = ((4.0 + mining) * f64::from(craft_resource.amount) - 4.0) * 0.0068;
        ops::set_amount(w, ingot, if amount < 2.0 { 2 } else { amount as i32 });
    } else {
        ops::set_amount(w, ingot, 2);
    }
    if difficulty > mining_value {
        *failure = true;
        base::delete(w, ingot);
    } else {
        base::delete(w, item);
    }
    if !ops::is_deleted(w, ingot) {
        muo_server::mobile::part2::items::add_to_backpack(w, from, ingot);
    }
    muo_content::mx::play_sound(w, from, 0x2A);
    muo_content::mx::play_sound(w, from, 0x240);
    true
}

/// `SalvageIngots(from)`.
fn salvage_ingots(w: &mut World, bag: ItemId, from: MobileId) {
    let pack = muo_server::mobile::part1::props_b::backpack(w, from);
    let has_tool = pack.is_some_and(|p| {
        !muo_server::item::enumerable::find_items(w, p, true, |w, i| {
            muo_content::items::tool::is_tool(w, i)
                && muo_content::items::tool::class(w, i).craft_system == "Blacksmithy"
        })
        .is_empty()
    });
    if !has_tool {
        deps::send_localized_message(w, from, 1079822); // You need a blacksmithing tool in order to salvage ingots.
        return;
    }
    let (_, forge) = crate::craft::defs::check_anvil_and_forge(w, from, 2);
    if !forge {
        deps::send_localized_message(w, from, 1044265); // You must be near a forge.
        return;
    }
    let (mut salvaged, mut not_salvaged, mut failure) = (0, 0, false);
    for item in children(w, bag) {
        if ops::is_deleted(w, item) {
            continue;
        }
        let ok = match metal_armor_or_weapon(w, item) {
            Some(r) => resmelt(w, bag, from, item, r, &mut failure),
            None => is_a(w, item, "Server.Items.DragonBardingDeed"),
        };
        if ok {
            salvaged += 1;
        } else {
            not_salvaged += 1;
        }
    }
    if failure {
        deps::send_localized_message(w, from, 1079975); // You failed to smelt some metal for lack of skill.
    } else {
        // Salvaged: ~1_COUNT~/~2_NUM~ blacksmithed items
        muo_items::special::util::msg_args(
            w,
            from,
            1079973,
            &format!("{salvaged}\t{}", salvaged + not_salvaged),
        );
    }
}

/// `SalvageCloth(from)`.
fn salvage_cloth(w: &mut World, bag: ItemId, from: MobileId) {
    const CLOTH_TYPES: [&str; 7] = [
        "Server.Items.Leather",
        "Server.Items.Cloth",
        "Server.Items.SpinedLeather",
        "Server.Items.HornedLeather",
        "Server.Items.BarbedLeather",
        "Server.Items.Bandage",
        "Server.Items.Bone",
    ];
    let pack = muo_server::mobile::part1::props_b::backpack(w, from);
    let scissors =
        pack.and_then(|p| muo_content::factory::find_item_by_type(w, p, "Server.Items.Scissors"));
    let Some(scissors) = scissors else {
        deps::send_localized_message(w, from, 1079823); // You need scissors in order to salvage cloth.
        return;
    };
    let (mut salvaged, mut not_salvaged) = (0, 0);
    for item in children(w, bag) {
        if !interfaces::is_scissorable(w, item) {
            continue;
        }
        if muo_items::skill_items::dyes::can_scissor(w, from, item)
            && interfaces::scissor(w, item, from, scissors) == Some(true)
        {
            salvaged += 1;
        } else {
            not_salvaged += 1;
        }
    }
    // Salvaged: ~1_COUNT~/~2_NUM~ tailored items
    muo_items::special::util::msg_args(
        w,
        from,
        1079974,
        &format!("{salvaged}\t{}", salvaged + not_salvaged),
    );
    for item in children(w, bag) {
        if w.item(item)
            .is_some_and(|i| CLOTH_TYPES.contains(&i.ty.full_name))
        {
            muo_server::mobile::part2::items::add_to_backpack(w, from, item);
        }
    }
}

fn salvage_all(w: &mut World, bag: ItemId, from: MobileId) {
    salvage_ingots(w, bag, from);
    salvage_cloth(w, bag, from);
}

/// One of the three entries: `number`, whether it is enabled and what a click does.
fn entry(number: i32, enabled: bool, act: fn(&mut World, ItemId, MobileId)) -> ContextMenuEntry {
    let mut e = ContextMenuEntry::new(number, -1);
    e.enabled = enabled;
    e.on_click = Some(Rc::new(move |w: &mut World, from, target| {
        if let EntityId::Item(bag) = target
            && muo_server::mobile::part2::stats::check_alive(w, from, true)
            && !ops::is_deleted(w, bag)
            && w.item(bag).is_some_and(|i| i.ty.is(&SALVAGE_BAG))
        {
            act(w, bag, from);
        }
    }));
    e
}

/// `Server.Items.SalvageBag` (`[SerializationGenerator(0)]`).
pub static SALVAGE_BAG: ItemType = ItemType {
    full_name: "Server.Items.SalvageBag",
    parent: Some(&BAG),
    construct: Some(salvage_bag_new),
    default_weight: |w, id| {
        crate::template_keys::item_float(w, id, "defaultWeight", BAG.default_weight)
    },
    label_number: |w, id| crate::template_keys::item_int(w, id, "labelNumber", BAG.label_number),
    get_context_menu_entries: |w, id, from, list| {
        (BAG.get_context_menu_entries)(w, id, from, list);
        if w.mobile(from).is_some_and(|m| (m.ty.alive)(w, from)) {
            let pack = muo_server::mobile::part1::props_b::backpack(w, from).map(EntityId::Item);
            let in_pack = ops::is_child_of(w, id, pack);
            let resmelt = in_pack && resmeltables(w, id);
            let scissor = in_pack && scissorables(w, id);
            list.push(entry(6277, resmelt, salvage_ingots));
            list.push(entry(6278, scissor, salvage_cloth));
            list.push(entry(6276, resmelt && scissor, salvage_all));
        }
    },
    serialize: |w, id, out| {
        (BAG.serialize)(w, id, out);
        out.write_encoded_int(0);
    },
    deserialize: |w, id, r| {
        (BAG.deserialize)(w, id, r)?;
        let _version = r.read_encoded_int()?;
        Ok(())
    },
    ..BAG
};

pub fn register(types: &mut TypeRegistry) {
    types.register_item(&SALVAGE_BAG, &[]);
}
