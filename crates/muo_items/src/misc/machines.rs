//! `EffectController`, the communication crystals (`BroadcastCrystal`, `ReceiverCrystal`), `PowerGenerator`,
//! `ControlPanel` and `ProjectedItem`.
//!
//! `ProjectedItem`'s effect packet is `HuedEffect` of type `FixedXYZ` built directly; the item's
//! `ProcessDelta`/`SendInfoTo` overrides run through the matching `ItemType` fields.

use muo_content::gumps::base::{DynamicGumpLogic, DynamicGumpObj, RelayInfo};
use muo_content::gumps::builder::DynamicGumpBuilder;
use muo_content::gumps::send_gump_to_mobile;
use muo_content::items::addon::{self, ADDON_COMPONENT, BASE_ADDON};
use muo_content::misc::aos_helpers::{DamageOptions, DamageSplit, damage};
use muo_content::props::{Kind, PropDef, Value};
use muo_content::utilities::types::{Numeric, PropType};

use muo_primitives::{DateTime, MessageType, Point2D, Point3D, TimeSpan, utility};
use muo_server::World;
use muo_server::effects::{self, EffectLayer};
use muo_server::ext::Ext;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, base, deps, messages, ops};
use muo_server::mobile::AccessLevel;
use muo_server::net_state::NetStateId;
use muo_server::packets::message::MessageType as ItemMessage;
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::target::{self, TargetFlags, Targeted};
use muo_server::timer::{self, TimerHandle};
use muo_server::type_registry::TypeRegistry;

const GM: AccessLevel = AccessLevel::GameMaster;
const INT: Kind = Kind::Plain(PropType::Numeric(Numeric::Int));
const BOOL: Kind = Kind::Plain(PropType::Bool);

fn item_of(e: EntityId) -> Option<ItemId> {
    match e {
        EntityId::Item(i) => Some(i),
        _ => None,
    }
}

fn delete_item(w: &mut World, id: ItemId) {
    if let Some(ty) = w.item(id).map(|i| i.ty) {
        (ty.delete)(w, id);
    }
}

fn msg(w: &mut World, to: MobileId, number: i32) {
    deps::send_localized_message(w, to, number);
}

fn cant_reach(w: &mut World, from: MobileId) {
    muo_server::mobile::part2::messages::local_overhead_message_number(
        w,
        from,
        MessageType::REGULAR,
        0x3B2,
        1019045,
        "",
    );
}

fn set_item_graphic(w: &mut World, id: ItemId, graphic: i32) {
    if let Some(ty) = w.item(id).map(|i| i.ty) {
        (ty.set_item_id)(w, id, graphic);
    }
}

// ---- EffectController ----

/// `ECEffectType`.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
#[repr(i32)]
pub enum EcEffectType {
    #[default]
    None = 0,
    Moving = 1,
    Location = 2,
    Target = 3,
    Lightning = 4,
}

impl EcEffectType {
    fn from_i32(n: i32) -> Self {
        match n {
            1 => Self::Moving,
            2 => Self::Location,
            3 => Self::Target,
            4 => Self::Lightning,
            _ => Self::None,
        }
    }
}

/// `EffectTriggerType`.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
#[repr(i32)]
pub enum EffectTriggerType {
    None = 0,
    #[default]
    Sequenced = 1,
    DoubleClick = 2,
    InRange = 3,
}

impl EffectTriggerType {
    fn from_i32(n: i32) -> Self {
        match n {
            1 => Self::Sequenced,
            2 => Self::DoubleClick,
            3 => Self::InRange,
            _ => Self::None,
        }
    }
}

/// The fields of `EffectController`.
#[derive(Clone, Debug)]
pub struct EcData {
    pub effect_delay: TimeSpan,
    pub trigger_delay: TimeSpan,
    pub sound_delay: TimeSpan,
    pub source: Option<EntityId>,
    pub target: Option<EntityId>,
    pub sequence: Option<ItemId>,
    pub fixed_direction: bool,
    pub explodes: bool,
    pub play_sound_at_trigger: bool,
    pub effect_type: EcEffectType,
    pub effect_layer: u8,
    pub trigger_type: EffectTriggerType,
    pub effect_item_id: i32,
    pub effect_hue: i32,
    pub render_mode: i32,
    pub speed: i32,
    pub duration: i32,
    pub particle_effect: i32,
    pub explode_particle_effect: i32,
    pub explode_sound: i32,
    pub unknown: i32,
    pub sound_id: i32,
    pub trigger_range: i32,
}

impl Default for EcData {
    fn default() -> Self {
        Self {
            effect_delay: TimeSpan::ZERO,
            trigger_delay: TimeSpan::ZERO,
            sound_delay: TimeSpan::ZERO,
            source: None,
            target: None,
            sequence: None,
            fixed_direction: false,
            explodes: false,
            play_sound_at_trigger: false,
            effect_type: EcEffectType::None,
            effect_layer: 255,
            trigger_type: EffectTriggerType::Sequenced,
            effect_item_id: 0,
            effect_hue: 0,
            render_mode: 0,
            speed: 0,
            duration: 0,
            particle_effect: 0,
            explode_particle_effect: 0,
            explode_sound: 0,
            unknown: 0,
            sound_id: 0,
            trigger_range: 0,
        }
    }
}

fn ec(w: &World, id: ItemId) -> EcData {
    w.item(id)
        .and_then(|i| i.ext.get::<EcData>())
        .cloned()
        .unwrap_or_default()
}

fn ec_edit(w: &mut World, id: ItemId, f: impl FnOnce(&mut EcData)) {
    if let Some(i) = w.item_mut(id) {
        if i.ext.get::<EcData>().is_none() {
            i.ext.insert(EcData::default());
        }
        if let Some(d) = i.ext.get_mut::<EcData>() {
            f(d);
        }
    }
}

/// `EffectController()`.
pub fn effect_controller_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = crate::simple::from_template(w, ty, "Server.Items.EffectController");
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(EcData::default()); // TriggerType = Sequenced; EffectLayer = (EffectLayer)255
    }
    id
}

/// `SoundId = value`.
pub fn effect_controller_set_sound_id(w: &mut World, id: ItemId, value: i32) {
    ec_edit(w, id, |d| d.sound_id = value);
}

/// `TriggerType = value`.
pub fn effect_controller_set_trigger_type(w: &mut World, id: ItemId, value: EffectTriggerType) {
    ec_edit(w, id, |d| d.trigger_type = value);
}

/// `TriggerRange = value`.
pub fn effect_controller_set_trigger_range(w: &mut World, id: ItemId, value: i32) {
    ec_edit(w, id, |d| d.trigger_range = value);
}

fn entity_location(w: &World, e: EntityId) -> (Point3D, Option<muo_server::map::MapId>) {
    match e {
        EntityId::Item(i) => w
            .item(i)
            .map_or((Point3D::default(), None), |x| (x.location, x.map)),
        EntityId::Mobile(m) => w
            .mobile(m)
            .map_or((Point3D::default(), None), |x| (x.location, x.map)),
    }
}

fn ec_play_sound(w: &mut World, id: ItemId, trigger: EntityId) {
    let d = ec(w, id);
    let ent = if d.play_sound_at_trigger {
        trigger
    } else {
        EntityId::Item(id)
    };
    let (loc, map) = match ent {
        EntityId::Item(i) => (ops::get_world_location(w, i), w.item(i).and_then(|x| x.map)),
        EntityId::Mobile(_) => entity_location(w, ent),
    };
    effects::play_sound(w, loc, map, d.sound_id);
}

fn ec_do_effect(w: &mut World, id: ItemId, trigger: EntityId) {
    let Some(i) = w.item(id) else { return };
    let d = ec(w, id);
    if i.deleted() || d.trigger_type == EffectTriggerType::None {
        return;
    }
    if let EntityId::Mobile(m) = trigger
        && w.mobile(m)
            .is_some_and(|x| x.hidden && x.access_level > AccessLevel::Player)
    {
        return;
    }
    if d.sound_id > 0 {
        timer::start(w, d.sound_delay, TimeSpan::ZERO, 1, move |w| {
            ec_play_sound(w, id, trigger)
        });
    }
    if let Some(seq) = d.sequence {
        timer::start(w, d.trigger_delay, TimeSpan::ZERO, 1, move |w| {
            ec_do_effect(w, seq, trigger)
        });
    }
    if d.effect_type != EcEffectType::None {
        timer::start(w, d.effect_delay, TimeSpan::ZERO, 1, move |w| {
            ec_internal_do_effect(w, id, trigger)
        });
    }
}

