//! Secure trades, plus the trade members of `NetState` (`Trades`, `AddTrade`, `FindTrade`,
//! `FindTradeContainer`, `RemoveTrade`, `CancelAllTrades`, `ValidateAllTrades`).
//!
//! The trades live in `World::trades` by [`TradeId`]; `NetStateData::trades` lists the ids a connection takes
//! part in. Every function re-reads the trade from the table, so an item's virtual (`OnItemRemoved` clearing the
//! checks while `Update` hands items over) sees the live state. The two `SecureTradeInfo.Dispose` calls run
//! from zero-delay timers after `Close`.

use muo_primitives::TimeSpan;

use crate::iaccount::{
    account_deposit_gold, account_deposit_plat, account_totals, account_withdraw_gold,
    account_withdraw_plat, gold_enabled,
};
use crate::ids::{ItemId, MobileId};
use crate::net_state::{NetStateId, TradeId};
use crate::packets::secure_trade as pk;
use crate::timer;
use crate::world::World;

/// `TradeFlag` of Network/Packets (`UpdateGold`, `UpdateLedger` for the new trade window).
pub use crate::packets::secure_trade::TradeFlag;

/// `SecureTradeInfo`.
#[derive(Clone, Debug)]
pub struct SecureTradeInfo {
    pub mobile: MobileId,
    pub container: Option<ItemId>,
    pub virtual_check: Option<ItemId>,
    pub accepted: bool,
    pub is_disposed: bool,
}

impl SecureTradeInfo {
    /// `SecureTradeInfo(owner, m, c)`.
    fn new(w: &mut World, m: MobileId, c: ItemId) -> Self {
        crate::mobile::part2::items::add_item(w, m, c);
        let check = crate::item::virtual_check::new_virtual_check(w, 0, 0);
        if let Some(ty) = w.item(c).map(|i| i.ty) {
            (ty.drop_item)(w, c, check);
        }
        Self {
            mobile: m,
            container: Some(c),
            virtual_check: Some(check),
            accepted: false,
            is_disposed: false,
        }
    }

    /// `Gold`: the check's.
    pub fn gold(&self, w: &World) -> i32 {
        self.virtual_check
            .map_or(0, |c| crate::item::virtual_check::gold(w, c))
    }

    /// `Plat`: the check's.
    pub fn plat(&self, w: &World) -> i32 {
        self.virtual_check
            .map_or(0, |c| crate::item::virtual_check::plat(w, c))
    }
}

/// `SecureTrade`.
pub struct SecureTrade {
    pub from: SecureTradeInfo,
    pub to: SecureTradeInfo,
    pub valid: bool,
}

pub fn trade(w: &World, id: TradeId) -> Option<&SecureTrade> {
    w.trades.get(&id)
}

fn ns_of(w: &World, m: MobileId) -> Option<NetStateId> {
    w.mobile(m).and_then(|m| m.net_state)
}

fn new_secure_trading(w: &World, m: MobileId) -> bool {
    crate::item::deps::new_secure_trading(w, m)
}

