//! The account handler (`AccountHandler`) and `AccountSecurity.Configure`.
//!
//! The account login runs in [`on_account_login`] (`EventSink.AccountLogin`) and the game login in
//! [`on_game_server_login`] (`GameServer.GameServerLoginEvent`). The password worker thread is in (see
//! `password_worker.rs`), so with the worker enabled a login is handed over (`Deferred`) and completed when the
//! verdict lands.

use std::net::IpAddr;
use std::rc::Rc;

use indexmap::IndexMap;
use muo_server::World;
use muo_server::commands::{CommandEventArgs, CommandSystem};
use muo_server::events::AccountLoginEventArgs;
use muo_server::iaccount::AccountId;
use muo_server::ids::MobileId;
use muo_server::mobile::AccessLevel;
use muo_server::mobile::part1::combat::to_string as mobile_to_string;
use muo_server::net_state::NetStateId;
use muo_server::packets::account::{
    ALRReason, AccountChars, CharSlot, DeleteResultType, send_character_delete_result,
    send_character_list_update,
};
use tracing::info;

use super::security::PasswordProtectionAlgorithm;
use super::{
    AccountComment, AccountHooks, account, account_mut, accounts, check_access, check_password,
    create_account, get_account, has_access, is_banned, log_access, mobile_at,
};
use crate::network::game_server::GameLoginEventArgs;

/// The statics of `AccountHandler` and `AccountSecurity`.
pub struct HandlerState {
    pub max_accounts_per_ip: i32,
    pub auto_account_creation: bool,
    /// `RestrictDeletion = !TestCenter.Enabled`.
    pub restrict_deletion: bool,
    pub password_command_enabled: bool,
    pub lockdown_level: AccessLevel,
    pub ip_table: Option<IndexMap<IpAddr, i32>>,
    /// `AccountSecurity.CurrentAlgorithm`.
    pub current_algorithm: PasswordProtectionAlgorithm,
}

impl Default for HandlerState {
    fn default() -> Self {
        HandlerState {
            max_accounts_per_ip: 1,
            auto_account_creation: true,
            restrict_deletion: true,
            password_command_enabled: false,
            lockdown_level: AccessLevel::Player,
            ip_table: None,
            current_algorithm: PasswordProtectionAlgorithm::Argon2,
        }
    }
}

const DELETE_DELAY_DAYS: i64 = 7;

pub fn state(w: &mut World) -> &mut HandlerState {
    w.resources.get_mut::<HandlerState>()
}

pub fn current_algorithm(w: &World) -> PasswordProtectionAlgorithm {
    w.resources
        .get::<HandlerState>()
        .map_or(PasswordProtectionAlgorithm::Argon2, |s| s.current_algorithm)
}

/// `AccountHandler.LockdownLevel`.
pub fn lockdown_level(w: &World) -> AccessLevel {
    w.resources
        .get::<HandlerState>()
        .map_or(AccessLevel::Player, |s| s.lockdown_level)
}

/// `AccountHandler.IPTable`: built from the accounts' first login address on first use.
pub fn ip_table(w: &mut World) -> &mut IndexMap<IpAddr, i32> {
    if state(w).ip_table.is_none() {
        let mut table = IndexMap::new();
        for a in accounts(w).iter() {
            if let Some(ip) = a.login_ips.first() {
                *table.entry(*ip).or_insert(0) += 1;
            }
        }
        state(w).ip_table = Some(table);
    }
    state(w).ip_table.as_mut().expect("just built")
}

