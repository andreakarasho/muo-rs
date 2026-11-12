//! What a mobile sends to the clients around it (sounds, remove/incoming packets, `SendEverything`,
//! `ClearScreen`), `UpdateRegion`, `GetDirectionTo`, and the `Effects` wrappers (`MovingEffect`, `FixedEffect`,
//! `FixedParticles`, `BoltEffect`..).

use muo_primitives::{Direction, Point2D, Point3D, Rectangle2D, Serial};

use super::deps;
use crate::effects::{self as fx, EffectLayer};
use crate::ids::{EntityId, MobileId};
use crate::map::enumerate;
use crate::mobile::Healthbar;
use crate::mobile::part1::combat::in_update_range;
use crate::net_state::NetStateId;
use crate::packets::{entity, mobile as mpk, player};
use crate::world::World;

/// `m_NetState.SendSoundEffect(soundID, this)`.
pub fn send_sound(w: &mut World, id: MobileId, sound_id: i32) {
    if let Some(m) = w.mobile(id)
        && let Some(ns) = m.net_state
    {
        let loc = m.location;
        crate::packets::effect::send_sound_effect(w, ns, sound_id, loc);
    }
}

pub fn send_sound_at(w: &mut World, id: MobileId, sound_id: i32, p: Point3D) {
    if let Some(ns) = w.mobile(id).and_then(|m| m.net_state) {
        crate::packets::effect::send_sound_effect(w, ns, sound_id, p);
    }
}

/// To the clients that can see this mobile.
pub fn play_sound(w: &mut World, id: MobileId, sound_id: i32) {
    let Some(m) = w.mobile(id) else { return };
    let (Some(map), loc) = (m.map, m.location) else {
        return;
    };
    if sound_id == -1 {
        return;
    }
    for (ns, viewer) in deps::clients_in_range(w, map, loc) {
        if deps::can_see_mobile(w, viewer, id) {
            crate::packets::effect::send_sound_effect(w, ns, sound_id, loc);
        }
    }
}

pub fn send_remove_packet(w: &mut World, id: MobileId, everyone: bool) {
    let Some(m) = w.mobile(id) else { return };
    let (Some(map), loc, serial, own) = (m.map, m.location, m.serial, m.net_state) else {
        return;
    };
    for (ns, viewer) in deps::clients_in_range(w, map, loc) {
        if Some(ns) != own && (everyone || !deps::can_see_mobile(w, viewer, id)) {
            entity::send_remove_entity(w, ns, serial);
        }
    }
}

/// Tells our client to forget everything around us.
pub fn clear_screen(w: &mut World, id: MobileId) {
    let Some(m) = w.mobile(id) else { return };
    let (Some(map), Some(ns), loc) = (m.map, m.net_state, m.location) else {
        return;
    };
    let range = w.maps.global_max_update_range;

    let mobiles: Vec<MobileId> =
        enumerate::mobiles_in_range(w, Some(map), loc.x, loc.y, range).collect();
    for o in mobiles {
        if o != id
            && let Some(om) = w.mobile(o)
            && in_update_range(loc, om.location)
        {
            let serial = om.serial;
            entity::send_remove_entity(w, ns, serial);
        }
    }

    let items: Vec<_> = enumerate::items_in_range(w, Some(map), loc.x, loc.y, range).collect();
    for item in items {
        let Some(it) = w.item(item) else { continue };
        let (ity, iloc, serial) = (it.ty, it.location, it.serial);
        let r = (ity.get_update_range)(w, item, id);
        if muo_primitives::utility::in_range_3d(loc, iloc, r) {
            entity::send_remove_entity(w, ns, serial);
        }
    }
}

/// Everything in range, to our own client.
pub fn send_everything(w: &mut World, id: MobileId) {
    let Some(m) = w.mobile(id) else { return };
    let (Some(map), Some(ns), loc) = (m.map, m.net_state, m.location) else {
        return;
    };
    let ty = m.ty;
    let range = w.maps.global_max_update_range;

    let mobiles: Vec<MobileId> =
        enumerate::mobiles_in_range(w, Some(map), loc.x, loc.y, range).collect();
    for o in mobiles {
        let Some(om) = w.mobile(o) else { continue };
        let oloc = om.location;
        if (ty.can_see_mobile)(w, id, o) && in_update_range(loc, oloc) {
            mpk::send_mobile_incoming(w, ns, id, o);
            if w.net_state(ns).is_some_and(|d| d.caps.stygian_abyss()) {
                mpk::send_mobile_healthbar(w, ns, o, Healthbar::Yellow);
                mpk::send_mobile_healthbar(w, ns, o, Healthbar::Poison);
            }
            let oty = w.mobile(o).map(|m| m.ty);
            if let Some(oty) = oty {
                if (oty.is_dead_bonded_pet)(w, o) {
                    mpk::send_bonded_status(w, ns, o.0, true);
                }
                (oty.send_opl_packet_to)(w, o, ns);
            }
        }
    }

    let items: Vec<_> = enumerate::items_in_range(w, Some(map), loc.x, loc.y, range).collect();
    for item in items {
        let Some(it) = w.item(item) else { continue };
        let (ity, iloc) = (it.ty, it.location);
        let r = (ity.get_update_range)(w, item, id);
        if (ty.can_see_item)(w, id, item) && muo_primitives::utility::in_range_3d(loc, iloc, r) {
            (ity.send_info_to)(w, item, ns, &[]);
        }
    }

    let bounds = Rectangle2D::new(loc.x - range, loc.y - range, range * 2 + 1, range * 2 + 1);
    let multis: Vec<_> = enumerate::multis_in_bounds(w, Some(map), bounds, false).collect();
    for multi in multis {
        if let Some(mty) = w.item(multi).map(|i| i.ty) {
            (mty.send_info_to)(w, multi, ns, &[]);
        }
    }
}

