//! `BaseHire`: a hireable fighter paid by the day. The classes are tables in `humans.rs`.

use muo_content::creature::control::{control_master, set_control_master};
use muo_content::creature::setters::new_creature;
use muo_content::creature::{self, BASE_CREATURE, BASE_CREATURE_C, CreatureType};
use muo_content::items::gold::GOLD;
use muo_content::mx;
use muo_primitives::{DateTime, TimeSpan};
use muo_server::context_menu_entry::ContextMenuEntry;
use muo_server::events::SpeechEventArgs;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::container;
use muo_server::mobile::part2::{messages, speech};
use muo_server::mobile::{MobileDelta, MobileType};
use muo_server::skills::{self, SkillName};
use muo_server::timer::{self, TimerHandle};
use muo_server::world::World;

use crate::common::*;

/// What `BaseHire` adds to a `BaseCreature`.
pub struct HireData {
    /// `_nextPay`.
    pub next_pay: DateTime,
    /// `_holdGold`.
    pub hold_gold: i32,
    /// `_isHired`.
    pub is_hired: bool,
    /// `GoldOnDeath` (not serialized).
    pub gold_on_death: i32,
}

pub fn data(w: &World, id: MobileId) -> Option<&HireData> {
    w.mobile(id)?.ext.get::<HireData>()
}

fn data_mut(w: &mut World, id: MobileId) -> Option<&mut HireData> {
    w.mobile_mut(id)?.ext.get_mut::<HireData>()
}

/// `PayTimer`: one timer for every hired hireling.
#[derive(Default)]
struct PayTimer {
    hires: Vec<MobileId>,
    timer: TimerHandle,
}

/// `new BaseHire(...)` for class `ty`: `BaseCreature(ai, FightMode.Aggressor)` and `ControlSlots = 2` (the
/// `BaseHire(AIType)` constructor's `HoldGold = 8` is the template key `npcHoldGold`).
pub fn new_hire(w: &mut World, ty: &'static MobileType, ai: AIType) -> MobileId {
    let id = new_creature(w, ty, ai, FightMode::Aggressor, DEFAULT_RANGE_PERCEPTION, 1);
    if let Some(m) = w.mobile_mut(id) {
        m.ext.insert(HireData {
            next_pay: DateTime::MIN,
            hold_gold: 0,
            is_hired: false,
            gold_on_death: 0,
        });
    }
    control_slots(w, id, 2);
    id
}

/// `HoldGold = v`.
pub fn set_hold_gold(w: &mut World, id: MobileId, v: i32) {
    if let Some(d) = data_mut(w, id) {
        d.hold_gold = v;
    }
}

/// `new T()` of a hireling class whose constructor is its template: `BaseHire(ai)` with the template's `"ai"`,
/// then the template (its `npcHoldGold` is the `BaseHire(AIType)` constructor's).
pub fn new_template_hire(w: &mut World, ty: &'static MobileType) -> MobileId {
    let t = muo_content::templates::table(w);
    let ai = t.roll_args(w, ty.full_name).ai;
    let id = new_hire(w, ty, ai);
    muo_content::templates::apply_mobile(w, id);
    id
}

/// `IsHired` setter: a change refreshes the notoriety and the properties.
pub fn set_is_hired(w: &mut World, id: MobileId, v: bool) {
    if data(w, id).is_some_and(|d| d.is_hired) == v {
        return;
    }
    if let Some(d) = data_mut(w, id) {
        d.is_hired = v;
    }
    if let Some(t) = w.mobile(id).map(|m| m.ty) {
        (t.delta)(w, id, MobileDelta::NOTO);
    }
    muo_server::mobile::part2::properties::invalidate_properties(w, id);
}

/// `Pay` / `PerDayCost()`.
pub fn per_day_cost(w: &mut World, id: MobileId) -> i32 {
    let mut sum = 0.0;
    for s in [
        SkillName::Anatomy,
        SkillName::Tactics,
        SkillName::Macing,
        SkillName::Swords,
        SkillName::Fencing,
        SkillName::Archery,
        SkillName::MagicResist,
        SkillName::Healing,
        SkillName::Magery,
        SkillName::Parry,
    ] {
        sum += skills::value(w, id, s);
    }
    (sum as i32) / 35 + 1
}

/// `GetOwner()`.
pub fn get_owner(w: &mut World, id: MobileId) -> Option<MobileId> {
    if !creature::read(w, id, false, |d| d.controlled) {
        return None;
    }
    let owner = control_master(w, id);
    set_is_hired(w, id, true);
    let owner = owner?;
    if mx::deleted(w, owner) {
        messages::say_number(w, id, 1005653, ""); // Hmmm. I seem to have lost my master.
        set_control_master(w, id, None);
        return None;
    }
    Some(owner)
}

