//! SolenHelper: the solen hives' shared rules.

use muo_primitives::MessageType;
use muo_server::ids::MobileId;
use muo_server::mobile::part2::messages;
use muo_server::world::World;

use super::control::get_master;
use super::data::cd;
use crate::player_mobile::{self, SolenFriendship};

/// `SolenHelper.PackPicnicBasket(solen)`.
pub fn pack_picnic_basket(w: &mut World, solen: MobileId) {
    if crate::hooks::rnd::below(w, 100) < 1 {
        let basket = crate::factory::make_item(w, "Server.Items.PicnicBasket");
        if let Some(b) = basket {
            // `new BeverageBottle(BeverageType.Wine)`
            let wine = crate::factory::make_item_args(
                w,
                "Server.Items.BeverageBottle",
                &[crate::props::Value::Enum(4)],
            );
            if let Some(i) = wine {
                muo_server::item::container::drop_item_default(w, b, i);
            }
            if let Some(i) = crate::factory::make_item(w, "Server.Items.CheeseWedge") {
                muo_server::item::container::drop_item_default(w, b, i);
            }
        }
        super::setters::pack_item(w, solen, basket);
    }
}

fn friendship_of(w: &World, m: MobileId) -> Option<SolenFriendship> {
    player_mobile::data(w, m).map(|d| d.solen_friendship)
}

fn check(w: &World, m: MobileId, kind: SolenFriendship) -> bool {
    if cd(w, m).is_some()
        && let Some(master) = get_master(w, m)
        && player_mobile::data(w, master).is_some()
    {
        return check(w, master, kind);
    }
    friendship_of(w, m) == Some(kind)
}

/// `CheckRedFriendship(m)`.
pub fn check_red_friendship(w: &World, m: MobileId) -> bool {
    check(w, m, SolenFriendship::Red)
}

/// `CheckBlackFriendship(m)`.
pub fn check_black_friendship(w: &World, m: MobileId) -> bool {
    check(w, m, SolenFriendship::Black)
}

fn on_damage(w: &mut World, from: MobileId, kind: SolenFriendship) {
    if cd(w, from).is_some()
        && let Some(master) = get_master(w, from)
        && player_mobile::data(w, master).is_some()
    {
        on_damage(w, master, kind);
    }
    if friendship_of(w, from) == Some(kind) {
        // The solen revoke their friendship. You will now be considered an intruder.
        messages::local_overhead_message_number(w, from, MessageType::REGULAR, 0x3B2, 1054103, "");
        if let Some(d) = player_mobile::data_mut(w, from) {
            d.solen_friendship = SolenFriendship::None;
        }
    }
}

/// `OnRedDamage(from)`.
pub fn on_red_damage(w: &mut World, from: MobileId) {
    on_damage(w, from, SolenFriendship::Red);
}

/// `OnBlackDamage(from)`.
pub fn on_black_damage(w: &mut World, from: MobileId) {
    on_damage(w, from, SolenFriendship::Black);
}
