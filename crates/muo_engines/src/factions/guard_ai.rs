//! The faction guard AI past the movement: `SpellCombo`, `FactionGuardAI`'s bandage,
//! potion, spell and weapon-swap decisions (`IsDamaged`, `IsPoisoned`, `TimeUntilBandage`, `IsAllowed`, `DequipWeapon`,
//! `EquipWeapon`, `StartBandage`, `UseItemByType`, `GetStatMod`, `RandomOffenseSpell`, `FindDispelTarget`, `CanDispel`).
//! Spells are made by class name through the spell registry, as MageAI does. `ComboEntry.Hold` isn't kept: nothing reads
//! it.

use muo_content::creature::{cd, is_creature};
use muo_content::hooks::rnd;
use muo_content::mx;
use muo_content::spells::{registry::new_spell_by_name, spell};
use muo_primitives::{DateTime, Point3D, TimeSpan};
use muo_server::World;
use muo_server::ids::{EntityId, MobileId};
use muo_server::interfaces::{SpellId, WeaponRef};
use muo_server::mobile::part1::props_b;
use muo_server::skills::{SkillName, base as skill_base, value as skill_value};
use muo_server::target::{self, TargetCancelType, TargetFlags, Targeted};

use super::mobiles::{BLESS, CURSE, MAGIC, MELEE, SMART, gd};

const MANA_RESERVE: i32 = 30;

/// `SpellCombo`: the mana it needs and its `(spell class, chance)` entries.
pub struct SpellCombo {
    mana: i32,
    entries: &'static [(&'static str, i32)],
}

pub static SIMPLE: SpellCombo = SpellCombo {
    mana: 50,
    entries: &[
        ("ParalyzeSpell", 20),
        ("ExplosionSpell", 100),
        ("PoisonSpell", 30),
        ("EnergyBoltSpell", 100),
    ],
};

pub static STRONG: SpellCombo = SpellCombo {
    mana: 90,
    entries: &[
        ("ParalyzeSpell", 20),
        ("ExplosionSpell", 50),
        ("PoisonSpell", 30),
        ("ExplosionSpell", 100),
        ("EnergyBoltSpell", 100),
        ("PoisonSpell", 30),
        ("EnergyBoltSpell", 100),
    ],
};

/// The fields `FactionGuardAI` adds to the AI.
#[derive(Default)]
pub struct GuardAiState {
    bandage: bool,
    bandage_start: DateTime,
    combo: Option<&'static SpellCombo>,
    combo_index: i32,
    release_target: DateTime,
}

fn state<T>(w: &mut World, g: MobileId, f: impl FnOnce(&mut GuardAiState) -> T) -> T {
    let a = muo_content::ai::ai_mut(w, g).expect("a guard has an AI");
    if a.ext.get::<GuardAiState>().is_none() {
        a.ext.insert(GuardAiState {
            combo_index: -1,
            ..Default::default()
        });
    }
    f(a.ext.get_mut::<GuardAiState>().unwrap())
}

fn mk(w: &mut World, g: MobileId, class: &str) -> Option<SpellId> {
    new_spell_by_name(w, class, g, None)
}

fn is_a(w: &World, sp: SpellId, class: &str) -> bool {
    spell::spell_type(w, sp).is_some_and(|t| t.full_name.rsplit('.').next() == Some(class))
}

/// `SpellCombo.Process`: the next entry that passes its chance, else the combo ends.
fn process(w: &mut World, g: MobileId, targ: MobileId) -> Option<SpellId> {
    loop {
        let (combo, idx) = state(w, g, |s| {
            s.combo_index += 1;
            (s.combo, s.combo_index)
        });
        let combo = combo?;
        let Some(&(class, chance)) = combo.entries.get(idx as usize) else {
            state(w, g, |s| {
                s.combo = None;
                s.combo_index = -1;
            });
            return None;
        };
        if class == "PoisonSpell" && props_b::poisoned(w, targ) {
            continue;
        }
        if chance > rnd::below(w, 100) {
            return mk(w, g, class);
        }
    }
}

pub fn is_damaged(w: &World, g: MobileId) -> bool {
    mx::hits(w, g) < mx::hits_max(w, g)
}

/// `TimeUntilBandage`: `TimeSpan::MAX` when no bandage is running.
pub fn time_until_bandage(w: &mut World, g: MobileId) -> TimeSpan {
    let (running, start) = state(w, g, |s| (s.bandage, s.bandage_start));
    let delay = muo_items::skill_items::bandage::context_delay(w, g);
    let Some(delay) = delay.filter(|_| running) else {
        state(w, g, |s| s.bandage = false);
        return TimeSpan::MAX;
    };
    let ts = start + delay - w.now;
    if ts < TimeSpan::from_millis_f64(-1000.0) {
        state(w, g, |s| s.bandage = false);
        return TimeSpan::MAX;
    }
    ts.max(TimeSpan::ZERO)
}

