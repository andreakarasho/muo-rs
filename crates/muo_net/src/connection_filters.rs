//! Connection filters: the checks a new socket passes before it is accepted.
//!
//! Registry of the gates the accept path consults, and their lifecycle. `should_deny` runs on the game
//! loop once per accepted socket, so implementations must be allocation-free, never block, and do no I/O.

use crate::CancelToken;
use std::net::IpAddr;
use std::panic::{AssertUnwindSafe, catch_unwind};
use tracing::{error, info, warn};

/// A gate consulted for every inbound connection, before the socket is configured and before any
/// per-connection allocation (`IConnectionFilter`). Side effects a hit implies (contributing to the ban
/// channel, promoting to an OS firewall) are the filter's own business.
pub trait ConnectionFilter {
    /// Stable id for logging/config (e.g. `firewall`, `blocklist`).
    fn name(&self) -> &str;
    /// Reads configuration; called by [`ConnectionFilters::register`]. No I/O beyond config.
    fn register(&mut self);
    /// Starts any background hydration. The token is set on shutdown.
    fn start(&mut self, token: &CancelToken);
    /// Flushes and tears down. Called during shutdown.
    fn stop(&mut self);
    /// True to deny the connection. A filter returning `Err` (or panicking) is unregistered and the
    /// connection fails open.
    fn should_deny(&mut self, address: IpAddr) -> Result<bool, String>;
}

#[derive(Default)]
pub struct ConnectionFilters {
    filters: Vec<Box<dyn ConnectionFilter>>,
}

impl ConnectionFilters {
    pub fn filters(&self) -> &[Box<dyn ConnectionFilter>] {
        &self.filters
    }

    /// Registers a gate. Idempotent by name. Filters are consulted in registration order, so register
    /// the cheapest and most selective first.
    pub fn register(&mut self, mut filter: Box<dyn ConnectionFilter>) {
        if self.filters.iter().any(|f| f.name() == filter.name()) {
            return;
        }

        filter.register();
        info!("Registered connection filter '{}'", filter.name());
        self.filters.push(filter);
    }

    /// The name of the first filter that denies the connection, if any (short-circuits).
    pub fn should_deny(&mut self, address: IpAddr) -> Option<String> {
        let mut i = 0;
        while i < self.filters.len() {
            // A faulty filter must not take down the accept loop for every connection.
            let answer = catch_unwind(AssertUnwindSafe(|| self.filters[i].should_deny(address)))
                .unwrap_or_else(|_| Err("panicked".into()));

            match answer {
                Ok(true) => return Some(self.filters[i].name().to_string()),
                Ok(false) => i += 1,
                Err(e) => {
                    // One that faults once faults for every connection: drop it and fail open.
                    error!(
                        "Connection filter '{}' threw on the accept path; unregistering it: {e}",
                        self.filters[i].name()
                    );
                    self.filters.remove(i);
                }
            }
        }

        None
    }

    pub fn start(&mut self, token: &CancelToken) {
        for f in &mut self.filters {
            // A filter that cannot hydrate must not crash boot; it simply denies nothing.
            if catch_unwind(AssertUnwindSafe(|| f.start(token))).is_err() {
                error!(
                    "Connection filter '{}' failed to start; continuing without it",
                    f.name()
                );
            }
        }
    }

    pub fn stop(&mut self) {
        for f in &mut self.filters {
            if catch_unwind(AssertUnwindSafe(|| f.stop())).is_err() {
                warn!("Connection filter '{}' threw while stopping", f.name());
            }
        }
    }

    pub fn reset_for_testing(&mut self) {
        self.filters.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::rc::Rc;

    struct Fake {
        name: &'static str,
        deny: bool,
        throws: bool,
        calls: Rc<Cell<i32>>,
        configured: Rc<Cell<bool>>,
    }

    impl Fake {
        fn boxed(
            name: &'static str,
            deny: bool,
            throws: bool,
        ) -> (Box<Self>, Rc<Cell<i32>>, Rc<Cell<bool>>) {
            let calls = Rc::new(Cell::new(0));
            let configured = Rc::new(Cell::new(false));
            (
                Box::new(Fake {
                    name,
                    deny,
                    throws,
                    calls: calls.clone(),
                    configured: configured.clone(),
                }),
                calls,
                configured,
            )
        }
    }

    impl ConnectionFilter for Fake {
        fn name(&self) -> &str {
            self.name
        }
        fn register(&mut self) {
            self.configured.set(true);
        }
        fn start(&mut self, _t: &CancelToken) {}
        fn stop(&mut self) {}
        fn should_deny(&mut self, _a: IpAddr) -> Result<bool, String> {
            self.calls.set(self.calls.get() + 1);
            if self.throws {
                return Err("simulated filter bug".into());
            }
            Ok(self.deny)
        }
    }

    fn addr() -> IpAddr {
        "1.2.3.4".parse().unwrap()
    }

    #[test]
    fn no_filters_denies_nothing() {
        assert_eq!(ConnectionFilters::default().should_deny(addr()), None);
    }

    #[test]
    fn register_is_idempotent_by_name() {
        let mut f = ConnectionFilters::default();
        f.register(Fake::boxed("dupe", false, false).0);
        f.register(Fake::boxed("dupe", true, false).0);
        // The second registration is ignored, so the deny instance never gets consulted.
        assert_eq!(f.filters().len(), 1);
        assert_eq!(f.should_deny(addr()), None);
    }

    #[test]
    fn first_denying_filter_short_circuits_and_is_named() {
        let (a, ca, _) = Fake::boxed("allow-all", false, false);
        let (b, cb, _) = Fake::boxed("deny-all", true, false);
        let (c, cc, _) = Fake::boxed("never-reached", true, false);
        let mut f = ConnectionFilters::default();
        f.register(a);
        f.register(b);
        f.register(c);

        assert_eq!(f.should_deny(addr()).as_deref(), Some("deny-all"));
        assert_eq!((ca.get(), cb.get(), cc.get()), (1, 1, 0));
    }

    #[test]
    fn throwing_filter_is_unregistered_and_fails_open() {
        let (bad, _, _) = Fake::boxed("bad", true, true);
        let (good, cg, _) = Fake::boxed("good", false, false);
        let mut f = ConnectionFilters::default();
        f.register(bad);
        f.register(good);

        assert_eq!(f.should_deny(addr()), None);
        assert_eq!(f.filters().len(), 1);
        assert_eq!(f.filters()[0].name(), "good");
        // Remaining filters still run on the same pass the faulty one was dropped in.
        assert_eq!(cg.get(), 1);
    }

    #[test]
    fn register_configures_immediately() {
        let (x, _, configured) = Fake::boxed("cfg", false, false);
        let mut f = ConnectionFilters::default();
        f.register(x);
        assert!(configured.get());
    }
}
