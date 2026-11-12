//! Hooks whose target is in `muo_content` itself: assigned once, in `Configure`, so no feature silently vanishes
//! because a `*Hooks` field was read but never set (see `critical_hooks_are_set` in
//! `crates/muo/tests/whole.rs`).

use muo_server::World;
use muo_server::ids::{ItemId, MobileId};

use crate::creature::packing::PackHooks;
use crate::items::{armor, runic, slayer, weapon};
use crate::loot_pack::{LootHooks, OldWeaponRolls};

/// `AccountGold.Configure`: the settings (`accountGold.enable` defaults to `Core.TOL`) were never read, so a TOL server
/// answered `balance` with the plain gold text and kept gold in bank boxes as items.
fn configure_account_gold(w: &mut World) {
    let tol = crate::core::tol(w);
    let enabled = w.config.get_setting("accountGold.enable", tol);
    let on_bank = w.config.get_setting("accountGold.convertOnBank", true);
    let on_trade = w.config.get_setting("accountGold.convertOnTrade", false);
    let g = w.resources.get_mut::<muo_server::iaccount::AccountGold>();
    g.enabled = enabled;
    g.convert_on_bank = on_bank;
    g.convert_on_trade = on_trade;
}

pub fn configure(w: &mut World) {
    configure_account_gold(w);
    configure_loot(w);
    configure_world(w);
    configure_combat(w);
    configure_player(w);
}

pub fn pack_slayer(w: &mut World, item: ItemId, from: MobileId) {
    let name = w.mobile(from).map_or("", |m| m.ty.full_name);
    let s = slayer::loot_slayer_type(w, name);
    if let Some(d) = weapon::data_mut(w, item) {
        d.slayer = s as i32;
    }
}

fn configure_loot(w: &mut World) {
    let h = w.resources.get_mut::<LootHooks>();
    h.apply_aos_attributes = Some(|w, item, luck, props, min, max| {
        runic::apply_to_item(w, item, false, luck, props, min, max)
    });
    h.apply_old_weapon = Some(|w, item, r: OldWeaponRolls| {
        if let Some(v) = r.acc.and_then(weapon::WeaponAccuracyLevel::from_i32) {
            weapon::set_accuracy_level(w, item, v);
        }
        if let Some(v) = r.dmg.and_then(weapon::WeaponDamageLevel::from_i32) {
            weapon::set_damage_level(w, item, v);
        }
        if let Some(v) = r.dur.and_then(weapon::WeaponDurabilityLevel::from_i32) {
            weapon::set_durability_level(w, item, v);
        }
        if r.silver {
            if let Some(d) = weapon::data_mut(w, item) {
                d.slayer = slayer::SlayerName::Silver as i32;
            }
        } else if r.loot_slayer {
            let plain = weapon::data(w, item).is_some_and(|d| {
                d.accuracy_level as i32 == 0
                    && d.damage_level as i32 == 0
                    && d.durability_level as i32 == 0
                    && d.slayer == 0
            });
            if plain {
                pack_slayer(w, item, r.from);
            }
        }
    });
    h.apply_old_armor = Some(|w, item, prot, dur| {
        if let Some(v) = prot.and_then(armor::ArmorProtectionLevel::from_i32) {
            armor::set_protection_level(w, item, v);
        }
        if let Some(v) = dur.and_then(armor::ArmorDurabilityLevel::from_i32) {
            armor::set_durability_level(w, item, v);
        }
    });

    let p = w.resources.get_mut::<PackHooks>();
    p.apply_aos_attributes =
        Some(|w, item, count, min, max| runic::apply_to_item(w, item, false, 0, count, min, max));
    p.set_armor_levels = Some(|w, item, prot, dur| {
        if let Some(v) = armor::ArmorProtectionLevel::from_i32(prot) {
            armor::set_protection_level(w, item, v);
        }
        if let Some(v) = armor::ArmorDurabilityLevel::from_i32(dur) {
            armor::set_durability_level(w, item, v);
        }
    });
    p.set_weapon_levels = Some(|w, item, silver, dmg, acc, dur| {
        if silver && let Some(d) = weapon::data_mut(w, item) {
            d.slayer = slayer::SlayerName::Silver as i32;
        }
        if let Some(v) = weapon::WeaponDamageLevel::from_i32(dmg) {
            weapon::set_damage_level(w, item, v);
        }
        if let Some(v) = weapon::WeaponAccuracyLevel::from_i32(acc) {
            weapon::set_accuracy_level(w, item, v);
        }
        if let Some(v) = weapon::WeaponDurabilityLevel::from_i32(dur) {
            weapon::set_durability_level(w, item, v);
        }
    });
    // Instruments (muo_items) get their slayer in that crate's wiring; this is the weapon branch.
    p.set_loot_slayer = Some(|w, item, from| {
        if weapon::data(w, item).is_some() {
            pack_slayer(w, item, from);
        }
    });
}

