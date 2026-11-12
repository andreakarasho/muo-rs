//! The first-run prompt that creates the owner account.
//!
//! The questions go through `ConsoleInputHandler.ReadLine` (the binary's handler, `ConsoleHandle`). Headless, the
//! `HeadlessConsoleInputException` from `Initialize` is logged and no account is created.

use muo_server::World;
use muo_server::mobile::AccessLevel;
use tracing::{info, warn};

use crate::accounting::{self, create_account};

fn read_line(w: &World) -> Option<String> {
    let handler = w
        .resources
        .get::<crate::systems::console::ConsoleHandle>()?
        .0
        .clone()?;
    match handler.read_line() {
        Ok(line) => line,
        Err(e) => {
            warn!("{e}");
            None
        }
    }
}

/// `AccountPrompt.Initialize`.
pub fn initialize(w: &mut World) {
    if accounting::accounts(w).count() != 0 {
        return;
    }
    warn!("This server has no accounts.");
    info!("Do you want to create the owner account now? (y/n):");
    match read_line(w) {
        Some(a) if a.eq_ignore_ascii_case("y") => {
            info!("Input Username:");
            let username = read_line(w).unwrap_or_default();
            info!("Input Password:");
            let password = read_line(w).unwrap_or_default();
            let id = create_account(w, &username, &password);
            if let Some(a) = accounting::account_mut(w, id) {
                a.access_level = AccessLevel::Owner;
            }
            info!("Owner account created: {username}");
            super::server_access::add_protected_account(w, id, true);
        }
        _ => warn!("No owner account created."),
    }
}
