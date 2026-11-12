//! `ParagonChest` and `TreasureMapChest`.

use std::rc::Rc;

use indexmap::IndexSet;
use muo_content::factory::{item_is, make_item};
use muo_content::gumps::base::{RelayInfo, StaticGumpLogic, StaticGumpObj};
use muo_content::gumps::builder::StaticGumpBuilder;
use muo_content::gumps::send_gump_to_mobile;
use muo_content::items::armor::{ArmorDurabilityLevel, ArmorProtectionLevel};
use muo_content::items::cont::{
    self, ContClass, LOCKABLE_CLASS, LOCKABLE_CONTAINER, TrapType, new_lockable,
};
use muo_content::items::gold::new_gold;
use muo_content::items::weapon::{WeaponAccuracyLevel, WeaponDamageLevel, WeaponDurabilityLevel};
use muo_content::props::{Ctor, Kind, Param, Value, add_ctors};
use muo_content::utilities::types::{Numeric, PropType};
use muo_content::{core, loot};
use muo_io::{GenericReader, GenericWriter};
use muo_primitives::{DateTime, TimeSpan, utility};
use muo_server::World;
use muo_server::context_menu_entry::ContextMenuEntry;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ItemType, ops};
use muo_server::lr_reason::LRReason;
use muo_server::map::MapId;
use muo_server::mobile::AccessLevel;
use muo_server::net_state::NetStateId;
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::timer::{self, TimerHandle};
use muo_server::type_registry::{Phase, TypeRegistry};

use crate::simple::{random, random_bool, random_min_max};

const INT: Kind = Kind::Plain(PropType::Numeric(Numeric::Int));
const STR: Kind = Kind::Plain(PropType::Str);
const BOOL: Kind = Kind::Plain(PropType::Bool);
const ENTITY: Kind = Kind::Plain(PropType::Entity);

fn drop_item(w: &mut World, chest: ItemId, item: Option<ItemId>) {
    if let Some(item) = item
        && let Some(ty) = w.item(chest).map(|i| i.ty)
    {
        (ty.drop_item)(w, chest, item);
    }
}

fn delete(w: &mut World, id: ItemId) {
    if let Some(ty) = w.item(id).map(|i| i.ty) {
        (ty.delete)(w, id);
    }
}

fn set_trap_and_skill(w: &mut World, cont_id: ItemId, level: i32) {
    if let Some(d) = cont::ext_mut(w, cont_id) {
        d.trap_type = TrapType::ExplosionTrap;
        d.trap_power = level * 25;
        d.trap_level = level;
    }
}

/// The part of `Fill` both chests share: a piece of gear gets random attributes (AOS) or levels (pre-AOS).
fn add_gear(w: &mut World, chest: ItemId, item: ItemId, stats: fn(&mut World) -> (i32, i32, i32)) {
    let aos = core::aos(w);
    if item_is(w, item, "Server.Items.BaseWeapon") {
        if aos {
            let (c, min, max) = stats(w);
            muo_content::items::runic::apply_to_item(w, item, false, 0, c, min, max);
        } else {
            let d = random(w, 6);
            muo_content::items::weapon::set_damage_level(
                w,
                item,
                WeaponDamageLevel::from_i32(d).unwrap_or(WeaponDamageLevel::Regular),
            );
            let a = random(w, 6);
            muo_content::items::weapon::set_accuracy_level(
                w,
                item,
                WeaponAccuracyLevel::from_i32(a).unwrap_or(WeaponAccuracyLevel::Regular),
            );
            let u = random(w, 6);
            muo_content::items::weapon::set_durability_level(
                w,
                item,
                WeaponDurabilityLevel::from_i32(u).unwrap_or(WeaponDurabilityLevel::Regular),
            );
        }
        drop_item(w, chest, Some(item));
    } else if item_is(w, item, "Server.Items.BaseArmor") {
        if aos {
            let (c, min, max) = stats(w);
            muo_content::items::runic::apply_to_item(w, item, false, 0, c, min, max);
        } else {
            let p = random(w, 6);
            muo_content::items::armor::set_protection_level(
                w,
                item,
                ArmorProtectionLevel::from_i32(p).unwrap_or(ArmorProtectionLevel::Regular),
            );
            let d = random(w, 6);
            muo_content::items::armor::set_durability_level(
                w,
                item,
                ArmorDurabilityLevel::from_i32(d).unwrap_or(ArmorDurabilityLevel::Regular),
            );
        }
        drop_item(w, chest, Some(item));
    } else if item_is(w, item, "Server.Items.BaseHat") {
        if aos {
            let (c, min, max) = stats(w);
            muo_content::items::runic::apply_to_item(w, item, false, 0, c, min, max);
        }
        drop_item(w, chest, Some(item));
    } else if item_is(w, item, "Server.Items.BaseJewel") {
        let (c, min, max) = stats(w);
        muo_content::items::runic::apply_to_item(w, item, false, 0, c, min, max);
        drop_item(w, chest, Some(item));
    }
}

