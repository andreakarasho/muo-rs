//! The [`SkillItemFns`](super::hooks::SkillItemFns) fields over what this crate owns: food, weapons, armor,
//! containers, corpses, houses (through the multis crate's `HouseOps`) and the thieves guild. [`configure`] fills
//! those fields only; the crates that own potions, traps, factions, books, instruments and spells fill theirs.

use muo_primitives::Point3D;
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::map::MapId;
use muo_server::mobile::part1::props_b;

use super::hooks::{ArmorFacts, CorpseFacts, SkillHooks, WeaponFacts, WeaponKind, WornArmor};
use crate::aos::{AosArmorAttribute, AosAttribute, AttrKind};
use crate::items::{self, armor, cont, corpse, food, weapon};
use crate::player_mobile::{self, NpcGuild};
use crate::regions::house::house_ops;

// ---- houses ----

fn find_house_at(w: &World, m: MobileId) -> Option<ItemId> {
    house_ops(w)?.find_house_at(w, m)
}

fn find_house_at_point(w: &World, p: Point3D, map: Option<MapId>, height: i32) -> Option<ItemId> {
    house_ops(w)?.find_house_at_point(w, p, map, height)
}

fn house_is_friend(w: &World, house: ItemId, m: MobileId) -> bool {
    house_ops(w).is_some_and(|o| o.is_friend(w, house, m))
}

fn is_inside_mobile(w: &World, house: ItemId, m: MobileId) -> bool {
    house_ops(w).is_some_and(|o| o.is_inside_mobile(w, house, m))
}

// ---- items ----

fn food_poisoned(w: &World, item: ItemId) -> Option<bool> {
    food::is_food(w, item).then(|| food::data(w, item).is_some_and(|d| d.poison.is_some()))
}

fn is_identifiable(w: &World, item: ItemId) -> bool {
    weapon::is_weapon(w, item) || armor::is_armor(w, item)
}

fn set_identified(w: &mut World, item: ItemId) -> bool {
    if weapon::is_weapon(w, item) {
        if let Some(d) = weapon::data_mut(w, item) {
            d.identified = true;
        }
        muo_server::item::ops::invalidate_properties(w, item);
        true
    } else if armor::is_armor(w, item) {
        armor::set_identified(w, item, true);
        true
    } else {
        false
    }
}

fn weapon_facts(w: &World, item: ItemId) -> Option<WeaponFacts> {
    if !weapon::is_weapon(w, item) {
        return None;
    }
    let d = weapon::data(w, item)?;
    let layer = w.item(item).map(|i| i.layer);
    Some(WeaponFacts {
        hit_points: d.hit_points,
        max_hit_points: d.max_hit_points,
        min_damage: weapon::min_damage(w, item),
        max_damage: weapon::max_damage(w, item),
        one_handed: layer == Some(muo_server::layer::Layer::OneHanded),
        kind: match weapon::weapon_type(w, item) {
            weapon::WeaponType::Slashing => WeaponKind::Slashing,
            weapon::WeaponType::Piercing => WeaponKind::Piercing,
            weapon::WeaponType::Bashing => WeaponKind::Bashing,
            weapon::WeaponType::Ranged => WeaponKind::Ranged,
            _ => WeaponKind::Other,
        },
        poisoned: d.poison.is_some() && d.poison_charges > 0,
    })
}

fn armor_facts(w: &World, item: ItemId) -> Option<ArmorFacts> {
    if !armor::is_armor(w, item) {
        return None;
    }
    let d = armor::data(w, item)?;
    Some(ArmorFacts {
        hit_points: d.hit_points,
        max_hit_points: d.max_hit_points,
        armor_rating: armor::armor_rating(w, item),
    })
}

fn trappable_container(w: &World, item: ItemId) -> Option<(bool, i32)> {
    if !w.item(item)?.ty.is(&cont::TRAPPABLE_CONTAINER) {
        return None;
    }
    let e = cont::ext(w, item)?;
    Some((e.trap_type != cont::TrapType::None, e.trap_power))
}

fn disarm_trappable_container(w: &mut World, item: ItemId) {
    if let Some(e) = cont::ext_mut(w, item) {
        e.trap_power = 0;
        e.trap_level = 0;
        e.trap_type = cont::TrapType::None;
    }
    muo_server::item::ops::invalidate_properties(w, item);
}

fn execute_trap(w: &mut World, item: ItemId, from: MobileId) -> bool {
    let is_trappable = w
        .item(item)
        .is_some_and(|i| i.ty.is(&cont::TRAPPABLE_CONTAINER));
    is_trappable && (cont::class(w, item).execute_trap)(w, item, from)
}

fn lockpickable_picker(w: &World, item: ItemId) -> Option<Option<String>> {
    if !w.item(item)?.ty.is(&cont::LOCKABLE_CONTAINER) {
        return None;
    }
    let picker = cont::ext(w, item)?.picker;
    Some(picker.and_then(|p| props_b::name(w, p)))
}

