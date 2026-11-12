//! `AddGump`, `CategorizedAddGump`, `CAGNode`, `CAGCategory`, `CAGObject` and `CAGLoader`: the search menu (`[AddMenu`) and the categorized menu `[add` opens with no arguments.
//!
//! The category tree is `Data/categorization.json`, loaded on first use into a resource ([`CagTree`]) with the
//! categories in an arena. `CAGLoader.Load` instantiates each object's class to find its graphic and hue, and a
//! category's objects replace whatever nodes it already had (a category that is both a leaf and a parent
//! of another entry loses its sub-categories: kept).

use muo_primitives::dotnet::InvariantCase;
use std::rc::Rc;

use muo_content::gumps::GumpButtonType;
use muo_content::gumps::base::{
    DynamicGumpLogic, DynamicGumpObj, LegacyGumpLogic, LegacyGumpObj, RelayInfo,
};
use muo_content::gumps::builder::DynamicGumpBuilder;
use muo_content::gumps::legacy::LegacyLayout;
use muo_server::commands::{CommandEventArgs, CommandSystem};
use muo_server::ids::{EntityId, MobileId};
use muo_server::mobile::AccessLevel;
use muo_server::mobile::part2::messages::send_message;
use muo_server::net_state::NetStateId;
use muo_server::target::{self, Target, TargetCancelType, TargetFlags, Targeted};
use muo_server::type_registry::EntityType;
use muo_server::world::World;

use crate::hooks::CommandHooks;
use muo_content::gumps::base_grid::{mobile_of, send};
use muo_content::gumps::props_config::*;
use muo_content::gumps::props_layout::{
    Nav, add_props_frame_h, add_props_header_with_back, next_row_h,
};

pub fn configure(w: &mut World) {
    CommandSystem::register(
        w,
        "AddMenu",
        AccessLevel::GameMaster,
        &[],
        Rc::new(add_menu_on_command),
    );
    // The menus `Add` opens are these.
    let h = w.resources.get_mut::<CommandHooks>();
    if h.add_gump.is_none() {
        h.add_gump = Some(|w, from, search, categorized| {
            if categorized {
                open_categorized(w, from);
            } else {
                open_add_gump_search(w, from, search);
            }
        });
    }
}

fn type_name(t: EntityType) -> &'static str {
    let full = t.full_name();
    full.rsplit(['.', '+']).next().unwrap_or(full)
}

// ---- AddGump ---------------------------------------------------------------------------------------------------

/// A `ConstructorInfo` of the search results: the class, and the index of the constructor in its table (`None` for
/// the parameterless one of a class without a table).
#[derive(Copy, Clone)]
pub struct SearchResult {
    pub ty: EntityType,
    pub ctor: Option<usize>,
}

/// `AddGump.MatchEmptyCtor(match)`: every item and mobile class whose name contains the text and that has a public
/// `[Constructible]` constructor with no required parameter (the first such), sorted by class name.
pub fn match_empty_ctor(w: &World, text: &str) -> Vec<SearchResult> {
    if text.trim().is_empty() {
        return Vec::new();
    }
    let text = text.lower_inv();
    let mut results = Vec::new();
    let types = w
        .types
        .item_types()
        .iter()
        .map(|t| EntityType::Item(t))
        .chain(w.types.mobile_types().iter().map(|t| EntityType::Mobile(t)));
    for t in types {
        if !type_name(t).lower_inv().contains(&text) {
            continue;
        }
        match muo_content::props::ctors_of(w, t.full_name()) {
            Some(ctors) => {
                if let Some(i) = ctors
                    .iter()
                    .position(|c| c.params.iter().all(|p| p.default.is_some()))
                {
                    results.push(SearchResult {
                        ty: t,
                        ctor: Some(i),
                    });
                }
            }
            None if t.construct_available() => results.push(SearchResult { ty: t, ctor: None }),
            None => {}
        }
    }
    // `Array.Sort` with `ConstructorNameComparer`: an unstable introsort; `net_sort` keeps its order among ties.
    muo_primitives::dotnet::net_sort(&mut results, |a, b| type_name(a.ty).cmp(type_name(b.ty)));
    results
}

