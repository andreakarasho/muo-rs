//! `WinterGiftGiver2004`, the box handed out at login between
//! 2004-12-24 and 2005-01-01 (never within the dates a server runs now, but still registered).

use muo_content::factory::make_item;
use muo_content::special_systems::gift_giving::{self, GiftGiver, GiftResult};
use muo_primitives::{DateTime, utility};
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::item::container::drop_item_default;
use muo_server::mobile::part2::messages::send_message_hue;
use muo_server::type_registry::{Phase, TypeRegistry};

struct WinterGiftGiver2004;

static GIVER: WinterGiftGiver2004 = WinterGiftGiver2004;

impl GiftGiver for WinterGiftGiver2004 {
    fn start(&self) -> DateTime {
        DateTime::from_ymd(2004, 12, 24).expect("a date")
    }

    fn finish(&self) -> DateTime {
        DateTime::from_ymd(2005, 1, 1).expect("a date")
    }

    fn give_gift(&self, w: &mut World, mob: MobileId) {
        let Some(gift_box) = make_item(w, "Server.Items.GiftBox") else {
            return;
        };
        for name in [
            "Server.Items.MistletoeDeed",
            "Server.Items.PileOfGlacialSnow",
            "Server.Items.LightOfTheWinterSolstice",
        ] {
            if let Some(i) = make_item(w, name) {
                drop_item_default(w, gift_box, i);
            }
        }

        let random = utility::random(&mut w.rng, 100);
        let name = if random < 60 {
            "Server.Items.DecorativeTopiary"
        } else if random < 84 {
            "Server.Items.FestiveCactus"
        } else {
            "Server.Items.SnowyTree"
        };
        if let Some(i) = make_item(w, name) {
            drop_item_default(w, gift_box, i);
        }

        match gift_giving::give_gift_item(w, mob, gift_box) {
            GiftResult::Backpack => {
                send_message_hue(
                    w,
                    mob,
                    0x482,
                    "Happy Holidays from the team!  Gift items have been placed in your backpack.",
                );
            }
            GiftResult::BankBox => {
                send_message_hue(
                    w,
                    mob,
                    0x482,
                    "Happy Holidays from the team!  Gift items have been placed in your bank box.",
                );
            }
        }
    }
}

/// `WinterGiftGiver2004.Initialize()`.
fn initialize(w: &mut World) {
    gift_giving::register(w, &GIVER);
}

pub fn register(types: &mut TypeRegistry) {
    types.register_phase_default(
        Phase::Initialize,
        "Server.Misc.WinterGiftGiver2004.Initialize",
        initialize,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_window_is_christmas_2004() {
        assert!(GIVER.start() < GIVER.finish());
        assert_eq!(
            GIVER.finish() - GIVER.start(),
            muo_primitives::TimeSpan::from_days(8)
        );
        assert_eq!(GIVER.minimum_age(), muo_primitives::TimeSpan::from_days(30));
    }
}
