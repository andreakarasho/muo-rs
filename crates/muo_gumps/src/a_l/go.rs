//! The `[Go` gump: `GoCategory`, `GoLocation`, `GoLocations`, `LocationTree` and `GoGump`.
//!
//! The category tree is an arena (`LocationTree::nodes`), a node refers to its parent and children by index. The
//! trees live in the [`GoLocations`] resource, loaded the first time a map is asked for; `LastBranch` (the
//! category each mobile last browsed to) is kept next to each tree, and a gump holds the tree it browses.

use std::collections::HashMap;
use std::rc::Rc;

use muo_content::gumps::base::{DynamicGumpLogic, DynamicGumpObj, RelayInfo};
use muo_content::gumps::builder::DynamicGumpBuilder;
use muo_primitives::Point3D;
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::map::MapId;
use muo_server::net_state::NetStateId;
use serde::Deserialize;

use super::common::{mobile_of, send};
use muo_content::gumps::props_config::{OFFSET_SIZE, SET_WIDTH, TEXT_HUE};
use muo_content::gumps::props_layout::{self as layout, Nav};

/// `GoLocation`.
#[derive(Debug)]
pub struct GoLocation {
    pub name: String,
    pub location: Point3D,
}

/// `GoCategory`.
#[derive(Debug, Default)]
pub struct GoCategory {
    pub name: Option<String>,
    pub parent: Option<usize>,
    pub categories: Vec<usize>,
    pub locations: Vec<GoLocation>,
}

/// `LocationTree` (the `LastBranch` dictionary is [`GoLocations::last_branch`]).
#[derive(Debug)]
pub struct LocationTree {
    pub map: MapId,
    pub nodes: Vec<GoCategory>,
    /// None when the file did not load.
    pub root: Option<usize>,
}

#[derive(Deserialize)]
struct RawLocation {
    #[serde(default)]
    name: String,
    location: [i32; 3],
}

#[derive(Deserialize)]
struct RawCategory {
    name: Option<String>,
    locations: Option<Vec<RawLocation>>,
    categories: Option<Vec<RawCategory>>,
}

impl LocationTree {
    fn empty(map: MapId) -> Self {
        Self {
            map,
            nodes: Vec::new(),
            root: None,
        }
    }

    /// `new LocationTree(map, lastBranch, root)`: `SetParents`.
    fn from_raw(map: MapId, raw: RawCategory) -> Self {
        let mut t = Self {
            map,
            nodes: Vec::new(),
            root: None,
        };
        let root = t.add(raw, None);
        t.root = Some(root);
        t
    }

    fn add(&mut self, raw: RawCategory, parent: Option<usize>) -> usize {
        let index = self.nodes.len();
        self.nodes.push(GoCategory {
            name: raw.name,
            parent,
            categories: Vec::new(),
            locations: raw
                .locations
                .unwrap_or_default()
                .into_iter()
                .map(|l| GoLocation {
                    name: l.name,
                    location: Point3D::new(l.location[0], l.location[1], l.location[2]),
                })
                .collect(),
        });
        for c in raw.categories.unwrap_or_default() {
            let child = self.add(c, Some(index));
            self.nodes[index].categories.push(child);
        }
        index
    }
}

/// `GoLocations`' statics: one tree per map, and `LastBranch` per tree.
#[derive(Default)]
pub struct GoLocations {
    trees: HashMap<u8, Rc<LocationTree>>,
    last_branch: HashMap<(u8, MobileId), usize>,
}

fn file_for(map: MapId) -> (u8, &'static str, MapId) {
    // Felucca is also the answer for any map not listed
    match map {
        MapId::ILSHENAR => (2, "ilshenar", MapId::ILSHENAR),
        MapId::TER_MUR => (5, "termur", MapId::TER_MUR),
        MapId::TRAMMEL => (1, "trammel", MapId::TRAMMEL),
        MapId(3) => (3, "malas", MapId(3)),
        MapId(4) => (4, "tokuno", MapId(4)),
        _ => (0, "felucca", MapId::FELUCCA),
    }
}

/// `GoLocations.GetLocations(map)`.
pub fn get_locations(w: &mut World, map: Option<MapId>) -> Rc<LocationTree> {
    let (key, file, map) = file_for(map.unwrap_or(MapId::FELUCCA));
    if let Some(t) = w.resources.get_mut::<GoLocations>().trees.get(&key) {
        return t.clone();
    }
    let tree = Rc::new(load_locations(w, file, map));
    w.resources
        .get_mut::<GoLocations>()
        .trees
        .insert(key, tree.clone());
    tree
}

fn load_locations(w: &World, file: &str, map: MapId) -> LocationTree {
    let path = w
        .core
        .base_directory
        .join(format!("Data/Locations/{file}.json"));
    let result = std::fs::read_to_string(&path)
        .map_err(|e| e.to_string())
        .and_then(|t| parse_tree(map, &t));
    match result {
        Ok(tree) => tree,
        Err(e) => {
            tracing::error!("Failed to load file {}: {e}", path.display());
            LocationTree::empty(map)
        }
    }
}

/// `JsonConfig.Deserialize<GoCategory>` and `new LocationTree(..)`.
pub fn parse_tree(map: MapId, text: &str) -> Result<LocationTree, String> {
    muo_data::json::from_str::<RawCategory>(text.trim_start_matches('\u{feff}'))
        .map(|raw| LocationTree::from_raw(map, raw))
        .map_err(|e| e.to_string())
}

