//! Effect packets.
//!
//! A builder returns its bytes so a loop over clients builds a packet once and reuses them.
//! `IEntity` arguments are `EffectEntity` values.

use muo_primitives::{Point3D, Serial};

use super::Pk;
use crate::effects::{EffectType, ScreenEffectType};
use crate::net_state::NetStateId;
use crate::world::World;

pub const SOUND_PACKET_LENGTH: usize = 12;
pub const PARTICLE_EFFECT_LENGTH: usize = 49;
pub const HUED_EFFECT_LENGTH: usize = 36;
pub const BOLT_EFFECT_LENGTH: usize = 36;

/// The parts of an `IEntity` an effect packet reads.
#[derive(Copy, Clone, Debug)]
pub struct EffectEntity {
    pub serial: Serial,
    pub location: Point3D,
}

pub fn send_sound_effect(w: &mut World, ns: NetStateId, sound_id: i32, target: Point3D) {
    if w.cannot_send_packets(ns) {
        return;
    }
    let bytes = create_sound_effect(sound_id, target);
    w.send(ns, &bytes);
}

pub fn create_sound_effect(sound_id: i32, target: Point3D) -> Vec<u8> {
    let mut p = Pk::new(SOUND_PACKET_LENGTH);
    p.u8(0x54) // Packet ID
        .u8(1) // flags
        .i16(sound_id as i16)
        .i16(0) // volume
        .i16(target.x as i16)
        .i16(target.y as i16)
        .i16(target.z as i16);
    p.finish()
}

#[allow(clippy::too_many_arguments)]
pub fn create_particle_effect(
    kind: EffectType,
    from: Serial,
    to: Serial,
    item_id: i32,
    from_point: Point3D,
    to_point: Point3D,
    speed: i32,
    duration: i32,
    fixed_direction: bool,
    explode: bool,
    hue: i32,
    render_mode: i32,
    effect: i32,
    explode_effect: i32,
    explode_sound: i32,
    serial: Serial,
    layer: i32,
    unknown: i32,
) -> Vec<u8> {
    let mut p = Pk::new(PARTICLE_EFFECT_LENGTH);
    p.u8(0xC7) // Packet ID
        .u8(kind as u8)
        .serial(from)
        .serial(to)
        .i16(item_id as i16)
        .i16(from_point.x as i16)
        .i16(from_point.y as i16)
        .i8(from_point.z as i8)
        .i16(to_point.x as i16)
        .i16(to_point.y as i16)
        .i8(to_point.z as i8)
        .u8(speed as u8)
        .u8(duration as u8)
        .u8(0)
        .u8(0)
        .bool(fixed_direction)
        .bool(explode)
        .i32(hue)
        .i32(render_mode)
        .i16(effect as i16)
        .i16(explode_effect as i16)
        .i16(explode_sound as i16)
        .serial(serial)
        .u8(layer as u8)
        .i16(unknown as i16);
    p.finish()
}