pub struct AddGump {
    page: usize,
    search_results: Vec<SearchResult>,
    search_string: String,
    explicit_search: bool,
}

pub fn new_add_gump(
    search: &str,
    page: usize,
    results: Vec<SearchResult>,
    explicit: bool,
) -> DynamicGumpObj<AddGump> {
    DynamicGumpObj::new(
        50,
        50,
        AddGump {
            page,
            search_results: results,
            search_string: search.to_string(),
            explicit_search: explicit,
        },
    )
}

impl DynamicGumpLogic for AddGump {
    const TYPE_NAME: &'static str = "Server.Gumps.AddGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        b.add_page(0);
        b.add_background(0, 0, 420, 280, 5054);
        b.add_image_tiled(10, 10, 400, 20, 2624);
        b.add_alpha_region(10, 10, 400, 20);
        b.add_image_tiled(41, 11, 184, 18, 0xBBC);
        b.add_image_tiled(42, 12, 182, 16, 2624);
        b.add_alpha_region(42, 12, 182, 16);
        b.add_button(10, 9, 4011, 4013, 1);
        b.add_text_entry(44, 10, 180, 20, 0x480, 0, &self.search_string);
        b.add_html_localized_color(230, 10, 100, 20, 3010005, 0x7FFF, false, false);
        b.add_image_tiled(10, 40, 400, 200, 2624);
        b.add_alpha_region(10, 40, 400, 200);
        let n = self.search_results.len();
        if n > 0 {
            let mut i = self.page * 10;
            while i < (self.page + 1) * 10 && i < n {
                let index = (i % 10) as i32;
                b.add_label(
                    44,
                    39 + index * 20,
                    0x480,
                    type_name(self.search_results[i].ty),
                );
                b.add_button(10, 39 + index * 20, 4023, 4025, 4 + i as i32);
                i += 1;
            }
        } else {
            b.add_label(
                15,
                44,
                0x480,
                if self.explicit_search {
                    "Nothing matched your search terms."
                } else {
                    "No results to display."
                },
            );
        }
        b.add_image_tiled(10, 250, 400, 20, 2624);
        b.add_alpha_region(10, 250, 400, 20);
        if self.page > 0 {
            b.add_button(10, 249, 4014, 4016, 2);
        } else {
            b.add_image(10, 249, 4014, 0);
        }
        b.add_html_localized_color(
            44,
            250,
            170,
            20,
            1061028, // Previous page
            if self.page > 0 { 0x7FFF } else { 0x5EF7 },
            false,
            false,
        );
        let has_next = (self.page + 1) * 10 < n;
        if has_next {
            b.add_button(210, 249, 4005, 4007, 3);
        } else {
            b.add_image(210, 249, 4005, 0);
        }
        b.add_html_localized_color(
            244,
            250,
            170,
            20,
            1061027, // Next page
            if has_next { 0x7FFF } else { 0x5EF7 },
            false,
            false,
        );
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let Some(from) = mobile_of(w, sender) else {
            return;
        };
        let n = self.search_results.len();
        match info.button_id {
            1 => {
                // Search
                let m = info.get_text_entry(0).unwrap_or("").trim().to_string();
                if m.chars().count() < 3 {
                    send_message(w, from, "Invalid search string.");
                    send(
                        w,
                        from,
                        new_add_gump(&m, self.page, self.search_results.clone(), false),
                    );
                } else {
                    let results = match_empty_ctor(w, &m);
                    send(w, from, new_add_gump(&m, 0, results, true));
                }
            }
            2 => {
                if self.page > 0 {
                    send(
                        w,
                        from,
                        new_add_gump(
                            &self.search_string,
                            self.page - 1,
                            self.search_results.clone(),
                            true,
                        ),
                    );
                }
            }
            3 => {
                if (self.page + 1) * 10 < n {
                    send(
                        w,
                        from,
                        new_add_gump(
                            &self.search_string,
                            self.page + 1,
                            self.search_results.clone(),
                            true,
                        ),
                    );
                }
            }
            b => {
                let index = b - 4;
                if index >= 0 && (index as usize) < n {
                    send_message(
                        w,
                        from,
                        "Where do you wish to place this object? <ESC> to cancel.",
                    );
                    begin_internal_target(
                        w,
                        from,
                        self.search_results[index as usize],
                        self.search_results.clone(),
                        self.search_string.clone(),
                        self.page,
                    );
                }
            }
        }
    }
}

