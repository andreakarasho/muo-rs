//! The faction items: `BaseSystemController` (and its generated serializer), `Sigil`, `BaseMonolith`,
//! `StrongholdMonolith`, `TownMonolith`, `FactionStone`, `JoinStone` and `TownStone`.
//!
//! Each class keeps its fields in an `Ext` struct; the static lists `Sigil.Sigils` and `BaseMonolith.Monoliths`
//! are `FactionsState::{sigils, monoliths}`.

use muo_io::{GenericReader, GenericWriter};
use muo_primitives::{DateTime, MessageType};
use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, base, ops};
use muo_server::light_type::LightType;
use muo_server::lr_reason::LRReason;
use muo_server::mobile::AccessLevel;
use muo_server::mobile::part1::props_b;
use muo_server::mobile::part2::messages as mm;
use muo_server::property_list::PropertyList;
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::target::{self, TargetFlags, Targeted};
use muo_server::type_registry::TypeRegistry;

use super::state::{read_faction_ref, read_town_ref, write_faction_ref, write_town_ref};
use super::tables::{FACTION_DEFS, TOWN_DEFS};
use super::{find, st_mut, town};

/// `Sigil.CorruptionGrace`, `CorruptionPeriod`, `ReturnPeriod`, `PurificationPeriod`.
pub fn sigil_corruption_grace(w: &World) -> muo_primitives::TimeSpan {
    muo_primitives::TimeSpan::from_mins(if muo_content::core::se(w) { 30 } else { 15 })
}

pub fn sigil_corruption_period(w: &World) -> muo_primitives::TimeSpan {
    muo_primitives::TimeSpan::from_hours(if muo_content::core::se(w) { 10 } else { 24 })
}

pub const SIGIL_RETURN_PERIOD: muo_primitives::TimeSpan = muo_primitives::TimeSpan::from_hours(1);
pub const SIGIL_PURIFICATION_PERIOD: muo_primitives::TimeSpan =
    muo_primitives::TimeSpan::from_days(3);

/// `Sigil.OwnershipHue`.
pub const OWNERSHIP_HUE: i32 = 0xB;

/// `BaseSystemController._labelNumber`.
#[derive(Default)]
pub struct Controller {
    pub label_number: i32,
}

/// The fields of `Sigil`.
#[derive(Default)]
pub struct SigilData {
    pub town: Option<usize>,
    pub corrupted: Option<usize>,
    pub corrupting: Option<usize>,
    pub last_monolith: Option<ItemId>,
    pub last_stolen: DateTime,
    pub grace_start: DateTime,
    pub corruption_start: DateTime,
    pub purification_start: DateTime,
}

/// The fields of `BaseMonolith`.
#[derive(Default)]
pub struct MonolithData {
    pub town: Option<usize>,
    pub faction: Option<usize>,
    pub sigil: Option<ItemId>,
}

/// `FactionStone.Faction` / `JoinStone.Faction`.
#[derive(Default)]
pub struct StoneData {
    pub faction: Option<usize>,
}

/// `TownStone.Town`.
#[derive(Default)]
pub struct TownStoneData {
    pub town: Option<usize>,
}

fn rd(r: &mut dyn GenericReader) -> muo_io::Result<DateTime> {
    Ok(DateTime::from_ticks(r.read_date_time_ticks()?))
}

// ---- BaseSystemController ----

/// `LabelNumber`: `_labelNumber > 0 ? _labelNumber : DefaultLabelNumber`.
fn controller_label(w: &World, id: ItemId, default: i32) -> i32 {
    let n = w
        .item(id)
        .and_then(|i| i.ext.get::<Controller>())
        .map_or(0, |c| c.label_number);
    if n > 0 { n } else { default }
}

/// `AssignName(name)` for a `TextDefinition` that is a number or nothing.
pub fn assign_name(w: &mut World, id: ItemId, number: Option<i32>) {
    let n = number.filter(|&n| n > 0).unwrap_or(0);
    if let Some(c) = w.item_mut(id).and_then(|i| i.ext.get_mut::<Controller>()) {
        c.label_number = n;
    }
    ops::set_name(w, id, None);
    ops::invalidate_properties(w, id);
}

fn controller_serialize(w: &World, id: ItemId, out: &mut dyn GenericWriter) {
    (ITEM_BASE.serialize)(w, id, out);
    out.write_i32(1); // SerializationVersion
    out.write_i32(
        w.item(id)
            .and_then(|i| i.ext.get::<Controller>())
            .map_or(0, |c| c.label_number),
    );
}

fn controller_deserialize(
    w: &mut World,
    id: ItemId,
    r: &mut dyn GenericReader,
) -> muo_io::Result<()> {
    (ITEM_BASE.deserialize)(w, id, r)?;
    let version = r.read_i32()?;
    if version >= 1 {
        let n = r.read_i32()?;
        if let Some(c) = w.item_mut(id).and_then(|i| i.ext.get_mut::<Controller>()) {
            c.label_number = n;
        }
    }
    Ok(())
}

