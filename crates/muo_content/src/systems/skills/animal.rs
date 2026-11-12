//! Animal lore (with `AnimalLoreGump`) and animal taming.
//!
//! `SummonFamiliarSpell.Table` (the Necromancy familiars) and `Faction.Find` / `FactionWarHorse` are the other
//! crates'; [`AnimalTamingHooks`] stands for them (set by `muo_spells` and `muo_engines`).

use std::cell::Cell;
use std::rc::Rc;

use indexmap::IndexSet;
use muo_primitives::{DateTime, MessageType, TimeSpan, utility};
use muo_server::World;
use muo_server::ids::{EntityId, MobileId};
use muo_server::mobile::base as mb;
use muo_server::mobile::part1::props_b;
use muo_server::mobile::part2::{damage as dmg, messages as mm, stats};
use muo_server::skills::{self, SkillName};
use muo_server::target::{self, Targeted};
use muo_server::timer;

use super::common::{
    TargetOptions, check_target_skill, local_msg, mobile_is, msg, private_msg, set_target,
};
use super::skills_info::set_callback;
use crate::core;
use crate::creature::enums::{FoodType, MAX_OWNERS, OrderType, PackInstinct};
use crate::creature::{self, cd, cd_mut, control, ctype};
use crate::gumps::base::{DynamicGumpLogic, DynamicGumpObj};
use crate::gumps::builder::DynamicGumpBuilder;
use crate::gumps::layout::GumpButtonType;
use crate::gumps::send_gump_to_mobile;
use crate::hooks::hooks;
use crate::mx;
use crate::player_mobile::RACE_ELF;

fn entity_mobile(t: Targeted) -> Option<MobileId> {
    match t {
        Targeted::Entity(EntityId::Mobile(m)) => Some(m),
        _ => None,
    }
}

type FactionWarHorseAllowsFn = fn(&World, MobileId, MobileId) -> Option<bool>;
/// Systems the taming reads that other crates own.
#[derive(Default)]
pub struct AnimalTamingHooks {
    /// `SummonFamiliarSpell.Table.TryGetValue(tamer, out bc)` then `bc is DarkWolfFamiliar { Deleted: false }`.
    /// Unset: false.
    pub has_dark_wolf_familiar: Option<fn(&World, MobileId) -> bool>,
    /// `creature is FactionWarHorse warHorse`: `Some(allowed)` where allowed is
    /// `Faction.Find(from) == warHorse.Faction != null`; `None` when it is not a war horse. Unset: `None`.
    pub faction_war_horse_allows: Option<FactionWarHorseAllowsFn>,
}

/// The taming hooks the faction and necromancy crates set.
pub fn hooks_mut_taming(w: &mut World) -> &mut AnimalTamingHooks {
    w.resources.get_mut::<AnimalTamingHooks>()
}

/// `AnimalTaming.m_BeingTamed` and `DisableMessage`.
#[derive(Default)]
pub struct AnimalTamingState {
    being_tamed: IndexSet<MobileId>,
    pub disable_message: bool,
}

fn state(w: &mut World) -> &mut AnimalTamingState {
    w.resources.get_mut::<AnimalTamingState>()
}

// ---- AnimalLore -------------------------------------------------------------------------------------------------

pub fn lore_on_use(w: &mut World, m: MobileId) -> TimeSpan {
    set_target(w, m, 8, TargetOptions::default(), lore_on_target);
    msg(w, m, 500328); // What animal should I look at?
    TimeSpan::from_secs(1)
}

fn lore_on_target(w: &mut World, from: MobileId, targeted: Targeted) {
    let c = entity_mobile(targeted).filter(|&c| creature::is_creature(w, c));
    if !mx::alive(w, from) {
        msg(w, from, 500331); // The spirits of the dead are not the province of animal lore.
        return;
    }
    let Some(c) = c else {
        msg(w, from, 500329); // That's not an animal!
        return;
    };
    let d = cd(w, c).expect("creature");
    let (dead_pet, controlled, tamable) = (d.is_dead_pet, d.controlled, d.tamable());
    let lore = skills::value(w, from, SkillName::AnimalLore);
    if dead_pet {
        msg(w, from, 500331); // The spirits of the dead are not the province of animal lore.
    } else if !creature::misc::body_is_animal(w, c)
        && !creature::misc::body_is_monster(w, c)
        && !creature::misc::body_is_sea(w, c)
    {
        msg(w, from, 500329); // That's not an animal!
    } else if !controlled && lore < 100.0 {
        // At your skill level, you can only lore tamed creatures.
        msg(w, from, 1049674);
    } else if !controlled && !tamable && lore < 110.0 {
        // At your skill level, you can only lore tamed or tameable creatures.
        msg(w, from, 1049675);
    } else if !check_target_skill(w, from, SkillName::AnimalLore, targeted, 0.0, 120.0) {
        msg(w, from, 500334); // You can't think of anything you know offhand.
    } else {
        display_lore_gump(w, from, c);
    }
}

