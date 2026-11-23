//! `GargoylesPickaxe`, `SturdyPickaxe` and `ProspectorsTool`: weapons that harvest
//! (`BaseAxe` with `IUsesRemaining`, and a `BaseBashing` whose double click prospects).

use muo_content::items::weapon::{self, BASE_BASHING};
use muo_io::{GenericReader, GenericWriter};
use muo_server::item::{base, deps, ops};
use muo_server::target::{self, TargetFlags, Targeted};

use super::*;

/// `Utility.RandomMinMax(101, 125)`.
fn gargoyles_uses(w: &mut World) -> i32 {
    random_min_max(w, 101, 125)
}

/// `GargoylesPickaxe(uses)`: `base(0xE85 + Utility.Random(2))` then the uses.
fn gargoyles_with(w: &mut World, ty: &'static ItemType, uses: i32) -> ItemId {
    let graphic = 0xE85 + random(&mut w.rng, 2);
    let id = new_weapon(w, ty, graphic);
    ctor::set_uses_remaining(w, id, uses);
    ctor::set_show_uses_remaining(w, id, true);
    id
}

/// `GargoylesPickaxe()`: `this(Utility.RandomMinMax(101, 125))`; the arguments are evaluated first.
fn gargoyles_default(w: &mut World, ty: &'static ItemType) -> ItemId {
    let uses = gargoyles_uses(w);
    gargoyles_with(w, ty, uses)
}

weapon_leaf! {
    /// `Server.Items.GargoylesPickaxe`.
    GARGOYLES_PICKAXE, GARGOYLES_PICKAXE_CLASS, gargoyles_pickaxe_new = "Server.Items.GargoylesPickaxe" : BASE_AXE / BASE_AXE_CLASS, via gargoyles_default, v0;
}

/// `new GargoylesPickaxe(uses)`.
pub fn gargoyles_pickaxe_with_uses(w: &mut World, uses: i32) -> ItemId {
    gargoyles_with(w, &GARGOYLES_PICKAXE, uses)
}

weapon_leaf! {
    /// `Server.Items.SturdyPickaxe`.
    STURDY_PICKAXE, STURDY_PICKAXE_CLASS, sturdy_pickaxe_new = "Server.Items.SturdyPickaxe" : BASE_AXE / BASE_AXE_CLASS, json (), v0;
    @json;
}

/// `new SturdyPickaxe(uses)`.
pub fn sturdy_pickaxe_with_uses(w: &mut World, uses: i32) -> ItemId {
    let id = sturdy_pickaxe_new(w, &STURDY_PICKAXE);
    ctor::set_uses_remaining(w, id, uses);
    id
}

// ---- ProspectorsTool ----

/// What the mining system answers when a prospector's tool is used on a tile.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Prospect {
    /// 1049048: "You cannot use your prospector tool on that."
    Cannot,
    /// 1049049: "That ore looks to be prospected already."
    Already,
    /// 1049061: "You cannot improve valorite ore through prospecting."
    Valorite,
    /// The bank's vein was raised from this index.
    Improved(i32),
}

type ProspectFn = fn(&mut World, MobileId, ItemId, Targeted) -> Prospect;
/// `Prospect`'s use of `Mining.System`: set by the harvest engine (muo_engines). Unset: nothing can be prospected.
#[derive(Default)]
pub struct ProspectHooks {
    pub prospect: Option<ProspectFn>,
}

use muo_server::ids::{EntityId, MobileId};

/// `new ProspectorsTool()`: `base(graphic)`, then its template (50 uses, `ShowUsesRemaining` as
/// `IUsesRemaining.ShowUsesRemaining` is always true).
pub fn prospectors_tool_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let g = muo_content::items::equip_props::graphic(w, ty);
    let id = new_weapon(w, ty, g);
    muo_content::templates::apply_item_as(w, id, "Server.Items.ProspectorsTool");
    id
}

fn in_reach(w: &World, tool: ItemId, from: MobileId) -> bool {
    let pack = w.mobile(from).and_then(|m| m.backpack);
    ops::is_child_of(w, tool, pack.map(EntityId::Item))
        || w.item(tool)
            .is_some_and(|i| i.parent == Some(EntityId::Mobile(from)))
}

/// `Prospect(from, toProspect)`.
fn prospect(w: &mut World, tool: ItemId, from: MobileId, to: Targeted) {
    if !in_reach(w, tool, from) {
        deps::send_localized_message(w, from, 1042001); // That must be in your pack for you to use it.
        return;
    }
    let outcome = match w.resources.get::<ProspectHooks>().and_then(|h| h.prospect) {
        Some(f) => f(w, from, tool, to),
        None => Prospect::Cannot,
    };
    match outcome {
        Prospect::Cannot => deps::send_localized_message(w, from, 1049048), // You cannot use your prospector tool on that.
        Prospect::Already => deps::send_localized_message(w, from, 1049049), // That ore looks to be prospected already.
        Prospect::Valorite => deps::send_localized_message(w, from, 1049061), // You cannot improve valorite ore through prospecting.
        Prospect::Improved(vein_index) => {
            deps::send_localized_message(w, from, 1049050 + vein_index);
            let uses = weapon::data(w, tool).map_or(0, |d| d.uses_remaining) - 1;
            ctor::set_uses_remaining(w, tool, uses);
            if uses <= 0 {
                deps::send_localized_message(w, from, 1049062); // You have used up your prospector's tool.
                base::delete(w, tool);
            }
        }
    }
}

/// `Server.Items.ProspectorsTool`: `[SerializableField(0)] UsesRemaining` after the version.
pub static PROSPECTORS_TOOL: ItemType = ItemType {
    full_name: "Server.Items.ProspectorsTool",
    parent: Some(&BASE_BASHING),
    construct: Some(|w| prospectors_tool_new(w, &PROSPECTORS_TOOL)),
    on_double_click: |w, id, from| {
        if in_reach(w, id, from) {
            target::begin_target(w, from, 2, true, TargetFlags::NONE, move |w, from, t| {
                prospect(w, id, from, t)
            });
        } else {
            deps::send_localized_message(w, from, 1042001); // That must be in your pack for you to use it.
        }
    },
    serialize: |w, id, out: &mut dyn GenericWriter| {
        (BASE_BASHING.serialize)(w, id, out);
        out.write_i32(0);
        out.write_i32(weapon::data(w, id).map_or(0, |d| d.uses_remaining));
    },
    deserialize: |w, id, r: &mut dyn GenericReader| {
        (BASE_BASHING.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        let uses = r.read_i32()?;
        if let Some(d) = weapon::data_mut(w, id) {
            d.uses_remaining = uses;
            d.show_uses_remaining = true;
        }
        Ok(())
    },
    ..BASE_BASHING
};

/// Registers the harvest-tool weapons.
pub fn register(types: &mut TypeRegistry) {
    types.register_item(&GARGOYLES_PICKAXE, &[]);
    types.register_item(&STURDY_PICKAXE, &[]);
    types.register_item(&PROSPECTORS_TOOL, &[]);
}
