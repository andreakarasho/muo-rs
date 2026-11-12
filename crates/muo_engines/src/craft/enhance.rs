//! `Enhance`: giving a plain weapon or armor the properties of a special resource.
//!
//! `IArcaneEquip.IsArcane`: only the arcane gloves are armor that implements it, and they are refused.

use std::rc::Rc;

use muo_content::aos::{AosArmorAttribute, AosAttribute, AosWeaponAttribute, AttrKind};
use muo_content::craft::{self as rc, CraftResource};
use muo_content::items::{self, armor, weapon, weapon_combat};
use muo_primitives::utility;
use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{deps, ops};
use muo_server::layer::Layer;
use muo_server::skills;
use muo_server::target::{self, Target, TargetFlags, Targeted};

use super::item::{self as craft_item, ConsumeType};
use super::model::*;

/// `EnhanceResult`.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum EnhanceResult {
    None,
    NotInBackpack,
    BadItem,
    BadResource,
    AlreadyEnhanced,
    Success,
    Failure,
    Broken,
    NoResources,
    NoSkill,
}

/// `Enhance.CheckResult(ref res, chance)`.
type ResistFn = fn(&World, ItemId) -> i32;

fn check_result(w: &mut World, res: &mut EnhanceResult, chance: i32) {
    if *res != EnhanceResult::Success {
        return; // we've already failed..
    }
    let random = utility::random(&mut w.rng, 100);
    if random < 10 {
        *res = EnhanceResult::Failure;
    } else if chance > random {
        *res = EnhanceResult::Broken;
    }
}