/// What `AnimalLoreGump` reads of its creature, taken when it is made.
pub struct AnimalLoreGump {
    name: String,
    hits: (i32, i32),
    stam: (i32, i32),
    mana: (i32, i32),
    stats: (i32, i32, i32),
    barding_difficulty: f64,
    loyalty_number: i32,
    virtual_armor: i32,
    resist: [i32; 5],
    damage: [i32; 5],
    damage_range: (i32, i32),
    skill: [String; 8],
    cu_sidhe: bool,
    food_pref: i32,
    pack_instinct: i32,
    aos: bool,
    se: bool,
    ml: bool,
}

const LABEL_COLOR: i32 = 0x24E5;
const DASHES: &str = "<div align=right>---</div>";

fn format_skill(w: &mut World, c: MobileId, name: SkillName) -> String {
    if skills::base(w, c, name) < 10.0 {
        return DASHES.to_string();
    }
    format!("<div align=right>{:.1}</div>", skills::value(w, c, name))
}

fn format_attributes(cur: i32, max: i32) -> String {
    if max == 0 {
        DASHES.to_string()
    } else {
        format!("<div align=right>{cur}/{max}</div>")
    }
}

fn format_stat(v: i32) -> String {
    if v == 0 {
        DASHES.to_string()
    } else {
        format!("<div align=right>{v}</div>")
    }
}

fn format_double(v: f64) -> String {
    if v == 0.0 {
        DASHES.to_string()
    } else {
        format!("<div align=right>{v:.1}</div>")
    }
}

fn format_element(v: i32, color: Option<&str>) -> String {
    match color {
        None if v == 0 => DASHES.to_string(),
        None => format!("<div align=right>{v}%</div>"),
        Some(c) if v == 0 => format!("<BASEFONT COLOR={c}><div align=right>---</div>"),
        Some(c) => format!("<BASEFONT COLOR={c}><div align=right>{v}%</div>"),
    }
}

fn format_damage(min: i32, max: i32) -> String {
    if min <= 0 || max <= 0 {
        DASHES.to_string()
    } else {
        format!("<div align=right>{min}-{max}</div>")
    }
}

/// `AnimalLoreGump.DisplayTo(from, c)`.
pub fn display_lore_gump(w: &mut World, from: MobileId, c: MobileId) {
    if w.mobile(from).is_none_or(|m| m.net_state.is_none()) || mx::deleted(w, c) {
        return;
    }
    let cx = ctype(w, c).expect("creature");
    let uncalmable = (cx.uncalmable)(w, c);
    let mut bd = hooks(w)
        .and_then(|h| h.instrument_base_difficulty)
        .map_or(0.0, |f| f(w, c));
    if uncalmable {
        bd = 0.0;
    }
    let favorite = (cx.favorite_food)(w, c);
    let food_pref = if favorite.contains(FoodType::MEAT) {
        1049564
    } else if favorite.contains(FoodType::FRUITS_AND_VEGGIES) {
        1049565
    } else if favorite.contains(FoodType::GRAINS_AND_HAY) {
        1049566
    } else if favorite.contains(FoodType::FISH) {
        1049568
    } else if favorite.contains(FoodType::METAL) {
        1049567
    } else if favorite.contains(FoodType::EGGS) {
        1044477
    } else {
        3000340
    };
    let pack = (cx.pack_instinct)(w, c);
    let pack_instinct = if pack.contains(PackInstinct::CANINE) {
        1049570
    } else if pack.contains(PackInstinct::OSTARD) {
        1049571
    } else if pack.contains(PackInstinct::FELINE) {
        1049572
    } else if pack.contains(PackInstinct::ARACHNID) {
        1049573
    } else if pack.contains(PackInstinct::DAEMON) {
        1049574
    } else if pack.contains(PackInstinct::BEAR) {
        1049575
    } else if pack.contains(PackInstinct::EQUINE) {
        1049576
    } else if pack.contains(PackInstinct::BULL) {
        1049577
    } else {
        3000340
    };
    let d = cd(w, c).expect("creature");
    let loyalty_number = if !d.controlled || d.loyalty == 0 {
        1061643
    } else {
        1049595 + d.loyalty / 10
    };
    let damage = [
        d.physical_damage,
        d.fire_damage,
        d.cold_damage,
        d.poison_damage,
        d.energy_damage,
    ];
    let damage_range = (d.damage_min, d.damage_max);
    let (hits, stam, mana) = {
        let m = w.mobile(c).expect("creature");
        (
            (m.hits, mb::hits_max(w, c)),
            (m.stam, mb::stam_max(w, c)),
            (m.mana, mb::mana_max(w, c)),
        )
    };
    let cu_sidhe = mobile_is(w, c, "Server.Mobiles.CuSidhe");
    let third = if cu_sidhe {
        SkillName::Healing
    } else {
        SkillName::Poisoning
    };
    let skill = [
        format_skill(w, c, SkillName::Wrestling),
        format_skill(w, c, SkillName::Tactics),
        format_skill(w, c, SkillName::MagicResist),
        format_skill(w, c, SkillName::Anatomy),
        format_skill(w, c, third),
        format_skill(w, c, SkillName::Magery),
        format_skill(w, c, SkillName::EvalInt),
        format_skill(w, c, SkillName::Meditation),
    ];
    let gump = AnimalLoreGump {
        name: props_b::name(w, c).unwrap_or_default(),
        hits,
        stam,
        mana,
        stats: (mb::str(w, c), mb::dex(w, c), mb::int(w, c)),
        barding_difficulty: bd,
        loyalty_number,
        virtual_armor: w.mobile(c).map_or(0, |m| m.virtual_armor),
        resist: [
            mb::physical_resistance(w, c),
            mb::fire_resistance(w, c),
            mb::cold_resistance(w, c),
            mb::poison_resistance(w, c),
            mb::energy_resistance(w, c),
        ],
        damage,
        damage_range,
        skill,
        cu_sidhe,
        food_pref,
        pack_instinct,
        aos: core::aos(w),
        se: core::se(w),
        ml: core::ml(w),
    };
    send_gump_to_mobile(w, from, Box::new(DynamicGumpObj::new(250, 50, gump)), false);
}

