//! The Uzeraan Turmoil quest's items and mobiles (namespace `Server.Engines.Quests.Haven`).
//!

use muo_content::creature::setters::set_skill_range;
use muo_content::creature::{BASE_CREATURE, BASE_CREATURE_C, CreatureType};
use muo_content::items::addon::{self, ADDON_COMPONENT, AddonClass, BASE_ADDON, BASE_ADDON_CLASS};
use muo_content::items::cont::METAL_CHEST;
use muo_content::items::corpse::{self, CORPSE};
use muo_primitives::{Direction, Point3D};
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::{ItemType, ops};
use muo_server::mobile::MobileType;
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::skills::SkillName;

use super::core::{Conversation, Quest, can_offer_quest, focus_to, has_quest, load, with_quest};
use super::hooks::quest_hooks;
use super::items::{
    DYNAMIC_TELEPORTER, HORN_OF_RETREAT, HornData, QUEST_ITEM, QuestItemClass, TELEPORTER_CLASS,
    TeleporterClass, new_horn, new_teleporter,
};
use super::quester::{
    BASE_QUESTER, QUESTER_C, QUESTER_OPS, QUESTER_VC, QuesterOps, get_new_container,
};
use super::util::{
    add_item_hued, add_item_named, amount_in, drop_amount, drop_new, say_to_number, set_body,
};
use super::uzeraan::*;
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

fn turn_to(w: &mut World, npc: MobileId, to: MobileId) {
    let d = muo_content::mx::get_direction_to(w, npc, to);
    muo_content::mx::set_direction(w, npc, d);
}

fn init_body(w: &mut World, id: MobileId, hue: i32, female: bool) {
    muo_server::mobile::part2::stats::init_stats(w, id, 100, 100, 25);
    muo_server::mobile::base::set_hue(w, id, hue);
    muo_server::mobile::part1::props_b::set_female(w, id, female);
    set_body(w, id, if female { 0x191 } else { 0x190 });
}

fn hair(w: &mut World, id: MobileId, item: i32, hue: i32) {
    muo_server::mobile::part1::props_b::set_hair_item_id(w, id, item);
    muo_server::mobile::part1::props_b::set_hair_hue(w, id, hue);
}

fn ignite(w: &mut World, light: ItemId) {
    muo_items::lights::base::ignite(w, light);
}

fn place_or_message(w: &mut World, player: MobileId, item: ItemId) -> bool {
    if muo_server::mobile::part2::items::place_in_backpack(w, player, item) {
        true
    } else {
        muo_server::item::base::delete(w, item);
        muo_content::mx::msg(w, player, 1046260);
        false
    }
}

// ---- the quest items ----

