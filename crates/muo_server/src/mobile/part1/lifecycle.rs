//! Mobile lifecycle: Serialize, Delete, Map and Location, ProcessDelta, ranges,
//! light levels, resistances and the name properties. See `part1.rs`.

use crate::layer::Layer;
use muo_io::GenericWriter;
use muo_primitives::{Body, BodyTable, Point3D, Serial, utility};

use crate::guild::GuildType;
use crate::ids::{EntityId, ItemId, MobileId};
use crate::map::MapId;
use crate::mobile::part2::{deps, effects, items};
use crate::mobile::{AccessLevel, Healthbar, Mobile, MobileDelta, ResistanceType};
use crate::net_state::NetStateId;
use crate::packets::mobile::MobileMovingCache;
use crate::packets::{self, Pk};
use crate::property_list::PropertyList;
use crate::race::Races;
use crate::region;
use crate::region::RegionId;
use crate::world::World;

/// `Mobile.m_GuildTypes`.
const GUILD_TYPES: [&str; 3] = ["", " (Chaos)", " (Order)"];

/// `entity?.Deleted != false ? Serial.MinusOne : entity.Serial` (`Write(ISerializable)`) for a mobile.
fn mobile_ref(w: &World, id: Option<MobileId>) -> Serial {
    match id {
        Some(id) if w.mobile(id).is_some_and(|m| !m.deleted) => id.0,
        _ => Serial::MINUS_ONE,
    }
}

/// The same for an item.
fn item_ref(w: &World, id: Option<ItemId>) -> Serial {
    match id {
        Some(id) if w.item(id).is_some_and(|i| !i.deleted()) => id.0,
        _ => Serial::MINUS_ONE,
    }
}

/// `Mobile.Race`'s index: a null `m_Race` reads as `Race.DefaultRace` (`Races[0]`), which is null before races load.
fn race_index(w: &World, m: &Mobile) -> Option<u8> {
    match m.race {
        Some(r) => Some(r.0),
        None => w
            .resources
            .get::<Races>()
            .and_then(|r| r.default_race())
            .map(|r| r.race_index as u8),
    }
}

/// `Serialize` (version 38).
pub fn serialize(w: &World, id: MobileId, writer: &mut dyn GenericWriter) {
    let Some(m) = w.mobile(id) else { return };

    writer.write_i32(38); // version

    writer.write_anchored_time_ticks(m.last_str_gain.ticks());
    writer.write_anchored_time_ticks(m.last_int_gain.ticks());
    writer.write_anchored_time_ticks(m.last_dex_gain.ticks());

    let mut hairflag = 0u8;
    if m.hair_item_id > 0 {
        hairflag |= 0x01;
    }
    if m.facial_hair_item_id > 0 {
        hairflag |= 0x02;
    }
    writer.write_u8(hairflag);

    if hairflag & 0x01 != 0 {
        writer.write_i32(m.hair_item_id);
        writer.write_i32(m.hair_hue);
    }
    if hairflag & 0x02 != 0 {
        writer.write_i32(m.facial_hair_item_id);
        writer.write_i32(m.facial_hair_hue);
    }

    writer.write_race_index(race_index(w, m));

    writer.write_i32(m.tithing_points);

    writer.write_serial(item_ref(w, m.corpse));

    writer.write_bool(m.cant_walk);

    // VirtueInfo.Serialize(writer, Virtues);

    writer.write_i32(m.thirst);
    writer.write_i32(m.bac);

    writer.write_i32(m.followers_max);

    writer.write_i32(m.magic_damage_absorb);

    writer.write_serial(mobile_ref(w, m.guild_fealty));

    // `Write(ISerializable)` writes MinusOne for a deleted (disbanded) guild
    writer.write_serial(
        m.guild
            .and_then(|g| crate::guild::guild(w, g))
            .filter(|g| !g.disbanded)
            .map_or(Serial::MINUS_ONE, |g| g.serial),
    );

    writer.write_bool(m.display_guild_title);

    writer.write_bool(m.can_swim);

    writer.write_bool(m.squelched);

    writer.write_serial(item_ref(w, m.holding));

    writer.write_i32(m.virtual_armor);

    writer.write_i32(m.base_sound_id);

    writer.write_bool(m.disarm_ready);
    writer.write_bool(m.stun_ready);

    writer.write_i32(m.stat_cap);

    writer.write_i32(m.name_hue);

    writer.write_i32(m.hunger);

    writer.write_point3d(m.location);
    writer.write_i32(m.body.0);
    writer.write_opt_string(m.name.as_deref());
    writer.write_opt_string(m.guild_title.as_deref());
    writer.write_bool(m.criminal);
    writer.write_i32(m.kills);
    writer.write_i32(m.speech_hue);
    writer.write_i32(m.emote_hue);
    writer.write_i32(m.whisper_hue);
    writer.write_i32(m.yell_hue);
    writer.write_opt_string(m.language.as_deref());
    writer.write_bool(m.female);
    writer.write_bool(m.warmode);
    writer.write_bool(m.hidden);
    writer.write_u8(m.direction.bits());
    writer.write_i32(m.hue);
    writer.write_i32(m.str);
    writer.write_i32(m.dex);
    writer.write_i32(m.int);
    writer.write_i32(m.hits);
    writer.write_i32(m.stam);
    writer.write_i32(m.mana);

    writer.write_map_index(m.map.map(|m| m.0));

    writer.write_bool(m.blessed);
    writer.write_i32(m.fame);
    writer.write_i32(m.karma);
    writer.write_u8(m.access_level as u8);
    // `Skills.Serialize` recomputes `Total` while writing; done on a copy, as this only has `&World`.
    // shortcut: the live skills keep their cached total, it is the same sum and is recomputed when read.
    let mut skills = m.skills.clone().unwrap_or_default();
    skills.write(writer);

    writer.write_i32(m.items.len() as i32);
    for &item in &m.items {
        writer.write_serial(item_ref(w, Some(item)));
    }

    writer.write_bool(m.player);
    writer.write_opt_string(m.title.as_deref());
    writer.write_opt_string(m.profile.as_deref());
    writer.write_bool(m.profile_locked);
    writer.write_bool(m.auto_page_notify);

    writer.write_point3d(m.logout_location);
    writer.write_map_index(m.logout_map.map(|m| m.0));

    writer.write_u8(m.str_lock as u8);
    writer.write_u8(m.dex_lock as u8);
    writer.write_u8(m.int_lock as u8);
}