pub fn update_region(w: &mut World, id: MobileId) {
    let Some(m) = w.mobile(id) else { return };
    if m.deleted {
        return;
    }
    let (loc, map, old) = (m.location, m.map, m.region);
    let ty = m.ty;
    let new_region = deps::region_find(w, loc, map);
    if new_region != old {
        deps::region_on_region_change(w, id, Some(old), Some(new_region));
        if let Some(m) = w.mobile_mut(id) {
            m.region = new_region;
        }
        (ty.on_region_change)(w, id, old, new_region);
    }
}

pub fn send_incoming_packet(w: &mut World, id: MobileId) {
    let Some(m) = w.mobile(id) else { return };
    let (Some(map), loc, serial) = (m.map, m.location, m.serial) else {
        return;
    };
    let ty = m.ty;
    for (ns, viewer) in deps::clients_in_range(w, map, loc) {
        if deps::can_see_mobile(w, viewer, id) {
            send_incoming_to(w, id, ns, viewer);
            let _ = serial;
            let _ = ty;
        }
    }
}

/// The body of the `SendIncomingPacket` / `SetLocation` / `OnHiddenChanged` loops for one client: the incoming
/// packet, both healthbars on Stygian Abyss clients, the bonded status, and the property list.
pub fn send_incoming_to(w: &mut World, id: MobileId, ns: NetStateId, viewer: MobileId) {
    mpk::send_mobile_incoming(w, ns, viewer, id);
    if w.net_state(ns).is_some_and(|d| d.caps.stygian_abyss()) {
        mpk::send_mobile_healthbar(w, ns, id, Healthbar::Poison);
        mpk::send_mobile_healthbar(w, ns, id, Healthbar::Yellow);
    }
    let Some(m) = w.mobile(id) else { return };
    let (ty, serial) = (m.ty, m.serial);
    if (ty.is_dead_bonded_pet)(w, id) {
        mpk::send_bonded_status(w, ns, serial, true);
    }
    (ty.send_opl_packet_to)(w, id, ns);
}

pub fn set_direction(w: &mut World, id: MobileId, dir: Direction) {
    if let Some(m) = w.mobile_mut(id) {
        m.direction = dir;
    }
}

pub fn get_direction_to(w: &World, id: MobileId, x: i32, y: i32, run: bool) -> Direction {
    let Some(m) = w.mobile(id) else {
        return Direction::NORTH;
    };
    let dx = m.location.x - x;
    let dy = m.location.y - y;

    let rx = (dx - dy) * 44;
    let ry = (dx + dy) * 44;

    let ax = rx.abs();
    let ay = ry.abs();

    let mut ret = if (ay >> 1) - ax >= 0 {
        if ry > 0 {
            Direction::UP
        } else {
            Direction::DOWN
        }
    } else if (ax >> 1) - ay >= 0 {
        if rx > 0 {
            Direction::LEFT
        } else {
            Direction::RIGHT
        }
    } else if rx >= 0 && ry >= 0 {
        Direction::WEST
    } else if rx >= 0 {
        Direction::SOUTH
    } else if ry < 0 {
        Direction::EAST
    } else {
        Direction::NORTH
    };

    if run {
        ret |= Direction::RUNNING;
    }
    ret
}

/// `GetDirectionTo(IPoint2D p, run = false)`: a null point is `North | (run ? Running : 0)`.
pub fn get_direction_to_point(w: &World, id: MobileId, p: Option<Point2D>, run: bool) -> Direction {
    match p {
        None => {
            Direction::NORTH
                | if run {
                    Direction::RUNNING
                } else {
                    Direction::NORTH
                }
        }
        Some(p) => get_direction_to(w, id, p.x, p.y, run),
    }
}

// ---------- Effects wrappers ----------

