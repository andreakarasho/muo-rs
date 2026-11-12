//! Effect packets sent to the clients in range of a point.
//!
//! Entity arguments are `EntityId`s resolved against the world; `Map` is `Option<MapId>` (null map sends
//! nothing). Default arguments of the overloads (`speed = 10`, `hue = 0`, ...) are passed explicitly.

use muo_primitives::{Point3D, Serial};

use crate::ids::{EntityId, ItemId, MobileId};
use crate::map::MapId;
use crate::net_state::NetStateId;
use crate::packets::effect::{self as fx, EffectEntity};
use crate::world::World;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum EffectType {
    Moving,
    Lightning,
    FixedXYZ,
    FixedFrom,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u16)]
pub enum ScreenEffectType {
    FadeOut = 0x00,
    FadeIn = 0x01,
    LightFlash = 0x02,
    FadeInOut = 0x03,
    DarkFlash = 0x04,
}

/// `EffectLayer`: any byte is a value (`(EffectLayer)255` is "none").
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct EffectLayer(pub u8);

#[allow(non_upper_case_globals)]
impl EffectLayer {
    pub const Head: Self = Self(0);
    pub const RightHand: Self = Self(1);
    pub const LeftHand: Self = Self(2);
    pub const Waist: Self = Self(3);
    pub const LeftFoot: Self = Self(4);
    pub const RightFoot: Self = Self(5);
    pub const CenterFeet: Self = Self(7);
    /// `(EffectLayer)255`.
    pub const None: Self = Self(255);
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum ParticleSupportType {
    Full,
    #[default]
    Detect,
    None,
}

/// `Effects.ParticleSupportType` (a static): kept in `w.resources`, defaulting to `Detect`.
#[derive(Default)]
struct ParticleSupport(ParticleSupportType);

pub fn particle_support_type(w: &World) -> ParticleSupportType {
    w.resources
        .get::<ParticleSupport>()
        .map_or(ParticleSupportType::Detect, |p| p.0)
}

pub fn set_particle_support_type(w: &mut World, v: ParticleSupportType) {
    w.resources.get_mut::<ParticleSupport>().0 = v;
}

/// `Map.GetClientsInRange(p, range)` collected (the callers send, which needs the world mutably). `None` is the
/// overload without a range: `Core.GlobalMaxUpdateRange`.
pub fn clients_in_range(w: &World, map: MapId, p: Point3D, range: Option<i32>) -> Vec<NetStateId> {
    let range = range.unwrap_or(w.maps.global_max_update_range);
    crate::map::enumerate::clients_in_range(w, Some(map), p.x, p.y, range).collect()
}

/// `Effects.SendParticlesTo(state)`.
pub fn send_particles_to(w: &World, ns: NetStateId) -> bool {
    let support = particle_support_type(w);
    support == ParticleSupportType::Full
        || support == ParticleSupportType::Detect
            && w.net_state(ns)
                .is_some_and(|d| d.caps.is_uotd_client() || d.caps.is_enhanced_client())
}

/// A bare `Entity`: a point with a serial (usually `Serial.Zero`), location and map; not in
/// any world list, so `ProcessDelta` and the rest do nothing.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct PointEntity {
    pub serial: Serial,
    pub location: Point3D,
    pub map: Option<MapId>,
}

/// An `IEntity` an effect can be sent from or to: an item or mobile, or an `Entity`.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Targeted {
    Entity(EntityId),
    Point(PointEntity),
}

impl Targeted {
    /// `new Entity(Serial.Zero, loc, map)`.
    pub fn point(location: Point3D, map: Option<MapId>) -> Self {
        Self::Point(PointEntity {
            serial: Serial::ZERO,
            location,
            map,
        })
    }
}

impl From<EntityId> for Targeted {
    fn from(e: EntityId) -> Self {
        Self::Entity(e)
    }
}

impl From<ItemId> for Targeted {
    fn from(e: ItemId) -> Self {
        Self::Entity(e.into())
    }
}