pub fn delete(w: &mut World, id: MobileId) {
    let Some(m) = w.mobile(id) else { return };
    if m.deleted {
        return;
    }
    let ty = m.ty;

    if let Some(ns) = m.net_state {
        let reason = format!(
            "Player {} has been deleted.",
            super::combat::to_string(w, id)
        );
        deps::cancel_all_trades(w, ns);
        w.disconnect(ns, &reason);
    }

    items::drop_holding(w, id);

    let old_region = w.mobile(id).map_or(RegionId(0), |m| m.region);
    region::on_region_change(w, id, old_region, RegionId(0));
    if let Some(m) = w.mobile_mut(id) {
        m.region = RegionId(0); // no region
    }

    (ty.on_delete)(w, id);

    let mut i = w.mobile(id).map_or(0, |m| m.items.len());
    while i > 0 {
        i -= 1;
        let item = w.mobile(id).and_then(|m| m.items.get(i).copied());
        if let Some(item) = item
            && let Some(it) = w.item(item)
        {
            let it_ty = it.ty;
            (it_ty.on_parent_deleted)(w, item, EntityId::Mobile(id));
        }
    }

    effects::send_remove_packet(w, id, true);

    if let Some(g) = w.mobile(id).and_then(|m| m.guild)
        && let Some(vt) = crate::guild::guild(w, g).map(|g| g.vt)
    {
        (vt.on_delete)(w, g, id);
    }

    let (map, loc) = match w.mobile_mut(id) {
        Some(m) => {
            m.deleted = true;
            (m.map, m.location)
        }
        None => return,
    };

    if let Some(map) = map {
        crate::map::sector::on_leave_mobile(w, map, loc, id);
    }
    if let Some(m) = w.mobile_mut(id) {
        m.map = None;
        m.mount_item = None;
        // The mobile is gone from the world by the end, so this is cleared before `OnAfterDelete`.
        m.property_list = None;
    }

    // `World.RemoveEntity` runs first and then OnAfterDelete on the detached object: kept in the table for the
    // handler's reads, hidden from lookups (`World::detaching`).
    w.detaching.push(id.0);
    (ty.on_after_delete)(w, id);
    w.detaching.retain(|&s| s != id.0);
    w.remove_mobile(id);
}

// `SkipSerialization` is the vtable default (`false`).

/// `Mobile.Map` getter.
pub fn map(w: &World, id: MobileId) -> Option<MapId> {
    w.mobile(id).and_then(|m| m.map)
}

pub fn set_map(w: &mut World, id: MobileId, value: Option<MapId>) {
    let Some(m) = w.mobile(id) else { return };
    if m.deleted || m.map == value {
        return;
    }

    let (ns, old_map, loc) = (m.net_state, m.map, m.location);
    if let Some(ns) = ns {
        deps::validate_all_trades(w, ns);
    }
    if let Some(old) = old_map {
        crate::map::sector::on_leave_mobile(w, old, loc, id);

        effects::clear_screen(w, id);
        effects::send_remove_packet(w, id, true);
    }

    set_items_map(w, id, value);

    if let Some(m) = w.mobile_mut(id) {
        m.map = value;
    }
    internal_map_change(w, id, old_map);
}

/// `for (i..) Items[i].Map = value`.
fn set_items_map(w: &mut World, id: MobileId, value: Option<MapId>) {
    let mut i = 0;
    while let Some(item) = w.mobile(id).and_then(|m| m.items.get(i).copied()) {
        crate::item::ops::set_map(w, item, value);
        i += 1;
    }
}

/// `Mobile.Location` getter.
pub fn location(w: &World, id: MobileId) -> Point3D {
    w.mobile(id).map_or(Point3D::default(), |m| m.location)
}

/// `Location` setter: `SetLocation(value, true)`.
pub fn set_location(w: &mut World, id: MobileId, value: Point3D) {
    if let Some(ty) = w.mobile(id).map(|m| m.ty) {
        (ty.set_location)(w, id, value, true);
    }
}