/// `AccountSecurity.Configure` then `AccountHandler.Configure`.
pub fn configure(w: &mut World) {
    super::install_gold_hooks(w);
    let name: String = w
        .config
        .get_or_update_setting("accountSecurity.encryptionAlgorithm", "Argon2".to_owned())
        .unwrap_or_else(|_| "Argon2".to_owned());
    let alg = name.parse().unwrap_or(PasswordProtectionAlgorithm::Argon2);
    if alg < PasswordProtectionAlgorithm::Sha2 {
        // An obsolete algorithm stops the server from starting.
        panic!(
            "Security: {} is obsolete and not secure. Do not use it.",
            alg.name()
        );
    }
    state(w).current_algorithm = alg;

    let max = w
        .config
        .get_or_update_setting("accountHandler.maxAccountsPerIP", 1_i32)
        .unwrap_or(1);
    let auto = w
        .config
        .get_or_update_setting("accountHandler.enableAutoAccountCreation", true)
        .unwrap_or(true);
    let pw_cmd = w
        .config
        .get_or_update_setting("accountHandler.enablePlayerPasswordCommand", false)
        .unwrap_or(false);
    let s = state(w);
    s.max_accounts_per_ip = max;
    s.auto_account_creation = auto;
    s.password_command_enabled = pw_cmd;
    if pw_cmd {
        CommandSystem::register(
            w,
            "Password",
            AccessLevel::Player,
            &[],
            Rc::new(password_on_command),
        );
    }
}

/// `AccountHandler.Initialize`.
pub fn initialize(w: &mut World) {
    w.events.account_login.subscribe(on_account_login);
    crate::network::events::subscribe::<GameLoginEventArgs>(w, on_game_server_login);
    // PasswordWorker.Stop is subscribed when the worker thread starts (password_worker.rs).
}

/// `MatchClassC`: the first three bytes of two IPv4 addresses are equal.
fn match_class_c(a: IpAddr, b: IpAddr) -> bool {
    match (a, b) {
        (IpAddr::V4(a), IpAddr::V4(b)) => a.octets()[..3] == b.octets()[..3],
        _ => a == b,
    }
}

/// `Password_OnCommand`.
fn password_on_command(w: &mut World, e: &mut CommandEventArgs) {
    let from = e.mobile;
    let Some(acct) = w.mobile(from).and_then(|m| m.account) else {
        return;
    };
    let Some(first) = account(w, acct).and_then(|a| a.login_ips.first().copied()) else {
        return;
    };
    let Some(ns) = w.mobile(from).and_then(|m| m.net_state) else {
        return;
    };

    let say = |w: &mut World, t: &str| muo_server::commands::send_message(w, from, t);
    if e.length() == 0 {
        say(w, "You must specify the new password.");
        return;
    }
    if e.length() == 1 {
        say(
            w,
            "To prevent potential typing mistakes, you must type the password twice. Use the format:",
        );
        say(w, "Password \"(newPassword)\" \"(repeated)\"");
        return;
    }
    let pass = e.get_string(0);
    let pass2 = e.get_string(1);
    if pass != pass2 {
        say(w, "The passwords do not match.");
        return;
    }
    if !pass.chars().all(|c| (0x20..0x7F).contains(&(c as u32))) {
        say(w, "That is not a valid password.");
        return;
    }
    let Some(addr) = w.net_state(ns).and_then(|d| d.address) else {
        return;
    };
    if match_class_c(first, addr) {
        // Confirmed from the callback: off-loop the write has not landed yet here.
        super::password_worker::set_password(
            w,
            acct,
            &pass,
            Some(Box::new(move |w, _| {
                muo_server::commands::send_message(
                    w,
                    from,
                    "The password to your account has changed.",
                );
            })),
        );
    } else if let Some(f) = crate::hooks::hooks(w).and_then(|h| h.page_account_password) {
        let text = format!(
            "[Automated: Change Password]<br>Desired password: {pass}<br>Current IP address: {addr}<br>Account IP address: {first}"
        );
        f(w, from, &text);
    } else {
        say(
            w,
            "Your IP address does not match that which created this account.",
        );
    }
}

/// The packets' view of `IAccount`: `a[i]` raw names and `Limit`.
pub fn account_chars(w: &mut World, id: AccountId) -> AccountChars {
    let len = account(w, id).map_or(0, |a| a.mobiles.len());
    let mut slots = Vec::with_capacity(len);
    for i in 0..len as i32 {
        slots.push(mobile_at(w, id, i).map(|m| CharSlot {
            raw_name: muo_server::mobile::part1::props_b::raw_name(w, m),
        }));
    }
    AccountChars {
        slots,
        limit: super::limit(w),
    }
}

