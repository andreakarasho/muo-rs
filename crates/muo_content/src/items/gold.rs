//! Gold, bank checks, keys and key rings.
//!
//! The key targets use their fixed ranges and `TargetFlags.None`; `BaseDoor` targets reach doors through
//! [`CommonItemHooks`].

use std::rc::Rc;

use muo_io::Result;
use muo_primitives::utility;
use muo_server::World;
use muo_server::ext::Ext;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ItemType, LootType, TotalType, base, container, containers, deps, ops};
use muo_server::packets::message::{AffixType, MessageType};
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::skills::SkillName;
use muo_server::target::{self, TargetFlags, Targeted};
use muo_server::type_registry::TypeRegistry;

use super::common::hooks;
use super::cont;
use crate::core;

/// `{n:N0}`.
pub fn n0(n: i64) -> String {
    let s = n.unsigned_abs().to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    if n < 0 { format!("-{out}") } else { out }
}

// ---- the AccountGold conversion shared by Gold and BankCheck ----

/// The tail of `Gold.OnAdded` / `BankCheck.OnAdded`: when `parent` is a bank box or a trade window with account
/// gold on, the amount goes to the owner's account and the item is deleted. Returns true when it did.
fn deposit_on_added(w: &mut World, id: ItemId, parent: EntityId, amount: i32) -> bool {
    if !muo_server::iaccount::gold_enabled(w) {
        return false;
    }
    let acct = w.resources.get::<muo_server::iaccount::AccountGold>();
    let (on_trade, on_bank, threshold) = acct.map_or((false, false, 1_000_000_000), |a| {
        (a.convert_on_trade, a.convert_on_bank, a.currency_threshold)
    });

    // var root = parent as Container; while (root?.Parent is Container c) root = c; parent = root ?? parent;
    let mut parent = parent;
    if let EntityId::Item(mut root) = parent
        && w.item(root).is_some_and(|i| i.cont.is_some())
    {
        while let Some(EntityId::Item(p)) = w.item(root).and_then(|i| i.parent) {
            if w.item(p).is_some_and(|i| i.cont.is_some()) {
                root = p;
            } else {
                break;
            }
        }
        parent = EntityId::Item(root);
    }

    let mut owner = None;
    // (trade, true when the owner is `From`)
    let mut trade_info: Option<(muo_server::net_state::TradeId, bool)> = None;
    if let EntityId::Item(p) = parent {
        let ty = w.item(p).map(|i| i.ty);
        let is_trade_cont = ty.is_some_and(|t| {
            t.is(&muo_server::item::secure_trade_container::SECURE_TRADE_CONTAINER)
        });
        if is_trade_cont && on_trade {
            if let Some(tid) = muo_server::item::secure_trade_container::trade_of(w, p)
                && let Some(t) = muo_server::secure_trade::trade(w, tid)
            {
                if t.from.container == Some(p) {
                    trade_info = Some((tid, true));
                    owner = Some(t.from.mobile);
                } else if t.to.container == Some(p) {
                    trade_info = Some((tid, false));
                    owner = Some(t.to.mobile);
                }
            }
        } else if on_bank && containers::is_bank_box(w, p) {
            owner = containers::owner(w, p);
        }
    }

    let Some(owner) = owner else { return false };
    if !muo_server::iaccount::account_deposit_gold(w, owner, amount) {
        return false;
    }

    if let Some((tid, from_side)) = trade_info {
        let (check, mobile) = match muo_server::secure_trade::trade(w, tid) {
            Some(t) => {
                let i = if from_side { &t.from } else { &t.to };
                (i.virtual_check, i.mobile)
            }
            None => (None, owner),
        };
        let has_ns = deps::net_state_of(w, owner).is_some();
        if has_ns
            && !deps::new_secure_trading(w, owner)
            && let Some(c) = check
        {
            let plat = amount / threshold;
            let gold = amount % threshold;
            let (p0, g0) = (
                muo_server::item::virtual_check::plat(w, c),
                muo_server::item::virtual_check::gold(w, c),
            );
            muo_server::item::virtual_check::set_plat(w, c, p0 + plat);
            muo_server::item::virtual_check::set_gold(w, c, g0 + gold);
        }
        if let Some(c) = check {
            muo_server::item::virtual_check::update_trade(w, c, mobile);
        }
    }

    muo_server::mobile::part2::messages::send_localized_message(
        w,
        owner,
        1042763,
        &n0(i64::from(amount)),
        0x3B2,
    );
    if let Some(ty) = w.item(id).map(|i| i.ty) {
        (ty.delete)(w, id);
    }
    if let EntityId::Item(p) = parent
        && let Some(ty) = w.item(p).map(|i| i.ty)
    {
        (ty.update_totals)(w, p);
    }
    true
}