fn internal_map_change(w: &mut World, id: MobileId, old_map: Option<MapId>) {
    let Some(m) = w.mobile(id) else { return };
    let ty = m.ty;
    let ns = m.net_state;
    let (map, loc) = (m.map, m.location);

    if let Some(map) = map {
        crate::map::sector::on_enter_mobile(w, map, loc, id);
    }
    effects::update_region(w, id);

    if let Some(ns) = ns {
        crate::net_glue::reset_movement_state(w, ns);

        if let Some(map) = map {
            packets::map::send_map_change(w, ns, Some(map));

            packets::map::send_map_patches_of_world(w, ns);

            let season = (ty.get_season)(w, id);
            packets::player::send_season_change(w, ns, season as u8, true);

            packets::mobile::send_mobile_update(w, ns, id);
            let size = w.maps.get(map).map(|m| (m.width, m.height));
            if let Some((width, height)) = size {
                packets::player::send_server_change(w, ns, loc, width, height);
            }
        }

        packets::mobile::send_mobile_incoming(w, ns, id, id);

        packets::mobile::send_mobile_update(w, ns, id);
        (ty.check_light_levels)(w, id, true);
        packets::mobile::send_mobile_update(w, ns, id);
    }

    effects::send_everything(w, id);
    effects::send_incoming_packet(w, id);

    // The rest is sent on `ns` after the `if`; the senders ignore a null state.
    if let Some(ns) = ns {
        packets::mobile::send_mobile_incoming(w, ns, id, id);
        packets::account::send_supported_feature_of_core(w, ns);
        packets::mobile::send_mobile_update(w, ns, id);
        packets::mobile::send_mobile_attributes(w, ns, id, false);
    }

    (ty.on_map_change)(w, id, old_map);
}

pub fn move_to_world(w: &mut World, id: MobileId, new_location: Point3D, map: MapId) {
    let Some(m) = w.mobile(id) else { return };
    if m.deleted {
        return;
    }
    let ty = m.ty;
    let old_location = m.location;
    let old_map = m.map;

    if old_map == Some(map) {
        (ty.set_location)(w, id, new_location, true);
        return;
    }

    items::close_bank_box(w, id);

    if let Some(old) = old_map {
        crate::map::sector::on_leave_mobile(w, old, old_location, id);

        effects::clear_screen(w, id);
        effects::send_remove_packet(w, id, true);
    }

    set_items_map(w, id, Some(map));

    if let Some(m) = w.mobile_mut(id) {
        m.map = Some(map);
        m.location = new_location;
    }
    internal_map_change(w, id, old_map);
    (ty.on_location_change)(w, id, old_location);

    let region_id = w.mobile(id).map_or(RegionId(0), |m| m.region);
    if let Some(r) = w.region(region_id) {
        let rty = r.ty;
        (rty.on_location_changed)(w, region_id, id, old_location);
    }
}

/// `Mobile.X` getter.
pub fn x(w: &World, id: MobileId) -> i32 {
    location(w, id).x
}

pub fn set_x(w: &mut World, id: MobileId, value: i32) {
    let l = location(w, id);
    set_location(w, id, Point3D::new(value, l.y, l.z));
}

/// `Mobile.Y` getter.
pub fn y(w: &World, id: MobileId) -> i32 {
    location(w, id).y
}

pub fn set_y(w: &mut World, id: MobileId, value: i32) {
    let l = location(w, id);
    set_location(w, id, Point3D::new(l.x, value, l.z));
}

/// `Mobile.Z` getter.
pub fn z(w: &World, id: MobileId) -> i32 {
    location(w, id).z
}

pub fn set_z(w: &mut World, id: MobileId, value: i32) {
    let l = location(w, id);
    set_location(w, id, Point3D::new(l.x, l.y, value));
}

/// What `ProcessDelta` decides to send, from the delta flags alone (the first half of `ProcessDelta`).
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct DeltaPlan {
    pub send_hits: bool,
    pub send_stam: bool,
    pub send_mana: bool,
    pub send_all: bool,
    pub send_any: bool,
    pub send_incoming: bool,
    pub send_nonlocal_incoming: bool,
    pub send_update: bool,
    pub send_remove: bool,
    pub send_public_stats: bool,
    pub send_private_stats: bool,
    pub send_moving: bool,
    pub send_nonlocal_moving: bool,
    pub send_opl_update: bool,
    pub send_hair: bool,
    pub send_facial_hair: bool,
    pub remove_hair: bool,
    pub remove_facial_hair: bool,
    pub send_healthbar_poison: bool,
    pub send_healthbar_yellow: bool,
}

