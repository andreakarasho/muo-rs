//! The Dark Tides quest's items and mobiles (namespace `Server.Engines.Quests.Necro`).
//!
//! `SummonedPaladinMoongate` is a plain item with a ten second life; `Mardoth`'s and `Horus`'s resurrection gump goes through [`QuestHooks::resurrect_gump_healer`](super::hooks).

use muo_content::creature::enums::{AIType, FightMode};
use muo_content::creature::setters::{new_creature, pack_gold, set_skill};
use muo_content::creature::{BASE_CREATURE, BASE_CREATURE_C, CreatureType};
use muo_content::gumps::{
    DynamicGumpBuilder, DynamicGumpLogic, DynamicGumpObj, send_gump_to_mobile,
};
use muo_content::items::addon::{self, ADDON_COMPONENT, BASE_ADDON};
use muo_content::items::cont::METAL_BOX;
use muo_primitives::{Point3D, TimeSpan};
use muo_server::World;
use muo_server::context_menu_entry::ContextMenuEntry;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, ops};
use muo_server::map::MapId;
use muo_server::mobile::{AccessLevel, MobileType};
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::skills::SkillName;

use super::core::{Conversation, Quest, can_offer_quest, focus_to, load, with_quest};
use super::dark_tides::*;
use super::hooks::quest_hooks;
use super::items::{
    DYNAMIC_TELEPORTER, HORN_OF_RETREAT, HornClass, HornData, QUEST_ITEM, QuestItemClass,
    TELEPORTER_CLASS, TeleporterClass, new_horn, new_teleporter,
};
use super::quester::{
    BASE_QUESTER, QUESTER_C, QUESTER_OPS, QUESTER_VC, QuesterOps, get_new_container,
};
use super::util::{add_item_hued, add_item_named, drop_amount, drop_new, in_backpack, set_body};
use crate::{quest_item, quester};

fn quest(w: &World, player: MobileId) -> Option<Quest> {
    load(w, player).filter(|q| q.is(&QUEST))
}

fn pending(q: &Quest, c: &super::core::ObjClass) -> Option<usize> {
    q.find_objective(c)
        .filter(|&i| !q.objectives[i].completed(q))
}

fn is_player(w: &World, m: MobileId) -> bool {
    w.mobile(m)
        .is_some_and(|m| m.ty.is(&muo_content::player_mobile::PLAYER_MOBILE))
}

// ---- the barriers, the horn, the teleporter ----

fn staff(w: &World, m: MobileId) -> bool {
    w.mobile(m)
        .is_some_and(|x| x.access_level > AccessLevel::Player)
}

quest_item! {
    /// `CrystalCaveBarrier`.
    CRYSTAL_CAVE_BARRIER = "Server.Engines.Quests.Necro.CrystalCaveBarrier" : ITEM_BASE, |w| {
        let id = ops::new_item_with_id(w, &CRYSTAL_CAVE_BARRIER, 0x3967);
        ops::set_movable(w, id, false);
        id
    };
    on_move_over: |w, _, m| {
        if staff(w, m) {
            return true;
        }
        let mut mob = Some(m);
        if muo_content::creature::is_creature(w, m) {
            let controlled = muo_content::creature::read(w, m, false, |c| c.controlled);
            mob = if controlled { muo_content::creature::control::control_master(w, m) } else { None };
        }
        let Some(pm) = mob.filter(|p| is_player(w, *p)) else { return false };
        if let Some(qs) = quest(w, pm)
            && let Some(i) = qs.find_objective(&SPEAK_CAVE_PASSWORD_OBJECTIVE)
            && qs.objectives[i].completed(&qs)
        {
            muo_content::mx::msg(w, m, 1060648);
            return true;
        }
        super::util::msg_hue(w, m, 1060649, 0x66D);
        false
    },
}

quest_item! {
    /// `VaultOfSecretsBarrier`.
    VAULT_OF_SECRETS_BARRIER = "Server.Engines.Quests.Necro.VaultOfSecretsBarrier" : ITEM_BASE, |w| {
        let id = ops::new_item_with_id(w, &VAULT_OF_SECRETS_BARRIER, 0x49E);
        ops::set_movable(w, id, false);
        ops::set_visible(w, id, false);
        id
    };
    on_move_over: |w, id, m| {
        if staff(w, m) {
            return true;
        }
        if is_player(w, m) && muo_content::player_mobile::data(w, m).is_some_and(|d| d.profession == 4) {
            super::util::msg_hue(w, m, 1060188, 0x24); // The wicked may not enter!
            return false;
        }
        (ITEM_BASE.on_move_over)(w, id, m)
    },
}

static DARK_TIDES_HORN_CLASS: HornClass = HornClass {
    validate_use: |w, _, from| is_player(w, from) && super::core::quest_is(w, from, &QUEST),
};

quest_item! {
    /// `DarkTidesHorn`.
    DARK_TIDES_HORN = "Server.Engines.Quests.Necro.DarkTidesHorn" : HORN_OF_RETREAT, |w| {
        let id = new_horn(w, &DARK_TIDES_HORN);
        if let Some(h) = w.item_mut(id).and_then(|i| i.ext.get_mut::<HornData>()) {
            h.dest_loc = Point3D::new(2103, 1319, -68);
            h.dest_map = Some(MapId::MALAS);
        }
        id
    };
    family: Some(&DARK_TIDES_HORN_CLASS),
}

static DARK_TIDES_TELEPORTER_CLASS: TeleporterClass = TeleporterClass {
    get_destination: |w, id, player| {
        let qs = quest(w, player)?;
        let malas = |x, y, z| Some((Point3D::new(x, y, z), MapId::MALAS));
        if qs.is_objective_in_progress(&FIND_MAABUS_TOMB_OBJECTIVE) {
            with_quest(w, player, |w, q| {
                q.add_conversation(w, Conversation::new(&RADAR_CONVERSATION))
            });
            return malas(2038, 1263, -90);
        }
        let _ = id;
        if qs.is_objective_in_progress(&FIND_CRYSTAL_CAVE_OBJECTIVE) {
            return malas(1194, 521, -90);
        }
        if qs.is_objective_in_progress(&FIND_CITY_OF_LIGHT_OBJECTIVE) {
            return malas(1091, 519, -90);
        }
        if qs.is_objective_in_progress(&RETURN_TO_CRYSTAL_CAVE_OBJECTIVE) {
            return malas(1194, 521, -90);
        }
        if has_lost_calling_scroll(w, player) {
            return malas(1194, 521, -90);
        }
        None
    },
    ..TELEPORTER_CLASS
};

