//! Default bodies of `Mobile`'s virtual members: the base behavior. A few defaults are the plain
//! value of a member that does nothing observable.

use muo_io::{GenericReader, GenericWriter};
use muo_primitives::{Direction, MessageType, Point2D, Point3D, TimeSpan, utility};

use crate::context_menu_entry::ContextMenuEntry;
use crate::guild::GuildId;
use crate::ids::{EntityId, ItemId, MobileId};
use crate::interfaces::{SpellId, WeaponRef};
use crate::item::{DeathMoveResult, TotalType};
use crate::lr_reason::LRReason;
use crate::map::MapId;
use crate::mobile::mobile_type::MOBILE_BASE;
use crate::mobile::part1::props_b;
use crate::mobile::{
    AccessLevel, ApplyPoisonResult, MobileDelta, ResistanceType, SpeechContext, StatType,
};
use crate::mods::{ModId, ResistanceMod, SkillMod, StatMod};
use crate::net_state::NetStateId;
use crate::poison::PoisonId;
use crate::property_list::PropertyList;
use crate::race::RaceId;
use crate::region::RegionId;
use crate::script_hooks::{self as sh, ScriptArg, ScriptHook};
use crate::skills::SkillName;
use crate::speech_event::SpeechEventArgs;
use crate::target::{TargetId, Targeted};
use crate::world::World;

/// `[Constructible]` `new Mobile()`.
pub fn construct(w: &mut World) -> MobileId {
    w.new_mobile(&MOBILE_BASE)
}

/// `x.racial_skill_bonus(..)`: the virtual call (an override runs; `racial_skill_bonus_base` is what `base.racial_skill_bonus(..)` reaches).
pub fn racial_skill_bonus(w: &World, id: MobileId) -> f64 {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.racial_skill_bonus)(w, id),
        None => racial_skill_bonus_base(w, id),
    }
}

pub fn racial_skill_bonus_base(_w: &World, _id: MobileId) -> f64 {
    0.0
}

/// `x.base_physical_resistance(..)`: the virtual call (an override runs; `base_physical_resistance_base` is what `base.base_physical_resistance(..)` reaches).
pub fn base_physical_resistance(w: &World, id: MobileId) -> i32 {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.base_physical_resistance)(w, id),
        None => base_physical_resistance_base(w, id),
    }
}

pub fn base_physical_resistance_base(_w: &World, _id: MobileId) -> i32 {
    0
}

/// `x.base_fire_resistance(..)`: the virtual call (an override runs; `base_fire_resistance_base` is what `base.base_fire_resistance(..)` reaches).
pub fn base_fire_resistance(w: &World, id: MobileId) -> i32 {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.base_fire_resistance)(w, id),
        None => base_fire_resistance_base(w, id),
    }
}

pub fn base_fire_resistance_base(_w: &World, _id: MobileId) -> i32 {
    0
}

/// `x.base_cold_resistance(..)`: the virtual call (an override runs; `base_cold_resistance_base` is what `base.base_cold_resistance(..)` reaches).
pub fn base_cold_resistance(w: &World, id: MobileId) -> i32 {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.base_cold_resistance)(w, id),
        None => base_cold_resistance_base(w, id),
    }
}

pub fn base_cold_resistance_base(_w: &World, _id: MobileId) -> i32 {
    0
}

/// `x.base_poison_resistance(..)`: the virtual call (an override runs; `base_poison_resistance_base` is what `base.base_poison_resistance(..)` reaches).
pub fn base_poison_resistance(w: &World, id: MobileId) -> i32 {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.base_poison_resistance)(w, id),
        None => base_poison_resistance_base(w, id),
    }
}

pub fn base_poison_resistance_base(_w: &World, _id: MobileId) -> i32 {
    0
}

/// `x.base_energy_resistance(..)`: the virtual call (an override runs; `base_energy_resistance_base` is what `base.base_energy_resistance(..)` reaches).
pub fn base_energy_resistance(w: &World, id: MobileId) -> i32 {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.base_energy_resistance)(w, id),
        None => base_energy_resistance_base(w, id),
    }
}

pub fn base_energy_resistance_base(_w: &World, _id: MobileId) -> i32 {
    0
}

/// `x.physical_resistance(..)`: the virtual call (an override runs; `physical_resistance_base` is what `base.physical_resistance(..)` reaches).
pub fn physical_resistance(w: &mut World, id: MobileId) -> i32 {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.physical_resistance)(w, id),
        None => physical_resistance_base(w, id),
    }
}

pub fn physical_resistance_base(w: &mut World, id: MobileId) -> i32 {
    get_resistance(w, id, ResistanceType::Physical)
}

/// `x.fire_resistance(..)`: the virtual call (an override runs; `fire_resistance_base` is what `base.fire_resistance(..)` reaches).
pub fn fire_resistance(w: &mut World, id: MobileId) -> i32 {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.fire_resistance)(w, id),
        None => fire_resistance_base(w, id),
    }
}

pub fn fire_resistance_base(w: &mut World, id: MobileId) -> i32 {
    get_resistance(w, id, ResistanceType::Fire)
}

/// `x.cold_resistance(..)`: the virtual call (an override runs; `cold_resistance_base` is what `base.cold_resistance(..)` reaches).
pub fn cold_resistance(w: &mut World, id: MobileId) -> i32 {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.cold_resistance)(w, id),
        None => cold_resistance_base(w, id),
    }
}

pub fn cold_resistance_base(w: &mut World, id: MobileId) -> i32 {
    get_resistance(w, id, ResistanceType::Cold)
}

/// `x.poison_resistance(..)`: the virtual call (an override runs; `poison_resistance_base` is what `base.poison_resistance(..)` reaches).
pub fn poison_resistance(w: &mut World, id: MobileId) -> i32 {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.poison_resistance)(w, id),
        None => poison_resistance_base(w, id),
    }
}

pub fn poison_resistance_base(w: &mut World, id: MobileId) -> i32 {
    get_resistance(w, id, ResistanceType::Poison)
}

/// `x.energy_resistance(..)`: the virtual call (an override runs; `energy_resistance_base` is what `base.energy_resistance(..)` reaches).
pub fn energy_resistance(w: &mut World, id: MobileId) -> i32 {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.energy_resistance)(w, id),
        None => energy_resistance_base(w, id),
    }
}

pub fn energy_resistance_base(w: &mut World, id: MobileId) -> i32 {
    get_resistance(w, id, ResistanceType::Energy)
}

/// `x.new_guild_display(..)`: the virtual call (an override runs; `new_guild_display_base` is what `base.new_guild_display(..)` reaches).
pub fn new_guild_display(w: &World, id: MobileId) -> bool {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.new_guild_display)(w, id),
        None => new_guild_display_base(w, id),
    }
}

pub fn new_guild_display_base(_w: &World, _id: MobileId) -> bool {
    false
}

/// `x.ignore_mobiles(..)`: the virtual call (an override runs; `ignore_mobiles_base` is what `base.ignore_mobiles(..)` reaches).
pub fn ignore_mobiles(w: &World, id: MobileId) -> bool {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.ignore_mobiles)(w, id),
        None => ignore_mobiles_base(w, id),
    }
}

pub fn ignore_mobiles_base(w: &World, id: MobileId) -> bool {
    w.mobile(id).is_some_and(|m| m.ignore_mobiles)
}

pub fn set_ignore_mobiles(w: &mut World, id: MobileId, value: bool) {
    crate::mobile::part1::props_a::set_ignore_mobiles_base(w, id, value);
}

pub fn paralyzed(w: &World, id: MobileId) -> bool {
    w.mobile(id).is_some_and(|m| m.paralyzed)
}

/// `x.set_paralyzed(..)`: the virtual call (an override runs; `set_paralyzed_base` is what `base.set_paralyzed(..)` reaches).
pub fn set_paralyzed(w: &mut World, id: MobileId, a2: bool) {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.set_paralyzed)(w, id, a2),
        None => set_paralyzed_base(w, id, a2),
    }
}

pub fn set_paralyzed_base(w: &mut World, id: MobileId, value: bool) {
    crate::mobile::part1::props_a::set_paralyzed_base(w, id, value);
}

pub fn regen_through_poison(w: &World, _id: MobileId) -> bool {
    crate::mobile::part1::props_a::global_regen_through_poison(w)
}

/// `x.can_regen_hits(..)`: the virtual call (an override runs; `can_regen_hits_base` is what `base.can_regen_hits(..)` reaches).
pub fn can_regen_hits(w: &World, id: MobileId) -> bool {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.can_regen_hits)(w, id),
        None => can_regen_hits_base(w, id),
    }
}

pub fn can_regen_hits_base(w: &World, id: MobileId) -> bool {
    w.mobile(id).is_some_and(|m| {
        (m.ty.alive)(w, id) && ((m.ty.regen_through_poison)(w, id) || m.poison.is_none())
    })
}

/// `x.can_regen_stam(..)`: the virtual call (an override runs; `can_regen_stam_base` is what `base.can_regen_stam(..)` reaches).
pub fn can_regen_stam(w: &World, id: MobileId) -> bool {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.can_regen_stam)(w, id),
        None => can_regen_stam_base(w, id),
    }
}

pub fn can_regen_stam_base(w: &World, id: MobileId) -> bool {
    w.mobile(id).is_some_and(|m| (m.ty.alive)(w, id))
}

/// `x.can_regen_mana(..)`: the virtual call (an override runs; `can_regen_mana_base` is what `base.can_regen_mana(..)` reaches).
pub fn can_regen_mana(w: &World, id: MobileId) -> bool {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.can_regen_mana)(w, id),
        None => can_regen_mana_base(w, id),
    }
}

pub fn can_regen_mana_base(w: &World, id: MobileId) -> bool {
    w.mobile(id).is_some_and(|m| (m.ty.alive)(w, id))
}

pub fn combatant(w: &World, id: MobileId) -> Option<MobileId> {
    w.mobile(id).and_then(|m| m.combatant)
}

/// `x.set_combatant(..)`: the virtual call (an override runs; `set_combatant_base` is what `base.set_combatant(..)` reaches).
pub fn set_combatant(w: &mut World, id: MobileId, a2: Option<MobileId>) {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.set_combatant)(w, id, a2),
        None => set_combatant_base(w, id, a2),
    }
}

pub fn set_combatant_base(w: &mut World, id: MobileId, value: Option<MobileId>) {
    crate::mobile::part1::props_a::set_combatant_base(w, id, value);
}

/// `x.is_dead_bonded_pet(..)`: the virtual call (an override runs; `is_dead_bonded_pet_base` is what `base.is_dead_bonded_pet(..)` reaches).
pub fn is_dead_bonded_pet(w: &World, id: MobileId) -> bool {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.is_dead_bonded_pet)(w, id),
        None => is_dead_bonded_pet_base(w, id),
    }
}

pub fn is_dead_bonded_pet_base(_w: &World, _id: MobileId) -> bool {
    false
}

/// `x.armor_rating(..)`: the virtual call (an override runs; `armor_rating_base` is what `base.armor_rating(..)` reaches).
pub fn armor_rating(w: &World, id: MobileId) -> f64 {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.armor_rating)(w, id),
        None => armor_rating_base(w, id),
    }
}

pub fn armor_rating_base(_w: &World, _id: MobileId) -> f64 {
    0.0
}

pub fn alive(w: &World, id: MobileId) -> bool {
    w.mobile(id)
        .is_some_and(|m| !m.deleted && (!m.player || !m.body.is_ghost()))
}

pub fn retain_pack_locs_on_death(w: &World, _id: MobileId) -> bool {
    crate::mobile::part1::props_a::core_aos(w)
}

pub fn show_visible_damage(w: &World, _id: MobileId) -> bool {
    crate::mobile::part1::props_a::default_show_visible_damage(w)
}

pub fn can_see_visible_damage(w: &World, _id: MobileId) -> bool {
    crate::mobile::part1::props_a::default_can_see_visible_damage(w)
}

/// `x.should_check_stat_timers(..)`: the virtual call (an override runs; `should_check_stat_timers_base` is what `base.should_check_stat_timers(..)` reaches).
pub fn should_check_stat_timers(w: &World, id: MobileId) -> bool {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.should_check_stat_timers)(w, id),
        None => should_check_stat_timers_base(w, id),
    }
}

pub fn should_check_stat_timers_base(_w: &World, _id: MobileId) -> bool {
    true
}

/// `x.max_weight(..)`: the virtual call (an override runs; `max_weight_base` is what `base.max_weight(..)` reaches).
pub fn max_weight(w: &World, id: MobileId) -> i32 {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.max_weight)(w, id),
        None => max_weight_base(w, id),
    }
}

pub fn max_weight_base(_w: &World, _id: MobileId) -> i32 {
    i32::MAX
}

/// `x.luck(..)`: the virtual call (an override runs; `luck_base` is what `base.luck(..)` reaches).
pub fn luck(w: &World, id: MobileId) -> i32 {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.luck)(w, id),
        None => luck_base(w, id),
    }
}

pub fn luck_base(_w: &World, _id: MobileId) -> i32 {
    0
}

/// `x.hue(..)`: the virtual call (an override runs; `hue_base` is what `base.hue(..)` reaches).
pub fn hue(w: &World, id: MobileId) -> i32 {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.hue)(w, id),
        None => hue_base(w, id),
    }
}

pub fn hue_base(w: &World, id: MobileId) -> i32 {
    w.mobile(id)
        .map_or(0, |m| if m.hue_mod != -1 { m.hue_mod } else { m.hue })
}

/// `x.set_hue(..)`: the virtual call (an override runs; `set_hue_base` is what `base.set_hue(..)` reaches).
pub fn set_hue(w: &mut World, id: MobileId, a2: i32) {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.set_hue)(w, id, a2),
        None => set_hue_base(w, id, a2),
    }
}

pub fn set_hue_base(w: &mut World, id: MobileId, value: i32) {
    let Some(m) = w.mobile_mut(id) else { return };
    let old_hue = m.hue;

    if old_hue != value {
        m.hue = value;
        props_b::delta(w, id, MobileDelta::HUE);
    }
}

/// `x.name(..)`: the virtual call (an override runs; `name_base` is what `base.name(..)` reaches).
pub fn name(w: &World, id: MobileId) -> Option<String> {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.name)(w, id),
        None => name_base(w, id),
    }
}

pub fn name_base(w: &World, id: MobileId) -> Option<String> {
    w.mobile(id)
        .and_then(|m| m.name_mod.clone().or_else(|| m.name.clone()))
}

/// `x.set_name(..)`: the virtual call (an override runs; `set_name_base` is what `base.set_name(..)` reaches).
pub fn set_name(w: &mut World, id: MobileId, a2: Option<String>) {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.set_name)(w, id, a2),
        None => set_name_base(w, id, a2),
    }
}

pub fn set_name_base(w: &mut World, id: MobileId, value: Option<String>) {
    props_b::set_name_base(w, id, value);
}

/// `x.weapon(..)`: the virtual call (an override runs; `weapon_base` is what `base.weapon(..)` reaches).
pub fn weapon(w: &mut World, id: MobileId) -> Option<WeaponRef> {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.weapon)(w, id),
        None => weapon_base(w, id),
    }
}

pub fn weapon_base(w: &mut World, id: MobileId) -> Option<WeaponRef> {
    props_b::weapon_base(w, id)
}

/// `x.keeps_items_on_death(..)`: the virtual call (an override runs; `keeps_items_on_death_base` is what `base.keeps_items_on_death(..)` reaches).
pub fn keeps_items_on_death(w: &World, id: MobileId) -> bool {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.keeps_items_on_death)(w, id),
        None => keeps_items_on_death_base(w, id),
    }
}

pub fn keeps_items_on_death_base(w: &World, id: MobileId) -> bool {
    w.mobile(id)
        .is_some_and(|m| m.access_level > AccessLevel::Player)
}

/// `x.murderer(..)`: the virtual call (an override runs; `murderer_base` is what `base.murderer(..)` reaches).
pub fn murderer(w: &World, id: MobileId) -> bool {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.murderer)(w, id),
        None => murderer_base(w, id),
    }
}

pub fn murderer_base(w: &World, id: MobileId) -> bool {
    w.mobile(id).is_some_and(|m| m.kills >= 5)
}

pub fn criminal(w: &World, id: MobileId) -> bool {
    w.mobile(id).is_some_and(|m| m.criminal)
}

pub fn set_criminal(w: &mut World, id: MobileId, value: bool) {
    props_b::set_criminal_base(w, id, value);
}

pub fn can_target(_w: &World, _id: MobileId) -> bool {
    true
}

/// `x.click_title(..)`: the virtual call (an override runs; `click_title_base` is what `base.click_title(..)` reaches).
pub fn click_title(w: &World, id: MobileId) -> bool {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.click_title)(w, id),
        None => click_title_base(w, id),
    }
}

pub fn click_title_base(_w: &World, _id: MobileId) -> bool {
    true
}

pub fn property_title(w: &World, id: MobileId) -> bool {
    !props_b::old_property_titles(w) || w.mobile(id).is_some_and(|m| (m.ty.click_title)(w, id))
}

/// `x.show_fame_title(..)`: the virtual call (an override runs; `show_fame_title_base` is what `base.show_fame_title(..)` reaches).
pub fn show_fame_title(w: &World, id: MobileId) -> bool {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.show_fame_title)(w, id),
        None => show_fame_title_base(w, id),
    }
}

pub fn show_fame_title_base(_w: &World, _id: MobileId) -> bool {
    true
}

/// `x.str(..)`: the virtual call (an override runs; `str_base` is what `base.str(..)` reaches).
pub fn str(w: &World, id: MobileId) -> i32 {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.str)(w, id),
        None => str_base(w, id),
    }
}

pub fn str_base(w: &World, id: MobileId) -> i32 {
    w.mobile(id).map_or(0, |m| {
        (m.str + props_b::get_stat_offset(w, id, StatType::STR)).clamp(1, 65000)
    })
}

pub fn set_str(w: &mut World, id: MobileId, value: i32) {
    if w.mobile(id).is_some_and(|m| m.stat_mods.is_empty()) {
        props_b::set_raw_str(w, id, value);
    }
}

/// `x.dex(..)`: the virtual call (an override runs; `dex_base` is what `base.dex(..)` reaches).
pub fn dex(w: &World, id: MobileId) -> i32 {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.dex)(w, id),
        None => dex_base(w, id),
    }
}

pub fn dex_base(w: &World, id: MobileId) -> i32 {
    w.mobile(id).map_or(0, |m| {
        (m.dex + props_b::get_stat_offset(w, id, StatType::DEX)).clamp(0, 65000)
    })
}

pub fn set_dex(w: &mut World, id: MobileId, value: i32) {
    if w.mobile(id).is_some_and(|m| m.stat_mods.is_empty()) {
        props_b::set_raw_dex(w, id, value);
    }
}

/// `x.int(..)`: the virtual call (an override runs; `int_base` is what `base.int(..)` reaches).
pub fn int(w: &World, id: MobileId) -> i32 {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.int)(w, id),
        None => int_base(w, id),
    }
}

pub fn int_base(w: &World, id: MobileId) -> i32 {
    w.mobile(id).map_or(0, |m| {
        (m.int + props_b::get_stat_offset(w, id, StatType::INT)).clamp(0, 65000)
    })
}

pub fn set_int(w: &mut World, id: MobileId, value: i32) {
    if w.mobile(id).is_some_and(|m| m.stat_mods.is_empty()) {
        props_b::set_raw_int(w, id, value);
    }
}

/// `x.hits_max(..)`: the virtual call (an override runs; `hits_max_base` is what `base.hits_max(..)` reaches).
pub fn hits_max(w: &World, id: MobileId) -> i32 {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.hits_max)(w, id),
        None => hits_max_base(w, id),
    }
}

pub fn hits_max_base(w: &World, id: MobileId) -> i32 {
    w.mobile(id).map_or(0, |m| 50 + (m.ty.str)(w, id) / 2)
}

