//! `Mobile` properties, first part: `DamageEntry`, the statics declared there, and the properties
//! up to `Blessed`. The virtual ones (`IgnoreMobiles`, `Paralyzed`, `Combatant`, `RegenThroughPoison`..) have their
//! base bodies in `mobile::base` (the setters call `set_*_base` here); the helpers they share
//! (`check_combat_time`, `expire_combatant`) are here too.
//!
//! Statics: every static of this range is in [`PropsAStatics`] (`DragEffects`, `ActionDelay`, `AutoManifestTimeout`,
//! `NoSpeechLOS`, `ExpireCombatantDelay`, `MaxPlayerResistance`..) with an accessor fn each, except `GhostChars`,
//! `CreateCorpseHandler` and `_disableCastParalyze`, which `part2::statics::Part2Statics` holds.

use muo_primitives::body::BodyTable;
use muo_primitives::{Body, DateTime, Serial, TimeSpan};

use crate::ids::{ItemId, MobileId};
use crate::item::TotalType;
use crate::mobile::part2::statics::ACCESS_LEVEL_NAMES;
use crate::mobile::{
    AccessLevel, DamageEntry, Mobile, MobileDelta, StatLockType, VisibleDamageType,
};
use crate::race::{self, RaceId, Races};
use crate::skills;
use crate::timer::{self, TimerHandle};
use crate::world::World;

/// `WarmodeCatchCount`: warmode changes allowed within `WarmodeSpamCatch` before the delay.
pub const WARMODE_CATCH_COUNT: i32 = 4;

/// The `Mobile` statics of this part that `part2` does not hold. One value in `w.resources`.
#[derive(Clone, Debug)]
pub struct PropsAStatics {
    pub damage_entry_expire_delay: TimeSpan,
    pub max_player_resistance: i32,
    pub action_message_delay: i32,
    pub global_regen_through_poison: bool,
    pub visible_damage_type: VisibleDamageType,
    pub default_show_visible_damage: bool,
    pub default_can_see_visible_damage: bool,
    pub expire_aggressors_delay: TimeSpan,
    pub expire_combatant_delay: TimeSpan,
    pub drag_effects: bool,
    pub no_speech_los: bool,
    pub auto_manifest_timeout: TimeSpan,
    pub action_delay: i32,
}

impl Default for PropsAStatics {
    fn default() -> Self {
        PropsAStatics {
            damage_entry_expire_delay: TimeSpan::from_mins(2),
            max_player_resistance: 70,
            action_message_delay: 125,
            global_regen_through_poison: true,
            visible_damage_type: VisibleDamageType::None,
            default_show_visible_damage: false,
            default_can_see_visible_damage: false,
            expire_aggressors_delay: TimeSpan::from_secs(5),
            expire_combatant_delay: TimeSpan::from_mins(1),
            drag_effects: true,
            no_speech_los: false,
            auto_manifest_timeout: TimeSpan::from_secs(5),
            action_delay: 500,
        }
    }
}

/// Reads a static of [`PropsAStatics`], with its default when nothing wrote one yet.
pub fn read_static<R>(w: &World, f: impl FnOnce(&PropsAStatics) -> R) -> R {
    match w.resources.get::<PropsAStatics>() {
        Some(s) => f(s),
        None => f(&PropsAStatics::default()),
    }
}

/// Mutable access to [`PropsAStatics`].
pub fn statics_mut(w: &mut World) -> &mut PropsAStatics {
    w.resources.get_mut::<PropsAStatics>()
}

/// `Core.AOS`.
pub fn core_aos(w: &World) -> bool {
    w.core_aos()
}

/// `Core.SE`.
pub fn core_se(w: &World) -> bool {
    w.core_se()
}

/// `WarmodeSpamCatch`: `TimeSpan.FromSeconds(Core.SE ? 1.0 : 0.5)`.
pub fn warmode_spam_catch(w: &World) -> TimeSpan {
    if core_se(w) {
        TimeSpan::from_secs(1)
    } else {
        TimeSpan::from_millis(500)
    }
}

/// `WarmodeSpamDelay`: `TimeSpan.FromSeconds(Core.SE ? 4.0 : 2.0)`.
pub fn warmode_spam_delay(w: &World) -> TimeSpan {
    if core_se(w) {
        TimeSpan::from_secs(4)
    } else {
        TimeSpan::from_secs(2)
    }
}

pub fn expire_combatant_delay(w: &World) -> TimeSpan {
    read_static(w, |s| s.expire_combatant_delay)
}