quest_item! {
    /// `DarkTidesTeleporter` (`DynamicTeleporter()`: graphic 0x1822, hue 0x482).
    DARK_TIDES_TELEPORTER = "Server.Engines.Quests.Necro.DarkTidesTeleporter" : DYNAMIC_TELEPORTER, |w| {
        new_teleporter(w, &DARK_TIDES_TELEPORTER, 0x1822, 0x482)
    };
    family: Some(&DARK_TIDES_TELEPORTER_CLASS),
}

// ---- KronusScroll and its box ----

static KRONUS_SCROLL_CLASS: QuestItemClass = QuestItemClass {
    can_drop: |w, _, player| !super::core::quest_is(w, player, &QUEST),
    accepted: |w, id| w.item(id).is_none_or(|i| i.deleted()),
};

const WELL_OF_TEARS: muo_primitives::Rectangle2D =
    muo_primitives::Rectangle2D::new(2080, 1346, 10, 10);

/// `CallingTimer`: six ticks, one a second.
fn calling_tick(w: &mut World, player: MobileId, step: i32) {
    if muo_content::mx::deleted(w, player) {
        return;
    }
    if !muo_server::mobile::part1::props_b::mounted(w, player) {
        let a = if muo_content::hooks::rnd::bool(w) {
            16
        } else {
            17
        };
        muo_content::mx::animate(w, player, a, 7, 1, true, false, 0);
    }
    if step == 4 {
        let (loc, map) = (
            muo_content::mx::location(w, player),
            muo_content::mx::map(w, player),
        );
        let e = muo_server::item::effect_item::create_targeted(w, loc, map, TimeSpan::from_secs(1));
        muo_server::effects::send_location_particles(w, e, 0, 0, 0, 0x13C4);
        muo_server::effects::play_sound(w, loc, map, 0x243);
        for _ in 0..15 {
            let x = WELL_OF_TEARS.x() + muo_content::hooks::rnd::below(w, WELL_OF_TEARS.width());
            let y = WELL_OF_TEARS.y() + muo_content::hooks::rnd::below(w, WELL_OF_TEARS.height());
            let z = w.maps.get_average_z(MapId::MALAS, x, y);
            let dz = muo_content::hooks::rnd::min_max(w, 5, 20);
            let from = Point3D::new(x, y, z + dz);
            let to = Point3D::new(x, y, z);
            const HUES: [i32; 5] = [0x481, 0x482, 0x489, 0x497, 0x66D];
            let hue = HUES[muo_content::hooks::rnd::below(w, 5) as usize];
            muo_server::effects::send_moving_effect_map(
                w,
                Some(MapId::MALAS),
                0x36D4,
                from,
                to,
                0,
                0,
                false,
                true,
                hue,
                0,
            );
        }
    }
    if step < 5 {
        muo_content::mx::set_frozen(w, player, true);
    } else {
        muo_content::mx::set_frozen(w, player, false);
        begin_summon(w, player);
    }
}