/// `new SecureTrade(from, to)`: the table entry and the packets of its constructor.
pub fn new_trade(w: &mut World, from: MobileId, to: MobileId) -> TradeId {
    let id = TradeId(w.next_trade_id);
    w.next_trade_id += 1;

    let fc = crate::item::secure_trade_container::new_secure_trade_container(w, Some(id));
    let from_info = SecureTradeInfo::new(w, from, fc);
    let tc = crate::item::secure_trade_container::new_secure_trade_container(w, Some(id));
    let to_info = SecureTradeInfo::new(w, to, tc);
    w.trades.insert(
        id,
        SecureTrade {
            from: from_info,
            to: to_info,
            valid: true,
        },
    );

    let from704565 = new_secure_trading(w, from);
    let to704565 = new_secure_trading(w, to);

    if let Some(ns) = ns_of(w, from) {
        let name = w.mobile(to).and_then(|m| (m.ty.name)(w, to));
        crate::packets::mobile::send_mobile_status(w, ns, from, to);
        pk::send_update_secure_trade(w, ns, fc, false, false);
        pk::send_secure_trade_equip(w, ns, tc, to);
        pk::send_update_secure_trade(w, ns, fc, false, false);
        pk::send_secure_trade_equip(w, ns, fc, from);
        pk::send_display_secure_trade(w, ns, to, fc, tc, name.as_deref());
        pk::send_update_secure_trade(w, ns, fc, false, false);
        if let Some((gold, plat)) = account_totals(w, from)
            && from704565
        {
            pk::send_update_secure_trade_flag(w, ns, fc, TradeFlag::UpdateLedger, gold, plat);
        }
    }

    if let Some(ns) = ns_of(w, to) {
        let name = w.mobile(from).and_then(|m| (m.ty.name)(w, from));
        crate::packets::mobile::send_mobile_status(w, ns, to, from);
        pk::send_update_secure_trade(w, ns, tc, false, false);
        pk::send_secure_trade_equip(w, ns, fc, from);
        pk::send_update_secure_trade(w, ns, tc, false, false);
        pk::send_secure_trade_equip(w, ns, tc, to);
        pk::send_display_secure_trade(w, ns, from, tc, fc, name.as_deref());
        pk::send_update_secure_trade(w, ns, tc, false, false);
        if let Some((gold, plat)) = account_totals(w, to)
            && to704565
        {
            pk::send_update_secure_trade_flag(w, ns, tc, TradeFlag::UpdateLedger, gold, plat);
        }
    }
    id
}

fn container_len(w: &World, c: ItemId) -> usize {
    w.item(c).map_or(0, |c| c.items.len())
}

/// The loop of `Cancel`/`Update` for one side: its items to `new_owner`'s backpack.
fn pass_items(
    w: &mut World,
    side: &SecureTradeInfo,
    other: MobileId,
    new_owner: MobileId,
    accepted: bool,
) {
    let Some(container) = side.container else {
        return;
    };
    let mut i = container_len(w, container);
    while i > 0 {
        i -= 1;
        let Some(item) = w.item(container).and_then(|c| c.items.get(i).copied()) else {
            continue; // `if (i < list.Count)`
        };
        if Some(item) == side.virtual_check {
            continue;
        }
        on_secure_trade(w, item, side.mobile, other, new_owner, accepted);
        if w.item(item).is_some_and(|it| !it.deleted()) {
            crate::mobile::part2::items::add_to_backpack(w, new_owner, item);
        }
    }
}

/// `Cancel`: every item goes back to its own side's backpack.
pub fn cancel(w: &mut World, id: TradeId) {
    let Some(t) = w.trades.get(&id) else { return };
    if !t.valid {
        return;
    }
    let (from, to) = (t.from.clone(), t.to.clone());
    pass_items(w, &from, to.mobile, from.mobile, false);
    pass_items(w, &to, from.mobile, to.mobile, false);
    close(w, id);
}

/// `Close`.
pub fn close(w: &mut World, id: TradeId) {
    let Some(t) = w.trades.get(&id) else { return };
    if !t.valid {
        return;
    }
    let (from, to) = (t.from.clone(), t.to.clone());
    for side in [&from, &to] {
        if let (Some(ns), Some(c)) = (ns_of(w, side.mobile), side.container) {
            pk::send_close_secure_trade(w, ns, c);
        }
    }
    if let Some(t) = w.trades.get_mut(&id) {
        t.valid = false;
    }
    for side in [&from, &to] {
        if let Some(ns) = ns_of(w, side.mobile) {
            remove_trade(w, ns, id);
        }
    }
    for is_from in [true, false] {
        timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 1, move |w| {
            dispose(w, id, is_from)
        });
    }
}

/// `SecureTradeInfo.Dispose`.
fn dispose(w: &mut World, id: TradeId, is_from: bool) {
    let Some(t) = w.trades.get_mut(&id) else {
        return;
    };
    let info = if is_from { &mut t.from } else { &mut t.to };
    let (check, container) = (info.virtual_check.take(), info.container.take());
    info.is_disposed = true;
    for item in [check, container].into_iter().flatten() {
        if let Some(ty) = w.item(item).map(|i| i.ty) {
            (ty.delete)(w, item);
        }
    }
    // both sides gone: drop the trade
    if w.trades
        .get(&id)
        .is_some_and(|t| t.from.is_disposed && t.to.is_disposed)
    {
        w.trades.shift_remove(&id);
    }
}