macro_rules! uz_item {
    ($(#[$m:meta])* $name:ident, $class:ident = $full:literal, graphic $g:literal) => {
        static $class: QuestItemClass = QuestItemClass {
            can_drop: |w, _, player| !super::core::quest_is(w, player, &QUEST),
            accepted: |w, id| w.item(id).is_none_or(|i| i.deleted()),
        };
        quest_item! {
            $(#[$m])*
            $name = $full : QUEST_ITEM, |w| ops::new_item_with_id(w, &$name, $g);
            family: Some(&$class),
            default_weight: |_, _| 1.0,
        }
    };
}

uz_item!(/// `QuestDaemonBlood`.
    QUEST_DAEMON_BLOOD, DAEMON_BLOOD_CLASS = "Server.Engines.Quests.Haven.QuestDaemonBlood", graphic 0xF7D);
uz_item!(/// `QuestDaemonBone`.
    QUEST_DAEMON_BONE, DAEMON_BONE_CLASS = "Server.Engines.Quests.Haven.QuestDaemonBone", graphic 0xF80);
uz_item!(/// `QuestFertileDirt`.
    QUEST_FERTILE_DIRT, FERTILE_DIRT_CLASS = "Server.Engines.Quests.Haven.QuestFertileDirt", graphic 0xF81);

static SCROLL_CLASS: QuestItemClass = QuestItemClass {
    can_drop: |w, _, player| {
        !load(w, player).is_some_and(|q| {
            q.is(&QUEST) && q.is_objective_in_progress(&RETURN_SCROLL_OF_POWER_OBJECTIVE)
        })
    },
    accepted: |w, id| w.item(id).is_none_or(|i| i.deleted()),
};

quest_item! {
    @make
    /// `SchmendrickScrollOfPower`.
    SCHMENDRICK_SCROLL_OF_POWER = "Server.Engines.Quests.Haven.SchmendrickScrollOfPower" : QUEST_ITEM, |w| {
        let id = ops::new_item_with_id(w, &SCHMENDRICK_SCROLL_OF_POWER, 0xE34);
        muo_server::item::base::set_hue(w, id, 0x34D);
        id
    };
    family: Some(&SCROLL_CLASS),
    default_weight: |_, _| 1.0,
    label_number: |_, _| 1049118, // a scroll with ancient markings
}

// ---- UzeraanTurmoilHorn, UzeraanTurmoilTeleporter, DaemonBloodChest ----

static HORN_CLASS: super::items::HornClass = super::items::HornClass {
    validate_use: |w, _, from| is_player(w, from) && super::core::quest_is(w, from, &QUEST),
};

quest_item! {
    /// `UzeraanTurmoilHorn`.
    UZERAAN_TURMOIL_HORN = "Server.Engines.Quests.Haven.UzeraanTurmoilHorn" : HORN_OF_RETREAT, |w| {
        let id = new_horn(w, &UZERAAN_TURMOIL_HORN);
        if let Some(h) = w.item_mut(id).and_then(|i| i.ext.get_mut::<HornData>()) {
            h.dest_loc = Point3D::new(3597, 2582, 0);
            h.dest_map = Some(muo_server::map::MapId::TRAMMEL);
        }
        id
    };
    family: Some(&HORN_CLASS),
}

static UZERAAN_TELEPORTER_CLASS: TeleporterClass = TeleporterClass {
    get_destination: |w, _, player| {
        let qs = quest(w, player)?;
        let profession = qs.profession;
        let to = |x, y, z| Some((Point3D::new(x, y, z), muo_server::map::MapId::TRAMMEL));
        if qs.is_objective_in_progress(&FIND_SCHMENDRICK_OBJECTIVE)
            || qs.is_objective_in_progress(&FIND_APPRENTICE_OBJECTIVE)
            || has_lost_scroll_of_power(w, player)
        {
            return to(5222, 1858, 0);
        }
        if qs.is_objective_in_progress(&FIND_DRYAD_OBJECTIVE) || has_lost_fertile_dirt(w, player) {
            return to(3557, 2690, 2);
        }
        if profession != 5
            && (qs.is_objective_in_progress(&GET_DAEMON_BONE_OBJECTIVE)
                || has_lost_daemon_bone(w, player))
        {
            return to(3422, 2653, 48);
        }
        if qs.is_objective_in_progress(&CASH_BANK_CHECK_OBJECTIVE) {
            return to(3624, 2610, 0);
        }
        None
    },
    ..TELEPORTER_CLASS
};

quest_item! {
    /// `UzeraanTurmoilTeleporter` (`DynamicTeleporter()`: graphic 0x1822, hue 0x482).
    UZERAAN_TURMOIL_TELEPORTER = "Server.Engines.Quests.Haven.UzeraanTurmoilTeleporter" : DYNAMIC_TELEPORTER, |w| {
        new_teleporter(w, &UZERAAN_TURMOIL_TELEPORTER, 0x1822, 0x482)
    };
    family: Some(&UZERAAN_TELEPORTER_CLASS),
}

quest_item! {
    /// `DaemonBloodChest`.
    DAEMON_BLOOD_CHEST = "Server.Engines.Quests.Haven.DaemonBloodChest" : METAL_CHEST, |w| {
        let id = muo_content::items::cont::new_lockable(w, &DAEMON_BLOOD_CHEST, 0x9AB);
        ops::set_movable(w, id, false);
        id
    };
    on_double_click: |w, id, from| {
        let loc = ops::get_world_location(w, id);
        let Some(qs) = quest(w, from).filter(|_| is_player(w, from) && muo_content::mx::in_range_point(w, from, loc, 2)) else {
            (METAL_CHEST.on_double_click)(w, id, from);
            return;
        };
        let obj = qs.find_objective(&GET_DAEMON_BLOOD_OBJECTIVE);
        let open = obj.is_some_and(|i| !qs.objectives[i].completed(&qs));
        if !open && !has_lost_daemon_blood(w, from) {
            (METAL_CHEST.on_double_click)(w, id, from);
            return;
        }
        match muo_content::factory::make_item(w, "Server.Engines.Quests.Haven.QuestDaemonBlood") {
            Some(v) if muo_server::mobile::part2::items::place_in_backpack(w, from, v) => {
                super::util::msg_hue(w, from, 1049331, 0x22);
                if let Some(i) = obj.filter(|_| open) {
                    with_quest(w, from, |w, q| q.complete_objective(w, i));
                }
            }
            v => {
                super::util::msg_hue(w, from, 1049338, 0x22);
                if let Some(v) = v {
                    muo_server::item::base::delete(w, v);
                }
            }
        }
    },
}

// ---- Cannon ----

/// `CannonDirection`.
const NORTH: i32 = 0;
const EAST: i32 = 1;
const SOUTH: i32 = 2;

/// `Cannon`'s `_cannonDirection` and `_canoneer`.
#[derive(Clone, Default)]
pub struct CannonData {
    pub direction: i32,
    pub canoneer: Option<MobileId>,
}

fn cannon(w: &World, id: ItemId) -> CannonData {
    w.item(id)
        .and_then(|i| i.ext.get::<CannonData>())
        .cloned()
        .unwrap_or_default()
}

fn canoneer_active(w: &World, c: Option<MobileId>) -> bool {
    c.is_some_and(|m| {
        !muo_content::mx::deleted(w, m)
            && w.mobile(m)
                .and_then(|x| x.ext.get::<bool>())
                .copied()
                .unwrap_or(false)
    })
}

/// `new Cannon(direction)`.
pub fn new_cannon(w: &mut World, direction: i32) -> ItemId {
    let id = addon::new_addon(w, &CANNON);
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(CannonData {
            direction,
            canoneer: None,
        });
    }
    let rows: [(i32, i32, i32, i32); 3] = match direction {
        NORTH => [(0xE8D, 0, 0, 0), (0xE8C, 0, 1, 0), (0xE8B, 0, 2, 0)],
        EAST => [(0xE96, 0, 0, 0), (0xE95, -1, 0, 0), (0xE94, -2, 0, 0)],
        SOUTH => [(0xE91, 0, 0, 0), (0xE92, 0, -1, 0), (0xE93, 0, -2, 0)],
        _ => [(0xE8E, 0, 0, 0), (0xE8F, 1, 0, 0), (0xE90, 2, 0, 0)],
    };
    for (g, x, y, z) in rows {
        let c = addon::new_component(w, &CANNON_COMPONENT, g);
        addon::add_component(w, id, c, x, y, z);
    }
    id
}

/// `Cannon.DoFireEffect(target)`.
fn do_fire_effect(w: &mut World, id: ItemId, target: Point3D) {
    let (loc, map) = w
        .item(id)
        .map_or((Point3D::ZERO, None), |i| (i.location, i.map));
    let from = match cannon(w, id).direction {
        NORTH => Point3D::new(loc.x, loc.y - 1, loc.z),
        EAST => Point3D::new(loc.x + 1, loc.y, loc.z),
        SOUTH => Point3D::new(loc.x, loc.y + 1, loc.z),
        _ => Point3D::new(loc.x - 1, loc.y, loc.z),
    };
    muo_server::effects::send_location_effect(w, from, map, 0x36B0, 16, 1, 0, 0);
    muo_server::effects::play_sound(w, from, map, 0x11D);
    muo_server::effects::send_location_effect(w, target, map, 0x36B0, 16, 1, 0, 0);
    muo_server::effects::play_sound(w, target, map, 0x11D);
}

/// `Server.Engines.Quests.Haven.CannonComponent` (`[Constructible] CannonComponent(int itemID)`: no parameterless
/// constructor).
pub static CANNON_COMPONENT: ItemType = ItemType {
    full_name: "Server.Engines.Quests.Haven.CannonComponent",
    parent: Some(&ADDON_COMPONENT),
    construct: None,
    serialize: |w, id, out| {
        (ADDON_COMPONENT.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (ADDON_COMPONENT.deserialize)(w, id, r)?;
        r.read_i32()?;
        Ok(())
    },
    ..ADDON_COMPONENT
};

static CANNON_CLASS: AddonClass = AddonClass { ..BASE_ADDON_CLASS };

/// `Server.Engines.Quests.Haven.Cannon`: `[SerializationGenerator(0, false)]`.
pub static CANNON: ItemType = ItemType {
    full_name: "Server.Engines.Quests.Haven.Cannon",
    parent: Some(&BASE_ADDON),
    // `Cannon(CannonDirection direction)` is the only constructor: no parameterless one.
    construct: None,
    make: Some(|w| new_cannon(w, NORTH)),
    family: Some(&CANNON_CLASS),
    handles_on_movement: |w, id| {
        let c = cannon(w, id).canoneer;
        canoneer_active(w, c)
    },
    on_movement: |w, id, m, old| {
        let c = cannon(w, id);
        if !canoneer_active(w, c.canoneer) {
            return;
        }
        let Some(canoneer) = c.canoneer else { return };
        let (loc, _) = w
            .item(id)
            .map_or((Point3D::ZERO, None), |i| (i.location, i.map));
        let mloc = muo_content::mx::location(w, m);
        let can_fire = match c.direction {
            NORTH => {
                mloc.x >= loc.x - 7
                    && mloc.x <= loc.x + 7
                    && mloc.y == loc.y - 7
                    && old.y < loc.y - 7
            }
            EAST => {
                mloc.y >= loc.y - 7
                    && mloc.y <= loc.y + 7
                    && mloc.x == loc.x + 7
                    && old.x > loc.x + 7
            }
            SOUTH => {
                mloc.x >= loc.x - 7
                    && mloc.x <= loc.x + 7
                    && mloc.y == loc.y + 7
                    && old.y > loc.y + 7
            }
            _ => {
                mloc.y >= loc.y - 7
                    && mloc.y <= loc.y + 7
                    && mloc.x == loc.x - 7
                    && old.x < loc.x - 7
            }
        };
        if can_fire && will_fire(w, canoneer, m) {
            // Fire(Canoneer, m)
            do_fire_effect(w, id, mloc);
            muo_content::mx::damage(w, m, 9999, Some(canoneer));
        }
    },
    serialize: |w, id, out| {
        (BASE_ADDON.serialize)(w, id, out);
        out.write_i32(0);
        let c = cannon(w, id);
        out.write_enum(4, i64::from(c.direction));
        out.write_mobile(w, c.canoneer);
    },
    deserialize: |w, id, r| {
        (BASE_ADDON.deserialize)(w, id, r)?;
        r.read_i32()?;
        let direction = r.read_enum(4)? as i32;
        let canoneer = r.read_mobile(w)?;
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(CannonData {
                direction,
                canoneer,
            });
        }
        Ok(())
    },
    ..BASE_ADDON
};

// ---- the corpses ----

fn corpse_label(w: &mut World, id: ItemId, from: MobileId, named: i32, anon: i32) {
    let noto = corpse::corpse_notoriety(w, from, id);
    let hue = muo_server::notoriety::get_hue(w, noto);
    let Some(ns) = muo_server::mobile::part1::props_b::net_state(w, from) else {
        return;
    };
    let (serial, graphic, name) = w
        .item(id)
        .map_or((muo_primitives::Serial::MINUS_ONE, 0, String::new()), |i| {
            (i.serial, i.item_id, i.name.clone().unwrap_or_default())
        });
    if graphic == 0x2006 {
        muo_server::packets::message::send_message_localized(
            w,
            ns,
            serial,
            graphic,
            muo_server::packets::message::MessageType::LABEL,
            hue,
            3,
            named,
            "",
            &name,
        );
    } else {
        muo_server::packets::message::send_message_localized(
            w,
            ns,
            serial,
            graphic,
            muo_server::packets::message::MessageType::LABEL,
            hue,
            3,
            anon,
            "",
            "",
        );
    }
}

fn corpse_name_property(
    w: &World,
    id: ItemId,
    list: &mut muo_server::property_list::PropertyList,
    named: i32,
    anon: i32,
) {
    if w.item(id).is_some_and(|i| i.item_id == 0x2006) {
        list.add_text("a human corpse");
        list.add_str(
            named,
            &w.item(id).and_then(|i| i.name.clone()).unwrap_or_default(),
        );
    } else {
        list.add(anon);
    }
}

/// `new MilitiaFighterCorpse(owner, hair, hairHue, facialHair, facialHairHue, equipItems)`.
pub fn new_militia_fighter_corpse(
    w: &mut World,
    owner: MobileId,
    hair: (i32, i32),
    facial: (i32, i32),
    equip: Vec<ItemId>,
) -> ItemId {
    corpse::new_corpse_of(w, &MILITIA_FIGHTER_CORPSE, owner, hair, facial, equip)
}

/// `Server.Engines.Quests.Haven.MilitiaFighterCorpse`.
// `Corpse.Mobile_CreateCorpseHandler` makes it for a dead `MilitiaFighter` through `CorpseClassHooks::quest_corpse`
// (set in `quests::external::install`).
pub static MILITIA_FIGHTER_CORPSE: ItemType = ItemType {
    full_name: "Server.Engines.Quests.Haven.MilitiaFighterCorpse",
    parent: Some(&CORPSE),
    construct: None,
    add_name_property: |w, id, list| corpse_name_property(w, id, list, 1049318, 1049319),
    on_single_click: |w, id, from| corpse_label(w, id, from, 1049318, 1049319),
    on_double_click: |w, id, from| {
        if muo_content::mx::in_range_point(w, from, ops::get_world_location(w, id), 2) {
            super::util::msg_hue(w, from, 1049661, 0x22);
        }
    },
    serialize: |w, id, out| {
        (CORPSE.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (CORPSE.deserialize)(w, id, r)?;
        r.read_i32()?;
        Ok(())
    },
    ..CORPSE
};

/// `SchmendrickApprenticeCorpse._lantern`.
#[derive(Clone, Default)]
struct ApprenticeLantern(Option<ItemId>);

fn new_schmendrick_apprentice_corpse(w: &mut World) -> ItemId {
    // GetOwner(): a deleted Mobile with a human skin hue, male, body 0x190 and a random name
    let owner = w.new_mobile(&muo_server::mobile::MOBILE_BASE);
    let hue = muo_content::vendor::human_random_skin_hue(w);
    muo_server::mobile::base::set_hue(w, owner, hue);
    muo_server::mobile::part1::props_b::set_female(w, owner, false);
    set_body(w, owner, 0x190);
    let name = muo_content::misc::small::random_name(w, "male");
    if let Some(f) = w.mobile(owner).map(|m| m.ty.set_name) {
        f(w, owner, Some(name));
    }
    // Race.Human.RandomHair(false), RandomHairHue(), RandomFacialHair(false), RandomHairHue()
    let hair_item = super::util::human_random_hair(w, false);
    let hair_hue = super::util::human_random_hair_hue(w);
    let facial_item = super::util::human_random_facial_hair(w, false);
    let facial_hue = super::util::human_random_hair_hue(w);
    // GetEquipment()
    let mut equip = Vec::new();
    let h = super::core::random_bright_hue(w);
    equip.extend(add_hued_loose(w, "Server.Items.Robe", h));
    let h = muo_primitives::utility::random_neutral_hue(&mut w.rng);
    equip.extend(add_hued_loose(w, "Server.Items.WizardsHat", h));
    let h = muo_primitives::utility::random_neutral_hue(&mut w.rng);
    equip.extend(add_hued_loose(w, "Server.Items.Shoes", h));
    equip.extend(muo_content::factory::make_item(w, "Server.Items.Spellbook"));

    let id = corpse::new_corpse_of(
        w,
        &SCHMENDRICK_APPRENTICE_CORPSE,
        owner,
        (hair_item, hair_hue),
        (facial_item, facial_hue),
        equip.clone(),
    );
    muo_server::mobile::base::delete(w, owner);
    ops::set_direction(w, id, Direction::WEST);
    for item in equip {
        muo_server::item::container::drop_item_default(w, id, item);
    }
    let lantern = muo_content::factory::make_item(w, "Server.Items.Lantern");
    if let Some(l) = lantern {
        ops::set_movable(w, l, false);
        muo_items::lights::base::set_protected(w, l, true);
        ignite(w, l);
    }
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(ApprenticeLantern(lantern));
    }
    if let Some(d) = corpse::data_mut(w, id) {
        d.owner = None;
    }
    id
}

fn add_hued_loose(w: &mut World, name: &str, hue: i32) -> Option<ItemId> {
    let it = muo_content::factory::make_item(w, name)?;
    super::quester::set_hue(w, it, hue);
    Some(it)
}

fn lantern_of(w: &World, id: ItemId) -> Option<ItemId> {
    w.item(id)?.ext.get::<ApprenticeLantern>()?.0
}

/// `Server.Engines.Quests.Haven.SchmendrickApprenticeCorpse`.
pub static SCHMENDRICK_APPRENTICE_CORPSE: ItemType = ItemType {
    full_name: "Server.Engines.Quests.Haven.SchmendrickApprenticeCorpse",
    parent: Some(&CORPSE),
    construct: Some(new_schmendrick_apprentice_corpse),
    add_name_property: |w, id, list| corpse_name_property(w, id, list, 1049144, 1049145),
    on_single_click: |w, id, from| corpse_label(w, id, from, 1049144, 1049145),
    on_double_click: |w, id, from| {
        // Open(from, checkSelfLoot)
        if !muo_content::mx::in_range_point(w, from, ops::get_world_location(w, id), 2)
            || !is_player(w, from)
        {
            return;
        }
        let obj = quest(w, from).and_then(|q| pending(&q, &FIND_APPRENTICE_OBJECTIVE));
        let Some(obj) = obj else {
            super::util::msg_hue(w, from, 1049143, 0x22);
            return;
        };
        match muo_content::factory::make_item(
            w,
            "Server.Engines.Quests.Haven.SchmendrickScrollOfPower",
        ) {
            Some(s) if muo_server::mobile::part2::items::place_in_backpack(w, from, s) => {
                super::util::msg_hue(w, from, 1049147, 0x22); // You find the scroll and put it in your pack.
                with_quest(w, from, |w, q| q.complete_objective(w, obj));
            }
            s => {
                super::util::msg_hue(w, from, 1049146, 0x22);
                if let Some(s) = s {
                    muo_server::item::base::delete(w, s);
                }
            }
        }
    },
    on_location_change: |w, id, _old| {
        if let Some(l) = lantern_of(w, id).filter(|l| w.item(*l).is_some_and(|i| !i.deleted())) {
            let loc = w.item(id).map_or(Point3D::ZERO, |i| i.location);
            ops::set_location(w, l, Point3D::new(loc.x, loc.y + 1, loc.z));
        }
    },
    on_map_change: |w, id| {
        if let Some(l) = lantern_of(w, id).filter(|l| w.item(*l).is_some_and(|i| !i.deleted())) {
            let map = w.item(id).and_then(|i| i.map);
            ops::set_map(w, l, map);
        }
    },
    on_after_delete: |w, id| {
        (CORPSE.on_after_delete)(w, id);
        if let Some(l) = lantern_of(w, id).filter(|l| w.item(*l).is_some_and(|i| !i.deleted())) {
            muo_server::item::base::delete(w, l);
        }
        if let Some(o) = corpse::owner(w, id) {
            muo_server::mobile::base::delete(w, o);
        }
    },
    serialize: |w, id, out| {
        (CORPSE.serialize)(w, id, out);
        out.write_i32(0);
        out.write_item(w, lantern_of(w, id));
    },
    deserialize: |w, id, r| {
        (CORPSE.deserialize)(w, id, r)?;
        r.read_i32()?;
        let lantern = r.read_item(w)?;
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(ApprenticeLantern(lantern));
        }
        Ok(())
    },
    ..CORPSE
};

