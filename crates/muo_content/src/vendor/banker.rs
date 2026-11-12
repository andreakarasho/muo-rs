//! The static members of `Banker`: balance, withdraw and deposit. The `Banker` class
//! itself (`crates/muo_mobiles/src/townfolk`) calls these; `BaseVendor` and `Gold` reach them from here.

use muo_server::iaccount;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::{TotalType, container, ops};
use muo_server::mobile::AccessLevel;
use muo_server::world::World;

use crate::items::gold::{self, BANK_CHECK, GOLD};

fn account_total_gold(w: &World, m: MobileId) -> Option<i64> {
    if !iaccount::gold_enabled(w) {
        return None;
    }
    let (gold, plat) = iaccount::account_totals(w, m)?;
    let threshold = w
        .resources
        .get::<iaccount::AccountGold>()
        .map_or(1_000_000_000, |a| a.currency_threshold);
    Some(i64::from(gold) + i64::from(plat) * i64::from(threshold))
}

fn bank_no_create(w: &mut World, m: MobileId) -> Option<ItemId> {
    muo_server::mobile::part2::items::find_bank_no_create(w, m)
}

fn gold_items(w: &World, bank: ItemId) -> Vec<ItemId> {
    muo_server::item::enumerable::find_items_by_type(w, bank, true, &GOLD, |_, _| true)
}

fn check_items(w: &World, bank: ItemId) -> Vec<ItemId> {
    muo_server::item::enumerable::find_items_by_type(w, bank, true, &BANK_CHECK, |_, _| true)
}

/// `GetBalance(m)`.
pub fn get_balance(w: &mut World, m: MobileId) -> i32 {
    let mut balance: i64 = 0;
    if let Some(t) = account_total_gold(w, m) {
        balance = t;
        if balance >= i64::from(i32::MAX) {
            return i32::MAX;
        }
    }
    if let Some(bank) = bank_no_create(w, m) {
        for g in gold_items(w, bank) {
            balance += i64::from(w.item(g).map_or(0, |i| i.amount));
        }
        if balance >= i64::from(i32::MAX) {
            return i32::MAX;
        }
        for c in check_items(w, bank) {
            balance += i64::from(gold::worth(w, c));
        }
    }
    balance.clamp(0, i64::from(i32::MAX)) as i32
}

/// `HasRequiredBalance`: the gold piles and checks that cover `required`, or `None`.
fn has_required_balance(
    w: &mut World,
    required: i32,
    m: MobileId,
) -> Option<(Vec<ItemId>, Vec<ItemId>)> {
    let bank = bank_no_create(w, m)?;
    let mut balance: i64 = 0;
    let mut golds = Vec::new();
    for g in gold_items(w, bank) {
        balance += i64::from(w.item(g).map_or(0, |i| i.amount));
        golds.push(g);
        if balance >= i64::from(required) {
            return Some((golds, Vec::new()));
        }
    }
    let mut checks = Vec::new();
    for c in check_items(w, bank) {
        balance += i64::from(gold::worth(w, c));
        checks.push(c);
        if balance >= i64::from(required) {
            return Some((golds, checks));
        }
    }
    None
}

fn delete(w: &mut World, id: ItemId) {
    if let Some(t) = w.item(id).map(|i| i.ty) {
        (t.delete)(w, id);
    }
}

/// `Withdraw(from, amount)`.
pub fn withdraw(w: &mut World, from: MobileId, mut amount: i32) -> bool {
    // If for whatever reason the TOL checks fail, we should still try old methods for withdrawing currency.
    if iaccount::gold_enabled(w) && iaccount::account_withdraw_gold(w, from, amount) {
        return true;
    }
    let Some((golds, checks)) = has_required_balance(w, amount, from) else {
        return false;
    };
    for g in golds {
        if amount <= 0 {
            break;
        }
        let a = w.item(g).map_or(0, |i| i.amount);
        if a <= amount {
            amount -= a;
            delete(w, g);
        } else {
            ops::set_amount(w, g, a - amount);
            amount = 0;
        }
    }
    for c in checks {
        if amount <= 0 {
            break;
        }
        let worth = gold::worth(w, c);
        if worth <= amount {
            amount -= worth;
            delete(w, c);
        } else {
            gold::set_worth(w, c, worth - amount);
            amount = 0;
        }
    }
    true
}

/// Slots needed for `remaining` as new piles and checks.
fn slots_needed(mut r: i32, use_checks: bool) -> i32 {
    let mut slots = 0;
    while r > 0 {
        if !use_checks || r < 5000 {
            r -= r.min(60000);
        } else if r <= 1_000_000 {
            r = 0;
        } else {
            r -= 1_000_000;
        }
        slots += 1;
    }
    slots
}

/// Tops off the existing piles and checks in the box; returns what is left of `amount`.
fn top_off(w: &mut World, items: &[ItemId], mut remaining: i32, use_checks: bool) -> i32 {
    for &item in items {
        if remaining <= 0 {
            break;
        }
        let Some(ty) = w.item(item).map(|i| i.ty) else {
            continue;
        };
        if ty.is(&GOLD) {
            let a = w.item(item).map_or(0, |i| i.amount);
            if a < 60000 {
                let add = (60000 - a).min(remaining);
                ops::set_amount(w, item, a + add);
                remaining -= add;
            }
        } else if use_checks && ty.is(&BANK_CHECK) {
            let c = gold::worth(w, item);
            if c < 1_000_000 {
                let add = (1_000_000 - c).min(remaining);
                gold::set_worth(w, item, c + add);
                remaining -= add;
            }
        }
    }
    remaining
}

