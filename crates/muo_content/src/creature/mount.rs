//! `BaseMount` (a creature a player rides) and `MountItem`, the
//! invisible item on the rider's mount layer that stands for it. Leaf mounts (Horse, Nightmare...) are
//! `creature_leaf!` classes under `BASE_MOUNT`; their `StepsMax`, `MountAbilityDelay`, `DoMountAbility` ... are
//! [`CreatureType`] fields.

use muo_io::{GenericReader, GenericWriter};
use muo_primitives::{DateTime, MessageType, TimeSpan};
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{DeathMoveResult, ITEM_BASE, ItemType};
use muo_server::layer::Layer;
use muo_server::map::MapId;
use muo_server::mobile::part1::{lifecycle, props_b};
use muo_server::mobile::part2::{deps, items as mitems, messages};
use muo_server::mobile::{AccessLevel, MobileType};
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::timer;
use muo_server::world::World;

use super::control::get_master;
use super::data::{cd, read};
use super::enums::AIType;
pub use super::vtable::BASE_CREATURE_C as BASE_MOUNT_C;
use super::vtable::{BASE_CREATURE, BASE_CREATURE_C, ctype};
use crate::misc::stamina_system::{self, StepTraits, StepsKey};
use crate::mx;

/// `BaseMount`'s fields: `_nextMountAbility`, `_internalItem`, `_rider`.
#[derive(Default)]
pub struct MountState {
    pub next_mount_ability: DateTime,
    pub internal_item: Option<ItemId>,
    pub rider: Option<MobileId>,
}

/// `MountItem._mount`.
pub struct MountItemData {
    pub mount: Option<MobileId>,
}

pub fn state(w: &World, id: MobileId) -> Option<&MountState> {
    w.mobile(id)?.ext.get::<MountState>()
}

fn state_mut(w: &mut World, id: MobileId) -> Option<&mut MountState> {
    w.mobile_mut(id)?.ext.get_mut::<MountState>()
}

/// `m is BaseMount`.
pub fn is_base_mount(w: &World, id: MobileId) -> bool {
    state(w, id).is_some()
}

/// `BaseMount.Rider`.
pub fn rider(w: &World, id: MobileId) -> Option<MobileId> {
    state(w, id)?.rider
}

/// `BaseMount.InternalItem`.
pub fn internal_item(w: &World, id: MobileId) -> Option<ItemId> {
    state(w, id)?.internal_item
}

/// `MountItem._mount`.
pub fn mount_of_item(w: &World, item: ItemId) -> Option<MobileId> {
    w.item(item)?.ext.get::<MountItemData>()?.mount
}

pub static MOUNT_ITEM: ItemType = ItemType {
    full_name: "Server.Mobiles.MountItem",
    parent: Some(&ITEM_BASE),
    // `new MountItem(BaseMount, int)` is not [Constructible].
    construct: None,
    serialize: item_serialize,
    deserialize: item_deserialize,
    default_weight: |_, _| 0.0,
    on_after_delete: item_on_after_delete,
    on_parent_death: item_on_parent_death,
    ..ITEM_BASE
};

fn item_serialize(w: &World, id: ItemId, out: &mut dyn GenericWriter) {
    (ITEM_BASE.serialize)(w, id, out);
    out.write_i32(0);
    out.write_mobile(w, mount_of_item(w, id));
}

fn item_deserialize(w: &mut World, id: ItemId, r: &mut dyn GenericReader) -> muo_io::Result<()> {
    (ITEM_BASE.deserialize)(w, id, r)?;
    let _version = r.read_i32()?;
    let mount = r.read_mobile(w)?;
    if let Some(it) = w.item_mut(id) {
        it.ext = muo_server::Ext::new(MountItemData { mount });
    }
    // [AfterDeserialization(false)]: `Timer.DelayCall(AfterDeserialize)`.
    timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 1, move |w| {
        if w.item(id).is_some() && mount_of_item(w, id).is_none() {
            muo_server::item::base::delete(w, id);
        }
    });
    Ok(())
}

fn item_on_after_delete(w: &mut World, id: ItemId) {
    let mount = mount_of_item(w, id);
    if let Some(it) = w.item_mut(id) {
        it.ext = muo_server::Ext::new(MountItemData { mount: None });
    }
    if let Some(m) = mount {
        muo_server::mobile::base::delete(w, m);
    }
    (ITEM_BASE.on_after_delete)(w, id);
}

