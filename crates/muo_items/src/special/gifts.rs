//! `TapestryOfSosaria`, `RoseOfTrinsic`, `HearthOfHomeFire`, `ZoogiFungus` and `PowderOfTranslocation`.
//!
//! `PowderOfTranslocation` recharges the `TranslocationItem` classes of `solen`.

use std::rc::Rc;

use muo_content::gumps::base::{RelayInfo, StaticGumpLogic, StaticGumpObj};
use muo_content::gumps::builder::StaticGumpBuilder;
use muo_content::gumps::send_gump_to_mobile;
use muo_content::items::addon::{
    self, AddonClass, BASE_ADDON, BASE_ADDON_CLASS, BASE_ADDON_DEED, DeedClass,
};
use muo_content::items::house_deps;

use muo_primitives::{DateTime, TimeSpan};
use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, LootType, deps, messages, ops};
use muo_server::light_type::LightType;
use muo_server::mobile::StatType;
use muo_server::mods::StatMod;
use muo_server::net_state::NetStateId;
use muo_server::target::{self, TargetFlags, Targeted};
use muo_server::timer::{self, TimerHandle};
use muo_server::type_registry::TypeRegistry;

use super::hooks::hooks;
use super::util::{in_pack, msg, overhead};

/// `ISecurable.Level` of the classes of this file (a `SecureLevel` int).
#[derive(Clone, Copy, Debug, Default)]
pub struct Secure(pub i32);

pub fn level(w: &World, id: ItemId) -> i32 {
    w.item(id)
        .and_then(|i| i.ext.get::<Secure>())
        .map_or(0, |s| s.0)
}

fn context_menu(
    w: &mut World,
    id: ItemId,
    from: MobileId,
    list: &mut Vec<muo_server::context_menu_entry::ContextMenuEntry>,
) {
    (ITEM_BASE.get_context_menu_entries)(w, id, from, list);
    if let Some(e) = muo_content::items::common::hooks(w)
        .and_then(|h| h.secure_level_entry)
        .and_then(|f| f(w, from, id))
    {
        list.push(e);
    }
}

/// `from.InRange(item.GetWorldLocation(), range)`.
fn in_reach(w: &World, id: ItemId, from: MobileId, range: i32) -> bool {
    muo_content::mx::in_range_point(w, from, ops::get_world_location(w, id), range)
}

// ---- TapestryOfSosaria ----

/// `TapestryOfSosariaGump`.
struct TapestryGump;

impl StaticGumpLogic for TapestryGump {
    const TYPE_NAME: &'static str = "Server.Items.TapestryOfSosaria+TapestryOfSosariaGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut StaticGumpBuilder) {
        b.add_page(0);
        b.add_image(0, 0, 0x2C95, 0);
    }
}

/// `Server.Items.TapestryOfSosaria` (`[Flippable(0x234E, 0x234F)]`).
pub static TAPESTRY_OF_SOSARIA: ItemType = ItemType {
    full_name: "Server.Items.TapestryOfSosaria",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let id = ops::new_item_with_id(w, &TAPESTRY_OF_SOSARIA, 0x234E);
        ops::set_loot_type(w, id, LootType::Blessed);
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(Secure(0));
        }
        id
    }),
    default_weight: crate::class_props::default_weight,
    label_number: crate::class_props::label_number, // The Tapestry of Sosaria
    get_context_menu_entries: context_menu,
    on_double_click: |w, id, from| {
        if in_reach(w, id, from, 2) {
            if super::util::net_state(w, from).is_some() {
                send_gump_to_mobile(
                    w,
                    from,
                    Box::new(StaticGumpObj::new(50, 50, TapestryGump)),
                    false,
                );
            }
        } else {
            overhead(w, from, 0x3B2, 1019045); // I can't reach that.
        }
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_encoded_int(1);
        out.write_enum(4, i64::from(level(w, id)));
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let version = r.read_encoded_int()?;
        let lvl = if version < 1 {
            r.read_encoded_int()?
        } else {
            r.read_enum(4)? as i32
        };
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(Secure(lvl));
        }
        Ok(())
    },
    ..ITEM_BASE
};

// ---- RoseOfTrinsic ----

const SPAWN_TIME: TimeSpan = TimeSpan::from_secs(4 * 3600);

#[derive(Clone, Copy, Debug)]
pub struct RoseData {
    pub petals: i32,
    pub next_spawn_time: DateTime,
    pub level: i32,
    pub timer: Option<TimerHandle>,
}

