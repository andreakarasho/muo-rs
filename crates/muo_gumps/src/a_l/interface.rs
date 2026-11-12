//! The gumps of the `[interface` command: `InterfaceGump` (the paged list of what `[interface` found),
//! `InterfaceItemGump` and `InterfaceMobileGump` (the actions on one object). The command itself is
//! `muo_commands::generic::interface`, which hands the objects to `CommandHooks::interface_gump`.

use muo_commands::generic::conditional::Binding;
use muo_commands::logging::log_action;
use muo_content::gumps::base::{LegacyGumpLogic, LegacyGumpObj, RelayInfo};
use muo_content::gumps::base_grid::{
    ARROW_LEFT_HEIGHT, ARROW_LEFT_ID1, ARROW_LEFT_ID2, ARROW_LEFT_WIDTH, ARROW_RIGHT_HEIGHT,
    ARROW_RIGHT_ID1, ARROW_RIGHT_ID2, ARROW_RIGHT_WIDTH, BaseGrid,
};
use muo_content::props::{self, Access, ClassInfo};
use muo_primitives::MessageType;
use muo_primitives::html;
use muo_server::World;
use muo_server::commands::CommandSystem;
use muo_server::ids::{EntityId, MobileId};
use muo_server::map::MapId;
use muo_server::mobile::part2::messages::send_message;
use muo_server::net_state::NetStateId;

use super::common::{mobile_of, send};

const ENTRIES_PER_PAGE: usize = 15;
const OFFSET_SIZE: i32 = 1;

/// What the three gumps pass to each other: `m_From`, `m_Columns`, `m_List`, `m_Page`.
#[derive(Clone)]
struct Ctx {
    from: MobileId,
    columns: Vec<String>,
    list: Vec<EntityId>,
    page: usize,
}

fn class_of(w: &World, e: EntityId) -> ClassInfo {
    let chain = props::class_chain(w, e);
    ClassInfo {
        full_name: chain.first().copied().unwrap_or_default(),
        chain,
        is_item: matches!(e, EntityId::Item(_)),
    }
}

fn type_name(w: &World, e: EntityId) -> String {
    let c = props::class_chain(w, e);
    c.first()
        .map_or("", |n| n.rsplit('.').next().unwrap_or(n))
        .to_owned()
}

fn exists(w: &World, e: EntityId) -> bool {
    match e {
        EntityId::Item(i) => w.item(i).is_some(),
        EntityId::Mobile(m) => w.mobile(m).is_some(),
    }
}

fn name_of(w: &World, m: MobileId) -> String {
    w.mobile(m)
        .and_then(|x| (x.ty.name)(w, m))
        .unwrap_or_default()
}

fn access(w: &World, m: MobileId) -> muo_server::mobile::AccessLevel {
    muo_content::mx::access_level(w, m)
}

/// `new InterfaceGump(from, columns, list, page, select)`; `columns[0]` is "Object".
pub fn open(w: &mut World, from: MobileId, list: Vec<EntityId>, view: Vec<String>) {
    let mut columns = vec!["Object".to_owned()];
    columns.extend(view);
    // the first object spells each column as its properties are declared (the array is rewritten in place)
    if let Some(&first) = list.first() {
        let class = class_of(w, first);
        let level = access(w, from);
        for c in columns.iter_mut().skip(1) {
            if let Ok(b) = Binding::bind(w, &class, level, c, Access::READ) {
                *c = b.canonical_name();
            }
        }
    }
    let g = new_list(
        w,
        Ctx {
            from,
            columns,
            list,
            page: 0,
        },
        None,
    );
    send(w, from, g);
}

// ---- InterfaceGump ---------------------------------------------------------------------------------------------

pub struct InterfaceGump {
    c: Ctx,
    select: Option<EntityId>,
}

