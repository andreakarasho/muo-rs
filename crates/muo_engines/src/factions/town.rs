//! `Town`, `GuardList` and `VendorList`: a town's guard and vendor lists, upkeep and
//! income, `Capture`, the order-firing target, `GrantTownSilver`, and `Town.Initialize`.
//!
//! A town is its index in [`TOWN_DEFS`]. `GuardLists` follow the guard definitions of the owning faction
//! (`FactionsState::town_guards[town][definition]`), `VendorLists` follow [`VENDOR_DEFS`].

use muo_primitives::dotnet::IgnoreCase;
use muo_primitives::{DateTime, TimeSpan};
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::mobile::AccessLevel;
use muo_server::mobile::part2::messages as mm;
use muo_server::region as regions;
use muo_server::target::{self, TargetFlags, Targeted};
use muo_server::timer;

use muo_content::hooks::rnd;

use super::tables::{FACTION_DEFS, TOWN_DEFS, VENDOR_DEFS};
use super::{FACET, st, st_mut};

pub const SILVER_CAPTURE_BONUS: i32 = 10000;
pub const TAX_CHANGE_PERIOD: TimeSpan = TimeSpan::from_hours(12);
pub const INCOME_PERIOD: TimeSpan = TimeSpan::from_days(1);

/// `Town.Owner`.
pub fn owner(w: &World, t: usize) -> Option<usize> {
    st(w).and_then(|s| s.towns[t].owner)
}

/// `Town.Tax`.
pub fn tax(w: &World, t: usize) -> Option<i32> {
    st(w).map(|s| s.towns[t].tax)
}

/// `TaxChangeReady`.
pub fn tax_change_ready(w: &World, t: usize) -> bool {
    st(w).is_some_and(|s| s.towns[t].last_tax_change + TAX_CHANGE_PERIOD < w.now)
}

/// `ConstructGuardLists()`.
pub fn construct_guard_lists(w: &mut World, t: usize) {
    let n = owner(w, t).map_or(0, |f| FACTION_DEFS[f].guards.len());
    st_mut(w).town_guards[t] = vec![Vec::new(); n];
}

/// `FinanceUpkeep`.
pub fn finance_upkeep(w: &World, t: usize) -> i32 {
    st(w).map_or(0, |s| {
        s.town_vendors[t]
            .iter()
            .zip(VENDOR_DEFS.iter())
            .map(|(l, d)| l.len() as i32 * d.upkeep)
            .sum()
    })
}

/// `SheriffUpkeep`.
pub fn sheriff_upkeep(w: &World, t: usize) -> i32 {
    let Some(f) = owner(w, t) else { return 0 };
    st(w).map_or(0, |s| {
        s.town_guards[t]
            .iter()
            .zip(FACTION_DEFS[f].guards.iter())
            .map(|(l, d)| l.len() as i32 * d.upkeep)
            .sum()
    })
}

/// `DailyIncome`.
pub fn daily_income(w: &World, t: usize) -> i32 {
    10000 * (100 + st(w).map_or(0, |s| s.towns[t].tax)) / 100
}

/// `NetCashFlow`.
pub fn net_cash_flow(w: &World, t: usize) -> i32 {
    daily_income(w, t) - finance_upkeep(w, t) - sheriff_upkeep(w, t)
}

/// `Monolith`: the first `TownMonolith` of the town.
pub fn monolith(w: &World, t: usize) -> Option<muo_server::ids::ItemId> {
    st(w)?.monoliths.iter().copied().find(|&m| {
        super::items::is_town_monolith(w, m) && super::items::monolith_town(w, m) == Some(t)
    })
}

/// `Town.FromRegion(reg)`.
pub fn from_region(w: &World, region: muo_server::region::RegionId) -> Option<usize> {
    if w.region(region)?.map != FACET {
        return None;
    }
    (0..TOWN_DEFS.len()).find(|&t| regions::is_part_of_name(w, region, TOWN_DEFS[t].region, false))
}

/// `Town.TryParse(s)`: the town whose friendly name is `s`, ignoring case.
pub fn parse(s: &str) -> Option<usize> {
    TOWN_DEFS.iter().position(|d| d.friendly_name.eq_ic(s))
}

/// `IsSheriff(mob)`.
pub fn is_sheriff(w: &World, t: usize, m: MobileId) -> bool {
    w.mobile(m).is_some_and(|x| {
        x.access_level >= AccessLevel::GameMaster
            || st(w).is_some_and(|s| s.towns[t].sheriff == Some(m))
    })
}

