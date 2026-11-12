//! `CraftGump`, `CraftGumpItem` and `QueryMakersMarkGump`.
//!
//! A gump draws what it finds in the world (`from.Backpack.FindItems()`, the success chance), but a layout has no
//! world, so the numbers it draws are collected when the gump is sent and the gump only draws.

use std::rc::Rc;

use muo_content::gumps::{
    DynamicGumpBuilder, DynamicGumpLogic, DynamicGumpObj, GumpButtonType, RelayInfo,
    StaticGumpLogic, StaticGumpObj, close_gump, send_gump_to_mobile,
};
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::net_state::NetStateId;
use muo_server::skills;

use super::item as craft_item;
use super::model::*;
use super::{context_of, modify_context};

const LABEL_HUE: i32 = 0x480;
const LABEL_COLOR: i32 = 0x7FFF;
const FONT_COLOR: i32 = 0xFFFFFF;
const RED_LABEL_HUE: i32 = 0x20;
const RED_LABEL_COLOR: i32 = 0x6400;
const GREY_LABEL_COLOR: i32 = 0x3DEF;

/// `CraftGump.CraftPage`.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum CraftPage {
    None,
    PickResource,
    PickResource2,
}

/// `CraftGump.GetButtonID(type, index)`.
pub fn button_id(kind: i32, index: i32) -> i32 {
    1 + kind + index * 7
}

/// `AddHtmlText(ref builder, x, y, w, h, back, scroll, numberColor, stringColor)`.
#[allow(clippy::too_many_arguments)]
fn add_td_text(
    b: &mut DynamicGumpBuilder,
    td: &Td,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    back: bool,
    scroll: bool,
    number_color: i32,
    string_color: i32,
) {
    use muo_content::text::{HtmlText, add_html_text_builder};
    match add_html_text_builder(Some(td), back, scroll, number_color, string_color) {
        Some(HtmlText::Localized {
            number,
            args: None,
            color: Some(color),
            back,
            scroll,
        }) => b.add_html_localized_color(x, y, width, height, number, color, back, scroll),
        Some(HtmlText::Localized {
            number,
            args: None,
            color: None,
            back,
            scroll,
        }) => b.add_html_localized(x, y, width, height, number, back, scroll),
        Some(HtmlText::Localized {
            number,
            args: Some(a),
            color,
            back,
            scroll,
        }) => b.add_html_localized_args(
            x,
            y,
            width,
            height,
            number,
            &a,
            color.unwrap_or(0),
            back,
            scroll,
        ),
        Some(HtmlText::Html { text, back, scroll }) => {
            b.add_html(x, y, width, height, &text, back, scroll)
        }
        None => {}
    }
}

/// What the resource lists draw: a name and how many of it the player carries.
#[derive(Clone)]
struct ResRow {
    name: Td,
    count: i32,
}

/// `CraftGump`.
pub struct CraftGumpLogic {
    from: MobileId,
    sys: Rc<CraftSystem>,
    tool: Option<ItemId>,
    page: CraftPage,
    notice: Td,
    ctx: Option<CraftContext>,
    res: Option<ResRow>,
    res2: Option<ResRow>,
    res_list: Vec<ResRow>,
    /// The names of the "make last" items.
    last_items: Vec<Td>,
}

/// How many of the items of `ty` (and subclasses) `from` carries, backpack and inside.
fn count_in_pack(w: &World, from: MobileId, ty: &str) -> i32 {
    let Some(pack) =
        muo_server::item::deps::find_item_on_layer(w, from, muo_server::layer::Layer::Backpack)
    else {
        return 0;
    };
    let Some(t) = craft_item::resolve(w, ty) else {
        return 0;
    };
    muo_server::item::container::get_amount(w, pack, t, true)
}

/// `CraftSystem.GetContext(m)`'s sub-resource display: `LastResourceIndex` picks which one is named.
fn sub_res_row(w: &World, from: MobileId, col: &SubResCol, index: i32) -> ResRow {
    let mut name = col.name.clone();
    let mut ty = col.res_type;
    if index > -1
        && let Some(sub) = col.items.get(index as usize)
    {
        name = sub.name.clone();
        ty = sub.item_type;
    }
    ResRow {
        name,
        count: count_in_pack(w, from, ty),
    }
}

