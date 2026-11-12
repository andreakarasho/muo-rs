//! ShadowFiend, ShadowKnight, JukaMage: classes with timers or a periodic `OnThink` ability.

use muo_content::creature::{BASE_CREATURE, BASE_CREATURE_C, cd, cd_mut, control};
use muo_content::factory;
use muo_content::misc::aos_helpers;
use muo_content::mx;
use muo_io::{GenericReader, GenericWriter};
use muo_primitives::{DateTime, Point3D, TimeSpan};
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::effect_item::{self, DEFAULT_DURATION};
use muo_server::mobile::part1::{combat, props_b};
use muo_server::mobile::part2::{enumerators, messages};
use muo_server::timer::{self, TimerHandle};

use crate::common::*;

// ---- ShadowFiend ----

/// `m_Timer`: the `UnhideTimer` (one 30 second delay, never repeated).
#[derive(Clone, Copy, Default)]
struct UnhideTimer(TimerHandle);

/// `m_Timer = new UnhideTimer(this); m_Timer.Start();`
pub fn shadow_fiend_start_timer(w: &mut World, id: MobileId) {
    let h = timer::start(w, TimeSpan::from_secs(30), TimeSpan::ZERO, 1, move |w| {
        // OnTick
        if mx::deleted(w, id) {
            return;
        }
        for m in enumerators::get_mobiles_in_range(w, id, 3) {
            let hidden_player = w.mobile(m).is_some_and(|x| x.hidden);
            if m != id
                && mx::player(w, m)
                && hidden_player
                && mx::can_be_harmful(w, id, m)
                && mx::access_level(w, m) == muo_server::mobile::AccessLevel::Player
            {
                props_b::set_hidden(w, m, false);
            }
        }
    });
    if let Some(m) = w.mobile_mut(id) {
        m.ext.insert(UnhideTimer(h));
    }
}

pub fn shadow_fiend_serialize(_w: &World, _id: MobileId, out: &mut dyn GenericWriter) {
    out.write_i32(0);
}
/// `[AfterDeserialization]`: the timer starts again.
pub fn shadow_fiend_deserialize(
    w: &mut World,
    id: MobileId,
    r: &mut dyn GenericReader,
) -> muo_io::Result<()> {
    let _version = r.read_i32()?;
    shadow_fiend_start_timer(w, id);
    Ok(())
}

/// `Backpack?.Destroy(); Effects.SendLocationEffect(Location, Map, 0x376A, 10, 1); return true;`
pub fn shadow_fiend_on_before_death(w: &mut World, id: MobileId) -> bool {
    if let Some(p) = props_b::backpack(w, id) {
        muo_server::item::base::delete(w, p);
    }
    let (loc, map) = (mx::location(w, id), mx::map(w, id));
    muo_server::effects::send_location_effect(w, loc, map, 0x376A, 10, 1, 0, 0);
    true
}

/// `m_Timer?.Stop(); m_Timer = null; base.OnAfterDelete();`
pub fn shadow_fiend_on_after_delete(w: &mut World, id: MobileId) {
    let t = w
        .mobile(id)
        .and_then(|m| m.ext.get::<UnhideTimer>())
        .copied();
    if let Some(mut t) = t {
        t.0.cancel(w);
        if let Some(m) = w.mobile_mut(id) {
            m.ext.insert(UnhideTimer::default());
        }
    }
    (muo_server::mobile::MOBILE_BASE.on_after_delete)(w, id);
}

// ---- ShadowKnight ----

#[derive(Clone, Copy, Default)]
struct ShadowKnightState {
    has_teleported_away: bool,
    sound_timer: TimerHandle,
}

fn knight_state(w: &World, id: MobileId) -> ShadowKnightState {
    w.mobile(id)
        .and_then(|m| m.ext.get::<ShadowKnightState>())
        .copied()
        .unwrap_or_default()
}
fn set_knight_state(w: &mut World, id: MobileId, s: ShadowKnightState) {
    if let Some(m) = w.mobile_mut(id) {
        m.ext.insert(s);
    }
}

/// `base.OnDeath(c);` then the DemonKnight artifact chance.
pub fn shadow_knight_on_death(w: &mut World, id: MobileId, c: Option<ItemId>) {
    (BASE_CREATURE.on_death)(w, id, c);
    let (summoned, no_awards) =
        cd(w, id).map_or((false, false), |d| (d.summoned, d.no_kill_awards));
    if !summoned && !no_awards && crate::classes::demon_knight::check_artifact_chance(w, id) {
        crate::classes::demon_knight::distribute_artifact_from(w, id);
    }
}

/// `base.OnCombatantChange(); if (Hidden && Combatant != null) Combatant = null;`
pub fn shadow_knight_on_combatant_change(w: &mut World, id: MobileId) {
    (BASE_CREATURE.on_combatant_change)(w, id);
    if w.mobile(id).is_some_and(|m| m.hidden) && mx::combatant(w, id).is_some() {
        mx::set_combatant(w, id, None);
    }
}

