//! `SpecialScroll`, `PowerScroll`, `StatCapScroll`, `ScrollofAlacrity` and `ScrollofTranscendence`.
//!
//! `SpecialScroll` is an abstract family: [`ScrollClass`] holds what the four scrolls override.
//!
//! The `MLQuestSystem` check (an active enhanced-skillgain quest) is [`SpecialHooks::gain_skill_quest`] (set by the
//! ML quests); `ScrollofTranscendence.CreateRandom` needs `SkillsInfo.RandomSkill` of the skills system
//! (`muo_content::skills::skills_info::random_skill`) and is [`transcendence_create_random`].

use muo_content::aos::skill_label;
use muo_content::gumps::base::{DynamicGumpLogic, DynamicGumpObj, RelayInfo};
use muo_content::gumps::builder::DynamicGumpBuilder;
use muo_content::gumps::send_gump_to_mobile;
use muo_content::props::{Ctor, Kind, Param, Value, add_ctors};
use muo_content::utilities::types::{Numeric, PropType};
use muo_io::{GenericReader, GenericWriter};
use muo_primitives::dotnet::InvariantCase;
use muo_primitives::{Point3D, TimeSpan, utility};
use muo_server::World;
use muo_server::effects::{self, EffectLayer, Targeted};
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, LootType, effect_item, ops};
use muo_server::net_state::NetStateId;
use muo_server::skills::{self, SkillLock, SkillName, SkillTable};
use muo_server::timer;
use muo_server::type_registry::TypeRegistry;

use super::hooks::hooks;
use super::util::{in_pack, msg, text};

/// `SpecialScroll._skill` and `_value`.
#[derive(Clone, Copy, Debug)]
pub struct ScrollData {
    pub skill: SkillName,
    pub value: f64,
}

fn data(w: &World, id: ItemId) -> ScrollData {
    w.item(id)
        .and_then(|i| i.ext.get::<ScrollData>())
        .copied()
        .unwrap_or(ScrollData {
            skill: SkillName::Alchemy,
            value: 0.0,
        })
}

/// `Skill`.
pub fn skill(w: &World, id: ItemId) -> SkillName {
    data(w, id).skill
}

/// `Value`.
pub fn value(w: &World, id: ItemId) -> f64 {
    data(w, id).value
}

/// `scroll.GetType().CreateEntityInstance<SpecialScroll>()` then `Skill = scroll.Skill; Value = scroll.Value`
/// (the Justice virtue's duplicate of a champion scroll).
pub fn duplicate(w: &mut World, scroll: ItemId) -> Option<ItemId> {
    let ty = w.item(scroll)?.ty;
    let d = data(w, scroll);
    let dupe = (ty.construct?)(w);
    if let Some(i) = w.item_mut(dupe) {
        i.ext.insert(d);
    }
    ops::invalidate_properties(w, dupe);
    Some(dupe)
}

/// What each scroll overrides.
pub struct ScrollClass {
    pub message: i32,
    pub title: fn(&World, ItemId) -> i32,
    pub default_title: fn(&World, ItemId) -> String,
    pub skill_label: fn(&World, ItemId) -> i32,
    pub can_use: fn(&mut World, ItemId, MobileId) -> bool,
    pub use_: fn(&mut World, ItemId, MobileId),
}

static BASE_CLASS: ScrollClass = ScrollClass {
    message: 0,
    title: |_, _| 0,
    default_title: |_, _| String::new(),
    skill_label: |w, id| skill_label(skill(w, id)),
    can_use: base_can_use,
    use_: |_, _, _| {},
};

fn class(w: &World, id: ItemId) -> &'static ScrollClass {
    w.item(id)
        .and_then(|i| i.ty.family::<ScrollClass>())
        .unwrap_or(&BASE_CLASS)
}

/// `SpecialScroll(skill, value)`.
fn special_scroll_new(
    w: &mut World,
    ty: &'static ItemType,
    skill: SkillName,
    value: f64,
) -> ItemId {
    let id = crate::simple::from_template(w, ty, "Server.Items.SpecialScroll");
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(ScrollData { skill, value });
    }
    id
}

