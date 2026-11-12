//! `SkillsGump`: `EditSkillGump`, `SkillsGump` and `SkillsGumpGroup`.

use muo_content::gumps::base::{LegacyGumpLogic, LegacyGumpObj, RelayInfo};
use muo_primitives::html::center_rgb;
use muo_server::World;
use muo_server::ids::{EntityId, MobileId};
use muo_server::mobile::AccessLevel;
use muo_server::net_state::NetStateId;
use muo_server::skills::{SkillLock, SkillName, SkillTable};

use super::common::send;
use super::hooks::props_host;
use crate::m_z::props::host::{PropObj, PropValue};
use muo_content::gumps::props_config::*;
use muo_content::gumps::props_layout::*;

/// `Type.Name` of a full class name.
pub fn short_type_name(full_name: &str) -> &str {
    full_name.rsplit('.').next().unwrap_or(full_name)
}

/// `SkillsGumpGroup`: a name and its skills, sorted by the skills' names.
#[derive(Clone, Debug)]
pub struct SkillsGumpGroup {
    pub name: &'static str,
    pub skills: Vec<SkillName>,
}

fn skill_name_of(w: &World, skill: SkillName) -> String {
    w.resources
        .get::<SkillTable>()
        .and_then(|t| t.info.get(skill as usize))
        .map(|i| i.name.clone())
        .unwrap_or_default()
}

