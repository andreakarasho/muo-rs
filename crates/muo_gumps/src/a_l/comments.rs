//! `CommentsGump`.

use std::rc::Rc;

use muo_content::accounting::{AccountComment, Accounts};
use muo_content::gumps::GumpButtonType;
use muo_content::gumps::base::{DynamicGumpLogic, DynamicGumpObj, RelayInfo};
use muo_content::gumps::builder::DynamicGumpBuilder;
use muo_server::World;
use muo_server::commands::{CommandEventArgs, CommandSystem};
use muo_server::iaccount::AccountId;
use muo_server::ids::MobileId;
use muo_server::mobile::AccessLevel;
use muo_server::net_state::NetStateId;
use muo_server::prompt::{Prompt, add_prompt};
use muo_server::target::{TargetFlags, Targeted, begin_target};

use super::common::{mobile_of, send};

/// `CommentsGump.Configure`.
pub fn configure(w: &mut World) {
    CommandSystem::register(
        w,
        "Comments",
        AccessLevel::Counselor,
        &[],
        Rc::new(|w: &mut World, e: &mut CommandEventArgs| {
            // Usage: Comments; Description: View/Modify/Add account comments.
            muo_content::mx::send_message(
                w,
                e.mobile,
                "Select the player to view account comments.",
            );
            begin_target(w, e.mobile, -1, false, TargetFlags::NONE, on_target);
        }),
    );
}

fn on_target(w: &mut World, from: MobileId, target: Targeted) {
    let m = match target {
        Targeted::Entity(muo_server::ids::EntityId::Mobile(m)) if muo_content::mx::player(w, m) => {
            m
        }
        _ => {
            muo_content::mx::send_message(w, from, "You must target a player.");
            return;
        }
    };
    match w.mobile(m).and_then(|mm| mm.account) {
        None => muo_content::mx::send_message(
            w,
            from,
            "That player doesn't have an account loaded... weird.",
        ),
        Some(a) => display_to(w, from, a),
    }
}

/// The gump, with what `BuildLayout` reads from the account (taken when it is made).
pub struct CommentsGump {
    acct: AccountId,
    username: String,
    /// (added by, last modified `H:mm M/d/yy`, content)
    comments: Vec<(String, String, String)>,
}

fn snapshot(w: &mut World, acct: AccountId) -> Option<CommentsGump> {
    let a = w.resources.get_mut::<Accounts>().get(acct)?;
    Some(CommentsGump {
        acct,
        username: a.username.clone(),
        comments: a
            .comments
            .iter()
            .map(|c| {
                let t = c.last_modified;
                (
                    c.added_by.clone(),
                    format!(
                        "{}:{:02} {}/{}/{:02}",
                        t.hour(),
                        t.minute(),
                        t.month(),
                        t.day(),
                        t.year() % 100
                    ),
                    c.content().to_owned(),
                )
            })
            .collect(),
    })
}

impl DynamicGumpLogic for CommentsGump {
    const TYPE_NAME: &'static str = "Server.Gumps.CommentsGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        b.add_page(0);
        b.add_image_tiled(0, 0, 410, 448, 0xA40);
        b.add_alpha_region(1, 1, 408, 446);

        let title = format!("Comments for '{}'", self.username);
        let mut x = 205 - title.encode_utf16().count() as i32 / 2 * 7;
        if x < 120 {
            x = 120;
        }

        b.add_label(x, 12, 2100, &title);

        b.add_page(1);
        b.add_button(12, 12, 0xFA8, 0xFAA, 0x7F);
        b.add_label(48, 12, 2100, "Add Comment");

        if !self.comments.is_empty() {
            for (i, (added_by, modified, content)) in self.comments.iter().enumerate() {
                let i = i as i32;
                if i >= 5 && i % 5 == 0 {
                    b.add_button_type(368, 12, 0xFA5, 0xFA7, 0, GumpButtonType::Page, i / 5 + 1);
                    b.add_label(298, 12, 2100, "Next Page");
                    b.add_page(i / 5 + 1);
                    b.add_button_type(12, 12, 0xFAE, 0xFB0, 0, GumpButtonType::Page, i / 5);
                    b.add_label(48, 12, 2100, "Prev Page");
                }

                let html = format!("[Added By: {added_by} on {modified}]<br>{content}");
                b.add_html(12, 44 + i % 5 * 80, 386, 70, &html, true, true);
            }
        } else {
            b.add_label(12, 44, 2100, "There are no comments for this account.");
        }
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        if info.button_id == 0x7F {
            let Some(from) = mobile_of(w, sender) else {
                return;
            };
            muo_content::mx::send_message(
                w,
                from,
                "Enter the text for the account comment (or press [Esc] to cancel):",
            );
            let acct = self.acct;
            let prompt = Prompt {
                on_cancel: Some(Rc::new(move |w: &mut World, from| {
                    display_to(w, from, acct)
                })),
                on_response: Some(Rc::new(move |w: &mut World, from, text: &str| {
                    muo_content::mx::send_message(w, from, "Comment added.");
                    let name =
                        muo_server::mobile::part1::props_b::raw_name(w, from).unwrap_or_default();
                    let comment = AccountComment::new(w, &name, text);
                    if let Some(a) = w.resources.get_mut::<Accounts>().get_mut(acct) {
                        a.comments.push(comment);
                    }
                    display_to(w, from, acct);
                })),
            };
            let id = add_prompt(w, prompt);
            muo_server::mobile::part1::props_b::set_prompt(w, from, Some(id));
        }
    }
}

/// `DisplayTo(from, acct)`: `BuildLayout` reads the account at send time, which is now.
fn display_to(w: &mut World, from: MobileId, acct: AccountId) {
    if w.mobile(from).and_then(|m| m.net_state).is_none() {
        return;
    }
    if let Some(d) = snapshot(w, acct) {
        send(w, from, DynamicGumpObj::new(30, 30, d));
    }
}