// ---- Uzeraan ----

static UZERAAN_OPS: QuesterOps = QuesterOps {
    auto_talk_range: |_, _, _| 3,
    can_talk_to: |w, _, to| super::core::quest_is(w, to, &QUEST),
    on_talk: uzeraan_on_talk,
    ..QUESTER_OPS
};

fn uzeraan_on_talk(w: &mut World, uzeraan: MobileId, player: MobileId, _context_menu: bool) {
    let Some(qs) = quest(w, player) else { return };
    let add = |w: &mut World, c: Conversation| {
        with_quest(w, player, |w, q| q.add_conversation(w, c));
    };
    if has_lost_scroll_of_power(w, player) {
        add(w, lost_scroll_of_power(true));
        return;
    }
    if has_lost_fertile_dirt(w, player) {
        add(w, lost_fertile_dirt(true));
        return;
    }
    if has_lost_daemon_blood(w, player) {
        add(w, Conversation::new(&LOST_DAEMON_BLOOD_CONVERSATION));
        return;
    }
    if has_lost_daemon_bone(w, player) {
        add(w, Conversation::new(&LOST_DAEMON_BONE_CONVERSATION));
        return;
    }
    let magician = qs.profession == 2;
    if magician {
        let pack = w.mobile(player).and_then(|m| m.backpack);
        let few = pack.is_none()
            || [
                "BlackPearl",
                "Bloodmoss",
                "Garlic",
                "Ginseng",
                "MandrakeRoot",
                "Nightshade",
                "SulfurousAsh",
                "SpidersSilk",
            ]
            .iter()
            .any(|r| amount_in(w, pack, &format!("Server.Items.{r}")) < 30);
        if few {
            add(w, Conversation::new(&FEW_REAGENTS_CONVERSATION));
        }
    }
    let p = |c| pending(&qs, c);
    if let Some(i) = p(&FIND_UZERAAN_BEGIN_OBJECTIVE) {
        with_quest(w, player, |w, q| q.complete_objective(w, i));
        return;
    }
    if let Some(i) = p(&FIND_UZERAAN_FIRST_TASK_OBJECTIVE) {
        with_quest(w, player, |w, q| q.complete_objective(w, i));
        return;
    }
    if let Some(i) = p(&FIND_UZERAAN_ABOUT_REPORT_OBJECTIVE) {
        let cont = get_new_container(w);
        if magician {
            drop_amount(w, cont, "Server.Items.MarkScroll", 5);
            drop_amount(w, cont, "Server.Items.RecallScroll", 5);
            for _ in 0..5 {
                drop_new(w, cont, "Server.Items.RecallRune");
            }
        } else {
            drop_amount(w, cont, "Server.Items.Gold", 300);
            for _ in 0..6 {
                drop_new(w, cont, "Server.Items.NightSightPotion");
                drop_new(w, cont, "Server.Items.LesserHealPotion");
            }
        }
        if place_or_message(w, player, cont) {
            with_quest(w, player, |w, q| q.complete_objective(w, i));
        }
        return;
    }
    for (c, msg) in [
        (&RETURN_SCROLL_OF_POWER_OBJECTIVE, 1049378), // Hand me the scroll, if you have it.
        (&RETURN_FERTILE_DIRT_OBJECTIVE, 1049381),    // Hand me the Fertile Dirt, if you have it.
        (&RETURN_DAEMON_BLOOD_OBJECTIVE, 1049379),    // Hand me the Vial of Blood, if you have it.
        (&RETURN_DAEMON_BONE_OBJECTIVE, 1049380),     // Hand me the Daemon Bone, if you have it.
    ] {
        if p(c).is_some() {
            focus_to(w, uzeraan, player);
            say_to_number(w, uzeraan, player, msg);
            return;
        }
    }
    say_to_number(w, uzeraan, player, 1049357); // I have nothing more for you at this time.
}