pub fn expire_aggressors_delay(w: &World) -> TimeSpan {
    read_static(w, |s| s.expire_aggressors_delay)
}

pub fn drag_effects(w: &World) -> bool {
    read_static(w, |s| s.drag_effects)
}

pub fn action_delay(w: &World) -> i32 {
    read_static(w, |s| s.action_delay)
}

pub fn auto_manifest_timeout(w: &World) -> TimeSpan {
    read_static(w, |s| s.auto_manifest_timeout)
}

pub fn no_speech_los(w: &World) -> bool {
    read_static(w, |s| s.no_speech_los)
}

pub fn max_player_resistance(w: &World) -> i32 {
    read_static(w, |s| s.max_player_resistance)
}

pub fn action_message_delay(w: &World) -> i32 {
    read_static(w, |s| s.action_message_delay)
}

pub fn global_regen_through_poison(w: &World) -> bool {
    read_static(w, |s| s.global_regen_through_poison)
}

pub fn visible_damage_type(w: &World) -> VisibleDamageType {
    read_static(w, |s| s.visible_damage_type)
}

pub fn default_show_visible_damage(w: &World) -> bool {
    read_static(w, |s| s.default_show_visible_damage)
}

pub fn default_can_see_visible_damage(w: &World) -> bool {
    read_static(w, |s| s.default_can_see_visible_damage)
}

// ---------------------------------------------------------------------------------------------------------------
// DamageEntry
// ---------------------------------------------------------------------------------------------------------------

impl DamageEntry {
    /// `new DamageEntry(damager)`: `LastDamage` is `DateTime.MinValue`.
    pub fn new(damager: MobileId) -> DamageEntry {
        DamageEntry {
            damager,
            damage_given: 0,
            last_damage: DateTime::MIN,
            responsible: Vec::new(),
        }
    }

    /// `DamageEntry.HasExpired`: `Core.Now > LastDamage + ExpireDelay`.
    pub fn has_expired(&self, w: &World) -> bool {
        w.now > self.last_damage + damage_entry_expire_delay(w)
    }
}

pub fn damage_entry_expire_delay(w: &World) -> TimeSpan {
    read_static(w, |s| s.damage_entry_expire_delay)
}

pub fn set_damage_entry_expire_delay(w: &mut World, value: TimeSpan) {
    statics_mut(w).damage_entry_expire_delay = value;
}

// ---------------------------------------------------------------------------------------------------------------
// Properties
// ---------------------------------------------------------------------------------------------------------------

/// Cancels the timer a mobile field holds (`_xTimerToken.Cancel()`); the field is reset.
fn cancel_timer(w: &mut World, id: MobileId, field: fn(&mut Mobile) -> &mut TimerHandle) {
    let Some(m) = w.mobile_mut(id) else { return };
    let mut h = std::mem::take(field(m));
    h.cancel(w);
}

/// `Mobile.Race` getter: `m_Race ??= Race.DefaultRace`.
pub fn race(w: &mut World, id: MobileId) -> RaceId {
    let Some(m) = w.mobile_mut(id) else {
        return RaceId(0);
    };
    *m.race.get_or_insert(RaceId(0))
}

/// `Mobile.Race` setter. `None` is the default race.
pub fn set_race(w: &mut World, id: MobileId, value: Option<RaceId>) {
    let old_race = race(w, id);
    let new_race = value.unwrap_or(RaceId(0));
    if let Some(m) = w.mobile_mut(id) {
        m.race = Some(new_race);
    }

    let body = w
        .resources
        .get::<Races>()
        .and_then(|r| r.get(new_race))
        .map(|r| race::body(w, r, id));
    if let Some(body) = body {
        crate::mobile::part1::props_b::set_body(w, id, Body::from(body));
    }
    if let Some(m) = w.mobile(id) {
        (m.ty.update_resistances)(w, id);
    }

    crate::mobile::base::delta(w, id, MobileDelta::RACE);

    if let Some(m) = w.mobile(id) {
        (m.ty.on_race_change)(w, id, old_race);
    }
}

pub fn set_virtual_armor_mod(w: &mut World, id: MobileId, value: i32) {
    let Some(m) = w.mobile_mut(id) else { return };
    if m.virtual_armor_mod != value {
        m.virtual_armor_mod = value;
        crate::mobile::base::delta(w, id, MobileDelta::ARMOR);
    }
}

/// `Mobile.SkillsTotal`: `Skills?.Total ?? 0`.
pub fn skills_total(w: &World, id: MobileId) -> i32 {
    skills::total(w, id)
}

