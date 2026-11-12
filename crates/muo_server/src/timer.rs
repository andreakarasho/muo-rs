//! Timers: the timer wheel, `DelayCall`, `DelayStateCall` and
//! `TimerExecutionToken` (there is no allocation pool).
//!
//! The wheel keeps one list per slot. Every timer is a
//! node in an arena (`TimerWheel::nodes`, linked by index) and is named by a generation-checked [`TimerHandle`]
//! (which is also the `TimerExecutionToken`: a stale handle is simply not running).
//!
//! Timer conventions, once for the whole crate:
//! - `Timer.StartTimer(..)` / `DelayCall(..)` are [`start`]: a closure replaces both the `Action` callback and the
//!   `DelayStateCall` state arguments. The node is freed when the timer detaches.
//! - A `class FooTimer : Timer` with `OnTick` becomes a closure capturing ids and state; the timer object a
//!   field held becomes a `TimerHandle` (or `Option<TimerHandle>`). A subclass that restarts or reconfigures itself
//!   uses [`create`] (a node that survives `Stop`, freed by [`release`]) and finds itself with [`current`].
//! - The callback is taken out of its node while it runs, so it can hold `&mut World`; a timer stopped during its own
//!   callback stays stopped.
//!
//! // shortcut: `Timer.Pause` (awaitable; no script calls it), `DumpInfo` (nothing calls it; timers have no type name
//! to report) and `OnDetach` overrides (only `DelayCallTimer`'s pool return, which `release` does) don't exist.

use muo_primitives::{DateTime, TimeSpan};

use crate::world::World;

const RING_SIZE_POW2: i64 = 12;
const RING_SIZE: usize = 1 << RING_SIZE_POW2;
const RING_LAYERS: usize = 3;
const TICK_RATE_POW2: i64 = 3;
/// Milliseconds of simulated time one wheel turn advances (`Timer.TickRate`).
pub const TICK_RATE: i64 = 1 << TICK_RATE_POW2;
const MAX_DURATION: i64 = TICK_RATE << (RING_SIZE_POW2 * RING_LAYERS as i64 - 1);
const NIL: u32 = u32::MAX;

pub type Callback = Box<dyn FnMut(&mut World)>;

/// Names one timer; the `TimerExecutionToken`. `Default` is a handle to nothing (`default(TimerExecutionToken)`).
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
pub struct TimerHandle {
    index: u32,
    generation: u32,
}

struct Node {
    generation: u32,
    in_use: bool,
    release_on_detach: bool,
    callback: Option<Callback>,
    ring: i32,
    slot: i32,
    remaining: i64,
    next: u32,
    prev: u32,
    /// Milliseconds, already rounded up to the tick rate.
    delay: i64,
    interval: i64,
    count: i32,
    index: i32,
    running: bool,
    version: i32,
    next_time: DateTime,
}

/// `Timer`'s static state: the three rings, the executing chains and the last tick the wheel turned at.
pub struct TimerWheel {
    nodes: Vec<Node>,
    free: Vec<u32>,
    rings: [Vec<u32>; RING_LAYERS],
    ring_indexes: [usize; RING_LAYERS],
    executing_rings: [u32; RING_LAYERS],
    last_tick_turned: i64,
    executing: Option<TimerHandle>,
}

impl Default for TimerWheel {
    fn default() -> Self {
        TimerWheel::new(0)
    }
}

/// `value` milliseconds rounded up to the next multiple of the tick rate; zero and below is one tick.
fn round_ticks_to_next_power_of_two(value: i64) -> i64 {
    if value <= 0 {
        return TICK_RATE;
    }
    let mask = TICK_RATE - 1;
    (value + mask) & !mask
}

fn rounded(span: TimeSpan) -> i64 {
    round_ticks_to_next_power_of_two(span.total_milliseconds() as i64)
}

