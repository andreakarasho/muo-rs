//! Stealing: the skill, its `StealingTarget` and the `StolenItem` queue.
//!
//! `StealableArtifacts` and the `Sigil` branch (Factions) are [`SkillItemFns`].

use std::collections::VecDeque;

use muo_primitives::{DateTime, TimeSpan, utility};
use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{LootType, TotalType, base as ib, ops as iops};
use muo_server::layer::Layer;
use muo_server::lr_reason::LRReason;
use muo_server::mobile::part2::{enumerators, items as mitems, messages as mm};
use muo_server::mobile::{AccessLevel, MobileDelta, base as mb};
use muo_server::skills::{self, SkillName};
use muo_server::target::{TargetCancelType, Targeted};

use super::common::{
    TargetOptions, check_target_skill, in_safe_zone, invulnerable_vendor, mobile_is, msg,
    set_target,
};
use super::hooks::ops;
use super::skills_info::set_callback;
use crate::core;
use crate::creature;
use crate::items::cont;
use crate::items::corpse;
use crate::misc::titles;
use crate::mx;
use crate::player_mobile::{self, NpcGuild};
use muo_server::item::container::CONTAINER;

/// `Stealing.ClassicMode`, `SuspendOnMurder`, `MaxWeightToSteal` and `CanStealContainers`.
#[derive(Copy, Clone, Debug)]
pub struct StealingSettings {
    pub classic_mode: bool,
    pub suspend_on_murder: bool,
    pub max_weight_to_steal: i32,
    pub can_steal_containers: bool,
}

impl Default for StealingSettings {
    fn default() -> Self {
        Self {
            classic_mode: true,
            suspend_on_murder: true,
            max_weight_to_steal: 10,
            can_steal_containers: true,
        }
    }
}

fn settings(w: &World) -> StealingSettings {
    w.resources
        .get::<StealingSettings>()
        .copied()
        .unwrap_or_default()
}

/// `Stealing.ClassicMode`.
pub fn classic_mode(w: &World) -> bool {
    settings(w).classic_mode
}

/// `Stealing.Configure()`.
pub fn configure(w: &mut World) {
    let aos = core::aos(w);
    let s = StealingSettings {
        classic_mode: w.config.get_setting("stealing.classicMode", !aos),
        suspend_on_murder: w.config.get_setting("stealing.suspendOnMurder", !aos),
        can_steal_containers: w.config.get_setting("stealing.canStealContainers", !aos),
        max_weight_to_steal: w.config.get_setting("stealing.maxWeightToSteal", 10),
    };
    *w.resources.get_mut::<StealingSettings>() = s;
}

pub fn is_in_guild(w: &World, m: MobileId) -> bool {
    player_mobile::data(w, m).is_some_and(|d| d.npc_guild == NpcGuild::ThievesGuild)
}

pub fn is_innocent_to(w: &mut World, from: MobileId, to: MobileId) -> bool {
    muo_server::notoriety::compute(w, from, to) == muo_server::notoriety::INNOCENT
}

pub fn is_empty_handed(w: &World, from: MobileId) -> bool {
    mitems::find_item_on_layer(w, from, Layer::OneHanded).is_none()
        && mitems::find_item_on_layer(w, from, Layer::TwoHanded).is_none()
}

fn is_in_locked_container(w: &World, item: ItemId) -> bool {
    match w.item(item).and_then(|i| i.parent) {
        Some(EntityId::Item(p)) => (cont::class(w, p).locked)(w, p) || is_in_locked_container(w, p),
        _ => false,
    }
}

pub fn on_use(w: &mut World, m: MobileId) -> TimeSpan {
    if !is_empty_handed(w, m) {
        msg(w, m, 1005584); // Both hands must be free to steal.
    } else if in_safe_zone(w, m) {
        mm::send_message(w, m, "You may not steal in this area.");
    } else {
        set_target(
            w,
            m,
            1,
            TargetOptions {
                allow_nonlocal: true,
                on_cancel: Some(|w: &mut World, from: MobileId, _: TargetCancelType| {
                    let t = w.tick_count;
                    if let Some(x) = w.mobile_mut(from) {
                        x.next_skill_time = t;
                    }
                }),
                ..Default::default()
            },
            move |w, from, targeted| on_target(w, from, m, targeted),
        );
        mb::revealing_action(w, m);

        msg(w, m, 502698); // Which item do you want to steal?
    }

    TimeSpan::from_secs(30)
}