/// `Utility.Random(6) switch { Broadsword, Cutlass, Katana, Longsword, Scimitar, VikingSword }`.
fn random_sword(w: &mut World) -> Option<ItemId> {
    const SWORDS: [&str; 6] = [
        "Server.Items.Broadsword",
        "Server.Items.Cutlass",
        "Server.Items.Katana",
        "Server.Items.Longsword",
        "Server.Items.Scimitar",
        "Server.Items.VikingSword",
    ];
    let n = muo_content::hooks::rnd::below(w, 6) as usize;
    muo_content::factory::make_item(w, SWORDS[n])
}

fn uzeraan_on_drag_drop(w: &mut World, uzeraan: MobileId, from: MobileId, dropped: ItemId) -> bool {
    let fallback = |w: &mut World| (BASE_QUESTER.on_drag_drop)(w, uzeraan, from, dropped);
    if !is_player(w, from) {
        return fallback(w);
    }
    let Some(qs) = quest(w, from) else {
        return fallback(w);
    };
    let is = |w: &World, name: &str| super::util::item_is(w, dropped, name);
    let magician = qs.profession == 2;

    if is(w, "Server.Engines.Quests.Haven.UzeraanTurmoilHorn") {
        if muo_content::player_mobile::young(w, from) {
            let charges = super::items::horn(w, dropped).map_or(0, |h| h.charges);
            if charges < 10 {
                say_to_number(w, uzeraan, from, 1049384); // I have recharged the item for you.
                if let Some(h) = super::items::horn_mut(w, dropped) {
                    h.charges = 10;
                }
            } else {
                say_to_number(w, uzeraan, from, 1049385); // That doesn't need recharging yet.
            }
        } else {
            muo_content::mx::msg(w, from, 1114333); // You must be young to have this item recharged.
        }
        return false;
    }

    if is(w, "Server.Engines.Quests.Haven.SchmendrickScrollOfPower") {
        if let Some(obj) = pending(&qs, &RETURN_SCROLL_OF_POWER_OBJECTIVE) {
            let cont = get_new_container(w);
            let level = if muo_content::player_mobile::young(w, from) {
                0
            } else {
                1
            };
            let m = muo_items::maps::treasure::treasure_map_new(
                w,
                &muo_items::maps::treasure::TREASURE_MAP,
                level,
                Some(muo_server::map::MapId::TRAMMEL),
            );
            muo_server::item::container::drop_item_default(w, cont, m);
            drop_new(w, cont, "Server.Items.Shovel");
            drop_new(w, cont, "Server.Engines.Quests.Haven.UzeraanTurmoilHorn");
            if !place_or_message(w, from, cont) {
                return false;
            }
            muo_server::item::base::delete(w, dropped);
            with_quest(w, from, |w, q| q.complete_objective(w, obj));
            return true;
        }
    } else if is(w, "Server.Engines.Quests.Haven.QuestFertileDirt") {
        if let Some(obj) = pending(&qs, &RETURN_FERTILE_DIRT_OBJECTIVE) {
            let cont = get_new_container(w);
            if magician {
                for r in [
                    "BlackPearl",
                    "Bloodmoss",
                    "Garlic",
                    "Ginseng",
                    "MandrakeRoot",
                    "Nightshade",
                    "SulfurousAsh",
                    "SpidersSilk",
                ] {
                    drop_amount(w, cont, &format!("Server.Items.{r}"), 20);
                }
                for _ in 0..3 {
                    if let Some(s) = muo_content::loot::random_scroll(
                        w,
                        0,
                        23,
                        muo_content::loot::SpellbookType::Regular,
                    ) {
                        muo_server::item::container::drop_item_default(w, cont, s);
                    }
                }
            } else {
                drop_amount(w, cont, "Server.Items.Gold", 300);
                drop_amount(w, cont, "Server.Items.Bandage", 25);
                for _ in 0..5 {
                    drop_new(w, cont, "Server.Items.LesserHealPotion");
                }
            }
            if !place_or_message(w, from, cont) {
                return false;
            }
            muo_server::item::base::delete(w, dropped);
            with_quest(w, from, |w, q| q.complete_objective(w, obj));
            return true;
        }
    } else if is(w, "Server.Engines.Quests.Haven.QuestDaemonBlood") {
        if let Some(obj) = pending(&qs, &RETURN_DAEMON_BLOOD_OBJECTIVE) {
            let reward = if magician {
                let cont = get_new_container(w);
                drop_amount(w, cont, "Server.Items.ExplosionScroll", 4);
                drop_new(w, cont, "Server.Items.MagicWizardsHat");
                Some(cont)
            } else {
                let weapon = random_sword(w);
                if let Some(wp) = weapon {
                    if muo_server::mobile::part1::props_a::core_aos(w) {
                        if let Some(f) = quest_hooks(w).and_then(|h| h.apply_runic_attributes) {
                            f(w, wp, 3, 20, 40);
                        }
                    } else {
                        use muo_content::items::weapon::{
                            WeaponAccuracyLevel, WeaponDamageLevel, WeaponDurabilityLevel,
                            set_accuracy_level, set_damage_level, set_durability_level,
                        };
                        let scaled = |w: &mut World| {
                            muo_content::creature::packing::random_min_max_scaled(w, 2, 4)
                        };
                        let n = scaled(w);
                        set_damage_level(
                            w,
                            wp,
                            WeaponDamageLevel::from_i32(n).unwrap_or(WeaponDamageLevel::Regular),
                        );
                        let n = scaled(w);
                        set_accuracy_level(
                            w,
                            wp,
                            WeaponAccuracyLevel::from_i32(n)
                                .unwrap_or(WeaponAccuracyLevel::Regular),
                        );
                        let n = scaled(w);
                        set_durability_level(
                            w,
                            wp,
                            WeaponDurabilityLevel::from_i32(n)
                                .unwrap_or(WeaponDurabilityLevel::Regular),
                        );
                    }
                    if let Some(d) = muo_content::items::weapon::data_mut(w, wp) {
                        d.slayer = muo_content::items::slayer::SlayerName::Silver as i32; // weapon.Slayer = SlayerName.Silver
                    }
                }
                weapon
            };
            let Some(reward) = reward else { return false };
            if !place_or_message(w, from, reward) {
                return false;
            }
            muo_server::item::base::delete(w, dropped);
            with_quest(w, from, |w, q| q.complete_objective(w, obj));
            return true;
        }
    } else if is(w, "Server.Engines.Quests.Haven.QuestDaemonBone")
        && let Some(obj) = pending(&qs, &RETURN_DAEMON_BONE_OBJECTIVE)
    {
        let cont = get_new_container(w);
        let check = muo_content::items::gold::new_bank_check(w, 2000);
        muo_server::item::container::drop_item_default(w, cont, check);
        drop_new(w, cont, "Server.Items.EnchantedSextant");
        if !place_or_message(w, from, cont) {
            return false;
        }
        muo_server::item::base::delete(w, dropped);
        with_quest(w, from, |w, q| q.complete_objective(w, obj));
        return true;
    }
    fallback(w)
}