/// `AccountHandler.DeleteRequest`.
pub fn delete_request(w: &mut World, ns: NetStateId, index: i32) {
    let acct = w
        .net_state(ns)
        .and_then(|d| d.account.clone())
        .and_then(|n| get_account(w, &n));
    let Some(acct) = acct else {
        w.disconnect(
            ns,
            "Attempted to delete a character but the account could not be found.",
        );
        return;
    };
    let desc = w
        .net_state(ns)
        .map(|d| d.description.clone())
        .unwrap_or_default();

    let length = account(w, acct).map_or(0, |a| a.length());
    let res;
    if index < 0 || index >= length {
        res = DeleteResultType::BadRequest;
    } else {
        let m = mobile_at(w, acct, index);
        let now = w.now;
        let restrict = state(w).restrict_deletion;
        let level = account(w, acct).map_or(AccessLevel::Player, |a| a.access_level);
        match m {
            None => res = DeleteResultType::CharNotExist,
            Some(m) if w.mobile(m).is_some_and(|mm| mm.net_state.is_some()) => {
                res = DeleteResultType::CharBeingPlayed;
            }
            Some(m)
                if level == AccessLevel::Player
                    && restrict
                    && w.mobile(m).is_some_and(|mm| {
                        now < mm.created + muo_primitives::TimeSpan::from_days(DELETE_DELAY_DAYS)
                    }) =>
            {
                res = DeleteResultType::CharTooYoung;
            }
            // Don't need to check current location, if netstate is null, they're logged out
            Some(m)
                if w.mobile(m)
                    .is_some_and(|mm| mm.access_level == AccessLevel::Player)
                    && in_jail(w, m) =>
            {
                res = DeleteResultType::BadRequest;
            }
            Some(m) => {
                let serial = w.mobile(m).map(|mm| mm.serial).unwrap_or_default();
                info!("Client: {desc}: Deleting character {index} ({serial})");
                let text = format!(
                    "Character #{} {} deleted by {}",
                    index + 1,
                    mobile_to_string(w, m),
                    desc
                );
                let comment = AccountComment::new(w, "System", &text);
                if let Some(a) = account_mut(w, acct) {
                    a.comments.push(comment);
                }
                if let Some(ty) = w.mobile(m).map(|mm| mm.ty) {
                    (ty.delete)(w, m);
                }
                let chars = account_chars(w, acct);
                send_character_list_update(w, ns, Some(&chars));
                return;
            }
        }
    }
    send_character_delete_result(w, ns, res);
    let chars = account_chars(w, acct);
    send_character_list_update(w, ns, Some(&chars));
}

/// `Region.Find(m.LogoutLocation, m.LogoutMap).IsPartOf<JailRegion>()`.
fn in_jail(w: &World, m: MobileId) -> bool {
    w.resources
        .get::<AccountHooks>()
        .is_some_and(|h| (h.in_jail_region)(w, m))
}

/// `CanCreate(ip)`.
pub fn can_create(w: &mut World, ip: IpAddr) -> bool {
    let max = state(w).max_accounts_per_ip;
    ip_table(w).get(&ip).is_none_or(|n| *n < max)
}

/// `IsValidUsername`.
pub fn is_valid_username(username: &str) -> bool {
    !username.is_empty()
        // Usernames must not start with a space, end with a space, or end with a period
        && !username.starts_with(' ')
        && !username.ends_with(' ')
        && !username.ends_with('.')
        && !username.contains(['<', '>', ':', '"', '/', '\\', '|', '?', '*'])
}

/// `IsValidPassword`.
pub fn is_valid_password(password: &str) -> bool {
    !password.is_empty()
}

fn create_account_for(
    w: &mut World,
    ns: NetStateId,
    address: IpAddr,
    username: &str,
    password: &str,
) -> Option<AccountId> {
    if !is_valid_username(username) || !is_valid_password(password) {
        return None;
    }
    let desc = w
        .net_state(ns)
        .map(|d| d.description.clone())
        .unwrap_or_default();
    if !can_create(w, address) {
        let max = state(w).max_accounts_per_ip;
        info!(
            "Login: {desc} Account '{username}' not created, ip already has {max} account{}.",
            if max == 1 { "" } else { "s" }
        );
        return None;
    }
    info!("Login: {desc}: Creating new account '{username}'");
    Some(create_account(w, username, password))
}

