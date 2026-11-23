//! JSON-only classes: a template with `"parent": "<registered class>"` and a `"type"` no Rust class has.
//!
//! The class is a leaked copy of the parent's vtable with its own name, so it saves under its own name and reads
//! back with the parent's record format. `construct` cannot capture the class, so each class gets a slot whose
//! `construct` is a distinct monomorphized function.
//! shortcut: 1024 slots per kind; a server with more JSON-only classes needs a larger table.

use muo_server::ids::{ItemId, MobileId};
use muo_server::item::ItemType;
use muo_server::mobile::MobileType;
use muo_server::type_registry::EntityType;
use muo_server::world::World;

use super::{Kind, TemplateTable};
use crate::creature::CreatureType;

const SLOTS: usize = 32 * 32;

/// The JSON-only classes made so far, by slot.
#[derive(Default)]
pub struct RuntimeTypes {
    pub mobiles: Vec<&'static MobileType>,
    pub items: Vec<&'static ItemType>,
}

macro_rules! slot_table {
    ($f:ident, $t:ty; $($a:literal)*) => { slot_table!(@outer $f, $t; [$($a)*]; $($a)*) };
    (@outer $f:ident, $t:ty; $inner:tt; $($a:literal)*) => { [$(slot_table!(@inner $f, $t; $a; $inner)),*] };
    (@inner $f:ident, $t:ty; $a:literal; [$($b:literal)*]) => { [$($f::<$a, $b> as $t),*] };
}

type MobileCtor = fn(&mut World) -> MobileId;
type ItemCtor = fn(&mut World) -> ItemId;

static MOBILE_CTORS: [[MobileCtor; 32]; 32] = slot_table!(mobile_ctor, MobileCtor;
    0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31);
static ITEM_CTORS: [[ItemCtor; 32]; 32] = slot_table!(item_ctor, ItemCtor;
    0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31);

fn mobile_ctor<const A: usize, const B: usize>(w: &mut World) -> MobileId {
    construct_mobile(w, A * 32 + B)
}

fn item_ctor<const A: usize, const B: usize>(w: &mut World) -> ItemId {
    construct_item(w, A * 32 + B)
}

fn construct_mobile(w: &mut World, slot: usize) -> MobileId {
    let ty = w.resources.get::<RuntimeTypes>().map(|r| r.mobiles[slot]);
    let ty = ty.expect("a slot's constructor exists only once the slot is filled");
    let parent = ty.parent.expect("a JSON-only class always has a parent");
    let id = match parent.construct.or(parent.make) {
        Some(ctor) => {
            let id = ctor(w);
            if let Some(m) = w.mobile_mut(id) {
                m.ty = ty;
                // the parent's constructor recorded the parent's class for the save; record this one too, or
                // the next save refuses the world ("serialized but never registered")
                let _ = w.mobile_state.type_table.register(ty.full_name);
            }
            id
        }
        None if is_creature_class(parent) => {
            let a = super::table(w).roll_args(w, ty.full_name);
            crate::creature::setters::new_creature(
                w,
                ty,
                a.ai,
                a.fight_mode,
                a.range_perception,
                a.range_fight,
            )
        }
        None => w.new_mobile(ty),
    };
    super::apply_mobile_as(w, id, ty.full_name);
    id
}

fn construct_item(w: &mut World, slot: usize) -> ItemId {
    let ty = w.resources.get::<RuntimeTypes>().map(|r| r.items[slot]);
    let ty = ty.expect("a slot's constructor exists only once the slot is filled");
    let parent = ty.parent.expect("a JSON-only class always has a parent");
    let id = match parent.construct.or(parent.make) {
        Some(ctor) => {
            let id = ctor(w);
            if let Some(i) = w.item_mut(id) {
                i.ty = ty;
                let _ = w.item_state.type_table.register(ty.full_name); // as for mobiles above
            }
            id
        }
        None => muo_server::item::ops::new_item_with_id(w, ty, 0),
    };
    super::apply_item_as(w, id, ty.full_name);
    id
}