/// `Server.Engines.Quests.Necro.KronusScroll`.
pub static KRONUS_SCROLL: ItemType = ItemType {
    full_name: "Server.Engines.Quests.Necro.KronusScroll",
    parent: Some(&QUEST_ITEM),
    construct: Some(|w| {
        let id = ops::new_item_with_id(w, &KRONUS_SCROLL, 0);
        muo_content::templates::apply_item_as(w, id, KRONUS_SCROLL.full_name);
        id
    }),
    family: Some(&KRONUS_SCROLL_CLASS),
    default_weight: |w, id| {
        crate::template_keys::item_float(w, id, "defaultWeight", QUEST_ITEM.default_weight)
    },
    label_number: |w, id| {
        crate::template_keys::item_int(w, id, "labelNumber", QUEST_ITEM.label_number)
    },
    on_double_click: |w, id, from| {
        if !ops::is_child_of(w, id, Some(EntityId::Mobile(from))) || !is_player(w, from) {
            return;
        }
        let Some(qs) = quest(w, from) else { return };
        if qs.is_objective_in_progress(&FIND_MARDOTH_ABOUT_KRONUS_OBJECTIVE) {
            super::util::msg_hue(w, from, 1060151, 0x41);
        } else if qs.is_objective_in_progress(&FIND_WELL_OF_TEARS_OBJECTIVE) {
            super::util::msg_hue(w, from, 1060152, 0x41);
        } else if qs.is_objective_in_progress(&USE_CALLING_SCROLL_OBJECTIVE) {
            let loc = muo_content::mx::location(w, from);
            if muo_content::mx::map(w, from) == Some(MapId::MALAS)
                && WELL_OF_TEARS.contains_xy(loc.x, loc.y)
            {
                if let Some(i) = pending(&qs, &USE_CALLING_SCROLL_OBJECTIVE) {
                    with_quest(w, from, |w, q| q.complete_objective(w, i));
                }
                muo_server::item::base::delete(w, id);
                // new CallingTimer(pm).Start(): immediately, then every second, six ticks
                let mut step = 0;
                muo_server::timer::start(w, TimeSpan::ZERO, TimeSpan::from_secs(1), 6, move |w| {
                    calling_tick(w, from, step);
                    step += 1;
                });
            } else {
                super::util::msg_hue(w, from, 1060152, 0x41);
            }
        } else {
            super::util::msg_hue(w, from, 1060150, 0x41);
        }
    },
    serialize: |w, id, out| {
        (QUEST_ITEM.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (QUEST_ITEM.deserialize)(w, id, r)?;
        r.read_i32()?;
        Ok(())
    },
    ..QUEST_ITEM
};

/// `Server.Engines.Quests.Necro.KronusScrollBox` (a `MetalBox`).
pub static KRONUS_SCROLL_BOX: ItemType = ItemType {
    full_name: "Server.Engines.Quests.Necro.KronusScrollBox",
    parent: Some(&METAL_BOX),
    construct: Some(|w| {
        let id = muo_content::items::cont::new_lockable(w, &KRONUS_SCROLL_BOX, 0);
        muo_content::templates::apply_item_as(w, id, KRONUS_SCROLL_BOX.full_name);
        for _ in 0..40 {
            if let Some(s) = muo_content::loot::random_scroll(
                w,
                0,
                15,
                muo_content::loot::SpellbookType::Necromancer,
            ) {
                ops::set_movable(w, s, false);
                muo_server::item::container::drop_item_default(w, id, s);
            }
        }
        id
    }),
    on_double_click: |w, id, from| {
        if is_player(w, from)
            && muo_content::mx::in_range_point(w, from, ops::get_world_location(w, id), 2)
            && let Some(qs) = quest(w, from)
        {
            let obj = pending(&qs, &FIND_CALLING_SCROLL_OBJECTIVE);
            if obj.is_some() || has_lost_calling_scroll(w, from) {
                let scroll =
                    muo_content::factory::make_item(w, "Server.Engines.Quests.Necro.KronusScroll");
                if scroll.is_some_and(|s| {
                    muo_server::mobile::part2::items::place_in_backpack(w, from, s)
                }) {
                    super::util::msg_hue(w, from, 1060120, 0x41);
                    if let Some(i) = obj {
                        with_quest(w, from, |w, q| q.complete_objective(w, i));
                    }
                } else {
                    super::util::msg_hue(w, from, 1060148, 0x41); // You were unable to take the scroll.
                    if let Some(s) = scroll {
                        muo_server::item::base::delete(w, s);
                    }
                }
            }
        }
        (METAL_BOX.on_double_click)(w, id, from);
    },
    serialize: |w, id, out| {
        (METAL_BOX.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (METAL_BOX.deserialize)(w, id, r)?;
        r.read_i32()?;
        Ok(())
    },
    ..METAL_BOX
};

// ---- ScrollOfAbraxus ----

/// `ScrollOfAbraxusGump`.
pub struct AbraxusLogic;

impl DynamicGumpLogic for AbraxusLogic {
    const TYPE_NAME: &'static str = "Server.Engines.Quests.Necro.ScrollOfAbraxusGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        b.add_page(0);
        b.add_image(0, 0, 1228, 0);
        b.add_image(340, 255, 9005, 0);
        b.add_html_localized_color(25, 36, 350, 210, 1060116, 1, false, true);
    }
}

static ABRAXUS_CLASS: QuestItemClass = QuestItemClass {
    can_drop: |w, _, player| !super::core::quest_is(w, player, &QUEST),
    accepted: |w, id| w.item(id).is_none_or(|i| i.deleted()),
};

/// `Server.Engines.Quests.Necro.ScrollOfAbraxus`.
pub static SCROLL_OF_ABRAXUS: ItemType = ItemType {
    full_name: "Server.Engines.Quests.Necro.ScrollOfAbraxus",
    parent: Some(&QUEST_ITEM),
    construct: Some(|w| crate::ml_quests::item_support::template_new(w, &SCROLL_OF_ABRAXUS)),
    family: Some(&ABRAXUS_CLASS),
    default_weight: |w, id| {
        crate::template_keys::item_float(w, id, "defaultWeight", QUEST_ITEM.default_weight)
    },
    label_number: |w, id| {
        crate::template_keys::item_int(w, id, "labelNumber", QUEST_ITEM.label_number)
    },
    on_added: |w, id, parent| {
        (QUEST_ITEM.on_added)(w, id, parent);
        if let Some(EntityId::Mobile(pm)) = ops::root_parent(w, id)
            && is_player(w, pm)
            && let Some(qs) = quest(w, pm)
            && let Some(i) = pending(&qs, &RETRIEVE_ABRAXUS_SCROLL_OBJECTIVE)
        {
            with_quest(w, pm, |w, q| q.complete_objective(w, i));
        }
    },
    on_double_click: |w, id, from| {
        if in_backpack(w, id, from) {
            if muo_server::mobile::part1::props_b::net_state(w, from).is_some() {
                send_gump_to_mobile(
                    w,
                    from,
                    Box::new(DynamicGumpObj::new(150, 50, AbraxusLogic)),
                    false,
                );
            }
            if is_player(w, from)
                && let Some(qs) = quest(w, from)
                && let Some(i) = pending(&qs, &READ_ABRAXUS_SCROLL_OBJECTIVE)
            {
                with_quest(w, from, |w, q| q.complete_objective(w, i));
            }
        } else {
            muo_content::mx::msg(w, from, 1042001); // That must be in your pack for you to use it.
        }
    },
    serialize: |w, id, out| {
        (QUEST_ITEM.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (QUEST_ITEM.deserialize)(w, id, r)?;
        r.read_i32()?;
        Ok(())
    },
    ..QUEST_ITEM
};

// ---- MaabusCoffin ----

/// `MaabusCoffin`'s `_maabus` and `_spawnLocation`.
#[derive(Clone, Default)]
pub struct CoffinData {
    pub maabus: Option<MobileId>,
    pub spawn_location: Point3D,
}

fn coffin(w: &World, id: ItemId) -> CoffinData {
    w.item(id)
        .and_then(|i| i.ext.get::<CoffinData>())
        .cloned()
        .unwrap_or_default()
}

/// `item is MaabusCoffin`.
pub fn coffin_is(w: &World, id: ItemId) -> bool {
    w.item(id)
        .is_some_and(|i| i.ext.get::<CoffinData>().is_some())
}

/// `MaabusCoffinComponent`'s `_fullItemID` and `_emptyItemID`.
#[derive(Clone, Copy, Default)]
pub struct CoffinComponentData {
    pub full: i32,
    pub empty: i32,
}

/// `new MaabusCoffinComponent(full, empty)`.
fn new_coffin_component(w: &mut World, full: i32, empty: i32) -> ItemId {
    let id = addon::new_component(w, &MAABUS_COFFIN_COMPONENT, full);
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(CoffinComponentData { full, empty });
    }
    id
}

fn turn_components(w: &mut World, coffin_id: ItemId, empty: bool) {
    for c in addon::components(w, coffin_id) {
        if let Some(d) = w
            .item(c)
            .and_then(|i| i.ext.get::<CoffinComponentData>())
            .copied()
            && let Some(f) = w.item(c).map(|i| i.ty.set_item_id)
        {
            f(w, c, if empty { d.empty } else { d.full });
        }
    }
}

/// `MaabusCoffin.Awake(caller)`.
pub fn coffin_awake(w: &mut World, id: ItemId, caller: MobileId) {
    let d = coffin(w, id);
    if d.maabus.is_some() || d.spawn_location == Point3D::ZERO {
        return;
    }
    turn_components(w, id, true);
    let map = w.item(id).and_then(|i| i.map).unwrap_or(MapId::INTERNAL);
    if let Some(m) = muo_content::factory::make_mobile(w, "Server.Engines.Quests.Necro.Maabus") {
        muo_server::mobile::base::move_to_world(w, m, d.spawn_location, map);
        let dir = muo_content::mx::get_direction_to(w, m, caller);
        muo_content::mx::set_direction(w, m, dir);
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(CoffinData {
                maabus: Some(m),
                spawn_location: d.spawn_location,
            });
        }
    }
    muo_server::timer::start(
        w,
        TimeSpan::from_secs_f64(7.5),
        TimeSpan::ZERO,
        1,
        move |w| begin_sleep(w, id),
    );
}

