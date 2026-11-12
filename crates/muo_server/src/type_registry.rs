//! The type registry: no assemblies and no reflection, so the content crate registers what a
//! reflection scan found: every concrete `Item`/`Mobile` class (name -> `&'static ItemType`/`MobileType`, with the
//! `[TypeAlias]` names), and the static `ConfigurePrompts()`/`Configure()`/`Initialize()` methods
//! (`AssemblyHandler.Invoke`) as explicit functions with their `[CallPriority]` (default 50; lower first; equal
//! priorities run in registration order).
//!
//! A class registered here has a serialization constructor by construction (the loader makes it with
//! `Item::new_for_load`), so "does not have a serialization constructor" cannot happen. An abstract class is
//! simply not registered, which the loader reports as "not found".
//!
//! Lookup rules of `TypeCache`: by name (`Type.Name`) or full name (`Type.FullName`), case-insensitive by default
//! (`FindTypeByName`), or by the XXH3 hash of the case-sensitive full name (`FindTypeByHash`, what saves use).
//! Items are searched before mobiles, in registration order.

use muo_primitives::dotnet::InvariantCase;
use std::collections::HashMap;

use muo_primitives::hash::compute_hash64;

use crate::item::{ITEM_BASE, ItemType};
use crate::mobile::{MOBILE_BASE, MobileType};
use crate::world::World;

/// The default `[CallPriority]`.
pub const DEFAULT_CALL_PRIORITY: i32 = 50;

/// A registered class.
#[derive(Copy, Clone)]
pub enum EntityType {
    Item(&'static ItemType),
    Mobile(&'static MobileType),
}

impl EntityType {
    pub fn full_name(self) -> &'static str {
        match self {
            Self::Item(t) => t.full_name,
            Self::Mobile(t) => t.full_name,
        }
    }

    pub fn construct_available(self) -> bool {
        match self {
            Self::Item(t) => t.construct.is_some(),
            Self::Mobile(t) => t.construct.is_some(),
        }
    }
}

/// Which static method name `AssemblyHandler.Invoke(name)` was asked for.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Phase {
    ConfigurePrompts,
    Configure,
    Initialize,
}

#[derive(Copy, Clone)]
struct PhaseFn {
    priority: i32,
    name: &'static str,
    run: fn(&mut World),
}

/// `Type.Name` of a full name: the text after the last `.` and then after the last `+` (nested classes).
fn short_name(full_name: &str) -> &str {
    let s = full_name.rsplit('.').next().unwrap_or(full_name);
    s.rsplit('+').next().unwrap_or(s)
}

#[derive(Default)]
pub struct TypeRegistry {
    /// Registration order, for enumeration (`[add`'s type list, `VerifySerialization`'s counts).
    items: Vec<&'static ItemType>,
    mobiles: Vec<&'static MobileType>,
    /// `_fullNameMap`, keyed by the hash of the case-sensitive name (class names and aliases).
    by_hash: HashMap<u64, Vec<EntityType>>,
    /// The four `TypeCache` maps folded into one: lower-case name and full name -> classes.
    by_lower_name: HashMap<String, Vec<EntityType>>,
    by_lower_full: HashMap<String, Vec<EntityType>>,
    by_name: HashMap<String, Vec<EntityType>>,
    prompts: Vec<PhaseFn>,
    configure: Vec<PhaseFn>,
    initialize: Vec<PhaseFn>,
}

fn push_unique(v: &mut Vec<EntityType>, t: EntityType) {
    let same = |a: &EntityType| match (a, &t) {
        (EntityType::Item(x), EntityType::Item(y)) => std::ptr::eq(*x, *y),
        (EntityType::Mobile(x), EntityType::Mobile(y)) => std::ptr::eq(*x, *y),
        _ => false,
    };
    if !v.iter().any(same) {
        v.push(t);
    }
}

impl TypeRegistry {
    /// An empty registry plus the two base classes, `Server.Item` and `Server.Mobile` (both concrete).
    pub fn new() -> Self {
        let mut r = Self::default();
        r.register_item(&ITEM_BASE, &[]);
        r.register_mobile(&MOBILE_BASE, &[]);
        r
    }