/// `Enhance.Invoke(from, craftSystem, tool, item, resource, resType, ref resMessage)`.
#[allow(clippy::too_many_arguments)]
pub fn invoke(
    w: &mut World,
    from: MobileId,
    sys: &Rc<CraftSystem>,
    tool: Option<ItemId>,
    item: ItemId,
    resource: CraftResource,
    res_type: &'static str,
    res_message: &mut Option<Td>,
) -> EnhanceResult {
    let Some(ty) = w.item(item).map(|i| i.ty) else {
        return EnhanceResult::BadItem;
    };
    let pack = deps::find_item_on_layer(w, from, Layer::Backpack);
    if !pack.is_some_and(|p| ops::is_child_of(w, item, Some(EntityId::Item(p)))) {
        return EnhanceResult::NotInBackpack;
    }
    let is_weapon = ty.is(&weapon::BASE_WEAPON);
    let is_armor = ty.is(&armor::BASE_ARMOR);
    if !is_weapon && !is_armor {
        return EnhanceResult::BadItem;
    }
    // `item is IArcaneEquip eq && eq.IsArcane`: the arcane gloves (the only BaseArmor that is one).
    if is_armor && muo_items::armor::special::is_arcane(w, item) {
        return EnhanceResult::BadItem;
    }
    if rc::is_standard(resource) {
        return EnhanceResult::BadResource;
    }

    let num = (sys.vt.can_craft)(w, from, tool, Some(ty.full_name));
    if num > 0 {
        *res_message = Some(Td::of_number(num));
        return EnhanceResult::None;
    }

    let Some(ci) = sys.search_for(ty.full_name) else {
        return EnhanceResult::BadItem;
    };
    let craft = sys.items[ci].clone();
    if craft.resources.is_empty() {
        return EnhanceResult::BadItem;
    }

    let mut all_required = true;
    if craft_item::success_chance(w, sys, &craft, from, false, &mut all_required) <= 0.0 {
        return EnhanceResult::NoSkill;
    }

    if rc::resource_types(resource).is_empty() {
        return EnhanceResult::BadResource;
    }
    let Some(attributes) = rc::attribute_info(resource) else {
        return EnhanceResult::BadResource;
    };

    let (mut res_hue, mut max_amount) = (0, 0);
    if !craft_item::consume_res(
        w,
        from,
        sys,
        &craft,
        Some(res_type),
        &mut res_hue,
        &mut max_amount,
        ConsumeType::None,
        res_message,
        false,
    ) {
        return EnhanceResult::NoResources;
    }

    if sys.name == "Blacksmithy"
        && let Some(hammer) = deps::find_item_on_layer(w, from, Layer::OneHanded)
        && muo_content::factory::item_is(w, hammer, "Server.Items.AncientSmithyHammer")
    {
        let left = muo_content::items::tool::uses_remaining(w, hammer) - 1;
        muo_content::items::tool::set_uses_remaining(w, hammer, left);
        if left < 1
            && let Some(t) = w.item(hammer).map(|i| i.ty)
        {
            (t.delete)(w, hammer);
        }
    }

    let (phys, fire, cold, pois, nrgy, dura, luck, lreq, dinc);
    let (
        phys_bonus,
        fire_bonus,
        cold_bonus,
        nrgy_bonus,
        pois_bonus,
        dura_bonus,
        luck_bonus,
        lreq_bonus,
        dinc_bonus,
    );
    let base_chance = 20;

    if is_weapon {
        if !rc::is_standard(weapon::data(w, item).map_or(CraftResource::None, |d| d.resource)) {
            return EnhanceResult::AlreadyEnhanced;
        }
        (phys, fire, cold, pois, nrgy) = (0, 0, 0, 0, 0);
        dura = weapon::data(w, item).map_or(0, |d| d.max_hit_points);
        luck = items::attr(w, item, AttrKind::Attributes, AosAttribute::Luck as u32);
        lreq = items::attr(
            w,
            item,
            AttrKind::Weapon,
            AosWeaponAttribute::LowerStatReq as u32,
        );
        dinc = items::attr(
            w,
            item,
            AttrKind::Attributes,
            AosAttribute::WeaponDamage as u32,
        );
        phys_bonus = false;
        fire_bonus = attributes.weapon_fire_damage > 0;
        cold_bonus = attributes.weapon_cold_damage > 0;
        nrgy_bonus = attributes.weapon_energy_damage > 0;
        pois_bonus = attributes.weapon_poison_damage > 0;
        dura_bonus = attributes.weapon_durability > 0;
        luck_bonus = attributes.weapon_luck > 0;
        lreq_bonus = attributes.weapon_lower_requirements > 0;
        dinc_bonus = dinc > 0;
    } else {
        if !rc::is_standard(armor::data(w, item).map_or(CraftResource::None, |d| d.resource)) {
            return EnhanceResult::AlreadyEnhanced;
        }
        let res = |f: fn(&muo_server::item::ItemType) -> ResistFn, w: &World| f(ty)(w, item);
        phys = res(|t| t.physical_resistance, w);
        fire = res(|t| t.fire_resistance, w);
        cold = res(|t| t.cold_resistance, w);
        pois = res(|t| t.poison_resistance, w);
        nrgy = res(|t| t.energy_resistance, w);
        dura = armor::data(w, item).map_or(0, |d| d.max_hit_points);
        luck = items::attr(w, item, AttrKind::Attributes, AosAttribute::Luck as u32);
        lreq = items::attr(
            w,
            item,
            AttrKind::Armor,
            AosArmorAttribute::LowerStatReq as u32,
        );
        phys_bonus = attributes.armor_physical_resist > 0;
        fire_bonus = attributes.armor_fire_resist > 0;
        cold_bonus = attributes.armor_cold_resist > 0;
        nrgy_bonus = attributes.armor_energy_resist > 0;
        pois_bonus = attributes.armor_poison_resist > 0;
        dura_bonus = attributes.armor_durability > 0;
        luck_bonus = attributes.armor_luck > 0;
        lreq_bonus = attributes.armor_lower_requirements > 0;
        dinc = 0;
        dinc_bonus = false;
    }

    let skill = skills::value(w, from, sys.main_skill) as i32;
    let mut chance_base = base_chance;
    if skill >= 100 {
        chance_base -= (skill - 90) / 10;
    }

    let mut res = EnhanceResult::Success;
    if phys_bonus {
        check_result(w, &mut res, chance_base + phys);
    }
    if fire_bonus {
        check_result(w, &mut res, chance_base + fire);
    }
    if cold_bonus {
        check_result(w, &mut res, chance_base + cold);
    }
    if nrgy_bonus {
        check_result(w, &mut res, chance_base + nrgy);
    }
    if pois_bonus {
        check_result(w, &mut res, chance_base + pois);
    }
    if dura_bonus {
        check_result(w, &mut res, chance_base + dura / 40);
    }
    if luck_bonus {
        check_result(w, &mut res, chance_base + 10 + luck / 2);
    }
    if lreq_bonus {
        check_result(w, &mut res, chance_base + lreq / 4);
    }
    if dinc_bonus {
        check_result(w, &mut res, chance_base + dinc / 4);
    }

    match res {
        EnhanceResult::Broken => {
            if !craft_item::consume_res(
                w,
                from,
                sys,
                &craft,
                Some(res_type),
                &mut res_hue,
                &mut max_amount,
                ConsumeType::Half,
                res_message,
                false,
            ) {
                return EnhanceResult::NoResources;
            }
            (ty.delete)(w, item);
        }
        EnhanceResult::Success => {
            if !craft_item::consume_res(
                w,
                from,
                sys,
                &craft,
                Some(res_type),
                &mut res_hue,
                &mut max_amount,
                ConsumeType::All,
                res_message,
                false,
            ) {
                return EnhanceResult::NoResources;
            }
            if is_weapon {
                weapon::set_resource(w, item, resource);
                let hue = weapon_combat::elemental_damage_hue(w, item);
                if hue > 0 {
                    (ty.set_hue)(w, item, hue);
                }
            } else {
                armor::set_resource(w, item, resource);
            }
        }
        EnhanceResult::Failure
            if !craft_item::consume_res(
                w,
                from,
                sys,
                &craft,
                Some(res_type),
                &mut res_hue,
                &mut max_amount,
                ConsumeType::Half,
                res_message,
                false,
            ) =>
        {
            return EnhanceResult::NoResources;
        }
        _ => {}
    }
    res
}