fn reagents_and_gems(w: &mut World, chest: ItemId, reagents: i32, gems: i32) {
    for _ in 0..reagents {
        let item = loot::random_possible_reagent(w);
        if let Some(i) = item {
            let n = random_min_max(w, 40, 60);
            ops::set_amount(w, i, n);
        }
        drop_item(w, chest, item);
    }
    for _ in 0..gems {
        let item = loot::random_gem(w);
        drop_item(w, chest, item);
    }
}

fn treasure_map(w: &mut World, level: i32, map: MapId) -> ItemId {
    crate::maps::treasure::treasure_map_new(
        w,
        &crate::maps::treasure::TREASURE_MAP,
        level,
        Some(map),
    )
}

// ---- ParagonChest ----

/// `ParagonChest._name` (`[SerializableField(0)]`).
#[derive(Clone, Debug, Default)]
struct ParagonName(Option<String>);

fn paragon_name(w: &World, id: ItemId) -> Option<String> {
    w.item(id)
        .and_then(|i| i.ext.get::<ParagonName>())
        .and_then(|n| n.0.clone())
}

/// `ParagonChest.GetRandomAOSStats`.
fn paragon_stats(w: &mut World) -> (i32, i32, i32) {
    let rnd = random(w, 15);
    if rnd < 1 {
        (random_min_max(w, 2, 6), 20, 70)
    } else if rnd < 3 {
        (random_min_max(w, 2, 4), 20, 50)
    } else if rnd < 6 {
        (random_min_max(w, 2, 3), 20, 40)
    } else if rnd < 10 {
        (random_min_max(w, 1, 2), 10, 30)
    } else {
        (1, 10, 20)
    }
}

/// `ParagonChest.Fill(level)`.
fn paragon_fill(w: &mut World, id: ItemId, level: i32) {
    set_trap_and_skill(w, id, level);
    cont::set_locked(w, id, true);
    let skill = match level {
        1 => Some(36),
        2 => Some(76),
        3 => Some(84),
        4 => Some(92),
        5 => Some(100),
        _ => None,
    };
    if let Some(s) = skill
        && let Some(d) = cont::ext_mut(w, id)
    {
        d.required_skill = s;
    }
    if let Some(d) = cont::ext_mut(w, id) {
        d.lock_level = d.required_skill - 10;
        d.max_lock_level = d.required_skill + 40;
    }
    let gold = new_gold(w, level * 200);
    drop_item(w, id, Some(gold));
    for _ in 0..level {
        let s = loot::random_scroll(w, 0, 63, muo_content::loot::SpellbookType::Regular);
        drop_item(w, id, s);
    }
    for _ in 0..level * 2 {
        let item = if core::aos(w) {
            loot::random_armor_or_shield_or_weapon_or_jewelry(w, false, false)
        } else {
            loot::random_armor_or_shield_or_weapon(w, false, false)
        };
        if let Some(item) = item {
            add_gear(w, id, item, paragon_stats);
        }
    }
    reagents_and_gems(w, id, level, level);
    let felucca = random_bool(w);
    let t = treasure_map(
        w,
        level + 1,
        if felucca {
            MapId::FELUCCA
        } else {
            MapId::TRAMMEL
        },
    );
    drop_item(w, id, Some(t));
}

/// `ParagonChest(name, level)`.
pub fn paragon_chest_new(w: &mut World, name: Option<String>, level: i32) -> ItemId {
    const IDS: [i32; 4] = [0x9AB, 0xE40, 0xE41, 0xE7C];
    const HUES: [i32; 11] = [
        0x0, 0x455, 0x47E, 0x89F, 0x8A5, 0x8AB, 0x966, 0x96D, 0x972, 0x973, 0x979,
    ];
    let gid = IDS[random(w, IDS.len() as i32) as usize];
    let id = new_lockable(w, &PARAGON_CHEST, gid);
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(ParagonName(name));
    }
    let h = HUES[random(w, HUES.len() as i32) as usize];
    crate::simple::hue(w, id, h);
    paragon_fill(w, id, level);
    id
}

