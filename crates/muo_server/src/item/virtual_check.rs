//! The "offer of currency" item in a secure trade window.
//!
//! The edit gump (`GumpActivator`, `IVirtualCheckGump`) is muo_gumps's; it fills [`VirtualCheckGumpHooks`]. `Editor`
//! is [`VirtualCheckData::editor`], the mobile whose gump it is.

use crate::Ext;
use crate::ids::{ItemId, MobileId};
use crate::item::ItemType;
use crate::item::deps;
use crate::item::item_type::ITEM_BASE;
use crate::item::ops;
use crate::net_state::TradeId;
use crate::property_list::PropertyList;
use crate::world::World;

/// `Server.Items.VirtualCheck`.
pub static VIRTUAL_CHECK: ItemType = ItemType {
    full_name: "Server.Items.VirtualCheck",
    parent: Some(&ITEM_BASE),
    construct: None,
    skip_serialization: |_, _| true,
    is_virtual_item: |_, _| true,
    display_weight: |_, _| false,
    display_loot_type: |_, _| false,
    default_weight: |_, _| 0.0,
    default_name: |_, _| Some("Offer Of Currency".to_string()),
    is_accessible_to,
    on_single_click,
    on_double_click_secure_trade,
    get_properties,
    on_after_delete,
    serialize: |w, id, wr| {
        (ITEM_BASE.serialize)(w, id, wr);
        wr.write_i32(0); // version
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        r.read_i32()?;
        Ok(())
    },
    ..ITEM_BASE
};

/// `_plat` and `_gold`.
#[derive(Clone, Debug, Default)]
pub struct VirtualCheckData {
    pub plat: i32,
    pub gold: i32,
    /// `Editor`: the user the open `IVirtualCheckGump` belongs to.
    pub editor: Option<MobileId>,
}

/// `GumpActivator` and the `IVirtualCheckGump` calls (`Send`, `Close`, `Refresh(true)`), set by muo_gumps.
/// Unset, `UseEditGump` has nothing to open and the base double click runs.
#[derive(Clone, Copy, Default)]
pub struct VirtualCheckGumpHooks {
    pub send: Option<fn(&mut World, MobileId, ItemId)>,
    pub close: Option<fn(&mut World, MobileId, ItemId)>,
    pub refresh: Option<fn(&mut World, MobileId, ItemId)>,
}

/// `UseEditGump`: `virtualChecks.useEditGump`, default `Core.TOL`.
fn use_edit_gump(w: &World) -> bool {
    w.config.get_setting(
        "virtualChecks.useEditGump",
        w.expansion() >= muo_data::expansion::Expansion::TOL,
    )
}

fn set_editor(w: &mut World, id: ItemId, user: Option<MobileId>) {
    if let Some(d) = w
        .item_mut(id)
        .and_then(|i| i.ext.get_mut::<VirtualCheckData>())
    {
        d.editor = user;
    }
}

fn on_double_click_secure_trade(w: &mut World, id: ItemId, from: MobileId) {
    let hooks = w
        .resources
        .get::<VirtualCheckGumpHooks>()
        .copied()
        .unwrap_or_default();
    let editor = w
        .item(id)
        .and_then(|i| i.ext.get::<VirtualCheckData>())
        .and_then(|d| d.editor);
    if let (Some(send), Some(refresh)) = (hooks.send, hooks.refresh)
        && use_edit_gump(w)
        && is_accessible_to(w, id, from)
    {
        match editor {
            Some(user) if !ops::is_deleted(w, id) => refresh(w, user, id),
            _ => {
                set_editor(w, id, Some(from));
                send(w, from, id);
            }
        }
    } else {
        if let Some(user) = editor {
            if let Some(close) = hooks.close {
                close(w, user, id);
            }
            set_editor(w, id, None);
        }
        crate::item::base::on_double_click_secure_trade_base(w, id, from);
    }
}

