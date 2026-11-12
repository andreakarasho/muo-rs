//! The paragon conversion of a creature. `ContentHooks` and the creature code
//! call it through the `paragon_*` fields set by [`initialize`].

use muo_content::creature::{self, speeds};
use muo_content::hooks::{hooks_mut, rnd};
use muo_content::mx;
use muo_primitives::{Point3D, TimeSpan};
use muo_server::ids::MobileId;
use muo_server::map::MapId;
use muo_server::mobile::part1::props_b;
use muo_server::skills::{self, SkillName};
use muo_server::timer;
use muo_server::world::World;

/// `ChestChance`, `ChocolateIngredientChance`.
pub const CHEST_CHANCE: f64 = 0.10;
pub const CHOCOLATE_INGREDIENT_CHANCE: f64 = 0.20;
/// `Paragon.Hue`.
pub const HUE: i32 = 0x501;
pub const HITS_BUFF: f64 = 5.0;
pub const STR_BUFF: f64 = 1.05;
pub const INT_BUFF: f64 = 1.20;
pub const DEX_BUFF: f64 = 1.20;
pub const SKILLS_BUFF: f64 = 1.20;
pub const SPEED_BUFF: f64 = 1.20;
pub const FAME_BUFF: f64 = 1.40;
pub const KARMA_BUFF: f64 = 1.40;
pub const DAMAGE_BUFF: i32 = 5;

const FAST_REGEN_RATE: TimeSpan = TimeSpan::from_millis(500);
const CPU_SAVER_RATE: TimeSpan = TimeSpan::from_secs(2);

/// `Paragon.Artifacts`.
pub const ARTIFACTS: &[&str] = &[
    "Server.Items.GoldBricks",
    "Server.Items.PhillipsWoodenSteed",
    "Server.Items.AlchemistsBauble",
    "Server.Items.ArcticDeathDealer",
    "Server.Items.BlazeOfDeath",
    "Server.Items.BowOfTheJukaKing",
    "Server.Items.BurglarsBandana",
    "Server.Items.CavortingClub",
    "Server.Items.EnchantedTitanLegBone",
    "Server.Items.GwennosHarp",
    "Server.Items.IolosLute",
    "Server.Items.LunaLance",
    "Server.Items.NightsKiss",
    "Server.Items.NoxRangersHeavyCrossbow",
    "Server.Items.OrcishVisage",
    "Server.Items.PolarBearMask",
    "Server.Items.ShieldOfInvulnerability",
    "Server.Items.StaffOfPower",
    "Server.Items.VioletCourage",
    "Server.Items.HeartOfTheLion",
    "Server.Items.WrathOfTheDryad",
    "Server.Items.PixieSwatter",
    "Server.Items.GlovesOfThePugilist",
];

fn is_paragon(w: &World, bc: MobileId) -> bool {
    creature::read(w, bc, false, |d| d.is_paragon)
}

/// Applies `f` to every skill base value that is above 0.
fn scale_skills(w: &mut World, bc: MobileId, f: impl Fn(f64) -> f64) {
    for i in 0..SkillName::COUNT {
        let Some(name) = SkillName::from_i32(i as i32) else {
            continue;
        };
        let base = skills::base(w, bc, name);
        if base > 0.0 {
            skills::set_base(w, bc, name, f(base));
        }
    }
}

fn refill(w: &mut World, bc: MobileId) {
    let (h, m, s) = w
        .mobile(bc)
        .map(|x| {
            (
                (x.ty.hits_max)(w, bc),
                (x.ty.mana_max)(w, bc),
                (x.ty.stam_max)(w, bc),
            )
        })
        .unwrap_or_default();
    props_b::set_hits(w, bc, h);
    props_b::set_mana(w, bc, m);
    props_b::set_stam(w, bc, s);
}