impl TimerWheel {
    /// `Timer.Init(tickCount)`: empty rings, the wheel starting at `tick_count`.
    pub fn new(tick_count: i64) -> TimerWheel {
        TimerWheel {
            nodes: Vec::new(),
            free: Vec::new(),
            rings: std::array::from_fn(|_| vec![NIL; RING_SIZE]),
            ring_indexes: [0; RING_LAYERS],
            executing_rings: [NIL; RING_LAYERS],
            last_tick_turned: tick_count,
            executing: None,
        }
    }

    /// `Timer.Init(tickCount)` on an existing wheel: drops every timer (their handles go stale).
    pub fn init(&mut self, tick_count: i64) {
        for n in &mut self.nodes {
            if n.in_use {
                n.in_use = false;
                n.running = false;
                n.callback = None;
                n.generation = n.generation.wrapping_add(1).max(1);
            }
        }
        self.free = (0..self.nodes.len() as u32).rev().collect();
        self.rings = std::array::from_fn(|_| vec![NIL; RING_SIZE]);
        self.ring_indexes = [0; RING_LAYERS];
        self.executing_rings = [NIL; RING_LAYERS];
        self.last_tick_turned = tick_count;
    }

    /// `Timer.MillisecondsUntilNextTick(tickCount)`.
    pub fn milliseconds_until_next_tick(&self, tick_count: i64) -> i64 {
        (TICK_RATE - tick_count.wrapping_sub(self.last_tick_turned)).max(0)
    }

    /// The timer whose callback is running right now (`this` inside `OnTick`).
    pub fn current(&self) -> Option<TimerHandle> {
        self.executing
    }

    fn alloc(
        &mut self,
        delay: TimeSpan,
        interval: TimeSpan,
        count: i32,
        callback: Callback,
        release: bool,
    ) -> TimerHandle {
        let node = |generation| Node {
            generation,
            in_use: true,
            release_on_detach: release,
            callback: Some(callback),
            ring: -1,
            slot: -1,
            remaining: 0,
            next: NIL,
            prev: NIL,
            delay: rounded(delay),
            interval: rounded(interval),
            count,
            index: 0,
            running: false,
            version: 0,
            next_time: DateTime::MIN,
        };
        if let Some(i) = self.free.pop() {
            let generation = self.nodes[i as usize].generation;
            self.nodes[i as usize] = node(generation);
            TimerHandle {
                index: i,
                generation,
            }
        } else {
            self.nodes.push(node(1));
            TimerHandle {
                index: self.nodes.len() as u32 - 1,
                generation: 1,
            }
        }
    }

    fn live(&self, h: TimerHandle) -> Option<usize> {
        let n = self.nodes.get(h.index as usize)?;
        (n.in_use && n.generation == h.generation).then_some(h.index as usize)
    }

    fn free_node(&mut self, i: usize) {
        let n = &mut self.nodes[i];
        n.in_use = false;
        n.running = false;
        n.callback = None;
        n.generation = n.generation.wrapping_add(1).max(1);
        self.free.push(i as u32);
    }

    fn attach(&mut self, i: usize, other: u32) {
        self.nodes[i].next = other;
        if other != NIL {
            self.nodes[other as usize].prev = i as u32;
        }
    }

    fn detach(&mut self, i: usize) {
        let (prev, next) = (self.nodes[i].prev, self.nodes[i].next);
        if prev != NIL {
            self.nodes[prev as usize].next = next;
        }
        if next != NIL {
            self.nodes[next as usize].prev = prev;
        }
        self.nodes[i].next = NIL;
        self.nodes[i].prev = NIL;
    }

    fn internal_stop(&mut self, i: usize) {
        self.nodes[i].running = false;
        let (ring, slot, next) = (self.nodes[i].ring, self.nodes[i].slot, self.nodes[i].next);
        if ring < 0 {
            return;
        }
        let (ring, slot) = (ring as usize, slot as usize);
        // We are the head on the timer ring, or on the executing ring.
        if self.rings[ring][slot] == i as u32 {
            self.rings[ring][slot] = next;
        }
        if self.executing_rings[ring] == i as u32 {
            self.executing_rings[ring] = next;
        }
    }