fn item_on_parent_death(w: &mut World, id: ItemId, _parent: MobileId) -> DeathMoveResult {
    if let Some(m) = mount_of_item(w, id) {
        set_rider(w, m, None);
    }
    DeathMoveResult::RemainEquipped
}

/// `new MountItem(mount, itemID)`.
fn new_mount_item(w: &mut World, mount: MobileId, item_id: i32) -> ItemId {
    let id = w.new_item(&MOUNT_ITEM);
    if let Some(it) = w.item_mut(id) {
        it.item_id = item_id;
        it.layer = Layer::Mount;
        it.ext = muo_server::Ext::new(MountItemData { mount: Some(mount) });
    }
    muo_server::item::ops::set_movable(w, id, false);
    id
}

/// `BaseMount(bodyID, itemID, aiType, fightMode, rangePerception, rangeFight)`.
#[allow(clippy::too_many_arguments)] // mirrors the BaseMount constructor
pub fn new_mount(
    w: &mut World,
    ty: &'static MobileType,
    body_id: i32,
    item_id: i32,
    ai: AIType,
    mode: super::enums::FightMode,
    range_perception: i32,
    range_fight: i32,
) -> MobileId {
    let id = super::setters::new_creature(w, ty, ai, mode, range_perception, range_fight);
    if let Some(m) = w.mobile_mut(id) {
        m.ext.insert(MountState::default());
    }
    props_b::set_body(w, id, muo_primitives::Body(body_id));
    let item = new_mount_item(w, id, item_id);
    if let Some(s) = state_mut(w, id) {
        s.internal_item = Some(item);
    }
    id
}

/// `BaseMount.ItemID` set.
pub fn set_item_id(w: &mut World, id: MobileId, value: i32) {
    if let Some(item) = internal_item(w, id) {
        muo_server::item::base::set_item_id_base(w, item, value);
    }
}

/// `BaseMount.Dismount(m)`.
pub fn dismount(w: &mut World, m: MobileId) {
    if let Some(mount) = props_b::mount(w, m) {
        deps::mount_clear_rider(w, mount);
    }
}

/// `BaseMount.Rider` set.
pub fn set_rider(w: &mut World, id: MobileId, value: Option<MobileId>) {
    let current = rider(w, id);
    if current == value {
        return;
    }
    match (value, current) {
        (None, Some(r)) => {
            let (mut loc, mut map) = (mx::location(w, r), mx::map(w, r));
            if map.is_none_or(|m| m == MapId::INTERNAL)
                && let Some(rm) = w.mobile(r)
            {
                loc = rm.logout_location;
                map = rm.logout_map;
            }
            if let Some(dir) = w.mobile(r).map(|m| m.direction) {
                props_b::set_direction(w, id, dir);
            }
            lifecycle::set_location(w, id, loc);
            lifecycle::set_map(w, id, map);
            if let Some(item) = internal_item(w, id) {
                muo_server::item::ops::internalize(w, item);
            }
        }
        (Some(v), cur) => {
            if let Some(c) = cur {
                dismount(w, c);
            }
            dismount(w, v);
            if let Some(item) = internal_item(w, id) {
                mitems::add_item(w, v, item);
            }
            if let Some(dir) = w.mobile(id).map(|m| m.direction) {
                props_b::set_direction(w, v, dir);
            }
            mitems::internalize(w, id);
            if let Some(f) = crate::hooks::hooks(w).and_then(|h| h.cancel_bola_target) {
                f(w, v);
            }
        }
        (None, None) => {}
    }
    if let Some(s) = state_mut(w, id) {
        s.rider = value;
    }
    if value.is_none() {
        stamina_system::on_dismount(w, StepsKey::Entity(EntityId::Mobile(id)));
    }
}