/// Opens (or replaces) `from`'s craft gump: `new CraftGump(from, craftSystem, tool, notice, page)`.
pub fn send_craft_gump(
    w: &mut World,
    from: MobileId,
    sys: &Rc<CraftSystem>,
    tool: Option<ItemId>,
    notice: Option<Td>,
    page: CraftPage,
) {
    let ctx = context_of(w, sys.name, from);
    let res = sys.sub_res.init.then(|| {
        sub_res_row(
            w,
            from,
            &sys.sub_res,
            ctx.as_ref().map_or(-1, |c| c.last_resource_index),
        )
    });
    let res2 = sys.sub_res2.init.then(|| {
        sub_res_row(
            w,
            from,
            &sys.sub_res2,
            ctx.as_ref().map_or(-1, |c| c.last_resource_index2),
        )
    });
    let res_list = match page {
        CraftPage::None => Vec::new(),
        CraftPage::PickResource | CraftPage::PickResource2 => {
            let col = if page == CraftPage::PickResource {
                &sys.sub_res
            } else {
                &sys.sub_res2
            };
            col.items
                .iter()
                .map(|s| ResRow {
                    name: s.name.clone(),
                    count: count_in_pack(w, from, s.item_type),
                })
                .collect()
        }
    };
    let last_items = ctx
        .as_ref()
        .map(|c| c.items.iter().map(|&i| sys.items[i].name.clone()).collect())
        .unwrap_or_default();
    let logic = CraftGumpLogic {
        from,
        sys: sys.clone(),
        tool,
        page,
        notice: notice.unwrap_or(Td::EMPTY),
        ctx,
        res,
        res2,
        res_list,
        last_items,
    };
    // SendTo: the item gump is closed first.
    if let Some(ns) = w.mobile(from).and_then(|m| m.net_state) {
        close_gump::<CraftGumpItemLogic>(w, ns);
    }
    send_gump_to_mobile(w, from, Box::new(DynamicGumpObj::new(40, 40, logic)), false);
}

fn name_label(b: &mut DynamicGumpBuilder, x: i32, y: i32, name: &Td, width: i32) {
    if name.number > 0 {
        b.add_html_localized_color(x, y, width, 18, name.number, LABEL_COLOR, false, false);
    } else {
        b.add_label(x, y, LABEL_HUE, name.string.as_deref().unwrap_or(""));
    }
}

impl CraftGumpLogic {
    fn create_group_list(&self, b: &mut DynamicGumpBuilder) {
        b.add_button(15, 60, 4005, 4007, button_id(6, 3));
        b.add_html_localized_color(50, 63, 150, 18, 1044014, LABEL_COLOR, false, false); // LAST TEN
        for (i, g) in self.sys.groups.iter().enumerate() {
            let i = i as i32;
            b.add_button(15, 80 + i * 20, 4005, 4007, button_id(0, i));
            if g.name.number > 0 {
                b.add_html_localized_color(
                    50,
                    83 + i * 20,
                    150,
                    18,
                    g.name.number,
                    LABEL_COLOR,
                    false,
                    false,
                );
            } else {
                b.add_label(
                    50,
                    83 + i * 20,
                    LABEL_HUE,
                    g.name.string.as_deref().unwrap_or(""),
                );
            }
        }
    }

    fn page_buttons(b: &mut DynamicGumpBuilder, i: i32) {
        if i > 0 {
            b.add_button_type(370, 260, 4005, 4007, 0, GumpButtonType::Page, i / 10 + 1);
            b.add_html_localized_color(405, 263, 100, 18, 1044045, LABEL_COLOR, false, false); // NEXT PAGE
        }
        b.add_page(i / 10 + 1);
        if i > 0 {
            b.add_button_type(220, 260, 4014, 4015, 0, GumpButtonType::Page, i / 10);
            b.add_html_localized_color(255, 263, 100, 18, 1044044, LABEL_COLOR, false, false); // PREV PAGE
        }
    }

    fn create_item_list(&self, b: &mut DynamicGumpBuilder, group: i32) {
        if group == 501 {
            // 501 : Last 10
            self.create_make_last_list(b);
            return;
        }
        let Some(g) = self.sys.groups.get(group as usize) else {
            return;
        };
        for (i, &ci) in g.items.iter().enumerate() {
            let i = i as i32;
            let index = i % 10;
            if index == 0 {
                Self::page_buttons(b, i);
            }
            b.add_button(220, 60 + index * 20, 4005, 4007, button_id(1, i));
            name_label(b, 255, 63 + index * 20, &self.sys.items[ci].name, 220);
            b.add_button(480, 60 + index * 20, 4011, 4012, button_id(2, i));
        }
    }