impl From<MobileId> for Targeted {
    fn from(e: MobileId) -> Self {
        Self::Entity(e.into())
    }
}

/// What the effect packets read of an `IEntity`: serial, location, map.
fn entity(w: &World, e: impl Into<Targeted>) -> Option<(EffectEntity, Option<MapId>)> {
    let ent = |serial, location| EffectEntity { serial, location };
    match e.into() {
        Targeted::Point(p) => Some((ent(p.serial, p.location), p.map)),
        Targeted::Entity(EntityId::Mobile(id)) => {
            w.mobile(id).map(|m| (ent(m.serial, m.location), m.map))
        }
        Targeted::Entity(EntityId::Item(id)) => {
            w.item(id).map(|i| (ent(i.serial, i.location), i.map))
        }
    }
}

/// `IEntity.ProcessDelta()`.
fn process_delta(w: &mut World, e: impl Into<Targeted>) {
    match e.into() {
        Targeted::Point(_) => {}
        Targeted::Entity(EntityId::Mobile(id)) => crate::mobile::base::process_delta(w, id),
        Targeted::Entity(EntityId::Item(id)) => crate::item::base::process_delta(w, id),
    }
}

/// `(e as Mobile)?.ProcessDelta()`.
fn process_delta_mobile(w: &mut World, e: impl Into<Targeted>) {
    if let Targeted::Entity(EntityId::Mobile(id)) = e.into() {
        crate::mobile::base::process_delta(w, id);
    }
}
/// `state.Mobile.ProcessDelta()`. A client without a mobile is skipped.
fn process_delta_of(w: &mut World, ns: NetStateId) {
    if let Some(m) = w.net_mobile(ns) {
        crate::mobile::base::process_delta(w, m);
    }
}

pub fn play_sound_entity(w: &mut World, e: impl Into<Targeted> + Copy, sound_id: i32) {
    if let Some((ent, map)) = entity(w, e) {
        play_sound(w, ent.location, map, sound_id);
    }
}

pub fn play_sound(w: &mut World, p: Point3D, map: Option<MapId>, sound_id: i32) {
    if sound_id <= -1 {
        return;
    }
    if let Some(map) = map {
        for ns in clients_in_range(w, map, p, None) {
            process_delta_of(w, ns);
            w.send(ns, &fx::create_sound_effect(sound_id, p));
        }
    }
}

pub fn send_bolt_effect(w: &mut World, e: impl Into<Targeted> + Copy, sound: bool, hue: i32) {
    let Some((ent, Some(map))) = entity(w, e) else {
        return;
    };

    process_delta(w, e);

    for ns in clients_in_range(w, map, ent.location, None) {
        let sees = w.net_mobile(ns).is_some_and(|m| match e.into() {
            Targeted::Entity(id) => crate::mobile::base::can_see_object(w, m, Some(id)),
            Targeted::Point(_) => true, // CanSee(object) is true for anything but an Item/Mobile
        });
        if sees {
            if send_particles_to(w, ns) {
                let pre = fx::create_target_particle_effect(ent, 0, 10, 5, 0, 0, 5031, 3, 0);
                w.send(ns, &pre);
            }

            w.send(ns, &fx::create_bolt_effect(ent, hue));

            if sound {
                w.send(ns, &fx::create_sound_effect(0x29, ent.location));
            }
        }
    }
}

pub fn send_location_effect_entity(
    w: &mut World,
    e: impl Into<Targeted> + Copy,
    item_id: i32,
    duration: i32,
    speed: i32,
    hue: i32,
    render_mode: i32,
) {
    if let Some((ent, map)) = entity(w, e) {
        send_location_effect(
            w,
            ent.location,
            map,
            item_id,
            duration,
            speed,
            hue,
            render_mode,
        );
    }
}

