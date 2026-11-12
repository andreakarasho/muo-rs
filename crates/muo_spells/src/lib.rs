//! The concrete spells: one `static` [`SpellType`] per spell class, built from `muo_content::spells`, and the items the
//! spells make (the fields, the wall of stone, the arcane focus). Each circle or school is a module
//! (`first`.. `eighth`, `necromancy`, `chivalry`, `bushido`, `ninjitsu`, `spellweaving`, `mysticism`).
//!
//! A spell's `Target(T)` is an `fn(&mut World, SpellId, Targeted)` given to `begin_spell_target`; its per-instance
//! state (RecallSpell's rune entry and runebook) is the spell's `ext`. The abstract bases (`PaladinSpell`,
//! `NecromancerSpell`..) are `SpellType`s whose `family` is the school's vtable (`PaladinType`..), which a concrete
//! class replaces with its own.

// The spells keep their branch order (identical branches, blocks in conditions, nested ifs) and argument lists.

pub mod bushido;
pub mod chivalry;
pub mod common;
#[rustfmt::skip] // generated, and tests/ctors.rs reads the text
mod ctor_tables;
pub mod eighth;
pub mod fields;
pub mod fifth;
pub mod first;
pub mod fourth;
pub mod gargoyle;
pub mod mysticism;
pub mod necromancy;
pub mod ninjitsu;
mod props_gen;
pub mod second;
pub mod seventh;
pub mod sixth;
pub mod spellweaving;
pub mod third;

use muo_content::core;
use muo_content::spells::registry::{Registered, registry};
use muo_server::World;
use muo_server::type_registry::{Phase, TypeRegistry};