fn ec_internal_do_effect(w: &mut World, id: ItemId, trigger: EntityId) {
    let d = ec(w, id);
    let mut from = d.source.unwrap_or(trigger);
    let mut to = d.target.unwrap_or(trigger);
    let this = EntityId::Item(id);
    match d.effect_type {
        EcEffectType::Lightning => {
            effects::send_bolt_effect(w, effects::Targeted::from(from), false, d.effect_hue)
        }
        EcEffectType::Location => {
            let (loc, map) = entity_location(w, from);
            let e = muo_server::item::effect_item::create_targeted(
                w,
                loc,
                map,
                muo_server::item::effect_item::DEFAULT_DURATION,
            );
            effects::send_location_particles_full(
                w,
                e,
                d.effect_item_id,
                d.speed,
                d.duration,
                d.effect_hue,
                d.render_mode,
                d.particle_effect,
                d.unknown,
            );
        }
        EcEffectType::Moving => {
            if from == this {
                let (loc, map) = entity_location(w, from);
                from = EntityId::Item(muo_server::item::effect_item::create(
                    w,
                    loc,
                    map,
                    muo_server::item::effect_item::DEFAULT_DURATION,
                ));
            }
            if to == this {
                let (loc, map) = entity_location(w, to);
                to = EntityId::Item(muo_server::item::effect_item::create(
                    w,
                    loc,
                    map,
                    muo_server::item::effect_item::DEFAULT_DURATION,
                ));
            }
            effects::send_moving_particles_layer(
                w,
                effects::Targeted::from(from),
                effects::Targeted::from(to),
                d.effect_item_id,
                d.speed,
                d.duration,
                d.fixed_direction,
                d.explodes,
                d.effect_hue,
                d.render_mode,
                d.particle_effect,
                d.explode_particle_effect,
                d.explode_sound,
                i32::from(d.effect_layer),
                d.unknown,
            );
        }
        EcEffectType::Target => effects::send_target_particles_full(
            w,
            effects::Targeted::from(from),
            d.effect_item_id,
            d.speed,
            d.duration,
            d.effect_hue,
            d.render_mode,
            d.particle_effect,
            EffectLayer(d.effect_layer),
            d.unknown,
        ),
        EcEffectType::None => {}
    }
}

fn ec_of(w: &World, e: EntityId) -> EcData {
    item_of(e).map_or_else(EcData::default, |i| ec(w, i))
}

macro_rules! ec_int {
    ($n:literal, $f:ident) => {
        PropDef {
            name: $n,
            kind: INT,
            read: GM,
            write: GM,
            get: |w, e| Value::Int(i64::from(ec_of(w, e).$f)),
            set: Some(|w, e, v| {
                if let Some(i) = item_of(e) {
                    ec_edit(w, i, |d| d.$f = v.i32());
                }
                Ok(())
            }),
        }
    };
}

macro_rules! ec_bool {
    ($n:literal, $f:ident) => {
        PropDef {
            name: $n,
            kind: BOOL,
            read: GM,
            write: GM,
            get: |w, e| Value::Bool(ec_of(w, e).$f),
            set: Some(|w, e, v| {
                if let Some(i) = item_of(e) {
                    ec_edit(w, i, |d| d.$f = v.bool());
                }
                Ok(())
            }),
        }
    };
}

macro_rules! ec_span {
    ($n:literal, $f:ident) => {
        PropDef {
            name: $n,
            kind: Kind::TimeSpan,
            read: GM,
            write: GM,
            get: |w, e| Value::TimeSpan(ec_of(w, e).$f),
            set: Some(|w, e, v| {
                if let (Some(i), Value::TimeSpan(t)) = (item_of(e), v) {
                    ec_edit(w, i, |d| d.$f = t);
                }
                Ok(())
            }),
        }
    };
}

const EC_EFFECT_TYPES: &[(&str, i64)] = &[
    ("None", 0),
    ("Moving", 1),
    ("Location", 2),
    ("Target", 3),
    ("Lightning", 4),
];
const EC_TRIGGER_TYPES: &[(&str, i64)] = &[
    ("None", 0),
    ("Sequenced", 1),
    ("DoubleClick", 2),
    ("InRange", 3),
];

static EFFECT_CONTROLLER_PROPS: &[PropDef] = &[
    ec_span!("EffectDelay", effect_delay),
    ec_span!("TriggerDelay", trigger_delay),
    ec_span!("SoundDelay", sound_delay),
    PropDef {
        name: "Sequence",
        kind: Kind::Plain(PropType::Entity),
        read: GM,
        write: GM,
        get: |w, e| Value::Entity(ec_of(w, e).sequence.map(EntityId::Item)),
        set: Some(|w, e, v| {
            let seq = v
                .entity()
                .and_then(item_of)
                .filter(|s| is_effect_controller(w, *s));
            if let Some(i) = item_of(e) {
                ec_edit(w, i, |d| d.sequence = seq);
            }
            Ok(())
        }),
    },
    ec_bool!("FixedDirection", fixed_direction),
    ec_bool!("Explodes", explodes),
    ec_bool!("PlaySoundAtTrigger", play_sound_at_trigger),
    PropDef {
        name: "EffectType",
        kind: Kind::Plain(PropType::Enum(EC_EFFECT_TYPES)),
        read: GM,
        write: GM,
        get: |w, e| Value::Enum(ec_of(w, e).effect_type as i64),
        set: Some(|w, e, v| {
            if let Some(i) = item_of(e) {
                ec_edit(w, i, |d| d.effect_type = EcEffectType::from_i32(v.i32()));
            }
            Ok(())
        }),
    },
    PropDef {
        name: "EffectLayer",
        kind: INT,
        read: GM,
        write: GM,
        get: |w, e| Value::Int(i64::from(ec_of(w, e).effect_layer)),
        set: Some(|w, e, v| {
            if let Some(i) = item_of(e) {
                ec_edit(w, i, |d| d.effect_layer = v.i32() as u8);
            }
            Ok(())
        }),
    },
    PropDef {
        name: "TriggerType",
        kind: Kind::Plain(PropType::Enum(EC_TRIGGER_TYPES)),
        read: GM,
        write: GM,
        get: |w, e| Value::Enum(ec_of(w, e).trigger_type as i64),
        set: Some(|w, e, v| {
            if let Some(i) = item_of(e) {
                ec_edit(w, i, |d| {
                    d.trigger_type = EffectTriggerType::from_i32(v.i32())
                });
            }
            Ok(())
        }),
    },
    ec_int!("EffectItemId", effect_item_id),
    ec_int!("EffectHue", effect_hue),
    ec_int!("RenderMode", render_mode),
    ec_int!("Speed", speed),
    ec_int!("Duration", duration),
    ec_int!("ParticleEffect", particle_effect),
    ec_int!("ExplodeParticleEffect", explode_particle_effect),
    ec_int!("ExplodeSound", explode_sound),
    ec_int!("Unknown", unknown),
    ec_int!("SoundId", sound_id),
    ec_int!("TriggerRange", trigger_range),
    PropDef {
        name: "SourceItem",
        kind: Kind::Plain(PropType::Entity),
        read: GM,
        write: GM,
        get: |w, e| {
            Value::Entity(
                ec_of(w, e)
                    .source
                    .filter(|s| matches!(s, EntityId::Item(_))),
            )
        },
        set: Some(|w, e, v| {
            if let Some(i) = item_of(e) {
                let s = v.entity().filter(|s| matches!(s, EntityId::Item(_)));
                ec_edit(w, i, |d| d.source = s);
            }
            Ok(())
        }),
    },
    PropDef {
        name: "SourceMobile",
        kind: Kind::Plain(PropType::Entity),
        read: GM,
        write: GM,
        get: |w, e| {
            Value::Entity(
                ec_of(w, e)
                    .source
                    .filter(|s| matches!(s, EntityId::Mobile(_))),
            )
        },
        set: Some(|w, e, v| {
            if let Some(i) = item_of(e) {
                let s = v.entity().filter(|s| matches!(s, EntityId::Mobile(_)));
                ec_edit(w, i, |d| d.source = s);
            }
            Ok(())
        }),
    },
    PropDef {
        name: "SourceNull",
        kind: BOOL,
        read: GM,
        write: GM,
        get: |w, e| Value::Bool(ec_of(w, e).source.is_none()),
        set: Some(|w, e, v| {
            if v.bool()
                && let Some(i) = item_of(e)
            {
                ec_edit(w, i, |d| d.source = None);
            }
            Ok(())
        }),
    },
    PropDef {
        name: "TargetItem",
        kind: Kind::Plain(PropType::Entity),
        read: GM,
        write: GM,
        get: |w, e| {
            Value::Entity(
                ec_of(w, e)
                    .target
                    .filter(|s| matches!(s, EntityId::Item(_))),
            )
        },
        set: Some(|w, e, v| {
            if let Some(i) = item_of(e) {
                let s = v.entity().filter(|s| matches!(s, EntityId::Item(_)));
                ec_edit(w, i, |d| d.target = s);
            }
            Ok(())
        }),
    },
    PropDef {
        name: "TargetMobile",
        kind: Kind::Plain(PropType::Entity),
        read: GM,
        write: GM,
        get: |w, e| {
            Value::Entity(
                ec_of(w, e)
                    .target
                    .filter(|s| matches!(s, EntityId::Mobile(_))),
            )
        },
        set: Some(|w, e, v| {
            if let Some(i) = item_of(e) {
                let s = v.entity().filter(|s| matches!(s, EntityId::Mobile(_)));
                ec_edit(w, i, |d| d.target = s);
            }
            Ok(())
        }),
    },
    PropDef {
        name: "TargetNull",
        kind: BOOL,
        read: GM,
        write: GM,
        get: |w, e| Value::Bool(ec_of(w, e).target.is_none()),
        set: Some(|w, e, v| {
            if v.bool()
                && let Some(i) = item_of(e)
            {
                ec_edit(w, i, |d| d.target = None);
            }
            Ok(())
        }),
    },
];