    fn on_detach(&mut self, i: usize) {
        if self.nodes[i].running {
            tracing::error!("timer detached while still running");
            return;
        }
        self.nodes[i].ring = -1;
        self.nodes[i].slot = -1;
        if self.nodes[i].release_on_detach {
            self.free_node(i);
        }
    }

    fn stop_node(&mut self, i: usize) {
        if !self.nodes[i].running {
            return;
        }
        self.internal_stop(i);
        self.detach(i);
        self.nodes[i].version += 1;
        self.on_detach(i);
    }

    fn add_timer(&mut self, i: usize, delay: i64, now: DateTime) {
        let actual_delay = delay;
        let mut delay = delay;
        let mut resolution_pow2 = TICK_RATE_POW2;
        for layer in 0..RING_LAYERS {
            let resolution = 1i64 << resolution_pow2;
            let next_pow2 = resolution_pow2 + RING_SIZE_POW2;
            let max = 1i64 << next_pow2;
            let last_ring = layer == RING_LAYERS - 1;

            if delay < max || last_ring {
                let ring_index = self.ring_indexes[layer] as i64;
                let mut remaining = delay & (resolution - 1);
                let mut slot = (delay >> resolution_pow2) + ring_index + i64::from(remaining > 0);

                // Round up if we have a delay of 0
                if delay == 0 {
                    slot += 1;
                    remaining = 0;
                }

                if slot >= RING_SIZE as i64 {
                    slot -= RING_SIZE as i64;
                    // Only reachable on the last ring with a timer past max capacity: it goes on the last slot.
                    // (Testing `slot > RingSize` would index out of range at exactly RingSize.)
                    if last_ring && slot >= RING_SIZE as i64 {
                        tracing::error!(
                            "Timer has a duration of {actual_delay}ms, more than max capacity of {MAX_DURATION}ms."
                        );
                        slot = (ring_index - 1).max(0);
                    }
                }

                let head = self.rings[layer][slot as usize];
                let n = &mut self.nodes[i];
                n.next_time = now + TimeSpan::from_millis(n.delay);
                self.attach(i, head);
                let n = &mut self.nodes[i];
                n.remaining = remaining;
                n.ring = layer as i32;
                n.slot = slot as i32;
                self.rings[layer][slot as usize] = i as u32;
                return;
            }

            // The remaining amount until we turn this ring
            delay -= resolution * (RING_SIZE - self.ring_indexes[layer]) as i64;
            resolution_pow2 = next_pow2;
        }
    }
}

impl TimerHandle {
    /// `Timer.Running`; false for a stale or default handle.
    pub fn running(self, w: &World) -> bool {
        w.timers
            .live(self)
            .is_some_and(|i| w.timers.nodes[i].running)
    }

    /// `Timer.Index`: how many times it ticked since it started.
    pub fn index(self, w: &World) -> i32 {
        w.timers.live(self).map_or(0, |i| w.timers.nodes[i].index)
    }

    /// `Timer.RemainingCount`: `i32::MAX` when infinite, 0 for a stale handle.
    pub fn remaining_count(self, w: &World) -> i32 {
        w.timers.live(self).map_or(0, |i| {
            let n = &w.timers.nodes[i];
            if n.count == 0 {
                i32::MAX
            } else {
                n.count - n.index
            }
        })
    }

    /// `Timer.Next`: when the next tick is due (`DateTime::MIN` for a stale handle).
    pub fn next(self, w: &World) -> DateTime {
        w.timers
            .live(self)
            .map_or(DateTime::MIN, |i| w.timers.nodes[i].next_time)
    }

    /// `TimerExecutionToken.Cancel()`: stops the timer, frees it, and resets this handle. Safe to repeat.
    pub fn cancel(&mut self, w: &mut World) {
        if let Some(i) = w.timers.live(*self) {
            w.timers.nodes[i].release_on_detach = true;
            w.timers.stop_node(i);
        }
        *self = TimerHandle::default();
    }
}

