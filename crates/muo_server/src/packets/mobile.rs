//! Mobile packets.
//!
//! Legacy-packet shapes are the same as the `Create*` span builders: `create_x` returns the bytes.

use muo_data::expansion::Expansion;

use super::Pk;
use super::movement::notoriety_compute;
use crate::ids::MobileId;
use crate::layer::Layer;
use crate::mobile::Healthbar;
use crate::net_state::{NetCaps, NetStateId};
use crate::race::Races;
use crate::world::World;
use muo_primitives::Serial;

pub const BONDED_STATUS_PACKET_LENGTH: usize = 11;
pub const DEATH_ANIMATION_PACKET_LENGTH: usize = 13;
pub const MOBILE_MOVING_PACKET_LENGTH: usize = 17;
/// Mobile Moving Packet plus 2 bytes for regular/stygian flags.
pub const MOBILE_MOVING_PACKET_CACHE_BYTE_LENGTH: usize = MOBILE_MOVING_PACKET_LENGTH + 2;

pub const ATTRIBUTE_MAXIMUM: i32 = 100;
pub const MOBILE_STATUS_COMPACT_LENGTH: usize = 43;
pub const MOBILE_STATUS_LENGTH: usize = 70;
pub const MOBILE_STATUS_AOS_LENGTH: usize = 88;
pub const MOBILE_STATUS_ML_LENGTH: usize = 91;
pub const MOBILE_STATUS_HS_LENGTH: usize = 121;

/// `Core.Expansion` and `ExpansionInfo.CoreExpansion.MobileStatusVersion`.
pub fn core_expansion(w: &World) -> (Expansion, i32) {
    (
        w.expansion(),
        w.core_expansion_info()
            .map_or(6, |i| i.mobile_status_version),
    )
}

/// `ns.ExpansionInfo.Id` as an `Expansion`.
pub fn ns_expansion(w: &World, caps: &NetCaps) -> Expansion {
    match w.expansion_table() {
        Some(t) => Expansion::from_index(caps.expansion_info(t).id).unwrap_or(Expansion::None),
        None => w.expansion(),
    }
}

pub fn create_bonded_status(serial: Serial, bonded: bool) -> Vec<u8> {
    let mut pk = Pk::new(BONDED_STATUS_PACKET_LENGTH);
    pk.u8(0xBF); // Packet ID
    pk.u16(11); // Length
    pk.u16(0x19); // Subpacket ID
    pk.u8(0); // Command
    pk.serial(serial);
    pk.bool(bonded);
    pk.finish()
}

pub fn send_bonded_status(w: &mut World, ns: NetStateId, serial: Serial, bonded: bool) {
    if w.cannot_send_packets(ns) {
        return;
    }
    w.send(ns, &create_bonded_status(serial, bonded));
}

pub fn create_death_animation(killed: Serial, corpse: Serial) -> Vec<u8> {
    let mut pk = Pk::new(DEATH_ANIMATION_PACKET_LENGTH);
    pk.u8(0xAF); // Packet ID
    pk.serial(killed);
    pk.serial(corpse);
    pk.i32(0); // ??
    pk.finish()
}

pub fn send_death_animation(w: &mut World, ns: NetStateId, killed: Serial, corpse: Serial) {
    if w.cannot_send_packets(ns) {
        return;
    }
    w.send(ns, &create_death_animation(killed, corpse));
}

/// The packet flags of a mobile for the regular / Stygian Abyss clients.
fn packet_flags(w: &World, id: MobileId, stygian_abyss: bool) -> u8 {
    w.mobile(id)
        .map_or(0, |m| (m.ty.get_packet_flags)(w, id, stygian_abyss) as u8)
}

/// `CreateMobileMoving(buffer, m, noto, packetFlags)` on a fresh buffer.
pub fn create_mobile_moving(w: &World, m: MobileId, noto: i32, packet_flags: u8) -> Vec<u8> {
    let mut cache = MobileMovingCache::default();
    cache.create(w, m, noto, packet_flags);
    cache.0[..MOBILE_MOVING_PACKET_LENGTH].to_vec()
}

/// The caller-owned 19-byte buffer of `SendMobileMovingUsingCache`: the packet plus the regular and Stygian
/// flags. Byte 0 is the packet id once it has been built, and the build is skipped from then on (only the flags
/// and notoriety bytes change), which is how one buffer serves every viewer of a move.
#[derive(Clone, Debug, Default)]
pub struct MobileMovingCache(pub [u8; MOBILE_MOVING_PACKET_CACHE_BYTE_LENGTH]);

impl MobileMovingCache {
    fn create(&mut self, w: &World, m: MobileId, noto: i32, packet_flags: u8) {
        if self.0[0] == 0
            && let Some(mob) = w.mobile(m)
        {
            let loc = mob.location;
            let hue = if mob.solid_hue_override >= 0 {
                mob.solid_hue_override
            } else {
                (mob.ty.hue)(w, m)
            };

            let mut pk = Pk::new(MOBILE_MOVING_PACKET_LENGTH);
            pk.u8(0x77); // Packet ID
            pk.serial(mob.serial);
            pk.i16(crate::mobile::part1::props_b::body(w, m).0 as i16);
            pk.i16(loc.x as i16);
            pk.i16(loc.y as i16);
            pk.i8(loc.z as i8);
            pk.u8(mob.direction.bits());
            pk.i16(hue as i16);
            let head = pk.finish();
            self.0[..head.len()].copy_from_slice(&head);
        }

        self.0[15] = packet_flags;
        self.0[16] = noto as u8;
    }
}

pub fn send_mobile_moving_notoriety(
    w: &mut World,
    ns: NetStateId,
    source: MobileId,
    target: MobileId,
) {
    let noto = notoriety_compute(w, source, target);
    send_mobile_moving(w, ns, target, noto);
}