// ---- Gold ----

/// `new Gold(amount)`.
pub fn new_gold(w: &mut World, amount: i32) -> ItemId {
    let id = crate::template_keys::template_new(w, &GOLD);
    ops::set_amount(w, id, amount);
    id
}

/// `Server.Items.Gold`.
pub static GOLD: ItemType = ItemType {
    full_name: "Server.Items.Gold",
    parent: Some(&muo_server::item::ITEM_BASE),
    construct: Some(|w| new_gold(w, 1)),
    default_weight: |w, _| if core::ml(w) { 0.02 / 3.0 } else { 0.02 },
    get_drop_sound: |w, id| match w.item(id).map_or(0, |i| i.amount) {
        ..=1 => 0x2E4,
        2..=5 => 0x2E5,
        _ => 0x2E6,
    },
    on_amount_change: |w, id, old| {
        let new = w.item(id).map_or(0, |i| i.amount);
        ops::update_total(w, id, id, TotalType::Gold, new - old);
    },
    on_added: |w, id, parent| {
        base::on_added_base(w, id, parent);
        let amount = w.item(id).map_or(0, |i| i.amount);
        deposit_on_added(w, id, parent, amount);
    },
    get_total: |w, id, ty| {
        let mut total = base::get_total_base(w, id, ty);
        if ty == TotalType::Gold {
            total += w.item(id).map_or(0, |i| i.amount);
        }
        total
    },
    serialize: |w, id, out| {
        base::serialize_base(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        base::deserialize_base(w, id, r)?;
        let _version = r.read_i32()?;
        Ok(())
    },
    ..muo_server::item::ITEM_BASE
};

// ---- BankCheck ----

/// What `BankCheck` adds.
#[derive(Clone, Debug, Default)]
pub struct BankCheckData {
    pub worth: i32,
}

pub fn worth(w: &World, id: ItemId) -> i32 {
    w.item(id)
        .and_then(|i| i.ext.get::<BankCheckData>())
        .map_or(0, |d| d.worth)
}

/// `Worth = value` (`[InvalidateProperties]`).
pub fn set_worth(w: &mut World, id: ItemId, value: i32) {
    if let Some(d) = w
        .item_mut(id)
        .and_then(|i| i.ext.get_mut::<BankCheckData>())
    {
        d.worth = value;
    }
    ops::invalidate_properties(w, id);
}

/// `new BankCheck(worth)`.
pub fn new_bank_check(w: &mut World, worth: i32) -> ItemId {
    let id = ops::new_item_with_id(w, &BANK_CHECK, 0x14F0);
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(BankCheckData { worth });
    }
    (BANK_CHECK.set_hue)(w, id, 0x34);
    ops::set_loot_type(w, id, LootType::Blessed);
    id
}