/// `Initializer.Configure()` for the spells of this crate: `Register(id, typeof(..))`, and the hooks and event
/// handlers the spell classes own.
pub fn configure(w: &mut World) {
    // `new FireFieldItem(itemID, loc, caster, map, duration, val)` for the Firebomb.
    muo_content::items::common::hooks_mut(w).fire_field_item =
        Some(|w, item_id, loc, caster, map, duration, val| {
            caster.map(|c| {
                fourth::fire_field::new_fire_field_item(w, item_id, loc, c, map, duration, val, 2)
            })
        });
    first::reactive_armor::init(w);
    second::protection::init(w);
    fifth::incognito::init(w);
    fifth::magic_reflect::init(w);
    seventh::polymorph::init(w);
    chivalry::divine_fury::init(w);
    necromancy::init(w);
    bushido::init(w);
    ninjitsu::init(w);
    mysticism::init(w);
    spellweaving::init(w);
    init_shared_hooks(w);
    init_content_hooks(w);

    let r = registry(w);
    // First to Eighth circle
    r.register(0, Registered::Spell(&first::clumsy::CLUMSY_SPELL));
    r.register(1, Registered::Spell(&first::create_food::CREATE_FOOD_SPELL));
    r.register(2, Registered::Spell(&first::feeblemind::FEEBLEMIND_SPELL));
    r.register(3, Registered::Spell(&first::heal::HEAL_SPELL));
    r.register(4, Registered::Spell(&first::magic_arrow::MAGIC_ARROW_SPELL));
    r.register(5, Registered::Spell(&first::night_sight::NIGHT_SIGHT_SPELL));
    r.register(
        6,
        Registered::Spell(&first::reactive_armor::REACTIVE_ARMOR_SPELL),
    );
    r.register(7, Registered::Spell(&first::weaken::WEAKEN_SPELL));
    r.register(8, Registered::Spell(&second::agility::AGILITY_SPELL));
    r.register(9, Registered::Spell(&second::cunning::CUNNING_SPELL));
    r.register(10, Registered::Spell(&second::cure::CURE_SPELL));
    r.register(11, Registered::Spell(&second::harm::HARM_SPELL));
    r.register(12, Registered::Spell(&second::magic_trap::MAGIC_TRAP_SPELL));
    r.register(
        13,
        Registered::Spell(&second::remove_trap::REMOVE_TRAP_SPELL),
    );
    r.register(14, Registered::Spell(&second::protection::PROTECTION_SPELL));
    r.register(15, Registered::Spell(&second::strength::STRENGTH_SPELL));
    r.register(16, Registered::Spell(&third::bless::BLESS_SPELL));
    r.register(17, Registered::Spell(&third::fireball::FIREBALL_SPELL));
    r.register(18, Registered::Spell(&third::magic_lock::MAGIC_LOCK_SPELL));
    r.register(19, Registered::Spell(&third::poison::POISON_SPELL));
    r.register(
        20,
        Registered::Spell(&third::telekinesis::TELEKINESIS_SPELL),
    );
    r.register(21, Registered::Spell(&third::teleport::TELEPORT_SPELL));
    r.register(22, Registered::Spell(&third::unlock::UNLOCK_SPELL));
    r.register(
        23,
        Registered::Spell(&third::wall_of_stone::WALL_OF_STONE_SPELL),
    );
    r.register(24, Registered::Spell(&fourth::arch_cure::ARCH_CURE_SPELL));
    r.register(
        25,
        Registered::Spell(&fourth::arch_protection::ARCH_PROTECTION_SPELL),
    );
    r.register(26, Registered::Spell(&fourth::curse::CURSE_SPELL));
    r.register(27, Registered::Spell(&fourth::fire_field::FIRE_FIELD_SPELL));
    r.register(
        28,
        Registered::Spell(&fourth::greater_heal::GREATER_HEAL_SPELL),
    );
    r.register(29, Registered::Spell(&fourth::lightning::LIGHTNING_SPELL));
    r.register(30, Registered::Spell(&fourth::mana_drain::MANA_DRAIN_SPELL));
    r.register(31, Registered::Spell(&fourth::recall::RECALL_SPELL));
    r.register(
        32,
        Registered::Spell(&fifth::blade_spirits::BLADE_SPIRITS_SPELL),
    );
    r.register(
        33,
        Registered::Spell(&fifth::dispel_field::DISPEL_FIELD_SPELL),
    );
    r.register(34, Registered::Spell(&fifth::incognito::INCOGNITO_SPELL));
    r.register(
        35,
        Registered::Spell(&fifth::magic_reflect::MAGIC_REFLECT_SPELL),
    );
    r.register(36, Registered::Spell(&fifth::mind_blast::MIND_BLAST_SPELL));
    r.register(37, Registered::Spell(&fifth::paralyze::PARALYZE_SPELL));
    r.register(
        38,
        Registered::Spell(&fifth::poison_field::POISON_FIELD_SPELL),
    );
    r.register(
        39,
        Registered::Spell(&fifth::summon_creature::SUMMON_CREATURE_SPELL),
    );
    r.register(40, Registered::Spell(&sixth::dispel::DISPEL_SPELL));
    r.register(
        41,
        Registered::Spell(&sixth::energy_bolt::ENERGY_BOLT_SPELL),
    );
    r.register(42, Registered::Spell(&sixth::explosion::EXPLOSION_SPELL));
    r.register(
        43,
        Registered::Spell(&sixth::invisibility::INVISIBILITY_SPELL),
    );
    r.register(44, Registered::Spell(&sixth::mark::MARK_SPELL));
    r.register(45, Registered::Spell(&sixth::mass_curse::MASS_CURSE_SPELL));
    r.register(
        46,
        Registered::Spell(&sixth::paralyze_field::PARALYZE_FIELD_SPELL),
    );
    r.register(47, Registered::Spell(&sixth::reveal::REVEAL_SPELL));
    r.register(
        48,
        Registered::Spell(&seventh::chain_lightning::CHAIN_LIGHTNING_SPELL),
    );
    r.register(
        49,
        Registered::Spell(&seventh::energy_field::ENERGY_FIELD_SPELL),
    );
    r.register(
        50,
        Registered::Spell(&seventh::flame_strike::FLAME_STRIKE_SPELL),
    );
    r.register(
        51,
        Registered::Spell(&seventh::gate_travel::GATE_TRAVEL_SPELL),
    );
    r.register(
        52,
        Registered::Spell(&seventh::mana_vampire::MANA_VAMPIRE_SPELL),
    );
    r.register(
        53,
        Registered::Spell(&seventh::mass_dispel::MASS_DISPEL_SPELL),
    );
    r.register(
        54,
        Registered::Spell(&seventh::meteor_swarm::METEOR_SWARM_SPELL),
    );
    r.register(55, Registered::Spell(&seventh::polymorph::POLYMORPH_SPELL));
    r.register(56, Registered::Spell(&eighth::earthquake::EARTHQUAKE_SPELL));
    r.register(
        57,
        Registered::Spell(&eighth::energy_vortex::ENERGY_VORTEX_SPELL),
    );
    r.register(
        58,
        Registered::Spell(&eighth::resurrection::RESURRECTION_SPELL),
    );
    r.register(
        59,
        Registered::Spell(&eighth::elementals::AIR_ELEMENTAL_SPELL),
    );
    r.register(
        60,
        Registered::Spell(&eighth::elementals::SUMMON_DAEMON_SPELL),
    );
    r.register(
        61,
        Registered::Spell(&eighth::elementals::EARTH_ELEMENTAL_SPELL),
    );
    r.register(
        62,
        Registered::Spell(&eighth::elementals::FIRE_ELEMENTAL_SPELL),
    );
    r.register(
        63,
        Registered::Spell(&eighth::elementals::WATER_ELEMENTAL_SPELL),
    );

    let (aos, se, ml, sa) = (core::aos(w), core::se(w), core::ml(w), core::sa(w));
    let r = registry(w);
    if !aos {
        return;
    }

    // Necromancy spells
    r.register(
        100,
        Registered::Spell(&necromancy::animate_dead::ANIMATE_DEAD_SPELL),
    );
    r.register(
        101,
        Registered::Spell(&necromancy::blood_oath::BLOOD_OATH_SPELL),
    );
    r.register(
        102,
        Registered::Spell(&necromancy::corpse_skin::CORPSE_SKIN_SPELL),
    );
    r.register(
        103,
        Registered::Spell(&necromancy::curse_weapon::CURSE_WEAPON_SPELL),
    );
    r.register(
        104,
        Registered::Spell(&necromancy::evil_omen::EVIL_OMEN_SPELL),
    );
    r.register(
        105,
        Registered::Spell(&necromancy::forms::HORRIFIC_BEAST_SPELL),
    );
    r.register(106, Registered::Spell(&necromancy::forms::LICH_FORM_SPELL));
    r.register(
        107,
        Registered::Spell(&necromancy::mind_rot::MIND_ROT_SPELL),
    );
    r.register(
        108,
        Registered::Spell(&necromancy::pain_spike::PAIN_SPIKE_SPELL),
    );
    r.register(
        109,
        Registered::Spell(&necromancy::poison_strike::POISON_STRIKE_SPELL),
    );
    r.register(
        110,
        Registered::Spell(&necromancy::strangle::STRANGLE_SPELL),
    );
    r.register(
        111,
        Registered::Spell(&necromancy::summon_familiar::SUMMON_FAMILIAR_SPELL),
    );
    r.register(
        112,
        Registered::Spell(&necromancy::forms::VAMPIRIC_EMBRACE_SPELL),
    );
    r.register(
        113,
        Registered::Spell(&necromancy::vengeful_spirit::VENGEFUL_SPIRIT_SPELL),
    );
    r.register(114, Registered::Spell(&necromancy::wither::WITHER_SPELL));
    r.register(
        115,
        Registered::Spell(&necromancy::forms::WRAITH_FORM_SPELL),
    );

    if se {
        r.register(
            116,
            Registered::Spell(&necromancy::exorcism::EXORCISM_SPELL),
        );
    }

    // Paladin abilities
    r.register(
        200,
        Registered::Spell(&chivalry::cleanse_by_fire::CLEANSE_BY_FIRE_SPELL),
    );
    r.register(
        201,
        Registered::Spell(&chivalry::close_wounds::CLOSE_WOUNDS_SPELL),
    );
    r.register(
        202,
        Registered::Spell(&chivalry::consecrate_weapon::CONSECRATE_WEAPON_SPELL),
    );
    r.register(
        203,
        Registered::Spell(&chivalry::dispel_evil::DISPEL_EVIL_SPELL),
    );
    r.register(
        204,
        Registered::Spell(&chivalry::divine_fury::DIVINE_FURY_SPELL),
    );
    r.register(
        205,
        Registered::Spell(&chivalry::enemy_of_one::ENEMY_OF_ONE_SPELL),
    );
    r.register(
        206,
        Registered::Spell(&chivalry::holy_light::HOLY_LIGHT_SPELL),
    );
    r.register(
        207,
        Registered::Spell(&chivalry::noble_sacrifice::NOBLE_SACRIFICE_SPELL),
    );
    r.register(
        208,
        Registered::Spell(&chivalry::remove_curse::REMOVE_CURSE_SPELL),
    );
    r.register(
        209,
        Registered::Spell(&chivalry::sacred_journey::SACRED_JOURNEY_SPELL),
    );

    if se {
        // Samurai abilities
        r.register(
            400,
            Registered::Special(&bushido::moves::HONORABLE_EXECUTION),
        );
        r.register(
            401,
            Registered::Spell(&bushido::confidence::CONFIDENCE_SPELL),
        );
        r.register(402, Registered::Spell(&bushido::evasion::EVASION_SPELL));
        r.register(
            403,
            Registered::Spell(&bushido::counter_attack::COUNTER_ATTACK_SPELL),
        );
        r.register(404, Registered::Special(&bushido::moves::LIGHTNING_STRIKE));
        r.register(405, Registered::Special(&bushido::moves::MOMENTUM_STRIKE));

        // Ninja abilities
        r.register(500, Registered::Special(&ninjitsu::moves::FOCUS_ATTACK));
        r.register(501, Registered::Special(&ninjitsu::moves::DEATH_STRIKE));
        r.register(
            502,
            Registered::Spell(&ninjitsu::animal_form::ANIMAL_FORM_SPELL),
        );
        r.register(503, Registered::Special(&ninjitsu::moves::KI_ATTACK));
        r.register(504, Registered::Special(&ninjitsu::moves::SURPRISE_ATTACK));
        r.register(505, Registered::Special(&ninjitsu::moves::BACKSTAB));
        r.register(
            506,
            Registered::Spell(&ninjitsu::shadow_jump::SHADOWJUMP_SPELL),
        );
        r.register(
            507,
            Registered::Spell(&ninjitsu::mirror_image::MIRROR_IMAGE_SPELL),
        );
    }

    if ml {
        // Spellweaving spells
        r.register(
            600,
            Registered::Spell(&spellweaving::spells::ARCANE_CIRCLE_SPELL),
        );
        r.register(
            601,
            Registered::Spell(&spellweaving::spells::GIFT_OF_RENEWAL_SPELL),
        );
        r.register(
            602,
            Registered::Spell(&spellweaving::spells::IMMOLATING_WEAPON_SPELL),
        );
        r.register(
            603,
            Registered::Spell(&spellweaving::spells::ATTUNE_WEAPON_SPELL),
        );
        r.register(
            604,
            Registered::Spell(&spellweaving::spells::THUNDERSTORM_SPELL),
        );
        r.register(
            605,
            Registered::Spell(&spellweaving::summons::NATURE_FURY_SPELL),
        );
        r.register(
            606,
            Registered::Spell(&spellweaving::summons::SUMMON_FEY_SPELL),
        );
        r.register(
            607,
            Registered::Spell(&spellweaving::summons::SUMMON_FIEND_SPELL),
        );
        r.register(
            608,
            Registered::Spell(&spellweaving::forms::REAPER_FORM_SPELL),
        );
        r.register(
            610,
            Registered::Spell(&spellweaving::spells::ESSENCE_OF_WIND_SPELL),
        );
        r.register(
            612,
            Registered::Spell(&spellweaving::forms::ETHEREAL_VOYAGE_SPELL),
        );
        r.register(
            613,
            Registered::Spell(&spellweaving::spells::WORD_OF_DEATH_SPELL),
        );
        r.register(
            614,
            Registered::Spell(&spellweaving::spells::GIFT_OF_LIFE_SPELL),
        );
    }

    if sa {
        // Mysticism spells
        r.register(
            682,
            Registered::Spell(&mysticism::spells::EAGLE_STRIKE_SPELL),
        );
        r.register(
            683,
            Registered::Spell(&mysticism::spells::ANIMATED_WEAPON_SPELL),
        );
        r.register(684, Registered::Spell(&mysticism::spells::STONE_FORM_SPELL));
        r.register(
            687,
            Registered::Spell(&mysticism::spells::CLEANSING_WINDS_SPELL),
        );
        r.register(688, Registered::Spell(&mysticism::spells::BOMBARD_SPELL));
        r.register(
            689,
            Registered::Spell(&mysticism::spell_plague::SPELL_PLAGUE_SPELL),
        );
        r.register(690, Registered::Spell(&mysticism::spells::HAIL_STORM_SPELL));
        r.register(
            691,
            Registered::Spell(&mysticism::spells::NETHER_CYCLONE_SPELL),
        );
    }
}

