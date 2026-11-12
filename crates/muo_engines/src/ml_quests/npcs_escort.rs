//! `Bravehorn` (Heritage) and `Lissbet` (Ilshenar): the two quest givers on a
//! `BaseEscortable`. `StaticMLQuester` is true for both, so they use the quest list registered for their class
//! (`muo_mobiles::npcs::townfolk::is_static_ml_quester`).

use muo_content::creature::BASE_CREATURE_C;
use muo_mobiles::npcs::townfolk::{
    BASE_ESCORTABLE, BASE_ESCORTABLE_C, EscortClass, default_destinations, escort_make_with,
};
use muo_primitives::{Body, utility};
use muo_server::ids::MobileId;
use muo_server::mobile::ResistanceType;
use muo_server::type_registry::TypeRegistry;
use muo_server::world::World;

use super::npc_support::*;
use super::quester::{self, Quester};

// ---- Bravehorn ----

/// `Bravehorn.InitBody()`.
fn bravehorn_init_body(w: &mut World, id: MobileId) {
    set_body(w, id, Body(0xEA));
    set_str_range(w, id, 41, 71);
    set_dex_range(w, id, 47, 77);
    set_int_range(w, id, 27, 57);
    set_hits_range(w, id, 27, 41);
    set_mana(w, id, 0);
    set_damage(w, id, 5, 9);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 20, 25);
    set_resistance_range(w, id, ResistanceType::Cold, 5, 10);
    set_skill_range(w, id, SkillName::MagicResist, 26.8, 44.5);
    set_skill_range(w, id, SkillName::Tactics, 29.8, 47.5);
    set_skill_range(w, id, SkillName::Wrestling, 29.8, 47.5);
    muo_server::mobile::part1::props_a::set_fame(w, id, 300);
    muo_server::mobile::part1::props_a::set_karma(w, id, 0);
    muo_server::mobile::part1::props_a::set_virtual_armor(w, id, 24);
}

/// `Bravehorn.InitOutfit()` does nothing.
static BRAVEHORN_E: EscortClass = EscortClass {
    init_outfit: |_, _| {},
    possible_destinations: default_destinations,
};

fn bravehorn_new(w: &mut World) -> MobileId {
    escort_make_with(w, &BRAVEHORN, bravehorn_init_body)
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Bravehorn`.
    BRAVEHORN, BRAVEHORN_C = "Server.Engines.MLQuests.Definitions.Bravehorn" : BASE_ESCORTABLE / BASE_ESCORTABLE_C, ctor bravehorn_new, encoded false;
    c {
        subclass: Some(&BRAVEHORN_E),
        initial_innocent: |_w, _id| true,
        default_name: |_w, _id| Some("Bravehorn"),
    }
    m {
        get_attack_sound: |_w, _id| 0x82,
        get_hurt_sound: |_w, _id| 0x83,
        get_death_sound: |_w, _id| 0x84,
    }
}

// ---- Lissbet ----

/// `Lissbet.InitBody()`.
fn lissbet_init_body(w: &mut World, id: MobileId) {
    set_str_range(w, id, 40, 50);
    set_dex_range(w, id, 70, 80);
    set_int_range(w, id, 80, 90);
    let skin = muo_content::vendor::human_random_skin_hue(w);
    set_hue(w, id, skin);
    set_female(w, id, true);
    set_body(w, id, Body(401));
    set_title(w, id, Some("the flower girl".to_string()));
    set_hair_item_id(w, id, 0x203D);
    set_hair_hue(w, id, 0x1BB);
}

/// `Lissbet.InitOutfit()`.
fn lissbet_init_outfit(w: &mut World, id: MobileId) {
    let h = utility::random_yellow_hue(&mut w.rng);
    add_item_hue(w, id, "Kilt", Some(h));
    let h = utility::random_yellow_hue(&mut w.rng);
    add_item_hue(w, id, "FancyShirt", Some(h));
    add_item_hue(w, id, "Sandals", None);
}

static LISSBET_E: EscortClass = EscortClass {
    init_outfit: lissbet_init_outfit,
    possible_destinations: default_destinations,
};

fn lissbet_new(w: &mut World) -> MobileId {
    escort_make_with(w, &LISSBET, lissbet_init_body)
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Lissbet`.
    LISSBET, LISSBET_C = "Server.Engines.MLQuests.Definitions.Lissbet" : BASE_ESCORTABLE / BASE_ESCORTABLE_C, ctor lissbet_new, encoded false;
    c {
        subclass: Some(&LISSBET_E),
        initial_innocent: |_w, _id| true,
        default_name: |_w, _id| Some("Lissbet"),
        can_shout: |_w, _id| true,
    }
    m {
    }
}

/// `Lissbet.Shout(pm)`.
fn lissbet_shout(w: &mut World, id: MobileId, pm: MobileId) {
    // 1074204: Greetings seeker.  I have an urgent matter for you, if you are willing.
    // 1074222: Could I trouble you for some assistance?
    let n = *utility::random_element(&mut w.rng, &[1074204, 1074222]).unwrap_or(&1074204);
    quester::tell(w, Quester::Mobile(id), pm, n);
}

/// `Shout(pm)` overrides: class full name -> function.
#[allow(clippy::type_complexity)] // the callback type, spelled out
pub static SHOUTS: &[(&str, fn(&mut World, MobileId, MobileId))] =
    &[("Server.Engines.MLQuests.Definitions.Lissbet", lissbet_shout)];

pub fn register(types: &mut TypeRegistry) {
    types.register_mobile(&BRAVEHORN, &[]);
    types.register_mobile(&LISSBET, &[]);
}

#[allow(dead_code)]
fn unused() -> &'static muo_content::creature::CreatureType {
    &BASE_CREATURE_C
}