fn me(id: MobileId) -> EntityId {
    EntityId::Mobile(id)
}

/// `MovingEffect(to, itemID, speed, duration, fixedDirection, explodes, hue, renderMode)`.
#[allow(clippy::too_many_arguments)]
pub fn moving_effect(
    w: &mut World,
    id: MobileId,
    to: EntityId,
    item_id: i32,
    speed: i32,
    duration: i32,
    fixed_direction: bool,
    explodes: bool,
    hue: i32,
    render_mode: i32,
) {
    fx::send_moving_effect(
        w,
        me(id),
        to,
        item_id,
        speed,
        duration,
        fixed_direction,
        explodes,
        hue,
        render_mode,
    );
}

/// `MovingEffect(to, itemID, speed, duration, fixedDirection, explodes)`.
#[allow(clippy::too_many_arguments)]
pub fn moving_effect_plain(
    w: &mut World,
    id: MobileId,
    to: EntityId,
    item_id: i32,
    speed: i32,
    duration: i32,
    fixed_direction: bool,
    explodes: bool,
) {
    moving_effect(
        w,
        id,
        to,
        item_id,
        speed,
        duration,
        fixed_direction,
        explodes,
        0,
        0,
    );
}

/// `MovingParticles(to, itemID, speed, duration, fixedDirection, explodes, hue, renderMode, effect,
/// explodeEffect, explodeSound, layer, unknown)`.
#[allow(clippy::too_many_arguments)]
pub fn moving_particles_layer(
    w: &mut World,
    id: MobileId,
    to: EntityId,
    item_id: i32,
    speed: i32,
    duration: i32,
    fixed_direction: bool,
    explodes: bool,
    hue: i32,
    render_mode: i32,
    effect: i32,
    explode_effect: i32,
    explode_sound: i32,
    layer: EffectLayer,
    unknown: i32,
) {
    fx::send_moving_particles_layer(
        w,
        me(id),
        to,
        item_id,
        speed,
        duration,
        fixed_direction,
        explodes,
        hue,
        render_mode,
        effect,
        explode_effect,
        explode_sound,
        i32::from(layer.0),
        unknown,
    );
}

/// `MovingParticles(to, itemID, speed, duration, fixedDirection, explodes, hue, renderMode, effect,
/// explodeEffect, explodeSound, unknown)`: layer 255.
#[allow(clippy::too_many_arguments)]
pub fn moving_particles_hue(
    w: &mut World,
    id: MobileId,
    to: EntityId,
    item_id: i32,
    speed: i32,
    duration: i32,
    fixed_direction: bool,
    explodes: bool,
    hue: i32,
    render_mode: i32,
    effect: i32,
    explode_effect: i32,
    explode_sound: i32,
    unknown: i32,
) {
    fx::send_moving_particles_hue(
        w,
        me(id),
        to,
        item_id,
        speed,
        duration,
        fixed_direction,
        explodes,
        hue,
        render_mode,
        effect,
        explode_effect,
        explode_sound,
        unknown,
    );
}

/// `MovingParticles(to, itemID, speed, duration, fixedDirection, explodes, effect, explodeEffect, explodeSound,
/// unknown)`.
#[allow(clippy::too_many_arguments)]
pub fn moving_particles(
    w: &mut World,
    id: MobileId,
    to: EntityId,
    item_id: i32,
    speed: i32,
    duration: i32,
    fixed_direction: bool,
    explodes: bool,
    effect: i32,
    explode_effect: i32,
    explode_sound: i32,
    unknown: i32,
) {
    fx::send_moving_particles(
        w,
        me(id),
        to,
        item_id,
        speed,
        duration,
        fixed_direction,
        explodes,
        effect,
        explode_effect,
        explode_sound,
        unknown,
    );
}

/// `MovingParticles(to, itemID, speed, duration, fixedDirection, explodes, effect, explodeEffect, explodeSound)`.
#[allow(clippy::too_many_arguments)]
pub fn moving_particles_short(
    w: &mut World,
    id: MobileId,
    to: EntityId,
    item_id: i32,
    speed: i32,
    duration: i32,
    fixed_direction: bool,
    explodes: bool,
    effect: i32,
    explode_effect: i32,
    explode_sound: i32,
) {
    fx::send_moving_particles_hue(
        w,
        me(id),
        to,
        item_id,
        speed,
        duration,
        fixed_direction,
        explodes,
        0,
        0,
        effect,
        explode_effect,
        explode_sound,
        0,
    );
}

/// `FixedEffect(itemID, speed, duration, hue, renderMode)`.
pub fn fixed_effect_hue(
    w: &mut World,
    id: MobileId,
    item_id: i32,
    speed: i32,
    duration: i32,
    hue: i32,
    render_mode: i32,
) {
    fx::send_target_effect(w, me(id), item_id, speed, duration, hue, render_mode);
}