/// `x.stam_max(..)`: the virtual call (an override runs; `stam_max_base` is what `base.stam_max(..)` reaches).
pub fn stam_max(w: &World, id: MobileId) -> i32 {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.stam_max)(w, id),
        None => stam_max_base(w, id),
    }
}

pub fn stam_max_base(w: &World, id: MobileId) -> i32 {
    w.mobile(id).map_or(0, |m| (m.ty.dex)(w, id))
}

/// `x.mana_max(..)`: the virtual call (an override runs; `mana_max_base` is what `base.mana_max(..)` reaches).
pub fn mana_max(w: &World, id: MobileId) -> i32 {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.mana_max)(w, id),
        None => mana_max_base(w, id),
    }
}

pub fn mana_max_base(w: &World, id: MobileId) -> i32 {
    w.mobile(id).map_or(0, |m| (m.ty.int)(w, id))
}

pub fn hued_item_id(w: &World, id: MobileId) -> i32 {
    w.mobile(id)
        .map_or(0, |m| if m.female { 0x2107 } else { 0x2106 })
}

/// `x.get_properties(..)`: the virtual call (an override runs; `get_properties_base` is what `base.get_properties(..)` reaches).
pub fn get_properties(w: &World, id: MobileId, a2: &mut PropertyList) {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.get_properties)(w, id, a2),
        None => get_properties_base(w, id, a2),
    }
}

pub fn get_properties_base(w: &World, id: MobileId, list: &mut PropertyList) {
    if let Some(m) = w.mobile(id) {
        (m.ty.add_name_properties)(w, id, list);
    }
}

/// `x.serialize(..)`: the virtual call (an override runs; `serialize_base` is what `base.serialize(..)` reaches).
pub fn serialize(w: &World, id: MobileId, a2: &mut dyn GenericWriter) {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.serialize)(w, id, a2),
        None => serialize_base(w, id, a2),
    }
}

pub fn serialize_base(w: &World, id: MobileId, writer: &mut dyn GenericWriter) {
    crate::mobile::part1::lifecycle::serialize(w, id, writer);
}

/// `x.delete(..)`: the virtual call (an override runs; `delete_base` is what `base.delete(..)` reaches).
pub fn delete(w: &mut World, id: MobileId) {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.delete)(w, id),
        None => delete_base(w, id),
    }
}

pub fn delete_base(w: &mut World, id: MobileId) {
    crate::mobile::part1::lifecycle::delete(w, id);
}

/// `x.skip_serialization(..)`: the virtual call (an override runs; `skip_serialization_base` is what `base.skip_serialization(..)` reaches).
pub fn skip_serialization(w: &World, id: MobileId) -> bool {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.skip_serialization)(w, id),
        None => skip_serialization_base(w, id),
    }
}

pub fn skip_serialization_base(_w: &World, _id: MobileId) -> bool {
    false
}

/// `MoveToWorld` (the base body: what a `base.MoveToWorld` call reaches; the vtable entry)
pub fn move_to_world_base(w: &mut World, id: MobileId, new_location: Point3D, map: MapId) {
    crate::mobile::part1::lifecycle::move_to_world(w, id, new_location, map);
}

/// `mobile.MoveToWorld(..)`: the virtual call (PlayerMobile rechecks the town protection, NatureFury ..).
pub fn move_to_world(w: &mut World, id: MobileId, new_location: Point3D, map: MapId) {
    let Some(ty) = w.mobile(id).map(|m| m.ty) else {
        return;
    };
    (ty.move_to_world)(w, id, new_location, map);
}

/// `x.process_delta(..)`: the virtual call (an override runs; `process_delta_base` is what `base.process_delta(..)` reaches).
pub fn process_delta(w: &mut World, id: MobileId) {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.process_delta)(w, id),
        None => process_delta_base(w, id),
    }
}

pub fn process_delta_base(w: &mut World, id: MobileId) {
    crate::mobile::part1::lifecycle::process_delta(w, id);
}

/// `x.on_before_spawn(..)`: the virtual call (an override runs; `on_before_spawn_base` is what `base.on_before_spawn(..)` reaches).
pub fn on_before_spawn(w: &mut World, id: MobileId, a2: Point3D, a3: MapId) {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.on_before_spawn)(w, id, a2, a3),
        None => on_before_spawn_base(w, id, a2, a3),
    }
}

pub fn on_before_spawn_base(_w: &mut World, _id: MobileId, _location: Point3D, _map: MapId) {}

/// `x.on_after_spawn(..)`: the virtual call (an override runs; `on_after_spawn_base` is what `base.on_after_spawn(..)` reaches).
pub fn on_after_spawn(w: &mut World, id: MobileId) {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.on_after_spawn)(w, id),
        None => on_after_spawn_base(w, id),
    }
}

pub fn on_after_spawn_base(_w: &mut World, _id: MobileId) {}

pub fn in_range_point2d(w: &World, id: MobileId, p: Point2D, range: i32) -> bool {
    w.mobile(id)
        .is_some_and(|m| utility::in_range(p.x, p.y, m.location.x, m.location.y, range))
}

pub fn in_range_point3d(w: &World, id: MobileId, p: Point3D, range: i32) -> bool {
    w.mobile(id)
        .is_some_and(|m| utility::in_range(p.x, p.y, m.location.x, m.location.y, range))
}

pub fn in_range_ipoint2d(w: &World, id: MobileId, p: Point2D, range: i32) -> bool {
    w.mobile(id)
        .is_some_and(|m| utility::in_range(p.x, p.y, m.location.x, m.location.y, range))
}

/// `x.on_race_change(..)`: the virtual call (an override runs; `on_race_change_base` is what `base.on_race_change(..)` reaches).
pub fn on_race_change(w: &mut World, id: MobileId, a2: RaceId) {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.on_race_change)(w, id, a2),
        None => on_race_change_base(w, id, a2),
    }
}

pub fn on_race_change_base(_w: &mut World, _id: MobileId, _old_race: RaceId) {}

pub fn compute_light_levels(w: &mut World, id: MobileId) -> (i32, i32) {
    crate::mobile::part1::lifecycle::compute_light_levels(w, id)
}

/// `x.compute_base_light_levels(..)`: the virtual call (an override runs; `compute_base_light_levels_base` is what `base.compute_base_light_levels(..)` reaches).
pub fn compute_base_light_levels(w: &World, id: MobileId) -> (i32, i32) {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.compute_base_light_levels)(w, id),
        None => compute_base_light_levels_base(w, id),
    }
}

pub fn compute_base_light_levels_base(w: &World, id: MobileId) -> (i32, i32) {
    w.mobile(id).map_or((0, 0), |m| (0, m.light_level))
}

/// `x.check_light_levels(..)`: the virtual call (an override runs; `check_light_levels_base` is what `base.check_light_levels(..)` reaches).
pub fn check_light_levels(w: &mut World, id: MobileId, a2: bool) {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.check_light_levels)(w, id, a2),
        None => check_light_levels_base(w, id, a2),
    }
}

pub fn check_light_levels_base(_w: &mut World, _id: MobileId, _force_resend: bool) {}

pub fn update_resistances(w: &mut World, id: MobileId) {
    crate::mobile::part1::lifecycle::update_resistances(w, id);
}

pub fn get_resistance(w: &mut World, id: MobileId, ty: ResistanceType) -> i32 {
    crate::mobile::part1::lifecycle::get_resistance(w, id, ty)
}

pub fn add_resistance_mod(w: &mut World, id: MobileId, mod_: ResistanceMod) {
    crate::mobile::part1::lifecycle::add_resistance_mod(w, id, mod_);
}

pub fn get_resistance_mod(w: &World, id: MobileId, name: &str) -> Option<usize> {
    crate::mobile::part1::lifecycle::get_resistance_mod(w, id, name)
}

pub fn remove_resistance_mod_by_name(w: &mut World, id: MobileId, name: &str) {
    crate::mobile::part1::lifecycle::remove_resistance_mod_by_name(w, id, name);
}

pub fn remove_resistance_mod(w: &mut World, id: MobileId, mod_: ModId) {
    crate::mobile::part1::lifecycle::remove_resistance_mod(w, id, mod_);
}

pub fn compute_resistances(w: &mut World, id: MobileId) {
    crate::mobile::part1::lifecycle::compute_resistances(w, id);
}

/// `x.get_min_resistance(..)`: the virtual call (an override runs; `get_min_resistance_base` is what `base.get_min_resistance(..)` reaches).
pub fn get_min_resistance(w: &World, id: MobileId, a2: ResistanceType) -> i32 {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.get_min_resistance)(w, id, a2),
        None => get_min_resistance_base(w, id, a2),
    }
}

pub fn get_min_resistance_base(_w: &World, _id: MobileId, _ty: ResistanceType) -> i32 {
    i32::MIN
}

/// `x.get_max_resistance(..)`: the virtual call (an override runs; `get_max_resistance_base` is what `base.get_max_resistance(..)` reaches).
pub fn get_max_resistance(w: &World, id: MobileId, a2: ResistanceType) -> i32 {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.get_max_resistance)(w, id, a2),
        None => get_max_resistance_base(w, id, a2),
    }
}

pub fn get_max_resistance_base(w: &World, id: MobileId, ty: ResistanceType) -> i32 {
    crate::mobile::part1::lifecycle::get_max_resistance(w, id, ty)
}

pub fn send_properties_to(w: &mut World, id: MobileId, ns: NetStateId) {
    crate::mobile::part1::lifecycle::send_properties_to(w, id, ns);
}

/// `x.on_aos_single_click(..)`: the virtual call (an override runs; `on_aos_single_click_base` is what `base.on_aos_single_click(..)` reaches).
pub fn on_aos_single_click(w: &mut World, id: MobileId, a2: MobileId) {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.on_aos_single_click)(w, id, a2),
        None => on_aos_single_click_base(w, id, a2),
    }
}

pub fn on_aos_single_click_base(w: &mut World, id: MobileId, from: MobileId) {
    crate::mobile::part1::lifecycle::on_aos_single_click(w, id, from);
}

/// `x.apply_name_suffix(..)`: the virtual call (an override runs; `apply_name_suffix_base` is what `base.apply_name_suffix(..)` reaches).
pub fn apply_name_suffix(w: &World, id: MobileId, a2: &str) -> String {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.apply_name_suffix)(w, id, a2),
        None => apply_name_suffix_base(w, id, a2),
    }
}

pub fn apply_name_suffix_base(_w: &World, _id: MobileId, suffix: &str) -> String {
    suffix.to_owned()
}

/// `x.add_name_properties(..)`: the virtual call (an override runs; `add_name_properties_base` is what `base.add_name_properties(..)` reaches).
pub fn add_name_properties(w: &World, id: MobileId, a2: &mut PropertyList) {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.add_name_properties)(w, id, a2),
        None => add_name_properties_base(w, id, a2),
    }
}

pub fn add_name_properties_base(w: &World, id: MobileId, list: &mut PropertyList) {
    crate::mobile::part1::lifecycle::add_name_properties(w, id, list);
}

pub fn get_child_properties(_w: &World, _id: MobileId, _list: &mut PropertyList, _item: ItemId) {}

pub fn get_child_name_properties(
    _w: &World,
    _id: MobileId,
    _list: &mut PropertyList,
    _item: ItemId,
) {
}

/// `x.on_skill_invalidated(..)`: the virtual call (an override runs; `on_skill_invalidated_base` is what `base.on_skill_invalidated(..)` reaches).
pub fn on_skill_invalidated(w: &mut World, id: MobileId, a2: SkillName) {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.on_skill_invalidated)(w, id, a2),
        None => on_skill_invalidated_base(w, id, a2),
    }
}

pub fn on_skill_invalidated_base(_w: &mut World, _id: MobileId, _skill: SkillName) {}

pub fn validate_skill_mods(w: &mut World, id: MobileId) {
    let len = w.mobile(id).map_or(0, |m| m.skill_mods.len());
    for i in (0..len).rev() {
        let Some(m) = w.mobile(id) else { return };
        let Some(sm) = m.skill_mods.get(i) else {
            continue;
        };
        if !sm.check_condition(w) {
            let mut sm = w.mobile_mut(id).unwrap().skill_mods.remove(i);
            sm.owner = None;
            crate::skills::update(w, id, sm.skill);
        }
    }
}

pub fn add_skill_mod(w: &mut World, id: MobileId, mod_: SkillMod) {
    let Some(ty) = w.mobile(id).map(|m| m.ty) else {
        return;
    };
    (ty.validate_skill_mods)(w, id);
    let skill = mod_.skill;
    let Some(m) = w.mobile_mut(id) else { return };
    let mut mod_ = mod_;
    mod_.owner = Some(id);
    m.skill_mods.push(mod_);
    crate::skills::update(w, id, skill);
}

pub fn get_skill_mod(w: &World, id: MobileId, name: &str) -> Option<usize> {
    // `ValidateSkillMods()` would run first, but a `&World` can't, so a mod whose condition has lapsed is
    // skipped here and removed by the next validation
    w.mobile(id)?
        .skill_mods
        .iter()
        .position(|sm| sm.check_condition(w) && sm.name.as_deref() == Some(name))
}

pub fn remove_skill_mod_by_name(w: &mut World, id: MobileId, name: &str) {
    let Some(m) = w.mobile(id) else { return };
    for i in (0..m.skill_mods.len()).rev() {
        let Some(m) = w.mobile_mut(id) else { return };
        if m.skill_mods
            .get(i)
            .is_some_and(|sm| sm.name.as_deref() == Some(name))
        {
            let mut sm = m.skill_mods.remove(i);
            sm.owner = None;
            crate::skills::update(w, id, sm.skill);
        }
    }
    if let Some(ty) = w.mobile(id).map(|m| m.ty) {
        (ty.validate_skill_mods)(w, id);
    }
}

pub fn remove_skill_mod(w: &mut World, id: MobileId, mod_: ModId) {
    let Some(m) = w.mobile_mut(id) else { return };
    if let Some(i) = m.skill_mods.iter().position(|sm| sm.id == mod_) {
        let mut sm = m.skill_mods.remove(i);
        sm.owner = None;
        crate::skills::update(w, id, sm.skill);
    }
    if let Some(ty) = w.mobile(id).map(|m| m.ty) {
        (ty.validate_skill_mods)(w, id);
    }
}

pub fn on_help_request(_w: &mut World, _id: MobileId, _from: MobileId) {}

/// `x.get_logout_delay(..)`: the virtual call (an override runs; `get_logout_delay_base` is what `base.get_logout_delay(..)` reaches).
pub fn get_logout_delay(w: &World, id: MobileId) -> TimeSpan {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.get_logout_delay)(w, id),
        None => get_logout_delay_base(w, id),
    }
}

pub fn get_logout_delay_base(w: &World, id: MobileId) -> TimeSpan {
    // Region.GetLogoutDelay(this)
    let Some(r) = w.mobile(id).map(|m| m.region) else {
        return TimeSpan::ZERO;
    };
    match w.region(r).map(|r| r.ty) {
        Some(rty) => (rty.get_logout_delay)(w, r, id),
        None => TimeSpan::ZERO,
    }
}

pub fn paralyze(w: &mut World, id: MobileId, duration: TimeSpan) {
    let Some(m) = w.mobile(id) else { return };
    if !m.paralyzed {
        (m.ty.set_paralyzed)(w, id, true);
        crate::mobile::part1::combat::start_paralyze_timer(w, id, duration);
    }
}

pub fn freeze(w: &mut World, id: MobileId, duration: TimeSpan) {
    let Some(m) = w.mobile(id) else { return };
    if !m.frozen {
        crate::mobile::part1::props_a::set_frozen(w, id, true);
        crate::mobile::part1::combat::start_freeze_timer(w, id, duration);
    }
}

pub fn send_skill_message(w: &mut World, id: MobileId) {
    if crate::mobile::part1::combat::claim_action_message(w, id) {
        crate::mobile::part1::combat::send_skill_message_text(w, id);
    }
}

pub fn send_action_message(w: &mut World, id: MobileId) {
    if crate::mobile::part1::combat::claim_action_message(w, id) {
        crate::mobile::part1::combat::send_action_message_text(w, id);
    }
}

pub fn clear_hands(w: &mut World, id: MobileId) {
    use crate::layer::Layer;
    use crate::mobile::part2::items::find_item_on_layer;
    let Some(ty) = w.mobile(id).map(|m| m.ty) else {
        return;
    };
    if let Some(item) = find_item_on_layer(w, id, Layer::OneHanded) {
        (ty.clear_hand)(w, id, item);
    }
    if let Some(item) = find_item_on_layer(w, id, Layer::TwoHanded) {
        (ty.clear_hand)(w, id, item);
    }
}

pub fn clear_hand(w: &mut World, id: MobileId, item: ItemId) {
    let Some(it) = w.item(item) else { return };
    if it.movable() && !(it.ty.allow_equipped_cast)(w, item, id) {
        match w.mobile(id).and_then(|m| m.backpack) {
            None => {
                crate::mobile::part2::items::add_to_backpack(w, id, item);
            }
            Some(pack) => {
                if let Some(pty) = w.item(pack).map(|i| i.ty) {
                    (pty.drop_item)(w, pack, item);
                }
            }
        }
    }
}

pub fn attack(w: &mut World, id: MobileId, m: MobileId) {
    let Some(ty) = w.mobile(id).map(|m| m.ty) else {
        return;
    };
    if (ty.check_attack)(w, id, m) {
        (ty.set_combatant)(w, id, Some(m));
    }
}

pub fn check_attack(w: &mut World, id: MobileId, m: MobileId) -> bool {
    let (Some(a), Some(b)) = (w.mobile(id), w.mobile(m)) else {
        return false;
    };
    let ty = a.ty;
    crate::mobile::part1::combat::in_update_range(a.location, b.location)
        && (ty.can_see_mobile)(w, id, m)
        && crate::mobile::part1::combat::in_los(w, id, m)
}

/// `x.on_combatant_change(..)`: the virtual call (an override runs; `on_combatant_change_base` is what `base.on_combatant_change(..)` reaches).
pub fn on_combatant_change(w: &mut World, id: MobileId) {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.on_combatant_change)(w, id),
        None => on_combatant_change_base(w, id),
    }
}

pub fn on_combatant_change_base(_w: &mut World, _id: MobileId) {}

pub fn on_hunger_changed(_w: &mut World, _id: MobileId, _old_value: i32) {}

pub fn on_thirst_changed(_w: &mut World, _id: MobileId, _old_value: i32) {}

pub fn aggressive_action(w: &mut World, id: MobileId, aggressor: MobileId) {
    if let Some(m) = w.mobile(id) {
        (m.ty.aggressive_action_criminal)(w, id, aggressor, false);
    }
}

/// `x.aggressive_action_criminal(..)`: the virtual call (an override runs; `aggressive_action_criminal_base` is what `base.aggressive_action_criminal(..)` reaches).
pub fn aggressive_action_criminal(w: &mut World, id: MobileId, a2: MobileId, a3: bool) {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.aggressive_action_criminal)(w, id, a2, a3),
        None => aggressive_action_criminal_base(w, id, a2, a3),
    }
}

pub fn aggressive_action_criminal_base(
    w: &mut World,
    id: MobileId,
    aggressor: MobileId,
    criminal: bool,
) {
    if aggressor == id {
        return;
    }
    let mut args = crate::events::AggressiveActionEventArgs {
        aggressed: id,
        aggressor,
        criminal,
    };
    crate::events::invoke(w, |e| &mut e.aggressive_action, &mut args);
    crate::mobile::part1::combat::record_aggression(w, id, aggressor, criminal);
}

pub fn get_total(w: &World, id: MobileId, ty: TotalType) -> i32 {
    w.mobile(id).map_or(0, |m| match ty {
        TotalType::Gold => m.total_gold,
        TotalType::Items => m.total_items,
        TotalType::Weight => m.total_weight,
    })
}