/// `BaseSystemController`.
pub static BASE_SYSTEM_CONTROLLER: ItemType = ItemType {
    full_name: "Server.Factions.BaseSystemController",
    parent: Some(&ITEM_BASE),
    construct: None,
    label_number: |w, id| controller_label(w, id, 0),
    serialize: controller_serialize,
    deserialize: controller_deserialize,
    ..ITEM_BASE
};

fn new_controller(w: &mut World, ty: &'static ItemType, item_id: i32) -> ItemId {
    let id = ops::new_item_with_id(w, ty, item_id);
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(Controller::default());
    }
    id
}

// ---- Sigil ----

pub fn sigil(w: &World, id: ItemId) -> Option<&SigilData> {
    w.item(id)?.ext.get::<SigilData>()
}

pub fn sigil_mut(w: &mut World, id: ItemId) -> Option<&mut SigilData> {
    w.item_mut(id)?.ext.get_mut::<SigilData>()
}

pub fn is_sigil(w: &World, id: ItemId) -> bool {
    w.item(id).is_some_and(|i| i.ty.is(&SIGIL))
}

/// `new Sigil(town)`.
pub fn new_sigil(w: &mut World, town: Option<usize>) -> ItemId {
    let id = new_controller(w, &SIGIL, 0x1869);
    ops::set_movable(w, id, false);
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(SigilData::default());
    }
    sigil_set_town(w, id, town);
    st_mut(w).sigils.push(id);
    id
}

/// `Town` setter.
pub fn sigil_set_town(w: &mut World, id: ItemId, t: Option<usize>) {
    if let Some(d) = sigil_mut(w, id) {
        d.town = t;
    }
    sigil_update(w, id);
}

/// `Corrupted` setter.
pub fn sigil_set_corrupted(w: &mut World, id: ItemId, f: Option<usize>) {
    if let Some(d) = sigil_mut(w, id) {
        d.corrupted = f;
    }
    sigil_update(w, id);
}

/// `Corrupting` setter.
pub fn sigil_set_corrupting(w: &mut World, id: ItemId, f: Option<usize>) {
    if let Some(d) = sigil_mut(w, id) {
        d.corrupting = f;
    }
    sigil_update(w, id);
}

/// `IsBeingCorrupted`.
pub fn is_being_corrupted(w: &World, id: ItemId) -> bool {
    let Some(d) = sigil(w, id) else { return false };
    d.corrupting.is_some()
        && d.last_monolith
            .is_some_and(|m| is_stronghold_monolith(w, m))
        && d.last_monolith.and_then(|m| monolith_faction(w, m)) == d.corrupting
}

pub fn is_corrupted(w: &World, id: ItemId) -> bool {
    sigil(w, id).is_some_and(|d| d.corrupted.is_some())
}

pub fn is_purifying(w: &World, id: ItemId) -> bool {
    sigil(w, id).is_some_and(|d| d.purification_start != DateTime::MIN)
}

pub fn is_corrupting(w: &World, id: ItemId) -> bool {
    sigil(w, id).is_some_and(|d| d.corrupting.is_some() && d.corrupting != d.corrupted)
}

/// `Update()`.
pub fn sigil_update(w: &mut World, id: ItemId) {
    let Some(t) = sigil(w, id).map(|d| d.town) else {
        return;
    };
    let item_id = t.map_or(0x1869, |t| TOWN_DEFS[t].sigil_id);
    base::set_item_id(w, id, item_id);
    let name = match t {
        None => None,
        Some(t) if is_corrupted(w, id) || is_purifying(w, id) => {
            Some(TOWN_DEFS[t].corrupted_sigil_name)
        }
        Some(t) => Some(TOWN_DEFS[t].sigil_name),
    };
    assign_name(w, id, name);
    ops::invalidate_properties(w, id);
}

fn sigil_get_properties(w: &World, id: ItemId, list: &mut PropertyList) {
    (ITEM_BASE.get_properties)(w, id, list);
    let Some(d) = sigil(w, id) else { return };
    if let Some(f) = d.corrupted {
        list.add(FACTION_DEFS[f].sigil_control); // SigilControl.AddTo(list)
    } else {
        list.add(1042256); // This sigil is not corrupted.
    }
    if is_corrupting(w, id) {
        list.add(1042257); // This sigil is in the process of being corrupted.
    } else if is_purifying(w, id) {
        list.add(1042258); // This sigil has recently been corrupted, and is undergoing purification.
    } else {
        list.add(1042259); // This sigil is not in the process of being corrupted.
    }
}

fn sigil_on_single_click(w: &mut World, id: ItemId, from: MobileId) {
    (ITEM_BASE.on_single_click)(w, id, from);
    if let Some(f) = sigil(w, id).and_then(|d| d.corrupted) {
        let n = FACTION_DEFS[f].sigil_control;
        if n > 0 {
            ops::label_to_number(w, id, from, n, "");
        }
    } else {
        ops::label_to_number(w, id, from, 1042256, ""); // This sigil is not corrupted.
    }
    if is_corrupting(w, id) {
        ops::label_to_number(w, id, from, 1042257, "");
    } else if is_purifying(w, id) {
        ops::label_to_number(w, id, from, 1042258, "");
    } else {
        ops::label_to_number(w, id, from, 1042259, "");
    }
}

