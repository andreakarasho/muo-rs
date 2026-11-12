//! `LoginStats`: the welcome line a player gets on login.

use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::mobile::part2::messages::send_message;

use crate::network::events::PlayerLogin;

/// `LoginStats.OnLogin(pm)`, subscribed to `PlayerMobile.PlayerLoginEvent`.
pub fn on_login(w: &mut World, e: &mut PlayerLogin) {
    let pm: MobileId = e.0;
    let user_count = w.net_states.len();
    let item_count = w.items.len();
    let mobile_count = w.mobiles.len();
    let name = w
        .mobile(pm)
        .and_then(|m| (m.ty.name)(w, pm))
        .unwrap_or_default();
    let s = |n: usize| if n == 1 { "" } else { "s" };
    let text = format!(
        "Welcome, {name}! There {} currently {user_count} user{} online, with {item_count} item{} and {mobile_count} mobile{} in the world.",
        if user_count == 1 { "is" } else { "are" },
        s(user_count),
        s(item_count),
        s(mobile_count)
    );
    send_message(w, pm, &text);
}