/// `Mobile.SkillsCap` getter: `Skills?.Cap ?? 0`.
pub fn skills_cap(w: &World, id: MobileId) -> i32 {
    w.mobile(id)
        .and_then(|m| m.skills.as_ref())
        .map_or(0, |s| s.cap)
}

pub fn set_skills_cap(w: &mut World, id: MobileId, value: i32) {
    if let Some(s) = w.mobile_mut(id).and_then(|m| m.skills.as_mut()) {
        s.cap = value;
    }
}

pub fn set_hunger(w: &mut World, id: MobileId, value: i32) {
    let Some(m) = w.mobile_mut(id) else { return };
    let old_value = m.hunger;
    if old_value != value {
        m.hunger = value;
        let ty = m.ty;
        (ty.on_hunger_changed)(w, id, old_value);
    }
}

pub fn set_thirst(w: &mut World, id: MobileId, value: i32) {
    let Some(m) = w.mobile_mut(id) else { return };
    let old_value = m.thirst;
    if old_value != value {
        m.thirst = value;
        let ty = m.ty;
        (ty.on_thirst_changed)(w, id, old_value);
    }
}

/// `Item.TotalWeight + Item.PileWeight` of the held item.
fn held_weight(w: &World, item: ItemId) -> i32 {
    crate::item::ops::get_total(w, item, TotalType::Weight) + crate::item::ops::pile_weight(w, item)
}

pub fn set_holding(w: &mut World, id: MobileId, value: Option<ItemId>) {
    let Some(m) = w.mobile(id) else { return };
    let (current, ty) = (m.holding, m.ty);
    if current == value {
        return;
    }

    if let Some(held) = current {
        let weight = held_weight(w, held);
        (ty.update_total)(w, id, held, TotalType::Weight, -weight);
        if let Some(item) = w.item_mut(held)
            && item.held_by == Some(id)
        {
            item.held_by = None;
        }
    }

    // `m_Holding` is still set here, so this is `value != null && m_Holding != null`.
    if value.is_some() && current.is_some() {
        crate::mobile::part2::items::drop_holding(w, id);
    }

    if let Some(m) = w.mobile_mut(id) {
        m.holding = value;
    }

    if let Some(held) = value {
        let weight = held_weight(w, held);
        (ty.update_total)(w, id, held, TotalType::Weight, weight);
        if let Some(item) = w.item_mut(held) {
            item.held_by.get_or_insert(id);
        }
    }
}

/// `Mobile.Paralyzed` setter: the base body of `MobileType::set_paralyzed`.
pub fn set_paralyzed_base(w: &mut World, id: MobileId, value: bool) {
    let Some(m) = w.mobile_mut(id) else { return };
    if m.paralyzed != value {
        m.paralyzed = value;
        crate::mobile::base::delta(w, id, MobileDelta::FLAGS);

        crate::mobile::part2::messages::send_localized_message(
            w,
            id,
            if value { 502381 } else { 502382 },
            "",
            0x3B2,
        );
        cancel_timer(w, id, |m| &mut m.para_timer);
    }

    if !value && let Some(ns) = w.mobile(id).and_then(|m| m.net_state) {
        let now = w.tick_count;
        if let Some(d) = w.net_state_mut(ns)
            && now.wrapping_sub(d.movement.next_movement_time) > 0
        {
            d.movement.next_movement_time = now;
        }
    }
}

/// `Mobile.IgnoreMobiles` setter: the base body of `MobileType::set_ignore_mobiles`.
pub fn set_ignore_mobiles_base(w: &mut World, id: MobileId, value: bool) {
    let Some(m) = w.mobile_mut(id) else { return };
    if m.ignore_mobiles != value {
        m.ignore_mobiles = value;
        crate::mobile::base::delta(w, id, MobileDelta::FLAGS);
    }
}

pub fn set_frozen(w: &mut World, id: MobileId, value: bool) {
    let Some(m) = w.mobile_mut(id) else { return };
    if m.frozen != value {
        m.frozen = value;
        crate::mobile::base::delta(w, id, MobileDelta::FLAGS);
        cancel_timer(w, id, |m| &mut m.frozen_timer);
    }
}

/// `m_NetState.SendStatLockInfo(this)`.
fn send_stat_lock_info(w: &mut World, id: MobileId) {
    if let Some(ns) = w.mobile(id).and_then(|m| m.net_state) {
        crate::packets::player::send_stat_lock_info(w, ns, id);
    }
}

pub fn set_str_lock(w: &mut World, id: MobileId, value: StatLockType) {
    let Some(m) = w.mobile_mut(id) else { return };
    if m.str_lock != value {
        m.str_lock = value;
        send_stat_lock_info(w, id);
    }
}

