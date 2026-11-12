//! Server access: `Configuration/server-access.json` and the protected accounts
//! that a login resets to owner.

use muo_primitives::dotnet::InvariantCase;
use std::collections::BTreeSet;
use std::path::PathBuf;

use muo_server::World;
use muo_server::events::AccountLoginEventArgs;
use muo_server::iaccount::AccountId;
use muo_server::mobile::AccessLevel;
use muo_server::packets::account::ALRReason;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::accounting::{account, account_mut, check_password, get_account, is_banned, set_banned};

const CONFIGURATION_PATH: &str = "Configuration/server-access.json";

/// `ServerAccessConfiguration`.
#[derive(Default, Serialize, Deserialize, Clone)]
pub struct ServerAccessConfiguration {
    #[serde(rename = "protectedAccounts", default)]
    pub protected_accounts: BTreeSet<String>,
}

fn path(w: &World) -> PathBuf {
    w.core.base_directory.join(CONFIGURATION_PATH)
}

fn config(w: &mut World) -> &mut ServerAccessConfiguration {
    w.resources.get_mut::<ServerAccessConfiguration>()
}

/// `SaveConfiguration`.
pub fn save_configuration(w: &mut World) {
    let p = path(w);
    let c = config(w).clone();
    if let Some(dir) = p.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    match serde_json::to_string_pretty(&c) {
        Ok(text) => {
            if let Err(e) = std::fs::write(&p, text) {
                warn!("Could not write {}: {e}", p.display());
            }
        }
        Err(e) => warn!("Could not serialize {}: {e}", p.display()),
    }
}

/// `AddProtectedAccount`.
pub fn add_protected_account(w: &mut World, acct: AccountId, save: bool) {
    let Some(username) = account(w, acct).map(|a| a.username.lower_inv()) else {
        return;
    };
    config(w).protected_accounts.insert(username.clone());
    info!("Protected account added: {username}");
    if save {
        save_configuration(w);
    }
}

/// `RemoveProtectedAccount`.
pub fn remove_protected_account(w: &mut World, acct: AccountId, save: bool) {
    let Some(username) = account(w, acct).map(|a| a.username.lower_inv()) else {
        return;
    };
    config(w).protected_accounts.remove(&username);
    info!("Protected account removed: {username}");
    if save {
        save_configuration(w);
    }
}

/// `ServerAccess.Configure`.
pub fn configure(w: &mut World) {
    let p = path(w);
    if !p.exists() {
        save_configuration(w);
        return;
    }
    // `JsonConfig.Deserialize` throws on a bad file: startup stops
    let loaded = match std::fs::read_to_string(&p)
        .map_err(|e| e.to_string())
        .and_then(|t| {
            muo_data::json::from_str::<ServerAccessConfiguration>(&t).map_err(|e| e.to_string())
        }) {
        Ok(c) => c,
        Err(e) => {
            w.core.startup_error = Some(format!("Failed to deserialize {}: {e}.", p.display()));
            return;
        }
    };
    if !loaded.protected_accounts.is_empty() {
        let list = loaded
            .protected_accounts
            .iter()
            .cloned()
            .collect::<Vec<_>>()
            .join(", ");
        info!("Protected accounts registered: {list}");
    }
    *config(w) = loaded;
}

/// `ServerAccess.Initialize`.
pub fn initialize(w: &mut World) {
    w.events
        .account_login
        .subscribe(event_sink_reset_protected_account);
}

/// `EventSink_ResetProtectedAccount`.
pub fn event_sink_reset_protected_account(w: &mut World, e: &mut AccountLoginEventArgs) {
    let username = e.username.lower_inv();
    if !config(w).protected_accounts.contains(&username) {
        return;
    }
    let Some(acct) = get_account(w, &username) else {
        return;
    };
    let owner = account(w, acct).is_some_and(|a| a.access_level >= AccessLevel::Owner);
    let banned = is_banned(w, acct);
    if !banned && owner || !check_password(w, acct, &e.password) {
        return;
    }
    set_banned(w, acct, false);
    if let Some(a) = account_mut(w, acct) {
        a.access_level = AccessLevel::Owner;
    }
    warn!("Protected account \"{username}\" has been reset.");
    if e.reject_reason == ALRReason::Blocked as u8
        || e.reject_reason == ALRReason::BadPass as u8
        || e.reject_reason == ALRReason::BadComm as u8
    {
        e.accepted = true;
    }
}
