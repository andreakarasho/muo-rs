//! The solen queens and warriors: `BurstSac`, the acid, and the friendship checks.

use muo_content::creature::solen;
use muo_content::creature::vtable::BASE_CREATURE_C;
use muo_content::creature::{BASE_CREATURE, pets};
use muo_io::{GenericReader, GenericWriter};
use muo_primitives::MessageType;
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::mobile::part2::messages;

/// `private bool _burstSac`.
#[derive(Default)]
pub struct BurstSac(pub bool);

pub fn burst_sac_init(w: &mut World, id: MobileId) {
    if let Some(m) = w.mobile_mut(id) {
        m.ext.insert(BurstSac(false));
    }
}

/// `[SerializationGenerator(0, false)]` with `_burstSac`: the int version, then the bool.
pub fn burst_sac_serialize(w: &World, id: MobileId, out: &mut dyn GenericWriter) {
    out.write_i32(0);
    out.write_bool(burst_sac(w, id));
}

pub fn burst_sac_deserialize(
    w: &mut World,
    id: MobileId,
    r: &mut dyn GenericReader,
) -> muo_io::Result<()> {
    let _version = r.read_i32()?;
    let sac = r.read_bool()?;
    if let Some(m) = w.mobile_mut(id) {
        m.ext.insert(BurstSac(sac));
    }
    Ok(())
}

fn burst_sac(w: &World, id: MobileId) -> bool {
    w.mobile(id)
        .and_then(|m| m.ext.get::<BurstSac>())
        .is_some_and(|b| b.0)
}
fn set_burst_sac(w: &mut World, id: MobileId) {
    if let Some(b) = w.mobile_mut(id).and_then(|m| m.ext.get_mut::<BurstSac>()) {
        b.0 = true;
    }
}

/// The shared `OnDamage`: the burst sac message at 50 hits, then acid on whoever stands next to it.
fn solen_on_damage(
    w: &mut World,
    id: MobileId,
    amount: i32,
    from: Option<MobileId>,
    will_kill: bool,
    red: bool,
) {
    if let Some(f) = from {
        if red {
            solen::on_red_damage(w, f);
        } else {
            solen::on_black_damage(w, f);
        }
    }
    if !will_kill {
        if !burst_sac(w, id) {
            if muo_content::mx::hits(w, id) < 50 {
                // * The solen's sac bursts!
                messages::public_overhead_message_number(
                    w,
                    id,
                    MessageType::REGULAR,
                    0x3B2,
                    1080038,
                    "",
                    true,
                );
                set_burst_sac(w, id);
            }
        } else if let Some(f) = from.filter(|&f| f != id && muo_content::mx::in_range(w, id, f, 1))
        {
            messages::public_overhead_message_number(
                w,
                id,
                MessageType::REGULAR,
                0x3B2,
                1080060,
                "",
                true,
            );
            pets::spill_acid(w, id, Some(f), 1);
        }
    }
    (BASE_CREATURE.on_damage)(w, id, amount, from, will_kill);
}

macro_rules! solen_class {
    ($enemy:ident, $damage:ident, $before:ident, $check:path, $red:expr) => {
        pub fn $enemy(w: &mut World, id: MobileId, m: MobileId) -> bool {
            if $check(w, m) {
                return false;
            }
            (BASE_CREATURE_C.is_enemy)(w, id, m)
        }
        pub fn $damage(
            w: &mut World,
            id: MobileId,
            amount: i32,
            from: Option<MobileId>,
            will_kill: bool,
        ) {
            solen_on_damage(w, id, amount, from, will_kill, $red);
        }
        /// `SpillAcid(4); return base.OnBeforeDeath();`
        pub fn $before(w: &mut World, id: MobileId) -> bool {
            pets::spill_acid(w, id, None, 4);
            (BASE_CREATURE.on_before_death)(w, id)
        }
    };
}
solen_class!(
    black_solen_queen_is_enemy,
    black_solen_queen_on_damage,
    black_solen_queen_on_before_death,
    solen::check_black_friendship,
    false
);
solen_class!(
    black_solen_warrior_is_enemy,
    black_solen_warrior_on_damage,
    black_solen_warrior_on_before_death,
    solen::check_black_friendship,
    false
);
solen_class!(
    red_solen_queen_is_enemy,
    red_solen_queen_on_damage,
    red_solen_queen_on_before_death,
    solen::check_red_friendship,
    true
);
solen_class!(
    red_solen_warrior_is_enemy,
    red_solen_warrior_on_damage,
    red_solen_warrior_on_before_death,
    solen::check_red_friendship,
    true
);