/// `Convert(bc)`.
pub fn convert(w: &mut World, bc: MobileId) {
    if is_paragon(w, bc) {
        return;
    }
    muo_server::mobile::base::set_hue(w, bc, HUE);

    if creature::read(w, bc, -1, |d| d.hits_max_seed) >= 0 {
        creature::edit(w, bc, |d| {
            d.hits_max_seed = (f64::from(d.hits_max_seed) * HITS_BUFF) as i32
        });
    }
    let (s, i, d) = w.mobile(bc).map_or((0, 0, 0), |m| (m.str, m.int, m.dex));
    props_b::set_raw_str(w, bc, (f64::from(s) * STR_BUFF) as i32);
    props_b::set_raw_int(w, bc, (f64::from(i) * INT_BUFF) as i32);
    props_b::set_raw_dex(w, bc, (f64::from(d) * DEX_BUFF) as i32);

    refill(w, bc);
    scale_skills(w, bc, |b| b * SKILLS_BUFF);

    creature::edit(w, bc, |d| {
        d.passive_speed /= SPEED_BUFF;
        d.active_speed /= SPEED_BUFF;
    });
    speeds::scale_move_speed(w, bc, 1.0 / SPEED_BUFF);
    let passive = creature::read(w, bc, 0.0, |d| d.passive_speed);
    speeds::set_current_speed(w, bc, passive);

    creature::edit(w, bc, |d| {
        d.damage_min += DAMAGE_BUFF;
        d.damage_max += DAMAGE_BUFF;
    });

    let fame = w.mobile(bc).map_or(0, |m| m.fame);
    if fame > 0 {
        muo_server::mobile::part1::props_a::set_fame(w, bc, (f64::from(fame) * FAME_BUFF) as i32);
    }
    if w.mobile(bc).map_or(0, |m| m.fame) > 32000 {
        muo_server::mobile::part1::props_a::set_fame(w, bc, 32000);
    }

    // TODO: mana regeneration rate = Sqrt(buffedFame) / 4.

    let karma = w.mobile(bc).map_or(0, |m| m.karma);
    if karma != 0 {
        let k = (f64::from(karma) * KARMA_BUFF) as i32;
        muo_server::mobile::part1::props_a::set_karma(w, bc, k);
        if k.abs() > 32000 {
            muo_server::mobile::part1::props_a::set_karma(w, bc, 32000 * k.signum());
        }
    }

    start_stam_regen(w, bc);
}

/// `UnConvert(bc)`.
pub fn un_convert(w: &mut World, bc: MobileId) {
    if !is_paragon(w, bc) {
        return;
    }
    muo_server::mobile::base::set_hue(w, bc, 0);

    if creature::read(w, bc, -1, |d| d.hits_max_seed) >= 0 {
        creature::edit(w, bc, |d| {
            d.hits_max_seed = (f64::from(d.hits_max_seed) / HITS_BUFF) as i32
        });
    }
    let (s, i, d) = w.mobile(bc).map_or((0, 0, 0), |m| (m.str, m.int, m.dex));
    props_b::set_raw_str(w, bc, (f64::from(s) / STR_BUFF) as i32);
    props_b::set_raw_int(w, bc, (f64::from(i) / INT_BUFF) as i32);
    props_b::set_raw_dex(w, bc, (f64::from(d) / DEX_BUFF) as i32);

    refill(w, bc);
    scale_skills(w, bc, |b| b / SKILLS_BUFF);

    creature::edit(w, bc, |d| {
        d.passive_speed *= SPEED_BUFF;
        d.active_speed *= SPEED_BUFF;
    });
    speeds::scale_move_speed(w, bc, SPEED_BUFF);
    speeds::snap_speeds_to_table(w, bc); // an ulp of scaling drift must not read as hand-tuned
    let passive = creature::read(w, bc, 0.0, |d| d.passive_speed);
    speeds::set_current_speed(w, bc, passive);

    creature::edit(w, bc, |d| {
        d.damage_min -= DAMAGE_BUFF;
        d.damage_max -= DAMAGE_BUFF;
    });

    let fame = w.mobile(bc).map_or(0, |m| m.fame);
    if fame > 0 {
        muo_server::mobile::part1::props_a::set_fame(w, bc, (f64::from(fame) / FAME_BUFF) as i32);
    }
    let karma = w.mobile(bc).map_or(0, |m| m.karma);
    if karma != 0 {
        muo_server::mobile::part1::props_a::set_karma(
            w,
            bc,
            (f64::from(karma) / KARMA_BUFF) as i32,
        );
    }
}