pub fn update_total(w: &mut World, id: MobileId, sender: ItemId, ty: TotalType, delta: i32) {
    if delta == 0
        || w.item(sender)
            .is_some_and(|i| (i.ty.is_virtual_item)(w, sender))
    {
        return;
    }
    let Some(mty) = w.mobile(id).map(|m| m.ty) else {
        return;
    };
    match ty {
        TotalType::Gold => {
            w.mobile_mut(id).unwrap().total_gold += delta;
            (mty.delta)(w, id, MobileDelta::GOLD);
        }
        TotalType::Items => {
            w.mobile_mut(id).unwrap().total_items += delta;
        }
        TotalType::Weight => {
            let total = {
                let m = w.mobile_mut(id).unwrap();
                m.total_weight += delta;
                m.total_weight
            };
            (mty.delta)(w, id, MobileDelta::WEIGHT);
            (mty.on_weight_change)(w, id, total - delta);
        }
    }
}

pub fn update_totals(w: &mut World, id: MobileId) {
    let Some(m) = w.mobile_mut(id) else { return };
    let old_weight = m.total_weight;
    m.total_gold = 0;
    m.total_items = 0;
    m.total_weight = 0;
    let (mty, items, holding) = (m.ty, m.items.clone(), m.holding);

    // Item.PileWeight: ceil(Weight * Amount), Weight being the item's own or its default
    let pile_weight = |w: &World, item: ItemId| -> i32 {
        let Some(i) = w.item(item) else { return 0 };
        let weight = if i.weight >= 0.0 {
            i.weight
        } else {
            (i.ty.default_weight)(w, item)
        };
        (weight * f64::from(i.amount)).ceil() as i32
    };
    for item in items {
        let Some(ity) = w.item(item).map(|i| i.ty) else {
            continue;
        };
        (ity.update_totals)(w, item);
        if (ity.is_virtual_item)(w, item) {
            continue;
        }
        let gold = (ity.get_total)(w, item, TotalType::Gold);
        let count = (ity.get_total)(w, item, TotalType::Items);
        let weight = (ity.get_total)(w, item, TotalType::Weight) + pile_weight(w, item);
        if let Some(m) = w.mobile_mut(id) {
            m.total_gold += gold;
            m.total_items += count + 1;
            m.total_weight += weight;
        }
    }
    if let Some(h) = holding
        && let Some(hty) = w.item(h).map(|i| i.ty)
    {
        let weight = (hty.get_total)(w, h, TotalType::Weight) + pile_weight(w, h);
        if let Some(m) = w.mobile_mut(id) {
            m.total_weight += weight;
        }
    }
    let new_weight = w.mobile(id).map_or(0, |m| m.total_weight);
    if new_weight != old_weight {
        (mty.on_weight_change)(w, id, old_weight);
    }
}

pub fn on_target_change(_w: &mut World, _id: MobileId) {}

/// `x.check_context_menu_display(..)`: the virtual call (an override runs; `check_context_menu_display_base` is what `base.check_context_menu_display(..)` reaches).
pub fn check_context_menu_display(w: &World, id: MobileId, a2: EntityId) -> bool {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.check_context_menu_display)(w, id, a2),
        None => check_context_menu_display_base(w, id, a2),
    }
}

pub fn check_context_menu_display_base(_w: &World, _id: MobileId, _target: EntityId) -> bool {
    true
}

/// `x.on_move(..)`: the virtual call (an override runs; `on_move_base` is what `base.on_move(..)` reaches).
pub fn on_move(w: &mut World, id: MobileId, a2: Direction) -> bool {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.on_move)(w, id, a2),
        None => on_move_base(w, id, a2),
    }
}

pub fn on_move_base(w: &mut World, id: MobileId, d: Direction) -> bool {
    let Some(m) = w.mobile_mut(id) else {
        return false;
    };
    if m.hidden && m.access_level == AccessLevel::Player {
        let steps = m.allowed_stealth_steps;
        m.allowed_stealth_steps -= 1;
        let ty = m.ty;
        if steps <= 0 || d.is_running() || crate::mobile::part1::props_b::mounted(w, id) {
            (ty.revealing_action)(w, id);
        }
    }
    true
}

/// `x.check_movement(..)`: the virtual call (an override runs; `check_movement_base` is what `base.check_movement(..)` reaches).
pub fn check_movement(w: &mut World, id: MobileId, a2: Direction) -> (bool, i32) {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.check_movement)(w, id, a2),
        None => check_movement_base(w, id, a2),
    }
}

pub fn check_movement_base(w: &mut World, id: MobileId, d: Direction) -> (bool, i32) {
    crate::movement::check_movement(w, id, d)
}

/// `x.move_dir(..)`: the virtual call (an override runs; `move_dir_base` is what `base.move_dir(..)` reaches).
pub fn move_dir(w: &mut World, id: MobileId, a2: Direction) -> bool {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.move_dir)(w, id, a2),
        None => move_dir_base(w, id, a2),
    }
}

pub fn move_dir_base(w: &mut World, id: MobileId, d: Direction) -> bool {
    let Some(m) = w.mobile(id).filter(|m| !m.deleted) else {
        return false;
    };
    let (ty, old_location, facing) = (m.ty, m.location, m.direction);
    crate::mobile::part2::items::close_bank_box(w, id);

    let new_location;
    if (facing & Direction::MASK) == (d & Direction::MASK) {
        // we are actually moving (not just a direction change)
        let (ok, loc) = crate::mobile::part1::combat::can_move(w, id, d, old_location);
        if !ok {
            return false;
        }
        new_location = loc;
        (ty.disruptive_action)(w, id);
    } else {
        new_location = old_location;
    }

    crate::mobile::part1::combat::charge_movement(w, id, d);

    (ty.set_location)(w, id, new_location, false);
    crate::mobile::part2::effects::set_direction(w, id, d); // SetDirection(d): no delta (the property sets one)
    crate::mobile::part1::combat::notify_moved(w, id, old_location);
    (ty.on_after_move)(w, id, old_location);
    true
}

pub fn on_after_move(_w: &mut World, _id: MobileId, _old_location: Point3D) {}

/// `x.compute_movement_speed(..)`: the virtual call (an override runs; `compute_movement_speed_base` is what `base.compute_movement_speed(..)` reaches).
pub fn compute_movement_speed(w: &World, id: MobileId, a2: Direction, a3: bool) -> i32 {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.compute_movement_speed)(w, id, a2, a3),
        None => compute_movement_speed_base(w, id, a2, a3),
    }
}

pub fn compute_movement_speed_base(
    w: &World,
    id: MobileId,
    dir: Direction,
    _check_turning: bool,
) -> i32 {
    // shortcut: `Mounted` (a `&mut World` lookup that caches) read as a cached or worn mount item; the same
    // shortcut `net_glue` takes
    let mounted = w.mobile(id).is_some_and(|m| {
        m.mount_item.is_some()
            || crate::mobile::part2::items::find_item_on_layer(w, id, crate::layer::Layer::Mount)
                .is_some()
    });
    let mv = w.resources.get::<crate::movement::Movement>();
    let get = |f: fn(&crate::movement::Movement) -> i32| mv.map_or(0, f);
    match (mounted, dir.is_running()) {
        (true, true) => get(|m| m.run_mount_delay),
        (true, false) => get(|m| m.walk_mount_delay),
        (false, true) => get(|m| m.run_foot_delay),
        (false, false) => get(|m| m.walk_foot_delay),
    }
}

pub fn on_move_off(_w: &mut World, _id: MobileId, _m: MobileId) -> bool {
    true
}

/// `x.on_move_over(..)`: the virtual call (an override runs; `on_move_over_base` is what `base.on_move_over(..)` reaches).
pub fn on_move_over(w: &mut World, id: MobileId, a2: MobileId) -> bool {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.on_move_over)(w, id, a2),
        None => on_move_over_base(w, id, a2),
    }
}

pub fn on_move_over_base(w: &mut World, id: MobileId, m: MobileId) -> bool {
    let Some(this) = w.mobile(id) else {
        return true;
    };
    if this.map.is_none() || this.deleted {
        return true;
    }
    match w.mobile(m).map(|x| x.ty) {
        Some(mty) => (mty.check_shove)(w, m, id),
        None => true,
    }
}

/// `x.check_shove(..)`: the virtual call (an override runs; `check_shove_base` is what `base.check_shove(..)` reaches).
pub fn check_shove(w: &mut World, id: MobileId, a2: MobileId) -> bool {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.check_shove)(w, id, a2),
        None => check_shove_base(w, id, a2),
    }
}

pub fn check_shove_base(w: &mut World, id: MobileId, shoved: MobileId) -> bool {
    let (Some(m), Some(s)) = (w.mobile(id), w.mobile(shoved)) else {
        return true;
    };
    let free = m
        .map
        .and_then(|map| w.maps.get(map))
        .is_some_and(|map| map.rules.contains(crate::map::MapRules::FREE_MOVEMENT));
    if free {
        return true;
    }
    let ty = m.ty;
    if !(s.ty.alive)(w, shoved)
        || !(ty.alive)(w, id)
        || (s.ty.is_dead_bonded_pet)(w, shoved)
        || (ty.is_dead_bonded_pet)(w, id)
    {
        return true;
    }
    let shoved_hidden = s.hidden;
    if shoved_hidden && s.access_level > AccessLevel::Player {
        return true;
    }
    if !m.pushing {
        let staff = m.access_level > AccessLevel::Player;
        let stam = m.stam;
        w.mobile_mut(id).unwrap().pushing = true;
        let number;
        if staff {
            number = if shoved_hidden { 1019041 } else { 1019040 };
        } else if stam == (ty.stam_max)(w, id) {
            number = if shoved_hidden { 1019043 } else { 1019042 };
            crate::mobile::part1::props_b::set_stam(w, id, stam - 10);
            (ty.revealing_action)(w, id);
        } else {
            return false;
        }
        crate::mobile::part2::messages::send_localized_message(w, id, number, "", 0x3B2);
    }
    true
}

// ---- Second half ----

use crate::mobile::part1 as p1;
use crate::mobile::part2 as p2;

/// `x.on_movement(..)`: the virtual call (an override runs; `on_movement_base` is what `base.on_movement(..)` reaches).
pub fn on_movement(w: &mut World, id: MobileId, a2: MobileId, a3: Point3D) {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.on_movement)(w, id, a2, a3),
        None => on_movement_base(w, id, a2, a3),
    }
}

pub fn on_movement_base(_w: &mut World, _id: MobileId, _m: MobileId, _old_location: Point3D) {}

/// `x.criminal_action(..)`: the virtual call (an override runs; `criminal_action_base` is what `base.criminal_action(..)` reaches).
pub fn criminal_action(w: &mut World, id: MobileId, a2: bool) {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.criminal_action)(w, id, a2),
        None => criminal_action_base(w, id, a2),
    }
}

pub fn criminal_action_base(w: &mut World, id: MobileId, message: bool) {
    if w.mobile(id).is_none_or(|m| m.deleted) {
        return;
    }
    let ty = w.mobile(id).map(|m| m.ty).unwrap_or(&MOBILE_BASE);
    (ty.set_criminal)(w, id, true);
    p2::deps::region_on_criminal_action(w, id, message);
}

/// `x.is_snoop(..)`: the virtual call (an override runs; `is_snoop_base` is what `base.is_snoop(..)` reaches).
pub fn is_snoop(w: &World, id: MobileId, a2: MobileId) -> bool {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.is_snoop)(w, id, a2),
        None => is_snoop_base(w, id, a2),
    }
}

pub fn is_snoop_base(_w: &World, id: MobileId, from: MobileId) -> bool {
    from != id
}

pub fn check_resurrect(_w: &World, _id: MobileId) -> bool {
    true
}

pub fn on_before_resurrect(_w: &mut World, _id: MobileId) {}

pub fn on_after_resurrect(_w: &mut World, _id: MobileId) {}

/// `x.resurrect(..)`: the virtual call (an override runs; `resurrect_base` is what `base.resurrect(..)` reaches).
pub fn resurrect(w: &mut World, id: MobileId) {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.resurrect)(w, id),
        None => resurrect_base(w, id),
    }
}

pub fn resurrect_base(w: &mut World, id: MobileId) {
    let Some(ty) = w.mobile(id).map(|m| m.ty) else {
        return;
    };
    if (ty.alive)(w, id) {
        return;
    }
    if !p2::deps::region_on_resurrect(w, id) {
        return;
    }
    if !(ty.check_resurrect)(w, id) {
        return;
    }
    (ty.on_before_resurrect)(w, id);

    p2::items::close_bank_box(w, id);

    props_b::set_poison(w, id, None);
    props_b::set_warmode(w, id, false);

    props_b::set_hits(w, id, 10);
    let stam_max = (ty.stam_max)(w, id);
    props_b::set_stam(w, id, stam_max);
    props_b::set_mana(w, id, 0);

    props_b::set_body_mod(w, id, muo_primitives::Body::new(0));
    let race = p1::props_a::race(w, id);
    let body = w
        .resources
        .get::<crate::race::Races>()
        .and_then(|r| r.get(race))
        .map(|r| crate::race::alive_body(w, r, id));
    if let Some(b) = body {
        props_b::set_body(w, id, muo_primitives::Body::new(b));
    }

    (ty.process_delta)(w, id);

    let mut i = w.mobile(id).map_or(0, |m| m.items.len());
    while i > 0 {
        i -= 1;
        let Some(item) = w.mobile(id).and_then(|m| m.items.get(i).copied()) else {
            continue;
        };
        if let Some(it) = w.item(item)
            && it.item_id == 0x204E
        {
            let ity = it.ty;
            (ity.delete)(w, item);
        }
    }

    p2::effects::send_incoming_packet(w, id);

    (ty.on_after_resurrect)(w, id);
}

/// `x.on_delete(..)`: the virtual call (an override runs; `on_delete_base` is what `base.on_delete(..)` reaches).
pub fn on_delete(w: &mut World, id: MobileId) {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.on_delete)(w, id),
        None => on_delete_base(w, id),
    }
}

pub fn on_delete_base(w: &mut World, id: MobileId) {
    if let Some(sp) = w.mobile(id).and_then(|m| m.spawner) {
        p2::deps::spawner_remove(w, sp, EntityId::Mobile(id));
    }
    if let Some(m) = w.mobile_mut(id) {
        m.spawner = None;
    }
}

pub fn check_spell_cast(_w: &mut World, _id: MobileId, _spell: SpellId) -> bool {
    true
}

pub fn on_spell_cast(w: &mut World, id: MobileId, _spell: SpellId) {
    let now = w.tick_count;
    if let Some(ns) = w.mobile(id).and_then(|m| m.net_state)
        && let Some(d) = w.net_state_mut(ns)
        && now.wrapping_sub(d.movement.next_movement_time) > 0
    {
        d.movement.next_movement_time = now;
    }
}

pub fn on_weight_change(_w: &mut World, _id: MobileId, _old_value: i32) {}

/// `x.on_skill_change(..)`: the virtual call (an override runs; `on_skill_change_base` is what `base.on_skill_change(..)` reaches).
pub fn on_skill_change(w: &mut World, id: MobileId, a2: SkillName, a3: f64) {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.on_skill_change)(w, id, a2, a3),
        None => on_skill_change_base(w, id, a2, a3),
    }
}

pub fn on_skill_change_base(_w: &mut World, _id: MobileId, _skill: SkillName, _old_base: f64) {}

/// `x.on_after_delete(..)`: the virtual call (an override runs; `on_after_delete_base` is what `base.on_after_delete(..)` reaches).
pub fn on_after_delete(w: &mut World, id: MobileId) {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.on_after_delete)(w, id),
        None => on_after_delete_base(w, id),
    }
}

pub fn on_after_delete_base(w: &mut World, id: MobileId) {
    p1::combat::stop_aggr_expire(w, id);
    p1::combat::check_aggr_expire(w, id);

    for which in [
        p2::stats::Regen::Hits,
        p2::stats::Regen::Stam,
        p2::stats::Regen::Mana,
    ] {
        p2::stats::stop_regen_timer(w, id, which);
    }
    let fields: [fn(&mut crate::Mobile) -> &mut crate::timer::TimerHandle; 8] = [
        |m| &mut m.poison_timer,
        |m| &mut m.combat_timer,
        |m| &mut m.expire_combatant_timer,
        |m| &mut m.logout_timer,
        |m| &mut m.expire_criminal_timer,
        |m| &mut m.para_timer,
        |m| &mut m.frozen_timer,
        |m| &mut m.auto_manifest_timer,
    ];
    for f in fields {
        let Some(m) = w.mobile_mut(id) else { return };
        let mut h = *f(m);
        h.cancel(w);
        if let Some(m) = w.mobile_mut(id) {
            *f(m) = h;
        }
    }
}

/// `x.allow_skill_use(..)`: the virtual call (an override runs; `allow_skill_use_base` is what `base.allow_skill_use(..)` reaches).
pub fn allow_skill_use(w: &mut World, id: MobileId, a2: SkillName) -> bool {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.allow_skill_use)(w, id, a2),
        None => allow_skill_use_base(w, id, a2),
    }
}

pub fn allow_skill_use_base(_w: &mut World, _id: MobileId, _name: SkillName) -> bool {
    true
}

pub fn use_skill(w: &mut World, id: MobileId, name: SkillName) -> bool {
    crate::skills::use_skill(w, id, name as i32)
}

pub fn use_skill_id(w: &mut World, id: MobileId, skill_id: i32) -> bool {
    crate::skills::use_skill(w, id, skill_id)
}

/// `x.get_parent_move_result_for(..)`: the virtual call (an override runs; `get_parent_move_result_for_base` is what `base.get_parent_move_result_for(..)` reaches).
pub fn get_parent_move_result_for(w: &mut World, id: MobileId, a2: ItemId) -> DeathMoveResult {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.get_parent_move_result_for)(w, id, a2),
        None => get_parent_move_result_for_base(w, id, a2),
    }
}

pub fn get_parent_move_result_for_base(
    w: &mut World,
    id: MobileId,
    item: ItemId,
) -> DeathMoveResult {
    if let Some(ty) = w.item(item).map(|i| i.ty) {
        (ty.on_parent_death)(w, item, id)
    } else {
        DeathMoveResult::default()
    }
}

/// `x.get_inventory_move_result_for(..)`: the virtual call (an override runs; `get_inventory_move_result_for_base` is what `base.get_inventory_move_result_for(..)` reaches).
pub fn get_inventory_move_result_for(w: &mut World, id: MobileId, a2: ItemId) -> DeathMoveResult {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.get_inventory_move_result_for)(w, id, a2),
        None => get_inventory_move_result_for_base(w, id, a2),
    }
}

pub fn get_inventory_move_result_for_base(
    w: &mut World,
    id: MobileId,
    item: ItemId,
) -> DeathMoveResult {
    if let Some(ty) = w.item(item).map(|i| i.ty) {
        (ty.on_inventory_death)(w, item, id)
    } else {
        DeathMoveResult::default()
    }
}