/// `Timer.StartTimer` / `Timer.DelayCall` (every overload): runs `callback` after `delay`, then every `interval`,
/// `count` times (0 is forever). `delay`/`interval` are rounded up to 8ms. The timer is freed once it finishes.
pub fn start(
    w: &mut World,
    delay: TimeSpan,
    interval: TimeSpan,
    count: i32,
    callback: impl FnMut(&mut World) + 'static,
) -> TimerHandle {
    let h = w
        .timers
        .alloc(delay, interval, count, Box::new(callback), true);
    begin(w, h);
    h
}

/// `new Timer(delay, interval, count)` without `Start()`: a node that survives `Stop`, for a timer that restarts.
/// `new Timer(delay)` is `count` 1 and a zero interval; `new Timer(interval, count)` is `(interval, interval, count)`.
pub fn create(
    w: &mut World,
    delay: TimeSpan,
    interval: TimeSpan,
    count: i32,
    callback: impl FnMut(&mut World) + 'static,
) -> TimerHandle {
    w.timers
        .alloc(delay, interval, count, Box::new(callback), false)
}

/// `Timer.Start()`: no-op when already running; restarts the index.
pub fn begin(w: &mut World, h: TimerHandle) {
    if w.state == crate::world::WorldState::Saving {
        tracing::error!("Attempted to start a timer while world is {:?}", w.state);
    }
    let now = w.now;
    let t = &mut w.timers;
    let Some(i) = t.live(h) else { return };
    if t.nodes[i].running {
        return;
    }
    t.nodes[i].index = 0;
    t.nodes[i].running = true;
    let delay = t.nodes[i].delay;
    t.add_timer(i, delay, now);
}

/// `Timer.Stop()`: safe on a stopped or stale handle.
pub fn stop(w: &mut World, h: TimerHandle) {
    if w.state == crate::world::WorldState::Saving {
        tracing::error!("Attempted to stop a timer while world is {:?}", w.state);
    }
    if let Some(i) = w.timers.live(h) {
        w.timers.stop_node(i);
    }
}

/// Frees a [`create`]d timer for good; stops it first.
pub fn release(w: &mut World, h: TimerHandle) {
    if let Some(i) = w.timers.live(h) {
        w.timers.nodes[i].release_on_detach = true;
        if w.timers.nodes[i].running {
            w.timers.stop_node(i);
        } else {
            w.timers.free_node(i);
        }
    }
}

/// `Timer.Delay = value`, effective at the next `Start()`.
pub fn set_delay(w: &mut World, h: TimerHandle, delay: TimeSpan) {
    if let Some(i) = w.timers.live(h) {
        w.timers.nodes[i].delay = rounded(delay);
    }
}

/// `Timer.Interval = value`, effective at the next reschedule.
pub fn set_interval(w: &mut World, h: TimerHandle, interval: TimeSpan) {
    if let Some(i) = w.timers.live(h) {
        w.timers.nodes[i].interval = rounded(interval);
    }
}

/// The handle of the timer whose callback is executing (`this` in `OnTick`).
pub fn current(w: &World) -> Option<TimerHandle> {
    w.timers.current()
}

/// `Timer.Slice(tickCount)` with `w.tick_count`: turns the wheel once per elapsed 8ms.
pub fn slice(w: &mut World) {
    let mut delta = w.tick_count.wrapping_sub(w.timers.last_tick_turned);
    while delta >= TICK_RATE {
        delta -= TICK_RATE;
        w.timers.last_tick_turned = w.timers.last_tick_turned.wrapping_add(TICK_RATE);
        turn(w);
    }
}