fn check_blessed(w: &World, item: ItemId, by: Option<MobileId>) -> bool {
    match by {
        Some(m) => ib::check_blessed(w, item, m),
        // `m != null && m == BlessedFor` is false for a null `m`.
        None => w.item(item).is_some_and(|i| {
            i.loot_type == LootType::Blessed
                || (muo_server::item::deps::insurance_enabled(w) && i.insured())
        }),
    }
}

/// `_thief.Backpack?.CheckHold(_thief, item, false, true) == true`.
fn pack_can_hold(w: &mut World, thief: MobileId, item: ItemId) -> bool {
    match w.mobile(thief).and_then(|m| m.backpack) {
        Some(pack) => {
            muo_server::item::container::check_hold_items(w, pack, Some(thief), item, false, true)
        }
        None => false,
    }
}

/// `StealingTarget.TryStealItem(toSteal, ref caught)`.
fn try_steal_item(
    w: &mut World,
    thief: MobileId,
    to_steal: ItemId,
    caught: &mut bool,
) -> Option<ItemId> {
    let mut stolen = None;

    let root = iops::root_parent(w, to_steal);
    let mob_root = match root {
        Some(EntityId::Mobile(m)) => Some(m),
        _ => None,
    };
    let root_is_player = mob_root.is_some_and(|m| w.mobile(m).is_some_and(|x| x.player));

    let (parent_none, movable) = w
        .item(to_steal)
        .map_or((true, false), |i| (i.parent.is_none(), i.movable()));
    let si = (parent_none || !movable) && ops(w).stealable_instance(w, to_steal);

    let thief_young = player_mobile::young(w, thief);
    let thief_kills = w.mobile(thief).map_or(0, |m| m.kills);
    let thief_access = w
        .mobile(thief)
        .map_or(AccessLevel::Player, |m| m.access_level);
    let mob_root_is_creature = mob_root.is_some_and(|m| creature::is_creature(w, m));

    if !is_empty_handed(w, thief) {
        msg(w, thief, 1005584); // Both hands must be free to steal.
    } else if in_safe_zone(w, thief) {
        mm::send_message(w, thief, "You may not steal in this area.");
    } else if thief_young && (root_is_player || mob_root_is_creature) {
        // You cannot steal from people or monsters right now.  Practice on chests and barrels.
        msg(w, thief, 502700);
    } else if root_is_player && !is_in_guild(w, thief) {
        msg(w, thief, 1005596); // You must be in the thieves guild to steal from other players.
    } else if settings(w).suspend_on_murder
        && root_is_player
        && is_in_guild(w, thief)
        && thief_kills > 0
    {
        msg(w, thief, 502706); // You are currently suspended from the thieves guild.
    } else if mob_root.is_some_and(|m| player_mobile::young(w, m)) {
        msg(w, thief, 502699); // You cannot steal from the Young.
    } else if mob_root.is_some_and(|m| invulnerable_vendor(w, m)) {
        msg(w, thief, 1005598); // You can't steal from shopkeepers.
    } else if mob_root.is_some_and(|m| mobile_is(w, m, "Server.Mobiles.PlayerVendor")) {
        msg(w, thief, 502709); // You can't steal from vendors.
    } else if !mb::can_see_item(w, thief, to_steal) {
        msg(w, thief, 500237); // Target can not be seen.
    } else if ops(w).is_sigil(w, to_steal) {
        let loc = iops::get_world_location(w, to_steal);
        if !mx::in_range_point(w, thief, loc, 1) {
            msg(w, thief, 502703); // You must be standing next to an item to steal it.
        } else if root.is_some() {
            // not on the ground
            msg(w, thief, 502710); // You can't steal that!
        } else if let Some(sig) = ops(w).steal_sigil(w, thief, to_steal) {
            return Some(sig);
        }
    } else if !pack_can_hold(w, thief, to_steal) {
        msg(w, thief, 1048147); // Your backpack can't hold anything else.
    } else if !si && (parent_none || !movable)
        || w.item(to_steal)
            .is_some_and(|i| i.loot_type == LootType::Newbied)
        || check_blessed(w, to_steal, mob_root)
        || !settings(w).can_steal_containers
            && !si
            && w.item(to_steal).is_some_and(|i| i.ty.is(&CONTAINER))
    {
        msg(w, thief, 502710); // You can't steal that!
    } else if !mx::in_range_point(w, thief, iops::get_world_location(w, to_steal), 1) {
        msg(w, thief, 502703); // You must be standing next to an item to steal it.
    } else if si && skills::value(w, thief, SkillName::Stealing) < 100.0 {
        // You're not skilled enough to attempt the theft of this item.
        mm::send_localized_message(w, thief, 1060025, "", 0x66D);
    } else if matches!(
        w.item(to_steal).and_then(|i| i.parent),
        Some(EntityId::Mobile(_))
    ) {
        msg(w, thief, 1005585); // You cannot steal items which are equipped.
    } else if root == Some(EntityId::Mobile(thief)) {
        msg(w, thief, 502704); // You catch yourself red-handed.
    } else if mob_root.is_some_and(|m| {
        w.mobile(m)
            .is_some_and(|x| x.access_level > AccessLevel::Player)
    }) {
        msg(w, thief, 502710); // You can't steal that!
    } else if mob_root.is_some_and(|m| !mb::can_be_harmful(w, thief, m)) {
        // (no message here)
    } else if matches!(root, Some(EntityId::Item(r)) if corpse::is_corpse(w, r)) {
        msg(w, thief, 502710); // You can't steal that!
    } else if thief_access < AccessLevel::GameMaster && is_in_locked_container(w, to_steal) {
        msg(w, thief, 501747); // It appears to be locked.
    } else {
        let wt =
            iops::weight(w, to_steal) + f64::from(ib::get_total(w, to_steal, TotalType::Weight));
        let mut reject = LRReason::Inspecific;

        let non_deco_container = matches!(root, Some(EntityId::Item(r))
            if w.item(r).is_some_and(|i| i.ty.is(&CONTAINER) && !(i.ty.is_deco_container)(w, r)));
        if wt > f64::from(settings(w).max_weight_to_steal) {
            // This item is too heavy to steal from someone's backpack.
            msg(w, thief, 502722);
        } else if non_deco_container && !ib::check_lift(w, to_steal, thief, to_steal, &mut reject) {
            msg(w, thief, 502710); // You can't steal that!
        } else {
            let (stackable, amount) = w
                .item(to_steal)
                .map_or((false, 0), |i| (i.stackable(), i.amount));
            let weight = iops::weight(w, to_steal);
            let target = Targeted::Entity(EntityId::Item(to_steal));
            let stealing = skills::value(w, thief, SkillName::Stealing);

            if stackable && amount > 1 {
                let max_amount = ((stealing / 10.0 / weight) as i32).clamp(1, amount);

                let amt = utility::random_min_max(&mut w.rng, 1, max_amount);

                if amt >= amount {
                    let pile_weight = (weight * f64::from(amount)).ceil() as i32 * 10;

                    if check_target_skill(
                        w,
                        thief,
                        SkillName::Stealing,
                        target,
                        f64::from(pile_weight) - 22.5,
                        f64::from(pile_weight) + 27.5,
                    ) {
                        stolen = Some(to_steal);
                    }
                } else {
                    let pile_weight = (weight * f64::from(amt)).ceil() as i32 * 10;

                    if check_target_skill(
                        w,
                        thief,
                        SkillName::Stealing,
                        target,
                        f64::from(pile_weight) - 22.5,
                        f64::from(pile_weight) + 27.5,
                    ) {
                        stolen = Some(
                            mitems::lift_item_dupe(w, to_steal, amount - amt).unwrap_or(to_steal),
                        );
                    }
                }
            } else {
                let iw = wt.ceil() as i32 * 10;

                if check_target_skill(
                    w,
                    thief,
                    SkillName::Stealing,
                    target,
                    f64::from(iw) - 22.5,
                    f64::from(iw) + 27.5,
                ) {
                    stolen = Some(to_steal);
                }
            }

            if stolen.is_some() {
                msg(w, thief, 502724); // You successfully steal the item.

                if si {
                    iops::set_movable(w, to_steal, true);
                    ops(w).clear_stealable_instance(w, to_steal);
                }
            } else {
                msg(w, thief, 502723); // You fail to steal the item.
            }

            let stealing = skills::value(w, thief, SkillName::Stealing);
            *caught = stealing < f64::from(utility::random(&mut w.rng, 150));
        }
    }

    stolen
}

