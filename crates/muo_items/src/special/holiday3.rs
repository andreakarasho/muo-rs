//! `HolidayTree`, `HolidayTimepiece`, `WristWatch` and `HolidayTreeDeed` (the tree is only reachable through its deed).

use muo_content::gumps::base::{RelayInfo, StaticGumpLogic, StaticGumpObj};
use muo_content::gumps::builder::StaticGumpBuilder;
use muo_content::gumps::send_gump_to_mobile;
use muo_content::items::house_deps::{self, SimpleAddon};
use muo_content::props::{self, Kind, PropDef, Value};
use muo_content::utilities::types::PropType;

use muo_primitives::{Point3D, TimeSpan};
use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, deps, ops};
use muo_server::map::MapId;
use muo_server::mobile::AccessLevel;
use muo_server::net_state::NetStateId;
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::target::Targeted;
use muo_server::type_registry::TypeRegistry;

use super::util::{access_level, delete, deleted, msg};
use super::vet::begin_target;
use crate::simple::leaf;
use crate::skill_items::small::CLOCK;

// ---- HolidayTimepiece, WristWatch ----

/// `HolidayTimepiece()`.
pub fn holiday_timepiece_new(w: &mut World) -> ItemId {
    crate::simple::from_template(w, &HOLIDAY_TIMEPIECE, "Server.Items.HolidayTimepiece")
}

leaf! {
    /// `Server.Items.HolidayTimepiece`.
    HOLIDAY_TIMEPIECE = "Server.Items.HolidayTimepiece" : CLOCK, v0;
    construct: Some(holiday_timepiece_new),
    label_number: crate::class_props::label_number, // a holiday timepiece
    default_weight: crate::class_props::default_weight,
}

/// `WristWatch()`.
pub fn wrist_watch_new(w: &mut World) -> ItemId {
    crate::simple::from_template(w, &WRIST_WATCH, "Server.Items.WristWatch")
}

leaf! {
    /// `Server.Items.WristWatch`.
    WRIST_WATCH = "Server.Items.WristWatch" : CLOCK, enc;
    construct: Some(wrist_watch_new),
    label_number: crate::class_props::label_number, // a wrist watch
    default_weight: crate::class_props::default_weight,
}

// ---- HolidayTree ----

/// `HolidayTreeType`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HolidayTreeType {
    Classic,
    Modern,
}

/// `HolidayTree._placer` and `_components` (None: null).
#[derive(Clone, Debug, Default)]
struct TreeData {
    placer: Option<MobileId>,
    components: Option<Vec<Option<ItemId>>>,
}

/// `Ornament._tree` / `TreeTrunk._tree`.
#[derive(Clone, Copy, Debug, Default)]
struct TreeRef(Option<ItemId>);

fn tree(w: &World, id: ItemId) -> TreeData {
    w.item(id)
        .and_then(|i| i.ext.get::<TreeData>())
        .cloned()
        .unwrap_or_default()
}

fn set_tree(w: &mut World, id: ItemId, d: TreeData) {
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(d);
    }
}

fn tree_of(w: &World, id: ItemId) -> Option<ItemId> {
    w.item(id)
        .and_then(|i| i.ext.get::<TreeRef>())
        .and_then(|t| t.0)
}

/// `_components.Add(item)`: with no list the add is dropped.
fn components_add(w: &mut World, tree_id: ItemId, item: ItemId) {
    if let Some(c) = w
        .item_mut(tree_id)
        .and_then(|i| i.ext.get_mut::<TreeData>())
        .and_then(|d| d.components.as_mut())
    {
        c.push(Some(item));
    }
}

/// `AddItem(x, y, z, item)`.
fn tree_add_item(w: &mut World, tree_id: ItemId, x: i32, y: i32, z: i32, item: ItemId) {
    let (loc, map) = w
        .item(tree_id)
        .map_or((Point3D::default(), None), |i| (i.location, i.map));
    ops::move_to_world_map(w, item, Point3D::new(loc.x + x, loc.y + y, loc.z + z), map);
    components_add(w, tree_id, item);
}