impl DeltaPlan {
    /// The branches of `ProcessDelta` before the first packet. `opl_enabled` is `ObjectPropertyList.Enabled`.
    pub fn new(
        delta: MobileDelta,
        hair_item_id: i32,
        facial_hair_item_id: i32,
        opl_enabled: bool,
    ) -> DeltaPlan {
        let mut p = DeltaPlan {
            send_opl_update: opl_enabled && delta.intersects(MobileDelta::PROPERTIES),
            ..DeltaPlan::default()
        };
        let attrs = delta & MobileDelta::ATTRIBUTES;

        if attrs != MobileDelta::NONE {
            p.send_any = true;

            if attrs == MobileDelta::ATTRIBUTES {
                p.send_all = true;
            } else {
                p.send_hits = attrs.intersects(MobileDelta::HITS);
                p.send_stam = attrs.intersects(MobileDelta::STAM);
                p.send_mana = attrs.intersects(MobileDelta::MANA);
            }
        }

        if delta.intersects(MobileDelta::GHOST_UPDATE) {
            p.send_nonlocal_incoming = true;
        }

        if delta.intersects(MobileDelta::HUE) {
            p.send_nonlocal_incoming = true;
            p.send_update = true;
            p.send_remove = true;
        }

        if delta.intersects(MobileDelta::DIRECTION) {
            p.send_nonlocal_moving = true;
            p.send_update = true;
        }

        if delta.intersects(MobileDelta::BODY) {
            p.send_update = true;
            p.send_incoming = true;
        }

        if delta.intersects(MobileDelta::FLAGS | MobileDelta::NOTO) {
            p.send_moving = true;
        }

        if delta.intersects(MobileDelta::HEALTHBAR_POISON) {
            p.send_healthbar_poison = true;
        }

        if delta.intersects(MobileDelta::HEALTHBAR_YELLOW) {
            p.send_healthbar_yellow = true;
        }

        if delta.intersects(MobileDelta::NAME) {
            p.send_all = false;
            p.send_hits = false;
            p.send_any = p.send_stam || p.send_mana;
            p.send_public_stats = true;
        }

        if delta.intersects(
            MobileDelta::WEAPON_DAMAGE
                | MobileDelta::RESISTANCES
                | MobileDelta::STAT
                | MobileDelta::WEIGHT
                | MobileDelta::GOLD
                | MobileDelta::ARMOR
                | MobileDelta::STAT_CAP
                | MobileDelta::FOLLOWERS
                | MobileDelta::TITHING_POINTS
                | MobileDelta::RACE,
        ) {
            p.send_private_stats = true;
        }

        if delta.intersects(MobileDelta::HAIR) {
            if hair_item_id <= 0 {
                p.remove_hair = true;
            }
            p.send_hair = true;
        }

        if delta.intersects(MobileDelta::FACIAL_HAIR) {
            if facial_hair_item_id <= 0 {
                p.remove_facial_hair = true;
            }
            p.send_facial_hair = true;
        }

        p
    }

    /// The test after the local packets: whether anything goes to the clients in range. Also folds the
    /// nonlocal flags in.
    pub fn nonlocal(&mut self) -> bool {
        self.send_moving = self.send_moving || self.send_nonlocal_moving;
        self.send_incoming = self.send_incoming || self.send_nonlocal_incoming;
        self.send_hits = self.send_hits || self.send_all;

        self.send_remove
            || self.send_incoming
            || self.send_public_stats
            || self.send_hits
            || self.send_moving
            || self.send_opl_update
            || self.send_hair
            || self.send_facial_hair
            || self.send_healthbar_poison
            || self.send_healthbar_yellow
    }
}

/// `OutgoingVirtualHairPackets.CreateHairEquipUpdatePacket`: 0x2E.
pub fn create_hair_equip_update_packet(
    w: &World,
    m: MobileId,
    hair_serial: Serial,
    item_id: i32,
    hue: i32,
    layer: Layer,
) -> Vec<u8> {
    let (serial, solid) = w
        .mobile(m)
        .map_or((Serial::ZERO, -1), |m| (m.serial, m.solid_hue_override));
    let mut pk = Pk::new(15);
    pk.u8(0x2E)
        .serial(hair_serial)
        .i16(item_id as i16)
        .u8(0)
        .u8(layer.0)
        .serial(serial)
        .i16(if solid >= 0 { solid } else { hue } as i16);
    pk.finish()
}

/// `OutgoingVirtualHairPackets.CreateRemoveHairPacket`: 0x1D.
pub fn create_remove_hair_packet(hair_serial: Serial) -> Vec<u8> {
    let mut pk = Pk::new(5);
    pk.u8(0x1D).serial(hair_serial);
    pk.finish()
}

/// The hair or facial hair packet `ProcessDelta` sends: the remove packet or the equip update.
fn hair_packet(w: &World, id: MobileId, facial: bool, remove: bool) -> Vec<u8> {
    let Some(m) = w.mobile(id) else {
        return Vec::new();
    };
    let (serial, item_id, hue, layer) = if facial {
        (
            m.facial_hair_serial,
            m.facial_hair_item_id,
            m.facial_hair_hue,
            Layer::FacialHair,
        )
    } else {
        (m.hair_serial, m.hair_item_id, m.hair_hue, Layer::Hair)
    };
    if remove {
        create_remove_hair_packet(serial)
    } else {
        create_hair_equip_update_packet(w, id, serial, item_id, hue, layer)
    }
}

