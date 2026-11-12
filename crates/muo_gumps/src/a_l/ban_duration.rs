//! `BanDurationGump`.

use muo_content::accounting::{AccountComment, Accounts};
use muo_content::gumps::base::{LegacyGumpLogic, LegacyGumpObj, RelayInfo};
use muo_content::gumps::legacy::LegacyLayout;
use muo_primitives::TimeSpan;
use muo_primitives::parse::{to_f64, to_i32};
use muo_server::World;
use muo_server::iaccount::AccountId;
use muo_server::mobile::AccessLevel;
use muo_server::net_state::NetStateId;

use super::common::{mobile_of, mobile_string, send};

pub struct BanDurationGump {
    list: Vec<AccountId>,
}

/// `new BanDurationGump(list)`.
pub fn new(list: Vec<AccountId>) -> LegacyGumpObj<BanDurationGump> {
    let mut g = LegacyGumpObj::new((640 - 500) / 2, (480 - 305) / 2, BanDurationGump { list });
    let (width, height) = (500, 305);
    let l = &mut g.layout;
    l.add_page(0);
    l.add_background(0, 0, width, height, 5054);
    l.add_html(
        10,
        10,
        width - 20,
        20,
        "<CENTER>Ban Duration</CENTER>",
        false,
        false,
    );
    add_button_labeled(l, 15, 45, 1, "Infinite");
    add_button_labeled(l, 15, 65, 2, "From D:H:M:S");
    add_input(l, 3, 0, "Days");
    add_input(l, 4, 1, "Hours");
    add_input(l, 5, 2, "Minutes");
    add_input(l, 6, 3, "Seconds");
    l.add_html(170, 45, 240, 20, "Comments:", false, false);
    add_text_field(l, 170, 65, 315, height - 80, 10);
    g
}

/// `new BanDurationGump(account)`.
pub fn new_one(a: AccountId) -> LegacyGumpObj<BanDurationGump> {
    new(vec![a])
}

fn add_button_labeled(l: &mut LegacyLayout, x: i32, y: i32, button_id: i32, text: &str) {
    l.add_button(x, y - 1, 4005, 4007, button_id);
    l.add_html(x + 35, y, 240, 20, text, false, false);
}

fn add_text_field(l: &mut LegacyLayout, x: i32, y: i32, width: i32, height: i32, index: i32) {
    l.add_background(x - 2, y - 2, width + 4, height + 4, 0x2486);
    l.add_text_entry(x + 2, y + 2, width - 4, height - 4, 0, index, "");
}

fn add_input(l: &mut LegacyLayout, bid: i32, idx: i32, name: &str) {
    let (x, y) = (15, 95 + idx * 50);
    add_button_labeled(l, x, y, bid, name);
    add_text_field(l, x + 35, y + 20, 100, 20, idx);
}

/// `new TimeSpan(d, h, m, s)`: None when the total is out of range.
fn time_span(d: i32, h: i32, m: i32, s: i32) -> Option<TimeSpan> {
    let secs = (i64::from(d) * 86400)
        .checked_add(i64::from(h) * 3600)?
        .checked_add(i64::from(m) * 60)?
        .checked_add(i64::from(s))?;
    secs.checked_mul(10_000_000).map(TimeSpan::from_ticks)
}

#[cfg(test)]
pub fn time_span_for_test(d: i32, h: i32, m: i32, s: i32) -> Option<TimeSpan> {
    time_span(d, h, m, s)
}

/// `TimeSpan.FromX(double)` throws for NaN and out-of-range values.
fn from_units(v: f64, ticks: f64) -> Option<TimeSpan> {
    let t = v * ticks;
    (!t.is_nan() && t >= i64::MIN as f64 && t <= i64::MAX as f64)
        .then(|| TimeSpan::from_ticks(t as i64))
}

impl LegacyGumpLogic for BanDurationGump {
    const TYPE_NAME: &'static str = "Server.Gumps.BanDurationGump";

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let Some(from) = mobile_of(w, sender) else {
            return;
        };
        if w.mobile(from)
            .is_none_or(|m| m.access_level < AccessLevel::Administrator)
        {
            return;
        }
        let from_str = mobile_string(w, from);

        let t = |i: i32| info.get_text_entry(i).map(str::to_owned);
        let (d, h, m, s) = (t(0), t(1), t(2), t(3));
        let c = t(10);

        let duration = match info.button_id {
            0 => {
                let accounts = w.resources.get_mut::<Accounts>();
                for id in &self.list {
                    if let Some(a) = accounts.get_mut(*id) {
                        a.set_unspecified_ban(Some(&from_str));
                    }
                }
                muo_content::mx::send_message(w, from, "Duration unspecified.");
                return;
            }
            1 => Some(TimeSpan::MAX),
            2 => match (&d, &h, &m, &s) {
                (Some(d), Some(h), Some(m), Some(s)) => {
                    time_span(to_i32(d), to_i32(h), to_i32(m), to_i32(s))
                }
                _ => None,
            },
            3 => d
                .as_deref()
                .and_then(|v| from_units(to_f64(v), 864_000_000_000.0)),
            4 => h
                .as_deref()
                .and_then(|v| from_units(to_f64(v), 36_000_000_000.0)),
            5 => m
                .as_deref()
                .and_then(|v| from_units(to_f64(v), 600_000_000.0)),
            6 => s
                .as_deref()
                .and_then(|v| from_units(to_f64(v), 10_000_000.0)),
            _ => return,
        };

        if let Some(duration) = duration {
            // c?.Trim().DefaultIfNullOrEmpty(null)
            let comment = c.map(|c| c.trim().to_owned()).filter(|c| !c.is_empty());
            let from_name =
                muo_server::mobile::part1::props_b::raw_name(w, from).unwrap_or_default();
            let now = w.now;
            let text = if duration == TimeSpan::MAX {
                "Infinite".to_owned()
            } else {
                duration.to_string()
            };
            for id in &self.list {
                let entry = comment.as_ref().map(|c| {
                    AccountComment::new(w, &from_name, &format!("Duration: {text}, Comment: {c}"))
                });
                if let Some(a) = w.resources.get_mut::<Accounts>().get_mut(*id) {
                    a.set_ban_tags(Some(&from_str), now, duration);
                    if let Some(e) = entry {
                        a.comments.push(e);
                    }
                }
            }
            muo_content::mx::send_message(w, from, &format!("Ban Duration: {text}"));
        } else {
            muo_content::mx::send_message(w, from, "Time values were improperly formatted.");
            send(w, from, new(self.list.clone()));
        }
    }
}
