//! A timer-wheel decay scheduler with O(1) registration.
//!
//! Coarse buckets of items by time of decay, plus a priority queue for the items due within the current window.
//! An item that becomes decay eligible must be passed to `ops::update_decay_registration` once its parent, map and
//! location are final. The scheduler drops items that stop being eligible, but nothing enrolls one that becomes
//! eligible while untracked. `Decays` and `DecayTime` overrides must be stable rather than recomputed per read.
//!
//! The statics are [`DecayScheduler`] in the world's resources. Removal from a `PriorityQueue` is O(n); here a
//! removed entry stays in the heap and is skipped when it surfaces (each live entry has a generation).

use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};

use indexmap::IndexSet;
use muo_primitives::{DateTime, TimeSpan, utility};

use crate::ids::ItemId;
use crate::item::DECAY_SLOT_NONE;
use crate::timer::{self, TimerHandle};
use crate::world::World;

/// `Item.DecaySlot` values; any non-negative value is a bucket index.
pub const SLOT_NONE: i8 = DECAY_SLOT_NONE;
pub const SLOT_OVERFLOW: i8 = -2;
pub const SLOT_ACTIVE: i8 = -3;

const BUCKET_COUNT: usize = 12;

/// The `decay.*` settings of `ServerConfiguration`, with their defaults.
#[derive(Copy, Clone, Debug)]
pub struct DecayConfig {
    /// `decay.maxItemsPerTick`.
    pub max_items_per_tick: i32,
    /// `decay.tickInterval`.
    pub tick_interval: TimeSpan,
    /// `decay.bucketInterval`.
    pub bucket_interval: TimeSpan,
    /// `decay.jitterMaxMs`.
    pub jitter_max_milliseconds: i32,
}

impl Default for DecayConfig {
    fn default() -> Self {
        DecayConfig {
            max_items_per_tick: 250,
            tick_interval: TimeSpan::from_millis(256),
            bucket_interval: TimeSpan::from_mins(5),
            jitter_max_milliseconds: 25,
        }
    }
}

/// The statics of `DecayScheduler`.
pub struct DecayScheduler {
    config: DecayConfig,
    configured: bool,
    /// Timer wheel buckets.
    buckets: Vec<IndexSet<ItemId>>,
    current_bucket_index: usize,
    next_bucket_rotation: DateTime,
    /// Items with a decay beyond the total bucket span.
    overflow_bucket: IndexSet<ItemId>,
    total_bucket_span: TimeSpan,
    next_overflow_check: DateTime,
    /// Items due within the current bucket window, soonest first.
    active_queue: BinaryHeap<Reverse<(DateTime, u64, ItemId)>>,
    /// The live heap entry of each active item.
    active_gen: HashMap<ItemId, u64>,
    next_gen: u64,
    /// `Shared`: the timer, made on first use.
    timer: Option<TimerHandle>,
}

impl Default for DecayScheduler {
    fn default() -> Self {
        DecayScheduler {
            config: DecayConfig::default(),
            configured: false,
            buckets: (0..BUCKET_COUNT).map(|_| IndexSet::new()).collect(),
            current_bucket_index: 0,
            next_bucket_rotation: DateTime::MIN,
            overflow_bucket: IndexSet::new(),
            total_bucket_span: TimeSpan::ZERO,
            next_overflow_check: DateTime::MIN,
            active_queue: BinaryHeap::new(),
            active_gen: HashMap::new(),
            next_gen: 0,
            timer: None,
        }
    }
}

fn state(w: &mut World) -> &mut DecayScheduler {
    w.resources.get_mut::<DecayScheduler>()
}

/// `DecayScheduler.Configure()`: applies `config` and re-bases the schedule on `Core.Now`.
pub fn configure(w: &mut World, config: DecayConfig) {
    let now = w.now;
    let old_timer = state(w).timer.take();
    if let Some(h) = old_timer {
        timer::release(w, h);
    }
    let s = state(w);
    *s = DecayScheduler::default();
    s.config = config;
    s.configured = true;
    s.total_bucket_span =
        TimeSpan::from_ticks(config.bucket_interval.ticks() * BUCKET_COUNT as i64);
    s.next_bucket_rotation = now + config.bucket_interval;
    s.next_overflow_check = now + s.total_bucket_span;
}

/// Lazy setup: a world that never configured gets the defaults.
fn ensure_configured(w: &mut World) {
    if !state(w).configured {
        configure(w, DecayConfig::default());
    }
}

/// `ResetForTests()`: drops all tracked items and re-bases the rotation schedule on `Core.Now`.
pub fn reset_for_tests(w: &mut World) {
    ensure_configured(w);
    let now = w.now;
    if let Some(h) = state(w).timer {
        timer::stop(w, h);
    }
    let s = state(w);
    s.active_queue.clear();
    s.active_gen.clear();
    s.overflow_bucket.clear();
    for b in &mut s.buckets {
        b.clear();
    }
    s.current_bucket_index = 0;
    s.next_bucket_rotation = now + s.config.bucket_interval;
    s.next_overflow_check = now + s.total_bucket_span;
}