impl DynamicGumpLogic for AnimalLoreGump {
    const TYPE_NAME: &'static str = "Server.SkillHandlers.AnimalLoreGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        let l = |b: &mut DynamicGumpBuilder, x, y, n| {
            b.add_html_localized_color(x, y, 160, 18, n, LABEL_COLOR, false, false)
        };
        b.add_page(0);

        b.add_image(100, 100, 2080, 0);
        b.add_image(118, 137, 2081, 0);
        b.add_image(118, 207, 2081, 0);
        b.add_image(118, 277, 2081, 0);
        b.add_image(118, 347, 2083, 0);

        b.add_html(
            147,
            108,
            210,
            18,
            &format!("<center><i>{}</i></center>", self.name),
            false,
            false,
        );

        b.add_button(240, 77, 2093, 2093, 2);

        b.add_image(140, 138, 2091, 0);
        b.add_image(140, 335, 2091, 0);

        let pages = if self.aos { 5 } else { 3 };
        let mut page = 0;

        page += 1;
        b.add_page(page);

        b.add_image(128, 152, 2086, 0);
        b.add_html_localized_color(147, 150, 160, 18, 1049593, 200, false, false); // Attributes

        l(b, 153, 168, 1049578); // Hits
        b.add_html(
            280,
            168,
            75,
            18,
            &format_attributes(self.hits.0, self.hits.1),
            false,
            false,
        );

        l(b, 153, 186, 1049579); // Stamina
        b.add_html(
            280,
            186,
            75,
            18,
            &format_attributes(self.stam.0, self.stam.1),
            false,
            false,
        );

        l(b, 153, 204, 1049580); // Mana
        b.add_html(
            280,
            204,
            75,
            18,
            &format_attributes(self.mana.0, self.mana.1),
            false,
            false,
        );

        l(b, 153, 222, 1028335); // Strength
        b.add_html(320, 222, 35, 18, &format_stat(self.stats.0), false, false);

        l(b, 153, 240, 3000113); // Dexterity
        b.add_html(320, 240, 35, 18, &format_stat(self.stats.1), false, false);

        l(b, 153, 258, 3000112); // Intelligence
        b.add_html(320, 258, 35, 18, &format_stat(self.stats.2), false, false);

        if self.aos {
            let mut y = 276;

            if self.se {
                l(b, 153, 276, 1070793); // Barding Difficulty
                b.add_html(
                    320,
                    y,
                    35,
                    18,
                    &format_double(self.barding_difficulty),
                    false,
                    false,
                );

                y += 18;
            }

            b.add_image(128, y + 2, 2086, 0);
            b.add_html_localized_color(147, y, 160, 18, 1049594, 200, false, false); // Loyalty Rating
            y += 18;

            l(b, 153, y, self.loyalty_number);
        } else {
            b.add_image(128, 278, 2086, 0);
            b.add_html_localized_color(147, 276, 160, 18, 3001016, 200, false, false); // Miscellaneous

            l(b, 153, 294, 1049581); // Armor Rating
            b.add_html(
                320,
                294,
                35,
                18,
                &format_stat(self.virtual_armor),
                false,
                false,
            );
        }

        b.add_button_type(340, 358, 5601, 5605, 0, GumpButtonType::Page, page + 1);
        b.add_button_type(317, 358, 5603, 5607, 0, GumpButtonType::Page, pages);

        if self.aos {
            page += 1;
            b.add_page(page);

            b.add_image(128, 152, 2086, 0);
            b.add_html_localized_color(147, 150, 160, 18, 1061645, 200, false, false); // Resistances

            let rows: [(i32, i32, Option<&str>); 5] = [
                (168, 1061646, None),
                (186, 1061647, Some("#FF0000")),
                (204, 1061648, Some("#000080")),
                (222, 1061649, Some("#008000")),
                (240, 1061650, Some("#BF80FF")),
            ];
            for (i, (y, n, color)) in rows.into_iter().enumerate() {
                l(b, 153, y, n);
                b.add_html(
                    320,
                    y,
                    35,
                    18,
                    &format_element(self.resist[i], color),
                    false,
                    false,
                );
            }

            b.add_button_type(340, 358, 5601, 5605, 0, GumpButtonType::Page, page + 1);
            b.add_button_type(317, 358, 5603, 5607, 0, GumpButtonType::Page, page - 1);

            page += 1;
            b.add_page(page);

            b.add_image(128, 152, 2086, 0);
            b.add_html_localized_color(147, 150, 160, 18, 1017319, 200, false, false); // Damage

            for (i, (y, n, color)) in rows.into_iter().enumerate() {
                l(b, 153, y, n);
                b.add_html(
                    320,
                    y,
                    35,
                    18,
                    &format_element(self.damage[i], color),
                    false,
                    false,
                );
            }

            if self.ml {
                l(b, 153, 258, 1076750); // Base Damage
                b.add_html(
                    300,
                    258,
                    55,
                    18,
                    &format_damage(self.damage_range.0, self.damage_range.1),
                    false,
                    false,
                );
            }

            b.add_button_type(340, 358, 5601, 5605, 0, GumpButtonType::Page, page + 1);
            b.add_button_type(317, 358, 5603, 5607, 0, GumpButtonType::Page, page - 1);
        }

        page += 1;
        b.add_page(page);

        b.add_image(128, 152, 2086, 0);
        b.add_html_localized_color(147, 150, 160, 18, 3001030, 200, false, false); // Combat Ratings

        l(b, 153, 168, 1044103); // Wrestling
        b.add_html(320, 168, 35, 18, &self.skill[0], false, false);
        l(b, 153, 186, 1044087); // Tactics
        b.add_html(320, 186, 35, 18, &self.skill[1], false, false);
        l(b, 153, 204, 1044086); // Magic Resistance
        b.add_html(320, 204, 35, 18, &self.skill[2], false, false);
        l(b, 153, 222, 1044061); // Anatomy
        b.add_html(320, 222, 35, 18, &self.skill[3], false, false);

        if self.cu_sidhe {
            l(b, 153, 240, 1044077); // Healing
        } else {
            l(b, 153, 240, 1044090); // Poisoning
        }
        b.add_html(320, 240, 35, 18, &self.skill[4], false, false);

        b.add_image(128, 260, 2086, 0);
        b.add_html_localized_color(147, 258, 160, 18, 3001032, 200, false, false); // Lore & Knowledge

        l(b, 153, 276, 1044085); // Magery
        b.add_html(320, 276, 35, 18, &self.skill[5], false, false);
        l(b, 153, 294, 1044076); // Evaluating Intelligence
        b.add_html(320, 294, 35, 18, &self.skill[6], false, false);
        l(b, 153, 312, 1044106); // Meditation
        b.add_html(320, 312, 35, 18, &self.skill[7], false, false);

        b.add_button_type(340, 358, 5601, 5605, 0, GumpButtonType::Page, page + 1);
        b.add_button_type(317, 358, 5603, 5607, 0, GumpButtonType::Page, page - 1);

        page += 1;
        b.add_page(page);

        b.add_image(128, 152, 2086, 0);
        b.add_html_localized_color(147, 150, 160, 18, 1049563, 200, false, false); // Preferred Foods

        l(b, 153, 168, self.food_pref);

        b.add_image(128, 188, 2086, 0);
        b.add_html_localized_color(147, 186, 160, 18, 1049569, 200, false, false); // Pack Instincts

        l(b, 153, 204, self.pack_instinct);

        if !self.aos {
            b.add_image(128, 224, 2086, 0);
            b.add_html_localized_color(147, 222, 160, 18, 1049594, 200, false, false); // Loyalty Rating

            l(b, 153, 240, self.loyalty_number);
        }

        b.add_button_type(340, 358, 5601, 5605, 0, GumpButtonType::Page, 1);
        b.add_button_type(317, 358, 5603, 5607, 0, GumpButtonType::Page, page - 1);
    }
}

