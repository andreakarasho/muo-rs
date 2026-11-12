//! Ban exemptions: combines the manual allowlist and the login allowlist into
//! the one answer `BanChannel.IsExempt` asks for, so neither source has to know about the other.

use std::net::IpAddr;

use muo_net::bans::ban_reasons;
use muo_server::World;

use super::login_allowlist::LoginAllowlist;
use super::manual_allowlist::{self, ManualAllowlist};
use super::{NetworkRegistrations, unix_now};

/// `BanExemptions.Configure`: sets `BanChannel.IsExempt`.
pub fn configure(w: &mut World) {
    let manual = manual_allowlist::handle(w);
    let login = w.resources.get_mut::<LoginAllowlist>().clone();
    w.resources.get_mut::<NetworkRegistrations>().is_exempt =
        Some(Box::new(move |address, reason| {
            is_exempt(&manual, address, reason, |a, r| {
                login.is_exempt_from_escalation(a, r, unix_now())
            })
        }));
}

/// `IsExempt(address, reason, loginAllowlist)`. The login allowlist is stateful (calling it spends a strike), so
/// it is not asked once the answer is already decided.
pub fn is_exempt(
    manual: &ManualAllowlist,
    address: IpAddr,
    reason: &str,
    login_allowlist: impl FnOnce(IpAddr, &str) -> bool,
) -> bool {
    // Never suppress an operator's explicit ban. Checked first so it also costs no strike.
    if !ban_reasons::is_behavioral(reason) {
        return false;
    }
    // Deliberate and unconditional, so it wins and must not spend the earned list's strikes.
    if manual.contains(address) {
        return true;
    }
    login_allowlist(address, reason)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::bans::snapshot::BlocklistSnapshot;
    use std::cell::Cell;

    fn manual(lines: &str) -> ManualAllowlist {
        let m = ManualAllowlist::default();
        m.load_for_testing(BlocklistSnapshot::build(lines.as_bytes()).0);
        m
    }

    #[test]
    fn a_manual_ban_is_never_exempt_and_costs_no_strike() {
        let asked = Cell::new(false);
        let m = manual("1.2.3.4");
        assert!(!is_exempt(
            &m,
            "1.2.3.4".parse().unwrap(),
            ban_reasons::MANUAL,
            |_, _| {
                asked.set(true);
                true
            }
        ));
        assert!(!asked.get());
    }

    #[test]
    fn the_manual_list_wins_before_the_login_list_is_asked() {
        let asked = Cell::new(false);
        let m = manual("1.2.3.4");
        assert!(is_exempt(
            &m,
            "1.2.3.4".parse().unwrap(),
            ban_reasons::RATE_LIMIT,
            |_, _| {
                asked.set(true);
                false
            }
        ));
        assert!(!asked.get());
        assert!(!is_exempt(
            &m,
            "5.6.7.8".parse().unwrap(),
            ban_reasons::RATE_LIMIT,
            |_, _| false
        ));
        assert!(is_exempt(
            &m,
            "5.6.7.8".parse().unwrap(),
            ban_reasons::RATE_LIMIT,
            |_, _| true
        ));
    }

    // Addresses come from TEST-NET-1 (192.0.2.0/24). There is no null-address case: an address is a non-optional
    // IpAddr here.
    fn never_called(_: IpAddr, _: &str) -> bool {
        panic!("The login allowlist must not be consulted once the answer is already decided.");
    }

    fn listed() -> IpAddr {
        "192.0.2.10".parse().unwrap()
    }

    #[test]
    fn manual_allowlist_exempts_behavioral_contributions() {
        let m = manual("192.0.2.10");
        // Subtracting from the blocklist does nothing for behavioural detections, which never consult it.
        for reason in [
            ban_reasons::FOREIGN_PROTOCOL,
            ban_reasons::RATE_LIMIT,
            ban_reasons::SILENT_CONNECT,
            ban_reasons::INVALID_SEED,
        ] {
            assert!(is_exempt(&m, listed(), reason, never_called), "{reason}");
        }
    }

    #[test]
    fn manual_allowlist_covers_cidr_entries() {
        // Carve-outs are CIDRs, so a shared-CGNAT player is only covered if ranges work here.
        let m = manual("192.0.2.0/24");
        assert!(is_exempt(
            &m,
            listed(),
            ban_reasons::RATE_LIMIT,
            never_called
        ));
        assert!(is_exempt(
            &m,
            "192.0.2.254".parse().unwrap(),
            ban_reasons::RATE_LIMIT,
            never_called
        ));
        assert!(!is_exempt(
            &m,
            "192.0.3.1".parse().unwrap(),
            ban_reasons::RATE_LIMIT,
            |_, _| false
        ));
    }

    #[test]
    fn manual_bans_are_never_exempt_even_when_allowlisted() {
        let m = manual("192.0.2.10");
        // An explicit decision outranks the operator's own carve-out, and must not cost a strike.
        assert!(!is_exempt(&m, listed(), ban_reasons::MANUAL, never_called));
    }

    #[test]
    fn unopted_reasons_are_never_exempt() {
        let m = manual("192.0.2.10");
        assert!(!is_exempt(
            &m,
            listed(),
            ban_reasons::BLOCKLIST,
            never_called
        ));
        assert!(!is_exempt(&m, listed(), "some-future-reason", never_called));
    }

    #[test]
    fn falls_through_to_the_login_allowlist_when_not_file_listed() {
        let m = manual("");
        let consulted = Cell::new(0);
        let result = is_exempt(
            &m,
            "192.0.2.11".parse().unwrap(),
            ban_reasons::RATE_LIMIT,
            |_, _| {
                consulted.set(consulted.get() + 1);
                true
            },
        );
        assert!(result);
        assert_eq!(consulted.get(), 1);
    }
}
