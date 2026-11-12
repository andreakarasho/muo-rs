//! `BankBox`.
//!
//! Save layout (`[SerializationGenerator(0, false)]`): `Container`'s part, then the version `int` 0, `Owner`
//! (a mobile serial, -1 for none) and `Opened`.

use muo_io::{GenericReader, GenericWriter, Result};

use crate::Ext;
use crate::ids::{ItemId, MobileId};
use crate::item::container::{self, CONTAINER};
use crate::item::deps;
use crate::item::ops;
use crate::item::serialize::{read_mobile, write_mobile};
use crate::item::{DeathMoveResult, ItemType, TotalType};
use crate::layer::Layer;
use crate::mobile::AccessLevel;
use crate::world::World;

/// `Server.Items.BankBox` (made by `Mobile.BankBox`, not by `[add`).
pub static BANK_BOX: ItemType = ItemType {
    full_name: "Server.Items.BankBox",
    parent: Some(&CONTAINER),
    construct: None,
    default_max_weight: |_, _| 0,
    is_virtual_item: |_, _| true,
    on_single_click: |_, _, _| {},
    on_double_click: |_, _, _| {},
    on_parent_death,
    is_accessible_to,
    on_drag_drop,
    on_drag_drop_into,
    check_hold,
    get_total,
    serialize,
    deserialize,
    ..CONTAINER
};

/// The fields `BankBox` adds.
#[derive(Clone, Debug, Default)]
pub struct BankBoxData {
    /// `Owner`.
    pub owner: Option<MobileId>,
    /// `Opened`.
    pub opened: bool,
}

/// `BankBox.SendDeleteOnClose`.
#[derive(Default)]
pub struct BankBoxStatics {
    pub send_delete_on_close: bool,
}

/// `item is BankBox`.
pub fn is_bank_box(w: &World, id: ItemId) -> bool {
    w.item(id).is_some_and(|i| i.ty.is(&BANK_BOX))
}

fn data(w: &World, id: ItemId) -> Option<&BankBoxData> {
    w.item(id)?.ext.get::<BankBoxData>()
}

fn data_mut(w: &mut World, id: ItemId) -> Option<&mut BankBoxData> {
    w.item_mut(id)?.ext.get_mut::<BankBoxData>()
}

/// `Owner`.
pub fn owner(w: &World, id: ItemId) -> Option<MobileId> {
    data(w, id).and_then(|d| d.owner)
}

/// `Opened`.
pub fn bank_box_opened(w: &World, id: ItemId) -> bool {
    data(w, id).is_some_and(|d| d.opened)
}

/// `new BankBox(owner)`.
pub fn new_bank_box(w: &mut World, owner: Option<MobileId>) -> ItemId {
    let id = container::new_container(w, &BANK_BOX, 0xE7C);
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(BankBoxData::default());
    }
    (BANK_BOX.set_layer)(w, id, Layer::Bank);
    ops::set_movable(w, id, false);
    if let Some(d) = data_mut(w, id) {
        d.owner = owner;
    }
    id
}

/// `Open()`.
pub fn open(w: &mut World, id: ItemId) {
    let owner = owner(w, id);
    // `Owner?.AccessLevel < Administrator` is false for a null owner.
    let restricted = owner.is_some_and(|o| {
        w.mobile(o)
            .is_some_and(|m| m.access_level < AccessLevel::Administrator)
    });
    if !deps::bank_access(w) && restricted {
        if let Some(o) = owner {
            deps::send_message_hue(w, o, 0x22, "Bank access is temporarily disabled.");
        }
        return;
    }

    if let Some(d) = data_mut(w, id) {
        d.opened = true;
    }

    if let Some(o) = owner {
        let items = ops::get_total(w, id, TotalType::Items);
        let weight = ops::get_total(w, id, TotalType::Weight);
        deps::private_overhead_message(
            w,
            o,
            0x3B2,
            &format!("Bank container has {items} items, {weight} stones"),
        );

        if let Some(ns) = deps::net_state_of(w, o) {
            crate::packets::equipment::send_equip_update(w, ns, id);
        }
        if let Some(ty) = w.item(id).map(|i| i.ty) {
            (ty.display_to)(w, id, o);
        }
    }
}