pub fn process_delta(w: &mut World, id: MobileId) {
    let Some(m) = w.mobile(id) else { return };
    let delta = m.delta_flags;
    if delta == MobileDelta::NONE {
        return;
    }

    let ty = m.ty;
    let (hair_item_id, facial_hair_item_id) = (m.hair_item_id, m.facial_hair_item_id);
    let mut p = DeltaPlan::new(
        delta,
        hair_item_id,
        facial_hair_item_id,
        crate::item::deps::opl_enabled(w),
    );

    if let Some(m) = w.mobile_mut(id) {
        m.delta_flags = MobileDelta::NONE;
        m.in_delta_queue = false;
    }

    super::props_b::hair_serial(w, id); // `var hairSerial = HairSerial;` assigns the virtual serial
    super::props_b::facial_hair_serial(w, id);
    let hair = hair_packet(w, id, false, p.remove_hair);
    let facial_hair = hair_packet(w, id, true, p.remove_facial_hair);
    let mut moving_cache = MobileMovingCache::default();

    let our_state = w.mobile(id).and_then(|m| m.net_state);
    if let Some(ns) = our_state {
        let sa = w.net_state(ns).is_some_and(|d| d.caps.stygian_abyss());

        if p.send_update {
            crate::net_glue::reset_movement_state(w, ns);
            packets::mobile::send_mobile_update(w, ns, id);
        }

        if p.send_incoming {
            packets::mobile::send_mobile_incoming(w, ns, id, id);
        }

        if p.send_moving || !sa && (p.send_healthbar_poison || p.send_healthbar_yellow) {
            packets::mobile::send_mobile_moving_using_cache_notoriety(
                w,
                ns,
                &mut moving_cache,
                id,
                id,
            );
        }

        if sa {
            if p.send_healthbar_poison {
                packets::mobile::send_mobile_healthbar(w, ns, id, Healthbar::Poison);
            }

            if p.send_healthbar_yellow {
                packets::mobile::send_mobile_healthbar(w, ns, id, Healthbar::Yellow);
            }
        }

        if p.send_public_stats || p.send_private_stats {
            packets::mobile::send_mobile_status_self(w, ns, id);
        } else if p.send_all {
            packets::mobile::send_mobile_attributes(w, ns, id, false);
        } else if p.send_any {
            if p.send_hits {
                packets::mobile::send_mobile_hits(w, ns, id, false);
            }

            if p.send_mana {
                packets::mobile::send_mobile_mana(w, ns, id, false);
            }

            if p.send_stam {
                packets::mobile::send_mobile_stam(w, ns, id, false);
            }
        }

        if p.send_stam || p.send_mana {
            crate::mobile::part2::deps::party_on_mana_stam_changed(w, id, p.send_mana, p.send_stam);
        }

        if p.send_hair {
            w.send(ns, &hair);
        }

        if p.send_facial_hair {
            w.send(ns, &facial_hair);
        }

        if p.send_opl_update {
            (ty.send_opl_packet_to)(w, id, ns);
        }
    }

    // TODO: Is it even valid to send packets to our state while we have a null map? Look into failing fast
    let Some(m) = w.mobile(id) else { return };
    let (Some(map), loc) = (m.map, m.location) else {
        return;
    };

    if !p.nonlocal() {
        return;
    }

    let range = w.maps.global_max_update_range;
    let states: Vec<NetStateId> =
        crate::map::enumerate::clients_in_range(w, Some(map), loc.x, loc.y, range).collect();
    let serial = id.0;
    let mut moving_cache = MobileMovingCache::default();

    for ns in states {
        let Some(beholder) = w.net_mobile(ns) else {
            continue;
        };
        let Some(bty) = w.mobile(beholder).map(|b| b.ty) else {
            continue;
        };
        if beholder == id || !(bty.can_see_mobile)(w, beholder, id) {
            continue;
        }
        let sa = w.net_state(ns).is_some_and(|d| d.caps.stygian_abyss());

        if p.send_remove {
            packets::entity::send_remove_entity(w, ns, serial);
        }

        if p.send_incoming {
            packets::mobile::send_mobile_incoming(w, ns, beholder, id);

            if (ty.is_dead_bonded_pet)(w, id) {
                packets::mobile::send_bonded_status(w, ns, serial, true);
            }
        }

        if p.send_moving || !sa && (p.send_healthbar_poison || p.send_healthbar_yellow) {
            packets::mobile::send_mobile_moving_using_cache_notoriety(
                w,
                ns,
                &mut moving_cache,
                beholder,
                id,
            );
        }

        if sa {
            if p.send_healthbar_poison {
                packets::mobile::send_mobile_healthbar(w, ns, id, Healthbar::Poison);
            }

            if p.send_healthbar_yellow {
                packets::mobile::send_mobile_healthbar(w, ns, id, Healthbar::Yellow);
            }
        }

        if p.send_public_stats {
            let renamable = (ty.can_be_renamed_by)(w, id, beholder);
            packets::mobile::send_mobile_status_compact(w, ns, id, renamable);
        } else if p.send_hits {
            packets::mobile::send_mobile_hits(w, ns, id, true);
        }

        if p.send_hair {
            w.send(ns, &hair);
        }

        if p.send_facial_hair {
            w.send(ns, &facial_hair);
        }

        (ty.send_opl_packet_to)(w, id, ns);
    }
}

/// The base values, then `m_Region?.AlterLightLevel`.
pub fn compute_light_levels(w: &mut World, id: MobileId) -> (i32, i32) {
    let Some(m) = w.mobile(id) else { return (0, 0) };
    let region_id = m.region;
    let (mut global, mut personal) = (m.ty.compute_base_light_levels)(w, id);

    if let Some(r) = w.region(region_id) {
        let rty = r.ty;
        (rty.alter_light_level)(w, region_id, id, &mut global, &mut personal);
    }
    (global, personal)
}

pub fn update_resistances(w: &mut World, id: MobileId) {
    let Some(m) = w.mobile_mut(id) else { return };
    let mut delta = false;

    if m.resistances.is_empty() {
        m.resistances = vec![i32::MIN; 5];
    } else {
        for r in m.resistances.iter_mut() {
            if *r != i32::MIN {
                *r = i32::MIN;
                delta = true;
            }
        }
    }

    if delta {
        let ty = m.ty;
        (ty.delta)(w, id, MobileDelta::RESISTANCES);
    }
}