// ---- AnimalTaming -----------------------------------------------------------------------------------------------

/// `DisableMessage`.
pub fn disable_message(w: &World) -> bool {
    w.resources
        .get::<AnimalTamingState>()
        .is_some_and(|s| s.disable_message)
}

/// `AnimalTaming.DisableMessage = value`.
pub fn set_disable_message(w: &mut World, value: bool) {
    state(w).disable_message = value;
}

pub fn taming_on_use(w: &mut World, m: MobileId) -> TimeSpan {
    mb::revealing_action(w, m);

    let range = if core::aos(w) { 3 } else { 2 };
    set_taming_target(w, m, range);
    mb::revealing_action(w, m);

    if !disable_message(w) {
        msg(w, m, 502789); // Tame which animal?
    }

    TimeSpan::from_secs(30)
}

fn set_taming_target(w: &mut World, m: MobileId, range: i32) {
    let set_skill_time = Rc::new(Cell::new(true));
    let finish = set_skill_time.clone();
    set_target(
        w,
        m,
        range,
        TargetOptions {
            on_finish: Some(Rc::new(move |w: &mut World, from: MobileId| {
                if finish.get() {
                    let t = w.tick_count;
                    if let Some(x) = w.mobile_mut(from) {
                        x.next_skill_time = t;
                    }
                }
            })),
            ..Default::default()
        },
        move |w, from, targeted| taming_on_target(w, from, targeted, &set_skill_time),
    );
}