fn bank_check_double_click(w: &mut World, id: ItemId, from: MobileId) {
    // This probably isn't OSI accurate, but we can't just make the quests redundant.
    let enabled = muo_server::iaccount::gold_enabled(w);
    let b = if enabled {
        muo_server::mobile::part1::props_b::backpack(w, from)
    } else {
        muo_server::mobile::part2::items::find_bank_no_create(w, from)
    };
    let in_box = b.is_some_and(|b| ops::is_child_of(w, id, Some(EntityId::Item(b))));
    if !in_box {
        // This must be in your backpack to use it. : That must be in your bank box to use it.
        deps::send_localized_message(w, from, if enabled { 1080058 } else { 1047026 });
        return;
    }
    let value = worth(w, id);
    if enabled {
        if !muo_server::iaccount::account_deposit_gold(w, from, value) {
            return;
        }
        if let Some(ty) = w.item(id).map(|i| i.ty) {
            (ty.delete)(w, id);
        }
        // Gold was deposited in your account:
        muo_server::mobile::part2::messages::send_localized_message_affix(
            w,
            from,
            1042672,
            true,
            &n0(i64::from(value)),
            "",
            0x3B2,
        );
    } else {
        // Internalize the check to free its slot for deposit; reserve one slot so it can bounce back.
        ops::record_bounce(w, id);
        ops::internalize(w, id);
        let deposited = hooks(w)
            .and_then(|h| h.banker_deposit_up_to)
            .map_or(0, |f| f(w, from, value, false, 1));
        if deposited >= value {
            if let Some(ty) = w.item(id).map(|i| i.ty) {
                (ty.delete)(w, id);
            }
        } else if deposited > 0 {
            set_worth(w, id, value - deposited);
            ops::bounce(w, id, from);
        } else {
            ops::bounce(w, id, from);
            deps::send_localized_message(w, from, 500390); // Your bank box is full.
            return;
        }
        muo_server::mobile::part2::messages::send_localized_message_affix(
            w,
            from,
            1042672,
            true,
            &n0(i64::from(deposited)),
            "",
            0x3B2,
        );
    }
    if let Some(f) = hooks(w).and_then(|h| h.bank_check_cashed) {
        f(w, from);
    }
}

/// `Server.Items.BankCheck`.
pub static BANK_CHECK: ItemType = ItemType {
    full_name: "Server.Items.BankCheck",
    parent: Some(&muo_server::item::ITEM_BASE),
    construct: None,
    make: Some(|w| new_bank_check(w, 0)),
    default_weight: |w, id| {
        crate::template_keys::item_float(
            w,
            id,
            "defaultWeight",
            muo_server::item::ITEM_BASE.default_weight,
        )
    },
    display_loot_type: |w, _| core::aos(w),
    label_number: |w, id| {
        crate::template_keys::item_int(
            w,
            id,
            "labelNumber",
            muo_server::item::ITEM_BASE.label_number,
        )
    },
    get_properties: |w, id, list| {
        base::get_properties_base(w, id, list);
        if core::ml(w) {
            list.add_str(1060738, &n0(i64::from(worth(w, id)))); // value: ~1_val~
        } else {
            list.add_int(1060738, worth(w, id));
        }
    },
    on_added: |w, id, parent| {
        base::on_added_base(w, id, parent);
        let v = worth(w, id);
        deposit_on_added(w, id, parent, v);
    },
    on_single_click: |w, id, from| {
        let Some(ns) = deps::net_state_of(w, from) else {
            return;
        };
        let item_id = w.item(id).map_or(0, |i| i.item_id);
        let affix = format!(" {}", worth(w, id));
        muo_server::packets::message::send_message_localized_affix(
            w,
            ns,
            id.0,
            item_id,
            MessageType::LABEL,
            0x3B2,
            3,
            1041361,
            "",
            AffixType::APPEND,
            &affix,
            "",
        );
    },
    on_double_click: bank_check_double_click,
    serialize: |w, id, out| {
        base::serialize_base(w, id, out);
        out.write_i32(0);
        out.write_i32(worth(w, id));
    },
    deserialize: |w, id, r| {
        base::deserialize_base(w, id, r)?;
        let _version = r.read_i32()?;
        let worth = r.read_i32()?;
        if let Some(i) = w.item_mut(id) {
            i.ext = Ext::new(BankCheckData { worth });
        }
        Ok(())
    },
    ..muo_server::item::ITEM_BASE
};

// ---- Key ----

int_enum! {
    /// `KeyType`: the item id of the key.
    pub enum KeyType { Copper = 0x100E, Gold = 0x100F, Iron = 0x1010, Rusty = 0x1013 }
}

