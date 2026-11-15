//! MLDryad, Satyr, VorpalBunny, SavageShaman (monsters5).

use muo_content::creature::{BASE_CREATURE_C, cd, ctype};
use muo_content::factory;
use muo_content::misc::aos_helpers::{self, DamageSplit};
use muo_content::mx;
use muo_content::player_mobile;
use muo_primitives::{DateTime, TimeSpan};
use muo_server::World;
use muo_server::effects::EffectLayer;
use muo_server::ids::MobileId;
use muo_server::layer::Layer;
use muo_server::mobile::AccessLevel;
use muo_server::mobile::part1::props_b;
use muo_server::mobile::part2::{effects, enumerators, items};
use muo_server::mods::SkillMod;
use muo_server::skills::{self, SkillName};
use muo_server::timer::{self};

use crate::common::*;

fn range_perception(w: &World, id: MobileId) -> i32 {
    cd(w, id).map_or(10, |d| d.range_perception)
}

/// `UndressItem(m, layer)`: a movable item on the layer goes to the pack.
fn undress_item(w: &mut World, m: MobileId, layer: Layer) {
    let Some(item) = items::find_item_on_layer(w, m, layer) else {
        return;
    };
    if w.item(item).is_some_and(|i| i.movable()) {
        items::place_in_backpack(w, m, item);
    }
}

fn undress_torso(w: &mut World, m: MobileId) {
    for l in [
        Layer::OuterTorso,
        Layer::InnerTorso,
        Layer::MiddleTorso,
        Layer::Pants,
        Layer::Shirt,
    ] {
        undress_item(w, m, l);
    }
}

// ---- MLDryad ----

#[derive(Clone, Copy)]
struct DryadState {
    next_peace: DateTime,
    next_undress: DateTime,
}
impl Default for DryadState {
    fn default() -> Self {
        DryadState {
            next_peace: DateTime::MIN,
            next_undress: DateTime::MIN,
        }
    }
}
fn dryad_state(w: &World, id: MobileId) -> DryadState {
    w.mobile(id)
        .and_then(|m| m.ext.get::<DryadState>())
        .copied()
        .unwrap_or_default()
}
fn set_dryad_state(w: &mut World, id: MobileId, s: DryadState) {
    if let Some(m) = w.mobile_mut(id) {
        m.ext.insert(s);
    }
}

pub fn ml_dryad_on_think(w: &mut World, id: MobileId) {
    (BASE_CREATURE_C.on_think)(w, id);
    ml_dryad_area_peace(w, id);
    ml_dryad_area_undress(w, id);
}

/// `IsValidTarget(PlayerMobile m)`.
fn dryad_valid_target(w: &mut World, id: MobileId, m: MobileId) -> bool {
    let Some(peaced) = player_mobile::data(w, m).map(|d| d.peaced_until) else {
        return false;
    };
    peaced < w.now
        && !w.mobile(m).is_some_and(|x| x.hidden)
        && mx::access_level(w, m) == AccessLevel::Player
        && mx::can_be_harmful(w, id, m)
}

pub fn ml_dryad_area_peace(w: &mut World, id: MobileId) {
    let s = dryad_state(w, id);
    if mx::combatant(w, id).is_none()
        || mx::deleted(w, id)
        || !mx::alive(w, id)
        || s.next_peace > w.now
        || rnd::double(w) < 0.9
    {
        return;
    }
    let duration = TimeSpan::from_secs(i64::from(rnd::min_max(w, 20, 80)));
    let range = range_perception(w, id);
    for m in enumerators::get_mobiles_in_range(w, id, range) {
        if player_mobile::data(w, m).is_some() && dryad_valid_target(w, id, m) {
            let until = w.now + duration;
            if let Some(d) = player_mobile::data_mut(w, m) {
                d.peaced_until = until;
            }
            mx::msg(w, m, 1072065); // You gaze upon the dryad's beauty, and forget to continue battling!
            effects::fixed_particles(w, m, 0x376A, 1, 20, 0x7F5, EffectLayer::Waist);
            mx::set_combatant(w, m, None);
        }
    }
    let mut s = dryad_state(w, id);
    s.next_peace = w.now + TimeSpan::from_secs(10);
    set_dryad_state(w, id, s);
    mx::play_sound(w, id, 0x1D3);
}