fn begin_sleep(w: &mut World, id: ItemId) {
    let Some(m) = coffin(w, id).maabus else {
        return;
    };
    let (loc, map) = (muo_content::mx::location(w, m), muo_content::mx::map(w, m));
    muo_server::effects::play_sound(w, loc, map, 0x48E);
    muo_server::timer::start(
        w,
        TimeSpan::from_secs_f64(2.5),
        TimeSpan::ZERO,
        1,
        move |w| sleep(w, id),
    );
}

/// `MaabusCoffin.Sleep()`.
fn sleep(w: &mut World, id: ItemId) {
    let Some(m) = coffin(w, id).maabus else {
        return;
    };
    let (loc, map) = (muo_content::mx::location(w, m), muo_content::mx::map(w, m));
    let e = muo_server::item::effect_item::create_targeted(
        w,
        loc,
        map,
        muo_server::item::effect_item::DEFAULT_DURATION,
    );
    muo_server::effects::send_location_particles(w, e, 0x3728, 10, 10, 0x7E7);
    muo_server::effects::play_sound(w, loc, map, 0x1FE);
    muo_server::mobile::base::delete(w, m);
    if let Some(d) = w.item_mut(id).and_then(|i| i.ext.get_mut::<CoffinData>()) {
        d.maabus = None;
    }
    turn_components(w, id, false);
}

/// `Server.Engines.Quests.Necro.MaabusCoffinComponent`.
pub static MAABUS_COFFIN_COMPONENT: ItemType = ItemType {
    full_name: "Server.Engines.Quests.Necro.MaabusCoffinComponent",
    parent: Some(&ADDON_COMPONENT),
    construct: None,
    serialize: |w, id, out| {
        (ADDON_COMPONENT.serialize)(w, id, out);
        out.write_i32(0);
        let d = w
            .item(id)
            .and_then(|i| i.ext.get::<CoffinComponentData>())
            .copied()
            .unwrap_or_default();
        out.write_i32(d.full);
        out.write_i32(d.empty);
    },
    deserialize: |w, id, r| {
        (ADDON_COMPONENT.deserialize)(w, id, r)?;
        r.read_i32()?;
        let d = CoffinComponentData {
            full: r.read_i32()?,
            empty: r.read_i32()?,
        };
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(d);
        }
        Ok(())
    },
    ..ADDON_COMPONENT
};

/// `Server.Engines.Quests.Necro.MaabusCoffin`.
pub static MAABUS_COFFIN: ItemType = ItemType {
    full_name: "Server.Engines.Quests.Necro.MaabusCoffin",
    parent: Some(&BASE_ADDON),
    construct: Some(|w| {
        let id = addon::new_addon(w, &MAABUS_COFFIN);
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(CoffinData::default());
        }
        for (full, empty, x, y) in [
            (0x1C2B, 0x1C2B, -1, -1),
            (0x1D16, 0x1C2C, 0, -1),
            (0x1D17, 0x1C2D, 1, -1),
            (0x1D51, 0x1C2E, 2, -1),
            (0x1D4E, 0x1C2A, 0, 0),
            (0x1D4D, 0x1C29, 1, 0),
            (0x1D4C, 0x1C28, 2, 0),
        ] {
            let c = new_coffin_component(w, full, empty);
            addon::add_component(w, id, c, x, y, 0);
        }
        id
    }),
    serialize: |w, id, out| {
        (BASE_ADDON.serialize)(w, id, out);
        out.write_i32(0);
        let d = coffin(w, id);
        out.write_mobile(w, d.maabus);
        out.write_point3d(d.spawn_location);
    },
    deserialize: |w, id, r| {
        (BASE_ADDON.deserialize)(w, id, r)?;
        r.read_i32()?;
        let maabus = r.read_mobile(w)?;
        let spawn_location = r.read_point3d()?;
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(CoffinData {
                maabus,
                spawn_location,
            });
        }
        // [AfterDeserialization(false)]: Sleep()
        sleep(w, id);
        Ok(())
    },
    ..BASE_ADDON
};

// ---- Horus ----

static HORUS_OPS: QuesterOps = QuesterOps {
    auto_talk_range: |_, _, _| 3,
    can_talk_to: |w, _, to| {
        quest(w, to).is_some_and(|q| q.is_objective_in_progress(&FIND_CRYSTAL_CAVE_OBJECTIVE))
    },
    on_talk: |w, _, player, _| {
        if let Some(qs) = quest(w, player)
            && let Some(i) = pending(&qs, &FIND_CRYSTAL_CAVE_OBJECTIVE)
        {
            with_quest(w, player, |w, q| q.complete_objective(w, i));
        }
    },
    ..QUESTER_OPS
};