fn rose(w: &World, id: ItemId) -> RoseData {
    w.item(id)
        .and_then(|i| i.ext.get::<RoseData>())
        .copied()
        .unwrap_or(RoseData {
            petals: 0,
            next_spawn_time: DateTime::MIN,
            level: 0,
            timer: None,
        })
}

fn rose_mut(w: &mut World, id: ItemId) -> Option<&mut RoseData> {
    w.item_mut(id)?.ext.get_mut::<RoseData>()
}

fn stop_spawn_timer(w: &mut World, id: ItemId) {
    if let Some(h) = rose_mut(w, id).and_then(|d| d.timer.take()) {
        timer::stop(w, h);
    }
}

fn start_spawn_timer(w: &mut World, id: ItemId, delay: TimeSpan) {
    stop_spawn_timer(w, id);
    let h = timer::start(w, delay, TimeSpan::ZERO, 1, move |w| {
        if ops::is_deleted(w, id) {
            return;
        }
        if let Some(d) = rose_mut(w, id) {
            d.timer = None;
        }
        let p = rose(w, id).petals;
        set_petals(w, id, p + 1);
    });
    let at = w.now + delay;
    if let Some(d) = rose_mut(w, id) {
        d.timer = Some(h);
        d.next_spawn_time = at;
    }
}

/// `Petals = value`.
pub fn set_petals(w: &mut World, id: ItemId, value: i32) {
    if value >= 10 {
        if let Some(d) = rose_mut(w, id) {
            d.petals = 10;
        }
        stop_spawn_timer(w, id);
    } else {
        if let Some(d) = rose_mut(w, id) {
            d.petals = value.max(0);
        }
        start_spawn_timer(w, id, SPAWN_TIME);
    }
    ops::invalidate_properties(w, id);
}

/// `RoseOfTrinsicPetal(amount)`.
pub fn rose_petal_new(w: &mut World, amount: i32) -> ItemId {
    let id = crate::simple::from_template(w, &ROSE_OF_TRINSIC_PETAL, "Server.Items.RoseOfTrinsicPetal");
    ops::set_amount(w, id, amount);
    super::hue(w, id, 0xE);
    id
}

/// `Server.Items.RoseOfTrinsic` (`[Flippable(0x234C, 0x234D)]`).
pub static ROSE_OF_TRINSIC: ItemType = ItemType {
    full_name: "Server.Items.RoseOfTrinsic",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let id = ops::new_item_with_id(w, &ROSE_OF_TRINSIC, 0x234D);
        ops::set_loot_type(w, id, LootType::Blessed);
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(RoseData {
                petals: 0,
                next_spawn_time: DateTime::MIN,
                level: 0,
                timer: None,
            });
        }
        start_spawn_timer(w, id, TimeSpan::from_secs(60));
        id
    }),
    default_weight: crate::class_props::default_weight,
    label_number: crate::class_props::label_number, // Rose of Trinsic
    on_after_duped: |w, id, new_item| {
        (ITEM_BASE.on_after_duped)(w, id, new_item);
        if w.item(new_item).is_some_and(|i| i.ty.is(&ROSE_OF_TRINSIC)) {
            let next = rose(w, id).next_spawn_time;
            if let Some(d) = rose_mut(w, new_item) {
                d.next_spawn_time = next;
            }
        }
    },
    get_properties: |w, id, list| {
        (ITEM_BASE.get_properties)(w, id, list);
        list.add_int(1062925, rose(w, id).petals); // Petals:  ~1_COUNT~
    },
    get_context_menu_entries: context_menu,
    on_double_click: |w, id, from| {
        if !in_reach(w, id, from, 2) {
            overhead(w, from, 0x3B2, 1019045); // I can't reach that.
        } else if rose(w, id).petals > 0 {
            let petal = rose_petal_new(w, rose(w, id).petals);
            deps::add_to_backpack(w, from, petal);
            set_petals(w, id, 0);
        }
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_encoded_int(1);
        let d = rose(w, id);
        out.write_i32(d.petals);
        out.write_date_time_ticks(d.next_spawn_time.ticks());
        out.write_enum(4, i64::from(d.level));
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let version = r.read_encoded_int()?;
        let (petals, next, lvl);
        if version < 1 {
            petals = r.read_encoded_int()?;
            // shortcut: `ReadDeltaTime` is relative to the clock; the world's `now` stands in for it.
            next = DateTime::from_ticks(r.read_delta_time_ticks(w.now.ticks())?);
            lvl = r.read_encoded_int()?;
        } else {
            petals = r.read_i32()?;
            next = DateTime::from_ticks(r.read_date_time_ticks()?);
            lvl = r.read_enum(4)? as i32;
        }
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(RoseData {
                petals,
                next_spawn_time: next,
                level: lvl,
                timer: None,
            });
        }
        // [AfterDeserialization]
        if petals < 10 {
            let delay = next - w.now;
            start_spawn_timer(w, id, delay);
        }
        Ok(())
    },
    ..ITEM_BASE
};

