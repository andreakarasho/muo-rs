//! Gift giving: holiday gifts handed out at login.

use muo_primitives::{DateTime, TimeSpan};
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::mobile::part2::items as mitems;

use crate::accounting;
use crate::misc::stamina_system;

/// `GiftResult`.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum GiftResult {
    Backpack,
    BankBox,
}

/// `GiftGiver`: a gift available for a time frame to accounts old enough.
pub trait GiftGiver {
    /// `MinimumAge`: 30 days.
    fn minimum_age(&self) -> TimeSpan {
        TimeSpan::from_days(30)
    }
    fn start(&self) -> DateTime;
    fn finish(&self) -> DateTime;
    fn give_gift(&self, w: &mut World, mob: MobileId);
}

/// `m_Givers`.
#[derive(Default)]
pub struct GiftGivers(Vec<&'static dyn GiftGiver>);

pub fn register(w: &mut World, giver: &'static dyn GiftGiver) {
    w.resources.get_mut::<GiftGivers>().0.push(giver);
}

/// `OnLogin(pm)`.
pub fn on_login(w: &mut World, pm: MobileId) {
    let Some(acct_id) = w.mobile(pm).and_then(|m| m.account) else {
        return;
    };
    let Some((created, last_login)) =
        accounting::account(w, acct_id).map(|a| (a.created, a.last_login))
    else {
        return;
    };

    let now = w.now;
    let givers: Vec<&'static dyn GiftGiver> = w.resources.get_mut::<GiftGivers>().0.clone();

    for giver in givers {
        if now < giver.start() || now >= giver.finish() {
            continue; // not in the correct time frame
        }

        if created > giver.start() - giver.minimum_age() {
            continue; // newly created account
        }

        if last_login >= giver.start() {
            continue; // already got one
        }

        delay_give_gift(w, giver, TimeSpan::from_secs(5), pm);
    }

    if let Some(a) = accounting::account_mut(w, acct_id) {
        a.last_login = now;
    }
}

/// `DelayGiveGift(delay, mob)`.
pub fn delay_give_gift(
    w: &mut World,
    giver: &'static dyn GiftGiver,
    delay: TimeSpan,
    mob: MobileId,
) {
    muo_server::timer::start(w, delay, TimeSpan::ZERO, 1, move |w| {
        giver.give_gift(w, mob)
    });
}

/// `GiveGift(mob, item)`: into the backpack, or the bank box when it won't fit or the player is overloaded.
pub fn give_gift_item(w: &mut World, mob: MobileId, item: ItemId) -> GiftResult {
    if mitems::place_in_backpack(w, mob, item) && !stamina_system::is_overloaded(w, mob) {
        return GiftResult::Backpack;
    }

    if let Some(bank) = muo_server::mobile::part1::props_b::bank_box(w, mob) {
        muo_server::item::container::drop_item_default(w, bank, item);
    }
    GiftResult::BankBox
}