fn is_effect_controller(w: &World, id: ItemId) -> bool {
    w.item(id).is_some_and(|i| i.ty.is(&EFFECT_CONTROLLER))
}

item_leaf_raw! {
    /// `Server.Items.EffectController`.
    EFFECT_CONTROLLER = "Server.Items.EffectController" : ITEM_BASE;
    construct: Some(|w| effect_controller_new(w, &EFFECT_CONTROLLER)),
    default_name: crate::class_props::default_name,
    handles_on_movement: |w, id| ec(w, id).trigger_type == EffectTriggerType::InRange,
    on_double_click: |w, id, from| {
        if ec(w, id).trigger_type == EffectTriggerType::DoubleClick {
            ec_do_effect(w, id, EntityId::Mobile(from));
        }
    },
    on_movement: |w, id, m, old| {
        let d = ec(w, id);
        let Some(mm) = w.mobile(m) else { return };
        if mm.location == old || d.trigger_type != EffectTriggerType::InRange {
            return;
        }
        let at = ops::get_world_location(w, id);
        if utility::in_range_3d(at, mm.location, d.trigger_range) && !utility::in_range_3d(at, old, d.trigger_range) {
            ec_do_effect(w, id, EntityId::Mobile(m));
        }
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(0);
        let d = ec(w, id);
        out.write_time_span_ticks(d.effect_delay.ticks());
        out.write_time_span_ticks(d.trigger_delay.ticks());
        out.write_time_span_ticks(d.sound_delay.ticks());
        out.write_entity(w, d.source);
        out.write_entity(w, d.target);
        out.write_item(w, d.sequence);
        out.write_bool(d.fixed_direction);
        out.write_bool(d.explodes);
        out.write_bool(d.play_sound_at_trigger);
        out.write_enum(4, d.effect_type as i64);
        out.write_enum(4, i64::from(d.effect_layer)); // EffectLayer is an int enum: encoded
        out.write_enum(4, d.trigger_type as i64);
        for v in [
            d.effect_item_id, d.effect_hue, d.render_mode, d.speed, d.duration, d.particle_effect,
            d.explode_particle_effect, d.explode_sound, d.unknown, d.sound_id, d.trigger_range,
        ] {
            out.write_i32(v);
        }
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        r.read_i32()?;
        let d = EcData {
            effect_delay: TimeSpan::from_ticks(r.read_time_span_ticks()?),
            trigger_delay: TimeSpan::from_ticks(r.read_time_span_ticks()?),
            sound_delay: TimeSpan::from_ticks(r.read_time_span_ticks()?),
            source: r.read_entity(w)?,
            target: r.read_entity(w)?,
            sequence: r.read_item(w)?,
            fixed_direction: r.read_bool()?,
            explodes: r.read_bool()?,
            play_sound_at_trigger: r.read_bool()?,
            effect_type: EcEffectType::from_i32(r.read_enum(4)? as i32),
            effect_layer: r.read_enum(4)? as u8,
            trigger_type: EffectTriggerType::from_i32(r.read_enum(4)? as i32),
            effect_item_id: r.read_i32()?,
            effect_hue: r.read_i32()?,
            render_mode: r.read_i32()?,
            speed: r.read_i32()?,
            duration: r.read_i32()?,
            particle_effect: r.read_i32()?,
            explode_particle_effect: r.read_i32()?,
            explode_sound: r.read_i32()?,
            unknown: r.read_i32()?,
            sound_id: r.read_i32()?,
            trigger_range: r.read_i32()?,
        };
        if let Some(i) = w.item_mut(id) {
            i.ext = Ext::new(d);
        }
        Ok(())
    },
}

// ---- CommunicationCrystals ----

/// `CrystalRechargeInfo.Table`: the gem class and what it recharges.
const CRYSTAL_RECHARGE: [(&str, i32); 8] = [
    ("Server.Items.Citrine", 500),
    ("Server.Items.Amber", 500),
    ("Server.Items.Tourmaline", 750),
    ("Server.Items.Emerald", 1000),
    ("Server.Items.Sapphire", 1000),
    ("Server.Items.Amethyst", 1000),
    ("Server.Items.StarSapphire", 1250),
    ("Server.Items.Diamond", 2000),
];

fn recharge_of(w: &World, item: ItemId) -> Option<i32> {
    let name = w.item(item)?.ty.full_name;
    CRYSTAL_RECHARGE
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, a)| *a)
}

/// `BroadcastCrystal.MaxCharges`.
pub const MAX_CHARGES: i32 = 2000;

#[derive(Clone, Debug, Default)]
pub struct BroadcastData {
    pub charges: i32,
    pub receivers: Vec<Option<ItemId>>,
}

#[derive(Clone, Debug, Default)]
struct ReceiverData {
    sender: Option<ItemId>,
}

fn bc(w: &World, id: ItemId) -> BroadcastData {
    w.item(id)
        .and_then(|i| i.ext.get::<BroadcastData>())
        .cloned()
        .unwrap_or_default()
}

fn bc_edit(w: &mut World, id: ItemId, f: impl FnOnce(&mut BroadcastData)) {
    if let Some(d) = w
        .item_mut(id)
        .and_then(|i| i.ext.get_mut::<BroadcastData>())
    {
        f(d);
    }
}

fn rc_sender(w: &World, id: ItemId) -> Option<ItemId> {
    w.item(id)
        .and_then(|i| i.ext.get::<ReceiverData>())
        .and_then(|d| d.sender)
}

/// `BroadcastCrystal(charges)`.
pub fn broadcast_crystal_new(w: &mut World, ty: &'static ItemType, charges: i32) -> ItemId {
    let id = crate::simple::from_template(w, ty, "Server.Items.BroadcastCrystal");
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(BroadcastData {
            charges,
            receivers: Vec::new(),
        });
    }
    id
}

/// `ReceiverCrystal()`.
pub fn receiver_crystal_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = crate::simple::from_template(w, ty, "Server.Items.ReceiverCrystal");
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(ReceiverData::default());
    }
    id
}

fn bc_active(w: &World, id: ItemId) -> bool {
    w.item(id).is_some_and(|i| i.item_id == 0x1ECD)
}

fn bc_set_active(w: &mut World, id: ItemId, value: bool) {
    set_item_graphic(w, id, if value { 0x1ECD } else { 0x1ED0 });
    ops::invalidate_properties(w, id);
}

fn rc_active(w: &World, id: ItemId) -> bool {
    w.item(id).is_some_and(|i| i.item_id == 0x1ED1)
}

fn rc_set_active(w: &mut World, id: ItemId, value: bool) {
    set_item_graphic(w, id, if value { 0x1ED1 } else { 0x1ED0 });
    ops::invalidate_properties(w, id);
}

fn bc_set_charges(w: &mut World, id: ItemId, value: i32) {
    if bc(w, id).charges != value {
        bc_edit(w, id, |d| d.charges = value);
        ops::invalidate_properties(w, id);
    }
}

fn add_receiver(w: &mut World, id: ItemId, receiver: ItemId) {
    bc_edit(w, id, |d| d.receivers.push(Some(receiver)));
    ops::invalidate_properties(w, id);
}

fn remove_receiver(w: &mut World, id: ItemId, receiver: ItemId) {
    bc_edit(w, id, |d| {
        if let Some(p) = d.receivers.iter().position(|r| *r == Some(receiver)) {
            d.receivers.remove(p);
        }
    });
    ops::invalidate_properties(w, id);
}

/// `ReceiverCrystal.Sender = value` (`OnSenderChanged`).
fn rc_set_sender(w: &mut World, id: ItemId, value: Option<ItemId>) {
    let old = rc_sender(w, id);
    if old == value {
        return;
    }
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(ReceiverData { sender: value });
    }
    if let Some(o) = old {
        remove_receiver(w, o, id);
    }
    if let Some(n) = value {
        add_receiver(w, n, id);
    }
}

fn transmit_message(w: &mut World, rc: ItemId, from: MobileId, message: &str) {
    if !rc_active(w, rc) {
        return;
    }
    let name = w
        .mobile(from)
        .and_then(|m| (m.ty.name)(w, from))
        .unwrap_or_default();
    let text = format!("{name} says {message}");
    match ops::root_parent(w, rc) {
        Some(EntityId::Mobile(m)) => muo_server::mobile::part2::messages::send_message_hue(
            w,
            m,
            0x2B2,
            &format!("Crystal: {text}"),
        ),
        Some(EntityId::Item(i)) => messages::public_overhead_message(
            w,
            i,
            ItemMessage::REGULAR,
            0x2B2,
            false,
            &format!("Crystal: {text}"),
        ),
        None => messages::public_overhead_message(w, rc, ItemMessage::REGULAR, 0x2B2, false, &text),
    }
}