pub fn get_resistance(w: &mut World, id: MobileId, ty: ResistanceType) -> i32 {
    let Some(m) = w.mobile(id) else { return 0 };
    let v = ty as usize;
    if v >= m.resistances.len() {
        return 0;
    }
    let res = m.resistances[v];
    if res == i32::MIN {
        let t = m.ty;
        (t.compute_resistances)(w, id);
        return w.mobile(id).map_or(0, |m| m.resistances[v]);
    }
    res
}

/// `GetResistance` for a caller that holds only `&World` (the `[CommandProperty]` getters): a stale slot is computed
/// again and not stored, so the answer is the same as [`get_resistance`]'s.
pub fn peek_resistance(w: &World, id: MobileId, ty: ResistanceType) -> i32 {
    let Some(m) = w.mobile(id) else { return 0 };
    let v = ty as usize;
    match m.resistances.get(v) {
        None => 0,
        Some(&i32::MIN) => calc_resistances(w, id).map_or(0, |r| r[v]),
        Some(&r) => r,
    }
}

pub fn add_resistance_mod(w: &mut World, id: MobileId, mod_: crate::mods::ResistanceMod) {
    let Some(m) = w.mobile_mut(id) else { return };
    m.resistance_mods.push(mod_);
    let ty = m.ty;
    (ty.update_resistances)(w, id);
}

/// The index of the first mod named `name`.
pub fn get_resistance_mod(w: &World, id: MobileId, name: &str) -> Option<usize> {
    w.mobile(id)?
        .resistance_mods
        .iter()
        .position(|m| m.name.as_deref() == Some(name))
}

pub fn remove_resistance_mod_by_name(w: &mut World, id: MobileId, name: &str) {
    let Some(m) = w.mobile_mut(id) else { return };
    if m.resistance_mods.is_empty() {
        return; // `_resistanceMods == null`
    }
    m.resistance_mods
        .retain(|x| x.name.as_deref() != Some(name));
    let ty = m.ty;
    (ty.update_resistances)(w, id);
}

pub fn remove_resistance_mod(w: &mut World, id: MobileId, mod_: crate::mods::ModId) {
    let Some(m) = w.mobile_mut(id) else { return };
    if m.resistance_mods.is_empty() {
        return; // `_resistanceMods == null`
    }
    if let Some(i) = m.resistance_mods.iter().position(|x| x.id == mod_) {
        m.resistance_mods.remove(i);
    }
    let ty = m.ty;
    (ty.update_resistances)(w, id);
}

pub fn compute_resistances(w: &mut World, id: MobileId) {
    if let Some(res) = calc_resistances(w, id)
        && let Some(m) = w.mobile_mut(id)
    {
        m.resistances = res.to_vec();
    }
}

/// The body of `ComputeResistances` as a value.
fn calc_resistances(w: &World, id: MobileId) -> Option<[i32; 5]> {
    let m = w.mobile(id)?;
    let ty = m.ty;

    let mut res = [0i32; 5];
    res[0] += (ty.base_physical_resistance)(w, id);
    res[1] += (ty.base_fire_resistance)(w, id);
    res[2] += (ty.base_cold_resistance)(w, id);
    res[3] += (ty.base_poison_resistance)(w, id);
    res[4] += (ty.base_energy_resistance)(w, id);

    for m in &m.resistance_mods {
        let v = m.ty as usize;
        if v < res.len() {
            res[v] += m.offset;
        }
    }

    let items = m.items.clone();
    for item in items {
        let Some(it) = w.item(item) else { continue };
        let it_ty = it.ty;
        if (it_ty.check_property_conflict)(w, item, id) {
            continue;
        }
        res[0] += (it_ty.physical_resistance)(w, item);
        res[1] += (it_ty.fire_resistance)(w, item);
        res[2] += (it_ty.cold_resistance)(w, item);
        res[3] += (it_ty.poison_resistance)(w, item);
        res[4] += (it_ty.energy_resistance)(w, item);
    }

    let types = [
        ResistanceType::Physical,
        ResistanceType::Fire,
        ResistanceType::Cold,
        ResistanceType::Poison,
        ResistanceType::Energy,
    ];
    for (i, t) in types.into_iter().enumerate() {
        let min = (ty.get_min_resistance)(w, id, t);
        let mut max = (ty.get_max_resistance)(w, id, t);
        if max < min {
            max = min;
        }
        if res[i] > max {
            res[i] = max;
        } else if res[i] < min {
            res[i] = min;
        }
    }

    Some(res)
}

pub fn get_max_resistance(w: &World, id: MobileId, _ty: ResistanceType) -> i32 {
    if w.mobile(id).is_some_and(|m| m.player) {
        crate::mobile::part1::props_a::max_player_resistance(w)
    } else {
        i32::MAX
    }
}

pub fn get_aos_status(w: &World, id: MobileId, index: i32) -> i32 {
    crate::mobile::part1::props_b::aos_status_handler(w).map_or(0, |h| h(w, id, index))
}