/// `Server.Items.ParagonChest` (`[Flippable]`: the flip is in `own_flips.rs`). Its only constructor takes
/// `(string name, int level)`, so there is no parameterless `construct`.
pub static PARAGON_CHEST: ItemType = ItemType {
    full_name: "Server.Items.ParagonChest",
    parent: Some(&LOCKABLE_CONTAINER),
    construct: None,
    on_single_click: |w, id, from| {
        (LOCKABLE_CONTAINER.on_single_click)(w, id, from);
        let name = paragon_name(w, id).unwrap_or_default();
        ops::label_to_number(w, id, from, 1063449, &name);
    },
    get_properties: |w, id, list| {
        (LOCKABLE_CONTAINER.get_properties)(w, id, list);
        let name = paragon_name(w, id).unwrap_or_default();
        list.add_str(1063449, &name);
    },
    serialize: |w, id, out| {
        (LOCKABLE_CONTAINER.serialize)(w, id, out);
        out.write_i32(0);
        out.write_opt_string(paragon_name(w, id).as_deref());
    },
    deserialize: |w, id, r| {
        (LOCKABLE_CONTAINER.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        let name = r.read_string()?;
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(ParagonName(name));
        }
        Ok(())
    },
    ..LOCKABLE_CONTAINER
};

static PARAGON_CTORS: [Ctor; 1] = [Ctor {
    access: None,
    params: &[
        Param {
            name: "name",
            kind: STR,
            default: None,
            ty: "",
        },
        Param {
            name: "level",
            kind: INT,
            default: None,
            ty: "",
        },
    ],
    make: |w, v| EntityId::Item(paragon_chest_new(w, v[0].string(), v[1].i32())),
}];

// ---- TreasureMapChest ----

/// The fields of `TreasureMapChest`.
#[derive(Clone, Debug, Default)]
pub struct TmcData {
    pub guardians: Option<Vec<MobileId>>,
    pub temporary: bool,
    pub owner: Option<MobileId>,
    pub level: i32,
    pub expire: TimerHandle,
    pub lifted: Option<IndexSet<ItemId>>,
    /// False only while the constructor fills the chest; deserialized chests are always filled.
    pub filled: bool,
}

fn data(w: &World, id: ItemId) -> Option<&TmcData> {
    w.item(id)?.ext.get::<TmcData>()
}

fn data_mut(w: &mut World, id: ItemId) -> Option<&mut TmcData> {
    w.item_mut(id)?.ext.get_mut::<TmcData>()
}

/// `TreasureMapChest.Level`.
pub fn level(w: &World, id: ItemId) -> i32 {
    data(w, id).map_or(0, |d| d.level)
}

/// `Temporary = value`.
pub fn set_temporary(w: &mut World, id: ItemId, value: bool) {
    if let Some(d) = data_mut(w, id) {
        d.temporary = value;
    }
}

/// `AddToGuardians(m)`.
pub fn add_to_guardians(w: &mut World, id: ItemId, m: MobileId) {
    if let Some(d) = data_mut(w, id) {
        d.guardians.get_or_insert_with(Vec::new).push(m);
    }
}

fn start_expire(w: &mut World, id: ItemId, delay: TimeSpan) -> TimerHandle {
    timer::start(w, delay, TimeSpan::ZERO, 1, move |w| delete(w, id))
}

/// `TreasureMapChest.GetRandomAOSStats`.
fn tmc_stats(w: &mut World) -> (i32, i32, i32) {
    let rnd = random(w, 15);
    if core::se(w) {
        return if rnd < 1 {
            (random_min_max(w, 3, 5), 50, 100)
        } else if rnd < 3 {
            (random_min_max(w, 2, 5), 40, 80)
        } else if rnd < 6 {
            (random_min_max(w, 2, 4), 30, 60)
        } else if rnd < 10 {
            (random_min_max(w, 1, 3), 20, 40)
        } else {
            (1, 10, 20)
        };
    }
    if rnd < 1 {
        (random_min_max(w, 2, 5), 20, 70)
    } else if rnd < 3 {
        (random_min_max(w, 2, 4), 20, 50)
    } else if rnd < 6 {
        (random_min_max(w, 2, 3), 20, 40)
    } else if rnd < 10 {
        (random_min_max(w, 1, 2), 10, 30)
    } else {
        (1, 10, 20)
    }
}

/// `TreasureMapChest.Artifacts`.
const ARTIFACTS: [&str; 19] = [
    "CandelabraOfSouls",
    "GoldBricks",
    "PhillipsWoodenSteed",
    "ArcticDeathDealer",
    "BlazeOfDeath",
    "BurglarsBandana",
    "CavortingClub",
    "DreadPirateHat",
    "EnchantedTitanLegBone",
    "GwennosHarp",
    "IolosLute",
    "LunaLance",
    "NightsKiss",
    "NoxRangersHeavyCrossbow",
    "PolarBearMask",
    "VioletCourage",
    "HeartOfTheLion",
    "ColdBlood",
    "AlchemistsBauble",
];