pub fn ml_dryad_area_undress(w: &mut World, id: MobileId) {
    let s = dryad_state(w, id);
    if mx::combatant(w, id).is_none()
        || mx::deleted(w, id)
        || !mx::alive(w, id)
        || s.next_undress > w.now
        || rnd::double(w) >= 0.005
    {
        return;
    }
    let range = range_perception(w, id);
    for m in enumerators::get_mobiles_in_range(w, id, range) {
        let ok = mx::player(w, m)
            && !w.mobile(m).is_some_and(|x| x.female || x.hidden)
            && mx::access_level(w, m) == AccessLevel::Player
            && mx::can_be_harmful(w, id, m);
        if ok {
            undress_torso(w, m);
            mx::msg(w, m, 1072197); // The dryad's beauty makes your blood race. Your clothing is too confining.
        }
    }
    let mut s = dryad_state(w, id);
    s.next_undress = w.now + TimeSpan::from_secs(60);
    set_dryad_state(w, id, s);
}

// ---- Satyr ----

#[derive(Clone, Copy)]
struct SatyrState {
    next_peace: DateTime,
    next_provoke: DateTime,
    next_suppress: DateTime,
    next_undress: DateTime,
}
impl Default for SatyrState {
    fn default() -> Self {
        SatyrState {
            next_peace: DateTime::MIN,
            next_provoke: DateTime::MIN,
            next_suppress: DateTime::MIN,
            next_undress: DateTime::MIN,
        }
    }
}
fn satyr_state(w: &World, id: MobileId) -> SatyrState {
    w.mobile(id)
        .and_then(|m| m.ext.get::<SatyrState>())
        .copied()
        .unwrap_or_default()
}
fn update_satyr(w: &mut World, id: MobileId, f: impl FnOnce(&mut SatyrState)) {
    let mut s = satyr_state(w, id);
    f(&mut s);
    if let Some(m) = w.mobile_mut(id) {
        m.ext.insert(s);
    }
}

pub fn satyr_on_think(w: &mut World, id: MobileId) {
    (BASE_CREATURE_C.on_think)(w, id);
    for step in 0..4 {
        let target = mx::combatant(w, id);
        match step {
            0 => satyr_peace(w, id, target),
            1 => satyr_undress(w, id, target),
            2 => satyr_suppress(w, id, target),
            _ => satyr_provoke(w, id, target),
        }
    }
}

pub fn satyr_peace(w: &mut World, id: MobileId, target: Option<MobileId>) {
    let Some(target) = target else { return };
    if mx::deleted(w, id)
        || !mx::alive(w, id)
        || satyr_state(w, id).next_peace > w.now
        || rnd::double(w) < 0.9
    {
        return;
    }
    let peaced = player_mobile::data(w, target).map(|d| d.peaced_until);
    if peaced.is_some_and(|p| p < w.now)
        && !w.mobile(target).is_some_and(|x| x.hidden)
        && mx::can_be_harmful(w, id, target)
    {
        let until = w.now + TimeSpan::from_secs(60);
        if let Some(d) = player_mobile::data_mut(w, target) {
            d.peaced_until = until;
        }
        mx::msg(w, target, 500616); // You hear lovely music, and forget to continue battling!
        effects::fixed_particles(w, target, 0x376A, 1, 32, 0x15BD, EffectLayer::Waist);
        mx::set_combatant(w, target, None);
        mx::play_sound(w, id, 0x58D);
    }
    let now = w.now;
    update_satyr(w, id, |s| s.next_peace = now + TimeSpan::from_secs(10));
}