quester! {
    /// `Server.Engines.Quests.Haven.Uzeraan`.
    UZERAAN, UZERAAN_C, UZERAAN_V = "Server.Engines.Quests.Haven.Uzeraan" : BASE_QUESTER / QUESTER_C / QUESTER_VC,
        ops UZERAAN_OPS, encoded false, title Some("the Conjurer"), extra |_, _| {};
    v {
        init_body: |w, id| muo_content::templates::apply_mobile_as(w, id, "Server.Engines.Quests.Haven.Uzeraan"),
        init_outfit: |_, _| {},
    }
    c { default_name: muo_content::templates::creature_props::default_name, }
    m { on_drag_drop: uzeraan_on_drag_drop, }
}

// ---- Schmendrick ----

static SCHMENDRICK_OPS: QuesterOps = QuesterOps {
    auto_talk_range: |_, _, _| 7,
    can_talk_to: |w, _, to| {
        quest(w, to).is_some_and(|q| q.has_objective(&FIND_SCHMENDRICK_OBJECTIVE))
    },
    on_talk: |w, schmendrick, player, context_menu| {
        let Some(qs) = quest(w, player) else { return };
        if has_lost_scroll_of_power(w, player) {
            focus_to(w, schmendrick, player);
            with_quest(w, player, |w, q| {
                q.add_conversation(w, lost_scroll_of_power(false))
            });
            return;
        }
        if let Some(i) = pending(&qs, &FIND_SCHMENDRICK_OBJECTIVE) {
            focus_to(w, schmendrick, player);
            with_quest(w, player, |w, q| q.complete_objective(w, i));
        } else if context_menu {
            focus_to(w, schmendrick, player);
            say_to_number(w, schmendrick, player, 1049357); // I have nothing more for you at this time.
        }
    },
    ..QUESTER_OPS
};

