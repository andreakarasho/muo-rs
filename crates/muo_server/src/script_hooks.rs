//! Interception points a scripting layer fills in (`muo_content::scripting`).
//!
//! A dispatch site asks [`mobile`] / [`item`] before running a class's default behavior; `true` means a script
//! handled it and the default is skipped. Without a `ScriptHooks` resource, or for a class whose script does not
//! define the hook, the cost is one resource lookup and one map probe, with no allocation.

use std::collections::HashMap;

use crate::ids::{EntityId, ItemId, MobileId};
use crate::item::ItemType;
use crate::mobile::MobileType;
use crate::world::World;

/// The per-class hooks; the script function name and its parameters (the entity first) are in [`ScriptHook::fn_name`].
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum ScriptHook {
    /// `on_use(item, from)`: an item is double-clicked.
    Use,
    /// `on_death(m, corpse)`: after the corpse is made; skips the class's `OnDeath`.
    Death,
    /// `on_think(m)`: the creature AI think tick; skips the class's `OnThink`.
    Think,
    /// `on_damage(m, amount, from)`: before damage applies; handled means no damage.
    Damage,
    /// `on_hear(m, from, text)`: speech heard by a mobile; skips its `OnSpeech`.
    Hear,
    /// `on_spawn(e)`: after the class's template is applied. Nothing to skip.
    Spawn,
}

impl ScriptHook {
    pub const ALL: [ScriptHook; 6] = [
        ScriptHook::Use,
        ScriptHook::Death,
        ScriptHook::Think,
        ScriptHook::Damage,
        ScriptHook::Hear,
        ScriptHook::Spawn,
    ];

    /// The script function's name and parameter count (the entity included).
    pub const fn fn_name(self) -> (&'static str, usize) {
        match self {
            ScriptHook::Use => ("on_use", 2),
            ScriptHook::Death => ("on_death", 2),
            ScriptHook::Think => ("on_think", 1),
            ScriptHook::Damage => ("on_damage", 3),
            ScriptHook::Hear => ("on_hear", 3),
            ScriptHook::Spawn => ("on_spawn", 1),
        }
    }

    pub const fn bit(self) -> u8 {
        1 << self as u8
    }
}

/// An argument after the entity.
#[derive(Copy, Clone, Debug)]
pub enum ScriptArg<'a> {
    Mobile(Option<MobileId>),
    Item(Option<ItemId>),
    Int(i64),
    Str(&'a str),
}

pub type ScriptCallFn = fn(&mut World, EntityId, ScriptHook, &[ScriptArg]) -> bool;

/// Which classes have a script defining which hooks, keyed by the class's address (classes are `'static`).
#[derive(Default)]
pub struct ScriptHooks {
    pub mobiles: HashMap<usize, u8>,
    pub items: HashMap<usize, u8>,
    pub call: Option<ScriptCallFn>,
}

pub fn mobile_class_key(ty: &'static MobileType) -> usize {
    std::ptr::from_ref(ty) as usize
}

pub fn item_class_key(ty: &'static ItemType) -> usize {
    std::ptr::from_ref(ty) as usize
}

fn bound(map: &HashMap<usize, u8>, key: usize, hook: ScriptHook) -> bool {
    !map.is_empty() && map.get(&key).is_some_and(|b| b & hook.bit() != 0)
}

/// The mobile's class has a script defining `hook`.
pub fn mobile_has(w: &World, id: MobileId, hook: ScriptHook) -> bool {
    let (Some(h), Some(m)) = (w.resources.get::<ScriptHooks>(), w.mobile(id)) else {
        return false;
    };
    bound(&h.mobiles, mobile_class_key(m.ty), hook)
}

/// Runs the mobile class's script `hook`; true when the script handled it (skip the default).
pub fn mobile(w: &mut World, id: MobileId, hook: ScriptHook, args: &[ScriptArg]) -> bool {
    let call = match (w.resources.get::<ScriptHooks>(), w.mobile(id)) {
        (Some(h), Some(m)) if bound(&h.mobiles, mobile_class_key(m.ty), hook) => h.call,
        _ => return false,
    };
    call.is_some_and(|c| c(w, EntityId::Mobile(id), hook, args))
}

/// Runs the item class's script `hook`; true when the script handled it (skip the default).
pub fn item(w: &mut World, id: ItemId, hook: ScriptHook, args: &[ScriptArg]) -> bool {
    let call = match (w.resources.get::<ScriptHooks>(), w.item(id)) {
        (Some(h), Some(i)) if bound(&h.items, item_class_key(i.ty), hook) => h.call,
        _ => return false,
    };
    call.is_some_and(|c| c(w, EntityId::Item(id), hook, args))
}
