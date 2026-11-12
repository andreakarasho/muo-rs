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