/// `Server.Items.RoseOfTrinsicPetal`.
pub static ROSE_OF_TRINSIC_PETAL: ItemType = ItemType {
    full_name: "Server.Items.RoseOfTrinsicPetal",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| rose_petal_new(w, 1)),
    default_weight: crate::class_props::default_weight,
    label_number: crate::class_props::label_number, // Petal of the Rose of Trinsic
    on_double_click: |w, id, from| {
        if !in_pack(w, id, from) {
            msg(w, from, 1042038); // You must have the object in your backpack to use it.
        } else if muo_server::mobile::base::get_stat_mod(w, from, "RoseOfTrinsicPetal", false)
            .is_some()
        {
            // You have eaten one of these recently and eating another would provide no benefit.
            msg(w, from, 1062927);
        } else {
            muo_content::mx::play_sound(w, from, 0x1EE);
            let sm = StatMod::new(
                w,
                StatType::STR,
                Some("RoseOfTrinsicPetal"),
                5,
                TimeSpan::from_secs(300),
                None,
            );
            muo_server::mobile::base::add_stat_mod(w, from, sm);
            muo_server::item::base::consume(w, id, 1);
        }
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_encoded_int(0);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = r.read_encoded_int()?;
        Ok(())
    },
    ..ITEM_BASE
};

// ---- HearthOfHomeFire ----

fn light_component(w: &mut World, item_id: i32) -> ItemId {
    let c = addon::new_component(w, &addon::ADDON_COMPONENT, item_id);
    ops::set_light(w, c, LightType::Circle150);
    c
}

/// `HearthOfHomeFire(east)`.
pub fn hearth_of_home_fire_new(w: &mut World, east: bool) -> ItemId {
    let id = addon::new_addon(w, &HEARTH_OF_HOME_FIRE);
    let rows: [(i32, i32, i32); 2] = if east {
        [(0x2352, 0, 0), (0x2358, 0, -1)]
    } else {
        [(0x2360, 0, 0), (0x2366, -1, 0)]
    };
    for (item_id, x, y) in rows {
        let c = light_component(w, item_id);
        addon::add_component(w, id, c, x, y, 0);
    }
    id
}

static HEARTH_CLASS: AddonClass = AddonClass {
    deed: Some(hearth_deed_new),
    ..BASE_ADDON_CLASS
};

/// `Server.Items.HearthOfHomeFire`.
pub static HEARTH_OF_HOME_FIRE: ItemType = ItemType {
    full_name: "Server.Items.HearthOfHomeFire",
    parent: Some(&BASE_ADDON),
    construct: None,
    make: Some(|w| hearth_of_home_fire_new(w, false)),
    family: Some(&HEARTH_CLASS),
    serialize: |w, id, out| {
        (BASE_ADDON.serialize)(w, id, out);
        out.write_encoded_int(0);
    },
    deserialize: |w, id, r| {
        (BASE_ADDON.deserialize)(w, id, r)?;
        let _version = r.read_encoded_int()?;
        Ok(())
    },
    ..BASE_ADDON
};

/// `HearthOfHomeFireDeed()`.
pub fn hearth_deed_new(w: &mut World) -> ItemId {
    let id = addon::new_deed(w, &HEARTH_OF_HOME_FIRE_DEED);
    ops::set_loot_type(w, id, LootType::Blessed);
    id
}

static HEARTH_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, deed| hearth_of_home_fire_new(w, addon::deed_east(w, deed)),
    select_direction: None,
};

/// `HearthOfHomeFireDeed.InternalGump`.
struct HearthGump {
    deed: ItemId,
}