/// `Horus.OnPasswordSpoken(from)`.
fn on_password_spoken(w: &mut World, from: MobileId) {
    let obj = quest(w, from).and_then(|q| pending(&q, &SPEAK_CAVE_PASSWORD_OBJECTIVE));
    match obj {
        Some(i) => {
            with_quest(w, from, |w, q| q.complete_objective(w, i));
        }
        None => muo_content::mx::msg(w, from, 1060185), // Horus ignores you.
    }
}

quester! {
    /// `Server.Engines.Quests.Necro.Horus`.
    HORUS, HORUS_C, HORUS_V = "Server.Engines.Quests.Necro.Horus" : BASE_QUESTER / QUESTER_C / QUESTER_VC,
        ops HORUS_OPS, encoded false, title Some("the Guardian"), extra |_, _| {};
    v {
        init_body: |w, id| muo_content::templates::apply_mobile_as(w, id, "Server.Engines.Quests.Necro.Horus"),
        init_outfit: |_, _| {},
    }
    c { default_name: muo_content::templates::creature_props::default_name, }
    m {
        get_context_menu_entries: |w, horus, from, list| {
            (BASE_QUESTER.get_context_menu_entries)(w, horus, from, list);
            if !muo_content::mx::alive(w, from) || !is_player(w, from) {
                return;
            }
            if let Some(qs) = quest(w, from) {
                let enabled = pending(&qs, &SPEAK_CAVE_PASSWORD_OBJECTIVE).is_some();
                let mut e = ContextMenuEntry::new(6193, 3);
                e.enabled = enabled;
                e.on_click = Some(std::rc::Rc::new(|w: &mut World, from: MobileId, target: EntityId| {
                    if matches!(target, EntityId::Mobile(_)) && muo_content::mx::alive(w, from) && is_player(w, from) {
                        on_password_spoken(w, from);
                    }
                }));
                list.push(e);
            }
        },
        on_movement: |w, horus, m, old| {
            (BASE_QUESTER.on_movement)(w, horus, m, old);
            if !muo_content::mx::in_range_point(w, horus, muo_content::mx::location(w, m), 2)
                || muo_content::mx::in_range_point(w, horus, old, 2)
                || !is_player(w, m)
            {
                return;
            }
            let Some(qs) = quest(w, m) else { return };
            if let Some(i) = pending(&qs, &RETURN_TO_CRYSTAL_CAVE_OBJECTIVE) {
                with_quest(w, m, |w, q| q.complete_objective(w, i));
                return;
            }
            if let Some(i) = pending(&qs, &FIND_HORUS_ABOUT_REWARD_OBJECTIVE) {
                let cont = get_new_container(w);
                drop_amount(w, cont, "Server.Items.Gold", 500);
                if let Some(jewel) = muo_content::factory::make_item(w, "Server.Items.GoldBracelet") {
                    if muo_server::mobile::part1::props_a::core_aos(w)
                        && let Some(f) = quest_hooks(w).and_then(|h| h.apply_runic_attributes)
                    {
                        f(w, jewel, 3, 20, 40);
                    }
                    muo_server::item::container::drop_item_default(w, cont, jewel);
                }
                if muo_server::mobile::part2::items::place_in_backpack(w, m, cont) {
                    with_quest(w, m, |w, q| q.complete_objective(w, i));
                } else {
                    muo_server::item::base::delete(w, cont);
                    muo_content::mx::msg(w, m, 1046260);
                }
            }
        },
    }
}

// ---- Maabus ----

static MAABUS_OPS: QuesterOps = QuesterOps {
    can_talk_to: |_, _, _| false,
    ..QUESTER_OPS
};

quester! {
    /// `Server.Engines.Quests.Necro.Maabus`.
    MAABUS, MAABUS_C, MAABUS_V = "Server.Engines.Quests.Necro.Maabus" : BASE_QUESTER / QUESTER_C / QUESTER_VC,
        ops MAABUS_OPS, encoded false, title None, extra |_, _| {}, makeonly;
    v {
        init_body: |w, id| muo_content::templates::apply_mobile_as(w, id, "Server.Engines.Quests.Necro.Maabus"),
    }
    c { default_name: muo_content::templates::creature_props::default_name, }
    m { }
}

// ---- Mardoth ----

static MARDOTH_OPS: QuesterOps = QuesterOps {
    auto_talk_range: |_, _, _| 3,
    can_talk_to: |w, _, to| match quest(w, to) {
        None => !super::core::has_quest(w, to) && offerable(w, to),
        Some(q) => q.has_objective(&FIND_MARDOTH_ABOUT_VAULT_OBJECTIVE),
    },
    on_talk: mardoth_on_talk,
    ..QUESTER_OPS
};

/// `CanOfferQuest(to, typeof(DarkTidesQuest))` as a read.
fn offerable(w: &World, to: MobileId) -> bool {
    let Some(d) = muo_content::player_mobile::data(w, to) else {
        return false;
    };
    if !QUEST.professions.contains(&d.profession) {
        return false;
    }
    let ns = muo_server::mobile::part1::props_b::net_state(w, to);
    if ns.is_some_and(|ns| muo_content::gumps::has_gump::<super::gumps::OfferLogic>(w, ns)) {
        return false;
    }
    !d.done_quests.iter().any(|(t, restart)| {
        let ours = match t {
            muo_content::player_mobile::QuestTypeRef::Indexed(i) => *i == QUEST.slot as i32,
            muo_content::player_mobile::QuestTypeRef::Named(n) => n == QUEST.name,
            muo_content::player_mobile::QuestTypeRef::None => false,
        };
        ours && w.now < *restart
    })
}