pub fn set_dex_lock(w: &mut World, id: MobileId, value: StatLockType) {
    let Some(m) = w.mobile_mut(id) else { return };
    if m.dex_lock != value {
        m.dex_lock = value;
        send_stat_lock_info(w, id);
    }
}

pub fn set_int_lock(w: &mut World, id: MobileId, value: StatLockType) {
    let Some(m) = w.mobile_mut(id) else { return };
    if m.int_lock != value {
        m.int_lock = value;
        send_stat_lock_info(w, id);
    }
}

/// `Mobile.ChangingCombatant`: `m_ChangingCombatant > 0`.
pub fn changing_combatant(w: &World, id: MobileId) -> bool {
    w.mobile(id).is_some_and(|m| m.changing_combatant > 0)
}

/// `Mobile.ExpireCombatant`: the expire-combatant timer's callback.
pub fn expire_combatant(w: &mut World, id: MobileId) {
    if let Some(m) = w.mobile(id) {
        (m.ty.set_combatant)(w, id, None);
    }
    cancel_timer(w, id, |m| &mut m.expire_combatant_timer);
}

/// Starts the expire-combatant and combat timers of `Combatant`'s setter if they are not running.
fn start_combat_timers(w: &mut World, id: MobileId) {
    let expire_running = w
        .mobile(id)
        .is_some_and(|m| m.expire_combatant_timer.running(w));
    if !expire_running {
        let delay = expire_combatant_delay(w);
        let h = timer::start(w, delay, TimeSpan::ZERO, 1, move |w| {
            expire_combatant(w, id)
        });
        if let Some(m) = w.mobile_mut(id) {
            m.expire_combatant_timer = h;
        }
    }

    let combat_running = w.mobile(id).is_some_and(|m| m.combat_timer.running(w));
    if !combat_running {
        let h = timer::start(w, TimeSpan::from_millis(10), TimeSpan::ZERO, 0, move |w| {
            check_combat_time(w, id)
        });
        if let Some(m) = w.mobile_mut(id) {
            m.combat_timer = h;
        }
    }
}

/// `Mobile.Combatant` setter: the base body of `MobileType::set_combatant`.
pub fn set_combatant_base(w: &mut World, id: MobileId, value: Option<MobileId>) {
    let Some(m) = w.mobile_mut(id) else { return };
    if m.deleted || m.combatant == value || value == Some(id) {
        return;
    }

    let old = m.combatant;
    let ty = m.ty;

    m.changing_combatant += 1;
    m.combatant = value;

    let rejected = value.is_some_and(|c| !(ty.can_be_harmful_msg)(w, id, c, false))
        || !crate::mobile::part2::deps::region_on_combatant_change(w, id, old, value);
    if rejected {
        if let Some(m) = w.mobile_mut(id) {
            m.combatant = old;
            m.changing_combatant -= 1;
        }
        return;
    }

    let net_state = w.mobile(id).and_then(|m| m.net_state);
    match value {
        None => {
            if let Some(ns) = net_state {
                crate::packets::combat::send_change_combatant(w, ns, Serial::ZERO);
            }
            cancel_timer(w, id, |m| &mut m.expire_combatant_timer);
            cancel_timer(w, id, |m| &mut m.combat_timer);
        }
        Some(c) => {
            if let Some(ns) = net_state {
                crate::packets::combat::send_change_combatant(w, ns, c.0);
            }
            start_combat_timers(w, id);

            if (ty.can_be_harmful_msg)(w, id, c, false) {
                (ty.do_harmful)(w, id, c, false);
                // "due to reflection, might make m_Combatant null"
                if let Some(now) = w.mobile(id).and_then(|m| m.combatant)
                    && let Some(cty) = w.mobile(now).map(|cm| cm.ty)
                {
                    let sound = (cty.get_anger_sound)(w, now);
                    crate::mobile::part2::effects::play_sound(w, now, sound);
                }
            }
        }
    }

    (ty.on_combatant_change)(w, id);
    if let Some(m) = w.mobile_mut(id) {
        m.changing_combatant -= 1;
    }
}