impl StaticGumpLogic for HearthGump {
    const TYPE_NAME: &'static str = "Server.Items.HearthOfHomeFireDeed+InternalGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut StaticGumpBuilder) {
        b.add_background(0, 0, 350, 250, 0xA28);
        b.add_page(0);
        b.add_item(90, 52, 0x2367, 0);
        b.add_item(112, 35, 0x2360, 0);
        b.add_button(70, 35, 0x868, 0x869, 1); // South
        b.add_item(220, 35, 0x2352, 0);
        b.add_item(242, 52, 0x2358, 0);
        b.add_button(185, 35, 0x868, 0x869, 2); // East
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        if ops::is_deleted(w, self.deed) || info.button_id == 0 {
            return;
        }
        let east = info.button_id != 1;
        if let Some(d) = w
            .item_mut(self.deed)
            .and_then(|i| i.ext.get_mut::<addon::DeedData>())
        {
            d.east = east;
        }
        if let Some(m) = w.net_states.get(&sender).and_then(|n| n.mobile) {
            addon::deed_on_double_click(w, self.deed, m); // SendTarget: base.OnDoubleClick
        }
    }
}

/// `Server.Items.HearthOfHomeFireDeed`.
pub static HEARTH_OF_HOME_FIRE_DEED: ItemType = ItemType {
    full_name: "Server.Items.HearthOfHomeFireDeed",
    parent: Some(&BASE_ADDON_DEED),
    construct: Some(hearth_deed_new),
    family: Some(&HEARTH_DEED_CLASS),
    label_number: crate::class_props::label_number, // Hearth of the Home Fire
    on_double_click: |w, id, from| {
        if in_pack(w, id, from) {
            send_gump_to_mobile(
                w,
                from,
                Box::new(StaticGumpObj::new(150, 50, HearthGump { deed: id })),
                false,
            );
        } else {
            msg(w, from, 1042001); // That must be in your pack for you to use it.
        }
    },
    serialize: |w, id, out| {
        (BASE_ADDON_DEED.serialize)(w, id, out);
        out.write_encoded_int(0);
    },
    deserialize: |w, id, r| {
        (BASE_ADDON_DEED.deserialize)(w, id, r)?;
        let _version = r.read_encoded_int()?;
        Ok(())
    },
    ..BASE_ADDON_DEED
};

// ---- Solen Items: ZoogiFungus, PowderOfTranslocation ----

fn stackable_new(w: &mut World, ty: &'static ItemType, item_id: i32, amount: i32) -> ItemId {
    let id = ops::new_item_with_id(w, ty, item_id);
    ops::set_stackable(w, id, true);
    ops::set_amount(w, id, amount);
    id
}

/// `ZoogiFungus(amount)`.
pub fn zoogi_fungus_new(w: &mut World, amount: i32) -> ItemId {
    stackable_new(w, &ZOOGI_FUNGUS, 0x26B7, amount)
}

/// `Server.Items.ZoogiFungus` (`ICommodity`, `IsDeedable => Core.ML`).
pub static ZOOGI_FUNGUS: ItemType = ItemType {
    full_name: "Server.Items.ZoogiFungus",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| zoogi_fungus_new(w, 1)),
    default_weight: crate::class_props::default_weight,
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_encoded_int(0);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = r.read_encoded_int()?;
        Ok(())
    },
    ..ITEM_BASE
};

/// `PowderOfTranslocation(amount)`.
pub fn powder_of_translocation_new(w: &mut World, amount: i32) -> ItemId {
    stackable_new(w, &POWDER_OF_TRANSLOCATION, 0x26B8, amount)
}

/// `item.SendLocalizedMessageTo(to, number, hue)`.
fn item_msg(w: &mut World, item: ItemId, to: MobileId, number: i32, hue: i32, args: &str) {
    if !ops::is_deleted(w, item) && deps::can_see(w, to, item) {
        messages::send_localized_hue(w, item, to, number, hue, args);
    }
}

fn powder_on_target(w: &mut World, powder: ItemId, from: MobileId, targeted: Targeted) {
    if ops::is_deleted(w, powder) {
        return;
    }
    if !in_reach(w, powder, from, 2) {
        overhead(w, from, 0x3B2, 1019045); // I can't reach that.
        return;
    }
    let api = match targeted {
        Targeted::Entity(EntityId::Item(t)) => hooks(w)
            .and_then(|h| h.translocation.iter().find(|a| (a.is_item)(w, t)).copied())
            .map(|a| (t, a)),
        _ => None,
    };
    let Some((item, api)) = api else {
        // Powder of translocation has no effect on this item.
        item_msg(w, powder, from, 1054140, 0x59, "");
        return;
    };
    let (charges, recharges, max_charges, max_recharges) = (api.get)(w, item);
    if charges >= max_charges {
        // This item cannot absorb any more powder of translocation.
        item_msg(w, powder, from, 1054137, 0x59, "");
    } else if recharges >= max_recharges {
        // This item has been oversaturated with powder of translocation and can no longer be recharged.
        item_msg(w, powder, from, 1054138, 0x59, "");
    } else {
        let amount = w.item(powder).map_or(0, |i| i.amount);
        if charges + amount > max_charges {
            let delta = max_charges - charges;
            ops::set_amount(w, powder, amount - delta);
            (api.set)(w, item, max_charges, recharges + delta);
        } else {
            (api.set)(w, item, charges + amount, recharges + amount);
            super::util::delete(w, powder);
        }
        // The ~1_translocationItem~ glows with green energy and absorbs magical power from the powder.
        let (number, string) = (api.name)(w, item);
        if number > 0 {
            item_msg(w, item, from, 1054139, 0x43, &format!("#{number}"));
        } else if let Some(s) = string {
            item_msg(w, item, from, 1054139, 0x43, &s);
        }
    }
}

