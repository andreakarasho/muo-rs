//! The `BaseTool` base of every craft tool (`ToolData` in `Item::ext`,
//! `ToolClass` for what a tool class overrides). The leaves (SmithHammer, Tongs, runic tools..) are
//! `muo_items::skill_items`; the craft system they open is muo_engines', reached through [`crate::craft::CraftHooks`].
//!
//! The T2A flow of `OnDoubleClick` (`T2ACraftToolTarget`) is the craft engine's `show_craft_menu` hook.

use muo_io::{GenericReader, Result};
use muo_server::World;
use muo_server::ext::Ext;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, base, deps, ops};
use muo_server::layer::Layer;
use muo_server::packets::message::AffixType;
use muo_server::serialization_ext::ReadEntityExt;

use super::equip_hooks::CraftArgs;
use crate::craft;

int_enum! {
    /// `ToolQuality`.
    pub enum ToolQuality { Low = 0, Regular = 1, Exceptional = 2 }
}

/// What `BaseTool` adds to an `Item`: `Crafter`, `Quality`, `UsesRemaining` (and the unsaved `ShowUsesRemaining`).
#[derive(Clone, Debug)]
pub struct ToolData {
    pub crafter: Option<String>,
    pub quality: ToolQuality,
    pub uses_remaining: i32,
    pub show_uses_remaining: bool,
}

/// The virtuals of a tool class.
#[derive(Copy, Clone)]
pub struct ToolClass {
    /// `CraftSystem`, by name (`Blacksmithy`, `Tailoring`..); empty for a tool that opens none.
    pub craft_system: &'static str,
    /// `BaseHarvestTool.HarvestSystem`, by name (`Mining`, `Lumberjacking`, `Fishing`); empty for a craft tool.
    pub harvest_system: &'static str,
    /// `BreakOnDepletion`.
    pub break_on_depletion: bool,
    /// `OnCraft(..)`.
    pub on_craft: fn(&mut World, ItemId, &CraftArgs) -> i32,
}

pub static BASE_TOOL_CLASS: ToolClass = ToolClass {
    craft_system: "",
    harvest_system: "",
    break_on_depletion: true,
    on_craft: on_craft_base,
};

pub fn class(w: &World, id: ItemId) -> &'static ToolClass {
    w.item(id)
        .and_then(|i| i.ty.family::<ToolClass>())
        .unwrap_or(&BASE_TOOL_CLASS)
}

pub fn data(w: &World, id: ItemId) -> Option<&ToolData> {
    w.item(id)?.ext.get::<ToolData>()
}

pub fn data_mut(w: &mut World, id: ItemId) -> Option<&mut ToolData> {
    w.item_mut(id)?.ext.get_mut::<ToolData>()
}

/// `item is BaseTool`.
pub fn is_tool(w: &World, id: ItemId) -> bool {
    w.item(id).is_some_and(|i| i.ty.is(&BASE_TOOL))
}

pub fn uses_remaining(w: &World, id: ItemId) -> i32 {
    data(w, id).map_or(0, |d| d.uses_remaining)
}

/// `UsesRemaining = value` (`InvalidateProperties` when it changed).
pub fn set_uses_remaining(w: &mut World, id: ItemId, value: i32) {
    let changed = match data_mut(w, id) {
        Some(d) if d.uses_remaining != value => {
            d.uses_remaining = value;
            true
        }
        _ => false,
    };
    if changed {
        ops::invalidate_properties(w, id);
    }
}

/// `Crafter = value`.
pub fn set_crafter(w: &mut World, id: ItemId, value: Option<String>) {
    let changed = match data_mut(w, id) {
        Some(d) if d.crafter != value => {
            d.crafter = value;
            true
        }
        _ => false,
    };
    if changed {
        ops::invalidate_properties(w, id);
    }
}

pub fn quality(w: &World, id: ItemId) -> ToolQuality {
    data(w, id).map_or(ToolQuality::Regular, |d| d.quality)
}

/// `GetUsesScalar()`.
pub fn uses_scalar(w: &World, id: ItemId) -> i32 {
    if quality(w, id) == ToolQuality::Exceptional {
        200
    } else {
        100
    }
}

/// `ScaleUses()`.
pub fn scale_uses(w: &mut World, id: ItemId) {
    let s = uses_scalar(w, id);
    let v = uses_remaining(w, id) * s / 100;
    set_uses_remaining(w, id, v);
    ops::invalidate_properties(w, id);
}

/// `UnscaleUses()`.
pub fn unscale_uses(w: &mut World, id: ItemId) {
    let s = uses_scalar(w, id);
    let v = uses_remaining(w, id) * 100 / s;
    set_uses_remaining(w, id, v);
}