/// `IsRegistered(item)`: tracked in any bucket, the overflow bucket, or the active queue.
pub fn is_registered(w: &World, id: ItemId) -> bool {
    let Some(s) = w.resources.get::<DecayScheduler>() else {
        return false;
    };
    s.overflow_bucket.contains(&id)
        || s.buckets.iter().any(|b| b.contains(&id))
        || s.active_gen.contains_key(&id)
}

fn is_empty(s: &DecayScheduler) -> bool {
    s.active_gen.is_empty()
        && s.overflow_bucket.is_empty()
        && s.buckets.iter().all(|b| b.is_empty())
}

fn set_slot(w: &mut World, id: ItemId, slot: i8) {
    if let Some(i) = w.item_mut(id) {
        i.decay_slot = slot;
    }
}

fn enqueue_active(w: &mut World, id: ItemId, decay_time: DateTime) {
    let s = state(w);
    let gen_ = s.next_gen;
    s.next_gen += 1;
    s.active_queue.push(Reverse((decay_time, gen_, id)));
    s.active_gen.insert(id, gen_);
    set_slot(w, id, SLOT_ACTIVE);
}

/// `Register(item)`: call after an item becomes decay-eligible.
pub fn register(w: &mut World, id: ItemId) {
    let Some(ty) = w.item(id).filter(|i| !i.deleted()).map(|i| i.ty) else {
        return;
    };
    if !(ty.can_decay)(w, id) {
        return;
    }
    ensure_configured(w);

    // Start timer if not running
    start_timer(w);

    let decay_time = crate::item::ops::scheduled_decay_time(w, id);
    let time_until_decay = decay_time - w.now;
    let (bucket_interval, total_span, current) = {
        let s = state(w);
        (
            s.config.bucket_interval,
            s.total_bucket_span,
            s.current_bucket_index,
        )
    };

    if time_until_decay <= bucket_interval {
        // Already due or due very soon: straight to the active queue.
        enqueue_active(w, id, decay_time);
    } else if time_until_decay > total_span {
        state(w).overflow_bucket.insert(id);
        set_slot(w, id, SLOT_OVERFLOW);
    } else {
        let offset = get_bucket_offset(bucket_interval, time_until_decay);
        let absolute = (current + offset) % BUCKET_COUNT;
        state(w).buckets[absolute].insert(id);
        set_slot(w, id, absolute as i8);
    }
}

/// `Unregister(item)`: call when an item is picked up, deleted, or otherwise ineligible.
pub fn unregister(w: &mut World, id: ItemId) {
    let Some(slot) = w.item(id).map(|i| i.decay_slot) else {
        return;
    };
    if slot == SLOT_NONE {
        return;
    }
    set_slot(w, id, SLOT_NONE);

    let s = state(w);
    match slot {
        SLOT_OVERFLOW => {
            s.overflow_bucket.shift_remove(&id);
        }
        SLOT_ACTIVE => {
            s.active_gen.remove(&id);
        }
        _ => {
            if let Some(b) = s.buckets.get_mut(slot as usize) {
                b.shift_remove(&id);
            }
        }
    }
}

/// The bucket offset for an item, 0 to `BucketCount - 1` buckets ahead of the current one. Items due within
/// the bucket interval go to the active queue, so offset 0 means one to two intervals ahead.
fn get_bucket_offset(bucket_interval: TimeSpan, time_until_decay: TimeSpan) -> usize {
    let offset = time_until_decay.ticks() / bucket_interval.ticks() - 1;
    offset.clamp(0, BUCKET_COUNT as i64 - 1) as usize
}

fn start_timer(w: &mut World) {
    let handle = match state(w).timer {
        Some(h) => h,
        None => {
            let interval = state(w).config.tick_interval;
            let h = timer::create(w, interval, interval, 0, on_tick);
            state(w).timer = Some(h);
            h
        }
    };
    timer::begin(w, handle);
}

fn on_tick(w: &mut World) {
    // Stop timer if nothing left to track
    let now = w.now;
    if process_tick(w, now) {
        if let Some(h) = state(w).timer {
            timer::stop(w, h);
        }
        return;
    }

    // Apply jitter for next tick to prevent synchronization with other systems
    let (jitter_max, tick_interval) = {
        let s = state(w);
        (s.config.jitter_max_milliseconds, s.config.tick_interval)
    };
    if jitter_max > 0 {
        let jitter = utility::random_from(&mut w.rng, -jitter_max, jitter_max * 2 + 1);
        if let Some(h) = state(w).timer {
            timer::set_interval(
                w,
                h,
                tick_interval + TimeSpan::from_millis(i64::from(jitter)),
            );
        }
    }
}

/// `ProcessTick(now)`: advances the wheel; true when nothing is left to track.
pub fn process_tick(w: &mut World, now: DateTime) -> bool {
    ensure_configured(w);
    // Process items from active queue first (smaller queue = faster log(n) operations)
    process_active_queue(w, now);

    // Then check if it's time to rotate buckets (adds items for next tick)
    if now >= state(w).next_bucket_rotation {
        rotate_buckets(w, now);
    }

    // Check overflow bucket on its own schedule (when items might enter the regular window)
    if now >= state(w).next_overflow_check {
        process_overflow(w, now);
    }

    is_empty(state(w))
}

