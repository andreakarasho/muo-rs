//! `OssianGrimoire` and the blacksmithy bulk order rewards
//! (ColoredAnvil, AncientSmithyHammer, PowderOfTemperament, GlovesOfMining).

use std::rc::Rc;

use muo_content::aos::{AosAttribute, AttrKind};
use muo_content::craft::CraftResource;
use muo_content::items::armor::{
    ArmorClass, ArmorMaterialType, BASE_ARMOR, BASE_ARMOR_CLASS, new_armor,
};
use muo_content::items::set_attr;
use muo_content::items::tool::{BASE_TOOL, BASE_TOOL_CLASS, ToolClass, new_tool};

use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, LootType, deps, ops};
use muo_server::layer::Layer;
use muo_server::mods::SkillMod;
use muo_server::packets::message::AffixType;
use muo_server::skills::SkillName;
use muo_server::target::{self, TargetFlags, Targeted};
use muo_server::type_registry::TypeRegistry;

use super::util::{delete, msg};
use crate::armor::{armor_leaf, skill_bonus};
use crate::simple::leaf;

// ---- OssianGrimoire ----

/// `OssianGrimoire()`.
pub fn ossian_grimoire_new(w: &mut World) -> ItemId {
    let id = crate::skill_items::spellbook::new_spellbook(w, &OSSIAN_GRIMOIRE, 0, 0x2253);
    let layer = if muo_content::core::ml(w) {
        Layer::OneHanded
    } else {
        Layer::Invalid
    };
    (OSSIAN_GRIMOIRE.set_layer)(w, id, layer);
    ops::set_loot_type(w, id, LootType::Blessed);
    skill_bonus(w, id, 0, SkillName::Necromancy, 10.0);
    set_attr(
        w,
        id,
        AttrKind::Attributes,
        AosAttribute::RegenMana as u32,
        1,
    );
    set_attr(
        w,
        id,
        AttrKind::Attributes,
        AosAttribute::CastSpeed as u32,
        1,
    );
    set_attr(
        w,
        id,
        AttrKind::Attributes,
        AosAttribute::IncreasedKarmaLoss as u32,
        5,
    );
    id
}

/// `Server.Items.OssianGrimoire`.
pub static OSSIAN_GRIMOIRE: ItemType = ItemType {
    full_name: "Server.Items.OssianGrimoire",
    parent: Some(&crate::skill_items::spellbook::NECROMANCER_SPELLBOOK),
    construct: Some(ossian_grimoire_new),
    label_number: crate::class_props::label_number, // Ossian Grimoire
    serialize: |w, id, out| {
        (crate::skill_items::spellbook::NECROMANCER_SPELLBOOK.serialize)(w, id, out);
        out.write_encoded_int(0);
    },
    deserialize: |w, id, r| {
        (crate::skill_items::spellbook::NECROMANCER_SPELLBOOK.deserialize)(w, id, r)?;
        let _version = r.read_encoded_int()?;
        Ok(())
    },
    ..crate::skill_items::spellbook::NECROMANCER_SPELLBOOK
};

// ---- ColoredAnvil ----

/// `CraftResources.GetRandomResource(DullCopper, Valorite)?.Hue ?? 0`.
fn random_ore_hue(w: &mut World) -> i32 {
    let r = muo_primitives::utility::random_min_max(
        &mut w.rng,
        CraftResource::DullCopper as i32,
        CraftResource::Valorite as i32,
    );
    CraftResource::from_i32(r).map_or(0, |r| muo_content::craft::hue(w, r))
}

leaf! {
    /// `Server.Items.ColoredAnvil` (`[Anvil]`, `[Flippable(0xFAF, 0xFB0)]`).
    COLORED_ANVIL = "Server.Items.ColoredAnvil" : ITEM_BASE, v0;
    construct: Some(|w| {
        let id = crate::simple::from_template(w, &COLORED_ANVIL, "Server.Items.ColoredAnvil");
        let h = random_ore_hue(w);
        super::hue(w, id, h); // TODO: Color weighted by rarity?
        id
    }),
    default_weight: crate::class_props::default_weight,
}