/// `AddOrnament(x, y, z, itemID)`.
fn add_ornament(w: &mut World, tree_id: ItemId, x: i32, y: i32, z: i32, item_id: i32) {
    let o = ops::new_item_with_id(w, &ORNAMENT, item_id);
    ops::set_movable(w, o, false);
    set_tree_ref(w, o, Some(tree_id));
    tree_add_item(w, tree_id, x + 1, y + 1, z + 11, o);
}

fn set_tree_ref(w: &mut World, id: ItemId, t: Option<ItemId>) {
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(TreeRef(t));
    }
}

const CLASSIC: [(i32, i32, i32, i32); 27] = [
    (0, 0, 2, 0xF22),
    (0, 0, 9, 0xF18),
    (0, 0, 15, 0xF20),
    (0, 0, 19, 0xF17),
    (0, 0, 20, 0xF24),
    (0, 0, 20, 0xF1F),
    (0, 0, 20, 0xF19),
    (0, 0, 21, 0xF1B),
    (0, 0, 28, 0xF2F),
    (0, 0, 30, 0xF23),
    (0, 0, 32, 0xF2A),
    (0, 0, 33, 0xF30),
    (0, 0, 34, 0xF29),
    (0, 1, 7, 0xF16),
    (0, 1, 7, 0xF1E),
    (0, 1, 12, 0xF0F),
    (0, 1, 13, 0xF13),
    (0, 1, 18, 0xF12),
    (0, 1, 19, 0xF15),
    (0, 1, 25, 0xF28),
    (0, 1, 29, 0xF1A),
    (0, 1, 37, 0xF2B),
    (1, 0, 13, 0xF10),
    (1, 0, 14, 0xF1C),
    (1, 0, 16, 0xF14),
    (1, 0, 17, 0xF26),
    (1, 0, 22, 0xF27),
];

const MODERN: [(i32, i32, i32, i32); 23] = [
    (0, 0, 2, 0xF2F),
    (0, 0, 2, 0xF20),
    (0, 0, 2, 0xF22),
    (0, 0, 5, 0xF30),
    (0, 0, 5, 0xF15),
    (0, 0, 5, 0xF1F),
    (0, 0, 5, 0xF2B),
    (0, 0, 6, 0xF0F),
    (0, 0, 7, 0xF1E),
    (0, 0, 7, 0xF24),
    (0, 0, 8, 0xF29),
    (0, 0, 9, 0xF18),
    (0, 0, 14, 0xF1C),
    (0, 0, 15, 0xF13),
    (0, 0, 15, 0xF20),
    (0, 0, 16, 0xF26),
    (0, 0, 17, 0xF12),
    (0, 0, 18, 0xF17),
    (0, 0, 20, 0xF1B),
    (0, 0, 23, 0xF28),
    (0, 0, 25, 0xF18),
    (0, 0, 25, 0xF2A),
    (0, 1, 7, 0xF16),
];

/// `HolidayTree(from, type, loc)`.
pub fn holiday_tree_new(
    w: &mut World,
    from: MobileId,
    kind: HolidayTreeType,
    loc: Point3D,
) -> ItemId {
    let id = crate::simple::from_template(w, &HOLIDAY_TREE, "Server.Items.HolidayTree");
    let map = w.mobile(from).and_then(|m| m.map);
    ops::move_to_world_map(w, id, loc, map);
    set_tree(
        w,
        id,
        TreeData {
            placer: Some(from),
            components: None,
        },
    );
    match kind {
        HolidayTreeType::Classic => {
            super::item_id_of(w, id, 0xCD7);
            set_tree(
                w,
                id,
                TreeData {
                    placer: Some(from),
                    components: Some(Vec::with_capacity(28)),
                },
            );
            let trunk = ops::new_item_with_id(w, &TREE_TRUNK, 0xCD6);
            ops::set_movable(w, trunk, false);
            ops::move_to_world_map(w, trunk, loc, map);
            set_tree_ref(w, trunk, Some(id));
            tree_add_item(w, id, 0, 0, 0, trunk);
            for (x, y, z, g) in CLASSIC {
                add_ornament(w, id, x, y, z, g);
            }
        }
        HolidayTreeType::Modern => {
            super::item_id_of(w, id, 0x1B7E);
            set_tree(
                w,
                id,
                TreeData {
                    placer: Some(from),
                    components: Some(Vec::with_capacity(23)),
                },
            );
            for (x, y, z, g) in MODERN {
                add_ornament(w, id, x, y, z, g);
            }
        }
    }
    id
}

