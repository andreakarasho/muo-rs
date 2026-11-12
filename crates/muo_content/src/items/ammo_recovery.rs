//! The SE-era recovery of the ammo a player missed with. The banked ammo
//! is held off the player (here, a resource); one repeating timer per tracked player gathers it once the archer has
//! disengaged (out of warmode and not running).

use indexmap::IndexMap;
use muo_primitives::TimeSpan;
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::timer::{self, TimerHandle};

use crate::core;
use crate::factory;

/// How often the archer is re-checked.
const RECOVERY_INTERVAL: TimeSpan = TimeSpan::from_secs(10);
/// A running step is only current if one landed this recently.
const RUNNING_WINDOW_MILLIS: i64 = 500;

struct RecoveryState {
    /// `Dictionary<Type, int>`: insertion order.
    ammo: IndexMap<&'static str, i32>,
    timer: TimerHandle,
}

/// `_states`: only players who banked recoverable ammo appear.
#[derive(Default)]
struct States(IndexMap<MobileId, RecoveryState>);

/// `AmmoRecovery.Bank(player, ammoType)`.
pub fn bank(w: &mut World, player: MobileId, ammo_type: &'static str) {
    if !core::se(w) {
        return;
    }
    if !w.resources.get_mut::<States>().0.contains_key(&player) {
        let timer = timer::start(w, RECOVERY_INTERVAL, RECOVERY_INTERVAL, 0, move |w| {
            on_recovery_tick(w, player)
        });
        w.resources.get_mut::<States>().0.insert(
            player,
            RecoveryState {
                ammo: IndexMap::new(),
                timer,
            },
        );
    }
    if let Some(s) = w.resources.get_mut::<States>().0.get_mut(&player) {
        *s.ammo.entry(ammo_type).or_insert(0) += 1;
    }
}

/// `AmmoRecovery.Forget(player)`.
pub fn forget(w: &mut World, player: MobileId) {
    if let Some(mut s) = w.resources.get_mut::<States>().0.shift_remove(&player) {
        s.timer.cancel(w);
    }
}

fn on_recovery_tick(w: &mut World, player: MobileId) {
    let Some(count) = w
        .resources
        .get_mut::<States>()
        .0
        .get(&player)
        .map(|s| s.ammo.len())
    else {
        return;
    };
    let Some(m) = w.mobile(player) else {
        forget(w, player);
        return;
    };
    if m.deleted || count == 0 {
        forget(w, player);
        return;
    }
    // Gather only once the archer has disengaged: out of warmode and not running.
    let running =
        m.direction.is_running() && w.tick_count - m.last_move_time < RUNNING_WINDOW_MILLIS;
    if !crate::mx::alive(w, player) || m.warmode || running {
        return; // Not settled yet; try again on the next tick.
    }
    recover(w, player);
    forget(w, player);
}

fn recover(w: &mut World, player: MobileId) {
    let ammo: Vec<(&'static str, i32)> = w
        .resources
        .get_mut::<States>()
        .0
        .get(&player)
        .map(|s| s.ammo.iter().map(|(k, v)| (*k, *v)).collect())
        .unwrap_or_default();
    for (ty, amount) in ammo {
        if amount <= 0 {
            continue;
        }
        let Some(item) = factory::make_item(w, ty) else {
            continue;
        };
        muo_server::item::ops::set_amount(w, item, amount);

        let name = match w.item(item).and_then(|i| i.name.clone()) {
            Some(n) => n,
            None => {
                let mut label = w.item(item).map_or(0, |i| (i.ty.label_number)(w, item));
                // Arrow (1023903/1023904) and bolt (1027163/1027164) name clilocs keep their plural form two entries
                // above the singular, so bump the label when recovering more than one.
                if amount != 1 && matches!(label, 1023903 | 1023904 | 1027163 | 1027164) {
                    label += 2;
                }
                format!("#{label}")
            }
        };
        let amount = w.item(item).map_or(amount, |i| i.amount);
        muo_server::mobile::part2::items::place_in_backpack(w, player, item);
        // You recover ~1_NUM~ ~2_AMMO~.
        muo_server::mobile::part2::messages::send_localized_message(
            w,
            player,
            1073504,
            &format!("{amount}\t{name}"),
            0x3B2,
        );
    }
    if let Some(s) = w.resources.get_mut::<States>().0.get_mut(&player) {
        s.ammo.clear();
    }
}

/// Sets the two hooks the weapon and the player call this through.
pub fn configure(w: &mut World) {
    w.resources
        .get_mut::<crate::items::weapon_hooks::WeaponHooks>()
        .ammo_recovery_bank = Some(bank);
    w.resources
        .get_mut::<crate::player_mobile::hooks::SystemHooks>()
        .ammo_recovery_forget = Some(forget);
}

pub fn register(types: &mut muo_server::type_registry::TypeRegistry) {
    types.register_phase_default(
        muo_server::type_registry::Phase::Configure,
        "Server.Items.AmmoRecovery.Configure",
        configure,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn banked_ammo_is_counted_per_type_in_order_and_forgotten() {
        let mut w = World::new();
        let p = crate::player_mobile::new_player_mobile(&mut w);
        let was_se = core::se(&w);
        if !was_se {
            bank(&mut w, p, "Server.Items.Arrow");
            assert!(
                w.resources.get_mut::<States>().0.is_empty(),
                "nothing is banked before SE"
            );
            return;
        }
        bank(&mut w, p, "Server.Items.Bolt");
        bank(&mut w, p, "Server.Items.Arrow");
        bank(&mut w, p, "Server.Items.Bolt");
        let s = &w.resources.get_mut::<States>().0[&p];
        assert_eq!(
            s.ammo.iter().map(|(k, v)| (*k, *v)).collect::<Vec<_>>(),
            [("Server.Items.Bolt", 2), ("Server.Items.Arrow", 1)]
        );
        forget(&mut w, p);
        assert!(w.resources.get_mut::<States>().0.is_empty());
    }
}