fn new_list(w: &mut World, c: Ctx, select: Option<EntityId>) -> LegacyGumpObj<InterfaceGump> {
    let mut g = LegacyGumpObj::new(
        30,
        30,
        InterfaceGump {
            c: c.clone(),
            select,
        },
    );
    let l = &mut g.layout;
    let mut gr = BaseGrid::default();
    let ncol = c.columns.len() as i32;
    gr.add_new_page(l);

    if c.page > 0 {
        gr.add_entry_button(
            l,
            20,
            ARROW_LEFT_ID1,
            ARROW_LEFT_ID2,
            1,
            ARROW_LEFT_WIDTH,
            ARROW_LEFT_HEIGHT,
            1,
        );
    } else {
        gr.add_entry_header(l, 20, 1);
    }
    gr.add_entry_html(
        l,
        40 + ncol * 130 - 20 + (ncol - 2) * OFFSET_SIZE,
        &html::center(&format!(
            "Page {} of {}",
            c.page + 1,
            c.list.len().div_ceil(ENTRIES_PER_PAGE)
        )),
    );
    if (c.page + 1) * ENTRIES_PER_PAGE < c.list.len() {
        gr.add_entry_button(
            l,
            20,
            ARROW_RIGHT_ID1,
            ARROW_RIGHT_ID2,
            2,
            ARROW_RIGHT_WIDTH,
            ARROW_RIGHT_HEIGHT,
            1,
        );
    } else {
        gr.add_entry_header(l, 20, 1);
    }

    if c.columns.len() > 1 {
        gr.add_new_line();
        for (i, col) in c.columns.iter().enumerate() {
            gr.add_entry_html(l, 130 + if i == 0 { 40 } else { 0 }, col);
        }
        gr.add_entry_header(l, 20, 1);
    }

    let level = access(w, c.from);
    let mut line = 0;
    let mut i = c.page * ENTRIES_PER_PAGE;
    while line < ENTRIES_PER_PAGE && i < c.list.len() {
        gr.add_new_line();
        let obj = c.list[i];
        let deleted = !exists(w, obj);
        if !deleted {
            let text = match obj {
                EntityId::Item(_) => type_name(w, obj),
                EntityId::Mobile(m) => name_of(w, m),
            };
            gr.add_entry_html(l, 40 + 130, &text);
        }
        if deleted {
            gr.add_entry_html(l, 40 + 130, "(deleted)");
            for _ in 1..c.columns.len() {
                gr.add_entry_html(l, 130, "---");
            }
            gr.add_entry_header(l, 20, 1);
        } else {
            let class = class_of(w, obj);
            for col in &c.columns[1..] {
                let value = Binding::bind(w, &class, level, col, Access::READ)
                    .ok()
                    .and_then(|b| b.eval(w, obj))
                    .map_or_else(
                        || "---".to_owned(),
                        |(v, def)| crate::m_z::props::registry::value_text(w, def, &v),
                    );
                gr.add_entry_html(l, 130, &value);
            }
            let sel = select == Some(obj);
            gr.add_entry_button(
                l,
                20,
                if sel { 9762 } else { ARROW_RIGHT_ID1 },
                if sel { 9763 } else { ARROW_RIGHT_ID2 },
                3 + i as i32,
                ARROW_RIGHT_WIDTH,
                ARROW_RIGHT_HEIGHT,
                1,
            );
        }
        i += 1;
        line += 1;
    }
    gr.finish_page(l);
    g
}

impl LegacyGumpLogic for InterfaceGump {
    const TYPE_NAME: &'static str = "Server.Commands.Generic.InterfaceGump";

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let Some(from) = mobile_of(w, sender) else {
            return;
        };
        let c = self.c.clone();
        let again = |w: &mut World, c: Ctx, select| {
            let g = new_list(w, c, select);
            send(w, from, g);
        };
        match info.button_id {
            1 => {
                if c.page > 0 {
                    again(
                        w,
                        Ctx {
                            page: c.page - 1,
                            ..c
                        },
                        self.select,
                    );
                }
            }
            2 => {
                if (c.page + 1) * ENTRIES_PER_PAGE < c.list.len() {
                    again(
                        w,
                        Ctx {
                            page: c.page + 1,
                            ..c
                        },
                        self.select,
                    );
                }
            }
            id => {
                let Some(v) = usize::try_from(id - 3).ok().filter(|v| *v < c.list.len()) else {
                    return;
                };
                let obj = c.list[v];
                if !muo_content::targets::is_accessible(w, c.from, Some(obj)) {
                    muo_server::item::deps::send_localized_message(w, c.from, 500447); // That is not accessible.
                    again(w, c, self.select);
                } else if !exists(w, obj) {
                    again(w, c, self.select);
                } else {
                    match obj {
                        EntityId::Item(_) => {
                            let from = c.from;
                            let g = new_item(w, c, obj);
                            send(w, from, g)
                        }
                        EntityId::Mobile(m) => {
                            let from = c.from;
                            let g = new_mobile(w, c, m);
                            send(w, from, g)
                        }
                    }
                }
            }
        }
    }
}

// ---- InterfaceItemGump -----------------------------------------------------------------------------------------

pub struct InterfaceItemGump {
    c: Ctx,
    item: EntityId,
}

fn action_row(
    gr: &mut BaseGrid,
    l: &mut muo_content::gumps::legacy::LegacyLayout,
    text: &str,
    id: i32,
) {
    gr.add_new_line();
    gr.add_entry_html(l, 20 + OFFSET_SIZE + 160, text);
    gr.add_entry_button(
        l,
        20,
        ARROW_RIGHT_ID1,
        ARROW_RIGHT_ID2,
        id,
        ARROW_RIGHT_WIDTH,
        ARROW_RIGHT_HEIGHT,
        1,
    );
}