/// `BaseMount.OnRiderDamaged`.
pub fn on_rider_damaged(
    w: &mut World,
    id: MobileId,
    amount: i32,
    from: Option<MobileId>,
    will_kill: bool,
) {
    let Some(r) = rider(w, id) else { return };
    let attacker = match from {
        Some(f) => Some(f),
        None => muo_server::mobile::part2::damage::find_most_recent_damager(w, r, true),
    };
    let next = state(w, id).map_or(DateTime::MIN, |s| s.next_mount_ability);
    if !(attacker == Some(id) || attacker == Some(r) || will_kill || w.now < next) {
        let c = ctype(w, id).expect("creature");
        if (c.do_mount_ability)(w, id, amount, from) {
            let delay = (c.mount_ability_delay)(w, id);
            let now = w.now;
            if let Some(s) = state_mut(w, id) {
                s.next_mount_ability = now + delay;
            }
        }
    }
}

/// `BaseMount.CheckMountAllowed(mob)`.
pub fn check_mount_allowed(w: &mut World, mob: MobileId) -> bool {
    let mut result = true;
    if crate::player_mobile::data(w, mob).is_some() {
        let region = w.mobile(mob).map(|m| m.region);
        if region
            .and_then(|r| crate::systems::regions::base_region::family(w, r))
            .is_some_and(|f| !f.mounts_allowed)
        {
            mx::msg(w, mob, 1042317); // You may not ride at this time
            result = false;
        }
        let reason = crate::player_mobile::members::mount_block_reason(w, mob);
        if reason != crate::player_mobile::BlockMountType::None {
            mx::msg(w, mob, reason as i32);
            result = false;
        }
        let (gargoyle, ns, hue) = {
            let (_, name, g) = muo_server::item::deps::race_allows(w, mob, 0);
            let _ = name;
            (
                g,
                w.mobile(mob).and_then(|m| m.net_state),
                w.mobile(mob).map_or(0, |m| m.speech_hue),
            )
        };
        if gargoyle {
            messages::private_overhead_message_number(
                w,
                mob,
                MessageType::REGULAR,
                hue,
                1112281,
                "",
                ns,
            ); // Gargoyles are unable to ride animals.
            result = false;
        }
    }
    result
}

/// `OnDoubleClick(from)`.
fn on_double_click(w: &mut World, id: MobileId, from: MobileId) {
    if read(w, id, false, |d| d.is_dead_pet) {
        return;
    }
    let ns = w.mobile(from).and_then(|m| m.net_state);
    let private = |w: &mut World, number: i32| {
        messages::private_overhead_message_number(
            w,
            id,
            MessageType::REGULAR,
            0x3B2,
            number,
            "",
            ns,
        );
    };
    if props_b::is_body_mod(w, from) && !super::misc::body_is_human(w, from) {
        if w.core_aos() {
            private(w, 1062061); // You cannot ride a mount in your current form.
        } else {
            mx::msg(w, from, 1061628); // You can't do that while polymorphed.
        }
        return;
    }
    if !check_mount_allowed(w, from) {
        return;
    }
    if props_b::mounted(w, from) {
        mx::msg(w, from, 1005583); // Please dismount first.
        return;
    }
    let c = ctype(w, id).expect("creature");
    let female = w.mobile(from).is_some_and(|m| m.female);
    let allowed = if female {
        (c.allow_female_rider)(w, id)
    } else {
        (c.allow_male_rider)(w, id)
    };
    if !allowed {
        (c.on_disallowed_rider)(w, id, from);
        return;
    }
    if !crate::player_mobile::hooks::design_check(w, from) {
        return;
    }
    if props_b::has_trade(w, from) {
        messages::send_localized_message(w, from, 1042317, "", 0x41); // You may not ride at this time
        return;
    }
    if mx::in_range(w, from, id, 1) {
        let can_access =
            mx::access_level(w, from) >= AccessLevel::GameMaster || get_master(w, id) == Some(from);
        if can_access {
            if w.mobile(id).is_some_and(|m| m.poison.is_some()) {
                private(w, 1049692); // This mount is too ill to ride.
            } else {
                set_rider(w, id, Some(from));
            }
        } else if !read(w, id, false, |d| d.controlled) && !read(w, id, false, |d| d.summoned) {
            private(w, 501263); // That mount does not look broken! You would have to tame it to ride it.
        } else {
            private(w, 501264); // This isn't your mount; it refuses to let you ride.
        }
    } else {
        mx::msg(w, from, 500206); // That is too far away to ride.
    }
}

