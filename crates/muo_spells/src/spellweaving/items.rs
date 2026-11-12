//! `TransientItem` and `ArcaneFocus`.

use muo_io::{GenericReader, GenericWriter, Result};
use muo_primitives::{DateTime, TimeSpan};
use muo_server::World;
use muo_server::ext::Ext;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, LootType, base, ops};
use muo_server::mobile::part2::messages as mm;
use muo_server::property_list::PropertyList;
use muo_server::timer::{self, TimerHandle};

/// What a `TransientItem` adds to an `Item`.
#[derive(Clone, Debug, Default)]
pub struct TransientData {
    pub expiration: DateTime,
    pub timer: TimerHandle,
}

/// What an `ArcaneFocus` adds.
#[derive(Clone, Debug, Default)]
pub struct FocusData {
    pub strength_bonus: i32,
}

/// `TransientItem.Expiration`.
pub fn expiration(w: &World, id: ItemId) -> DateTime {
    data(w, id).map_or(DateTime::default(), |d| d.expiration)
}

fn data(w: &World, id: ItemId) -> Option<&TransientData> {
    w.item(id)?.ext.get::<TransientData>()
}

fn data_mut(w: &mut World, id: ItemId) -> Option<&mut TransientData> {
    w.item_mut(id)?.ext.get_mut::<TransientData>()
}

/// `item.StrengthBonus`.
pub fn strength_bonus(w: &World, id: ItemId) -> i32 {
    w.item(id)
        .and_then(|i| i.ext.get::<FocusData>())
        .map_or(0, |d| d.strength_bonus)
}

/// `item.StrengthBonus = value`.
pub fn set_strength_bonus(w: &mut World, id: ItemId, value: i32) {
    if let Some(d) = w.item_mut(id).and_then(|i| i.ext.get_mut::<FocusData>()) {
        d.strength_bonus = value;
    }
}

/// `new TransientItem(itemID, lifeSpan)`'s state, on an already made item.
pub fn init_transient(w: &mut World, id: ItemId, life_span: TimeSpan) {
    let expiration = w.now + life_span;
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(TransientData {
            expiration,
            timer: TimerHandle::default(),
        });
    }
    start_timer(w, id);
}

fn start_timer(w: &mut World, id: ItemId) {
    let h = timer::start(
        w,
        TimeSpan::from_secs(5),
        TimeSpan::from_secs(5),
        0,
        move |w| check_expiry(w, id),
    );
    if let Some(d) = data_mut(w, id) {
        d.timer = h;
    }
}

/// `CheckExpiry()`.
pub fn check_expiry(w: &mut World, id: ItemId) {
    let Some(exp) = data(w, id).map(|d| d.expiration) else {
        return;
    };
    if exp - w.now <= TimeSpan::ZERO {
        let parent = match ops::root_parent(w, id) {
            Some(EntityId::Mobile(m)) => Some(m),
            _ => None,
        };
        expire(w, id, parent);
    } else {
        ops::invalidate_properties(w, id);
    }
}

/// `Expire(parent)`.
pub fn expire(w: &mut World, id: ItemId, parent: Option<MobileId>) {
    if let Some(p) = parent {
        // The ~1_name~ expired...
        let name = item_name_arg(w, id);
        mm::send_localized_message(w, p, 1072515, &name, 0x3B2);
    }
    let loc = ops::get_world_location(w, id);
    let map = w.item(id).and_then(|i| i.map);
    muo_server::effects::play_sound(w, loc, map, 0x201);
    base::delete(w, id);
}

fn item_name_arg(w: &World, id: ItemId) -> String {
    match w.item(id).and_then(|i| i.name.clone()) {
        Some(n) => n,
        None => {
            let ty = w.item(id).map(|i| i.ty);
            format!("#{}", ty.map_or(0, |t| (t.label_number)(w, id)))
        }
    }
}

/// `SendTimeRemainingMessage(to)`.
pub fn send_time_remaining_message(w: &mut World, id: ItemId, to: MobileId) {
    let remaining = data(w, id).map_or(TimeSpan::ZERO, |d| {
        (d.expiration - w.now).max(TimeSpan::ZERO)
    });
    // ~1_name~ will expire in ~2_val~ seconds!
    let args = format!("{}\t{:.0}", item_name_arg(w, id), remaining.total_seconds());
    mm::send_localized_message(w, to, 1072516, &args, 0x3B2);
}

/// `ResetExpiration(duration)`.
pub fn reset_expiration(w: &mut World, id: ItemId, duration: TimeSpan) {
    let exp = w.now + duration;
    if let Some(d) = data_mut(w, id) {
        d.expiration = exp;
    }
}

fn on_delete(w: &mut World, id: ItemId) {
    if let Some(d) = data_mut(w, id) {
        let mut t = d.timer;
        t.cancel(w);
    }
    base::on_delete_base(w, id);
}