/// `SpecialScroll.CanUse`.
fn base_can_use(w: &mut World, id: ItemId, from: MobileId) -> bool {
    if ops::is_deleted(w, id) {
        return false;
    }
    if !in_pack(w, id, from) {
        msg(w, from, 1042001); // That must be in your pack for you to use it.
        return false;
    }
    true
}

/// `0.#` of a double.
fn fmt_0_hash(v: f64) -> String {
    let s = format!("{:.1}", (v * 10.0).round() / 10.0);
    s.strip_suffix(".0").map_or(s.clone(), str::to_string)
}

fn floor10(v: f64) -> f64 {
    (v * 10.0).floor() / 10.0
}

/// `GetSkillName()`: the lower-cased `SkillInfo.Table` name, `???` out of range.
fn skill_name(w: &World, id: ItemId) -> String {
    let index = skill(w, id) as usize;
    w.resources
        .get::<SkillTable>()
        .and_then(|t| t.info.get(index))
        .map_or_else(|| "???".to_string(), |i| i.name.lower_inv())
}

/// `AosSkillBonuses.GetLowercaseLabel(skill)`.
pub fn lowercase_label(skill: SkillName) -> i32 {
    match skill {
        SkillName::Provocation => 1049473,
        SkillName::MagicResist => 1049471,
        SkillName::AnimalTaming => 1049472,
        SkillName::Macing => 1049470,
        SkillName::Necromancy => 1060842,
        SkillName::Focus => 1061613,
        SkillName::Chivalry => 1061615,
        SkillName::Bushido => 1062935,
        SkillName::Ninjitsu => 1062936,
        SkillName::Spellweaving => 1074397,
        SkillName::Mysticism => 1112544,
        SkillName::Imbuing => 1112545,
        SkillName::Throwing => 1112553,
        other => 1042347 + other as i32,
    }
}

// ---- the gump ----

/// `SpecialScroll.SpecialScrollGump`.
pub struct SpecialScrollGump {
    scroll: ItemId,
    message: i32,
    title: i32,
    default_title: String,
    skill_label: i32,
}

impl DynamicGumpLogic for SpecialScrollGump {
    const TYPE_NAME: &'static str = "Server.Items.SpecialScroll+SpecialScrollGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        b.add_page(0);
        b.add_background(25, 10, 420, 200, 5054);
        b.add_image_tiled(33, 20, 401, 181, 2624);
        b.add_alpha_region(33, 20, 401, 181);
        b.add_html_localized(40, 48, 387, 100, self.message, true, true);
        b.add_html_localized_color(125, 148, 200, 20, 1049478, 0x7FFF, false, false); // Do you wish to use this scroll?
        b.add_button(100, 172, 4005, 4007, 1);
        b.add_html_localized_color(135, 172, 120, 20, 1046362, 0x7FFF, false, false); // Yes
        b.add_button(275, 172, 4005, 4007, 0);
        b.add_html_localized_color(310, 172, 120, 20, 1046363, 0x7FFF, false, false); // No
        if self.title != 0 {
            b.add_html_localized_color(40, 20, 260, 20, self.title, 0x7FFF, false, false);
        } else {
            b.add_html(40, 20, 260, 20, &self.default_title, false, false);
        }
        if self.skill_label > 0 {
            b.add_html_localized_color(310, 20, 120, 20, self.skill_label, 0x7FFF, false, false);
        }
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        if info.button_id == 1
            && let Some(m) = w.net_states.get(&sender).and_then(|n| n.mobile)
        {
            use_scroll(w, self.scroll, m);
        }
    }
}

/// `scroll.Use(from)`.
pub fn use_scroll(w: &mut World, id: ItemId, from: MobileId) {
    let f = class(w, id).use_;
    f(w, id, from);
}

fn scroll_on_double_click(w: &mut World, id: ItemId, from: MobileId) {
    let c = class(w, id);
    if !(c.can_use)(w, id, from) {
        return;
    }
    if super::util::net_state(w, from).is_none() || ops::is_deleted(w, id) {
        return;
    }
    let gump = SpecialScrollGump {
        scroll: id,
        message: c.message,
        title: (c.title)(w, id),
        default_title: (c.default_title)(w, id),
        skill_label: (c.skill_label)(w, id),
    };
    send_gump_to_mobile(w, from, Box::new(DynamicGumpObj::new(25, 50, gump)), false);
}

