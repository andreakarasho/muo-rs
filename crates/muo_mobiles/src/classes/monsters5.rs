//! Revenant (made by the Vengeful Spirit spell) and KhaldunRevenant: creatures
//! whose constructors take a target, never saved (`SkipSerialization`).

use muo_content::ai::{self, ActionType};
use muo_content::creature::{BASE_CREATURE, BASE_CREATURE_C, control};
use muo_content::mx;
use muo_primitives::{DateTime, Point3D, TimeSpan};
use muo_server::World;
use muo_server::effects::EffectLayer;
use muo_server::ids::{EntityId, MobileId};
use muo_server::item::{base as ibase, effect_item, ops};
use muo_server::mobile::part2::{effects, items};
use muo_server::skills::{self, SkillName};
use muo_server::type_registry::TypeRegistry;

use crate::common::*;
use crate::hand::si;
use crate::leaf::creature_leaf;

/// The fixed target and the end time (`m_Target`, `m_ExpireTime`).
#[derive(Clone, Copy)]
struct Bound {
    target: MobileId,
    expire: DateTime,
}

fn bound(w: &World, id: MobileId) -> Option<Bound> {
    w.mobile(id)?.ext.get::<Bound>().copied()
}

fn constant_focus(w: &World, id: MobileId) -> Option<MobileId> {
    bound(w, id).map(|b| b.target)
}

/// `AddItem(new T { Hue = hue, Movable = false })`.
fn add_fixed(w: &mut World, id: MobileId, name: &str, hue: i32) {
    if let Some(i) = muo_content::factory::make_item(w, &si(name)) {
        ibase::set_hue(w, i, hue);
        ops::set_movable(w, i, false);
        items::add_item(w, id, i);
    }
}

// ---- Revenant ----