pub fn is_allowed(w: &World, g: MobileId, flag: u8) -> bool {
    gd(w, g).is_some_and(|d| d.flags & flag == flag)
}

/// `DequipWeapon`: the wielded weapon (not fists) goes into the pack.
pub fn dequip_weapon(w: &mut World, g: MobileId) -> bool {
    let Some(pack) = props_b::backpack(w, g) else {
        return false;
    };
    let Some(WeaponRef::Item(wi)) = muo_server::mobile::base::weapon(w, g) else {
        return false;
    };
    let on_guard = w
        .item(wi)
        .is_some_and(|i| i.parent == Some(EntityId::Mobile(g)));
    if !on_guard || muo_content::factory::item_is(w, wi, "Server.Items.Fists") {
        return false;
    }
    if let Some(ty) = w.item(pack).map(|i| i.ty) {
        (ty.drop_item)(w, pack, wi);
    }
    true
}

/// `EquipWeapon`.
pub fn equip_weapon(w: &mut World, g: MobileId) -> bool {
    let Some(pack) = props_b::backpack(w, g) else {
        return false;
    };
    muo_content::factory::find_item_by_type(w, pack, "Server.Items.BaseWeapon")
        .is_some_and(|i| muo_server::mobile::base::equip_item(w, g, i))
}

fn start_bandage(w: &mut World, g: MobileId) -> bool {
    state(w, g, |s| s.bandage = false);
    let has = props_b::backpack(w, g)
        .and_then(|p| muo_content::factory::find_item_by_type(w, p, "Server.Items.Bandage"))
        .is_some();
    if !has {
        return false;
    }
    let began = muo_items::skill_items::bandage::begin_heal(w, g, g);
    let now = w.now;
    state(w, g, |s| {
        s.bandage = began;
        s.bandage_start = now;
    });
    began
}

fn use_item_by_type(w: &mut World, g: MobileId, class: &str) -> bool {
    let Some(item) =
        props_b::backpack(w, g).and_then(|p| muo_content::factory::find_item_by_type(w, p, class))
    else {
        return false;
    };
    let requip = dequip_weapon(w, g);
    if let Some(ty) = w.item(item).map(|i| i.ty) {
        (ty.on_double_click)(w, item, g);
    }
    if requip {
        equip_weapon(w, g);
    }
    true
}

/// `GetStatMod(mob, type)`: the offset of the mobile's `[Magic] <stat> Curse` mod, 0 without one.
pub fn get_stat_mod(w: &World, m: MobileId, stat: &str) -> i32 {
    muo_server::mobile::base::get_stat_mod(w, m, &format!("[Magic] {stat} Curse"), false)
        .and_then(|i| w.mobile(m).map(|x| x.stat_mods[i].offset))
        .unwrap_or(0)
}

fn random_offense_spell(w: &mut World, g: MobileId) -> Option<SpellId> {
    let magery = skill_value(w, g, SkillName::Magery);
    let max_circle = (((magery + 20.0) / (100.0 / 7.0)) as i32).max(1);
    let class = match rnd::below(w, max_circle * 2) {
        0 | 1 => "MagicArrowSpell",
        2 | 3 => "HarmSpell",
        4 | 5 => "FireballSpell",
        6 | 7 => "LightningSpell",
        8 => "MindBlastSpell",
        9 => "ParalyzeSpell",
        10 => "EnergyBoltSpell",
        11 => "ExplosionSpell",
        _ => "FlameStrikeSpell",
    };
    mk(w, g, class)
}

/// `CanDispel(m)`.
pub fn can_dispel(w: &mut World, g: MobileId, m: MobileId) -> bool {
    is_creature(w, m)
        && cd(w, m).is_some_and(|d| d.summoned)
        && mx::can_be_harmful_quiet(w, g, m)
        && !muo_content::creature::misc::is_animated_dead(w, m)
}

fn dist(w: &World, g: MobileId, m: MobileId) -> f64 {
    muo_content::creature::misc::distance_sqrt(w, g, m)
}