/// `Enhance.BeginTarget(from, craftSystem, tool)`.
pub fn begin_target(w: &mut World, from: MobileId, sys: &Rc<CraftSystem>, tool: Option<ItemId>) {
    let Some(ctx) = super::context_of(w, sys.name, from) else {
        return;
    };
    let last = ctx.last_resource_index;
    let sub = &sys.sub_res;
    if last >= 0 && (last as usize) < sub.items.len() {
        let res = &sub.items[last as usize];
        if skills::value(w, from, sys.main_skill) < res.required_skill {
            craft_item::show_craft_menu(w, from, sys, tool, Some(res.message.clone()));
        } else {
            let resource = rc::from_type(res.item_type);
            if resource != CraftResource::None {
                let res_type = res.item_type;
                let sys2 = sys.clone();
                let mut t = Target::new(w, 2, false, TargetFlags::NONE);
                t.handlers.on_target = Some(Rc::new(
                    move |w: &mut World, from: MobileId, targeted: Targeted| {
                        let Targeted::Entity(EntityId::Item(item)) = targeted else {
                            return;
                        };
                        let mut message = None;
                        let r =
                            invoke(w, from, &sys2, tool, item, resource, res_type, &mut message);
                        let number = match r {
                            EnhanceResult::NotInBackpack => Some(1061005),
                            EnhanceResult::AlreadyEnhanced => Some(1061012),
                            EnhanceResult::BadItem => Some(1061011),
                            EnhanceResult::BadResource => Some(1061010),
                            EnhanceResult::Broken => Some(1061080),
                            EnhanceResult::Failure => Some(1061082),
                            EnhanceResult::Success => Some(1061008),
                            EnhanceResult::NoSkill => Some(1044153),
                            _ => None,
                        };
                        let message = number.map(Td::of_number).or(message);
                        craft_item::show_craft_menu(w, from, &sys2, tool, message);
                    },
                ));
                let id = target::add_target(w, t);
                target::set_mobile_target(w, from, Some(id));
                // Target an item to enhance with the properties of your selected material.
                deps::send_localized_message(w, from, 1061004);
            } else {
                craft_item::show_craft_menu(w, from, sys, tool, Some(Td::of_number(1061010)));
            }
        }
    } else {
        craft_item::show_craft_menu(w, from, sys, tool, Some(Td::of_number(1061010)));
    }
}