fn find_owner(w: &World, parent: EntityId) -> Option<MobileId> {
    match parent {
        EntityId::Item(i) => match ops::root_parent(w, i) {
            Some(EntityId::Mobile(m)) => Some(m),
            _ => None,
        },
        EntityId::Mobile(m) => Some(m),
    }
}

/// The `toSteal is Sigil sig` branch of `StealingTarget.TryStealItem` after the range and ground checks: returns
/// the sigil when it was stolen.
pub fn steal_sigil(w: &mut World, thief: MobileId, sig: ItemId) -> Option<ItemId> {
    use muo_server::mobile::part1::combat::can_begin_action;
    let pl = super::player_state(w, thief);
    let faction = pl.as_ref().map(|(f, _)| *f);
    let msg = |w: &mut World, n: i32| mm::send_localized_message(w, thief, n, "", 0x3B2);
    if faction.is_none() {
        msg(w, 1005588); // You must join a faction to do that
    } else if !can_begin_action(w, thief, "Server.Spells.Fifth.IncognitoSpell") {
        msg(w, 1010581); // You cannot steal the sigil when you are incognito
    } else if muo_items::skill_items::thief::is_disguised(w, thief) {
        msg(w, 1010583); // You cannot steal the sigil while disguised
    } else if !can_begin_action(w, thief, "Server.Spells.Seventh.PolymorphSpell") {
        msg(w, 1010582); // You cannot steal the sigil while polymorphed
    } else if muo_content::spells::helper::under_transformation(w, thief) {
        msg(w, 1061622); // You cannot steal the sigil while in that form.
    } else if muo_spells::ninjitsu::animal_form::under_transformation(w, thief) {
        msg(w, 1063222); // You cannot steal the sigil while mimicking an animal.
    } else if pl.as_ref().is_some_and(|(_, ps)| ps.is_leaving()) {
        msg(w, 1005589); // You are currently quitting a faction and cannot steal the town sigil
    } else if is_being_corrupted(w, sig)
        && sigil(w, sig)
            .and_then(|d| d.last_monolith)
            .and_then(|m| monolith_faction(w, m))
            == faction
    {
        msg(w, 1005590); // You cannot steal your own sigil
    } else if is_purifying(w, sig) {
        msg(w, 1005592); // You cannot steal this sigil until it has been purified
    } else if !muo_server::mobile::part2::stats::check_target_skill(
        w,
        thief,
        muo_server::skills::SkillName::Stealing,
        Targeted::Entity(EntityId::Item(sig)),
        80.0,
        80.0,
    ) {
        msg(w, 1005594); // You do not have enough skill to steal the sigil
    } else if exists_on(w, thief) {
        msg(w, 1010258); // The sigil has gone back to its home location because you already have a sigil.
    } else if !w
        .mobile(thief)
        .and_then(|m| m.backpack)
        .is_some_and(|pack| {
            muo_server::item::container::check_hold_items(w, pack, Some(thief), sig, false, true)
        })
    {
        msg(w, 1010259); // The sigil has gone home because your backpack is full
    } else {
        let now = w.now;
        if is_being_corrupted(w, sig)
            && let Some(d) = sigil_mut(w, sig)
        {
            d.grace_start = now; // begin grace period
        }
        msg(w, 1010586); // YOU STOLE THE SIGIL!!!
        if let Some(m) = sigil(w, sig).and_then(|d| d.last_monolith)
            && monolith(w, m).is_some_and(|d| d.sigil.is_some())
        {
            monolith_set_sigil(w, m, None);
        }
        if let Some(d) = sigil_mut(w, sig) {
            d.last_stolen = now;
        }
        return Some(sig);
    }
    None
}

/// `ExistsOn(mob)`.
pub fn exists_on(w: &World, mob: MobileId) -> bool {
    find_in_pack(w, mob).is_some()
}

fn find_in_pack(w: &World, mob: MobileId) -> Option<ItemId> {
    let pack = w.mobile(mob)?.backpack?;
    muo_server::item::enumerable::find_item_by_type(w, pack, true, &SIGIL)
}

/// All the sigils in a backpack (`EnumerateItemsByType<Sigil>`).
pub fn sigils_in_pack(w: &World, mob: MobileId) -> Vec<ItemId> {
    match w.mobile(mob).and_then(|m| m.backpack) {
        Some(pack) => {
            muo_server::item::enumerable::find_items_by_type(w, pack, true, &SIGIL, |_, _| true)
        }
        None => Vec::new(),
    }
}

fn sigil_on_added(w: &mut World, id: ItemId, parent: EntityId) {
    (ITEM_BASE.on_added)(w, id, parent);
    if let Some(m) = find_owner(w, parent) {
        props_b::set_solid_hue_override(w, m, OWNERSHIP_HUE);
    }
}

fn sigil_on_removed(w: &mut World, id: ItemId, parent: EntityId) {
    (ITEM_BASE.on_removed)(w, id, parent);
    if let Some(m) = find_owner(w, parent) {
        props_b::set_solid_hue_override(w, m, -1);
    }
}