pub fn update_from_currency(w: &mut World, id: TradeId) {
    if let Some(t) = w.trades.get(&id) {
        let (l, r) = (t.from.clone(), t.to.clone());
        update_currency(w, &l, &r);
    }
}

pub fn update_to_currency(w: &mut World, id: TradeId) {
    if let Some(t) = w.trades.get(&id) {
        let (l, r) = (t.to.clone(), t.from.clone());
        update_currency(w, &l, &r);
    }
}

/// `UpdateCurrency(left, right)`.
fn update_currency(w: &mut World, left: &SecureTradeInfo, right: &SecureTradeInfo) {
    if new_secure_trading(w, left.mobile)
        && let (Some(ns), Some(c)) = (ns_of(w, left.mobile), left.container)
    {
        let (gold, plat) = account_totals(w, left.mobile).unwrap_or_default();
        pk::send_update_secure_trade_flag(w, ns, c, TradeFlag::UpdateLedger, gold, plat);
    }
    if new_secure_trading(w, right.mobile)
        && let (Some(ns), Some(c)) = (ns_of(w, right.mobile), right.container)
    {
        let (gold, plat) = (left.gold(w), left.plat(w));
        pk::send_update_secure_trade_flag(w, ns, c, TradeFlag::UpdateGold, gold, plat);
    }
}

/// `From.Mobile.NetState.SendUpdateSecureTrade(From.Container, From.Accepted, To.Accepted)` and the mirror.
fn send_accept_state(w: &mut World, id: TradeId) {
    let Some(t) = w.trades.get(&id) else { return };
    let (f, o) = (t.from.clone(), t.to.clone());
    if let (Some(ns), Some(c)) = (ns_of(w, f.mobile), f.container) {
        pk::send_update_secure_trade(w, ns, c, f.accepted, o.accepted);
    }
    if let (Some(ns), Some(c)) = (ns_of(w, o.mobile), o.container) {
        pk::send_update_secure_trade(w, ns, c, o.accepted, f.accepted);
    }
}

/// `ClearChecks`: each side's `Accepted = false` unless disposed, then `Trade.Update()`.
pub fn clear_accepted_and_update(w: &mut World, id: TradeId) {
    if let Some(t) = w.trades.get_mut(&id) {
        if !t.from.is_disposed {
            t.from.accepted = false;
        }
        if !t.to.is_disposed {
            t.to.accepted = false;
        }
    }
    update(w, id);
}

/// `SecureTradeInfo.Accepted = value` for the side of `m` (the trade packet's accept).
pub fn set_accepted(w: &mut World, id: TradeId, m: MobileId, value: bool) {
    if let Some(t) = w.trades.get_mut(&id) {
        if t.from.mobile == m {
            t.from.accepted = value;
        } else if t.to.mobile == m {
            t.to.accepted = value;
        }
    }
}

/// `Update`.
pub fn update(w: &mut World, id: TradeId) {
    let Some(t) = w.trades.get(&id) else { return };
    if !t.valid {
        return;
    }
    let (from, to) = (t.from.clone(), t.to.clone());

    if !from.is_disposed && from.accepted && !to.is_disposed && to.accepted {
        let mut allowed = true;
        for (side, other) in [(&from, &to), (&to, &from)] {
            let Some(container) = side.container else {
                continue;
            };
            let mut i = container_len(w, container);
            while allowed && i > 0 {
                i -= 1;
                let Some(item) = w.item(container).and_then(|c| c.items.get(i).copied()) else {
                    continue;
                };
                if Some(item) == side.virtual_check {
                    continue;
                }
                if !allow_secure_trade(w, item, side.mobile, other.mobile, other.mobile, true) {
                    allowed = false;
                }
            }
        }

        if gold_enabled(w) {
            for side in [&from, &to] {
                if let Some((total_gold, total_plat)) = account_totals(w, side.mobile)
                    && (total_plat < side.plat(w) || total_gold < side.gold(w))
                {
                    allowed = false;
                    crate::commands::send_message(
                        w,
                        side.mobile,
                        "You do not have enough currency to complete this trade.",
                    );
                }
            }
        }

        if !allowed {
            if let Some(t) = w.trades.get_mut(&id) {
                t.from.accepted = false;
                t.to.accepted = false;
            }
            send_accept_state(w, id);
            return;
        }

        if gold_enabled(w)
            && account_totals(w, from.mobile).is_some()
            && account_totals(w, to.mobile).is_some()
        {
            handle_account_gold_trade(w, &from, &to);
        }

        pass_items(w, &from, to.mobile, to.mobile, true);
        pass_items(w, &to, from.mobile, from.mobile, true);
        close(w, id);
    } else if !from.is_disposed && !to.is_disposed {
        send_accept_state(w, id);
    }
}