/// `AnimalTaming.CheckMastery(tamer, creature)`.
pub fn check_mastery(w: &World, tamer: MobileId, creature: MobileId) -> bool {
    w.resources
        .get::<AnimalTamingHooks>()
        .and_then(|h| h.has_dark_wolf_familiar)
        .is_some_and(|f| f(w, tamer))
        && ["DireWolf", "GreyWolf", "TimberWolf", "WhiteWolf"]
            .iter()
            .any(|n| mobile_is(w, creature, &format!("Server.Mobiles.{n}")))
}

/// `AnimalTaming.MustBeSubdued(bc)`.
pub fn must_be_subdued(w: &World, bc: MobileId) -> bool {
    let Some(c) = ctype(w, bc) else { return false };
    cd(w, bc).is_some_and(|d| d.owners.is_empty())
        && (c.subdue_before_tame)(w, bc)
        && mx::hits(w, bc) > mx::hits_max(w, bc) / 10
}

/// `AnimalTaming.ScaleStats(bc, scalar)`.
pub fn scale_stats(w: &mut World, bc: MobileId, scalar: f64) {
    let (s, d, i) = w.mobile(bc).map_or((0, 0, 0), |m| (m.str, m.dex, m.int));
    if s > 0 {
        props_b::set_raw_str(w, bc, ((f64::from(s) * scalar) as i32).max(1));
    }
    if d > 0 {
        props_b::set_raw_dex(w, bc, ((f64::from(d) * scalar) as i32).max(1));
    }
    if i > 0 {
        props_b::set_raw_int(w, bc, ((f64::from(i) * scalar) as i32).max(1));
    }
    let seeds = cd(w, bc).map_or((0, 0), |d| (d.hits_max_seed, d.stam_max_seed));
    if seeds.0 > 0 {
        let v = ((f64::from(seeds.0) * scalar) as i32).max(1);
        cd_mut(w, bc).expect("creature").hits_max_seed = v;
        let h = mx::hits(w, bc);
        props_b::set_hits(w, bc, h);
    }
    if seeds.1 > 0 {
        let v = ((f64::from(seeds.1) * scalar) as i32).max(1);
        cd_mut(w, bc).expect("creature").stam_max_seed = v;
        let s = w.mobile(bc).map_or(0, |m| m.stam);
        props_b::set_stam(w, bc, s);
    }
}