/// `FindDispelTarget(activeOnly)`.
pub fn find_dispel_target(w: &mut World, g: MobileId, active_only: bool) -> Option<MobileId> {
    let int = w.mobile(g).map_or(0, |m| (m.ty.int)(w, g));
    let auto_dispel =
        muo_content::creature::vtable::ctype(w, g).is_some_and(|c| (c.auto_dispel)(w, g));
    if mx::deleted(w, g) || int < 95 || can_dispel(w, g, g) || auto_dispel {
        return None;
    }
    let comb = mx::combatant(w, g);
    let usable = |w: &World, m: MobileId| {
        !mx::deleted(w, m)
            && mx::alive(w, m)
            && !w.mobile(m).is_some_and(|x| (x.ty.is_dead_bonded_pet)(w, m))
    };
    if active_only {
        let (mut active, mut prio) = (None, 0.0);
        if let Some(c) =
            comb.filter(|&c| usable(w, c) && mx::in_range(w, g, c, 12) && can_dispel(w, g, c))
        {
            active = Some(c);
            prio = dist(w, g, c);
            if prio <= 2.0 {
                return active;
            }
        }
        let others: Vec<MobileId> = w.mobile(g).map_or(vec![], |m| {
            m.aggressed
                .iter()
                .map(|i| i.defender)
                .chain(m.aggressors.iter().map(|i| i.attacker))
                .collect()
        });
        for m in others {
            if Some(m) != comb
                && mx::combatant(w, m) == Some(g)
                && mx::in_range(w, g, m, 12)
                && can_dispel(w, g, m)
            {
                let p = dist(w, g, m);
                if active.is_none() || p < prio {
                    active = Some(m);
                    prio = p;
                    if prio <= 2.0 {
                        return active;
                    }
                }
            }
        }
        return active;
    }
    mx::map(w, g)?;
    let (mut active, mut inactive) = (None, None);
    let (mut act_prio, mut inact_prio) = (0.0, 0.0);
    if let Some(c) = comb.filter(|&c| usable(w, c) && can_dispel(w, g, c)) {
        active = Some(c);
        inactive = Some(c);
        act_prio = dist(w, g, c);
        inact_prio = act_prio;
    }
    for m in mx::mobiles_in_range(w, g, 12) {
        if m != g && can_dispel(w, g, m) {
            let p = dist(w, g, m);
            if inactive.is_none() || p < inact_prio {
                inactive = Some(m);
                inact_prio = p;
            }
            if (comb == Some(m) || mx::combatant(w, m) == Some(g))
                && (active.is_none() || p < act_prio)
            {
                active = Some(m);
                act_prio = p;
            }
        }
    }
    active.or(inactive)
}

/// The start of `Think` after the combatant is settled: releases the held spell target. Returns the dispel target.
pub fn before_movement(
    w: &mut World,
    g: MobileId,
    combatant: Option<MobileId>,
) -> Option<MobileId> {
    let dispel = find_dispel_target(w, g, true);
    let now = w.now;
    let Some(tid) = w.mobile(g).and_then(|m| m.target) else {
        return dispel;
    };
    if state(w, g, |s| s.release_target) == DateTime::MIN {
        state(w, g, |s| {
            s.release_target = now + TimeSpan::from_millis_f64(10_000.0)
        });
    }
    if now <= state(w, g, |s| s.release_target) {
        return dispel;
    }
    let (flags, range, sp) = w
        .targets
        .get(&tid)
        .map_or((TargetFlags::NONE, -1, None), |t| {
            (t.flags, t.range, t.spell)
        });
    if let (true, Some(to_harm)) = (flags.contains(TargetFlags::HARMFUL), dispel.or(combatant)) {
        let can_see = w
            .mobile(g)
            .map(|m| m.ty)
            .is_some_and(|t| (t.can_see_mobile)(w, g, to_harm));
        if mx::map(w, g) == mx::map(w, to_harm)
            && (range < 0 || mx::in_range(w, g, to_harm, range))
            && can_see
            && mx::in_los(w, g, to_harm)
        {
            target::invoke(w, tid, g, Targeted::Entity(EntityId::Mobile(to_harm)));
        } else if sp.is_some_and(|s| is_a(w, s, "DispelSpell")) {
            target::cancel(w, tid, g, TargetCancelType::Canceled);
        }
    } else if flags.contains(TargetFlags::BENEFICIAL) {
        target::invoke(w, tid, g, Targeted::Entity(EntityId::Mobile(g)));
    } else {
        target::cancel(w, tid, g, TargetCancelType::Canceled);
    }
    state(w, g, |s| s.release_target = DateTime::MIN);
    dispel
}

/// A spell that was made but never cast leaves the table.
fn discard(w: &mut World, sp: Option<SpellId>) {
    if let Some(sp) = sp {
        spell::remove_spell(w, sp);
    }
}

