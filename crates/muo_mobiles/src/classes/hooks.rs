//! The hook fields other crates left for classes of Mobiles/{Monsters,Animals} (Configure phase).

use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};

/// `targeted is ICarvable carvable` then `carvable.Carve(from, blade)`: the items first, then the one mobile (Sheep).
fn carvable_carve(w: &mut World, target: EntityId, from: MobileId, blade: ItemId) -> bool {
    match target {
        EntityId::Item(i) => muo_content::items::interfaces::carve(w, i, from, blade),
        EntityId::Mobile(m) if muo_content::factory::mobile_is(w, m, "Server.Mobiles.Sheep") => {
            crate::hand::sheep_carve(w, m, from, blade);
            true
        }
        EntityId::Mobile(m)
            if muo_content::factory::mobile_is(w, m, "Server.Mobiles.PlagueBeastLord") =>
        {
            crate::hand::plague_beast_lord_carve(w, m, from, blade);
            true
        }
        _ => false,
    }
}

pub fn configure(w: &mut World) {
    super::monsters5::configure(w);
    super::ethereal::configure(w);
    // PlagueBeastLord: the Mutation Core items' view of it, and the paralyze spell's.
    muo_items::special::hooks::hooks_mut(w).plague_lord =
        Some(muo_items::special::hooks::PlagueLordApi {
            is_lord: crate::hand::plague_beast_lord_is_lord,
            opened_by: crate::hand::plague_beast_lord_opened_by,
            unfreeze: crate::hand::plague_beast_lord_unfreeze,
        });
    w.resources
        .get_mut::<muo_content::spells::hooks::SpellHooks>()
        .plague_beast_lord_on_paralyzed = Some(|w, m, from| {
        let lord = crate::hand::plague_beast_lord_is_lord(w, m);
        if lord {
            crate::hand::plague_beast_lord_on_paralyzed(w, m, from);
        }
        lord
    });
    muo_content::hooks::hooks_mut(w).new_acid = Some(|w, duration, tick, min, max| {
        muo_items::misc::small::acid_new(w, &muo_items::misc::small::ACID, duration, tick, min, max)
    });
    muo_content::items::common::hooks_mut(w).devour = Some(crate::hand::devour_hook);
    let bev = w
        .resources
        .get_mut::<muo_items::food::beverage::BeverageHooks>();
    bev.cow_try_milk = Some(cow_try_milk);
    // `EndlessDecanter.HandleThrow`: the water elemental's `HasDecanter`
    bev.elemental_has_decanter = Some(|w, m| {
        muo_content::factory::mobile_is(w, m, "Server.Mobiles.WaterElemental").then(|| {
            w.mobile(m)
                .and_then(|x| x.ext.get::<crate::hand::HasDecanter>())
                .is_some_and(|d| d.0)
        })
    });
    bev.elemental_take_decanter = Some(|w, m| {
        if let Some(x) = w.mobile_mut(m) {
            x.ext.insert(crate::hand::HasDecanter(false));
        }
    });
    let b = w
        .resources
        .get_mut::<muo_items::last::deeds::BardingHooks>();
    b.pet_state = Some(super::swamp_dragon::deed_pet_state);
    b.apply = Some(super::swamp_dragon::apply_deed_barding);
    w.resources
        .get_mut::<muo_content::hooks::ContentHooks>()
        .swamp_dragon_absorb = Some(super::swamp_dragon::barding_absorb);
    // MeerMage.StopEffect: AOS.Damage with pure fire, a lit torch, and both death events.
    w.resources
        .get_mut::<muo_content::hooks::ContentHooks>()
        .meer_mage_stop = Some(crate::hand::meer_mage_stop_effect);
    w.resources
        .get_mut::<muo_items::lights::base::LightHooks>()
        .meer_mage_stop = Some(crate::hand::meer_mage_stop_effect);
    muo_content::creature::events::subscribe_death(w, |w, m| {
        crate::hand::meer_mage_stop_effect(w, m, false)
    });
    muo_content::player_mobile::subscribe_death(w, |w, m| {
        crate::hand::meer_mage_stop_effect(w, m, false)
    });
    let t = w.resources.get_mut::<muo_content::targets::TargetHooks>();
    t.carvable_carve = Some(carvable_carve);
    t.swamp_dragon_remove_barding = Some(super::swamp_dragon::remove_barding);
    w.resources
        .get_mut::<muo_content::skills::hooks::SkillHooks>()
        .fns
        .swamp_dragon_barding = Some(super::swamp_dragon::barding_facts);
}

/// `Cow.TryMilk(from)`; None when the mobile is no cow (the signature of `BeverageHooks::cow_try_milk`).
pub fn cow_try_milk(w: &mut World, cow: MobileId, from: MobileId) -> Option<bool> {
    crate::hand::cow_try_milk(w, cow, from)
}