pub fn satyr_suppress(w: &mut World, id: MobileId, target: Option<MobileId>) {
    let Some(target) = target else { return };
    if tables(w).satyr.contains_key(&target)
        || mx::deleted(w, id)
        || !mx::alive(w, id)
        || satyr_state(w, id).next_suppress > w.now
        || rnd::double(w) < 0.9
    {
        return;
    }
    let delay = TimeSpan::from_secs(i64::from(rnd::min_max(w, 20, 80)));
    if !w.mobile(target).is_some_and(|x| x.hidden) && mx::can_be_harmful(w, id, target) {
        mx::msg(w, target, 1072061); // You hear jarring music, suppressing your strength.
        let names: Vec<String> = w
            .resources
            .get::<skills::SkillTable>()
            .map(|t| t.info.iter().map(|i| i.name.clone()).collect())
            .unwrap_or_default();
        for (i, name) in names.iter().enumerate() {
            let Some(skill) = SkillName::from_i32(i as i32) else {
                continue;
            };
            let base = skills::base(w, target, skill);
            let m = SkillMod::new_timed(
                w,
                skill,
                Some(&format!("{name}Satyr")),
                true,
                base * -0.28,
                delay,
            );
            muo_server::mobile::base::add_skill_mod(w, target, m);
        }
        let mut count = (delay.total_seconds() / 1.25).round_ties_even() as i32;
        let h = timer::start(
            w,
            TimeSpan::ZERO,
            TimeSpan::from_millis(1250),
            0,
            move |w| {
                let gone = mx::deleted(w, target) || !mx::alive(w, target);
                let expired = count < 0;
                count -= 1;
                if gone || expired {
                    satyr_suppress_remove(w, Some(target));
                } else {
                    effects::fixed_particles(w, target, 0x376A, 1, 32, 0x15BD, EffectLayer::Waist);
                }
            },
        );
        tables(w).satyr.insert(target, h);
        mx::play_sound(w, id, 0x58C);
    }
    let now = w.now;
    update_satyr(w, id, |s| s.next_suppress = now + TimeSpan::from_secs(10));
}

/// `SuppressRemove(target)`.
pub fn satyr_suppress_remove(w: &mut World, target: Option<MobileId>) {
    let Some(target) = target else { return };
    if let Some(h) = tables(w).satyr.remove(&target)
        && h.running(w)
    {
        timer::stop(w, h);
    }
}

pub fn satyr_undress(w: &mut World, id: MobileId, target: Option<MobileId>) {
    let Some(target) = target else { return };
    if mx::deleted(w, id)
        || !mx::alive(w, id)
        || satyr_state(w, id).next_undress > w.now
        || rnd::double(w) >= 0.005
    {
        return;
    }
    if mx::player(w, target)
        && w.mobile(target).is_some_and(|x| x.female && !x.hidden)
        && mx::can_be_harmful(w, id, target)
    {
        undress_torso(w, target);
        mx::msg(w, target, 1072196); // The satyr's music makes your blood race. Your clothing is too confining.
    }
    let now = w.now;
    update_satyr(w, id, |s| s.next_undress = now + TimeSpan::from_secs(60));
}

pub fn satyr_provoke(w: &mut World, id: MobileId, target: Option<MobileId>) {
    let Some(target) = target else { return };
    if mx::deleted(w, id)
        || !mx::alive(w, id)
        || satyr_state(w, id).next_provoke > w.now
        || rnd::double(w) < 0.95
    {
        return;
    }
    let range = range_perception(w, id);
    for c in enumerators::get_mobiles_in_range(w, id, range) {
        let Some(ct) = ctype(w, c) else { continue };
        let Some(d) = cd(w, c) else { continue };
        let (is_paragon, bard_provoked) = (d.is_paragon, d.bard_provoked);
        if c == id
            || c == target
            || (ct.unprovokable)(w, c)
            || is_paragon
            || bard_provoked
            || mx::access_level(w, c) != AccessLevel::Player
            || !mx::can_be_harmful(w, c, target)
        {
            continue;
        }
        muo_content::creature::pets::provoke(w, c, id, target, true);
        if mx::player(w, target) {
            mx::msg(w, target, 1072062); // You hear angry music, and start to fight.
        }
        mx::play_sound(w, id, 0x58A);
        break;
    }
    let now = w.now;
    update_satyr(w, id, |s| s.next_provoke = now + TimeSpan::from_secs(10));
}