/// `new Revenant(caster, target, duration)`.
pub fn new_revenant(
    w: &mut World,
    caster: MobileId,
    target: MobileId,
    duration: TimeSpan,
) -> MobileId {
    let id = new_creature(
        w,
        &REVENANT,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    body(w, id, 400);
    hue(w, id, 1);
    let mut scalar = skills::value(w, caster, SkillName::SpiritSpeak) * 0.01;
    let expire = w.now + duration;
    if let Some(m) = w.mobile_mut(id) {
        m.ext.insert(Bound { target, expire });
    }
    set_str(w, id, 200);
    set_dex(w, id, 150);
    set_int(w, id, 150);
    set_damage(w, id, 16, 17);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_skill(w, id, SkillName::MagicResist, 100.0 * scalar);
    set_skill(w, id, SkillName::Tactics, 100.0);
    set_skill(w, id, SkillName::Swords, 100.0 * scalar);
    set_skill(w, id, SkillName::DetectHidden, 75.0 * scalar);
    scalar /= 1.2;
    let s = (20.0 * scalar) as i32;
    set_resistance_range(w, id, ResistanceType::Physical, 40 + s, 50 + s);
    set_resistance_range(w, id, ResistanceType::Cold, 40 + s, 50 + s);
    set_resistance(w, id, ResistanceType::Fire, s);
    set_resistance(w, id, ResistanceType::Poison, 100);
    set_resistance_range(w, id, ResistanceType::Energy, 40 + s, 50 + s);
    fame(w, id, 0);
    karma(w, id, 0);
    control_slots(w, id, 3);
    virtual_armor(w, id, 32);
    add_fixed(w, id, "DeathShroud", 0x455);
    add_fixed(w, id, "Halberd", 1);
    id
}

fn revenant_on_think(w: &mut World, id: MobileId) {
    let Some(b) = bound(w, id) else { return };
    if !mx::alive(w, b.target) || w.now > b.expire {
        muo_server::mobile::base::kill(w, id);
        return;
    }
    if mx::map(w, id) != mx::map(w, b.target) || !mx::in_range(w, id, b.target, 15) {
        let from_map = mx::map(w, id);
        let from = mx::location(w, id);
        let to_map = mx::map(w, b.target);
        let mut to = mx::location(w, b.target);
        if let Some(tm) = to_map {
            for _ in 0..5 {
                let (x, y) = (to.x - 4 + rnd::below(w, 9), to.y - 4 + rnd::below(w, 9));
                let z = to.z;
                if muo_server::map::fit::can_spawn_mobile_at(w, tm, x, y, z) {
                    to = Point3D::new(x, y, z);
                    break;
                }
                let z = w.maps.get_average_z(tm, x, y);
                if muo_server::map::fit::can_spawn_mobile_at(w, tm, x, y, z) {
                    to = Point3D::new(x, y, z);
                    break;
                }
            }
        }
        muo_server::mobile::part1::lifecycle::set_map(w, id, to_map);
        muo_server::mobile::part1::lifecycle::set_location(w, id, to);
        muo_server::mobile::base::process_delta(w, id);
        let e = effect_item::create(w, from, from_map, effect_item::DEFAULT_DURATION);
        muo_server::effects::send_location_particles_full(w, e, 0x3728, 1, 13, 37, 7, 5023, 0);
        effects::fixed_particles_full(w, id, 0x3728, 1, 13, 5023, 37, 7, EffectLayer::Waist, 0);
        mx::play_sound(w, id, 0x37D);
    }
    let hidden = w.mobile(b.target).is_some_and(|m| m.hidden);
    let ready = w.tick_count - w.mobile(id).map_or(0, |m| m.next_skill_time) >= 0;
    if hidden
        && mx::in_range(w, id, b.target, 3)
        && ready
        && muo_server::mobile::base::use_skill(w, id, SkillName::DetectHidden)
        && let Some(t) = w.mobile(id).and_then(|m| m.target)
    {
        muo_server::target::invoke(
            w,
            t,
            id,
            muo_server::target::Targeted::Entity(EntityId::Mobile(id)),
        );
    }
    mx::set_combatant(w, id, Some(b.target));
    if let Some(d) = muo_content::creature::cd_mut(w, id) {
        d.focus_mob = Some(b.target);
    }
    ai::set_action(w, id, ActionType::Combat);
    (BASE_CREATURE_C.on_think)(w, id);
}

fn revenant_on_before_death(w: &mut World, id: MobileId) -> bool {
    let (loc, map) = (mx::location(w, id), mx::map(w, id));
    muo_server::effects::play_sound(w, loc, map, 0x10B);
    let e = effect_item::create(w, loc, map, TimeSpan::from_secs(10));
    muo_server::effects::send_location_particles_full(w, e, 0x37CC, 1, 50, 2101, 7, 9909, 0);
    if let Some(ty) = w.mobile(id).map(|m| m.ty) {
        (ty.delete)(w, id);
    }
    false
}

creature_leaf! {
    /// `Server.Mobiles.Revenant` (not constructible by the registry: `new_revenant`).
    REVENANT, REVENANT_C = "Server.Mobiles.Revenant" : BASE_CREATURE / BASE_CREATURE_C, ctor None, encoded false;
    c {
        default_name: |_w, _id| Some("a revenant"),
        constant_focus: constant_focus,
        no_house_restrictions: |_w, _id| true,
        dispel_difficulty: |_w, _id| 80.0,
        dispel_focus: |_w, _id| 20.0,
        always_murderer: |_w, _id| true,
        bleed_immune: |_w, _id| true,
        bard_immune: |_w, _id| true,
        poison_immune: |_w, _id| Some(PoisonId(4)),
        on_think: revenant_on_think,
    }
    m {
        skip_serialization: |_w, _id| true,
        ignore_mobiles: |_w, _id| true,
        display_paperdoll_to: |_w, _id, _to| {},
        on_before_death: revenant_on_before_death,
    }
}

// ---- KhaldunRevenant ----

/// `new KhaldunRevenant(target)`.
pub fn new_khaldun_revenant(w: &mut World, target: MobileId) -> MobileId {
    let id = new_creature(
        w,
        &KHALDUN_REVENANT,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    body(w, id, 0x3CA);
    hue(w, id, 0x41CE);
    let expire = w.now + TimeSpan::from_mins(10);
    if let Some(m) = w.mobile_mut(id) {
        m.ext.insert(Bound { target, expire });
    }
    set_str_range(w, id, 401, 500);
    set_dex_range(w, id, 296, 315);
    set_int_range(w, id, 101, 200);
    set_hits_range(w, id, 241, 300);
    set_stam_range(w, id, 242, 280);
    set_damage(w, id, 20, 30);
    set_damage_type(w, id, ResistanceType::Physical, 50);
    set_damage_type(w, id, ResistanceType::Cold, 50);
    set_skill_range(w, id, SkillName::MagicResist, 100.1, 150.0);
    set_skill_range(w, id, SkillName::Tactics, 90.1, 100.0);
    set_skill_range(w, id, SkillName::Swords, 140.1, 150.0);
    set_skill_range(w, id, SkillName::Wrestling, 90.1, 100.0);
    set_resistance_range(w, id, ResistanceType::Physical, 55, 65);
    set_resistance_range(w, id, ResistanceType::Fire, 30, 40);
    set_resistance_range(w, id, ResistanceType::Cold, 60, 70);
    set_resistance_range(w, id, ResistanceType::Poison, 20, 30);
    set_resistance_range(w, id, ResistanceType::Energy, 20, 30);
    fame(w, id, 0);
    karma(w, id, 0);
    virtual_armor(w, id, 60);
    add_fixed(w, id, "Halberd", 0x41CE);
    id
}

fn is_inside_khaldun(w: &World, m: Option<MobileId>) -> bool {
    m.and_then(|m| w.mobile(m))
        .is_some_and(|x| muo_server::region::is_part_of_name(w, x.region, "Khaldun", false))
}

/// `[OnEvent(nameof(PlayerMobile.PlayerDeathEvent))] OnPlayerDeathEvent(m)`.
pub fn on_player_death_event(w: &mut World, m: MobileId) {
    let mut last_killer = w.mobile(m).and_then(|x| x.last_killer);
    if let Some(k) = last_killer
        && muo_content::creature::cd(w, k).is_some()
    {
        last_killer = control::get_master(w, k);
    }
    let Some(killer) = last_killer else { return };
    if !is_inside_khaldun(w, Some(m))
        || !is_inside_khaldun(w, Some(killer))
        || !mx::player(w, killer)
        || tables(w).khaldun.contains(&killer)
    {
        return;
    }
    let reportable = w.mobile(m).is_some_and(|x| {
        x.aggressors
            .iter()
            .any(|ai| ai.attacker == killer && ai.can_report_murder)
    });
    if reportable {
        summon_revenant(w, m, killer);
    }
}

/// `SummonRevenant(victim, killer)`.
pub fn summon_revenant(w: &mut World, victim: MobileId, killer: MobileId) {
    let revenant = new_khaldun_revenant(w, killer);
    let (loc, map) = (mx::location(w, victim), mx::map(w, victim));
    if let Some(map) = map {
        muo_server::mobile::base::move_to_world(w, revenant, loc, map);
    }
    mx::set_combatant(w, revenant, Some(killer));
    effects::fixed_particles(w, revenant, 0, 0, 0, 0x13A7, EffectLayer::Waist);
    let (l, m) = (mx::location(w, revenant), mx::map(w, revenant));
    muo_server::effects::play_sound(w, l, m, 0x29);
    tables(w).khaldun.insert(killer);
}

fn khaldun_on_think(w: &mut World, id: MobileId) {
    let Some(b) = bound(w, id) else { return };
    if !mx::alive(w, b.target) || w.now > b.expire {
        if let Some(ty) = w.mobile(id).map(|m| m.ty) {
            (ty.delete)(w, id);
        }
        return;
    }
    ai::set_action(w, id, ActionType::Combat);
    (BASE_CREATURE_C.on_think)(w, id);
}

fn khaldun_on_before_death(w: &mut World, id: MobileId) -> bool {
    let (loc, map) = (mx::location(w, id), mx::map(w, id));
    muo_server::effects::send_location_effect(w, loc, map, 0x376A, 10, 1, 0, 0);
    true
}

fn khaldun_on_delete(w: &mut World, id: MobileId) {
    if let Some(b) = bound(w, id) {
        tables(w).khaldun.remove(&b.target);
    }
    (BASE_CREATURE.on_delete)(w, id);
}

creature_leaf! {
    /// `Server.Mobiles.KhaldunRevenant` (not constructible by the registry: `new_khaldun_revenant`).
    KHALDUN_REVENANT, KHALDUN_REVENANT_C = "Server.Mobiles.KhaldunRevenant" : BASE_CREATURE / BASE_CREATURE_C, ctor None, encoded false;
    c {
        default_name: |_w, _id| Some("a revenant"),
        delete_corpse_on_death: |_w, _id| true,
        constant_focus: constant_focus,
        always_attackable: |_w, _id| true,
        bard_immune: |_w, _id| true,
        poison_immune: |_w, _id| Some(PoisonId(4)),
        on_think: khaldun_on_think,
    }
    m {
        skip_serialization: |_w, _id| true,
        display_paperdoll_to: |_w, _id, _to| {},
        get_idle_sound: |_w, _id| 0x1BF,
        get_anger_sound: |_w, _id| 0x107,
        get_death_sound: |_w, _id| 0xFD,
        on_before_death: khaldun_on_before_death,
        on_delete: khaldun_on_delete,
    }
}

// ---- VorpalBunny.BunnyHole ----

/// `new BunnyHole()`: a hole that closes after forty seconds.
pub static BUNNY_HOLE: muo_server::item::ItemType = muo_server::item::ItemType {
    full_name: "Server.Mobiles.VorpalBunny+BunnyHole",
    parent: Some(&muo_server::item::ITEM_BASE),
    construct: None,
    make: Some(|w| {
        let id = ops::new_item_with_id(w, &BUNNY_HOLE, 0x913);
        ops::set_movable(w, id, false);
        ibase::set_hue(w, id, 1);
        muo_server::timer::start(w, TimeSpan::from_secs(40), TimeSpan::ZERO, 1, move |w| {
            ibase::delete(w, id)
        });
        id
    }),
    default_name: |_, _| Some("a mysterious rabbit hole".to_owned()),
    skip_serialization: |_, _| true,
    ..muo_server::item::ITEM_BASE
};

pub fn register(types: &mut TypeRegistry) {
    types.register_item(&BUNNY_HOLE, &[]);
    types.register_mobile(&REVENANT, &[]);
    types.register_mobile(&KHALDUN_REVENANT, &[]);
}

pub fn configure(w: &mut World) {
    w.resources
        .get_mut::<muo_content::spells::hooks::SpellHooks>()
        .new_revenant = Some(|w, caster, target, d| Some(new_revenant(w, caster, target, d)));
    muo_content::player_mobile::subscribe_death(w, on_player_death_event);
}

use muo_server::poison::PoisonId;