    fn create_make_last_list(&self, b: &mut DynamicGumpBuilder) {
        if self.ctx.is_none() {
            return;
        }
        if self.last_items.is_empty() {
            b.add_html_localized_color(230, 62, 200, 22, 1044165, LABEL_COLOR, false, false); // You haven't made anything yet.
            return;
        }
        for (i, name) in self.last_items.iter().enumerate() {
            let i = i as i32;
            let index = i % 10;
            if index == 0 {
                Self::page_buttons(b, i);
            }
            b.add_button(220, 60 + index * 20, 4005, 4007, button_id(3, i));
            name_label(b, 255, 63 + index * 20, name, 220);
            b.add_button(480, 60 + index * 20, 4011, 4012, button_id(4, i));
        }
    }

    fn create_res_list(&self, b: &mut DynamicGumpBuilder) {
        for (i, row) in self.res_list.iter().enumerate() {
            let i = i as i32;
            let index = i % 10;
            if index == 0 {
                if i > 0 {
                    b.add_button_type(485, 260, 4005, 4007, 0, GumpButtonType::Page, i / 10 + 1);
                }
                b.add_page(i / 10 + 1);
                if i > 0 {
                    b.add_button_type(455, 260, 4014, 4015, 0, GumpButtonType::Page, i / 10);
                }
                b.add_button(220, 260, 4005, 4007, button_id(6, 4));
                let color_label = if self.ctx.as_ref().is_some_and(|c| c.do_not_color) {
                    1061590
                } else {
                    1061591
                };
                b.add_html_localized_color(
                    255,
                    263,
                    200,
                    18,
                    color_label,
                    LABEL_COLOR,
                    false,
                    false,
                );
            }
            b.add_button(220, 60 + index * 20, 4005, 4007, button_id(5, i));
            if row.name.number > 0 {
                b.add_html_localized_args(
                    255,
                    63 + index * 20,
                    250,
                    18,
                    row.name.number,
                    &row.count.to_string(),
                    LABEL_COLOR,
                    false,
                    false,
                );
            } else {
                b.add_label(
                    255,
                    63 + index * 20,
                    LABEL_HUE,
                    &format!(
                        "{} ({})",
                        row.name.string.as_deref().unwrap_or(""),
                        row.count
                    ),
                );
            }
        }
    }
}

fn res_line(b: &mut DynamicGumpBuilder, y: i32, button: i32, row: &ResRow) {
    b.add_button(15, y, 4005, 4007, button);
    if row.name.number > 0 {
        b.add_html_localized_args(
            50,
            y + 3,
            250,
            18,
            row.name.number,
            &row.count.to_string(),
            LABEL_COLOR,
            false,
            false,
        );
    } else {
        b.add_label(
            50,
            y + 3,
            LABEL_HUE,
            &format!(
                "{} ({} Available)",
                row.name.string.as_deref().unwrap_or(""),
                row.count
            ),
        );
    }
}

impl DynamicGumpLogic for CraftGumpLogic {
    const TYPE_NAME: &'static str = "Server.Engines.Craft.CraftGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        let sys = &self.sys;
        b.add_page(0);
        b.add_background(0, 0, 530, 437, 5054);
        b.add_image_tiled(10, 10, 510, 22, 2624);
        b.add_image_tiled(10, 292, 150, 45, 2624);
        b.add_image_tiled(165, 292, 355, 45, 2624);
        b.add_image_tiled(10, 342, 510, 85, 2624);
        b.add_image_tiled(10, 37, 200, 250, 2624);
        b.add_image_tiled(215, 37, 305, 250, 2624);
        b.add_alpha_region(10, 10, 510, 417);

        if sys.gump_title.number > 0 {
            b.add_html_localized_color(
                10,
                12,
                510,
                20,
                sys.gump_title.number,
                LABEL_COLOR,
                false,
                false,
            );
        } else {
            b.add_html(
                10,
                12,
                510,
                20,
                sys.gump_title.string.as_deref().unwrap_or(""),
                false,
                false,
            );
        }

        b.add_html_localized_color(10, 37, 200, 22, 1044010, LABEL_COLOR, false, false); // <CENTER>CATEGORIES</CENTER>
        b.add_html_localized_color(215, 37, 305, 22, 1044011, LABEL_COLOR, false, false); // <CENTER>SELECTIONS</CENTER>
        b.add_html_localized_color(10, 302, 150, 25, 1044012, LABEL_COLOR, false, false); // <CENTER>NOTICES</CENTER>

        b.add_button(15, 402, 4017, 4019, 0);
        b.add_html_localized_color(50, 405, 150, 18, 1011441, LABEL_COLOR, false, false); // EXIT

        b.add_button(270, 402, 4005, 4007, button_id(6, 2));
        b.add_html_localized_color(305, 405, 150, 18, 1044013, LABEL_COLOR, false, false); // MAKE LAST