/// Creates the new piles and checks for `remaining` in `box_`, at most `slots` of them.
fn fill_new(
    w: &mut World,
    box_: ItemId,
    mut remaining: i32,
    use_checks: bool,
    mut slots: i32,
) -> i32 {
    while remaining > 0 && slots > 0 {
        let item;
        if !use_checks || remaining < 5000 {
            let pile = remaining.min(60000);
            item = gold::new_gold(w, pile);
            remaining -= pile;
        } else if remaining <= 1_000_000 {
            item = gold::new_bank_check(w, remaining);
            remaining = 0;
        } else {
            item = gold::new_bank_check(w, 1_000_000);
            remaining -= 1_000_000;
        }
        container::drop_item_default(w, box_, item);
        slots -= 1;
    }
    remaining
}

fn bank_items_of_gold_and_checks(w: &World, box_: ItemId) -> Vec<ItemId> {
    muo_server::item::enumerable::find_items_by_types(w, box_, false, &[&GOLD, &BANK_CHECK])
}

/// `Deposit(from, amount, useChecks)`.
pub fn deposit(w: &mut World, from: MobileId, amount: i32, use_checks: bool) -> bool {
    // If for whatever reason the TOL checks fail, we should still try old methods for depositing currency.
    if amount <= 0 || iaccount::gold_enabled(w) && iaccount::account_deposit_gold(w, from, amount) {
        return true;
    }
    let Some(box_) = muo_server::mobile::part1::props_b::bank_box(w, from) else {
        return false;
    };
    let items = bank_items_of_gold_and_checks(w, box_);

    // Pre-calculate available capacity without modifying anything
    let mut remaining = amount;
    for &item in &items {
        if remaining <= 0 {
            break;
        }
        let Some(ty) = w.item(item).map(|i| i.ty) else {
            continue;
        };
        if ty.is(&GOLD) {
            remaining -= (60000 - w.item(item).map_or(0, |i| i.amount)).min(remaining);
        } else if use_checks && ty.is(&BANK_CHECK) {
            remaining -= (1_000_000 - gold::worth(w, item)).min(remaining);
        }
    }

    // Calculate new item slots needed for the remainder
    if remaining > 0 {
        let needed = slots_needed(remaining, use_checks);
        let max_items = container::max_items(w, box_);
        if max_items != 0 && needed > max_items - ops::get_total(w, box_, TotalType::Items) {
            return false;
        }
    }

    // Capacity verified: execute deposit
    let remaining = top_off(w, &items, amount, use_checks);
    fill_new(w, box_, remaining, use_checks, i32::MAX);
    true
}

/// `DepositUpTo(from, amount, useChecks, reserveSlots)`: returns the amount deposited.
pub fn deposit_up_to(
    w: &mut World,
    from: MobileId,
    amount: i32,
    use_checks: bool,
    reserve_slots: i32,
) -> i32 {
    if iaccount::gold_enabled(w) && iaccount::account_deposit_gold(w, from, amount) {
        return amount;
    }
    let Some(box_) = bank_no_create(w, from) else {
        return 0;
    };

    // Top off existing gold piles and checks
    let items = bank_items_of_gold_and_checks(w, box_);
    let remaining = top_off(w, &items, amount, use_checks);
    if remaining == 0 {
        return amount;
    }

    // Calculate how many new item slots can be filled
    let max_items = container::max_items(w, box_);
    let gm = w
        .mobile(from)
        .is_some_and(|m| m.access_level >= AccessLevel::GameMaster);
    let slots_to_fill = if max_items == 0 || gm {
        i32::MAX
    } else {
        let mut slots_to_fill = max_items - ops::get_total(w, box_, TotalType::Items);
        if reserve_slots > 0 {
            // Count slots needed to determine if the full amount fits. Only reserve if it doesn't: the caller
            // will delete the reserved item on full deposit.
            let mut slots = slots_to_fill;
            let mut r = remaining;
            while r > 0 {
                if !use_checks || r < 5000 {
                    r -= r.min(60000);
                } else if r <= 1_000_000 {
                    r = 0;
                } else {
                    r -= 1_000_000;
                }
                slots -= 1;
                if slots < 0 {
                    slots_to_fill -= reserve_slots;
                    break;
                }
            }
        }
        slots_to_fill
    };

    let left = fill_new(w, box_, remaining, use_checks, slots_to_fill);
    amount - left
}

/// `Deposit(cont, amount)`.
pub fn deposit_into(w: &mut World, cont: ItemId, mut amount: i32) {
    while amount > 0 {
        let item;
        if amount < 5000 {
            item = gold::new_gold(w, amount);
            amount = 0;
        } else if amount <= 1_000_000 {
            item = gold::new_bank_check(w, amount);
            amount = 0;
        } else {
            item = gold::new_bank_check(w, 1_000_000);
            amount -= 1_000_000;
        }
        container::drop_item_default(w, cont, item);
    }
}
