//! Email: SMTP over implicit TLS (`lettre` with rustls and the RustCrypto
//! provider, as `crate::http_tls`), retried with doubling waits on its own thread. `SendQueueEmail` builds its text
//! in the help system (`muo_engines::help`), which owns `PageEntry`; it calls [`send_text`].

use std::path::Path;
use std::sync::mpsc;
use std::time::Duration;

use lettre::message::{Attachment, Mailbox, MultiPart, SinglePart, header::ContentType};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use muo_server::World;
use tracing::{error, info, warn};

use crate::configuration::email_configuration::{EmailConfiguration, MailboxAddress};

/// How long `SendCrashEmail` holds up the exit.
const CRASH_EMAIL_TIMEOUT: Duration = Duration::from_secs(30);
/// A retry wait stops doubling at an hour.
const MAX_RETRY_DELAY_SECONDS: i32 = 3600;

/// `RetryDelays(attempts, firstDelaySeconds)`: the waits between send attempts, doubling, none after the last.
pub fn retry_delays(attempts: i32, first_delay_seconds: i32) -> Vec<Duration> {
    let n = (attempts.max(1) - 1) as usize;
    let mut seconds = first_delay_seconds.clamp(0, MAX_RETRY_DELAY_SECONDS);
    let mut delays = Vec::with_capacity(n);
    for _ in 0..n {
        delays.push(Duration::from_secs(seconds as u64));
        seconds = (seconds * 2).min(MAX_RETRY_DELAY_SECONDS);
    }
    delays
}

/// What a send needs from `EmailConfiguration`, copied so the thread owns it.
#[derive(Clone)]
struct Smtp {
    server: String,
    port: u16,
    user: String,
    password: String,
    retries: i32,
    retry_delay: i32,
}

fn mailbox(m: &MailboxAddress) -> Result<Mailbox, String> {
    let address = m
        .address
        .parse()
        .map_err(|e| format!("{}: {e}", m.address))?;
    Ok(Mailbox::new(
        (!m.name.is_empty()).then(|| m.name.clone()),
        address,
    ))
}

/// The SMTP settings and the sender; `None` when `EmailConfiguration.EmailEnabled` is off.
fn config(w: &World) -> Option<(Smtp, MailboxAddress)> {
    let c = w
        .resources
        .get::<EmailConfiguration>()
        .filter(|c| c.email_enabled)?;
    Some((
        Smtp {
            server: c.email_server.clone(),
            port: c.email_port as u16,
            user: c.email_server_username.clone(),
            password: c.email_server_password.clone(),
            retries: c.email_send_retry_count,
            retry_delay: c.email_send_retry_delay,
        },
        c.from_address.clone(),
    ))
}

fn send_once(smtp: &Smtp, message: &Message) -> Result<(), String> {
    // lettre wants a process-wide provider when none is compiled in; losing the race to another installer is fine.
    let _ = rustls::crypto::CryptoProvider::install_default(rustls_rustcrypto::provider());
    let mailer = SmtpTransport::relay(&smtp.server)
        .map_err(|e| e.to_string())?
        .port(smtp.port)
        .credentials(Credentials::new(smtp.user.clone(), smtp.password.clone()))
        .timeout(Some(Duration::from_secs(30)))
        .build();
    mailer.send(message).map(|_| ()).map_err(|e| e.to_string())
}

/// `SendAsync(message)`: the attempts, on the calling thread.
fn send_with_retries(smtp: &Smtp, message: &Message, subject: &str) {
    let attempts = smtp.retries;
    let delays = retry_delays(attempts, smtp.retry_delay);
    let mut last = String::new();
    for i in 0..attempts.max(0) as usize {
        match send_once(smtp, message) {
            Ok(()) => return,
            Err(e) => {
                if i == 0 {
                    warn!("Sending email {subject} failed: {e}");
                }
                last = e;
                if let Some(d) = delays.get(i) {
                    std::thread::sleep(*d);
                }
            }
        }
    }
    error!("Email {subject} not sent after {attempts} attempts: {last}");
}