/// `TreasureMapChest.Fill(cont, level)`.
pub fn fill(w: &mut World, id: ItemId, level: i32) {
    ops::set_movable(w, id, false);
    cont::set_locked(w, id, true);
    if level == 0 {
        if let Some(d) = cont::ext_mut(w, id) {
            d.lock_level = 0; // ILockpickable.CannotPick
        }
        let n = random_min_max(w, 50, 100);
        let gold = new_gold(w, n);
        drop_item(w, id, Some(gold));
        if utility::random_double(&mut w.rng) < 0.75 {
            let t = treasure_map(w, 0, MapId::TRAMMEL);
            drop_item(w, id, Some(t));
        }
    } else {
        set_trap_and_skill(w, id, level);
        let skill = match level {
            1 => 36,
            2 => 76,
            3 => 84,
            4 => 92,
            _ => 100,
        };
        if let Some(d) = cont::ext_mut(w, id) {
            d.required_skill = skill;
            d.lock_level = skill - 10;
            d.max_lock_level = skill + 40;
        }
        let gold = new_gold(w, level * 1000);
        drop_item(w, id, Some(gold));
        for _ in 0..level * 5 {
            let s = loot::random_scroll(w, 0, 63, muo_content::loot::SpellbookType::Regular);
            drop_item(w, id, s);
        }
        let items = if core::se(w) {
            match level {
                1 => 5,
                2 => 10,
                3 => 15,
                4 => 38,
                5 => 50,
                6 => 60,
                _ => 0,
            }
        } else {
            level * 6
        };
        for _ in 0..items {
            let item = if core::aos(w) {
                loot::random_armor_or_shield_or_weapon_or_jewelry(w, false, false)
            } else {
                loot::random_armor_or_shield_or_weapon(w, false, false)
            };
            if let Some(item) = item {
                add_gear(w, id, item, tmc_stats);
            }
        }
    }
    let reagents = if level == 0 { 12 } else { level * 3 };
    let gems = if level == 0 { 2 } else { level * 3 };
    reagents_and_gems(w, id, reagents, gems);
    if level == 6 && core::aos(w) {
        let i = random(w, ARTIFACTS.len() as i32) as usize;
        let item = make_item(w, &format!("Server.Items.{}", ARTIFACTS[i]));
        drop_item(w, id, item);
    }
}

/// `TreasureMapChest(owner, level, temporary)`: `base(0xE40)`.
pub fn treasure_map_chest_new(
    w: &mut World,
    owner: Option<MobileId>,
    level: i32,
    temporary: bool,
) -> ItemId {
    let id = new_lockable(w, &TREASURE_MAP_CHEST, 0xE40);
    let expire = start_expire(w, id, TimeSpan::from_hours(3));
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(TmcData {
            temporary,
            owner,
            level,
            expire,
            ..TmcData::default()
        });
    }
    fill(w, id, level);
    if let Some(d) = data_mut(w, id) {
        d.filled = true;
    }
    id
}

fn check_locked(w: &mut World, id: ItemId, from: MobileId) -> bool {
    if !cont::locked(w, id) {
        return false;
    }
    let access = w
        .mobile(from)
        .map_or(AccessLevel::Player, |m| m.access_level);
    if level(w, id) == 0 && access < AccessLevel::GameMaster {
        let guardians = data(w, id)
            .and_then(|d| d.guardians.clone())
            .unwrap_or_default();
        if guardians
            .iter()
            .any(|&g| muo_server::mobile::base::alive(w, g))
        {
            // You must first kill the guardians before you may open this chest.
            muo_server::item::deps::send_localized_message(w, from, 1046448);
            return true;
        }
        cont::lock_pick(w, id, from);
        return false;
    }
    (LOCKABLE_CLASS.check_locked)(w, id, from)
}

static TMC_CLASS: ContClass = ContClass {
    check_locked,
    ..LOCKABLE_CLASS
};

fn check_loot(w: &mut World, id: ItemId, m: MobileId, criminal_action: bool) -> bool {
    let Some(d) = data(w, id) else { return true };
    if d.temporary {
        return false;
    }
    let owner = d.owner;
    let access = w.mobile(m).map_or(AccessLevel::Player, |x| x.access_level);
    if access >= AccessLevel::GameMaster || owner.is_none() || owner == Some(m) {
        return true;
    }
    if let Some(o) = owner
        && muo_content::spells::hooks::hooks(w)
            .and_then(|h| h.party_contains)
            .is_some_and(|f| f(w, o, m))
    {
        return true;
    }
    let rules = w
        .item(id)
        .and_then(|i| i.map)
        .and_then(|mp| w.maps.get(mp))
        .map(|mp| mp.rules);
    if rules.is_some_and(|r| !r.contains(muo_data::map_definitions::MapRules::HARMFUL_RESTRICTIONS))
    {
        if criminal_action {
            muo_server::mobile::base::criminal_action(w, m, true);
        } else {
            muo_server::item::deps::send_localized_message(w, m, 1010630); // Taking someone else's treasure is a criminal offense!
        }
        return true;
    }
    muo_server::item::deps::send_localized_message(w, m, 1010631); // You did not discover this chest!
    false
}