pub fn kill(w: &mut World, id: MobileId) {
    let Some(ty) = w.mobile(id).map(|m| m.ty) else {
        return;
    };
    if !(ty.can_be_damaged)(w, id) {
        return;
    }
    if !(ty.alive)(w, id) || (ty.is_dead_bonded_pet)(w, id) {
        return;
    }
    if w.mobile(id).is_none_or(|m| m.deleted) {
        return;
    }
    if !p2::deps::region_on_before_death(w, id) {
        return;
    }
    if !(ty.on_before_death)(w, id) {
        return;
    }

    p2::items::close_bank_box(w, id);

    if let Some(ns) = w.mobile(id).and_then(|m| m.net_state) {
        p2::deps::cancel_all_trades(w, ns);
    }

    p2::deps::spell_on_caster_killed(w, id);

    if let Some(t) = w.mobile(id).and_then(|m| m.target) {
        crate::target::cancel(w, t, id, crate::target::TargetCancelType::Canceled);
    }

    (ty.disruptive_action)(w, id);

    props_b::set_warmode(w, id, false);

    p2::items::drop_holding(w, id);

    props_b::set_hits(w, id, 0);
    props_b::set_stam(w, id, 0);
    props_b::set_mana(w, id, 0);

    props_b::set_poison(w, id, None);
    (ty.set_combatant)(w, id, None);

    if w.mobile(id).is_some_and(|m| m.paralyzed) {
        (ty.set_paralyzed)(w, id, false);
    }
    if w.mobile(id).is_some_and(|m| m.frozen) {
        p1::props_a::set_frozen(w, id, false);
    }

    let mut content: Vec<ItemId> = Vec::new();
    let mut equip: Vec<ItemId> = Vec::new();
    let mut move_to_pack: Vec<ItemId> = Vec::new();

    let items_copy: Vec<ItemId> = w.mobile(id).map(|m| m.items.clone()).unwrap_or_default();
    let pack = props_b::backpack(w, id);

    for &item in &items_copy {
        if Some(item) == pack {
            continue;
        }
        match (ty.get_parent_move_result_for)(w, id, item) {
            DeathMoveResult::MoveToCorpse => {
                content.push(item);
                equip.push(item);
            }
            DeathMoveResult::MoveToBackpack => move_to_pack.push(item),
            _ => {}
        }
    }

    if let Some(pack) = pack {
        let pack_copy: Vec<ItemId> = w.item(pack).map(|p| p.items.clone()).unwrap_or_default();
        for &item in &pack_copy {
            if (ty.get_inventory_move_result_for)(w, id, item) == DeathMoveResult::MoveToCorpse {
                content.push(item);
            } else {
                move_to_pack.push(item);
            }
        }

        for &item in &move_to_pack {
            if (ty.retain_pack_locs_on_death)(w, id)
                && w.item(item)
                    .is_some_and(|i| i.parent == Some(EntityId::Item(pack)))
            {
                continue;
            }
            if let Some(pty) = w.item(pack).map(|p| p.ty) {
                (pty.drop_item)(w, pack, item);
            }
        }
    }

    let handler = p2::statics::read(w, |s| s.create_corpse_handler);
    let c = handler.and_then(|h| h(w, id, &content, &equip));

    if let Some(m) = w.mobile(id)
        && let Some(map) = m.map
    {
        let (loc, serial, own) = (m.location, m.serial, m.net_state);
        let corpse_serial = c.map_or(muo_primitives::Serial::ZERO, |c| c.0);
        for (ns, viewer) in p2::deps::clients_in_range(w, map, loc) {
            if Some(ns) != own {
                crate::packets::mobile::send_death_animation(w, ns, serial, corpse_serial);
                if !p2::deps::can_see_mobile(w, viewer, id) {
                    crate::packets::entity::send_remove_entity(w, ns, serial);
                }
            }
        }
    }

    p2::deps::region_on_death(w, id);
    if !sh::mobile(w, id, ScriptHook::Death, &[ScriptArg::Item(c)]) {
        (ty.on_death)(w, id, c);
    }
}

/// `x.on_before_death(..)`: the virtual call (an override runs; `on_before_death_base` is what `base.on_before_death(..)` reaches).
pub fn on_before_death(w: &mut World, id: MobileId) -> bool {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.on_before_death)(w, id),
        None => on_before_death_base(w, id),
    }
}

pub fn on_before_death_base(_w: &mut World, _id: MobileId) -> bool {
    true
}

/// `x.on_death(..)`: the virtual call (an override runs; `on_death_base` is what `base.on_death(..)` reaches).
pub fn on_death(w: &mut World, id: MobileId, a2: Option<ItemId>) {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.on_death)(w, id, a2),
        None => on_death_base(w, id, a2),
    }
}

pub fn on_death_base(w: &mut World, id: MobileId, _c: Option<ItemId>) {
    let Some(ty) = w.mobile(id).map(|m| m.ty) else {
        return;
    };
    let sound = (ty.get_death_sound)(w, id);
    if sound >= 0 {
        crate::effects::play_sound_entity(w, EntityId::Mobile(id), sound);
    }

    if !w.mobile(id).is_some_and(|m| m.player) {
        (ty.delete)(w, id);
        return;
    }

    if let Some(ns) = w.mobile(id).and_then(|m| m.net_state) {
        crate::packets::player::send_death_status(w, ns);
    }

    props_b::set_warmode(w, id, false);

    props_b::set_body_mod(w, id, muo_primitives::Body::new(0));
    let race = p1::props_a::race(w, id);
    let body = w
        .resources
        .get::<crate::race::Races>()
        .and_then(|r| r.get(race))
        .map(|r| crate::race::ghost_body(w, r, id));
    if let Some(b) = body {
        props_b::set_body(w, id, muo_primitives::Body::new(b));
    }

    let shroud = crate::item::ops::new_item_with_id(w, &crate::item::ITEM_BASE, 0x204E);
    crate::item::ops::set_movable(w, shroud, false);
    if let Some(sty) = w.item(shroud).map(|i| i.ty) {
        (sty.set_layer)(w, shroud, crate::layer::Layer::OuterTorso);
    }

    p2::items::add_item(w, id, shroud);

    if let Some(m) = w.mobile_mut(id) {
        m.items.retain(|&i| i != shroud);
        m.items.insert(0, shroud);
    }

    props_b::set_poison(w, id, None);
    (ty.set_combatant)(w, id, None);

    props_b::set_hits(w, id, 0);
    props_b::set_stam(w, id, 0);
    props_b::set_mana(w, id, 0);

    (ty.process_delta)(w, id);

    (ty.check_stat_timers)(w, id);
}

pub fn check_target(
    _w: &mut World,
    _id: MobileId,
    _from: MobileId,
    _targ: TargetId,
    _targeted: Targeted,
) -> bool {
    true
}

pub fn use_item(w: &mut World, id: MobileId, item: ItemId) {
    use crate::item::ops as io;
    let Some(it) = w.item(item) else { return };
    let ity = it.ty;
    if it.deleted() || it.quest_item() || w.mobile(id).is_none_or(|m| m.deleted) {
        return;
    }
    let Some(ty) = w.mobile(id).map(|m| m.ty) else {
        return;
    };

    (ty.disruptive_action)(w, id);

    if !p2::deps::spell_on_caster_using_object(w, id, EntityId::Item(item)) {
        return;
    }

    let root = io::root_parent(w, item);
    let mut okay = false;
    let loc = w.mobile(id).map_or(Point3D::default(), |m| m.location);
    let wl = io::get_world_location(w, item);

    if !p1::combat::in_update_range(loc, wl) {
        (ity.on_double_click_out_of_range)(w, item, id);
    } else if !(ty.can_see_item)(w, id, item) {
        (ity.on_double_click_cant_see)(w, item, id);
    } else if !(ity.is_accessible_to)(w, item, id) {
        let imap = w.item(item).and_then(|i| i.map);
        let rid = crate::region::find(w, wl, imap);
        let handled = match w.region(rid).map(|r| r.ty) {
            Some(rty) => (rty.send_inaccessible_message)(w, rid, item, id),
            None => false,
        };
        if !handled {
            (ity.on_double_click_not_accessible)(w, item, id);
        }
    } else if !p2::stats::check_alive(w, id, false) {
        (ity.on_double_click_dead)(w, item, id);
    } else if io::in_secure_trade(w, item) {
        (ity.on_double_click_secure_trade)(w, item, id);
    } else if !(ty.allow_item_use)(w, id, item) || !(ity.check_item_use)(w, item, id, item) {
    } else if let Some(EntityId::Mobile(rm)) = root
        && w.mobile(rm).is_some_and(|r| (r.ty.is_snoop)(w, rm, id))
    {
        (ity.on_snoop)(w, item, id);
    } else if p2::deps::region_on_double_click(w, id, Targeted::Entity(EntityId::Item(item))) {
        okay = true;
    }

    if okay {
        // TODO: Is this correct?
        if w.item(item).is_some_and(|i| !i.deleted()) {
            (ity.on_item_used)(w, item, id, item);
        }
        if w.item(item).is_some_and(|i| !i.deleted())
            && !sh::item(w, item, ScriptHook::Use, &[ScriptArg::Mobile(Some(id))])
        {
            (ity.on_double_click)(w, item, id);
        }
    }
}

pub fn use_mobile(w: &mut World, id: MobileId, m: MobileId) {
    if w.mobile(m).is_none_or(|o| o.deleted) || w.mobile(id).is_none_or(|o| o.deleted) {
        return;
    }
    let (ty, loc) = {
        let me = w.mobile(id).unwrap();
        (me.ty, me.location)
    };
    let (mty, mloc) = {
        let o = w.mobile(m).unwrap();
        (o.ty, o.location)
    };

    (ty.disruptive_action)(w, id);

    if !p2::deps::spell_on_caster_using_object(w, id, EntityId::Mobile(m)) {
        return;
    }

    if !p1::combat::in_update_range(loc, mloc) {
        (mty.on_double_click_out_of_range)(w, m, id);
    } else if !(ty.can_see_mobile)(w, id, m) {
        (mty.on_double_click_cant_see)(w, m, id);
    } else if !p2::stats::check_alive(w, id, false) {
        (mty.on_double_click_dead)(w, m, id);
    } else if p2::deps::region_on_double_click(w, id, Targeted::Entity(EntityId::Mobile(m)))
        && w.mobile(m).is_some_and(|o| !o.deleted)
        // a script's `on_use` on the class (an NPC that opens its own window) replaces the default
        && !sh::mobile(w, m, ScriptHook::Use, &[ScriptArg::Mobile(Some(id))])
    {
        (mty.on_double_click)(w, m, id);
    }
}

pub fn lift(w: &mut World, id: MobileId, item: ItemId, amount: i32) -> (bool, LRReason) {
    use crate::item::ops as io;
    let mut rejected = true;
    let mut reject = LRReason::Inspecific;
    let mut amount = amount;

    let Some(m) = w.mobile(id) else {
        return (rejected, reject);
    };
    let (ty, state) = (m.ty, m.net_state);
    if w.item(item).is_none() {
        return (rejected, reject);
    }
    let ity = w
        .item(item)
        .map(|i| i.ty)
        .unwrap_or(&crate::item::ITEM_BASE);

    let (access, next_action) = (m.access_level, m.next_action_time);
    if access >= AccessLevel::GameMaster || w.tick_count.wrapping_sub(next_action) >= 0 {
        if p2::stats::check_alive(w, id, true) {
            (ty.disruptive_action)(w, id);

            let wl = io::get_world_location(w, item);
            let loc = w.mobile(id).map_or(Point3D::default(), |m| m.location);
            if w.mobile(id).is_some_and(|m| m.holding.is_some()) {
                reject = LRReason::AreHolding;
            } else if access < AccessLevel::GameMaster && !utility::in_range_3d(loc, wl, 2) {
                reject = LRReason::OutOfRange;
            } else if !(ty.can_see_item)(w, id, item)
                || !p1::combat::in_los_object(w, id, p2::deps::los_item(item))
            {
                reject = LRReason::OutOfSight;
            } else if !(ity.verify_move)(w, item, id) || !(ity.is_accessible_to)(w, item, id) {
                reject = LRReason::CannotLift;
            } else if (ity.nontransferable)(w, item)
                && amount != w.item(item).map_or(0, |i| i.amount)
            {
                if w.item(item).is_some_and(|i| i.quest_item()) {
                    // Stacks of quest items cannot be unstacked.
                    p2::messages::send_localized_message(w, id, 1074868, "", 0x3B2);
                }
                reject = LRReason::CannotLift;
            } else if !(ity.check_lift)(w, item, id, item, &mut reject) {
            } else {
                let root = io::root_parent(w, item);
                let refuse = match root {
                    Some(EntityId::Mobile(rm)) => match w.mobile(rm).map(|r| r.ty) {
                        Some(t) => !(t.check_nonlocal_lift)(w, rm, id, item),
                        None => true,
                    },
                    _ => false,
                };
                if refuse {
                    reject = LRReason::TryToSteal;
                } else if !(ty.on_drag_lift)(w, id, item)
                    || !(ity.on_drag_lift)(w, item, id)
                    || !p2::stats::check_alive(w, id, true)
                {
                    reject = LRReason::Inspecific;
                } else {
                    io::set_last_moved(w, item);

                    if let Some(sp) = w.item(item).and_then(|i| i.spawner) {
                        p2::deps::spawner_remove(w, sp, EntityId::Item(item));
                        io::set_spawner(w, item, None);
                    }

                    let old_amount = w.item(item).map_or(0, |i| i.amount);
                    if old_amount <= 0 {
                        tracing::error!(
                            "Item {} ({:?}) has amount of {}, but must be at least 1",
                            ity.full_name,
                            item.0,
                            old_amount
                        );
                    } else {
                        amount = amount.clamp(1, old_amount);
                        if amount < old_amount {
                            p2::items::lift_item_dupe(w, item, amount);
                        }
                    }

                    let map = w.mobile(id).and_then(|m| m.map);
                    let root_ok = match root {
                        None => true,
                        Some(EntityId::Item(_)) => true,
                        Some(EntityId::Mobile(_)) => false,
                    };
                    if p1::props_a::drag_effects(w)
                        && let Some(map) = map
                        && root_ok
                    {
                        let root_item = match root {
                            Some(EntityId::Item(r)) => Some(r),
                            _ => None,
                        };
                        let (root_serial, root_loc) = match root_item.and_then(|r| w.item(r)) {
                            Some(r) => (r.serial, r.location),
                            None => (
                                muo_primitives::Serial::ZERO,
                                w.item(item).map_or(Point3D::default(), |i| i.location),
                            ),
                        };
                        let (fserial, floc) = w
                            .mobile(id)
                            .map_or((muo_primitives::Serial::ZERO, Point3D::default()), |m| {
                                (m.serial, m.location)
                            });
                        let (iid, ihue) = w.item(item).map_or((0, 0), |i| (i.item_id, i.hue));
                        for (ns, viewer) in p2::deps::clients_in_range(w, map, floc) {
                            if viewer != id
                                && p2::deps::can_see_mobile(w, viewer, id)
                                && p2::deps::in_los_mobile(w, viewer, id)
                                && p2::deps::can_see_entity(w, viewer, root)
                            {
                                p2::effects::send_drag_effect(
                                    w,
                                    ns,
                                    root_serial,
                                    root_loc,
                                    fserial,
                                    floc,
                                    iid,
                                    ihue,
                                    amount,
                                );
                            }
                        }
                    }

                    let (fix_loc, fix_map, should_fix) = {
                        let i = w.item(item);
                        (
                            i.map_or(Point3D::default(), |i| i.location),
                            i.and_then(|i| i.map),
                            i.is_some_and(|i| i.parent.is_none()),
                        )
                    };

                    io::record_bounce(w, item);
                    (ity.on_item_lifted)(w, item, id, item);
                    io::internalize(w, item);

                    p1::props_a::set_holding(w, id, Some(item));

                    let sound = (ity.get_lift_sound)(w, item, id);
                    p2::effects::send_sound(w, id, sound);

                    let delay = p1::props_a::action_delay(w);
                    let now = w.tick_count;
                    if let Some(m) = w.mobile_mut(id) {
                        m.next_action_time = now + i64::from(delay);
                    }

                    if let Some(fm) = fix_map
                        && should_fix
                    {
                        crate::map::surface::fix_column(w, fm, fix_loc.x, fix_loc.y);
                    }

                    reject = LRReason::Inspecific;
                    rejected = false;
                }
            }
        } else {
            reject = LRReason::Inspecific;
        }
    } else {
        (ty.send_action_message)(w, id);
        reject = LRReason::Inspecific;
    }

    if rejected && let Some(ns) = state {
        crate::packets::player::send_lift_reject(w, ns, reject);

        // A rejection must not resynchronize state this client was never sent (client-supplied serial).
        if w.item(item).is_none_or(|i| i.deleted()) || !io::is_sent_to(w, item, Some(id)) {
            return (rejected, reject);
        }

        let parent = w.item(item).and_then(|i| i.parent);
        match parent {
            Some(EntityId::Item(_)) => {
                crate::packets::container::send_container_content_update(w, ns, item)
            }
            Some(EntityId::Mobile(_)) => {
                // Don't re-show a duped-layer item on its own (see Item.IsDupedEquipLayer).
                if !io::is_duped_equip_layer(w, item) {
                    crate::packets::equipment::send_equip_update(w, ns, item);
                }
            }
            None => (ity.send_info_to)(w, item, ns, &[]),
        }

        if parent.is_some() && !io::is_duped_equip_layer(w, item) {
            (ity.send_opl_packet_to)(w, item, ns);
        }
    }

    (rejected, reject)
}

pub fn send_drop_effect(w: &mut World, id: MobileId, item: ItemId) {
    use crate::item::ops as io;
    if !p1::props_a::drag_effects(w) || w.item(item).is_none_or(|i| i.deleted()) {
        return;
    }
    let Some(m) = w.mobile(id) else { return };
    let (map, loc, serial) = (m.map, m.location, m.serial);
    let root = io::root_parent(w, item);
    let root_item = match root {
        Some(EntityId::Item(r)) => Some(r),
        _ => None,
    };
    let Some(map) = map else { return };
    if root.is_some() && root_item.is_none() {
        return;
    }
    let (root_serial, root_loc) = match root_item.and_then(|r| w.item(r)) {
        Some(r) => (r.serial, r.location),
        None => (
            muo_primitives::Serial::ZERO,
            w.item(item).map_or(Point3D::default(), |i| i.location),
        ),
    };
    let (iid, ihue, amount) = w
        .item(item)
        .map_or((0, 0, 0), |i| (i.item_id, i.hue, i.amount));
    for (ns, viewer) in p2::deps::clients_in_range(w, map, loc) {
        if w.net_state(ns).is_some_and(|d| d.caps.stygian_abyss())
            || viewer == id
            || !p2::deps::can_see_mobile(w, viewer, id)
            || !p2::deps::in_los_mobile(w, viewer, id)
            || !p2::deps::can_see_entity(w, viewer, root)
        {
            continue;
        }
        p2::effects::send_drag_effect(w, ns, serial, loc, root_serial, root_loc, iid, ihue, amount);
    }
}

/// The shared head and tail of the three `Drop` overloads: `action` is the drop itself.
fn drop_with(w: &mut World, id: MobileId, action: impl FnOnce(&mut World, ItemId) -> bool) -> bool {
    use crate::item::ops as io;
    let item = w.mobile(id).and_then(|m| m.holding);
    let valid = item.is_some_and(|i| {
        w.item(i)
            .is_some_and(|it| it.held_by == Some(id) && it.map == Some(MapId::INTERNAL))
    });

    p1::props_a::set_holding(w, id, None);

    let (true, Some(item)) = (valid, item) else {
        return false;
    };

    let mut bounced = true;

    io::set_last_moved(w, item);

    if !action(w, item) {
        io::bounce(w, item, id);
    } else {
        bounced = false;
    }

    io::clear_bounce(w, item);

    if !bounced {
        let ty = w.mobile(id).map(|m| m.ty).unwrap_or(&MOBILE_BASE);
        (ty.send_drop_effect)(w, id, item);
    }

    !bounced
}

/// `Drop(null, loc)` (Item or Mobile overload): a missing target bounces the held item.
pub fn drop_missing(w: &mut World, id: MobileId) -> bool {
    drop_with(w, id, |_, _| false)
}

pub fn drop_item(w: &mut World, id: MobileId, to: ItemId, loc: Point3D) -> bool {
    drop_with(w, id, |w, item| {
        let Some(ity) = w.item(item).map(|i| i.ty) else {
            return false;
        };
        w.item(to).is_some() && (ity.drop_to_item)(w, item, id, to, loc)
    })
}

pub fn drop_world(w: &mut World, id: MobileId, loc: Point3D) -> bool {
    drop_with(w, id, |w, item| {
        let Some(ity) = w.item(item).map(|i| i.ty) else {
            return false;
        };
        (ity.drop_to_world)(w, item, id, loc)
    })
}

pub fn drop_mobile(w: &mut World, id: MobileId, to: MobileId, loc: Point3D) -> bool {
    drop_with(w, id, |w, item| {
        let Some(ity) = w.item(item).map(|i| i.ty) else {
            return false;
        };
        w.mobile(to).is_some() && (ity.drop_to_mobile)(w, item, id, to, loc)
    })
}