/// `AnimalTaming.ScaleSkills(bc, scalar, capScalar)`.
pub fn scale_skills(w: &mut World, bc: MobileId, scalar: f64, cap_scalar: f64) {
    let len = w
        .resources
        .get::<muo_server::skills::SkillTable>()
        .map_or(0, |t| t.info.len());
    for i in 0..len {
        let Some(name) = SkillName::from_i32(i as i32) else {
            continue;
        };
        let b = skills::base(w, bc, name) * scalar;
        skills::set_base(w, bc, name, b);

        let cap = (skills::cap(w, bc, name) * cap_scalar).max(100.0);
        skills::set_cap(w, bc, name, cap);

        let b = skills::base(w, bc, name);
        if b > skills::cap(w, bc, name) {
            skills::set_cap(w, bc, name, b);
        }
    }
}

fn taming_on_target(
    w: &mut World,
    from: MobileId,
    targeted: Targeted,
    set_skill_time: &Cell<bool>,
) {
    mb::revealing_action(w, from);

    let Some(mobile) = entity_mobile(targeted) else {
        msg(w, from, 502801); // You can't tame that!
        return;
    };

    if !creature::is_creature(w, mobile) {
        // That being cannot be tamed.
        private_msg(w, mobile, from, 502469);
        return;
    }
    let creature = mobile;
    let c = ctype(w, creature).expect("creature");
    let d = cd(w, creature).expect("creature");

    if !d.tamable() {
        // That creature cannot be tamed.
        private_msg(w, creature, from, 1049655);
        return;
    }

    if d.controlled {
        // That animal looks tame already.
        private_msg(w, creature, from, 502804);
        return;
    }

    let female = w.mobile(from).is_some_and(|m| m.female);
    if female && !(c.allow_female_tamer)(w, creature) {
        // That creature can only be tamed by males.
        private_msg(w, creature, from, 1049653);
        return;
    }

    if !female && !(c.allow_male_tamer)(w, creature) {
        // That creature can only be tamed by females.
        private_msg(w, creature, from, 1049652);
        return;
    }

    if mobile_is(w, creature, "Server.Mobiles.CuSidhe")
        && w.mobile(from).and_then(|m| m.race) != Some(RACE_ELF)
    {
        // You can't tame that!
        private_msg(w, creature, from, 502801);
        return;
    }

    let (followers, followers_max) = w
        .mobile(from)
        .map_or((0, 0), |m| (m.followers, m.followers_max));
    if followers + d.control_slots > followers_max {
        msg(w, from, 1049611); // You have too many followers to tame that creature.
        return;
    }

    if d.owners.len() >= MAX_OWNERS && !d.owners.contains(&from) {
        // This animal has had too many owners and is too upset for you to tame.
        private_msg(w, creature, from, 1005615);
        return;
    }

    if must_be_subdued(w, creature) {
        // You must subdue this creature before you can tame it!
        private_msg(w, creature, from, 1054025);
        return;
    }

    let min_tame_skill = cd(w, creature).map_or(0.0, |d| d.min_tame_skill);
    if !(check_mastery(w, from, creature)
        || skills::value(w, from, SkillName::AnimalTaming) >= min_tame_skill)
    {
        // You have no chance of taming this creature.
        private_msg(w, creature, from, 502806);
        return;
    }

    if let Some(ok) = w
        .resources
        .get::<AnimalTamingHooks>()
        .and_then(|h| h.faction_war_horse_allows)
        .and_then(|f| f(w, from, creature))
        && !ok
    {
        // You cannot tame this creature.
        private_msg(w, creature, from, 1042590);
        return;
    }

    if state(w).being_tamed.contains(&creature) {
        // Someone else is already taming this.
        private_msg(w, creature, from, 502802);
    } else if (c.can_anger_on_tame)(w, creature) && utility::random_double(&mut w.rng) < 0.95 {
        // You seem to anger the beast!
        private_msg(w, creature, from, 502805);

        let snd = mb::get_anger_sound(w, creature);
        mx::play_sound(w, creature, snd);
        let dir = mx::get_direction_to(w, creature, from);
        mx::set_direction(w, creature, dir);

        let pacified = cd(w, creature).is_some_and(|d| d.bard_pacified);
        if pacified && utility::random_double(&mut w.rng) < 0.75 {
            timer::start(w, TimeSpan::from_secs(2), TimeSpan::ZERO, 1, move |w| {
                if let Some(d) = cd_mut(w, creature) {
                    d.bard_pacified = true;
                }
            });
        } else {
            let now = w.now;
            if let Some(d) = cd_mut(w, creature) {
                d.bard_end_time = now;
            }
        }

        if let Some(d) = cd_mut(w, creature) {
            d.bard_pacified = false;
        }

        if cd(w, creature).is_some_and(|d| d.ai.is_some()) {
            let dir = w
                .mobile(creature)
                .map_or_else(Default::default, |m| m.direction);
            crate::ai::movement::do_move(w, creature, dir, false);
        }

        if crate::player_mobile::data(w, from).is_some()
            && !hooks(w)
                .and_then(|h| h.enemy_protection)
                .is_some_and(|f| f(w, from))
        {
            mx::set_combatant(w, creature, Some(from));
        }
    } else {
        state(w).being_tamed.insert(creature);

        // You start to tame the creature.
        mm::local_overhead_message_number(w, from, MessageType::EMOTE, 0x59, 1010597, "");

        // *begins taming a creature.*
        mm::nonlocal_overhead_message_number(w, from, MessageType::EMOTE, 0x59, 1010598, "");

        let count = utility::random_from(&mut w.rng, 3, 2);
        start_taming_timer(w, from, creature, count);

        set_skill_time.set(false);
    }
}