quester! {
    /// `Server.Engines.Quests.Haven.Schmendrick`.
    SCHMENDRICK, SCHMENDRICK_C, SCHMENDRICK_V = "Server.Engines.Quests.Haven.Schmendrick"
        : BASE_QUESTER / QUESTER_C / QUESTER_VC,
        ops SCHMENDRICK_OPS, encoded false, title Some("the High Mage"), extra |_, _| {};
    v {
        init_body: |w, id| muo_content::templates::apply_mobile_as(w, id, "Server.Engines.Quests.Haven.Schmendrick"),
        init_outfit: |_, _| {},
    }
    c { default_name: muo_content::templates::creature_props::default_name, }
    m {
        on_movement: |w, npc, m, old| super::emino_mobs::resurrect_on_movement(w, npc, m, old, true),
        on_drag_drop: |w, schmendrick, from, dropped| {
            if !super::util::item_is(w, dropped, "Server.Items.BlankScroll") || !has_lost_scroll_of_power(w, from) {
                return (BASE_QUESTER.on_drag_drop)(w, schmendrick, from, dropped);
            }
            focus_to(w, schmendrick, from);
            let Some(scroll) = muo_content::factory::make_item(w, "Server.Engines.Quests.Haven.SchmendrickScrollOfPower") else {
                return false;
            };
            if !muo_server::mobile::part2::items::place_in_backpack(w, from, scroll) {
                muo_server::item::base::delete(w, scroll);
                muo_content::mx::msg(w, from, 1046260);
                return false;
            }
            muo_server::item::base::consume(w, dropped, 1);
            muo_content::mx::msg(w, from, 1049346);
            super::util::deleted_item(w, dropped)
        },
    }
}