/// `Math.Round(x)`: to even on a tie.
fn round_half_even(x: f64) -> f64 {
    let r = x.round();
    if (x - x.trunc()).abs() == 0.5 && r % 2.0 != 0.0 {
        r - x.signum()
    } else {
        r
    }
}

/// `CheckConvert(bc, location, map)`.
pub fn check_convert(w: &mut World, bc: MobileId, _location: Point3D, map: MapId) -> bool {
    if !w.core_aos() {
        return false;
    }
    if map != MapId::ILSHENAR {
        return false;
    }
    let is = |n: &str| muo_content::factory::mobile_is(w, bc, n);
    if is("Server.Mobiles.BaseChampion")
        || is("Server.Mobiles.Harrower")
        || muo_content::vendor::is_vendor(w, bc)
        || is("Server.Mobiles.BaseEscortable")
        || is("Server.Mobiles.Clone")
        || is_paragon(w, bc)
    {
        return false;
    }
    let fame = w.mobile(bc).map_or(0, |m| m.fame).min(32000);
    let chance = 1.0 / round_half_even(20.0 - f64::from(fame) / 3200.0);
    chance > rnd::double(w)
}

/// `CheckArtifactChance(m, bc)`.
pub fn check_artifact_chance(w: &mut World, m: MobileId, bc: MobileId) -> bool {
    if !w.core_aos() {
        return false;
    }
    let fame = f64::from(w.mobile(bc).map_or(0, |x| x.fame).min(32000));
    let round3 = |x: f64| (x * 1000.0).round_ties_even() / 1000.0;
    let luck = f64::from(muo_server::mobile::base::luck(w, m));
    let chance = 1.0
        / ((100.0 * (0.83 - round3((round3(fame / 6000.0) + 0.001).log10()))).max(10.0)
            * (100.0 - luck.sqrt())
            / 100.0);
    chance > rnd::double(w)
}

/// `GiveArtifactTo(m)`.
pub fn give_artifact_to(w: &mut World, m: MobileId) {
    let i = rnd::below(w, ARTIFACTS.len() as i32) as usize;
    let Some(item) = muo_content::factory::make_item(w, ARTIFACTS[i]) else {
        return;
    };
    if muo_server::mobile::part2::items::add_to_backpack(w, m, item) {
        mx::send_message(
            w,
            m,
            "As a reward for slaying the mighty paragon, an artifact has been placed in your backpack.",
        );
    } else {
        mx::send_message(
            w,
            m,
            "As your backpack is full, your reward for destroying the legendary paragon has been placed at your feet.",
        );
    }
}

/// `MondainsLegacy.Artifacts`.
const ML_ARTIFACTS: &[&str] = &[
    "Server.Items.AegisOfGrace",
    "Server.Items.BladeDance",
    "Server.Items.BloodwoodSpirit",
    "Server.Items.Bonesmasher",
    "Server.Items.Boomstick",
    "Server.Items.BrightsightLenses",
    "Server.Items.FeyLeggings",
    "Server.Items.FleshRipper",
    "Server.Items.HelmOfSwiftness",
    "Server.Items.PadsOfTheCuSidhe",
    "Server.Items.QuiverOfRage",
    "Server.Items.QuiverOfElements",
    "Server.Items.RaedsGlory",
    "Server.Items.RighteousAnger",
    "Server.Items.RobeOfTheEclipse",
    "Server.Items.RobeOfTheEquinox",
    "Server.Items.SoulSeeker",
    "Server.Items.TalonBite",
    "Server.Items.TotemOfVoid",
    "Server.Items.WildfireBow",
    "Server.Items.Windsong",
];

/// `MondainsLegacy.CheckArtifactChance(m, bc)`.
pub fn ml_check_artifact_chance(w: &mut World, m: MobileId, bc: MobileId) -> bool {
    w.core_ml() && check_artifact_chance(w, m, bc)
}