fn sigil_check_lift(
    w: &mut World,
    _id: ItemId,
    from: MobileId,
    _item: ItemId,
    _r: &mut LRReason,
) -> bool {
    mm::send_localized_message(w, from, 1005225, "", 0x3B2); // You must use the stealing skill to pick up the sigil
    false
}

fn sigil_on_double_click(w: &mut World, id: ItemId, from: MobileId) {
    let pack = w.mobile(from).and_then(|m| m.backpack);
    if ops::is_child_of(w, id, pack.map(EntityId::Item)) {
        target::begin_target(w, from, 1, false, TargetFlags::NONE, move |w, from, obj| {
            sigil_on_target(w, id, from, obj)
        });
        mm::send_localized_message(w, from, 1042251, "", 0x3B2); // Click on a sigil monolith or player
    }
}

fn begin_corrupting(w: &mut World, id: ItemId, f: Option<usize>) {
    let now = w.now;
    if let Some(d) = sigil_mut(w, id) {
        d.corrupting = f;
        d.corruption_start = now;
    }
}

fn clear_corrupting(w: &mut World, id: ItemId) {
    if let Some(d) = sigil_mut(w, id) {
        d.corrupting = None;
        d.corruption_start = DateTime::MIN;
    }
}

fn sigil_on_target(w: &mut World, id: ItemId, from: MobileId, obj: Targeted) {
    let pack = w.mobile(from).and_then(|m| m.backpack);
    if ops::is_deleted(w, id) || !ops::is_child_of(w, id, pack.map(EntityId::Item)) {
        return;
    }
    let msg = |w: &mut World, n: i32| mm::send_localized_message(w, from, n, "", 0x3B2);
    match obj {
        Targeted::Entity(EntityId::Mobile(targ)) => {
            if muo_content::player_mobile::data(w, targ).is_some() {
                let (to_faction, from_faction) = (find(w, targ), find(w, from));
                if to_faction.is_none() {
                    msg(w, 1005223); // You cannot give the sigil to someone not in a faction
                } else if from_faction != to_faction {
                    msg(w, 1005222); // You cannot give the sigil to someone not in your faction
                } else if exists_on(w, targ) {
                    msg(w, 1005220); // You cannot give this sigil to someone who already has a sigil
                } else if !muo_server::mobile::base::alive(w, targ) {
                    msg(w, 1042248); // You cannot give a sigil to a dead person.
                } else if w.mobile(from).is_some_and(|m| m.net_state.is_some())
                    && w.mobile(targ).is_some_and(|m| m.net_state.is_some())
                    && let Some(p) = w.mobile(targ).and_then(|m| m.backpack)
                    && let Some(ty) = w.item(p).map(|i| i.ty)
                {
                    (ty.drop_item)(w, p, id);
                }
            } else {
                msg(w, 1005221); // You cannot give the sigil to them
            }
        }
        Targeted::Entity(EntityId::Item(mono)) if is_monolith(w, mono) => {
            sigil_on_monolith(w, id, from, mono);
        }
        _ => msg(w, 1005224), // You can't use the sigil on that
    }
    sigil_update(w, id);
}

fn sigil_on_monolith(w: &mut World, id: ItemId, from: MobileId, mono: ItemId) {
    let msg = |w: &mut World, n: i32| mm::send_localized_message(w, from, n, "", 0x3B2);
    let (sigil_town, corrupted, corrupting, grace_start) = {
        let d = sigil(w, id).expect("sigil data");
        (d.town, d.corrupted, d.corrupting, d.grace_start)
    };
    let mono_town = monolith_town(w, mono);
    if is_stronghold_monolith(w, mono) {
        let mono_faction = monolith_faction(w, mono);
        if mono_faction.is_none() || mono_faction != find(w, from) {
            msg(w, 1042246); // You can't place that on an enemy monolith
        } else if mono_town.is_none() || mono_town != sigil_town {
            msg(w, 1042247); // That is not the correct faction monolith
        } else {
            monolith_set_sigil(w, mono, Some(id));
            let new_controller = mono_faction;
            let old_controller = corrupting;
            let now = w.now;
            if old_controller.is_none() {
                if corrupted != new_controller {
                    begin_corrupting(w, id, new_controller);
                }
            } else if grace_start > DateTime::MIN && grace_start + sigil_corruption_grace(w) < now {
                if corrupted != new_controller {
                    begin_corrupting(w, id, new_controller); // grace time over, reset period
                } else {
                    clear_corrupting(w, id);
                }
                if let Some(d) = sigil_mut(w, id) {
                    d.grace_start = DateTime::MIN;
                }
            } else if new_controller == old_controller {
                if let Some(d) = sigil_mut(w, id) {
                    d.grace_start = DateTime::MIN; // returned within grace period
                }
            } else if grace_start == DateTime::MIN
                && let Some(d) = sigil_mut(w, id)
            {
                d.grace_start = now;
            }
            if let Some(d) = sigil_mut(w, id) {
                d.purification_start = DateTime::MIN;
            }
        }
    } else if mono_town.is_none() || mono_town != sigil_town {
        msg(w, 1042245); // This is not the correct town sigil monolith
    } else if corrupted.is_none() || corrupted != find(w, from) {
        // Your faction did not corrupt this sigil.  Take it to your stronghold.
        msg(w, 1042244);
    } else {
        monolith_set_sigil(w, mono, Some(id));
        let now = w.now;
        if let Some(d) = sigil_mut(w, id) {
            d.corrupting = None;
            d.purification_start = now;
            d.corruption_start = DateTime::MIN;
        }
        if let Some(t) = sigil_town {
            town::capture(w, t, corrupted);
        }
        if let Some(d) = sigil_mut(w, id) {
            d.corrupted = None;
        }
    }
}