/// `AddGump.InternalTarget`.
fn begin_internal_target(
    w: &mut World,
    from: MobileId,
    ctor: SearchResult,
    results: Vec<SearchResult>,
    search: String,
    page: usize,
) {
    let mut t = Target::new(w, -1, true, TargetFlags::NONE);
    {
        let (results, search) = (results.clone(), search.clone());
        t.handlers.on_target = Some(Rc::new(move |w, from, o| {
            let p = match o {
                Targeted::Entity(EntityId::Item(i)) => muo_server::item::ops::get_world_top(w, i),
                Targeted::Entity(EntityId::Mobile(m)) => {
                    w.mobile(m).map(|m| m.location).unwrap_or_default()
                }
                Targeted::Land(l) => l.location,
                Targeted::Static(s) => s.location,
            };
            crate::add::invoke_ctor(w, from, p, p, ctor.ty, ctor.ctor);
            begin_internal_target(w, from, ctor, results.clone(), search.clone(), page);
        }));
    }
    t.handlers.on_target_cancel = Some(Rc::new(move |w, from, cancel| {
        if cancel == TargetCancelType::Canceled {
            send(w, from, new_add_gump(&search, page, results.clone(), true));
        }
    }));
    let id = target::add_target(w, t);
    target::set_mobile_target(w, from, Some(id));
}

/// `[Usage("AddMenu [searchString]")]`
fn add_menu_on_command(w: &mut World, e: &mut CommandEventArgs) {
    let val = e.arg_string.trim().to_string();
    let (results, explicit) = if val.is_empty() {
        (Vec::new(), false)
    } else if val.chars().count() < 3 {
        send_message(w, e.mobile, "Invalid search string.");
        (Vec::new(), false)
    } else {
        (match_empty_ctor(w, &val), true)
    };
    send(w, e.mobile, new_add_gump(&val, 0, results, explicit));
}

/// `new AddGump(search, 0, matches, ..)` for `[add <text>` with no exact match: `explicitSearch` is false there.
pub fn open_add_gump_search(w: &mut World, from: MobileId, search: &str) {
    let results = if search.chars().count() >= 3 {
        match_empty_ctor(w, search)
    } else {
        Vec::new()
    };
    send(w, from, new_add_gump(search, 0, results, false));
}

// ---- CAG tree --------------------------------------------------------------------------------------------------

#[derive(Clone)]
pub struct CagObject {
    pub ty: Option<EntityType>,
    pub item_id: Option<i32>,
    pub hue: Option<i32>,
    pub parent: usize,
}

impl CagObject {
    /// `Title`.
    pub fn title(&self) -> String {
        self.ty
            .map_or_else(|| "bad type".to_string(), |t| type_name(t).to_string())
    }
}

#[derive(Clone)]
pub enum CagNode {
    Category(usize),
    Object(CagObject),
}

#[derive(Clone)]
pub struct CagCategory {
    pub title: String,
    pub parent: Option<usize>,
    pub nodes: Option<Vec<CagNode>>,
}

#[derive(Default)]
pub struct CagTree {
    pub cats: Vec<CagCategory>,
    loaded: bool,
}

impl CagTree {
    fn title_of(&self, n: &CagNode) -> String {
        match n {
            CagNode::Category(c) => self.cats[*c].title.clone(),
            CagNode::Object(o) => o.title(),
        }
    }
}

/// `CAGCategory.Root`: loaded on first use. Category 0 is the root.
fn ensure_loaded(w: &mut World) {
    if w.resources.get_mut::<CagTree>().loaded {
        return;
    }
    let tree = load(w);
    *w.resources.get_mut::<CagTree>() = tree;
}