/// `x.mutate_speech(..)`: the virtual call (an override runs; `mutate_speech_base` is what `base.mutate_speech(..)` reaches).
pub fn mutate_speech(
    w: &mut World,
    id: MobileId,
    a2: &mut Vec<MobileId>,
    a3: &mut String,
    a4: &mut SpeechContext,
) -> bool {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.mutate_speech)(w, id, a2, a3, a4),
        None => mutate_speech_base(w, id, a2, a3, a4),
    }
}

pub fn mutate_speech_base(
    w: &mut World,
    id: MobileId,
    _hears: &mut Vec<MobileId>,
    text: &mut String,
    context: &mut SpeechContext,
) -> bool {
    let Some(ty) = w.mobile(id).map(|m| m.ty) else {
        return false;
    };
    if (ty.alive)(w, id) {
        return false;
    }

    let ghost: Vec<char> = p2::statics::read(w, |s| s.ghost_chars.clone())
        .unwrap_or_else(|| p2::statics::DEFAULT_GHOST_CHARS.to_vec());

    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        if c != ' ' {
            out.push(
                utility::random_element(&mut w.rng, &ghost)
                    .copied()
                    .unwrap_or(c),
            );
        } else {
            out.push(' ');
        }
    }
    *text = out;

    *context = SpeechContext::Ghost;
    true
}

pub fn manifest(w: &mut World, id: MobileId, delay: TimeSpan) {
    props_b::set_warmode(w, id, true);
    let Some(m) = w.mobile_mut(id) else { return };
    let mut h = m.auto_manifest_timer;
    h.cancel(w);
    // AutoManifest: `if (!Alive) Warmode = false`
    let h = crate::timer::start(w, delay, TimeSpan::ZERO, 1, move |w| {
        if let Some(ty) = w.mobile(id).map(|m| m.ty)
            && !(ty.alive)(w, id)
        {
            props_b::set_warmode(w, id, false);
        }
    });
    if let Some(m) = w.mobile_mut(id) {
        m.auto_manifest_timer = h;
    }
}

pub fn check_speech_manifest(w: &mut World, id: MobileId) -> bool {
    let Some(m) = w.mobile(id) else { return false };
    let ty = m.ty;
    if (ty.alive)(w, id) {
        return false;
    }
    let delay = p1::props_a::auto_manifest_timeout(w);
    let (warmode, running) = (m.warmode, m.auto_manifest_timer.running(w));
    if delay > TimeSpan::ZERO && (!warmode || running) {
        (ty.manifest)(w, id, delay);
        return true;
    }
    false
}

pub fn check_hears_mutated_speech(
    w: &World,
    _id: MobileId,
    m: MobileId,
    context: SpeechContext,
) -> bool {
    context != SpeechContext::Ghost
        || w.mobile(m)
            .is_some_and(|o| (o.ty.alive)(w, m) && !p1::props_b::can_hear_ghosts(w, m))
}

/// `x.do_speech(..)`: the virtual call (an override runs; `do_speech_base` is what `base.do_speech(..)` reaches).
pub fn do_speech(w: &mut World, id: MobileId, a2: &str, a3: &[i32], a4: MessageType, a5: i32) {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.do_speech)(w, id, a2, a3, a4, a5),
        None => do_speech_base(w, id, a2, a3, a4, a5),
    }
}

pub fn do_speech_base(
    w: &mut World,
    id: MobileId,
    text: &str,
    keywords: &[i32],
    ty: MessageType,
    hue: i32,
) {
    if w.mobile(id).is_none_or(|m| m.deleted)
        || crate::commands::CommandSystem::handle(w, id, text, ty)
    {
        return;
    }
    let mty = w.mobile(id).map(|m| m.ty).unwrap_or(&MOBILE_BASE);

    let mut range = 15;
    let mut ty = ty;

    if ty == MessageType::REGULAR {
        if let Some(m) = w.mobile_mut(id) {
            m.speech_hue = hue;
        }
    } else if ty == MessageType::EMOTE {
        if let Some(m) = w.mobile_mut(id) {
            m.emote_hue = hue;
        }
    } else if ty == MessageType::WHISPER {
        if let Some(m) = w.mobile_mut(id) {
            m.whisper_hue = hue;
        }
        range = 1;
    } else if ty == MessageType::YELL {
        if let Some(m) = w.mobile_mut(id) {
            m.yell_hue = hue;
        }
        range = 18;
    } else if [
        MessageType::SYSTEM,
        MessageType::LABEL,
        MessageType::FOCUS,
        MessageType::SPELL,
        MessageType::GUILD,
        MessageType::ALLIANCE,
        MessageType::COMMAND,
        MessageType::ENCODED,
    ]
    .contains(&ty)
    {
    } else {
        ty = MessageType::REGULAR;
    }

    let mut reg_args = SpeechEventArgs::new(id, text.to_string(), ty, hue, keywords.to_vec());

    crate::events::invoke_speech(w, &mut reg_args);
    p2::deps::region_on_speech(w, id, &mut reg_args);
    (mty.on_said)(w, id, &mut reg_args);

    if reg_args.blocked {
        return;
    }

    let text = reg_args.speech.clone();

    if text.is_empty() {
        return;
    }

    let Some(m) = w.mobile(id) else { return };
    let Some(map) = m.map else { return };
    let loc = m.location;

    let mut hears: Vec<MobileId> = Vec::new();
    let mut on_speech: Vec<EntityId> = Vec::new();

    let heard_list: Vec<MobileId> =
        crate::map::enumerate::mobiles_in_range(w, Some(map), loc.x, loc.y, range).collect();
    for heard in heard_list {
        let Some(hm) = w.mobile(heard) else { continue };
        let (hty, hplayer) = (hm.ty, hm.player);
        if !(hty.can_see_mobile)(w, heard, id)
            || (!p1::props_a::no_speech_los(w) && hplayer && !p1::combat::in_los(w, heard, id))
        {
            continue;
        }
        let Some(hm) = w.mobile(heard) else { continue };

        if hm.net_state.is_some() {
            hears.push(heard);
        }

        if (hty.handles_on_speech)(w, heard, id) || sh::mobile_has(w, heard, ScriptHook::Hear) {
            on_speech.push(EntityId::Mobile(heard));
        }

        let items = hm.items.clone();
        for item in items {
            let Some(it) = w.item(item) else { continue };
            if (it.ty.handles_on_speech)(w, item) {
                on_speech.push(EntityId::Item(item));
            }
            if crate::item::ops::is_container(w, item) {
                p2::speech::add_speech_items_from(w, &mut on_speech, item);
            }
        }
    }

    let item_list: Vec<ItemId> =
        crate::map::enumerate::items_in_range(w, Some(map), loc.x, loc.y, range).collect();
    for item in item_list {
        let Some(it) = w.item(item) else { continue };
        if (it.ty.handles_on_speech)(w, item) {
            on_speech.push(EntityId::Item(item));
        }
        if crate::item::ops::is_container(w, item) {
            p2::speech::add_speech_items_from(w, &mut on_speech, item);
        }
    }

    let mut mutate_context = SpeechContext::None;
    let mut mutated_text = text.clone();
    let mut mutated_args: Option<SpeechEventArgs> = None;

    if (mty.mutate_speech)(w, id, &mut hears, &mut mutated_text, &mut mutate_context) {
        mutated_args = Some(SpeechEventArgs::new(
            id,
            mutated_text.clone(),
            ty,
            hue,
            Vec::new(),
        ));
    }

    (mty.check_speech_manifest)(w, id);

    (mty.process_delta)(w, id);

    // TODO: Should this be sorted like onSpeech is below?
    for &heard in &hears {
        let Some(hm) = w.mobile(heard) else { continue };
        let hty = hm.ty;
        let ns = hm.net_state;
        let Some(sp) = w.mobile(id) else { return };
        let (serial, body, language, name) = (
            sp.serial,
            sp.cur_body().body_id(),
            sp.language.clone(),
            (sp.ty.name)(w, id).unwrap_or_default(),
        );
        let ghosted = mutated_args.is_some()
            && (mty.check_hears_mutated_speech)(w, id, heard, mutate_context);
        let shown: &str = if ghosted {
            mutated_text.as_str()
        } else {
            text.as_str()
        };
        let bytes = crate::packets::message::create_message(
            serial,
            body,
            crate::packets::message::MessageType::from_bits_retain(ty.bits() as u8),
            hue,
            3,
            false,
            language.as_deref(),
            &name,
            shown,
        );
        match mutated_args.as_mut() {
            Some(a) if ghosted => (hty.on_speech)(w, heard, a),
            _ => (hty.on_speech)(w, heard, &mut reg_args),
        }
        if let Some(ns) = ns {
            w.send(ns, &bytes);
        }
    }

    if on_speech.len() > 1 {
        // LocationComparer: squared distance with x and y scaled by 11; List.Sort (introsort): ties come out
        // in the introsort's fixed order.
        let rel = loc;
        let dist = |w: &World, e: &EntityId| -> i32 {
            let p = match e {
                EntityId::Mobile(m) => w.mobile(*m).map(|m| m.location),
                EntityId::Item(i) => w.item(*i).map(|i| i.location),
            }
            .unwrap_or_default();
            let (x, y, z) = ((rel.x - p.x) * 11, (rel.y - p.y) * 11, rel.z - p.z);
            x * x + y * y + z * z
        };
        let mut keyed: Vec<(i32, EntityId)> = on_speech.iter().map(|e| (dist(w, e), *e)).collect();
        muo_primitives::dotnet::net_sort(&mut keyed, |a, b| a.0.cmp(&b.0));
        on_speech = keyed.into_iter().map(|k| k.1).collect();
    }

    for obj in on_speech {
        match obj {
            EntityId::Mobile(heard) => {
                let Some(hty) = w.mobile(heard).map(|m| m.ty) else {
                    continue;
                };
                let use_mut = mutated_args.is_some()
                    && (mty.check_hears_mutated_speech)(w, id, heard, mutate_context);
                let args = match mutated_args.as_mut() {
                    Some(a) if use_mut => a,
                    _ => &mut reg_args,
                };
                let heard_by_script = [ScriptArg::Mobile(Some(id)), ScriptArg::Str(&args.speech)];
                if !sh::mobile(w, heard, ScriptHook::Hear, &heard_by_script) {
                    (hty.on_speech)(w, heard, args);
                }
            }
            EntityId::Item(item) => {
                if let Some(ity) = w.item(item).map(|i| i.ty) {
                    (ity.on_speech)(w, item, &mut reg_args);
                }
            }
        }
    }
}

/// `x.get_damage_master(..)`: the virtual call (an override runs; `get_damage_master_base` is what `base.get_damage_master(..)` reaches).
pub fn get_damage_master(w: &World, id: MobileId, a2: MobileId) -> Option<MobileId> {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.get_damage_master)(w, id, a2),
        None => get_damage_master_base(w, id, a2),
    }
}

pub fn get_damage_master_base(_w: &World, _id: MobileId, _damagee: MobileId) -> Option<MobileId> {
    None
}

pub fn register_damage(
    w: &mut World,
    id: MobileId,
    amount: i32,
    from: Option<MobileId>,
) -> Option<usize> {
    p2::damage::register_damage(w, id, amount, from)
}

/// `x.on_damage(..)`: the virtual call (an override runs; `on_damage_base` is what `base.on_damage(..)` reaches).
pub fn on_damage(w: &mut World, id: MobileId, a2: i32, a3: Option<MobileId>, a4: bool) {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.on_damage)(w, id, a2, a3, a4),
        None => on_damage_base(w, id, a2, a3, a4),
    }
}

pub fn on_damage_base(
    _w: &mut World,
    _id: MobileId,
    _amount: i32,
    _from: Option<MobileId>,
    _will_kill: bool,
) {
}

/// `x.can_be_damaged(..)`: the virtual call (an override runs; `can_be_damaged_base` is what `base.can_be_damaged(..)` reaches).
pub fn can_be_damaged(w: &World, id: MobileId) -> bool {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.can_be_damaged)(w, id),
        None => can_be_damaged_base(w, id),
    }
}

pub fn can_be_damaged_base(w: &World, id: MobileId) -> bool {
    w.mobile(id).is_some_and(|m| !m.blessed)
}

/// `x.damage(..)`: the virtual call (an override runs; `damage_base` is what `base.damage(..)` reaches).
pub fn damage(w: &mut World, id: MobileId, a2: i32, a3: Option<MobileId>, a4: bool, a5: bool) {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.damage)(w, id, a2, a3, a4, a5),
        None => damage_base(w, id, a2, a3, a4, a5),
    }
}

pub fn damage_base(
    w: &mut World,
    id: MobileId,
    amount: i32,
    from: Option<MobileId>,
    inform_mount: bool,
    ignore_evil_omen: bool,
) {
    p2::damage::damage(w, id, amount, from, inform_mount, ignore_evil_omen);
}

pub fn on_heal(_w: &mut World, _id: MobileId, _amount: &mut i32, _from: Option<MobileId>) {}

/// `x.deserialize(..)`: the virtual call (an override runs; `deserialize_base` is what `base.deserialize(..)` reaches).
pub fn deserialize(w: &mut World, id: MobileId, a2: &mut dyn GenericReader) -> muo_io::Result<()> {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.deserialize)(w, id, a2),
        None => deserialize_base(w, id, a2),
    }
}

pub fn deserialize_base(
    w: &mut World,
    id: MobileId,
    reader: &mut dyn GenericReader,
) -> muo_io::Result<()> {
    p2::serialization::deserialize(w, id, reader)
}

pub fn check_stat_timers(w: &mut World, id: MobileId) {
    use p2::stats::{Regen, start_regen_timer, stop_regen_timer};
    let Some(m) = w.mobile(id) else { return };
    if m.deleted {
        return;
    }
    let ty = m.ty;

    if w.mobile(id).map_or(0, |m| m.hits) < (ty.hits_max)(w, id) {
        if (ty.can_regen_hits)(w, id) {
            start_regen_timer(w, id, Regen::Hits);
        } else {
            stop_regen_timer(w, id, Regen::Hits);
        }
    } else {
        let v = (ty.hits_max)(w, id);
        props_b::set_hits(w, id, v);
    }

    if w.mobile(id).map_or(0, |m| m.stam) < (ty.stam_max)(w, id) {
        if (ty.can_regen_stam)(w, id) {
            start_regen_timer(w, id, Regen::Stam);
        } else {
            stop_regen_timer(w, id, Regen::Stam);
        }
    } else {
        let v = (ty.stam_max)(w, id);
        props_b::set_stam(w, id, v);
    }

    if w.mobile(id).map_or(0, |m| m.mana) < (ty.mana_max)(w, id) {
        if (ty.can_regen_mana)(w, id) {
            start_regen_timer(w, id, Regen::Mana);
        } else {
            stop_regen_timer(w, id, Regen::Mana);
        }
    } else {
        let v = (ty.mana_max)(w, id);
        props_b::set_mana(w, id, v);
    }
}

pub fn can_paperdoll_be_opened_by(w: &World, id: MobileId, _from: MobileId) -> bool {
    let Some(m) = w.mobile(id) else { return false };
    let body = props_b::body(w, id);
    let human = w
        .resources
        .get::<muo_primitives::BodyTable>()
        .is_some_and(|t| t.is_human(body));
    human || body.is_ghost() || m.body_mod.body_id() != 0
}

pub fn get_child_context_menu_entries(
    _w: &mut World,
    _id: MobileId,
    _from: MobileId,
    _list: &mut Vec<ContextMenuEntry>,
    _item: ItemId,
) {
}

/// `x.get_context_menu_entries(..)`: the virtual call (an override runs; `get_context_menu_entries_base` is what `base.get_context_menu_entries(..)` reaches).
pub fn get_context_menu_entries(
    w: &mut World,
    id: MobileId,
    a2: MobileId,
    a3: &mut Vec<ContextMenuEntry>,
) {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.get_context_menu_entries)(w, id, a2, a3),
        None => get_context_menu_entries_base(w, id, a2, a3),
    }
}

pub fn get_context_menu_entries_base(
    _w: &mut World,
    _id: MobileId,
    _from: MobileId,
    _list: &mut Vec<ContextMenuEntry>,
) {
}

/// `x.on_item_added(..)`: the virtual call (an override runs; `on_item_added_base` is what `base.on_item_added(..)` reaches).
pub fn on_item_added(w: &mut World, id: MobileId, a2: ItemId) {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.on_item_added)(w, id, a2),
        None => on_item_added_base(w, id, a2),
    }
}

pub fn on_item_added_base(_w: &mut World, _id: MobileId, _item: ItemId) {}

/// `x.on_item_removed(..)`: the virtual call (an override runs; `on_item_removed_base` is what `base.on_item_removed(..)` reaches).
pub fn on_item_removed(w: &mut World, id: MobileId, a2: ItemId) {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.on_item_removed)(w, id, a2),
        None => on_item_removed_base(w, id, a2),
    }
}

pub fn on_item_removed_base(_w: &mut World, _id: MobileId, _item: ItemId) {}

/// `x.on_sub_item_added(..)`: the virtual call (an override runs; `on_sub_item_added_base` is what `base.on_sub_item_added(..)` reaches).
pub fn on_sub_item_added(w: &mut World, id: MobileId, a2: ItemId) {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.on_sub_item_added)(w, id, a2),
        None => on_sub_item_added_base(w, id, a2),
    }
}

pub fn on_sub_item_added_base(_w: &mut World, _id: MobileId, _item: ItemId) {}

/// `x.on_sub_item_removed(..)`: the virtual call (an override runs; `on_sub_item_removed_base` is what `base.on_sub_item_removed(..)` reaches).
pub fn on_sub_item_removed(w: &mut World, id: MobileId, a2: ItemId) {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.on_sub_item_removed)(w, id, a2),
        None => on_sub_item_removed_base(w, id, a2),
    }
}

pub fn on_sub_item_removed_base(_w: &mut World, _id: MobileId, _item: ItemId) {}

pub fn on_item_bounce_cleared(_w: &mut World, _id: MobileId, _item: ItemId) {}

/// `x.on_sub_item_bounce_cleared(..)`: the virtual call (an override runs; `on_sub_item_bounce_cleared_base` is what `base.on_sub_item_bounce_cleared(..)` reaches).
pub fn on_sub_item_bounce_cleared(w: &mut World, id: MobileId, a2: ItemId) {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.on_sub_item_bounce_cleared)(w, id, a2),
        None => on_sub_item_bounce_cleared_base(w, id, a2),
    }
}

pub fn on_sub_item_bounce_cleared_base(_w: &mut World, _id: MobileId, _item: ItemId) {}

#[allow(clippy::too_many_arguments)]
pub fn animate(
    w: &mut World,
    id: MobileId,
    action: i32,
    frame_count: i32,
    repeat_count: i32,
    forward: bool,
    repeat: bool,
    delay: i32,
) {
    let Some(m) = w.mobile(id) else { return };
    let Some(map) = m.map else { return };
    let (ty, loc, serial) = (m.ty, m.location, m.serial);
    let (mut action, mut frame_count) = (action, frame_count);

    (ty.process_delta)(w, id);

    for (ns, viewer) in p2::deps::clients_in_range(w, map, loc) {
        if !p2::deps::can_see_mobile(w, viewer, id) {
            continue;
        }
        if let Some(vty) = w.mobile(viewer).map(|v| v.ty) {
            (vty.process_delta)(w, viewer);
        }

        let Some(m) = w.mobile(id) else { return };
        if m.cur_body().is_gargoyle() {
            frame_count = 10;
            action = if m.flying {
                match action {
                    9..=11 => 71,
                    12..=14 => 72,
                    20 => 77,
                    31 => 71,
                    34 => 78,
                    200..=259 => 75,
                    260..=270 => 75,
                    a => a,
                }
            } else {
                match action {
                    200..=259 => 17,
                    260..=270 => 16,
                    a => a,
                }
            };
        }

        crate::packets::mobile::send_mobile_animation(
            w,
            ns,
            serial,
            action,
            frame_count,
            repeat_count,
            forward,
            repeat,
            delay,
        );
    }
}