fn scroll_serialize(w: &World, id: ItemId, out: &mut dyn GenericWriter) {
    (ITEM_BASE.serialize)(w, id, out);
    out.write_i32(0);
    let d = data(w, id);
    out.write_enum(4, d.skill as i64);
    out.write_f64(d.value);
}

fn scroll_deserialize(w: &mut World, id: ItemId, r: &mut dyn GenericReader) -> muo_io::Result<()> {
    (ITEM_BASE.deserialize)(w, id, r)?;
    let _version = r.read_i32()?;
    let skill = SkillName::from_i32(r.read_enum(4)? as i32).unwrap_or_default();
    let value = r.read_f64()?;
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(ScrollData { skill, value });
    }
    Ok(())
}

/// `Server.Items.SpecialScroll` (abstract).
pub static SPECIAL_SCROLL: ItemType = ItemType {
    full_name: "Server.Items.SpecialScroll",
    parent: Some(&ITEM_BASE),
    construct: None,
    family: Some(&BASE_CLASS),
    default_weight: crate::class_props::default_weight,
    on_double_click: scroll_on_double_click,
    serialize: scroll_serialize,
    deserialize: scroll_deserialize,
    ..ITEM_BASE
};

/// The shared effects of the power and stat scrolls.
fn power_effects(w: &mut World, from: MobileId) {
    let (loc, map) = (
        muo_content::mx::location(w, from),
        muo_content::mx::map(w, from),
    );
    let fx = effect_item::create_targeted(w, loc, map, effect_item::DEFAULT_DURATION);
    effects::send_location_particles(w, fx, 0, 0, 0, 5060);
    effects::play_sound(w, loc, map, 0x243);
    for (dx, dy) in [(-6, -6), (-4, -6), (-6, -4)] {
        effects::send_moving_particles_layer(
            w,
            Targeted::point(Point3D::new(loc.x + dx, loc.y + dy, loc.z + 15), map),
            from,
            0x36D4,
            7,
            0,
            false,
            true,
            0x497,
            0,
            9502,
            1,
            0,
            255,
            0x100,
        );
    }
    effects::send_target_particles_full(
        w,
        from,
        0x375A,
        35,
        90,
        0x00,
        0x00,
        9502,
        EffectLayer(255),
        0x100,
    );
}

// ---- PowerScroll ----

const PRE_AOS: [SkillName; 22] = [
    SkillName::Blacksmith,
    SkillName::Tailoring,
    SkillName::Swords,
    SkillName::Fencing,
    SkillName::Macing,
    SkillName::Archery,
    SkillName::Wrestling,
    SkillName::Parry,
    SkillName::Tactics,
    SkillName::Anatomy,
    SkillName::Healing,
    SkillName::Magery,
    SkillName::Meditation,
    SkillName::EvalInt,
    SkillName::MagicResist,
    SkillName::AnimalTaming,
    SkillName::AnimalLore,
    SkillName::Veterinary,
    SkillName::Musicianship,
    SkillName::Provocation,
    SkillName::Discordance,
    SkillName::Peacemaking,
];
const AOS: [SkillName; 6] = [
    SkillName::Chivalry,
    SkillName::Focus,
    SkillName::Necromancy,
    SkillName::Stealing,
    SkillName::Stealth,
    SkillName::SpiritSpeak,
];
const SE: [SkillName; 2] = [SkillName::Ninjitsu, SkillName::Bushido];
const ML: [SkillName; 1] = [SkillName::Spellweaving];

/// `PowerScroll.Skills` for the current expansion (the first answer is cached in a static; the era does not
/// change at run time).
pub fn power_scroll_skills(w: &World) -> Vec<SkillName> {
    let mut v = PRE_AOS.to_vec();
    if muo_content::core::aos(w) {
        v.extend(AOS);
    }
    if muo_content::core::se(w) {
        v.extend(SE);
    }
    if muo_content::core::ml(w) {
        v.extend(ML);
    }
    v
}