/// `Mobile.CheckCombatTime`: the combat timer's callback.
pub fn check_combat_time(w: &mut World, id: MobileId) {
    let Some(m) = w.mobile(id) else { return };
    if w.tick_count.wrapping_sub(m.next_combat_time) < 0 {
        return;
    }
    let ty = m.ty;

    let combatant = (ty.combatant)(w, id);

    // If no combatant, wrong map, one of us is a ghost, or cannot see, or deleted, then stop combat
    // (deleted, map, type) of the combatant and of this mobile
    let facts = |w: &World, c: MobileId| w.mobile(c).map(|x| (x.deleted, x.map, x.ty));
    let stop = match (
        combatant.and_then(|c| facts(w, c).map(|f| (c, f))),
        facts(w, id),
    ) {
        (Some((c, (cdel, cmap, cty))), Some((mdel, mmap, _))) => {
            cdel || mdel
                || cmap != mmap
                || !(cty.alive)(w, c)
                || !(ty.alive)(w, id)
                || !(ty.can_see_mobile)(w, id, c)
                || (cty.is_dead_bonded_pet)(w, c)
                || (ty.is_dead_bonded_pet)(w, id)
        }
        _ => true,
    };
    if stop {
        (ty.set_combatant)(w, id, None);
        return;
    }

    // The weapon part belongs to content (`IWeapon`): `weapon_combat::combat_swing`.
    let swing = w
        .resources
        .get::<crate::mobile::part1::props_b::PropsBStatics>()
        .and_then(|s| s.combat_swing);
    if let (Some(swing), Some(c)) = (swing, combatant) {
        swing(w, id, c);
    }
}

fn get_total(w: &World, id: MobileId, ty: TotalType) -> i32 {
    w.mobile(id).map_or(0, |m| (m.ty.get_total)(w, id, ty))
}

/// `Mobile.TotalGold`: `GetTotal(TotalType.Gold)`.
pub fn total_gold(w: &World, id: MobileId) -> i32 {
    get_total(w, id, TotalType::Gold)
}

/// `Mobile.TotalItems`: `GetTotal(TotalType.Items)`.
pub fn total_items(w: &World, id: MobileId) -> i32 {
    get_total(w, id, TotalType::Items)
}

/// `Mobile.TotalWeight`: `GetTotal(TotalType.Weight)`.
pub fn total_weight(w: &World, id: MobileId) -> i32 {
    get_total(w, id, TotalType::Weight)
}

pub fn set_tithing_points(w: &mut World, id: MobileId, value: i32) {
    let Some(m) = w.mobile_mut(id) else { return };
    if m.tithing_points != value {
        m.tithing_points = value;
        crate::mobile::base::delta(w, id, MobileDelta::TITHING_POINTS);
    }
}

pub fn set_followers(w: &mut World, id: MobileId, value: i32) {
    let Some(m) = w.mobile_mut(id) else { return };
    if m.followers != value {
        m.followers = value;
        crate::mobile::base::delta(w, id, MobileDelta::FOLLOWERS);
    }
}

pub fn set_followers_max(w: &mut World, id: MobileId, value: i32) {
    let Some(m) = w.mobile_mut(id) else { return };
    if m.followers_max != value {
        m.followers_max = value;
        crate::mobile::base::delta(w, id, MobileDelta::FOLLOWERS);
    }
}

pub fn set_target(w: &mut World, id: MobileId, value: Option<crate::target::TargetId>) {
    crate::target::set_mobile_target(w, id, value);
}

pub fn set_spell(w: &mut World, id: MobileId, value: Option<crate::interfaces::SpellId>) {
    let Some(m) = w.mobile_mut(id) else { return };
    if m.spell.is_some() && value.is_some() {
        tracing::warn!("Spell has been overwritten.");
    }
    m.spell = value;
}

pub fn set_virtual_armor(w: &mut World, id: MobileId, value: i32) {
    let Some(m) = w.mobile_mut(id) else { return };
    if m.virtual_armor != value {
        m.virtual_armor = value;
        crate::mobile::base::delta(w, id, MobileDelta::ARMOR);
    }
}

/// `Mobile.DamageEntries`: prunes the expired entries (oldest first), then lists them.
pub fn damage_entries(w: &mut World, id: MobileId) -> &[DamageEntry] {
    crate::mobile::part2::damage::prune_expired_damage_entries(w, id);
    w.mobile(id).map_or(&[], |m| m.damage_entries.as_slice())
}

pub fn set_light_level(w: &mut World, id: MobileId, value: i32) {
    let Some(m) = w.mobile_mut(id) else { return };
    if m.light_level != value {
        m.light_level = value;
        let ty = m.ty;
        (ty.check_light_levels)(w, id, false);
    }
}

pub fn set_player(w: &mut World, id: MobileId, value: bool) {
    let Some(m) = w.mobile_mut(id) else { return };
    m.player = value;
    crate::mobile::part2::properties::invalidate_properties(w, id);
    crate::mobile::base::check_stat_timers(w, id);
}