/// `CAGLoader.Load`.
fn load(w: &mut World) -> CagTree {
    let mut tree = CagTree {
        cats: vec![CagCategory {
            title: "Add Menu".into(),
            parent: None,
            nodes: None,
        }],
        loaded: true,
    };
    let path = w.core.base_directory.join("Data/categorization.json");
    let text = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(err) => {
            // `JsonConfig.Deserialize` throws; the menu cannot be built.
            tracing::error!("Failed to deserialize {}: {err}", path.display());
            return tree;
        }
    };
    let list: serde_json::Value =
        match muo_data::json::from_str(text.trim_start_matches('\u{feff}')) {
            Ok(v) => v,
            Err(err) => {
                tracing::error!("Failed to deserialize {}: {err}", path.display());
                return tree;
            }
        };
    let Some(list) = list.as_array() else {
        tracing::error!("Failed to deserialize {}.", path.display());
        return tree;
    };
    for cag in list {
        let category = cag.get("category").and_then(|c| c.as_str()).unwrap_or("");
        let mut parent = 0usize;
        // Navigate through the dot notation categories until we find the last one
        for name in category.split('.') {
            if tree.cats[parent].nodes.is_none() {
                // No children, so let's make one
                tree.cats.push(CagCategory {
                    title: name.to_string(),
                    parent: Some(parent),
                    nodes: None,
                });
                let id = tree.cats.len() - 1;
                tree.cats[parent].nodes = Some(vec![CagNode::Category(id)]);
                parent = id;
                continue;
            }
            let old_parent = parent;
            let found = tree.cats[parent]
                .nodes
                .iter()
                .flatten()
                .find_map(|n| match n {
                    CagNode::Category(c) if tree.cats[*c].title == name => Some(*c),
                    _ => None,
                });
            if let Some(c) = found {
                parent = c;
            }
            // Didn't find the child, let's add it
            if old_parent == parent {
                tree.cats.push(CagCategory {
                    title: name.to_string(),
                    parent: Some(parent),
                    nodes: None,
                });
                let id = tree.cats.len() - 1;
                tree.cats[parent]
                    .nodes
                    .get_or_insert_with(Vec::new)
                    .push(CagNode::Category(id));
                parent = id;
            }
        }
        // Set the objects associated with the child most node
        let mut nodes: Vec<CagNode> = Vec::new();
        let objects = cag
            .get("objects")
            .and_then(|o| o.as_array())
            .cloned()
            .unwrap_or_default();
        for obj in &objects {
            let name = obj.get("type").and_then(|t| t.as_str()).unwrap_or("");
            // TypeConverter: `FindTypeByName(name) ?? FindTypeByName(name, true)` (short name, then full name, both ignoring case)
            let ty = w
                .types
                .find_type_by_name(name, false, true)
                .or_else(|| w.types.find_type_by_name(name, true, true));
            if ty.is_none() {
                tracing::warn!("Attempted to deserialize type {name} which does not exist.");
            }
            let mut o = CagObject {
                ty,
                item_id: obj.get("gfx").and_then(|v| v.as_i64()).map(|v| v as i32),
                hue: obj.get("hue").and_then(|v| v.as_i64()).map(|v| v as i32),
                parent,
            };
            // Set ItemID and Hue
            if o.hue.is_none() || o.item_id.is_none() {
                // A null type is dropped, as for a constructor failure.
                let Some(t) = ty else { continue };
                if !fill_graphic(w, t, &mut o) {
                    continue;
                }
            }
            nodes.push(CagNode::Object(o));
        }
        tree.cats[parent].nodes = Some(nodes);
    }
    tree
}

/// The `type.CreateInstance<..>()` half of `CAGLoader`: the graphic and the hue of a throwaway instance. False when
/// it cannot be constructed.
fn fill_graphic(w: &mut World, t: EntityType, o: &mut CagObject) -> bool {
    let hue_of = |hue: i32| {
        let hue = hue & 0x7FFF;
        let hue = if hue & 0x4000 != 0 { 0 } else { hue };
        if hue == 0 { None } else { Some(hue) }
    };
    match t {
        EntityType::Item(ty) => {
            let Some(construct) = ty.construct else {
                return false;
            };
            let item = construct(w);
            if o.item_id.is_none() {
                let mut item_id = w.item(item).map_or(1, |i| i.item_id);
                if muo_content::props::is_class(w, EntityId::Item(item), "Server.Items.BaseAddon") {
                    let comps = muo_content::items::addon::components(w, item);
                    if comps.len() == 1 {
                        item_id = w.item(comps[0]).map_or(item_id, |c| c.item_id);
                    }
                }
                if item_id > muo_data::TileData::MAX_ITEM_VALUE as i32 {
                    item_id = 1;
                }
                o.item_id = Some(item_id);
            }
            if o.hue.is_none() {
                o.hue = hue_of(w.item(item).map_or(0, |i| i.hue));
            }
            muo_server::item::base::delete(w, item);
            true
        }
        EntityType::Mobile(ty) => {
            let Some(construct) = ty.construct else {
                return false;
            };
            let m = construct(w);
            if o.item_id.is_none() {
                let body = w.mobile(m).map_or(0, |x| x.cur_body().0);
                o.item_id = Some(muo_content::ai::transfer::shrink_lookup(w, body, 1));
            }
            if o.hue.is_none() {
                o.hue = hue_of(w.mobile(m).map_or(0, |x| (x.ty.hue)(w, m)));
            }
            muo_server::mobile::base::delete(w, m);
            true
        }
    }
}