fn crystal_reach_check(w: &mut World, crystal: ItemId, from: MobileId, range: i32) -> bool {
    let loc = ops::get_world_location(w, crystal);
    let same_map = w.mobile(from).map(|m| m.map) == w.item(crystal).map(|i| i.map);
    if !same_map || !deps::mobile_in_range(w, from, loc, range) {
        cant_reach(w, from);
        return false;
    }
    true
}

fn broadcast_target(w: &mut World, crystal: ItemId, from: MobileId, targeted: Targeted) {
    if let Some(ty) = w.item(crystal).map(|i| i.ty)
        && !(ty.is_accessible_to)(w, crystal, from)
    {
        return;
    }
    if !crystal_reach_check(w, crystal, from, 2) {
        return;
    }
    let ent = match targeted {
        Targeted::Entity(e) => Some(e),
        _ => None,
    };
    if ent == Some(EntityId::Item(crystal)) {
        if bc_active(w, crystal) {
            bc_set_active(w, crystal, false);
            msg(w, from, 500672); // You turn the crystal off.
        } else if bc(w, crystal).charges > 0 {
            bc_set_active(w, crystal, true);
            msg(w, from, 500673); // You turn the crystal on.
        } else {
            msg(w, from, 500676); // This crystal is out of charges.
        }
    } else if let Some(EntityId::Item(rx)) = ent.filter(|e| matches!(e, EntityId::Item(i) if w.item(*i).is_some_and(|x| x.ty.is(&RECEIVER_CRYSTAL)))) {
        let count = bc(w, crystal).receivers.len();
        let sender = rc_sender(w, rx);
        if count >= 10 {
            msg(w, from, 1010042); // This broadcast crystal is already linked to 10 receivers.
        } else if sender == Some(crystal) {
            msg(w, from, 500674); // This crystal is already linked with that crystal.
        } else if sender.is_some() {
            msg(w, from, 1010043); // That receiver crystal is already linked to another broadcast crystal.
        } else {
            rc_set_sender(w, rx, Some(crystal));
            msg(w, from, 500675); // That crystal has been linked to this crystal.
        }
    } else if ent == Some(EntityId::Mobile(from)) {
        for rcv in bc(w, crystal).receivers.clone().into_iter().flatten() {
            rc_set_sender(w, rcv, None);
        }
        msg(w, from, 1010046); // You unlink the broadcast crystal from all of its receivers.
    } else {
        let item = ent.and_then(item_of);
        let movable = item.is_some_and(|i| {
            let ty = w.item(i).map(|x| x.ty);
            ty.is_some_and(|t| (t.verify_move)(w, i, from))
        });
        let info = item.filter(|_| movable).and_then(|i| recharge_of(w, i));
        match (item, info) {
            (Some(item), Some(amount)) if movable => {
                if bc(w, crystal).charges >= MAX_CHARGES {
                    msg(w, from, 500678); // This crystal is already fully charged.
                } else {
                    muo_server::item::base::consume(w, item, 1);
                    let cur = bc(w, crystal).charges;
                    if cur + amount >= MAX_CHARGES {
                        bc_set_charges(w, crystal, MAX_CHARGES);
                        msg(w, from, 500679); // You completely recharge the crystal.
                    } else {
                        bc_set_charges(w, crystal, cur + amount);
                        msg(w, from, 500680); // You recharge the crystal.
                    }
                }
            }
            _ => msg(w, from, 500681), // You cannot use this crystal on that.
        }
    }
}

fn receiver_target(w: &mut World, crystal: ItemId, from: MobileId, targeted: Targeted) {
    if let Some(ty) = w.item(crystal).map(|i| i.ty)
        && !(ty.is_accessible_to)(w, crystal, from)
    {
        return;
    }
    if !crystal_reach_check(w, crystal, from, 2) {
        return;
    }
    let ent = match targeted {
        Targeted::Entity(e) => Some(e),
        _ => None,
    };
    if ent == Some(EntityId::Item(crystal)) {
        let on = !rc_active(w, crystal);
        rc_set_active(w, crystal, on);
        msg(w, from, if on { 500673 } else { 500672 }); // You turn the crystal on / off.
    } else if ent == Some(EntityId::Mobile(from)) {
        if rc_sender(w, crystal).is_some() {
            rc_set_sender(w, crystal, None);
            msg(w, from, 1010044); // You unlink the receiver crystal.
        } else {
            msg(w, from, 1010045); // That receiver crystal is not linked.
        }
    } else {
        if let Some(item) = ent.and_then(item_of) {
            let movable = w
                .item(item)
                .map(|x| x.ty)
                .is_some_and(|t| (t.verify_move)(w, item, from));
            if movable && recharge_of(w, item).is_some() {
                msg(w, from, 500677); // This crystal cannot be recharged.
                return;
            }
        }
        msg(w, from, 1010045); // That receiver crystal is not linked.
    }
}

fn crystal_double_click(w: &mut World, id: ItemId, from: MobileId, range: i32, broadcast: bool) {
    let loc = ops::get_world_location(w, id);
    if !deps::mobile_in_range(w, from, loc, 2) {
        cant_reach(w, from);
        return;
    }
    target::begin_target(
        w,
        from,
        range,
        false,
        TargetFlags::NONE,
        move |w, from, t| {
            if broadcast {
                broadcast_target(w, id, from, t)
            } else {
                receiver_target(w, id, from, t)
            }
        },
    );
}

item_leaf_raw! {
    /// `Server.Items.BroadcastCrystal` `[SerializationGenerator(1)]`.
    BROADCAST_CRYSTAL = "Server.Items.BroadcastCrystal" : ITEM_BASE;
    construct: Some(|w| broadcast_crystal_new(w, &BROADCAST_CRYSTAL, 2000)),
    label_number: crate::class_props::label_number, // communication crystal
    handles_on_speech: |_, _| true,
    get_properties: |w, id, list| {
        base::get_properties_base(w, id, list);
        list.add(if bc_active(w, id) { 1060742 } else { 1060743 }); // active / inactive
        list.add(1060745); // broadcast
        list.add_str(1060741, &bc(w, id).charges.to_string()); // charges: ~1_val~
        let n = bc(w, id).receivers.len();
        if n > 0 {
            list.add_str(1060746, &n.to_string()); // links: ~1_val~
        }
    },
    on_single_click: |w, id, from| {
        base::on_single_click_base(w, id, from);
        ops::label_to_number(w, id, from, if bc_active(w, id) { 1060742 } else { 1060743 }, "");
        ops::label_to_number(w, id, from, 1060745, "");
        ops::label_to_number(w, id, from, 1060741, &bc(w, id).charges.to_string());
        let n = bc(w, id).receivers.len();
        if n > 0 {
            ops::label_to_number(w, id, from, 1060746, &n.to_string());
        }
    },
    on_speech: |w, id, e| {
        let d = bc(w, id);
        let root_other = match ops::root_parent(w, id) {
            Some(EntityId::Mobile(_)) | None => false,
            Some(EntityId::Item(_)) => true,
        };
        if !bc_active(w, id) || d.receivers.is_empty() || root_other || e.message_type == MessageType::EMOTE {
            return;
        }
        for rcv in d.receivers.clone() {
            let deleted = rcv.is_none_or(|r| ops::is_deleted(w, r));
            if deleted {
                if let Some(r) = rcv {
                    remove_receiver(w, id, r);
                }
            } else if bc(w, id).charges > 0 {
                if let Some(r) = rcv {
                    transmit_message(w, r, e.mobile, &e.speech);
                }
                let c = bc(w, id).charges;
                bc_set_charges(w, id, c - 1);
            } else {
                bc_set_active(w, id, false);
                break;
            }
        }
    },
    on_double_click: |w, id, from| crystal_double_click(w, id, from, 2, true),
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_encoded_int(1);
        let d = bc(w, id);
        out.write_i32(d.charges);
        out.write_encoded_int(d.receivers.len() as i32);
        for r in d.receivers {
            out.write_item(w, r);
        }
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let version = r.read_encoded_int()?;
        let (charges, receivers);
        if version < 1 {
            charges = r.read_encoded_int()?;
            receivers = r.read_item_list_opt(w)?.unwrap_or_default().into_iter().map(Some).collect();
        } else {
            charges = r.read_i32()?;
            let count = r.read_encoded_int()?;
            let mut list = Vec::new();
            for _ in 0..count.max(0) {
                list.push(r.read_item(w)?);
            }
            receivers = list;
        }
        if let Some(i) = w.item_mut(id) {
            i.ext = Ext::new(BroadcastData { charges, receivers });
        }
        Ok(())
    },
}