fn corpse_facts(w: &World, item: ItemId) -> Option<CorpseFacts> {
    if !corpse::is_corpse(w, item) {
        return None;
    }
    let d = corpse::data(w, item)?;
    let name = |m: MobileId| props_b::name(w, m).unwrap_or_default();
    // `((Body)c.Amount).IsHuman`
    let body = muo_primitives::Body(w.item(item).map_or(0, |i| i.amount) as _);
    Some(CorpseFacts {
        forensicist: d.forensicist.clone(),
        is_human_body: w
            .resources
            .get::<muo_primitives::BodyTable>()
            .is_some_and(|t| t.is_human(body)),
        killer_name: d.killer.map(name),
        looter_names: d.looters.iter().map(|&m| name(m)).collect(),
    })
}

fn set_corpse_forensicist(w: &mut World, item: ItemId, name: String) {
    if let Some(d) = corpse::data_mut(w, item) {
        d.forensicist = Some(name);
    }
}

fn is_thieves_guild_member(w: &World, m: MobileId) -> bool {
    player_mobile::data(w, m).is_some_and(|d| d.npc_guild == NpcGuild::ThievesGuild)
}

fn corpse_channeled(w: &World, item: ItemId) -> bool {
    corpse::get_flag(w, item, corpse::flag::CHANNELED)
}

fn channel_corpse(w: &mut World, item: ItemId) {
    corpse::set_flag(w, item, corpse::flag::CHANNELED, true);
    muo_server::item::base::set_hue(w, item, 0x835);
}

// ---- Stealth, Meditation ----

fn worn_armor(w: &World, m: MobileId) -> Vec<WornArmor> {
    let Some(mob) = w.mobile(m) else {
        return Vec::new();
    };
    mob.items
        .iter()
        .filter(|&&i| armor::is_armor(w, i))
        .map(|&i| {
            (
                armor::class(w, i).material_type as i32,
                armor::body_position(w, i) as i32,
                items::attr(w, i, AttrKind::Armor, AosArmorAttribute::MageArmor as u32) != 0,
            )
        })
        .collect()
}

fn armor_rating(w: &World, m: MobileId) -> f64 {
    muo_server::mobile::base::armor_rating_base(w, m)
}

fn okay_to_hold_while_meditating(w: &World, item: ItemId) -> bool {
    (weapon::is_weapon(w, item) || armor::is_armor(w, item))
        && items::attr(
            w,
            item,
            AttrKind::Attributes,
            AosAttribute::SpellChanneling as u32,
        ) != 0
}

/// Fills the fields of this crate's classes, and what `PlayerMobile` asks of them.
pub fn configure(w: &mut World) {
    let f = &mut w.resources.get_mut::<SkillHooks>().fns;
    f.find_house_at = Some(find_house_at);
    f.find_house_at_point = Some(find_house_at_point);
    f.house_is_friend = Some(house_is_friend);
    f.is_inside_mobile = Some(is_inside_mobile);
    f.food_poisoned = Some(food_poisoned);
    f.is_identifiable = Some(is_identifiable);
    f.set_identified = Some(set_identified);
    f.weapon_facts = Some(weapon_facts);
    f.armor_facts = Some(armor_facts);
    f.trappable_container = Some(trappable_container);
    f.disarm_trappable_container = Some(disarm_trappable_container);
    f.execute_trap = Some(execute_trap);
    f.lockpickable_picker = Some(lockpickable_picker);
    f.corpse_facts = Some(corpse_facts);
    f.set_corpse_forensicist = Some(set_corpse_forensicist);
    f.is_thieves_guild_member = Some(is_thieves_guild_member);
    f.corpse_channeled = Some(corpse_channeled);
    f.channel_corpse = Some(channel_corpse);
    f.worn_armor = Some(worn_armor);
    f.armor_rating = Some(armor_rating);
    f.okay_to_hold_while_meditating = Some(okay_to_hold_while_meditating);

    // What `PlayerMobile` asks of these systems.
    let s = w
        .resources
        .get_mut::<crate::player_mobile::hooks::SystemHooks>();
    s.item_is_stolen = Some(super::stealing::is_stolen_ref);
    s.item_is_trapped =
        Some(|w, item| trappable_container(w, item).is_some_and(|(trapped, _)| trapped));
    s.in_safe_zone = Some(super::common::in_safe_zone);

    // `food.Eat(from)` of the context menu.
    let c = w
        .resources
        .get_mut::<crate::context_menus::ContextMenuHooks>();
    c.is_food = Some(food::is_food);
    c.food_eat = Some(|w, item, from| {
        food::eat(w, item, from);
    });

    // `m.Guild as Guild` and `IsAlly` for the spells.
    let sp = crate::spells::hooks::hooks_mut(w);
    sp.guild_of = Some(|w, m| w.mobile(m).and_then(|x| x.guild).map(|g| g.0));
    sp.guild_is_ally = Some(|w, a, b| {
        crate::misc::guild_class::is_ally(
            w,
            muo_server::guild::GuildId(a),
            muo_server::guild::GuildId(b),
        )
    });
}