/// What `Key` adds.
#[derive(Clone, Debug)]
pub struct KeyData {
    pub max_range: i32,
    pub link: Option<ItemId>,
    pub description: Option<String>,
    pub key_value: u32,
}

fn key_data(w: &World, id: ItemId) -> Option<&KeyData> {
    w.item(id)?.ext.get::<KeyData>()
}

fn key_data_mut(w: &mut World, id: ItemId) -> Option<&mut KeyData> {
    w.item_mut(id)?.ext.get_mut::<KeyData>()
}

/// `item is Key`.
pub fn is_key(w: &World, id: ItemId) -> bool {
    w.item(id).is_some_and(|i| i.ty.is(&KEY))
}

/// `Key.Link`.
pub fn key_link(w: &World, id: ItemId) -> Option<ItemId> {
    key_data(w, id).and_then(|d| d.link)
}

/// `Key.KeyValue`.
pub fn key_value(w: &World, id: ItemId) -> u32 {
    key_data(w, id).map_or(0, |d| d.key_value)
}

/// `Key.Description` (`[InvalidateProperties]`).
pub fn set_key_description(w: &mut World, id: ItemId, value: Option<String>) {
    if let Some(d) = key_data_mut(w, id) {
        d.description = value;
    }
    ops::invalidate_properties(w, id);
}

/// `Key.KeyValue = value` (`[InvalidateProperties]`).
pub fn set_key_value(w: &mut World, id: ItemId, value: u32) {
    if let Some(d) = key_data_mut(w, id) {
        d.key_value = value;
    }
    ops::invalidate_properties(w, id);
}

/// `new Key(type, val, link)`.
pub fn new_key_of(w: &mut World, ty: KeyType, val: u32, link: Option<ItemId>) -> ItemId {
    let id = ops::new_item_with_id(w, &KEY, ty as i32);
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(KeyData {
            max_range: 3,
            link,
            description: None,
            key_value: val,
        });
    }
    id
}

/// `new Key(val)` (an iron key).
pub fn new_key(w: &mut World, val: u32) -> ItemId {
    new_key_of(w, KeyType::Iron, val, None)
}

/// `Key.RandomValue()`.
pub fn random_value(w: &mut World) -> u32 {
    (4_294_967_294.0_f64 * utility::random_double(&mut w.rng)) as u32 + 1
}

/// `Key.RemoveKeys(Mobile m, keyValue)`.
pub fn remove_keys_mobile(w: &mut World, m: MobileId, key_value: u32) {
    if key_value == 0 {
        return;
    }
    if let Some(b) = muo_server::mobile::part1::props_b::backpack(w, m) {
        remove_keys(w, b, key_value);
    }
    if let Some(b) = muo_server::mobile::part1::props_b::bank_box(w, m) {
        remove_keys(w, b, key_value);
    }
}

/// `Key.RemoveKeys(Container cont, keyValue)`.
pub fn remove_keys(w: &mut World, cont: ItemId, key_value: u32) {
    if key_value == 0 {
        return;
    }
    for item in muo_server::item::enumerable::find_items(w, cont, true, |_, _| true) {
        let Some(ty) = w.item(item).map(|i| i.ty) else {
            continue;
        };
        if ty.is(&KEY) {
            if key_value_of(w, item) == key_value {
                (ty.delete)(w, item);
            }
        } else if ty.is(&KEY_RING) {
            ring_remove_key(w, item, key_value);
        }
    }
}

fn key_value_of(w: &World, id: ItemId) -> u32 {
    key_value(w, id)
}

/// `Key.ContainsKey(Container cont, keyValue)`.
pub fn contains_key(w: &World, cont: ItemId, key_value: u32) -> bool {
    let all = muo_server::item::enumerable::find_items(w, cont, true, |_, _| true);
    all.into_iter().any(|item| {
        let Some(ty) = w.item(item).map(|i| i.ty) else {
            return false;
        };
        if ty.is(&KEY) {
            key_value_of(w, item) == key_value
        } else if ty.is(&KEY_RING) {
            ring_contains_key(w, item, key_value)
        } else {
            false
        }
    })
}