fn lifted_add(w: &mut World, id: ItemId, item: ItemId) -> bool {
    data_mut(w, id).is_some_and(|d| d.lifted.get_or_insert_with(IndexSet::new).insert(item))
}

fn remove_entry(enabled: bool) -> ContextMenuEntry {
    let mut e = ContextMenuEntry::new(6149, 3);
    e.enabled = enabled;
    e.on_click = Some(Rc::new(
        |w: &mut World, from: MobileId, target: EntityId| {
            let EntityId::Item(chest) = target else {
                return;
            };
            if !muo_server::mobile::part2::stats::check_alive(w, from, true)
                || ops::is_deleted(w, chest)
            {
                return;
            }
            if data(w, chest).and_then(|d| d.owner) != Some(from) {
                return;
            }
            begin_remove(w, from, chest);
        },
    ));
    e
}

/// `RemoveGump`.
struct RemoveGump {
    chest: ItemId,
}

impl StaticGumpLogic for RemoveGump {
    const TYPE_NAME: &'static str = "Server.Items.TreasureMapChest+RemoveGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut StaticGumpBuilder) {
        b.add_page(0);
        b.set_no_close();
        b.set_no_dispose();
        b.add_background(30, 0, 240, 240, 2620);
        // When this treasure chest is removed, any items still inside of it will be lost.
        b.add_html_localized_color(45, 15, 200, 80, 1048125, 0x7FFF, false, false);
        // Are you certain you're ready to remove this chest?
        b.add_html_localized_color(45, 95, 200, 60, 1048126, 0x7FFF, false, false);
        b.add_button(40, 153, 4005, 4007, 1);
        b.add_html_localized_color(75, 155, 180, 40, 1048127, 0x7FFF, false, false); // Remove the Treasure Chest
        b.add_button(40, 195, 4005, 4007, 2);
        b.add_html_localized_color(75, 197, 180, 35, 1006045, 0x7FFF, false, false); // Cancel
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        if info.button_id == 1
            && let Some(m) = w.net_states.get(&sender).and_then(|n| n.mobile)
        {
            end_remove(w, self.chest, m);
        }
    }
}

/// `BeginRemove(from)`.
fn begin_remove(w: &mut World, from: MobileId, chest: ItemId) {
    let alive = muo_server::mobile::base::alive(w, from);
    if w.mobile(from).is_none_or(|m| m.net_state.is_none()) || !alive || ops::is_deleted(w, chest) {
        return;
    }
    send_gump_to_mobile(
        w,
        from,
        Box::new(StaticGumpObj::new(15, 15, RemoveGump { chest })),
        false,
    );
}

/// `EndRemove(from)`.
fn end_remove(w: &mut World, chest: ItemId, from: MobileId) {
    let loc = ops::get_world_location(w, chest);
    if ops::is_deleted(w, chest)
        || data(w, chest).and_then(|d| d.owner) != Some(from)
        || !muo_server::mobile::base::in_range_point3d(w, from, loc, 3)
    {
        return;
    }
    // The old, rusted chest crumbles when you hit it.
    muo_server::mobile::part2::messages::send_localized_message(w, from, 1048124, "", 0x8A5);
    delete(w, chest);
}

fn write_tmc(w: &World, id: ItemId, out: &mut dyn GenericWriter) {
    let d = data(w, id).cloned().unwrap_or_default();
    let guardians: Option<Vec<MobileId>> = d.guardians.map(|g| {
        g.into_iter()
            .filter(|&m| w.mobile(m).is_some_and(|x| !x.deleted))
            .collect()
    });
    match guardians {
        Some(g) => {
            out.write_bool(true);
            out.write_encoded_int(g.len() as i32);
            for m in g {
                out.write_mobile(w, Some(m));
            }
        }
        None => out.write_bool(false),
    }
    out.write_bool(d.temporary);
    out.write_mobile(w, d.owner);
    out.write_i32(d.level);
    let next = if d.expire == TimerHandle::default() {
        DateTime::MIN
    } else {
        d.expire.next(w)
    };
    out.write_anchored_time_ticks(next.ticks());
    match d.lifted {
        Some(l) => {
            let l: Vec<ItemId> = l
                .into_iter()
                .filter(|&i| w.item(i).is_some_and(|x| !x.deleted()))
                .collect();
            out.write_bool(true);
            out.write_encoded_int(l.len() as i32);
            for i in l {
                out.write_item(w, Some(i));
            }
        }
        None => out.write_bool(false),
    }
}