/// `AddHire(m)`.
fn add_hire(w: &mut World, id: MobileId, m: MobileId) -> bool {
    if let Some(owner) = get_owner(w, id) {
        let name = w
            .mobile(owner)
            .and_then(|o| (o.ty.name)(w, owner))
            .unwrap_or_default();
        mx::msg_args(w, m, 1043283, &name); // I am following ~1_NAME~.
        return false;
    }
    if set_control_master(w, id, Some(m)) {
        set_is_hired(w, id, true);
        return true;
    }
    false
}

/// `SayHireCost()`.
fn say_hire_cost(w: &mut World, id: MobileId) {
    let pay = per_day_cost(w, id);
    messages::say_number(w, id, 1043256, &pay.to_string());
}

/// `OnHireDragDrop(from, item)`.
fn on_hire_drag_drop(w: &mut World, id: MobileId, from: MobileId, item: ItemId) -> bool {
    let say_to = |w: &mut World, n: i32| speech::say_to_number(w, id, from, n, "");
    let pay = per_day_cost(w, id);
    if pay == 0 {
        say_to(w, 500200); // I have no need for that.
        return false;
    }
    // Is the creature already hired
    if creature::read(w, id, false, |d| d.controlled) {
        say_to(w, 1042495); // I have already been hired.
        return false;
    }
    // Is the item the payment in gold
    if !w.item(item).is_some_and(|i| i.ty.is(&GOLD)) {
        say_to(w, 1043268); // Tis crass of me, but I want gold
        return false;
    }
    // Is the payment in gold sufficient
    let amount = w.item(item).map_or(0, |i| i.amount);
    if amount < pay {
        say_hire_cost(w, id);
        return false;
    }
    let slots = creature::read(w, id, 0, |d| d.control_slots);
    let (followers, max) = w
        .mobile(from)
        .map_or((0, 0), |m| (m.followers, m.followers_max));
    if followers + slots > max {
        say_to(w, 500896); // I see you already have an escort.
        return false;
    }
    // Try to add the hireling as a follower
    if !add_hire(w, id, from) {
        return false;
    }
    // I thank thee for paying me.  I will work for thee for ~1_NUMBER~ days.
    speech::say_to_number(w, id, from, 1043258, &(amount / pay).to_string());

    let now = w.now;
    if let Some(d) = data_mut(w, id) {
        d.hold_gold += amount;
        d.next_pay = now + TimeSpan::from_mins(30);
    }
    register_timer(w, id);
    true
}

fn on_drag_drop(w: &mut World, id: MobileId, from: MobileId, item: ItemId) -> bool {
    on_hire_drag_drop(w, id, from, item) || (BASE_CREATURE.on_drag_drop)(w, id, from, item)
}

fn on_speech(w: &mut World, id: MobileId, e: &mut SpeechEventArgs) {
    // Check for a greeting, a 'hire', or a 'servant'
    if !e.handled
        && mx::in_range(w, e.mobile, id, 6)
        && (e.has_keyword(0x003B) || e.has_keyword(0x0162) || e.has_keyword(0x000C))
    {
        if creature::read(w, id, false, |d| d.controlled) {
            messages::say_number(w, id, 1042495, ""); // I have already been hired.
        } else {
            e.handled = true;
            say_hire_cost(w, id);
        }
    }
    (BASE_CREATURE.on_speech)(w, id, e);
}

fn get_context_menu_entries(
    w: &mut World,
    id: MobileId,
    from: MobileId,
    list: &mut Vec<ContextMenuEntry>,
) {
    if mx::deleted(w, id) {
        return;
    }
    if !creature::read(w, id, false, |d| d.controlled) {
        let mut e = ContextMenuEntry::new(6120, 3);
        e.on_click = Some(std::rc::Rc::new(|w: &mut World, _from, target| {
            if let EntityId::Mobile(h) = target
                && data(w, h).is_some()
            {
                say_hire_cost(w, h);
            }
        }));
        list.push(e);
    } else {
        (BASE_CREATURE.get_context_menu_entries)(w, id, from, list);
    }
}

fn on_before_death(w: &mut World, id: MobileId) -> bool {
    let gold = w
        .mobile(id)
        .and_then(|m| m.backpack)
        .map_or(0, |p| container::get_amount(w, p, &GOLD, true));
    if let Some(d) = data_mut(w, id) {
        d.gold_on_death = gold;
    }
    (BASE_CREATURE.on_before_death)(w, id)
}