/// `Close()`.
pub fn bank_box_close(w: &mut World, id: ItemId) {
    if let Some(d) = data_mut(w, id) {
        d.opened = false;
    }
    let send = w
        .resources
        .get::<BankBoxStatics>()
        .is_some_and(|s| s.send_delete_on_close);
    if send
        && let Some(o) = owner(w, id)
        && let Some(ns) = deps::net_state_of(w, o)
    {
        crate::packets::entity::send_remove_entity(w, ns, id.0);
    }
}

fn on_parent_death(w: &mut World, id: ItemId, parent: MobileId) -> DeathMoveResult {
    container::remain_equipped(w, id, parent)
}

fn is_accessible_to(w: &mut World, id: ItemId, check: MobileId) -> bool {
    let staff = w
        .mobile(check)
        .is_some_and(|m| m.access_level >= AccessLevel::GameMaster);
    ((Some(check) == owner(w, id) && bank_box_opened(w, id)) || staff)
        && crate::item::base::is_accessible_to_base(w, id, check)
}

fn may_use(w: &World, id: ItemId, from: MobileId) -> bool {
    (Some(from) == owner(w, id) && bank_box_opened(w, id))
        || w.mobile(from)
            .is_some_and(|m| m.access_level >= AccessLevel::GameMaster)
}

fn on_drag_drop(w: &mut World, id: ItemId, from: MobileId, dropped: ItemId) -> bool {
    may_use(w, id, from) && (CONTAINER.on_drag_drop)(w, id, from, dropped)
}

fn on_drag_drop_into(
    w: &mut World,
    id: ItemId,
    from: MobileId,
    item: ItemId,
    p: muo_primitives::Point3D,
) -> bool {
    may_use(w, id, from) && container::on_drag_drop_into_default(w, id, from, item, p)
}

#[allow(clippy::too_many_arguments)]
fn check_hold(
    w: &mut World,
    id: ItemId,
    m: Option<MobileId>,
    item: ItemId,
    message: bool,
    check_items: bool,
    plus_items: i32,
    plus_weight: i32,
) -> bool {
    // This is a horrible hack. TODO: Refactor this by moving BankBox out of the core.
    let (enabled, convert) = (
        deps::account_gold_enabled(w),
        deps::account_gold_convert_on_bank(w),
    );
    if enabled && convert {
        let name = w
            .item(item)
            .map(|i| i.ty.full_name.rsplit('.').next().unwrap_or(""));
        if matches!(name, Some("Gold" | "BankCheck")) {
            return true;
        }
    }
    container::check_hold_default(
        w,
        id,
        m,
        item,
        message,
        check_items,
        plus_items,
        plus_weight,
    )
}

fn get_total(w: &World, id: ItemId, ty: TotalType) -> i32 {
    if deps::account_gold_enabled(w)
        && ty == TotalType::Gold
        && let Some(owner) = owner(w, id)
        && let Some(total) = deps::account_total_gold(w, owner)
    {
        return total;
    }
    (CONTAINER.get_total)(w, id, ty)
}

fn serialize(w: &World, id: ItemId, writer: &mut dyn GenericWriter) {
    container::serialize(w, id, writer);
    writer.write_i32(0); // version
    write_mobile(w, writer, owner(w, id));
    writer.write_bool(bank_box_opened(w, id));
}

fn deserialize(w: &mut World, id: ItemId, reader: &mut dyn GenericReader) -> Result<()> {
    (CONTAINER.deserialize)(w, id, reader)?;
    let _version = reader.read_i32()?;
    let owner = read_mobile(w, reader)?;
    let opened = reader.read_bool()?;
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(BankBoxData { owner, opened });
    }

    // [AfterDeserialization]
    if owner.is_none() {
        deps::delay_delete(w, id);
    }
    Ok(())
}