/// `FixedEffect(itemID, speed, duration)`.
pub fn fixed_effect(w: &mut World, id: MobileId, item_id: i32, speed: i32, duration: i32) {
    fx::send_target_effect(w, me(id), item_id, speed, duration, 0, 0);
}

/// `FixedParticles(itemID, speed, duration, effect, hue, renderMode, layer, unknown)`.
#[allow(clippy::too_many_arguments)]
pub fn fixed_particles_full(
    w: &mut World,
    id: MobileId,
    item_id: i32,
    speed: i32,
    duration: i32,
    effect: i32,
    hue: i32,
    render_mode: i32,
    layer: EffectLayer,
    unknown: i32,
) {
    fx::send_target_particles_full(
        w,
        me(id),
        item_id,
        speed,
        duration,
        hue,
        render_mode,
        effect,
        layer,
        unknown,
    );
}

/// `FixedParticles(itemID, speed, duration, effect, hue, renderMode, layer)`.
#[allow(clippy::too_many_arguments)]
pub fn fixed_particles_hue(
    w: &mut World,
    id: MobileId,
    item_id: i32,
    speed: i32,
    duration: i32,
    effect: i32,
    hue: i32,
    render_mode: i32,
    layer: EffectLayer,
) {
    fx::send_target_particles_full(
        w,
        me(id),
        item_id,
        speed,
        duration,
        hue,
        render_mode,
        effect,
        layer,
        0,
    );
}

/// `FixedParticles(itemID, speed, duration, effect, layer, unknown)`.
#[allow(clippy::too_many_arguments)]
pub fn fixed_particles_unknown(
    w: &mut World,
    id: MobileId,
    item_id: i32,
    speed: i32,
    duration: i32,
    effect: i32,
    layer: EffectLayer,
    unknown: i32,
) {
    fx::send_target_particles_full(
        w,
        me(id),
        item_id,
        speed,
        duration,
        0,
        0,
        effect,
        layer,
        unknown,
    );
}

/// `FixedParticles(itemID, speed, duration, effect, layer)`.
pub fn fixed_particles(
    w: &mut World,
    id: MobileId,
    item_id: i32,
    speed: i32,
    duration: i32,
    effect: i32,
    layer: EffectLayer,
) {
    fx::send_target_particles(w, me(id), item_id, speed, duration, effect, layer);
}

/// `BoltEffect(hue)`: `Effects.SendBoltEffect(this, true, hue)`.
pub fn bolt_effect(w: &mut World, id: MobileId, hue: i32) {
    fx::send_bolt_effect(w, me(id), true, hue);
}

/// `SendDeathAnimation` helper used by `Kill`: `OutgoingMobilePackets.CreateDeathAnimation` + `Send`.
pub fn send_death_animation(w: &mut World, ns: NetStateId, killed: Serial, corpse: Serial) {
    mpk::send_death_animation(w, ns, killed, corpse);
}

/// `ns.SendDeathStatus()`.
pub fn send_death_status(w: &mut World, ns: NetStateId) {
    player::send_death_status(w, ns);
}

/// `ns.SendDragEffect(..)` shared by `Lift` and `SendDropEffect`.
#[allow(clippy::too_many_arguments)]
pub fn send_drag_effect(
    w: &mut World,
    ns: NetStateId,
    src_serial: Serial,
    src_location: Point3D,
    trg_serial: Serial,
    trg_location: Point3D,
    item_id: i32,
    hue: i32,
    amount: i32,
) {
    player::send_drag_effect(
        w,
        ns,
        src_serial,
        src_location,
        trg_serial,
        trg_location,
        item_id,
        hue,
        amount,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mobile::MOBILE_BASE;

    #[test]
    fn direction_to_matches_the_quadrants() {
        let mut w = World::new();
        let id = w.new_mobile(&MOBILE_BASE);
        w.mobile_mut(id).unwrap().location = Point3D::new(100, 100, 0);
        let d = |x, y| get_direction_to(&w, id, x, y, false);
        assert_eq!(d(100, 90), Direction::NORTH);
        assert_eq!(d(110, 100), Direction::EAST);
        assert_eq!(d(90, 100), Direction::WEST);
        assert_eq!(d(100, 110), Direction::SOUTH);
        assert_eq!(d(95, 95), Direction::UP);
        assert_eq!(d(105, 105), Direction::DOWN);
        assert_eq!(d(95, 105), Direction::LEFT);
        assert_eq!(d(105, 95), Direction::RIGHT);
        assert_eq!(
            get_direction_to(&w, id, 90, 100, true),
            Direction::WEST | Direction::RUNNING
        );
        assert_eq!(
            get_direction_to_point(&w, id, None, true),
            Direction::NORTH | Direction::RUNNING
        );
    }
}
