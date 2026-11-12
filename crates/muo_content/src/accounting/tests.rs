use muo_io::{GenericReader, SaveReader, SaveWriter};
use muo_server::World;
use muo_server::mobile::MOBILE_BASE;
use muo_server::persistence::{LoadContext, Persistence, ScriptedPrompt};

use super::persistence::AccountsPersistence;
use super::*;

/// A record from an existing `Saves/Accounts/Accounts.bin` (account `admin`, Owner, one character
/// 0x00000001, one login address 127.0.0.1).
const STORED_RECORD: &str = "06010561646d696e0501e100246172676f6e32696424763d3139246d3d31363338342c743d312c703d312434474e734b6b723139732f455968363942456f6b3767244461437a61312b574475694877446841446f757a7376446a77354d4f5a78702f4a47624f2f4a4a6761304106000000000ac6b238623adf0800000000000000000701000000ffffffffffffffffffffffffffffffffffffffffffffffff000001047f000001c87a229a2400000000";

fn hex(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}

fn world_with_a_mobile() -> World {
    let mut w = World::new();
    let m = w.new_mobile(&MOBILE_BASE);
    assert_eq!(m.0.value(), 1);
    w
}

#[test]
fn a_stored_record_round_trips_byte_for_byte() {
    let w = world_with_a_mobile();
    let bytes = hex(STORED_RECORD);
    let mut r = SaveReader::new(&bytes);
    let mut a = Account::blank(Serial::new(1), DateTime::MIN);
    a.deserialize(&w, &mut r).unwrap();
    assert_eq!(r.position(), bytes.len() as u64);
    assert_eq!(a.username, "admin");
    assert_eq!(a.password_algorithm, PasswordProtectionAlgorithm::Argon2);
    assert_eq!(a.access_level, AccessLevel::Owner);
    assert_eq!(a.mobiles.len(), 7);
    assert!(a.mobiles[0].is_some() && a.mobiles[1].is_none());
    assert_eq!(a.login_ips, vec!["127.0.0.1".parse::<IpAddr>().unwrap()]);
    assert!(a.email.is_none());

    let mut out = SaveWriter::new(true);
    a.serialize(&w, &mut out);
    assert_eq!(out.as_slice(), &bytes[..]);
}

#[test]
fn stored_argon2_hash_in_the_save_still_verifies_as_a_hash_of_the_right_shape() {
    let w = world_with_a_mobile();
    let bytes = hex(STORED_RECORD);
    let mut a = Account::blank(Serial::new(1), DateTime::MIN);
    a.deserialize(&w, &mut SaveReader::new(&bytes)).unwrap();
    let h = a.password.as_deref().unwrap();
    assert!(!security::needs_rehash(a.password_algorithm, h));
    assert!(!security::validate_password(
        a.password_algorithm,
        h,
        "wrong"
    ));
}

#[test]
fn accounts_are_found_without_regard_to_case_and_a_new_password_is_argon2() {
    let mut w = World::new();
    let id = create_account(&mut w, "Probe", "probe-pass-1");
    assert_eq!(get_account(&w, "PROBE"), Some(id));
    assert_eq!(
        account(&w, id).unwrap().password_algorithm,
        PasswordProtectionAlgorithm::Argon2
    );
    assert!(check_password(&mut w, id, "probe-pass-1"));
    assert!(!check_password(&mut w, id, "probe-pass-2"));
    assert!(!try_set_username(&mut w, id, "probe"));
    assert!(try_set_username(&mut w, id, "other"));
    assert_eq!(get_account(&w, "probe"), None);
    assert_eq!(get_account(&w, "OTHER"), Some(id));
}

#[test]
fn an_old_hash_is_upgraded_by_a_good_login() {
    let mut w = World::new();
    let id = create_account(&mut w, "legacy", "pw");
    {
        let a = account_mut(&mut w, id).unwrap();
        a.password_algorithm = PasswordProtectionAlgorithm::Sha2;
        let phrase = security::derive_phrase(PasswordProtectionAlgorithm::Sha2, "legacy", "pw");
        a.password = security::encrypt_password(PasswordProtectionAlgorithm::Sha2, &phrase);
    }
    assert!(!check_password(&mut w, id, "bad"));
    assert_eq!(
        account(&w, id).unwrap().password_algorithm,
        PasswordProtectionAlgorithm::Sha2
    );
    assert!(check_password(&mut w, id, "pw"));
    assert_eq!(
        account(&w, id).unwrap().password_algorithm,
        PasswordProtectionAlgorithm::Argon2
    );
    assert!(check_password(&mut w, id, "pw"));
}