fn describe(w: &World, ns: NetStateId) -> String {
    w.net_state(ns)
        .map(|d| d.description.clone())
        .unwrap_or_default()
}

fn set_state_account(w: &mut World, ns: NetStateId, acct: Option<AccountId>) {
    let name = acct
        .and_then(|id| account(w, id))
        .map(|a| a.username.clone());
    if let Some(d) = w.net_state_mut(ns) {
        d.account = name;
    }
}

/// `EventSink_AccountLogin`.
pub fn on_account_login(w: &mut World, e: &mut AccountLoginEventArgs) {
    let ns = NetStateId(e.net_state as u32);
    let un = e.username.clone();
    let pw = e.password.clone();
    e.accepted = false;

    let Some(address) = w.net_state(ns).and_then(|d| d.address) else {
        e.reject_reason = ALRReason::BadComm as u8;
        return;
    };

    let Some(acct) = get_account(w, &un) else {
        // To prevent someone from making an account of just '' or a bunch of meaningless spaces
        if state(w).auto_account_creation && !un.trim().is_empty() {
            let acct = create_account_for(w, ns, address, &un, &pw);
            set_state_account(w, ns, acct);
            e.accepted = acct.is_some_and(|id| check_access(w, id, address));
            if !e.accepted {
                e.reject_reason = ALRReason::BadComm as u8;
            }
        } else {
            info!("Login: {} Invalid username '{un}'", describe(w, ns));
            e.reject_reason = ALRReason::Invalid as u8;
        }
        return;
    };

    if !has_access(w, acct, address) {
        info!("Login: {} Access denied for '{un}'", describe(w, ns));
        e.reject_reason = if lockdown_level(w) > AccessLevel::Player {
            ALRReason::BadComm as u8
        } else {
            ALRReason::BadPass as u8
        };
    } else {
        handle_password_check(w, e, ns, acct, &pw, address);
    }
}

fn handle_password_check(
    w: &mut World,
    e: &mut AccountLoginEventArgs,
    ns: NetStateId,
    acct: AccountId,
    pw: &str,
    address: IpAddr,
) {
    let username = account(w, acct)
        .map(|a| a.username.clone())
        .unwrap_or_default();
    match dispatch_password_check(w, ns, acct, pw) {
        PasswordCheckDispatch::Deferred => {
            e.deferred = true;
            return;
        }
        PasswordCheckDispatch::Saturated => {
            // Reject rather than verify inline: steering work back onto the loop is what a flood wants.
            tracing::warn!(
                "Login: {} Password verification queue full, rejecting '{username}'",
                describe(w, ns)
            );
            e.reject_reason = ALRReason::BadComm as u8;
            return;
        }
        PasswordCheckDispatch::Inline => {}
    }
    if !check_password(w, acct, pw) {
        info!(
            "Login: {} Invalid password for '{username}'",
            describe(w, ns)
        );
        e.reject_reason = ALRReason::BadPass as u8;
        return;
    }
    apply_verified_login(w, e, ns, acct, address);
}

/// `PasswordCheckDispatch`.
enum PasswordCheckDispatch {
    /// Verify on the loop.
    Inline,
    /// Handed to the worker; no verdict yet.
    Deferred,
    /// The queue is full.
    Saturated,
}

/// `DispatchPasswordCheck`: hands the check to the worker, whatever algorithm it uses. Every protection is safe off
/// the loop, and `AccountSecurity.Configure` refuses anything below SHA2 as the configured algorithm, so a cheap
/// digest only appears as a stored hash awaiting migration (whose upgrade hash dominates the job).
fn dispatch_password_check(
    w: &mut World,
    ns: NetStateId,
    acct: AccountId,
    pw: &str,
) -> PasswordCheckDispatch {
    use super::password_worker::{self as worker, PasswordJob};
    if !worker::enabled(w) {
        return PasswordCheckDispatch::Inline;
    }
    let current = current_algorithm(w);
    let Some(a) = account(w, acct) else {
        return PasswordCheckDispatch::Inline;
    };
    let stored = a.password.clone().unwrap_or_default();
    let needs_upgrade =
        a.password_algorithm != current || super::security::needs_rehash(current, &stored);
    let job = PasswordJob {
        account: acct,
        ns: Some(ns),
        stored_hash: a.password.clone(),
        stored_algorithm: a.password_algorithm,
        verify_phrase: Some(super::security::derive_phrase(
            a.password_algorithm,
            &a.username,
            pw,
        )),
        hash_phrase: needs_upgrade
            .then(|| super::security::derive_phrase(current, &a.username, pw)),
        target_algorithm: current,
    };
    let queued = worker::try_enqueue(
        w,
        job,
        Box::new(|w, j, outcome| {
            if let Some(ns) = j.ns {
                complete_deferred_account_login(w, ns, j.account, outcome.verified);
            }
        }),
    );
    if queued {
        PasswordCheckDispatch::Deferred
    } else {
        PasswordCheckDispatch::Saturated
    }
}