// ---- CategorizedAddGump ----------------------------------------------------------------------------------------

const ENTRY_HEIGHT_CAG: i32 = ENTRY_HEIGHT + 4;
const SET_OFFSET_Y_CAG: i32 = SET_OFFSET_Y + (ENTRY_HEIGHT_CAG - 20) / 2 / 2;
const ENTRY_WIDTH: i32 = 180;
const ENTRY_COUNT: usize = 15;
const TOTAL_WIDTH: i32 = OFFSET_SIZE + ENTRY_WIDTH + OFFSET_SIZE + SET_WIDTH + OFFSET_SIZE;

pub struct CategorizedAddGump {
    owner: MobileId,
    category: usize,
    page: usize,
}

/// `new CategorizedAddGump(owner)`.
pub fn open_categorized(w: &mut World, from: MobileId) {
    ensure_loaded(w);
    let g = new_categorized(w, from, 0, 0);
    send(w, from, g);
}

pub fn new_categorized(
    w: &World,
    owner: MobileId,
    category: usize,
    page: usize,
) -> LegacyGumpObj<CategorizedAddGump> {
    let mut g = LegacyGumpObj::new(
        GUMP_OFFSET_X,
        GUMP_OFFSET_Y,
        CategorizedAddGump {
            owner,
            category,
            page,
        },
    );
    let tree = w.resources.get::<CagTree>();
    let empty = CagTree::default();
    let tree = tree.unwrap_or(&empty);
    let Some(cat) = tree.cats.get(category) else {
        return g;
    };
    let nodes = cat.nodes.as_deref().unwrap_or(&[]);
    let count = nodes
        .len()
        .saturating_sub(page * ENTRY_COUNT)
        .min(ENTRY_COUNT);
    let l: &mut LegacyLayout = &mut g.layout;
    l.add_page(0);
    let (mut x, mut y) = add_props_frame_h(l, TOTAL_WIDTH, count as i32 + 1, ENTRY_HEIGHT_CAG);
    add_props_header_with_back(
        l,
        TOTAL_WIDTH,
        &mut x,
        &mut y,
        Some(&cat.title),
        cat.parent.is_some().then_some(1),
        (page > 0).then_some(2),
        ((page + 1) * ENTRY_COUNT < nodes.len()).then_some(3),
        Nav {
            kind: GumpButtonType::Reply,
            param: 1,
        },
        ENTRY_HEIGHT_CAG,
    );
    let mut index = page * ENTRY_COUNT;
    let mut i = 0usize;
    while i < ENTRY_COUNT && index < nodes.len() {
        next_row_h(&mut x, &mut y, ENTRY_HEIGHT_CAG);
        let node = &nodes[index];
        l.add_image_tiled(x, y, ENTRY_WIDTH, ENTRY_HEIGHT_CAG, ENTRY_GUMP_ID);
        l.add_label_cropped(
            x + TEXT_OFFSET_X,
            y + (ENTRY_HEIGHT_CAG - 20) / 2,
            ENTRY_WIDTH - TEXT_OFFSET_X,
            ENTRY_HEIGHT_CAG,
            TEXT_HUE,
            &tree.title_of(node),
        );
        x += ENTRY_WIDTH + OFFSET_SIZE;
        if SET_GUMP_ID != 0 {
            l.add_image_tiled(x, y, SET_WIDTH, ENTRY_HEIGHT_CAG, SET_GUMP_ID);
        }
        l.add_button(
            x + SET_OFFSET_X,
            y + SET_OFFSET_Y_CAG,
            SET_BUTTON_ID1,
            SET_BUTTON_ID2,
            i as i32 + 4,
        );
        if let CagNode::Object(obj) = node {
            let item_id = obj.item_id.unwrap_or(0);
            if item_id == 0 {
                tracing::warn!(
                    "Type {} does not have a valid item id or shrink table entry.",
                    obj.title()
                );
            }
            let b = muo_server::item::bounds::bounds(w, item_id as usize);
            if item_id != 1 && b.height() < ENTRY_HEIGHT_CAG * 2 {
                let ax = x - OFFSET_SIZE - 22 - (i as i32 % 2) * 44 - b.width() / 2 - b.x();
                if b.height() < ENTRY_HEIGHT_CAG {
                    l.add_item(
                        ax,
                        y + ENTRY_HEIGHT_CAG / 2 - b.height() / 2 - b.y(),
                        item_id,
                        0,
                    );
                } else {
                    l.add_item(
                        ax,
                        y + ENTRY_HEIGHT_CAG - 1 - b.height() - b.y(),
                        item_id,
                        0,
                    );
                }
            }
        }
        i += 1;
        index += 1;
    }
    g
}

