//! The `SkillItemFns` over classes this crate owns that belong to no folder of Items/Skill Items: `BaseBook`
//! (Inscribe: `Inscribe.IsEmpty`, `Inscribe.Copy`) and `BaseTrap` (DetectHidden's reveal).

use muo_content::skills::hooks::BookFacts;
use muo_server::World;
use muo_server::ids::ItemId;
use muo_server::item::ops;
use muo_server::type_registry::{Phase, TypeRegistry};

use crate::books;
use crate::traps::trap_classes::BASE_TRAP;

/// `Inscribe.IsEmpty(book)` and `Writable`.
fn book_facts(w: &World, item: ItemId) -> Option<BookFacts> {
    if !books::is_book(w, item) {
        return None;
    }
    let d = books::data(w, item)?;
    Some(BookFacts {
        is_empty: d.pages.iter().all(|p| p.iter().all(|l| l.is_empty())),
        writable: d.writable,
    })
}

/// `Inscribe.Copy(src, dst)`: title and author, then the lines of the pages both books have.
fn copy_book(w: &mut World, src: ItemId, dst: ItemId) {
    let Some(s) = books::data(w, src).cloned() else {
        return;
    };
    let Some(d) = books::data_mut(w, dst) else {
        return;
    };
    d.title = s.title;
    d.author = s.author;
    for (i, page) in s.pages.iter().enumerate() {
        if i >= d.pages.len() {
            break;
        }
        d.pages[i] = page.clone();
    }
    ops::invalidate_properties(w, dst);
}

fn configure(w: &mut World) {
    let f = &mut w.resources.get_mut::<muo_content::skills::SkillHooks>().fns;
    f.book_facts = Some(book_facts);
    f.copy_book = Some(copy_book);
    // `item is BaseTrap { Visible: false }`.
    f.is_hidden_trap = Some(|w, item| {
        w.item(item)
            .is_some_and(|i| i.ty.is(&BASE_TRAP) && !i.visible())
    });
    f.set_trap_visible = Some(ops::set_visible);
}

/// Registers the hooks.
pub fn register(types: &mut TypeRegistry) {
    types.register_phase_default(
        Phase::Configure,
        "Server.Items.SkillItemHooks.Configure",
        configure,
    );
}