/// `CompleteDeferredAccountLogin(state, acct, verified)`: resumes a login whose password check ran on the
/// verification thread.
pub fn complete_deferred_account_login(
    w: &mut World,
    ns: NetStateId,
    acct: AccountId,
    verified: bool,
) {
    let username = account(w, acct)
        .map(|a| a.username.clone())
        .unwrap_or_default();
    let mut e = AccountLoginEventArgs::new(ns.0 as usize, username.clone(), String::new());
    if verified {
        match w.net_state(ns).and_then(|d| d.address) {
            Some(address) => apply_verified_login(w, &mut e, ns, acct, address),
            None => return,
        }
    } else {
        info!(
            "Login: {} Invalid password for '{username}'",
            describe(w, ns)
        );
        e.reject_reason = ALRReason::BadPass as u8;
    }
    crate::network::incoming_account::complete_account_login_deferred(
        w,
        ns,
        e.accepted,
        e.reject_reason,
    );
}

/// `ApplyVerifiedLogin`: everything after the password is known good.
fn apply_verified_login(
    w: &mut World,
    e: &mut AccountLoginEventArgs,
    ns: NetStateId,
    acct: AccountId,
    address: IpAddr,
) {
    let username = account(w, acct)
        .map(|a| a.username.clone())
        .unwrap_or_default();
    if is_banned(w, acct) {
        info!("Login: {} Banned account '{username}'", describe(w, ns));
        e.reject_reason = ALRReason::Blocked as u8;
        return;
    }
    info!(
        "Login: {} Valid credentials for '{username}'",
        describe(w, ns)
    );
    set_state_account(w, ns, Some(acct));
    e.accepted = true;
    log_access(w, acct, address);
    crate::network::login_allowlist::record_login(w, Some(address));
}

/// `OnGameServerLogin`.
pub fn on_game_server_login(w: &mut World, e: &mut GameLoginEventArgs) {
    let ns = e.state;
    let un = e.username.clone();
    let Some(address) = w.net_state(ns).and_then(|d| d.address) else {
        e.accepted = false;
        return;
    };
    let desc = describe(w, ns);
    let Some(acct) = get_account(w, &un) else {
        e.accepted = false;
        return;
    };
    if !has_access(w, acct, address) {
        info!("Login: {desc} Access denied for '{un}'");
        e.accepted = false;
    }
    // The auth id was only issued after the account login packet verified this password, so re-deriving the hash
    // costs a second Argon2 verify to answer the same question.
    else if !e.pre_authenticated && !check_password(w, acct, &e.password) {
        info!("Login: {desc} Invalid password for '{un}'");
        e.accepted = false;
    } else if is_banned(w, acct) {
        info!("Login: {desc} Banned account '{un}'");
        e.accepted = false;
    } else {
        log_access(w, acct, address);
        crate::network::login_allowlist::record_login(w, Some(address));
        info!("Login: {desc} Account '{un}' at character list");
        set_state_account(w, ns, Some(acct));
        e.accepted = true;
        e.city_info = crate::login::character_creation::get_starting_cities(w);
    }
}