/// `SendTrackingSound`.
fn send_tracking_sound(w: &mut World, id: MobileId) {
    if w.mobile(id).is_some_and(|m| m.hidden) {
        let (loc, map) = (mx::location(w, id), mx::map(w, id));
        muo_server::effects::play_sound(w, loc, map, 0x2C8);
        mx::set_combatant(w, id, None);
    } else {
        mx::set_frozen(w, id, false);
        let mut s = knight_state(w, id);
        s.sound_timer.cancel(w);
        set_knight_state(w, id, s);
    }
}

/// `OnThink`: once, below half its hits, it teleports away hidden and frozen, giving a tracking sound.
pub fn shadow_knight_on_think(w: &mut World, id: MobileId) {
    let hits = mx::hits(w, id);
    if !knight_state(w, id).has_teleported_away
        && hits < muo_server::mobile::base::hits_max(w, id) / 2
        && let Some(map) = mx::map(w, id)
    {
        for _ in 0..10 {
            let here = mx::location(w, id);
            let dx = rnd::min_max(w, 5, 10) * if rnd::bool(w) { 1 } else { -1 };
            let x = here.x + dx;
            let dy = rnd::min_max(w, 5, 10) * if rnd::bool(w) { 1 } else { -1 };
            let y = here.y + dy;
            let z = here.z;
            if !muo_server::map::fit::can_fit(w, map, x, y, z, 16, false, false, true) {
                continue;
            }
            let from = here;
            let to = Point3D::new(x, y, z);
            if !combat::in_los_point(w, id, to) {
                continue;
            }
            muo_server::mobile::base::set_location(w, id, to, false);
            muo_server::mobile::base::process_delta(w, id);
            props_b::set_hidden(w, id, true);
            mx::set_combatant(w, id, None);
            let e = effect_item::create(w, from, Some(map), DEFAULT_DURATION);
            muo_server::effects::send_location_particles(w, e, 0x3728, 10, 10, 2023);
            let e = effect_item::create(w, to, Some(map), DEFAULT_DURATION);
            muo_server::effects::send_location_particles(w, e, 0x3728, 10, 10, 5023);
            muo_server::effects::play_sound(w, to, Some(map), 0x1FE);
            let h = timer::start(
                w,
                TimeSpan::from_secs(5),
                TimeSpan::from_millis(2500),
                0,
                move |w| send_tracking_sound(w, id),
            );
            set_knight_state(
                w,
                id,
                ShadowKnightState {
                    has_teleported_away: true,
                    sound_timer: h,
                },
            );
            mx::set_frozen(w, id, true);
            break;
        }
    }
    (BASE_CREATURE_C.on_think)(w, id);
}

// ---- JukaMage ----

/// `m_NextAbilityTime` (not saved).
#[derive(Clone, Copy)]
pub(super) struct NextAbility(pub(super) DateTime);

const JUKA_LORD: &str = "Server.Mobiles.JukaLord";

/// `OnThink`: now and then a juka lord in a fight is empowered for 20 seconds.
pub fn juka_mage_on_think(w: &mut World, id: MobileId) {
    let next = w
        .mobile(id)
        .and_then(|m| m.ext.get::<NextAbility>())
        .map_or(DateTime::MIN, |n| n.0);
    if w.now >= next {
        let mut to_buff = None;
        for m in enumerators::get_mobiles_in_range(w, id, 8) {
            if factory::mobile_is(w, m, JUKA_LORD)
                && control::is_friend(w, id, m)
                && mx::combatant(w, m).is_some()
                && muo_server::mobile::base::can_be_beneficial(w, id, m)
                && combat::can_begin_action(w, m, "JukaMage")
                && mx::in_los(w, id, m)
            {
                to_buff = Some(m);
                break;
            }
        }
        if let Some(b) = to_buff {
            if muo_server::mobile::base::can_be_beneficial(w, id, b)
                && combat::begin_action(w, b, "JukaMage")
            {
                let secs = rnd::min_max(w, 30, 60);
                set_next(w, id, w.now + TimeSpan::from_secs(i64::from(secs)));
                messages::say(w, b, true, "Give me the power to destroy my enemies!");
                messages::say(w, id, true, "Fight well my lord!");
                muo_server::mobile::base::do_beneficial(w, id, b);
                // SpellHelper.Turn(this, toBuff)
                let d = mx::get_direction_to(w, id, b);
                mx::set_direction(w, id, d);

                let max_hits = cd(w, b).map_or(-1, |d| d.hits_max_seed);
                let raw_str = w.mobile(b).map_or(0, |m| m.str);
                let raw_dex = w.mobile(b).map_or(0, |m| m.dex);

                let to_scale = max_hits;
                if to_scale > 0 {
                    let add = aos_helpers::scale(to_scale, 75);
                    if let Some(d) = cd_mut(w, b) {
                        d.hits_max_seed += add;
                    }
                    let h = mx::hits(w, b);
                    props_b::set_hits(w, b, h + add);
                }
                let to_scale = raw_str;
                if to_scale > 0 {
                    props_b::set_raw_str(w, b, raw_str + aos_helpers::scale(to_scale, 50));
                }
                let to_scale = raw_dex;
                if to_scale > 0 {
                    props_b::set_raw_dex(w, b, raw_dex + aos_helpers::scale(to_scale, 50));
                    let stam = w.mobile(b).map_or(0, |m| m.stam);
                    props_b::set_stam(w, b, stam + aos_helpers::scale(to_scale, 50));
                }
                // toBuff.Hits = toBuff.Hits; toBuff.Stam = toBuff.Stam;
                let h = mx::hits(w, b);
                props_b::set_hits(w, b, h);
                let s = w.mobile(b).map_or(0, |m| m.stam);
                props_b::set_stam(w, b, s);

                muo_server::mobile::part2::effects::fixed_particles(
                    w,
                    b,
                    0x375A,
                    10,
                    15,
                    5017,
                    muo_server::effects::EffectLayer::Waist,
                );
                mx::play_sound(w, b, 0x1EE);
                timer::start(w, TimeSpan::from_secs(20), TimeSpan::ZERO, 1, move |w| {
                    juka_mage_unbuff(w, b, max_hits, raw_str, raw_dex)
                });
            }
        } else {
            let secs = rnd::min_max(w, 2, 5);
            set_next(w, id, w.now + TimeSpan::from_secs(i64::from(secs)));
        }
    }
    (BASE_CREATURE_C.on_think)(w, id);
}