/// `IsFinance(mob)`.
pub fn is_finance(w: &World, t: usize, m: MobileId) -> bool {
    w.mobile(m).is_some_and(|x| {
        x.access_level >= AccessLevel::GameMaster
            || st(w).is_some_and(|s| s.towns[t].finance == Some(m))
    })
}

fn type_name(w: &World, m: MobileId) -> Option<&'static str> {
    w.mobile(m).map(|x| x.ty.full_name)
}

/// `RegisterGuard(guard)`.
pub fn register_guard(w: &mut World, t: usize, guard: MobileId) -> bool {
    let (Some(f), Some(name)) = (owner(w, t), type_name(w, guard)) else {
        return false;
    };
    // `FindGuardList(guard.GetType())`: the owner's definition of that type.
    let Some(i) = FACTION_DEFS[f]
        .guards
        .iter()
        .position(|d| d.type_name == name)
    else {
        return false;
    };
    match st_mut(w).town_guards[t].get_mut(i) {
        Some(list) => {
            list.push(guard);
            true
        }
        None => false,
    }
}

/// `UnregisterGuard(guard)`.
pub fn unregister_guard(w: &mut World, t: usize, guard: MobileId) -> bool {
    let (Some(f), Some(name)) = (owner(w, t), type_name(w, guard)) else {
        return false;
    };
    let Some(i) = FACTION_DEFS[f]
        .guards
        .iter()
        .position(|d| d.type_name == name)
    else {
        return false;
    };
    let Some(list) = st_mut(w).town_guards[t].get_mut(i) else {
        return false;
    };
    match list.iter().position(|&g| g == guard) {
        Some(p) => {
            list.remove(p);
            true
        }
        None => false,
    }
}

/// `RegisterVendor(vendor)`.
pub fn register_vendor(w: &mut World, t: usize, vendor: MobileId) -> bool {
    let Some(name) = type_name(w, vendor) else {
        return false;
    };
    let Some(i) = VENDOR_DEFS.iter().position(|d| d.type_name == name) else {
        return false;
    };
    st_mut(w).town_vendors[t][i].push(vendor);
    true
}

/// `UnregisterVendor(vendor)`.
pub fn unregister_vendor(w: &mut World, t: usize, vendor: MobileId) -> bool {
    let Some(name) = type_name(w, vendor) else {
        return false;
    };
    let Some(i) = VENDOR_DEFS.iter().position(|d| d.type_name == name) else {
        return false;
    };
    let list = &mut st_mut(w).town_vendors[t][i];
    match list.iter().position(|&v| v == vendor) {
        Some(p) => {
            list.remove(p);
            true
        }
        None => false,
    }
}

/// `BuildFinanceList()`.
pub fn build_finance_list(w: &World, t: usize) -> Vec<MobileId> {
    let Some(s) = st(w) else { return Vec::new() };
    s.town_vendors[t]
        .iter()
        .chain(s.town_guards[t].iter())
        .flatten()
        .copied()
        .collect()
}

fn delete_mobile(w: &mut World, m: MobileId) {
    muo_server::mobile::base::delete(w, m);
}

/// `Capture(f)`.
pub fn capture(w: &mut World, t: usize, f: Option<usize>) {
    let old = owner(w, t);
    if old == f {
        return;
    }
    let now = w.now;
    if old.is_none() {
        // going from unowned to owned
        st_mut(w).towns[t].last_income = now;
        if let Some(f) = f {
            st_mut(w).factions[f].silver += SILVER_CAPTURE_BONUS;
        }
    } else if f.is_none() {
        // going from owned to unowned
        st_mut(w).towns[t].last_income = DateTime::MIN;
    } else if let Some(f) = f {
        // otherwise changing hands, income timer doesn't change
        st_mut(w).factions[f].silver += SILVER_CAPTURE_BONUS;
    }
    st_mut(w).towns[t].owner = f;

    super::set_town_sheriff(w, t, None);
    super::set_town_finance(w, t, None);

    if let Some(m) = monolith(w, t) {
        super::items::monolith_set_faction(w, m, f);
    }

    let vendors: Vec<MobileId> = st(w).map_or_else(Vec::new, |s| {
        s.town_vendors[t]
            .iter()
            .flat_map(|l| l.iter().rev())
            .copied()
            .collect()
    });
    for v in vendors {
        delete_mobile(w, v);
    }
    let guards: Vec<MobileId> = st(w).map_or_else(Vec::new, |s| {
        s.town_guards[t]
            .iter()
            .flat_map(|l| l.iter().rev())
            .copied()
            .collect()
    });
    for g in guards {
        delete_mobile(w, g);
    }
    construct_guard_lists(w, t);
}