pub fn send_mobile_moving(w: &mut World, ns: NetStateId, target: MobileId, noto: i32) {
    if w.cannot_send_packets(ns) {
        return;
    }
    let sa = w.net_state(ns).is_some_and(|d| d.caps.stygian_abyss());
    let flags = packet_flags(w, target, sa);
    let bytes = create_mobile_moving(w, target, noto, flags);
    w.send(ns, &bytes);
}

pub fn send_mobile_moving_using_cache_notoriety(
    w: &mut World,
    ns: NetStateId,
    cache: &mut MobileMovingCache,
    source: MobileId,
    target: MobileId,
) {
    let noto = notoriety_compute(w, source, target);
    send_mobile_moving_using_cache(w, ns, cache, target, noto);
}

pub fn send_mobile_moving_using_cache(
    w: &mut World,
    ns: NetStateId,
    cache: &mut MobileMovingCache,
    target: MobileId,
    noto: i32,
) {
    if w.cannot_send_packets(ns) {
        return;
    }

    // Cache the packet flags for regular/stygian if the packet hasn't been built yet
    if cache.0[0] == 0 {
        cache.0[17] = packet_flags(w, target, false);
        cache.0[18] = packet_flags(w, target, true);
    }

    let sa = w.net_state(ns).is_some_and(|d| d.caps.stygian_abyss());
    let flags = if sa { cache.0[18] } else { cache.0[17] };
    cache.create(w, target, noto, flags);
    w.send(ns, &cache.0[..MOBILE_MOVING_PACKET_LENGTH]);
}

/// `WriteAttribute(max, cur, normalize, reverse)`.
fn write_attribute(pk: &mut Pk, max: i32, cur: i32, normalize: bool, reverse: bool) {
    let (a, b) = if normalize && max != 0 {
        let scaled = cur.wrapping_mul(ATTRIBUTE_MAXIMUM).wrapping_div(max);
        if reverse {
            (scaled, ATTRIBUTE_MAXIMUM)
        } else {
            (ATTRIBUTE_MAXIMUM, scaled)
        }
    } else if reverse {
        (cur, max)
    } else {
        (max, cur)
    };
    pk.i16(a as i16);
    pk.i16(b as i16);
}

/// (hits, hits max), (mana, mana max), (stam, stam max) of a mobile; the max values are virtual.
fn hits(w: &World, id: MobileId) -> Option<(Serial, i32, i32)> {
    let m = w.mobile(id)?;
    Some((m.serial, (m.ty.hits_max)(w, id), m.hits))
}
fn mana(w: &World, id: MobileId) -> Option<(Serial, i32, i32)> {
    let m = w.mobile(id)?;
    Some((m.serial, (m.ty.mana_max)(w, id), m.mana))
}
fn stam(w: &World, id: MobileId) -> Option<(Serial, i32, i32)> {
    let m = w.mobile(id)?;
    Some((m.serial, (m.ty.stam_max)(w, id), m.stam))
}

fn create_attribute(packet_id: u8, a: Option<(Serial, i32, i32)>, normalize: bool) -> Vec<u8> {
    let Some((serial, max, cur)) = a else {
        return Vec::new();
    };
    let mut pk = Pk::new(9);
    pk.u8(packet_id);
    pk.serial(serial);
    write_attribute(&mut pk, max, cur, normalize, false);
    pk.finish()
}

pub fn create_mobile_hits(w: &World, m: MobileId, normalize: bool) -> Vec<u8> {
    create_attribute(0xA1, hits(w, m), normalize)
}

pub fn send_mobile_hits(w: &mut World, ns: NetStateId, m: MobileId, normalize: bool) {
    if w.cannot_send_packets(ns) {
        return;
    }
    let bytes = create_mobile_hits(w, m, normalize);
    w.send(ns, &bytes);
}

pub fn create_mobile_mana(w: &World, m: MobileId, normalize: bool) -> Vec<u8> {
    create_attribute(0xA2, mana(w, m), normalize)
}

pub fn send_mobile_mana(w: &mut World, ns: NetStateId, m: MobileId, normalize: bool) {
    if w.cannot_send_packets(ns) {
        return;
    }
    let bytes = create_mobile_mana(w, m, normalize);
    w.send(ns, &bytes);
}

pub fn create_mobile_stam(w: &World, m: MobileId, normalize: bool) -> Vec<u8> {
    create_attribute(0xA3, stam(w, m), normalize)
}

pub fn send_mobile_stam(w: &mut World, ns: NetStateId, m: MobileId, normalize: bool) {
    if w.cannot_send_packets(ns) {
        return;
    }
    let bytes = create_mobile_stam(w, m, normalize);
    w.send(ns, &bytes);
}

pub fn create_mobile_attributes(w: &World, m: MobileId, normalize: bool) -> Vec<u8> {
    let (Some((serial, hits_max, hits)), Some((_, mana_max, mana)), Some((_, stam_max, stam))) =
        (hits(w, m), mana(w, m), stam(w, m))
    else {
        return Vec::new();
    };
    let mut pk = Pk::new(17);
    pk.u8(0x2D); // Packet ID
    pk.serial(serial);
    write_attribute(&mut pk, hits_max, hits, normalize, false);
    write_attribute(&mut pk, mana_max, mana, normalize, false);
    write_attribute(&mut pk, stam_max, stam, normalize, false);
    pk.finish()
}

pub fn send_mobile_attributes(w: &mut World, ns: NetStateId, m: MobileId, normalize: bool) {
    if w.cannot_send_packets(ns) {
        return;
    }
    let bytes = create_mobile_attributes(w, m, normalize);
    w.send(ns, &bytes);
}

pub fn create_mobile_name(w: &World, m: MobileId) -> Vec<u8> {
    let Some(mob) = w.mobile(m) else {
        return Vec::new();
    };
    let name = (mob.ty.name)(w, m);
    let mut pk = Pk::new(37);
    pk.u8(0x98); // Packet ID
    pk.u16(37);
    pk.serial(mob.serial);
    let _ = pk.write_latin1_fixed(name.as_deref().unwrap_or(""), 29);
    pk.u8(0); // Null terminator
    pk.finish()
}