item_leaf_raw! {
    /// `Server.Items.ReceiverCrystal` `[SerializationGenerator(0)]`.
    RECEIVER_CRYSTAL = "Server.Items.ReceiverCrystal" : ITEM_BASE;
    construct: Some(|w| receiver_crystal_new(w, &RECEIVER_CRYSTAL)),
    label_number: crate::class_props::label_number, // communication crystal
    get_properties: |w, id, list| {
        base::get_properties_base(w, id, list);
        list.add(if rc_active(w, id) { 1060742 } else { 1060743 }); // active / inactive
        list.add(1060744); // receiver
    },
    on_single_click: |w, id, from| {
        base::on_single_click_base(w, id, from);
        ops::label_to_number(w, id, from, if rc_active(w, id) { 1060742 } else { 1060743 }, "");
        ops::label_to_number(w, id, from, 1060744, "");
    },
    on_double_click: |w, id, from| crystal_double_click(w, id, from, -1, false),
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_encoded_int(0);
        out.write_item(w, rc_sender(w, id));
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        r.read_encoded_int()?;
        let sender = r.read_item(w)?;
        if let Some(i) = w.item_mut(id) {
            i.ext = Ext::new(ReceiverData { sender });
        }
        Ok(())
    },
}

static BROADCAST_CRYSTAL_PROPS: &[PropDef] = &[
    PropDef {
        name: "Charges",
        kind: INT,
        read: GM,
        write: GM,
        get: |w, e| Value::Int(item_of(e).map_or(0, |i| i64::from(bc(w, i).charges))),
        set: Some(|w, e, v| {
            if let Some(i) = item_of(e) {
                bc_set_charges(w, i, v.i32());
            }
            Ok(())
        }),
    },
    PropDef {
        name: "Active",
        kind: BOOL,
        read: GM,
        write: GM,
        get: |w, e| Value::Bool(item_of(e).is_some_and(|i| bc_active(w, i))),
        set: Some(|w, e, v| {
            if let Some(i) = item_of(e) {
                bc_set_active(w, i, v.bool());
            }
            Ok(())
        }),
    },
];

static RECEIVER_CRYSTAL_PROPS: &[PropDef] = &[
    PropDef {
        name: "Sender",
        kind: Kind::Plain(PropType::Entity),
        read: GM,
        write: GM,
        get: |w, e| Value::Entity(item_of(e).and_then(|i| rc_sender(w, i)).map(EntityId::Item)),
        set: Some(|w, e, v| {
            let s = v
                .entity()
                .and_then(item_of)
                .filter(|s| w.item(*s).is_some_and(|x| x.ty.is(&BROADCAST_CRYSTAL)));
            if let Some(i) = item_of(e) {
                rc_set_sender(w, i, s);
            }
            Ok(())
        }),
    },
    PropDef {
        name: "Active",
        kind: BOOL,
        read: GM,
        write: GM,
        get: |w, e| Value::Bool(item_of(e).is_some_and(|i| rc_active(w, i))),
        set: Some(|w, e, v| {
            if let Some(i) = item_of(e) {
                rc_set_active(w, i, v.bool());
            }
            Ok(())
        }),
    },
];

// ---- PowerGenerator, ControlPanel ----

/// `ControlPanel.m_UseTimeout`.
const USE_TIMEOUT: TimeSpan = TimeSpan::from_mins(2);

/// What `ControlPanel` adds to an `AddonComponent`.
#[derive(Clone, Debug, Default)]
pub struct PanelData {
    pub side_length: i32,
    pub path: Vec<Point2D>,
    user: Option<MobileId>,
    last_use: DateTime,
    damage_table: Vec<MobileId>,
}

fn panel(w: &World, id: ItemId) -> PanelData {
    w.item(id)
        .and_then(|i| i.ext.get::<PanelData>())
        .cloned()
        .unwrap_or_default()
}

fn panel_edit(w: &mut World, id: ItemId, f: impl FnOnce(&mut PanelData)) {
    if let Some(d) = w.item_mut(id).and_then(|i| i.ext.get_mut::<PanelData>()) {
        f(d);
    }
}

/// `ControlPanel(sideLength)`.
pub fn control_panel_new(w: &mut World, side_length: i32) -> ItemId {
    let id = addon::new_component(w, &CONTROL_PANEL, 0xBDC);
    super::hue(w, id, 0x835);
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(PanelData {
            side_length,
            ..Default::default()
        });
    }
    id
}

/// `ControlPanel.SideLength = value`.
pub fn panel_set_side_length(w: &mut World, id: ItemId, value: i32) {
    let value = value.clamp(3, 6);
    if panel(w, id).side_length != value {
        panel_edit(w, id, |d| d.side_length = value);
        panel_init_path(w, id);
    }
}

/// `ControlPanel.InitPath()`: a depth-first walk from (0, 0) to the far corner.
pub fn panel_init_path(w: &mut World, id: ItemId) {
    let n = panel(w, id).side_length.max(0) as usize;
    let side = n as i32;
    let mut stack = vec![Point2D::new(0, 0); n * n + 1];
    let mut current = stack[0];
    let mut size = 1usize;
    let mut visited = vec![vec![false; n]; n];
    visited[0][0] = n > 0;
    // 0 left, 1 up, 2 right, 3 down
    if n == 0 {
        panel_edit(w, id, |d| d.path.clear());
        return;
    }
    loop {
        let mut choices = [0u8; 4];
        let mut count = 0;
        let v = |x: i32, y: i32| visited[x as usize][y as usize];
        if current.x > 0 && !v(current.x - 1, current.y) {
            choices[count] = 0;
            count += 1;
        }
        if current.y > 0 && !v(current.x, current.y - 1) {
            choices[count] = 1;
            count += 1;
        }
        if current.x < side - 1 && !v(current.x + 1, current.y) {
            choices[count] = 2;
            count += 1;
        }
        if current.y < side - 1 && !v(current.x, current.y + 1) {
            choices[count] = 3;
            count += 1;
        }
        if count > 0 {
            let dir = choices[utility::random(&mut w.rng, count as i32) as usize];
            current = match dir {
                0 => Point2D::new(current.x - 1, current.y),
                1 => Point2D::new(current.x, current.y - 1),
                2 => Point2D::new(current.x + 1, current.y),
                _ => Point2D::new(current.x, current.y + 1),
            };
            stack[size] = current;
            size += 1;
            if current.x == side - 1 && current.y == side - 1 {
                break;
            }
            visited[current.x as usize][current.y as usize] = true;
        } else {
            size -= 1;
            current = stack[size - 1];
        }
    }
    let path = stack[..size].to_vec();
    panel_edit(w, id, |d| d.path = path);
    let user = panel(w, id).user;
    if let Some(u) = user {
        if let Some(ns) = w.mobile(u).and_then(|m| m.net_state) {
            muo_content::gumps::close_gump::<PowerGeneratorGump>(w, ns);
        }
        panel_edit(w, id, |d| d.user = None);
    }
}

fn panel_do_damage(w: &mut World, id: ItemId, to: MobileId) {
    messages::send_message_to(
        w,
        id,
        to,
        "The generator shoots an arc of electricity at you!",
        0x3B2,
    );
    muo_server::mobile::part2::effects::bolt_effect(w, to, 0);
    muo_server::mobile::part2::messages::local_overhead_message_number(
        w,
        to,
        MessageType::REGULAR,
        0xC9,
        1114443,
        "",
    ); // * Your body convulses from electric shock *
    let name = w
        .mobile(to)
        .and_then(|m| (m.ty.name)(w, to))
        .unwrap_or_default();
    muo_server::mobile::part2::messages::nonlocal_overhead_message_number(
        w,
        to,
        MessageType::REGULAR,
        0xC9,
        1114444,
        &name,
    ); // * ~1_NAME~ spasms from electric shock *
    damage(
        w,
        to,
        Some(to),
        60,
        DamageSplit {
            nrgy: 100,
            ..Default::default()
        },
        DamageOptions::default(),
    );
    if !muo_server::mobile::base::alive(w, to) {
        return;
    }
    if !panel(w, id).damage_table.contains(&to) {
        muo_server::mobile::part1::props_a::set_frozen(w, to, true);
        panel_edit(w, id, |d| d.damage_table.push(to));
        let mut step = 0;
        timer::start(
            w,
            TimeSpan::from_secs(5),
            TimeSpan::from_secs(5),
            0,
            move |w| {
                // DamageTimer.OnTick
                let ended = |w: &mut World, h: Option<TimerHandle>| {
                    panel_edit(w, id, |d| d.damage_table.retain(|m| *m != to));
                    muo_server::mobile::part1::props_a::set_frozen(w, to, false);
                    if let Some(mut h) = h {
                        h.cancel(w);
                    }
                };
                let _ = &ended;
                if ops::is_deleted(w, id)
                    || w.mobile(to).is_none_or(|m| m.deleted)
                    || !muo_server::mobile::base::alive(w, to)
                {
                    panel_edit(w, id, |d| d.damage_table.retain(|m| *m != to));
                    muo_server::mobile::part1::props_a::set_frozen(w, to, false);
                    return;
                }
                muo_content::mx::play_sound(w, to, 0x28);
                muo_server::mobile::part2::messages::local_overhead_message_number(
                    w,
                    to,
                    MessageType::REGULAR,
                    0xC9,
                    1114443,
                    "",
                );
                let name = w
                    .mobile(to)
                    .and_then(|m| (m.ty.name)(w, to))
                    .unwrap_or_default();
                muo_server::mobile::part2::messages::nonlocal_overhead_message_number(
                    w,
                    to,
                    MessageType::REGULAR,
                    0xC9,
                    1114444,
                    &name,
                );
                damage(
                    w,
                    to,
                    Some(to),
                    20,
                    DamageSplit {
                        nrgy: 100,
                        ..Default::default()
                    },
                    DamageOptions::default(),
                );
                step += 1;
                if step >= 3 || !muo_server::mobile::base::alive(w, to) {
                    panel_edit(w, id, |d| d.damage_table.retain(|m| *m != to));
                    muo_server::mobile::part1::props_a::set_frozen(w, to, false);
                }
            },
        );
        // shortcut: `DamageTimer.Stop()` ends the repeat; here the tick above clears the table and the
        // frozen flag and the three ticks are the whole life of the timer (`End()` on the third).
    }
}