fn new_item(w: &mut World, c: Ctx, item: EntityId) -> LegacyGumpObj<InterfaceItemGump> {
    let title = type_name(w, item);
    let mut g = LegacyGumpObj::new(30, 30, InterfaceItemGump { c, item });
    let l = &mut g.layout;
    let mut gr = BaseGrid::default();
    gr.add_new_page(l);
    gr.add_entry_button(
        l,
        20,
        ARROW_LEFT_ID1,
        ARROW_LEFT_ID2,
        1,
        ARROW_LEFT_WIDTH,
        ARROW_LEFT_HEIGHT,
        1,
    );
    gr.add_entry_html(l, 160, &title);
    gr.add_entry_header(l, 20, 1);
    action_row(&mut gr, l, "Properties", 2);
    action_row(&mut gr, l, "Delete", 3);
    action_row(&mut gr, l, "Go there", 4);
    action_row(&mut gr, l, "Move to target", 5);
    action_row(&mut gr, l, "Bring to pack", 6);
    gr.finish_page(l);
    g
}

/// `InvokeCommand`: `CommandSystem.Handle(from, Prefix + text)`.
fn invoke_command(w: &mut World, from: MobileId, text: &str) {
    let prefix = w.resources.get_mut::<CommandSystem>().prefix.clone();
    CommandSystem::handle(w, from, &format!("{prefix}{text}"), MessageType::REGULAR);
}

fn serial_of(w: &World, e: EntityId) -> muo_primitives::Serial {
    match e {
        EntityId::Item(i) => w.item(i).map(|x| x.serial),
        EntityId::Mobile(m) => w.mobile(m).map(|x| x.serial),
    }
    .unwrap_or_default()
}

fn open_properties(w: &mut World, from: MobileId, e: EntityId) {
    if let Some(f) = w
        .resources
        .get_mut::<muo_commands::hooks::CommandHooks>()
        .properties_gump
    {
        f(w, from, e);
    }
}

impl LegacyGumpLogic for InterfaceItemGump {
    const TYPE_NAME: &'static str = "Server.Commands.Generic.InterfaceItemGump";

    fn on_response(&mut self, w: &mut World, _sender: NetStateId, info: &RelayInfo) {
        let c = self.c.clone();
        let from = c.from;
        let back = |w: &mut World, c: &Ctx, item| {
            let g = new_list(w, c.clone(), Some(item));
            send(w, from, g);
        };
        let item = self.item;
        let EntityId::Item(id) = item else { return };
        if !exists(w, item) {
            return back(w, &c, item);
        }
        if !muo_content::targets::is_accessible(w, from, Some(item)) {
            send_message(w, from, "That is no longer accessible.");
            return back(w, &c, item);
        }
        let stay = |w: &mut World| {
            let g = new_item(w, c.clone(), item);
            send(w, from, g);
        };
        match info.button_id {
            0 | 1 => back(w, &c, item),
            2 => {
                stay(w);
                open_properties(w, from, item);
            }
            3 => {
                log_action(w, from, "deleting", item);
                muo_server::item::base::delete(w, id);
                back(w, &c, item);
            }
            4 => {
                stay(w);
                let s = serial_of(w, item);
                invoke_command(w, from, &format!("Go {s}"));
            }
            5 => {
                stay(w);
                muo_content::targets::move_target(w, from, item);
            }
            6 => {
                let owner = match muo_server::item::ops::root_parent(w, id) {
                    Some(EntityId::Mobile(m)) => Some(m),
                    _ => None,
                };
                let (cannot_see_owner, hidden_equal) =
                    match owner.and_then(|m| w.mobile(m).map(|x| (m, x))) {
                        Some((m, x)) => {
                            let on_map = x.map.is_some_and(|mp| mp != MapId::INTERNAL);
                            (
                                on_map
                                    && !muo_content::targets::is_accessible(
                                        w,
                                        from,
                                        Some(EntityId::Mobile(m)),
                                    ),
                                !on_map && x.hidden && x.access_level >= access(w, from),
                            )
                        }
                        None => (false, false),
                    };
                if cannot_see_owner || hidden_equal {
                    send_message(w, from, "You can not get what you can not see.");
                } else {
                    stay(w);
                    muo_server::mobile::part2::items::add_to_backpack(w, from, id);
                }
            }
            _ => {}
        }
    }
}

// ---- InterfaceMobileGump ---------------------------------------------------------------------------------------

pub struct InterfaceMobileGump {
    c: Ctx,
    mobile: MobileId,
}