fn handle_account_gold_trade(w: &mut World, from: &SecureTradeInfo, to: &SecureTradeInfo) {
    let (mut from_plat_send, mut from_gold_send, mut from_plat_recv, mut from_gold_recv) =
        (0, 0, 0, 0);
    let (mut to_plat_send, mut to_gold_send, mut to_plat_recv, mut to_gold_recv) = (0, 0, 0, 0);
    let (f, t) = (from.mobile, to.mobile);
    let (fp, fg, tp, tg) = (from.plat(w), from.gold(w), to.plat(w), to.gold(w));

    // `&` (not `&&`): the withdraw runs whenever it is evaluated.
    if (fp > 0) & account_withdraw_plat(w, f, fp) {
        from_plat_send = fp;
        if account_deposit_plat(w, t, fp) {
            to_plat_recv = from_plat_send;
        }
    }
    if (fg > 0) & account_withdraw_gold(w, f, fg) {
        from_gold_send = fg;
        if account_deposit_gold(w, t, fg) {
            to_gold_recv = from_gold_send;
        }
    }
    if (tp > 0) & account_withdraw_plat(w, t, tp) {
        to_plat_send = tp;
        if account_deposit_plat(w, f, tp) {
            from_plat_recv = to_plat_send;
        }
    }
    if (tg > 0) & account_withdraw_gold(w, t, tg) {
        to_gold_send = tg;
        if account_deposit_gold(w, f, tg) {
            from_gold_recv = to_gold_send;
        }
    }

    account_gold_trade_messages(
        w,
        f,
        t,
        from_plat_send,
        from_gold_send,
        from_plat_recv,
        from_gold_recv,
    );
    account_gold_trade_messages(
        w,
        t,
        f,
        to_plat_send,
        to_gold_send,
        to_plat_recv,
        to_gold_recv,
    );
}

// ---- NetState's trade members ----

/// `NetState.RemoveTrade(trade)`.
pub fn remove_trade(w: &mut World, ns: NetStateId, id: TradeId) {
    if let Some(d) = w.net_state_mut(ns)
        && let Some(l) = d.trades.as_mut()
    {
        l.retain(|&t| t != id);
        if l.is_empty() {
            d.trades = None;
        }
    }
}

/// `NetState.FindTrade(m)`.
pub fn find_trade(w: &World, ns: NetStateId, m: MobileId) -> Option<TradeId> {
    w.net_state(ns)?
        .trades
        .as_ref()?
        .iter()
        .copied()
        .find(|id| {
            w.trades
                .get(id)
                .is_some_and(|t| t.from.mobile == m || t.to.mobile == m)
        })
}

/// `NetState.FindTradeContainer(m)`.
pub fn find_trade_container(w: &World, ns: NetStateId, m: MobileId) -> Option<ItemId> {
    let me = w.net_mobile(ns)?;
    for id in w.net_state(ns)?.trades.as_ref()? {
        let t = w.trades.get(id)?;
        if t.from.mobile == me && t.to.mobile == m {
            return t.from.container;
        }
        if t.from.mobile == m && t.to.mobile == me {
            return t.to.container;
        }
    }
    None
}

/// `NetState.AddTrade(state)`: `ns` is `this`, `other` the `state` argument. The container of this side.
pub fn add_trade(w: &mut World, ns: NetStateId, other: NetStateId) -> Option<ItemId> {
    let (me, them) = (w.net_mobile(ns)?, w.net_mobile(other)?);
    let id = new_trade(w, me, them);
    for s in [ns, other] {
        if let Some(d) = w.net_state_mut(s) {
            d.trades.get_or_insert_with(Vec::new).push(id);
        }
    }
    w.trades.get(&id)?.from.container
}

