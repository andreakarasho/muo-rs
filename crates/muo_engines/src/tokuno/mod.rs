//! Treasures of Tokuno: `TreasuresOfTokuno` (eras, config, `HandleKill`), the lesser and greater artifacts,
//! `BasePigmentsOfTokuno` and `TreasuresOfTokunoPersistence`.

use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::item::ItemType;
use muo_server::type_registry::{DEFAULT_CALL_PRIORITY, Phase, TypeRegistry};
use serde::{Deserialize, Serialize};

pub mod artifacts;
pub mod gumps;
pub mod lists;
pub mod pigments;
pub mod vendor;

pub const CONFIGURATION_PATH: &str = "Configuration/tot.json";
pub const ITEMS_PER_REWARD: i32 = 10;

/// `TreasuresOfTokunoEra`.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum Era {
    #[default]
    None,
    ToTOne,
    ToTTwo,
    ToTThree,
}

impl Era {
    pub fn from_i32(v: i32) -> Era {
        match v {
            1 => Era::ToTOne,
            2 => Era::ToTTwo,
            3 => Era::ToTThree,
            _ => Era::None,
        }
    }
}

/// `ToTConfig`.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ToTConfig {
    pub drop_era: Era,
    pub reward_era: Era,
}

impl Default for ToTConfig {
    fn default() -> Self {
        ToTConfig {
            drop_era: Era::None,
            reward_era: Era::ToTOne,
        }
    }
}

/// `TreasuresOfTokuno.Configure`: reads `Configuration/tot.json` (a missing file is the default, not written).
pub fn configure(w: &mut World) {
    let path = w.core.base_directory.join(CONFIGURATION_PATH);
    let cfg = match muo_data::json::deserialize_file::<ToTConfig>(&path) {
        Ok(Some(c)) => c,
        Ok(None) => ToTConfig::default(),
        Err(e) => {
            tracing::warn!("{CONFIGURATION_PATH}: {e}; using defaults");
            ToTConfig::default()
        }
    };
    w.resources.insert(cfg);
}

fn config(w: &World) -> ToTConfig {
    w.resources.get::<ToTConfig>().cloned().unwrap_or_default()
}

fn save_configuration(w: &mut World) {
    let path = w.core.base_directory.join(CONFIGURATION_PATH);
    let cfg = config(w);
    if let Err(e) = muo_data::json::serialize_file(&path, &cfg) {
        tracing::warn!("Could not write {CONFIGURATION_PATH}: {e}");
    }
}

pub fn drop_era(w: &World) -> Era {
    config(w).drop_era
}

pub fn reward_era(w: &World) -> Era {
    config(w).reward_era
}

/// `DropEra = value` (saves the configuration).
pub fn set_drop_era(w: &mut World, value: Era) {
    let mut c = config(w);
    c.drop_era = value;
    w.resources.insert(c);
    save_configuration(w);
}

/// `RewardEra = value` (saves the configuration).
pub fn set_reward_era(w: &mut World, value: Era) {
    let mut c = config(w);
    c.reward_era = value;
    w.resources.insert(c);
    save_configuration(w);
}

/// `m_LesserArtifacts[(int)era - 1]`.
fn lesser_by_era(era: Era) -> &'static [&'static str] {
    match era {
        Era::ToTOne => lists::LESSER_ARTIFACTS_ONE,
        Era::ToTTwo => lists::LESSER_ARTIFACTS_TWO,
        Era::ToTThree => lists::LESSER_ARTIFACTS_THREE,
        Era::None => &[],
    }
}

/// `type.InTypeList(names)`: the class or one of its parents is listed.
pub fn in_names(ty: &'static ItemType, names: &[&str]) -> bool {
    let mut t = Some(ty);
    while let Some(c) = t {
        if names.contains(&c.full_name) {
            return true;
        }
        t = c.parent;
    }
    false
}

/// `Region.IsPartOf<HouseRegion>()`, `BaseBoat.FindBoatAt` and the two Tokuno dungeon regions.
fn check_location(w: &World, m: MobileId) -> bool {
    let Some(mob) = w.mobile(m) else { return false };
    let in_house = muo_content::hooks::world_hooks(w)
        .and_then(|h| h.in_house_region)
        .is_some_and(|f| f(w, m));
    if in_house || muo_multis::boats::find_boat_at(w, mob.location, mob.map).is_some() {
        return false;
    }
    // TODO: a CanReach of something check as opposed to above?
    if muo_server::region::is_part_of_name(w, mob.region, "Yomotsu Mines", false)
        || muo_server::region::is_part_of_name(w, mob.region, "Fan Dancer's Dojo", false)
    {
        return true;
    }
    mob.map == Some(muo_server::map::MapId::TOKUNO)
}