fn sigil_serialize(w: &World, id: ItemId, out: &mut dyn GenericWriter) {
    controller_serialize(w, id, out);
    out.write_i32(0); // version
    let Some(d) = sigil(w, id) else { return };
    write_town_ref(out, d.town);
    write_faction_ref(out, d.corrupted);
    write_faction_ref(out, d.corrupting);
    out.write_item(w, d.last_monolith);
    out.write_date_time_ticks(d.last_stolen.ticks());
    out.write_date_time_ticks(d.grace_start.ticks());
    out.write_date_time_ticks(d.corruption_start.ticks());
    out.write_date_time_ticks(d.purification_start.ticks());
}

fn sigil_deserialize(w: &mut World, id: ItemId, r: &mut dyn GenericReader) -> muo_io::Result<()> {
    controller_deserialize(w, id, r)?;
    let version = r.read_i32()?;
    if version == 0 {
        let d = SigilData {
            town: read_town_ref(r)?,
            corrupted: read_faction_ref(r)?,
            corrupting: read_faction_ref(r)?,
            last_monolith: r.read_item(w)?,
            last_stolen: rd(r)?,
            grace_start: rd(r)?,
            corruption_start: rd(r)?,
            purification_start: rd(r)?,
        };
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(d);
        }
        sigil_update(w, id);
        if let Some(EntityId::Mobile(m)) = ops::root_parent(w, id) {
            props_b::set_solid_hue_override(w, m, OWNERSHIP_HUE);
        }
    }
    Ok(())
}

/// `ReturnHome()`.
pub fn return_home(w: &mut World, id: ItemId) -> bool {
    let d = sigil(w, id);
    let mut monolith = d.and_then(|d| d.last_monolith);
    if monolith.is_none()
        && let Some(t) = d.and_then(|d| d.town)
    {
        monolith = town::monolith(w, t);
    }
    let alive = monolith.filter(|&m| w.item(m).is_some_and(|i| !i.deleted()));
    if let Some(m) = alive {
        monolith_set_sigil(w, m, Some(id));
    }
    alive.is_some()
}

fn sigil_delete(w: &mut World, id: ItemId) {
    if return_home(w, id) {
        return;
    }
    (ITEM_BASE.delete)(w, id);
}

fn sigil_on_parent_deleted(w: &mut World, id: ItemId, parent: EntityId) {
    (ITEM_BASE.on_parent_deleted)(w, id, parent);
    return_home(w, id);
}

fn sigil_on_after_delete(w: &mut World, id: ItemId) {
    (ITEM_BASE.on_after_delete)(w, id);
    st_mut(w).sigils.retain(|&s| s != id);
}

pub static SIGIL: ItemType = ItemType {
    full_name: "Server.Factions.Sigil",
    parent: Some(&BASE_SYSTEM_CONTROLLER),
    construct: None,
    get_properties: sigil_get_properties,
    on_single_click: sigil_on_single_click,
    check_lift: sigil_check_lift,
    on_added: sigil_on_added,
    on_removed: sigil_on_removed,
    on_double_click: sigil_on_double_click,
    on_parent_deleted: sigil_on_parent_deleted,
    on_after_delete: sigil_on_after_delete,
    delete: sigil_delete,
    serialize: sigil_serialize,
    deserialize: |w, id, r| {
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(Controller::default());
        }
        sigil_deserialize(w, id, r)?;
        st_mut(w).sigils.push(id);
        Ok(())
    },
    ..BASE_SYSTEM_CONTROLLER
};

// ---- monoliths ----

pub fn monolith(w: &World, id: ItemId) -> Option<&MonolithData> {
    w.item(id)?.ext.get::<MonolithData>()
}

fn monolith_mut(w: &mut World, id: ItemId) -> Option<&mut MonolithData> {
    w.item_mut(id)?.ext.get_mut::<MonolithData>()
}

pub fn is_monolith(w: &World, id: ItemId) -> bool {
    w.item(id).is_some_and(|i| i.ty.is(&BASE_MONOLITH))
}

pub fn is_stronghold_monolith(w: &World, id: ItemId) -> bool {
    w.item(id).is_some_and(|i| i.ty.is(&STRONGHOLD_MONOLITH))
}

pub fn is_town_monolith(w: &World, id: ItemId) -> bool {
    w.item(id).is_some_and(|i| i.ty.is(&TOWN_MONOLITH))
}

