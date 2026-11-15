//! `BaseChampion` and the champions Barracoon, Mephitis, Rikktor, Semidar, Serado, Neira and Lord Oaks.
//! `BaseChampion` is `[SerializationGenerator(0, false)]`: its part is "BaseCreature + version int 0".

use muo_content::creature::data::cd_mut;
use muo_content::creature::death::get_looting_rights;
use muo_content::creature::{BASE_CREATURE, BASE_CREATURE_C, cd, speeds};
use muo_content::hooks::rnd;
use muo_content::misc::aos_helpers::{self, DamageSplit};
use muo_content::{factory, loot, mx};
use muo_io::{GenericReader, GenericWriter};
use muo_primitives::{Point3D, TimeSpan};
use muo_server::World;
use muo_server::effects::EffectLayer;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::{LootType, ops};
use muo_server::layer::Layer;
use muo_server::map::MapId;
use muo_server::mobile::MobileType;
use muo_server::mobile::part1::{combat, props_b};
use muo_server::mobile::part2::{effects, items, messages};
use muo_server::poison::PoisonId;
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::type_registry::TypeRegistry;

use super::hooks;
use crate::common::*;

const DEADLY: PoisonId = PoisonId(3);
const LETHAL: PoisonId = PoisonId(4);

/// `MonsterStatuetteType` numbers.
const SLIME: i32 = 27;
const SPIDER: i32 = 30;
const OPHIDIAN_ARCH_MAGE: i32 = 31;
const OPHIDIAN_WARRIOR: i32 = 32;

/// `ChampionSkullType` numbers.
const POWER: i32 = 0;
const ENLIGHTENMENT: i32 = 1;
const VENOM: i32 = 2;
const PAIN: i32 = 3;
const GREED: i32 = 4;
const DEATH: i32 = 5;

/// What the subclasses of `BaseChampion` override as constants.
pub struct ChampionInfo {
    pub skull: i32,
    pub unique: &'static [&'static str],
    pub shared: &'static [&'static str],
    pub decorative: &'static [&'static str],
    pub statues: &'static [i32],
}

static CHAMPIONS: &[(&str, ChampionInfo)] = &[
    (
        "Server.Mobiles.Barracoon",
        ChampionInfo {
            skull: GREED,
            unique: &["Server.Items.FangOfRactus"],
            shared: &[
                "Server.Items.EmbroideredOakLeafCloak",
                "Server.Items.DjinnisRing",
                "Server.Items.DetectiveBoots",
                "Server.Items.GuantletsOfAnger",
            ],
            decorative: &["Server.Items.SwampTile", "Server.Items.MonsterStatuette"],
            statues: &[SLIME],
        },
    ),
    (
        "Server.Mobiles.Mephitis",
        ChampionInfo {
            skull: VENOM,
            unique: &["Server.Items.Calm"],
            shared: &[
                "Server.Items.OblivionsNeedle",
                "Server.Items.ANecromancerShroud",
                "Server.Items.EmbroideredOakLeafCloak",
                "Server.Items.TheMostKnowledgePerson",
            ],
            decorative: &["Server.Items.Web", "Server.Items.MonsterStatuette"],
            statues: &[SPIDER],
        },
    ),
    (
        "Server.Mobiles.Semidar",
        ChampionInfo {
            skull: PAIN,
            unique: &["Server.Items.GladiatorsCollar"],
            shared: &[
                "Server.Items.RoyalGuardSurvivalKnife",
                "Server.Items.ANecromancerShroud",
                "Server.Items.LieutenantOfTheBritannianRoyalGuard",
            ],
            decorative: &["Server.Items.LavaTile", "Server.Items.DemonSkull"],
            statues: &[],
        },
    ),
    (
        "Server.Mobiles.Rikktor",
        ChampionInfo {
            skull: POWER,
            unique: &["Server.Items.CrownOfTalKeesh"],
            shared: &[
                "Server.Items.TheMostKnowledgePerson",
                "Server.Items.BraveKnightOfTheBritannia",
                "Server.Items.LieutenantOfTheBritannianRoyalGuard",
            ],
            decorative: &[
                "Server.Items.LavaTile",
                "Server.Items.MonsterStatuette",
                "Server.Items.MonsterStatuette",
            ],
            statues: &[OPHIDIAN_ARCH_MAGE, OPHIDIAN_WARRIOR],
        },
    ),
    (
        "Server.Mobiles.Serado",
        ChampionInfo {
            skull: POWER,
            unique: &["Server.Items.Pacify"],
            shared: &[
                "Server.Items.BraveKnightOfTheBritannia",
                "Server.Items.DetectiveBoots",
                "Server.Items.EmbroideredOakLeafCloak",
                "Server.Items.LieutenantOfTheBritannianRoyalGuard",
            ],
            decorative: &["Server.Items.Futon", "Server.Items.SwampTile"],
            statues: &[],
        },
    ),
    (
        "Server.Mobiles.Neira",
        ChampionInfo {
            skull: DEATH,
            unique: &["Server.Items.ShroudOfDeciet"],
            shared: &[
                "Server.Items.ANecromancerShroud",
                "Server.Items.CaptainJohnsHat",
            ],
            decorative: &[
                "Server.Items.WallBlood",
                "Server.Items.TatteredAncientMummyWrapping",
            ],
            statues: &[],
        },
    ),
    (
        "Server.Mobiles.LordOaks",
        ChampionInfo {
            skull: ENLIGHTENMENT,
            unique: &["Server.Items.OrcChieftainHelm"],
            shared: &[
                "Server.Items.RoyalGuardSurvivalKnife",
                "Server.Items.DjinnisRing",
                "Server.Items.LieutenantOfTheBritannianRoyalGuard",
                "Server.Items.SamaritanRobe",
                "Server.Items.DetectiveBoots",
                "Server.Items.TheMostKnowledgePerson",
            ],
            decorative: &[
                "Server.Items.WaterTile",
                "Server.Items.WindSpirit",
                "Server.Items.Pier",
                "Server.Items.DirtPatch",
            ],
            statues: &[],
        },
    ),
    (
        "Server.Mobiles.Ilhenir",
        ChampionInfo {
            skull: PAIN,
            unique: &[],
            shared: &[
                "Server.Items.ANecromancerShroud",
                "Server.Items.LieutenantOfTheBritannianRoyalGuard",
                "Server.Items.OblivionsNeedle",
                "Server.Items.TheRobeOfBritanniaAri",
            ],
            decorative: &["Server.Items.MonsterStatuette"],
            statues: &[28, 29], // MonsterStatuetteType.PlagueBeast, RedDeath
        },
    ),
    (
        "Server.Mobiles.Meraktus",
        ChampionInfo {
            skull: PAIN,
            unique: &["Server.Items.Subdue"],
            shared: &[],
            decorative: &[
                "Server.Items.ArtifactLargeVase",
                "Server.Items.ArtifactVase",
                "Server.Items.MinotaurStatueDeed",
            ],
            statues: &[36], // MonsterStatuetteType.Minotaur
        },
    ),
    (
        "Server.Mobiles.Twaulo",
        ChampionInfo {
            skull: PAIN,
            unique: &["Server.Items.Quell"],
            shared: &[
                "Server.Items.TheMostKnowledgePerson",
                "Server.Items.OblivionsNeedle",
            ],
            decorative: &["Server.Items.Pier", "Server.Items.MonsterStatuette"],
            statues: &[35], // MonsterStatuetteType.DreadHorn
        },
    ),
];