fn new_mobile(w: &mut World, c: Ctx, mobile: MobileId) -> LegacyGumpObj<InterfaceMobileGump> {
    let from = c.from;
    let (player, has_net) = w
        .mobile(mobile)
        .map_or((false, false), |m| (m.player, m.net_state.is_some()));
    let alive = w.mobile(mobile).is_some_and(|m| (m.ty.alive)(w, mobile));
    let title = name_of(w, mobile);
    let mut g = LegacyGumpObj::new(30, 30, InterfaceMobileGump { c, mobile });
    let l = &mut g.layout;
    let mut gr = BaseGrid::default();
    gr.add_new_page(l);
    gr.add_entry_button(
        l,
        20,
        ARROW_LEFT_ID1,
        ARROW_LEFT_ID2,
        1,
        ARROW_LEFT_WIDTH,
        ARROW_LEFT_HEIGHT,
        1,
    );
    gr.add_entry_html(l, 160, &title);
    gr.add_entry_header(l, 20, 1);
    action_row(&mut gr, l, "Properties", 2);
    if !player {
        action_row(&mut gr, l, "Delete", 3);
    }
    if mobile != from {
        action_row(&mut gr, l, "Go to there", 4);
        action_row(&mut gr, l, "Bring them here", 5);
    }
    action_row(&mut gr, l, "Move to target", 6);
    if from == mobile || access(w, from) > access(w, mobile) {
        if alive {
            action_row(&mut gr, l, "Kill", 7);
        } else {
            action_row(&mut gr, l, "Resurrect", 8);
        }
    }
    if has_net {
        action_row(&mut gr, l, "Client", 9);
    }
    gr.finish_page(l);
    g
}

impl LegacyGumpLogic for InterfaceMobileGump {
    const TYPE_NAME: &'static str = "Server.Commands.Generic.InterfaceMobileGump";

    fn on_response(&mut self, w: &mut World, _sender: NetStateId, info: &RelayInfo) {
        let c = self.c.clone();
        let from = c.from;
        let m = self.mobile;
        let e = EntityId::Mobile(m);
        let back = |w: &mut World, c: &Ctx| {
            let g = new_list(w, c.clone(), Some(e));
            send(w, from, g);
        };
        if !exists(w, e) {
            return back(w, &c);
        }
        if !muo_content::targets::is_accessible(w, from, Some(e)) {
            send_message(w, from, "That is no longer accessible.");
            return back(w, &c);
        }
        let stay = |w: &mut World| {
            let g = new_mobile(w, c.clone(), m);
            send(w, from, g);
        };
        let may_harm = from == m || access(w, from) > access(w, m);
        match info.button_id {
            0 | 1 => back(w, &c),
            2 => {
                stay(w);
                open_properties(w, from, e);
            }
            3 => {
                if !w.mobile(m).is_some_and(|x| x.player) {
                    log_action(w, from, "deleting", e);
                    muo_server::mobile::base::delete(w, m);
                    back(w, &c);
                }
            }
            4 => {
                stay(w);
                let s = serial_of(w, e);
                invoke_command(w, from, &format!("Go {s}"));
            }
            5 => match w.mobile(from).map(|x| (x.location, x.map)) {
                Some((loc, Some(map))) if map != MapId::INTERNAL => {
                    stay(w);
                    let ty = w.mobile(m).map(|x| x.ty);
                    if let Some(ty) = ty {
                        (ty.move_to_world)(w, m, loc, map);
                    }
                }
                _ => send_message(w, from, "You cannot bring that person here."),
            },
            6 => {
                stay(w);
                muo_content::targets::move_target(w, from, e);
            }
            7 => {
                if may_harm {
                    muo_server::mobile::base::kill(w, m);
                }
                stay(w);
            }
            8 => {
                if may_harm {
                    muo_server::mobile::part2::effects::play_sound(w, m, 0x214);
                    muo_server::mobile::part2::effects::fixed_effect(w, m, 0x376A, 10, 16);
                    let ty = w.mobile(m).map(|x| x.ty);
                    if let Some(ty) = ty {
                        (ty.resurrect)(w, m);
                    }
                }
                stay(w);
            }
            9 => {
                stay(w);
                if w.mobile(m).is_some_and(|x| x.net_state.is_some())
                    && let Some(f) = w
                        .resources
                        .get_mut::<muo_commands::hooks::CommandHooks>()
                        .client_gump
                {
                    f(w, from, m);
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
pub fn new_list_for_test(
    w: &mut World,
    from: MobileId,
    list: Vec<EntityId>,
    view: Vec<String>,
) -> LegacyGumpObj<InterfaceGump> {
    let mut columns = vec!["Object".to_owned()];
    columns.extend(view);
    new_list(
        w,
        Ctx {
            from,
            columns,
            list,
            page: 0,
        },
        None,
    )
}