        // Mark option
        if sys.mark_option {
            b.add_button(270, 362, 4005, 4007, button_id(6, 6));
            let mark = self
                .ctx
                .as_ref()
                .map_or(CraftMarkOption::MarkItem, |c| c.mark_option) as i32;
            b.add_html_localized_color(
                305,
                365,
                150,
                18,
                1044017 + mark,
                LABEL_COLOR,
                false,
                false,
            ); // MARK ITEM
        }
        // Resmelt option
        if sys.resmelt {
            b.add_button(15, 342, 4005, 4007, button_id(6, 1));
            b.add_html_localized_color(50, 345, 150, 18, 1044259, LABEL_COLOR, false, false); // SMELT ITEM
        }
        // Repair option
        if sys.repair {
            b.add_button(270, 342, 4005, 4007, button_id(6, 5));
            b.add_html_localized_color(305, 345, 150, 18, 1044260, LABEL_COLOR, false, false); // REPAIR ITEM
        }
        // Enhance option
        if sys.can_enhance {
            b.add_button(270, 382, 4005, 4007, button_id(6, 8));
            b.add_html_localized_color(305, 385, 150, 18, 1061001, LABEL_COLOR, false, false); // ENHANCE ITEM
        }

        add_td_text(
            b,
            &self.notice,
            170,
            295,
            350,
            40,
            false,
            false,
            LABEL_COLOR,
            FONT_COLOR,
        );

        // If the system has more than one resource
        if let Some(row) = &self.res {
            res_line(b, 362, button_id(6, 0), row);
        }
        // For dragon scales
        if let Some(row) = &self.res2 {
            res_line(b, 382, button_id(6, 7), row);
        }

        self.create_group_list(b);

        match self.page {
            CraftPage::PickResource | CraftPage::PickResource2 => self.create_res_list(b),
            CraftPage::None => {
                if let Some(c) = &self.ctx
                    && c.last_group_index > -1
                {
                    self.create_item_list(b, c.last_group_index);
                }
            }
        }
    }

    fn on_response(&mut self, w: &mut World, _sender: NetStateId, info: &RelayInfo) {
        if info.button_id <= 0 {
            return; // Canceled
        }
        let button = info.button_id - 1;
        let kind = button % 7;
        let index = button / 7;
        self.respond(w, kind, index);
    }
}

impl CraftGumpLogic {
    fn resend(&self, w: &mut World, notice: Option<Td>, page: CraftPage) {
        send_craft_gump(w, self.from, &self.sys, self.tool, notice, page);
    }

    /// `CraftGump.CraftItem(item)`.
    fn craft_item(&self, w: &mut World, item: usize) {
        let sys = self.sys.clone();
        let ty = sys.items[item].item_type;
        let num = (sys.vt.can_craft)(w, self.from, self.tool, Some(ty));
        if num > 0 {
            self.resend(w, Some(Td::of_number(num)), CraftPage::None);
            return;
        }
        let mut type_res = None;
        if let Some(c) = context_of(w, sys.name, self.from) {
            let ci = &sys.items[item];
            let (res, res_index) = if ci.use_sub_res2 {
                (&sys.sub_res2, c.last_resource_index2)
            } else {
                (&sys.sub_res, c.last_resource_index)
            };
            if res_index >= 0 && (res_index as usize) < res.items.len() {
                type_res = Some(res.items[res_index as usize].item_type);
            }
        }
        // CraftSystem.CreateItem: only items the system lists.
        if sys.search_for(ty).is_some() {
            craft_item::craft(w, self.from, &sys, item, type_res, self.tool, None);
        }
    }

    fn respond(&mut self, w: &mut World, kind: i32, index: i32) {
        let sys = self.sys.clone();
        let from = self.from;
        let ctx = context_of(w, sys.name, from);
        let has_ctx = ctx.is_some();
        match kind {
            0 => {
                // Show group
                if has_ctx && index >= 0 && (index as usize) < sys.groups.len() {
                    modify_context(w, sys.name, from, |c| c.last_group_index = index);
                    self.resend(w, None, CraftPage::None);
                }
            }
            1 => {
                // Create item
                let Some(c) = ctx else { return };
                let g = c.last_group_index;
                if g >= 0 && (g as usize) < sys.groups.len() {
                    let group = &sys.groups[g as usize];
                    if index >= 0 && (index as usize) < group.items.len() {
                        self.craft_item(w, group.items[index as usize]);
                    }
                }
            }
            2 => {
                // Item details
                let Some(c) = ctx else { return };
                let g = c.last_group_index;
                if g >= 0 && (g as usize) < sys.groups.len() {
                    let group = &sys.groups[g as usize];
                    if index >= 0 && (index as usize) < group.items.len() {
                        send_craft_item_gump(w, from, &sys, group.items[index as usize], self.tool);
                    }
                }
            }
            3 => {
                // Create item (last 10)
                let Some(c) = ctx else { return };
                if index >= 0 && (index as usize) < c.items.len() {
                    self.craft_item(w, c.items[index as usize]);
                }
            }
            4 => {
                // Item details (last 10)
                let Some(c) = ctx else { return };
                if index >= 0 && (index as usize) < c.items.len() {
                    send_craft_item_gump(w, from, &sys, c.items[index as usize], self.tool);
                }
            }
            5 => self.resource_selected(w, index),
            6 => self.misc_button(w, index),
            _ => {}
        }
    }