fn turn(w: &mut World) {
    let now = w.now;
    let mut turn_next = false;

    // Detach the chain from the timer wheel. This allows adding timers to the same slot during execution.
    {
        let t = &mut w.timers;
        for i in 0..RING_LAYERS {
            if i == 0 || turn_next {
                t.ring_indexes[i] += 1;
                let mut ring_index = t.ring_indexes[i];
                turn_next = ring_index >= RING_SIZE;
                if turn_next {
                    ring_index = 0;
                    t.ring_indexes[i] = 0;
                }
                t.executing_rings[i] = t.rings[i][ring_index];
                t.rings[i][ring_index] = NIL;
            } else {
                t.executing_rings[i] = NIL;
            }
        }
    }

    for layer in 0..RING_LAYERS {
        while w.timers.executing_rings[layer] != NIL {
            let t = &mut w.timers;
            let i = t.executing_rings[layer] as usize;
            // Set the executing timer to the next in the link list because we will be detaching.
            t.executing_rings[layer] = t.nodes[i].next;
            t.detach(i);

            // Check to see if it's running just in case it was stopped by another timer
            if t.nodes[i].running {
                let remaining = t.nodes[i].remaining;
                if layer > 0 && remaining > 0 {
                    t.add_timer(i, remaining, now); // Promote
                } else {
                    execute(w, i);
                }
            }
        }
    }
}

fn execute(w: &mut World, i: usize) {
    let now = w.now;
    let t = &mut w.timers;
    let n = &t.nodes[i];
    let finished = n.count != 0 && n.index + 1 >= n.count;

    // Stop the timer from running so that if it is started in its callback, it will be started.
    if finished {
        t.internal_stop(i);
        t.nodes[i].version += 1;
    }

    let n = &mut t.nodes[i];
    let (version, generation) = (n.version, n.generation);
    let mut callback = n.callback.take();
    let handle = TimerHandle {
        index: i as u32,
        generation,
    };
    let outer = t.executing.replace(handle);

    if let Some(cb) = callback.as_mut() {
        cb(w);
    }

    let t = &mut w.timers;
    t.executing = outer;
    if t.live(handle).is_none() {
        return; // released during its own callback
    }
    if t.nodes[i].callback.is_none() {
        t.nodes[i].callback = callback;
    }

    // Starting doesn't change the timer version, so check if it's finished and if it's still running.
    if t.nodes[i].version != version || finished && t.nodes[i].running {
        return;
    }

    if !finished {
        let interval = t.nodes[i].interval;
        t.add_timer(i, interval, now);
    } else {
        t.on_detach(i); // already stopped and detached
    }
    if t.live(handle).is_some() {
        t.nodes[i].index += 1;
    }
}