fn build(
    from: &MailboxAddress,
    to: &MailboxAddress,
    subject: &str,
    text: &str,
    attachment: Option<&Path>,
) -> Result<Message, String> {
    let b = Message::builder()
        .from(mailbox(from)?)
        .to(mailbox(to)?)
        .subject(subject);
    let body = SinglePart::plain(text.to_owned());
    match attachment {
        None => b.singlepart(body),
        Some(p) => {
            let data = std::fs::read(p).map_err(|e| format!("{}: {e}", p.display()))?;
            let name = p
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            let part = Attachment::new(name).body(
                data,
                ContentType::parse("application/octet-stream").map_err(|e| e.to_string())?,
            );
            b.multipart(MultiPart::mixed().singlepart(body).singlepart(part))
        }
    }
    .map_err(|e| e.to_string())
}

/// `_ = SendAsync(message)` for a text mail from `FromAddress` to the address `to` picks. Nothing when email is off.
pub fn send_text(
    w: &World,
    to: fn(&EmailConfiguration) -> MailboxAddress,
    subject: &str,
    text: &str,
) {
    let Some((smtp, from)) = config(w) else {
        return;
    };
    let to = to(w
        .resources
        .get::<EmailConfiguration>()
        .expect("checked by config"));
    let message = match build(&from, &to, subject, text, None) {
        Ok(m) => m,
        Err(e) => return error!("Email {subject} not built: {e}"),
    };
    let subject = subject.to_owned();
    std::thread::spawn(move || send_with_retries(&smtp, &message, &subject));
}

/// `Email.SendCrashEmail(filePath)`: blocks up to the crash timeout; a send left running dies with the exit.
pub fn send_crash_email(w: &World, file_path: &Path) {
    let Some((smtp, from)) = config(w) else {
        return;
    };
    let to = w
        .resources
        .get::<EmailConfiguration>()
        .expect("checked by config")
        .crash_address
        .clone();
    let subject = "Automated Server Crash Report";
    let text = "Automated Server Crash Report. See attachment for details.";
    let message = match build(&from, &to, subject, text, Some(file_path)) {
        Ok(m) => m,
        Err(e) => return error!("Sending crash email failed: {e}"),
    };
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        send_with_retries(&smtp, &message, subject);
        let _ = tx.send(());
    });
    if rx.recv_timeout(CRASH_EMAIL_TIMEOUT).is_err() {
        warn!("Crash email not sent after {CRASH_EMAIL_TIMEOUT:?}, giving up");
    } else {
        info!("Crash email finished");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retry_delays_double_and_cap() {
        let s = |v: &[u64]| {
            v.iter()
                .map(|x| Duration::from_secs(*x))
                .collect::<Vec<_>>()
        };
        assert_eq!(retry_delays(5, 3), s(&[3, 6, 12, 24]));
        assert_eq!(retry_delays(1, 3), s(&[]));
        assert_eq!(retry_delays(0, 3), s(&[]));
        assert_eq!(retry_delays(4, 2000), s(&[2000, 3600, 3600]));
        assert_eq!(retry_delays(3, -5), s(&[0, 0]));
    }

    // Email stays disabled: a fresh World has no
    // EmailConfiguration, which `config` treats as off).
    #[test]
    fn send_crash_email_does_nothing_while_email_is_disabled() {
        let w = World::new();
        let report = std::env::temp_dir()
            .join("crash-email-test-missing")
            .join("Crash.log");
        send_crash_email(&w, &report);
    }

    #[test]
    fn send_text_does_nothing_while_email_is_disabled() {
        let w = World::new();
        send_text(&w, |c| c.crash_address.clone(), "Other", "text");
    }

    #[test]
    fn retry_waits_double_from_the_configured_delay_and_stop_at_the_last_attempt() {
        let s = |v: &[u64]| {
            v.iter()
                .map(|x| Duration::from_secs(*x))
                .collect::<Vec<_>>()
        };
        assert_eq!(retry_delays(5, 3), s(&[3, 6, 12, 24]));
    }

    #[test]
    fn fewer_than_two_attempts_never_wait() {
        for attempts in [i32::MIN, 0, 1] {
            assert!(retry_delays(attempts, 3).is_empty(), "{attempts}");
        }
    }

    #[test]
    fn retry_waits_never_overflow_and_stop_doubling_at_an_hour() {
        for first in [8, i32::MAX] {
            let delays = retry_delays(64, first);
            assert!(
                delays.iter().all(|d| *d <= Duration::from_secs(3600)),
                "{first}"
            );
            assert_eq!(*delays.last().unwrap(), Duration::from_secs(3600));
        }
    }
}