/// `Quality = value`: unscale, set, scale.
pub fn set_quality(w: &mut World, id: ItemId, value: ToolQuality) {
    unscale_uses(w, id);
    if let Some(d) = data_mut(w, id) {
        d.quality = value;
    }
    scale_uses(w, id);
}

/// `new BaseTool(uses, itemID)` of class `ty`.
pub fn new_tool(w: &mut World, ty: &'static ItemType, item_id: i32, uses: i32) -> ItemId {
    let id = ops::new_item_with_id(w, ty, item_id);
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(ToolData {
            crafter: None,
            quality: ToolQuality::Regular,
            uses_remaining: uses,
            show_uses_remaining: true,
        });
    }
    id
}

/// `new BaseTool(itemID)`: `Utility.RandomMinMax(25, 75)` uses.
pub fn new_tool_random(w: &mut World, ty: &'static ItemType, item_id: i32) -> ItemId {
    let uses = super::random_min_max(w, 25, 75);
    new_tool(w, ty, item_id, uses)
}

/// `OnCraft(..)` of `BaseTool`.
fn on_craft_base(w: &mut World, id: ItemId, a: &CraftArgs) -> i32 {
    set_quality(
        w,
        id,
        ToolQuality::from_i32(a.quality).unwrap_or(ToolQuality::Regular),
    );
    if a.makers_mark {
        let name = muo_server::mobile::part1::props_b::raw_name(w, a.from);
        set_crafter(w, id, name);
    }
    if super::core_uor(w) {
        if quality(w, id) == ToolQuality::Exceptional {
            set_uses_remaining(w, id, 100);
        }
    } else if quality(w, id) != ToolQuality::Regular {
        let uses = uses_remaining(w, id);
        let q = quality(w, id) as i32;
        set_uses_remaining(w, id, uses + (uses as f64 * (q - 1) as f64 * 0.2) as i32);
    }
    a.quality
}

/// `OnCraft(..)` through the class.
pub fn on_craft(w: &mut World, id: ItemId, a: &CraftArgs) -> i32 {
    (class(w, id).on_craft)(w, id, a)
}

/// `BaseTool.CheckAccessible(tool, m)`: `tool.IsChildOf(m) || tool.Parent == m`.
pub fn check_accessible(w: &World, tool: ItemId, m: MobileId) -> bool {
    ops::is_child_of(w, tool, Some(EntityId::Mobile(m)))
        || w.item(tool)
            .is_some_and(|i| i.parent == Some(EntityId::Mobile(m)))
}

/// `BaseTool.CheckTool(tool, m)`: no other tool in either hand (except an AncientSmithyHammer).
pub fn check_tool(w: &World, tool: ItemId, m: MobileId) -> bool {
    let bad = |check: Option<ItemId>| {
        check.is_some_and(|c| {
            c != tool
                && w.item(c).is_some_and(|i| {
                    i.ty.is(&BASE_TOOL)
                        && !super::is_named(i.ty, "Server.Items.AncientSmithyHammer")
                })
        })
    };
    if bad(deps::find_item_on_layer(w, m, Layer::OneHanded)) {
        return false;
    }
    !bad(deps::find_item_on_layer(w, m, Layer::TwoHanded))
}

/// `DisplayDurabilityTo(m)`.
pub fn display_durability_to(w: &mut World, id: ItemId, m: MobileId) {
    let uses = uses_remaining(w, id);
    ops::label_to_affix(
        w,
        id,
        m,
        1017323,
        AffixType::APPEND,
        &format!(": {uses}"),
        "",
    ); // Durability
}

fn double_click(w: &mut World, id: ItemId, from: MobileId) {
    let pack = muo_server::mobile::part1::props_b::backpack(w, from);
    let in_pack = pack.is_some_and(|p| ops::is_child_of(w, id, Some(EntityId::Item(p))));
    let held = w
        .item(id)
        .is_some_and(|i| i.parent == Some(EntityId::Mobile(from)));
    if !(in_pack || held) {
        deps::send_localized_message(w, from, 1042001); // That must be in your pack for you to use it.
        return;
    }
    let system = class(w, id).craft_system;
    let Some(h) = craft::hooks(w) else { return };
    let (can, show) = (h.can_craft, h.show_craft_menu);
    let Some(can) = can else { return };
    let num = can(w, system, from, id, None);
    // Blacksmithing shows the gump regardless of proximity of an anvil and forge after SE
    if num > 0 && (num != 1044267 || !crate::core::se(w)) {
        deps::send_localized_message(w, from, num);
    } else if let Some(show) = show {
        show(w, system, from, id);
    }
}