fn on_target(w: &mut World, from: MobileId, thief: MobileId, target: Targeted) {
    mb::revealing_action(w, from);

    let mut stolen: Option<ItemId> = None;
    let mut root: Option<EntityId> = None;
    let mut caught = false;

    match target {
        Targeted::Entity(EntityId::Item(item)) => {
            root = iops::root_parent(w, item);
            stolen = try_steal_item(w, thief, item, &mut caught);
        }
        Targeted::Entity(EntityId::Mobile(mobile)) => {
            let pack = w.mobile(mobile).and_then(|m| m.backpack);
            let items = pack
                .and_then(|p| w.item(p))
                .map(|p| p.items.clone())
                .unwrap_or_default();
            if !items.is_empty() {
                root = Some(EntityId::Mobile(mobile));
                let pick = *utility::random_element(&mut w.rng, &items).expect("not empty");
                stolen = try_steal_item(w, thief, pick, &mut caught);
            }
        }
        _ => msg(w, thief, 502710), // You can't steal that!
    }

    titles::award_karma(w, from, -50, true);

    let mob_root = match root {
        Some(EntityId::Mobile(m)) => Some(m),
        _ => None,
    };

    if let Some(stolen) = stolen {
        if matches!(root, Some(EntityId::Item(r)) if w.item(r).is_some_and(|i| i.ty.is(&CONTAINER)))
        {
            ib::on_item_lifted(w, stolen, from, stolen);
        }

        mitems::add_to_backpack(w, from, stolen);

        let is_container = w.item(stolen).is_some_and(|i| i.ty.is(&CONTAINER));
        let stackable = w.item(stolen).is_some_and(|i| i.stackable());
        if !(is_container || stackable) {
            stolen_add(w, stolen, thief, mob_root);
        }
    }

    let corpse_root = match root {
        Some(EntityId::Item(r)) if corpse::is_corpse(w, r) => Some(r),
        _ => None,
    };
    let criminal_corpse = corpse_root.is_some_and(|c| corpse::is_criminal_action(w, c, thief));

    if caught {
        if root.is_none() || criminal_corpse {
            mb::criminal_action(w, thief, false);
        } else if let Some(mob_root) = mob_root {
            if !is_in_guild(w, mob_root) && is_innocent_to(w, thief, mob_root) {
                mb::criminal_action(w, thief, false);
            }

            let name = |w: &World, m: MobileId| {
                muo_server::mobile::part1::props_b::name(w, m).unwrap_or_default()
            };
            let message = format!(
                "You notice {} trying to steal from {}.",
                name(w, thief),
                name(w, mob_root)
            );

            for ns in enumerators::get_clients_in_range(w, thief, 8) {
                let m = w.net_states.get(&ns).and_then(|n| n.mobile);
                if let Some(m) = m.filter(|&m| m != thief) {
                    mm::send_message(w, m, &message);
                }
            }
        }
    } else if criminal_corpse {
        mb::criminal_action(w, thief, false);
    }

    if let Some(mob_root) = mob_root
        && w.mobile(mob_root).is_some_and(|m| m.player)
        && player_mobile::data(w, thief).is_some()
        && is_innocent_to(w, thief, mob_root)
        && !is_in_guild(w, mob_root)
    {
        if let Some(d) = player_mobile::data_mut(w, thief) {
            d.perma_flags.push(mob_root);
        }
        mb::delta(w, thief, MobileDelta::NOTO);
    }

    const TARGETER_COOLDOWN: i64 = 30000; // 30s
    const SKILL_COOLDOWN: i64 = 10000; // 10s

    // Calculate how much time has passed since the targeter was opened
    let tick = w.tick_count;
    let next = w.mobile(from).map_or(0, |m| m.next_skill_time);
    let ticks_since_targeter = (tick - (next - TARGETER_COOLDOWN)) as i32;
    let remaining = (SKILL_COOLDOWN as i32 - ticks_since_targeter).max(0);
    if let Some(m) = w.mobile_mut(from) {
        m.next_skill_time = tick + i64::from(remaining);
    }
}