// ---- the skill mod of the hammer and the gloves ----

/// A bonus item's `_bonus` and `_skillMod` (the mobile it was added to and the mod).
#[derive(Clone, Copy, Debug, Default)]
pub struct BonusData {
    pub bonus: i32,
    skill_mod: Option<(MobileId, muo_server::mods::ModId)>,
}

pub fn bonus_data(w: &World, id: ItemId) -> BonusData {
    w.item(id)
        .and_then(|i| i.ext.get::<BonusData>())
        .copied()
        .unwrap_or_default()
}

fn set_bonus_data(w: &mut World, id: ItemId, d: BonusData) {
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(d);
    }
}

fn remove_mod(w: &mut World, id: ItemId) {
    let mut d = bonus_data(w, id);
    if let Some((owner, m)) = d.skill_mod.take() {
        muo_server::mobile::base::remove_skill_mod(w, owner, m);
    }
    set_bonus_data(w, id, d);
}

fn add_mod(w: &mut World, id: ItemId, mobile: MobileId, skill: SkillName, name: &str) {
    let mut d = bonus_data(w, id);
    let sm = SkillMod::new_default(w, skill, Some(name), true, f64::from(d.bonus));
    let mid = sm.id;
    muo_server::mobile::base::add_skill_mod(w, mobile, sm);
    d.skill_mod = Some((mobile, mid));
    set_bonus_data(w, id, d);
}

fn parent_mobile(w: &World, id: ItemId) -> Option<MobileId> {
    match w.item(id)?.parent {
        Some(EntityId::Mobile(m)) => Some(m),
        _ => None,
    }
}

/// `Bonus = value` of the hammer and the gloves.
fn set_bonus(w: &mut World, id: ItemId, value: i32, skill: SkillName, name: &str) {
    let mut d = bonus_data(w, id);
    d.bonus = value;
    set_bonus_data(w, id, d);
    ops::invalidate_properties(w, id);
    if value == 0 {
        remove_mod(w, id);
    } else if d.skill_mod.is_none() {
        if let Some(m) = parent_mobile(w, id) {
            add_mod(w, id, m, skill, name);
        }
    } else if let Some((owner, m)) = d.skill_mod {
        muo_server::mods::set_skill_mod_value(w, owner, m, f64::from(value));
    }
}

fn on_added_bonus(w: &mut World, id: ItemId, parent: EntityId, skill: SkillName, name: &str) {
    if bonus_data(w, id).bonus != 0
        && let EntityId::Mobile(m) = parent
    {
        remove_mod(w, id);
        add_mod(w, id, m, skill, name);
    }
}

/// `[AfterDeserialization]`.
fn after_deserialization_bonus(w: &mut World, id: ItemId, skill: SkillName, name: &str) {
    if bonus_data(w, id).bonus != 0
        && let Some(m) = parent_mobile(w, id)
    {
        add_mod(w, id, m, skill, name);
    }
}

// ---- AncientSmithyHammer ----

/// `AncientSmithyHammer(bonus, uses)`.
pub fn ancient_smithy_hammer_new(w: &mut World, bonus: i32, uses: i32) -> ItemId {
    let id = new_tool(w, &ANCIENT_SMITHY_HAMMER, 0x13E4, uses);
    set_bonus_data(
        w,
        id,
        BonusData {
            bonus,
            skill_mod: None,
        },
    );
    (ANCIENT_SMITHY_HAMMER.set_layer)(w, id, Layer::OneHanded);
    super::hue(w, id, 0x482);
    id
}

/// `AncientSmithyHammer.Bonus = value`.
pub fn set_hammer_bonus(w: &mut World, id: ItemId, value: i32) {
    set_bonus(w, id, value, SkillName::Blacksmith, "AncientSmithyHammer");
}