pub fn send_opl_packet_to(w: &mut World, id: MobileId, ns: NetStateId) {
    if w.core.opl_enabled {
        crate::mobile::part1::props_b::ensure_property_list(w, id); // the PropertyList getter builds it
        crate::packets::entity::send_opl_info_mobile(w, ns, id);
    }
}

/// `x.on_access_level_changed(..)`: the virtual call (an override runs; `on_access_level_changed_base` is what `base.on_access_level_changed(..)` reaches).
pub fn on_access_level_changed(w: &mut World, id: MobileId, a2: AccessLevel) {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.on_access_level_changed)(w, id, a2),
        None => on_access_level_changed_base(w, id, a2),
    }
}

pub fn on_access_level_changed_base(_w: &mut World, _id: MobileId, _old_level: AccessLevel) {}

pub fn on_fame_change(_w: &mut World, _id: MobileId, _old_value: i32) {}

pub fn on_karma_change(_w: &mut World, _id: MobileId, _old_value: i32) {}

/// `x.revealing_action(..)`: the virtual call (an override runs; `revealing_action_base` is what `base.revealing_action(..)` reaches).
pub fn revealing_action(w: &mut World, id: MobileId) {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.revealing_action)(w, id),
        None => revealing_action_base(w, id),
    }
}

pub fn revealing_action_base(w: &mut World, id: MobileId) {
    let Some(m) = w.mobile(id) else { return };
    let ty = m.ty;
    if m.hidden && m.access_level == AccessLevel::Player {
        props_b::set_hidden(w, id, false);
    }

    (ty.disruptive_action)(w, id); // Anything that unhides you will also distrupt meditation
}

pub fn on_said(w: &mut World, id: MobileId, e: &mut SpeechEventArgs) {
    if w.mobile(id).is_some_and(|m| m.squelched) {
        let ml = w.core_ml();
        if ml {
            // You can not say anything, you have been muted.
            p2::messages::send_localized_message(w, id, 500168, "", 0x3B2);
        } else {
            // Cliloc ITSELF changed during ML.
            p2::messages::send_message(w, id, "You can not say anything, you have been squelched.");
        }
        e.blocked = true;
    }

    if !e.blocked
        && let Some(ty) = w.mobile(id).map(|m| m.ty)
    {
        (ty.revealing_action)(w, id);
    }
}

/// `x.handles_on_speech(..)`: the virtual call (an override runs; `handles_on_speech_base` is what `base.handles_on_speech(..)` reaches).
pub fn handles_on_speech(w: &World, id: MobileId, a2: MobileId) -> bool {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.handles_on_speech)(w, id, a2),
        None => handles_on_speech_base(w, id, a2),
    }
}

pub fn handles_on_speech_base(_w: &World, _id: MobileId, _from: MobileId) -> bool {
    false
}

/// `x.on_speech(..)`: the virtual call (an override runs; `on_speech_base` is what `base.on_speech(..)` reaches).
pub fn on_speech(w: &mut World, id: MobileId, a2: &mut SpeechEventArgs) {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.on_speech)(w, id, a2),
        None => on_speech_base(w, id, a2),
    }
}

pub fn on_speech_base(_w: &mut World, _id: MobileId, _e: &mut SpeechEventArgs) {}

/// `x.on_map_change(..)`: the virtual call (an override runs; `on_map_change_base` is what `base.on_map_change(..)` reaches).
pub fn on_map_change(w: &mut World, id: MobileId, a2: Option<MapId>) {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.on_map_change)(w, id, a2),
        None => on_map_change_base(w, id, a2),
    }
}

pub fn on_map_change_base(_w: &mut World, _id: MobileId, _old_map: Option<MapId>) {}

pub fn get_season(w: &World, id: MobileId) -> i32 {
    w.mobile(id)
        .and_then(|m| m.map)
        .and_then(|map| w.maps.get(map))
        .map_or(1, |m| m.season)
}

pub fn get_packet_flags(w: &World, id: MobileId, stygian_abyss: bool) -> i32 {
    let Some(m) = w.mobile(id) else { return 0 };
    let mut flags = 0x0;

    let (paralyzed, frozen) = (m.paralyzed, m.frozen);
    if paralyzed
        || frozen
        || (p2::statics::read(w, |s| s.disable_cast_paralyze)
            && m.spell
                .is_some_and(|s| p2::deps::spell_blocks_movement(w, s)))
    {
        flags |= 0x01;
    }

    if m.female {
        flags |= 0x02;
    }

    if stygian_abyss {
        if m.flying {
            flags |= 0x04;
        }
    } else if m.poison.is_some() {
        flags |= 0x04;
    }

    if m.blessed || m.yellow_healthbar {
        flags |= 0x08;
    }

    if (m.ty.ignore_mobiles)(w, id) {
        flags |= 0x10;
    }

    if m.warmode {
        flags |= 0x40;
    }

    if m.hidden {
        flags |= 0x80;
    }

    flags
}

pub fn on_gender_changed(_w: &mut World, _id: MobileId, _old_female: bool) {}

/// `x.toggle_flying(..)`: the virtual call (an override runs; `toggle_flying_base` is what `base.toggle_flying(..)` reaches).
pub fn toggle_flying(w: &mut World, id: MobileId) {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.toggle_flying)(w, id),
        None => toggle_flying_base(w, id),
    }
}

pub fn toggle_flying_base(_w: &mut World, _id: MobileId) {}

pub fn on_warmode_changed(_w: &mut World, _id: MobileId) {}

/// `x.on_hidden_changed(..)`: the virtual call (an override runs; `on_hidden_changed_base` is what `base.on_hidden_changed(..)` reaches).
pub fn on_hidden_changed(w: &mut World, id: MobileId) {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.on_hidden_changed)(w, id),
        None => on_hidden_changed_base(w, id),
    }
}

pub fn on_hidden_changed_base(w: &mut World, id: MobileId) {
    if let Some(m) = w.mobile_mut(id) {
        m.allowed_stealth_steps = 0;
    }
    let Some(m) = w.mobile(id) else { return };
    let Some(map) = m.map else { return };
    let (loc, serial) = (m.location, m.serial);

    for (ns, viewer) in p2::deps::clients_in_range(w, map, loc) {
        if !p2::deps::can_see_mobile(w, viewer, id) {
            crate::packets::entity::send_remove_entity(w, ns, serial);
        } else {
            crate::packets::mobile::send_mobile_incoming(w, ns, viewer, id);

            let Some(ty) = w.mobile(id).map(|m| m.ty) else {
                return;
            };
            if (ty.is_dead_bonded_pet)(w, id) {
                crate::packets::mobile::send_bonded_status(w, ns, serial, true);
            }

            (ty.send_opl_packet_to)(w, id, ns);
        }
    }
}

pub fn on_connected(_w: &mut World, _id: MobileId) {}

pub fn on_before_disconnected(_w: &mut World, _id: MobileId) {}

pub fn on_disconnected(_w: &mut World, _id: MobileId) {}

/// `x.on_net_state_changed(..)`: the virtual call (an override runs; `on_net_state_changed_base` is what `base.on_net_state_changed(..)` reaches).
pub fn on_net_state_changed(w: &mut World, id: MobileId) {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.on_net_state_changed)(w, id),
        None => on_net_state_changed_base(w, id),
    }
}

pub fn on_net_state_changed_base(_w: &mut World, _id: MobileId) {}

pub fn can_see_object(w: &mut World, id: MobileId, o: Option<EntityId>) -> bool {
    let Some(ty) = w.mobile(id).map(|m| m.ty) else {
        return true;
    };
    match o {
        Some(EntityId::Item(i)) => (ty.can_see_item)(w, id, i),
        Some(EntityId::Mobile(m)) => (ty.can_see_mobile)(w, id, m),
        None => true,
    }
}

/// `x.can_see_item(..)`: the virtual call (an override runs; `can_see_item_base` is what `base.can_see_item(..)` reaches).
pub fn can_see_item(w: &mut World, id: MobileId, a2: ItemId) -> bool {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.can_see_item)(w, id, a2),
        None => can_see_item_base(w, id, a2),
    }
}

pub fn can_see_item_base(w: &mut World, id: MobileId, item: ItemId) -> bool {
    let (Some(m), Some(it)) = (w.mobile(id), w.item(item)) else {
        return false;
    };
    let (ty, map, access) = (m.ty, m.map, m.access_level);
    let parent = it.parent;
    if map == Some(MapId::INTERNAL) {
        return false;
    }

    if it.map == Some(MapId::INTERNAL) {
        return false;
    }

    match parent {
        Some(EntityId::Item(parent)) => {
            let ok = (ty.can_see_item)(w, id, parent)
                && w.item(parent)
                    .is_some_and(|p| (p.ty.is_child_visible_to)(w, parent, id, item));
            if !ok {
                return false;
            }
        }
        Some(EntityId::Mobile(pm)) if !(ty.can_see_mobile)(w, id, pm) => return false,
        Some(EntityId::Mobile(_)) | None => {}
    }

    if p2::deps::item_is_bank_box(w, item)
        && access <= AccessLevel::Counselor
        && (p2::deps::bank_box_owner(w, item) != Some(id) || !p2::deps::bank_box_opened(w, item))
    {
        return false;
    }

    if p2::deps::trade_hides_container(w, item, id) {
        return false;
    }

    let Some(it) = w.item(item) else { return false };
    !it.deleted() && it.map == map && (it.visible() || (it.ty.can_see_staff_only)(w, item, id))
}

/// `x.can_see_mobile(..)`: the virtual call (an override runs; `can_see_mobile_base` is what `base.can_see_mobile(..)` reaches).
pub fn can_see_mobile(w: &mut World, id: MobileId, a2: MobileId) -> bool {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.can_see_mobile)(w, id, a2),
        None => can_see_mobile_base(w, id, a2),
    }
}

pub fn can_see_mobile_base(w: &mut World, id: MobileId, m: MobileId) -> bool {
    let (Some(me), Some(o)) = (w.mobile(id), w.mobile(m)) else {
        return false;
    };
    if me.deleted || o.deleted || me.map == Some(MapId::INTERNAL) || o.map == Some(MapId::INTERNAL)
    {
        return false;
    }

    let se = w.core_se();
    let spirit_speak = se && p2::deps::spirit_speak_value(w, id) >= 100.0;

    id == m
        || o.map == me.map
            && (!o.hidden
                || me.access_level != AccessLevel::Player
                    && (me.access_level >= o.access_level
                        || me.access_level >= AccessLevel::Administrator))
            && ((o.ty.alive)(w, m)
                || spirit_speak
                || !(me.ty.alive)(w, id)
                || me.access_level > AccessLevel::Player
                || o.warmode)
}

/// `x.can_be_renamed_by(..)`: the virtual call (an override runs; `can_be_renamed_by_base` is what `base.can_be_renamed_by(..)` reaches).
pub fn can_be_renamed_by(w: &World, id: MobileId, a2: MobileId) -> bool {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.can_be_renamed_by)(w, id, a2),
        None => can_be_renamed_by_base(w, id, a2),
    }
}

pub fn can_be_renamed_by_base(w: &World, id: MobileId, from: MobileId) -> bool {
    match (w.mobile(id), w.mobile(from)) {
        (Some(m), Some(f)) => {
            f.access_level >= AccessLevel::GameMaster && f.access_level > m.access_level
        }
        _ => false,
    }
}

pub fn on_guild_title_change(_w: &mut World, _id: MobileId, _old_title: Option<String>) {}

pub fn on_after_name_change(
    _w: &mut World,
    _id: MobileId,
    _old_name: Option<String>,
    _new_name: Option<String>,
) {
}

pub fn on_guild_change(_w: &mut World, _id: MobileId, _old_guild: Option<GuildId>) {}

pub fn safe_body(_w: &World, _id: MobileId, body: i32) -> i32 {
    let mut delta = -1;
    let mut i = 0;
    while delta < 0 && i < p2::statics::INVALID_BODIES.len() {
        delta = p2::statics::INVALID_BODIES[i] - body;
        i += 1;
    }
    if delta != 0 { body } else { 0 }
}

/// `x.set_location(..)`: the virtual call (an override runs; `set_location_base` is what `base.set_location(..)` reaches).
pub fn set_location(w: &mut World, id: MobileId, a2: Point3D, a3: bool) {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.set_location)(w, id, a2, a3),
        None => set_location_base(w, id, a2, a3),
    }
}

pub fn set_location_base(w: &mut World, id: MobileId, new_location: Point3D, is_teleport: bool) {
    let Some(m) = w.mobile(id) else { return };
    if m.deleted {
        return;
    }
    let old_location = m.location;
    if old_location == new_location {
        return;
    }
    let ty = m.ty;

    if let Some(m) = w.mobile_mut(id) {
        m.location = new_location;
    }
    if let Some(map) = w.mobile(id).and_then(|m| m.map) {
        crate::map::sector::on_move_mobile(w, map, old_location, id);
    }
    p2::effects::update_region(w, id);

    p2::items::close_bank_box(w, id);

    let own = w.mobile(id).and_then(|m| m.net_state);
    let no_move_hs = w.mobile(id).is_some_and(|m| m.no_move_hs);
    let high_seas = |w: &World, ns| w.net_state(ns).is_some_and(|d| d.caps.high_seas());
    let stygian = |w: &World, ns| w.net_state(ns).is_some_and(|d| d.caps.stygian_abyss());

    if let Some(ns) = own {
        p2::deps::validate_all_trades(w, ns);

        if is_teleport && (!high_seas(w, ns) || !no_move_hs) {
            crate::net_glue::reset_movement_state(w, ns);
            crate::packets::mobile::send_mobile_update(w, ns, id);
        }
    }

    let Some(map) = w.mobile(id).and_then(|m| m.map) else {
        (ty.on_location_change)(w, id, old_location);
        p2::deps::region_on_location_changed(w, id, old_location);
        return;
    };
    let serial = w
        .mobile(id)
        .map_or(muo_primitives::Serial::ZERO, |m| m.serial);
    let range = w.maps.global_max_update_range;

    // First, send a remove message to everyone who can no longer see us. (inOldRange && !inNewRange)
    for (ns, viewer) in p2::deps::clients_in_range(w, map, old_location) {
        let vloc = w.mobile(viewer).map_or(Point3D::default(), |v| v.location);
        if Some(ns) != own && !p1::combat::in_update_range(new_location, vloc) {
            crate::packets::entity::send_remove_entity(w, ns, serial);
        }
    }

    if let Some(our_state) = own {
        // We are attached to a client, so it's a bit more complex. We need to send new items and people to
        // ourself, and ourself to other clients.
        let mobiles: Vec<MobileId> = crate::map::enumerate::mobiles_in_range(
            w,
            Some(map),
            new_location.x,
            new_location.y,
            range,
        )
        .collect();
        for o in mobiles {
            if o == id {
                continue;
            }
            let Some(om) = w.mobile(o) else { continue };
            let oloc = om.location;
            if !p1::combat::in_update_range(new_location, oloc) {
                continue;
            }

            let in_old_range = p1::combat::in_update_range(old_location, oloc);
            let ns = om.net_state;
            let oty = om.ty;

            if let Some(ns) = ns
                && ((is_teleport && (!high_seas(w, ns) || !no_move_hs)) || !in_old_range)
                && (oty.can_see_mobile)(w, o, id)
            {
                p2::effects::send_incoming_to(w, id, ns, o);
            }

            if in_old_range || !(ty.can_see_mobile)(w, id, o) {
                continue;
            }

            crate::packets::mobile::send_mobile_incoming(w, our_state, id, o);
            if stygian(w, our_state) {
                crate::packets::mobile::send_mobile_healthbar(
                    w,
                    our_state,
                    o,
                    crate::mobile::Healthbar::Poison,
                );
                crate::packets::mobile::send_mobile_healthbar(
                    w,
                    our_state,
                    o,
                    crate::mobile::Healthbar::Yellow,
                );
            }
            if (oty.is_dead_bonded_pet)(w, o) {
                crate::packets::mobile::send_bonded_status(w, our_state, o.0, true);
            }
            (oty.send_opl_packet_to)(w, o, our_state);
        }

        let items: Vec<ItemId> = crate::map::enumerate::items_in_range(
            w,
            Some(map),
            new_location.x,
            new_location.y,
            range,
        )
        .collect();
        for item in items {
            let Some(it) = w.item(item) else { continue };
            let (ity, iloc) = (it.ty, it.location);
            let r = (ity.get_update_range)(w, item, id);
            if !utility::in_range_3d(old_location, iloc, r)
                && utility::in_range_3d(new_location, iloc, r)
                && (ty.can_see_item)(w, id, item)
            {
                (ity.send_info_to)(w, item, our_state, &[]);
            }
        }
    } else {
        // We're not attached to a client, so simply send an Incoming
        for (ns, viewer) in p2::deps::clients_in_range(w, map, new_location) {
            let vloc = w.mobile(viewer).map_or(Point3D::default(), |v| v.location);
            if ((is_teleport && (!high_seas(w, ns) || !no_move_hs))
                || !p1::combat::in_update_range(old_location, vloc))
                && p2::deps::can_see_mobile(w, viewer, id)
            {
                p2::effects::send_incoming_to(w, id, ns, viewer);
            }
        }
    }

    (ty.on_location_change)(w, id, old_location);

    p2::deps::region_on_location_changed(w, id, old_location);
}

/// `x.on_location_change(..)`: the virtual call (an override runs; `on_location_change_base` is what `base.on_location_change(..)` reaches).
pub fn on_location_change(w: &mut World, id: MobileId, a2: Point3D) {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.on_location_change)(w, id, a2),
        None => on_location_change_base(w, id, a2),
    }
}

pub fn on_location_change_base(_w: &mut World, _id: MobileId, _old_location: Point3D) {}

pub fn get_default_weapon(_w: &World, _id: MobileId) -> WeaponRef {
    WeaponRef::Default
}

pub fn check_lift(
    _w: &mut World,
    _id: MobileId,
    _from: MobileId,
    _item: ItemId,
    _reject: &mut LRReason,
) -> bool {
    true
}

/// `x.check_nonlocal_lift(..)`: the virtual call (an override runs; `check_nonlocal_lift_base` is what `base.check_nonlocal_lift(..)` reaches).
pub fn check_nonlocal_lift(w: &mut World, id: MobileId, a2: MobileId, a3: ItemId) -> bool {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.check_nonlocal_lift)(w, id, a2, a3),
        None => check_nonlocal_lift_base(w, id, a2, a3),
    }
}

pub fn check_nonlocal_lift_base(
    w: &mut World,
    id: MobileId,
    from: MobileId,
    _item: ItemId,
) -> bool {
    let (Some(m), Some(f)) = (w.mobile(id), w.mobile(from)) else {
        return false;
    };
    from == id || (f.access_level > m.access_level && f.access_level >= AccessLevel::GameMaster)
}

/// `x.check_trade(..)`: the virtual call (an override runs; `check_trade_base` is what `base.check_trade(..)` reaches).
#[allow(clippy::too_many_arguments)]
pub fn check_trade(
    w: &mut World,
    id: MobileId,
    a2: MobileId,
    a3: ItemId,
    a4: ItemId,
    a5: bool,
    a6: bool,
    a7: i32,
    a8: i32,
) -> bool {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.check_trade)(w, id, a2, a3, a4, a5, a6, a7, a8),
        None => check_trade_base(w, id, a2, a3, a4, a5, a6, a7, a8),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn check_trade_base(
    _w: &mut World,
    _id: MobileId,
    _to: MobileId,
    _item: ItemId,
    _cont: ItemId,
    _message: bool,
    _check_items: bool,
    _plus_items: i32,
    _plus_weight: i32,
) -> bool {
    true
}