pub fn monolith_town(w: &World, id: ItemId) -> Option<usize> {
    monolith(w, id).and_then(|d| d.town)
}

pub fn monolith_faction(w: &World, id: ItemId) -> Option<usize> {
    monolith(w, id).and_then(|d| d.faction)
}

pub fn monolith_sigil(w: &World, id: ItemId) -> Option<ItemId> {
    monolith(w, id).and_then(|d| d.sigil)
}

/// `new StrongholdMonolith(town, faction)` / `new TownMonolith(town)`.
pub fn new_monolith(w: &mut World, stronghold: bool, t: Option<usize>, f: Option<usize>) -> ItemId {
    let ty = if stronghold {
        &STRONGHOLD_MONOLITH
    } else {
        &TOWN_MONOLITH
    };
    let id = new_controller(w, ty, 0x1183);
    ops::set_movable(w, id, false);
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(MonolithData::default());
    }
    monolith_set_town(w, id, t);
    monolith_set_faction(w, id, f);
    st_mut(w).monoliths.push(id);
    id
}

/// `Town` setter (`OnTownChanged`).
pub fn monolith_set_town(w: &mut World, id: ItemId, t: Option<usize>) {
    if let Some(d) = monolith_mut(w, id) {
        d.town = t;
    }
    let number = t.map(|t| {
        if is_stronghold_monolith(w, id) {
            TOWN_DEFS[t].stronghold_monolith_name
        } else {
            TOWN_DEFS[t].town_monolith_name
        }
    });
    assign_name(w, id, number);
}

/// `Faction` setter.
pub fn monolith_set_faction(w: &mut World, id: ItemId, f: Option<usize>) {
    if let Some(d) = monolith_mut(w, id) {
        d.faction = f;
    }
    base::set_hue(w, id, f.map_or(0, |f| FACTION_DEFS[f].hue_primary));
}

/// `Sigil` setter.
pub fn monolith_set_sigil(w: &mut World, id: ItemId, value: Option<ItemId>) {
    let old = monolith_sigil(w, id);
    if old == value {
        return;
    }
    if let Some(d) = monolith_mut(w, id) {
        d.sigil = value;
    }
    if let Some(o) = old {
        ops::set_light(w, o, LightType::Empty);
    }
    if let Some(s) = value
        && let Some(last) = sigil(w, s).and_then(|d| d.last_monolith)
        && last != id
        && monolith_sigil(w, last) == Some(s)
    {
        monolith_set_sigil(w, last, None);
    }
    if let Some(s) = value
        && let Some(d) = sigil_mut(w, s)
    {
        d.last_monolith = Some(id);
    }
    update_sigil(w, id);
}

/// `UpdateSigil()`.
pub fn update_sigil(w: &mut World, id: ItemId) {
    let Some(s) = monolith_sigil(w, id) else {
        return;
    };
    if w.item(s).is_none_or(|i| i.deleted()) {
        return;
    }
    ops::set_light(w, s, LightType::Circle225);
    let (loc, map) = w.item(id).map(|i| (i.location, i.map)).unwrap_or_default();
    ops::move_to_world_map(
        w,
        s,
        muo_primitives::Point3D::new(loc.x, loc.y, loc.z + 18),
        map,
    );
}

fn monolith_serialize(w: &World, id: ItemId, out: &mut dyn GenericWriter) {
    controller_serialize(w, id, out);
    out.write_i32(0); // BaseMonolith version
    if let Some(d) = monolith(w, id) {
        write_town_ref(out, d.town);
        write_faction_ref(out, d.faction);
        out.write_item(w, d.sigil);
    }
    out.write_i32(0); // StrongholdMonolith / TownMonolith version
}

fn monolith_deserialize(
    w: &mut World,
    id: ItemId,
    r: &mut dyn GenericReader,
) -> muo_io::Result<()> {
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(Controller::default());
        i.ext.insert(MonolithData::default());
    }
    controller_deserialize(w, id, r)?;
    st_mut(w).monoliths.push(id);
    let version = r.read_i32()?;
    if version == 0 {
        let t = read_town_ref(r)?;
        monolith_set_town(w, id, t);
        let f = read_faction_ref(r)?;
        monolith_set_faction(w, id, f);
        let s = r.read_item(w)?;
        if let Some(d) = monolith_mut(w, id) {
            d.sigil = s;
        }
    }
    let _version = r.read_i32()?;
    Ok(())
}

fn monolith_on_after_delete(w: &mut World, id: ItemId) {
    (ITEM_BASE.on_after_delete)(w, id);
    st_mut(w).monoliths.retain(|&m| m != id);
}

pub static BASE_MONOLITH: ItemType = ItemType {
    full_name: "Server.Factions.BaseMonolith",
    parent: Some(&BASE_SYSTEM_CONTROLLER),
    construct: None,
    on_location_change: |w, id, old| {
        (ITEM_BASE.on_location_change)(w, id, old);
        update_sigil(w, id);
    },
    on_map_change: |w, id| {
        (ITEM_BASE.on_map_change)(w, id);
        update_sigil(w, id);
    },
    on_after_delete: monolith_on_after_delete,
    serialize: monolith_serialize,
    deserialize: monolith_deserialize,
    ..BASE_SYSTEM_CONTROLLER
};