    fn resource_selected(&mut self, w: &mut World, index: i32) {
        let sys = self.sys.clone();
        let from = self.from;
        let (col, second) = match self.page {
            CraftPage::PickResource => (&sys.sub_res, false),
            CraftPage::PickResource2 => (&sys.sub_res2, true),
            CraftPage::None => return,
        };
        if index < 0 || index as usize >= col.items.len() {
            return;
        }
        let res = &col.items[index as usize];
        if skills::base(w, from, sys.main_skill) < res.required_skill {
            self.resend(w, Some(res.message.clone()), CraftPage::None);
        } else {
            modify_context(w, sys.name, from, |c| {
                if second {
                    c.last_resource_index2 = index;
                } else {
                    c.last_resource_index = index;
                }
            });
            self.resend(w, None, CraftPage::None);
        }
    }

    fn misc_button(&mut self, w: &mut World, index: i32) {
        let sys = self.sys.clone();
        let from = self.from;
        let ctx = context_of(w, sys.name, from);
        match index {
            0 => {
                // Resource selection
                if sys.sub_res.init {
                    self.resend(w, None, CraftPage::PickResource);
                }
            }
            1 => {
                // Smelt item
                if sys.resmelt {
                    super::resmelt::do_resmelt(w, from, &sys, self.tool);
                }
            }
            2 => {
                // Make last
                let Some(c) = ctx else { return };
                match c.last_made() {
                    Some(item) => self.craft_item(w, item),
                    None => self.resend(w, Some(Td::of_number(1044165)), self.page), // You haven't made anything yet.
                }
            }
            3 => {
                // Last 10
                if ctx.is_some() {
                    modify_context(w, sys.name, from, |c| c.last_group_index = 501);
                    self.resend(w, None, CraftPage::None);
                }
            }
            4 => {
                // Toggle use resource hue
                if ctx.is_some() {
                    modify_context(w, sys.name, from, |c| c.do_not_color = !c.do_not_color);
                    self.resend(w, None, self.page);
                }
            }
            5 => {
                // Repair item
                if sys.repair {
                    super::repair::do_repair(w, from, &sys, self.tool);
                }
            }
            6 => {
                // Toggle mark option
                if ctx.is_none() || !sys.mark_option {
                    return;
                }
                modify_context(w, sys.name, from, |c| {
                    c.mark_option = match c.mark_option {
                        CraftMarkOption::MarkItem => CraftMarkOption::DoNotMark,
                        CraftMarkOption::DoNotMark => CraftMarkOption::PromptForMark,
                        CraftMarkOption::PromptForMark => CraftMarkOption::MarkItem,
                    }
                });
                self.resend(w, None, self.page);
            }
            7 => {
                // Resource selection 2
                if sys.sub_res2.init {
                    self.resend(w, None, CraftPage::PickResource2);
                }
            }
            8
                // Enhance item
                if sys.can_enhance => {
                    super::enhance::begin_target(w, from, &sys, self.tool);
                }
            _ => {}
        }
    }
}

// ---- CraftGumpItem ----

/// One resource line of the item gump.
struct ItemResLine {
    name: Td,
    amount: i32,
    retains_color: bool,
}

/// `CraftGumpItem`.
pub struct CraftGumpItemLogic {
    from: MobileId,
    sys: Rc<CraftSystem>,
    index: usize,
    tool: Option<ItemId>,
    item_id: i32,
    needs_recipe: bool,
    markable: bool,
    chance: f64,
    exceptional: f64,
    supports_expansion: bool,
    /// `ExpansionInfo.GetInfo(expansion).Name`, for the expansions with no cliloc.
    expansion_name: String,
    lines: Vec<ItemResLine>,
    crop_scroll: bool,
}

/// `CraftGumpItem.RequiredExpansionMessage(expansion)`; `name` is the expansion's `ExpansionInfo` name.
pub(super) fn required_expansion_message(expansion: Expansion, name: &str) -> Td {
    match expansion {
        Expansion::SE => Td::of_number(1063363), // * Requires the "Samurai Empire" expansion
        Expansion::ML => Td::of_number(1072651), // * Requires the "Mondain's Legacy" expansion
        _ => Td::of_text(format!("* Requires the \"{name}\" expansion")),
    }
}

