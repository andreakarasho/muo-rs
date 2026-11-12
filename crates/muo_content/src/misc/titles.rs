//! `Titles`: fame and karma titles, skill titles and the name shown on the paperdoll.

use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::map::MapId;
use muo_server::mobile::part1::props_a;
use muo_server::mobile::part2::messages;
use muo_server::skills::{self, SkillLock, SkillName, SkillTable};

use crate::player_mobile::{self, PlayerFlag};

pub const MIN_FAME: i32 = 0;
pub const MAX_FAME: i32 = 15000;
pub const MIN_KARMA: i32 = -15000;
pub const MAX_KARMA: i32 = 15000;

/// `m_Levels`: Neophyte .. Legendary, in the three columns (normal, Bushido, Ninjitsu).
const LEVELS: [[&str; 3]; 10] = [
    ["Neophyte", "Neophyte", "Neophyte"],
    ["Novice", "Novice", "Novice"],
    ["Apprentice", "Apprentice", "Apprentice"],
    ["Journeyman", "Journeyman", "Journeyman"],
    ["Expert", "Expert", "Expert"],
    ["Adept", "Adept", "Adept"],
    ["Master", "Master", "Master"],
    ["Grandmaster", "Grandmaster", "Grandmaster"],
    ["Elder", "Tatsujin", "Shinobi"],
    ["Legendary", "Kengo", "Ka-ge"],
];