fn key_desc(w: &World, id: ItemId) -> Option<String> {
    let d = key_data(w, id)?;
    // _keyValue == 0 ? "(blank)" : _description.DefaultIfNullOrEmpty(null)?.Trim()
    if d.key_value == 0 {
        return Some("(blank)".into());
    }
    d.description
        .as_deref()
        .filter(|s| !s.is_empty())
        .map(|s| s.trim().to_string())
}

fn unreachable(w: &mut World, from: MobileId) {
    deps::send_localized_message(w, from, 501661); // That key is unreachable.
}

fn key_in_backpack(w: &World, key: ItemId, from: MobileId) -> bool {
    // `IsChildOf(from.Backpack)`: a null backpack makes `IsChildOf(null)` true only for a parentless item.
    let pack = w.mobile(from).and_then(|m| m.backpack);
    match pack {
        Some(p) => ops::is_child_of(w, key, Some(EntityId::Item(p))),
        None => ops::is_child_of(w, key, None),
    }
}

fn key_double_click(w: &mut World, id: ItemId, from: MobileId) {
    if !key_in_backpack(w, id, from) {
        unreachable(w, from);
        return;
    }
    let (kv, range) = key_data(w, id).map_or((0, 3), |d| (d.key_value, d.max_range));
    let number;
    let mut t;
    if kv != 0 {
        number = 501662; // What shall I use this key on?
        t = target::Target::new(w, range, false, TargetFlags::NONE);
        t.check_los = false;
        t.handlers.on_target = Some(Rc::new(move |w: &mut World, from, targeted| {
            unlock_on_target(w, id, from, targeted)
        }));
    } else {
        number = 501663; // This key is a key blank. Which key would you like to make a copy of?
        t = target::Target::new(w, 3, false, TargetFlags::NONE);
        t.handlers.on_target = Some(Rc::new(move |w: &mut World, from, targeted| {
            copy_on_target(w, id, from, targeted)
        }));
    }
    deps::send_localized_message(w, from, number);
    let tid = target::add_target(w, t);
    target::set_mobile_target(w, from, Some(tid));
}

fn unlock_on_target(w: &mut World, key: ItemId, from: MobileId, targeted: Targeted) {
    if ops::is_deleted(w, key) || !key_in_backpack(w, key, from) {
        unreachable(w, from);
        return;
    }
    let number;
    match targeted {
        Targeted::Entity(EntityId::Item(it)) if it == key => {
            number = 501665; // Enter a description for this key.
            let p = muo_server::prompt::SimplePrompt::new(
                Some(Rc::new(move |w: &mut World, from, text: &str| {
                    if ops::is_deleted(w, key) || !key_in_backpack(w, key, from) {
                        unreachable(w, from);
                        return;
                    }
                    set_key_description(w, key, Some(utility::fix_html(text)));
                })),
                false,
            );
            let pid = muo_server::prompt::add_prompt(w, p.into_prompt());
            muo_server::mobile::part1::props_b::set_prompt(w, from, Some(pid));
        }
        Targeted::Entity(EntityId::Item(it)) if is_lockable(w, it) => {
            number = if use_on(w, key, from, it) { -1 } else { 501668 }; // This key doesn't seem to unlock that.
        }
        _ => number = 501666, // You can't unlock that!
    }
    if number != -1 {
        deps::send_localized_message(w, from, number);
    }
}

fn copy_on_target(w: &mut World, key: ItemId, from: MobileId, targeted: Targeted) {
    if ops::is_deleted(w, key) || !key_in_backpack(w, key, from) {
        unreachable(w, from);
        return;
    }
    let number;
    match targeted {
        Targeted::Entity(EntityId::Item(k)) if is_key(w, k) => {
            let kd = key_data(w, k).cloned();
            let Some(kd) = kd else { return };
            if kd.key_value == 0 {
                number = 501675; // This key is also blank.
            } else if muo_server::mobile::part2::stats::check_target_skill(
                w,
                from,
                SkillName::Tinkering,
                targeted,
                0.0,
                75.0,
            ) {
                number = 501676; // You make a copy of the key.
                if let Some(d) = key_data_mut(w, key) {
                    d.description = kd.description.clone();
                    d.key_value = kd.key_value;
                    d.link = kd.link;
                    d.max_range = kd.max_range;
                }
                ops::invalidate_properties(w, key);
            } else if utility::random_double(&mut w.rng) < 0.1 {
                // 10% chance to destroy the key
                deps::send_localized_message(w, from, 501677); // You fail to make a copy of the key.
                number = 501678; // The key was destroyed in the attempt.
                if let Some(ty) = w.item(key).map(|i| i.ty) {
                    (ty.delete)(w, key);
                }
            } else {
                number = 501677;
            }
        }
        _ => number = 501688, // Not a key.
    }
    deps::send_localized_message(w, from, number);
}