/// `Server.Items.PowderOfTranslocation`.
pub static POWDER_OF_TRANSLOCATION: ItemType = ItemType {
    full_name: "Server.Items.PowderOfTranslocation",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| powder_of_translocation_new(w, 1)),
    default_weight: crate::class_props::default_weight,
    on_double_click: |w, id, from| {
        if in_reach(w, id, from, 2) {
            let mut t = target::Target::new(w, -1, false, TargetFlags::NONE);
            t.handlers.on_target = Some(Rc::new(move |w: &mut World, from, targeted| {
                powder_on_target(w, id, from, targeted)
            }));
            let tid = target::add_target(w, t);
            target::set_mobile_target(w, from, Some(tid));
        } else {
            overhead(w, from, 0x3B2, 1019045); // I can't reach that.
        }
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_encoded_int(0);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = r.read_encoded_int()?;
        Ok(())
    },
    ..ITEM_BASE
};

static AMOUNT_CTORS: [muo_content::props::Ctor; 1] = [muo_content::props::Ctor {
    access: None,
    params: &[muo_content::props::Param {
        name: "amount",
        kind: muo_content::props::Kind::Plain(muo_content::utilities::types::PropType::Numeric(
            muo_content::utilities::types::Numeric::Int,
        )),
        default: Some(muo_content::props::Value::Int(1)),
        ty: "",
    }],
    make: |w, v| EntityId::Item(powder_of_translocation_new(w, v[0].i32())),
}];

static ZOOGI_CTORS: [muo_content::props::Ctor; 1] = [muo_content::props::Ctor {
    access: None,
    params: AMOUNT_CTORS[0].params,
    make: |w, v| EntityId::Item(zoogi_fungus_new(w, v[0].i32())),
}];

/// Configure phase: constructors, `ISecurable`, commodities and flips.
pub(super) fn configure(w: &mut World) {
    use muo_content::props::add_ctors;
    add_ctors(w, "Server.Items.PowderOfTranslocation", &AMOUNT_CTORS);
    add_ctors(w, "Server.Items.ZoogiFungus", &ZOOGI_CTORS);
    crate::commodity::register(w, &ZOOGI_FUNGUS, muo_content::core::ml);
    house_deps::deps_mut(w).securables.push(|w, id| {
        let ty = w.item(id)?.ty;
        if ty.is(&TAPESTRY_OF_SOSARIA) {
            Some(level(w, id) as u8)
        } else if ty.is(&ROSE_OF_TRINSIC) {
            Some(rose(w, id).level as u8)
        } else {
            None
        }
    });
    house_deps::deps_mut(w)
        .securable_setters
        .push(|w, id, level| {
            let Some(ty) = w.item(id).map(|i| i.ty) else {
                return false;
            };
            if ty.is(&TAPESTRY_OF_SOSARIA) {
                if let Some(i) = w.item_mut(id) {
                    i.ext.insert(Secure(i32::from(level)));
                }
                true
            } else if ty.is(&ROSE_OF_TRINSIC) {
                if let Some(r) = rose_mut(w, id) {
                    r.level = i32::from(level);
                }
                true
            } else {
                false
            }
        });
}

pub fn register(types: &mut TypeRegistry) {
    for t in [
        &TAPESTRY_OF_SOSARIA,
        &ROSE_OF_TRINSIC,
        &ROSE_OF_TRINSIC_PETAL,
        &HEARTH_OF_HOME_FIRE,
        &HEARTH_OF_HOME_FIRE_DEED,
        &ZOOGI_FUNGUS,
        &POWDER_OF_TRANSLOCATION,
    ] {
        types.register_item(t, &[]);
    }
}