fn tree_deed(w: &mut World, _id: ItemId) -> ItemId {
    holiday_tree_deed_new(w)
}

fn tree_could_fit(w: &mut World, _id: ItemId, p: Point3D, map: MapId) -> bool {
    muo_server::map::fit::can_fit_point(w, map, p, 20)
}

/// `HolidayTree.OnAfterDelete()`.
fn tree_on_after_delete(w: &mut World, id: ItemId) {
    (ITEM_BASE.on_after_delete)(w, id);
    let comps = tree(w, id).components.unwrap_or_default();
    for c in comps.into_iter().flatten() {
        if !deleted(w, c) {
            delete(w, c);
        }
    }
    let mut d = tree(w, id);
    d.components = None;
    set_tree(w, id, d);
}

/// `[AfterDeserialization]` of the tree: an old tree, or one with no house, is re-deeded.
fn tree_after_deserialization(w: &mut World, id: ItemId) {
    if deleted(w, id) {
        return;
    }
    let empty = tree(w, id).components.is_none_or(|c| c.is_empty());
    if empty || muo_multis::house::access::find_house_at_item(w, id).is_none() {
        let deed = holiday_tree_deed_new(w);
        let (loc, map) = w
            .item(id)
            .map_or((Point3D::default(), None), |i| (i.location, i.map));
        ops::move_to_world_map(w, deed, loc, map);
        delete(w, id);
    }
}

fn tree_on_double_click(w: &mut World, id: ItemId, from: MobileId) {
    if !muo_content::mx::in_range_point(w, from, ops::get_world_location(w, id), 1) {
        msg(w, from, 500446); // That is too far away.
        return;
    }
    let placer = tree(w, id).placer;
    if placer.is_some_and(|p| p != from) && access_level(w, from) < AccessLevel::GameMaster {
        msg(w, from, 503396); // You cannot take this tree down.
        return;
    }
    let deed = holiday_tree_deed_new(w);
    deps::add_to_backpack(w, from, deed);
    delete(w, id);
    if let Some(h) = muo_multis::house::access::find_house_at_item(w, id) {
        // Addons.Remove(this)
        muo_content::items::addon::house_addons_remove(w, h, id);
    }
    msg(w, from, 503393); // A deed for the tree has been placed in your backpack.
}

/// `Server.Items.HolidayTree` (`IAddon`; `[SerializationGenerator(2, false)]`; no `[Constructible]`).
pub static HOLIDAY_TREE: ItemType = ItemType {
    full_name: "Server.Items.HolidayTree",
    parent: Some(&ITEM_BASE),
    construct: None,
    label_number: crate::class_props::label_number, // a tree for the holidays
    on_after_delete: tree_on_after_delete,
    on_double_click: tree_on_double_click,
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(2);
        let d = tree(w, id);
        out.write_mobile(w, d.placer);
        let comps = d.components.unwrap_or_default();
        out.write_encoded_int(comps.len() as i32);
        for c in comps {
            out.write_item(w, c);
        }
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let version = r.read_i32()?;
        let placer = r.read_mobile(w)?;
        let mut comps = Vec::new();
        if version < 2 {
            // With no list the items are kept.
            let count = r.read_i32()?;
            for _ in 0..count {
                if let Some(item) = r.read_item(w)? {
                    comps.push(Some(item));
                }
            }
        } else {
            let count = r.read_encoded_int()?;
            for _ in 0..count {
                comps.push(r.read_item(w)?);
            }
        }
        set_tree(
            w,
            id,
            TreeData {
                placer,
                components: Some(comps),
            },
        );
        muo_server::timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 1, move |w| {
            tree_after_deserialization(w, id)
        });
        Ok(())
    },
    ..ITEM_BASE
};