/// The `SkullType`, `UniqueList`, `SharedList`, `DecorativeList` and `StatueTypes` of a champion class.
pub fn info(ty: &'static MobileType) -> Option<&'static ChampionInfo> {
    let mut t = Some(ty);
    while let Some(c) = t {
        if let Some((_, i)) = CHAMPIONS.iter().find(|(n, _)| *n == c.full_name) {
            return Some(i);
        }
        t = c.parent;
    }
    None
}

/// `GetArtifact()`.
pub fn get_artifact(w: &mut World, id: MobileId) -> Option<ItemId> {
    let info = info(w.mobile(id)?.ty)?;
    let r = rnd::double(w);
    if r < 0.05 {
        create_artifact(w, info, info.unique)
    } else if r < 0.15 {
        create_artifact(w, info, info.shared)
    } else if r < 0.30 {
        create_artifact(w, info, info.decorative)
    } else {
        None
    }
}

/// `CreateArtifact(list)`.
pub fn create_artifact(w: &mut World, info: &ChampionInfo, list: &[&str]) -> Option<ItemId> {
    if list.is_empty() {
        return None;
    }
    let name = *muo_primitives::utility::random_element(&mut w.rng, list)?;
    let artifact = loot::construct(w, name)?;
    if !info.statues.is_empty() && factory::item_is(w, artifact, "Server.Items.MonsterStatuette") {
        let kind = *muo_primitives::utility::random_element(&mut w.rng, info.statues)?;
        muo_items::special::statuette::set_type(w, artifact, kind);
        ops::set_loot_type(w, artifact, LootType::Regular);
    }
    Some(artifact)
}

fn create_random_power_scroll(w: &mut World) -> ItemId {
    let r = rnd::double(w);
    let level = if r < 0.05 {
        20
    } else if r < 0.4 {
        15
    } else {
        10
    };
    muo_items::special::scrolls::power_scroll_create_random_no_craft(w, level, level)
}

/// `!Core.SE || m.Alive` into the pack, else the corpse, else the pack.
pub(crate) fn give(w: &mut World, to: MobileId, item: ItemId) {
    if !w.core_se() || mx::alive(w, to) {
        items::add_to_backpack(w, to, item);
    } else if let Some(c) = w
        .mobile(to)
        .and_then(|m| m.corpse)
        .filter(|&c| w.item(c).is_some_and(|i| !i.deleted()))
    {
        muo_server::item::container::drop_item_default(w, c, item);
    } else {
        items::add_to_backpack(w, to, item);
    }
}

pub(crate) fn is_player_mobile(w: &World, m: MobileId) -> bool {
    w.mobile(m)
        .is_some_and(|x| x.ty.is(&muo_content::player_mobile::PLAYER_MOBILE))
}

/// The protector of `m` (`JusticeVirtue.GetProtector`) when it may be rewarded with `m`'s drop: the
/// `prot == null || prot.Map != pm.Map || prot.Murderer || prot.Criminal || !CheckMapRegion` guard, then the chance
/// (`60/80/100` by Justice level) rolled once.
pub(crate) fn justice_reward(w: &mut World, m: MobileId) -> Option<MobileId> {
    let h = hooks::get(w)?;
    let prot = h.justice_protector?(w, m)?;
    let same_region = h.justice_check_map_region.is_some_and(|f| f(w, m, prot));
    let level = h.justice_level.map_or(0, |f| f(w, prot));
    let (pm_map, p) = (mx::map(w, m), w.mobile(prot)?);
    if p.map != pm_map || (p.ty.murderer)(w, prot) || p.criminal || !same_region {
        return None;
    }
    let chance = match level {
        1 => 60,
        2 => 80,
        3 => 100,
        _ => 0,
    };
    (chance > 0 && chance > rnd::below(w, 100)).then_some(prot)
}

/// `BaseChampion.GivePowerScrollTo(m, ps)`.
pub fn give_power_scroll_to(w: &mut World, m: MobileId, ps: ItemId) {
    mx::msg(w, m, 1049524); // You have received a scroll of power!
    give(w, m, ps);
    if !is_player_mobile(w, m) {
        return;
    }
    if let Some(prot) = justice_reward(w, m) {
        let (skill, value) = (
            muo_items::special::scrolls::skill(w, ps),
            muo_items::special::scrolls::value(w, ps),
        );
        let scroll = muo_items::special::scrolls::power_scroll_new(w, skill, value);
        mx::msg(w, prot, 1049368); // You have been rewarded for your dedication to Justice!
        give(w, prot, scroll);
    }
}

/// The mobiles with a right to the loot, last damager first (the rights are walked backwards).
pub(crate) fn rights_holders(w: &mut World, id: MobileId) -> Vec<MobileId> {
    let hits_max = mx::hits_max(w, id);
    let rights = get_looting_rights(w, id, hits_max);
    rights
        .iter()
        .rev()
        .filter(|r| r.has_right)
        .map(|r| r.mobile)
        .collect()
}

/// `GivePowerScrolls()`.
pub fn give_power_scrolls(w: &mut World, id: MobileId) {
    if mx::map(w, id) != Some(MapId::FELUCCA) {
        return;
    }
    let mut to_give = rights_holders(w, id);
    if to_give.is_empty() {
        return;
    }
    for m in to_give.clone() {
        if !is_player_mobile(w, m) {
            continue;
        }
        let award = hooks::get(w).and_then(|h| h.award_valor);
        if let Some(gained_path) = award.and_then(|f| f(w, m, 800)) {
            // You have gained a path in Valor! / You have gained in Valor!
            mx::msg(w, m, if gained_path { 1054032 } else { 1054030 });
        }
    }
    muo_primitives::random::Rng::shuffle(&mut w.rng, &mut to_give);
    for i in 0..6 {
        let m = to_give[i % to_give.len()];
        let ps = create_random_power_scroll(w);
        give_power_scroll_to(w, m, ps);
    }
}

/// `new GoodiesTimer(map, x, y).Start()`: a pile of gold with an effect, after a random delay.
pub(crate) fn goodies_timer(w: &mut World, map: MapId, x: i32, y: i32, gold: (i32, i32)) {
    let delay = TimeSpan::from_secs_f64(rnd::double(w) * 10.0);
    muo_server::timer::start(w, delay, TimeSpan::ZERO, 1, move |w| {
        let mut z = w.maps.get_average_z(map, x, y);
        let mut can_fit = muo_server::map::fit::can_fit(w, map, x, y, z, 6, false, false, true);
        let mut i = -3;
        while !can_fit && i <= 3 {
            can_fit = muo_server::map::fit::can_fit(w, map, x, y, z + i, 6, false, false, true);
            if can_fit {
                z += i;
            }
            i += 1;
        }
        if !can_fit {
            return;
        }
        let amount = rnd::min_max(w, gold.0, gold.1);
        let Some(g) = factory::make_item_amount(w, "Server.Items.Gold", amount) else {
            return;
        };
        let loc = Point3D::new(x, y, z);
        ops::move_to_world_map(w, g, loc, Some(map));
        if rnd::double(w) < 0.05 {
            return;
        }
        let particles = |w: &mut World, id, speed, dur, eff| {
            let e = muo_server::item::effect_item::create(
                w,
                loc,
                Some(map),
                muo_server::item::effect_item::DEFAULT_DURATION,
            );
            muo_server::effects::send_location_particles(w, e, id, speed, dur, eff);
        };
        match rnd::below(w, 3) {
            0 => {
                particles(w, 0x3709, 10, 30, 5052); // Fire column
                muo_server::effects::play_sound(w, loc, Some(map), 0x208);
            }
            1 => {
                particles(w, 0x36BD, 20, 10, 5044); // Explosion
                muo_server::effects::play_sound(w, loc, Some(map), 0x307);
            }
            _ => particles(w, 0x36FE, 10, 10, 5052), // Ball of fire
        }
    });
}

fn no_kill_awards(w: &World, id: MobileId) -> bool {
    cd(w, id).is_some_and(|d| d.no_kill_awards)
}

fn on_before_death(w: &mut World, id: MobileId) -> bool {
    // `NoGoodies` is false for every champion class.
    if !no_kill_awards(w, id) {
        give_power_scrolls(w, id);
        if let Some(map) = mx::map(w, id) {
            let loc = mx::location(w, id);
            for x in -12..=12 {
                for y in -12..=12 {
                    if f64::from(x * x + y * y).sqrt() <= 12.0 {
                        goodies_timer(w, map, loc.x + x, loc.y + y, (500, 1000));
                    }
                }
            }
        }
    }
    (BASE_CREATURE.on_before_death)(w, id)
}

fn on_death(w: &mut World, id: MobileId, c: Option<ItemId>) {
    if mx::map(w, id) == Some(MapId::FELUCCA) {
        let to_give = rights_holders(w, id);
        let skull_type = w.mobile(id).and_then(|m| info(m.ty)).map_or(0, |i| i.skull);
        if let Some(new_skull) = hooks::get(w).and_then(|h| h.skull_new) {
            let skull = new_skull(w, skull_type);
            if let Some(&m) = muo_primitives::utility::random_element(&mut w.rng, &to_give) {
                items::add_to_backpack(w, m, skull);
            } else if let Some(c) = c {
                muo_server::item::container::drop_item_default(w, c, skull);
            }
        }
    }
    (BASE_CREATURE.on_death)(w, id, c);
}

/// `BaseChampion` (abstract).
pub static BASE_CHAMPION_C: muo_content::creature::CreatureType =
    muo_content::creature::CreatureType {
        can_move_over_obstacles: |_, _| true,
        can_destroy_obstacles: |_, _| true,
        ..BASE_CREATURE_C
    };

pub static BASE_CHAMPION: MobileType = MobileType {
    full_name: "Server.Mobiles.BaseChampion",
    parent: Some(&BASE_CREATURE),
    construct: None,
    family: Some(&BASE_CHAMPION_C),
    serialize: |w, id, out| {
        (BASE_CREATURE.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (BASE_CREATURE.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        Ok(())
    },
    on_before_death,
    on_death,
    ..BASE_CREATURE
};

// ---- Barracoon ----

fn barracoon_new(w: &mut World) -> MobileId {
    muo_content::templates::new_creature(w, &BARRACOON)
}

/// `Polymorph(m)`.
pub fn polymorph(w: &mut World, m: MobileId) {
    if !combat::can_begin_action(w, m, "Server.Spells.Seventh.PolymorphSpell")
        || !combat::can_begin_action(w, m, "Server.Spells.Fifth.IncognitoSpell")
        || props_b::is_body_mod(w, m)
    {
        return;
    }
    if let Some(mount) = props_b::mount(w, m) {
        muo_content::creature::mount::set_rider(w, mount, None);
    }
    if props_b::mount(w, m).is_some() {
        return;
    }
    if combat::begin_action(w, m, "Server.Spells.Seventh.PolymorphSpell") {
        for layer in [Layer::OneHanded, Layer::TwoHanded] {
            if let Some(d) = items::find_item_on_layer(w, m, layer)
                && w.item(d).is_some_and(|i| i.movable())
            {
                items::add_to_backpack(w, m, d);
            }
        }
        props_b::set_body_mod(w, m, muo_primitives::Body(42));
        props_b::set_hue_mod(w, m, 0);
        muo_server::timer::start(w, TimeSpan::from_mins(3), TimeSpan::ZERO, 1, move |w| {
            if !combat::can_begin_action(w, m, "Server.Spells.Seventh.PolymorphSpell") {
                props_b::set_body_mod(w, m, muo_primitives::Body(0));
                props_b::set_hue_mod(w, m, -1);
                combat::end_action(w, m, "Server.Spells.Seventh.PolymorphSpell");
            }
        });
    }
}

/// `SpawnRatmen(target)`.
pub fn spawn_ratmen(w: &mut World, id: MobileId, target: MobileId) {
    if mx::map(w, id).is_none_or(|m| m == MapId::INTERNAL) {
        return;
    }
    let mut rats = 0;
    for m in mx::mobiles_in_range(w, id, 10) {
        if cd(w, m).is_some()
            && ["Ratman", "RatmanArcher", "RatmanMage"].iter().any(|n| {
                w.mobile(m)
                    .is_some_and(|x| x.ty.full_name == format!("Server.Mobiles.{n}"))
            })
        {
            rats += 1;
            if rats >= 16 {
                return;
            }
        }
    }
    mx::play_sound(w, id, 0x3D);
    let rats = rnd::min_max(w, 3, 6);
    for _ in 0..rats {
        let name = match rnd::below(w, 5) {
            2 | 3 => "Server.Mobiles.RatmanArcher",
            4 => "Server.Mobiles.RatmanMage",
            _ => "Server.Mobiles.Ratman",
        };
        muo_content::creature::control::spawn_minion_by_name(w, id, name, Some(target), None);
    }
}

/// `DoSpecialAbility(target)`.
pub fn barracoon_special_ability(w: &mut World, id: MobileId, target: MobileId) {
    if mx::deleted(w, target) {
        return;
    }
    if rnd::double(w) < 0.6 {
        polymorph(w, target);
    }
    if rnd::double(w) < 0.2 {
        spawn_ratmen(w, id, target);
    }
    if mx::hits(w, id) < 500 && !props_b::is_body_mod(w, id) {
        polymorph(w, id);
    }
}

creature_leaf! {
    /// `Server.Mobiles.Barracoon`.
    BARRACOON, BARRACOON_C = "Server.Mobiles.Barracoon" : BASE_CHAMPION / BASE_CHAMPION_C, ctor Some(barracoon_new), encoded false;
    c {
        default_name: |_w, _id| Some("Barracoon"),
        always_murderer: |_w, _id| true,
        auto_dispel: |_w, _id| true,
        auto_dispel_chance: |_w, _id| 1.0,
        bard_immune: |w, _id| !w.core_se(),
        unprovokable: |w, _id| w.core_se(),
        uncalmable: |w, _id| w.core_se(),
        poison_immune: |_w, _id| Some(DEADLY),
        generate_loot: |w, id| add_loot_n(w, id, loot_pack::ultra_rich(w), 3),
        on_got_melee_attack: |w, id, attacker, damage| {
            (BASE_CHAMPION_C.on_got_melee_attack)(w, id, attacker, damage);
            barracoon_special_ability(w, id, attacker);
        },
        on_gave_melee_attack: |w, id, defender, damage| {
            (BASE_CHAMPION_C.on_gave_melee_attack)(w, id, defender, damage);
            barracoon_special_ability(w, id, defender);
        },
    }
    m {
        show_fame_title: |_, _| false,
        click_title: |_, _| false,
    }
}

// ---- Mephitis ----

fn mephitis_new(w: &mut World) -> MobileId {
    muo_content::templates::new_creature(w, &MEPHITIS)
}

creature_leaf! {
    /// `Server.Mobiles.Mephitis`.
    MEPHITIS, MEPHITIS_C = "Server.Mobiles.Mephitis" : BASE_CHAMPION / BASE_CHAMPION_C, ctor Some(mephitis_new), encoded false;
    c {
        default_name: |_w, _id| Some("Mephitis"),
        poison_immune: |_w, _id| Some(LETHAL),
        hit_poison: |_w, _id| Some(LETHAL),
        generate_loot: |w, id| add_loot_n(w, id, loot_pack::ultra_rich(w), 4),
    }
    m {
    }
}

// ---- Rikktor ----

fn rikktor_new(w: &mut World) -> MobileId {
    muo_content::templates::new_creature(w, &RIKKTOR)
}

/// The victims of an area attack around `center`: `m != this`, harmable (or a living player), and a creature that
/// is controlled, summoned or on another team (a player is not a `BaseCreature`, so none passes).
fn area_targets(w: &mut World, id: MobileId, center: MobileId, range: i32) -> Vec<MobileId> {
    let team = cd(w, id).map_or(0, |d| d.team);
    let mut out = Vec::new();
    for m in mx::mobiles_in_range(w, center, range) {
        if m == id || !(mx::can_be_harmful(w, id, m) || mx::player(w, m) && mx::alive(w, m)) {
            continue;
        }
        let Some(d) = cd(w, m) else { continue };
        if !(d.controlled || d.summoned || d.team != team) {
            continue;
        }
        out.push(m);
    }
    out
}

/// `Earthquake()`.
pub fn earthquake(w: &mut World, id: MobileId) {
    if mx::map(w, id).is_none() {
        return;
    }
    mx::play_sound(w, id, 0x2F3);
    for m in area_targets(w, id, id, 8) {
        let damage = ((f64::from(mx::hits(w, m)) * 0.6) as i32).clamp(10, 75);
        mx::do_harmful(w, id, m);
        aos_helpers::damage(
            w,
            m,
            Some(id),
            damage,
            DamageSplit {
                phys: 100,
                fire: 0,
                cold: 0,
                pois: 0,
                nrgy: 0,
            },
            Default::default(),
        );
        let human = muo_content::skills::common::is_human(w, m);
        if mx::alive(w, m) && human && props_b::mount(w, m).is_none() {
            mx::animate(w, m, 20, 7, 1, true, false, 0); // take hit
        }
    }
}

creature_leaf! {
    /// `Server.Mobiles.Rikktor`.
    RIKKTOR, RIKKTOR_C = "Server.Mobiles.Rikktor" : BASE_CHAMPION / BASE_CHAMPION_C, ctor Some(rikktor_new), encoded false;
    c {
        default_name: |_w, _id| Some("Rikktor"),
        poison_immune: |_w, _id| Some(LETHAL),
        scale_type: |_w, _id| ScaleType::All,
        scales: |_w, _id| 20,
        generate_loot: |w, id| add_loot_n(w, id, loot_pack::ultra_rich(w), 4),
        on_gave_melee_attack: |w, id, defender, damage| {
            (BASE_CHAMPION_C.on_gave_melee_attack)(w, id, defender, damage);
            if rnd::double(w) < 0.2 {
                earthquake(w, id);
            }
        },
    }
    m {
        get_anger_sound: |w, _id| rnd::from(w, 0x2CE, 2),
        get_idle_sound: |_w, _id| 0x2D2,
        get_attack_sound: |w, _id| rnd::from(w, 0x2C7, 5),
        get_hurt_sound: |_w, _id| 0x2D1,
        get_death_sound: |_w, _id| 0x2CC,
    }
}

// ---- Semidar ----

fn semidar_new(w: &mut World) -> MobileId {
    muo_content::templates::new_creature(w, &SEMIDAR)
}

fn caster_is_male(w: &World, caster: MobileId) -> bool {
    w.mobile(caster).is_some_and(|m| m.cur_body().is_male())
}

/// `SemidarDrainLife`: a `DrainLifeAreaAttack` with `ChanceToTrigger => 0.25`.
static SEMIDAR_DRAIN_LIFE: muo_content::abilities::MonsterAbility =
    muo_content::abilities::MonsterAbility {
        name: "SemidarDrainLife",
        chance: 0.25,
        ..muo_content::abilities::DRAIN_LIFE_AREA_ATTACK
    };
static SEMIDAR_ABILITIES: [&muo_content::abilities::MonsterAbility; 1] = [&SEMIDAR_DRAIN_LIFE];

creature_leaf! {
    /// `Server.Mobiles.Semidar`.
    SEMIDAR, SEMIDAR_C = "Server.Mobiles.Semidar" : BASE_CHAMPION / BASE_CHAMPION_C, ctor Some(semidar_new), encoded false;
    c {
        default_name: |_w, _id| Some("Semidar"),
        unprovokable: |_w, _id| true,
        poison_immune: |_w, _id| Some(LETHAL),
        get_monster_abilities: |_, _| &SEMIDAR_ABILITIES,
        generate_loot: |w, id| {
            add_loot_n(w, id, loot_pack::ultra_rich(w), 4);
            add_loot(w, id, loot_pack::filthy_rich(w));
        },
        check_reflect: |w, _id, caster, reflect| {
            if caster_is_male(w, caster) {
                *reflect = true; // Always reflect if caster isn't female
            }
        },
        alter_damage_scalar_from: |w, _id, caster, scalar| {
            if caster_is_male(w, caster) {
                *scalar = 20.0; // Male bodies always reflect.. damage scaled 20x
            }
        },
    }
    m {
    }
}

// ---- Serado ----

fn serado_new(w: &mut World) -> MobileId {
    let id = muo_content::templates::new_creature(w, &SERADO);
    let seed = crate::hand::seed::random_bonsai_seed(w);
    pack_item(w, id, seed);
    id
}

/// `ScaleResistances()`.
fn serado_scale_resistances(w: &mut World, id: MobileId) {
    let max = mx::hits_max(w, id);
    let lost = f64::from(max - mx::hits(w, id)) / f64::from(max);
    for (t, from) in [
        (ResistanceType::Physical, 30),
        (ResistanceType::Fire, 60),
        (ResistanceType::Cold, 60),
        (ResistanceType::Poison, 90),
        (ResistanceType::Energy, 50),
    ] {
        set_resistance(w, id, t, from + (lost * f64::from(95 - from)) as i32);
    }
}

/// `DoCounter(attacker)`.
fn serado_counter(w: &mut World, id: MobileId, attacker: MobileId) {
    if mx::map(w, id).is_none() {
        return;
    }
    if rnd::double(w) >= 0.2 {
        return;
    }
    let mut target = None;
    if let Some(d) = cd(w, attacker) {
        if d.bard_provoked {
            return;
        }
        target = muo_content::creature::control::get_master(w, attacker);
    }
    let target = match target {
        Some(t) if mx::in_range(w, t, id, 25) => t,
        _ => attacker,
    };
    mx::animate(w, id, 10, 4, 1, true, false, 0);
    for m in area_targets(w, id, target, 8) {
        mx::do_harmful(w, id, m);
        let n = rnd::min_max(w, 20, 25);
        aos_helpers::damage(
            w,
            m,
            Some(id),
            n,
            DamageSplit {
                phys: 0,
                fire: 0,
                cold: 0,
                pois: 0,
                nrgy: 100,
            },
            aos_helpers::DamageOptions {
                ignore_armor: true,
                ..Default::default()
            },
        );
        effects::fixed_particles_hue(w, m, 0x36BD, 1, 10, 0x1F78, 0xA6, 0, EffectLayer::None);
        muo_server::mobile::base::apply_poison(w, m, Some(id), LETHAL);
    }
}

creature_leaf! {
    /// `Server.Mobiles.Serado`.
    SERADO, SERADO_C = "Server.Mobiles.Serado" : BASE_CHAMPION / BASE_CHAMPION_C, ctor Some(serado_new), encoded false;
    c {
        default_name: |_w, _id| Some("Serado"),
        treasure_map_level: |_w, _id| 5,
        hit_poison: |_w, _id| Some(LETHAL),
        poison_immune: |_w, _id| Some(LETHAL),
        hit_poison_chance: |_w, _id| 0.8,
        feathers: |_w, _id| 30,
        get_weapon_ability: |_w, _id| Some(muo_content::items::weapon::Ability::DoubleStrike),
        generate_loot: |w, id| {
            add_loot_n(w, id, loot_pack::ultra_rich(w), 4);
            add_loot(w, id, loot_pack::filthy_rich(w));
            add_loot_n(w, id, &loot_pack::GEMS, 6);
        },
        // TODO: Hit Lightning Area.
        on_damaged_by_spell: |w, id, attacker, damage| {
            (BASE_CHAMPION_C.on_damaged_by_spell)(w, id, attacker, damage);
            serado_scale_resistances(w, id);
            serado_counter(w, id, attacker);
        },
        on_got_melee_attack: |w, id, attacker, damage| {
            (BASE_CHAMPION_C.on_got_melee_attack)(w, id, attacker, damage);
            serado_scale_resistances(w, id);
            serado_counter(w, id, attacker);
        },
    }
    m {
        show_fame_title: |_, _| false,
        click_title: |_, _| false,
    }
}

// ---- Neira ----

const SPEED_BOOST_SCALAR: f64 = 1.2;

/// `Neira._speedBoost`: not serialized.
#[derive(Default)]
struct NeiraData {
    speed_boost: bool,
}

fn neira_boost(w: &World, id: MobileId) -> bool {
    w.mobile(id)
        .and_then(|m| m.ext.get::<NeiraData>())
        .is_some_and(|d| d.speed_boost)
}

fn set_neira_boost(w: &mut World, id: MobileId, v: bool) {
    if let Some(m) = w.mobile_mut(id) {
        m.ext.insert(NeiraData { speed_boost: v });
    }
}

fn neira_new(w: &mut World) -> MobileId {
    let id = muo_content::templates::new_creature(w, &NEIRA);
    if let Some(shroud) = factory::make_item(w, "Server.Items.HoodedShroudOfShadows") {
        ops::set_movable(w, shroud, false);
        items::add_item(w, id, shroud);
    }
    if let Some(weapon) = factory::make_item(w, "Server.Items.Scimitar") {
        if let Some(d) = muo_content::items::weapon::data_mut(w, weapon) {
            d.skill = SkillName::Wrestling as i32;
        }
        muo_server::item::base::set_hue(w, weapon, 38);
        ops::set_movable(w, weapon, false);
        items::add_item(w, id, weapon);
    }
    // new SkeletalMount().Rider = this;
    let vm = muo_content::creature::virtual_mount::new_virtual_mount_item(w, id);
    items::add_item(w, id, vm);
    id
}

/// `CheckSpeedBoost()`.
fn check_speed_boost(w: &mut World, id: MobileId) {
    if mx::hits(w, id) < mx::hits_max(w, id) / 4 {
        if !neira_boost(w, id) {
            if let Some(d) = cd_mut(w, id) {
                d.active_speed /= SPEED_BOOST_SCALAR;
                d.passive_speed /= SPEED_BOOST_SCALAR;
            }
            set_neira_boost(w, id, true);
        }
    } else if neira_boost(w, id) {
        if let Some(d) = cd_mut(w, id) {
            d.active_speed *= SPEED_BOOST_SCALAR;
            d.passive_speed *= SPEED_BOOST_SCALAR;
        }
        set_neira_boost(w, id, false);
    }
    // Assume active
    speeds::set_current_speed_to_active(w, id);
}

fn new_unholy_bone(w: &mut World, loc: Point3D, map: Option<MapId>) {
    if let Some(b) = factory::make_item(w, "Server.Items.UnholyBone") {
        ops::move_to_world_map(w, b, loc, map);
    }
}

/// `AddUnholyBone(target, chanceToThrow)`.
fn add_unholy_bone(w: &mut World, id: MobileId, target: MobileId, chance_to_throw: f64) {
    if mx::map(w, id).is_none() {
        return;
    }
    if chance_to_throw >= rnd::double(w) {
        let d = mx::get_direction_to(w, id, target);
        mx::set_direction(w, id, d);
        effects::moving_effect(
            w,
            id,
            mx::entity(target),
            0xF7E,
            10,
            1,
            true,
            false,
            0x496,
            0,
        );
        muo_server::timer::start(w, TimeSpan::from_secs(1), TimeSpan::ZERO, 1, move |w| {
            if mx::can_be_harmful(w, id, target) {
                mx::do_harmful(w, id, target);
                let n = rnd::min_max(w, 10, 20);
                aos_helpers::damage(
                    w,
                    target,
                    Some(id),
                    n,
                    DamageSplit {
                        phys: 100,
                        fire: 0,
                        cold: 0,
                        pois: 0,
                        nrgy: 0,
                    },
                    Default::default(),
                );
                new_unholy_bone(w, mx::location(w, target), mx::map(w, target));
            }
        });
    } else {
        new_unholy_bone(w, mx::location(w, id), mx::map(w, id));
    }
}

/// `[SerializationGenerator(1, false)]`: the version only; version 0 also stored `_speedBoost`.
fn neira_serialize(_w: &World, _id: MobileId, out: &mut dyn GenericWriter) {
    out.write_i32(1);
}

fn neira_deserialize(w: &mut World, id: MobileId, r: &mut dyn GenericReader) -> muo_io::Result<()> {
    let version = r.read_i32()?;
    if version < 1 {
        r.read_bool()?; // _speedBoost
    }
    check_speed_boost(w, id);
    Ok(())
}

creature_leaf! {
    /// `Server.Mobiles.Neira`.
    NEIRA, NEIRA_C = "Server.Mobiles.Neira" : BASE_CHAMPION / BASE_CHAMPION_C, ctor Some(neira_new), state neira_serialize, neira_deserialize;
    c {
        default_name: |_w, _id| Some("Neira"),
        always_murderer: |_w, _id| true,
        bard_immune: |w, _id| !w.core_se(),
        unprovokable: |w, _id| w.core_se(),
        uncalmable: |w, _id| w.core_se(),
        poison_immune: |_w, _id| Some(DEADLY),
        generate_loot: |w, id| {
            add_loot_n(w, id, loot_pack::ultra_rich(w), 3);
            add_loot(w, id, loot_pack::meager(w));
        },
        on_gave_melee_attack: |w, id, defender, damage| {
            (BASE_CHAMPION_C.on_gave_melee_attack)(w, id, defender, damage);
            if rnd::double(w) < 0.1 {
                add_unholy_bone(w, id, defender, 0.25);
            }
            check_speed_boost(w, id);
        },
        on_got_melee_attack: |w, id, attacker, damage| {
            (BASE_CHAMPION_C.on_got_melee_attack)(w, id, attacker, damage);
            if rnd::double(w) < 0.1 {
                add_unholy_bone(w, id, attacker, 0.25);
            }
        },
        on_damaged_by_spell: |w, id, from, damage| {
            (BASE_CHAMPION_C.on_damaged_by_spell)(w, id, from, damage);
            if rnd::double(w) < 0.1 {
                add_unholy_bone(w, id, from, 1.0);
            }
        },
    }
    m {
        show_fame_title: |_, _| false,
        click_title: |_, _| false,
        on_before_death: |w, id| {
            if let Some(mount) = props_b::mount(w, id) {
                muo_content::creature::mount::set_rider(w, mount, None);
                muo_server::mobile::base::delete(w, mount);
            }
            (BASE_CHAMPION.on_before_death)(w, id)
        },
        on_damage: |w, id, amount, from, will_kill| {
            check_speed_boost(w, id);
            (BASE_CHAMPION.on_damage)(w, id, amount, from, will_kill);
        },
    }
}

// ---- LordOaks ----

/// `LordOaks._queen` and `_spawnedQueen`.
#[derive(Default)]
struct OaksData {
    queen: Option<MobileId>,
    spawned_queen: bool,
}

fn oaks(w: &World, id: MobileId) -> Option<&OaksData> {
    w.mobile(id)?.ext.get::<OaksData>()
}

fn oaks_mut(w: &mut World, id: MobileId) -> Option<&mut OaksData> {
    w.mobile_mut(id)?.ext.get_mut::<OaksData>()
}

fn lord_oaks_new(w: &mut World) -> MobileId {
    let args = muo_content::templates::table(w).roll_args(w, LORD_OAKS.full_name);
    let id = new_creature(
        w,
        &LORD_OAKS,
        args.ai,
        args.fight_mode,
        args.range_perception,
        args.range_fight,
    );
    if let Some(m) = w.mobile_mut(id) {
        m.ext.insert(OaksData::default());
    }
    muo_content::templates::apply_mobile(w, id);
    id
}

/// `IsQueenAlive`.
fn is_queen_alive(w: &World, id: MobileId) -> bool {
    oaks(w, id)
        .and_then(|d| d.queen)
        .is_some_and(|q| !mx::deleted(w, q))
}

/// `SpawnQueen()`.
fn spawn_queen(w: &mut World, id: MobileId) {
    let spawned = oaks(w, id).is_some_and(|d| d.spawned_queen);
    if spawned || mx::deleted(w, id) || mx::map(w, id).is_none_or(|m| m == MapId::INTERNAL) {
        return;
    }
    messages::say_number(w, id, 1042153, ""); // Come forth my queen!
    let Some(queen) = factory::make_mobile(w, "Server.Mobiles.Silvani") else {
        return;
    };
    let loc = mx::location(w, id);
    muo_content::creature::control::spawn_minion(w, id, queen, None, Some(loc));
    if let Some(d) = oaks_mut(w, id) {
        d.queen = Some(queen);
        d.spawned_queen = true;
    }
}

/// `target.Damage(Utility.Random(20, 10), this); target.Stam -= Utility.Random(20, 10); target.Mana -= ..`
pub(crate) fn drain_hit(w: &mut World, id: MobileId, target: MobileId) {
    let n = rnd::from(w, 20, 10);
    mx::damage(w, target, n, Some(id));
    let n = rnd::from(w, 20, 10);
    let stam = w.mobile(target).map_or(0, |m| m.stam);
    props_b::set_stam(w, target, stam - n);
    let n = rnd::from(w, 20, 10);
    let mana = w.mobile(target).map_or(0, |m| m.mana);
    props_b::set_mana(w, target, mana - n);
}

fn oaks_serialize(w: &World, id: MobileId, out: &mut dyn GenericWriter) {
    out.write_i32(0);
    out.write_mobile(w, oaks(w, id).and_then(|d| d.queen));
    out.write_bool(oaks(w, id).is_some_and(|d| d.spawned_queen));
}

fn oaks_deserialize(w: &mut World, id: MobileId, r: &mut dyn GenericReader) -> muo_io::Result<()> {
    let _version = r.read_i32()?;
    let queen = r.read_mobile(w)?;
    let spawned_queen = r.read_bool()?;
    if let Some(m) = w.mobile_mut(id) {
        m.ext.insert(OaksData {
            queen,
            spawned_queen,
        });
    }
    Ok(())
}

static LORD_OAKS_ABILITIES: [&muo_content::abilities::MonsterAbility; 1] =
    [&muo_content::abilities::SUMMON_PIXIES_COUNTER];

creature_leaf! {
    /// `Server.Mobiles.LordOaks`.
    LORD_OAKS, LORD_OAKS_C = "Server.Mobiles.LordOaks" : BASE_CHAMPION / BASE_CHAMPION_C, ctor Some(lord_oaks_new), state oaks_serialize, oaks_deserialize;
    c {
        default_name: |_w, _id| Some("Lord Oaks"),
        auto_dispel: |_w, _id| true,
        can_fly: |_w, _id| true,
        bard_immune: |w, _id| !w.core_se(),
        unprovokable: |w, _id| w.core_se(),
        uncalmable: |w, _id| w.core_se(),
        opposition_group: |_w, _id| Some(&muo_content::ai::opposition_group::FEY_AND_UNDEAD),
        poison_immune: |_w, _id| Some(DEADLY),
        get_monster_abilities: |_, _| &LORD_OAKS_ABILITIES,
        can_use_ability: |w, id, _ability| is_queen_alive(w, id),
        generate_loot: |w, id| add_loot_n(w, id, loot_pack::ultra_rich(w), 5),
        alter_damage_scalar_from: |w, id, _caster, scalar| {
            if is_queen_alive(w, id) {
                *scalar *= 0.1;
            }
        },
        on_gave_melee_attack: |w, id, defender, damage| {
            (BASE_CHAMPION_C.on_gave_melee_attack)(w, id, defender, damage);
            drain_hit(w, id, defender);
        },
        on_got_melee_attack: |w, id, attacker, damage| {
            (BASE_CHAMPION_C.on_got_melee_attack)(w, id, attacker, damage);
            drain_hit(w, id, attacker);
        },
    }
    m {
        get_anger_sound: |_w, _id| 0x2F8,
        get_idle_sound: |_w, _id| 0x2F8,
        get_attack_sound: |w, _id| rnd::from(w, 0x2F5, 2),
        get_hurt_sound: |_w, _id| 0x2F9,
        get_death_sound: |_w, _id| 0x2F7,
        // Aggression lands before the damage it causes, so the queen is up before the first hit is scaled.
        aggressive_action: |w, id, aggressor| {
            (BASE_CHAMPION.aggressive_action)(w, id, aggressor);
            spawn_queen(w, id);
        },
        on_combatant_change: |w, id| {
            (BASE_CHAMPION.on_combatant_change)(w, id);
            if mx::combatant(w, id).is_some() {
                spawn_queen(w, id);
            }
        },
    }
}

pub(crate) fn all() -> Vec<&'static MobileType> {
    vec![
        &BARRACOON, &MEPHITIS, &RIKKTOR, &SEMIDAR, &SERADO, &NEIRA, &LORD_OAKS,
    ]
}

pub(crate) fn register(types: &mut TypeRegistry) {
    for t in all() {
        types.register_mobile(t, &[]);
    }
}