fn panel_solve(w: &mut World, id: ItemId, from: MobileId) {
    let loc = ops::get_world_location(w, id);
    let map = w.item(id).and_then(|i| i.map);
    effects::play_sound(w, loc, map, 0x211);
    effects::play_sound(w, loc, map, 0x1F3);
    effects::send_location_effect(w, loc, map, 0x36B0, 4, 4, 0, 0);
    effects::send_location_effect(
        w,
        Point3D::new(loc.x - 1, loc.y - 1, loc.z + 2),
        map,
        0x36B0,
        4,
        4,
        0,
        0,
    );
    effects::send_location_effect(
        w,
        Point3D::new(loc.x - 2, loc.y - 1, loc.z + 2),
        map,
        0x36B0,
        4,
        4,
        0,
        0,
    );
    deps::send_message(w, from, "You scrounge some gems from the wreckage.");
    let side = panel(w, id).side_length;
    for _ in 0..side {
        if let Some(g) = muo_content::factory::make_item(w, "Server.Items.ArcaneGem") {
            deps::add_to_backpack(w, from, g);
        }
    }
    let stack = |w: &mut World, name: &str, amount: i32, at: Option<Point3D>| {
        if let Some(i) = muo_content::factory::make_item(w, name) {
            ops::set_amount(w, i, amount);
            match at {
                Some(p) => ops::move_to_world_map(w, i, p, map),
                None => deps::add_to_backpack(w, from, i),
            }
        }
    };
    stack(w, "Server.Items.Diamond", side, None);
    stack(
        w,
        "Server.Items.ShadowIronOre",
        9,
        Some(Point3D::new(loc.x - 1, loc.y, loc.z + 2)),
    );
    stack(
        w,
        "Server.Items.ShadowIronOre",
        14,
        Some(Point3D::new(loc.x - 2, loc.y - 1, loc.z + 2)),
    );
    delete_item(w, id);
}

/// `ControlPanel.PowerGeneratorGump`.
pub struct PowerGeneratorGump {
    from: MobileId,
    panel: ItemId,
    step: usize,
    hint: bool,
    side_length: i32,
    path: Vec<Point2D>,
    lockpicking: f64,
}

fn show_gump(w: &mut World, from: MobileId, panel_id: ItemId, step: usize, hint: bool) {
    if w.mobile(from).and_then(|m| m.net_state).is_none() || ops::is_deleted(w, panel_id) {
        return;
    }
    let d = panel(w, panel_id);
    // BuildLayout indexes `Path[_step]`: an unset path (the constructor never calls InitPath) throws there, and
    // no gump is sent.
    if d.path.len() <= step + usize::from(hint) || d.path.is_empty() {
        return;
    }
    let lockpicking =
        muo_server::skills::value_ref(w, from, muo_server::skills::SkillName::Lockpicking);
    let g = PowerGeneratorGump {
        from,
        panel: panel_id,
        step,
        hint,
        side_length: d.side_length,
        path: d.path,
        lockpicking,
    };
    send_gump_to_mobile(w, from, Box::new(DynamicGumpObj::new(5, 30, g)), false);
}

impl DynamicGumpLogic for PowerGeneratorGump {
    const TYPE_NAME: &'static str = "Server.Items.ControlPanel+PowerGeneratorGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        let s = self.side_length;
        b.add_background(50, 0, 530, 410, 0xA28);
        b.add_image(0, 0, 0x28C8, 0);
        b.add_image(547, 0, 0x28C9, 0);
        b.add_background(95, 20, 442, 90, 0xA28);
        b.add_html(229, 35, 300, 45, "GENERATOR CONTROL PANEL", false, false);
        b.add_html(
            223,
            60,
            300,
            70,
            "Use the Directional Controls to",
            false,
            false,
        );
        b.add_html(253, 75, 300, 85, "Close the Grid Circuit", false, false);
        b.add_image(140, 40, 0x28D3, 0);
        b.add_image(420, 40, 0x28D3, 0);
        b.add_background(365, 120, 178, 210, 0x1400);
        b.add_image(365, 115, 0x28D4, 0);
        b.add_image(365, 288, 0x28D4, 0);
        b.add_image(414, 189, 0x589, 0);
        b.add_image(435, 210, 0xA52, 0);
        b.add_button(408, 222, 0x29EA, 0x29EC, 1); // Left
        b.add_button(448, 185, 0x29CC, 0x29CE, 2); // Up
        b.add_button(473, 222, 0x29D6, 0x29D8, 3); // Right
        b.add_button(448, 243, 0x29E0, 0x29E2, 4); // Down
        b.add_background(90, 115, 30 + 40 * s, 30 + 40 * s, 0xA28);
        b.add_background(100, 125, 10 + 40 * s, 10 + 40 * s, 0x1400);
        for i in 0..s {
            for j in 0..s - 1 {
                b.add_image(120 + 40 * i, 162 + 40 * j, 0x13F9, 0);
            }
        }
        for i in 0..s - 1 {
            for j in 0..s {
                b.add_image(138 + 40 * i, 147 + 40 * j, 0x13FD, 0);
            }
        }
        // 0 gray, 1 blue, 2 red
        let mut hues = vec![vec![0u8; s as usize]; s as usize];
        for n in self.path.iter().take(self.step + 1) {
            hues[n.x as usize][n.y as usize] = 1;
        }
        if let Some(last) = self.path.last() {
            hues[last.x as usize][last.y as usize] = 2;
        }
        for i in 0..s {
            for j in 0..s {
                let id = match hues[i as usize][j as usize] {
                    0 => 0x25F8,
                    1 => 0x868,
                    _ => 0x9A8,
                };
                b.add_image(110 + 40 * i, 135 + 40 * j, id, 0);
            }
        }
        let cur = self.path[self.step];
        b.add_image(118 + 40 * cur.x, 143 + 40 * cur.y, 0x13A8, 0);
        if self.hint {
            let next = self.path[self.step + 1];
            b.add_image(119 + 40 * next.x, 143 + 40 * next.y, 0x939, 0);
        }
        if self.lockpicking >= 65.0 {
            b.add_button(365, 350, 0xFA6, 0xFA7, 5);
            b.add_html(
                405,
                345,
                140,
                40,
                "Attempt to Decipher the Circuit Path",
                false,
                false,
            );
        }
    }

    fn on_response(&mut self, w: &mut World, _sender: NetStateId, info: &RelayInfo) {
        use muo_server::skills::SkillName;
        let (from, id) = (self.from, self.panel);
        // `CheckAlive()` runs (and says "you are dead") only when the earlier terms did not already end it
        if ops::is_deleted(w, id)
            || info.button_id == 0
            || !muo_server::mobile::part2::stats::check_alive(w, from, true)
        {
            panel_edit(w, id, |d| d.user = None);
            return;
        }
        let loc = ops::get_world_location(w, id);
        let same_map = w.mobile(from).map(|m| m.map) == w.item(id).map(|i| i.map);
        if !same_map || !deps::mobile_in_range(w, from, loc, 3) {
            msg(w, from, 500446); // That is too far away.
            panel_edit(w, id, |d| d.user = None);
            return;
        }
        let path = panel(w, id).path;
        let Some(&next) = path.get(self.step + 1) else {
            return;
        };
        let now = w.now;
        if info.button_id == 5 {
            // Attempt to Decipher
            let lockpicking = muo_server::skills::value_ref(w, from, SkillName::Lockpicking);
            if lockpicking < 65.0 {
                return;
            }
            muo_content::mx::play_sound(w, from, 0x241);
            if 40.0 + utility::random_double(&mut w.rng) * 80.0 < lockpicking {
                show_gump(w, from, id, self.step, true);
                panel_edit(w, id, |d| d.last_use = now);
            } else {
                panel_do_damage(w, id, from);
                panel_edit(w, id, |d| d.user = None);
            }
        } else {
            let cur = path[self.step];
            let (nx, ny) = match info.button_id {
                1 => (cur.x - 1, cur.y),
                2 => (cur.x, cur.y - 1),
                3 => (cur.x + 1, cur.y),
                4 => (cur.x, cur.y + 1),
                _ => return,
            };
            if next.x == nx && next.y == ny {
                if self.step + 1 == path.len() - 1 {
                    panel_solve(w, id, from);
                    panel_edit(w, id, |d| d.user = None);
                } else {
                    muo_content::mx::play_sound(w, from, 0x1F4);
                    show_gump(w, from, id, self.step + 1, false);
                    panel_edit(w, id, |d| d.last_use = now);
                }
            } else {
                panel_do_damage(w, id, from);
                panel_edit(w, id, |d| d.user = None);
            }
        }
    }
}