fn read_mobiles_encoded(w: &World, r: &mut dyn GenericReader) -> muo_io::Result<Vec<MobileId>> {
    let n = r.read_encoded_int()?;
    let mut v = Vec::new();
    for _ in 0..n {
        v.extend(r.read_mobile(w)?);
    }
    Ok(v)
}

fn read_items_encoded(w: &World, r: &mut dyn GenericReader) -> muo_io::Result<IndexSet<ItemId>> {
    let n = r.read_encoded_int()?;
    let mut v = IndexSet::new();
    for _ in 0..n {
        v.extend(r.read_item(w)?);
    }
    Ok(v)
}

/// `DeserializeExpireTimer(delay)`: `delay` is None for `TimeSpan.MinValue`.
fn deserialize_expire(
    w: &mut World,
    id: ItemId,
    temporary: bool,
    delay: Option<TimeSpan>,
) -> TimerHandle {
    if temporary {
        return TimerHandle::default();
    }
    start_expire(w, id, delay.unwrap_or(TimeSpan::ZERO))
}

fn read_expire_delay(
    w: &World,
    r: &mut dyn GenericReader,
    anchored: bool,
) -> muo_io::Result<Option<TimeSpan>> {
    let ticks = if anchored {
        r.read_anchored_time_ticks()?
    } else {
        r.read_delta_time_ticks(w.now.ticks())?
    };
    let next = DateTime::from_ticks(ticks);
    Ok(if next == DateTime::MIN {
        None
    } else {
        Some(next - w.now)
    })
}

fn read_tmc(
    w: &mut World,
    id: ItemId,
    r: &mut dyn GenericReader,
    version: i32,
) -> muo_io::Result<()> {
    let mut d = TmcData {
        filled: true,
        ..TmcData::default()
    };
    let delay;
    match version {
        2 | 3 => {
            if version == 2 {
                let g = read_mobiles_encoded(w, r)?;
                d.guardians = if g.is_empty() { None } else { Some(g) };
            } else if r.read_bool()? {
                d.guardians = Some(read_mobiles_encoded(w, r)?);
            }
            d.temporary = r.read_bool()?;
            d.owner = r.read_mobile(w)?;
            d.level = r.read_i32()?;
            delay = read_expire_delay(w, r, false)?;
            if version == 2 {
                let l = read_items_encoded(w, r)?;
                d.lifted = if l.is_empty() { None } else { Some(l) };
            } else if r.read_bool()? {
                d.lifted = Some(read_items_encoded(w, r)?);
            }
        }
        v if v < 2 => {
            d.owner = r.read_mobile(w)?;
            d.level = r.read_i32()?;
            delay = read_expire_delay(w, r, false)?;
            d.lifted = Some(r.read_item_list(w)?.into_iter().collect());
        }
        _ => {
            if r.read_bool()? {
                d.guardians = Some(read_mobiles_encoded(w, r)?);
            }
            d.temporary = r.read_bool()?;
            d.owner = r.read_mobile(w)?;
            d.level = r.read_i32()?;
            delay = read_expire_delay(w, r, true)?;
            if r.read_bool()? {
                d.lifted = Some(read_items_encoded(w, r)?);
            }
        }
    }
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(d.clone());
    }
    let h = deserialize_expire(w, id, d.temporary, delay);
    if let Some(d) = data_mut(w, id) {
        d.expire = h;
    }
    // [AfterDeserialization]: `if (_expireTimer == null) Timer.DelayCall(Delete)`
    if h == TimerHandle::default() {
        timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 1, move |w| delete(w, id));
    }
    Ok(())
}