pub fn send_mobile_name(w: &mut World, ns: NetStateId, m: MobileId) {
    if w.cannot_send_packets(ns) {
        return;
    }
    let bytes = create_mobile_name(w, m);
    w.send(ns, &bytes);
}

pub fn create_mobile_animation(
    mobile: Serial,
    action: i32,
    frame_count: i32,
    repeat_count: i32,
    forward: bool,
    repeat: bool,
    delay: i32,
) -> Vec<u8> {
    let mut pk = Pk::new(14);
    pk.u8(0x6E); // Packet ID
    pk.serial(mobile);
    pk.i16(action as i16);
    pk.i16(frame_count as i16);
    pk.i16(repeat_count as i16);
    pk.bool(!forward); // protocol has really "reverse" but I find this more intuitive
    pk.bool(repeat);
    pk.u8(delay as u8);
    pk.finish()
}

#[allow(clippy::too_many_arguments)]
pub fn send_mobile_animation(
    w: &mut World,
    ns: NetStateId,
    mobile: Serial,
    action: i32,
    frame_count: i32,
    repeat_count: i32,
    forward: bool,
    repeat: bool,
    delay: i32,
) {
    if w.cannot_send_packets(ns) {
        return;
    }
    let bytes = create_mobile_animation(
        mobile,
        action,
        frame_count,
        repeat_count,
        forward,
        repeat,
        delay,
    );
    w.send(ns, &bytes);
}

pub fn create_new_mobile_animation(
    mobile: Serial,
    action: i32,
    frame_count: i32,
    delay: i32,
) -> Vec<u8> {
    let mut pk = Pk::new(10);
    pk.u8(0xE2); // Packet ID
    pk.serial(mobile);
    pk.i16(action as i16);
    pk.i16(frame_count as i16);
    pk.u8(delay as u8);
    pk.finish()
}

pub fn send_new_mobile_animation(
    w: &mut World,
    ns: NetStateId,
    mobile: Serial,
    action: i32,
    frame_count: i32,
    delay: i32,
) {
    if w.cannot_send_packets(ns) {
        return;
    }
    w.send(
        ns,
        &create_new_mobile_animation(mobile, action, frame_count, delay),
    );
}

pub fn create_mobile_healthbar_level(serial: Serial, healthbar: Healthbar, level: i32) -> Vec<u8> {
    let mut pk = Pk::new(12);
    pk.u8(0x17); // Packet ID
    pk.u16(12);
    pk.serial(serial);
    pk.i16(1); // Show bar
    pk.i16(healthbar as i16);
    pk.u8(level as u8); // 0 is off for that bar type
    pk.finish()
}

pub fn create_mobile_healthbar(w: &World, m: MobileId, healthbar: Healthbar) -> Vec<u8> {
    let Some(mob) = w.mobile(m) else {
        return Vec::new();
    };
    match healthbar {
        Healthbar::Poison => {
            // `m.Poison?.Level + 1 ?? 0`: the level, not the registry index (Darkglow is index 10, level 0)
            let level = mob.poison.map_or(0, |p| crate::poison::level(w, p) + 1);
            create_mobile_healthbar_level(mob.serial, Healthbar::Poison, level)
        }
        Healthbar::Yellow => create_mobile_healthbar_level(
            mob.serial,
            Healthbar::Yellow,
            i32::from(mob.blessed || mob.yellow_healthbar),
        ),
        _ => {
            tracing::warn!("Packets: Invalid Healthbar {healthbar:?} in CreateMobileHealthbar");
            create_mobile_healthbar_level(mob.serial, Healthbar::Normal, 0)
        }
    }
}

pub fn send_mobile_healthbar(w: &mut World, ns: NetStateId, m: MobileId, healthbar: Healthbar) {
    if w.cannot_send_packets(ns) {
        return;
    }
    let bytes = create_mobile_healthbar(w, m, healthbar);
    w.send(ns, &bytes);
}

pub fn create_mobile_status_compact(w: &mut World, m: MobileId, can_be_renamed: bool) -> Vec<u8> {
    create_mobile_status(w, m, 0, can_be_renamed, (0, 0))
}

pub fn send_mobile_status_compact(
    w: &mut World,
    ns: NetStateId,
    m: MobileId,
    can_be_renamed: bool,
) {
    if w.cannot_send_packets(ns) {
        return;
    }
    let bytes = create_mobile_status_compact(w, m, can_be_renamed);
    w.send(ns, &bytes);
}

/// `SendMobileStatus(m)`: `SendMobileStatus(m, m)`.
pub fn send_mobile_status_self(w: &mut World, ns: NetStateId, m: MobileId) {
    send_mobile_status(w, ns, m, m);
}

/// The status `version` and packet length `SendMobileStatus` picks.
pub fn mobile_status_version(
    beholder_is_beheld: bool,
    core_expansion: Expansion,
    core_max_version: i32,
    ns_expansion: Expansion,
    extended_status: bool,
) -> (i32, usize) {
    if !beholder_is_beheld {
        (0, MOBILE_STATUS_COMPACT_LENGTH)
    } else if core_max_version >= 6 && ns_expansion >= Expansion::HS && extended_status {
        (6, MOBILE_STATUS_HS_LENGTH)
    } else if core_max_version >= 5 && ns_expansion >= Expansion::ML {
        (5, MOBILE_STATUS_ML_LENGTH)
    } else if core_max_version >= 4 && ns_expansion >= Expansion::AOS {
        (4, MOBILE_STATUS_AOS_LENGTH)
    } else {
        let _ = core_expansion;
        (3, MOBILE_STATUS_LENGTH)
    }
}