pub fn open_trade(w: &mut World, id: MobileId, from: MobileId, offer: Option<ItemId>) -> bool {
    if !p2::deps::player_trading_enabled(w) {
        p2::messages::send_message_hue(w, from, 0x22, "Player trading is temporarily disabled.");
        return false;
    }

    let (Some(me), Some(f)) = (w.mobile(id), w.mobile(from)) else {
        return false;
    };
    let (ty, fty) = (me.ty, f.ty);
    if !f.player || !me.player || !(fty.alive)(w, from) || !(ty.alive)(w, id) {
        return false;
    }

    let (our_state, their_state) = (me.net_state, f.net_state);
    let (Some(ours), Some(theirs)) = (our_state, their_state) else {
        return false;
    };

    let cont = p2::deps::find_trade_container(w, theirs, id);

    if !(fty.check_trade)(
        w,
        from,
        id,
        offer.unwrap_or(ItemId(muo_primitives::Serial::ZERO)),
        cont.unwrap_or(ItemId(muo_primitives::Serial::ZERO)),
        true,
        true,
        0,
        0,
    ) {
        return false;
    }

    let cont = match cont {
        Some(c) => Some(c),
        None => p2::deps::add_trade(w, theirs, ours),
    };

    if let (Some(offer), Some(cont)) = (offer, cont)
        && let Some(cty) = w.item(cont).map(|i| i.ty)
    {
        (cty.drop_item)(w, cont, offer);
    }

    true
}

/// `x.on_drag_drop(..)`: the virtual call (an override runs; `on_drag_drop_base` is what `base.on_drag_drop(..)` reaches).
pub fn on_drag_drop(w: &mut World, id: MobileId, a2: MobileId, a3: ItemId) -> bool {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.on_drag_drop)(w, id, a2, a3),
        None => on_drag_drop_base(w, id, a2, a3),
    }
}

pub fn on_drag_drop_base(w: &mut World, id: MobileId, from: MobileId, dropped: ItemId) -> bool {
    if from == id {
        let pack = props_b::backpack(w, id);
        let Some(pack) = pack else { return false };
        let Some(dty) = w.item(dropped).map(|i| i.ty) else {
            return false;
        };
        return (dty.drop_to_item)(w, dropped, from, pack, Point3D::new(-1, -1, 0));
    }

    let (Some(m), Some(f)) = (w.mobile(id), w.mobile(from)) else {
        return false;
    };
    let ty = m.ty;
    utility::in_range_3d(f.location, m.location, 2) && (ty.open_trade)(w, id, from, Some(dropped))
}

/// `x.check_equip(..)`: the virtual call (an override runs; `check_equip_base` is what `base.check_equip(..)` reaches).
pub fn check_equip(w: &mut World, id: MobileId, a2: ItemId) -> bool {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.check_equip)(w, id, a2),
        None => check_equip_base(w, id, a2),
    }
}

pub fn check_equip_base(w: &mut World, id: MobileId, item: ItemId) -> bool {
    let items: Vec<ItemId> = w.mobile(id).map(|m| m.items.clone()).unwrap_or_default();
    let Some(ity) = w.item(item).map(|i| i.ty) else {
        return true;
    };
    let ilayer = w
        .item(item)
        .map_or(crate::layer::Layer::Invalid, |i| i.layer);
    for e in items {
        let Some(ety) = w.item(e).map(|i| i.ty) else {
            continue;
        };
        let elayer = w.item(e).map_or(crate::layer::Layer::Invalid, |i| i.layer);
        if (ety.check_conflicting_layer)(w, e, id, item, ilayer)
            || (ity.check_conflicting_layer)(w, item, id, e, elayer)
        {
            return false;
        }
    }
    true
}

/// `x.on_equip(..)`: the virtual call (an override runs; `on_equip_base` is what `base.on_equip(..)` reaches).
pub fn on_equip(w: &mut World, id: MobileId, a2: ItemId) -> bool {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.on_equip)(w, id, a2),
        None => on_equip_base(w, id, a2),
    }
}

pub fn on_equip_base(w: &mut World, id: MobileId, item: ItemId) -> bool {
    // For some reason OSI allows equipping quest items, but they are unmarked in the process
    if w.item(item).is_some_and(|i| i.quest_item()) {
        crate::item::ops::set_quest_item(w, item, false);

        // An item must be in your backpack (and not in a container within) to be toggled as a quest item.
        p2::messages::send_localized_message(w, id, 1074769, "", 0x3B2);
    }
    true
}

/// `x.on_drag_lift(..)`: the virtual call (an override runs; `on_drag_lift_base` is what `base.on_drag_lift(..)` reaches).
pub fn on_drag_lift(w: &mut World, id: MobileId, a2: ItemId) -> bool {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.on_drag_lift)(w, id, a2),
        None => on_drag_lift_base(w, id, a2),
    }
}

pub fn on_drag_lift_base(_w: &mut World, _id: MobileId, _item: ItemId) -> bool {
    true
}

pub fn on_dropped_item_into(
    _w: &mut World,
    _id: MobileId,
    _item: ItemId,
    _container: ItemId,
    _loc: Point3D,
) -> bool {
    true
}

pub fn on_dropped_item_onto(_w: &mut World, _id: MobileId, _item: ItemId, _target: ItemId) -> bool {
    true
}

pub fn on_dropped_item_to_item(
    _w: &mut World,
    _id: MobileId,
    _item: ItemId,
    _target: ItemId,
    _loc: Point3D,
) -> bool {
    true
}

pub fn on_dropped_item_to_mobile(
    _w: &mut World,
    _id: MobileId,
    _item: ItemId,
    _target: MobileId,
) -> bool {
    true
}

/// `x.on_dropped_item_to_world(..)`: the virtual call (an override runs; `on_dropped_item_to_world_base` is what `base.on_dropped_item_to_world(..)` reaches).
pub fn on_dropped_item_to_world(w: &mut World, id: MobileId, a2: ItemId, a3: Point3D) -> bool {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.on_dropped_item_to_world)(w, id, a2, a3),
        None => on_dropped_item_to_world_base(w, id, a2, a3),
    }
}

pub fn on_dropped_item_to_world_base(
    _w: &mut World,
    _id: MobileId,
    _item: ItemId,
    _location: Point3D,
) -> bool {
    true
}

pub fn on_item_used(_w: &mut World, _id: MobileId, _from: MobileId, _item: ItemId) {}

/// `x.check_nonlocal_drop(..)`: the virtual call (an override runs; `check_nonlocal_drop_base` is what `base.check_nonlocal_drop(..)` reaches).
pub fn check_nonlocal_drop(
    w: &mut World,
    id: MobileId,
    a2: MobileId,
    a3: ItemId,
    a4: ItemId,
) -> bool {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.check_nonlocal_drop)(w, id, a2, a3, a4),
        None => check_nonlocal_drop_base(w, id, a2, a3, a4),
    }
}

pub fn check_nonlocal_drop_base(
    w: &mut World,
    id: MobileId,
    from: MobileId,
    _item: ItemId,
    _target: ItemId,
) -> bool {
    let (Some(m), Some(f)) = (w.mobile(id), w.mobile(from)) else {
        return false;
    };
    from == id || (f.access_level > m.access_level && f.access_level >= AccessLevel::GameMaster)
}

/// `x.check_item_use(..)`: the virtual call (an override runs; `check_item_use_base` is what `base.check_item_use(..)` reaches).
pub fn check_item_use(w: &mut World, id: MobileId, a2: MobileId, a3: ItemId) -> bool {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.check_item_use)(w, id, a2, a3),
        None => check_item_use_base(w, id, a2, a3),
    }
}

pub fn check_item_use_base(_w: &mut World, _id: MobileId, _from: MobileId, _item: ItemId) -> bool {
    true
}

pub fn on_item_lifted(_w: &mut World, _id: MobileId, _from: MobileId, _item: ItemId) {}

/// `x.allow_item_use(..)`: the virtual call (an override runs; `allow_item_use_base` is what `base.allow_item_use(..)` reaches).
pub fn allow_item_use(w: &mut World, id: MobileId, a2: ItemId) -> bool {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.allow_item_use)(w, id, a2),
        None => allow_item_use_base(w, id, a2),
    }
}

pub fn allow_item_use_base(_w: &mut World, _id: MobileId, _item: ItemId) -> bool {
    true
}

/// `x.allow_equip_from(..)`: the virtual call (an override runs; `allow_equip_from_base` is what `base.allow_equip_from(..)` reaches).
pub fn allow_equip_from(w: &World, id: MobileId, a2: MobileId) -> bool {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.allow_equip_from)(w, id, a2),
        None => allow_equip_from_base(w, id, a2),
    }
}

pub fn allow_equip_from_base(w: &World, id: MobileId, mob: MobileId) -> bool {
    let (Some(m), Some(o)) = (w.mobile(id), w.mobile(mob)) else {
        return false;
    };
    mob == id || (o.access_level >= AccessLevel::GameMaster && o.access_level > m.access_level)
}

pub fn equip_item(w: &mut World, id: MobileId, item: ItemId) -> bool {
    let Some(it) = w.item(item) else { return false };
    let ity = it.ty;
    if it.deleted() || !(ity.can_equip)(w, item, id) {
        return false;
    }
    let Some(ty) = w.mobile(id).map(|m| m.ty) else {
        return false;
    };

    if (ty.check_equip)(w, id, item) && (ty.on_equip)(w, id, item) && (ity.on_equip)(w, item, id) {
        if !p2::deps::spell_on_caster_equipping(w, id, item) {
            return false;
        }

        p2::items::add_item(w, id, item);
        return true;
    }

    false
}

/// `x.delta(..)`: the virtual call (an override runs; `delta_base` is what `base.delta(..)` reaches).
pub fn delta(w: &mut World, id: MobileId, a2: MobileDelta) {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.delta)(w, id, a2),
        None => delta_base(w, id, a2),
    }
}

pub fn delta_base(w: &mut World, id: MobileId, flag: MobileDelta) {
    let Some(m) = w.mobile_mut(id) else { return };
    if m.map.is_none() || m.map == Some(MapId::INTERNAL) || m.deleted {
        return;
    }

    m.delta_flags |= flag;

    if !m.in_delta_queue {
        m.in_delta_queue = true;
        w.resources
            .get_mut::<p2::statics::Part2Statics>()
            .delta_queue
            .push_back(id);
    }
}

/// `x.on_kills_change(..)`: the virtual call (an override runs; `on_kills_change_base` is what `base.on_kills_change(..)` reaches).
pub fn on_kills_change(w: &mut World, id: MobileId, a2: i32) {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.on_kills_change)(w, id, a2),
        None => on_kills_change_base(w, id, a2),
    }
}

pub fn on_kills_change_base(_w: &mut World, _id: MobileId, _old_value: i32) {}

/// `x.display_paperdoll_to(..)`: the virtual call (an override runs; `display_paperdoll_to_base` is what `base.display_paperdoll_to(..)` reaches).
pub fn display_paperdoll_to(w: &mut World, id: MobileId, a2: MobileId) {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.display_paperdoll_to)(w, id, a2),
        None => display_paperdoll_to_base(w, id, a2),
    }
}

pub fn display_paperdoll_to_base(w: &mut World, id: MobileId, to: MobileId) {
    let mut args = (to, id);
    crate::events::invoke_paperdoll_request(w, &mut args);
}

pub fn on_paperdoll_request(w: &mut World, id: MobileId) {
    let Some(ty) = w.mobile(id).map(|m| m.ty) else {
        return;
    };
    if (ty.can_paperdoll_be_opened_by)(w, id, id) {
        (ty.display_paperdoll_to)(w, id, id);
    }
}

pub fn on_stats_query(w: &mut World, id: MobileId, from: MobileId) {
    let (Some(m), Some(f)) = (w.mobile(id), w.mobile(from)) else {
        return;
    };
    let (loc, map, own, party) = (m.location, m.map, m.net_state, m.party);
    let (fty, floc, fmap, fns) = (f.ty, f.location, f.map, f.net_state);

    if fmap == map
        && p1::combat::in_update_range(loc, floc)
        && (fty.can_see_mobile)(w, from, id)
        && let Some(ns) = fns
    {
        crate::packets::mobile::send_mobile_status(w, ns, from, id);
    }

    if from == id
        && let Some(ns) = own
    {
        crate::packets::player::send_stat_lock_info(w, ns, id);
    }

    if let Some(p) = party {
        p2::deps::party_on_stats_query(w, p, from, id);
    }
}

pub fn on_skills_query(w: &mut World, id: MobileId, from: MobileId) {
    if from == id
        && let Some(ns) = w.mobile(id).and_then(|m| m.net_state)
    {
        p2::deps::send_skills_update(w, ns, id);
    }
}

/// `x.on_region_change(..)`: the virtual call (an override runs; `on_region_change_base` is what `base.on_region_change(..)` reaches).
pub fn on_region_change(w: &mut World, id: MobileId, a2: RegionId, a3: RegionId) {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.on_region_change)(w, id, a2, a3),
        None => on_region_change_base(w, id, a2, a3),
    }
}

pub fn on_region_change_base(_w: &mut World, _id: MobileId, _old: RegionId, _new: RegionId) {}

/// `x.on_single_click(..)`: the virtual call (an override runs; `on_single_click_base` is what `base.on_single_click(..)` reaches).
pub fn on_single_click(w: &mut World, id: MobileId, a2: MobileId) {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.on_single_click)(w, id, a2),
        None => on_single_click_base(w, id, a2),
    }
}

pub fn on_single_click_base(w: &mut World, id: MobileId, from: MobileId) {
    let (Some(m), Some(f)) = (w.mobile(id), w.mobile(from)) else {
        return;
    };
    let ty = m.ty;
    if m.deleted
        || (m.access_level == AccessLevel::Player
            && p2::statics::read(w, |s| s.disable_hidden_self_click)
            && m.hidden
            && from == id)
    {
        return;
    }
    let from_ns = f.net_state;

    if p2::statics::read(w, |s| s.guild_click_message)
        && let Some(g) = m.guild.and_then(|g| crate::guild::guild(w, g))
        && (m.display_guild_title || m.player && g.ty != crate::guild::GuildType::Regular)
    {
        let title = m.guild_title.as_deref().map_or("", str::trim);
        let kind = p2::statics::GUILD_TYPES
            .get(g.ty as usize)
            .copied()
            .unwrap_or("");
        let text = if title.is_empty() {
            format!("[{}]{kind}", g.abbreviation)
        } else {
            format!("[{title}, {}]{kind}", g.abbreviation)
        };
        let speech_hue = m.speech_hue;
        p2::messages::private_overhead_message(
            w,
            id,
            MessageType::REGULAR,
            speech_hue,
            true,
            &text,
            from_ns,
        );
    }
    let Some(m) = w.mobile(id) else { return };

    let (name_hue, access_level) = (m.name_hue, m.access_level);
    let hue = if name_hue != -1 {
        name_hue
    } else if access_level > AccessLevel::Player {
        11
    } else {
        let noto = crate::notoriety::compute(w, from, id);
        crate::notoriety::get_hue(w, noto)
    };
    let Some(m) = w.mobile(id) else { return };

    let name = (ty.name)(w, id).unwrap_or_default();

    let mut prefix = "";
    let human = w
        .resources
        .get::<muo_primitives::BodyTable>()
        .is_some_and(|t| t.is_human(m.body));
    if (ty.show_fame_title)(w, id) && (m.player || human) && m.fame >= 10000 {
        prefix = if m.female { "Lady" } else { "Lord" };
    }

    let title = if (ty.click_title)(w, id) && m.title.as_deref().is_some_and(|t| !t.is_empty()) {
        m.title.clone().unwrap_or_default()
    } else {
        String::new()
    };
    let suffix = (ty.apply_name_suffix)(w, id, &title);

    let val = if !prefix.is_empty() && !suffix.is_empty() {
        format!("{prefix} {name} {suffix}")
    } else if !prefix.is_empty() {
        format!("{prefix} {name}")
    } else if !suffix.is_empty() {
        format!("{name} {suffix}")
    } else {
        name
    };

    let ascii = p2::statics::read(w, |s| s.ascii_click_message);
    p2::messages::private_overhead_message(w, id, MessageType::LABEL, hue, ascii, &val, from_ns);
}

/// `x.disruptive_action(..)`: the virtual call (an override runs; `disruptive_action_base` is what `base.disruptive_action(..)` reaches).
pub fn disruptive_action(w: &mut World, id: MobileId) {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.disruptive_action)(w, id),
        None => disruptive_action_base(w, id),
    }
}

pub fn disruptive_action_base(w: &mut World, id: MobileId) {
    if w.mobile(id).is_some_and(|m| m.meditating) {
        if let Some(m) = w.mobile_mut(id) {
            m.meditating = false;
        }
        p2::messages::send_localized_message(w, id, 500134, "", 0x3B2); // You stop meditating.
    }
}

/// `x.on_sector_activate(..)`: the virtual call (an override runs; `on_sector_activate_base` is what `base.on_sector_activate(..)` reaches).
pub fn on_sector_activate(w: &mut World, id: MobileId) {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.on_sector_activate)(w, id),
        None => on_sector_activate_base(w, id),
    }
}

pub fn on_sector_activate_base(_w: &mut World, _id: MobileId) {}

/// `x.on_sector_deactivate(..)`: the virtual call (an override runs; `on_sector_deactivate_base` is what `base.on_sector_deactivate(..)` reaches).
pub fn on_sector_deactivate(w: &mut World, id: MobileId) {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.on_sector_deactivate)(w, id),
        None => on_sector_deactivate_base(w, id),
    }
}

pub fn on_sector_deactivate_base(_w: &mut World, _id: MobileId) {}

/// `x.get_anger_sound(..)`: the virtual call (an override runs; `get_anger_sound_base` is what `base.get_anger_sound(..)` reaches).
pub fn get_anger_sound(w: &mut World, id: MobileId) -> i32 {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.get_anger_sound)(w, id),
        None => get_anger_sound_base(w, id),
    }
}

pub fn get_anger_sound_base(w: &mut World, id: MobileId) -> i32 {
    match w.mobile(id).map_or(0, |m| m.base_sound_id) {
        0 => -1,
        b => b,
    }
}

/// `x.get_idle_sound(..)`: the virtual call (an override runs; `get_idle_sound_base` is what `base.get_idle_sound(..)` reaches).
pub fn get_idle_sound(w: &mut World, id: MobileId) -> i32 {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.get_idle_sound)(w, id),
        None => get_idle_sound_base(w, id),
    }
}

pub fn get_idle_sound_base(w: &mut World, id: MobileId) -> i32 {
    match w.mobile(id).map_or(0, |m| m.base_sound_id) {
        0 => -1,
        b => b + 1,
    }
}

pub fn get_attack_sound(w: &mut World, id: MobileId) -> i32 {
    match w.mobile(id).map_or(0, |m| m.base_sound_id) {
        0 => -1,
        b => b + 2,
    }
}

/// `x.get_hurt_sound(..)`: the virtual call (an override runs; `get_hurt_sound_base` is what `base.get_hurt_sound(..)` reaches).
pub fn get_hurt_sound(w: &mut World, id: MobileId) -> i32 {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.get_hurt_sound)(w, id),
        None => get_hurt_sound_base(w, id),
    }
}

pub fn get_hurt_sound_base(w: &mut World, id: MobileId) -> i32 {
    match w.mobile(id).map_or(0, |m| m.base_sound_id) {
        0 => -1,
        b => b + 3,
    }
}

/// `x.get_death_sound(..)`: the virtual call (an override runs; `get_death_sound_base` is what `base.get_death_sound(..)` reaches).
pub fn get_death_sound(w: &mut World, id: MobileId) -> i32 {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.get_death_sound)(w, id),
        None => get_death_sound_base(w, id),
    }
}

pub fn get_death_sound_base(w: &mut World, id: MobileId) -> i32 {
    let Some(m) = w.mobile(id) else { return -1 };
    if m.base_sound_id != 0 {
        return m.base_sound_id + 4;
    }
    let (female, body) = (m.female, m.body);
    let human = w
        .resources
        .get::<muo_primitives::BodyTable>()
        .is_some_and(|t| t.is_human(body));
    if human {
        return utility::random_from(
            &mut w.rng,
            if female { 0x314 } else { 0x423 },
            if female { 4 } else { 5 },
        );
    }
    -1
}