static HAMMER_CLASS: ToolClass = ToolClass {
    craft_system: "Blacksmithy",
    ..BASE_TOOL_CLASS
};

/// `Server.Items.AncientSmithyHammer` (`[Flippable(0x13E4, 0x13E3)]`; `AncientSmithyHammer(int bonus, int uses = 600)`).
pub static ANCIENT_SMITHY_HAMMER: ItemType = ItemType {
    full_name: "Server.Items.AncientSmithyHammer",
    parent: Some(&BASE_TOOL),
    construct: None,
    family: Some(&HAMMER_CLASS),
    default_weight: crate::class_props::default_weight,
    label_number: crate::class_props::label_number, // ancient smithy hammer
    on_added: |w, id, parent| {
        (BASE_TOOL.on_added)(w, id, parent);
        on_added_bonus(w, id, parent, SkillName::Blacksmith, "AncientSmithyHammer");
    },
    on_removed: |w, id, parent| {
        (BASE_TOOL.on_removed)(w, id, parent);
        remove_mod(w, id);
    },
    get_properties: |w, id, list| {
        (BASE_TOOL.get_properties)(w, id, list);
        let b = bonus_data(w, id).bonus;
        if b != 0 {
            list.add_str(1060451, &format!("#1042354\t{b}")); // ~1_skillname~ +~2_val~
        }
    },
    serialize: |w, id, out| {
        (BASE_TOOL.serialize)(w, id, out);
        out.write_i32(0);
        out.write_i32(bonus_data(w, id).bonus);
    },
    deserialize: |w, id, r| {
        (BASE_TOOL.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        let bonus = r.read_i32()?;
        set_bonus_data(
            w,
            id,
            BonusData {
                bonus,
                skill_mod: None,
            },
        );
        after_deserialization_bonus(w, id, SkillName::Blacksmith, "AncientSmithyHammer");
        Ok(())
    },
    ..BASE_TOOL
};

// ---- GlovesOfMining ----

/// `BaseGlovesOfMining(bonus, itemID)` of class `ty`.
fn gloves_new(w: &mut World, ty: &'static ItemType, item_id: i32, bonus: i32) -> ItemId {
    let id = new_armor(w, ty, item_id);
    let h = random_ore_hue(w); // TODO: Color weighted by rarity?
    super::hue(w, id, h);
    set_bonus_data(
        w,
        id,
        BonusData {
            bonus,
            skill_mod: None,
        },
    );
    id
}

/// `Server.Items.BaseGlovesOfMining` (abstract).
pub static BASE_GLOVES_OF_MINING: ItemType = ItemType {
    full_name: "Server.Items.BaseGlovesOfMining",
    parent: Some(&BASE_ARMOR),
    construct: None,
    on_added: |w, id, parent| {
        (BASE_ARMOR.on_added)(w, id, parent);
        on_added_bonus(w, id, parent, SkillName::Mining, "MiningGloves");
    },
    on_removed: |w, id, parent| {
        (BASE_ARMOR.on_removed)(w, id, parent);
        remove_mod(w, id);
    },
    get_properties: |w, id, list| {
        (BASE_ARMOR.get_properties)(w, id, list);
        let b = bonus_data(w, id).bonus;
        if b != 0 {
            list.add_int(1062005, b); // mining bonus +~1_val~
        }
    },
    serialize: |w, id, out| {
        (BASE_ARMOR.serialize)(w, id, out);
        out.write_i32(0);
        out.write_i32(bonus_data(w, id).bonus);
    },
    deserialize: |w, id, r| {
        (BASE_ARMOR.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        let bonus = r.read_i32()?;
        set_bonus_data(
            w,
            id,
            BonusData {
                bonus,
                skill_mod: None,
            },
        );
        after_deserialization_bonus(w, id, SkillName::Mining, "MiningGloves");
        Ok(())
    },
    ..BASE_ARMOR
};

/// `BaseGlovesOfMining.Bonus = value`.
pub fn set_gloves_bonus(w: &mut World, id: ItemId, value: i32) {
    set_bonus(w, id, value, SkillName::Mining, "MiningGloves");
}

/// `LeatherGlovesOfMining(bonus)`: the only `[Constructible]` constructor.
pub fn leather_gloves_of_mining_new_bonus(w: &mut World, bonus: i32) -> ItemId {
    gloves_new(w, &LEATHER_GLOVES_OF_MINING, 0x13C6, bonus)
}

armor_leaf! {
    /// `Server.Items.LeatherGlovesOfMining` (`[Flippable(0x13c6, 0x13ce)]`).
    LEATHER_GLOVES_OF_MINING, LEATHER_GLOVES_OF_MINING_CLASS, leather_gloves_of_mining_new = "Server.Items.LeatherGlovesOfMining" : BASE_GLOVES_OF_MINING / BASE_ARMOR_CLASS, |w, ty| gloves_new(w, ty, 0x13C6, 0), encoded false;
    @no_construct none;
    default_weight: Some(1.0),
    material_type: ArmorMaterialType::Leather,
    default_resource: CraftResource::RegularLeather,
    def_med_allowance: muo_content::items::armor::ArmorMeditationAllowance::All,
}

/// `StuddedGlovesOfMining(bonus)`: the only `[Constructible]` constructor.
pub fn studded_gloves_of_mining_new_bonus(w: &mut World, bonus: i32) -> ItemId {
    gloves_new(w, &STUDDED_GLOVES_OF_MINING, 0x13D5, bonus)
}

armor_leaf! {
    /// `Server.Items.StuddedGlovesOfMining` (`[Flippable(0x13d5, 0x13dd)]`).
    STUDDED_GLOVES_OF_MINING, STUDDED_GLOVES_OF_MINING_CLASS, studded_gloves_of_mining_new = "Server.Items.StuddedGlovesOfMining" : BASE_GLOVES_OF_MINING / BASE_ARMOR_CLASS, |w, ty| gloves_new(w, ty, 0x13D5, 0), encoded false;
    @no_construct none;
    default_weight: Some(2.0),
    material_type: ArmorMaterialType::Studded,
    default_resource: CraftResource::RegularLeather,
}

/// `RingmailGlovesOfMining(bonus)`: the only `[Constructible]` constructor.
pub fn ringmail_gloves_of_mining_new_bonus(w: &mut World, bonus: i32) -> ItemId {
    gloves_new(w, &RINGMAIL_GLOVES_OF_MINING, 0x13EB, bonus)
}

armor_leaf! {
    /// `Server.Items.RingmailGlovesOfMining` (`[Flippable(0x13eb, 0x13f2)]`).
    RINGMAIL_GLOVES_OF_MINING, RINGMAIL_GLOVES_OF_MINING_CLASS, ringmail_gloves_of_mining_new = "Server.Items.RingmailGlovesOfMining" : BASE_GLOVES_OF_MINING / BASE_ARMOR_CLASS, |w, ty| gloves_new(w, ty, 0x13EB, 0), encoded false;
    @no_construct none;
    default_weight: Some(1.0),
    material_type: ArmorMaterialType::Ringmail,
}

// ---- PowderOfTemperament ----

/// `PowderOfTemperament._usesRemaining`.
#[derive(Clone, Copy, Debug, Default)]
struct Uses(i32);

pub fn uses(w: &World, id: ItemId) -> i32 {
    w.item(id)
        .and_then(|i| i.ext.get::<Uses>())
        .map_or(0, |u| u.0)
}

/// `UsesRemaining = value` (`[InvalidateProperties]`).
pub fn set_powder_uses(w: &mut World, id: ItemId, value: i32) {
    if uses(w, id) != value || w.item(id).is_some_and(|i| i.ext.get::<Uses>().is_none()) {
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(Uses(value));
        }
        ops::invalidate_properties(w, id);
    }
}

/// `PowderOfTemperament(charges)`.
pub fn powder_of_temperament_new(w: &mut World, charges: i32) -> ItemId {
    let id = crate::simple::from_template(w, &POWDER_OF_TEMPERAMENT, "Server.Items.PowderOfTemperament");
    set_powder_uses(w, id, charges);
    id
}

/// The `IDurability` of an armor, weapon or clothing item: `(HitPoints, MaxHitPoints, CanFortify, InitMaxHits)`.
fn durability(w: &World, id: ItemId) -> Option<(i32, i32, bool, i32)> {
    use muo_content::items::{armor, clothing, weapon};
    use muo_content::templates::item_props;
    let i = w.item(id)?;
    if let Some(d) = i.ext.get::<weapon::WeaponData>() {
        let c = weapon::class(w, id);
        Some((
            d.hit_points,
            d.max_hit_points,
            c.can_fortify,
            item_props::int(w, id, "initMaxHits", c.init_max_hits),
        ))
    } else if let Some(d) = i.ext.get::<clothing::ClothingData>() {
        let c = clothing::class(w, id);
        Some((
            d.hit_points,
            d.max_hit_points,
            c.can_fortify,
            c.init_max_hits.get(muo_content::core::ml(w)),
        ))
    } else if let Some(d) = i.ext.get::<armor::ArmorData>() {
        let c = armor::class(w, id);
        Some((
            d.hit_points,
            d.max_hit_points,
            c.can_fortify,
            item_props::int(w, id, "initMaxHits", c.init_max_hits),
        ))
    } else {
        None
    }
}

fn set_hit_points(w: &mut World, id: ItemId, v: i32) {
    use muo_content::items::{armor, clothing, weapon};
    let Some(i) = w.item(id) else { return };
    if i.ext.get::<weapon::WeaponData>().is_some() {
        weapon::set_hit_points(w, id, v);
    } else if i.ext.get::<clothing::ClothingData>().is_some() {
        clothing::set_hit_points(w, id, v);
    } else {
        armor::set_hit_points(w, id, v);
    }
}

fn set_max_hit_points(w: &mut World, id: ItemId, v: i32) {
    use muo_content::items::{armor, clothing, weapon};
    let Some(i) = w.item(id) else { return };
    if i.ext.get::<weapon::WeaponData>().is_some() {
        weapon::set_max_hit_points(w, id, v);
    } else if i.ext.get::<clothing::ClothingData>().is_some() {
        clothing::set_max_hit_points(w, id, v);
    } else {
        armor::set_max_hit_points(w, id, v);
    }
}

fn powder_on_target(w: &mut World, powder: ItemId, from: MobileId, targeted: Targeted) {
    if ops::is_deleted(w, powder) || uses(w, powder) <= 0 {
        msg(w, from, 1049086); // You have used up your powder of temperament.
        return;
    }
    let item = match targeted {
        Targeted::Entity(EntityId::Item(i)) if durability(w, i).is_some() => i,
        _ => {
            msg(w, from, 1049083); // You cannot use the powder on that item.
            return;
        }
    };
    let Some((cur, max, can_fortify, init_max)) = durability(w, item) else {
        return;
    };
    if !can_fortify {
        msg(w, from, 1049083); // You cannot use the powder on that item.
        return;
    }
    let backpack = w.mobile(from).and_then(|m| m.backpack).map(EntityId::Item);
    let item_parent_is_from = w.item(item).and_then(|i| i.parent) == Some(EntityId::Mobile(from));
    if !ops::is_child_of(w, item, backpack) && (!deps::core_ml(w) || !item_parent_is_from)
        || !ops::is_child_of(w, powder, backpack)
    {
        msg(w, from, 1042001); // That must be in your pack for you to use it.
        return;
    }
    let (orig_max, orig_cur) = (max, cur);
    if orig_max <= 0 {
        msg(w, from, 1049083); // You cannot use the powder on that item.
        return;
    }
    let init_max_hp = if muo_content::core::aos(w) {
        255
    } else {
        init_max
    };
    muo_content::items::unscale_durability(w, item);
    let max_now = durability(w, item).map_or(0, |d| d.1);
    if max_now >= init_max_hp {
        msg(w, from, 1049085); // The item cannot be improved any further.
        muo_content::items::scale_durability(w, item);
        return;
    }
    let bonus = (init_max_hp - max_now).min(10);
    set_max_hit_points(w, item, max_now + bonus);
    let hp_now = durability(w, item).map_or(0, |d| d.0);
    set_hit_points(w, item, hp_now + bonus);
    muo_content::items::scale_durability(w, item);
    let (hp2, max2, _, _) = durability(w, item).unwrap_or((0, 0, false, 0));
    if max2 > 255 {
        set_max_hit_points(w, item, 255);
    }
    if hp2 > 255 {
        set_hit_points(w, item, 255);
    }
    let max3 = durability(w, item).map_or(0, |d| d.1);
    if max3 > orig_max {
        msg(w, from, 1049084); // You successfully use the powder on the item.
        muo_content::mx::play_sound(w, from, 0x247);
        let u = uses(w, powder) - 1;
        set_powder_uses(w, powder, u);
        if u <= 0 {
            msg(w, from, 1049086); // You have used up your powder of fortifying.
            delete(w, powder);
        }
    } else {
        set_max_hit_points(w, item, orig_max);
        set_hit_points(w, item, orig_cur);
        msg(w, from, 1049085); // The item cannot be improved any further.
    }
}

/// `Server.Items.PowderOfTemperament` (`IUsesRemaining`).
pub static POWDER_OF_TEMPERAMENT: ItemType = ItemType {
    full_name: "Server.Items.PowderOfTemperament",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| powder_of_temperament_new(w, 10)),
    default_weight: crate::class_props::default_weight,
    label_number: crate::class_props::label_number, // powder of fortifying
    get_properties: |w, id, list| {
        (ITEM_BASE.get_properties)(w, id, list);
        list.add_int(1060584, uses(w, id)); // uses remaining: ~1_val~
    },
    on_single_click: |w, id, from| {
        ops::label_to_affix(
            w,
            id,
            from,
            1017323,
            AffixType::APPEND,
            &format!(": {}", uses(w, id)),
            "",
        ); // Durability
        (ITEM_BASE.on_single_click)(w, id, from);
    },
    on_double_click: |w, id, from| {
        if super::util::in_pack(w, id, from) {
            let mut t = target::Target::new(w, 2, false, TargetFlags::NONE);
            t.handlers.on_target = Some(Rc::new(move |w: &mut World, from, targeted| {
                powder_on_target(w, id, from, targeted)
            }));
            let tid = target::add_target(w, t);
            target::set_mobile_target(w, from, Some(tid));
        } else {
            msg(w, from, 1042001); // That must be in your pack for you to use it.
        }
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(0);
        out.write_i32(uses(w, id));
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        let n = r.read_i32()?;
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(Uses(n));
        }
        Ok(())
    },
    ..ITEM_BASE
};

pub fn register(types: &mut TypeRegistry) {
    for t in [
        &OSSIAN_GRIMOIRE,
        &COLORED_ANVIL,
        &ANCIENT_SMITHY_HAMMER,
        &LEATHER_GLOVES_OF_MINING,
        &STUDDED_GLOVES_OF_MINING,
        &RINGMAIL_GLOVES_OF_MINING,
        &POWDER_OF_TEMPERAMENT,
    ] {
        types.register_item(t, &[]);
    }
}
