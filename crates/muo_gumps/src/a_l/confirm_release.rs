//! `ConfirmReleaseGump`.

use muo_content::creature::control;
use muo_content::creature::enums::OrderType;
use muo_content::gumps::base::{RelayInfo, StaticGumpLogic, StaticGumpObj};
use muo_content::gumps::builder::StaticGumpBuilder;
use muo_content::hooks::hooks_mut;
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::net_state::NetStateId;

use super::common::send;

pub struct ConfirmReleaseGump {
    from: MobileId,
    pet: MobileId,
}

/// `new ConfirmReleaseGump(from, pet)`.
pub fn new(from: MobileId, pet: MobileId) -> StaticGumpObj<ConfirmReleaseGump> {
    StaticGumpObj::new(50, 50, ConfirmReleaseGump { from, pet })
}

/// Sets the hook the AI's release command opens this gump through.
pub fn configure(w: &mut World) {
    hooks_mut(w).confirm_release_gump = Some(|w, from, pet| send(w, from, new(from, pet)));
}

impl StaticGumpLogic for ConfirmReleaseGump {
    const TYPE_NAME: &'static str = "Server.Gumps.ConfirmReleaseGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut StaticGumpBuilder) {
        b.add_page(0);
        b.add_background(0, 0, 270, 120, 5054);
        b.add_background(10, 10, 250, 100, 3000);
        // Are you sure you want to release your pet?
        b.add_html_localized(20, 15, 230, 60, 1046257, true, true);
        b.add_button(20, 80, 4005, 4007, 2);
        b.add_html_localized(55, 80, 75, 20, 1011011, false, false); // CONTINUE
        b.add_button(135, 80, 4005, 4007, 1);
        b.add_html_localized(170, 80, 75, 20, 1011012, false, false); // CANCEL
    }

    fn on_response(&mut self, w: &mut World, _sender: NetStateId, info: &RelayInfo) {
        if info.button_id != 2 {
            return;
        }
        let (from, pet) = (self.from, self.pet);
        if muo_content::mx::deleted(w, pet) {
            return;
        }
        if !(control::control_master(w, pet) == Some(from)
            && muo_server::mobile::part2::stats::check_alive(w, from, true)
            && muo_content::mx::map(w, pet) == muo_content::mx::map(w, from)
            && muo_content::mx::in_range(w, pet, from, 14))
        {
            return;
        }
        control::issue_order(w, pet, OrderType::Release, Some(from), None);
    }
}