/// `new CraftGumpItem(from, craftSystem, craftItem, tool)` sent to `from`.
pub fn send_craft_item_gump(
    w: &mut World,
    from: MobileId,
    sys: &Rc<CraftSystem>,
    index: usize,
    tool: Option<ItemId>,
) {
    let item = sys.items[index].clone();
    let is_player = w
        .mobile(from)
        .is_some_and(|m| m.ty.is(&muo_content::player_mobile::PLAYER_MOBILE));
    let needs_recipe = item
        .recipe
        .is_some_and(|r| is_player && !muo_content::player_mobile::has_recipe(w, from, r));
    let item_id = craft_item::craft_item_id(w, &item);
    let markable = item.is_markable(w, item.item_type);

    let ctx = context_of(w, sys.name, from);
    let col = if item.use_sub_res2 {
        &sys.sub_res2
    } else {
        &sys.sub_res
    };
    let res_index = ctx.as_ref().map_or(-1, |c| {
        if item.use_sub_res2 {
            c.last_resource_index2
        } else {
            c.last_resource_index
        }
    });

    let mut all_required = true;
    let chance = craft_item::success_chance(w, sys, &item, from, false, &mut all_required);
    let exceptional = if markable {
        craft_item::exceptional_chance(w, sys, &item, chance, from).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let chance = chance.clamp(0.0, 1.0);

    let crop_scroll = item.resources.len() > 1
        && item
            .resources
            .last()
            .is_some_and(|r| r.item_type == "Server.Items.BlankScroll")
        && craft_item::derives(w, item.item_type, "Server.Items.SpellScroll");

    let mut lines = Vec::new();
    let count = item.resources.len() - usize::from(crop_scroll);
    for i in 0..count.min(4) {
        let cr = &item.resources[i];
        let mut ty = cr.item_type;
        let mut name = craft_item::res_name(w, cr);
        // Resource Mutation
        if ty == col.res_type
            && res_index > -1
            && let Some(sub) = col.items.get(res_index as usize)
        {
            ty = sub.item_type;
            name = if sub.generic_name_number > 0 {
                Td::of_number(sub.generic_name_number)
            } else {
                sub.name.clone()
            };
        }
        lines.push(ItemResLine {
            name,
            amount: cr.amount,
            retains_color: craft_item::retains_color_from(w, sys, &item, ty),
        });
    }

    let logic = CraftGumpItemLogic {
        from,
        sys: sys.clone(),
        index,
        tool,
        item_id,
        needs_recipe,
        markable,
        chance,
        exceptional,
        supports_expansion: item.required_expansion != Expansion::None
            && craft_item::supports_expansion(w, from, item.required_expansion),
        expansion_name: w.expansion_table().map_or_else(
            || item.required_expansion.name().to_owned(),
            |t| t.get_info(item.required_expansion.index()).name.clone(),
        ),
        lines,
        crop_scroll,
    };
    send_gump_to_mobile(w, from, Box::new(DynamicGumpObj::new(40, 40, logic)), false);
}

impl DynamicGumpLogic for CraftGumpItemLogic {
    const TYPE_NAME: &'static str = "Server.Engines.Craft.CraftGumpItem";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        let item = &self.sys.items[self.index];
        b.add_page(0);
        b.add_background(0, 0, 530, 417, 5054);
        b.add_image_tiled(10, 10, 510, 22, 2624);
        b.add_image_tiled(10, 37, 150, 148, 2624);
        b.add_image_tiled(165, 37, 355, 90, 2624);
        b.add_image_tiled(10, 190, 155, 22, 2624);
        b.add_image_tiled(10, 217, 150, 53, 2624);
        b.add_image_tiled(165, 132, 355, 80, 2624);
        b.add_image_tiled(10, 275, 155, 22, 2624);
        b.add_image_tiled(10, 302, 150, 53, 2624);
        b.add_image_tiled(165, 217, 355, 80, 2624);
        b.add_image_tiled(10, 360, 155, 22, 2624);
        b.add_image_tiled(165, 302, 355, 80, 2624);
        b.add_image_tiled(10, 387, 510, 22, 2624);
        b.add_alpha_region(10, 10, 510, 399);

        b.add_html_localized_color(170, 40, 150, 20, 1044053, LABEL_COLOR, false, false); // ITEM
        b.add_html_localized_color(10, 192, 150, 22, 1044054, LABEL_COLOR, false, false); // <CENTER>SKILLS</CENTER>
        b.add_html_localized_color(10, 277, 150, 22, 1044055, LABEL_COLOR, false, false); // <CENTER>MATERIALS</CENTER>
        b.add_html_localized_color(10, 362, 150, 22, 1044056, LABEL_COLOR, false, false); // <CENTER>OTHER</CENTER>

        let title = &self.sys.gump_title;
        if title.number > 0 {
            b.add_html_localized_color(10, 12, 510, 20, title.number, LABEL_COLOR, false, false);
        } else {
            b.add_html(
                10,
                12,
                510,
                20,
                title.string.as_deref().unwrap_or(""),
                false,
                false,
            );
        }

        b.add_button(15, 387, 4014, 4016, 0);
        b.add_html_localized_color(50, 390, 150, 18, 1044150, LABEL_COLOR, false, false); // BACK

        if self.needs_recipe {
            b.add_button_type(270, 387, 4005, 4007, 0, GumpButtonType::Page, 0);
            b.add_html_localized_color(305, 390, 150, 18, 1044151, GREY_LABEL_COLOR, false, false); // MAKE NOW
        } else {
            b.add_button(270, 387, 4005, 4007, 1);
            b.add_html_localized_color(305, 390, 150, 18, 1044151, LABEL_COLOR, false, false); // MAKE NOW
        }

        if item.name.number > 0 {
            b.add_html_localized_color(
                330,
                40,
                180,
                18,
                item.name.number,
                LABEL_COLOR,
                false,
                false,
            );
        } else {
            b.add_label(330, 40, LABEL_HUE, item.name_string());
        }

        if item.use_all_res {
            b.add_html_localized_color(170, 302, 310, 18, 1048176, LABEL_COLOR, false, false); // Makes as many as possible at once
        }

        let mut other_count = 1;

        // DrawItem
        b.add_item(20, 50, self.item_id, item.item_hue);
        if self.markable {
            b.add_html_localized_color(
                170,
                302 + other_count * 20,
                310,
                18,
                1044059,
                LABEL_COLOR,
                false,
                false,
            ); // This item may hold its maker's mark
            other_count += 1;
        }

        // DrawSkill
        for (i, s) in item.skills.iter().enumerate() {
            let i = i as i32;
            let min = s.min_skill.max(0.0);
            b.add_html_localized_color(
                170,
                132 + i * 20,
                200,
                18,
                skill_label(s.skill),
                LABEL_COLOR,
                false,
                false,
            );
            b.add_label(430, 132 + i * 20, LABEL_HUE, &format!("{min:.1}"));
        }
        b.add_html_localized_color(170, 80, 250, 18, 1044057, LABEL_COLOR, false, false); // Success Chance:
        b.add_label(430, 80, LABEL_HUE, &format!("{:.1}%", self.chance * 100.0));
        if self.markable {
            b.add_html_localized_color(170, 100, 250, 18, 1044058, 32767, false, false); // Exceptional Chance:
            b.add_label(
                430,
                100,
                LABEL_HUE,
                &format!("{:.1}%", self.exceptional * 100.0),
            );
        }

        // DrawResource
        let mut retained_color = false;
        for (i, line) in self.lines.iter().enumerate() {
            let i = i as i32;
            if !retained_color && line.retains_color {
                retained_color = true;
                b.add_html_localized_color(
                    170,
                    302 + other_count * 20,
                    310,
                    18,
                    1044152,
                    LABEL_COLOR,
                    false,
                    false,
                ); // * The item retains the color of this material
                other_count += 1;
                b.add_label(500, 219 + i * 20, LABEL_HUE, "*");
            }
            if line.name.number > 0 {
                b.add_html_localized_color(
                    170,
                    219 + i * 20,
                    310,
                    18,
                    line.name.number,
                    LABEL_COLOR,
                    false,
                    false,
                );
            } else {
                b.add_label(
                    170,
                    219 + i * 20,
                    LABEL_HUE,
                    line.name.string.as_deref().unwrap_or(""),
                );
            }
            b.add_label(430, 219 + i * 20, LABEL_HUE, &line.amount.to_string());
        }
        if item.name.number == 1041267 {
            // runebook
            let y = 219 + item.resources.len() as i32 * 20;
            b.add_html_localized_color(170, y, 310, 18, 1044447, LABEL_COLOR, false, false);
            b.add_label(430, y, LABEL_HUE, "1");
        }
        if self.crop_scroll {
            b.add_html_localized_color(
                170,
                302 + other_count * 20,
                360,
                18,
                1044379,
                LABEL_COLOR,
                false,
                false,
            ); // Inscribing scrolls also requires a blank scroll and mana.
            other_count += 1;
        }

        if item.required_expansion != Expansion::None {
            let text = required_expansion_message(item.required_expansion, &self.expansion_name);
            let (nc, sc) = if self.supports_expansion {
                (LABEL_COLOR, LABEL_HUE)
            } else {
                (RED_LABEL_COLOR, RED_LABEL_HUE)
            };
            add_td_text(
                b,
                &text,
                170,
                302 + other_count * 20,
                310,
                18,
                false,
                false,
                nc,
                sc,
            );
            other_count += 1;
        }

        if self.needs_recipe {
            b.add_html_localized_color(
                170,
                302 + other_count * 20,
                310,
                18,
                1073620,
                RED_LABEL_COLOR,
                false,
                false,
            ); // You have not learned this recipe.
        }
    }

    fn on_response(&mut self, w: &mut World, _sender: NetStateId, info: &RelayInfo) {
        let from = self.from;
        let sys = self.sys.clone();
        if info.button_id == 0 {
            // Back Button
            craft_item::show_craft_menu(w, from, &sys, self.tool, None);
            return;
        }
        // Make Button
        let ty = sys.items[self.index].item_type;
        let num = (sys.vt.can_craft)(w, from, self.tool, Some(ty));
        if num > 0 {
            craft_item::show_craft_menu(w, from, &sys, self.tool, Some(Td::of_number(num)));
            return;
        }
        let mut type_res = None;
        if let Some(c) = context_of(w, sys.name, from) {
            let item = &sys.items[self.index];
            let (res, res_index) = if item.use_sub_res2 {
                (&sys.sub_res2, c.last_resource_index2)
            } else {
                (&sys.sub_res, c.last_resource_index)
            };
            if res_index > -1
                && let Some(sub) = res.items.get(res_index as usize)
            {
                type_res = Some(sub.item_type);
            }
        }
        if sys.search_for(ty).is_some() {
            craft_item::craft(w, from, &sys, self.index, type_res, self.tool, None);
        }
    }
}