/// The hooks of other systems that the spell crate's classes answer as a whole: the special moves the weapons read
/// through the move table, the Dispel check of the weapons, and the stamina system's pain-spike algorithm.
fn init_shared_hooks(w: &mut World) {
    use muo_content::items::weapon_hooks::WeaponHooks;
    use muo_content::misc::stamina_system::{DfAlgorithm, StaminaSystem};
    use muo_content::spells::special_move as sm;

    let h = w.resources.get_mut::<WeaponHooks>();
    h.dispel_check_resisted = Some(sixth::dispel::new_dispel_check_resisted);
    h.special_move_active = Some(|w, m| sm::get_current_move(w, m).is_some());
    h.special_move_clear = Some(sm::clear_current_move);
    h.special_move_on_before_swing = Some(|w, a, d| match sm::get_current_move(w, a) {
        Some(mv) => (mv.on_before_swing)(w, a, d),
        None => true,
    });
    h.special_move_accuracy_bonus = Some(|w, a| match sm::current_move_unchecked(w, a) {
        Some(mv) => (mv.get_accuracy_bonus)(w, a),
        None => 0,
    });
    h.special_move_damage_scalar = Some(|w, a, d| match sm::current_move_unchecked(w, a) {
        Some(mv) => (mv.get_damage_scalar)(w, a, d),
        None => 1.0,
    });
    h.special_move_on_before_damage = Some(|w, a, d| match sm::current_move_unchecked(w, a) {
        Some(mv) => (mv.on_before_damage)(w, a, d),
        None => true,
    });
    h.special_move_ignore_armor = Some(|w, a| match sm::current_move_unchecked(w, a) {
        Some(mv) => (mv.ignore_armor)(w, a),
        None => false,
    });
    h.special_move_property_bonus = Some(|w, a| match sm::current_move_unchecked(w, a) {
        Some(mv) => (mv.get_property_bonus)(w, a),
        None => 1.0,
    });
    h.special_move_on_hit = Some(|w, a, d, dmg| {
        if let Some(mv) = sm::current_move_unchecked(w, a) {
            (mv.on_hit)(w, a, d, dmg);
        }
    });
    h.special_move_on_miss = Some(|w, a, d| {
        if let Some(mv) = sm::current_move_unchecked(w, a) {
            (mv.on_miss)(w, a, d);
        }
    });

    // `Core.AOS`-independent hooks of the weapon system that the spell and skill code answers.
    h.discordance_effect = Some(muo_content::skills::bard::discordance_effect);
    h.add_buff = Some(|w, m, icon, cliloc, duration| {
        muo_content::player_mobile::buff::add_buff(
            w,
            m,
            muo_content::player_mobile::buff::BuffInfo::new(
                muo_content::player_mobile::buff::BuffIcon(icon as i16),
                cliloc,
            )
            .duration(duration),
        );
    });
    h.remove_buff = Some(|w, m, icon| {
        muo_content::player_mobile::buff::remove_buff(
            w,
            m,
            muo_content::player_mobile::buff::BuffIcon(icon as i16),
        );
    });

    muo_content::spells::hooks::hooks_mut(w).set_dfa = Some(|w, v| {
        w.resources.get_mut::<StaminaSystem>().dfa = if v == 1 {
            DfAlgorithm::PainSpike
        } else {
            DfAlgorithm::Standard
        };
    });
}