pub fn send_mobile_status(w: &mut World, ns: NetStateId, beholder: MobileId, beheld: MobileId) {
    if w.cannot_send_packets(ns) || w.mobile(beheld).is_none() {
        return;
    }

    let caps = w.net_state(ns).map(|d| d.caps.clone()).unwrap_or_default();
    let (core_exp, max_version) = core_expansion(w);
    let (version, _length) = mobile_status_version(
        beholder == beheld,
        core_exp,
        max_version,
        ns_expansion(w, &caps),
        caps.extended_status(),
    );

    let renamed =
        (w.mobile(beheld).map(|m| m.ty.can_be_renamed_by)).is_some_and(|f| f(w, beheld, beholder));
    // `beheld.Weapon?.GetStatusDamage(beheld, ..)`: only the packets of version 4 and up carry it.
    let damage = if version >= 4
        && let Some(f) = w
            .resources
            .get::<crate::mobile::part1::props_b::PropsBStatics>()
            .and_then(|s| s.weapon_status_damage)
    {
        f(w, beheld)
    } else {
        (0, 0)
    };
    let bytes = create_mobile_status(w, beheld, version, renamed, damage);
    w.send(ns, &bytes);
}

pub fn create_mobile_status(
    w: &mut World,
    beheld: MobileId,
    version: i32,
    can_be_renamed: bool,
    weapon_damage: (i32, i32),
) -> Vec<u8> {
    let aos = core_expansion(w).0 >= Expansion::AOS;
    create_mobile_status_aos(w, beheld, version, can_be_renamed, aos, weapon_damage)
}

/// `CreateMobileStatus` with `Core.AOS` given.
pub fn create_mobile_status_aos(
    w: &mut World,
    beheld: MobileId,
    version: i32,
    can_be_renamed: bool,
    core_aos: bool,
    weapon_damage: (i32, i32),
) -> Vec<u8> {
    let Some(ty) = w.mobile(beheld).map(|m| m.ty) else {
        return Vec::new();
    };
    // `GetResistance` caches into `Resistances`, so it is read before `m` borrows the world.
    let physical = if version > 0 && core_aos {
        (ty.physical_resistance)(w, beheld)
    } else {
        0
    };
    let [fire, cold, poison, energy] = if version >= 4 {
        [
            ty.fire_resistance,
            ty.cold_resistance,
            ty.poison_resistance,
            ty.energy_resistance,
        ]
        .map(|f| f(w, beheld))
    } else {
        [0; 4]
    };
    let Some(m) = w.mobile(beheld) else {
        return Vec::new();
    };
    let name = (ty.name)(w, beheld);

    let mut pk = Pk::new(121);
    pk.u8(0x11); // Packet ID
    pk.zeros(2); // length, filled in by WritePacketLength
    pk.serial(m.serial);
    let _ = pk.write_latin1_fixed(name.as_deref().unwrap_or(""), 30);
    write_attribute(
        &mut pk,
        (ty.hits_max)(w, beheld),
        m.hits,
        version == 0,
        true,
    );
    pk.bool(can_be_renamed);
    pk.u8(version as u8);

    if version <= 0 {
        return pk.finish_with_length();
    }

    pk.bool(m.female);

    pk.i16((ty.str)(w, beheld) as i16);
    pk.i16((ty.dex)(w, beheld) as i16);
    pk.i16((ty.int)(w, beheld) as i16);

    pk.i16(m.stam as i16);
    pk.i16((ty.stam_max)(w, beheld) as i16);
    pk.i16(m.mana as i16);
    pk.i16((ty.mana_max)(w, beheld) as i16);

    pk.i32(m.total_gold);
    pk.i16(if core_aos {
        physical
    } else {
        ((ty.armor_rating)(w, beheld) + 0.5) as i32
    } as i16);
    pk.i16((crate::mobile::part1::props_b::body_weight(w) + m.total_weight) as i16);

    if version >= 5 {
        pk.i16((ty.max_weight)(w, beheld) as i16);
        // Race is never null (DefaultRace); an unregistered race slot reads as null here.
        let race_id = w.resources.get::<Races>().and_then(|r| match m.race {
            Some(id) => r.get(id),
            None => r.default_race(),
        });
        // Would be 0x00 if it's a non-ML enabled account but...
        pk.u8(race_id.map_or(0, |r| (r.race_id + 1) as u8));
    }

    pk.i16(m.stat_cap as i16);

    pk.u8(m.followers as u8);
    pk.u8(m.followers_max as u8);

    if version >= 4 {
        pk.i16(fire as i16); // Fire
        pk.i16(cold as i16); // Cold
        pk.i16(poison as i16); // Poison
        pk.i16(energy as i16); // Energy
        pk.i16((ty.luck)(w, beheld) as i16); // Luck

        let (min, max) = weapon_damage;
        pk.i16(min as i16); // Damage min
        pk.i16(max as i16); // Damage max

        pk.i32(m.tithing_points);
    }

    if version >= 6 {
        // TODO: once the new statuses are added, the length should be capped by expansion.
        for i in 0..15 {
            pk.i16(crate::mobile::part1::lifecycle::get_aos_status(w, beheld, i) as i16);
        }
    }

    pk.finish_with_length()
}

pub fn create_mobile_update(w: &World, m: MobileId, stygian_abyss: bool) -> Vec<u8> {
    let Some(mob) = w.mobile(m) else {
        return Vec::new();
    };
    let mut pk = Pk::new(19);
    pk.u8(0x20); // Packet ID
    pk.serial(mob.serial);
    pk.i16(crate::mobile::part1::props_b::body(w, m).0 as i16);
    pk.u8(0);
    pk.i16(if mob.solid_hue_override >= 0 {
        mob.solid_hue_override
    } else {
        (mob.ty.hue)(w, m)
    } as i16);
    pk.u8(packet_flags(w, m, stygian_abyss));
    pk.i16(mob.location.x as i16);
    pk.i16(mob.location.y as i16);
    pk.i16(0);
    pk.u8(mob.direction.bits());
    pk.i8(mob.location.z as i8);
    pk.finish()
}

