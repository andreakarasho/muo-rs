//! `Golem`, `PlagueSpawn` and `Leviathan`: classes whose constructors take
//! parameters with defaults (the registry builds them with the defaults; the `[add` tables and callers use the
//! `new_*` functions).

use muo_content::abilities::{self, MonsterAbility};
use muo_content::creature::{BASE_CREATURE, BASE_CREATURE_C, cd, control};
use muo_content::factory;
use muo_content::loot;
use muo_content::mx;
use muo_content::props::{Ctor, Kind, Param, Value};
use muo_content::utilities::types::PropType;
use muo_primitives::{DateTime, TimeSpan};
use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::mobile::part1::props_b;
use muo_server::poison::PoisonId;
use muo_server::type_registry::TypeRegistry;

use crate::common::*;
use crate::hand::si;
use crate::leaf::creature_leaf;
use crate::npcs::special::champions::{BASE_CHAMPION, BASE_CHAMPION_C};

const LETHAL: PoisonId = PoisonId(4);

fn to_f(scalar: f64, v: i32) -> i32 {
    (f64::from(v) * scalar) as i32
}

// ---- Golem ----

static GOLEM_ABILITIES: [&MonsterAbility; 1] = [&abilities::COLOSSAL_BLOW];

/// `new Golem(summoned, scalar)`.
pub fn new_golem(w: &mut World, summoned: bool, scalar: f64) -> MobileId {
    let id = new_creature(
        w,
        &GOLEM,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    body(w, id, 752);
    if summoned {
        hue(w, id, 2101);
    }
    set_str_range(w, id, to_f(scalar, 251), to_f(scalar, 350));
    set_dex_range(w, id, to_f(scalar, 76), to_f(scalar, 100));
    set_int_range(w, id, to_f(scalar, 101), to_f(scalar, 150));
    set_hits_range(w, id, to_f(scalar, 151), to_f(scalar, 210));
    set_damage(w, id, to_f(scalar, 13), to_f(scalar, 24));
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(
        w,
        id,
        ResistanceType::Physical,
        to_f(scalar, 35),
        to_f(scalar, 55),
    );
    if summoned {
        set_resistance_range(
            w,
            id,
            ResistanceType::Fire,
            to_f(scalar, 50),
            to_f(scalar, 60),
        );
    } else {
        set_resistance(w, id, ResistanceType::Fire, to_f(scalar, 100));
    }
    set_resistance_range(
        w,
        id,
        ResistanceType::Cold,
        to_f(scalar, 10),
        to_f(scalar, 30),
    );
    set_resistance_range(
        w,
        id,
        ResistanceType::Poison,
        to_f(scalar, 10),
        to_f(scalar, 25),
    );
    set_resistance_range(
        w,
        id,
        ResistanceType::Energy,
        to_f(scalar, 30),
        to_f(scalar, 40),
    );
    set_skill_range(
        w,
        id,
        SkillName::MagicResist,
        150.1 * scalar,
        190.0 * scalar,
    );
    set_skill_range(w, id, SkillName::Tactics, 60.1 * scalar, 100.0 * scalar);
    set_skill_range(w, id, SkillName::Wrestling, 60.1 * scalar, 100.0 * scalar);
    if summoned {
        fame(w, id, 10);
        karma(w, id, 10);
    } else {
        fame(w, id, 3500);
        karma(w, id, -3500);
    }
    if !summoned {
        let n = rnd::min_max(w, 13, 21);
        pack_new_amount(w, id, &si("IronIngot"), n);
        if rnd::double(w) < 0.1 {
            pack_new(w, id, &si("PowerCrystal"));
        }
        if rnd::double(w) < 0.15 {
            pack_new(w, id, &si("ClockworkAssembly"));
        }
        if rnd::double(w) < 0.2 {
            pack_new(w, id, &si("ArcaneGem"));
        }
        if rnd::double(w) < 0.25 {
            pack_new(w, id, &si("Gears"));
        }
    }
    control_slots(w, id, 3);
    id
}

fn controlled(w: &World, id: MobileId) -> bool {
    cd(w, id).is_some_and(|d| d.controlled)
}

/// `OnDamage`: a player master with the golem in range pays the damage from mana first.
fn golem_on_damage(
    w: &mut World,
    id: MobileId,
    amount: i32,
    from: Option<MobileId>,
    will_kill: bool,
) {
    let mut amount = amount;
    if let Some(master) = control::get_master(w, id).filter(|&m| mx::player(w, m))
        && mx::map(w, master) == mx::map(w, id)
        && mx::in_range_point(w, master, mx::location(w, id), 20)
    {
        let mana = w.mobile(master).map_or(0, |m| m.mana);
        if mana >= amount {
            props_b::set_mana(w, master, mana - amount);
        } else {
            amount -= mana;
            props_b::set_mana(w, master, 0);
            mx::damage(w, master, amount, None);
        }
    }
    (BASE_CREATURE.on_damage)(w, id, amount, from, will_kill);
}

creature_leaf! {
    /// `Server.Mobiles.Golem`.
    GOLEM, GOLEM_C = "Server.Mobiles.Golem" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(|w| new_golem(w, false, 1.0)), encoded false;
    c {
        corpse_name: muo_content::templates::creature_props::corpse_name,
        is_scared_of_scary_things: crate::cprops::is_scared_of_scary_things,
        is_scary_to_pets: crate::cprops::is_scary_to_pets,
        is_bondable: crate::cprops::is_bondable,
        favorite_food: |_w, _id| FoodType::empty(),
        default_name: muo_content::templates::creature_props::default_name,
        delete_on_release: crate::cprops::delete_on_release,
        auto_dispel: |w, id| !controlled(w, id),
        bleed_immune: muo_content::templates::creature_props::bleed_immune,
        bard_immune: |w, id| !w.core_aos() || controlled(w, id),
        poison_immune: |_w, _id| Some(LETHAL),
        get_monster_abilities: |_, _| &GOLEM_ABILITIES,
    }
    m {
        on_death: crate::hand::dawns_gear_on_death,
        get_anger_sound: |_w, _id| 541,
        get_idle_sound: |w, id| if !controlled(w, id) { 542 } else { (BASE_CREATURE.get_idle_sound)(w, id) },
        get_death_sound: |w, id| if !controlled(w, id) { 545 } else { (BASE_CREATURE.get_death_sound)(w, id) },
        get_attack_sound: |_w, _id| 562,
        get_hurt_sound: |w, id| if controlled(w, id) { 320 } else { (BASE_CREATURE.get_hurt_sound)(w, id) },
        on_damage: golem_on_damage,
    }
}

// ---- PlagueSpawn ----

/// `Owner` and `ExpireTime` (not saved).
#[derive(Clone, Copy)]
pub struct PlagueSpawnState {
    pub owner: Option<MobileId>,
    pub expire: DateTime,
}

/// `new PlagueSpawn(owner)`.
pub fn new_plague_spawn(w: &mut World, owner: Option<MobileId>) -> MobileId {
    let id = new_creature(
        w,
        &PLAGUE_SPAWN,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    let expire = w.now + TimeSpan::from_mins(1);
    if let Some(m) = w.mobile_mut(id) {
        m.ext.insert(PlagueSpawnState { owner, expire });
    }
    let h = rnd::from(w, 0x11, 15);
    hue(w, id, h);
    match rnd::below(w, 12) {
        0 => {
            body(w, id, 14);
            base_sound_id(w, id, 268);
        }
        1 => {
            body(w, id, 31);
            base_sound_id(w, id, 0x39D);
        }
        2 => body_list(w, id, &[400, 401]),
        3 => {
            body(w, id, 0x1D);
            base_sound_id(w, id, 0x9E);
        }
        4 => {
            body(w, id, 0x15);
            base_sound_id(w, id, 0xDB);
        }
        _ => {
            body(w, id, 51);
            base_sound_id(w, id, 456);
        }
    }
    set_str_range(w, id, 201, 300);
    set_dex(w, id, 80);
    set_int_range(w, id, 16, 20);
    set_hits_range(w, id, 121, 180);
    set_damage(w, id, 11, 17);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 35, 45);
    set_resistance_range(w, id, ResistanceType::Fire, 30, 40);
    set_resistance_range(w, id, ResistanceType::Cold, 25, 35);
    set_resistance_range(w, id, ResistanceType::Poison, 65, 75);
    set_resistance_range(w, id, ResistanceType::Energy, 25, 35);
    set_skill(w, id, SkillName::MagicResist, 25.0);
    set_skill(w, id, SkillName::Tactics, 25.0);
    set_skill(w, id, SkillName::Wrestling, 50.0);
    fame(w, id, 1000);
    karma(w, id, -1000);
    virtual_armor(w, id, 20);
    id
}

fn plague_spawn_state(w: &World, id: MobileId) -> Option<PlagueSpawnState> {
    w.mobile(id)?.ext.get::<PlagueSpawnState>().copied()
}

fn plague_spawn_on_think(w: &mut World, id: MobileId) {
    if let Some(PlagueSpawnState {
        owner: Some(o),
        expire,
    }) = plague_spawn_state(w, id)
        && (w.now >= expire
            || mx::deleted(w, o)
            || mx::map(w, id) != mx::map(w, o)
            || !mx::in_range(w, id, o, 16))
    {
        let snd = w
            .mobile(id)
            .map(|m| m.ty)
            .map_or(0, |m| (m.get_idle_sound)(w, id));
        mx::play_sound(w, id, snd);
        muo_server::mobile::base::delete(w, id);
        return;
    }
    (BASE_CREATURE_C.on_think)(w, id);
}

creature_leaf! {
    /// `Server.Mobiles.PlagueSpawn`.
    PLAGUE_SPAWN, PLAGUE_SPAWN_C = "Server.Mobiles.PlagueSpawn" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(|w| new_plague_spawn(w, None)), encoded false;
    c {
        corpse_name: muo_content::templates::creature_props::corpse_name,
        always_murderer: muo_content::templates::creature_props::always_murderer,
        default_name: muo_content::templates::creature_props::default_name,
        on_think: plague_spawn_on_think,
        generate_loot: muo_content::templates::creature_props::generate_loot,
    }
    m {
        display_paperdoll_to: crate::hand::restless_soul_display_paperdoll_to,
        get_context_menu_entries: crate::hand::restless_soul_get_context_menu_entries,
    }
}

// ---- Leviathan ----

/// `Fisher` (not saved).
#[derive(Clone, Copy)]
struct Fisher(Option<MobileId>);

/// `Leviathan.Artifacts`.
pub const LEVIATHAN_ARTIFACTS: [&str; 22] = [
    "Server.Items.CandelabraOfSouls",
    "Server.Items.GhostShipAnchor",
    "Server.Items.GoldBricks",
    "Server.Items.PhillipsWoodenSteed",
    "Server.Items.SeahorseStatuette",
    "Server.Items.ShipModelOfTheHMSCape",
    "Server.Items.AdmiralsHeartyRum",
    "Server.Items.AlchemistsBauble",
    "Server.Items.ArcticDeathDealer",
    "Server.Items.BlazeOfDeath",
    "Server.Items.BurglarsBandana",
    "Server.Items.CaptainQuacklebushsCutlass",
    "Server.Items.CavortingClub",
    "Server.Items.DreadPirateHat",
    "Server.Items.EnchantedTitanLegBone",
    "Server.Items.GwennosHarp",
    "Server.Items.IolosLute",
    "Server.Items.LunaLance",
    "Server.Items.NightsKiss",
    "Server.Items.NoxRangersHeavyCrossbow",
    "Server.Items.PolarBearMask",
    "Server.Items.VioletCourage",
];

/// `new Leviathan(fisher)`.
pub fn new_leviathan(w: &mut World, fisher: Option<MobileId>) -> MobileId {
    let id = new_creature(
        w,
        &LEVIATHAN,
        AIType::AiMage,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    set_fisher(w, id, fisher);
    body(w, id, 77);
    base_sound_id(w, id, 353);
    hue(w, id, 0x481);
    set_str(w, id, 1000);
    set_dex_range(w, id, 501, 520);
    set_int_range(w, id, 501, 515);
    set_hits(w, id, 1500);
    set_damage(w, id, 25, 33);
    set_damage_type(w, id, ResistanceType::Physical, 70);
    set_damage_type(w, id, ResistanceType::Cold, 30);
    set_resistance_range(w, id, ResistanceType::Physical, 55, 65);
    set_resistance_range(w, id, ResistanceType::Fire, 45, 55);
    set_resistance_range(w, id, ResistanceType::Cold, 45, 55);
    set_resistance_range(w, id, ResistanceType::Poison, 35, 45);
    set_resistance_range(w, id, ResistanceType::Energy, 25, 35);
    for s in [
        SkillName::EvalInt,
        SkillName::Magery,
        SkillName::MagicResist,
        SkillName::Meditation,
        SkillName::Tactics,
        SkillName::Wrestling,
    ] {
        set_skill_range(w, id, s, 97.6, 107.5);
    }
    fame(w, id, 24000);
    karma(w, id, -24000);
    virtual_armor(w, id, 50);
    can_swim(w, id, true);
    cant_walk(w, id, true);
    for item_id in [0x14F8, 0x14FA] {
        let rope = factory::make_item(w, &si("Rope"));
        if let Some(r) = rope {
            muo_server::item::base::set_item_id(w, r, item_id);
        }
        pack_item(w, id, rope);
    }
    id
}

pub fn set_fisher(w: &mut World, id: MobileId, fisher: Option<MobileId>) {
    if let Some(m) = w.mobile_mut(id) {
        m.ext.insert(Fisher(fisher));
    }
}

fn fisher(w: &World, id: MobileId) -> Option<MobileId> {
    w.mobile(id)?.ext.get::<Fisher>().and_then(|f| f.0)
}

/// `Leviathan.GiveArtifactTo(m)`.
pub fn give_artifact_to(w: &mut World, m: MobileId) {
    let Some(item) = loot::construct_any(w, &LEVIATHAN_ARTIFACTS) else {
        return;
    };
    // TODO: Confirm messages
    if muo_server::mobile::part2::items::add_to_backpack(w, m, item) {
        mx::send_message(
            w,
            m,
            "As a reward for slaying the mighty leviathan, an artifact has been placed in your backpack.",
        );
    } else {
        mx::send_message(
            w,
            m,
            "As your backpack is full, your reward for destroying the legendary leviathan has been placed at your feet.",
        );
    }
}

fn leviathan_on_before_death(w: &mut World, id: MobileId) -> bool {
    let map = if mx::map(w, id) == Some(muo_server::map::MapId::FELUCCA) {
        muo_server::map::MapId::FELUCCA
    } else {
        muo_server::map::MapId::TRAMMEL
    };
    let bottle = factory::make_item_args(w, &si("MessageInABottle"), &[Value::Map(Some(map))]);
    pack_item(w, id, bottle);
    (BASE_CREATURE.on_before_death)(w, id)
}

fn leviathan_on_killed_by(w: &mut World, id: MobileId, mob: MobileId) {
    (BASE_CREATURE_C.on_killed_by)(w, id, mob);
    if crate::npcs::special::paragon::check_artifact_chance(w, mob, id) {
        give_artifact_to(w, mob);
        if fisher(w, id) == Some(mob) {
            set_fisher(w, id, None);
        }
    }
}

fn leviathan_on_death(w: &mut World, id: MobileId, c: Option<ItemId>) {
    (BASE_CREATURE.on_death)(w, id, c);
    if let Some(f) = fisher(w, id)
        && w.core_se()
        && muo_server::skills::base(w, f, SkillName::Fishing) >= 100.0
        && rnd::below(w, 100) < 25
    {
        give_artifact_to(w, f);
    }
    set_fisher(w, id, None);
}

/// `LeviathanBreath : FireBreath` (physical 70, cold 30, hue 0x1ED, damage scalar 0.05, 5-7.5 second cooldown).
static LEVIATHAN_BREATH: MonsterAbility = MonsterAbility {
    name: "LeviathanBreath",
    min_cooldown: TimeSpan::from_secs(5),
    max_cooldown: TimeSpan::from_millis(7500),
    ..abilities::FIRE_BREATH
};
static LEVIATHAN_ABILITIES: [&MonsterAbility; 1] = [&LEVIATHAN_BREATH];

creature_leaf! {
    /// `Server.Mobiles.Leviathan`.
    LEVIATHAN, LEVIATHAN_C = "Server.Mobiles.Leviathan" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(|w| new_leviathan(w, None)), encoded false;
    c {
        corpse_name: muo_content::templates::creature_props::corpse_name,
        default_name: muo_content::templates::creature_props::default_name,
        treasure_map_level: muo_content::templates::creature_props::treasure_map_level,
        treasure_map_chance: crate::cprops::treasure_map_chance,
        get_monster_abilities: |_, _| &LEVIATHAN_ABILITIES,
        on_killed_by: leviathan_on_killed_by,
        generate_loot: muo_content::templates::creature_props::generate_loot,
    }
    m {
        on_before_death: leviathan_on_before_death,
        on_death: leviathan_on_death,
    }
}

// ---- Twaulo (a BaseChampion; the lists are in npcs/special/champions.rs) ----

static TWAULO_ABILITIES: [&MonsterAbility; 1] = [&abilities::SUMMON_PIXIES_COUNTER];

fn twaulo_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &TWAULO)
}