pub(super) fn set_next(w: &mut World, id: MobileId, at: DateTime) {
    if let Some(m) = w.mobile_mut(id) {
        m.ext.insert(NextAbility(at));
    }
}

fn juka_mage_unbuff(w: &mut World, b: MobileId, hits_max_seed: i32, raw_str: i32, raw_dex: i32) {
    combat::end_action(w, b, "JukaMage");
    if mx::deleted(w, b) {
        return;
    }
    if let Some(d) = cd_mut(w, b) {
        d.hits_max_seed = hits_max_seed;
    }
    props_b::set_raw_str(w, b, raw_str);
    props_b::set_raw_dex(w, b, raw_dex);
    let h = mx::hits(w, b);
    props_b::set_hits(w, b, h);
    let s = w.mobile(b).map_or(0, |m| m.stam);
    props_b::set_stam(w, b, s);
}

/// The constructor's `m_Timer = new UnhideTimer(this); m_Timer.Start();`.
pub fn shadow_fiend_ctor1(w: &mut World, id: MobileId) {
    shadow_fiend_start_timer(w, id);
}

/// `m_NextAbilityTime = Core.Now + TimeSpan.FromSeconds(Utility.RandomMinMax(2, 5));`
pub fn juka_mage_ctor1(w: &mut World, id: MobileId) {
    let secs = rnd::min_max(w, 2, 5);
    set_next(w, id, w.now + TimeSpan::from_secs(i64::from(secs)));
}

// ---- OrcBomber ----

/// `m_NextBomb` and `m_Thrown` (not saved).
#[derive(Clone, Copy)]
struct Bombs {
    next: DateTime,
    thrown: i32,
}

/// `OnActionCombat`: a bomb at the combatant, now and then two in quick succession.
pub fn orc_bomber_on_action_combat(w: &mut World, id: MobileId) {
    let Some(c) = mx::combatant(w, id) else {
        return;
    };
    if mx::deleted(w, c)
        || mx::map(w, c) != mx::map(w, id)
        || !mx::in_range(w, c, id, 12)
        || !mx::can_be_harmful(w, id, c)
        || !mx::in_los(w, id, c)
    {
        return;
    }
    let mut b = w
        .mobile(id)
        .and_then(|m| m.ext.get::<Bombs>())
        .copied()
        .unwrap_or(Bombs {
            next: DateTime::MIN,
            thrown: 0,
        });
    if w.now >= b.next {
        // ThrowBomb(combatant)
        mx::do_harmful(w, id, c);
        muo_server::mobile::part2::effects::moving_particles_layer(
            w,
            id,
            mx::entity(c),
            0x1C19,
            1,
            0,
            false,
            true,
            0,
            0,
            9502,
            6014,
            0x11D,
            muo_server::effects::EffectLayer::Waist,
            0,
        );
        timer::start(w, TimeSpan::from_secs(1), TimeSpan::ZERO, 1, move |w| {
            mx::play_sound(w, c, 0x11D);
            let amount = rnd::min_max(w, 10, 20);
            aos_helpers::damage(
                w,
                c,
                Some(id),
                amount,
                aos_helpers::DamageSplit {
                    phys: 0,
                    fire: 100,
                    cold: 0,
                    pois: 0,
                    nrgy: 0,
                },
                Default::default(),
            );
        });
        b.thrown += 1;
        b.next = if b.thrown % 2 == 1 && rnd::double(w) < 0.75 {
            w.now + TimeSpan::from_secs(3) // 75% chance to quickly throw another bomb
        } else {
            let d = 5.0 + 10.0 * rnd::double(w);
            w.now + TimeSpan::from_secs_f64(d) // 5-15 seconds
        };
        if let Some(m) = w.mobile_mut(id) {
            m.ext.insert(b);
        }
    }
}
