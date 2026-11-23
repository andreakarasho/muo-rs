//! The text half of BaseWeapon: `AddNameProperty`, `GetProperties`, `OnSingleClick` and
//! `OnSingleClickPreUOTD`, and `AosWeaponAttributes.GetProperties`.

use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::{LootType, ops};
use muo_server::layer::Layer;
use muo_server::packets::equipment::{EquipInfoAttribute, send_display_equipment_info};
use muo_server::property_list::PropertyList;
use muo_server::skills::SkillName;

use super::armor::append_space_with_article;
use super::equip_hooks::lower_text;
use super::slayer::{self, SlayerName};
use super::weapon::{
    self, WeaponAccuracyLevel, WeaponDamageLevel, WeaponDurabilityLevel, WeaponQuality, data,
};
use super::weapon_combat as wc;
use super::weapon_hooks as hk;
use crate::aos::{AosWeaponAttribute, Attrs, add_attribute_properties, add_skill_bonus_properties};
use crate::core;
use crate::craft::CraftResource;

/// `AosWeaponAttributes.GetProperties(list)`.
pub fn add_weapon_attribute_properties(a: &Attrs, list: &mut PropertyList, core_aos: bool) {
    use AosWeaponAttribute as W;
    let g = |x: W| a.get(core_aos, x as u32);
    if g(W::UseBestSkill) != 0 {
        list.add(1060400); // use best weapon skill
    }
    fn add(list: &mut PropertyList, n: i32, v: i32) {
        if v != 0 {
            list.add_int(n, v);
        }
    }
    add(list, 1060416, g(W::HitColdArea)); // hit cold area ~1_val~%
    add(list, 1060417, g(W::HitDispel)); // hit dispel ~1_val~%
    add(list, 1060418, g(W::HitEnergyArea)); // hit energy area ~1_val~%
    add(list, 1060419, g(W::HitFireArea)); // hit fire area ~1_val~%
    add(list, 1060420, g(W::HitFireball)); // hit fireball ~1_val~%
    add(list, 1060421, g(W::HitHarm)); // hit harm ~1_val~%
    add(list, 1060422, g(W::HitLeechHits)); // hit life leech ~1_val~%
    add(list, 1060423, g(W::HitLightning)); // hit lightning ~1_val~%
    add(list, 1060424, g(W::HitLowerAttack)); // hit lower attack ~1_val~%
    add(list, 1060425, g(W::HitLowerDefend)); // hit lower defense ~1_val~%
    add(list, 1060426, g(W::HitMagicArrow)); // hit magic arrow ~1_val~%
    add(list, 1060427, g(W::HitLeechMana)); // hit mana leech ~1_val~%
    add(list, 1060428, g(W::HitPhysicalArea)); // hit physical area ~1_val~%
    add(list, 1060429, g(W::HitPoisonArea)); // hit poison area ~1_val~%
    add(list, 1060430, g(W::HitLeechStam)); // hit stamina leech ~1_val~%
    let mage = g(W::MageWeapon);
    if mage != 0 {
        list.add_int(1060438, 30 - mage); // mage weapon -~1_val~ skill
    }
    add(list, 1060450, g(W::SelfRepair)); // self repair ~1_val~
}

/// `AddNameProperty(list)`.
pub fn add_name_property(w: &World, id: ItemId, list: &mut PropertyList) {
    let Some(d) = data(w, id) else { return };
    use CraftResource::*;
    let ore_type = match d.resource {
        DullCopper => 1053108,
        ShadowIron => 1053107,
        Copper => 1053106,
        Bronze => 1053105,
        Gold => 1053104,
        Agapite => 1053103,
        Verite => 1053102,
        Valorite => 1053101,
        SpinedLeather => 1061118,
        HornedLeather => 1061117,
        BarbedLeather => 1061116,
        RedScales => 1060814,
        YellowScales => 1060818,
        BlackScales => 1060820,
        GreenScales => 1060819,
        WhiteScales => 1060821,
        BlueScales => 1060815,
        _ => 0,
    };

    let name = ops::name(w, id);
    let label = w.item(id).map_or(0, |i| (i.ty.label_number)(w, id));

    if ore_type != 0 {
        match name {
            // ~1_oretype~ ~2_armortype~
            Some(name) => list.add_number_fmt(1053099, format_args!("#{ore_type}\t{name}")),
            Option::None => list.add_number_fmt(1053099, format_args!("#{ore_type}\t#{label}")),
        }
    } else {
        match name {
            Option::None => list.add(label),
            Some(name) => list.add_text(&name),
        }
    }

    if let Some(text) = d.engraved_text.as_deref().filter(|s| !s.is_empty()) {
        list.add_str(1062613, text);
    }
}