fn is_lockable(w: &World, it: ItemId) -> bool {
    w.item(it)
        .is_some_and(|i| i.ty.is(&cont::LOCKABLE_CONTAINER))
        || super::common::lockable_key_value(w, it).is_some()
}

fn lockable_key_value(w: &World, it: ItemId) -> u32 {
    if w.item(it)
        .is_some_and(|i| i.ty.is(&cont::LOCKABLE_CONTAINER))
    {
        return cont::ext(w, it).map_or(0, |d| d.key_value);
    }
    super::common::lockable_key_value(w, it).unwrap_or(0)
}

/// `Key.UseOn(from, ILockable o)`.
pub fn use_on(w: &mut World, key: ItemId, from: MobileId, o: ItemId) -> bool {
    if lockable_key_value(w, o) != key_value(w, key) {
        return false;
    }
    let is_cont = w
        .item(o)
        .is_some_and(|i| i.ty.is(&cont::LOCKABLE_CONTAINER));
    let now_locked = if is_cont {
        let l = !cont::locked(w, o);
        cont::set_locked(w, o, l);
        l
    } else {
        match super::common::lockable_toggle(w, o) {
            Some(l) => l,
            None => return false, // BaseDoor.UseLocks() is false (or the lockable system is absent)
        }
    };
    if now_locked {
        muo_server::item::messages::send_localized_message_to(w, o, from, 1048000, ""); // You lock it.
    } else {
        muo_server::item::messages::send_localized_message_to(w, o, from, 1048001, ""); // You unlock it.
    }
    if is_cont {
        let d = cont::ext(w, o).cloned().unwrap_or_default();
        if d.lock_level == MAGIC_LOCK
            && let Some(e) = cont::ext_mut(w, o)
        {
            e.lock_level = d.required_skill - 10;
        }
        if d.trap_type != cont::TrapType::None && d.trap_on_lockpick {
            if now_locked {
                muo_server::item::messages::send_localized_message_to(w, o, from, 501673, ""); // You re-enable the trap.
            } else {
                // You disable the trap temporarily.  Lock it again to re-enable it.
                muo_server::item::messages::send_localized_message_to(w, o, from, 501672, "");
            }
        }
    }
    true
}

/// `Server.Items.Key`.
pub static KEY: ItemType = ItemType {
    full_name: "Server.Items.Key",
    parent: Some(&muo_server::item::ITEM_BASE),
    construct: Some(|w| new_key(w, 0)),
    default_weight: |w, id| {
        crate::template_keys::item_float(
            w,
            id,
            "defaultWeight",
            muo_server::item::ITEM_BASE.default_weight,
        )
    },
    on_double_click: key_double_click,
    get_properties: |w, id, list| {
        base::get_properties_base(w, id, list);
        if let Some(desc) = key_desc(w, id) {
            list.add_text(&desc);
        }
    },
    on_single_click: |w, id, from| {
        base::on_single_click_base(w, id, from);
        if let Some(desc) = key_desc(w, id)
            && let Some(ns) = deps::net_state_of(w, from)
        {
            let item_id = w.item(id).map_or(0, |i| i.item_id);
            muo_server::packets::message::send_message(
                w,
                ns,
                id.0,
                item_id,
                MessageType::REGULAR,
                0x3B2,
                3,
                false,
                Some("ENU"),
                "",
                &desc,
            );
        }
    },
    serialize: |w, id, out| {
        base::serialize_base(w, id, out);
        let d = key_data(w, id).cloned().unwrap_or(KeyData {
            max_range: 0,
            link: None,
            description: None,
            key_value: 0,
        });
        out.write_i32(0);
        out.write_i32(d.max_range);
        out.write_item(w, d.link);
        out.write_opt_string(d.description.as_deref());
        out.write_u32(d.key_value);
    },
    deserialize: |w, id, r| {
        base::deserialize_base(w, id, r)?;
        let _version = r.read_i32()?;
        let max_range = r.read_i32()?;
        let link = r.read_item(w)?;
        let description = r.read_string()?;
        let key_value = r.read_u32()?;
        if let Some(i) = w.item_mut(id) {
            i.ext = Ext::new(KeyData {
                max_range,
                link,
                description,
                key_value,
            });
        }
        Ok(())
    },
    ..muo_server::item::ITEM_BASE
};