fn set_hue(w: &mut World, id: MobileId, value: i32) {
    (BASE_CREATURE.set_hue)(w, id, value);
    if let Some(item) = internal_item(w, id) {
        muo_server::item::base::set_hue_base(w, item, value);
    }
}

fn on_before_death(w: &mut World, id: MobileId) -> bool {
    set_rider(w, id, None);
    (BASE_CREATURE.on_before_death)(w, id)
}

fn on_after_delete(w: &mut World, id: MobileId) {
    if let Some(item) = internal_item(w, id) {
        // `InternalItem?.Delete()`: the item's own OnAfterDelete deletes the mount, which is already going.
        if let Some(it) = w.item_mut(item) {
            it.ext = muo_server::Ext::new(MountItemData { mount: None });
        }
        muo_server::item::base::delete(w, item);
    }
    if let Some(s) = state_mut(w, id) {
        s.internal_item = None;
    }
    (BASE_CREATURE.on_after_delete)(w, id);
}

fn on_delete(w: &mut World, id: MobileId) {
    set_rider(w, id, None);
    (BASE_CREATURE.on_delete)(w, id);
}

fn serialize(w: &World, id: MobileId, out: &mut dyn GenericWriter) {
    (BASE_CREATURE.serialize)(w, id, out);
    let s = state(w, id);
    out.write_i32(0);
    out.write_date_time_ticks(s.map_or(DateTime::MIN, |s| s.next_mount_ability).ticks());
    out.write_mobile(w, s.and_then(|s| s.rider));
    out.write_item(w, s.and_then(|s| s.internal_item));
}

fn deserialize(w: &mut World, id: MobileId, r: &mut dyn GenericReader) -> muo_io::Result<()> {
    (BASE_CREATURE.deserialize)(w, id, r)?;
    let _version = r.read_i32()?;
    let next = DateTime::from_ticks(r.read_date_time_ticks()?);
    let rider = r.read_mobile(w)?;
    let internal_item = r.read_item(w)?;
    if let Some(m) = w.mobile_mut(id) {
        m.ext.insert(MountState {
            next_mount_ability: next,
            internal_item,
            rider,
        });
    }
    // [AfterDeserialization(false)]: a mount without its item is deleted.
    timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 1, move |w| {
        if w.mobile(id).is_some() && internal_item_missing(w, id) {
            muo_server::mobile::base::delete(w, id);
        }
    });
    Ok(())
}

fn internal_item_missing(w: &World, id: MobileId) -> bool {
    internal_item(w, id).is_none_or(|i| w.item(i).is_none())
}

/// `Server.Mobiles.BaseMount`.
pub static BASE_MOUNT: MobileType = MobileType {
    full_name: "Server.Mobiles.BaseMount",
    parent: Some(&BASE_CREATURE),
    construct: None,
    family: Some(&BASE_CREATURE_C),
    serialize,
    deserialize,
    set_hue,
    on_before_death,
    on_after_delete,
    on_delete,
    on_double_click,
    ..BASE_CREATURE
};

/// `IHasSteps` of a `BaseMount`.
pub fn step_traits(w: &World, id: MobileId) -> Option<StepTraits> {
    if !is_base_mount(w, id) {
        return None;
    }
    let c = ctype(w, id)?;
    Some(StepTraits {
        steps_max: (c.steps_max)(w, id),
        steps_gained_per_idle_time: (c.steps_gained_per_idle_time)(w, id),
        idle_time_per_steps_gain: (c.idle_time_per_steps_gain)(w, id),
    })
}

/// Sets the hooks the server and the stamina system call into mounts with.
pub fn configure(w: &mut World) {
    w.resources.get_mut::<deps::MountDeps>().on_rider_damaged =
        Some(|w, mount, amount, from, will_kill| {
            if is_base_mount(w, mount) {
                on_rider_damaged(w, mount, amount, from, will_kill);
            }
        });
    w.resources.get_mut::<deps::MountDeps>().clear_rider = Some(|w, mount| {
        if is_base_mount(w, mount) {
            set_rider(w, mount, None);
        } else if let Some(f) = crate::hooks::hooks(w).and_then(|h| h.ethereal_dismount) {
            // the mount of an EtherealMount rider is the rider itself
            f(w, mount);
        }
    });
    let _ = cd;
}