/// `PowerScroll(skill, value)`.
pub fn power_scroll_new(w: &mut World, skill: SkillName, value: f64) -> ItemId {
    let id = special_scroll_new(w, &POWER_SCROLL, skill, value);
    super::hue(w, id, 0x481);
    if value - 105.0 < 0.1 || matches!(skill, SkillName::Blacksmith | SkillName::Tailoring) {
        ops::set_loot_type(w, id, LootType::Regular);
    }
    id
}

/// `PowerScroll.CreateRandom(min, max)`.
pub fn power_scroll_create_random(w: &mut World, min: i32, max: i32) -> ItemId {
    let (min, max) = (min / 5, max / 5);
    let skills = power_scroll_skills(w);
    let s = *utility::random_element(&mut w.rng, &skills).expect("not empty");
    let r = utility::random_min_max(&mut w.rng, min, max);
    power_scroll_new(w, s, f64::from(100 + r * 5))
}

/// `PowerScroll.CreateRandomNoCraft(min, max)`.
pub fn power_scroll_create_random_no_craft(w: &mut World, min: i32, max: i32) -> ItemId {
    let (min, max) = (min / 5, max / 5);
    let skills = power_scroll_skills(w);
    let s = loop {
        let s = *utility::random_element(&mut w.rng, &skills).expect("not empty");
        if !matches!(s, SkillName::Blacksmith | SkillName::Tailoring) {
            break s;
        }
    };
    let r = utility::random_min_max(&mut w.rng, min, max);
    power_scroll_new(w, s, f64::from(100 + r * 5))
}

/// The `(level)` of a power scroll value when it has a name (105, 110, 115, 120).
fn power_level(value: f64) -> Option<i32> {
    let trunc = floor10(value);
    if (105.0..=120.0).contains(&value) && value - trunc < 0.1 {
        let n = trunc as i32 - 105;
        if n % 5 == 0 {
            return Some(n / 5);
        }
    }
    None
}

static POWER_CLASS: ScrollClass = ScrollClass {
    // Using a scroll increases the maximum amount of a specific skill or your maximum statistics.
    message: 1049469,
    title: |w, id| {
        let v = value(w, id);
        if (105.0..=120.0).contains(&v) {
            let trunc = v.floor();
            if v - trunc < 0.1 {
                return 1049635 + (trunc as i32 - 105) / 5;
            }
        }
        0
    },
    default_title: |w, id| {
        muo_primitives::html::color_rgb(
            &format!(
                "Power Scroll ({} Skill):",
                fmt_0_hash(floor10(value(w, id)))
            ),
            0xFFFFFF,
        )
    },
    skill_label: BASE_CLASS.skill_label,
    can_use: |w, id, from| {
        if !base_can_use(w, id, from) {
            return false;
        }
        let s = skill(w, id);
        if w.mobile(from).and_then(|m| m.skills.as_ref()).is_none() {
            return false;
        }
        if skills::cap(w, from, s) >= value(w, id) {
            super::util::msg_args(w, from, 1049511, &format!("#{}", lowercase_label(s)));
            return false;
        }
        true
    },
    use_: |w, id, from| {
        if !(power_class().can_use)(w, id, from) {
            return;
        }
        let s = skill(w, id);
        super::util::msg_args(w, from, 1049513, &format!("#{}", lowercase_label(s)));
        skills::set_cap(w, from, s, value(w, id));
        power_effects(w, from);
        super::util::delete(w, id);
    },
};

fn power_class() -> &'static ScrollClass {
    &POWER_CLASS
}