impl LegacyGumpLogic for CategorizedAddGump {
    const TYPE_NAME: &'static str = "Server.Gumps.CategorizedAddGump";

    fn singleton(&self) -> bool {
        true
    }

    fn on_response(&mut self, w: &mut World, _sender: NetStateId, info: &RelayInfo) {
        let from = self.owner;
        let (nodes_len, parent) = match w
            .resources
            .get::<CagTree>()
            .and_then(|t| t.cats.get(self.category))
        {
            Some(c) => (c.nodes.as_ref().map_or(0, Vec::len), c.parent),
            None => return,
        };
        match info.button_id {
            0 => {} // Closed
            1 => {
                // Up
                if let Some(p) = parent {
                    let index = w
                        .resources
                        .get::<CagTree>()
                        .and_then(|t| {
                            t.cats[p].nodes.as_ref().and_then(|n| {
                                n.iter().position(
                                    |n| matches!(n, CagNode::Category(c) if *c == self.category),
                                )
                            })
                        })
                        .map_or(0, |i| i / ENTRY_COUNT);
                    let g = new_categorized(w, from, p, index);
                    send(w, from, g);
                }
            }
            2 => {
                if self.page > 0 {
                    let g = new_categorized(w, from, self.category, self.page - 1);
                    send(w, from, g);
                }
            }
            3 => {
                if (self.page + 1) * ENTRY_COUNT < nodes_len {
                    let g = new_categorized(w, from, self.category, self.page + 1);
                    send(w, from, g);
                }
            }
            b => {
                let index = (self.page * ENTRY_COUNT) as i64 + i64::from(b - 4);
                if index >= 0 && (index as usize) < nodes_len {
                    let node = w.resources.get::<CagTree>().and_then(|t| {
                        t.cats[self.category]
                            .nodes
                            .as_ref()
                            .map(|n| n[index as usize].clone())
                    });
                    if let Some(node) = node {
                        on_click(w, from, node, self.page);
                    }
                }
            }
        }
    }
}

/// `CAGNode.OnClick(from, page)`.
fn on_click(w: &mut World, from: MobileId, node: CagNode, page: usize) {
    match node {
        CagNode::Category(c) => {
            let g = new_categorized(w, from, c, 0);
            send(w, from, g);
        }
        CagNode::Object(o) => match o.ty {
            None => send_message(w, from, "That is an invalid type name."),
            Some(t) => {
                let prefix = w.resources.get_mut::<CommandSystem>().prefix.clone();
                CommandSystem::handle(
                    w,
                    from,
                    &format!("{prefix}Add {}", type_name(t)),
                    muo_primitives::MessageType::REGULAR,
                );
                let g = new_categorized(w, from, o.parent, page);
                send(w, from, g);
            }
        },
    }
}