/// One spell for a set of stat classes: the single one, or `many` (Bless/Curse) for two or more.
fn stat_spell(w: &mut World, g: MobileId, classes: &[&'static str], many: &str) -> Option<SpellId> {
    match classes.len() {
        0 => None,
        1 => mk(w, g, classes[0]),
        _ => mk(w, g, many),
    }
}

/// The end of `Think`: bandage, cure/heal, dispel, combos, buffs and curses, potions, then casting.
pub fn spells(w: &mut World, g: MobileId, combatant: Option<MobileId>, dispel: Option<MobileId>) {
    if (is_damaged(w, g) || props_b::poisoned(w, g))
        && skill_base(w, g, SkillName::Healing) > 20.0
        && time_until_bandage(w, g) == TimeSpan::MAX
    {
        start_bandage(w, g);
    }
    let cur = w.mobile(g).and_then(|m| m.spell);
    let next_ok = w
        .tick_count
        .wrapping_sub(w.mobile(g).map_or(0, |m| m.next_spell_time))
        >= 0;
    if cur.is_some() || !next_ok {
        if cur.is_some_and(|s| {
            spell::state(w, s) == muo_content::spells::types::SpellState::Sequencing
        }) {
            equip_weapon(w, g);
        }
        return;
    }
    let mut sp: Option<SpellId> = None;
    let (hits, hits_max) = (mx::hits(w, g), mx::hits_max(w, g));
    let mana = w.mobile(g).map_or(0, |m| m.mana);

    if props_b::poisoned(w, g) {
        let lesser = w.mobile(g).and_then(|m| m.poison) == Some(muo_server::poison::PoisonId(0));
        let ts = time_until_bandage(w, g);
        if !lesser
            || ts == TimeSpan::MAX
            || time_until_bandage(w, g) < TimeSpan::from_millis_f64(1500.0)
            || hits_max - hits > rnd::below(w, 250)
        {
            if is_allowed(w, g, BLESS) {
                sp = mk(w, g, "CureSpell");
            } else {
                use_item_by_type(w, g, "Server.Items.BaseCurePotion");
            }
        }
    } else if is_damaged(w, g) && hits_max - hits > rnd::below(w, 200) {
        let home = cd(w, g).map_or(Point3D::ZERO, |d| d.home);
        let loc = mx::location(w, g);
        if is_allowed(w, g, MAGIC)
            && hits * 100 / hits_max.max(1) < 10
            && home != Point3D::ZERO
            && !muo_primitives::utility::in_range(loc.x, loc.y, home.x, home.y, 15)
            && mana >= 11
        {
            let map = mx::map(w, g);
            sp = Some(muo_spells::fourth::recall::new_recall_spell(
                w,
                g,
                Some((home, map)),
                None,
                None,
            ));
        } else if is_allowed(w, g, BLESS) {
            let next_combat = w.mobile(g).map_or(0, |m| m.next_combat_time);
            if mana >= 11 && hits + 30 < hits_max {
                sp = mk(w, g, "GreaterHealSpell");
            } else if hits + 10 < hits_max && (mana < 11 || next_combat - w.tick_count > 2000) {
                sp = mk(w, g, "HealSpell");
            }
        } else if muo_server::mobile::part1::combat::can_begin_action(
            w,
            g,
            "Server.Items.BaseHealPotion",
        ) {
            use_item_by_type(w, g, "Server.Items.BaseHealPotion");
        }
    } else if let Some(d) = dispel
        .filter(|_| is_allowed(w, g, MAGIC) || is_allowed(w, g, BLESS) || is_allowed(w, g, CURSE))
    {
        let paralyzed = w.mobile(d).is_some_and(|m| m.paralyzed);
        sp = if !paralyzed && mana > MANA_RESERVE + 20 && rnd::below(w, 100) < 40 {
            mk(w, g, "ParalyzeSpell")
        } else {
            mk(w, g, "DispelSpell")
        };
    }

    if let Some(c) = combatant {
        if state(w, g, |s| s.combo).is_some() {
            if sp.is_none() {
                sp = process(w, g, c);
            } else {
                state(w, g, |s| {
                    s.combo = None;
                    s.combo_index = -1;
                });
            }
        } else if rnd::below(w, 100) < 20 && is_allowed(w, g, MAGIC) {
            if rnd::below(w, 100) < 80 {
                let combo = if is_allowed(w, g, SMART) {
                    &SIMPLE
                } else {
                    &STRONG
                };
                state(w, g, |s| {
                    s.combo = Some(combo);
                    s.combo_index = -1;
                });
                if mana >= MANA_RESERVE + combo.mana {
                    sp = process(w, g, c);
                } else {
                    state(w, g, |s| s.combo = None);
                    if mana >= MANA_RESERVE + 40 {
                        sp = random_offense_spell(w, g);
                    }
                }
            } else if mana >= MANA_RESERVE + 40 {
                sp = random_offense_spell(w, g);
            }
        }

        if sp.is_none() && rnd::below(w, 100) < 2 && mana >= MANA_RESERVE + 10 {
            let mut types: Vec<&'static str> = vec![];
            if get_stat_mod(w, g, "Str") <= 0 {
                types.push("StrengthSpell");
            }
            if get_stat_mod(w, g, "Dex") <= 0 && is_allowed(w, g, MELEE) {
                types.push("AgilitySpell");
            }
            if get_stat_mod(w, g, "Int") <= 0 && is_allowed(w, g, MAGIC) {
                types.push("CunningSpell");
            }
            if is_allowed(w, g, BLESS) {
                sp = stat_spell(w, g, &types, "BlessSpell");
            } else if let Some(t) = types.first() {
                match *t {
                    "StrengthSpell" => {
                        use_item_by_type(w, g, "Server.Items.BaseStrengthPotion");
                    }
                    "AgilitySpell" => {
                        use_item_by_type(w, g, "Server.Items.BaseAgilityPotion");
                    }
                    _ => {}
                }
            }
        }

        if sp.is_none()
            && rnd::below(w, 100) < 2
            && mana >= MANA_RESERVE + 10
            && is_allowed(w, g, CURSE)
        {
            if !props_b::poisoned(w, c) && rnd::below(w, 100) < 40 {
                sp = mk(w, g, "PoisonSpell");
            } else {
                let mut types: Vec<&'static str> = vec![];
                if get_stat_mod(w, c, "Str") >= 0 {
                    types.push("WeakenSpell");
                }
                if get_stat_mod(w, c, "Dex") >= 0 && is_allowed(w, g, MELEE) {
                    types.push("ClumsySpell");
                }
                if get_stat_mod(w, c, "Int") >= 0 && is_allowed(w, g, MAGIC) {
                    types.push("FeeblemindSpell");
                }
                sp = stat_spell(w, g, &types, "CurseSpell");
            }
        }
    }

    let hits = mx::hits(w, g); // a potion used above may have healed
    if sp.is_some() && hits_max - hits + 10 > rnd::below(w, 100) {
        let mut potion = match sp {
            Some(s) if is_a(w, s, "GreaterHealSpell") => Some("Server.Items.BaseHealPotion"),
            Some(s) if is_a(w, s, "CureSpell") => Some("Server.Items.BaseCurePotion"),
            Some(s) if is_a(w, s, "StrengthSpell") => Some("Server.Items.BaseStrengthPotion"),
            Some(s) if is_a(w, s, "AgilitySpell") => Some("Server.Items.BaseAgilityPotion"),
            _ => None,
        };
        if potion == Some("Server.Items.BaseHealPotion")
            && !muo_server::mobile::part1::combat::can_begin_action(
                w,
                g,
                "Server.Items.BaseHealPotion",
            )
        {
            potion = None;
        }
        if let Some(p) = potion
            && w.mobile(g).is_some_and(|m| m.target.is_none())
            && use_item_by_type(w, g, p)
        {
            let greater = sp.is_some_and(|s| is_a(w, s, "GreaterHealSpell"));
            let (hits, hits_max) = (mx::hits(w, g), mx::hits_max(w, g));
            if !greater {
                discard(w, sp.take());
            } else if hits + 30 > hits_max && hits + 10 < hits_max {
                discard(w, sp.take());
                sp = mk(w, g, "HealSpell");
            }
        }
    } else if sp.is_none()
        && w.mobile(g)
            .is_some_and(|m| m.stam < (m.ty.stam_max)(w, g) / 3)
        && is_allowed(w, g, MELEE)
    {
        use_item_by_type(w, g, "Server.Items.BaseRefreshPotion");
    }

    if !sp.is_some_and(|s| spell::cast(w, s)) {
        discard(w, sp);
        equip_weapon(w, g);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spell_combos_match_the_expected_tables() {
        assert_eq!((SIMPLE.mana, SIMPLE.entries.len()), (50, 4));
        assert_eq!((STRONG.mana, STRONG.entries.len()), (90, 7));
        assert_eq!(STRONG.entries[3], ("ExplosionSpell", 100));
        assert_eq!(STRONG.entries[1], ("ExplosionSpell", 50));
    }
}