// ---- StolenItem -------------------------------------------------------------------------------------------------

/// `StolenItem.StealTime`.
pub const STEAL_TIME: TimeSpan = TimeSpan::from_mins(2);

struct StolenItem {
    stolen: ItemId,
    victim: Option<MobileId>,
    expires: DateTime,
}

/// `StolenItem._queue`.
#[derive(Default)]
pub struct StolenItems(VecDeque<StolenItem>);

fn queue(w: &mut World) -> &mut VecDeque<StolenItem> {
    &mut w.resources.get_mut::<StolenItems>().0
}

/// `StolenItem.Add(item, thief, victim)`.
pub fn stolen_add(w: &mut World, item: ItemId, _thief: MobileId, victim: Option<MobileId>) {
    stolen_clean(w);

    let expires = w.now + STEAL_TIME;
    queue(w).push_back(StolenItem {
        stolen: item,
        victim,
        expires,
    });
}

/// `StolenItem.IsStolen(item, ref victim)`: the victim when it is (inside, `None` for a victimless theft).
pub fn is_stolen(w: &mut World, item: ItemId) -> Option<Option<MobileId>> {
    stolen_clean(w);

    let now = w.now;
    queue(w)
        .iter()
        .find(|si| si.stolen == item && now < si.expires)
        .map(|si| si.victim)
}