pub fn can_be_beneficial(w: &mut World, id: MobileId, target: MobileId) -> bool {
    if let Some(m) = w.mobile(id) {
        (m.ty.can_be_beneficial_full)(w, id, target, true, false)
    } else {
        false
    }
}

pub fn can_be_beneficial_msg(w: &mut World, id: MobileId, target: MobileId, message: bool) -> bool {
    if let Some(m) = w.mobile(id) {
        (m.ty.can_be_beneficial_full)(w, id, target, message, false)
    } else {
        false
    }
}

/// `x.can_be_beneficial_full(..)`: the virtual call (an override runs; `can_be_beneficial_full_base` is what `base.can_be_beneficial_full(..)` reaches).
pub fn can_be_beneficial_full(
    w: &mut World,
    id: MobileId,
    a2: MobileId,
    a3: bool,
    a4: bool,
) -> bool {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.can_be_beneficial_full)(w, id, a2, a3, a4),
        None => can_be_beneficial_full_base(w, id, a2, a3, a4),
    }
}

pub fn can_be_beneficial_full_base(
    w: &mut World,
    id: MobileId,
    target: MobileId,
    message: bool,
    allow_dead: bool,
) -> bool {
    let (Some(m), Some(t)) = (w.mobile(id), w.mobile(target)) else {
        return false;
    };
    let (ty, tty) = (m.ty, t.ty);
    if m.deleted
        || t.deleted
        || !(ty.alive)(w, id)
        || (ty.is_dead_bonded_pet)(w, id)
        || (!allow_dead && (!(tty.alive)(w, target) || (tty.is_dead_bonded_pet)(w, target)))
    {
        if message {
            // You can not perform beneficial acts on your target.
            p2::messages::send_localized_message(w, id, 1001017, "", 0x3B2);
        }
        return false;
    }

    if target == id {
        return true;
    }

    if !p2::deps::region_allow_beneficial(w, id, target) {
        // TODO: Pets
        if message {
            p2::messages::send_localized_message(w, id, 1001017, "", 0x3B2);
        }
        return false;
    }

    true
}

pub fn is_beneficial_criminal(w: &mut World, id: MobileId, target: MobileId) -> bool {
    if id == target {
        return false;
    }
    let n = crate::notoriety::compute(w, id, target);
    n == crate::notoriety::CRIMINAL || n == crate::notoriety::MURDERER
}

/// `x.on_beneficial_action(..)`: the virtual call (an override runs; `on_beneficial_action_base` is what `base.on_beneficial_action(..)` reaches).
pub fn on_beneficial_action(w: &mut World, id: MobileId, a2: MobileId, a3: bool) {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.on_beneficial_action)(w, id, a2, a3),
        None => on_beneficial_action_base(w, id, a2, a3),
    }
}

pub fn on_beneficial_action_base(
    w: &mut World,
    id: MobileId,
    _target: MobileId,
    is_criminal: bool,
) {
    if is_criminal && let Some(ty) = w.mobile(id).map(|m| m.ty) {
        (ty.criminal_action)(w, id, false);
    }
}

pub fn do_beneficial(w: &mut World, id: MobileId, target: MobileId) {
    let Some(ty) = w.mobile(id).map(|m| m.ty) else {
        return;
    };
    if w.mobile(target).is_none() {
        return;
    }
    let crim = (ty.is_beneficial_criminal)(w, id, target);
    (ty.on_beneficial_action)(w, id, target, crim);

    p2::deps::region_on_beneficial_action(w, id, target);
    p2::deps::region_on_got_beneficial_action(w, id, target);
}

pub fn beneficial_check(w: &mut World, id: MobileId, target: MobileId) -> bool {
    let Some(ty) = w.mobile(id).map(|m| m.ty) else {
        return false;
    };
    if (ty.can_be_beneficial_msg)(w, id, target, true) {
        (ty.do_beneficial)(w, id, target);
        return true;
    }
    false
}

pub fn can_be_harmful(w: &mut World, id: MobileId, target: MobileId) -> bool {
    if let Some(m) = w.mobile(id) {
        (m.ty.can_be_harmful_msg)(w, id, target, true)
    } else {
        false
    }
}

pub fn can_be_harmful_msg(w: &mut World, id: MobileId, target: MobileId, message: bool) -> bool {
    if let Some(m) = w.mobile(id) {
        (m.ty.can_be_harmful_full)(w, id, target, message, false)
    } else {
        false
    }
}

/// `x.can_be_harmful_full(..)`: the virtual call (an override runs; `can_be_harmful_full_base` is what `base.can_be_harmful_full(..)` reaches).
pub fn can_be_harmful_full(w: &mut World, id: MobileId, a2: MobileId, a3: bool, a4: bool) -> bool {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.can_be_harmful_full)(w, id, a2, a3, a4),
        None => can_be_harmful_full_base(w, id, a2, a3, a4),
    }
}

pub fn can_be_harmful_full_base(
    w: &mut World,
    id: MobileId,
    target: MobileId,
    message: bool,
    ignore_our_blessedness: bool,
) -> bool {
    let (Some(m), Some(t)) = (w.mobile(id), w.mobile(target)) else {
        return false;
    };
    let (ty, tty) = (m.ty, t.ty);
    if m.deleted
        || (!ignore_our_blessedness && m.blessed)
        || t.deleted
        || t.blessed
        || !(ty.alive)(w, id)
        || (ty.is_dead_bonded_pet)(w, id)
        || !(tty.alive)(w, target)
        || (tty.is_dead_bonded_pet)(w, target)
    {
        if message {
            // You can not perform negative acts on your target.
            p2::messages::send_localized_message(w, id, 1001018, "", 0x3B2);
        }
        return false;
    }

    if target == id {
        return true;
    }

    if !p2::deps::pvp_combat_enabled(w) && m.player && t.player {
        if message {
            p2::messages::send_localized_message(w, id, 1001018, "", 0x3B2);
        }
        return false;
    }

    // TODO: Pets
    if !p2::deps::region_allow_harmful(w, id, target) {
        if message {
            p2::messages::send_localized_message(w, id, 1001018, "", 0x3B2);
        }
        return false;
    }

    true
}

/// `x.is_harmful_criminal(..)`: the virtual call (an override runs; `is_harmful_criminal_base` is what `base.is_harmful_criminal(..)` reaches).
pub fn is_harmful_criminal(w: &mut World, id: MobileId, a2: MobileId) -> bool {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.is_harmful_criminal)(w, id, a2),
        None => is_harmful_criminal_base(w, id, a2),
    }
}

pub fn is_harmful_criminal_base(w: &mut World, id: MobileId, target: MobileId) -> bool {
    id != target && crate::notoriety::compute(w, id, target) == crate::notoriety::INNOCENT
}

pub fn on_harmful_action(w: &mut World, id: MobileId, _target: MobileId, is_criminal: bool) {
    if is_criminal && let Some(ty) = w.mobile(id).map(|m| m.ty) {
        (ty.criminal_action)(w, id, false);
    }
}

/// `x.do_harmful(..)`: the virtual call (an override runs; `do_harmful_base` is what `base.do_harmful(..)` reaches).
pub fn do_harmful(w: &mut World, id: MobileId, a2: MobileId, a3: bool) {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.do_harmful)(w, id, a2, a3),
        None => do_harmful_base(w, id, a2, a3),
    }
}

pub fn do_harmful_base(w: &mut World, id: MobileId, target: MobileId, indirect: bool) {
    let Some(m) = w.mobile(id) else { return };
    let ty = m.ty;
    let Some(tty) = w.mobile(target).map(|t| t.ty) else {
        return;
    };
    if m.deleted {
        return;
    }

    let is_criminal = (ty.is_harmful_criminal)(w, id, target);

    (ty.on_harmful_action)(w, id, target, is_criminal);
    (tty.aggressive_action_criminal)(w, target, id, is_criminal);

    p2::deps::region_on_did_harmful(w, id, target);
    p2::deps::region_on_got_harmful(w, id, target);

    if !indirect && !p1::props_a::changing_combatant(w, id) {
        (ty.set_combatant)(w, id, Some(target));
    }

    let mut h = w
        .mobile(id)
        .map(|m| m.expire_combatant_timer)
        .unwrap_or_default();
    h.cancel(w);
    if let Some(m) = w.mobile_mut(id) {
        m.expire_combatant_timer = h;
    }
    let delay = p1::props_a::expire_combatant_delay(w);
    let h = crate::timer::start(w, delay, TimeSpan::ZERO, 1, move |w| {
        p1::props_a::expire_combatant(w, id)
    });
    if let Some(m) = w.mobile_mut(id) {
        m.expire_combatant_timer = h;
    }
}

pub fn harmful_check(w: &mut World, id: MobileId, target: MobileId) -> bool {
    let Some(ty) = w.mobile(id).map(|m| m.ty) else {
        return false;
    };
    if (ty.can_be_harmful)(w, id, target) {
        (ty.do_harmful)(w, id, target, false);
        return true;
    }
    false
}

pub fn remove_stat_mod(w: &mut World, id: MobileId, mod_: ModId) {
    let Some(m) = w.mobile_mut(id) else { return };
    let ty = m.ty;
    let Some(pos) = m.stat_mods.iter().position(|s| s.id == mod_) else {
        return;
    };
    let mut sm = m.stat_mods.remove(pos);
    sm.remove(w);
    (ty.check_stat_timers)(w, id);
    (ty.delta)(w, id, MobileDelta::STAT | p2::stats::get_stat_delta(sm.ty));
}

pub fn remove_stat_mod_by_name(w: &mut World, id: MobileId, name: &str) {
    let Some(m) = w.mobile(id) else { return };
    let ty = m.ty;
    let mut i = m.stat_mods.len();
    while i > 0 {
        i -= 1;
        let Some(m) = w.mobile_mut(id) else { return };
        if i >= m.stat_mods.len() || m.stat_mods[i].name.as_deref() != Some(name) {
            continue;
        }
        let mut sm = m.stat_mods.remove(i);
        sm.remove(w);
        (ty.check_stat_timers)(w, id);
        (ty.delta)(w, id, MobileDelta::STAT | p2::stats::get_stat_delta(sm.ty));
    }
}

/// The index of the first match. `a && b || includeElapsed` makes
/// `includeElapsed` return the first mod whatever its name.
pub fn get_stat_mod(w: &World, id: MobileId, name: &str, include_elapsed: bool) -> Option<usize> {
    let m = w.mobile(id)?;
    m.stat_mods
        .iter()
        .position(|s| (s.name.as_deref() == Some(name) && !s.has_elapsed(w.now)) || include_elapsed)
}

pub fn add_stat_mod(w: &mut World, id: MobileId, mod_: StatMod) {
    let mut mod_ = mod_;
    mod_.owner = Some(id);
    let sty = mod_.ty;
    let Some(m) = w.mobile_mut(id) else { return };
    let ty = m.ty;
    m.stat_mods.push(mod_);
    (ty.delta)(w, id, MobileDelta::STAT | p2::stats::get_stat_delta(sty));
    (ty.check_stat_timers)(w, id);
}

pub fn on_raw_str_change(_w: &mut World, _id: MobileId, _old_value: i32) {}

pub fn on_raw_dex_change(_w: &mut World, _id: MobileId, _old_value: i32) {}

pub fn on_raw_int_change(_w: &mut World, _id: MobileId, _old_value: i32) {}

pub fn on_raw_stat_change(_w: &mut World, _id: MobileId, _stat: StatType, _old_value: i32) {}

pub fn on_hits_change(_w: &mut World, _id: MobileId, _old_value: i32) {}

pub fn on_stam_change(_w: &mut World, _id: MobileId, _old_value: i32) {}

/// `x.on_mana_change(..)`: the virtual call (an override runs; `on_mana_change_base` is what `base.on_mana_change(..)` reaches).
pub fn on_mana_change(w: &mut World, id: MobileId, a2: i32) {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.on_mana_change)(w, id, a2),
        None => on_mana_change_base(w, id, a2),
    }
}

pub fn on_mana_change_base(_w: &mut World, _id: MobileId, _old_value: i32) {}

/// `x.on_poison_immunity(..)`: the virtual call (an override runs; `on_poison_immunity_base` is what `base.on_poison_immunity(..)` reaches).
pub fn on_poison_immunity(w: &mut World, id: MobileId, a2: Option<MobileId>, a3: PoisonId) {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.on_poison_immunity)(w, id, a2, a3),
        None => on_poison_immunity_base(w, id, a2, a3),
    }
}

pub fn on_poison_immunity_base(
    w: &mut World,
    id: MobileId,
    _from: Option<MobileId>,
    _poison: PoisonId,
) {
    // * The poison seems to have no effect. *
    p2::messages::public_overhead_message_number(
        w,
        id,
        MessageType::EMOTE,
        0x3B2,
        1005534,
        "",
        true,
    );
}

pub fn on_higher_poison(_w: &mut World, _id: MobileId, _from: Option<MobileId>, _poison: PoisonId) {
}

pub fn on_poisoned(
    w: &mut World,
    id: MobileId,
    _from: Option<MobileId>,
    poison: PoisonId,
    _old_poison: Option<PoisonId>,
) {
    let level = crate::poison::level(w, poison);
    p2::messages::local_overhead_message_number(
        w,
        id,
        MessageType::REGULAR,
        0x21,
        1042857 + level * 2,
        "",
    );
    let name = w
        .mobile(id)
        .and_then(|m| (m.ty.name)(w, id))
        .unwrap_or_default();
    p2::messages::nonlocal_overhead_message_number(
        w,
        id,
        MessageType::REGULAR,
        0x21,
        1042858 + level * 2,
        &name,
    );
}

/// `x.check_poison_immunity(..)`: the virtual call (an override runs; `check_poison_immunity_base` is what `base.check_poison_immunity(..)` reaches).
pub fn check_poison_immunity(w: &World, id: MobileId, a2: Option<MobileId>, a3: PoisonId) -> bool {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.check_poison_immunity)(w, id, a2, a3),
        None => check_poison_immunity_base(w, id, a2, a3),
    }
}

pub fn check_poison_immunity_base(
    _w: &World,
    _id: MobileId,
    _from: Option<MobileId>,
    _poison: PoisonId,
) -> bool {
    false
}

pub fn check_higher_poison(
    w: &World,
    id: MobileId,
    _from: Option<MobileId>,
    poison: PoisonId,
) -> bool {
    w.mobile(id)
        .and_then(|m| m.poison)
        .is_some_and(|p| crate::poison::level(w, p) >= crate::poison::level(w, poison))
}

/// `x.apply_poison(..)`: the virtual call (an override runs; `apply_poison_base` is what `base.apply_poison(..)` reaches).
pub fn apply_poison(
    w: &mut World,
    id: MobileId,
    a2: Option<MobileId>,
    a3: PoisonId,
) -> ApplyPoisonResult {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.apply_poison)(w, id, a2, a3),
        None => apply_poison_base(w, id, a2, a3),
    }
}

/// `ApplyPoison`. The `poison == null` branch (cure, `Cured`) can't be asked here: `PoisonId` is
/// never null; call `cure_poison` for that.
pub fn apply_poison_base(
    w: &mut World,
    id: MobileId,
    from: Option<MobileId>,
    poison: PoisonId,
) -> ApplyPoisonResult {
    let Some(ty) = w.mobile(id).map(|m| m.ty) else {
        return ApplyPoisonResult::Poisoned;
    };
    if (ty.check_higher_poison)(w, id, from, poison) {
        (ty.on_higher_poison)(w, id, from, poison);
        return ApplyPoisonResult::HigherPoisonActive;
    }

    if (ty.check_poison_immunity)(w, id, from, poison) {
        (ty.on_poison_immunity)(w, id, from, poison);
        return ApplyPoisonResult::Immune;
    }

    let old_poison = w.mobile(id).and_then(|m| m.poison);
    props_b::set_poison(w, id, Some(poison));

    (ty.on_poisoned)(w, id, from, poison, old_poison);

    ApplyPoisonResult::Poisoned
}

pub fn check_cure(_w: &mut World, _id: MobileId, _from: Option<MobileId>) -> bool {
    true
}

pub fn on_cured(
    _w: &mut World,
    _id: MobileId,
    _from: Option<MobileId>,
    _old_poison: Option<PoisonId>,
) {
}

pub fn on_failed_cure(_w: &mut World, _id: MobileId, _from: Option<MobileId>) {}

pub fn cure_poison(w: &mut World, id: MobileId, from: Option<MobileId>) -> bool {
    let Some(ty) = w.mobile(id).map(|m| m.ty) else {
        return false;
    };
    if (ty.check_cure)(w, id, from) {
        let old_poison = w.mobile(id).and_then(|m| m.poison);
        props_b::set_poison(w, id, None);

        (ty.on_cured)(w, id, from, old_poison);

        return true;
    }

    (ty.on_failed_cure)(w, id, from);

    false
}

/// `x.on_double_click(..)`: the virtual call (an override runs; `on_double_click_base` is what `base.on_double_click(..)` reaches).
pub fn on_double_click(w: &mut World, id: MobileId, a2: MobileId) {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.on_double_click)(w, id, a2),
        None => on_double_click_base(w, id, a2),
    }
}

pub fn on_double_click_base(w: &mut World, id: MobileId, from: MobileId) {
    let Some(m) = w.mobile(id) else { return };
    let (ty, warmode) = (m.ty, m.warmode);
    if id == from
        && (!p2::statics::read(w, |s| s.disable_dismount_in_warmode) || !warmode)
        && let Some(mount) = props_b::mount(w, id)
    {
        p2::deps::mount_clear_rider(w, mount);
        return;
    }

    if (ty.can_paperdoll_be_opened_by)(w, id, from) {
        (ty.display_paperdoll_to)(w, id, from);
    }
}

pub fn on_double_click_out_of_range(_w: &mut World, _id: MobileId, _from: MobileId) {}

pub fn on_double_click_cant_see(_w: &mut World, _id: MobileId, _from: MobileId) {}

/// `x.on_double_click_dead(..)`: the virtual call (an override runs; `on_double_click_dead_base` is what `base.on_double_click_dead(..)` reaches).
pub fn on_double_click_dead(w: &mut World, id: MobileId, a2: MobileId) {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.on_double_click_dead)(w, id, a2),
        None => on_double_click_dead_base(w, id, a2),
    }
}

pub fn on_double_click_dead_base(w: &mut World, id: MobileId, from: MobileId) {
    let Some(ty) = w.mobile(id).map(|m| m.ty) else {
        return;
    };
    if (ty.can_paperdoll_be_opened_by)(w, id, from) {
        (ty.display_paperdoll_to)(w, id, from);
    }
}

#[cfg(test)]
mod speech_tests {
    use super::*;
    use crate::mobile::MobileType;

    fn first(_: &mut World, _: MobileId, e: &mut SpeechEventArgs) {
        e.handled = true;
    }
    fn second(w: &mut World, _: MobileId, e: &mut SpeechEventArgs) {
        if e.handled {
            w.resources.get_mut::<Seen>().0 += 1;
        }
    }
    #[derive(Default)]
    struct Seen(i32);

    static FIRST: MobileType = MobileType {
        full_name: "Test.First",
        handles_on_speech: |_, _, _| true,
        on_speech: first,
        ..MOBILE_BASE
    };
    static SECOND: MobileType = MobileType {
        full_name: "Test.Second",
        handles_on_speech: |_, _, _| true,
        on_speech: second,
        ..MOBILE_BASE
    };

    // Mobile.DoSpeech: one SpeechEventArgs is shared by every OnSpeech listener, nearest first.
    #[test]
    fn a_second_listener_sees_handled_set_by_the_first() {
        let mut w = crate::map::test_support::world();
        let speaker = w.new_mobile(&MOBILE_BASE);
        let a = w.new_mobile(&FIRST);
        let b = w.new_mobile(&SECOND);
        for (m, x) in [(speaker, 100), (a, 101), (b, 102)] {
            move_to_world(&mut w, m, Point3D::new(x, 100, 0), MapId::FELUCCA);
        }
        do_speech(&mut w, speaker, "hello", &[], MessageType::REGULAR, 0);
        assert_eq!(w.resources.get_mut::<Seen>().0, 1);
    }
}