// ---- Dryad ----

static DRYAD_OPS: QuesterOps = QuesterOps {
    auto_talk_range: |_, _, _| 4,
    can_talk_to: |w, _, to| quest(w, to).is_some_and(|q| q.has_objective(&FIND_DRYAD_OBJECTIVE)),
    on_talk: |w, dryad, player, context_menu| {
        let Some(qs) = quest(w, player) else { return };
        if has_lost_fertile_dirt(w, player) {
            focus_to(w, dryad, player);
            with_quest(w, player, |w, q| {
                q.add_conversation(w, lost_fertile_dirt(false))
            });
        } else if let Some(i) = pending(&qs, &FIND_DRYAD_OBJECTIVE) {
            focus_to(w, dryad, player);
            let dirt =
                muo_content::factory::make_item(w, "Server.Engines.Quests.Haven.QuestFertileDirt");
            if !dirt
                .is_some_and(|d| muo_server::mobile::part2::items::place_in_backpack(w, player, d))
            {
                if let Some(d) = dirt {
                    muo_server::item::base::delete(w, d);
                }
                muo_content::mx::msg(w, player, 1046260);
            } else {
                with_quest(w, player, |w, q| q.complete_objective(w, i));
            }
        } else if context_menu {
            focus_to(w, dryad, player);
            say_to_number(w, dryad, player, 1049357); // I have nothing more for you at this time.
        }
    },
    ..QUESTER_OPS
};

/// `SBDryad`: the buy list and the sell table.
static SB_DRYAD: muo_content::vendor::SbInfo = muo_content::vendor::SbInfo {
    name: "SBDryad",
    build: |_w, _v, sb| {
        use muo_content::vendor::BuyInfo;
        for (ty, price, item_id, sell) in [
            ("Server.Items.Bandage", 5, 0xE21, 2),
            ("Server.Items.Ginseng", 3, 0xF85, 2),
            ("Server.Items.Garlic", 3, 0xF84, 2),
            ("Server.Items.Bloodmoss", 5, 0xF7B, 3),
            ("Server.Items.Nightshade", 3, 0xF88, 2),
            ("Server.Items.SpidersSilk", 3, 0xF8D, 2),
            ("Server.Items.MandrakeRoot", 3, 0xF86, 2),
        ] {
            sb.buy.push(BuyInfo::new(None, ty, price, 20, item_id, 0));
            sb.sell.add(ty, sell);
        }
    },
};

quester! {
    /// `Server.Engines.Quests.Haven.Dryad`.
    DRYAD, DRYAD_C, DRYAD_V = "Server.Engines.Quests.Haven.Dryad" : BASE_QUESTER / QUESTER_C / QUESTER_VC,
        ops DRYAD_OPS, encoded false, title Some("the Dryad"), extra |w, id| {
            for skill in [
                SkillName::Peacemaking,
                SkillName::Cooking,
                SkillName::Provocation,
                SkillName::Musicianship,
                SkillName::Poisoning,
                SkillName::Archery,
                SkillName::Tailoring,
            ] {
                set_skill_range(w, id, skill, 80.0, 100.0);
            }
        };
    v {
        is_active_vendor: |_, _| true,
        init_sb_info: |w, id| vec![SB_DRYAD.new_for(w, id)],
        init_body: |w, id| init_body(w, id, 0x85A7, true),
        init_outfit: |w, id| {
            add_item_hued(w, id, "Server.Items.Kilt", 0x301);
            add_item_hued(w, id, "Server.Items.FancyShirt", 0x300);
            hair(w, id, 0x203D, 0x22); // Pony Tail
            if let Some(b) = add_item_named(w, id, "Server.Items.Bow") {
                ops::set_movable(w, b, false);
            }
        },
    }
    c {
        default_name: |_, _| Some("Anwin Brenna"),
        disallow_all_moves: |_, _| false,
        can_teach: |_, _| true,
    }
    m {
        click_title: |_, _| true,
        on_drag_drop: |w, dryad, from, dropped| {
            let fallback = |w: &mut World| (BASE_QUESTER.on_drag_drop)(w, dryad, from, dropped);
            if !is_player(w, from)
                || quest(w, from).is_none()
                || !super::util::item_is(w, dropped, "Server.Items.Apple")
                || !has_lost_fertile_dirt(w, from)
            {
                return fallback(w);
            }
            focus_to(w, dryad, from);
            let dirt = muo_content::factory::make_item(w, "Server.Engines.Quests.Haven.QuestFertileDirt");
            if !dirt.is_some_and(|d| muo_server::mobile::part2::items::place_in_backpack(w, from, d)) {
                if let Some(d) = dirt {
                    muo_server::item::base::delete(w, d);
                }
                muo_content::mx::msg(w, from, 1046260);
                return false;
            }
            muo_server::item::base::consume(w, dropped, 1);
            with_quest(w, from, |w, q| q.add_conversation(w, Conversation::new(&DRYAD_APPLE_CONVERSATION)));
            super::util::deleted_item(w, dropped)
        },
    }
}