const ENTRY_WIDTH: i32 = 180;
const ENTRY_COUNT: usize = 15;
const TOTAL_WIDTH: i32 = OFFSET_SIZE + ENTRY_WIDTH + OFFSET_SIZE + SET_WIDTH + OFFSET_SIZE;

#[derive(Clone)]
pub struct GoGump {
    node: usize,
    page: usize,
    from: MobileId,
    tree: Rc<LocationTree>,
}

fn key(tree: &LocationTree) -> u8 {
    file_for(tree.map).0
}

/// `if (node == tree.Root) LastBranch.Remove(from) else LastBranch[from] = node`.
fn remember(w: &mut World, tree: &LocationTree, from: MobileId, node: usize) {
    let g = w.resources.get_mut::<GoLocations>();
    if Some(node) == tree.root {
        g.last_branch.remove(&(key(tree), from));
    } else {
        g.last_branch.insert((key(tree), from), node);
    }
}

fn new_gump(
    w: &mut World,
    page: usize,
    from: MobileId,
    tree: Rc<LocationTree>,
    node: usize,
) -> DynamicGumpObj<GoGump> {
    remember(w, &tree, from, node);
    DynamicGumpObj::new(
        50,
        50,
        GoGump {
            node,
            page,
            from,
            tree,
        },
    )
}

#[cfg(test)]
impl GoGump {
    pub fn for_test(tree: Rc<LocationTree>, node: usize, page: usize) -> Self {
        Self {
            node,
            page,
            from: MobileId(muo_primitives::Serial::new(1)),
            tree,
        }
    }
}

/// `GoGump.DisplayTo(from)`.
pub fn display_to(w: &mut World, from: MobileId) {
    if w.mobile(from).and_then(|m| m.net_state).is_none() {
        return;
    }
    let tree = get_locations(w, muo_content::mx::map(w, from));
    let branch = w
        .resources
        .get_mut::<GoLocations>()
        .last_branch
        .get(&(key(&tree), from))
        .copied()
        .or(tree.root);
    let Some(branch) = branch else { return };
    let g = new_gump(w, 0, from, tree, branch);
    send(w, from, g);
}

impl GoGump {
    fn cat(&self) -> &GoCategory {
        &self.tree.nodes[self.node]
    }

    fn total(&self) -> usize {
        self.cat().categories.len() + self.cat().locations.len()
    }
}

impl DynamicGumpLogic for GoGump {
    const TYPE_NAME: &'static str = "Server.Gumps.GoGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        let node = self.cat();
        let total = self.total();
        let count =
            (total as i64 - (self.page * ENTRY_COUNT) as i64).clamp(0, ENTRY_COUNT as i64) as i32;

        b.add_page(0);

        let (mut x, mut y) = layout::add_props_frame(b, TOTAL_WIDTH, count + 1);
        layout::add_props_header_with_back(
            b,
            TOTAL_WIDTH,
            &mut x,
            &mut y,
            node.name.as_deref(),
            node.parent.is_some().then_some(1),
            (self.page > 0).then_some(2),
            ((self.page + 1) * ENTRY_COUNT < total).then_some(3),
            Nav {
                kind: muo_content::gumps::GumpButtonType::Reply,
                param: 1,
            },
            muo_content::gumps::props_config::ENTRY_HEIGHT,
        );

        let mut index = self.page * ENTRY_COUNT;
        let mut i = 0;
        while i < ENTRY_COUNT && index < total {
            layout::next_row(&mut x, &mut y);

            let name = if index >= node.categories.len() {
                &node.locations[index - node.categories.len()].name
            } else {
                self.tree.nodes[node.categories[index]]
                    .name
                    .as_deref()
                    .unwrap_or("")
            };

            layout::add_props_entry_button(
                b,
                &mut x,
                &mut y,
                ENTRY_WIDTH,
                name,
                Some(index as i32 + 4),
                TEXT_HUE,
            );
            i += 1;
            index += 1;
        }
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let Some(from) = mobile_of(w, sender) else {
            return;
        };
        let resend = |g: &GoGump, w: &mut World| {
            send(w, from, DynamicGumpObj::new(50, 50, g.clone()));
        };

        match info.button_id {
            1 => {
                if let Some(parent) = self.cat().parent {
                    self.page = 0;
                    self.node = parent;
                    remember(w, &self.tree, from, self.node);
                    resend(self, w);
                }
            }
            2 => {
                if self.page > 0 {
                    self.page -= 1;
                    resend(self, w);
                }
            }
            3 => {
                if (self.page + 1) * ENTRY_COUNT < self.total() {
                    self.page += 1;
                    resend(self, w);
                }
            }
            id => {
                let index = id - 4;
                if index < 0 {
                    return;
                }
                let index = index as usize;
                let cats = self.cat().categories.len();
                if index < cats {
                    self.page = 0;
                    self.node = self.cat().categories[index];
                    // LastBranch[from] = node, even when the node is the root
                    w.resources
                        .get_mut::<GoLocations>()
                        .last_branch
                        .insert((key(&self.tree), from), self.node);
                    resend(self, w);
                } else if index - cats < self.cat().locations.len() {
                    let p = self.cat().locations[index - cats].location;
                    muo_server::mobile::base::move_to_world(w, from, p, self.tree.map);
                }
            }
        }
        let _ = self.from;
    }
}