fn jail_at(w: &World, region: muo_server::region::RegionId) -> bool {
    muo_server::region::get_region_of_type(w, region, &crate::systems::jail::JAIL_REGION).is_some()
}

fn configure_world(w: &mut World) {
    use crate::items::corpse;
    // AccountHandler: `Region.Find(m.LogoutLocation, m.LogoutMap).IsPartOf<JailRegion>()`.
    w.resources
        .get_mut::<crate::accounting::AccountHooks>()
        .in_jail_region = |w, m| {
        let Some(mob) = w.mobile(m) else { return false };
        jail_at(
            w,
            muo_server::region::find(w, mob.logout_location, mob.logout_map),
        )
    };
    let h = w.resources.get_mut::<crate::hooks::WorldHooks>();
    h.mount_rider = Some(crate::creature::mount::rider);
    h.in_guarded_region = Some(|w, m| {
        w.mobile(m)
            .is_some_and(|x| crate::systems::regions::get_guarded(w, x.region).is_some())
    });
    h.corpse_animated = Some(corpse::animated);
    h.corpse_set_carved = Some(|w, c| corpse::set_flag(w, c, corpse::flag::CARVED, true));
    h.corpse_add_carved_item = Some(corpse::add_carved_item);
    h.corpse_is_criminal_action = Some(corpse::is_criminal_action);

    let c = crate::items::common::hooks_mut(w);
    c.corpse_notoriety = Some(corpse::corpse_notoriety);
    c.same_party = Some(|w, a, b| {
        let f = crate::hooks::world_hooks(w).and_then(|h| h.party_members);
        f.is_some_and(|f| f(w, a).contains(&b))
    });
    c.criminal_action = Some(|w, m, flag| {
        if let Some(ty) = w.mobile(m).map(|x| x.ty) {
            (ty.criminal_action)(w, m, flag);
        }
    });
    c.award_karma = Some(crate::misc::titles::award_karma);
    c.creature_on_carve = Some(crate::creature::misc::on_carve);
}

/// `RegenRates.GetArmorMeditationValue(ar)`.
fn armor_meditation_value(w: &World, ar: Option<ItemId>) -> f64 {
    use crate::aos::{AosArmorAttribute, AosAttribute, AttrKind};
    use crate::items::armor::{
        ArmorMeditationAllowance as Med, base_armor_rating_scaled, is_armor, meditation_allowance,
    };
    let Some(ar) = ar.filter(|a| is_armor(w, *a)) else {
        return 0.0;
    };
    if crate::items::attr(w, ar, AttrKind::Armor, AosArmorAttribute::MageArmor as u32) != 0
        || crate::items::attr(
            w,
            ar,
            AttrKind::Attributes,
            AosAttribute::SpellChanneling as u32,
        ) != 0
    {
        return 0.0;
    }
    match meditation_allowance(w, ar) {
        Med::None => base_armor_rating_scaled(w, ar),
        Med::Half => base_armor_rating_scaled(w, ar) / 2.0,
        Med::All => 0.0,
    }
}