/// `StolenItem.ReturnOnDeath(killed)`.
pub fn return_on_death(w: &mut World, killed: MobileId) {
    stolen_clean(w);

    let corpse = w.mobile(killed).and_then(|m| m.corpse);

    let n = queue(w).len();
    for i in 0..n {
        let now = w.now;
        let (stolen, victim, expired) = {
            let si = &queue(w)[i];
            (si.stolen, si.victim, now >= si.expires)
        };
        let Some(victim) = victim else { continue };
        if expired || iops::root_parent(w, stolen) != corpse.map(EntityId::Item) {
            continue;
        }

        if mitems::add_to_backpack(w, victim, stolen) {
            msg(w, victim, 1010464); // the item that was stolen is returned to you.
        } else {
            msg(w, victim, 1010463); // the item that was stolen from you falls to the ground.
        }

        queue(w)[i].expires = now; // such a hack
    }
}

/// `StolenItem.Clean()`.
pub fn stolen_clean(w: &mut World) {
    let now = w.now;
    let q = queue(w);
    while q.front().is_some_and(|si| now >= si.expires) {
        q.pop_front();
    }
}

pub fn initialize(w: &mut World) {
    set_callback(w, SkillName::Stealing, on_use);
    player_mobile::subscribe_death(w, return_on_death);
}

/// `StolenItem.IsStolen(item)` for a caller that can't clean the queue first (an expired entry is skipped).
pub fn is_stolen_ref(w: &World, item: ItemId) -> bool {
    w.resources
        .get::<StolenItems>()
        .is_some_and(|q| q.0.iter().any(|si| si.stolen == item && w.now < si.expires))
}
