//! The Inscription skill.
//!
//! `BaseBook`, `BlankScroll` and `RecallRune` are items other crates implement ([`SkillItemFns`]); the T2A craft menu
//! and the T2A runebook recipe (`DefInscription.CraftSystem`) are [`SkillGumpHooks`].

use std::rc::Rc;

use indexmap::IndexMap;
use muo_primitives::TimeSpan;
use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::mobile::part2::messages as mm;
use muo_server::skills::SkillName;
use muo_server::target::{TargetCancelType, Targeted};

use super::SkillGumpHooks;
use super::common::{TargetOptions, check_target_skill, msg, set_target};
use super::hooks::ops;
use super::skills_info::set_callback;
use crate::configuration::expansion_configuration::T2aCraftSystemEnabled;
use crate::mx;

/// `Inscribe.m_UseTable`.
#[derive(Default)]
pub struct InscribeUsers(IndexMap<ItemId, MobileId>);

fn users(w: &mut World) -> &mut InscribeUsers {
    w.resources.get_mut::<InscribeUsers>()
}

/// `Inscribe.GetUser(book)`.
pub fn get_user(w: &World, book: ItemId) -> Option<MobileId> {
    w.resources
        .get::<InscribeUsers>()
        .and_then(|u| u.0.get(&book).copied())
}

fn entity_item(t: Targeted) -> Option<ItemId> {
    match t {
        Targeted::Entity(EntityId::Item(i)) => Some(i),
        _ => None,
    }
}

fn timeout_cancel(w: &mut World, from: MobileId, kind: TargetCancelType) {
    if kind == TargetCancelType::Timeout {
        // You have waited too long to make your inscribe selection, your inscription attempt has timed out.
        msg(w, from, 501619);
    }
}

pub fn on_use(w: &mut World, m: MobileId) -> TimeSpan {
    let t2a = w
        .resources
        .get::<T2aCraftSystemEnabled>()
        .is_some_and(|e| e.0);
    if t2a {
        set_target(
            w,
            m,
            3,
            TargetOptions {
                on_cancel: Some(timeout_cancel),
                timeout_ms: Some(60000), // 1 minute
                ..Default::default()
            },
            t2a_on_target,
        );
        mm::send_ascii_message(
            w,
            m,
            "Target the book you wish to copy or scroll you want to use.",
        );
        return TimeSpan::from_secs(1);
    }

    set_target(
        w,
        m,
        3,
        TargetOptions {
            on_cancel: Some(timeout_cancel),
            timeout_ms: Some(60000), // 1 minute
            ..Default::default()
        },
        src_on_target,
    );
    msg(w, m, 1046295); // Target the book you wish to copy.

    TimeSpan::from_secs(1)
}

fn t2a_on_target(w: &mut World, from: MobileId, targeted: Targeted) {
    let item = entity_item(targeted);
    if let Some(scroll) = item.filter(|&i| ops(w).is_blank_scroll(w, i)) {
        let pack = w.mobile(from).and_then(|m| m.backpack).map(EntityId::Item);
        if !muo_server::item::ops::is_child_of(w, scroll, pack) {
            mm::send_ascii_message(w, from, "That must be in your pack for you to use it.");
        } else if let Some(f) = w
            .resources
            .get::<SkillGumpHooks>()
            .and_then(|h| h.show_inscription_menu)
        {
            f(w, from);
        }
    } else if item.is_some_and(|i| ops(w).is_recall_rune(w, i)) {
        // `system.CraftItems.SearchFor(typeof(Runebook))` then `CanCraft` and `CreateItem`.
        if let Some(f) = w
            .resources
            .get::<SkillGumpHooks>()
            .and_then(|h| h.craft_runebook)
        {
            f(w, from);
        }
    } else if let Some(book) = item.filter(|&i| ops(w).book_facts(w, i).is_some()) {
        pick_destination(w, from, book);
    } else {
        msg(w, from, 1046296); // That is not a book
    }
}

fn src_on_target(w: &mut World, from: MobileId, targeted: Targeted) {
    let Some(book) = entity_item(targeted).filter(|&i| ops(w).book_facts(w, i).is_some()) else {
        msg(w, from, 1046296); // That is not a book
        return;
    };
    pick_destination(w, from, book);
}

/// The part of both source targets that takes a book: empty, in use, or the destination target.
fn pick_destination(w: &mut World, from: MobileId, book: ItemId) {
    let facts = ops(w).book_facts(w, book).expect("a book");
    if facts.is_empty {
        msg(w, from, 501611); // Can't copy an empty book.
    } else if get_user(w, book).is_some() {
        msg(w, from, 501621); // Someone else is inscribing that item.
    } else {
        set_target(
            w,
            from,
            3,
            TargetOptions {
                on_cancel: Some(timeout_cancel),
                on_finish: Some(Rc::new(move |w: &mut World, _from: MobileId| {
                    // CancelUser(m_BookSrc)
                    users(w).0.shift_remove(&book);
                })),
                timeout_ms: Some(60000), // 1 minute
                ..Default::default()
            },
            move |w, from, targeted| dst_on_target(w, from, targeted, book),
        );
        msg(w, from, 501612); // Select a book to copy this to.
        users(w).0.insert(book, from); // SetUser
    }
}

fn dst_on_target(w: &mut World, from: MobileId, targeted: Targeted, src: ItemId) {
    if muo_server::item::ops::is_deleted(w, src) {
        return;
    }

    let dst = entity_item(targeted).filter(|&i| ops(w).book_facts(w, i).is_some());
    let Some(dst) = dst else {
        msg(w, from, 1046296); // That is not a book
        return;
    };
    let src_empty = ops(w).book_facts(w, src).is_none_or(|f| f.is_empty);
    let writable = ops(w).book_facts(w, dst).is_some_and(|f| f.writable);

    if src_empty {
        msg(w, from, 501611); // Can't copy an empty book.
    } else if dst == src {
        msg(w, from, 501616); // Cannot copy a book onto itself.
    } else if !writable {
        msg(w, from, 501614); // Cannot write into that book.
    } else if get_user(w, dst).is_some() {
        msg(w, from, 501621); // Someone else is inscribing that item.
    } else if check_target_skill(w, from, SkillName::Inscribe, targeted, 0.0, 50.0) {
        ops(w).copy_book(w, src, dst);

        msg(w, from, 501618); // You make a copy of the book.
        mx::play_sound(w, from, 0x249);
    } else {
        msg(w, from, 501617); // You fail to make a copy of the book.
    }
}

pub fn initialize(w: &mut World) {
    set_callback(w, SkillName::Inscribe, on_use);
}