fn release_tamer(w: &mut World, tamer: MobileId, creature: MobileId) {
    state(w).being_tamed.shift_remove(&creature);
    let t = w.tick_count;
    if let Some(x) = w.mobile_mut(tamer) {
        x.next_skill_time = t;
    }
}

/// `new InternalTimer(tamer, creature, count).Start()`.
fn start_taming_timer(w: &mut World, tamer: MobileId, creature: MobileId, max_count: i32) {
    let mut paralyzed = mb::paralyzed(w, creature);
    let start_time: DateTime = w.now;
    let mut count = 0;
    timer::start(
        w,
        TimeSpan::from_secs(3),
        TimeSpan::from_secs(3),
        max_count,
        move |w| {
            count += 1;
            let stop = |w: &mut World| {
                if let Some(h) = timer::current(w) {
                    timer::stop(w, h);
                }
            };

            let de = dmg::find_most_recent_damage_entry(w, creature, false)
                .and_then(|i| w.mobile(creature).map(|m| m.damage_entries[i].last_damage));
            let already_owned = cd(w, creature).is_some_and(|d| d.owners.contains(&tamer));
            let range = if core::aos(w) { 7 } else { 6 };

            if !mx::in_range(w, tamer, creature, range) {
                release_tamer(w, tamer, creature);
                // You are too far away to continue taming.
                private_msg(w, creature, tamer, 502795);
                stop(w);
            } else if !stats::check_alive(w, tamer, true) {
                release_tamer(w, tamer, creature);
                // You are dead, and cannot continue taming.
                private_msg(w, creature, tamer, 502796);
                stop(w);
            } else if !mb::can_see_mobile(w, tamer, creature)
                || !mx::in_los(w, tamer, creature)
                || !can_path(w, tamer, creature)
            {
                release_tamer(w, tamer, creature);
                // You do not have a clear path to the animal you are taming, and must cease your attempt.
                msg(w, tamer, 1049654);
                stop(w);
            } else if !cd(w, creature).is_some_and(|d| d.tamable()) {
                release_tamer(w, tamer, creature);
                // That creature cannot be tamed.
                private_msg(w, creature, tamer, 1049655);
                stop(w);
            } else if cd(w, creature).is_some_and(|d| d.controlled) {
                release_tamer(w, tamer, creature);
                // That animal looks tame already.
                private_msg(w, creature, tamer, 502804);
                stop(w);
            } else if cd(w, creature)
                .is_some_and(|d| d.owners.len() >= MAX_OWNERS && !d.owners.contains(&tamer))
            {
                release_tamer(w, tamer, creature);
                // This animal has had too many owners and is too upset for you to tame.
                private_msg(w, creature, tamer, 1005615);
                stop(w);
            } else if must_be_subdued(w, creature) {
                release_tamer(w, tamer, creature);
                // You must subdue this creature before you can tame it!
                private_msg(w, creature, tamer, 1054025);
                stop(w);
            } else if de.is_some_and(|last| last > start_time) {
                release_tamer(w, tamer, creature);
                // The animal is too angry to continue taming.
                private_msg(w, creature, tamer, 502794);
                stop(w);
            } else if count < max_count {
                mb::revealing_action(w, tamer);

                let (first, n) = match utility::random(&mut w.rng, 3) {
                    0 => (502790, 4),
                    1 => (1005608, 6),
                    _ => (1010593, 4),
                };
                let number = utility::random_from(&mut w.rng, first, n);
                mm::public_overhead_message_number(
                    w,
                    tamer,
                    MessageType::REGULAR,
                    0x3B2,
                    number,
                    "",
                    false,
                );

                if !already_owned {
                    // Passively check animal lore for gain
                    check_target_skill(
                        w,
                        tamer,
                        SkillName::AnimalLore,
                        Targeted::Entity(EntityId::Mobile(creature)),
                        0.0,
                        120.0,
                    );
                }

                if mb::paralyzed(w, creature) {
                    paralyzed = true;
                }
            } else {
                finish_taming(w, tamer, creature, already_owned, &mut paralyzed);
            }
        },
    );
}