fn on_death(w: &mut World, id: MobileId, c: Option<ItemId>) {
    let gold = data(w, id).map_or(0, |d| d.gold_on_death);
    if gold > 0
        && let (Some(c), Some(g)) = (c, muo_content::factory::make_item(w, "Server.Items.Gold"))
    {
        muo_server::item::ops::set_amount(w, g, gold);
        container::drop_item_default(w, c, g);
    }
    (BASE_CREATURE.on_death)(w, id, c);
}

fn delete(w: &mut World, id: MobileId) {
    (BASE_CREATURE.delete)(w, id);
    remove_timer(w, id);
}

// ---- PayTimer ----

/// `PayTimer.RegisterTimer(hire)`.
fn register_timer(w: &mut World, id: MobileId) {
    let running = w.resources.get_mut::<PayTimer>().timer.running(w);
    if !running {
        let h = timer::start(
            w,
            TimeSpan::from_mins(1),
            TimeSpan::from_mins(1),
            0,
            pay_tick,
        );
        w.resources.get_mut::<PayTimer>().timer = h;
    }
    let t = w.resources.get_mut::<PayTimer>();
    if !t.hires.contains(&id) {
        t.hires.push(id);
    }
}

/// `PayTimer.RemoveTimer(hire)`.
fn remove_timer(w: &mut World, id: MobileId) {
    let t = w.resources.get_mut::<PayTimer>();
    let before = t.hires.len();
    t.hires.retain(|&h| h != id);
    if before != t.hires.len() && t.hires.is_empty() {
        let mut h = std::mem::take(&mut t.timer);
        h.cancel(w);
    }
}

/// `PayTimer.OnTick`.
fn pay_tick(w: &mut World) {
    let hires = w.resources.get_mut::<PayTimer>().hires.clone();
    let mut queue = Vec::new();
    for hire in hires {
        let next = data(w, hire).map_or(DateTime::MIN, |d| d.next_pay);
        if next > w.now {
            continue;
        }
        let now = w.now;
        if let Some(d) = data_mut(w, hire) {
            d.next_pay = now + TimeSpan::from_mins(30);
        }
        let pay = per_day_cost(w, hire);
        let hold = data(w, hire).map_or(0, |d| d.hold_gold);
        if hold <= pay {
            queue.push(hire);
        } else if let Some(d) = data_mut(w, hire) {
            d.hold_gold -= pay;
        }
    }
    for hire in queue {
        get_owner(w, hire); // Sets owner to null
        messages::say_number(w, hire, 503235, ""); // I regret nothing!
        if let Some(t) = w.mobile(hire).map(|m| m.ty) {
            (t.delete)(w, hire);
        }
    }
}

// ---- types ----

pub static BASE_HIRE_C: CreatureType = CreatureType {
    is_bondable: |_, _| false,
    can_drop: |_, _| false,
    ..BASE_CREATURE_C
};

/// `Server.Mobiles.BaseHire`.
pub static BASE_HIRE: MobileType = MobileType {
    full_name: "Server.Mobiles.BaseHire",
    parent: Some(&BASE_CREATURE),
    construct: None,
    family: Some(&BASE_HIRE_C),
    keeps_items_on_death: |_, _| true,
    on_drag_drop,
    on_speech,
    get_context_menu_entries,
    on_before_death,
    on_death,
    delete,
    serialize: |w, id, out| {
        (BASE_CREATURE.serialize)(w, id, out);
        out.write_encoded_int(0);
        let d = data(w, id);
        out.write_date_time_ticks(d.map_or(DateTime::MIN, |d| d.next_pay).ticks());
        out.write_bool(d.is_some_and(|d| d.is_hired));
        out.write_i32(d.map_or(0, |d| d.hold_gold));
    },
    deserialize: |w, id, r| {
        (BASE_CREATURE.deserialize)(w, id, r)?;
        let _version = r.read_encoded_int()?;
        let next_pay = DateTime::try_from_ticks(r.read_date_time_ticks()?).unwrap_or(DateTime::MIN);
        let is_hired = r.read_bool()?;
        let hold_gold = r.read_i32()?;
        if let Some(m) = w.mobile_mut(id) {
            m.ext.insert(HireData {
                next_pay,
                hold_gold,
                is_hired,
                gold_on_death: 0,
            });
        }
        if is_hired {
            register_timer(w, id);
        }
        Ok(())
    },
    ..BASE_CREATURE
};