/// Server.Items.PowerScroll
pub static POWER_SCROLL: ItemType = ItemType {
    full_name: "Server.Items.PowerScroll",
    parent: Some(&SPECIAL_SCROLL),
    construct: Some(|w| power_scroll_new(w, SkillName::Alchemy, 0.0)),
    family: Some(&POWER_CLASS),
    add_name_property: |w, id, list| {
        let v = value(w, id);
        if let Some(level) = power_level(v) {
            list.add_localized_number(1049639 + level, lowercase_label(skill(w, id)));
            return;
        }
        list.add_text(&format!(
            "a power scroll of {} ({} Skill)",
            skill_name(w, id),
            fmt_0_hash(floor10(v))
        ));
    },
    on_single_click: |w, id, from| {
        (ITEM_BASE.on_single_click)(w, id, from);
        let v = value(w, id);
        if let Some(level) = power_level(v) {
            ops::label_to_number(
                w,
                id,
                from,
                1049639 + level,
                &format!("#{}", lowercase_label(skill(w, id))),
            );
            return;
        }
        ops::label_to(
            w,
            id,
            from,
            &format!(
                "a power scroll of {} ({} Skill)",
                skill_name(w, id),
                fmt_0_hash(floor10(v))
            ),
        );
    },
    serialize: |w, id, out| {
        (SPECIAL_SCROLL.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (SPECIAL_SCROLL.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        Ok(())
    },
    ..SPECIAL_SCROLL
};

// ---- StatCapScroll ----

fn has_stat_reward(w: &World, m: MobileId) -> bool {
    muo_content::player_mobile::data(w, m).is_some_and(|p| {
        p.flags
            .contains(muo_content::player_mobile::PlayerFlag::HAS_STAT_REWARD)
    })
}

/// `StatCapScroll(value)`.
pub fn stat_cap_scroll_new(w: &mut World, value: i32) -> ItemId {
    let id = special_scroll_new(w, &STAT_CAP_SCROLL, SkillName::Alchemy, f64::from(value));
    super::hue(w, id, 0x481);
    id
}

/// `Math.DivRem((int)Value - 230, 5, out rem)` when in `0..=4` and `rem == 0`.
fn stat_level(value: f64) -> (i32, i32) {
    let t = value as i32 - 230;
    (t / 5, t % 5)
}

fn stat_label(value: f64) -> String {
    let diff = value as i32 - 225;
    format!(
        "a scroll of power ({}{} Maximum Stats)",
        if diff >= 0 { "+" } else { "" },
        diff
    )
}

static STAT_CLASS: ScrollClass = ScrollClass {
    message: 1049469,
    title: |w, id| {
        let (level, rem) = stat_level(value(w, id));
        if (0..=4).contains(&level) && rem == 0 {
            1049458 + level
        } else {
            0
        }
    },
    default_title: |w, id| {
        let diff = value(w, id) as i32 - 225;
        muo_primitives::html::color_rgb(
            &format!(
                "Power Scroll ({}{} Maximum Stats):",
                if diff >= 0 { "+" } else { "" },
                diff
            ),
            0xFFFFFF,
        )
    },
    skill_label: |_, _| 0,
    can_use: |w, id, from| {
        if !base_can_use(w, id, from) {
            return false;
        }
        let mut new_value = value(w, id) as i32;
        if has_stat_reward(w, from) {
            new_value += 5;
        }
        if w.mobile(from).is_some_and(|m| m.stat_cap >= new_value) {
            msg(w, from, 1049510); // Your stats are too high for this power scroll.
            return false;
        }
        true
    },
    use_: |w, id, from| {
        if !(STAT_CLASS.can_use)(w, id, from) {
            return;
        }
        msg(w, from, 1049512); // You feel a surge of magic as the scroll enhances your powers!
        let v = value(w, id) as i32;
        let cap = if has_stat_reward(w, from) { v + 5 } else { v };
        muo_server::mobile::part1::props_b::set_stat_cap(w, from, cap);
        power_effects(w, from);
        super::util::delete(w, id);
    },
};

/// `Server.Items.StatCapScroll` (`[TypeAlias("Server.Items.StatScroll")]`).
pub static STAT_CAP_SCROLL: ItemType = ItemType {
    full_name: "Server.Items.StatCapScroll",
    parent: Some(&SPECIAL_SCROLL),
    construct: Some(|w| stat_cap_scroll_new(w, 230)),
    family: Some(&STAT_CLASS),
    add_name_property: |w, id, list| {
        let v = value(w, id);
        let (level, rem) = stat_level(v);
        if (0..=4).contains(&level) && rem == 0 {
            list.add_localized_number(1049463 + level, 1049476);
        } else {
            list.add_text(&stat_label(v));
        }
    },
    on_single_click: |w, id, from| {
        (ITEM_BASE.on_single_click)(w, id, from);
        let v = value(w, id);
        let level = (v as i32 - 230) / 5;
        if (0..=4).contains(&level) {
            ops::label_to_number(w, id, from, 1049463 + level, "#1049476");
        } else {
            ops::label_to(w, id, from, &stat_label(v));
        }
    },
    serialize: |w, id, out| {
        (SPECIAL_SCROLL.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (SPECIAL_SCROLL.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        Ok(())
    },
    ..SPECIAL_SCROLL
};

// ---- ScrollofAlacrity, ScrollofTranscendence ----

/// The checks both skill scrolls share after `base.CanUse`.
fn skillgain_can_use(w: &mut World, id: ItemId, from: MobileId) -> bool {
    if !base_can_use(w, id, from) || muo_content::player_mobile::data(w, from).is_none() {
        return false;
    }
    let s = skill(w, id);
    if hooks(w)
        .and_then(|h| h.gain_skill_quest)
        .is_some_and(|f| f(w, from, s))
    {
        text(
            w,
            from,
            "You are already under the effect of an enhanced skillgain quest.",
        );
        return false;
    }
    if muo_content::player_mobile::data(w, from).is_some_and(|p| p.accelerated_start > w.now) {
        msg(w, from, 1077951); // You are already under the effect of an accelerated skillgain scroll.
        return false;
    }
    true
}

/// `ScrollofAlacrity(skill)`.
pub fn scroll_of_alacrity_new(w: &mut World, skill: SkillName) -> ItemId {
    let id = special_scroll_new(w, &SCROLL_OF_ALACRITY, skill, 0.0);
    let ty = w.item(id).map(|i| i.ty).expect("item");
    (ty.set_item_id)(w, id, 0x14EF);
    super::hue(w, id, 0x4AB);
    id
}

static ALACRITY_CLASS: ScrollClass = ScrollClass {
    message: 1078602,
    title: |_, _| 0,
    default_title: |_, _| "<basefont color=#FFFFFF>Scroll of Alacrity:</basefont>".to_string(),
    skill_label: BASE_CLASS.skill_label,
    can_use: skillgain_can_use,
    use_: |w, id, from| {
        if !skillgain_can_use(w, id, from) {
            return;
        }
        let s = skill(w, id);
        let (base, cap) = (skills::base(w, from, s), skills::cap(w, from, s));
        if base >= cap || skills::lock(w, from, s) != SkillLock::Up {
            // You cannot increase this skill at this time. The skill may be locked or set to lower in your skill menu.
            msg(w, from, 1094935);
            return;
        }
        msg(w, from, 1077956);
        let (loc, map) = (
            muo_content::mx::location(w, from),
            muo_content::mx::map(w, from),
        );
        effects::play_sound(w, loc, map, 0x1E9);
        effects::send_target_particles_full(
            w,
            from,
            0x373A,
            35,
            45,
            0x00,
            0x00,
            9502,
            EffectLayer(255),
            0x100,
        );
        let until = w.now + TimeSpan::from_secs(15 * 60);
        if let Some(p) = muo_content::player_mobile::data_mut(w, from) {
            p.accelerated_start = until;
            p.accelerated_skill = s;
        }
        timer::start(
            w,
            TimeSpan::from_secs(15 * 60),
            TimeSpan::ZERO,
            1,
            move |w| {
                muo_content::mx::play_sound(w, from, 0x1F8);
                msg(w, from, 1077957);
            },
        );
        super::util::delete(w, id);
    },
};

/// `Server.Items.ScrollofAlacrity`.
pub static SCROLL_OF_ALACRITY: ItemType = ItemType {
    full_name: "Server.Items.ScrollofAlacrity",
    parent: Some(&SPECIAL_SCROLL),
    construct: Some(|w| scroll_of_alacrity_new(w, SkillName::Alchemy)),
    family: Some(&ALACRITY_CLASS),
    label_number: crate::class_props::label_number, // Scroll of Alacrity
    get_properties: |w, id, list| {
        (SPECIAL_SCROLL.get_properties)(w, id, list);
        list.add_localized_number(1071345, skill_label(skill(w, id))); // Skill: ~1_val~
        list.add_int(1071346, 15); // Duration: ~1_val~ minutes
    },
    serialize: |w, id, out| {
        (SPECIAL_SCROLL.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (SPECIAL_SCROLL.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        Ok(())
    },
    ..SPECIAL_SCROLL
};

/// `ScrollofTranscendence(skill, value)`.
pub fn scroll_of_transcendence_new(w: &mut World, skill: SkillName, value: f64) -> ItemId {
    let id = special_scroll_new(w, &SCROLL_OF_TRANSCENDENCE, skill, value);
    let ty = w.item(id).map(|i| i.ty).expect("item");
    (ty.set_item_id)(w, id, 0x14EF);
    super::hue(w, id, 0x490);
    id
}

/// `ScrollofTranscendence.CreateRandom(min, max)`.
pub fn transcendence_create_random(w: &mut World, min: i32, max: i32) -> ItemId {
    let s = muo_content::skills::skills_info::random_skill(w);
    let r = utility::random_min_max(&mut w.rng, min, max);
    scroll_of_transcendence_new(w, s, f64::from(r) / 10.0)
}

static TRANSCENDENCE_CLASS: ScrollClass = ScrollClass {
    message: 1094933,
    title: |_, _| 0,
    default_title: |w, id| {
        muo_primitives::html::color_rgb(
            &format!(
                "Scroll of Transcendence ({} Skill):",
                fmt_0_hash(floor10(value(w, id)))
            ),
            0xFFFFFF,
        )
    },
    skill_label: BASE_CLASS.skill_label,
    can_use: skillgain_can_use,
    use_: |w, id, from| {
        if !skillgain_can_use(w, id, from) {
            return;
        }
        let s = skill(w, id);
        let (base, cap) = (skills::base(w, from, s), skills::cap(w, from, s));
        let mut can_gain = false;
        let mut new_value = value(w, id);
        if base + new_value > cap {
            new_value = cap - base;
        }
        if base < cap && skills::lock(w, from, s) == SkillLock::Up {
            let total = f64::from(skills::total(w, from));
            let skills_cap = f64::from(muo_server::mobile::part1::props_a::skills_cap(w, from));
            if total + new_value * 10.0 > skills_cap {
                let n = w
                    .mobile(from)
                    .and_then(|m| m.skills.as_ref())
                    .map_or(0, |x| x.len());
                for i in 0..n as i32 {
                    let Some(sk) = SkillName::from_i32(i) else {
                        continue;
                    };
                    if skills::lock(w, from, sk) == SkillLock::Down
                        && skills::base(w, from, sk) >= new_value
                    {
                        let b = skills::base(w, from, sk);
                        skills::set_base(w, from, sk, b - new_value);
                        can_gain = true;
                        break;
                    }
                }
            } else {
                can_gain = true;
            }
        }
        if !can_gain {
            // You cannot increase this skill at this time...
            msg(w, from, 1094935);
            return;
        }
        super::util::msg_args(w, from, 1049513, &format!("#{}", lowercase_label(s)));
        let b = skills::base(w, from, s);
        skills::set_base(w, from, s, b + new_value);
        let (loc, map) = (
            muo_content::mx::location(w, from),
            muo_content::mx::map(w, from),
        );
        effects::play_sound(w, loc, map, 0x1F7);
        effects::send_target_particles_full(
            w,
            from,
            0x373A,
            35,
            45,
            0x00,
            0x00,
            9502,
            EffectLayer(255),
            0x100,
        );
        effects::send_target_particles_full(
            w,
            from,
            0x376A,
            35,
            45,
            0x00,
            0x00,
            9502,
            EffectLayer(255),
            0x100,
        );
        super::util::delete(w, id);
    },
};

/// `Server.Items.ScrollofTranscendence`.
pub static SCROLL_OF_TRANSCENDENCE: ItemType = ItemType {
    full_name: "Server.Items.ScrollofTranscendence",
    parent: Some(&SPECIAL_SCROLL),
    construct: Some(|w| scroll_of_transcendence_new(w, SkillName::Alchemy, 0.0)),
    family: Some(&TRANSCENDENCE_CLASS),
    label_number: crate::class_props::label_number, // Scroll of Transcendence
    get_properties: |w, id, list| {
        (SPECIAL_SCROLL.get_properties)(w, id, list);
        list.add_str(
            1151930,
            &format!(
                "#{}\t{}\t#{}",
                skill_label(skill(w, id)),
                fmt_0_hash(floor10(value(w, id))),
                1151931
            ),
        );
    },
    serialize: |w, id, out| {
        (SPECIAL_SCROLL.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (SPECIAL_SCROLL.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        Ok(())
    },
    ..SPECIAL_SCROLL
};

// ---- registration ----

const INT: Kind = Kind::Plain(PropType::Numeric(Numeric::Int));
const DOUBLE: Kind = Kind::Plain(PropType::Double);

fn skill_kind() -> Kind {
    Kind::Plain(PropType::Numeric(Numeric::Int))
}

static POWER_CTORS: [Ctor; 1] = [Ctor {
    access: None,
    params: &[
        Param {
            name: "skill",
            kind: INT,
            default: Some(Value::Int(0)),
            ty: "SkillName",
        },
        Param {
            name: "value",
            kind: DOUBLE,
            default: Some(Value::Float(0.0)),
            ty: "",
        },
    ],
    make: |w, v| {
        let s = SkillName::from_i32(v[0].i32()).unwrap_or_default();
        EntityId::Item(power_scroll_new(w, s, v[1].f64()))
    },
}];

static STAT_CTORS: [Ctor; 1] = [Ctor {
    access: None,
    params: &[Param {
        name: "value",
        kind: INT,
        default: Some(Value::Int(230)),
        ty: "",
    }],
    make: |w, v| EntityId::Item(stat_cap_scroll_new(w, v[0].i32())),
}];

static ALACRITY_CTORS: [Ctor; 1] = [Ctor {
    access: None,
    params: &[Param {
        name: "skill",
        kind: INT,
        default: Some(Value::Int(0)),
        ty: "SkillName",
    }],
    make: |w, v| {
        EntityId::Item(scroll_of_alacrity_new(
            w,
            SkillName::from_i32(v[0].i32()).unwrap_or_default(),
        ))
    },
}];

static TRANSCENDENCE_CTORS: [Ctor; 1] = [Ctor {
    access: None,
    params: &[
        Param {
            name: "skill",
            kind: INT,
            default: Some(Value::Int(0)),
            ty: "SkillName",
        },
        Param {
            name: "value",
            kind: DOUBLE,
            default: Some(Value::Float(0.0)),
            ty: "",
        },
    ],
    make: |w, v| {
        let s = SkillName::from_i32(v[0].i32()).unwrap_or_default();
        EntityId::Item(scroll_of_transcendence_new(w, s, v[1].f64()))
    },
}];

pub(super) fn configure(w: &mut World) {
    let _ = skill_kind;
    add_ctors(w, "Server.Items.PowerScroll", &POWER_CTORS);
    add_ctors(w, "Server.Items.StatCapScroll", &STAT_CTORS);
    add_ctors(w, "Server.Items.ScrollofAlacrity", &ALACRITY_CTORS);
    add_ctors(
        w,
        "Server.Items.ScrollofTranscendence",
        &TRANSCENDENCE_CTORS,
    );
}

pub fn register(types: &mut TypeRegistry) {
    types.register_item(&SPECIAL_SCROLL, &[]);
    types.register_item(&POWER_SCROLL, &[]);
    types.register_item(&STAT_CAP_SCROLL, &["Server.Items.StatScroll"]);
    types.register_item(&SCROLL_OF_ALACRITY, &[]);
    types.register_item(&SCROLL_OF_TRANSCENDENCE, &[]);
}