/// `GetProperties(list)`.
pub fn get_properties(w: &World, id: ItemId, list: &mut PropertyList) {
    muo_server::item::base::get_properties_base(w, id, list);
    let Some(d) = data(w, id) else { return };
    let Some(ty) = w.item(id).map(|i| i.ty) else {
        return;
    };
    let aos = core::aos(w);
    let ml = core::ml(w);

    if let Some(c) = &d.crafter {
        list.add_str(1050043, c); // crafted by ~1_NAME~
    }

    if super::equip_hooks::hooks(w)
        .and_then(|h| h.faction_item)
        .is_some_and(|f| f(w, id))
    {
        list.add(1041350); // faction item
    }

    add_skill_bonus_properties(&d.skill_bonuses, list, aos);

    if d.quality == WeaponQuality::Exceptional {
        list.add(1060636); // exceptional
    }

    let required = (ty.required_races)(w, id);
    if required == muo_server::race::ALLOW_ELVES_ONLY {
        list.add(1075086); // Elves Only
    }
    if required == muo_server::race::ALLOW_GARGOYLES_ONLY {
        list.add(1111709); // Gargoyles Only
    }

    let rarity = weapon::artifact_rarity(w, id);
    if rarity > 0 {
        list.add_int(1061078, rarity); // artifact rarity ~1_val~
    }

    // IUsesRemaining: BaseAxe, BasePoleArm and the ProspectorsTool.
    if (ty.is(&weapon::BASE_AXE)
        || ty.is(&weapon::BASE_POLE_ARM)
        || super::is_named(ty, "Server.Items.ProspectorsTool"))
        && d.show_uses_remaining
    {
        list.add_int(1060584, d.uses_remaining); // uses remaining: ~1_val~
    }

    if let Some(p) = d.poison
        && d.poison_charges > 0
    {
        list.add_int(1062412 + muo_server::poison::level(w, p), d.poison_charges);
    }

    for s in [d.slayer, d.slayer2] {
        if let Some(name) = SlayerName::from_i32(s).filter(|n| *n != SlayerName::None)
            && let Some((entry, _)) = slayer::entry_by_name(w, name)
        {
            list.add(slayer::title(w, entry));
        }
    }

    (ty.add_resistance_properties)(w, id, list);

    let is_ranged = ty.is(&weapon::BASE_RANGED);

    if ml && is_ranged && d.balanced {
        list.add(1072792); // Balanced
    }

    add_weapon_attribute_properties(&d.weapon_attributes, list, aos);

    if hk::immolating_is(w, id) {
        list.add(1111917); // Immolated
    }

    if ml && is_ranged && d.velocity != 0 {
        list.add_int(1072793, d.velocity); // Velocity ~1_val~%
    }

    add_attribute_properties(
        &d.attributes,
        list,
        aos,
        ml,
        wc::damage_bonus(w, id),
        wc::hit_chance_bonus(w, id),
        wc::luck_bonus(w, id),
    );

    let lower_stat_req = weapon::lower_stat_req(w, id);
    if lower_stat_req != 0 {
        list.add_int(1060435, lower_stat_req); // lower requirements ~1_val~%
    }

    let t = wc::get_damage_types(w, id, None);

    if t.phys != 0 {
        list.add_int(1060403, t.phys); // physical damage ~1_val~%
    }
    if t.cold != 0 {
        list.add_int(1060404, t.cold); // cold damage ~1_val~%
    }
    if t.fire != 0 {
        list.add_int(1060405, t.fire); // fire damage ~1_val~%
    }
    if t.pois != 0 {
        list.add_int(1060406, t.pois); // poison damage ~1_val~%
    }
    if t.nrgy != 0 {
        list.add_int(1060407, t.nrgy); // energy damage ~1_val
    }
    if ml && t.chaos != 0 {
        list.add_int(1072846, t.chaos); // chaos damage ~1_val~%
    }
    if ml && t.direct != 0 {
        list.add_int(1079978, t.direct); // Direct Damage: ~1_PERCENT~%
    }

    // weapon damage ~1_val~ - ~2_val~
    list.add_number_fmt(
        1061168,
        format_args!(
            "{}\t{}",
            weapon::min_damage(w, id),
            weapon::max_damage(w, id)
        ),
    );

    let speed = weapon::speed(w, id);
    if ml {
        list.add_number_fmt(1061167, format_args!("{speed}s")); // weapon speed ~1_val~
    } else {
        list.add_number_fmt(1061167, format_args!("{speed}"));
    }

    let max_range = weapon::max_range(w, id);
    if max_range > 1 {
        list.add_int(1061169, max_range); // range ~1_val~
    }

    let str_req = crate::misc::aos_helpers::scale(
        weapon::str_requirement(w, id),
        100 - weapon::lower_stat_req(w, id),
    );
    if str_req > 0 {
        list.add_int(1061170, str_req); // strength requirement ~1_val~
    }

    if w.item(id).is_some_and(|i| i.layer == Layer::TwoHanded) {
        list.add(1061171); // two-handed weapon
    } else {
        list.add(1061824); // one-handed weapon
    }

    if core::se(w)
        || d.weapon_attributes
            .get(aos, AosWeaponAttribute::UseBestSkill as u32)
            == 0
    {
        match weapon::skill(w, id) {
            SkillName::Swords => list.add(1061172), // skill required: swordsmanship
            SkillName::Macing => list.add(1061173), // skill required: mace fighting
            SkillName::Fencing => list.add(1061174), // skill required: fencing
            SkillName::Archery => list.add(1061175), // skill required: archery
            SkillName::Throwing => list.add(1112075), // skill required: throwing
            _ => {}
        }
    }

    if d.hit_points >= 0 && d.max_hit_points > 0 {
        // durability ~1_val~ / ~2_val~
        list.add_number_fmt(
            1060639,
            format_args!("{}\t{}", d.hit_points, d.max_hit_points),
        );
    }
}

