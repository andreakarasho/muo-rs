//! `DemonKnight`: the boss and the artifact helpers the other Dark Father minions call.

use muo_content::creature::death::get_looting_rights;
use muo_content::creature::{BASE_CREATURE, BASE_CREATURE_C};
use muo_content::items::weapon::Ability;
use muo_content::{factory, loot, loot_pack, mx};
use muo_primitives::{Point3D, TimeSpan};
use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{container, ops};
use muo_server::mobile::part2::effects;
use muo_server::poison::PoisonId;
use muo_server::type_registry::TypeRegistry;

use crate::common::*;
use crate::leaf::creature_leaf;

pub(crate) fn demon_knight_make(
    w: &mut World,
    ty: &'static muo_server::mobile::MobileType,
) -> MobileId {
    new_creature(
        w,
        ty,
        AIType::AiMage,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    )
}

const ALL_TYPES: [ResistanceType; 5] = [
    ResistanceType::Physical,
    ResistanceType::Fire,
    ResistanceType::Cold,
    ResistanceType::Poison,
    ResistanceType::Energy,
];

pub(crate) fn demon_knight_init(w: &mut World, id: MobileId) {
    name_random(w, id, "demon knight");
    title(w, id, "the Dark Father");
    body(w, id, 318);
    base_sound_id(w, id, 0x165);
    set_str(w, id, 500);
    set_dex(w, id, 100);
    set_int(w, id, 1000);
    set_hits(w, id, 30000);
    set_mana(w, id, 5000);
    set_damage(w, id, 17, 21);
    for t in ALL_TYPES {
        set_damage_type(w, id, t, 20);
    }
    for t in ALL_TYPES {
        set_resistance(w, id, t, 30);
    }
    set_skill_range(w, id, SkillName::Necromancy, 120.0, 120.0);
    set_skill_range(w, id, SkillName::SpiritSpeak, 120.0, 120.0);
    set_skill(w, id, SkillName::DetectHidden, 80.0);
    set_skill(w, id, SkillName::EvalInt, 100.0);
    set_skill(w, id, SkillName::Magery, 100.0);
    set_skill(w, id, SkillName::Meditation, 120.0);
    set_skill(w, id, SkillName::MagicResist, 150.0);
    set_skill(w, id, SkillName::Tactics, 100.0);
    set_skill(w, id, SkillName::Wrestling, 120.0);
    fame(w, id, 28000);
    karma(w, id, -28000);
    virtual_armor(w, id, 64);
}

fn demon_knight_new(w: &mut World) -> MobileId {
    let id = demon_knight_make(w, &DEMON_KNIGHT);
    demon_knight_init(w, id);
    id
}

const ARTIFACT_RARITY_10: [&str; 2] = ["LegacyOfTheDreadLord", "TheTaskmaster"];
const ARTIFACT_RARITY_11: [&str; 31] = [
    "TheDragonSlayer",
    "ArmorOfFortune",
    "GauntletsOfNobility",
    "HelmOfInsight",
    "HolyKnightsBreastplate",
    "JackalsCollar",
    "LeggingsOfBane",
    "MidnightBracers",
    "OrnateCrownOfTheHarrower",
    "ShadowDancerLeggings",
    "TunicOfFire",
    "VoiceOfTheFallenKing",
    "BraceletOfHealth",
    "OrnamentOfTheMagician",
    "RingOfTheElements",
    "RingOfTheVile",
    "Aegis",
    "ArcaneShield",
    "AxeOfTheHeavens",
    "BladeOfInsanity",
    "BoneCrusher",
    "BreathOfTheDead",
    "Frostbringer",
    "SerpentsFang",
    "StaffOfTheMagi",
    "TheBeserkersMaul",
    "TheDryadBow",
    "DivineCountenance",
    "HatOfTheMagi",
    "HuntersHeaddress",
    "SpiritOfTheTotem",
];

/// `DemonKnight.CreateRandomArtifact()`.
pub fn create_random_artifact(w: &mut World) -> Option<ItemId> {
    if !w.core_aos() {
        return None;
    }
    let (n10, n11) = (
        ARTIFACT_RARITY_10.len() as i32,
        ARTIFACT_RARITY_11.len() as i32,
    );
    let mut random = rnd::below(w, n10 * 5 + n11 * 4);
    let name = if random < n10 * 5 {
        ARTIFACT_RARITY_10[(random / 5) as usize]
    } else {
        random -= n10 * 5;
        ARTIFACT_RARITY_11[(random / 4) as usize]
    };
    loot::construct(w, &format!("Server.Items.{name}"))
}

/// `DemonKnight.FindRandomPlayer(creature)`.
pub fn find_random_player(w: &mut World, creature: MobileId) -> Option<MobileId> {
    let ty = w.mobile(creature)?.ty;
    let hits_max = (ty.hits_max)(w, creature);
    let mut rights = get_looting_rights(w, creature, hits_max);
    rights.retain(|r| r.has_right);
    muo_primitives::utility::random_element(&mut w.rng, &rights).map(|r| r.mobile)
}

/// `DistributeArtifact(creature)` = `DistributeArtifact(FindRandomPlayer(creature), CreateRandomArtifact())`.
pub fn distribute_artifact_from(w: &mut World, creature: MobileId) {
    let artifact = create_random_artifact(w);
    let to = find_random_player(w, creature);
    distribute_artifact(w, to, artifact);
}