/// `NetState.CancelAllTrades()`.
pub fn cancel_all_trades(w: &mut World, ns: NetStateId) {
    let Some(list) = w.net_state(ns).and_then(|d| d.trades.clone()) else {
        return;
    };
    for id in list.into_iter().rev() {
        cancel(w, id);
    }
}

/// `NetState.ValidateAllTrades()`.
pub fn validate_all_trades(w: &mut World, ns: NetStateId) {
    let Some(list) = w.net_state(ns).and_then(|d| d.trades.clone()) else {
        return;
    };
    for id in list.into_iter().rev() {
        let Some(t) = w.trades.get(&id) else { continue };
        let (f, o) = (t.from.mobile, t.to.mobile);
        let (Some(fm), Some(om)) = (w.mobile(f), w.mobile(o)) else {
            cancel(w, id);
            continue;
        };
        let bad = fm.deleted
            || om.deleted
            || !(fm.ty.alive)(w, f)
            || !(om.ty.alive)(w, o)
            || !(fm.ty.in_range_ipoint2d)(
                w,
                f,
                muo_primitives::Point2D::new(om.location.x, om.location.y),
                2,
            )
            || fm.map != om.map;
        if bad {
            cancel(w, id);
        }
    }
}

/// What the item code reads of `SecureTradeContainer.Trade`.
pub fn snapshot(w: &World, id: TradeId) -> Option<crate::item::deps::TradeSnapshot> {
    let t = w.trades.get(&id)?;
    Some(crate::item::deps::TradeSnapshot {
        from: (!t.from.is_disposed).then_some(t.from.mobile),
        to: (!t.to.is_disposed).then_some(t.to.mobile),
        from_container: t.from.container,
        to_container: t.to.container,
        from_check: t.from.virtual_check,
        to_check: t.to.virtual_check,
        valid: t.valid,
    })
}

/// `{n:#,0}`.
fn commas(n: i32) -> String {
    let digits = n.unsigned_abs().to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    if n < 0 { format!("-{out}") } else { out }
}

/// The static `HandleAccountGoldTrade(left, right, ...)`: the messages.
#[allow(clippy::too_many_arguments)]
fn account_gold_trade_messages(
    w: &mut World,
    left: MobileId,
    right: MobileId,
    plat_send: i32,
    gold_send: i32,
    plat_recv: i32,
    gold_recv: i32,
) {
    let name = crate::mobile::part1::props_b::raw_name(w, right).unwrap_or_default();
    let say = |w: &mut World, s: String| crate::commands::send_message(w, left, &s);

    if plat_send > 0 || gold_send > 0 {
        if plat_send > 0 && gold_send > 0 {
            say(
                w,
                format!(
                    "You traded {} platinum and {} gold to {name}.",
                    commas(plat_send),
                    commas(gold_send)
                ),
            );
        } else if plat_send > 0 {
            say(
                w,
                format!("You traded {} platinum to {name}.", commas(plat_send)),
            );
        } else if gold_send > 0 {
            say(
                w,
                format!("You traded {} gold to {name}.", commas(gold_send)),
            );
        }
    }

    if plat_recv > 0 || gold_recv > 0 {
        if plat_recv > 0 && gold_recv > 0 {
            say(
                w,
                format!(
                    "You received {} platinum and {} gold from {name}.",
                    commas(plat_recv),
                    commas(gold_recv)
                ),
            );
        } else if plat_recv > 0 {
            say(
                w,
                format!("You received {} platinum from {name}.", commas(plat_recv)),
            );
        } else if gold_recv > 0 {
            say(
                w,
                format!("You received {} gold from {name}.", commas(gold_recv)),
            );
        }
    }
}

/// `item.OnSecureTrade(from, to, newOwner, accepted)` through the item's type.
fn on_secure_trade(
    w: &mut World,
    item: ItemId,
    from: MobileId,
    to: MobileId,
    new_owner: MobileId,
    accepted: bool,
) {
    if let Some(ty) = w.item(item).map(|i| i.ty) {
        (ty.on_secure_trade)(w, item, from, to, new_owner, accepted);
    }
}