#[allow(clippy::too_many_arguments)]
pub fn create_target_particle_effect(
    e: EffectEntity,
    item_id: i32,
    speed: i32,
    duration: i32,
    hue: i32,
    render_mode: i32,
    effect: i32,
    layer: i32,
    unknown: i32,
) -> Vec<u8> {
    create_particle_effect(
        EffectType::FixedFrom,
        e.serial,
        Serial::ZERO,
        item_id,
        e.location,
        e.location,
        speed,
        duration,
        true,
        false,
        hue,
        render_mode,
        effect,
        1,
        0,
        e.serial,
        layer,
        unknown,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn create_location_particle_effect(
    e: EffectEntity,
    item_id: i32,
    speed: i32,
    duration: i32,
    hue: i32,
    render_mode: i32,
    effect: i32,
    unknown: i32,
) -> Vec<u8> {
    create_particle_effect(
        EffectType::FixedXYZ,
        e.serial,
        Serial::ZERO,
        item_id,
        e.location,
        e.location,
        speed,
        duration,
        true,
        false,
        hue,
        render_mode,
        effect,
        1,
        0,
        e.serial,
        255,
        unknown,
    )
}

/// `layer` is the `EffectLayer` as an int (`SendMovingParticles` passes `(EffectLayer)255`).
#[allow(clippy::too_many_arguments)]
pub fn create_moving_particle_effect(
    from: EffectEntity,
    to: EffectEntity,
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
) -> Vec<u8> {
    create_particle_effect(
        EffectType::Moving,
        from.serial,
        to.serial,
        item_id,
        from.location,
        to.location,
        speed,
        duration,
        fixed_direction,
        explodes,
        hue,
        render_mode,
        effect,
        explode_effect,
        explode_sound,
        Serial::ZERO,
        layer,
        unknown,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn create_hued_effect(
    kind: EffectType,
    from: Serial,
    to: Serial,
    item_id: i32,
    from_point: Point3D,
    to_point: Point3D,
    speed: i32,
    duration: i32,
    fixed_direction: bool,
    explode: bool,
    hue: i32,
    render_mode: i32,
) -> Vec<u8> {
    let mut p = Pk::new(HUED_EFFECT_LENGTH);
    p.u8(0xC0) // Packet ID
        .u8(kind as u8)
        .serial(from)
        .serial(to)
        .i16(item_id as i16)
        .i16(from_point.x as i16)
        .i16(from_point.y as i16)
        .i8(from_point.z as i8)
        .i16(to_point.x as i16)
        .i16(to_point.y as i16)
        .i8(to_point.z as i8)
        .u8(speed as u8)
        .u8(duration as u8)
        .u8(0)
        .u8(0)
        .bool(fixed_direction)
        .bool(explode)
        .i32(hue)
        .i32(render_mode);
    p.finish()
}

pub fn create_target_hued_effect(
    e: EffectEntity,
    item_id: i32,
    speed: i32,
    duration: i32,
    hue: i32,
    render_mode: i32,
) -> Vec<u8> {
    create_hued_effect(
        EffectType::FixedFrom,
        e.serial,
        Serial::ZERO,
        item_id,
        e.location,
        e.location,
        speed,
        duration,
        true,
        false,
        hue,
        render_mode,
    )
}

pub fn create_location_hued_effect(
    p: Point3D,
    item_id: i32,
    speed: i32,
    duration: i32,
    hue: i32,
    render_mode: i32,
) -> Vec<u8> {
    create_hued_effect(
        EffectType::FixedXYZ,
        Serial::ZERO,
        Serial::ZERO,
        item_id,
        p,
        p,
        speed,
        duration,
        true,
        false,
        hue,
        render_mode,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn create_moving_hued_effect(
    from: EffectEntity,
    to: EffectEntity,
    item_id: i32,
    speed: i32,
    duration: i32,
    fixed_direction: bool,
    explodes: bool,
    hue: i32,
    render_mode: i32,
) -> Vec<u8> {
    create_hued_effect(
        EffectType::Moving,
        from.serial,
        to.serial,
        item_id,
        from.location,
        to.location,
        speed,
        duration,
        fixed_direction,
        explodes,
        hue,
        render_mode,
    )
}

/// The overloads taking locations (and optionally serials) instead of entities: pass `Serial::ZERO` for none.
#[allow(clippy::too_many_arguments)]
pub fn create_moving_hued_effect_at(
    from: Serial,
    to: Serial,
    item_id: i32,
    from_location: Point3D,
    to_location: Point3D,
    speed: i32,
    duration: i32,
    fixed_direction: bool,
    explodes: bool,
    hue: i32,
    render_mode: i32,
) -> Vec<u8> {
    create_hued_effect(
        EffectType::Moving,
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
    )
}

pub fn create_bolt_effect(target: EffectEntity, hue: i32) -> Vec<u8> {
    create_hued_effect(
        EffectType::Lightning,
        target.serial,
        Serial::ZERO,
        0,
        target.location,
        target.location,
        0,
        0,
        false,
        false,
        hue,
        0,
    )
}

pub fn send_screen_effect(w: &mut World, ns: NetStateId, kind: ScreenEffectType) {
    if w.cannot_send_packets(ns) {
        return;
    }
    let mut p = Pk::new(28);
    p.u8(0x70) // Packet ID
        .u8(0x4)
        .zeros(8)
        .u16(kind as u16)
        .zeros(16);
    w.send(ns, &p.finish());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net_state::{NetOut, NetStateData};

    fn world() -> (World, NetStateId) {
        let mut w = World::new();
        let ns = NetStateId(1);
        w.net_states.insert(ns, NetStateData::default());
        (w, ns)
    }

    fn sent(w: &World) -> Vec<u8> {
        match &w.net_out[0] {
            NetOut::Send(_, b) => b.clone(),
            _ => panic!("not a send"),
        }
    }

    #[test]
    fn sound_effect() {
        let (mut w, ns) = world();
        send_sound_effect(&mut w, ns, 10, Point3D::new(1000, 10, 5));
        // legacy PlaySound: 0x54, flags 1, sound, volume 0, x, y, z
        assert_eq!(sent(&w), [0x54, 1, 0, 10, 0, 0, 0x03, 0xE8, 0, 10, 0, 5]);
    }

    #[test]
    fn particle_effect() {
        let b = create_particle_effect(
            EffectType::Moving,
            Serial::new(0x1000),
            Serial::new(0x2000),
            0x100,
            Point3D::new(1000, 100, -10),
            Point3D::new(1500, 500, 0),
            3,
            2,
            false,
            false,
            0x1024,
            1,
            3,
            0,
            0,
            Serial::new(0x4000),
            9,
            0,
        );
        let mut e = vec![0xC7, 0];
        e.extend([0, 0, 0x10, 0, 0, 0, 0x20, 0, 0x01, 0x00]);
        e.extend([0x03, 0xE8, 0, 100, 0xF6, 0x05, 0xDC, 0x01, 0xF4, 0]);
        e.extend([3, 2, 0, 0, 0, 0]);
        e.extend([0, 0, 0x10, 0x24, 0, 0, 0, 1, 0, 3, 0, 0, 0, 0]);
        e.extend([0, 0, 0x40, 0, 9, 0, 0]);
        assert_eq!(b.len(), PARTICLE_EFFECT_LENGTH);
        assert_eq!(b, e);
    }

    #[test]
    fn hued_effect() {
        let b = create_hued_effect(
            EffectType::Moving,
            Serial::new(0x1000),
            Serial::new(0x2000),
            0x100,
            Point3D::new(1000, 100, -10),
            Point3D::new(1500, 500, 0),
            3,
            2,
            false,
            false,
            0x1024,
            1,
        );
        let mut e = vec![0xC0, 0];
        e.extend([0, 0, 0x10, 0, 0, 0, 0x20, 0, 0x01, 0x00]);
        e.extend([0x03, 0xE8, 0, 100, 0xF6, 0x05, 0xDC, 0x01, 0xF4, 0]);
        e.extend([3, 2, 0, 0, 0, 0]);
        e.extend([0, 0, 0x10, 0x24, 0, 0, 0, 1]);
        assert_eq!(b.len(), HUED_EFFECT_LENGTH);
        assert_eq!(b, e);
    }

    #[test]
    fn screen_effect() {
        for (t, id) in [
            (ScreenEffectType::DarkFlash, 4u8),
            (ScreenEffectType::FadeInOut, 3),
        ] {
            let (mut w, ns) = world();
            send_screen_effect(&mut w, ns, t);
            let mut e = vec![0x70, 4];
            e.extend([0; 8]);
            e.extend([0, id]);
            e.extend([0; 16]);
            assert_eq!(sent(&w), e);
        }
    }

    #[test]
    fn bolt_effect() {
        let ent = EffectEntity {
            serial: Serial::new(0x1000),
            location: Point3D::new(1000, 100, -10),
        };
        let b = create_bolt_effect(ent, 0x1024);
        let mut e = vec![0xC0, 1, 0, 0, 0x10, 0, 0, 0, 0, 0, 0, 0];
        e.extend([0x03, 0xE8, 0, 100, 0xF6, 0x03, 0xE8, 0, 100, 0xF6]);
        e.extend([0, 0, 0, 0, 0, 0]);
        e.extend([0, 0, 0x10, 0x24, 0, 0, 0, 0]);
        assert_eq!(b.len(), BOLT_EFFECT_LENGTH);
        assert_eq!(b, e);
    }
}