/// `Server.Items.TreasureMapChest` (`[SerializationGenerator(4, false)]`). Its constructors take `(level)` and
/// `(owner, level, temporary)`, so there is no parameterless `construct`.
pub static TREASURE_MAP_CHEST: ItemType = ItemType {
    full_name: "Server.Items.TreasureMapChest",
    parent: Some(&LOCKABLE_CONTAINER),
    construct: None,
    family: Some(&TMC_CLASS),
    label_number: |_, _| 3000541,
    is_deco_container: |_, _| false,
    check_item_use: |w, id, from, item| {
        check_loot(w, id, from, item != id)
            && (LOCKABLE_CONTAINER.check_item_use)(w, id, from, item)
    },
    check_lift: |w, id, from, item, reject: &mut LRReason| {
        check_loot(w, id, from, true) && (LOCKABLE_CONTAINER.check_lift)(w, id, from, item, reject)
    },
    on_item_added: |w, id, item| {
        (LOCKABLE_CONTAINER.on_item_added)(w, id, item);
        if data(w, id).is_some_and(|d| d.filled) {
            lifted_add(w, id, item);
        }
    },
    on_item_lifted: |w, id, from, item| {
        muo_server::mobile::base::revealing_action(w, from);
        if !data(w, id)
            .and_then(|d| d.lifted.as_ref())
            .is_some_and(|l| l.contains(&item))
        {
            lifted_add(w, id, item);
            if utility::random_double(&mut w.rng) < 0.1 {
                // 10% chance to spawn a new monster
                let (lvl, loc, map) = (
                    level(w, id),
                    ops::get_world_location(w, id),
                    w.item(id).and_then(|i| i.map),
                );
                crate::maps::treasure::spawn(w, lvl, loc, map, Some(from), false);
            }
        }
        (LOCKABLE_CONTAINER.on_item_lifted)(w, id, from, item);
    },
    check_hold: |w, id, m, item, message, check_items, plus_items, plus_weight| {
        if let Some(m) = m
            && w.mobile(m)
                .is_some_and(|x| x.access_level < AccessLevel::GameMaster)
        {
            // The chest refuses to be filled with treasure again.
            muo_server::mobile::part2::messages::send_localized_message(w, m, 1048122, "", 0x8A5);
            return false;
        }
        (LOCKABLE_CONTAINER.check_hold)(
            w,
            id,
            m,
            item,
            message,
            check_items,
            plus_items,
            plus_weight,
        )
    },
    on_after_delete: |w, id| {
        if let Some(d) = data_mut(w, id) {
            let mut h = std::mem::take(&mut d.expire);
            h.cancel(w);
        }
        (LOCKABLE_CONTAINER.on_after_delete)(w, id);
    },
    get_context_menu_entries: |w, id, from, list| {
        (LOCKABLE_CONTAINER.get_context_menu_entries)(w, id, from, list);
        if muo_server::mobile::base::alive(w, from) {
            let owner = data(w, id).and_then(|d| d.owner);
            list.push(remove_entry(owner == Some(from)));
        }
    },
    serialize: |w, id, out| {
        (LOCKABLE_CONTAINER.serialize)(w, id, out);
        out.write_i32(4);
        write_tmc(w, id, out);
    },
    deserialize: |w, id, r| {
        (LOCKABLE_CONTAINER.deserialize)(w, id, r)?;
        let version = r.read_i32()?;
        read_tmc(w, id, r, version)
    },
    ..LOCKABLE_CONTAINER
};

static TMC_CTORS: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[Param {
            name: "level",
            kind: INT,
            default: None,
            ty: "",
        }],
        make: |w, v| EntityId::Item(treasure_map_chest_new(w, None, v[0].i32(), false)),
    },
    Ctor {
        access: None,
        params: &[
            Param {
                name: "owner",
                kind: ENTITY,
                default: None,
                ty: "Mobile",
            },
            Param {
                name: "level",
                kind: INT,
                default: None,
                ty: "",
            },
            Param {
                name: "temporary",
                kind: BOOL,
                default: Some(Value::Bool(false)),
                ty: "",
            },
        ],
        make: |w, v| {
            let owner = match v[0].entity() {
                Some(EntityId::Mobile(m)) => Some(m),
                _ => None,
            };
            EntityId::Item(treasure_map_chest_new(w, owner, v[1].i32(), v[2].bool()))
        },
    },
];

fn configure(w: &mut World) {
    add_ctors(w, "Server.Items.ParagonChest", &PARAGON_CTORS);
    add_ctors(w, "Server.Items.TreasureMapChest", &TMC_CTORS);
    w.resources
        .get_mut::<crate::maps::treasure::TreasureChestHooks>()
        .set_temporary = Some(set_temporary);
    w.resources
        .get_mut::<crate::maps::treasure::TreasureChestHooks>()
        .add_to_guardians = Some(add_to_guardians);
    w.resources
        .get_mut::<muo_content::items::house_deps::HouseDeps>()
        .treasure_map_chest_fill = Some(fill);
    w.resources
        .get_mut::<crate::skill_items::fishing::FishingHooks>()
        .fill_treasure_chest = Some(fill);
    muo_content::spells::hooks::hooks_mut(w).treasure_map_chest_level = Some(level);
    // `BaseCreature.OnDeath`: `PackItem(new ParagonChest(Name, treasureLevel))`
    w.resources
        .get_mut::<muo_content::hooks::DeathHooks>()
        .pack_paragon_chest = Some(|w, id, treasure_level| {
        let name = w.mobile(id).and_then(|m| (m.ty.name)(w, id));
        let chest = paragon_chest_new(w, name, treasure_level);
        muo_content::creature::setters::pack_item(w, id, Some(chest));
    });
}