fn mardoth_on_talk(w: &mut World, mardoth: MobileId, player: MobileId, context_menu: bool) {
    let qs = quest(w, player);
    if !super::core::has_quest(w, player) && can_offer_quest(w, player, &QUEST) {
        Quest::new(w, &QUEST, player).send_offer(w);
        return;
    }
    let Some(qs) = qs else { return };
    if has_lost_calling_scroll(w, player) {
        with_quest(w, player, |w, q| {
            q.add_conversation(w, lost_calling_scroll(true))
        });
        return;
    }
    if let Some(i) = pending(&qs, &FIND_MARDOTH_ABOUT_VAULT_OBJECTIVE) {
        with_quest(w, player, |w, q| q.complete_objective(w, i));
        return;
    }
    if let Some(i) = pending(&qs, &FIND_MARDOTH_ABOUT_KRONUS_OBJECTIVE) {
        with_quest(w, player, |w, q| q.complete_objective(w, i));
        return;
    }
    if let Some(i) = pending(&qs, &FIND_MARDOTH_END_OBJECTIVE) {
        let cont = get_new_container(w);
        drop_amount(w, cont, "Server.Items.PigIron", 20);
        drop_amount(w, cont, "Server.Items.NoxCrystal", 20);
        drop_amount(w, cont, "Server.Items.BatWing", 25);
        drop_amount(w, cont, "Server.Items.DaemonBlood", 20);
        drop_amount(w, cont, "Server.Items.GraveDust", 20);
        if let Some(weapon) = muo_content::factory::make_item(w, "Server.Items.BoneHarvester") {
            if let Some(d) = muo_content::items::weapon::data_mut(w, weapon) {
                d.slayer = muo_content::items::slayer::SlayerName::OrcSlaying as i32; // weapon.Slayer = SlayerName.OrcSlaying
            }
            if muo_server::mobile::part1::props_a::core_aos(w) {
                if let Some(f) = quest_hooks(w).and_then(|h| h.apply_runic_attributes) {
                    f(w, weapon, 3, 20, 40);
                }
            } else {
                use muo_content::items::weapon::{
                    WeaponAccuracyLevel, WeaponDamageLevel, WeaponDurabilityLevel,
                    set_accuracy_level, set_damage_level, set_durability_level,
                };
                let scaled =
                    |w: &mut World| muo_content::creature::packing::random_min_max_scaled(w, 2, 4);
                let n = scaled(w);
                set_damage_level(
                    w,
                    weapon,
                    WeaponDamageLevel::from_i32(n).unwrap_or(WeaponDamageLevel::Regular),
                );
                let n = scaled(w);
                set_accuracy_level(
                    w,
                    weapon,
                    WeaponAccuracyLevel::from_i32(n).unwrap_or(WeaponAccuracyLevel::Regular),
                );
                let n = scaled(w);
                set_durability_level(
                    w,
                    weapon,
                    WeaponDurabilityLevel::from_i32(n).unwrap_or(WeaponDurabilityLevel::Regular),
                );
            }
            muo_server::item::container::drop_item_default(w, cont, weapon);
        }
        let check = muo_content::items::gold::new_bank_check(w, 2000);
        muo_server::item::container::drop_item_default(w, cont, check);
        drop_new(w, cont, "Server.Items.EnchantedSextant");
        if !muo_server::mobile::part2::items::place_in_backpack(w, player, cont) {
            muo_server::item::base::delete(w, cont);
            muo_content::mx::msg(w, player, 1046260);
        } else {
            with_quest(w, player, |w, q| q.complete_objective(w, i));
        }
    } else if context_menu {
        focus_to(w, mardoth, player);
        muo_content::mx::msg(w, player, 1061821); // Mardoth has nothing more for you at this time.
    }
}

quester! {
    /// `Server.Engines.Quests.Necro.Mardoth`.
    MARDOTH, MARDOTH_C, MARDOTH_V = "Server.Engines.Quests.Necro.Mardoth" : BASE_QUESTER / QUESTER_C / QUESTER_VC,
        ops MARDOTH_OPS, encoded false, title Some("the Ancient Necromancer"), extra |_, _| {};
    v {
        init_body: |w, id| muo_content::templates::apply_mobile_as(w, id, "Server.Engines.Quests.Necro.Mardoth"),
        init_outfit: |_, _| {},
    }
    c { default_name: muo_content::templates::creature_props::default_name, }
    m {
        on_drag_drop: |w, mardoth, from, dropped| {
            if !super::util::item_is(w, dropped, "Server.Engines.Quests.Necro.DarkTidesHorn")
                || !is_player(w, from)
                || quest(w, from).is_none()
            {
                return (BASE_QUESTER.on_drag_drop)(w, mardoth, from, dropped);
            }
            if muo_content::player_mobile::young(w, from) {
                let charges = super::items::horn(w, dropped).map_or(0, |h| h.charges);
                if charges < 10 {
                    super::util::say_to_number(w, mardoth, from, 1049384); // I have recharged the item for you.
                    if let Some(h) = super::items::horn_mut(w, dropped) {
                        h.charges = 10;
                    }
                } else {
                    super::util::say_to_number(w, mardoth, from, 1049385); // That doesn't need recharging yet.
                }
            } else {
                muo_content::mx::msg(w, from, 1114333); // You must be young to have this item recharged.
            }
            false
        },
        on_movement: |w, npc, m, old| {
            (BASE_QUESTER.on_movement)(w, npc, m, old);
            super::emino_mobs::resurrect_on_movement(w, npc, m, old, true);
        },
    }
}

// ---- SummonedPaladin ----

/// `SummonedPaladin`'s `_necromancer` and `_toDelete`.
#[derive(Clone, Default)]
pub struct PaladinData {
    pub necromancer: Option<MobileId>,
    pub to_delete: bool,
}

fn paladin(w: &World, id: MobileId) -> PaladinData {
    w.mobile(id)
        .and_then(|m| m.ext.get::<PaladinData>())
        .cloned()
        .unwrap_or_default()
}

fn set_paladin(w: &mut World, id: MobileId, f: impl FnOnce(&mut PaladinData)) {
    if let Some(m) = w.mobile_mut(id) {
        if m.ext.get::<PaladinData>().is_none() {
            m.ext.insert(PaladinData::default());
        }
        if let Some(d) = m.ext.get_mut::<PaladinData>() {
            f(d);
        }
    }
}