/// `FameEntry`: a fame ceiling and its `KarmaEntry` (karma ceiling, title prefix) rows.
struct FameEntry {
    fame: i32,
    karma: [(i32, &'static str); 11],
}

const FAME_ENTRIES: [FameEntry; 5] = [
    FameEntry {
        fame: 1249,
        karma: [
            (-10000, "The Outcast "),
            (-5000, "The Despicable "),
            (-2500, "The Scoundrel "),
            (-1250, "The Unsavory "),
            (-625, "The Rude "),
            (624, ""),
            (1249, "The Fair "),
            (2499, "The Kind "),
            (4999, "The Good "),
            (9999, "The Honest "),
            (10000, "The Trustworthy "),
        ],
    },
    FameEntry {
        fame: 2499,
        karma: [
            (-10000, "The Wretched "),
            (-5000, "The Dastardly "),
            (-2500, "The Malicious "),
            (-1250, "The Dishonorable "),
            (-625, "The Disreputable "),
            (624, "The Notable "),
            (1249, "The Upstanding "),
            (2499, "The Respectable "),
            (4999, "The Honorable "),
            (9999, "The Commendable "),
            (10000, "The Estimable "),
        ],
    },
    FameEntry {
        fame: 4999,
        karma: [
            (-10000, "The Nefarious "),
            (-5000, "The Wicked "),
            (-2500, "The Vile "),
            (-1250, "The Ignoble "),
            (-625, "The Notorious "),
            (624, "The Prominent "),
            (1249, "The Reputable "),
            (2499, "The Proper "),
            (4999, "The Admirable "),
            (9999, "The Famed "),
            (10000, "The Great "),
        ],
    },
    FameEntry {
        fame: 9999,
        karma: [
            (-10000, "The Dread "),
            (-5000, "The Evil "),
            (-2500, "The Villainous "),
            (-1250, "The Sinister "),
            (-625, "The Infamous "),
            (624, "The Renowned "),
            (1249, "The Distinguished "),
            (2499, "The Eminent "),
            (4999, "The Noble "),
            (9999, "The Illustrious "),
            (10000, "The Glorious "),
        ],
    },
    FameEntry {
        fame: 10000,
        karma: [
            (-10000, "The Dread "),
            (-5000, "The Evil "),
            (-2500, "The Dark "),
            (-1250, "The Sinister "),
            (-625, "The Dishonored "),
            (624, ""),
            (1249, "The Distinguished "),
            (2499, "The Eminent "),
            (4999, "The Noble "),
            (9999, "The Illustrious "),
            (10000, "The Glorious "),
        ],
    },
];

/// `Titles.ComputeKillAwards(killed, map)`: `(fame, karma)`.
pub fn compute_kill_awards(w: &World, killed: MobileId, map: Option<MapId>) -> (i32, i32) {
    let (fame, karma) = w.mobile(killed).map_or((0, 0), |m| (m.fame, m.karma));
    let mut total_fame = fame / 100;
    let mut total_karma = -karma / 100;
    if crate::core::lbr(w) && map == Some(MapId::FELUCCA) {
        total_fame += total_fame / 10 * 3;
        total_karma += total_karma / 10 * 3;
    }
    (total_fame, total_karma)
}

/// `Titles.AwardFame(m, offset, message)`.
pub fn award_fame(w: &mut World, m: MobileId, mut offset: i32, message: bool) {
    let Some(fame) = w.mobile(m).map(|x| x.fame) else {
        return;
    };
    if offset > 0 {
        if fame >= MAX_FAME {
            return;
        }
        offset = (offset - fame / 100).max(0);
    } else if offset < 0 {
        if fame <= MIN_FAME {
            return;
        }
        offset = (offset - fame / 100).min(0);
    }
    offset = match fame + offset {
        n if n > MAX_FAME => MAX_FAME - fame,
        n if n < MIN_FAME => MIN_FAME - fame,
        _ => offset,
    };
    props_a::set_fame(w, m, fame + offset);
    if message {
        let number = match offset {
            41.. => 1019054,      // You have gained a lot of fame.
            21..=40 => 1019053,   // You have gained a good amount of fame.
            11..=20 => 1019052,   // You have gained some fame.
            1..=10 => 1019051,    // You have gained a little fame.
            ..=-41 => 1019058,    // You have lost a lot of fame.
            -40..=-21 => 1019057, // You have lost a good amount of fame.
            -20..=-11 => 1019056, // You have lost some fame.
            -10..=-1 => 1019055,  // You have lost a little fame.
            0 => return,
        };
        messages::send_localized_message(w, m, number, "", 0x3B2);
    }
}

fn karma_locked(w: &World, m: MobileId) -> Option<bool> {
    player_mobile::data(w, m).map(|d| d.flag(PlayerFlag::KARMA_LOCKED))
}

/// `Titles.SetKarma(m, newKarma, message)`.
pub fn set_karma(w: &mut World, m: MobileId, new_karma: i32, message: bool) {
    let Some(old) = w.mobile(m).map(|x| x.karma) else {
        return;
    };
    let new_karma = new_karma.clamp(MIN_KARMA, MAX_KARMA);
    let offset = new_karma - old;
    let was_positive = old >= 0;
    props_a::set_karma(w, m, new_karma);
    if message {
        send_karma_message(w, m, offset);
    }
    check_karma_lock(w, m, was_positive);
}

/// `Titles.AwardKarma(m, offset, message)`.
pub fn award_karma(w: &mut World, m: MobileId, mut offset: i32, message: bool) {
    let Some(karma) = w.mobile(m).map(|x| x.karma) else {
        return;
    };
    if offset > 0 {
        if crate::core::expansion(w) >= muo_data::expansion::Expansion::UOTD
            && !crate::core::aos(w)
            && karma_locked(w, m) == Some(true)
        {
            return;
        }
        if karma >= MAX_KARMA {
            return;
        }
        offset = (offset - karma / 100).max(0);
    } else if offset < 0 {
        if karma <= MIN_KARMA {
            return;
        }
        offset = (offset - karma / 100).min(0);
    }
    offset = match karma + offset {
        n if n > MAX_KARMA => MAX_KARMA - karma,
        n if n < MIN_KARMA => MIN_KARMA - karma,
        _ => offset,
    };
    let was_positive = karma >= 0;
    props_a::set_karma(w, m, karma + offset);
    if message {
        send_karma_message(w, m, offset);
    }
    check_karma_lock(w, m, was_positive);
}

fn send_karma_message(w: &mut World, m: MobileId, offset: i32) {
    if offset == 0 {
        return;
    }
    let number = match offset {
        41.. => 1019062,      // You have gained a lot of karma.
        21..=40 => 1019061,   // You have gained a good amount of karma.
        11..=20 => 1019060,   // You have gained some karma.
        1..=10 => 1019059,    // You have gained a little karma.
        ..=-41 => 1019066,    // You have lost a lot of karma.
        -40..=-21 => 1019065, // You have lost a good amount of karma.
        -20..=-11 => 1019064, // You have lost some karma.
        _ => 1019063,         // You have lost a little karma.
    };
    messages::send_localized_message(w, m, number, "", 0x3B2);
}

fn check_karma_lock(w: &mut World, m: MobileId, was_positive_karma: bool) {
    if crate::core::expansion(w) >= muo_data::expansion::Expansion::UOTD
        && !crate::core::aos(w)
        && was_positive_karma
        && w.mobile(m).is_some_and(|x| x.karma < 0)
        && karma_locked(w, m) == Some(false)
    {
        if let Some(d) = player_mobile::data_mut(w, m) {
            d.set_flag(PlayerFlag::KARMA_LOCKED, true);
        }
        // Karma is locked.  A mantra spoken at a shrine will unlock it again.
        messages::send_localized_message(w, m, 1042511, "", 0x22);
    }
}

/// `Titles.ComputeTitle(beholder, beheld)`.
pub fn compute_title(w: &mut World, beholder: MobileId, beheld: MobileId) -> String {
    let Some(b) = w.mobile(beheld) else {
        return String::new();
    };
    let (fame, karma, female, player) = (b.fame, b.karma, b.female, b.player);
    let ty = b.ty;
    let name = (ty.name)(w, beheld).unwrap_or_default();
    let custom = b.title.clone();
    let show_fame_title = (ty.show_fame_title)(w, beheld);
    let show_skill_title = show_fame_title && (beholder == beheld || fame >= 5000);
    let mut title = String::new();

    if show_fame_title || beholder == beheld {
        for (i, fe) in FAME_ENTRIES.iter().enumerate() {
            if fame <= fe.fame || i == FAME_ENTRIES.len() - 1 {
                for (j, (karma_ceiling, prefix)) in fe.karma.iter().enumerate() {
                    if karma <= *karma_ceiling || j == fe.karma.len() - 1 {
                        if fame >= 10000 {
                            let lord = if female { "Lady" } else { "Lord" };
                            title.push_str(&format!("{prefix}{lord} {name}"));
                        } else {
                            title.push_str(&format!("{prefix}{name}"));
                        }
                        break;
                    }
                }
                break;
            }
        }
    } else {
        title.push_str(&name);
    }

    if player_mobile::data(w, beheld).is_some_and(|d| d.flag(PlayerFlag::DISPLAY_CHAMPION_TITLE)) {
        let label = player_mobile::tables::champion_title_label(w, beheld);
        if label > 0 {
            // Should this be translated to the receivers language? Prefix titles aren't?
            let text = crate::items::equip_hooks::hooks(w)
                .and_then(|h| h.localized_text)
                .and_then(|f| f(w, label))
                .unwrap_or_default();
            title.push_str(&format!(": {text}"));
            // Do not display the skills title
            return title;
        }
    }

    let custom = custom.as_deref().map(str::trim).filter(|t| !t.is_empty());
    if let Some(c) = custom {
        title.push(' ');
        title.push_str(c);
    } else if show_skill_title
        && player
        && let Some(skill_title) = get_skill_title(w, beheld)
    {
        title.push_str(&format!(", {skill_title}"));
    }
    title
}

/// `Titles.GetSkillTitle(mob)`.
pub fn get_skill_title(w: &mut World, mob: MobileId) -> Option<String> {
    let highest = get_highest_skill(w, mob)?;
    let fixed = skills::base_fixed(w, mob, highest);
    if fixed < 300 {
        return None;
    }
    let female = w.mobile(mob).is_some_and(|m| m.female);
    let level = LEVELS[((fixed.min(1200) - 300) / 100) as usize][match highest {
        SkillName::Bushido => 1,
        SkillName::Ninjitsu => 2,
        _ => 0,
    }];
    let skill_title = w
        .resources
        .get::<SkillTable>()?
        .info
        .get(highest as usize)?
        .title
        .clone();
    Some(match skill_title.strip_suffix("man") {
        Some(stem) if female => format!("{level} {stem}woman"),
        _ => format!("{level} {skill_title}"),
    })
}

fn get_highest_skill(w: &mut World, m: MobileId) -> Option<SkillName> {
    if !crate::core::aos(w) {
        return skills::highest(w, m);
    }
    let len = w.resources.get::<SkillTable>().map_or(0, |t| t.info.len());
    let mut highest: Option<SkillName> = None;
    for i in 0..len {
        let check = SkillName::from_i32(i as i32)?;
        let Some(h) = highest else {
            highest = Some(check);
            continue;
        };
        let (cb, hb) = (skills::base_fixed(w, m, check), skills::base_fixed(w, m, h));
        if cb > hb
            || (skills::lock(w, m, h) != SkillLock::Up
                && skills::lock(w, m, check) == SkillLock::Up
                && cb == hb)
        {
            highest = Some(check);
        }
    }
    highest
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fame_award_scales_down_and_clamps() {
        let mut w = World::new();
        let m = w.new_mobile(&muo_server::mobile::MOBILE_BASE);
        award_fame(&mut w, m, 100, false);
        assert_eq!(w.mobile(m).unwrap().fame, 100);
        w.mobile_mut(m).unwrap().fame = 14990;
        award_fame(&mut w, m, 500, false);
        assert_eq!(w.mobile(m).unwrap().fame, MAX_FAME);
        award_fame(&mut w, m, 5, false);
        assert_eq!(w.mobile(m).unwrap().fame, MAX_FAME, "already at the cap");
    }

    #[test]
    fn karma_set_is_clamped() {
        let mut w = World::new();
        let m = w.new_mobile(&muo_server::mobile::MOBILE_BASE);
        set_karma(&mut w, m, 99999, false);
        assert_eq!(w.mobile(m).unwrap().karma, MAX_KARMA);
    }

    #[test]
    fn title_prefix_by_fame_and_karma() {
        let mut w = World::new();
        let m = w.new_mobile(&muo_server::mobile::MOBILE_BASE);
        {
            let x = w.mobile_mut(m).unwrap();
            x.name = Some("Bob".into());
            x.fame = 10000;
            x.karma = 10000;
        }
        // base show_fame_title is true; fame >= 10000 gives Lord
        assert_eq!(compute_title(&mut w, m, m), "The Glorious Lord Bob");
        w.mobile_mut(m).unwrap().fame = 0;
        w.mobile_mut(m).unwrap().karma = 0;
        assert_eq!(compute_title(&mut w, m, m), "Bob");
    }
}