/// `SkillsGumpGroup.Groups`.
pub fn groups(w: &World) -> Vec<SkillsGumpGroup> {
    use SkillName::*;
    let defs: [(&'static str, &[SkillName]); 7] = [
        (
            "Crafting",
            &[
                Alchemy,
                Blacksmith,
                Cartography,
                Carpentry,
                Cooking,
                Fletching,
                Inscribe,
                Tailoring,
                Tinkering,
                Imbuing,
            ],
        ),
        (
            "Bardic",
            &[Discordance, Musicianship, Peacemaking, Provocation],
        ),
        (
            "Magical",
            &[
                Chivalry,
                EvalInt,
                Magery,
                MagicResist,
                Meditation,
                Necromancy,
                SpiritSpeak,
                Ninjitsu,
                Bushido,
                Spellweaving,
                Mysticism,
            ],
        ),
        (
            "Miscellaneous",
            &[
                Camping,
                Fishing,
                Focus,
                Healing,
                Herding,
                Lockpicking,
                Lumberjacking,
                Mining,
                Snooping,
                Veterinary,
            ],
        ),
        (
            "Combat Ratings",
            &[
                Archery, Fencing, Macing, Parry, Swords, Tactics, Wrestling, Throwing,
            ],
        ),
        (
            "Actions",
            &[
                AnimalTaming,
                Begging,
                DetectHidden,
                Hiding,
                RemoveTrap,
                Poisoning,
                Stealing,
                Stealth,
                Tracking,
            ],
        ),
        (
            "Lore & Knowledge",
            &[Anatomy, AnimalLore, ArmsLore, Forensics, ItemID, TasteID],
        ),
    ];
    defs.iter()
        .map(|(name, skills)| {
            let mut skills = skills.to_vec();
            // string.CompareOrdinal on the names
            skills.sort_by_key(|a| skill_name_of(w, *a));
            SkillsGumpGroup { name, skills }
        })
        .collect()
}

fn skill_exists(w: &World, skill: SkillName) -> bool {
    w.resources
        .get::<SkillTable>()
        .is_some_and(|t| (skill as usize) < t.info.len())
}

// ---- EditSkillGump ----

const EDIT_ENTRY_WIDTH: i32 = 160;
const EDIT_TOTAL_WIDTH: i32 =
    OFFSET_SIZE + EDIT_ENTRY_WIDTH + OFFSET_SIZE + SET_WIDTH + OFFSET_SIZE;

pub struct EditSkillGump {
    from: MobileId,
    target: MobileId,
    skill: SkillName,
    selected: Option<usize>,
}

/// `new EditSkillGump(from, target, skill, selected)`.
pub fn new_edit_skill(
    w: &mut World,
    from: MobileId,
    target: MobileId,
    skill: SkillName,
    selected: Option<usize>,
) -> LegacyGumpObj<EditSkillGump> {
    let mut g = LegacyGumpObj::new(
        GUMP_OFFSET_X,
        GUMP_OFFSET_Y,
        EditSkillGump {
            from,
            target,
            skill,
            selected,
        },
    );

    let initial_text = format!("{:.1}", muo_server::skills::base(w, target, skill));
    let name = skill_name_of(w, skill);

    let l = &mut g.layout;
    l.add_page(0);

    let (mut x, mut y) = add_props_frame(l, EDIT_TOTAL_WIDTH, 2);
    add_props_entry_label(l, &mut x, &mut y, EDIT_ENTRY_WIDTH, &name);
    next_row(&mut x, &mut y);
    add_props_entry_text_input(
        l,
        &mut x,
        &mut y,
        EDIT_ENTRY_WIDTH,
        0,
        &initial_text,
        Some(1),
    );
    g
}

impl LegacyGumpLogic for EditSkillGump {
    const TYPE_NAME: &'static str = "Server.Gumps.EditSkillGump";

    fn on_response(&mut self, w: &mut World, _sender: NetStateId, info: &RelayInfo) {
        let (from, target, skill, selected) = (self.from, self.target, self.skill, self.selected);
        if info.button_id == 1 {
            let mut bad_format = false;
            if muo_content::mx::access_level(w, from) >= AccessLevel::GameMaster {
                let text = info.get_text_entry(0).map(str::trim);

                if let Some(text) = text.filter(|t| !t.is_empty()) {
                    match text.parse::<f64>() {
                        Ok(v) => {
                            muo_server::skills::set_base(w, target, skill, v);
                            let base = muo_server::skills::base(w, target, skill);
                            if let Some(host) = props_host(w) {
                                let name = skill_name_of(w, skill);
                                host.log_change(
                                    w,
                                    from,
                                    &PropObj::Entity(EntityId::Mobile(target)),
                                    &format!("[{name}: {base}].Base"),
                                    &PropValue::Text(base.to_string()),
                                );
                            }
                        }
                        Err(_) => bad_format = true,
                    }
                }
            } else {
                muo_content::mx::send_message(w, from, "You may not change that.");
            }

            if bad_format {
                muo_content::mx::send_message(w, from, "Bad format. ###.# expected.");
                let g = new_edit_skill(w, from, target, skill, selected);
                send(w, from, g);
            } else {
                let g = new_skills(w, from, target, selected);
                send(w, from, g);
            }
        } else {
            let g = new_skills(w, from, target, selected);
            send(w, from, g);
        }
    }
}

// ---- SkillsGump ----

const NAME_WIDTH: i32 = 107;
const VALUE_WIDTH: i32 = 128;

const TOTAL_WIDTH: i32 =
    OFFSET_SIZE + NAME_WIDTH + OFFSET_SIZE + VALUE_WIDTH + OFFSET_SIZE + SET_WIDTH + OFFSET_SIZE;

const INDENT_WIDTH: i32 = 12;

pub struct SkillsGump {
    from: MobileId,
    target: MobileId,
    groups: Vec<SkillsGumpGroup>,
    selected: Option<usize>,
}

/// `GetButtonID(type, index)`.
fn get_button_id(kind: i32, index: i32) -> i32 {
    1 + index * 3 + kind
}

/// `new SkillsGump(from, target, selected)`; `selected` is an index into [`groups`].
pub fn new_skills(
    w: &mut World,
    from: MobileId,
    target: MobileId,
    selected: Option<usize>,
) -> LegacyGumpObj<SkillsGump> {
    let groups = groups(w);
    let mut count = groups.len() as i32;
    if let Some(s) = selected {
        count += groups[s].skills.len() as i32;
    }

    let title = {
        let (ty, serial) = w
            .mobile(target)
            .map_or((String::new(), muo_primitives::Serial::ZERO), |m| {
                (short_type_name(m.ty.full_name).to_owned(), m.serial)
            });
        center_rgb(&format!("{ty} ({serial})"), 0xFAFAFA)
    };

    // Everything the layout reads from the target.
    struct SkillView {
        exists: bool,
        name: String,
        lock: SkillLock,
        base: f64,
    }
    let mut views = Vec::new();
    if let Some(s) = selected {
        for skill in groups[s].skills.clone() {
            views.push(SkillView {
                exists: skill_exists(w, skill),
                name: skill_name_of(w, skill),
                lock: muo_server::skills::lock(w, target, skill),
                base: muo_server::skills::base(w, target, skill),
            });
        }
    }

    let mut g = LegacyGumpObj::new(
        GUMP_OFFSET_X,
        GUMP_OFFSET_Y,
        SkillsGump {
            from,
            target,
            groups: groups.clone(),
            selected,
        },
    );
    let l = &mut g.layout;

    l.add_page(0);

    let (mut x, mut y) = add_props_frame(l, TOTAL_WIDTH, count + 1);
    add_props_header(
        l,
        TOTAL_WIDTH,
        &mut x,
        &mut y,
        &Header::new(Some(&title), None, None),
    );

    const EMPTY_WIDTH: i32 = TOTAL_WIDTH - PREV_WIDTH - NEXT_WIDTH - OFFSET_SIZE * 4;

    for (i, group) in groups.iter().enumerate() {
        x = BORDER_SIZE + OFFSET_SIZE;
        y += ENTRY_HEIGHT + OFFSET_SIZE;

        l.add_image_tiled(x, y, PREV_WIDTH, ENTRY_HEIGHT, HEADER_GUMP_ID);

        let is_selected = selected == Some(i);
        if is_selected {
            l.add_button(
                x + PREV_OFFSET_X,
                y + PREV_OFFSET_Y,
                0x15E2,
                0x15E6,
                get_button_id(0, i as i32),
            );
        } else {
            l.add_button(
                x + PREV_OFFSET_X,
                y + PREV_OFFSET_Y,
                0x15E1,
                0x15E5,
                get_button_id(0, i as i32),
            );
        }

        x += PREV_WIDTH + OFFSET_SIZE;

        l.add_image_tiled(x, y, EMPTY_WIDTH, ENTRY_HEIGHT, ENTRY_GUMP_ID);
        l.add_label(x + TEXT_OFFSET_X, y, TEXT_HUE, group.name);

        x += EMPTY_WIDTH;
        x += OFFSET_SIZE;

        if SET_GUMP_ID != 0 {
            l.add_image_tiled(x, y, SET_WIDTH, ENTRY_HEIGHT, SET_GUMP_ID);
        }

        if !is_selected {
            continue;
        }

        let indent_mask_y = y + ENTRY_HEIGHT + OFFSET_SIZE;

        for (j, sk) in views.iter().enumerate() {
            let j = j as i32;
            x = BORDER_SIZE + OFFSET_SIZE;
            y += ENTRY_HEIGHT + OFFSET_SIZE;

            x += OFFSET_SIZE;
            x += INDENT_WIDTH;

            l.add_image_tiled(x, y, PREV_WIDTH, ENTRY_HEIGHT, HEADER_GUMP_ID);

            l.add_button(
                x + PREV_OFFSET_X,
                y + PREV_OFFSET_Y,
                0x15E1,
                0x15E5,
                get_button_id(1, j),
            );

            x += PREV_WIDTH + OFFSET_SIZE;

            l.add_image_tiled(
                x,
                y,
                EMPTY_WIDTH - OFFSET_SIZE - INDENT_WIDTH,
                ENTRY_HEIGHT,
                ENTRY_GUMP_ID,
            );
            l.add_label(
                x + TEXT_OFFSET_X,
                y,
                TEXT_HUE,
                if sk.exists { &sk.name } else { "(null)" },
            );

            x += EMPTY_WIDTH - OFFSET_SIZE - INDENT_WIDTH;
            x += OFFSET_SIZE;

            if SET_GUMP_ID != 0 {
                l.add_image_tiled(x, y, SET_WIDTH, ENTRY_HEIGHT, SET_GUMP_ID);
            }

            if sk.exists {
                let (button_id1, button_id2, x_offset, y_offset) = match sk.lock {
                    SkillLock::Down => (0x985, 0x985, 6, 4),
                    SkillLock::Locked => (0x82C, 0x82C, 5, 2),
                    _ => (0x983, 0x983, 6, 4),
                };

                l.add_button(
                    x + x_offset,
                    y + y_offset,
                    button_id1,
                    button_id2,
                    get_button_id(2, j),
                );

                y += 1;
                x -= OFFSET_SIZE;
                x -= 1;
                x -= 50;

                l.add_image_tiled(x, y, 50, ENTRY_HEIGHT - 2, OFFSET_GUMP_ID);

                x += 1;
                y += 1;

                l.add_image_tiled(x, y, 48, ENTRY_HEIGHT - 4, ENTRY_GUMP_ID);

                l.add_label_cropped(
                    x + TEXT_OFFSET_X,
                    y - 1,
                    48 - TEXT_OFFSET_X,
                    ENTRY_HEIGHT - 3,
                    TEXT_HUE,
                    &format!("{:.1}", sk.base),
                );

                y -= 2;
            }
        }

        let group_skills = group.skills.len() as i32;
        l.add_image_tiled(
            BORDER_SIZE,
            indent_mask_y,
            INDENT_WIDTH + OFFSET_SIZE,
            group_skills * (ENTRY_HEIGHT + OFFSET_SIZE)
                - if i < groups.len() - 1 { OFFSET_SIZE } else { 0 },
            BACK_GUMP_ID + 4,
        );
    }
    g
}

impl LegacyGumpLogic for SkillsGump {
    const TYPE_NAME: &'static str = "Server.Gumps.SkillsGump";

    fn on_response(&mut self, w: &mut World, _sender: NetStateId, info: &RelayInfo) {
        let (from, target) = (self.from, self.target);
        let button_id = info.button_id - 1;

        let index = button_id / 3;
        let kind = button_id % 3;

        match kind {
            0 => {
                if index >= 0 && (index as usize) < self.groups.len() {
                    let new_selection = index as usize;

                    if self.selected != Some(new_selection) {
                        let g = new_skills(w, from, target, Some(new_selection));
                        send(w, from, g);
                    } else {
                        let g = new_skills(w, from, target, None);
                        send(w, from, g);
                    }
                }
            }
            1 => {
                if let Some(sel) = self.selected
                    && index >= 0
                    && (index as usize) < self.groups[sel].skills.len()
                {
                    let skill = self.groups[sel].skills[index as usize];

                    if skill_exists(w, skill) {
                        if muo_content::mx::access_level(w, from) >= AccessLevel::GameMaster {
                            let g = new_edit_skill(w, from, target, skill, self.selected);
                            send(w, from, g);
                        } else {
                            muo_content::mx::send_message(w, from, "You may not change that.");
                            let g = new_skills(w, from, target, self.selected);
                            send(w, from, g);
                        }
                    } else {
                        let g = new_skills(w, from, target, self.selected);
                        send(w, from, g);
                    }
                }
            }
            2 => {
                if let Some(sel) = self.selected
                    && index >= 0
                    && (index as usize) < self.groups[sel].skills.len()
                {
                    let skill = self.groups[sel].skills[index as usize];

                    if skill_exists(w, skill) {
                        if muo_content::mx::access_level(w, from) >= AccessLevel::GameMaster {
                            match muo_server::skills::lock(w, target, skill) {
                                SkillLock::Up => {
                                    muo_server::skills::set_lock_no_relay(
                                        w,
                                        target,
                                        skill,
                                        SkillLock::Down,
                                    );
                                    muo_server::skills::update(w, target, skill);
                                }
                                SkillLock::Down => {
                                    muo_server::skills::set_lock_no_relay(
                                        w,
                                        target,
                                        skill,
                                        SkillLock::Locked,
                                    );
                                    muo_server::skills::update(w, target, skill);
                                }
                                SkillLock::Locked => {
                                    muo_server::skills::set_lock_no_relay(
                                        w,
                                        target,
                                        skill,
                                        SkillLock::Up,
                                    );
                                    muo_server::skills::update(w, target, skill);
                                }
                            }
                        } else {
                            muo_content::mx::send_message(w, from, "You may not change that.");
                        }

                        let g = new_skills(w, from, target, self.selected);
                        send(w, from, g);
                    }
                }
            }
            _ => {}
        }
    }
}