fn panel_double_click(w: &mut World, id: ItemId, from: MobileId) {
    let loc = ops::get_world_location(w, id);
    if !deps::mobile_in_range(w, from, loc, 3) {
        msg(w, from, 500446); // That is too far away.
        return;
    }
    let d = panel(w, id);
    if let Some(user) = d.user {
        if user == from {
            return;
        }
        let gone = w.mobile(user).is_none_or(|m| m.deleted)
            || w.mobile(user).map(|m| m.map) != w.item(id).map(|i| i.map)
            || !deps::mobile_in_range(w, user, loc, 3)
            || w.mobile(user).and_then(|m| m.net_state).is_none()
            || w.now - d.last_use >= USE_TIMEOUT;
        if gone {
            if let Some(ns) = w.mobile(user).and_then(|m| m.net_state) {
                muo_content::gumps::close_gump::<PowerGeneratorGump>(w, ns);
            }
        } else {
            deps::send_message(w, from, "Someone is currently using the control panel.");
            return;
        }
    }
    let now = w.now;
    panel_edit(w, id, |d| {
        d.user = Some(from);
        d.last_use = now;
    });
    show_gump(w, from, id, 0, false);
}

item_leaf_raw! {
    /// `Server.Items.ControlPanel` `[SerializationGenerator(1)]`.
    CONTROL_PANEL = "Server.Items.ControlPanel" : ADDON_COMPONENT;
    construct: None,
    default_name: crate::class_props::default_name,
    on_double_click: panel_double_click,
    serialize: |w, id, out| {
        (ADDON_COMPONENT.serialize)(w, id, out);
        out.write_encoded_int(1);
        let d = panel(w, id);
        out.write_i32(d.side_length);
        out.write_encoded_int(d.path.len() as i32);
        for p in d.path {
            out.write_point2d(p);
        }
    },
    deserialize: |w, id, r| {
        (ADDON_COMPONENT.deserialize)(w, id, r)?;
        let version = r.read_encoded_int()?;
        let mut d = PanelData::default();
        if version < 1 {
            d.side_length = r.read_encoded_int()?;
            let n = r.read_encoded_int()?;
            for _ in 0..n.max(0) {
                let x = r.read_encoded_int()?;
                let y = r.read_encoded_int()?;
                d.path.push(Point2D::new(x, y));
            }
        } else {
            d.side_length = r.read_i32()?;
            let n = r.read_encoded_int()?;
            for _ in 0..n.max(0) {
                d.path.push(r.read_point2d()?);
            }
        }
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(d);
        }
        Ok(())
    },
}

static CONTROL_PANEL_PROPS: &[PropDef] = &[PropDef {
    name: "SideLength",
    kind: INT,
    read: GM,
    write: GM,
    get: |w, e| Value::Int(item_of(e).map_or(0, |i| i64::from(panel(w, i).side_length))),
    set: Some(|w, e, v| {
        if let Some(i) = item_of(e) {
            panel_set_side_length(w, i, v.i32());
        }
        Ok(())
    }),
}];

fn power_generator_component(w: &mut World, addon: ItemId, graphic: i32, x: i32, y: i32, z: i32) {
    let c = addon::new_component(w, &ADDON_COMPONENT, graphic);
    ops::set_name(w, c, Some("a power generator".to_string()));
    super::hue(w, c, 0x451);
    addon::add_component(w, addon, c, x, y, z);
}

/// `PowerGenerator(sideLength)`.
pub fn power_generator_new(w: &mut World, side_length: i32) -> ItemId {
    let id = addon::new_addon(w, &POWER_GENERATOR);
    power_generator_component(w, id, 0x4FA1, 0, 0, 0);
    power_generator_component(w, id, 0x76, -1, 0, 0);
    power_generator_component(w, id, 0x75, 0, -1, 0);
    power_generator_component(w, id, 0x37F4, 0, 0, 13);
    let p = control_panel_new(w, side_length);
    addon::add_component(w, id, p, 1, 0, -2);
    id
}

static POWER_GENERATOR_CLASS: addon::AddonClass = addon::AddonClass {
    share_hue: false,
    ..addon::BASE_ADDON_CLASS
};

item_leaf_raw! {
    /// `Server.Items.PowerGenerator` `[SerializationGenerator(0)]`.
    POWER_GENERATOR = "Server.Items.PowerGenerator" : BASE_ADDON;
    construct: Some(|w| {
        let n = super::random_min_max(w, 3, 6);
        power_generator_new(w, n)
    }),
    family: Some(&POWER_GENERATOR_CLASS),
    serialize: |w, id, out| {
        (BASE_ADDON.serialize)(w, id, out);
        out.write_encoded_int(0);
    },
    deserialize: |w, id, r| {
        (BASE_ADDON.deserialize)(w, id, r)?;
        r.read_encoded_int()?;
        Ok(())
    },
}

// ---- ProjectedItem ----

/// `ProjectedItem`'s fields; `_oldVisible` is not saved.
#[derive(Clone, Debug, Default)]
pub struct ProjectedData {
    pub effect_item_id: i32,
    pub minimum_visible: AccessLevel,
    pub old_visible: bool,
}

/// `ProjectedItem._active` and `_timer`.
#[derive(Default)]
struct ProjectedActive {
    items: Vec<ItemId>,
    timer: Option<TimerHandle>,
}

fn pj(w: &World, id: ItemId) -> ProjectedData {
    w.item(id)
        .and_then(|i| i.ext.get::<ProjectedData>())
        .cloned()
        .unwrap_or_default()
}

fn pj_edit(w: &mut World, id: ItemId, f: impl FnOnce(&mut ProjectedData)) {
    if let Some(d) = w
        .item_mut(id)
        .and_then(|i| i.ext.get_mut::<ProjectedData>())
    {
        f(d);
    }
}

/// `ProjectedItem(effectItemId)`.
pub fn projected_item_new(w: &mut World, effect_item_id: i32) -> ItemId {
    projected_item_new_of(w, &PROJECTED_ITEM, effect_item_id, AccessLevel::default())
}

/// `ProjectedItem(effectItemId)` run for a subclass `ty` (`SpawnerBorder`), which sets `MinimumVisible` itself.
pub fn projected_item_new_of(
    w: &mut World,
    ty: &'static ItemType,
    effect_item_id: i32,
    minimum_visible: AccessLevel,
) -> ItemId {
    let id = crate::simple::from_template(w, ty, "Server.Items.ProjectedItem");
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(ProjectedData {
            effect_item_id,
            minimum_visible,
            ..Default::default()
        });
    }
    id
}

fn pj_activate(w: &mut World, id: ItemId) {
    if w.resources
        .get::<ProjectedActive>()
        .is_some_and(|a| a.items.contains(&id))
    {
        return;
    }
    w.resources.get_mut::<ProjectedActive>().items.push(id);
    let t = w.resources.get::<ProjectedActive>().and_then(|a| a.timer);
    match t {
        None => {
            let h = timer::start(w, TimeSpan::ZERO, TimeSpan::from_secs(1), 0, pj_tick);
            w.resources.get_mut::<ProjectedActive>().timer = Some(h);
        }
        Some(h) if !h.running(w) => {
            let h = timer::start(
                w,
                TimeSpan::from_secs(1),
                TimeSpan::from_secs(1),
                0,
                pj_tick,
            );
            w.resources.get_mut::<ProjectedActive>().timer = Some(h);
        }
        Some(_) => {}
    }
}

fn pj_deactivate(w: &mut World, id: ItemId) {
    let a = w.resources.get_mut::<ProjectedActive>();
    if let Some(p) = a.items.iter().position(|i| *i == id) {
        a.items.remove(p);
        if a.items.is_empty()
            && let Some(mut t) = a.timer
        {
            a.timer = None;
            t.cancel(w);
        }
    }
}

fn pj_tick(w: &mut World) {
    let items = w
        .resources
        .get::<ProjectedActive>()
        .map(|a| a.items.clone())
        .unwrap_or_default();
    for item in items {
        if !pj_send_effect(w, item) {
            w.resources
                .get_mut::<ProjectedActive>()
                .items
                .retain(|i| *i != item);
        }
    }
    if w.resources
        .get::<ProjectedActive>()
        .is_some_and(|a| a.items.is_empty())
        && let Some(mut t) = w.resources.get::<ProjectedActive>().and_then(|a| a.timer)
    {
        w.resources.get_mut::<ProjectedActive>().timer = None;
        t.cancel(w);
    }
}