// ---- KeyRing ----

/// What `KeyRing` adds.
#[derive(Clone, Debug, Default)]
pub struct KeyRingData {
    pub keys: Vec<ItemId>,
}

const MAX_KEYS: usize = 20;
/// `ILockpickable.MagicLock`.
const MAGIC_LOCK: i32 = -255;

fn keys(w: &World, id: ItemId) -> Vec<ItemId> {
    w.item(id)
        .and_then(|i| i.ext.get::<KeyRingData>())
        .map(|d| d.keys.clone())
        .unwrap_or_default()
}

fn keys_mut(w: &mut World, id: ItemId) -> Option<&mut Vec<ItemId>> {
    w.item_mut(id)?
        .ext
        .get_mut::<KeyRingData>()
        .map(|d| &mut d.keys)
}

/// `new KeyRing()`.
pub fn new_key_ring(w: &mut World) -> ItemId {
    let id = ops::new_item_with_id(w, &KEY_RING, 0x1011);
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(KeyRingData::default());
    }
    id
}

fn update_item_id(w: &mut World, id: ItemId) {
    let n = keys(w, id).len();
    let graphic = match n {
        0 => 0x1011,
        1..=2 => 0x1769,
        3..=4 => 0x176A,
        _ => 0x176B,
    };
    (KEY_RING.set_item_id)(w, id, graphic);
}

/// `KeyRing.Add(key)`.
pub fn ring_add(w: &mut World, id: ItemId, key: ItemId) {
    ops::internalize(w, key);
    if let Some(k) = keys_mut(w, id) {
        k.push(key);
    }
    ops::invalidate_properties(w, id);
    update_item_id(w, id);
}

/// `KeyRing.RemoveKey(keyValue)`.
pub fn ring_remove_key(w: &mut World, id: ItemId, key_value_: u32) {
    for key in keys(w, id).into_iter().rev() {
        if key_value(w, key) == key_value_ {
            if let Some(ty) = w.item(key).map(|i| i.ty) {
                (ty.delete)(w, key);
            }
            if let Some(k) = keys_mut(w, id) {
                k.retain(|x| *x != key);
            }
        }
    }
    update_item_id(w, id);
}

/// `KeyRing.ContainsKey(keyValue)`.
pub fn ring_contains_key(w: &World, id: ItemId, key_value_: u32) -> bool {
    keys(w, id)
        .into_iter()
        .any(|k| key_value(w, k) == key_value_)
}

/// `KeyRing.Open(from)`.
pub fn ring_open(w: &mut World, id: ItemId, from: MobileId) {
    let Some(EntityId::Item(cont)) = w.item(id).and_then(|i| i.parent) else {
        return;
    };
    if w.item(cont).is_none_or(|i| i.cont.is_none()) {
        return;
    }
    let list = keys(w, id);
    for (i, key) in list
        .into_iter()
        .enumerate()
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
    {
        if !ops::is_deleted(w, key) && !container::try_drop_item(w, cont, Some(from), key, true) {
            break;
        }
        if let Some(k) = keys_mut(w, id)
            && i < k.len()
        {
            k.remove(i);
        }
    }
    update_item_id(w, id);
}