#[allow(clippy::too_many_arguments)]
pub fn send_location_effect(
    w: &mut World,
    p: Point3D,
    map: Option<MapId>,
    item_id: i32,
    duration: i32,
    speed: i32,
    hue: i32,
    render_mode: i32,
) {
    let effect = fx::create_location_hued_effect(p, item_id, speed, duration, hue, render_mode);
    send_packet(w, p, map, &effect);
}

pub fn send_location_particles(
    w: &mut World,
    e: impl Into<Targeted> + Copy,
    item_id: i32,
    speed: i32,
    duration: i32,
    effect: i32,
) {
    send_location_particles_full(w, e, item_id, speed, duration, 0, 0, effect, 0);
}

pub fn send_location_particles_unknown(
    w: &mut World,
    e: impl Into<Targeted> + Copy,
    item_id: i32,
    speed: i32,
    duration: i32,
    effect: i32,
    unknown: i32,
) {
    send_location_particles_full(w, e, item_id, speed, duration, 0, 0, effect, unknown);
}

#[allow(clippy::too_many_arguments)]
pub fn send_location_particles_full(
    w: &mut World,
    e: impl Into<Targeted> + Copy,
    item_id: i32,
    speed: i32,
    duration: i32,
    hue: i32,
    render_mode: i32,
    effect: i32,
    unknown: i32,
) {
    let Some((ent, Some(map))) = entity(w, e) else {
        return;
    };

    for ns in clients_in_range(w, map, ent.location, None) {
        process_delta_of(w, ns);

        if send_particles_to(w, ns) {
            let b = fx::create_location_particle_effect(
                ent,
                item_id,
                speed,
                duration,
                hue,
                render_mode,
                effect,
                unknown,
            );
            w.send(ns, &b);
        } else if item_id != 0 {
            let b = fx::create_location_hued_effect(
                ent.location,
                item_id,
                speed,
                duration,
                hue,
                render_mode,
            );
            w.send(ns, &b);
        }
    }
}

pub fn send_target_effect(
    w: &mut World,
    target: impl Into<Targeted> + Copy,
    item_id: i32,
    speed: i32,
    duration: i32,
    hue: i32,
    render_mode: i32,
) {
    process_delta_mobile(w, target);

    let Some((ent, map)) = entity(w, target) else {
        return;
    };
    let effect = fx::create_target_hued_effect(ent, item_id, speed, duration, hue, render_mode);
    send_packet(w, ent.location, map, &effect);
}

pub fn send_target_particles(
    w: &mut World,
    target: impl Into<Targeted> + Copy,
    item_id: i32,
    speed: i32,
    duration: i32,
    effect: i32,
    layer: EffectLayer,
) {
    send_target_particles_full(w, target, item_id, speed, duration, 0, 0, effect, layer, 0);
}

#[allow(clippy::too_many_arguments)]
pub fn send_target_particles_full(
    w: &mut World,
    target: impl Into<Targeted> + Copy,
    item_id: i32,
    speed: i32,
    duration: i32,
    hue: i32,
    render_mode: i32,
    effect: i32,
    layer: EffectLayer,
    unknown: i32,
) {
    process_delta_mobile(w, target);

    let Some((ent, Some(map))) = entity(w, target) else {
        return;
    };

    for ns in clients_in_range(w, map, ent.location, None) {
        process_delta_of(w, ns);

        if send_particles_to(w, ns) {
            let b = fx::create_target_particle_effect(
                ent,
                item_id,
                speed,
                duration,
                hue,
                render_mode,
                effect,
                i32::from(layer.0),
                unknown,
            );
            w.send(ns, &b);
        } else if item_id != 0 {
            let b = fx::create_target_hued_effect(ent, item_id, speed, duration, hue, render_mode);
            w.send(ns, &b);
        }
    }
}