pub fn set_title(w: &mut World, id: MobileId, value: Option<String>) {
    let Some(m) = w.mobile_mut(id) else { return };
    m.title = value;
    crate::mobile::part2::properties::invalidate_properties(w, id);
}

pub fn set_access_level(w: &mut World, id: MobileId, value: AccessLevel) {
    let Some(m) = w.mobile_mut(id) else { return };
    let old_value = m.access_level;
    if old_value == value {
        return;
    }
    m.access_level = value;
    let ty = m.ty;
    crate::mobile::base::delta(w, id, MobileDelta::NOTO);
    crate::mobile::part2::properties::invalidate_properties(w, id);

    let text = format!(
        "Your access level has been changed. You are now {}.",
        ACCESS_LEVEL_NAMES[value as usize]
    );
    crate::mobile::part2::messages::send_message(w, id, &text);

    crate::mobile::part2::effects::clear_screen(w, id);
    crate::mobile::part2::effects::send_everything(w, id);

    (ty.on_access_level_changed)(w, id, old_value);
}

pub fn set_fame(w: &mut World, id: MobileId, value: i32) {
    let Some(m) = w.mobile_mut(id) else { return };
    let old_value = m.fame;
    if old_value == value {
        return;
    }
    m.fame = value;
    let (ty, player, body) = (m.ty, m.player, m.body);

    let human = player
        || w.resources
            .get::<BodyTable>()
            .is_some_and(|t| t.is_human(body));
    if (ty.show_fame_title)(w, id) && human && (old_value >= 10000) != (value >= 10000) {
        crate::mobile::part2::properties::invalidate_properties(w, id);
    }

    (ty.on_fame_change)(w, id, old_value);
}

pub fn set_karma(w: &mut World, id: MobileId, value: i32) {
    let Some(m) = w.mobile_mut(id) else { return };
    let old = m.karma;
    if old != value {
        m.karma = value;
        let ty = m.ty;
        (ty.on_karma_change)(w, id, old);
    }
}