/// `ns?.Send(PropertyList.Buffer)`.
pub fn send_properties_to(w: &mut World, id: MobileId, ns: NetStateId) {
    super::props_b::ensure_property_list(w, id);
    let Some(buf) = w
        .mobile(id)
        .and_then(|m| m.property_list.as_ref().map(|p| p.buffer().to_vec()))
    else {
        return;
    };
    w.send(ns, &buf);
}

pub fn on_aos_single_click(w: &mut World, id: MobileId, from: MobileId) {
    super::props_b::ensure_property_list(w, id);
    let Some(m) = w.mobile(id) else { return };
    let Some(opl) = m.property_list.as_ref() else {
        return;
    };
    if opl.header <= 0 {
        return;
    }
    let header = opl.header;
    let header_args = opl.header_args.clone().unwrap_or_default();

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

    let (serial, body) = (m.serial, m.cur_body().0);
    let name = (m.ty.name)(w, id).unwrap_or_default();
    let Some(ns) = w.mobile(from).and_then(|m| m.net_state) else {
        return;
    };
    crate::packets::message::send_message_localized(
        w,
        ns,
        serial,
        body,
        crate::packets::message::MessageType::LABEL,
        hue,
        3,
        header,
        &name,
        &header_args,
    );
}

/// `Body.IsHuman` over the world's body table.
fn body_is_human(w: &World, body: Body) -> bool {
    w.resources
        .get::<BodyTable>()
        .is_some_and(|t| t.is_human(body))
}