// ---- VorpalBunny ----

/// `DelayBeginTunnel();`: three minutes later the bunny digs back to its lair.
pub fn vorpal_bunny_tunnel(w: &mut World, id: MobileId) {
    timer::start(w, TimeSpan::from_secs(180), TimeSpan::ZERO, 1, move |w| {
        muo_items::talismans::summons::begin_tunnel(w, id)
    });
}

// ---- SavageShaman ----

fn is_savage_paint(w: &World, m: MobileId) -> bool {
    w.mobile(m)
        .is_some_and(|x| x.body_mod.0 == 183 || x.body_mod.0 == 184)
}

pub fn savage_shaman_is_enemy(w: &mut World, id: MobileId, m: MobileId) -> bool {
    if is_savage_paint(w, m) {
        return false;
    }
    (BASE_CREATURE_C.is_enemy)(w, id, m)
}

pub fn savage_shaman_aggressive_action_criminal(
    w: &mut World,
    id: MobileId,
    aggressor: MobileId,
    criminal: bool,
) {
    (muo_content::creature::BASE_CREATURE.aggressive_action_criminal)(w, id, aggressor, criminal);
    if is_savage_paint(w, aggressor) {
        aos_helpers::damage(
            w,
            aggressor,
            None,
            50,
            DamageSplit {
                phys: 0,
                fire: 100,
                cold: 0,
                pois: 0,
                nrgy: 0,
            },
            Default::default(),
        );
        props_b::set_body_mod(w, aggressor, muo_primitives::Body(0));
        props_b::set_hue_mod(w, aggressor, -1);
        effects::fixed_particles(w, aggressor, 0x36BD, 20, 10, 5044, EffectLayer::Head);
        mx::play_sound(w, aggressor, 0x307);
        mx::msg(w, aggressor, 1040008); // Your skin is scorched as the tribal paint burns away!
        if player_mobile::data(w, aggressor).is_some() {
            player_mobile::set_savage_paint_expiration(w, aggressor, TimeSpan::ZERO);
        }
    }
}

pub fn savage_shaman_alter_melee_damage_to(
    w: &mut World,
    _id: MobileId,
    to: MobileId,
    damage: &mut i32,
) {
    const BIG: &[&str] = &[
        "Dragon",
        "WhiteWyrm",
        "SwampDragon",
        "Drake",
        "Nightmare",
        "Hiryu",
        "LesserHiryu",
        "Daemon",
    ];
    if BIG
        .iter()
        .any(|n| factory::mobile_is(w, to, &format!("Server.Mobiles.{n}")))
    {
        *damage *= 3;
    }
}

pub fn savage_shaman_on_got_melee_attack(
    w: &mut World,
    id: MobileId,
    attacker: MobileId,
    damage: i32,
) {
    (BASE_CREATURE_C.on_got_melee_attack)(w, id, attacker, damage);
    if rnd::double(w) < 0.1 {
        savage_shaman_begin_savage_dance(w, id);
    }
}

fn dance_next_move(w: &mut World, id: MobileId) {
    let t = w.tick_count + 1000;
    muo_content::ai::set_next_move(w, id, t);
}

pub fn savage_shaman_begin_savage_dance(w: &mut World, id: MobileId) {
    if mx::map(w, id).is_none() {
        return;
    }
    let queue: Vec<MobileId> = enumerators::get_mobiles_in_range(w, id, 8)
        .into_iter()
        .filter(|&m| m != id && factory::mobile_is(w, m, "Server.Mobiles.SavageShaman"))
        .collect();
    mx::animate(w, id, 111, 5, 1, true, false, 0); // Do a little dance...
    dance_next_move(w, id);
    if queue.len() < 3 {
        return;
    }
    for dancer in queue {
        mx::animate(w, dancer, 111, 5, 1, true, false, 0); // Get down tonight...
        dance_next_move(w, dancer);
    }
    timer::start(w, TimeSpan::from_secs(1), TimeSpan::ZERO, 1, move |w| {
        savage_shaman_end_savage_dance(w, id)
    });
}

