//! `BaseTalismanSummon` and the seventeen creatures a talisman
//! summons. Each is `Body`, `BaseSoundID` (and sometimes a hue) on a melee `BaseCreature`, saved as
//! `[SerializationGenerator(0)]` (an encoded version).

use std::rc::Rc;

use muo_content::creature::enums::{AIType, DEFAULT_RANGE_PERCEPTION, FightMode};
use muo_content::creature::setters::new_creature;
use muo_content::creature::{BASE_CREATURE, BASE_CREATURE_C, CreatureType};
use muo_primitives::{Body, TimeSpan, utility};
use muo_server::World;
use muo_server::context_menu_entry::ContextMenuEntry;
use muo_server::ids::{EntityId, MobileId};
use muo_server::item::effect_item;
use muo_server::mobile::MobileType;
use muo_server::type_registry::TypeRegistry;

/// `BaseTalismanSummon()`: `base(AIType.AI_Melee, FightMode.None)`.
pub fn new_summon(w: &mut World, ty: &'static MobileType, body: i32, sound: i32) -> MobileId {
    let id = new_creature(
        w,
        ty,
        AIType::AiMelee,
        FightMode::None,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    muo_server::mobile::part1::props_b::set_body(w, id, Body::new(body));
    if let Some(m) = w.mobile_mut(id) {
        m.base_sound_id = sound;
    }
    id
}

/// `TalismanReleaseEntry` (`base(6118, 3)`): the particles and sound at the owner, then `target.Delete()`.
fn release_entry() -> ContextMenuEntry {
    let mut e = ContextMenuEntry::new(6118, 3);
    e.on_click = Some(Rc::new(
        |w: &mut World, from: MobileId, target: EntityId| {
            if let Some((loc, map)) = w.mobile(from).map(|m| (m.location, m.map)) {
                let fx = effect_item::create(w, loc, map, effect_item::DEFAULT_DURATION);
                muo_server::effects::send_location_particles(w, fx, 0x3728, 8, 20, 5042);
            }
            muo_server::effects::play_sound_entity(w, from, 0x201);
            if let EntityId::Mobile(m) = target
                && let Some(ty) = w.mobile(m).map(|m| m.ty)
            {
                (ty.delete)(w, m);
            }
        },
    ));
    e
}

/// `BaseTalismanSummon`' constants: not commandable, initially innocent, a release entry for the master.
pub static BASE_TALISMAN_SUMMON_C: CreatureType = CreatureType {
    commandable: |_, _| false,
    initial_innocent: |_, _| true,
    add_custom_context_entries: |w, id, from, list| {
        let alive = muo_content::mx::alive(w, from);
        let master = muo_content::creature::cd(w, id).and_then(|d| d.master);
        if alive && master == Some(from) {
            list.push(release_entry());
        }
    },
    ..BASE_CREATURE_C
};

/// `Server.Mobiles.BaseTalismanSummon` (`[SerializationGenerator(0)]`; not constructible, it has no `[Constructible]`).
pub static BASE_TALISMAN_SUMMON: MobileType = MobileType {
    full_name: "Server.Mobiles.BaseTalismanSummon",
    parent: Some(&BASE_CREATURE),
    construct: None,
    family: Some(&BASE_TALISMAN_SUMMON_C),
    serialize: |w, id, out| {
        (BASE_CREATURE.serialize)(w, id, out);
        out.write_encoded_int(0);
    },
    deserialize: |w, id, r| {
        (BASE_CREATURE.deserialize)(w, id, r)?;
        let _version = r.read_encoded_int()?;
        Ok(())
    },
    ..BASE_CREATURE
};

macro_rules! think_or_base {
    () => {
        BASE_TALISMAN_SUMMON_C.on_think
    };
    ($t:expr) => {
        $t
    };
}

/// One summon: `init` runs after `Body`/`BaseSoundID` (the other statements of the constructor).
macro_rules! summon_leaves {
    ($($(#[$m:meta])* $name:ident, $class:ident = $full:literal, body $body:expr, sound $sound:expr, name $dn:literal $(, think $think:expr)? $(, init $init:expr)?;)*) => {
        $(
            /// The constants of the class.
            pub static $class: CreatureType = CreatureType {
                default_name: |_, _| Some($dn),
                on_think: think_or_base!($($think)?),
                ..BASE_TALISMAN_SUMMON_C
            };

            $(#[$m])*
            pub static $name: MobileType = MobileType {
                full_name: $full,
                parent: Some(&BASE_TALISMAN_SUMMON),
                construct: Some(|w| {
                    let body_of: fn(&mut World) -> i32 = $body;
                    let body = body_of(w);
                    let id = new_summon(w, &$name, body, $sound);
                    $( let init: fn(&mut World, MobileId) = $init; init(w, id); )?
                    id
                }),
                family: Some(&$class),
                // Each leaf is `[SerializationGenerator(0)]` too: its own version after the base's.
                serialize: |w, id, out| {
                    (BASE_TALISMAN_SUMMON.serialize)(w, id, out);
                    out.write_encoded_int(0);
                },
                deserialize: |w, id, r| {
                    (BASE_TALISMAN_SUMMON.deserialize)(w, id, r)?;
                    let _version = r.read_encoded_int()?;
                    Ok(())
                },
                ..BASE_TALISMAN_SUMMON
            };
        )*

        /// Every summon class.
        pub static LEAVES: &[&MobileType] = &[$(&$name),*];
    };
}

fn hue(w: &mut World, id: MobileId, value: i32) {
    if let Some(ty) = w.mobile(id).map(|m| m.ty) {
        (ty.set_hue)(w, id, value);
    }
}

/// `Utility.RandomList(..)`.
fn random_list(w: &mut World, list: &[i32]) -> i32 {
    list[utility::random(&mut w.rng, list.len() as i32) as usize]
}

summon_leaves! {
    /// `Server.Mobiles.SummonedAntLion`.
    SUMMONED_ANT_LION, SUMMONED_ANT_LION_C = "Server.Mobiles.SummonedAntLion", body |_| 787, sound 1006, name "an ant lion";
    /// `Server.Mobiles.SummonedArcticOgreLord`.
    SUMMONED_ARCTIC_OGRE_LORD, SUMMONED_ARCTIC_OGRE_LORD_C = "Server.Mobiles.SummonedArcticOgreLord", body |_| 135, sound 427, name "an arctic ogre lord";
    /// `Server.Mobiles.SummonedBakeKitsune`.
    SUMMONED_BAKE_KITSUNE, SUMMONED_BAKE_KITSUNE_C = "Server.Mobiles.SummonedBakeKitsune", body |_| 246, sound 0x4DD, name "a bake kitsune";
    /// `Server.Mobiles.SummonedBogling`.
    SUMMONED_BOGLING, SUMMONED_BOGLING_C = "Server.Mobiles.SummonedBogling", body |_| 779, sound 422, name "a bogling";
    /// `Server.Mobiles.SummonedBullFrog` (`Hue = Utility.RandomList(..)` between `Body` and `BaseSoundID`).
    SUMMONED_BULL_FROG, SUMMONED_BULL_FROG_C = "Server.Mobiles.SummonedBullFrog", body |_| 81, sound 0x266, name "a bull frog",
        init |w, id| {
            let h = random_list(w, &[0x5AC, 0x5A3, 0x59A, 0x591, 0x588, 0x57F]);
            hue(w, id, h);
        };
    /// `Server.Mobiles.SummonedChicken`.
    SUMMONED_CHICKEN, SUMMONED_CHICKEN_C = "Server.Mobiles.SummonedChicken", body |_| 0xD0, sound 0x6E, name "a chicken";
    /// `Server.Mobiles.SummonedCow` (`Body = Utility.RandomList(0xD8, 0xE7)`).
    SUMMONED_COW, SUMMONED_COW_C = "Server.Mobiles.SummonedCow", body |w| random_list(w, &[0xD8, 0xE7]), sound 0x78, name "a cow";
    /// `Server.Mobiles.SummonedDoppleganger`.
    SUMMONED_DOPPLEGANGER, SUMMONED_DOPPLEGANGER_C = "Server.Mobiles.SummonedDoppleganger", body |_| 0x309, sound 0x451, name "a doppleganger";
    /// `Server.Mobiles.SummonedFrostSpider`.
    SUMMONED_FROST_SPIDER, SUMMONED_FROST_SPIDER_C = "Server.Mobiles.SummonedFrostSpider", body |_| 20, sound 0x388, name "a frost spider";
    /// `Server.Mobiles.SummonedGreatHart`.
    SUMMONED_GREAT_HART, SUMMONED_GREAT_HART_C = "Server.Mobiles.SummonedGreatHart", body |_| 0xEA, sound 0x82, name "a great hart";
    /// `Server.Mobiles.SummonedLavaSerpent` (`OnThink` is overridden with an empty body).
    SUMMONED_LAVA_SERPENT, SUMMONED_LAVA_SERPENT_C = "Server.Mobiles.SummonedLavaSerpent", body |_| 90, sound 219, name "a lava serpent", think |_, _| {};
    /// `Server.Mobiles.SummonedOrcBrute`.
    SUMMONED_ORC_BRUTE, SUMMONED_ORC_BRUTE_C = "Server.Mobiles.SummonedOrcBrute", body |_| 189, sound 0x45A, name "an orc brute";
    /// `Server.Mobiles.SummonedPanther`.
    SUMMONED_PANTHER, SUMMONED_PANTHER_C = "Server.Mobiles.SummonedPanther", body |_| 0xD6, sound 0x462, name "a panther",
        init |w, id| hue(w, id, 0x901);
    /// `Server.Mobiles.SummonedSheep`.
    SUMMONED_SHEEP, SUMMONED_SHEEP_C = "Server.Mobiles.SummonedSheep", body |_| 0xCF, sound 0xD6, name "a sheep";
    /// `Server.Mobiles.SummonedSkeletalKnight`.
    SUMMONED_SKELETAL_KNIGHT, SUMMONED_SKELETAL_KNIGHT_C = "Server.Mobiles.SummonedSkeletalKnight", body |_| 147, sound 451, name "a skeletal knight";
    /// `Server.Mobiles.SummonedVorpalBunny` (`Hue = 0x480`, and after 30 minutes it digs back to its lair).
    SUMMONED_VORPAL_BUNNY, SUMMONED_VORPAL_BUNNY_C = "Server.Mobiles.SummonedVorpalBunny", body |_| 205, sound 0xC9, name "a vorpal bunny",
        init |w, id| {
            hue(w, id, 0x480);
            begin_tunnel_later(w, id);
        };
    /// `Server.Mobiles.SummonedWailingBanshee`.
    SUMMONED_WAILING_BANSHEE, SUMMONED_WAILING_BANSHEE_C = "Server.Mobiles.SummonedWailingBanshee", body |_| 310, sound 0x482, name "a wailing banshee";
}

/// `Timer.StartTimer(TimeSpan.FromMinutes(30.0), BeginTunnel)`.
fn begin_tunnel_later(w: &mut World, id: MobileId) {
    muo_server::timer::start(
        w,
        TimeSpan::from_secs(30 * 60),
        TimeSpan::ZERO,
        1,
        move |w| begin_tunnel(w, id),
    );
}

/// `BeginTunnel()`: a `VorpalBunny.BunnyHole` at the bunny, the bunny frozen, then gone after 5 seconds.
pub fn begin_tunnel(w: &mut World, id: MobileId) {
    if w.mobile(id).is_none_or(|m| m.deleted) {
        return;
    }
    let (loc, map) = w
        .mobile(id)
        .map(|m| (m.location, m.map))
        .unwrap_or_default();
    if let Some(hole) = muo_content::factory::make_item(w, "Server.Mobiles.VorpalBunny+BunnyHole") {
        muo_server::item::ops::move_to_world_map(w, hole, loc, map);
    }
    muo_server::mobile::part1::props_a::set_frozen(w, id, true);
    // * The bunny begins to dig a tunnel back to its underground lair *
    muo_server::mobile::part2::messages::say_number(w, id, 1114450, "");
    muo_server::mobile::part2::effects::play_sound(w, id, 0x247);
    muo_server::timer::start(w, TimeSpan::from_secs(5), TimeSpan::ZERO, 1, move |w| {
        if let Some(ty) = w.mobile(id).map(|m| m.ty) {
            (ty.delete)(w, id);
        }
    });
}

/// Registers the classes of this file.
pub fn register(types: &mut TypeRegistry) {
    types.register_mobile(&BASE_TALISMAN_SUMMON, &[]);
    for t in LEAVES {
        types.register_mobile(t, &[]);
    }
}