fn configure_combat(w: &mut World) {
    use muo_server::interfaces::WeaponRef;
    use muo_server::layer::Layer;
    use muo_server::mobile::part2::items::find_item_on_layer as on_layer;
    let h = crate::hooks::hooks_mut(w);
    h.weapon_on_hit = Some(|w, attacker, defender| {
        if let Some(WeaponRef::Item(i)) = muo_server::mobile::base::weapon(w, attacker)
            && weapon::data(w, i).is_some()
        {
            crate::items::weapon_combat::on_hit(w, i, attacker, defender, 1.0);
        }
    });
    h.uses_ranged_weapon = Some(|w, m| match muo_server::mobile::base::weapon(w, m) {
        Some(WeaponRef::Item(i)) => crate::factory::item_is(w, i, "Server.Items.BaseRanged"),
        _ => false,
    });
    h.weapon_max_range = Some(|w, m| match muo_server::mobile::base::weapon(w, m) {
        Some(WeaponRef::Item(i)) if weapon::data(w, i).is_some() => weapon::max_range(w, i),
        _ => 1,
    });
    // The loop in DestroyEquipment: Blessed/Newbied/BlessedFor, insured, mage weapons and durability.
    h.destroy_equipment_candidate = Some(|w, i| {
        use muo_server::item::LootType;
        let Some(it) = w.item(i) else { return false };
        if matches!(it.loot_type, LootType::Blessed | LootType::Newbied) || it.blessed_for.is_some()
        {
            return false;
        }
        if muo_server::item::deps::insurance_enabled(w) && it.insured() {
            return false;
        }
        let aos_item = weapon::data(w, i).is_some()
            || armor::data(w, i).is_some()
            || crate::items::clothing::data(w, i).is_some()
            || crate::items::jewel::data(w, i).is_some();
        if aos_item
            && crate::items::attr(
                w,
                i,
                crate::aos::AttrKind::Attributes,
                crate::aos::AosAttribute::SpellChanneling as u32,
            ) > 0
        {
            return false;
        }
        weapon::data(w, i)
            .map(|d| d.max_hit_points)
            .or_else(|| armor::data(w, i).map(|d| d.max_hit_points))
            .or_else(|| crate::items::clothing::data(w, i).map(|d| d.max_hit_points))
            .is_some_and(|hp| hp > 0)
    });
    let n = w
        .resources
        .get_mut::<crate::misc::notoriety::NotorietyHooks>();
    // `Notoriety.GetGuildFor(def, m)`, with its writes to the pet.
    n.guild_for = Some(|w, m| {
        let r = crate::misc::notoriety::resolve_guild_for(w, m);
        if r.clear_title {
            muo_server::mobile::part1::props_b::set_display_guild_title(w, m, false);
        }
        if let Some(g) = r.set_guild {
            muo_server::mobile::part1::props_b::set_guild(w, m, g);
        }
        r.guild
    });
    {
        let h = w
            .resources
            .get_mut::<crate::player_mobile::hooks::SystemHooks>();
        h.send_everything = Some(muo_server::mobile::part2::effects::send_everything);
        h.test_center_enabled = Some(crate::systems::special_systems::test_center::enabled);
        h.verify_profession = Some(crate::login::profession_info::verify_profession);
    }
    let n = w
        .resources
        .get_mut::<crate::misc::notoriety::NotorietyHooks>();
    // What `MobileNotoriety` reads of a `BaseCreature` (`target as BaseCreature`) and of vendors / criers.
    n.creature = Some(|w, m| {
        use crate::factory::mobile_is;
        let vendor_or_crier = mobile_is(w, m, "Server.Mobiles.PlayerVendor")
            || mobile_is(w, m, "Server.Mobiles.TownCrier");
        let (Some(d), Some(t)) = (
            crate::creature::data::cd(w, m),
            crate::creature::vtable::ctype(w, m),
        ) else {
            return crate::misc::notoriety::CreatureFacts {
                is_vendor_or_crier: vendor_or_crier,
                ..Default::default()
            };
        };
        crate::misc::notoriety::CreatureFacts {
            is_creature: true,
            controlled: d.controlled,
            control_master: crate::creature::control::control_master(w, m),
            guarding: d.control_order == crate::creature::enums::OrderType::Guard,
            control_target: d.control_target,
            summoned: d.summoned,
            summon_master: crate::creature::control::summon_master(w, m),
            always_murderer: (t.always_murderer)(w, m),
            is_invulnerable: (t.is_invulnerable)(w, m),
            always_attackable: (t.always_attackable)(w, m),
            initial_innocent: (t.initial_innocent)(w, m),
            is_animated_dead: crate::creature::misc::is_animated_dead(w, m),
            master: crate::creature::control::get_master(w, m),
            exempt_from_summoned_red: mobile_is(w, m, "Server.Mobiles.BaseFamiliar")
                || mobile_is(w, m, "Server.Mobiles.ArcaneFey")
                || mobile_is(w, m, "Server.Mobiles.Golem"),
            is_vendor_or_crier: vendor_or_crier,
        }
    });
    n.can_begin_polymorph = Some(|w, m| {
        muo_server::mobile::part1::combat::can_begin_action(
            w,
            m,
            "Server.Spells.Seventh.PolymorphSpell",
        )
    });
    w.resources
        .get_mut::<crate::misc::regen_rates::RegenHooks>()
        .armor_offset = Some(|w, m| {
        let mut rating = 0.0;
        if !crate::core::aos(w) {
            rating += armor_meditation_value(w, on_layer(w, m, Layer::TwoHanded));
        }
        rating += armor_meditation_value(w, on_layer(w, m, Layer::Neck));
        rating += armor_meditation_value(w, on_layer(w, m, Layer::Gloves));
        rating += armor_meditation_value(w, on_layer(w, m, Layer::Helm));
        rating += armor_meditation_value(w, on_layer(w, m, Layer::Arms));
        rating += armor_meditation_value(
            w,
            on_layer(w, m, Layer::InnerLegs).or_else(|| on_layer(w, m, Layer::Pants)),
        );
        rating += armor_meditation_value(
            w,
            on_layer(w, m, Layer::InnerTorso).or_else(|| on_layer(w, m, Layer::Shirt)),
        );
        rating / 4.0
    });
    let r = w
        .resources
        .get_mut::<crate::misc::regen_rates::RegenHooks>();
    r.creature_not_animated_dead = Some(|w, m| {
        crate::creature::data::cd(w, m).is_some() && !crate::creature::misc::is_animated_dead(w, m)
    });
    r.paragon_or_leviathan = Some(|w, m| {
        crate::creature::data::cd(w, m).is_some_and(|d| d.is_paragon)
            || crate::factory::mobile_is(w, m, "Server.Mobiles.Leviathan")
    });
}