fn slayer_title(w: &World, s: i32) -> Option<i32> {
    let name = SlayerName::from_i32(s).filter(|n| *n != SlayerName::None)?;
    slayer::entry_by_name(w, name).map(|(e, _)| slayer::title(w, e))
}

/// `OnSingleClick(from)`.
pub fn on_single_click(w: &mut World, id: ItemId, from: MobileId) {
    if !super::core_uotd(w) {
        on_single_click_pre_uotd(w, id, from);
        return;
    }
    let Some(d) = data(w, id).cloned() else {
        return;
    };
    let Some(ty) = w.item(id).map(|i| i.ty) else {
        return;
    };

    let mut attrs: Vec<EquipInfoAttribute> = Vec::new();

    if (ty.display_loot_type)(w, id) {
        match w.item(id).map(|i| i.loot_type) {
            Some(LootType::Blessed) => attrs.push(EquipInfoAttribute::new(1038021, -1)), // blessed
            Some(LootType::Cursed) => attrs.push(EquipInfoAttribute::new(1049643, -1)),  // cursed
            _ => {}
        }
    }

    if super::equip_hooks::hooks(w)
        .and_then(|h| h.faction_item)
        .is_some_and(|f| f(w, id))
    {
        attrs.push(EquipInfoAttribute::new(1041350, -1)); // faction item
    }

    if d.quality == WeaponQuality::Exceptional {
        attrs.push(EquipInfoAttribute::new(1018305 - d.quality as i32, -1));
    }

    let gm = w
        .mobile(from)
        .is_some_and(|m| m.access_level >= muo_server::mobile::AccessLevel::GameMaster);
    if d.identified || gm {
        for s in [d.slayer, d.slayer2] {
            if let Some(t) = slayer_title(w, s) {
                attrs.push(EquipInfoAttribute::new(t, -1));
            }
        }
        if d.durability_level != WeaponDurabilityLevel::Regular {
            attrs.push(EquipInfoAttribute::new(
                1038000 + d.durability_level as i32,
                -1,
            ));
        }
        if d.damage_level != WeaponDamageLevel::Regular {
            attrs.push(EquipInfoAttribute::new(1038015 + d.damage_level as i32, -1));
        }
        if d.accuracy_level != WeaponAccuracyLevel::Regular {
            attrs.push(EquipInfoAttribute::new(
                1038010 + d.accuracy_level as i32,
                -1,
            ));
        }
    } else if d.slayer != 0
        || d.slayer2 != 0
        || d.durability_level != WeaponDurabilityLevel::Regular
        || d.damage_level != WeaponDamageLevel::Regular
        || d.accuracy_level != WeaponAccuracyLevel::Regular
    {
        attrs.push(EquipInfoAttribute::new(1038000, -1)); // Unidentified
    }

    if d.poison.is_some() && d.poison_charges > 0 {
        attrs.push(EquipInfoAttribute::new(1017383, d.poison_charges));
    }

    let name = ops::name(w, id);
    let number = match &name {
        None => (ty.label_number)(w, id),
        Some(n) => {
            ops::label_to(w, id, from, n);
            1041000
        }
    };

    if attrs.is_empty() && d.crafter.is_none() && name.is_some() {
        return;
    }

    let serial = w.item(id).map(|i| i.serial);
    if let (Some(ns), Some(serial)) = (muo_server::item::deps::net_state_of(w, from), serial) {
        send_display_equipment_info(w, ns, serial, number, d.crafter.as_deref(), false, &attrs);
    }
}

