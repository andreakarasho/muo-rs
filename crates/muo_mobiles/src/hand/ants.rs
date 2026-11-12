//! The ants.

use muo_content::creature::solen;
use muo_content::creature::vtable::BASE_CREATURE_C;
use muo_server::World;
use muo_server::ids::MobileId;

macro_rules! friendship_enemy {
    ($name:ident, $check:path) => {
        /// `if (SolenHelper.Check<Colour>Friendship(m)) return false; return base.IsEnemy(m);`
        pub fn $name(w: &mut World, id: MobileId, m: MobileId) -> bool {
            if $check(w, m) {
                return false;
            }
            (BASE_CREATURE_C.is_enemy)(w, id, m)
        }
    };
}
friendship_enemy!(
    black_solen_infiltrator_queen_is_enemy,
    solen::check_black_friendship
);
friendship_enemy!(
    black_solen_infiltrator_warrior_is_enemy,
    solen::check_black_friendship
);
friendship_enemy!(black_solen_worker_is_enemy, solen::check_black_friendship);
friendship_enemy!(
    red_solen_infiltrator_queen_is_enemy,
    solen::check_red_friendship
);
friendship_enemy!(
    red_solen_infiltrator_warrior_is_enemy,
    solen::check_red_friendship
);
friendship_enemy!(red_solen_worker_is_enemy, solen::check_red_friendship);