/// `TreasuresOfTokuno.HandleKill(victim, killer)`.
pub fn handle_kill(w: &mut World, victim: MobileId, killer: MobileId) {
    use muo_content::player_mobile as pmob;
    let era = drop_era(w);
    if era == Era::None || pmob::data(w, killer).is_none() {
        return;
    }
    let Some(cd) = muo_content::creature::cd(w, victim) else {
        return;
    };
    let (controlled, owners) = (cd.controlled, cd.owners.len());
    if !check_location(w, victim)
        || !check_location(w, killer)
        || !muo_content::mx::in_range(w, killer, victim, 18)
    {
        return;
    }
    let fame = w.mobile(victim).map_or(0, |m| m.fame);
    if controlled || owners > 0 || fame <= 0 {
        return;
    }

    // 25000 for 1/100 chance, 10 hyrus
    // 1500, 1/1000 chance, 20 lizard men for that chance.
    let luck = f64::from(muo_server::mobile::base::luck(w, killer));
    let gain = (f64::from(fame) * (1.0 + luck.sqrt() / 100.0)) as i32;
    let x = {
        let Some(d) = pmob::data_mut(w, killer) else {
            return;
        };
        d.tot_total_monster_fame = d.tot_total_monster_fame.wrapping_add(gain);
        d.tot_total_monster_fame
    };

    // The exponential regression with only 2 datapoints (not OSI exact).
    const A: f64 = 0.000863316841;
    const B: f64 = 0.00000425531915;
    let chance = A * 10f64.powf(B * f64::from(x));
    if chance <= muo_content::hooks::rnd::double(w) {
        return;
    }

    let list = lesser_by_era(era);
    let n = muo_content::hooks::rnd::below(w, list.len() as i32) as usize;
    let Some(item) = muo_content::factory::make_item(w, list[n]) else {
        return;
    };

    muo_content::mx::msg(w, killer, 1062317); // For your valor in combating the fallen beast, a special artifact has been bestowed on you.
    if !muo_server::mobile::part2::items::place_in_backpack(w, killer, item) {
        let bank = muo_server::mobile::part1::props_b::bank_box(w, killer);
        let banked = bank.is_some_and(|b| {
            muo_server::item::container::try_drop_item(w, b, Some(killer), item, false)
        });
        if banked {
            muo_content::mx::msg(w, killer, 1079730); // The item has been placed into your bank box.
        } else {
            // You find an artifact, but your backpack and bank are too full to hold it.
            muo_content::mx::msg(w, killer, 1072523);
            let (loc, map) = (
                muo_content::mx::location(w, killer),
                muo_content::mx::map(w, killer),
            );
            muo_server::item::ops::move_to_world_map(w, item, loc, map);
        }
    }
    if let Some(d) = pmob::data_mut(w, killer) {
        d.tot_total_monster_fame = 0;
    }
}

/// `TreasuresOfTokunoPersistence`: the old save item (`TypeAlias("Server.Misc.TreasuresOfTokunoPersistance")`) whose
/// eras move into the configuration file when it loads; it deletes itself.
pub static PERSISTENCE: ItemType = ItemType {
    full_name: "Server.Misc.TreasuresOfTokunoPersistence",
    parent: Some(&muo_server::item::ITEM_BASE),
    construct: None,
    make: Some(|w| {
        let id = muo_server::item::ops::new_item_with_id(w, &PERSISTENCE, 1);
        muo_server::item::ops::set_movable(w, id, false);
        id
    }),
    serialize: |w, id, out| (muo_server::item::ITEM_BASE.serialize)(w, id, out),
    deserialize: |w, id, r| {
        (muo_server::item::ITEM_BASE.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        let reward = Era::from_i32(r.read_encoded_int()?);
        let drop = Era::from_i32(r.read_encoded_int()?);
        set_reward_era(w, reward);
        set_drop_era(w, drop);
        muo_server::timer::start(
            w,
            muo_primitives::TimeSpan::ZERO,
            muo_primitives::TimeSpan::ZERO,
            1,
            move |w| {
                if let Some(t) = w.item(id).map(|i| i.ty) {
                    (t.delete)(w, id);
                }
            },
        );
        Ok(())
    },
    ..muo_server::item::ITEM_BASE
};

/// Registers the classes and phases of the folder, and the hooks it owns.
pub fn register(types: &mut TypeRegistry) {
    artifacts::register(types);
    pigments::register(types);
    vendor::register(types);
    types.register_item(&PERSISTENCE, &["Server.Misc.TreasuresOfTokunoPersistance"]);
    types.register_phase(
        Phase::Configure,
        "TreasuresOfTokuno.Configure",
        DEFAULT_CALL_PRIORITY,
        |w| {
            configure(w);
            let h = w.resources.get_mut::<muo_gumps::m_z::hooks::MzHooks>();
            h.tot_drop_era = Some(|w| drop_era(w) as i32);
            h.tot_reward_era = Some(|w| reward_era(w) as i32);
            h.tot_set_drop_era = Some(|w, v| set_drop_era(w, Era::from_i32(v)));
            h.tot_set_reward_era = Some(|w, v| set_reward_era(w, Era::from_i32(v)));
            w.resources
                .get_mut::<muo_content::hooks::DeathHooks>()
                .on_death_rights_holder =
                Some(|w, victim, killer, _corpse| handle_kill(w, victim, killer));
        },
    );
}