/// `OnSingleClickPreUOTD(from)`.
pub fn on_single_click_pre_uotd(w: &mut World, id: ItemId, from: MobileId) {
    let Some(d) = data(w, id).cloned() else {
        return;
    };
    let Some(ty) = w.item(id).map(|i| i.ty) else {
        return;
    };
    let label = (ty.label_number)(w, id);

    let is_magic = d.durability_level as i32 > 0
        || d.accuracy_level as i32 > 0
        || d.damage_level as i32 > 0
        || d.slayer != 0;

    let name = ops::name(w, id);

    if is_magic && !d.identified {
        let n = name.unwrap_or_else(|| lower_text(w, label));
        ops::label_to(w, id, from, &format!("an unidentified {n}"));
        return;
    }

    let article_an = ops::item_data(w, id)
        .flags
        .contains(muo_data::tile_data::TileFlag::ARTICLE_AN);

    if is_magic {
        let mut b = String::new();

        let (durability, an) = match d.durability_level {
            WeaponDurabilityLevel::Durable => (Some("durable"), false),
            WeaponDurabilityLevel::Substantial => (Some("substantial"), false),
            WeaponDurabilityLevel::Massive => (Some("massive"), false),
            WeaponDurabilityLevel::Fortified => (Some("fortified"), false),
            WeaponDurabilityLevel::Indestructible => (Some("indestructible"), true),
            WeaponDurabilityLevel::Regular => (None, false),
        };
        if let Some(t) = durability {
            append_space_with_article(&mut b, t, an);
        }

        let (accuracy, an) = match d.accuracy_level {
            WeaponAccuracyLevel::Accurate => (Some("accurate"), true),
            WeaponAccuracyLevel::Surpassingly => (Some("surpassingly accurate"), false),
            WeaponAccuracyLevel::Eminently => (Some("eminently accurate"), true),
            WeaponAccuracyLevel::Exceedingly => (Some("exceedingly accurate"), false),
            WeaponAccuracyLevel::Supremely => (Some("supremely accurate"), false),
            WeaponAccuracyLevel::Regular => (None, false),
        };
        if let Some(t) = accuracy {
            append_space_with_article(&mut b, t, an);
        }

        if let Some(name) = SlayerName::from_i32(d.slayer).filter(|n| *n != SlayerName::None)
            && let Some((entry, _)) = slayer::entry_by_name(w, name)
        {
            // `SlayerEntry.SlayerText`: the lower-cased title and its article.
            let text = lower_text(w, slayer::title(w, entry));
            if !text.is_empty() {
                append_space_with_article(&mut b, &text, slayer::slayer_article_an(name));
            }
        }

        match &name {
            None => append_space_with_article(&mut b, &lower_text(w, label), article_an),
            Some(n) if !b.is_empty() => {
                b.push(' ');
                b.push_str(n);
            }
            Some(n) => b.push_str(n),
        }

        let damage_text = match d.damage_level {
            WeaponDamageLevel::Ruin => Some("ruin"),
            WeaponDamageLevel::Might => Some("might"),
            WeaponDamageLevel::Force => Some("force"),
            WeaponDamageLevel::Power => Some("power"),
            WeaponDamageLevel::Vanq => Some("vanquishing"),
            WeaponDamageLevel::Regular => None,
        };
        if let Some(t) = damage_text {
            b.push_str(" of ");
            b.push_str(t);
        }

        // TODO: Spells (of Ghoul's Touch)
        ops::label_to(w, id, from, &b);
        return;
    }

    let name = name.unwrap_or_else(|| {
        format!(
            "{} {}",
            if article_an { "an" } else { "a" },
            lower_text(w, label)
        )
    });

    let exceptional = d.quality == WeaponQuality::Exceptional;
    match &d.crafter {
        None => {
            let t = if exceptional {
                format!("{name} of exceptional quality")
            } else {
                name
            };
            ops::label_to(w, id, from, &t);
        }
        Some(c) => {
            let t = if exceptional {
                format!("{name} crafted with exceptional quality by {c}")
            } else {
                format!("{name} crafted by {c}")
            };
            ops::label_to(w, id, from, &t);
        }
    }
}
