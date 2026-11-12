//! `SpellScroll` (state: `ScrollData`) and the macro the
//! `scroll_list.rs` (113 subclasses) expands for each scroll.

use muo_content::context_menus::{ContextMenuHooks, entries};
use muo_content::player_mobile::hooks::design_check;
use muo_content::props::{Ctor, Kind, Param, Value, add_ctors};
use muo_content::spells::{registry, spell};
use muo_content::utilities::types::{Numeric, PropType};
use muo_server::World;
use muo_server::ext::Ext;
use muo_server::ids::{EntityId, ItemId};
use muo_server::item::{ITEM_BASE, ItemType, base, deps, ops};
use muo_server::type_registry::{Phase, TypeRegistry};

use super::hue;

/// `SpellScroll._spellID`.
#[derive(Clone, Copy, Debug, Default)]
pub struct ScrollData {
    pub spell_id: i32,
}

/// `SpellID`.
pub fn spell_id(w: &World, id: ItemId) -> Option<i32> {
    w.item(id)?.ext.get::<ScrollData>().map(|d| d.spell_id)
}

/// `item is SpellScroll`.
pub fn is_spell_scroll(w: &World, id: ItemId) -> bool {
    w.item(id).is_some_and(|i| i.ty.is(&SPELL_SCROLL))
}

/// A subclass's constructor: `base(spellID, itemID, amount)` then `Hue = hue` when the class sets one.
pub fn new_scroll_hued(
    w: &mut World,
    ty: &'static ItemType,
    spell_id: i32,
    item_id: i32,
    hue_value: i32,
    amount: i32,
) -> ItemId {
    let id = new_scroll(w, ty, spell_id, item_id, amount);
    if hue_value != 0 {
        hue(w, id, hue_value);
    }
    id
}

/// `new SpellScroll(spellID, itemID, amount)` of class `ty`.
pub fn new_scroll(
    w: &mut World,
    ty: &'static ItemType,
    spell_id: i32,
    item_id: i32,
    amount: i32,
) -> ItemId {
    let id = ops::new_item_with_id(w, ty, item_id);
    ops::set_stackable(w, id, true);
    ops::set_amount(w, id, amount);
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(ScrollData { spell_id });
    }
    id
}

fn on_double_click(w: &mut World, id: ItemId, from: muo_server::ids::MobileId) {
    if !design_check(w, from) {
        return; // They are customizing
    }
    let pack = muo_server::mobile::part1::props_b::backpack(w, from);
    if !ops::is_child_of(w, id, pack.map(EntityId::Item)) {
        deps::send_localized_message(w, from, 1042001); // That must be in your pack for you to use it.
        return;
    }
    let sid = spell_id(w, id).unwrap_or(0);
    match registry::new_spell_by_id(w, sid, from, Some(id)) {
        Some(sp) => {
            spell::cast(w, sp);
        }
        None => deps::send_localized_message(w, from, 502345), // This spell has been temporarily disabled.
    }
}

/// `Server.Items.SpellScroll`.
pub static SPELL_SCROLL: ItemType = ItemType {
    full_name: "Server.Items.SpellScroll",
    parent: Some(&ITEM_BASE),
    construct: None,
    default_weight: |_, _| 1.0,
    get_context_menu_entries: |w, id, from, list| {
        base::get_context_menu_entries_base(w, id, from, list);
        if muo_server::mobile::base::alive(w, from) && w.item(id).is_some_and(|i| i.movable()) {
            list.push(entries::add_to_spellbook());
        }
    },
    on_double_click,
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(0);
        out.write_i32(spell_id(w, id).unwrap_or(0));
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        let spell_id = r.read_i32()?;
        if let Some(i) = w.item_mut(id) {
            i.ext = Ext::new(ScrollData { spell_id });
        }
        Ok(())
    },
    ..ITEM_BASE
};

pub(super) const AMOUNT: Kind = Kind::Plain(PropType::Numeric(Numeric::Int));
/// `(int amount = 1)`.
pub(super) static AMOUNT_1: [Param; 1] = [Param {
    name: "amount",
    kind: AMOUNT,
    default: Some(Value::Int(1)),
    ty: "",
}];
static SPELL_SCROLL_CTOR: [Ctor; 1] = [Ctor {
    access: None,
    params: &[
        Param {
            name: "spellID",
            kind: AMOUNT,
            default: None,
            ty: "",
        },
        Param {
            name: "itemID",
            kind: AMOUNT,
            default: None,
            ty: "",
        },
        Param {
            name: "amount",
            kind: AMOUNT,
            default: Some(Value::Int(1)),
            ty: "",
        },
    ],
    make: |w, v| {
        EntityId::Item(new_scroll(
            w,
            &SPELL_SCROLL,
            v[0].i32(),
            v[1].i32(),
            v[2].i32(),
        ))
    },
}];

macro_rules! scrolls {
    ($($name:ident, $ctor:ident = $full:literal, $spell:expr, $gid:expr, $hue:expr, [$($alias:literal),*];)*) => {$(
        #[doc = concat!("`", $full, "`.")]
        pub static $name: ItemType = ItemType {
            full_name: $full,
            parent: Some(&SPELL_SCROLL),
            construct: Some(|w| new_scroll_hued(w, &$name, $spell, $gid, $hue, 1)),
            serialize: |w, id, out| {
                (SPELL_SCROLL.serialize)(w, id, out);
                out.write_i32(0);
            },
            deserialize: |w, id, r| {
                (SPELL_SCROLL.deserialize)(w, id, r)?;
                let _version = r.read_i32()?;
                Ok(())
            },
            ..SPELL_SCROLL
        };
        pub static $ctor: [Ctor; 1] = [Ctor {
            access: None,
            params: &AMOUNT_1,
            make: |w, v| EntityId::Item(new_scroll_hued(w, &$name, $spell, $gid, $hue, v[0].i32())),
        }];
    )*};
}

pub(super) use scrolls;

fn configure(w: &mut World) {
    w.resources.get_mut::<ContextMenuHooks>().is_spell_scroll = Some(is_spell_scroll);
    crate::commodity::register(w, &SPELL_SCROLL, muo_content::core::ml);
    add_ctors(w, "Server.Items.SpellScroll", &SPELL_SCROLL_CTOR);
    for (name, t) in super::scroll_list::CTORS {
        add_ctors(w, name, t);
    }
}

/// Registers `SpellScroll` and its subclasses.
pub fn register(types: &mut TypeRegistry) {
    types.register_item(&SPELL_SCROLL, &[]);
    for (t, aliases) in super::scroll_list::ALL {
        types.register_item(t, aliases);
    }
    types.register_phase_default(
        Phase::Configure,
        "Server.Items.SpellScroll.Configure",
        configure,
    );
}
