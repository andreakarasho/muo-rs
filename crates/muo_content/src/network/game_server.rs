//! `GameServer`: the game listener's configuration and startup.

use muo_server::World;
use muo_server::city_info::CityInfo;
use muo_server::net_state::NetStateId;

use super::events;

/// `GameServer.GameLoginEventArgs`.
pub struct GameLoginEventArgs {
    pub state: NetStateId,
    pub username: String,
    pub password: String,
    /// The auth id presented on this game login was issued to this account, from this address, after the account
    /// login packet verified the password.
    pub pre_authenticated: bool,
    pub accepted: bool,
    pub city_info: Vec<CityInfo>,
}

impl GameLoginEventArgs {
    pub fn new(
        state: NetStateId,
        username: String,
        password: String,
        pre_authenticated: bool,
    ) -> Self {
        Self {
            state,
            username,
            password,
            pre_authenticated,
            accepted: false,
            city_info: Vec::new(),
        }
    }
}

/// `GameServer.GameServerLoginEvent(e)`.
pub fn game_server_login_event(w: &mut World, e: &mut GameLoginEventArgs) {
    events::invoke(w, e);
}
