//! Serves the login path alone on 127.0.0.1:<port> (default 2596) for `muo-probe.py`, until <seconds> pass:
//! `cargo run -p muo_content --example login_serve -- 2596 60`. It stands in for `muo` while the binary is not wired
//! to the network: accounts live in memory, the maps are empty, and `$MUO_DISTRIBUTION` supplies
//! `Data/expansions.json`.
use muo_net::network::{Network, NetworkConfig};
use muo_primitives::DateTime;
use muo_server::World;
use muo_server::net_glue::NetworkHooks;
use muo_server::type_registry::{Phase, invoke_phase};
use std::time::{Duration, Instant};

fn main() {
    let port: u16 = std::env::args()
        .nth(1)
        .and_then(|p| p.parse().ok())
        .unwrap_or(2596);
    let secs: u64 = std::env::args()
        .nth(2)
        .and_then(|p| p.parse().ok())
        .unwrap_or(60);
    tracing_subscriber_init();
    let mut w = World::new();
    let base = std::path::PathBuf::from(
        std::env::var_os("MUO_DISTRIBUTION").expect("set MUO_DISTRIBUTION"),
    );
    w.core.base_directory = std::env::temp_dir().join("login_ws_base");
    std::fs::create_dir_all(w.core.base_directory.join("Configuration")).ok();
    let table = muo_data::expansion::ExpansionTable::load(&base).expect("expansions");
    w.resources.insert(table);
    w.core.expansion = Some(muo_data::expansion::Expansion::ML);
    w.maps.add(muo_server::map::Map::new(
        0,
        0,
        0,
        7168,
        4096,
        0,
        "Felucca",
        Default::default(),
    ));
    w.maps.add(muo_server::map::Map::new(
        1,
        1,
        1,
        7168,
        4096,
        1,
        "Trammel",
        Default::default(),
    ));
    muo_content::login::register(&mut w.types);
    muo_content::misc::race_definitions::configure(&mut w);
    invoke_phase(&mut w, Phase::Configure);
    invoke_phase(&mut w, Phase::Initialize);

    let cfg = NetworkConfig {
        listeners: vec![format!("127.0.0.1:{port}").parse().unwrap()],
        ..Default::default()
    };
    let mut net = Network::<World>::new(cfg, 0).unwrap();
    w.resources
        .get_mut::<muo_content::network::IncomingRegistrations>()
        .apply(&mut net.packets);
    let addrs = net.start();
    println!("listening {:?}", addrs);
    let mut hooks = NetworkHooks::new(net);
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(secs) {
        w.tick_count = start.elapsed().as_millis() as i64;
        w.now = DateTime::utc_now();
        muo_server::timer::slice(&mut w);
        hooks.slice(&mut w);
        std::thread::sleep(Duration::from_millis(2));
    }
}

mod log {
    //! A minimal stdout `tracing` subscriber (no extra dependency): `LEVEL: message` per event.

    use tracing::field::{Field, Visit};
    use tracing::span::{Attributes, Id, Record};
    use tracing::{Event, Metadata, Subscriber};

    struct Message(String);

    impl Visit for Message {
        fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
            if field.name() == "message" {
                self.0 = format!("{value:?}");
            }
        }
    }

    struct Console;

    impl Subscriber for Console {
        fn enabled(&self, m: &Metadata<'_>) -> bool {
            *m.level() <= tracing::Level::INFO
        }
        fn new_span(&self, _: &Attributes<'_>) -> Id {
            Id::from_u64(1)
        }
        fn record(&self, _: &Id, _: &Record<'_>) {}
        fn record_follows_from(&self, _: &Id, _: &Id) {}
        fn event(&self, e: &Event<'_>) {
            let mut m = Message(String::new());
            e.record(&mut m);
            println!("{}: {}", e.metadata().level(), m.0);
        }
        fn enter(&self, _: &Id) {}
        fn exit(&self, _: &Id) {}
    }

    pub fn init() {
        let _ = tracing::subscriber::set_global_default(Console);
    }
}
fn tracing_subscriber_init() {
    log::init();
}