/// What `[AfterDeserialization]` of an ornament or trunk does: joins the tree's list, or goes away.
fn part_after_deserialization(w: &mut World, id: ItemId) {
    match tree_of(w, id) {
        None => {
            muo_server::timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 1, move |w| {
                if !deleted(w, id) {
                    delete(w, id);
                }
            });
        }
        Some(t) => components_add(w, t, id),
    }
}

macro_rules! tree_part {
    ($name:ident, $full:literal, $label:expr, $dbl:expr) => {
        #[doc = concat!("`", $full, "` (`[SerializationGenerator(1, false)]`).")]
        pub static $name: ItemType = ItemType {
            full_name: $full,
            parent: Some(&ITEM_BASE),
            construct: None,
            label_number: |_, _| $label,
            on_double_click: $dbl,
            serialize: |w, id, out| {
                (ITEM_BASE.serialize)(w, id, out);
                out.write_i32(1);
                out.write_item(w, tree_of(w, id));
            },
            deserialize: |w, id, r| {
                (ITEM_BASE.deserialize)(w, id, r)?;
                let _version = r.read_i32()?; // version 0 holds the same tree reference
                let t = r.read_item(w)?;
                set_tree_ref(w, id, t);
                part_after_deserialization(w, id);
                Ok(())
            },
            ..ITEM_BASE
        };
    };
}

tree_part!(
    ORNAMENT,
    "Server.Items.HolidayTree+Ornament",
    1041118,
    ITEM_BASE.on_double_click
); // a tree ornaments
tree_part!(
    TREE_TRUNK,
    "Server.Items.HolidayTree+TreeTrunk",
    1041117,
    |w, id, from| {
        if let Some(t) = tree_of(w, id).filter(|&t| !deleted(w, t)) {
            tree_on_double_click(w, t, from);
        }
    }
);

// ---- HolidayTreeDeed ----

/// `HolidayTreeDeed()`.
pub fn holiday_tree_deed_new(w: &mut World) -> ItemId {
    crate::simple::from_template(w, &HOLIDAY_TREE_DEED, "Server.Items.HolidayTreeDeed")
}

/// `HolidayTreeDeed.ValidatePlacement(from, loc)`.
fn validate_placement(w: &mut World, deed: ItemId, from: MobileId, loc: Point3D) -> bool {
    if access_level(w, from) >= AccessLevel::GameMaster {
        return true;
    }
    if !muo_content::mx::in_range_point(w, from, ops::get_world_location(w, deed), 1) {
        msg(w, from, 500446); // That is too far away.
        return false;
    }
    if w.now.month() != 12 {
        // You will have to wait till next December to put your tree back up for display.
        msg(w, from, 1005700);
        return false;
    }
    let Some(map) = w.mobile(from).and_then(|m| m.map) else {
        return false;
    };
    let house = muo_multis::house::access::find_house_at_point(w, loc, Some(map), 20);
    if !house.is_some_and(|h| muo_multis::house::access::is_friend(w, h, from)) {
        msg(w, from, 1005701); // The holiday tree can only be placed in your house.
        return false;
    }
    if !muo_server::map::fit::can_fit_point(w, map, loc, 20) {
        msg(w, from, 500269); // You cannot build that there.
        return false;
    }
    true
}

/// `HolidayTreeDeed.EndPlace(from, type, loc)`.
fn end_place(w: &mut World, deed: ItemId, from: MobileId, kind: HolidayTreeType, loc: Point3D) {
    delete(w, deed);
    let t = holiday_tree_new(w, from, kind, loc);
    if let Some(h) = muo_multis::house::access::find_house_at_item(w, t) {
        muo_content::items::addon::house_addons_add(w, h, t);
    }
}