fn alive_and_can_decay(w: &World, id: ItemId) -> bool {
    w.item(id)
        .is_some_and(|i| !i.deleted() && (i.ty.can_decay)(w, id))
}

/// Rotates the timer wheel, moving the next bucket's contents into the active queue.
fn rotate_buckets(w: &mut World, now: DateTime) {
    let (bucket_interval, total_span, current) = {
        let s = state(w);
        s.next_bucket_rotation = now + s.config.bucket_interval;
        (
            s.config.bucket_interval,
            s.total_bucket_span,
            s.current_bucket_index,
        )
    };

    // Move all items from this bucket into the active queue (with validation)
    let bucket: Vec<ItemId> = state(w).buckets[current].iter().copied().collect();

    for id in bucket {
        if !alive_and_can_decay(w, id) {
            set_slot(w, id, SLOT_NONE); // Lazy cleanup - item was picked up or deleted
            continue;
        }

        let decay_time = crate::item::ops::scheduled_decay_time(w, id);
        let time_until_decay = decay_time - now;

        if time_until_decay > bucket_interval {
            // Deadline was pushed out (SetLastMoved/RestartDecay) - re-bucket or move to overflow
            if time_until_decay > total_span {
                state(w).overflow_bucket.insert(id);
                set_slot(w, id, SLOT_OVERFLOW);
            } else {
                let offset = get_bucket_offset(bucket_interval, time_until_decay);
                let actual = (current + offset) % BUCKET_COUNT;
                if actual != current {
                    state(w).buckets[actual].insert(id);
                    set_slot(w, id, actual as i8);
                } else {
                    // Still maps to current bucket - add to active queue
                    enqueue_active(w, id, decay_time);
                }
            }
            continue;
        }

        // Due within next window - add to active queue
        enqueue_active(w, id, decay_time);
    }

    // Clear the processed bucket, then advance to the next one.
    let s = state(w);
    s.buckets[current].clear();
    s.current_bucket_index = (current + 1) % BUCKET_COUNT;
}

/// Moves overflow items that are now within the regular bucket window.
fn process_overflow(w: &mut World, now: DateTime) {
    let (bucket_interval, total_span) = {
        let s = state(w);
        s.next_overflow_check = now + s.total_bucket_span;
        (s.config.bucket_interval, s.total_bucket_span)
    };

    let overflow: Vec<ItemId> = state(w).overflow_bucket.iter().copied().collect();
    for id in overflow {
        if !alive_and_can_decay(w, id) {
            set_slot(w, id, SLOT_NONE);
            state(w).overflow_bucket.shift_remove(&id);
            continue;
        }

        let decay_time = crate::item::ops::scheduled_decay_time(w, id);
        let time_until_decay = decay_time - now;

        if time_until_decay > total_span {
            continue; // Keep in overflow
        }

        // Now within regular bucket range - move to appropriate bucket
        state(w).overflow_bucket.shift_remove(&id);
        if time_until_decay <= bucket_interval {
            enqueue_active(w, id, decay_time);
        } else {
            let offset = get_bucket_offset(bucket_interval, time_until_decay);
            let current = state(w).current_bucket_index;
            let actual = (current + offset) % BUCKET_COUNT;
            state(w).buckets[actual].insert(id);
            set_slot(w, id, actual as i8);
        }
    }
}

/// Processes items from the active queue that are due for decay.
fn process_active_queue(w: &mut World, now: DateTime) {
    let mut processed = 0;
    let max = state(w).config.max_items_per_tick;

    loop {
        // Entries of unregistered items are skipped without counting.
        let (scheduled_time, id) = {
            let s = state(w);
            loop {
                match s.active_queue.peek() {
                    Some(&Reverse((_, gen_, id))) if s.active_gen.get(&id) != Some(&gen_) => {
                        s.active_queue.pop();
                    }
                    Some(&Reverse((t, _, id))) => break (Some(t), id),
                    None => break (None, ItemId(muo_primitives::Serial::ZERO)),
                }
            }
        };
        let Some(scheduled_time) = scheduled_time else {
            break;
        };
        if processed >= max {
            break;
        }

        // Not yet due - stop processing
        if scheduled_time > now {
            break;
        }

        processed += 1;
        {
            let s = state(w);
            s.active_queue.pop();
            s.active_gen.remove(&id);
        }
        set_slot(w, id, SLOT_NONE);

        if !alive_and_can_decay(w, id) {
            continue;
        }

        let Some(ty) = w.item(id).map(|i| i.ty) else {
            continue;
        };
        if crate::item::ops::scheduled_decay_time(w, id) > now {
            register(w, id);
        } else if (ty.on_decay)(w, id) {
            (ty.delete)(w, id);
        } else {
            // Refused by the region. Restart the clock rather than dropping the item, which has already left
            // the queue; re-registering as-is would spin on a due time in the past.
            crate::item::ops::restart_decay(w, id);
        }
    }
}