pub static STRONGHOLD_MONOLITH: ItemType = ItemType {
    full_name: "Server.Factions.StrongholdMonolith",
    parent: Some(&BASE_MONOLITH),
    label_number: |w, id| controller_label(w, id, 1041042), // A Faction Sigil Monolith
    ..BASE_MONOLITH
};

pub static TOWN_MONOLITH: ItemType = ItemType {
    full_name: "Server.Factions.TownMonolith",
    parent: Some(&BASE_MONOLITH),
    label_number: |w, id| controller_label(w, id, 1041403), // A Faction Town Sigil Monolith
    ..BASE_MONOLITH
};

// ---- stones ----

/// `from.InRange(GetWorldLocation(), 2)` or "I can't reach that."
fn reach_check(w: &mut World, id: ItemId, from: MobileId) -> bool {
    let loc = ops::get_world_location(w, id);
    if muo_server::mobile::base::in_range_point3d(w, from, loc, 2) {
        return true;
    }
    mm::local_overhead_message_number(w, from, MessageType::REGULAR, 0x3B2, 1019045, ""); // I can't reach that.
    false
}

/// `FactionGump.Exists(from)`.
pub fn faction_gump_exists(w: &World, from: MobileId) -> bool {
    let Some(ns) = w.mobile(from).and_then(|m| m.net_state) else {
        return false;
    };
    muo_content::gumps::system::gumps_of(w, ns)
        .any(|g| super::gumps::is_faction_gump(g.type_name()))
}

fn stone_data(w: &World, id: ItemId) -> Option<&StoneData> {
    w.item(id)?.ext.get::<StoneData>()
}

pub fn stone_faction(w: &World, id: ItemId) -> Option<usize> {
    stone_data(w, id).and_then(|d| d.faction)
}

/// `FactionStone.Faction` setter.
pub fn faction_stone_set_faction(w: &mut World, id: ItemId, f: Option<usize>) {
    if let Some(d) = w.item_mut(id).and_then(|i| i.ext.get_mut::<StoneData>()) {
        d.faction = f;
    }
    assign_name(w, id, f.map(|f| FACTION_DEFS[f].faction_stone_name));
}

/// `JoinStone.Faction` setter.
pub fn join_stone_set_faction(w: &mut World, id: ItemId, f: Option<usize>) {
    if let Some(d) = w.item_mut(id).and_then(|i| i.ext.get_mut::<StoneData>()) {
        d.faction = f;
    }
    base::set_hue(w, id, f.map_or(0, |f| FACTION_DEFS[f].hue_join));
    assign_name(w, id, f.map(|f| FACTION_DEFS[f].signup_name));
}

pub fn new_faction_stone(w: &mut World, f: Option<usize>) -> ItemId {
    let id = new_controller(w, &FACTION_STONE, 0xEDC);
    ops::set_movable(w, id, false);
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(StoneData::default());
    }
    faction_stone_set_faction(w, id, f);
    id
}

pub fn new_join_stone(w: &mut World, f: Option<usize>) -> ItemId {
    let id = new_controller(w, &JOIN_STONE, 0xEDC);
    ops::set_movable(w, id, false);
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(StoneData::default());
    }
    join_stone_set_faction(w, id, f);
    id
}

pub fn new_town_stone(w: &mut World, t: Option<usize>) -> ItemId {
    let id = new_controller(w, &TOWN_STONE, 0xEDE);
    ops::set_movable(w, id, false);
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(TownStoneData::default());
    }
    town_stone_set_town(w, id, t);
    id
}

pub fn town_stone_set_town(w: &mut World, id: ItemId, t: Option<usize>) {
    if let Some(d) = w
        .item_mut(id)
        .and_then(|i| i.ext.get_mut::<TownStoneData>())
    {
        d.town = t;
    }
    assign_name(w, id, t.map(|t| TOWN_DEFS[t].town_stone_name));
}

pub fn town_stone_town(w: &World, id: ItemId) -> Option<usize> {
    w.item(id)?.ext.get::<TownStoneData>().and_then(|d| d.town)
}

fn faction_stone_on_double_click(w: &mut World, id: ItemId, from: MobileId) {
    let Some(f) = stone_faction(w, id) else {
        return;
    };
    if !reach_check(w, id, from) {
        return;
    }
    if faction_gump_exists(w, from) {
        mm::send_localized_message(w, from, 1042160, "", 0x3B2); // You already have a faction menu open.
    } else if muo_content::player_mobile::data(w, from).is_some() {
        let existing = find(w, from);
        let gm = w
            .mobile(from)
            .is_some_and(|m| m.access_level >= AccessLevel::GameMaster);
        if existing == Some(f) || gm {
            let leaving = super::player_state(w, from).is_some_and(|(_, p)| p.is_leaving());
            if leaving {
                // You cannot use the faction stone until you have finished quitting your current faction
                mm::send_localized_message(w, from, 1005051, "", 0x3B2);
            } else {
                super::gumps::send_faction_stone_gump(w, from, f);
            }
        } else if existing.is_some() {
            mm::send_localized_message(w, from, 1005053, "", 0x3B2); // This is not your faction stone!
        } else {
            super::gumps::send_join_stone_gump(w, from, f);
        }
    }
}