/// `MondainsLegacy.GiveArtifactTo(m)`.
pub fn ml_give_artifact_to(w: &mut World, m: MobileId) {
    let i = rnd::below(w, ML_ARTIFACTS.len() as i32) as usize;
    let Some(item) = muo_content::factory::make_item(w, ML_ARTIFACTS[i]) else {
        return;
    };
    let msg = |w: &mut World, n: i32| {
        muo_server::mobile::part2::messages::send_localized_message(w, m, n, "", 0x3B2)
    };
    if muo_server::mobile::part2::items::add_to_backpack(w, m, item) {
        msg(w, 1072223); // An item has been placed in your backpack.
        msg(w, 1062317); // For your valor in combating the fallen beast, a special artifact has been bestowed on you.
    } else if let Some(bank) = props_b::bank_box(w, m)
        && muo_server::item::container::try_drop_item(w, bank, Some(m), item, false)
    {
        msg(w, 1072224); // An item has been placed in your bank box.
        msg(w, 1062317);
    } else {
        msg(w, 1072523); // You find an artifact, but your backpack and bank are too full to hold it.
    }
}

/// `new ParagonStamRegen(bc).Start()`: +1 stamina every half second (every two seconds above 75%).
fn start_stam_regen(w: &mut World, bc: MobileId) {
    timer::start(w, FAST_REGEN_RATE, FAST_REGEN_RATE, 0, move |w| {
        stam_regen_tick(w, bc)
    });
}

fn stam_regen_tick(w: &mut World, bc: MobileId) {
    let alive = !mx::deleted(w, bc)
        && is_paragon(w, bc)
        && w.mobile(bc).is_some_and(|m| m.map != Some(MapId::INTERNAL));
    let cur = timer::current(w);
    if alive {
        let (stam, max) = w
            .mobile(bc)
            .map_or((0, 0), |m| (m.stam, (m.ty.stam_max)(w, bc)));
        props_b::set_stam(w, bc, stam + 1);
        let stam = w.mobile(bc).map_or(0, |m| m.stam);
        if let Some(t) = cur {
            let d = if f64::from(stam) < f64::from(max) * 0.75 {
                FAST_REGEN_RATE
            } else {
                CPU_SAVER_RATE
            };
            timer::set_delay(w, t, d);
            timer::set_interval(w, t, d);
        }
    } else if let Some(mut t) = cur {
        t.cancel(w);
    }
}

/// Phase `Initialize`: the creature code reaches the paragon system through `ContentHooks`.
pub fn initialize(w: &mut World) {
    let h = hooks_mut(w);
    h.paragon_check_convert = Some(check_convert);
    h.paragon_convert = Some(|w, bc, on| {
        if on {
            convert(w, bc)
        } else {
            un_convert(w, bc)
        }
    });
    h.paragon_hits_buff = Some(HITS_BUFF);
    h.paragon_chest_chance = Some(CHEST_CHANCE);
    h.paragon_chocolate_chance = Some(CHOCOLATE_INGREDIENT_CHANCE);
    // `BaseCreature.OnKilledBy(mob)`'s paragon branch (`GivesMLMinorArtifact` creatures take
    // `MondainsLegacy.CheckArtifactChance/GiveArtifactTo` instead).
    muo_content::hooks::world_hooks_mut(w).on_killed_by_rights_holder = Some(|w, bc, mob| {
        let ml = muo_content::creature::vtable::ctype(w, bc)
            .is_some_and(|c| (c.gives_ml_minor_artifact)(w, bc));
        if ml {
            if ml_check_artifact_chance(w, mob, bc) {
                ml_give_artifact_to(w, mob);
            }
        } else if is_paragon(w, bc) && check_artifact_chance(w, mob, bc) {
            give_artifact_to(w, mob);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rounding_goes_to_even_on_a_tie() {
        assert_eq!(round_half_even(2.5), 2.0);
        assert_eq!(round_half_even(3.5), 4.0);
        assert_eq!(round_half_even(19.6), 20.0);
        assert_eq!(round_half_even(-2.5), -2.0);
    }
}