/// `new SummonedPaladin(necromancer)`.
pub fn new_summoned_paladin(w: &mut World, necromancer: MobileId) -> MobileId {
    let id = new_creature(
        w,
        &SUMMONED_PALADIN,
        AIType::AiMelee,
        FightMode::Aggressor,
        muo_content::creature::enums::DEFAULT_RANGE_PERCEPTION,
        1,
    );
    set_paladin(w, id, |d| d.necromancer = Some(necromancer));
    muo_server::mobile::part2::stats::init_stats(w, id, 45, 30, 5);
    muo_server::mobile::part1::props_a::set_title(w, id, Some("the Paladin".to_owned()));
    muo_server::mobile::base::set_hue(w, id, 0x83F3);
    muo_server::mobile::part1::props_b::set_female(w, id, false);
    set_body(w, id, 0x190);
    let name = muo_content::misc::small::random_name(w, "male");
    if let Some(f) = w.mobile(id).map(|m| m.ty.set_name) {
        f(w, id, Some(name));
    }
    muo_content::vendor::assign_random_hair_random_hue(w, id);
    super::util::assign_facial_item(w, id);
    let hh = w.mobile(id).map_or(0, |m| m.hair_hue);
    muo_server::mobile::part1::props_b::set_facial_hair_hue(w, id, hh);
    add_item_hued(w, id, "Server.Items.Boots", 0x1);
    add_item_named(w, id, "Server.Items.ChainChest");
    add_item_named(w, id, "Server.Items.ChainLegs");
    add_item_named(w, id, "Server.Items.RingmailArms");
    add_item_named(w, id, "Server.Items.PlateHelm");
    add_item_named(w, id, "Server.Items.PlateGloves");
    add_item_named(w, id, "Server.Items.PlateGorget");
    add_item_hued(w, id, "Server.Items.Cloak", 0xCF);
    add_item_named(w, id, "Server.Items.ThinLongsword");
    set_skill(w, id, SkillName::Swords, 50.0);
    set_skill(w, id, SkillName::Tactics, 50.0);
    pack_gold(w, id, 500);
    id
}

static SUMMONED_PALADIN_C: CreatureType = CreatureType {
    player_range_sensitive: |_, _| false,
    on_think: |w, id| {
        let d = paladin(w, id);
        let frozen = w.mobile(id).is_some_and(|m| m.frozen);
        if !d.to_delete && !frozen {
            let nec = d.necromancer;
            let gone = nec.is_none_or(|n| {
                muo_content::mx::deleted(w, n)
                    || w.mobile(n).is_some_and(|m| m.map == Some(MapId::INTERNAL))
            });
            if gone {
                muo_server::mobile::base::delete(w, id);
                return;
            }
            let nec = nec.expect("checked");
            if muo_content::mx::combatant(w, id) != Some(nec) {
                muo_content::mx::set_combatant(w, id, Some(nec));
            }
            if !muo_content::mx::alive(w, nec) {
                if super::core::quest_is(w, nec, &QUEST)
                    && let Some(qs) = quest(w, nec)
                    && !qs.has_objective(&FIND_MARDOTH_END_OBJECTIVE)
                {
                    with_quest(w, nec, |w, q| q.add_objective(w, find_mardoth_end(false)));
                }
                let name = w
                    .mobile(nec)
                    .and_then(|m| (m.ty.name)(w, nec))
                    .unwrap_or_default();
                // Say(1060139, name)
                muo_server::mobile::part2::messages::say_number(w, id, 1060139, &name); // You have made my work easy for me, ~1_NAME~.  My task here is done.
                set_paladin(w, id, |d| d.to_delete = true);
                muo_server::timer::start(w, TimeSpan::from_secs(5), TimeSpan::ZERO, 1, move |w| {
                    muo_server::mobile::base::delete(w, id);
                });
            } else if muo_content::mx::map(w, nec) != muo_content::mx::map(w, id)
                || distance_sqrt(w, id, nec)
                    > f64::from(muo_content::creature::read(w, id, 16, |c| c.range_perception) + 1)
            {
                let (loc, map) = (
                    muo_content::mx::location(w, id),
                    muo_content::mx::map(w, id),
                );
                puff2(w, loc, map, 2023);
                let (nloc, nmap) = (
                    muo_content::mx::location(w, nec),
                    muo_content::mx::map(w, nec),
                );
                puff2(w, nloc, nmap, 5023);
                muo_server::mobile::base::move_to_world(
                    w,
                    id,
                    nloc,
                    nmap.unwrap_or(MapId::INTERNAL),
                );
                muo_content::mx::play_sound(w, id, 0x1FE);
                muo_server::mobile::part2::messages::say_number(w, id, 1060140, ""); // You cannot escape me, knave of evil!
            }
        }
        (BASE_CREATURE_C.on_think)(w, id);
    },
    ..BASE_CREATURE_C
};

fn puff2(w: &mut World, loc: Point3D, map: Option<MapId>, effect: i32) {
    let e = muo_server::item::effect_item::create_targeted(
        w,
        loc,
        map,
        muo_server::item::effect_item::DEFAULT_DURATION,
    );
    muo_server::effects::send_location_particles(w, e, 0x3728, 10, 10, effect);
}

fn distance_sqrt(w: &World, a: MobileId, b: MobileId) -> f64 {
    let (la, lb) = (
        muo_content::mx::location(w, a),
        muo_content::mx::location(w, b),
    );
    let (dx, dy) = (f64::from(la.x - lb.x), f64::from(la.y - lb.y));
    (dx * dx + dy * dy).sqrt()
}

