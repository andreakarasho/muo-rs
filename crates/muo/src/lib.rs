//! The engine's wiring: content registration, the network that lives beside the `World`, and the loop hooks.

use muo_net::CancelToken;
use muo_net::network::{Network, NetworkConfig};
use muo_net::ping_server::{PingConfig, PingServer};
use muo_server::World;
use muo_server::core_loop::{LoopHooks, ServerHooks};
use muo_server::net_glue::NetworkHooks;

/// Lower layers first: a registration may look up what an earlier one registered.
pub fn register_content(types: &mut muo_server::type_registry::TypeRegistry) {
    muo_content::register(types);
    muo_spells::register(types);
    muo_items::register(types);
    muo_mobiles::register(types);
    muo_multis::register(types);
    muo_engines::register(types);
    muo_gumps::register(types);
    muo_commands::register(types);
    muo_content::scripting::register(types);
    // `BanConfiguration.Configure`: a bad `Configuration/bans.json` stops startup.
    types.register_phase_default(
        muo_server::type_registry::Phase::Configure,
        "Server.Network.Bans.BanConfiguration.Configure",
        |w| {
            let base = w.core.base_directory.clone();
            match muo_net::bans::BanSettings::configure(&base) {
                Ok(s) => w.resources.insert(s),
                Err(e) => w.core.startup_error = Some(e),
            }
        },
    );
}

/// `NetState.Configure/Start` and `PingServer.Configure/Start` from the configuration; the network is returned
/// to the caller, who owns it beside the world.
pub fn start_network(w: &mut World) -> Option<(NetworkHooks, PingServer, CancelToken)> {
    let cfg = &mut w.config;
    let mut nc = NetworkConfig {
        listeners: cfg.listeners().to_vec(),
        ban_settings: w
            .resources
            .get::<muo_net::bans::BanSettings>()
            .cloned()
            .unwrap_or_default(),
        ..NetworkConfig::default()
    };
    // `NetState.Configure`: the packet logging path is read only; the `network.*` keys are get-or-update (an I/O ring
    // setting this server has no use for is still written, so a fresh muo.json lists it).
    nc.packet_logging_path = cfg.get_setting(
        "netstate.packetLoggingPath",
        w.core
            .base_directory
            .join("Packets")
            .to_string_lossy()
            .into_owned(),
    );
    let _ = cfg.get_or_update_setting("network.maxOutstandingSends", 32_i32);
    let send = cfg
        .get_or_update_setting("network.sendBufferSize", 256 * 1024_i32)
        .unwrap_or(256 * 1024);
    let max = cfg
        .get_or_update_setting("network.sendBufferMaxSize", 2 * 1024 * 1024_i32)
        .unwrap_or(2 * 1024 * 1024);
    let _ = cfg.get_or_update_setting("network.sendBufferGrowthBudget", 256_i64 * 1024 * 1024);
    let _ = cfg.get_or_update_setting("network.memoryCeilingPercent", 80_i32);
    let _ = cfg.get_or_update_setting("network.maxBufferSlabs", 128_i32);
    let _ = cfg.get_or_update_setting("network.initialBufferSlabs", 1_i32);
    nc = nc.with_buffer_settings(i64::from(send), i64::from(max));
    if let Some(m) = cfg
        .get_setting_opt::<String>("network.encryptionMode")
        .and_then(|s| muo_net::encryption::EncryptionMode::from_setting(&s))
    {
        nc.encryption.mode = m;
    }
    nc.encryption.debug = cfg.get_setting("network.encryptionDebug", false);
    if !nc.encryption.mode.is_empty() {
        // `EncryptionManager.Configure` logs the mode (the flags enum prints as its name, or "Unencrypted, Encrypted")
        use muo_net::encryption::EncryptionMode as M;
        let m = nc.encryption.mode;
        let name = if m == M::BOTH {
            "Both"
        } else if m == M::ENCRYPTED {
            "Encrypted"
        } else {
            "Unencrypted"
        };
        tracing::info!("Encryption support enabled: {name}");
    }

    let ping_cfg = PingConfig {
        enabled: cfg
            .get_or_update_setting("pingServer.enabled", true)
            .unwrap_or(true),
        port: cfg.get_setting("pingServer.port", 12000_i32) as u16,
        max_queued: cfg.get_setting("pingServer.maxConnections", 2048_i32),
    };
    let listeners = nc.listeners.clone();

    let mut net = match Network::<World>::new(nc, w.tick_count) {
        Ok(n) => n,
        Err(e) => {
            tracing::error!("Failed to configure the network: {e}");
            return None;
        }
    };
    // The packet handlers content registered in its `Configure` functions (`IncomingPackets.Register`).
    if let Some(r) = w
        .resources
        .get::<muo_content::network::IncomingRegistrations>()
    {
        r.apply(&mut net.packets);
    }
    // The connection filters, ban reporters and exemption content set up in `Configure` (`ConnectionFilters.Register`,
    // `BanChannel.Register`, `BanChannel.IsExempt`).
    let reg = std::mem::take(
        w.resources
            .get_mut::<muo_content::network::bans::NetworkRegistrations>(),
    );
    for f in reg.filters {
        net.filters.register(f);
    }
    for r in reg.reporters {
        net.bans.register(r);
    }
    net.bans.is_exempt = reg.is_exempt;
    // `ClosingTokenSource.Token`: set by `Hooks::shutdown`.
    let closing = CancelToken::default();
    net.bans.start(&closing);
    net.filters.start(&closing);
    net.start();
    let mut ping = PingServer::new(ping_cfg);
    ping.start(&listeners);
    Some((NetworkHooks::new(net), ping, closing))
}