pub fn send_mobile_update(w: &mut World, ns: NetStateId, m: MobileId) {
    if w.cannot_send_packets(ns) {
        return;
    }
    let sa = w.net_state(ns).is_some_and(|d| d.caps.stygian_abyss());
    let bytes = create_mobile_update(w, m, sa);
    w.send(ns, &bytes);
}

pub fn create_mobile_incoming(
    w: &mut World,
    caps: &NetCaps,
    beholder: MobileId,
    beheld: MobileId,
) -> Vec<u8> {
    // The HairSerial/FacialHairSerial getters hand out the virtual serial on first use.
    crate::mobile::part1::props_b::hair_serial(w, beheld);
    crate::mobile::part1::props_b::facial_hair_serial(w, beheld);
    // `CanSee` can have side effects, so it runs before the packet borrows the world.
    let worn: Vec<crate::ItemId> = w
        .mobile(beheld)
        .map(|m| m.items.clone())
        .unwrap_or_default();
    let beholder_ty = w.mobile(beholder).map(|b| b.ty);
    let seen: Vec<bool> = worn
        .iter()
        .map(|&i| beholder_ty.is_some_and(|ty| (ty.can_see_item)(w, beholder, i)))
        .collect();
    let noto = notoriety_compute(w, beholder, beheld);
    let Some(mob) = w.mobile(beheld) else {
        return Vec::new();
    };
    let mut layers = [false; 256];

    let mut pk = Pk::new(23 + (mob.items.len() + 2) * 9);
    pk.u8(0x78); // Packet ID
    pk.zeros(2);

    let sa = caps.stygian_abyss();
    let new_packet = caps.new_mobile_incoming();
    let item_id_mask: i32 = if new_packet { 0xFFFF } else { 0x7FFF };

    let solid = mob.solid_hue_override;
    let hue = if solid >= 0 {
        solid
    } else {
        (mob.ty.hue)(w, beheld)
    };

    pk.serial(mob.serial);
    pk.i16(crate::mobile::part1::props_b::body(w, beheld).0 as i16);
    pk.i16(mob.location.x as i16);
    pk.i16(mob.location.y as i16);
    pk.i8(mob.location.z as i8);
    pk.u8(mob.direction.bits());
    pk.i16(hue as i16);
    pk.u8(packet_flags(w, beheld, sa));
    pk.u8(noto as u8);

    // One equipped entry: serial, itemID (| 0x8000 for the old packet when a hue follows), layer, hue.
    let write_entry = |pk: &mut Pk, serial: Serial, item_id: i32, layer: u8, hue: i32| {
        let mut item_id = item_id & item_id_mask;
        let write_hue = new_packet || hue != 0;

        if !new_packet && write_hue {
            item_id |= 0x8000;
        }

        pk.serial(serial);
        pk.u16(item_id as u16);
        pk.u8(layer);

        if write_hue {
            pk.i16(hue as i16);
        }
    };

    for (n, &item_id) in mob.items.iter().enumerate() {
        let Some(item) = w.item(item_id) else {
            continue;
        };
        let layer = item.layer.0;

        let can_see = seen.get(n).copied().unwrap_or(false);
        if item.deleted() || !can_see || layers[layer as usize] {
            continue;
        }

        layers[layer as usize] = true;
        let hue = if solid >= 0 { solid } else { item.hue };
        write_entry(&mut pk, item.serial, item.item_id, layer, hue);
    }

    if mob.hair_item_id > 0 && !layers[Layer::Hair.0 as usize] {
        layers[Layer::Hair.0 as usize] = true;
        let hue = if solid >= 0 { solid } else { mob.hair_hue };
        write_entry(
            &mut pk,
            mob.hair_serial,
            mob.hair_item_id,
            Layer::Hair.0,
            hue,
        );
    }

    if mob.facial_hair_item_id > 0 && !layers[Layer::FacialHair.0 as usize] {
        // (The layer would be marked here too; nothing reads it after.)
        let hue = if solid >= 0 {
            solid
        } else {
            mob.facial_hair_hue
        };
        write_entry(
            &mut pk,
            mob.facial_hair_serial,
            mob.facial_hair_item_id,
            Layer::FacialHair.0,
            hue,
        );
    }

    pk.i32(0); // terminate

    pk.finish_with_length()
}

