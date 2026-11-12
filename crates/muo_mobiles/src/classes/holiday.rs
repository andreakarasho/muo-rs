//! Halloween creatures: `PumpkinHead` (2011), `ZombieSkeleton` (2012, player zombies) and `NaughtyTwin` (2006,
//! trick or treat). The event engines that make them (`HalloweenHauntings`, `TrickOrTreat`, `PumpkinPatchSpawner`)
//! only run between `HolidaySettings.StartHalloween` and `FinishHalloween` (2021): see `holiday_events`.

use muo_content::creature::{BASE_CREATURE, BASE_CREATURE_C};
use muo_content::mx;
use muo_io::{GenericReader, GenericWriter};
use muo_primitives::{Point3D, TimeSpan};
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::map::MapId;
use muo_server::poison::PoisonId;
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::type_registry::TypeRegistry;

use crate::common::*;
use crate::leaf::creature_leaf;

// ---- PumpkinHead ----

fn pumpkin_head_new(w: &mut World) -> MobileId {
    let ai = if rnd::bool(w) {
        AIType::AiMelee
    } else {
        AIType::AiMage
    };
    let id = new_creature(
        w,
        &PUMPKIN_HEAD,
        ai,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    let b = 1246 + rnd::below(w, 2);
    body(w, id, b);
    base_sound_id(w, id, 268);
    set_str(w, id, 350);
    set_dex(w, id, 125);
    set_int(w, id, 250);
    set_hits(w, id, 500);
    set_mana(w, id, 1000);
    set_damage(w, id, 10, 15);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance(w, id, ResistanceType::Physical, 55);
    set_resistance(w, id, ResistanceType::Fire, 50);
    set_resistance(w, id, ResistanceType::Cold, 50);
    set_resistance(w, id, ResistanceType::Poison, 65);
    set_resistance(w, id, ResistanceType::Energy, 80);
    set_skill(w, id, SkillName::DetectHidden, 100.0);
    set_skill(w, id, SkillName::Meditation, 120.0);
    set_skill(w, id, SkillName::Necromancy, 100.0);
    set_skill(w, id, SkillName::SpiritSpeak, 120.0);
    set_skill(w, id, SkillName::Magery, 160.0);
    set_skill(w, id, SkillName::EvalInt, 100.0);
    set_skill(w, id, SkillName::MagicResist, 100.0);
    set_skill(w, id, SkillName::Tactics, 100.0);
    set_skill(w, id, SkillName::Wrestling, 80.0);
    fame(w, id, 5000);
    karma(w, id, -5000);
    virtual_armor(w, id, 49);
    id
}

/// `new Acid(TimeSpan.FromSeconds(10), 25, 30) { Name = "gooey nasty pumpkin hummus", Hue = 144 }`.
fn pumpkin_head_harmful_item(w: &mut World, _id: MobileId) -> Option<ItemId> {
    let acid = muo_items::misc::small::acid_new(
        w,
        &muo_items::misc::small::ACID,
        TimeSpan::from_secs_f64(10.0),
        TimeSpan::from_secs_f64(1.0),
        25,
        30,
    );
    muo_server::item::ops::set_name(w, acid, Some("gooey nasty pumpkin hummus".to_owned()));
    muo_items::simple::hue(w, acid, 144);
    Some(acid)
}

/// `OnDamage`: half of the time the pumpkin spills acid on the attacker (or on itself when it dies).
fn pumpkin_head_on_damage(
    w: &mut World,
    id: MobileId,
    amount: i32,
    from: Option<MobileId>,
    will_kill: bool,
) {
    if rnd::bool(w)
        && let Some(f) = from
        && mx::map(w, f).is_some()
        && mx::map(w, id).is_some_and(|m| m != MapId::INTERNAL)
        && mx::map(w, id) == mx::map(w, f)
        && mx::in_range(w, f, id, 12)
    {
        let target = if will_kill { id } else { f };
        muo_content::creature::pets::spill_acid(w, id, Some(target), if will_kill { 3 } else { 1 });
    }
    (BASE_CREATURE.on_damage)(w, id, amount, from, will_kill);
}

creature_leaf! {
    /// `Server.Mobiles.PumpkinHead`.
    PUMPKIN_HEAD, PUMPKIN_HEAD_C = "Server.Mobiles.PumpkinHead" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(pumpkin_head_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a killer pumpkin corpse"),
        default_name: |_w, _id| Some("a killer pumpkin"),
        auto_dispel: |_w, _id| true,
        bard_immune: |_w, _id| true,
        unprovokable: |_w, _id| true,
        area_peace_immune: |_w, _id| true,
        new_harmful_item: pumpkin_head_harmful_item,
        generate_loot: |w, id| {
            if rnd::double(w) < 0.05 {
                match rnd::below(w, 5) {
                    0 => pack_new(w, id, "Server.Items.Holiday.PaintedEvilClownMask"),
                    1 => pack_new(w, id, "Server.Items.Holiday.PaintedDaemonMask"),
                    2 => pack_new(w, id, "Server.Items.Holiday.PaintedPlagueMask"),
                    3 => pack_new(w, id, "Server.Items.Holiday.PaintedEvilJesterMask"),
                    _ => pack_new(w, id, "Server.Items.Holiday.PaintedPorcelainMask"),
                }
            }
            pack_new(w, id, "Server.Items.WrappedCandy");
            add_loot_n(w, id, loot_pack::ultra_rich(w), 2);
        },
    }
    m {
        on_damage: pumpkin_head_on_damage,
    }
}

// ---- ZombieSkeleton ----

/// `_deadPlayer`.
#[derive(Clone, Copy, Default)]
struct DeadPlayer(Option<MobileId>);

fn dead_player(w: &World, id: MobileId) -> Option<MobileId> {
    w.mobile(id)
        .and_then(|m| m.ext.get::<DeadPlayer>())
        .and_then(|d| d.0)
}

/// `ZombieSkeleton(player)`.
pub fn zombie_skeleton_new(w: &mut World, player: Option<MobileId>) -> MobileId {
    let id = new_creature(
        w,
        &ZOMBIE_SKELETON,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    if let Some(m) = w.mobile_mut(id) {
        m.ext.insert(DeadPlayer(player));
    }
    body(w, id, 0x93);
    base_sound_id(w, id, 0x1c3);
    set_str(w, id, 500);
    set_dex(w, id, 500);
    set_int(w, id, 500);
    set_hits(w, id, 2500);
    set_mana(w, id, 500);
    set_stam(w, id, 500);
    set_damage(w, id, 8, 18);
    set_damage_type(w, id, ResistanceType::Physical, 40);
    set_damage_type(w, id, ResistanceType::Cold, 60);
    set_resistance(w, id, ResistanceType::Fire, 50);
    set_resistance(w, id, ResistanceType::Energy, 50);
    set_resistance(w, id, ResistanceType::Physical, 50);
    set_resistance(w, id, ResistanceType::Cold, 50);
    set_resistance(w, id, ResistanceType::Poison, 50);
    set_skill_range(w, id, SkillName::MagicResist, 65.1, 80.0);
    set_skill_range(w, id, SkillName::Tactics, 95.1, 100.0);
    set_skill_range(w, id, SkillName::Wrestling, 85.1, 95.0);
    fame(w, id, 1000);
    karma(w, id, -1000);
    virtual_armor(w, id, 18);
    id
}

fn player_name(w: &World, p: MobileId) -> String {
    w.mobile(p)
        .and_then(|x| (x.ty.name)(w, p))
        .unwrap_or_default()
}

/// `DefaultName`: the dead player's name and "'s Zombie Skeleton", or "Zombie Skeleton".
fn zombie_name(w: &World, id: MobileId) -> Option<String> {
    let m = w.mobile(id)?;
    if m.name_mod.is_some() || m.name.is_some() {
        return (BASE_CREATURE.name)(w, id);
    }
    Some(match dead_player(w, id) {
        Some(p) => format!("{}{}", player_name(w, p), "'s Zombie Skeleton"),
        None => "Zombie Skeleton".to_string(),
    })
}

fn zombie_generate_loot(w: &mut World, id: MobileId) {
    let exists = dead_player(w, id).is_some_and(|p| !mx::deleted(w, p));
    match rnd::below(w, if exists { 8 } else { 10 }) {
        0 => pack_new(w, id, "Server.Items.LeftArm"),
        1 => pack_new(w, id, "Server.Items.RightArm"),
        2 => pack_new(w, id, "Server.Items.Torso"),
        3 => pack_new(w, id, "Server.Items.Bone"),
        4 => pack_new(w, id, "Server.Items.RibCage"),
        9 if exists => {
            let n = dead_player(w, id)
                .map(|p| player_name(w, p))
                .unwrap_or_default();
            let bones = muo_items::special::holiday4::player_bones_new(w, &n);
            pack_item(w, id, Some(bones));
        }
        _ => {}
    }
    add_loot(w, id, loot_pack::meager(w));
}

fn zombie_serialize(w: &World, id: MobileId, out: &mut dyn GenericWriter) {
    out.write_i32(0);
    out.write_mobile(w, dead_player(w, id));
}

fn zombie_deserialize(
    w: &mut World,
    id: MobileId,
    r: &mut dyn GenericReader,
) -> muo_io::Result<()> {
    let _version = r.read_i32()?;
    let p = r.read_mobile(w)?;
    if let Some(m) = w.mobile_mut(id) {
        m.ext.insert(DeadPlayer(p));
    }
    Ok(())
}

creature_leaf! {
    /// `Server.Engines.Events.ZombieSkeleton` (no `[Constructible]`).
    ZOMBIE_SKELETON, ZOMBIE_SKELETON_C = "Server.Engines.Events.ZombieSkeleton" : BASE_CREATURE / BASE_CREATURE_C, ctor None, state zombie_serialize, zombie_deserialize;
    c {
        corpse_name: |_w, _id| Some("a rotting corpse"),
        bleed_immune: |_w, _id| true,
        poison_immune: |_w, _id| Some(PoisonId(1)), // Poison.Regular
        generate_loot: zombie_generate_loot,
    }
    m {
        make: Some(|w| zombie_skeleton_new(w, None)),
        name: zombie_name,
        // `OnDelete()` does not call the base.
        on_delete: |w, id| {
            if let Some(p) = dead_player(w, id).filter(|&p| !mx::deleted(w, p)) {
                super::holiday_events::forget_zombie(w, p);
            }
        },
    }
}

// ---- NaughtyTwin ----

/// `m_From`.
#[derive(Clone, Copy, Default)]
struct TwinFrom(Option<MobileId>);

fn twin_from(w: &World, id: MobileId) -> Option<MobileId> {
    w.mobile(id)
        .and_then(|m| m.ext.get::<TwinFrom>())
        .and_then(|d| d.0)
}

/// `TrickOrTreat.CheckMobile(mobile)`.
pub(crate) fn check_mobile(w: &World, m: MobileId) -> bool {
    mx::map(w, m).is_some_and(|map| map != MapId::INTERNAL) && !mx::deleted(w, m) && mx::alive(w, m)
}

const FELUCCA_LOCATIONS: [(i32, i32, i32); 8] = [
    (4467, 1283, 5),   // Moonglow
    (1336, 1997, 5),   // Britain
    (1499, 3771, 5),   // Jhelom
    (771, 752, 5),     // Yew
    (2701, 692, 5),    // Minoc
    (1828, 2948, -20), // Trinsic
    (643, 2067, 5),    // Skara Brae
    (3563, 2139, 0),   // (New) Magincia: z is `Map.Trammel.GetAverageZ`
];
const MALAS_LOCATIONS: [(i32, i32, i32); 2] = [(1015, 527, -65), (1997, 1386, -85)]; // Luna, Umbra
const ILSHENAR_LOCATIONS: [(i32, i32, i32); 9] = [
    (1215, 467, -13),
    (722, 1366, -60),
    (744, 724, -28),
    (281, 1016, 0),
    (987, 1011, -32),
    (1174, 1286, -30),
    (1532, 1340, -3),
    (528, 216, -45),
    (1721, 218, 96),
];
const TOKUNO_LOCATIONS: [(i32, i32, i32); 3] = [(1169, 998, 41), (802, 1204, 25), (270, 628, 15)];

/// `NaughtyTwin.RandomMoongate(target)`.
fn random_moongate(w: &mut World, target: MobileId) -> Point3D {
    let list: &[(i32, i32, i32)] = match mx::map(w, target).map(|m| m.0) {
        Some(2) => &ILSHENAR_LOCATIONS,
        Some(3) => &MALAS_LOCATIONS,
        Some(4) => &TOKUNO_LOCATIONS,
        _ => &FELUCCA_LOCATIONS,
    };
    let (x, y, mut z) = list[rnd::below(w, list.len() as i32) as usize];
    if (x, y) == (3563, 2139) {
        z = w.maps.get_average_z(MapId::TRAMMEL, x, y);
    }
    Point3D::new(x, y, z)
}

/// `NaughtyTwin.StealCandyOrGate(target)`.
fn steal_candy_or_gate(w: &mut World, target: MobileId) {
    if !check_mobile(w, target) {
        return;
    }
    if rnd::bool(w) {
        const CANDY: [&str; 5] = [
            "Server.Items.WrappedCandy",
            "Server.Items.Lollipops",
            "Server.Items.NougatSwirl",
            "Server.Items.Taffy",
            "Server.Items.JellyBeans",
        ];
        let pack = w.mobile(target).and_then(|m| m.backpack);
        let item = pack.and_then(|p| muo_content::factory::find_item_by_types(w, p, &CANDY));
        mx::msg(w, target, 1113967); // Your naughty twin steals some of your candy.
        if let Some(i) = item.filter(|&i| !muo_server::item::ops::is_deleted(w, i)) {
            muo_server::item::base::delete(w, i);
        }
    } else {
        mx::msg(w, target, 1113972); // Your naughty twin teleports you away with a naughty laugh!
        let p = random_moongate(w, target);
        if let Some(map) = mx::map(w, target) {
            muo_server::mobile::base::move_to_world(w, target, p, map);
        }
    }
}

/// `NaughtyTwin(from)`.
pub fn naughty_twin_new(w: &mut World, from: Option<MobileId>) -> MobileId {
    let id = new_creature(
        w,
        &NAUGHTY_TWIN,
        AIType::AiMelee,
        FightMode::None,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    if let Some(f) = from.filter(|&f| check_mobile(w, f)) {
        let b = w.mobile(f).map_or(0, |m| m.cur_body().0);
        body(w, id, b);
        if let Some(m) = w.mobile_mut(id) {
            m.ext.insert(TwinFrom(Some(f)));
        }
        let n = player_name(w, f);
        name(w, id, Some(format!("{}{}", n, "'s Naughty Twin")));
        muo_server::timer::start(w, TimeSpan::from_secs(1), TimeSpan::ZERO, 1, move |w| {
            steal_candy_or_gate(w, f)
        });
    }
    id
}

creature_leaf! {
    /// `Server.Engines.Events.NaughtyTwin` (no `[Constructible]`).
    NAUGHTY_TWIN, NAUGHTY_TWIN_C = "Server.Engines.Events.NaughtyTwin" : BASE_CREATURE / BASE_CREATURE_C, ctor None, encoded false;
    c {
        on_think: |w, id| {
            if twin_from(w, id).is_none_or(|f| mx::deleted(w, f)) {
                muo_server::mobile::base::delete(w, id);
            }
        },
    }
    m {
        make: Some(|w| naughty_twin_new(w, None)),
    }
}

pub fn register(types: &mut TypeRegistry) {
    for t in [&PUMPKIN_HEAD, &ZOMBIE_SKELETON, &NAUGHTY_TWIN] {
        types.register_mobile(t, &[]);
    }
}