/// `SendMovingEffect(Map, itemID, from, to, ...)`: the origin is `from`, no serials.
#[allow(clippy::too_many_arguments)]
pub fn send_moving_effect_map(
    w: &mut World,
    map: Option<MapId>,
    item_id: i32,
    from: Point3D,
    to: Point3D,
    speed: i32,
    duration: i32,
    fixed_direction: bool,
    explodes: bool,
    hue: i32,
    render_mode: i32,
) {
    send_moving_effect_serials(
        w,
        Serial::ZERO,
        Serial::ZERO,
        from,
        map,
        item_id,
        from,
        to,
        speed,
        duration,
        fixed_direction,
        explodes,
        hue,
        render_mode,
    );
}

/// `SendMovingEffect(Point3D origin, Map, itemID, from, to, ...)`.
#[allow(clippy::too_many_arguments)]
pub fn send_moving_effect_origin(
    w: &mut World,
    origin: Point3D,
    map: Option<MapId>,
    item_id: i32,
    from: Point3D,
    to: Point3D,
    speed: i32,
    duration: i32,
    fixed_direction: bool,
    explodes: bool,
    hue: i32,
    render_mode: i32,
) {
    send_moving_effect_serials(
        w,
        Serial::ZERO,
        Serial::ZERO,
        origin,
        map,
        item_id,
        from,
        to,
        speed,
        duration,
        fixed_direction,
        explodes,
        hue,
        render_mode,
    );
}

/// `SendMovingEffect(IEntity from, Point3D to, ...)`.
#[allow(clippy::too_many_arguments)]
pub fn send_moving_effect_to_point(
    w: &mut World,
    from: impl Into<Targeted> + Copy,
    to: Point3D,
    item_id: i32,
    speed: i32,
    duration: i32,
    fixed_direction: bool,
    explodes: bool,
    hue: i32,
    render_mode: i32,
) {
    process_delta_mobile(w, from);
    let Some((f, map)) = entity(w, from) else {
        return;
    };
    send_moving_effect_serials(
        w,
        f.serial,
        Serial::ZERO,
        f.location,
        map,
        item_id,
        f.location,
        to,
        speed,
        duration,
        fixed_direction,
        explodes,
        hue,
        render_mode,
    );
}

/// `SendMovingEffect(IEntity from, IEntity to, ...)`.
#[allow(clippy::too_many_arguments)]
pub fn send_moving_effect(
    w: &mut World,
    from: impl Into<Targeted> + Copy,
    to: impl Into<Targeted> + Copy,
    item_id: i32,
    speed: i32,
    duration: i32,
    fixed_direction: bool,
    explodes: bool,
    hue: i32,
    render_mode: i32,
) {
    process_delta_mobile(w, from);
    process_delta_mobile(w, to);
    let (Some((f, map)), Some((t, _))) = (entity(w, from), entity(w, to)) else {
        return;
    };
    send_moving_effect_entities(
        w,
        from,
        to,
        f.location,
        map,
        item_id,
        f.location,
        t.location,
        speed,
        duration,
        fixed_direction,
        explodes,
        hue,
        render_mode,
    );
}

/// `SendMovingEffect(IEntity from, IEntity to, Point3D origin, Map, itemID, fromLocation, toLocation, ...)`.
#[allow(clippy::too_many_arguments)]
pub fn send_moving_effect_entities(
    w: &mut World,
    from: impl Into<Targeted> + Copy,
    to: impl Into<Targeted> + Copy,
    origin: Point3D,
    map: Option<MapId>,
    item_id: i32,
    from_location: Point3D,
    to_location: Point3D,
    speed: i32,
    duration: i32,
    fixed_direction: bool,
    explodes: bool,
    hue: i32,
    render_mode: i32,
) {
    process_delta_mobile(w, from);
    process_delta_mobile(w, to);
    let (Some((f, _)), Some((t, _))) = (entity(w, from), entity(w, to)) else {
        return;
    };
    send_moving_effect_serials(
        w,
        f.serial,
        t.serial,
        origin,
        map,
        item_id,
        from_location,
        to_location,
        speed,
        duration,
        fixed_direction,
        explodes,
        hue,
        render_mode,
    );
}