fn can_hurt(w: &mut World, id: MobileId, m: MobileId, friendly: bool) -> bool {
    !friendly
        && mx::can_be_harmful(w, id, m)
        && (!w.mobile(m).is_some_and(|x| x.hidden) || mx::access_level(w, m) == AccessLevel::Player)
}

pub fn savage_shaman_end_savage_dance(w: &mut World, id: MobileId) {
    if mx::deleted(w, id) {
        return;
    }
    let kind = rnd::below(w, 3);
    let mut queue = vec![];
    for m in enumerators::get_mobiles_in_range(w, id, 8) {
        let friendly = ["Savage", "SavageRider", "SavageShaman", "SavageRidgeback"]
            .iter()
            .any(|n| factory::mobile_is(w, m, &format!("Server.Mobiles.{n}")));
        let apply = match kind {
            0 => {
                friendly
                    && !muo_server::mobile::part1::props_b::poisoned(w, m)
                    && !muo_content::items::weapon_ability::is_wounded(w, m)
                    && muo_server::mobile::base::can_be_beneficial(w, id, m)
            }
            _ => can_hurt(w, id, m, friendly),
        };
        if apply {
            queue.push(m);
        }
    }
    for m in queue {
        match kind {
            0 => {
                // DoGreaterHeal: (40% of magery) + (1-10)
                muo_server::mobile::base::do_beneficial(w, id, m);
                let mut to_heal = (skills::value(w, id, SkillName::Magery) * 0.4) as i32;
                to_heal += rnd::min_max(w, 1, 10);
                muo_server::mobile::part2::damage::heal_from(w, m, to_heal, Some(id), true);
                effects::fixed_particles(w, m, 0x376A, 9, 32, 5030, EffectLayer::Waist);
                mx::play_sound(w, m, 0x202);
            }
            1 => {
                muo_server::mobile::base::do_harmful(w, id, m, false);
                let damage = if w.core_aos() {
                    let base = 6 + (skills::value(w, id, SkillName::EvalInt) / 5.0) as i32;
                    rnd::min_max(w, base, base + 3)
                } else {
                    rnd::min_max(w, 12, 20)
                };
                effects::bolt_effect(w, m, 0);
                muo_content::spells::helper::damage_aos(
                    w,
                    None,
                    TimeSpan::from_millis(250),
                    m,
                    Some(id),
                    f64::from(damage),
                    muo_content::spells::helper::SpellDamageSplit {
                        nrgy: 100,
                        ..Default::default()
                    },
                );
            }
            _ => {
                muo_server::mobile::base::do_harmful(w, id, m, false);
                muo_server::mobile::part2::deps::spell_on_caster_hurt(w, m);
                muo_server::mobile::base::set_paralyzed(w, m, false);
                let mut total = skills::value(w, id, SkillName::Magery)
                    + skills::value(w, id, SkillName::Poisoning);
                let (a, b) = (mx::location(w, id), mx::location(w, m));
                let (dx, dy) = (f64::from(a.x - b.x), f64::from(a.y - b.y));
                let dist = (dx * dx + dy * dy).sqrt();
                if dist >= 3.0 {
                    total -= (dist - 3.0) * 10.0;
                }
                let level = if total >= 200.0 {
                    if rnd::below(w, 10) == 0 { 3 } else { 2 }
                } else if total > 170.0 {
                    2
                } else if total > 130.0 {
                    1
                } else {
                    0
                };
                if let Some(p) = muo_content::misc::poison_kinds::get_poison_by_level(w, level) {
                    muo_server::mobile::base::apply_poison(w, m, Some(id), p);
                }
                effects::fixed_particles(w, m, 0x374A, 10, 15, 5021, EffectLayer::Waist);
                mx::play_sound(w, m, 0x474);
            }
        }
    }
}