/// `Server.Engines.Quests.Necro.SummonedPaladin`: `[SerializationGenerator(0, false)]`; the constructor takes the
/// necromancer, so `[add` cannot make one.
pub static SUMMONED_PALADIN: MobileType = MobileType {
    full_name: "Server.Engines.Quests.Necro.SummonedPaladin",
    parent: Some(&BASE_CREATURE),
    construct: None,
    family: Some(&SUMMONED_PALADIN_C),
    click_title: |_, _| false,
    is_harmful_criminal: |w, id, target| {
        paladin(w, id).necromancer != Some(target)
            && (BASE_CREATURE.is_harmful_criminal)(w, id, target)
    },
    on_death: |w, id, c| {
        (BASE_CREATURE.on_death)(w, id, c);
        if let Some(nec) = paladin(w, id).necromancer
            && let Some(qs) = quest(w, nec)
            && !qs.has_objective(&FIND_MARDOTH_END_OBJECTIVE)
        {
            with_quest(w, nec, |w, q| q.add_objective(w, find_mardoth_end(true)));
        }
    },
    serialize: |w, id, out| {
        (BASE_CREATURE.serialize)(w, id, out);
        out.write_i32(0);
        let d = paladin(w, id);
        out.write_mobile(w, d.necromancer);
        out.write_bool(d.to_delete);
    },
    deserialize: |w, id, r| {
        (BASE_CREATURE.deserialize)(w, id, r)?;
        r.read_i32()?;
        let necromancer = r.read_mobile(w)?;
        let to_delete = r.read_bool()?;
        if let Some(m) = w.mobile_mut(id) {
            m.ext.insert(PaladinData {
                necromancer,
                to_delete,
            });
        }
        if to_delete {
            // Timer.DelayCall(Delete)
            muo_server::timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 1, move |w| {
                muo_server::mobile::base::delete(w, id)
            });
        }
        Ok(())
    },
    ..BASE_CREATURE
};

/// `SummonedPaladin.BeginSummon(player)`: the portal, the paladin stepping out of it, his warning, the fight.
pub fn begin_summon(w: &mut World, player: MobileId) {
    muo_server::timer::start(w, TimeSpan::from_secs(4), TimeSpan::ZERO, 1, move |w| {
        summon_step(w, player, None, 0)
    });
}

/// One `SummonTimer.OnTick`.
fn summon_step(w: &mut World, player: MobileId, paladin_id: Option<MobileId>, step: i32) {
    if muo_content::mx::deleted(w, player) {
        if step > 0
            && let Some(p) = paladin_id
        {
            muo_server::mobile::base::delete(w, p);
        }
        return;
    }
    if step > 0 && paladin_id.is_none_or(|p| muo_content::mx::deleted(w, p)) {
        return;
    }
    let again = |w: &mut World, p: Option<MobileId>, secs: u64| {
        muo_server::timer::start(
            w,
            TimeSpan::from_secs(secs as i64),
            TimeSpan::ZERO,
            1,
            move |w| summon_step(w, player, p, step + 1),
        );
    };
    match step {
        0 => {
            let loc = Point3D::new(2091, 1348, -90);
            let gate = muo_content::factory::make_item(
                w,
                "Server.Engines.Quests.Necro.SummonedPaladinMoongate",
            );
            if let Some(g) = gate {
                ops::move_to_world_map(w, g, loc, Some(MapId::MALAS));
            }
            muo_server::effects::play_sound(w, loc, Some(MapId::MALAS), 0x20E);
            let p = new_summoned_paladin(w, player);
            muo_content::mx::set_frozen(w, p, true);
            muo_server::mobile::base::move_to_world(w, p, loc, MapId::MALAS);
            again(w, Some(p), 2);
        }
        1 => {
            let Some(p) = paladin_id else { return };
            let d = muo_content::mx::get_direction_to(w, p, player);
            muo_content::mx::set_direction(w, p, d);
            muo_server::mobile::part2::messages::say_number(w, p, 1060122, ""); // STOP WICKED ONE!
            again(w, Some(p), 3);
        }
        _ => {
            let Some(p) = paladin_id else { return };
            muo_content::mx::set_frozen(w, p, false);
            // I will slay you before I allow you to complete your evil rites!
            muo_server::mobile::part2::messages::say_number(w, p, 1060123, "");
            muo_content::mx::set_combatant(w, p, Some(player));
        }
    }
}

quest_item! {
    @make
    /// `SummonedPaladinMoongate`.
    SUMMONED_PALADIN_MOONGATE = "Server.Engines.Quests.Necro.SummonedPaladinMoongate" : ITEM_BASE, |w| {
        let id = ops::new_item_with_id(w, &SUMMONED_PALADIN_MOONGATE, 0xF6C);
        ops::set_movable(w, id, false);
        muo_server::item::base::set_hue(w, id, 0x482);
        ops::set_light(w, id, muo_server::light_type::LightType::Circle300);
        muo_server::timer::start(w, TimeSpan::from_secs(10), TimeSpan::ZERO, 1, move |w| {
            if w.item(id).is_some_and(|i| !i.deleted()) {
                muo_server::item::base::delete(w, id);
            }
        });
        id
    };
}

/// The mobile classes of this folder.
pub static MOBILES: &[&MobileType] = &[&HORUS, &MAABUS, &MARDOTH, &SUMMONED_PALADIN];

/// The item classes of this folder.
pub static ITEMS: &[&ItemType] = &[
    &CRYSTAL_CAVE_BARRIER,
    &DARK_TIDES_HORN,
    &DARK_TIDES_TELEPORTER,
    &KRONUS_SCROLL,
    &KRONUS_SCROLL_BOX,
    &MAABUS_COFFIN,
    &MAABUS_COFFIN_COMPONENT,
    &SCROLL_OF_ABRAXUS,
    &SUMMONED_PALADIN_MOONGATE,
    &VAULT_OF_SECRETS_BARRIER,
];

/// The `MaabusCoffinComponent` branch of `AnimateDeadSpell.Target(item)` (`SpellHooks::animate_maabus_coffin`): true
/// if the item is a component of a `MaabusCoffin` (the spell then ends), after waking Maabus for a Dark Tides player
/// whose `AnimateMaabusCorpseObjective` is pending.
pub fn animate_maabus_coffin(w: &mut World, item: ItemId, caster: MobileId) -> bool {
    if !w
        .item(item)
        .is_some_and(|i| i.ty.is(&MAABUS_COFFIN_COMPONENT))
    {
        return false;
    }
    let Some(addon) = addon::component_addon(w, item)
        .filter(|&a| w.item(a).is_some_and(|i| i.ty.is(&MAABUS_COFFIN)))
    else {
        return false;
    };
    super::core::with_quest_of(w, caster, &QUEST, |w, q| {
        if let Some(i) = q.find_objective(&ANIMATE_MAABUS_CORPSE_OBJECTIVE)
            && !q.objectives[i].completed(q)
        {
            coffin_awake(w, addon, caster);
            q.complete_objective(w, i);
        }
    });
    true
}