fn join_stone_on_double_click(w: &mut World, id: ItemId, from: MobileId) {
    let Some(f) = stone_faction(w, id) else {
        return;
    };
    if !reach_check(w, id, from) {
        return;
    }
    if faction_gump_exists(w, from) {
        mm::send_localized_message(w, from, 1042160, "", 0x3B2); // You already have a faction menu open.
    } else if find(w, from).is_none() && muo_content::player_mobile::data(w, from).is_some() {
        super::gumps::send_join_stone_gump(w, from, f);
    }
}

fn town_stone_on_double_click(w: &mut World, id: ItemId, from: MobileId) {
    let Some(t) = town_stone_town(w, id) else {
        return;
    };
    let faction = find(w, from);
    let gm = w
        .mobile(from)
        .is_some_and(|m| m.access_level >= AccessLevel::GameMaster);
    if faction.is_none() && !gm {
        return; // TODO: Message?
    }
    let owner = town::owner(w, t);
    let Some(owner) = owner.filter(|_| gm || faction == owner) else {
        mm::send_localized_message(w, from, 1010332, "", 0x3B2); // Your faction does not control this town
        return;
    };
    if !super::is_commander(w, owner, from) {
        mm::send_localized_message(w, from, 1005242, "", 0x3B2); // Only faction Leaders can use townstones
    } else if faction_gump_exists(w, from) {
        mm::send_localized_message(w, from, 1042160, "", 0x3B2); // You already have a faction menu open.
    } else if muo_content::player_mobile::data(w, from).is_some() {
        super::gumps::send_town_stone_gump(w, from, owner, t);
    }
}

fn stone_serialize(w: &World, id: ItemId, out: &mut dyn GenericWriter) {
    controller_serialize(w, id, out);
    out.write_i32(0); // version
    write_faction_ref(out, stone_faction(w, id));
}

fn town_stone_serialize(w: &World, id: ItemId, out: &mut dyn GenericWriter) {
    controller_serialize(w, id, out);
    out.write_i32(0); // version
    write_town_ref(out, town_stone_town(w, id));
}

pub static FACTION_STONE: ItemType = ItemType {
    full_name: "Server.Factions.FactionStone",
    parent: Some(&BASE_SYSTEM_CONTROLLER),
    construct: Some(|w| new_faction_stone(w, None)),
    default_name: |w, id| {
        crate::template_keys::item_name(w, id, BASE_SYSTEM_CONTROLLER.default_name)
    },
    on_double_click: faction_stone_on_double_click,
    serialize: stone_serialize,
    deserialize: |w, id, r| {
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(Controller::default());
            i.ext.insert(StoneData::default());
        }
        controller_deserialize(w, id, r)?;
        if r.read_i32()? == 0 {
            let f = read_faction_ref(r)?;
            faction_stone_set_faction(w, id, f);
        }
        Ok(())
    },
    ..BASE_SYSTEM_CONTROLLER
};

pub static JOIN_STONE: ItemType = ItemType {
    full_name: "Server.Factions.JoinStone",
    parent: Some(&BASE_SYSTEM_CONTROLLER),
    construct: Some(|w| new_join_stone(w, None)),
    default_name: |w, id| {
        crate::template_keys::item_name(w, id, BASE_SYSTEM_CONTROLLER.default_name)
    },
    on_double_click: join_stone_on_double_click,
    serialize: stone_serialize,
    deserialize: |w, id, r| {
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(Controller::default());
            i.ext.insert(StoneData::default());
        }
        controller_deserialize(w, id, r)?;
        if r.read_i32()? == 0 {
            let f = read_faction_ref(r)?;
            join_stone_set_faction(w, id, f);
        }
        Ok(())
    },
    ..BASE_SYSTEM_CONTROLLER
};

pub static TOWN_STONE: ItemType = ItemType {
    full_name: "Server.Factions.TownStone",
    parent: Some(&BASE_SYSTEM_CONTROLLER),
    construct: Some(|w| new_town_stone(w, None)),
    default_name: |w, id| {
        crate::template_keys::item_name(w, id, BASE_SYSTEM_CONTROLLER.default_name)
    },
    on_double_click: town_stone_on_double_click,
    serialize: town_stone_serialize,
    deserialize: |w, id, r| {
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(Controller::default());
            i.ext.insert(TownStoneData::default());
        }
        controller_deserialize(w, id, r)?;
        if r.read_i32()? == 0 {
            let t = read_town_ref(r)?;
            town_stone_set_town(w, id, t);
        }
        Ok(())
    },
    ..BASE_SYSTEM_CONTROLLER
};

pub fn register(types: &mut TypeRegistry) {
    for t in [
        &SIGIL,
        &STRONGHOLD_MONOLITH,
        &TOWN_MONOLITH,
        &FACTION_STONE,
        &JOIN_STONE,
        &TOWN_STONE,
    ] {
        types.register_item(t, &[]);
    }
}