/// Registers the classes of this file.
pub fn register(types: &mut TypeRegistry) {
    types.register_item(&PARAGON_CHEST, &[]);
    types.register_item(&TREASURE_MAP_CHEST, &[]);
    types.register_phase_default(
        Phase::Configure,
        "Server.Items.TreasureMapChest.Configure",
        configure,
    );
}

/// Treasure map chest guardian and lift tests. (Adding the spawned guardians on a dig needs the dig timer over client
/// map files; not tested here.)
#[cfg(test)]
mod behavior_tests {
    use super::*;
    use muo_primitives::Point3D;
    use muo_server::map::MapId;
    use muo_server::mobile::MOBILE_BASE;

    fn world() -> World {
        muo_server::map::test_support::world()
    }

    fn player_at(w: &mut World, x: i32, y: i32) -> MobileId {
        let m = w.new_mobile(&MOBILE_BASE);
        w.mobile_mut(m).unwrap().player = true;
        muo_server::mobile::base::move_to_world(w, m, Point3D::new(x, y, 0), MapId::FELUCCA);
        m
    }

    fn find_gold(w: &World, chest: ItemId, except: Option<ItemId>) -> Option<ItemId> {
        w.item(chest)?.items.iter().copied().find(|&i| {
            Some(i) != except
                && w.item(i).is_some_and(|x| {
                    muo_content::factory::item_is(w, i, "Server.Items.Gold") && x.amount > 0
                })
        })
    }

    // The guardian list starts null and is created by the first guardian added. A level 0 chest stays shut to
    // players until its guardians are dead.
    #[test]
    fn add_to_guardians_on_new_chest_keeps_level_zero_chest_shut() {
        let mut w = world();
        let player = player_at(&mut w, 5000, 610);
        let guardian = w.new_mobile(&MOBILE_BASE);
        let chest = treasure_map_chest_new(&mut w, None, 0, false);

        assert!(data(&w, chest).unwrap().guardians.is_none());

        add_to_guardians(&mut w, chest, guardian);

        assert_eq!(
            data(&w, chest).unwrap().guardians.as_deref(),
            Some(&[guardian][..])
        );
        assert!(check_locked(&mut w, chest, player));

        muo_server::mobile::base::delete(&mut w, guardian);

        assert!(!check_locked(&mut w, chest, player));
    }

    #[test]
    fn partial_lift_marks_split_remainder_as_lifted() {
        let mut w = world();
        let player = player_at(&mut w, 5000, 600);
        let chest = treasure_map_chest_new(&mut w, None, 1, false);
        ops::move_to_world_map(
            &mut w,
            chest,
            Point3D::new(5000, 600, 0),
            Some(MapId::FELUCCA),
        );
        cont::set_locked(&mut w, chest, false);

        let gold = find_gold(&w, chest, None).expect("the chest holds gold");
        let (rejected, _) = muo_server::mobile::base::lift(&mut w, player, gold, 1);
        assert!(!rejected);

        // The stack split re-adds the remainder as a brand-new item. It must count as already lifted, otherwise
        // every 1-coin pull grants a fresh guardian spawn roll.
        let remainder = find_gold(&w, chest, Some(gold)).expect("a split remainder");
        let lifted = data(&w, chest).unwrap().lifted.clone().unwrap_or_default();
        assert!(lifted.contains(&remainder));
        assert!(lifted.contains(&gold));
    }

    #[test]
    fn item_added_after_fill_is_marked_lifted() {
        let mut w = world();
        let chest = treasure_map_chest_new(&mut w, None, 1, false);
        let packed = muo_content::items::gold::new_gold(&mut w, 500);
        // Anything entering the chest after the initial fill (packed-back gold, split remainders, GM drops) was
        // never part of the original loot and must not grant spawn rolls when lifted back out.
        (w.item(chest).unwrap().ty.drop_item)(&mut w, chest, packed);
        assert!(
            data(&w, chest)
                .unwrap()
                .lifted
                .as_ref()
                .is_some_and(|l| l.contains(&packed))
        );
    }

    #[test]
    fn original_fill_loot_is_not_marked_lifted() {
        let mut w = world();
        let chest = treasure_map_chest_new(&mut w, None, 1, false);
        // The original loot must stay roll-eligible for its first lift.
        assert!(
            data(&w, chest)
                .unwrap()
                .lifted
                .as_ref()
                .is_none_or(|l| l.is_empty())
        );
    }
}