    fn add_names(&mut self, t: EntityType, full_name: &str) {
        push_unique(
            self.by_hash.entry(compute_hash64(full_name)).or_default(),
            t,
        );
        push_unique(
            self.by_name
                .entry(short_name(full_name).to_owned())
                .or_default(),
            t,
        );
        push_unique(
            self.by_lower_name
                .entry(short_name(full_name).lower_inv())
                .or_default(),
            t,
        );
        push_unique(
            self.by_lower_full.entry(full_name.lower_inv()).or_default(),
            t,
        );
    }

    /// Registers a concrete item class under its `full_name` and the `[TypeAlias]` full names `aliases`.
    /// Registering the same class again is a no-op.
    pub fn register_item(&mut self, ty: &'static ItemType, aliases: &[&str]) {
        if !self.items.iter().any(|t| std::ptr::eq(*t, ty)) {
            self.items.push(ty);
        }
        self.add_names(EntityType::Item(ty), ty.full_name);
        for a in aliases {
            self.add_names(EntityType::Item(ty), a);
        }
    }

    /// As [`TypeRegistry::register_item`] for a mobile class.
    pub fn register_mobile(&mut self, ty: &'static MobileType, aliases: &[&str]) {
        if !self.mobiles.iter().any(|t| std::ptr::eq(*t, ty)) {
            self.mobiles.push(ty);
        }
        self.add_names(EntityType::Mobile(ty), ty.full_name);
        for a in aliases {
            self.add_names(EntityType::Mobile(ty), a);
        }
    }

    /// Registered item classes, in registration order.
    pub fn item_types(&self) -> &[&'static ItemType] {
        &self.items
    }

    /// Registered mobile classes, in registration order.
    pub fn mobile_types(&self) -> &[&'static MobileType] {
        &self.mobiles
    }

    /// `FindTypeByHash(hash)`: items first, then mobiles.
    pub fn find_type_by_hash(&self, hash: u64) -> Option<EntityType> {
        self.by_hash.get(&hash)?.first().copied()
    }

    /// `FindTypeByName(name, fullName, ignoreCase)`; a blank name finds nothing.
    pub fn find_type_by_name(
        &self,
        name: &str,
        full_name: bool,
        ignore_case: bool,
    ) -> Option<EntityType> {
        if name.trim().is_empty() {
            return None;
        }
        let found = match (full_name, ignore_case) {
            (false, false) => self.by_name.get(name),
            (false, true) => self.by_lower_name.get(&name.lower_inv()),
            (true, false) => self.by_hash.get(&compute_hash64(name)),
            (true, true) => self.by_lower_full.get(&name.lower_inv()),
        };
        found?.first().copied()
    }

    /// `FindTypeByFullName(name, ignoreCase)`.
    pub fn find_type_by_full_name(&self, name: &str, ignore_case: bool) -> Option<EntityType> {
        self.find_type_by_name(name, true, ignore_case)
    }

