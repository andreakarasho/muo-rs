//! The humanoid monsters.

use muo_content::creature::vtable::BASE_CREATURE_C;
use muo_primitives::{Body, TimeSpan};
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::layer::Layer;
use muo_server::mobile::{
    part1::props_b,
    part2::{effects, items},
};

fn wears_orcish_kin_mask(w: &World, m: MobileId) -> bool {
    items::find_item_on_layer(w, m, Layer::Helm)
        .is_some_and(|i| muo_content::factory::item_is(w, i, "Server.Items.OrcishKinMask"))
}

/// `AOS.Damage(aggressor, 50, 0, 100, 0, 0, 0)` and the flame effect shared by the orc and savage reactions.
fn scorch(w: &mut World, aggressor: MobileId) {
    muo_content::misc::aos_helpers::damage(
        w,
        aggressor,
        None,
        50,
        muo_content::misc::aos_helpers::DamageSplit {
            phys: 0,
            fire: 100,
            cold: 0,
            pois: 0,
            nrgy: 0,
        },
        Default::default(),
    );
}

fn burn_effects(w: &mut World, aggressor: MobileId) {
    effects::fixed_particles(
        w,
        aggressor,
        0x36BD,
        20,
        10,
        5044,
        muo_server::effects::EffectLayer::Head,
    );
    effects::play_sound(w, aggressor, 0x307);
}

macro_rules! orc_kin {
    ($enemy:ident, $action:ident) => {
        /// `(!m.Player || m.FindItemOnLayer<OrcishKinMask>(Layer.Helm) == null) && base.IsEnemy(m)`
        pub fn $enemy(w: &mut World, id: MobileId, m: MobileId) -> bool {
            let player = w.mobile(m).is_some_and(|x| x.player);
            (!player || !wears_orcish_kin_mask(w, m)) && (BASE_CREATURE_C.is_enemy)(w, id, m)
        }

        /// `base.AggressiveAction(..)`, then an aggressor wearing an `OrcishKinMask` is burned and loses it.
        pub fn $action(w: &mut World, id: MobileId, aggressor: MobileId, criminal: bool) {
            (muo_content::creature::BASE_CREATURE.aggressive_action_criminal)(
                w, id, aggressor, criminal,
            );
            if let Some(item) = items::find_item_on_layer(w, aggressor, Layer::Helm)
                .filter(|&i| muo_content::factory::item_is(w, i, "Server.Items.OrcishKinMask"))
            {
                scorch(w, aggressor);
                muo_server::item::base::delete(w, item);
                burn_effects(w, aggressor);
            }
        }
    };
}
orc_kin!(orc_is_enemy, orc_aggressive_action_criminal);
orc_kin!(orc_captain_is_enemy, orc_captain_aggressive_action_criminal);
orc_kin!(orcish_lord_is_enemy, orcish_lord_aggressive_action_criminal);
orc_kin!(orcish_mage_is_enemy, orcish_mage_aggressive_action_criminal);
orc_kin!(orc_brute_is_enemy, orc_brute_aggressive_action_criminal);
orc_kin!(orc_bomber_is_enemy, orc_bomber_aggressive_action_criminal);

fn painted(w: &World, m: MobileId) -> bool {
    w.mobile(m)
        .is_some_and(|x| x.body_mod.0 == 183 || x.body_mod.0 == 184)
}

macro_rules! savage {
    ($enemy:ident, $action:ident) => {
        /// `if (m.BodyMod == 183 || m.BodyMod == 184) return false; return base.IsEnemy(m);`
        pub fn $enemy(w: &mut World, id: MobileId, m: MobileId) -> bool {
            if painted(w, m) {
                return false;
            }
            (BASE_CREATURE_C.is_enemy)(w, id, m)
        }

        /// `base.AggressiveAction(..)`, then a painted aggressor is burned and loses the paint.
        pub fn $action(w: &mut World, id: MobileId, aggressor: MobileId, criminal: bool) {
            (muo_content::creature::BASE_CREATURE.aggressive_action_criminal)(
                w, id, aggressor, criminal,
            );
            if painted(w, aggressor) {
                scorch(w, aggressor);
                props_b::set_body_mod(w, aggressor, Body(0));
                props_b::set_hue_mod(w, aggressor, -1);
                burn_effects(w, aggressor);
                muo_content::mx::msg(w, aggressor, 1040008); // Your skin is scorched as the tribal paint burns away!
                if muo_content::player_mobile::data(w, aggressor).is_some() {
                    muo_content::player_mobile::set_savage_paint_expiration(
                        w,
                        aggressor,
                        TimeSpan::ZERO,
                    );
                }
            }
        }
    };
}
savage!(savage_is_enemy, savage_aggressive_action_criminal);
savage!(
    savage_rider_is_enemy,
    savage_rider_aggressive_action_criminal
);