fn is_creature_class(t: &MobileType) -> bool {
    t.family
        .is_some_and(|f| f.downcast_ref::<CreatureType>().is_some())
}

/// Makes and registers a class for every template with a `parent` not seen before. A reload keeps the classes
/// already made (a changed `parent` needs a restart).
pub fn register_all(w: &mut World, t: &mut TemplateTable) -> Result<(), String> {
    let mut rt = w.resources.get_mut::<RuntimeTypes>().clone_lists();
    let mut todo: Vec<(Kind, String, String)> = Vec::new();
    for (kind, map) in [(Kind::Mobile, &t.mobiles), (Kind::Item, &t.items)] {
        for tmpl in map.values() {
            if let Some(p) = &tmpl.parent {
                todo.push((kind, tmpl.type_name.clone(), p.clone()));
            }
        }
    }
    todo.sort(); // slots are stable for the same files
    // A parent may itself be a JSON-only class made later in the list: repeat until a pass makes nothing.
    while !todo.is_empty() {
        let before = todo.len();
        let mut later = Vec::new();
        for (kind, name, parent) in todo {
            if !make_one(w, t, &mut rt, kind, &name, &parent, false)? {
                later.push((kind, name, parent));
            }
        }
        if later.len() == before {
            let (kind, name, parent) = &later[0];
            make_one(w, t, &mut rt, *kind, name, parent, true)?;
        }
        todo = later;
    }
    w.resources.insert(rt);
    Ok(())
}

/// Makes one class; `Ok(false)` when its parent is not registered yet (an error when `last_try`).
fn make_one(
    w: &mut World,
    t: &mut TemplateTable,
    rt: &mut RuntimeTypes,
    kind: Kind,
    name: &str,
    parent: &str,
    last_try: bool,
) -> Result<bool, String> {
    let file = t
        .get(kind, name)
        .map(|t| t.file.display().to_string())
        .unwrap_or_default();
    let err = |m: String| format!("{file}: {name}: {m}");
    let existing = w.types.find_type_by_full_name(name, false);
    let ours = match existing {
        Some(EntityType::Mobile(m)) => rt.mobiles.iter().any(|x| std::ptr::eq(*x, m)),
        Some(EntityType::Item(i)) => rt.items.iter().any(|x| std::ptr::eq(*x, i)),
        None => false,
    };
    if ours {
        return Ok(true);
    }
    if existing.is_some() {
        return Err(err(
            "a Rust class has this name; drop \"parent\" to template that class".into(),
        ));
    }
    match (kind, w.types.find_type_by_full_name(parent, false)) {
        (Kind::Mobile, Some(EntityType::Mobile(p))) => {
            if rt.mobiles.len() == SLOTS {
                return Err(err(format!("more than {SLOTS} JSON-only mobile classes")));
            }
            let i = rt.mobiles.len();
            let ty: &'static MobileType = Box::leak(Box::new(MobileType {
                full_name: t.intern(name),
                parent: Some(p),
                construct: Some(MOBILE_CTORS[i / 32][i % 32]),
                make: None,
                ..*p
            }));
            rt.mobiles.push(ty);
            w.types.register_mobile(ty, &[]);
        }
        (Kind::Item, Some(EntityType::Item(p))) => {
            if rt.items.len() == SLOTS {
                return Err(err(format!("more than {SLOTS} JSON-only item classes")));
            }
            let i = rt.items.len();
            let ty: &'static ItemType = Box::leak(Box::new(ItemType {
                full_name: t.intern(name),
                parent: Some(p),
                construct: Some(ITEM_CTORS[i / 32][i % 32]),
                make: None,
                ..*p
            }));
            rt.items.push(ty);
            w.types.register_item(ty, &[]);
        }
        (_, None) if !last_try => return Ok(false),
        _ => return Err(err(format!("\"parent\": no {kind:?} class {parent}"))),
    }
    Ok(true)
}

impl RuntimeTypes {
    fn clone_lists(&self) -> Self {
        Self {
            mobiles: self.mobiles.clone(),
            items: self.items.clone(),
        }
    }
}