/// `base.OnGaveMeleeAttack(..)`, then 10-29 damage and as much stamina and mana drained.
fn twaulo_on_gave_melee_attack(w: &mut World, id: MobileId, defender: MobileId, damage: i32) {
    (crate::npcs::special::champions::BASE_CHAMPION_C.on_gave_melee_attack)(
        w, id, defender, damage,
    );
    let d = rnd::from(w, 20, 10);
    mx::damage(w, defender, d, Some(id));
    let s = rnd::from(w, 20, 10);
    let stam = w.mobile(defender).map_or(0, |m| m.stam);
    props_b::set_stam(w, defender, stam - s);
    let m = rnd::from(w, 20, 10);
    let mana = w.mobile(defender).map_or(0, |x| x.mana);
    props_b::set_mana(w, defender, mana - m);
}

creature_leaf! {
    /// `Server.Mobiles.Twaulo`.
    TWAULO, TWAULO_C = "Server.Mobiles.Twaulo" : BASE_CHAMPION / BASE_CHAMPION_C, ctor Some(twaulo_new), encoded false;
    c {
        corpse_name: muo_content::templates::creature_props::corpse_name,
        default_name: muo_content::templates::creature_props::default_name,
        opposition_group: crate::cprops::opposition_group,
        unprovokable: crate::cprops::unprovokable,
        poison_immune: muo_content::templates::creature_props::poison_immune,
        treasure_map_level: muo_content::templates::creature_props::treasure_map_level,
        meat: muo_content::templates::creature_props::meat,
        hides: muo_content::templates::creature_props::hides,
        hide_type: crate::cprops::hide_type,
        get_monster_abilities: |_, _| &TWAULO_ABILITIES,
        generate_loot: muo_content::templates::creature_props::generate_loot,
        on_gave_melee_attack: twaulo_on_gave_melee_attack,
    }
    m {
    }
}