/// `item.AllowSecureTrade(from, to, newOwner, accepted)` through the item's type.
fn allow_secure_trade(
    w: &mut World,
    item: ItemId,
    from: MobileId,
    to: MobileId,
    new_owner: MobileId,
    accepted: bool,
) -> bool {
    match w.item(item).map(|i| i.ty) {
        Some(ty) => (ty.allow_secure_trade)(w, item, from, to, new_owner, accepted),
        None => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn comma_format() {
        assert_eq!(commas(0), "0");
        assert_eq!(commas(999), "999");
        assert_eq!(commas(1000), "1,000");
        assert_eq!(commas(-1234567), "-1,234,567");
    }

    // NetStateDisconnectTests.CancelAllTrades_CancelsEveryTrade.
    fn client(w: &mut World, ns: u32, name: &str) -> (NetStateId, MobileId) {
        use crate::mobile::MOBILE_BASE;
        let m = w.new_mobile(&MOBILE_BASE);
        crate::mobile::part2::stats::default_mobile_init(w, m);
        w.mobile_mut(m).unwrap().name = Some(name.to_string());
        let id = NetStateId(ns);
        w.net_states.insert(
            id,
            crate::net_state::NetStateData {
                mobile: Some(m),
                ..Default::default()
            },
        );
        w.mobile_mut(m).unwrap().net_state = Some(id);
        (id, m)
    }

    #[test]
    fn cancel_all_trades_cancels_every_trade() {
        let mut w = World::new();
        let (from, from_mobile) = client(&mut w, 1, "from");
        let (to, to_mobile) = client(&mut w, 2, "to");

        add_trade(&mut w, from, to);
        let trade = find_trade(&w, from, to_mobile).expect("trade");
        assert!(w.trades.get(&trade).unwrap().valid);

        cancel_all_trades(&mut w, from);

        assert!(w.trades.get(&trade).is_none_or(|t| !t.valid));
        assert!(w.net_state(from).unwrap().trades.is_none());
        assert!(w.net_state(to).unwrap().trades.is_none());
        assert!(find_trade(&w, from, to_mobile).is_none());
        assert!(find_trade(&w, to, from_mobile).is_none());
    }

    // A NameMod splits the reads: the trade receipt reads RawName, the
    // trade window and Mobile.ToString read Name.
    #[test]
    fn name_mod_raw_receipt_and_name_to_string() {
        use crate::net_state::NetOut;
        let mut w = World::new();
        let (from, from_mobile) = client(&mut w, 1, "from");
        let (_, to_mobile) = client(&mut w, 2, "to");
        crate::mobile::part1::props_b::set_name_mod(&mut w, to_mobile, Some("Disguise".into()));
        w.net_out.clear();
        account_gold_trade_messages(&mut w, from_mobile, to_mobile, 0, 7, 0, 0);
        // printable bytes only, so the packet's text reads the same ascii or UTF-16
        let sent = w.net_out.iter().any(|o| {
            matches!(o, NetOut::Send(n, b) if *n == from && {
                let t: String = b.iter().filter(|c| (0x20..0x7f).contains(*c)).map(|&c| c as char).collect();
                t.contains("gold to to.") && !t.contains("Disguise")
            })
        });
        assert!(sent, "receipt names the raw name");
        let s = crate::mobile::part1::combat::to_string(&w, to_mobile);
        assert!(s.ends_with("\"Disguise\""), "{s}");
    }

    // NetStateDisconnectTests.Send_AfterDisconnectHandedToSocket_DropsPacket / ..AfterImmediateDisconnect..
    #[test]
    fn send_after_disconnect_drops_packet() {
        let mut w = World::new();
        let (ns, _) = client(&mut w, 1, "x");
        w.disconnect(ns, "test");
        // `Disconnect()` only queues: sends before the handoff still go out (CannotSendPackets ignores Running)
        w.net_out.clear();
        w.send(ns, &[0x73, 0x00]);
        assert!(!w.cannot_send_packets(ns));
        assert_eq!(w.net_out.len(), 1);
        // the pump's mirror of the transport after the handoff
        w.net_state_mut(ns).unwrap().cannot_send = true;
        w.net_out.clear();
        w.send(ns, &[0x73, 0x00]);
        assert!(w.net_out.is_empty());
    }
}