// ---- MansionGuard ----

static MANSION_GUARD_OPS: QuesterOps = QuesterOps {
    auto_talk_range: |_, _, _| 3,
    can_talk_to: |w, q, to| {
        let _ = q;
        !has_quest(w, to) && !muo_content::player_mobile::data(w, to).is_none() && offerable(w, to)
    },
    on_talk: |w, guard, player, _| {
        if !has_quest(w, player) && can_offer_quest(w, player, &QUEST) {
            turn_to(w, guard, player);
            Quest::new(w, &QUEST, player).send_offer(w);
        }
    },
    ..QUESTER_OPS
};

/// `QuestSystem.CanOfferQuest(to, typeof(UzeraanTurmoilQuest))` without its side effect on the restart list: a
/// `CanTalkTo` check only reads.
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

quester! {
    /// `Server.Engines.Quests.Haven.MansionGuard`.
    MANSION_GUARD, MANSION_GUARD_C, MANSION_GUARD_V = "Server.Engines.Quests.Haven.MansionGuard"
        : BASE_QUESTER / QUESTER_C / QUESTER_VC,
        ops MANSION_GUARD_OPS, encoded false, title Some("the Mansion Guard"), extra |_, _| {};
    v {
        init_body: |w, id| muo_content::templates::apply_mobile_as(w, id, "Server.Engines.Quests.Haven.MansionGuard"),
        init_outfit: |_, _| {},
    }
    c { }
    m { }
}

// ---- MilitiaCanoneer ----

/// `IsEnemy(m)` of the militia: a non-player, non-vendor creature is an enemy when its master (or it) is evil.
fn militia_is_enemy(w: &mut World, _self: MobileId, m: MobileId) -> bool {
    let mut m = m;
    loop {
        let is_player = w.mobile(m).is_some_and(|x| x.player);
        if is_player || muo_content::vendor::is_vendor(w, m) {
            return false;
        }
        if muo_content::creature::is_creature(w, m)
            && let Some(master) = muo_content::creature::control::get_master(w, m)
        {
            m = master;
            continue;
        }
        return w.mobile(m).is_some_and(|x| x.karma < 0);
    }
}

/// `MilitiaCanoneer.WillFire(cannon, target)`.
fn will_fire(w: &mut World, canoneer: MobileId, target: MobileId) -> bool {
    let active = w
        .mobile(canoneer)
        .and_then(|m| m.ext.get::<bool>())
        .copied()
        .unwrap_or(false);
    if active && militia_is_enemy(w, canoneer, target) {
        turn_to(w, canoneer, target);
        const CLILOCS: [i32; 4] = [
            500651,  // You're evil, and must die!
            1049098, // I shall make short work of thee.
            1049320, // FIRE!
            1043149, // Thou deservest to die!
        ];
        let n = CLILOCS[muo_content::hooks::rnd::below(w, 4) as usize];
        super::util::say_number(w, canoneer, n);
        return true;
    }
    false
}

static CANONEER_OPS: QuesterOps = QuesterOps {
    can_talk_to: |_, _, _| false,
    save: |w, id, out| {
        out.write_bool(
            w.mobile(id)
                .and_then(|m| m.ext.get::<bool>())
                .copied()
                .unwrap_or(false),
        )
    },
    load: |w, id, r| {
        let active = r.read_bool()?;
        if let Some(m) = w.mobile_mut(id) {
            m.ext.insert(active);
        }
        Ok(())
    },
    ..QUESTER_OPS
};

quester! {
    /// `Server.Engines.Quests.Haven.MilitiaCanoneer`: its `Active` is saved.
    MILITIA_CANONEER, MILITIA_CANONEER_C, MILITIA_CANONEER_V = "Server.Engines.Quests.Haven.MilitiaCanoneer"
        : BASE_QUESTER / QUESTER_C / QUESTER_VC,
        ops CANONEER_OPS, encoded false, title Some("the Militia Canoneer"), extra |w, id| {
            if let Some(m) = w.mobile_mut(id) {
                m.ext.insert(true); // _active = true
            }
        };
    v {
        init_body: |w, id| muo_content::templates::apply_mobile_as(w, id, "Server.Engines.Quests.Haven.MilitiaCanoneer"),
        init_outfit: |_, _| {},
    }
    c { is_enemy: militia_is_enemy, }
    m { }
}

// ---- MilitiaFighter ----

static MILITIA_FIGHTER_C: CreatureType = CreatureType {
    is_enemy: militia_is_enemy,
    ..BASE_CREATURE_C
};

/// `Server.Engines.Quests.Haven.MilitiaFighter`.
pub static MILITIA_FIGHTER: MobileType = MobileType {
    full_name: "Server.Engines.Quests.Haven.MilitiaFighter",
    parent: Some(&BASE_CREATURE),
    construct: Some(|w| muo_content::templates::new_creature(w, &MILITIA_FIGHTER)),
    family: Some(&MILITIA_FIGHTER_C),
    click_title: |_, _| false,
    serialize: |w, id, out| {
        (BASE_CREATURE.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (BASE_CREATURE.deserialize)(w, id, r)?;
        r.read_i32()?;
        Ok(())
    },
    ..BASE_CREATURE
};

/// The mobile classes of this folder.
pub static MOBILES: &[&MobileType] = &[
    &DRYAD,
    &MANSION_GUARD,
    &MILITIA_CANONEER,
    &MILITIA_FIGHTER,
    &SCHMENDRICK,
    &UZERAAN,
];

/// The item classes of this folder.
pub static ITEMS: &[&ItemType] = &[
    &CANNON,
    &CANNON_COMPONENT,
    &DAEMON_BLOOD_CHEST,
    &MILITIA_FIGHTER_CORPSE,
    &QUEST_DAEMON_BLOOD,
    &QUEST_DAEMON_BONE,
    &QUEST_FERTILE_DIRT,
    &SCHMENDRICK_APPRENTICE_CORPSE,
    &SCHMENDRICK_SCROLL_OF_POWER,
    &UZERAAN_TURMOIL_HORN,
    &UZERAAN_TURMOIL_TELEPORTER,
];
