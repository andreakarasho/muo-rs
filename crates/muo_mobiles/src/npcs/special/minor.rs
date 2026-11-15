//! `Silvani`, `ServantOfSemidar`, `CapturedHordeMinion` and `DarkGuardian`: creatures with no
//! state of their own (each is `[SerializationGenerator(0, false)]` or `(0)`: parent part + version 0).

use muo_content::creature::{BASE_CREATURE, BASE_CREATURE_C, cd_mut};
use muo_content::hooks::rnd;
use muo_content::mx;
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::mobile::MobileType;
use muo_server::poison::PoisonId;
use muo_server::type_registry::TypeRegistry;

use crate::common::*;
use crate::monsters::leaves::{HORDE_MINION, HORDE_MINION_C, horde_minion_init};

const REGULAR: PoisonId = PoisonId(1);
const LETHAL: PoisonId = PoisonId(4);

// ---- Silvani ----

fn silvani_new(w: &mut World) -> MobileId {
    muo_content::templates::new_creature(w, &SILVANI)
}

static SILVANI_ABILITIES: [&muo_content::abilities::MonsterAbility; 1] =
    [&muo_content::abilities::SUMMON_PIXIES_COUNTER];

creature_leaf! {
    /// `Server.Mobiles.Silvani`.
    SILVANI, SILVANI_C = "Server.Mobiles.Silvani" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(silvani_new), encoded false;
    c {
        default_name: |_w, _id| Some("Silvani"),
        opposition_group: |_w, _id| Some(&muo_content::ai::opposition_group::FEY_AND_UNDEAD),
        can_fly: |_w, _id| true,
        unprovokable: |_w, _id| true,
        poison_immune: |_w, _id| Some(REGULAR),
        treasure_map_level: |_w, _id| 5,
        get_monster_abilities: |_, _| &SILVANI_ABILITIES,
        generate_loot: |w, id| add_loot_n(w, id, loot_pack::ultra_rich(w), 2),
        on_gave_melee_attack: |w, id, defender, damage| {
            (BASE_CREATURE_C.on_gave_melee_attack)(w, id, defender, damage);
            super::champions::drain_hit(w, id, defender);
        },
    }
    m {
    }
}

// ---- ServantOfSemidar ----

fn servant_new(w: &mut World) -> MobileId {
    muo_content::templates::new_creature(w, &SERVANT_OF_SEMIDAR)
}

creature_leaf! {
    /// `Server.Mobiles.ServantOfSemidar`.
    SERVANT_OF_SEMIDAR, SERVANT_OF_SEMIDAR_C = "Server.Mobiles.ServantOfSemidar" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(servant_new), encoded true;
    c {
        default_name: |_w, _id| Some("a Servant of Semidar"),
        disallow_all_moves: |_w, _id| true,
        initial_innocent: |_w, _id| true,
    }
    m {
        can_be_damaged: |_w, _id| false,
        add_name_properties: |w, id, list| {
            (BASE_CREATURE.add_name_properties)(w, id, list);
            list.add(1005494); // enslaved
        },
    }
}

// ---- CapturedHordeMinion ----

fn captured_horde_minion_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &CAPTURED_HORDE_MINION,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    horde_minion_init(w, id);
    if let Some(d) = cd_mut(w, id) {
        d.fight_mode = FightMode::None;
    }
    id
}

creature_leaf! {
    /// `Server.Mobiles.CapturedHordeMinion`.
    CAPTURED_HORDE_MINION, CAPTURED_HORDE_MINION_C = "Server.Mobiles.CapturedHordeMinion" : HORDE_MINION / HORDE_MINION_C, ctor Some(captured_horde_minion_new), encoded true;
    c {
        initial_innocent: |_w, _id| true,
    }
    m {
        can_be_damaged: |_w, _id| false,
    }
}

// ---- DarkGuardian ----

fn dark_guardian_new(w: &mut World) -> MobileId {
    muo_content::templates::new_creature(w, &DARK_GUARDIAN)
}

creature_leaf! {
    /// `Server.Mobiles.DarkGuardian`.
    DARK_GUARDIAN, DARK_GUARDIAN_C = "Server.Mobiles.DarkGuardian" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(dark_guardian_new), encoded true;
    c {
        corpse_name: |_w, _id| Some("a dark guardians' corpse"),
        default_name: |_w, _id| Some("a dark guardian"),
        opposition_group: |_w, _id| Some(&muo_content::ai::opposition_group::FEY_AND_UNDEAD),
        treasure_map_level: |_w, _id| 2,
        bleed_immune: |_w, _id| true,
        poison_immune: |_w, _id| Some(LETHAL),
        unprovokable: |_w, _id| true,
        generate_loot: |w, id| {
            add_loot(w, id, loot_pack::rich(w));
            add_loot_n(w, id, &loot_pack::MED_SCROLLS, 2);
        },
    }
    m {
    }
}

pub(crate) fn all() -> Vec<&'static MobileType> {
    vec![
        &SILVANI,
        &SERVANT_OF_SEMIDAR,
        &CAPTURED_HORDE_MINION,
        &DARK_GUARDIAN,
    ]
}

pub(crate) fn register(types: &mut TypeRegistry) {
    for t in all() {
        types.register_mobile(t, &[]);
    }
}

#[allow(dead_code)]
fn _unused(w: &mut World, id: MobileId) -> i32 {
    mx::hits(w, id) + rnd::below(w, 1)
}