/// `AosSkillBonuses.GetLabel(skill)`.
fn skill_label(skill: muo_server::skills::SkillName) -> i32 {
    muo_content::aos::skill_label(skill)
}

// ---- QueryMakersMarkGump ----

/// `QueryMakersMarkGump`.
pub struct QueryMakersMarkLogic {
    from: MobileId,
    sys: Rc<CraftSystem>,
    index: usize,
    quality: i32,
    type_res: Option<&'static str>,
    tool: Option<ItemId>,
    res_hue: Option<i32>,
}

/// `new QueryMakersMarkGump(quality, craftItem, craftSystem, typeRes, tool, resHue)` sent to `from`.
#[allow(clippy::too_many_arguments)]
pub fn send_query_makers_mark(
    w: &mut World,
    from: MobileId,
    sys: &Rc<CraftSystem>,
    index: usize,
    quality: i32,
    type_res: Option<&'static str>,
    tool: Option<ItemId>,
    res_hue: Option<i32>,
) {
    let logic = QueryMakersMarkLogic {
        from,
        sys: sys.clone(),
        index,
        quality,
        type_res,
        tool,
        res_hue,
    };
    send_gump_to_mobile(
        w,
        from,
        Box::new(StaticGumpObj::new(100, 200, logic)),
        false,
    );
}