fn handle_invalid_transfer(w: &mut World, id: ItemId, from: MobileId) {
    // `InvalidTransferMessage.SendMessageTo(from)`
    let n = (invalid_transfer_message(w, id))(w, id);
    if n != 0 {
        mm::send_localized_message(w, from, n, "", 0x3B2);
    }
    base::delete(w, id);
}

fn invalid_transfer_message(w: &World, id: ItemId) -> fn(&World, ItemId) -> i32 {
    if w.item(id).is_some_and(|i| i.ty.is(&ARCANE_FOCUS)) {
        |_, _| 1073480 // Your arcane focus disappears.
    } else {
        |_, _| 0
    }
}

fn get_properties(w: &World, id: ItemId, list: &mut PropertyList) {
    base::get_properties_base(w, id, list);
    let remaining = data(w, id).map_or(TimeSpan::ZERO, |d| {
        (d.expiration - w.now).max(TimeSpan::ZERO)
    });
    list.add_str(1072517, &format!("{:.0}", remaining.total_seconds())); // Lifespan: ~1_val~ seconds
}

fn serialize_transient(w: &World, id: ItemId, out: &mut dyn GenericWriter) {
    (ITEM_BASE.serialize)(w, id, out);
    out.write_i32(2); // SerializationVersion
    let exp = data(w, id).map_or(DateTime::MIN, |d| d.expiration);
    out.write_anchored_time_ticks(exp.ticks());
}

fn deserialize_transient(w: &mut World, id: ItemId, r: &mut dyn GenericReader) -> Result<()> {
    (ITEM_BASE.deserialize)(w, id, r)?;
    let version = r.read_i32()?;
    let now = w.now;
    let expiration = match version {
        0 => {
            // Deserialize(reader, 0): lifespan, then the creation time
            let lifespan = TimeSpan::from_ticks(r.read_time_span_ticks()?);
            let creation =
                DateTime::try_from_ticks(r.read_date_time_ticks()?).unwrap_or(DateTime::MIN);
            creation + lifespan
        }
        1 => {
            DateTime::try_from_ticks(r.read_delta_time_ticks(now.ticks())?).unwrap_or(DateTime::MIN)
        }
        _ => DateTime::try_from_ticks(r.read_anchored_time_ticks()?).unwrap_or(DateTime::MIN),
    };
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(TransientData {
            expiration,
            timer: TimerHandle::default(),
        });
    }
    // AfterDeserialization
    start_timer(w, id);
    Ok(())
}

/// `Server.Items.TransientItem`: an item that expires.
pub static TRANSIENT_ITEM: ItemType = ItemType {
    full_name: "Server.Items.TransientItem",
    parent: Some(&ITEM_BASE),
    construct: None,
    nontransferable: |_, _| true,
    handle_invalid_transfer,
    on_delete,
    get_properties,
    serialize: serialize_transient,
    deserialize: deserialize_transient,
    ..ITEM_BASE
};

/// `new ArcaneFocus(lifeSpan, strengthBonus)`.
pub fn new_arcane_focus(w: &mut World, life_span: TimeSpan, strength_bonus: i32) -> ItemId {
    let id = ops::new_item_with_id(w, &ARCANE_FOCUS, 0x3155);
    ops::set_loot_type(w, id, LootType::Blessed);
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(FocusData { strength_bonus });
    }
    init_transient(w, id, life_span);
    id
}

fn focus_properties(w: &World, id: ItemId, list: &mut PropertyList) {
    get_properties(w, id, list);
    list.add_int(1060485, strength_bonus(w, id)); // strength bonus ~1_val~
}

fn serialize_focus(w: &World, id: ItemId, out: &mut dyn GenericWriter) {
    serialize_transient(w, id, out);
    out.write_i32(0); // SerializationVersion
    out.write_i32(strength_bonus(w, id));
}

fn deserialize_focus(w: &mut World, id: ItemId, r: &mut dyn GenericReader) -> Result<()> {
    deserialize_transient(w, id, r)?;
    let _version = r.read_i32()?;
    let strength_bonus = r.read_i32()?;
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(FocusData { strength_bonus });
    }
    Ok(())
}

/// `Server.Items.ArcaneFocus`: `[Constructible]` as `ArcaneFocus()` (an hour, strength 1).
pub static ARCANE_FOCUS: ItemType = ItemType {
    full_name: "Server.Items.ArcaneFocus",
    parent: Some(&TRANSIENT_ITEM),
    construct: Some(|w| new_arcane_focus(w, TimeSpan::from_hours(1), 1)),
    label_number: |_, _| 1032629, // Arcane Focus
    get_properties: focus_properties,
    serialize: serialize_focus,
    deserialize: deserialize_focus,
    ..TRANSIENT_ITEM
};