/// `SendMovingEffect(Serial from, Serial to, Point3D origin, Map, ...)`.
#[allow(clippy::too_many_arguments)]
pub fn send_moving_effect_serials(
    w: &mut World,
    from: Serial,
    to: Serial,
    origin: Point3D,
    map: Option<MapId>,
    item_id: i32,
    from_location: Point3D,
    to_location: Point3D,
    speed: i32,
    duration: i32,
    fixed_direction: bool,
    explodes: bool,
    hue: i32,
    render_mode: i32,
) {
    let effect = fx::create_moving_hued_effect_at(
        from,
        to,
        item_id,
        from_location,
        to_location,
        speed,
        duration,
        fixed_direction,
        explodes,
        hue,
        render_mode,
    );
    send_packet(w, origin, map, &effect);
}

/// `SendMovingParticles(from, to, itemID, speed, duration, fixedDirection, explodes, effect, explodeEffect,
/// explodeSound, unknown)`.
#[allow(clippy::too_many_arguments)]
pub fn send_moving_particles(
    w: &mut World,
    from: impl Into<Targeted> + Copy,
    to: impl Into<Targeted> + Copy,
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
    send_moving_particles_hue(
        w,
        from,
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
        unknown,
    );
}

/// The overload with hue and render mode: layer is `(EffectLayer)255`.
#[allow(clippy::too_many_arguments)]
pub fn send_moving_particles_hue(
    w: &mut World,
    from: impl Into<Targeted> + Copy,
    to: impl Into<Targeted> + Copy,
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
    send_moving_particles_layer(
        w,
        from,
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
        255,
        unknown,
    );
}

/// `layer` is an `EffectLayer` as an int (255 is the unnamed "none").
#[allow(clippy::too_many_arguments)]
pub fn send_moving_particles_layer(
    w: &mut World,
    from: impl Into<Targeted> + Copy,
    to: impl Into<Targeted> + Copy,
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
    layer: i32,
    unknown: i32,
) {
    process_delta_mobile(w, from);
    process_delta_mobile(w, to);

    let Some((f, Some(map))) = entity(w, from) else {
        return;
    };
    let Some((t, _)) = entity(w, to) else {
        return;
    };

    for ns in clients_in_range(w, map, f.location, None) {
        process_delta_of(w, ns);

        if send_particles_to(w, ns) {
            let b = fx::create_moving_particle_effect(
                f,
                t,
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
                layer,
                unknown,
            );
            w.send(ns, &b);
        } else if item_id > 1 {
            let b = fx::create_moving_hued_effect(
                f,
                t,
                item_id,
                speed,
                duration,
                fixed_direction,
                explodes,
                hue,
                render_mode,
            );
            w.send(ns, &b);
        }
    }
}

pub fn send_packet(w: &mut World, origin: Point3D, map: Option<MapId>, effect: &[u8]) {
    let Some(map) = map else { return };

    for ns in clients_in_range(w, map, origin, None) {
        process_delta_of(w, ns);
        w.send(ns, effect);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net_state::NetStateData;

    #[test]
    fn particles_only_for_uotd_and_enhanced_unless_full() {
        let mut w = World::new();
        let ns = NetStateId(1);
        w.net_states.insert(ns, NetStateData::default());
        assert!(!send_particles_to(&w, ns));
        set_particle_support_type(&mut w, ParticleSupportType::Full);
        assert!(send_particles_to(&w, ns));
        set_particle_support_type(&mut w, ParticleSupportType::None);
        assert!(!send_particles_to(&w, ns));
    }

    #[test]
    fn no_map_or_negative_sound_sends_nothing() {
        let mut w = World::new();
        play_sound(&mut w, Point3D::ZERO, None, 5);
        play_sound(&mut w, Point3D::ZERO, Some(MapId(0)), -1);
        assert!(w.net_out.is_empty());
    }
}