#[test]
fn flags_tags_and_bans() {
    let mut w = World::new();
    w.now = DateTime::from_ymd(2026, 1, 1).unwrap();
    let id = create_account(&mut w, "u", "p");
    let a = account_mut(&mut w, id).unwrap();
    a.set_tag("x", "1");
    a.set_tag("x", "2");
    assert_eq!(a.tags.len(), 1);
    assert_eq!(a.get_tag("x"), Some("2"));
    a.remove_tag("x");
    assert!(a.tags.is_empty());

    set_banned(&mut w, id, true);
    assert!(is_banned(&mut w, id));
    // an expired ban clears itself
    let now = w.now;
    let a = account_mut(&mut w, id).unwrap();
    a.set_ban_tags(None, now - TimeSpan::from_hours(2), TimeSpan::from_hours(1));
    assert!(!is_banned(&mut w, id));
    assert!(!account(&w, id).unwrap().get_flag(0));
    set_banned(&mut w, id, true);
    let a = account_mut(&mut w, id).unwrap();
    a.set_tag("BanTime", &now.to_iso_string());
    a.set_tag("BanDuration", "Infinite");
    assert!(is_banned(&mut w, id));
}

#[test]
fn a_save_loads_back_the_same() {
    let dir = std::env::temp_dir().join(format!("muo-accounts-test-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);

    let mut w = World::new();
    let id = create_account(&mut w, "Admin", "secret");
    log_access(&mut w, id, "10.0.0.5".parse().unwrap());
    let c = AccountComment::new(&w, "System", "hello");
    account_mut(&mut w, id).unwrap().comments.push(c);
    account_mut(&mut w, id).unwrap().email = Some("a@b.c".into());
    let snap = AccountsPersistence::default().serialize(&w).unwrap();
    snap.write(&dir).unwrap();

    let mut w2 = World::new();
    let mut p = AccountsPersistence::default();
    let types_db = Default::default();
    let mut prompt = ScriptedPrompt::default();
    let mut ctx = LoadContext {
        save_path: &dir,
        types_db: &types_db,
        prompt: &mut prompt,
    };
    p.deserialize_indexes(&mut w2, &mut ctx).unwrap();
    p.deserialize(&mut w2, &mut ctx).unwrap();
    let id2 = get_account(&w2, "admin").expect("loaded");
    let a = account(&w2, id2).unwrap();
    assert_eq!(a.username, "Admin");
    assert_eq!(a.email.as_deref(), Some("a@b.c"));
    assert_eq!(a.comments[0].content(), "hello");
    assert_eq!(a.login_ips.len(), 1);
    assert!(check_password(&mut w2, id2, "secret"));

    // and writes the same bytes again
    let snap2 = p.serialize(&w2).unwrap();
    assert_eq!(snap.bin, snap2.bin);
    let _ = std::fs::remove_dir_all(&dir);
}

// ---- password worker, account passwords, account handler ------------------------------------------------------------

mod behavior_tests {
    use super::super::handler;
    use super::super::password_worker::{self as worker, PasswordJob};
    use super::super::security::{self, PasswordProtectionAlgorithm as Alg};
    use super::*;
    use std::cell::Cell;
    use std::rc::Rc;
    use std::time::{Duration, Instant};

    const PASSWORD: &str = "hunter2";
    /// The shipping default before #2562: Argon2i, m=8192, t=3, p=1.
    const STALE_ARGON2: &str = "$argon2i$v=19$m=8192,t=3,p=1$LD1XJz7P3wQmIJ+Tu6ScgA$NO5hBABsHQ172C5nDO2X4gWnB4jDef3x6WhLdVE2LFw";

    fn world_with(alg: Alg) -> World {
        let mut w = World::new();
        handler::state(&mut w).current_algorithm = alg;
        worker::set_enabled_for_testing(&mut w, true);
        w
    }

    /// `Argon2TestParameters.CurrentPrefix`: the hash prefix the configured Argon2 writes, up to the salt.
    fn current_prefix() -> String {
        let hash = security::encrypt_password(Alg::Argon2, "prefix-probe").unwrap();
        let mut end = 0;
        for _ in 0..4 {
            end += hash[end..].find('$').unwrap() + 1;
        }
        hash[..end].to_owned()
    }

    /// `JobFor(account, submitted)`.
    fn job_for(w: &World, id: AccountId, submitted: &str) -> PasswordJob {
        let a = account(w, id).unwrap();
        let current = handler::current_algorithm(w);
        let needs_upgrade = a.password_algorithm != current
            || security::needs_rehash(current, a.password.as_deref().unwrap_or_default());
        PasswordJob {
            account: id,
            ns: None,
            stored_hash: a.password.clone(),
            stored_algorithm: a.password_algorithm,
            verify_phrase: Some(security::derive_phrase(
                a.password_algorithm,
                &a.username,
                submitted,
            )),
            hash_phrase: needs_upgrade
                .then(|| security::derive_phrase(current, &a.username, submitted)),
            target_algorithm: current,
        }
    }

    /// `PumpUntil(enqueue, complete)`.
    fn pump_until(w: &mut World, complete: impl Fn() -> bool) {
        let ctx = w.core.loop_context.clone();
        let end = Instant::now() + Duration::from_secs(20);
        while !complete() && Instant::now() < end {
            ctx.execute_tasks(w);
            std::thread::sleep(Duration::from_millis(5));
        }
        ctx.execute_tasks(w);
    }

    /// A job with no NetState attached -- an admin password change -- must run and call back.
    #[test]
    fn runs_a_job_that_has_no_connection_attached() {
        let mut w = world_with(Alg::Argon2);
        let id = create_account(&mut w, "offloop-no-netstate-user", PASSWORD);
        let applied = Rc::new(Cell::new(false));
        let a = applied.clone();
        let job = PasswordJob {
            account: id,
            ns: None,
            stored_hash: None,
            stored_algorithm: Alg::None,
            verify_phrase: None,
            hash_phrase: Some(security::derive_phrase(
                Alg::Argon2,
                "offloop-no-netstate-user",
                "a-queued-password",
            )),
            target_algorithm: Alg::Argon2,
        };
        assert!(worker::try_enqueue(
            &mut w,
            job,
            Box::new(move |_, _, o| a.set(o.hash.is_some()))
        ));
        let a = applied.clone();
        pump_until(&mut w, move || a.get());
        assert!(applied.get());
        assert!(check_password(&mut w, id, "a-queued-password"));
        worker::stop(&mut w);
    }

    #[test]
    fn verifies_the_correct_password() {
        let mut w = world_with(Alg::Argon2);
        let id = create_account(&mut w, "offloop-correct-user", PASSWORD);
        assert!(worker::compute(&job_for(&w, id, PASSWORD)).verified);
    }

    #[test]
    fn rejects_the_wrong_password() {
        let mut w = world_with(Alg::Argon2);
        let id = create_account(&mut w, "offloop-wrong-user", PASSWORD);
        let outcome = worker::compute(&job_for(&w, id, "not-the-password"));
        assert!(!outcome.verified);
        assert!(outcome.hash.is_none());
    }

    #[test]
    fn produces_no_upgrade_when_parameters_are_current() {
        let mut w = world_with(Alg::Argon2);
        let id = create_account(&mut w, "offloop-current-user", PASSWORD);
        let outcome = worker::compute(&job_for(&w, id, PASSWORD));
        assert!(outcome.verified);
        assert!(outcome.hash.is_none());
    }

    #[test]
    fn produces_an_upgrade_when_parameters_are_stale() {
        let mut w = world_with(Alg::Argon2);
        let id = create_account(&mut w, "offloop-stale-user", PASSWORD);
        account_mut(&mut w, id).unwrap().password = Some(STALE_ARGON2.to_owned());
        let outcome = worker::compute(&job_for(&w, id, PASSWORD));
        assert!(outcome.verified);
        let hash = outcome.hash.unwrap();
        assert!(hash.starts_with("$argon2id$v=19$"));
        assert!(hash.starts_with(&current_prefix()));
    }

    #[test]
    fn produces_no_upgrade_when_the_password_is_wrong() {
        let mut w = world_with(Alg::Argon2);
        let id = create_account(&mut w, "offloop-wrong-stale-user", PASSWORD);
        account_mut(&mut w, id).unwrap().password = Some(STALE_ARGON2.to_owned());
        let outcome = worker::compute(&job_for(&w, id, "not-the-password"));
        assert!(!outcome.verified);
        assert!(outcome.hash.is_none());
    }

    /// Writes apply in dispatch order: one worker drains FIFO, results return through the loop context in order.
    #[test]
    fn writes_apply_in_dispatch_order() {
        let mut w = world_with(Alg::Argon2);
        let id = create_account(&mut w, "offloop-two-writes-user", PASSWORD);
        let done = Rc::new(Cell::new(0));
        for i in 1..=2 {
            let d = done.clone();
            let job = PasswordJob {
                account: id,
                ns: None,
                stored_hash: None,
                stored_algorithm: account(&w, id).unwrap().password_algorithm,
                verify_phrase: None,
                hash_phrase: Some(security::derive_phrase(
                    Alg::Argon2,
                    "offloop-two-writes-user",
                    &format!("password-{i}"),
                )),
                target_algorithm: Alg::Argon2,
            };
            assert!(worker::try_enqueue(
                &mut w,
                job,
                Box::new(move |_, _, _| d.set(d.get() + 1))
            ));
        }
        let d = done.clone();
        pump_until(&mut w, move || d.get() >= 2);
        assert_eq!(done.get(), 2);
        assert!(check_password(&mut w, id, "password-2"));
        assert!(!check_password(&mut w, id, "password-1"));
        worker::stop(&mut w);
    }

    #[test]
    fn uses_the_username_salted_phrase_for_sha_accounts() {
        for algorithm in [Alg::Sha1, Alg::Sha2] {
            let mut w = world_with(algorithm);
            let username = format!("offloop-phrase-{algorithm:?}-user");
            let id = create_account(&mut w, &username, PASSWORD);
            let a = account(&w, id).unwrap();
            // Verification must use the algorithm the hash was stored under...
            assert_eq!(
                security::derive_phrase(a.password_algorithm, &a.username, PASSWORD),
                format!("{username}{PASSWORD}")
            );
            // ...and a rehash the one it is moving to. Swapping these is the #2562 lockout.
            handler::state(&mut w).current_algorithm = Alg::Argon2;
            let a = account(&w, id).unwrap();
            assert_eq!(
                security::derive_phrase(handler::current_algorithm(&w), &a.username, PASSWORD),
                PASSWORD
            );
        }
    }

    #[test]
    fn new_account_can_log_in() {
        for algorithm in [Alg::Sha1, Alg::Sha2, Alg::Pbkdf2, Alg::Argon2] {
            let mut w = world_with(algorithm);
            let id = create_account(&mut w, &format!("new-{algorithm:?}-user"), PASSWORD);
            assert_eq!(account(&w, id).unwrap().password_algorithm, algorithm);
            assert!(check_password(&mut w, id, PASSWORD));
            assert!(!check_password(&mut w, id, "wrong-password"));
        }
    }

    // SetPassword assigns PasswordAlgorithm before deriving the phrase from it.
    #[test]
    fn upgrading_algorithm_does_not_lock_the_account_out() {
        for from in [Alg::Sha1, Alg::Sha2, Alg::Pbkdf2] {
            let mut w = world_with(from);
            let id = create_account(&mut w, &format!("upgrade-{from:?}-user"), PASSWORD);
            assert!(check_password(&mut w, id, PASSWORD));

            handler::state(&mut w).current_algorithm = Alg::Argon2;
            assert!(check_password(&mut w, id, PASSWORD));
            assert_eq!(account(&w, id).unwrap().password_algorithm, Alg::Argon2);

            // Must verify against what the rehash wrote.
            assert!(check_password(&mut w, id, PASSWORD));
            assert!(!check_password(&mut w, id, "wrong-password"));
        }
    }

    #[test]
    fn stale_argon2_parameters_are_rehashed_on_login() {
        let mut w = world_with(Alg::Argon2);
        let id = create_account(&mut w, "stale-params-user", PASSWORD);
        account_mut(&mut w, id).unwrap().password = Some(STALE_ARGON2.to_owned());

        assert!(check_password(&mut w, id, PASSWORD));
        // The cost parameters are shard-tunable; the variant and version are not.
        let stored = account(&w, id).unwrap().password.clone().unwrap();
        assert!(stored.starts_with("$argon2id$v=19$"));
        assert!(stored.starts_with(&current_prefix()));

        // Already current: verifying again must not rewrite the hash.
        assert!(check_password(&mut w, id, PASSWORD));
        assert_eq!(account(&w, id).unwrap().password.clone().unwrap(), stored);
    }

    #[test]
    fn is_valid_username_validates_correctly() {
        for (username, expected) in [
            ("", false),                 // Empty username
            (" ", false),                // Single space
            (".", false),                // Single period
            ("Invalid<Char", false),     // Contains forbidden character
            ("EndsWithSpace ", false),   // Ends with space
            ("EndsWithPeriod.", false),  // Ends with period
            (" StartsWithSpace", false), // Starts with space
            ("ValidUser123", true),      // Standard Username
            ("Valid.User", true),        // Valid with period
            ("ValidUser!@#", true),      // Contains valid special characters
        ] {
            assert_eq!(
                handler::is_valid_username(username),
                expected,
                "{username:?}"
            );
        }
    }
}