pub fn add_name_properties(w: &World, id: MobileId, list: &mut PropertyList) {
    let Some(m) = w.mobile(id) else { return };
    let ty = m.ty;

    let prefix =
        if (ty.show_fame_title)(w, id) && (m.player || body_is_human(w, m.body)) && m.fame >= 10000
        {
            if m.female { "Lady " } else { "Lord " }
        } else {
            " "
        };

    let guild: Option<(&str, &str, GuildType)> = m
        .guild
        .and_then(|g| crate::guild::guild(w, g))
        .map(|g| (g.abbreviation.as_str(), g.name.as_str(), g.ty));

    let title = m.title.as_deref().unwrap_or("");
    let has_title = (ty.property_title)(w, id) && !title.is_empty();
    let has_guild = guild.is_some() && (m.player || m.display_guild_title);

    let suffix = match (has_title, has_guild, guild) {
        (true, true, Some((abbr, _, _))) => format!(" {title} [{}]", utility::fix_html(abbr)),
        (true, _, _) => format!(" {title}"),
        (false, true, Some((abbr, _, _))) => format!(" [{}]", utility::fix_html(abbr)),
        _ => " ".to_owned(),
    };

    let name = (ty.name)(w, id);
    list.add_number_fmt(
        1050045, // ~1_PREFIX~~2_NAME~~3_SUFFIX~
        format_args!(
            "{prefix}\t{}\t{}",
            name.as_deref().unwrap_or(" "),
            (ty.apply_name_suffix)(w, id, &suffix)
        ),
    );

    if let Some((_, gname, gtype)) = guild
        && (m.display_guild_title || m.player && gtype != GuildType::Regular)
    {
        let type_ = GUILD_TYPES.get(gtype as usize).copied().unwrap_or("");

        let guild_title = m.guild_title.as_deref().map_or("", str::trim);

        if !guild_title.is_empty() {
            if (ty.new_guild_display)(w, id) {
                list.add_fmt(format_args!(
                    "{}, {}",
                    utility::fix_html(guild_title),
                    utility::fix_html(gname)
                ));
            } else {
                list.add_fmt(format_args!(
                    "{}, {} Guild{type_}",
                    utility::fix_html(guild_title),
                    utility::fix_html(gname)
                ));
            }
        } else {
            list.add_text(&utility::fix_html(gname));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mobile::{MOBILE_BASE, StatLockType};
    use muo_io::SaveWriter;
    use muo_primitives::{DateTime, Direction};

    fn w_with_mobile() -> (World, MobileId) {
        let mut w = World::new();
        let id = w.new_mobile(&MOBILE_BASE);
        (w, id)
    }

    #[test]
    fn serialize_golden_bytes() {
        let (mut w, id) = w_with_mobile();
        {
            let m = w.mobile_mut(id).unwrap();
            m.name = Some("Bob".into());
            m.location = Point3D::new(1, 2, 3);
            m.map = Some(MapId(1));
            m.body = Body(400);
            m.direction = Direction::EAST;
            m.hair_item_id = 0x203B;
            m.hair_hue = 7;
            m.str = 50;
            m.str_lock = StatLockType::Locked;
            m.last_str_gain = DateTime::from_ticks(5);
            m.skills = None;
        }
        let mut out = SaveWriter::new(true);
        serialize(&w, id, &mut out);
        let got = out.into_vec();

        let i = |v: i32| v.to_le_bytes().to_vec();
        let mut want: Vec<u8> = Vec::new();
        want.extend(i(38));
        want.extend(5i64.to_le_bytes());
        want.extend(0i64.to_le_bytes());
        want.extend(0i64.to_le_bytes());
        want.push(0x01); // hair flag
        want.extend(i(0x203B));
        want.extend(i(7));
        want.push(0xFF); // race (none loaded)
        want.extend(i(0)); // tithing
        want.extend(i(-1)); // corpse
        want.push(0); // cant walk
        want.extend(i(0)); // thirst
        want.extend(i(0)); // bac
        want.extend(i(5)); // followers max
        want.extend(i(0)); // magic absorb
        want.extend(i(-1)); // guild fealty
        want.extend(i(-1)); // guild
        want.extend([0, 0, 0]); // display guild title, can swim, squelched
        want.extend(i(-1)); // holding
        want.extend(i(0)); // virtual armor
        want.extend(i(0)); // base sound
        want.extend([0, 0]); // disarm, stun
        want.extend(i(225)); // stat cap
        want.extend(i(-1)); // name hue
        want.extend(i(0)); // hunger
        want.extend(i(1));
        want.extend(i(2));
        want.extend(i(3));
        want.extend(i(400)); // body
        want.extend([1, 3, b'B', b'o', b'b']); // name
        want.push(0); // guild title (null)
        want.push(0); // criminal
        want.extend(i(0)); // kills
        want.extend([0; 16]); // speech, emote, whisper, yell hues
        want.push(0); // language (null)
        want.extend([0, 0, 0]); // female, warmode, hidden
        want.push(2); // direction
        want.extend(i(0)); // hue
        want.extend(i(50));
        want.extend([0; 4 * 5]); // dex, int, hits, stam, mana
        want.push(1); // map
        want.push(0); // blessed
        want.extend(i(0)); // fame
        want.extend(i(0)); // karma
        want.push(0); // access level
        want.extend(i(3)); // skills: version, cap, count
        want.extend(i(7000));
        want.extend(i(0));
        want.extend(i(0)); // items
        want.push(0); // player
        want.push(0); // title (null)
        want.push(0); // profile (null)
        want.push(0); // profile locked
        want.push(1); // auto page notify
        want.extend([0; 12]); // logout location
        want.push(0xFF); // logout map
        want.extend([2, 0, 0]); // stat locks
        assert_eq!(got, want);
    }

    #[test]
    fn delta_plan_branches() {
        let p = DeltaPlan::new(MobileDelta::ATTRIBUTES, 0, 0, true);
        assert!(p.send_all && p.send_any && !p.send_hits);
        let p = DeltaPlan::new(MobileDelta::HITS | MobileDelta::NAME, 0, 0, true);
        assert!(p.send_public_stats && !p.send_hits && !p.send_any && !p.send_all);
        let p = DeltaPlan::new(MobileDelta::HAIR, 0, 5, true);
        assert!(p.send_hair && p.remove_hair && !p.send_facial_hair);
        let mut p = DeltaPlan::new(MobileDelta::DIRECTION, 0, 0, true);
        assert!(p.send_update && p.send_nonlocal_moving && !p.send_moving);
        assert!(p.nonlocal() && p.send_moving);
        let mut p = DeltaPlan::new(MobileDelta::STAT, 0, 0, true);
        assert!(p.send_private_stats && !p.nonlocal());
    }

    #[test]
    fn resistances_compute_and_clamp() {
        let (mut w, id) = w_with_mobile();
        let ty = w.mobile(id).unwrap().ty;
        (ty.update_resistances)(&mut w, id);
        assert_eq!(w.mobile(id).unwrap().resistances, vec![i32::MIN; 5]);
        w.mobile_mut(id).unwrap().player = true;
        assert_eq!(peek_resistance(&w, id, ResistanceType::Fire), 0);
        assert_eq!(
            w.mobile(id).unwrap().resistances,
            vec![i32::MIN; 5],
            "a peek stores nothing"
        );
        assert_eq!(get_resistance(&mut w, id, ResistanceType::Fire), 0);
        // `GetResistance` runs `ComputeResistances`, which fills every slot.
        assert_eq!(w.mobile(id).unwrap().resistances, vec![0; 5]);
        w.mobile_mut(id).unwrap().resistances = vec![i32::MIN; 5];
        (ty.compute_resistances)(&mut w, id);
        assert_eq!(w.mobile(id).unwrap().resistances, vec![0; 5]);
        assert_eq!(get_max_resistance(&w, id, ResistanceType::Fire), 70);
    }

    #[test]
    fn removing_a_mod_when_there_are_none_keeps_resistances() {
        let (mut w, id) = w_with_mobile();
        w.mobile_mut(id).unwrap().resistances = vec![5; 5];
        remove_resistance_mod_by_name(&mut w, id, "x");
        assert_eq!(w.mobile(id).unwrap().resistances, vec![5; 5]);
    }

    thread_local!(static PARTY_CALLS: std::cell::Cell<(u32, u32)> = const { std::cell::Cell::new((0, 0)) });

    #[test]
    fn party_hears_only_the_changed_stat() {
        use crate::mobile::part2::deps::{PartyDeps, party_on_mana_stam_changed};
        let (mut w, id) = w_with_mobile();
        w.resources.insert(PartyDeps {
            on_stats_query: None,
            on_mana_changed: Some(|_, _, _| {
                PARTY_CALLS.with(|c| c.set((c.get().0 + 1, c.get().1)))
            }),
            on_stam_changed: Some(|_, _, _| {
                PARTY_CALLS.with(|c| c.set((c.get().0, c.get().1 + 1)))
            }),
        });
        w.mobile_mut(id).unwrap().party = Some(crate::interfaces::PartyId(1));
        party_on_mana_stam_changed(&mut w, id, true, false);
        assert_eq!(PARTY_CALLS.with(|c| c.get()), (1, 0));
    }
}