/// `CheckAccount(mobCheck, accCheck)`: is `acc_check` a character of the same account as... `mob_check`.
pub fn check_account(w: &mut World, mob_check: MobileId, acc_check: Option<MobileId>) -> bool {
    let Some(a) = acc_check.and_then(|m| w.mobile(m)).and_then(|m| m.account) else {
        return false;
    };
    let len = account(w, a).map_or(0, |x| x.length());
    (0..len).any(|i| mobile_at(w, a, i) == Some(mob_check))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usernames() {
        assert!(is_valid_username("admin"));
        assert!(!is_valid_username(""));
        assert!(!is_valid_username(" a"));
        assert!(!is_valid_username("a "));
        assert!(!is_valid_username("a."));
        assert!(!is_valid_username("a/b"));
        assert!(is_valid_username("a b"));
    }

    /// A login handed to the password worker has no verdict yet (`Deferred`); the reply is sent when it lands.
    #[test]
    fn a_deferred_login_completes_when_the_worker_answers() {
        use muo_server::net_state::{NetOut, NetStateData};
        let mut w = World::new();
        super::super::password_worker::set_enabled_for_testing(&mut w, true);
        let acct = create_account(&mut w, "bob", "secret");
        let ns = NetStateId(5);
        let mut d = NetStateData::new(0);
        d.address = Some("127.0.0.1".parse().unwrap());
        w.net_states.insert(ns, d);

        let run = |w: &mut World, pw: &str| {
            let mut e = AccountLoginEventArgs::new(5, "bob".into(), pw.into());
            on_account_login(w, &mut e);
            assert!(e.deferred, "the check is on the worker");
            let ctx = w.core.loop_context.clone();
            let end = std::time::Instant::now() + std::time::Duration::from_secs(20);
            while w.net_out.is_empty() && std::time::Instant::now() < end {
                ctx.execute_tasks(w);
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
            std::mem::take(&mut w.net_out)
        };

        // a wrong password: 0x82 with BadPass (3), then a disconnect
        let out = run(&mut w, "wrong");
        assert!(matches!(&out[0], NetOut::SetAccount(n, None) if *n == ns));
        assert!(matches!(&out[1], NetOut::Send(n, b) if *n == ns && b[..] == [0x82, 3]));
        assert!(matches!(&out[2], NetOut::Disconnect(n, _) if *n == ns));

        // the right one: the account is set on the transport, then the server list ack (0xA8)
        w.net_states.get_mut(&ns).unwrap().cannot_send = false;
        let out = run(&mut w, "secret");
        assert!(matches!(&out[0], NetOut::SetAccount(n, Some(a)) if *n == ns && a == "bob"));
        assert!(matches!(&out[1], NetOut::Send(n, b) if *n == ns && b[0] == 0xA8));
        assert_eq!(w.net_state(ns).unwrap().account.as_deref(), Some("bob"));
        let _ = acct;
        super::super::password_worker::stop(&mut w);
    }

    // A pre-authenticated game login skips the password check; one without pre-authentication still checks it.
    fn game_login_world() -> (World, NetStateId) {
        use muo_server::net_state::NetStateData;
        let mut w = World::new();
        create_account(&mut w, "authid-preauth-user", "hunter2");
        let ns = NetStateId(9);
        let mut d = NetStateData::new(0);
        d.address = Some("127.0.0.1".parse().unwrap());
        w.net_states.insert(ns, d);
        (w, ns)
    }

    #[test]
    fn pre_authenticated_game_login_skips_the_password_check() {
        let (mut w, ns) = game_login_world();
        // a wrong password is accepted only because the auth id already vouched for the account
        let mut e = GameLoginEventArgs::new(
            ns,
            "authid-preauth-user".into(),
            "wrong-password".into(),
            true,
        );
        on_game_server_login(&mut w, &mut e);
        assert!(e.accepted);
    }

    #[test]
    fn game_login_without_pre_authentication_still_checks_the_password() {
        let (mut w, ns) = game_login_world();
        let mut wrong = GameLoginEventArgs::new(
            ns,
            "authid-preauth-user".into(),
            "wrong-password".into(),
            false,
        );
        on_game_server_login(&mut w, &mut wrong);
        assert!(!wrong.accepted);

        let mut right =
            GameLoginEventArgs::new(ns, "authid-preauth-user".into(), "hunter2".into(), false);
        on_game_server_login(&mut w, &mut right);
        assert!(right.accepted);
    }
}