/// `CheckIncome()` (every minute).
pub fn check_income(w: &mut World, t: usize) {
    let due = st(w).is_some_and(|s| s.towns[t].last_income + INCOME_PERIOD <= w.now);
    if !due || owner(w, t).is_none() {
        return;
    }
    process_income(w, t);
}

/// `ProcessIncome()`.
pub fn process_income(w: &mut World, t: usize) {
    st_mut(w).towns[t].last_income = w.now;
    let mut flow = net_cash_flow(w, t);
    let silver = |w: &World| st(w).map_or(0, |s| s.towns[t].silver);
    if silver(w) + flow < 0 {
        let mut to_delete = build_finance_list(w, t);
        while silver(w) + flow < 0 && !to_delete.is_empty() {
            let i = rnd::below(w, to_delete.len() as i32) as usize;
            let mob = to_delete.remove(i);
            delete_mobile(w, mob);
            flow = net_cash_flow(w, t);
        }
    }
    st_mut(w).towns[t].silver += flow;
}

/// `BeginOrderFiring(from)`.
pub fn begin_order_firing(w: &mut World, t: usize, from: MobileId) {
    let (fin, sher) = (is_finance(w, t, from), is_sheriff(w, t, from));
    let kind = match (fin, sher) {
        (true, true) => "vendor or guard", // GM only
        (true, false) => "vendor",
        (false, true) => "guard",
        // `type` stays null and prints as nothing.
        (false, false) => "",
    };
    mm::send_message(w, from, &format!("Target the {kind} you wish to dismiss."));
    target::begin_target(
        w,
        from,
        12,
        false,
        TargetFlags::NONE,
        move |w, from, obj| end_order_firing(w, t, from, obj),
    );
}

/// `EndOrderFiring(from, obj)`.
pub fn end_order_firing(w: &mut World, t: usize, from: MobileId, obj: Targeted) {
    let (fin, sher) = (is_finance(w, t, from), is_sheriff(w, t, from));
    let m = match obj {
        Targeted::Entity(muo_server::ids::EntityId::Mobile(m)) => Some(m),
        _ => None,
    };
    if let Some(m) = m {
        if fin && super::mobiles::vendor_town(w, m) == Some(t) {
            delete_mobile(w, m);
            return;
        }
        if sher && super::mobiles::guard_town(w, m) == Some(t) {
            delete_mobile(w, m);
            return;
        }
    }
    let text = match (fin, sher) {
        (true, true) => "That is not a vendor or guard!",
        (true, false) => "That is not a vendor!",
        (false, true) => "That is not a guard!",
        (false, false) => return,
    };
    mm::send_message(w, from, text);
}

/// `Town.Initialize`: `town.Sheriff = town.Sheriff` sets the `PlayerState` of the role.
pub fn initialize(w: &mut World) {
    for t in 0..TOWN_DEFS.len() {
        let (sheriff, finance) =
            st(w).map_or((None, None), |s| (s.towns[t].sheriff, s.towns[t].finance));
        super::set_town_sheriff(w, t, sheriff);
        super::set_town_finance(w, t, finance);
        // `StartIncomeTimer()`
        timer::start(
            w,
            TimeSpan::from_mins(1),
            TimeSpan::from_mins(1),
            0,
            move |w| check_income(w, t),
        );
    }
}

/// `GrantTownSilver_OnCommand`.
pub fn grant_town_silver(w: &mut World, e: &mut muo_server::commands::CommandEventArgs) {
    let from = e.mobile;
    let region = muo_server::mobile::part1::props_b::region(w, from);
    let Some(t) = from_region(w, region) else {
        mm::send_message(w, from, "You are not in a faction town.");
        return;
    };
    if e.length() == 0 {
        mm::send_message(w, from, "Format: GrantTownSilver <amount>");
        return;
    }
    let amount = e.get_int32(0);
    let silver = {
        let s = &mut st_mut(w).towns[t].silver;
        *s = s.wrapping_add(amount);
        *s
    };
    mm::send_message(
        w,
        from,
        &format!(
            "You have granted {} silver to the town. It now has {} silver.",
            muo_content::items::gold::n0(amount as i64),
            muo_content::items::gold::n0(silver as i64)
        ),
    );
}
