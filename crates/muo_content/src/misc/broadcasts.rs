//! Broadcasts: tells everyone online when the server shuts down or crashes.

use muo_server::World;

/// `Broadcasts.Initialize`.
pub fn initialize(w: &mut World) {
    w.events
        .server_crashed
        .subscribe(|w, _| w.broadcast(0x35, true, "The server has crashed."));
    w.events
        .shutdown
        .subscribe(|w, _| w.broadcast(0x35, true, "The server has shut down."));
}