/// The loop's hooks: the server's deltas and saves, then the network and the ping server.
pub struct Hooks {
    pub server: ServerHooks,
    pub net: Option<NetworkHooks>,
    pub ping: Option<PingServer>,
    pub closing: Option<CancelToken>,
}

impl LoopHooks for Hooks {
    /// `HandleClosed`: `PingServer.Shutdown()`, `NetState.Shutdown()`, `BanChannel.Stop()`, `ConnectionFilters.Stop()`.
    fn shutdown(&mut self, _w: &mut World) {
        if let Some(t) = &self.closing {
            t.store(true, std::sync::atomic::Ordering::SeqCst);
        }
        if let Some(p) = &mut self.ping {
            p.shutdown();
        }
        if let Some(n) = &mut self.net {
            n.net.shutdown();
            n.net.bans.stop();
            n.net.filters.stop();
        }
    }
    fn mobile_deltas(&mut self, w: &mut World) {
        self.server.mobile_deltas(w);
    }
    fn item_deltas(&mut self, w: &mut World) {
        self.server.item_deltas(w);
    }
    fn network_slice(&mut self, w: &mut World) {
        if let Some(n) = &mut self.net {
            n.network_slice(w);
            // bans a filter decided on while accepting (`BanChannel.Report`)
            let reports = w
                .resources
                .get_mut::<muo_content::network::bans::BanReportQueue>()
                .take();
            for (ip, ttl, reason) in reports {
                n.net.bans.report(ip, ttl, &reason);
            }
        }
        if let Some(p) = &mut self.ping {
            p.service();
        }
    }
    fn world_snapshot(&mut self, w: &mut World, path: &std::path::Path) {
        self.server.world_snapshot(w, path);
    }
    fn wait_for_write_completion(&mut self, w: &mut World) {
        self.server.wait_for_write_completion(w);
    }
    fn is_idle(&mut self, w: &mut World) -> bool {
        self.net.as_mut().is_none_or(|n| n.is_idle(w))
    }
    fn wait_for_completion(&mut self, w: &mut World, ms: i32) {
        match &mut self.net {
            Some(n) => n.wait_for_completion(w, ms),
            None => self.server.wait_for_completion(w, ms),
        }
    }
}