/// The last tick of the taming timer.
fn finish_taming(
    w: &mut World,
    tamer: MobileId,
    creature: MobileId,
    already_owned: bool,
    paralyzed: &mut bool,
) {
    mb::revealing_action(w, tamer);
    let t = w.tick_count;
    if let Some(x) = w.mobile_mut(tamer) {
        x.next_skill_time = t;
    }
    state(w).being_tamed.shift_remove(&creature);

    if mb::paralyzed(w, creature) {
        *paralyzed = true;
    }

    let target = Targeted::Entity(EntityId::Mobile(creature));
    if !already_owned {
        // Passively check animal lore for gain
        check_target_skill(w, tamer, SkillName::AnimalLore, target, 0.0, 120.0);
    }

    let (min_tame, owners) =
        cd(w, creature).map_or((0.0, 0), |d| (d.min_tame_skill, d.owners.len()));
    let mut min_skill = min_tame + owners as f64 * 6.0;

    let mastery = check_mastery(w, tamer, creature);
    if min_skill > -24.9 && mastery {
        min_skill = -24.9; // 50% at 0.0?
    }

    min_skill += 24.9;

    if mastery
        || already_owned
        || check_target_skill(
            w,
            tamer,
            SkillName::AnimalTaming,
            target,
            min_skill - 25.0,
            min_skill + 25.0,
        )
    {
        if cd(w, creature).is_some_and(|d| d.owners.is_empty()) {
            // First tame
            if mobile_is(w, creature, "Server.Mobiles.GreaterDragon") {
                scale_skills(w, creature, 0.72, 0.90); // 72% of original skills trainable to 90%
                // Greater dragons have a 90% cap reduction and 90% skill reduction on magery
                let cap = skills::cap(w, creature, SkillName::Magery);
                skills::set_base(w, creature, SkillName::Magery, cap);
            } else if *paralyzed {
                // 86% of original skills if they were paralyzed during the taming
                scale_skills(w, creature, 0.86, 0.86);
            } else {
                scale_skills(w, creature, 0.90, 0.90); // 90% of original skills
            }

            if (ctype(w, creature).expect("creature").stat_loss_after_tame)(w, creature) {
                scale_stats(w, creature, 0.50);
            }
        }

        if already_owned {
            msg(w, tamer, 502797); // That wasn't even challenging.
        } else {
            // It seems to accept you as master.
            private_msg(w, creature, tamer, 502799);
            if let Some(d) = cd_mut(w, creature) {
                d.owners.push(tamer);
            }
        }

        control::set_control_master(w, creature, Some(tamer));
        control::set_is_bonded(w, creature, false);
        if let Some(d) = cd_mut(w, creature) {
            d.control_target = Some(tamer);
        }
        control::set_control_order_raw(w, creature, OrderType::Follow);
    } else {
        // You fail to tame the creature.
        private_msg(w, creature, tamer, 502798);
    }
}

/// `CanPath()`.
fn can_path(w: &mut World, tamer: MobileId, creature: MobileId) -> bool {
    let p = mx::location(w, tamer);
    mx::in_range_point(w, creature, p, 1)
        || !crate::pathing::astar::find_path(w, creature, p).is_empty()
}

/// `TameEntry.OnClick` after the checks the context menu entry already made.
pub fn tame_from_context_menu(w: &mut World, from: MobileId, bc: MobileId) {
    if let Some(m) = w.mobile_mut(from) {
        m.target_locked = true;
    }
    state(w).disable_message = true;

    if mb::use_skill(w, from, SkillName::AnimalTaming)
        && let Some(t) = w.mobile(from).and_then(|m| m.target)
    {
        target::invoke(w, t, from, Targeted::Entity(EntityId::Mobile(bc)));
    }

    state(w).disable_message = false;
    if let Some(m) = w.mobile_mut(from) {
        m.target_locked = false;
    }
}

pub fn initialize(w: &mut World) {
    set_callback(w, SkillName::AnimalLore, lore_on_use);
    set_callback(w, SkillName::AnimalTaming, taming_on_use);
}

/// Sets the `ContentHooks` this file answers.
pub fn configure(w: &mut World) {
    let h = crate::hooks::hooks_mut(w);
    h.animal_taming_mastery = Some(check_mastery);
    h.tame_from_context_menu = Some(tame_from_context_menu);
}

#[allow(unused)]
fn _unused(w: &mut World, m: MobileId) {
    local_msg(w, m, 0, 0);
}