fn default_data() -> ToolData {
    ToolData {
        crafter: None,
        quality: ToolQuality::Regular,
        uses_remaining: 0,
        show_uses_remaining: true,
    }
}

/// `Server.Items.BaseTool` (abstract).
pub static BASE_TOOL: ItemType = ItemType {
    full_name: "Server.Items.BaseTool",
    parent: Some(&ITEM_BASE),
    construct: None,
    family: Some(&BASE_TOOL_CLASS),
    get_properties: |w, id, list| {
        base::get_properties_base(w, id, list);
        if let Some(d) = data(w, id) {
            if d.quality == ToolQuality::Exceptional {
                list.add(1060636); // exceptional
            }
            list.add_int(1060584, d.uses_remaining); // uses remaining: ~1_val~
        }
    },
    on_single_click: |w, id, from| {
        display_durability_to(w, id, from);
        base::on_single_click_base(w, id, from);
    },
    on_double_click: double_click,
    serialize: |w, id, out| {
        base::serialize_base(w, id, out);
        let d = data(w, id).cloned().unwrap_or_else(default_data);
        out.write_i32(2);
        out.write_opt_string(d.crafter.as_deref());
        out.write_enum(4, d.quality as i64);
        out.write_i32(d.uses_remaining);
    },
    deserialize: |w, id, r| {
        base::deserialize_base(w, id, r)?;
        let version = r.read_i32()?;
        let d = if version < 2 {
            // Deserialize(reader, version): the crafter was a Mobile whose name is copied on a zero-delay timer.
            let crafter = r
                .read_mobile(w)?
                .and_then(|m| muo_server::mobile::part1::props_b::raw_name(w, m));
            let quality = ToolQuality::from_i32(r.read_i32()?).unwrap_or(ToolQuality::Regular);
            ToolData {
                crafter,
                quality,
                uses_remaining: r.read_i32()?,
                show_uses_remaining: true,
            }
        } else {
            let crafter = r.read_string()?;
            let quality = read_quality(r)?;
            ToolData {
                crafter,
                quality,
                uses_remaining: r.read_i32()?,
                show_uses_remaining: true,
            }
        };
        if let Some(i) = w.item_mut(id) {
            i.ext = Ext::new(d);
        }
        Ok(())
    },
    ..ITEM_BASE
};

fn read_quality(r: &mut dyn GenericReader) -> Result<ToolQuality> {
    super::read_enum(r, ToolQuality::from_i32, "ToolQuality")
}

// ---- BaseHarvestTool ----

/// `OnCraft(..)` of `BaseHarvestTool`: quality and maker's mark only (no uses change).
fn harvest_on_craft(w: &mut World, id: ItemId, a: &CraftArgs) -> i32 {
    set_quality(
        w,
        id,
        ToolQuality::from_i32(a.quality).unwrap_or(ToolQuality::Regular),
    );
    if a.makers_mark {
        let name = muo_server::mobile::part1::props_b::raw_name(w, a.from);
        set_crafter(w, id, name);
    }
    a.quality
}

pub static BASE_HARVEST_TOOL_CLASS: ToolClass = ToolClass {
    on_craft: harvest_on_craft,
    ..BASE_TOOL_CLASS
};

/// `new BaseHarvestTool(itemID, usesRemaining)` of class `ty`.
pub fn new_harvest_tool(w: &mut World, ty: &'static ItemType, item_id: i32, uses: i32) -> ItemId {
    new_tool(w, ty, item_id, uses)
}

/// `Server.Items.BaseHarvestTool` (abstract): the same record as `BaseTool`, its double click starts harvesting.
pub static BASE_HARVEST_TOOL: ItemType = ItemType {
    full_name: "Server.Items.BaseHarvestTool",
    family: Some(&BASE_HARVEST_TOOL_CLASS),
    on_double_click: |w, id, from| {
        let pack = muo_server::mobile::part1::props_b::backpack(w, from);
        let in_pack = pack.is_some_and(|p| ops::is_child_of(w, id, Some(EntityId::Item(p))));
        let held = w
            .item(id)
            .is_some_and(|i| i.parent == Some(EntityId::Mobile(from)));
        if in_pack || held {
            super::weapon_hooks::harvest_begin(w, from, id);
        } else {
            deps::send_localized_message(w, from, 1042001); // That must be in your pack for you to use it.
        }
    },
    get_context_menu_entries: |w, id, from, list| {
        base::get_context_menu_entries_base(w, id, from, list);
        super::weapon_hooks::harvest_context_entries(w, from, id, list);
    },
    ..BASE_TOOL
};
