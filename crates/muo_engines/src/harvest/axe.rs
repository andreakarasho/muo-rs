//! `IAxe` of the logs: an axe used on a log of the pack cuts it into boards
//! (`Log.Axe(from, axe)` and the per-wood overrides), called from `HarvestTarget.OnTarget`.

use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::deps;
use muo_server::skills::{self, SkillName};

use muo_content::mx;

/// `(skill needed, board class)` of a log class; `None` if the item is no `Log`.
fn board_of(w: &World, log: ItemId) -> Option<(f64, &'static str)> {
    let ty = w.item(log)?.ty;
    if !muo_content::items::is_named(ty, "Server.Items.Log") {
        return None;
    }
    // The innermost override wins: the leaf classes before `Log`.
    const WOODS: &[(&str, f64, &str)] = &[
        (
            "Server.Items.HeartwoodLog",
            100.0,
            "Server.Items.HeartwoodBoard",
        ),
        (
            "Server.Items.BloodwoodLog",
            100.0,
            "Server.Items.BloodwoodBoard",
        ),
        (
            "Server.Items.FrostwoodLog",
            100.0,
            "Server.Items.FrostwoodBoard",
        ),
        ("Server.Items.OakLog", 65.0, "Server.Items.OakBoard"),
        ("Server.Items.AshLog", 80.0, "Server.Items.AshBoard"),
        ("Server.Items.YewLog", 95.0, "Server.Items.YewBoard"),
    ];
    for (name, skill, board) in WOODS {
        if muo_content::items::is_named(ty, name) {
            return Some((*skill, board));
        }
    }
    Some((0.0, "Server.Items.Board"))
}

/// `HarvestTarget`'s `targeted is IAxe obj && m_Tool is BaseAxe axe` branch: `None` if the target is no log or the
/// tool no axe, else whether it was handled.
pub fn axe_target(w: &mut World, item: ItemId, from: MobileId, tool: ItemId) -> Option<bool> {
    let (skill, board) = board_of(w, item)?;
    let tool_is_axe = w
        .item(tool)
        .is_some_and(|t| t.ty.is(&muo_content::items::weapon::BASE_AXE));
    if !tool_is_axe {
        return None;
    }
    let pack = deps::find_item_on_layer(w, from, muo_server::layer::Layer::Backpack);
    let in_pack = pack.is_some_and(|p| {
        muo_server::item::ops::is_child_of(w, item, Some(muo_server::ids::EntityId::Item(p)))
    });
    if !in_pack {
        deps::send_localized_message(w, from, 1062334); // This item must be in your backpack to be used.
    } else if try_create_boards(w, item, from, skill, board) {
        mx::play_sound(w, from, 0x13E);
    }
    Some(true)
}

/// `Log.TryCreateBoards(from, skill, item)`.
fn try_create_boards(
    w: &mut World,
    log: ItemId,
    from: MobileId,
    skill: f64,
    board_type: &str,
) -> bool {
    if w.item(log).is_none_or(|i| i.deleted())
        || !muo_server::mobile::base::can_see_item(w, from, log)
    {
        return false;
    }
    let Some(board) = muo_content::factory::make_item(w, board_type) else {
        return false;
    };
    if skills::value(w, from, SkillName::Carpentry) < skill
        && skills::value(w, from, SkillName::Lumberjacking) < skill
    {
        if let Some(t) = w.item(board).map(|i| i.ty) {
            (t.delete)(w, board);
        }
        deps::send_localized_message(w, from, 1072652); // You cannot work this strange and unusual wood.
        return false;
    }
    if let Some(t) = w.item(log).map(|i| i.ty) {
        (t.scissor_helper)(w, log, from, board, 1, false);
    }
    true
}