    pub fn find_item_by_hash(&self, hash: u64) -> Option<&'static ItemType> {
        self.by_hash.get(&hash)?.iter().find_map(|t| match t {
            EntityType::Item(i) => Some(*i),
            _ => None,
        })
    }

    pub fn find_mobile_by_hash(&self, hash: u64) -> Option<&'static MobileType> {
        self.by_hash.get(&hash)?.iter().find_map(|t| match t {
            EntityType::Mobile(m) => Some(*m),
            _ => None,
        })
    }

    /// `VerifySerialization`'s tallies: `Core.ScriptItems` and `Core.ScriptMobiles`.
    pub fn counts(&self) -> (usize, usize) {
        (self.items.len(), self.mobiles.len())
    }

    /// A static `<Phase>()` method of some class, with its `[CallPriority]`.
    pub fn register_phase(
        &mut self,
        phase: Phase,
        name: &'static str,
        priority: i32,
        run: fn(&mut World),
    ) {
        let list = match phase {
            Phase::ConfigurePrompts => &mut self.prompts,
            Phase::Configure => &mut self.configure,
            Phase::Initialize => &mut self.initialize,
        };
        list.push(PhaseFn {
            priority,
            name,
            run,
        });
    }

    /// `register_phase` at the default priority.
    pub fn register_phase_default(
        &mut self,
        phase: Phase,
        name: &'static str,
        run: fn(&mut World),
    ) {
        self.register_phase(phase, name, DEFAULT_CALL_PRIORITY, run);
    }

    fn phase_fns(&self, phase: Phase) -> Vec<PhaseFn> {
        let mut v = match phase {
            Phase::ConfigurePrompts => self.prompts.clone(),
            Phase::Configure => self.configure.clone(),
            Phase::Initialize => self.initialize.clone(),
        };
        // List.Sort(CallPriorityComparer): introsort over the registration order, so equal priorities are unstable
        muo_primitives::dotnet::net_sort(&mut v, |a, b| a.priority.cmp(&b.priority));
        v
    }

    /// The registered functions of a phase, `[CallPriority]` order: `(name, priority)`.
    pub fn phase_order(&self, phase: Phase) -> Vec<(&'static str, i32)> {
        self.phase_fns(phase)
            .iter()
            .map(|p| (p.name, p.priority))
            .collect()
    }
}

/// `AssemblyHandler.Invoke(method)`: runs every function registered for `phase`, `[CallPriority]` order.
pub fn invoke_phase(w: &mut World, phase: Phase) {
    for p in w.types.phase_fns(phase) {
        (p.run)(w);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static MY_ITEM: ItemType = ItemType {
        full_name: "Server.Items.MyItem",
        parent: Some(&ITEM_BASE),
        ..ITEM_BASE
    };

    #[test]
    fn lookups_by_name_full_name_hash_and_alias() {
        let mut r = TypeRegistry::new();
        r.register_item(&MY_ITEM, &["Server.Items.OldMyItem"]);
        let full = |t: Option<EntityType>| t.map(EntityType::full_name);
        assert_eq!(
            full(r.find_type_by_name("myitem", false, true)),
            Some("Server.Items.MyItem")
        );
        assert_eq!(
            full(r.find_type_by_name("MyItem", false, false)),
            Some("Server.Items.MyItem")
        );
        assert_eq!(full(r.find_type_by_name("myitem", false, false)), None);
        assert_eq!(
            full(r.find_type_by_full_name("server.items.myitem", true)),
            Some("Server.Items.MyItem")
        );
        assert_eq!(
            full(r.find_type_by_name("OldMyItem", false, true)),
            Some("Server.Items.MyItem")
        );
        let h = compute_hash64("Server.Items.OldMyItem");
        assert!(r.find_item_by_hash(h).is_some());
        assert!(r.find_mobile_by_hash(h).is_none());
        assert_eq!(full(r.find_type_by_name("  ", false, true)), None);
        assert_eq!(r.counts(), (2, 1));
    }

    #[test]
    fn phases_run_by_priority_then_registration() {
        static ORDER: AtomicUsize = AtomicUsize::new(0);
        static SEEN: [AtomicUsize; 3] = [
            AtomicUsize::new(0),
            AtomicUsize::new(0),
            AtomicUsize::new(0),
        ];
        fn mark(i: usize) {
            SEEN[i].store(ORDER.fetch_add(1, Ordering::SeqCst) + 1, Ordering::SeqCst);
        }
        let mut w = World::new();
        w.types
            .register_phase_default(Phase::Initialize, "a", |_| mark(0));
        w.types
            .register_phase(Phase::Initialize, "b", 0, |_| mark(1));
        w.types
            .register_phase_default(Phase::Initialize, "c", |_| mark(2));
        assert_eq!(
            w.types.phase_order(Phase::Initialize),
            [("b", 0), ("a", 50), ("c", 50)]
        );
        invoke_phase(&mut w, Phase::Initialize);
        let seen: Vec<_> = SEEN.iter().map(|s| s.load(Ordering::SeqCst)).collect();
        assert_eq!(seen, [2, 1, 3]);
    }
}
