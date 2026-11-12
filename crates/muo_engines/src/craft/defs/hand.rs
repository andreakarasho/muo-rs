//! The helper methods of `DefInscription` (`AddSpell`, `AddNecroSpell`, `AddMysticismSpell`) and `DefTinkering`
//! (`AddJewelrySet`) that their `InitCraftList` calls, and the fields they share (`_circle`, `_mana`, `_index`).

pub use crate::craft::state::t2a_enabled;
use muo_content::items::jewel::GemType;
use muo_io::text::TextDefinition as Td;

use crate::craft::model::*;

/// `DefInscription`'s `_circle`, `_mana`, `_index`.
#[derive(Default)]
pub struct HelperState {
    pub circle: i32,
    pub mana: i32,
    pub index: i32,
}

/// `DefInscription.Reg`.
#[derive(Copy, Clone)]
pub enum Reg {
    BlackPearl,
    Bloodmoss,
    Garlic,
    Ginseng,
    MandrakeRoot,
    Nightshade,
    SulfurousAsh,
    SpidersSilk,
}

const REG_TYPES: [&str; 8] = [
    "Server.Items.BlackPearl",
    "Server.Items.Bloodmoss",
    "Server.Items.Garlic",
    "Server.Items.Ginseng",
    "Server.Items.MandrakeRoot",
    "Server.Items.Nightshade",
    "Server.Items.SulfurousAsh",
    "Server.Items.SpidersSilk",
];

/// `AddSpell(type, params Reg[] regs)`.
pub fn add_spell(s: &mut CraftSystem, st: &mut HelperState, ty: &'static str, regs: &[Reg]) {
    let (min_skill, max_skill) = match st.circle {
        1 => (-10.8, 39.2),
        2 => (3.5, 53.5),
        3 => (17.8, 67.8),
        4 => (32.1, 82.1),
        5 => (46.4, 96.4),
        6 => (60.7, 110.7),
        7 => (75.0, 125.0),
        _ => (-25.0, 25.0),
    };
    let r0 = regs[0] as usize;
    let index = s.add_craft(
        ty,
        Td::of_number(1044369 + st.circle),
        Td::of_number(1044381 + st.index),
        None,
        None,
        min_skill,
        max_skill,
        REG_TYPES[r0],
        Td::of_number(1044353 + r0 as i32),
        1,
        Td::of_number(1044361 + r0 as i32),
    );
    st.index += 1;
    for r in &regs[1..] {
        let r = *r as usize;
        s.add_res(
            index,
            REG_TYPES[r],
            Td::of_number(1044353 + r as i32),
            1,
            Td::of_number(1044361 + r as i32),
        );
    }
    s.add_res(
        index,
        "Server.Items.BlankScroll",
        Td::of_number(1044377),
        1,
        Td::of_number(1044378),
    );
    s.set_mana_req(index, st.mana, 1042403);
}

/// `AddNecroSpell(spell, mana, minSkill, type, params Type[] regs)`.
pub fn add_necro_spell(
    s: &mut CraftSystem,
    _st: &mut HelperState,
    spell: i32,
    mana: i32,
    min_skill: f64,
    ty: &'static str,
    regs: &[&'static str],
) {
    let index = s.add_craft(
        ty,
        Td::of_number(1061677),
        Td::of_number(1060509 + spell),
        None,
        None,
        min_skill,
        min_skill + 1.0, // Yes, on OSI it's only 1.0 skill diff'.  Don't blame me, blame OSI.
        regs[0],
        Td::EMPTY,
        1,
        Td::of_number(501627),
    );
    for r in &regs[1..] {
        s.add_res(index, r, Td::EMPTY, 1, Td::of_number(501627));
    }
    s.add_res(
        index,
        "Server.Items.BlankScroll",
        Td::of_number(1044377),
        1,
        Td::of_number(1044378),
    );
    s.set_mana_req(index, mana, 1042403);
}

/// `AddMysticismSpell(spell, mana, minSkill, maxSkill, type, params Type[] regs)`.
#[allow(clippy::too_many_arguments)]
pub fn add_mysticism_spell(
    s: &mut CraftSystem,
    _st: &mut HelperState,
    spell: i32,
    mana: i32,
    min_skill: f64,
    max_skill: f64,
    ty: &'static str,
    regs: &[&'static str],
) {
    let index = s.add_craft(
        ty,
        Td::of_number(1111671),
        Td::of_number(1031678 + spell),
        None,
        None,
        min_skill,
        max_skill,
        regs[0],
        Td::EMPTY,
        1,
        Td::of_number(501627),
    );
    for r in &regs[1..] {
        s.add_res(index, r, Td::EMPTY, 1, Td::of_number(501627));
    }
    s.add_res(
        index,
        "Server.Items.BlankScroll",
        Td::of_number(1044377),
        1,
        Td::of_number(1044378),
    );
    s.set_mana_req(index, mana, 1042403);
}

/// `AddJewelrySet(gemType, itemType)`.
pub fn add_jewelry_set(
    s: &mut CraftSystem,
    _st: &mut HelperState,
    gem_type: GemType,
    item_type: &'static str,
) {
    let offset = gem_type as i32 - 1;
    for (ty, first) in [
        ("Server.Items.GoldRing", 1044176),
        ("Server.Items.SilverBeadNecklace", 1044185),
        ("Server.Items.GoldNecklace", 1044194),
        ("Server.Items.GoldEarrings", 1044203),
        ("Server.Items.GoldBeadNecklace", 1044212),
        ("Server.Items.GoldBracelet", 1044221),
    ] {
        let index = s.add_craft(
            ty,
            Td::of_number(1044049),
            Td::of_number(first + offset),
            None,
            None,
            40.0,
            90.0,
            "Server.Items.IronIngot",
            Td::of_number(1044036),
            2,
            Td::of_number(1044037),
        );
        s.add_res(
            index,
            item_type,
            Td::of_number(1044231 + offset),
            1,
            Td::of_number(1044240),
        );
    }
}