fn ring_in_pack(w: &World, id: ItemId, from: MobileId) -> bool {
    key_in_backpack(w, id, from)
}

fn ring_target(w: &mut World, ring: ItemId, from: MobileId, targeted: Targeted) {
    if ops::is_deleted(w, ring) || !ring_in_pack(w, ring, from) {
        deps::send_localized_message(w, from, 1060640); // The item must be in your backpack to use it.
        return;
    }
    match targeted {
        Targeted::Entity(EntityId::Item(it)) if it == ring => {
            ring_open(w, ring, from);
            deps::send_localized_message(w, from, 501685); // You open the keyring.
        }
        Targeted::Entity(EntityId::Item(it)) if is_lockable(w, it) => {
            for key in keys(w, ring) {
                if use_on(w, key, from, it) {
                    return;
                }
            }
            deps::send_localized_message(w, from, 1008140); // You do not have a key for that.
        }
        _ => deps::send_localized_message(w, from, 501666), // You can't unlock that!
    }
}

/// `Server.Items.KeyRing`.
pub static KEY_RING: ItemType = ItemType {
    full_name: "Server.Items.KeyRing",
    parent: Some(&muo_server::item::ITEM_BASE),
    construct: Some(new_key_ring),
    default_weight: |w, id| {
        crate::template_keys::item_float(
            w,
            id,
            "defaultWeight",
            muo_server::item::ITEM_BASE.default_weight,
        )
    },
    on_drag_drop: |w, id, from, dropped| {
        if !ring_in_pack(w, id, from) {
            deps::send_localized_message(w, from, 1060640); // The item must be in your backpack to use it.
            return false;
        }
        if !is_key(w, dropped) || key_value(w, dropped) == 0 {
            deps::send_localized_message(w, from, 501689); // Only non-blank keys can be put on a keyring.
            return false;
        }
        if keys(w, id).len() >= MAX_KEYS {
            deps::send_localized_message(w, from, 1008138); // This keyring is full.
            return false;
        }
        ring_add(w, id, dropped);
        deps::send_localized_message(w, from, 501691); // You put the key on the keyring.
        true
    },
    on_double_click: |w, id, from| {
        if !ring_in_pack(w, id, from) {
            deps::send_localized_message(w, from, 1060640);
            return;
        }
        deps::send_localized_message(w, from, 501680); // What do you want to unlock?
        let mut t = target::Target::new(w, -1, false, TargetFlags::NONE);
        t.handlers.on_target = Some(Rc::new(move |w: &mut World, from, targeted| {
            ring_target(w, id, from, targeted)
        }));
        let tid = target::add_target(w, t);
        target::set_mobile_target(w, from, Some(tid));
    },
    on_delete: |w, id| {
        base::on_delete_base(w, id);
        for key in keys(w, id) {
            if let Some(ty) = w.item(key).map(|i| i.ty) {
                (ty.delete)(w, key);
            }
        }
        if let Some(k) = keys_mut(w, id) {
            k.clear();
        }
    },
    serialize: |w, id, out| {
        base::serialize_base(w, id, out);
        out.write_encoded_int(1);
        let k = keys(w, id);
        out.write_encoded_int(k.len() as i32);
        for key in k {
            out.write_item(w, Some(key));
        }
    },
    deserialize: |w, id, r| {
        base::deserialize_base(w, id, r)?;
        let version = r.read_encoded_int()?;
        let list = if version < 1 {
            // Deserialize(reader, version): ReadEntityList<Key>() with a plain int count.
            r.read_item_list(w)?
        } else {
            // The generated reader keeps a null entry, so the count survives.
            r.read_item_list_kept(true)?
        };
        if let Some(i) = w.item_mut(id) {
            i.ext = Ext::new(KeyRingData { keys: list });
        }
        Ok(())
    },
    ..muo_server::item::ITEM_BASE
};

pub fn register(types: &mut TypeRegistry) {
    for ty in [&GOLD, &BANK_CHECK, &KEY, &KEY_RING] {
        types.register_item(ty, &[]);
    }
}

#[allow(dead_code)]
fn _r(_: Result<()>) {}