/// `HolidayTreeDeed.Placement_OnTarget`.
fn placement_on_target(
    w: &mut World,
    deed: ItemId,
    from: MobileId,
    kind: HolidayTreeType,
    t: Targeted,
) {
    let Some(mut loc) = super::vet::target_point(w, t) else {
        return;
    };
    if let Targeted::Static(s) = t {
        // NOTE: OSI does not properly normalize Z positioning here.
        loc.z -= w
            .maps
            .tile_data
            .item((s.item_id & 0x3FFF) as usize)
            .calc_height();
    }
    if validate_placement(w, deed, from, loc) {
        end_place(w, deed, from, kind, loc);
    }
}

/// `HolidayTreeChoiceGump`.
struct HolidayTreeChoiceGump {
    deed: ItemId,
}

impl StaticGumpLogic for HolidayTreeChoiceGump {
    const TYPE_NAME: &'static str = "Server.Items.HolidayTreeChoiceGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut StaticGumpBuilder) {
        b.add_page(0);
        b.add_background(0, 0, 220, 120, 5054);
        b.add_background(10, 10, 200, 100, 3000);
        b.add_button(20, 35, 4005, 4007, 1);
        b.add_html_localized(55, 35, 145, 25, 1018322, false, false); // Classic
        b.add_button(20, 65, 4005, 4007, 2);
        b.add_html_localized(55, 65, 145, 25, 1018321, false, false); // Modern
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        if deleted(w, self.deed) {
            return;
        }
        let Some(from) = w.net_states.get(&sender).and_then(|n| n.mobile) else {
            return;
        };
        let kind = match info.button_id {
            1 => HolidayTreeType::Classic,
            2 => HolidayTreeType::Modern,
            _ => return,
        };
        let deed = self.deed;
        begin_target(w, from, move |w, from, t| {
            placement_on_target(w, deed, from, kind, t)
        });
    }
}

/// `Server.Items.HolidayTreeDeed`.
pub static HOLIDAY_TREE_DEED: ItemType = ItemType {
    full_name: "Server.Items.HolidayTreeDeed",
    parent: Some(&ITEM_BASE),
    construct: Some(holiday_tree_deed_new),
    default_weight: crate::class_props::default_weight,
    label_number: crate::class_props::label_number, // a deed for a holiday tree
    on_double_click: |w, id, from| {
        if super::util::net_state(w, from).is_none() || deleted(w, id) {
            return;
        }
        send_gump_to_mobile(
            w,
            from,
            Box::new(StaticGumpObj::new(
                200,
                200,
                HolidayTreeChoiceGump { deed: id },
            )),
            false,
        );
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        Ok(())
    },
    ..ITEM_BASE
};

/// `[CommandProperty(GameMaster)] HolidayTree.Placer`.
static TREE_PROPS: &[PropDef] = &[PropDef {
    name: "Placer",
    kind: Kind::Plain(PropType::Entity),
    read: AccessLevel::GameMaster,
    write: AccessLevel::GameMaster,
    get: |w, e| match e {
        EntityId::Item(i) => Value::Entity(tree(w, i).placer.map(EntityId::Mobile)),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err("not an item".to_string());
        };
        let placer = match v.entity() {
            Some(EntityId::Mobile(m)) => Some(m),
            None => None,
            Some(_) => return Err("not a mobile".to_string()),
        };
        let mut d = tree(w, i);
        d.placer = placer;
        set_tree(w, i, d);
        Ok(())
    }),
}];

pub fn register(types: &mut TypeRegistry) {
    for t in [
        &HOLIDAY_TIMEPIECE,
        &WRIST_WATCH,
        &HOLIDAY_TREE,
        &ORNAMENT,
        &TREE_TRUNK,
        &HOLIDAY_TREE_DEED,
    ] {
        types.register_item(t, &[]);
    }
}

pub(super) fn configure(w: &mut World) {
    props::add(w, "Server.Items.HolidayTree", TREE_PROPS);
    house_deps::deps_mut(w).simple_addons.push(SimpleAddon {
        is: |w, id| w.item(id).is_some_and(|i| i.ty.is(&HOLIDAY_TREE)),
        deed: tree_deed,
        could_fit: tree_could_fit,
    });
}