fn configure_player(w: &mut World) {
    use crate::creature::control;
    use crate::creature::data::{cd, cd_mut};
    use crate::creature::enums::{MAX_LOYALTY, OrderType};
    use crate::creature::vtable::ctype;
    use crate::factory::mobile_is;
    use crate::player_mobile::hooks::{PetInfo, SystemHooks};
    use crate::systems::regions as rg;
    let s = w.resources.get_mut::<SystemHooks>();
    s.animal_taming_disable_message = Some(crate::systems::skills::animal::set_disable_message);
    s.stealing_classic_mode = Some(crate::systems::skills::stealing::classic_mode);
    s.corpse_set_criminal = Some(|w, c| {
        crate::items::corpse::set_flag(w, c, crate::items::corpse::flag::CRIMINAL, true)
    });
    s.stealth_on_use = Some(|w, m| {
        crate::systems::skills::stealth::stealth_on_use(w, m);
    });
    s.detect_hidden_try = Some(|w, d, t| {
        crate::systems::skills::misc_skills::try_detect_stealther(w, d, t);
    });
    s.in_jail_region = Some(|w, m| w.mobile(m).is_some_and(|x| jail_at(w, x.region)));
    s.guarded_region_protects = Some(|w, m| {
        w.mobile(m)
            .and_then(|x| rg::get_guarded(w, x.region))
            .is_some_and(|g| !rg::guarded::is_disabled(w, g))
    });
    s.region_not_young_protected = Some(|w, m| {
        w.mobile(m)
            .and_then(|x| rg::base_region::family(w, x.region))
            .is_some_and(|f| !f.young_protected)
    });
    s.dungeon_entrance = Some(|w, m| {
        let r = w.mobile(m)?.region;
        let d = muo_server::region::get_region_of_type(w, r, &rg::simple::DUNGEON_REGION)?;
        let entrance = rg::base_region::data(w, d)?.entrance;
        (entrance != muo_primitives::Point3D::default())
            .then(|| (entrance, w.region(d).map(|r| r.map)))
    });
    // ---- pets and creatures ----
    s.pet_info = Some(|w, pet| {
        let d = cd(w, pet)?;
        let (controlled, summoned, control_slots) = (d.controlled, d.summoned, d.control_slots);
        let backpack = muo_server::mobile::part2::items::find_item_on_layer(
            w,
            pet,
            muo_server::layer::Layer::Backpack,
        );
        let pack_animal = [
            "Server.Mobiles.PackLlama",
            "Server.Mobiles.PackHorse",
            "Server.Mobiles.Beetle",
        ]
        .iter()
        .any(|n| mobile_is(w, pet, n))
            && backpack.is_some_and(|b| w.item(b).is_some_and(|i| !i.items.is_empty()));
        let m = w.mobile(pet)?;
        Some(PetInfo {
            controlled,
            has_control_master: control::control_master(w, pet).is_some(),
            summoned,
            deleted: m.deleted,
            map: m.map,
            has_rider: crate::creature::mount::rider(w, pet).is_some(),
            pack_animal_with_items: pack_animal,
            escortable: mobile_is(w, pet, "Server.Mobiles.BaseEscortable"),
            control_slots,
        })
    });
    s.pet_auto_stable = Some(|w, pet, owner| {
        if let Some(d) = cd_mut(w, pet) {
            d.control_target = None;
        }
        control::set_control_order_raw(w, pet, OrderType::Stay);
        muo_server::mobile::part2::items::internalize(w, pet);
        control::set_control_master(w, pet, None);
        if let Some(d) = cd_mut(w, pet) {
            d.is_stabled = true;
            d.stabled_by = Some(owner);
        }
        control::set_loyalty(w, pet, MAX_LOYALTY);
    });
    s.pet_claim = Some(|w, pet, owner| {
        control::set_control_master(w, pet, Some(owner));
        if let Some(d) = cd_mut(w, pet) {
            d.control_target = Some(owner);
        }
        control::set_control_order_raw(w, pet, OrderType::Follow);
        if let Some((loc, Some(map))) = w.mobile(owner).map(|o| (o.location, o.map)) {
            muo_server::mobile::base::move_to_world(w, pet, loc, map);
        }
        if let Some(d) = cd_mut(w, pet) {
            d.is_stabled = false;
            d.stabled_by = None;
        }
        control::set_loyalty(w, pet, MAX_LOYALTY);
    });
    s.pet_clear_stabled = Some(|w, pet| {
        if let Some(d) = cd_mut(w, pet) {
            d.is_stabled = false;
            d.stabled_by = None;
        }
    });
    s.pet_mark_stabled = Some(|w, pet, owner| {
        if let Some(d) = cd_mut(w, pet) {
            d.is_stabled = true;
            d.stabled_by = Some(owner);
        }
    });
    s.pet_is_guarding =
        Some(|w, pet| cd(w, pet).is_some_and(|d| d.control_order == OrderType::Guard));
    s.pet_goes_with_owner = Some(|w, pet| {
        let d = cd(w, pet)?;
        if mobile_is(w, pet, "Server.Mobiles.BaseEscortable")
            || mobile_is(w, pet, "Server.Mobiles.BaseHire")
        {
            return None;
        }
        Some(d.summoned || d.is_bonded || d.is_dead_pet)
    });
    s.pet_release = Some(|w, pet| control::set_control_order_raw(w, pet, OrderType::Release));
    s.pet_dismiss_summoned = Some(|w, pet| {
        let snd = w
            .mobile(pet)
            .map(|x| x.ty)
            .map_or(0, |x| (x.get_anger_sound)(w, pet));
        crate::mx::play_sound(w, pet, snd);
        // Timer.StartTimer(pet.Delete)
        muo_server::timer::start(
            w,
            muo_primitives::TimeSpan::ZERO,
            muo_primitives::TimeSpan::ZERO,
            1,
            move |w| {
                muo_server::mobile::base::delete(w, pet);
            },
        );
    });
    s.cannot_be_harmed = Some(|w, t| {
        ctype(w, t).is_some_and(|c| (c.is_invulnerable)(w, t))
            || mobile_is(w, t, "Server.Mobiles.PlayerVendor")
            || mobile_is(w, t, "Server.Mobiles.TownCrier")
    });
    s.creature_uncontrolled = Some(|w, t| cd(w, t).map(|d| !d.controlled));
    s.creature_initial_innocent_uncontrolled = Some(|w, t| {
        ctype(w, t).is_some_and(|c| (c.initial_innocent)(w, t))
            && cd(w, t).is_some_and(|d| !d.controlled)
    });
    s.creature_controlled_by = Some(|w, t, m| control::control_master(w, t) == Some(m));
    s.creature_master = Some(|w, m| {
        cd(w, m)?;
        control::get_master(w, m)
    });
    s.creature_ignores_young_protection =
        Some(|w, m| ctype(w, m).is_some_and(|c| (c.ignore_young_protection)(w, m)));
}