/// `DistributeArtifact(to, artifact)`.
pub fn distribute_artifact(w: &mut World, to: Option<MobileId>, artifact: Option<ItemId>) {
    let (Some(to), Some(artifact)) = (to, artifact) else {
        return;
    };
    let pack = muo_server::mobile::part1::props_b::backpack(w, to);
    let placed = pack.is_some_and(|p| container::try_drop_item(w, p, Some(to), artifact, false));
    if !placed && let Some(bank) = muo_server::mobile::part1::props_b::bank_box(w, to) {
        container::drop_item_default(w, bank, artifact);
    }
    // For your valor in combating the fallen beast, a special artifact has been bestowed on you.
    mx::msg(w, to, 1062317);
}

/// `DemonKnight.GetArtifactChance(boss)`.
pub fn get_artifact_chance(w: &mut World, boss: MobileId) -> i32 {
    if !w.core_aos() {
        return 0;
    }
    let luck = loot_pack::get_luck_chance_for_killer(w, boss);
    if factory::mobile_is(w, boss, "Server.Mobiles.DemonKnight") {
        1500 + luck / 5
    } else {
        750 + luck / 10
    }
}

/// `DemonKnight.CheckArtifactChance(boss)`.
pub fn check_artifact_chance(w: &mut World, boss: MobileId) -> bool {
    let chance = get_artifact_chance(w, boss);
    chance > rnd::below(w, 100000)
}

fn on_death(w: &mut World, id: MobileId, c: Option<ItemId>) {
    (BASE_CREATURE.on_death)(w, id, c);
    let (summoned, no_awards) =
        muo_content::creature::cd(w, id).map_or((false, false), |d| (d.summoned, d.no_kill_awards));
    if !summoned && !no_awards && check_artifact_chance(w, id) {
        distribute_artifact_from(w, id);
    }
}

/// Does not call the base `OnDamage`.
fn on_damage(w: &mut World, id: MobileId, _amount: i32, from: Option<MobileId>, _will_kill: bool) {
    if let Some(f) = from
        && f != id
        && !tables(w).demon_knight_in_here
    {
        tables(w).demon_knight_in_here = true;
        let n = rnd::min_max(w, 8, 20);
        muo_content::misc::aos_helpers::damage(
            w,
            f,
            Some(id),
            n,
            muo_content::misc::aos_helpers::DamageSplit {
                phys: 100,
                fire: 0,
                cold: 0,
                pois: 0,
                nrgy: 0,
            },
            Default::default(),
        );
        effects::moving_effect(w, id, EntityId::Mobile(f), 0xECA, 10, 0, false, false, 0, 0);
        effects::play_sound(w, id, 0x491);
        if rnd::double(w) < 0.05 {
            muo_server::timer::start(w, TimeSpan::from_secs(1), TimeSpan::ZERO, 1, move |w| {
                create_bones(w, f)
            });
        }
        tables(w).demon_knight_in_here = false;
    }
}

/// `CreateBones_Callback(from)`.
fn create_bones(w: &mut World, from: MobileId) {
    let Some(map) = mx::map(w, from) else { return };
    let count = rnd::min_max(w, 1, 3);
    for _ in 0..count {
        let loc = mx::location(w, from);
        let x = loc.x + rnd::min_max(w, -1, 1);
        let y = loc.y + rnd::min_max(w, -1, 1);
        let mut z = loc.z;
        if !muo_server::map::fit::can_fit(w, map, x, y, z, 16, false, true, true) {
            z = w.maps.get_average_z(map, x, y);
            if z == loc.z || !muo_server::map::fit::can_fit(w, map, x, y, z, 16, false, true, true)
            {
                continue;
            }
        }
        let Some(bone) = factory::make_item(w, "Server.Items.UnholyBone") else {
            continue;
        };
        muo_server::item::base::set_hue(w, bone, 0);
        ops::set_name(w, bone, Some("unholy bones".to_owned()));
        let item_id = rnd::from(w, 0xECA, 9);
        muo_server::item::base::set_item_id(w, bone, item_id);
        ops::move_to_world_map(w, bone, Point3D::new(x, y, z), Some(map));
    }
}

creature_leaf! {
    /// `Server.Mobiles.DemonKnight`.
    DEMON_KNIGHT, DEMON_KNIGHT_C = "Server.Mobiles.DemonKnight" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(demon_knight_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a demon knight corpse"),
        ignore_young_protection: |w, _id| w.core_ml(),
        bard_immune: |w, _id| !w.core_se(),
        unprovokable: |w, _id| w.core_se(),
        area_peace_immune: |w, _id| w.core_se(),
        poison_immune: |_w, _id| Some(PoisonId(4)),
        treasure_map_level: |_w, _id| 1,
        get_weapon_ability: |w, _id| Some(match rnd::below(w, 3) {
            0 => Ability::DoubleStrike,
            1 => Ability::WhirlwindAttack,
            2 => Ability::CrushingBlow,
            _ => Ability::DoubleStrike,
        }),
        generate_loot: |w, id| {
            add_loot_n(w, id, loot_pack::super_boss(w), 2);
            let n = rnd::min_max(w, 6, 60);
            add_loot_n(w, id, &loot_pack::HIGH_SCROLLS, n);
        },
    }
    m {
        on_death: on_death,
        on_damage: on_damage,
    }
}

pub fn register(types: &mut TypeRegistry) {
    types.register_mobile(&DEMON_KNIGHT, &[]);
}