// ---- the `[add` overloads ----

static GOLEM_CTORS: [Ctor; 1] = [Ctor {
    access: None,
    params: &[
        Param {
            name: "summoned",
            kind: Kind::Plain(PropType::Bool),
            default: Some(Value::Bool(false)),
            ty: "",
        },
        Param {
            name: "scalar",
            kind: Kind::Plain(PropType::Double),
            default: Some(Value::Float(1.0)),
            ty: "",
        },
    ],
    make: |w, v| {
        let summoned = matches!(v.first(), Some(Value::Bool(true)));
        let scalar = match v.get(1) {
            Some(Value::Float(f)) => *f,
            _ => 1.0,
        };
        EntityId::Mobile(new_golem(w, summoned, scalar))
    },
}];

fn entity_arg(v: &[Value]) -> Option<MobileId> {
    match v.first() {
        Some(Value::Entity(Some(EntityId::Mobile(m)))) => Some(*m),
        _ => None,
    }
}

static PLAGUE_SPAWN_CTORS: [Ctor; 1] = [Ctor {
    access: None,
    params: &[Param {
        name: "owner",
        kind: Kind::Plain(PropType::Entity),
        default: Some(Value::Null),
        ty: "Mobile",
    }],
    make: |w, v| EntityId::Mobile(new_plague_spawn(w, entity_arg(v))),
}];

static LEVIATHAN_CTORS: [Ctor; 1] = [Ctor {
    access: None,
    params: &[Param {
        name: "fisher",
        kind: Kind::Plain(PropType::Entity),
        default: Some(Value::Null),
        ty: "Mobile",
    }],
    make: |w, v| EntityId::Mobile(new_leviathan(w, entity_arg(v))),
}];

fn configure(w: &mut World) {
    muo_content::props::add_ctors(w, "Server.Mobiles.Golem", &GOLEM_CTORS);
    muo_content::props::add_ctors(w, "Server.Mobiles.PlagueSpawn", &PLAGUE_SPAWN_CTORS);
    muo_content::props::add_ctors(w, "Server.Mobiles.Leviathan", &LEVIATHAN_CTORS);
}

pub fn register(types: &mut TypeRegistry) {
    for t in [&GOLEM, &PLAGUE_SPAWN, &LEVIATHAN, &TWAULO] {
        types.register_mobile(t, &[]);
    }
    types.register_phase_default(
        muo_server::type_registry::Phase::Configure,
        "Server.Mobiles.Misc4.Configure",
        configure,
    );
}