/// `Timer.ClearAllTimers(tickCount)`: stops every timer and restarts the wheel at `tick_count`.
pub fn clear_all_timers(w: &mut World, tick_count: i64) {
    w.timers.last_tick_turned = tick_count;
    for layer in 0..RING_LAYERS {
        for slot in 0..RING_SIZE {
            let mut node = w.timers.rings[layer][slot];
            while node != NIL {
                // Stopping unlinks the node, so read its successor first.
                let next = w.timers.nodes[node as usize].next;
                w.timers.stop_node(node as usize);
                node = next;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    //! Timer tests, plus the ordering and stop/restart rules.
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;

    use super::*;

    fn ms(v: i64) -> TimeSpan {
        TimeSpan::from_millis(v)
    }

    fn step(w: &mut World, tick: i64) {
        w.tick_count = tick;
        slice(w);
    }

    #[test]
    fn timer_test_various_times() {
        for (ticks, expected) in [
            (0, 8),
            (80, 80),
            (65, 72),
            (32767, 32768),
            (32768, 32768),
            (32833, 32840),
            (134217729, 134217736),
        ] {
            let mut w = World::new();
            w.timers.init(0);
            let executed = Rc::new(Cell::new(0));
            let seen = Rc::new(Cell::new(0i64));
            let (e, s) = (executed.clone(), seen.clone());
            start(&mut w, ms(ticks), TimeSpan::ZERO, 1, move |w| {
                s.set(w.tick_count);
                e.set(e.get() + 1);
            });
            for i in 1..=expected / 8 {
                step(&mut w, i * 8);
            }
            assert_eq!(executed.get(), 1, "delay {ticks}");
            assert_eq!(seen.get(), expected);
        }
    }

    #[test]
    fn timer_test_intervals() {
        let (delay, interval, count) = (1000, 30000, 2);
        let mut w = World::new();
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        start(&mut w, ms(delay), ms(interval), count, move |w| {
            l.borrow_mut().push(w.tick_count)
        });
        for i in 1..=(delay + interval * count as i64 - 1) / 8 {
            step(&mut w, i * 8);
        }
        assert_eq!(*log.borrow(), vec![1000, 31000]);
    }

    #[test]
    fn timer_started_on_tick() {
        let mut w = World::new();
        let executed = Rc::new(Cell::new(0));
        let e = executed.clone();
        let h = create(&mut w, ms(100), TimeSpan::ZERO, 1, move |w| {
            e.set(e.get() + 1);
            if e.get() == 1 {
                let me = current(w).unwrap();
                set_delay(w, me, ms(100));
                begin(w, me);
            }
        });
        begin(&mut w, h);
        step(&mut w, 128);
        assert_eq!(executed.get(), 1);
        step(&mut w, 256);
        assert_eq!(executed.get(), 2);
    }

    #[test]
    fn clear_all_timers_stops_every_timer_in_a_slot() {
        let mut w = World::new();
        let handles: Vec<_> = (0..3)
            .map(|_| {
                start(&mut w, ms(80), TimeSpan::ZERO, 1, |_| {
                    panic!("cleared timer fired")
                })
            })
            .collect();
        clear_all_timers(&mut w, 0);
        assert!(handles.iter().all(|h| !h.running(&w)));
        for tick in (8..=160).step_by(8) {
            step(&mut w, tick);
        }
    }

    #[test]
    fn one_slot_fires_last_started_first() {
        let mut w = World::new();
        let order = Rc::new(RefCell::new(Vec::new()));
        for n in 0..3 {
            let o = order.clone();
            start(&mut w, ms(80), TimeSpan::ZERO, 1, move |_| {
                o.borrow_mut().push(n)
            });
        }
        step(&mut w, 80);
        assert_eq!(*order.borrow(), vec![2, 1, 0]);
    }

    #[test]
    fn stopped_in_own_callback_stays_stopped_and_cancel_is_safe() {
        let mut w = World::new();
        let ran = Rc::new(Cell::new(0));
        let r = ran.clone();
        let mut h = start(&mut w, ms(8), ms(8), 0, move |w| {
            r.set(r.get() + 1);
            stop(w, current(w).unwrap());
        });
        step(&mut w, 80);
        assert_eq!(ran.get(), 1);
        assert!(!h.running(&w));
        h.cancel(&mut w);
        h.cancel(&mut w);
    }

    #[test]
    fn timer_stopped_by_an_earlier_one_in_the_slot_does_not_fire() {
        let mut w = World::new();
        let fired = Rc::new(Cell::new(false));
        let f = fired.clone();
        let victim = start(&mut w, ms(80), TimeSpan::ZERO, 1, move |_| f.set(true));
        start(&mut w, ms(80), TimeSpan::ZERO, 1, move |w| stop(w, victim)); // started last, fires first
        step(&mut w, 80);
        assert!(!fired.get());
    }

    #[test]
    fn long_delays_are_promoted_through_the_rings() {
        let mut w = World::new();
        let at = Rc::new(Cell::new(-1));
        let a = at.clone();
        start(&mut w, ms(40_000), TimeSpan::ZERO, 1, move |w| {
            a.set(w.tick_count)
        });
        for i in 1..=5000 {
            step(&mut w, i * 8);
        }
        assert_eq!(at.get(), 40_000);
    }

    #[test]
    fn stale_handle_is_not_running_and_remaining_count() {
        let mut w = World::new();
        let h = start(&mut w, ms(8), TimeSpan::ZERO, 3, |_| {});
        assert_eq!(h.remaining_count(&w), 3);
        let mut inf = start(&mut w, ms(8), ms(8), 0, |_| {});
        assert_eq!(inf.remaining_count(&w), i32::MAX);
        inf.cancel(&mut w);
        assert!(!inf.running(&w));
        assert!(!TimerHandle::default().running(&w));
    }
}