pub fn send_mobile_incoming(w: &mut World, ns: NetStateId, beholder: MobileId, beheld: MobileId) {
    if w.cannot_send_packets(ns) {
        return;
    }
    let caps = w.net_state(ns).map(|d| d.caps.clone()).unwrap_or_default();
    let bytes = create_mobile_incoming(w, &caps, beholder, beheld);
    w.send(ns, &bytes);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::item::ITEM_BASE;
    use crate::mobile::MOBILE_BASE;
    use crate::net_state::{NetOut, NetStateData};
    use crate::poison::PoisonId;
    use muo_net::protocol_changes::ProtocolChanges;
    use muo_primitives::{Direction, Point3D};

    fn world_with_ns(changes: ProtocolChanges) -> (World, NetStateId) {
        let mut w = World::new();
        let ns = NetStateId(1);
        let mut d = NetStateData::default();
        d.caps.protocol_changes = changes;
        w.net_states.insert(ns, d);
        (w, ns)
    }

    fn sent(w: &mut World) -> Vec<u8> {
        match w.net_out.pop() {
            Some(NetOut::Send(_, b)) => b,
            _ => panic!("nothing sent"),
        }
    }

    fn be16(v: i32) -> [u8; 2] {
        (v as i16).to_be_bytes()
    }

    /// `CreateMobileWithStats`: str 50 (hits max 75), hits 75 (clamped from 100), int 75, mana 100, dex 25, stam 100.
    fn stat_mobile(w: &mut World, name: Option<&str>) -> MobileId {
        let id = w.new_mobile(&MOBILE_BASE);
        let m = w.mobile_mut(id).unwrap();
        m.name = name.map(String::from);
        m.str = 50;
        m.hits = 75;
        m.int = 75;
        m.mana = 75;
        m.dex = 25;
        m.stam = 25;
        id
    }

    #[test]
    fn death_animation_and_bond() {
        let (mut w, ns) = world_with_ns(ProtocolChanges::empty());
        send_death_animation(&mut w, ns, Serial::new(1), Serial::new(0x1000));
        assert_eq!(
            sent(&mut w),
            vec![0xAF, 0, 0, 0, 1, 0, 0, 0x10, 0, 0, 0, 0, 0]
        );
        send_bonded_status(&mut w, ns, Serial::new(1), true);
        assert_eq!(sent(&mut w), vec![0xBF, 0, 11, 0, 0x19, 0, 0, 0, 0, 1, 1]);
    }

    #[test]
    fn mobile_moving() {
        for changes in [ProtocolChanges::STYGIAN_ABYSS, ProtocolChanges::empty()] {
            let (mut w, ns) = world_with_ns(changes);
            let id = w.new_mobile(&MOBILE_BASE);
            {
                let m = w.mobile_mut(id).unwrap();
                m.location = Point3D {
                    x: 1000,
                    y: 2000,
                    z: 7,
                };
                m.body.0 = 0x190;
                m.hue = 0x21;
                m.direction = Direction::SOUTH;
            }
            send_mobile_moving(&mut w, ns, id, 10);
            let sa = changes.contains(ProtocolChanges::STYGIAN_ABYSS);
            let flags = packet_flags(&w, id, sa);
            let mut e = vec![0x77];
            e.extend(id.0.value().to_be_bytes());
            e.extend(be16(0x190));
            e.extend(be16(1000));
            e.extend(be16(2000));
            e.extend([7, 4]);
            e.extend(be16(0x21));
            e.extend([flags, 10]);
            assert_eq!(sent(&mut w), e);

            // The cached variant sends the same bytes, and a built cache keeps its first build.
            let mut cache = MobileMovingCache::default();
            send_mobile_moving_using_cache(&mut w, ns, &mut cache, id, 10);
            assert_eq!(sent(&mut w), e);
            w.mobile_mut(id).unwrap().hue = 0x99;
            send_mobile_moving_using_cache(&mut w, ns, &mut cache, id, 5);
            let again = sent(&mut w);
            assert_eq!(&again[..15], &e[..15]);
            assert_eq!(again[16], 5);
        }
    }

    #[test]
    fn mobile_name() {
        for name in [
            None,
            Some("Kamron"),
            Some("Some Really Long Mobile Name That Gets Cut off"),
        ] {
            let (mut w, ns) = world_with_ns(ProtocolChanges::empty());
            let id = stat_mobile(&mut w, name);
            send_mobile_name(&mut w, ns, id);
            let mut e = vec![0x98, 0, 37];
            e.extend(id.0.value().to_be_bytes());
            let mut n = name.unwrap_or("").as_bytes().to_vec();
            n.resize(29, 0);
            e.extend(n);
            e.push(0);
            assert_eq!(sent(&mut w), e);
        }
    }

    #[test]
    fn animations() {
        let (mut w, ns) = world_with_ns(ProtocolChanges::empty());
        for (action, frames, repeat_count, reverse, repeat, delay) in [
            (200, 5, 1, false, false, 5u8),
            (10, 100, 25, true, false, 0),
        ] {
            send_mobile_animation(
                &mut w,
                ns,
                Serial::new(1),
                action,
                frames,
                repeat_count,
                !reverse,
                repeat,
                delay as i32,
            );
            let mut e = vec![0x6E, 0, 0, 0, 1];
            e.extend(be16(action));
            e.extend(be16(frames));
            e.extend(be16(repeat_count));
            e.extend([reverse as u8, repeat as u8, delay]);
            assert_eq!(sent(&mut w), e);
        }
        for (action, frames, delay) in [(200, 5, 5u8), (10, 100, 20)] {
            send_new_mobile_animation(&mut w, ns, Serial::new(1), action, frames, delay as i32);
            let mut e = vec![0xE2, 0, 0, 0, 1];
            e.extend(be16(action));
            e.extend(be16(frames));
            e.push(delay);
            assert_eq!(sent(&mut w), e);
        }
    }

    #[test]
    fn healthbars() {
        // `m.Poison?.Level + 1 ?? 0`: the level, not the index (LesserDarkglow is index 10, level 0)
        for (index, level) in [(-1i32, -1i32), (0, 0), (4, 4), (10, 0), (13, 3)] {
            let (mut w, ns) = world_with_ns(ProtocolChanges::empty());
            let mut ps = crate::poison::Poisons::default();
            for (i, l) in [(0, 0), (4, 4), (10, 0), (13, 3)] {
                ps.register(crate::poison::Poison {
                    index: i,
                    name: Box::leak(format!("p{i}").into_boxed_str()),
                    level: l,
                    family: crate::poison::PoisonFamily::Standard,
                    construct_timer: |_, _| Default::default(),
                })
                .unwrap();
            }
            *w.resources.get_mut::<crate::poison::Poisons>() = ps;
            let id = w.new_mobile(&MOBILE_BASE);
            w.mobile_mut(id).unwrap().poison = (index >= 0).then_some(PoisonId(index as u8));
            send_mobile_healthbar(&mut w, ns, id, Healthbar::Poison);
            let mut e = vec![0x17, 0, 12];
            e.extend(id.0.value().to_be_bytes());
            e.extend([0, 1, 0, 1, (level + 1) as u8]);
            assert_eq!(sent(&mut w), e);
        }
        for (blessed, yellow) in [(false, false), (true, false), (false, true), (true, true)] {
            let (mut w, ns) = world_with_ns(ProtocolChanges::empty());
            let id = w.new_mobile(&MOBILE_BASE);
            {
                let m = w.mobile_mut(id).unwrap();
                m.blessed = blessed;
                m.yellow_healthbar = yellow;
            }
            send_mobile_healthbar(&mut w, ns, id, Healthbar::Yellow);
            let mut e = vec![0x17, 0, 12];
            e.extend(id.0.value().to_be_bytes());
            e.extend([0, 1, 0, 2, (blessed || yellow) as u8]);
            assert_eq!(sent(&mut w), e);
        }
    }

    #[test]
    fn status_compact() {
        for renamed in [true, false] {
            let (mut w, ns) = world_with_ns(ProtocolChanges::empty());
            let id = stat_mobile(&mut w, Some("Random Mobile 1"));
            send_mobile_status_compact(&mut w, ns, id, renamed);
            // legacy MobileStatusCompact: hits normalized and reversed: (cur * 100 / max, 100)
            let mut e = vec![0x11, 0, 43];
            e.extend(id.0.value().to_be_bytes());
            let mut n = b"Random Mobile 1".to_vec();
            n.resize(30, 0);
            e.extend(n);
            e.extend(be16(100));
            e.extend(be16(100));
            e.extend([renamed as u8, 0]);
            assert_eq!(sent(&mut w), e);
        }
    }

    #[test]
    fn status_other_is_compact() {
        let (mut w, ns) = world_with_ns(ProtocolChanges::VERSION_70610);
        let a = stat_mobile(&mut w, Some("Random Mobile 1"));
        let b = stat_mobile(&mut w, Some("Random Mobile 2"));
        send_mobile_status(&mut w, ns, a, b);
        let out = sent(&mut w);
        assert_eq!(out.len(), 43);
        assert_eq!(&out[..3], &[0x11, 0, 43]);
    }

    #[test]
    fn status_version_choice() {
        use Expansion::*;
        assert_eq!(mobile_status_version(false, HS, 6, HS, true), (0, 43));
        assert_eq!(mobile_status_version(true, HS, 6, HS, true), (6, 121));
        assert_eq!(mobile_status_version(true, HS, 6, HS, false), (5, 91));
        assert_eq!(mobile_status_version(true, ML, 5, ML, true), (5, 91));
        assert_eq!(mobile_status_version(true, AOS, 4, AOS, true), (4, 88));
        assert_eq!(mobile_status_version(true, ML, 5, LBR, false), (3, 70));
    }

    /// The legacy `MobileStatusExtended`/`MobileStatus` self layout for type 3..6 (no weapon, no handlers).
    fn legacy_status_self(w: &mut World, id: MobileId, version: i32, aos: bool) -> Vec<u8> {
        let phys = (MOBILE_BASE.physical_resistance)(w, id);
        let [fire, cold, poison, energy] = [
            MOBILE_BASE.fire_resistance,
            MOBILE_BASE.cold_resistance,
            MOBILE_BASE.poison_resistance,
            MOBILE_BASE.energy_resistance,
        ]
        .map(|r| r(w, id));
        let m = w.mobile(id).unwrap();
        let mut b = vec![0x11, 0, 0];
        b.extend(m.serial.value().to_be_bytes());
        let mut n = m.name.clone().unwrap_or_default().into_bytes();
        n.resize(30, 0);
        b.extend(n);
        b.extend(be16(m.hits));
        b.extend(be16((MOBILE_BASE.hits_max)(w, id)));
        b.extend([0, version as u8, m.female as u8]);
        b.extend(be16(m.str));
        b.extend(be16(m.dex));
        b.extend(be16(m.int));
        b.extend(be16(m.stam));
        b.extend(be16((MOBILE_BASE.stam_max)(w, id)));
        b.extend(be16(m.mana));
        b.extend(be16((MOBILE_BASE.mana_max)(w, id)));
        b.extend(m.total_gold.to_be_bytes());
        b.extend(be16(if aos {
            phys
        } else {
            ((MOBILE_BASE.armor_rating)(w, id) + 0.5) as i32
        }));
        b.extend(be16(
            crate::mobile::part1::props_b::body_weight(w) + m.total_weight,
        ));
        if version >= 5 {
            b.extend(be16((MOBILE_BASE.max_weight)(w, id)));
            b.push(0); // no race registered
        }
        b.extend(be16(m.stat_cap));
        b.extend([m.followers as u8, m.followers_max as u8]);
        if version >= 4 {
            for r in [fire, cold, poison, energy, (MOBILE_BASE.luck)(w, id)] {
                b.extend(be16(r));
            }
            b.extend([0, 0, 0, 0]);
            b.extend(m.tithing_points.to_be_bytes());
        }
        if version >= 6 {
            b.extend([0u8; 30]);
        }
        let l = (b.len() as u16).to_be_bytes();
        b[1] = l[0];
        b[2] = l[1];
        b
    }

    #[test]
    fn status_self_all_versions() {
        let mut w = World::new();
        let id = stat_mobile(&mut w, Some("Random Mobile 1"));
        for (version, len) in [(3, 70), (4, 88), (5, 91), (6, 121)] {
            let got = create_mobile_status_aos(&mut w, id, version, false, version >= 4, (0, 0));
            assert_eq!(got.len(), len);
            assert_eq!(got, legacy_status_self(&mut w, id, version, version >= 4));
        }
        // pre-AOS armor rating branch
        let got = create_mobile_status_aos(&mut w, id, 3, false, false, (0, 0));
        assert_eq!(got, legacy_status_self(&mut w, id, 3, false));
    }

    #[test]
    fn mobile_update() {
        for (changes, solid) in [
            (ProtocolChanges::empty(), 0),
            (ProtocolChanges::STYGIAN_ABYSS, 100),
        ] {
            let (mut w, ns) = world_with_ns(changes);
            let id = w.new_mobile(&MOBILE_BASE);
            {
                let m = w.mobile_mut(id).unwrap();
                m.solid_hue_override = solid;
                m.hue = 7;
                m.location = Point3D { x: 5, y: 6, z: -1 };
                m.body.0 = 0x191;
            }
            send_mobile_update(&mut w, ns, id);
            let sa = changes.contains(ProtocolChanges::STYGIAN_ABYSS);
            let mut e = vec![0x20];
            e.extend(id.0.value().to_be_bytes());
            e.extend(be16(0x191));
            e.push(0);
            e.extend(be16(if solid >= 0 { solid } else { 7 }));
            e.push(packet_flags(&w, id, sa));
            e.extend(be16(5));
            e.extend(be16(6));
            e.extend(be16(0));
            e.extend([0, 0xFF]);
            assert_eq!(sent(&mut w), e);
        }
    }

    #[test]
    fn mobile_incoming() {
        let cases = [
            ProtocolChanges::VERSION_70331,
            ProtocolChanges::VERSION_6000,
            ProtocolChanges::VERSION_7000,
        ];
        for changes in cases {
            for (hair, hair_hue, facial, facial_hue) in
                [(0, 0, 0, 0), (10, 1024, 0, 0), (10, 1024, 11, 2048)]
            {
                let (mut w, ns) = world_with_ns(changes);
                let beholder = w.new_mobile(&MOBILE_BASE);
                let beheld = w.new_mobile(&MOBILE_BASE);
                let i1 = w.new_item(&ITEM_BASE);
                let i2 = w.new_item(&ITEM_BASE);
                for i in [i1, i2] {
                    let it = w.item_mut(i).unwrap();
                    it.layer = Layer::OneHanded;
                    it.item_id = 0xF3F;
                    it.hue = 0x30;
                    // CanSee(Item) is false on Map.Internal; the test shares the same legacy check
                    it.map = Some(crate::map::MapId::FELUCCA);
                    it.parent = Some(crate::ids::EntityId::Mobile(beheld));
                }
                for m in [beholder, beheld] {
                    w.mobile_mut(m).unwrap().map = Some(crate::map::MapId::FELUCCA);
                }
                {
                    let m = w.mobile_mut(beheld).unwrap();
                    m.items = vec![i1, i2]; // the second is a dupe of the layer
                    m.hair_item_id = hair;
                    m.hair_hue = hair_hue;
                    m.hair_serial = Serial::new(0x7000_0001);
                    m.facial_hair_item_id = facial;
                    m.facial_hair_hue = facial_hue;
                    m.facial_hair_serial = Serial::new(0x7000_0002);
                }
                send_mobile_incoming(&mut w, ns, beholder, beheld);

                // legacy MobileIncoming
                let newp = changes.contains(ProtocolChanges::NEW_MOBILE_INCOMING);
                let sa = changes.contains(ProtocolChanges::STYGIAN_ABYSS);
                let mask = if newp { 0xFFFF } else { 0x7FFF };
                let entry = |b: &mut Vec<u8>, serial: u32, id: i32, layer: u8, hue: i32| {
                    let wh = newp || hue != 0;
                    let mut id = id & mask;
                    if !newp && wh {
                        id |= 0x8000;
                    }
                    b.extend(serial.to_be_bytes());
                    b.extend((id as u16).to_be_bytes());
                    b.push(layer);
                    if wh {
                        b.extend(be16(hue));
                    }
                };
                let mut e = vec![0x78, 0, 0];
                e.extend(beheld.0.value().to_be_bytes());
                e.extend(be16(0));
                e.extend(be16(0));
                e.extend(be16(0));
                e.extend([0, 0]);
                e.extend(be16(0));
                e.extend([packet_flags(&w, beheld, sa), 3]);
                entry(&mut e, i1.0.value(), 0xF3F, 1, 0x30);
                if hair > 0 {
                    entry(&mut e, 0x7000_0001, hair, 0x0B, hair_hue);
                }
                if facial > 0 {
                    entry(&mut e, 0x7000_0002, facial, 0x10, facial_hue);
                }
                e.extend([0, 0, 0, 0]);
                let l = (e.len() as u16).to_be_bytes();
                e[1] = l[0];
                e[2] = l[1];
                assert_eq!(sent(&mut w), e, "{changes:?} {hair} {facial}");
            }
        }
    }

    #[test]
    fn attributes() {
        let (mut w, ns) = world_with_ns(ProtocolChanges::empty());
        let id = stat_mobile(&mut w, None);
        let s = id.0.value().to_be_bytes();
        for normalize in [false, true] {
            send_mobile_hits(&mut w, ns, id, normalize);
            let mut e = vec![0xA1];
            e.extend(s);
            if normalize {
                e.extend(be16(100));
                e.extend(be16(100));
            } else {
                e.extend(be16(75));
                e.extend(be16(75));
            }
            assert_eq!(sent(&mut w), e);

            send_mobile_attributes(&mut w, ns, id, normalize);
            let b = sent(&mut w);
            assert_eq!(b.len(), 17);
            assert_eq!(b[0], 0x2D);
            let hm = (MOBILE_BASE.hits_max)(&w, id);
            let mm = (MOBILE_BASE.mana_max)(&w, id);
            let sm = (MOBILE_BASE.stam_max)(&w, id);
            let mut e = vec![0x2D];
            e.extend(s);
            for (max, cur) in [(hm, 75), (mm, 75), (sm, 25)] {
                if normalize && max != 0 {
                    e.extend(be16(100));
                    e.extend(be16(cur * 100 / max));
                } else {
                    e.extend(be16(max));
                    e.extend(be16(cur));
                }
            }
            assert_eq!(b, e);

            send_mobile_mana(&mut w, ns, id, normalize);
            assert_eq!(sent(&mut w)[0], 0xA2);
            send_mobile_stam(&mut w, ns, id, normalize);
            assert_eq!(sent(&mut w)[0], 0xA3);
        }
    }
}