/// The hooks of creature, mobile and player code that call into the spells.
/// `AosSkillBonuses.CheckCancelMorph(m)`.
fn check_cancel_morph(w: &mut World, m: muo_server::ids::MobileId) {
    use muo_server::skills::{self, SkillName};
    // A transformation spell ends when its cast skill fell under the spell's minimum.
    if let Some(c) = muo_content::spells::helper::get_context(w, m) {
        let min = (c.spell.min_skill)(w, m);
        if skills::value(w, m, c.spell.cast_skill) < min {
            muo_content::spells::helper::remove_context(w, m, true);
        }
    }
    ninjitsu::animal_form::check_cancel_morph(w, m);
    if skills::value(w, m, SkillName::Magery) < 66.1 {
        seventh::polymorph::end_polymorph(w, m);
    }
    if skills::value(w, m, SkillName::Magery) < 38.1 {
        fifth::incognito::end_incognito(w, m);
    }
}

fn init_content_hooks(w: &mut World) {
    // Transformations `RegenRates` and `Poison` ask about.
    {
        use muo_content::misc::{poison_kinds::PoisonHooks, regen_rates::RegenHooks};
        use muo_content::spells::helper::under_transformation_of as under;
        use ninjitsu::animal_form::under_transformation_of as animal;
        let r = w.resources.get_mut::<RegenHooks>();
        r.horrific_beast = Some(|w, m| under(w, m, "Server.Spells.Necromancy.HorrificBeastSpell"));
        r.vampiric_embrace =
            Some(|w, m| under(w, m, "Server.Spells.Necromancy.VampiricEmbraceSpell"));
        r.lich_form = Some(|w, m| under(w, m, "Server.Spells.Necromancy.LichFormSpell"));
        r.dog_or_cat_form = Some(|w, m| animal(w, m, "Dog") || animal(w, m, "Cat"));
        r.kirin_form = Some(|w, m| animal(w, m, "Kirin"));
        let p = w.resources.get_mut::<PoisonHooks>();
        p.vampiric_embrace =
            Some(|w, m| under(w, m, "Server.Spells.Necromancy.VampiricEmbraceSpell"));
        p.unicorn_form = Some(|w, m| animal(w, m, "Unicorn"));
    }
    use muo_content::items::weapon_hooks::WeaponHooks;
    use muo_content::player_mobile::hooks::SystemHooks;
    use muo_content::spells::helper;
    use muo_content::spells::special_move as sm;

    muo_content::skills::animal::hooks_mut_taming(w).has_dark_wolf_familiar = Some(|w, tamer| {
        necromancy::summon_familiar::familiar_of(w, tamer).is_some_and(|bc| {
            w.mobile(bc).is_some_and(|m| !m.deleted)
                && muo_content::factory::mobile_is(w, bc, "Server.Mobiles.DarkWolfFamiliar")
        })
    });
    muo_content::items::equip_hooks::hooks_mut(w).check_cancel_morph = Some(check_cancel_morph);
    let c = muo_content::hooks::hooks_mut(w);
    c.creature_unregister_summon = Some(|w, bc, master, animated| {
        if animated {
            necromancy::animate_dead::unregister(w, master, bc);
        }
        if master.is_some() {
            necromancy::summon_familiar::unregister(w, master, bc);
        }
    });
    c.animate_dead_register = Some(necromancy::animate_dead::register);
    c.invisibility_stop_timer = Some(sixth::invisibility::stop_timer);
    c.death_adder_familiar_of = Some(|w, master| {
        necromancy::summon_familiar::familiar_of(w, master).filter(|&bc| {
            !w.mobile(bc).is_none_or(|m| m.deleted)
                && muo_content::factory::mobile_is(w, bc, "Server.Mobiles.DeathAdder")
        })
    });
    c.necro_familiar_of =
        Some(|w, master, bc| necromancy::summon_familiar::familiar_of(w, master) == Some(bc));
    c.valid_indirect_target = Some(helper::valid_indirect_target);

    let p = w.resources.get_mut::<SystemHooks>();
    p.confidence_stop_regenerating = Some(bushido::confidence::stop_regenerating);
    p.special_move_clear_all = Some(sm::clear_all_moves);
    p.special_move_clear_current = Some(sm::clear_current_move);
    p.transformation_under = Some(|w, m, name| {
        helper::get_context(w, m).is_some_and(|c| c.ty.rsplit('.').next() == Some(name))
    });
    p.invisibility_stop_timer = Some(sixth::invisibility::stop_timer);
    p.fly_cast = Some(gargoyle::fly_cast);
    p.dismount = Some(muo_content::creature::mount::dismount);
    p.death_strike_add_step = Some(ninjitsu::moves::add_step);
    p.curse_under_effect = Some(fourth::curse::under_effect_ref);

    // `Backpack?.FindItemByType<Bandage>()` then `Consume()` (the Cu Sidhe form heals with it).
    muo_content::spells::hooks::hooks_mut(w).consume_bandage = Some(|w, m| {
        let Some(pack) = w.mobile(m).and_then(|x| x.backpack) else {
            return false;
        };
        let Some(muo_server::type_registry::EntityType::Item(ty)) = w
            .types
            .find_type_by_full_name("Server.Items.Bandage", false)
        else {
            return false;
        };
        let Some(b) = muo_server::item::container::find_item_by_type(w, pack, ty, true) else {
            return false;
        };
        muo_server::item::base::consume(w, b, 1);
        true
    });

    let h = w.resources.get_mut::<WeaponHooks>();
    h.under_horrific_beast = Some(|w, m| {
        helper::under_transformation_of(w, m, "Server.Spells.Necromancy.HorrificBeastSpell")
    });
    h.in_necromancer_form_not_horrific = Some(|w, m| {
        helper::get_context(w, m).is_some_and(|c| {
            c.ty.starts_with("Server.Spells.Necromancy.")
                && c.ty != "Server.Spells.Necromancy.HorrificBeastSpell"
        })
    });
    h.bleed_immune_form = Some(|w, m| {
        helper::under_transformation_of(w, m, "Server.Spells.Necromancy.LichFormSpell")
            || helper::under_transformation_of(w, m, "Server.Spells.Necromancy.WraithFormSpell")
    });
    h.spell_helper_damage = Some(|w, delay, target, from, damage, split| {
        helper::damage_aos(
            w,
            None,
            delay,
            target,
            Some(from),
            damage,
            helper::SpellDamageSplit {
                phys: split[0],
                fire: split[1],
                cold: split[2],
                pois: split[3],
                nrgy: split[4],
                ..Default::default()
            },
        );
    });
    h.spell_helper_valid_indirect_target = Some(helper::valid_indirect_target);
    h.spell_blocks_movement = Some(|w, m| {
        w.mobile(m).and_then(|x| x.spell).is_some_and(|s| {
            muo_content::spells::spell::is_casting(w, s)
                && muo_content::spells::spell::spell_blocks_movement(w, s)
        })
    });
    h.mind_rot_scalar = Some(necromancy::mind_rot::scalar_of);
}

/// Registers this crate's types and phases at startup.
pub fn register(types: &mut TypeRegistry) {
    types.register_phase_default(
        muo_server::type_registry::Phase::Configure,
        "muo_spells.PropsGen.Configure",
        props_gen::configure,
    );
    ctor_tables::register(types);
    types.register_item(&third::wall_of_stone::WALL_OF_STONE, &[]);
    types.register_item(&fourth::fire_field::FIRE_FIELD_ITEM, &[]);
    types.register_item(&fifth::poison_field::POISON_FIELD, &[]);
    types.register_item(&sixth::paralyze_field::PARALYZE_FIELD, &[]);
    types.register_item(&seventh::energy_field::ENERGY_FIELD, &[]);
    types.register_item(&spellweaving::items::TRANSIENT_ITEM, &[]);
    types.register_item(&spellweaving::items::ARCANE_FOCUS, &[]);
    spellweaving::mobiles::register(types);
    ninjitsu::clone::register(types);
    types.register_phase_default(
        Phase::Configure,
        "Server.Spells.Initializer.Configure",
        configure,
    );
}

#[cfg(test)]
mod flow_tests;
#[cfg(test)]
mod tests;