/// `ProjectedItem.SendEffect()`: whether any client was in range.
fn pj_send_effect(w: &mut World, id: ItemId) -> bool {
    let d = pj(w, id);
    if d.old_visible {
        return false;
    }
    let Some(i) = w.item(id) else { return false };
    let (serial, loc, map, hue, ty) = (i.serial, i.location, i.map, i.hue, i.ty);
    let Some(map) = map else { return false };
    let bytes = muo_server::packets::effect::create_hued_effect(
        effects::EffectType::FixedXYZ,
        serial,
        serial,
        d.effect_item_id,
        loc,
        loc,
        1,
        20,
        true,
        false,
        hue - 1,
        3,
    );
    let range = (ty.get_max_update_range)(w, id);
    let mut found = false;
    for (ns, from) in deps::clients_in_range(w, map, loc, range) {
        found = true;
        let from = Some(from);
        if w.cannot_send_packets(ns) {
            continue;
        }
        let Some(from) = from else { continue };
        if w.mobile(from)
            .is_none_or(|m| m.access_level < d.minimum_visible)
        {
            continue;
        }
        if let Some(mty) = w.mobile(from).map(|m| m.ty) {
            (mty.process_delta)(w, from);
        }
        w.send(ns, &bytes);
    }
    found
}

item_leaf_raw! {
    /// `Server.Items.ProjectedItem` `[SerializationGenerator(0)]`.
    PROJECTED_ITEM = "Server.Items.ProjectedItem" : ITEM_BASE;
    construct: None,
    handles_on_movement: |_, _| true,
    can_see_staff_only: |w, id, from| {
        w.mobile(from).is_some_and(|m| m.access_level >= pj(w, id).minimum_visible)
    },
    process_delta: |w, id| {
        let visible = w.item(id).is_some_and(|i| i.visible());
        if pj(w, id).old_visible != visible {
            timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 1, move |w| {
                let visible = w.item(id).is_some_and(|i| i.visible());
                let graphic = if visible { pj(w, id).effect_item_id } else { 1 };
                set_item_graphic(w, id, graphic);
                pj_edit(w, id, |d| d.old_visible = visible);
                pj_activate(w, id);
            });
        }
        base::process_delta_base(w, id);
    },
    send_info_to: |w, id, ns, world| {
        base::send_info_to_base(w, id, ns, world);
        let m = w.net_state(ns).and_then(|n| n.mobile);
        let visible = w.item(id).is_some_and(|i| i.visible());
        let active = w.resources.get::<ProjectedActive>().is_some_and(|a| a.items.contains(&id));
        if let Some(m) = m
            && !visible
            && w.mobile(m).is_some_and(|x| x.access_level >= pj(w, id).minimum_visible)
            && !active
        {
            pj_activate(w, id);
        }
    },
    on_location_change: |w, id, old| {
        base::on_location_change_base(w, id, old);
        pj_activate(w, id);
    },
    on_map_change: |w, id| {
        base::on_map_change_base(w, id);
        pj_activate(w, id);
    },
    on_delete: |w, id| {
        pj_deactivate(w, id);
        base::on_delete_base(w, id);
    },
    on_movement: |w, id, m, old| {
        base::on_movement_base(w, id, m, old);
        let visible = w.item(id).is_some_and(|i| i.visible());
        let active = w.resources.get::<ProjectedActive>().is_some_and(|a| a.items.contains(&id));
        if !visible && !active && w.mobile(m).is_some_and(|x| x.access_level >= pj(w, id).minimum_visible) {
            pj_activate(w, id);
        }
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_encoded_int(0);
        let d = pj(w, id);
        out.write_i32(d.effect_item_id);
        out.write_enum(1, d.minimum_visible as i64);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        r.read_encoded_int()?;
        let effect_item_id = r.read_i32()?;
        let minimum_visible = AccessLevel::from_i32(r.read_enum(1)? as i32).unwrap_or_default();
        let old_visible = w.item(id).is_some_and(|i| i.visible()); // AfterDeserialization
        if let Some(i) = w.item_mut(id) {
            i.ext = Ext::new(ProjectedData { effect_item_id, minimum_visible, old_visible });
        }
        Ok(())
    },
}

static PROJECTED_ITEM_PROPS: &[PropDef] = &[PropDef {
    name: "EffectItemId",
    kind: INT,
    read: GM,
    write: GM,
    get: |w, e| Value::Int(item_of(e).map_or(0, |i| i64::from(pj(w, i).effect_item_id))),
    set: Some(|w, e, v| {
        if let Some(i) = item_of(e) {
            pj_edit(w, i, |d| d.effect_item_id = v.i32());
        }
        Ok(())
    }),
}];

// ---- registration ----

use muo_content::props::{Ctor, Param, add_ctors};

static PROJECTED_ITEM_CTORS: [Ctor; 1] = [Ctor {
    access: None,
    params: &[Param {
        name: "effectItemId",
        kind: INT,
        default: None,
        ty: "",
    }],
    make: |w, v| EntityId::Item(projected_item_new(w, v[0].i32())),
}];
static BROADCAST_CRYSTAL_CTORS: [Ctor; 1] = [Ctor {
    access: None,
    params: &[Param {
        name: "charges",
        kind: INT,
        default: Some(Value::Int(2000)),
        ty: "",
    }],
    make: |w, v| EntityId::Item(broadcast_crystal_new(w, &BROADCAST_CRYSTAL, v[0].i32())),
}];
static POWER_GENERATOR_CTORS: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[],
        make: |w, _| {
            let n = super::random_min_max(w, 3, 6);
            EntityId::Item(power_generator_new(w, n))
        },
    },
    Ctor {
        access: None,
        params: &[Param {
            name: "sideLength",
            kind: INT,
            default: None,
            ty: "",
        }],
        make: |w, v| EntityId::Item(power_generator_new(w, v[0].i32())),
    },
];

fn machines_configure(w: &mut World) {
    add_ctors(w, "Server.Items.ProjectedItem", &PROJECTED_ITEM_CTORS);
    add_ctors(w, "Server.Items.BroadcastCrystal", &BROADCAST_CRYSTAL_CTORS);
    add_ctors(w, "Server.Items.PowerGenerator", &POWER_GENERATOR_CTORS);
    muo_content::props::add(w, "Server.Items.EffectController", EFFECT_CONTROLLER_PROPS);
    muo_content::props::add(w, "Server.Items.BroadcastCrystal", BROADCAST_CRYSTAL_PROPS);
    muo_content::props::add(w, "Server.Items.ReceiverCrystal", RECEIVER_CRYSTAL_PROPS);
    muo_content::props::add(w, "Server.Items.ControlPanel", CONTROL_PANEL_PROPS);
    muo_content::props::add(w, "Server.Items.ProjectedItem", PROJECTED_ITEM_PROPS);
}

/// The classes of this file.
pub static ALL: &[&ItemType] = &[
    &EFFECT_CONTROLLER,
    &BROADCAST_CRYSTAL,
    &RECEIVER_CRYSTAL,
    &CONTROL_PANEL,
    &POWER_GENERATOR,
    &PROJECTED_ITEM,
];

/// Registers the classes of this file.
pub fn register(types: &mut TypeRegistry) {
    for t in ALL {
        types.register_item(t, &[]);
    }
    types.register_phase_default(
        muo_server::type_registry::Phase::Configure,
        "Server.Items.EffectController.Configure",
        machines_configure,
    );
}

#[cfg(test)]
mod power_generator_gump_tests {
    use super::*;
    use muo_server::mobile::MOBILE_BASE;
    use muo_server::net_state::NetStateData;

    /// `_panel.Deleted || ButtonID == 0 || !_from.CheckAlive()`: a ghost who closes the gump is not told they are dead,
    /// one who presses a button is.
    #[test]
    fn check_alive_only_runs_when_a_button_was_pressed() {
        let mut w = crate::test_world();
        let from = w.new_mobile(&MOBILE_BASE);
        {
            let m = w.mobile_mut(from).unwrap();
            m.player = true;
            m.body = muo_primitives::Body(402); // a ghost
        }
        let ns = NetStateId(1);
        w.net_states.insert(
            ns,
            NetStateData {
                mobile: Some(from),
                ..Default::default()
            },
        );
        w.mobile_mut(from).unwrap().net_state = Some(ns);
        let panel = w.new_item(&ITEM_BASE);
        let mut g = PowerGeneratorGump {
            from,
            panel,
            step: 0,
            hint: false,
            side_length: 3,
            path: Vec::new(),
            lockpicking: 0.0,
        };
        let sent = |w: &mut World| std::mem::take(&mut w.net_out).len();
        g.on_response(&mut w, ns, &RelayInfo::new(0, vec![], vec![]));
        assert_eq!(sent(&mut w), 0, "closing says nothing");
        g.on_response(&mut w, ns, &RelayInfo::new(1, vec![], vec![]));
        assert_eq!(
            sent(&mut w),
            1,
            "a button press by a ghost: 'I am dead and cannot do that'"
        );
    }
}