pub fn set_blessed(w: &mut World, id: MobileId, value: bool) {
    let Some(m) = w.mobile_mut(id) else { return };
    if m.blessed != value {
        m.blessed = value;
        crate::mobile::base::delta(w, id, MobileDelta::HEALTHBAR_YELLOW);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mobile::MOBILE_BASE;
    use crate::mobile::MobileType;
    use crate::mobile::part2::damage as dmg;

    /// A pet whose `GetDamageMaster` is the first mobile of the world (`PetMobile.Master`).
    static PET: MobileType = MobileType {
        full_name: "Server.Tests.PetMobile",
        parent: Some(&MOBILE_BASE),
        get_damage_master: |w, _, _| w.mobiles.keys().next().copied(),
        ..MOBILE_BASE
    };

    fn damagers(w: &mut World, victim: MobileId) -> Vec<MobileId> {
        damage_entries(w, victim)
            .iter()
            .map(|e| e.damager)
            .collect()
    }

    fn three(w: &mut World) -> (MobileId, MobileId, MobileId) {
        (
            w.new_mobile(&MOBILE_BASE),
            w.new_mobile(&MOBILE_BASE),
            w.new_mobile(&MOBILE_BASE),
        )
    }

    #[test]
    fn has_expired_after_expire_delay() {
        let mut w = World::new();
        let victim = w.new_mobile(&MOBILE_BASE);
        let mut e = DamageEntry::new(victim);
        e.last_damage = DateTime::MIN + TimeSpan::from_hours(1);
        w.now = e.last_damage + damage_entry_expire_delay(&w);
        assert!(!e.has_expired(&w));
        w.now += TimeSpan::from_millis(1);
        assert!(e.has_expired(&w));
        set_damage_entry_expire_delay(&mut w, TimeSpan::from_mins(10));
        assert!(!e.has_expired(&w));
    }

    #[test]
    fn statics_have_their_defaults() {
        let mut w = World::new();
        w.core.expansion = Some(muo_data::expansion::Expansion::LBR); // pre-SE: 0.5 s
        assert_eq!(max_player_resistance(&w), 70);
        assert_eq!(action_message_delay(&w), 125);
        assert!(global_regen_through_poison(&w));
        assert_eq!(expire_aggressors_delay(&w), TimeSpan::from_secs(5));
        assert_eq!(warmode_spam_catch(&w), TimeSpan::from_millis(500));
        assert_eq!(action_delay(&w), 500);
        assert_eq!(expire_combatant_delay(&w), TimeSpan::from_mins(1));
    }

    #[test]
    fn simple_setters_store_values() {
        let mut w = World::new();
        let id = w.new_mobile(&MOBILE_BASE);
        set_hunger(&mut w, id, 20);
        set_karma(&mut w, id, -5);
        set_virtual_armor_mod(&mut w, id, 3);
        let m = w.mobile(id).unwrap();
        assert_eq!((m.hunger, m.karma, m.virtual_armor_mod), (20, -5, 3));
    }

    // DamageEntryTests.FreshMobile_HasNoEntries
    #[test]
    fn fresh_mobile_has_no_entries() {
        let mut w = World::new();
        let m = w.new_mobile(&MOBILE_BASE);
        assert_eq!(damage_entries(&mut w, m).len(), 0);
        assert_eq!(dmg::find_most_recent_damage_entry(&mut w, m, true), None);
        assert_eq!(dmg::find_least_recent_damage_entry(&mut w, m, true), None);
        assert_eq!(dmg::find_most_total_damage_entry(&mut w, m, true), None);
        assert_eq!(dmg::find_least_total_damage_entry(&mut w, m, true), None);
        assert_eq!(dmg::find_damage_entry_for(&mut w, m, m), None);
    }

    // DamageEntryTests.RegisterDamage_OrdersLeastRecentToMostRecent
    #[test]
    fn register_damage_orders_least_recent_to_most_recent() {
        let mut w = World::new();
        let (victim, a, b) = three(&mut w);
        dmg::register_damage(&mut w, victim, 10, Some(a));
        dmg::register_damage(&mut w, victim, 20, Some(b));
        dmg::register_damage(&mut w, victim, 5, Some(a)); // a becomes most recent again

        assert_eq!(damagers(&mut w, victim), vec![b, a]);
        let i = dmg::find_damage_entry_for(&mut w, victim, a);
        assert_eq!(damage_entries(&mut w, victim)[i.unwrap()].damage_given, 15);
        assert_eq!(dmg::find_most_recent_damager(&mut w, victim, true), Some(a));
        assert_eq!(
            dmg::find_least_recent_damager(&mut w, victim, true),
            Some(b)
        );
    }

    // DamageEntryTests.FindRecent_HonorsAllowSelf
    #[test]
    fn find_recent_honors_allow_self() {
        let mut w = World::new();
        let victim = w.new_mobile(&MOBILE_BASE);
        let a = w.new_mobile(&MOBILE_BASE);
        dmg::register_damage(&mut w, victim, 10, Some(a));
        dmg::register_damage(&mut w, victim, 10, Some(victim)); // self is most recent

        assert_eq!(
            dmg::find_most_recent_damager(&mut w, victim, true),
            Some(victim)
        );
        assert_eq!(
            dmg::find_most_recent_damager(&mut w, victim, false),
            Some(a)
        );
        assert_eq!(
            dmg::find_least_recent_damager(&mut w, victim, false),
            Some(a)
        );
    }

    // DamageEntryTests.FindLeastRecent_HonorsAllowSelf
    #[test]
    fn find_least_recent_honors_allow_self() {
        let mut w = World::new();
        let victim = w.new_mobile(&MOBILE_BASE);
        let a = w.new_mobile(&MOBILE_BASE);
        dmg::register_damage(&mut w, victim, 10, Some(victim)); // self is least recent
        dmg::register_damage(&mut w, victim, 10, Some(a));

        assert_eq!(
            dmg::find_least_recent_damager(&mut w, victim, true),
            Some(victim)
        );
        assert_eq!(
            dmg::find_least_recent_damager(&mut w, victim, false),
            Some(a)
        );
    }

    // DamageEntryTests.FindTotal_PicksByDamage_MostRecentWinsTies
    #[test]
    fn find_total_picks_by_damage_most_recent_wins_ties() {
        let mut w = World::new();
        let victim = w.new_mobile(&MOBILE_BASE);
        let (a, b, c) = three(&mut w);
        dmg::register_damage(&mut w, victim, 30, Some(a));
        dmg::register_damage(&mut w, victim, 30, Some(b)); // ties a; b is more recent
        dmg::register_damage(&mut w, victim, 1, Some(c));

        assert_eq!(dmg::find_most_total_damager(&mut w, victim, true), Some(b));
        assert_eq!(dmg::find_least_total_damager(&mut w, victim, true), Some(c));
    }

    // DamageEntryTests.FindLeastTotal_MostRecentWinsTies
    #[test]
    fn find_least_total_most_recent_wins_ties() {
        let mut w = World::new();
        let victim = w.new_mobile(&MOBILE_BASE);
        let (a, b, c) = three(&mut w);
        dmg::register_damage(&mut w, victim, 30, Some(a));
        dmg::register_damage(&mut w, victim, 5, Some(b));
        dmg::register_damage(&mut w, victim, 5, Some(c)); // ties b for the minimum; c is more recent

        assert_eq!(dmg::find_most_total_damager(&mut w, victim, true), Some(a));
        assert_eq!(dmg::find_least_total_damager(&mut w, victim, true), Some(c));
    }

    // DamageEntryTests.Prune_RemovesExpiredPrefix_KeepsOrder
    #[test]
    fn prune_removes_expired_prefix_keeps_order() {
        let mut w = World::new();
        let victim = w.new_mobile(&MOBILE_BASE);
        let (a, b, _) = three(&mut w);
        w.now = DateTime::MIN + TimeSpan::from_hours(1);
        let start = w.now;

        dmg::register_damage(&mut w, victim, 10, Some(a));

        w.now = start + damage_entry_expire_delay(&w) + TimeSpan::from_secs(1);
        dmg::register_damage(&mut w, victim, 10, Some(b)); // a is now expired, b is live

        assert_eq!(damagers(&mut w, victim), vec![b]);
        assert_eq!(dmg::find_damage_entry_for(&mut w, victim, a), None);
    }

    // DamageEntryTests.Prune_AllExpired_EmptiesList
    #[test]
    fn prune_all_expired_empties_list() {
        let mut w = World::new();
        let victim = w.new_mobile(&MOBILE_BASE);
        let (a, b, _) = three(&mut w);
        w.now = DateTime::MIN + TimeSpan::from_hours(1);
        let start = w.now;

        dmg::register_damage(&mut w, victim, 10, Some(a));
        dmg::register_damage(&mut w, victim, 10, Some(b));

        w.now = start + damage_entry_expire_delay(&w) + TimeSpan::from_secs(1);
        assert_eq!(damage_entries(&mut w, victim).len(), 0);
        assert_eq!(
            dmg::find_most_recent_damage_entry(&mut w, victim, true),
            None
        );
    }

    // DamageEntryTests.ClearDamageEntries_UnlinksEveryNode (the intrusive links are a Vec here)
    #[test]
    fn clear_damage_entries_empties_the_list() {
        let mut w = World::new();
        let (victim, a, b) = three(&mut w);
        dmg::register_damage(&mut w, victim, 10, Some(a));
        dmg::register_damage(&mut w, victim, 10, Some(b));

        dmg::clear_damage_entries(&mut w, victim);

        assert_eq!(damage_entries(&mut w, victim).len(), 0);
    }

    // DamageEntryTests.FullHitPoints_ClearsEntries
    #[test]
    fn full_hit_points_clears_entries() {
        let mut w = World::new();
        let victim = w.new_mobile(&MOBILE_BASE);
        let a = w.new_mobile(&MOBILE_BASE);

        w.mobile_mut(victim).unwrap().str = 50; // HitsMax follows Str for a base Mobile
        crate::mobile::part1::props_b::set_hits(&mut w, victim, 10);
        dmg::register_damage(&mut w, victim, 10, Some(a));
        assert_eq!(damage_entries(&mut w, victim).len(), 1);

        // Also stops the HitsTimer the Hits = 10 write started.
        let max = (MOBILE_BASE.hits_max)(&w, victim);
        crate::mobile::part1::props_b::set_hits(&mut w, victim, max);

        assert_eq!(damage_entries(&mut w, victim).len(), 0);
    }

    // DamageEntryTests.RegisterDamage_AccumulatesResponsibleMaster
    #[test]
    fn register_damage_accumulates_responsible_master() {
        let mut w = World::new();
        let master = w.new_mobile(&MOBILE_BASE); // the first mobile: the pet's damage master
        let victim = w.new_mobile(&MOBILE_BASE);
        let pet = w.new_mobile(&PET);

        dmg::register_damage(&mut w, victim, 10, Some(pet));
        let i = dmg::register_damage(&mut w, victim, 5, Some(pet)).unwrap();

        let entry = &damage_entries(&mut w, victim)[i];
        assert_eq!(entry.damager, pet);
        assert_eq!(entry.damage_given, 15);
        assert_eq!(entry.responsible.len(), 1);
        assert_eq!(entry.responsible[0].damager, master);
        assert_eq!(entry.responsible[0].damage_given, 15);
    }
}