impl StaticGumpLogic for QueryMakersMarkLogic {
    const TYPE_NAME: &'static str = "Server.Engines.Craft.QueryMakersMarkGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        b.add_page(0);
        b.add_background(0, 0, 220, 170, 5054);
        b.add_background(10, 10, 200, 150, 3000);
        b.add_html_localized(20, 20, 180, 80, 1018317, false, false); // Do you wish to place your maker's mark on this item?
        b.add_html_localized(55, 100, 140, 25, 1011011, false, false); // CONTINUE
        b.add_button(20, 100, 4005, 4007, 1);
        b.add_html_localized(55, 125, 140, 25, 1011012, false, false); // CANCEL
        b.add_button(20, 125, 4005, 4007, 0);
    }

    fn on_response(&mut self, w: &mut World, _sender: NetStateId, info: &RelayInfo) {
        let from = self.from;
        let makers_mark = info.button_id == 1;
        if makers_mark {
            muo_server::item::deps::send_localized_message(w, from, 501808); // You mark the item.
        } else {
            muo_server::item::deps::send_localized_message(w, from, 501809); // Cancelled mark.
        }
        craft_item::complete_craft(
            w,
            from,
            &self.sys.clone(),
            self.index,
            self.quality,
            makers_mark,
            self.type_res,
            self.tool,
            self.res_hue,
            None,
        );
    }
}
