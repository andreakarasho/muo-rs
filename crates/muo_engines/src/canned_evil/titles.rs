//! `ChampionTitleContext`, `ChampionTitleSystem` and `ChampionTitle`: the behavior over the
//! data in `muo_content::player_mobile::tables` (the persistence file and its migrations are there).

use muo_content::player_mobile::tables::{self, ChampionTitle, ChampionTitleContext};
use muo_primitives::{DateTime, TimeSpan};
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::timer;

use super::info::ChampionSpawnType;

const LOSS_AMOUNT: i32 = 90;
const LOSS_DELAY: TimeSpan = TimeSpan::from_days(1);

/// The slot of a `ChampionSpawnType` in `ChampionTitleContext::titles`.
fn slot(t: ChampionSpawnType) -> usize {
    use ChampionSpawnType::*;
    match t {
        Abyss => 0,
        Arachnid => 1,
        ColdBlood => 2,
        ForestLord => 3,
        VerminHorde => 4,
        UnholyTerror => 5,
        SleepingDragon => 6,
        Corrupt => 7,
        Glade => 8,
    }
}

/// `ChampionTitleContext.Award(type, value)`: creates the player's context like `GetOrCreateChampionTitleContext`.
pub fn award(w: &mut World, pm: MobileId, t: ChampionSpawnType, value: i32) {
    let now = w.now;
    let c = tables::champion_titles_mut(w, pm);
    let title = c.titles[slot(t)].get_or_insert_with(ChampionTitle::default);
    title.value += value;
    if title.last_decay == DateTime::MIN {
        title.last_decay = now;
    }
}

/// `ChampionTitleContext.SetValue(type, value)`.
pub fn set_value(w: &mut World, pm: MobileId, t: ChampionSpawnType, value: i32) {
    tables::set_title_value(w, pm, slot(t), value);
}

/// `ChampionTitleSystem.AwardHarrowerTitle(pm)`: at least 1 point, one more per title above 900.
pub fn award_harrower_title(w: &mut World, pm: MobileId) {
    let c = tables::champion_titles_mut(w, pm);
    let mut count = 1;
    for t in c.titles.iter().flatten() {
        if t.value > 900 {
            count += 1;
        }
    }
    c.harrower = count.max(c.harrower); // Harrower titles never decay.
}

/// `ChampionTitleContext.CheckAtrophy()`: false when nothing is left.
fn check_atrophy(c: &mut ChampionTitleContext, now: DateTime) -> bool {
    let mut values_used = c.harrower > 0;
    // Titles in `ChampionSpawnInfo.Table` order, which is the enum's.
    for t in 0..9 {
        let Some(ty) = ChampionSpawnType::from_i32(t) else {
            continue;
        };
        let s = slot(ty);
        let Some(title) = c.titles[s].as_mut() else {
            continue;
        };
        let decay = title.last_decay;
        if decay > DateTime::MIN && decay + LOSS_DELAY < now {
            // `ChampionTitle.Atrophy(LossAmount)`
            let before = title.value;
            title.value -= LOSS_AMOUNT.min(before);
            if before != title.value {
                title.last_decay = now;
            }
            let used = title.value > 0;
            if !used {
                c.titles[s] = None;
            }
            values_used |= used;
        }
    }
    values_used
}

/// The 5 minute `ChampionTitleTimer`.
fn atrophy_tick(w: &mut World) {
    let now = w.now;
    let players: Vec<MobileId> = match w.resources.get::<tables::ChampionTitleTable>() {
        Some(t) => t.0.keys().copied().collect(),
        None => return,
    };
    for pm in players {
        let keep = match w
            .resources
            .get_mut::<tables::ChampionTitleTable>()
            .0
            .get_mut(&pm)
        {
            Some(c) => check_atrophy(c, now),
            None => continue,
        };
        if !keep {
            w.resources
                .get_mut::<tables::ChampionTitleTable>()
                .0
                .shift_remove(&pm);
        }
    }
}

/// `ChampionTitleSystem.Initialize()`.
pub fn initialize(w: &mut World) {
    // `[OnEvent(nameof(PlayerMobile.PlayerDeletedEvent))] OnPlayerDeleted`
    muo_content::player_mobile::subscribe_deleted(w, |w, pm| {
        w.resources
            .get_mut::<tables::ChampionTitleTable>()
            .0
            .shift_remove(&pm);
    });
    timer::start(
        w,
        TimeSpan::from_mins(5),
        TimeSpan::from_mins(5),
        0,
        atrophy_tick,
    );
}