/// `OnAfterDelete`: closes the editor's gump.
fn on_after_delete(w: &mut World, id: ItemId) {
    crate::item::base::on_after_delete_base(w, id);
    let editor = w
        .item(id)
        .and_then(|i| i.ext.get::<VirtualCheckData>())
        .and_then(|d| d.editor);
    if let Some(user) = editor {
        if let Some(close) = w
            .resources
            .get::<VirtualCheckGumpHooks>()
            .and_then(|h| h.close)
        {
            close(w, user, id);
        }
        set_editor(w, id, None);
    }
}

/// `new VirtualCheck(plat, gold)`.
pub fn new_virtual_check(w: &mut World, plat: i32, gold: i32) -> ItemId {
    let id = ops::new_item_with_id(w, &VIRTUAL_CHECK, 0x14F0);
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(VirtualCheckData {
            plat,
            gold,
            editor: None,
        });
    }
    ops::set_movable(w, id, false);
    id
}

pub fn plat(w: &World, id: ItemId) -> i32 {
    w.item(id)
        .and_then(|i| i.ext.get::<VirtualCheckData>())
        .map_or(0, |d| d.plat)
}

pub fn gold(w: &World, id: ItemId) -> i32 {
    w.item(id)
        .and_then(|i| i.ext.get::<VirtualCheckData>())
        .map_or(0, |d| d.gold)
}

/// `Plat` setter.
pub fn set_plat(w: &mut World, id: ItemId, value: i32) {
    if let Some(d) = w
        .item_mut(id)
        .and_then(|i| i.ext.get_mut::<VirtualCheckData>())
    {
        d.plat = value;
    }
    ops::invalidate_properties(w, id);
}

/// `Gold` setter.
pub fn set_gold(w: &mut World, id: ItemId, value: i32) {
    if let Some(d) = w
        .item_mut(id)
        .and_then(|i| i.ext.get_mut::<VirtualCheckData>())
    {
        d.gold = value;
    }
    ops::invalidate_properties(w, id);
}

/// `#,0` formatting.
fn grouped(v: i32) -> String {
    let s = v.unsigned_abs().to_string();
    let mut out = String::new();
    for (n, c) in s.chars().enumerate() {
        if n > 0 && (s.len() - n).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    if v < 0 { format!("-{out}") } else { out }
}

fn is_accessible_to(w: &mut World, id: ItemId, check: MobileId) -> bool {
    let Some(c) = ops::get_secure_trade_cont(w, id) else {
        return crate::item::base::is_accessible_to_base(w, id, check);
    };
    ops::root_parent(w, c) == Some(check.into()) && ops::is_child_of(w, id, Some(c.into()))
}

fn on_single_click(w: &mut World, id: ItemId, from: MobileId) {
    let text = format!(
        "Offer: {} platinum, {} gold",
        grouped(plat(w, id)),
        grouped(gold(w, id))
    );
    ops::label_to(w, id, from, &text);
}

fn get_properties(w: &World, id: ItemId, list: &mut PropertyList) {
    crate::item::base::get_properties_base(w, id, list);
    list.add_number_fmt(
        1060738, // value: ~1_val~
        format_args!(
            "{} platinum, {} gold",
            grouped(plat(w, id)),
            grouped(gold(w, id))
        ),
    );
}

/// `UpdateTrade(user)`.
pub fn update_trade(w: &mut World, id: ItemId, user: MobileId) {
    let Some(cont) = ops::get_secure_trade_cont(w, id) else {
        return;
    };
    let Some(trade) = crate::item::secure_trade_container::trade_of(w, cont) else {
        return;
    };
    update_for(w, trade, cont, user);
}

fn update_for(w: &mut World, trade: TradeId, cont: ItemId, user: MobileId) {
    let (from, to) = deps::trade_parties(w, trade);
    if Some(user) == from {
        deps::trade_update_currency(w, trade, true);
    } else if Some(user) == to {
        deps::trade_update_currency(w, trade, false);
    }
    crate::item::secure_trade_container::clear_checks(w, cont);
}
