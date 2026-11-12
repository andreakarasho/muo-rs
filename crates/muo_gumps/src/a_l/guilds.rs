//! The guild gumps (the gumps, prompts and the recruit target; "New Guild System" is `new_guilds`).
//!
//! The lists the list gumps show are `_guild.WarInvitations` and the like: the gump keeps the live list (so the
//! response indexes the list as it is then), the member lists are copies. The names shown are read when the gump
//! is made, which is when it is sent.

use std::rc::Rc;

use muo_content::gumps::GumpButtonType;
use muo_content::gumps::base::{
    DynamicGumpLogic, DynamicGumpObj, RelayInfo, StaticGumpLogic, StaticGumpObj,
};
use muo_content::gumps::builder::DynamicGumpBuilder;
use muo_content::gumps::system::close_gump;
use muo_content::misc::guild_class as gc;
use muo_content::misc::guild_class::GuildData;
use muo_server::World;
use muo_server::guild::{self, GuildId, GuildType};
use muo_server::ids::MobileId;
use muo_server::mobile::AccessLevel;
use muo_server::net_state::NetStateId;
use muo_server::prompt::{Prompt, add_prompt};
use muo_server::target::{Target, TargetFlags, Targeted, add_target, set_mobile_target};

use super::common::{mobile_of, send};
use super::hooks::hooks;

// ---- shared ----

fn data(w: &World, g: GuildId) -> Option<&GuildData> {
    gc::data(w, g)
}

fn access(w: &World, m: MobileId) -> AccessLevel {
    muo_content::mx::access_level(w, m)
}

fn has_net_state(w: &World, m: MobileId) -> bool {
    w.mobile(m).and_then(|x| x.net_state).is_some()
}

fn leader_of(w: &World, g: GuildId) -> Option<MobileId> {
    data(w, g).and_then(|d| d.leader)
}

fn stone_out_of_reach(w: &World, m: MobileId, g: GuildId) -> bool {
    // stone?.Deleted != false || !m.InRange(stone.GetWorldLocation(), 2)
    match data(w, g).and_then(|d| d.guildstone) {
        Some(stone) if w.item(stone).is_some_and(|i| !i.deleted()) => {
            let p = muo_server::item::ops::get_world_location(w, stone);
            !muo_content::mx::in_range_point(w, m, p, 2)
        }
        _ => true,
    }
}

/// `GuildGump.BadLeader`.
pub fn bad_leader(w: &World, m: MobileId, g: GuildId) -> bool {
    if muo_content::mx::deleted(w, m)
        || gc::disbanded(w, g)
        || access(w, m) < AccessLevel::GameMaster && leader_of(w, g) != Some(m)
    {
        return true;
    }
    stone_out_of_reach(w, m, g)
}

/// `GuildGump.BadMember`.
pub fn bad_member(w: &World, m: MobileId, g: GuildId) -> bool {
    if muo_content::mx::deleted(w, m)
        || gc::disbanded(w, g)
        || access(w, m) < AccessLevel::GameMaster && !gc::is_member(w, g, m)
    {
        return true;
    }
    stone_out_of_reach(w, m, g)
}

/// `GuildGump.EnsureClosed`.
pub fn ensure_closed(w: &mut World, m: MobileId) {
    let Some(ns) = w.mobile(m).and_then(|x| x.net_state) else {
        return;
    };
    close_gump::<DeclareFealtyGump>(w, ns);
    close_gump::<GrantGuildTitleGump>(w, ns);
    close_gump::<GuildAdminCandidatesGump>(w, ns);
    close_gump::<GuildCandidatesGump>(w, ns);
    close_gump::<GuildChangeTypeGump>(w, ns);
    close_gump::<GuildCharterGump>(w, ns);
    close_gump::<GuildDismissGump>(w, ns);
    close_gump::<GuildGump>(w, ns);
    close_gump::<GuildmasterGump>(w, ns);
    close_gump::<GuildRosterGump>(w, ns);
    close_gump::<GuildWarGump>(w, ns);
}

/// The start of every `DisplayTo`: `from?.NetState == null || guild == null`, then `EnsureClosed`.
fn begin(w: &mut World, from: MobileId, g: GuildId) -> bool {
    if !has_net_state(w, from) || data(w, g).is_none() {
        return false;
    }
    ensure_closed(w, from);
    true
}

fn set_prompt(w: &mut World, from: MobileId, p: Prompt) {
    let id = add_prompt(w, p);
    muo_server::mobile::part1::props_b::set_prompt(w, from, Some(id));
}

fn guild_label(w: &World, g: GuildId) -> String {
    let abbr = guild::guild(w, g)
        .map(|b| b.abbreviation.clone())
        .unwrap_or_default();
    format!("{} ({abbr})", gc::name(w, g))
}

/// `GuildMessage(number, true, affix)`.
fn guild_message(w: &mut World, g: GuildId, number: i32, affix: &str) {
    gc::guild_message_affix(w, g, number, true, affix, "", 0x3B2);
}

/// `info.Switches[0]` where it indexes the list.
fn selected(info: &RelayInfo, len: usize) -> Option<usize> {
    let i = *info.switches.first()?;
    (i >= 0 && (i as usize) < len).then_some(i as usize)
}

fn trimmed_or_empty(name: Option<&str>) -> String {
    name.map_or_else(|| "(empty)".to_owned(), |n| n.trim().to_owned())
}

fn mobile_names(w: &World, list: &[MobileId]) -> Vec<String> {
    list.iter()
        .map(|m| trimmed_or_empty(w.mobile(*m).and_then(|x| (x.ty.name)(w, *m)).as_deref()))
        .collect()
}

fn guild_names(w: &World, list: &[GuildId]) -> Vec<String> {
    list.iter()
        .map(|g| trimmed_or_empty(guild::guild(w, *g).map(|b| b.name.as_str())))
        .collect()
}

/// The body of `GuildListGump.BuildLayout` / `GuildMobileListGump.BuildLayout`.
fn build_list(
    b: &mut DynamicGumpBuilder,
    radio: bool,
    names: &[String],
    header: fn(&mut DynamicGumpBuilder),
) {
    b.set_no_move();
    b.add_page(0);
    b.add_background(0, 0, 550, 440, 5054);
    b.add_background(10, 10, 530, 420, 3000);

    header(b);

    for (i, name) in names.iter().enumerate() {
        let i = i as i32;
        if i % 11 == 0 {
            if i != 0 {
                b.add_button_type(300, 370, 4005, 4007, 0, GumpButtonType::Page, i / 11 + 1);
                b.add_html_localized(335, 370, 300, 35, 1011066, false, false); // Next page
            }

            b.add_page(i / 11 + 1);

            if i != 0 {
                b.add_button_type(20, 370, 4014, 4016, 0, GumpButtonType::Page, i / 11);
                b.add_html_localized(55, 370, 300, 35, 1011067, false, false); // Previous page
            }
        }

        if radio {
            b.add_radio(20, 35 + i % 11 * 30, 208, 209, false, i);
        }

        b.add_label(if radio { 55 } else { 20 }, 35 + i % 11 * 30, 0, name);
    }
}

fn hl(b: &mut DynamicGumpBuilder, x: i32, y: i32, w: i32, h: i32, number: i32) {
    b.add_html_localized(x, y, w, h, number, false, false);
}

/// Declares a list gump: the struct, its `DynamicGumpLogic` and the constructor's snapshot.
macro_rules! list_gump {
    ($name:ident, $type_name:literal, $item:ty, $radio:expr, $header:expr) => {
        pub struct $name {
            guild: GuildId,
            // the live-list gumps index the guild's own list at response time
            #[allow(dead_code)]
            list: Vec<$item>,
            names: Vec<String>,
        }

        impl DynamicGumpLogic for $name {
            const TYPE_NAME: &'static str = $type_name;

            fn singleton(&self) -> bool {
                true
            }

            fn build_layout(&self, b: &mut DynamicGumpBuilder) {
                build_list(b, $radio, &self.names, $header);
            }

            fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
                if let Some(from) = mobile_of(w, sender) {
                    self.respond(w, from, info);
                }
            }
        }
    };
}

fn send_list<T: DynamicGumpLogic>(w: &mut World, from: MobileId, g: T) {
    send(w, from, DynamicGumpObj::new(20, 30, g));
}

// ---- DeclareFealtyGump ----

list_gump!(
    DeclareFealtyGump,
    "Server.Gumps.DeclareFealtyGump",
    MobileId,
    true,
    |b| {
        hl(b, 20, 10, 400, 35, 1011097); // Declare your fealty
        b.add_button(20, 400, 4005, 4007, 1);
        hl(b, 55, 400, 250, 35, 1011098); // I have selected my new lord.
        b.add_button(300, 400, 4005, 4007, 0);
        hl(b, 335, 400, 100, 35, 1011012); // CANCEL
    }
);

impl DeclareFealtyGump {
    pub fn display_to(w: &mut World, from: MobileId, g: GuildId) {
        if !begin(w, from, g) {
            return;
        }
        let list = data(w, g).map(|d| d.members.clone()).unwrap_or_default();
        let names = mobile_names(w, &list);
        send_list(
            w,
            from,
            Self {
                guild: g,
                list,
                names,
            },
        );
    }

    fn respond(&mut self, w: &mut World, from: MobileId, info: &RelayInfo) {
        if info.button_id == 0 || bad_member(w, from, self.guild) {
            return;
        }
        if info.button_id == 1
            && let Some(i) = selected(info, self.list.len())
        {
            let m = self.list[i];
            if !muo_content::mx::deleted(w, m)
                && let Some(x) = w.mobile_mut(from)
            {
                x.guild_fealty = Some(m);
            }
        }
        GuildGump::display_to(w, from, self.guild);
    }
}

// ---- GrantGuildTitleGump ----

list_gump!(
    GrantGuildTitleGump,
    "Server.Gumps.GrantGuildTitleGump",
    MobileId,
    true,
    |b| {
        hl(b, 20, 10, 400, 35, 1011118); // Grant a title to another member.
        b.add_button(20, 400, 4005, 4007, 1);
        hl(b, 55, 400, 245, 30, 1011127); // I dub thee...
        b.add_button(300, 400, 4005, 4007, 2);
        hl(b, 335, 400, 100, 35, 1011012); // CANCEL
    }
);

impl GrantGuildTitleGump {
    pub fn display_to(w: &mut World, from: MobileId, g: GuildId) {
        if !begin(w, from, g) {
            return;
        }
        let list = data(w, g).map(|d| d.members.clone()).unwrap_or_default();
        let names = mobile_names(w, &list);
        send_list(
            w,
            from,
            Self {
                guild: g,
                list,
                names,
            },
        );
    }

    fn respond(&mut self, w: &mut World, from: MobileId, info: &RelayInfo) {
        if info.button_id == 0 || bad_member(w, from, self.guild) {
            return;
        }
        if info.button_id == 1 {
            // GuildMobileListGump copies the list: the index means the list as shown.
            let list = self.list.clone();
            if let Some(i) = selected(info, list.len()) {
                let m = list[i];
                if !muo_content::mx::deleted(w, m) {
                    muo_content::mx::msg(w, from, 1013074); // New title (20 characters max):
                    guild_title_prompt(w, from, m, self.guild);
                }
            }
        } else if info.button_id == 2 {
            GuildmasterGump::display_to(w, from, self.guild);
        }
    }
}

// ---- GuildAbbrvPrompt ----

fn guild_abbrv_prompt(w: &mut World, from: MobileId, g: GuildId) {
    set_prompt(
        w,
        from,
        Prompt {
            on_cancel: Some(Rc::new(move |w: &mut World, from| {
                if !bad_leader(w, from, g) {
                    GuildmasterGump::display_to(w, from, g);
                }
            })),
            on_response: Some(Rc::new(move |w: &mut World, from, text: &str| {
                if bad_leader(w, from, g) {
                    return;
                }
                let mut s: String = text.trim().to_owned();
                if s.chars().count() > 3 {
                    s = s.chars().take(3).collect();
                }
                if !s.is_empty() {
                    if guild::world_find_by_abbrev(w, &s).is_some() {
                        muo_content::mx::send_message(
                            w,
                            from,
                            &format!("{s} conflicts with the abbreviation of an existing guild."),
                        );
                    } else {
                        gc::set_abbreviation(w, g, &s);
                        guild_message(w, g, 1018025, &s); // Your guild abbreviation has changed:
                    }
                }
                GuildmasterGump::display_to(w, from, g);
            })),
        },
    );
}

// ---- GuildAcceptWarGump ----

list_gump!(
    GuildAcceptWarGump,
    "Server.Gumps.GuildAcceptWarGump",
    GuildId,
    true,
    |b| {
        hl(b, 20, 10, 400, 35, 1011147); // Select the guild to accept the invitations:
        b.add_button(20, 400, 4005, 4007, 1);
        hl(b, 55, 400, 245, 30, 1011100); // Accept war invitations.
        b.add_button(300, 400, 4005, 4007, 2);
        hl(b, 335, 400, 100, 35, 1011012); // CANCEL
    }
);

impl GuildAcceptWarGump {
    pub fn display_to(w: &mut World, from: MobileId, g: GuildId) {
        if !begin(w, from, g) {
            return;
        }
        let list = data(w, g)
            .map(|d| d.war_invitations.clone())
            .unwrap_or_default();
        let names = guild_names(w, &list);
        send_list(
            w,
            from,
            Self {
                guild: g,
                list,
                names,
            },
        );
    }

    fn respond(&mut self, w: &mut World, from: MobileId, info: &RelayInfo) {
        let me = self.guild;
        if bad_leader(w, from, me) {
            return;
        }
        // the list is the guild's own: indexed as it is now
        let live = data(w, me)
            .map(|d| d.war_invitations.clone())
            .unwrap_or_default();
        if info.button_id == 1 {
            if let Some(i) = selected(info, live.len()) {
                let g = live[i];
                if let Some(d) = gc::data_mut(w, me) {
                    d.war_invitations.retain(|x| *x != g);
                }
                if let Some(d) = gc::data_mut(w, g) {
                    d.war_declarations.retain(|x| *x != me);
                }
                gc::add_enemy(w, me, g);
                let label = guild_label(w, g);
                guild_message(w, me, 1018020, &label);

                if data(w, me).is_some_and(|d| !d.war_invitations.is_empty()) {
                    Self::display_to(w, from, me);
                } else {
                    GuildmasterGump::display_to(w, from, me);
                }
            }
        } else if info.button_id == 2 {
            GuildmasterGump::display_to(w, from, me);
        }
    }
}

// ---- GuildAdminCandidatesGump ----

list_gump!(
    GuildAdminCandidatesGump,
    "Server.Gumps.GuildAdminCandidatesGump",
    MobileId,
    true,
    |b| {
        hl(b, 20, 10, 400, 35, 1013075); // Accept or Refuse candidates for membership
        b.add_button(20, 400, 4005, 4007, 1);
        hl(b, 55, 400, 245, 30, 1013076); // Accept
        b.add_button(300, 400, 4005, 4007, 2);
        hl(b, 335, 400, 100, 35, 1013077); // Refuse
    }
);

impl GuildAdminCandidatesGump {
    pub fn display_to(w: &mut World, from: MobileId, g: GuildId) {
        if !begin(w, from, g) {
            return;
        }
        let list = data(w, g).map(|d| d.candidates.clone()).unwrap_or_default();
        let names = mobile_names(w, &list);
        send_list(
            w,
            from,
            Self {
                guild: g,
                list,
                names,
            },
        );
    }

    fn respond(&mut self, w: &mut World, from: MobileId, info: &RelayInfo) {
        let g = self.guild;
        if bad_leader(w, from, g) {
            return;
        }
        // GuildMobileListGump copies the list (`new List<Mobile>(list)`): the index means the list as shown.
        let list = self.list.clone();
        match info.button_id {
            0 => GuildmasterGump::display_to(w, from, g),
            1 => {
                // Accept
                let Some(i) = selected(info, list.len()) else {
                    return;
                };
                let m = list[i];
                if muo_content::mx::deleted(w, m) {
                    return;
                }
                let guild_faction = leader_of(w, g).and_then(|l| faction_of(w, l));
                let target_faction = faction_of(w, m);
                if guild_faction != target_faction {
                    if guild_faction.is_none() {
                        muo_content::mx::msg(w, from, 1013027); // That player cannot join a non-faction guild.
                    } else if target_faction.is_none() {
                        muo_content::mx::msg(w, from, 1013026); // That player must be in a faction before joining this guild.
                    } else {
                        muo_content::mx::msg(w, from, 1013028); // That person has a different faction affiliation.
                    }
                    return;
                }
                if faction_leaving(w, m) {
                    // OSI does this quite strangely, so we'll just do it this way
                    muo_content::mx::send_message(
                        w,
                        from,
                        "That person is quitting their faction and so you may not recruit them.",
                    );
                    return;
                }
                if let Some(d) = gc::data_mut(w, g) {
                    d.candidates.retain(|x| *x != m);
                    d.accepted.push(m);
                }
                if data(w, g).is_some_and(|d| !d.candidates.is_empty()) {
                    Self::display_to(w, from, g);
                } else {
                    GuildmasterGump::display_to(w, from, g);
                }
            }
            2 => {
                // Refuse
                let Some(i) = selected(info, list.len()) else {
                    return;
                };
                let m = list[i];
                if muo_content::mx::deleted(w, m) {
                    return;
                }
                if let Some(d) = gc::data_mut(w, g) {
                    d.candidates.retain(|x| *x != m);
                }
                if data(w, g).is_some_and(|d| !d.candidates.is_empty()) {
                    Self::display_to(w, from, g);
                } else {
                    GuildmasterGump::display_to(w, from, g);
                }
            }
            _ => {}
        }
    }
}

/// `Faction.Find(m)` as an opaque id (`PlayerState.Find(m)?.Faction`).
fn faction_of(w: &World, m: MobileId) -> Option<i32> {
    muo_content::player_mobile::hooks::faction_of(w, m)
}

/// `PlayerState.Find(m)?.IsLeaving == true`. Unset: false.
fn faction_leaving(w: &World, m: MobileId) -> bool {
    hooks(w)
        .and_then(|h| h.faction_is_leaving)
        .is_some_and(|f| f(w, m))
}

// ---- GuildCandidatesGump ----

list_gump!(
    GuildCandidatesGump,
    "Server.Gumps.GuildCandidatesGump",
    MobileId,
    false,
    |b| {
        hl(b, 20, 10, 500, 35, 1013030); // <center> Candidates </center>
        b.add_button(20, 400, 4005, 4007, 1);
        hl(b, 55, 400, 300, 35, 1011120); // Return to the main menu.
    }
);

impl GuildCandidatesGump {
    pub fn display_to(w: &mut World, from: MobileId, g: GuildId) {
        if !begin(w, from, g) {
            return;
        }
        let list = data(w, g).map(|d| d.candidates.clone()).unwrap_or_default();
        let names = mobile_names(w, &list);
        send_list(
            w,
            from,
            Self {
                guild: g,
                list,
                names,
            },
        );
    }

    fn respond(&mut self, w: &mut World, from: MobileId, info: &RelayInfo) {
        if bad_member(w, from, self.guild) {
            return;
        }
        if info.button_id == 1 {
            GuildGump::display_to(w, from, self.guild);
        }
    }
}

// ---- GuildChangeTypeGump ----

pub struct GuildChangeTypeGump {
    guild: GuildId,
}

impl GuildChangeTypeGump {
    pub fn display_to(w: &mut World, from: MobileId, g: GuildId) {
        if !begin(w, from, g) {
            return;
        }
        send(w, from, StaticGumpObj::new(20, 30, Self { guild: g }));
    }
}

impl StaticGumpLogic for GuildChangeTypeGump {
    const TYPE_NAME: &'static str = "Server.Gumps.GuildChangeTypeGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut muo_content::gumps::StaticGumpBuilder) {
        b.set_no_move();
        b.add_page(0);
        b.add_background(0, 0, 550, 400, 5054);
        b.add_background(10, 10, 530, 380, 3000);
        b.add_html_localized(20, 15, 510, 30, 1013062, false, false); // <center>Change Guild Type Menu</center>
        b.add_html_localized(50, 50, 450, 30, 1013066, false, false); // Please select the type of guild you would like to change to
        b.add_button(20, 100, 4005, 4007, 1);
        b.add_html_localized(85, 100, 300, 30, 1013063, false, false); // Standard guild
        b.add_button(20, 150, 4005, 4007, 2);
        b.add_item(50, 143, 7109, 0);
        b.add_html_localized(85, 150, 300, 300, 1013064, false, false); // Order guild
        b.add_button(20, 200, 4005, 4007, 3);
        b.add_item(45, 200, 7107, 0);
        b.add_html_localized(85, 200, 300, 300, 1013065, false, false); // Chaos guild
        b.add_button(300, 360, 4005, 4007, 4);
        b.add_html_localized(335, 360, 150, 30, 1011012, false, false); // CANCEL
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let Some(from) = mobile_of(w, sender) else {
            return;
        };
        let g = self.guild;
        let new_system = muo_content::misc::guild::new_guild_system(w);
        if new_system && !super::new_guilds::is_leader(w, from, g)
            || !new_system && bad_leader(w, from, g)
        {
            return;
        }

        let current = gc::guild_type(w, g);
        let new_type = match info.button_id {
            1 => GuildType::Regular,
            2 => GuildType::Order,
            3 => GuildType::Chaos,
            _ => current,
        };

        if current != new_type {
            if faction_of(w, from).is_some() {
                muo_content::mx::msg(w, from, 1010405); // You cannot change guild types while in a Faction!
            } else if data(w, g).is_some_and(|d| d.type_last_change.add_days(7.0) > w.now) {
                muo_content::mx::msg(w, from, 1011142); // You have already changed your guild type recently.
            } else {
                gc::set_type(w, g, new_type);
                let name = match new_type {
                    GuildType::Regular => "Regular",
                    GuildType::Chaos => "Chaos",
                    GuildType::Order => "Order",
                };
                guild_message(w, g, 1018022, name); // Guild Message: Your guild type has changed:
            }
        }

        if new_system {
            if muo_content::mx::player(w, from) {
                super::new_guilds::open_info_gump(w, from, g);
            }
            return;
        }

        GuildmasterGump::display_to(w, from, g);
    }
}

// ---- GuildCharterGump ----

const DEFAULT_WEBSITE: &str = "";

pub struct GuildCharterGump {
    guild: GuildId,
    charter: Option<String>,
    website: Option<String>,
}

impl GuildCharterGump {
    pub fn display_to(w: &mut World, from: MobileId, g: GuildId) {
        if !begin(w, from, g) {
            return;
        }
        let (charter, website) = data(w, g)
            .map(|d| (d.charter.clone(), d.website.clone()))
            .unwrap_or_default();
        send(
            w,
            from,
            DynamicGumpObj::new(
                20,
                30,
                Self {
                    guild: g,
                    charter,
                    website,
                },
            ),
        );
    }
}

fn website_or_default(website: &Option<String>) -> String {
    let w = website.as_deref().map_or("", str::trim);
    if w.is_empty() {
        DEFAULT_WEBSITE.to_owned()
    } else {
        w.to_owned()
    }
}

impl DynamicGumpLogic for GuildCharterGump {
    const TYPE_NAME: &'static str = "Server.Gumps.GuildCharterGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        b.set_no_move();
        b.add_page(0);
        b.add_background(0, 0, 550, 400, 5054);
        b.add_background(10, 10, 530, 380, 3000);
        b.add_button(20, 360, 4005, 4007, 1);
        hl(b, 55, 360, 300, 35, 1011120); // Return to the main menu.

        let charter = self
            .charter
            .as_deref()
            .map(str::trim)
            .filter(|c| !c.is_empty());
        match charter {
            None => hl(b, 20, 20, 400, 35, 1013032), // No charter has been defined.
            Some(c) => b.add_html(20, 20, 510, 75, c, true, true),
        }

        b.add_button(20, 200, 4005, 4007, 2);
        hl(b, 55, 200, 300, 20, 1011122); // Visit the guild website :
        b.add_html(
            55,
            220,
            300,
            20,
            &website_or_default(&self.website),
            false,
            false,
        );
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let Some(from) = mobile_of(w, sender) else {
            return;
        };
        if info.button_id == 0 || bad_member(w, from, self.guild) {
            return;
        }
        if info.button_id == 2 {
            let site = website_or_default(&data(w, self.guild).and_then(|d| d.website.clone()));
            muo_server::mobile::part2::stats::launch_browser(w, from, &site);
        }
        GuildGump::display_to(w, from, self.guild);
    }
}

// ---- GuildCharterPrompt ----

fn guild_charter_prompt(w: &mut World, from: MobileId, g: GuildId) {
    set_prompt(
        w,
        from,
        Prompt {
            on_cancel: Some(Rc::new(move |w: &mut World, from| {
                if !bad_leader(w, from, g) {
                    GuildmasterGump::display_to(w, from, g);
                }
            })),
            on_response: Some(Rc::new(move |w: &mut World, from, text: &str| {
                if bad_leader(w, from, g) {
                    return;
                }
                let s: String = text.trim().chars().take(50).collect();
                if !s.is_empty()
                    && let Some(d) = gc::data_mut(w, g)
                {
                    d.charter = Some(s);
                }
                muo_content::mx::msg(w, from, 1013072); // Enter the new website for the guild (50 characters max):
                guild_website_prompt(w, from, g);
                GuildmasterGump::display_to(w, from, g);
            })),
        },
    );
}

// ---- GuildDeclarePeaceGump ----

list_gump!(
    GuildDeclarePeaceGump,
    "Server.Gumps.GuildDeclarePeaceGump",
    GuildId,
    true,
    |b| {
        hl(b, 20, 10, 400, 35, 1011137); // Select the guild you wish to declare peace with.
        b.add_button(20, 400, 4005, 4007, 1);
        hl(b, 55, 400, 245, 30, 1011138); // Send the olive branch.
        b.add_button(300, 400, 4005, 4007, 2);
        hl(b, 335, 400, 100, 35, 1011012); // CANCEL
    }
);

impl GuildDeclarePeaceGump {
    pub fn display_to(w: &mut World, from: MobileId, g: GuildId) {
        if !begin(w, from, g) {
            return;
        }
        let list = data(w, g).map(|d| d.enemies.clone()).unwrap_or_default();
        let names = guild_names(w, &list);
        send_list(
            w,
            from,
            Self {
                guild: g,
                list,
                names,
            },
        );
    }

    fn respond(&mut self, w: &mut World, from: MobileId, info: &RelayInfo) {
        let me = self.guild;
        if bad_leader(w, from, me) {
            return;
        }
        let live = data(w, me).map(|d| d.enemies.clone()).unwrap_or_default();
        if info.button_id == 1 {
            if let Some(i) = selected(info, live.len()) {
                let g = live[i];
                gc::remove_enemy(w, me, g);
                let label = guild_label(w, g);
                guild_message(w, me, 1018018, &label); // Guild Message: You are now at peace with this guild:
                if data(w, me).is_some_and(|d| !d.enemies.is_empty()) {
                    Self::display_to(w, from, me);
                } else {
                    GuildmasterGump::display_to(w, from, me);
                }
            }
        } else if info.button_id == 2 {
            GuildmasterGump::display_to(w, from, me);
        }
    }
}

// ---- GuildDeclareWarGump ----

list_gump!(
    GuildDeclareWarGump,
    "Server.Gumps.GuildDeclareWarGump",
    GuildId,
    true,
    |b| {
        hl(b, 20, 10, 400, 35, 1011065); // Select the guild you wish to declare war on.
        b.add_button(20, 400, 4005, 4007, 1);
        hl(b, 55, 400, 245, 30, 1011068); // Send the challenge!
        b.add_button(300, 400, 4005, 4007, 2);
        hl(b, 335, 400, 100, 35, 1011012); // CANCEL
    }
);

impl GuildDeclareWarGump {
    pub fn display_to(w: &mut World, from: MobileId, g: GuildId, list: Vec<GuildId>) {
        if !has_net_state(w, from) || data(w, g).is_none() || list.is_empty() {
            return;
        }
        ensure_closed(w, from);
        let names = guild_names(w, &list);
        send_list(
            w,
            from,
            Self {
                guild: g,
                list,
                names,
            },
        );
    }

    fn respond(&mut self, w: &mut World, from: MobileId, info: &RelayInfo) {
        let me = self.guild;
        if bad_leader(w, from, me) {
            return;
        }
        if info.button_id == 1 {
            if let Some(i) = selected(info, self.list.len()) {
                let g = self.list[i];
                let invited_us = data(w, g).is_some_and(|d| d.war_invitations.contains(&me))
                    && data(w, me).is_some_and(|d| d.war_declarations.contains(&g));
                if g == me {
                    muo_content::mx::msg(w, from, 501184); // You cannot declare war against yourself!
                } else if invited_us || gc::is_war(w, me, Some(g)) {
                    muo_content::mx::msg(w, from, 501183); // You are already at war with that guild.
                } else if leader_of(w, me).and_then(|l| faction_of(w, l)).is_some() {
                    muo_content::mx::msg(w, from, 1005288); // You cannot declare war while you are in a faction
                } else {
                    if data(w, me).is_some_and(|d| !d.war_declarations.contains(&g)) {
                        if let Some(d) = gc::data_mut(w, me) {
                            d.war_declarations.push(g);
                        }
                        let label = guild_label(w, g);
                        guild_message(w, me, 1018019, &label);
                    }
                    if data(w, g).is_some_and(|d| !d.war_invitations.contains(&me)) {
                        if let Some(d) = gc::data_mut(w, g) {
                            d.war_invitations.push(me);
                        }
                        let label = guild_label(w, me);
                        guild_message(w, g, 1018021, &label);
                    }
                }
                GuildWarAdminGump::display_to(w, from, me);
            }
        } else if info.button_id == 2 {
            GuildmasterGump::display_to(w, from, me);
        }
    }
}

// ---- GuildDeclareWarPrompt ----

fn guild_declare_war_prompt(w: &mut World, from: MobileId, g: GuildId) {
    set_prompt(
        w,
        from,
        Prompt {
            on_cancel: Some(Rc::new(move |w: &mut World, from| {
                if !bad_leader(w, from, g) {
                    GuildWarAdminGump::display_to(w, from, g);
                }
            })),
            on_response: Some(Rc::new(move |w: &mut World, from, text: &str| {
                if bad_leader(w, from, g) {
                    return;
                }
                let text = text.trim();
                if text.chars().count() >= 3 {
                    let ids = guild::guild_ids(w);
                    let found = guild::search(ids.iter().filter_map(|i| guild::guild(w, *i)), text);
                    // SafeConvertList<BaseGuild, Guild>
                    let guilds: Vec<GuildId> = found
                        .into_iter()
                        .filter(|i| data(w, *i).is_some())
                        .collect();
                    if !guilds.is_empty() {
                        GuildDeclareWarGump::display_to(w, from, g, guilds);
                    } else {
                        GuildWarAdminGump::display_to(w, from, g);
                        muo_content::mx::msg(w, from, 1018003); // No guilds found matching - try another name in the search
                    }
                } else {
                    muo_content::mx::send_message(
                        w,
                        from,
                        "Search string must be at least three letters in length.",
                    );
                }
            })),
        },
    );
}

// ---- GuildDismissGump ----

list_gump!(
    GuildDismissGump,
    "Server.Gumps.GuildDismissGump",
    MobileId,
    true,
    |b| {
        hl(b, 20, 10, 400, 35, 1011124); // Whom do you wish to dismiss?
        b.add_button(20, 400, 4005, 4007, 1);
        hl(b, 55, 400, 245, 30, 1011125); // Kick them out!
        b.add_button(300, 400, 4005, 4007, 2);
        hl(b, 335, 400, 100, 35, 1011012); // CANCEL
    }
);

impl GuildDismissGump {
    pub fn display_to(w: &mut World, from: MobileId, g: GuildId) {
        if !begin(w, from, g) {
            return;
        }
        let list = data(w, g).map(|d| d.members.clone()).unwrap_or_default();
        let names = mobile_names(w, &list);
        send_list(
            w,
            from,
            Self {
                guild: g,
                list,
                names,
            },
        );
    }

    fn respond(&mut self, w: &mut World, from: MobileId, info: &RelayInfo) {
        let g = self.guild;
        if bad_leader(w, from, g) {
            return;
        }
        let can_admin =
            |w: &World| access(w, from) >= AccessLevel::GameMaster || leader_of(w, g) == Some(from);
        if info.button_id == 1 {
            if let Some(i) = selected(info, self.list.len()) {
                let m = self.list[i];
                if !muo_content::mx::deleted(w, m) {
                    gc::remove_member(w, g, m, 1018028);
                    if can_admin(w) {
                        GuildmasterGump::display_to(w, from, g);
                    }
                }
            }
        } else if info.button_id == 2 && can_admin(w) {
            GuildmasterGump::display_to(w, from, g);
        }
    }
}

// ---- GuildGump ----

pub struct GuildGump {
    guild: GuildId,
    mobile: MobileId,
    guild_name: String,
    /// "{title}: {name}" or the name, of the leader.
    leader_text: Option<String>,
    /// None is `yourself`.
    fealty_name: Option<String>,
    display_title: bool,
    candidates: bool,
    gm_access: bool,
}

impl GuildGump {
    pub fn display_to(w: &mut World, from: MobileId, g: GuildId) {
        if !begin(w, from, g) {
            return;
        }
        let leader = leader_of(w, g);
        let leader_text = leader.map(|l| {
            let m = w.mobile(l);
            let title = m
                .and_then(|x| x.guild_title.as_deref())
                .map_or("", str::trim);
            let name = m.and_then(|x| (x.ty.name)(w, l)).unwrap_or_default();
            let name = name.trim();
            if title.is_empty() {
                name.to_owned()
            } else {
                format!("{title}: {name}")
            }
        });
        let mut fealty = w.mobile(from).and_then(|m| m.guild_fealty);
        if fealty.is_none_or(|f| !gc::is_member(w, g, f)) {
            fealty = leader;
        }
        let fealty = fealty.unwrap_or(from);
        let fealty_name = (from != fealty).then(|| {
            trimmed_or_empty(
                w.mobile(fealty)
                    .and_then(|x| (x.ty.name)(w, fealty))
                    .as_deref(),
            )
        });
        let gump = Self {
            guild: g,
            mobile: from,
            guild_name: gc::name(w, g),
            leader_text,
            fealty_name,
            display_title: w.mobile(from).is_some_and(|m| m.display_guild_title),
            candidates: data(w, g).is_some_and(|d| !d.candidates.is_empty()),
            gm_access: access(w, from) >= AccessLevel::GameMaster || leader == Some(from),
        };
        send(w, from, DynamicGumpObj::new(20, 30, gump));
    }
}

impl DynamicGumpLogic for GuildGump {
    const TYPE_NAME: &'static str = "Server.Gumps.GuildGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        b.set_no_move();
        b.add_page(0);
        b.add_background(0, 0, 550, 400, 5054);
        b.add_background(10, 10, 530, 380, 3000);

        b.add_html(20, 15, 200, 35, &self.guild_name, false, false);

        if let Some(text) = &self.leader_text {
            b.add_html(220, 15, 250, 35, text, false, false);
        }

        b.add_button(20, 50, 4005, 4007, 1);
        hl(b, 55, 50, 100, 20, 1013022); // Loyal to

        match &self.fealty_name {
            None => hl(b, 55, 70, 470, 20, 1018002), // yourself
            Some(n) => b.add_html(55, 70, 470, 20, n, false, false),
        }

        b.add_button(215, 50, 4005, 4007, 2);
        hl(b, 250, 50, 170, 20, 1013023); // Display guild abbreviation
        hl(
            b,
            250,
            70,
            50,
            20,
            if self.display_title { 1011262 } else { 1011263 },
        ); // on/off

        b.add_button(20, 100, 4005, 4007, 3);
        hl(b, 55, 100, 470, 30, 1011086); // View the current roster.

        b.add_button(20, 130, 4005, 4007, 4);
        hl(b, 55, 130, 470, 30, 1011085); // Recruit someone into the guild.

        if self.candidates {
            b.add_button(20, 160, 4005, 4007, 5);
            hl(b, 55, 160, 470, 30, 1011093); // View list of candidates who have been sponsored to the guild.
        } else {
            b.add_image(20, 160, 4020, 0);
            hl(b, 55, 160, 470, 30, 1013031); // There are currently no candidates for membership.
        }

        b.add_button(20, 220, 4005, 4007, 6);
        hl(b, 55, 220, 470, 30, 1011087); // View the guild's charter.

        b.add_button(20, 250, 4005, 4007, 7);
        hl(b, 55, 250, 470, 30, 1011092); // Resign from the guild.

        b.add_button(20, 280, 4005, 4007, 8);
        hl(b, 55, 280, 470, 30, 1011095); // View list of guilds you are at war with.

        if self.gm_access {
            b.add_button(20, 310, 4005, 4007, 9);
            hl(b, 55, 310, 470, 30, 1011094); // Access guildmaster functions.
        } else {
            b.add_image(20, 310, 4020, 0);
            hl(b, 55, 310, 470, 30, 1018013); // Reserved for guildmaster
        }

        b.add_button(20, 360, 4005, 4007, 0);
        hl(b, 55, 360, 470, 30, 1011441); // EXIT
    }

    fn on_response(&mut self, w: &mut World, _sender: NetStateId, info: &RelayInfo) {
        let (m, g) = (self.mobile, self.guild);
        if bad_member(w, m, g) {
            return;
        }
        match info.button_id {
            1 => DeclareFealtyGump::display_to(w, m, g),
            2 => {
                let on = w.mobile(m).is_some_and(|x| x.display_guild_title);
                muo_server::mobile::part1::props_b::set_display_guild_title(w, m, !on);
                Self::display_to(w, m, g);
            }
            3 => GuildRosterGump::display_to(w, m, g),
            4 => guild_recruit_target(w, m, g),
            5 => GuildCandidatesGump::display_to(w, m, g),
            6 => GuildCharterGump::display_to(w, m, g),
            7 => gc::remove_member(w, g, m, 1018028),
            8 => GuildWarGump::display_to(w, m, g),
            9 if (access(w, m) >= AccessLevel::GameMaster || leader_of(w, g) == Some(m)) => {
                GuildmasterGump::display_to(w, m, g);
            }
            _ => {}
        }
    }
}

// ---- GuildmasterGump ----

pub struct GuildmasterGump {
    guild: GuildId,
    order_chaos: bool,
    ty: GuildType,
    candidates: bool,
}

impl GuildmasterGump {
    pub fn display_to(w: &mut World, from: MobileId, g: GuildId) {
        if !begin(w, from, g) {
            return;
        }
        let gump = Self {
            guild: g,
            order_chaos: gc::order_chaos(w),
            ty: gc::guild_type(w, g),
            candidates: data(w, g).is_some_and(|d| !d.candidates.is_empty()),
        };
        send(w, from, DynamicGumpObj::new(20, 30, gump));
    }
}

impl DynamicGumpLogic for GuildmasterGump {
    const TYPE_NAME: &'static str = "Server.Gumps.GuildmasterGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        b.set_no_move();
        b.add_page(0);
        b.add_background(0, 0, 550, 400, 5054);
        b.add_background(10, 10, 530, 380, 3000);

        hl(b, 20, 15, 510, 35, 1011121); // <center>GUILDMASTER FUNCTIONS</center>

        b.add_button(20, 40, 4005, 4007, 2);
        hl(b, 55, 40, 470, 30, 1011107); // Set the guild name.

        b.add_button(20, 70, 4005, 4007, 3);
        hl(b, 55, 70, 470, 30, 1011109); // Set the guild's abbreviation.

        if self.order_chaos {
            b.add_button(20, 100, 4005, 4007, 4);
            match self.ty {
                GuildType::Regular => hl(b, 55, 100, 470, 30, 1013059), // Change guild type: Currently Standard
                GuildType::Order => hl(b, 55, 100, 470, 30, 1013057), // Change guild type: Currently Order
                GuildType::Chaos => hl(b, 55, 100, 470, 30, 1013058), // Change guild type: Currently Chaos
            }
        }

        b.add_button(20, 130, 4005, 4007, 5);
        hl(b, 55, 130, 470, 30, 1011112); // Set the guild's charter.

        b.add_button(20, 160, 4005, 4007, 6);
        hl(b, 55, 160, 470, 30, 1011113); // Dismiss a member.

        b.add_button(20, 190, 4005, 4007, 7);
        hl(b, 55, 190, 470, 30, 1011114); // Go to the WAR menu.

        if self.candidates {
            b.add_button(20, 220, 4005, 4007, 8);
            hl(b, 55, 220, 470, 30, 1013056); // Administer the list of candidates
        } else {
            b.add_image(20, 220, 4020, 0);
            hl(b, 55, 220, 470, 30, 1013031); // There are currently no candidates for membership.
        }

        b.add_button(20, 250, 4005, 4007, 9);
        hl(b, 55, 250, 470, 30, 1011117); // Set the guildmaster's title.

        b.add_button(20, 280, 4005, 4007, 10);
        hl(b, 55, 280, 470, 30, 1011118); // Grant a title to another member.

        b.add_button(20, 310, 4005, 4007, 11);
        hl(b, 55, 310, 470, 30, 1011119); // Move this guildstone.

        b.add_button(20, 360, 4005, 4007, 1);
        hl(b, 55, 360, 245, 30, 1011120); // Return to the main menu.

        b.add_button(300, 360, 4005, 4007, 0);
        hl(b, 335, 360, 100, 30, 1011441); // EXIT
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let Some(from) = mobile_of(w, sender) else {
            return;
        };
        let g = self.guild;
        if bad_leader(w, from, g) {
            return;
        }
        match info.button_id {
            1 => GuildGump::display_to(w, from, g),
            2 => {
                muo_content::mx::msg(w, from, 1013060); // Enter new guild name (40 characters max):
                guild_name_prompt(w, from, g);
            }
            3 => {
                muo_content::mx::msg(w, from, 1013061); // Enter new guild abbreviation (3 characters max):
                guild_abbrv_prompt(w, from, g);
            }
            4 => {
                if !gc::order_chaos(w) {
                    return;
                }
                GuildChangeTypeGump::display_to(w, from, g);
            }
            5 => {
                muo_content::mx::msg(w, from, 1013071); // Enter the new guild charter (50 characters max):
                guild_charter_prompt(w, from, g);
            }
            6 => GuildDismissGump::display_to(w, from, g),
            7 => GuildWarAdminGump::display_to(w, from, g),
            8 => GuildAdminCandidatesGump::display_to(w, from, g),
            9 => {
                muo_content::mx::msg(w, from, 1013073); // Enter new guildmaster title (20 characters max):
                guild_title_prompt(w, from, from, g);
            }
            10 => GrantGuildTitleGump::display_to(w, from, g),
            11 => {
                // Move guildstone
                if let Some(stone) = data(w, g).and_then(|d| d.guildstone)
                    && let Some(f) = hooks(w).and_then(|h| h.new_guild_teleporter)
                {
                    let old = data(w, g).and_then(|d| d.teleporter);
                    if let Some(item) = f(w, stone) {
                        if let Some(old) = old {
                            muo_server::item::base::delete(w, old);
                        }
                        // Use the teleporting object placed in your backpack to move this guildstone.
                        muo_content::mx::msg(w, from, 501133);
                        muo_server::item::deps::add_to_backpack(w, from, item);
                        if let Some(d) = gc::data_mut(w, g) {
                            d.teleporter = Some(item);
                        }
                    }
                }
                Self::display_to(w, from, g);
            }
            _ => {}
        }
    }
}

// ---- GuildNamePrompt ----

fn guild_name_prompt(w: &mut World, from: MobileId, g: GuildId) {
    set_prompt(
        w,
        from,
        Prompt {
            on_cancel: Some(Rc::new(move |w: &mut World, from| {
                if !bad_leader(w, from, g) {
                    GuildmasterGump::display_to(w, from, g);
                }
            })),
            on_response: Some(Rc::new(move |w: &mut World, from, text: &str| {
                if bad_leader(w, from, g) {
                    return;
                }
                let s: String = text.trim().chars().take(40).collect();
                if !s.is_empty() {
                    if guild::world_find_by_name(w, &s).is_some() {
                        muo_content::mx::send_message(
                            w,
                            from,
                            &format!("{s} conflicts with the name of an existing guild."),
                        );
                    } else {
                        gc::set_name(w, g, &s);
                        guild_message(w, g, 1018024, &s); // The name of your guild has changed:
                    }
                }
                GuildmasterGump::display_to(w, from, g);
            })),
        },
    );
}

// ---- GuildRejectWarGump ----

list_gump!(
    GuildRejectWarGump,
    "Server.Gumps.GuildRejectWarGump",
    GuildId,
    true,
    |b| {
        hl(b, 20, 10, 400, 35, 1011148); // Select the guild to reject their invitations:
        b.add_button(20, 400, 4005, 4007, 1);
        hl(b, 55, 400, 245, 30, 1011101); // Reject war invitations.
        b.add_button(300, 400, 4005, 4007, 2);
        hl(b, 335, 400, 100, 35, 1011012); // CANCEL
    }
);

impl GuildRejectWarGump {
    pub fn display_to(w: &mut World, from: MobileId, g: GuildId) {
        if !begin(w, from, g) {
            return;
        }
        let list = data(w, g)
            .map(|d| d.war_invitations.clone())
            .unwrap_or_default();
        let names = guild_names(w, &list);
        send_list(
            w,
            from,
            Self {
                guild: g,
                list,
                names,
            },
        );
    }

    fn respond(&mut self, w: &mut World, from: MobileId, info: &RelayInfo) {
        let me = self.guild;
        if bad_leader(w, from, me) {
            return;
        }
        let live = data(w, me)
            .map(|d| d.war_invitations.clone())
            .unwrap_or_default();
        if info.button_id == 1 {
            if let Some(i) = selected(info, live.len()) {
                let g = live[i];
                if let Some(d) = gc::data_mut(w, me) {
                    d.war_invitations.retain(|x| *x != g);
                }
                if let Some(d) = gc::data_mut(w, g) {
                    d.war_declarations.retain(|x| *x != me);
                }
                if data(w, me).is_some_and(|d| !d.war_invitations.is_empty()) {
                    Self::display_to(w, from, me);
                } else {
                    GuildmasterGump::display_to(w, from, me);
                }
            }
        } else if info.button_id == 2 {
            GuildmasterGump::display_to(w, from, me);
        }
    }
}

// ---- GuildRescindDeclarationGump ----

list_gump!(
    GuildRescindDeclarationGump,
    "Server.Gumps.GuildRescindDeclarationGump",
    GuildId,
    true,
    |b| {
        hl(b, 20, 10, 400, 35, 1011150); // Select the guild to rescind our invitations:
        b.add_button(20, 400, 4005, 4007, 1);
        hl(b, 55, 400, 245, 30, 1011102); // Rescind your war declarations.
        b.add_button(300, 400, 4005, 4007, 2);
        hl(b, 335, 400, 100, 35, 1011012); // CANCEL
    }
);

impl GuildRescindDeclarationGump {
    pub fn display_to(w: &mut World, from: MobileId, g: GuildId) {
        if !begin(w, from, g) {
            return;
        }
        let list = data(w, g)
            .map(|d| d.war_declarations.clone())
            .unwrap_or_default();
        let names = guild_names(w, &list);
        send_list(
            w,
            from,
            Self {
                guild: g,
                list,
                names,
            },
        );
    }

    fn respond(&mut self, w: &mut World, from: MobileId, info: &RelayInfo) {
        let me = self.guild;
        if bad_leader(w, from, me) {
            return;
        }
        let live = data(w, me)
            .map(|d| d.war_declarations.clone())
            .unwrap_or_default();
        if info.button_id == 1 {
            if let Some(i) = selected(info, live.len()) {
                let g = live[i];
                if let Some(d) = gc::data_mut(w, me) {
                    d.war_declarations.retain(|x| *x != g);
                }
                if let Some(d) = gc::data_mut(w, g) {
                    d.war_invitations.retain(|x| *x != me);
                }
                if data(w, me).is_some_and(|d| !d.war_declarations.is_empty()) {
                    Self::display_to(w, from, me);
                } else {
                    GuildmasterGump::display_to(w, from, me);
                }
            }
        } else if info.button_id == 2 {
            GuildmasterGump::display_to(w, from, me);
        }
    }
}

// ---- GuildRosterGump ----

pub struct GuildRosterGump {
    guild: GuildId,
    guild_name: String,
    names: Vec<String>,
}

impl GuildRosterGump {
    pub fn display_to(w: &mut World, from: MobileId, g: GuildId) {
        if !begin(w, from, g) {
            return;
        }
        let list = data(w, g).map(|d| d.members.clone()).unwrap_or_default();
        let names = mobile_names(w, &list);
        send_list(
            w,
            from,
            Self {
                guild: g,
                guild_name: gc::name(w, g),
                names,
            },
        );
    }
}

impl DynamicGumpLogic for GuildRosterGump {
    const TYPE_NAME: &'static str = "Server.Gumps.GuildRosterGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        // the header needs the guild name, which a fn pointer cannot capture: build the same layout inline
        b.set_no_move();
        b.add_page(0);
        b.add_background(0, 0, 550, 440, 5054);
        b.add_background(10, 10, 530, 420, 3000);

        b.add_html(
            20,
            10,
            500,
            35,
            &format!("<center>{}</center>", self.guild_name),
            false,
            false,
        );
        b.add_button(20, 400, 4005, 4007, 1);
        hl(b, 55, 400, 300, 35, 1011120); // Return to the main menu.

        build_list_rows(b, false, &self.names);
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let Some(from) = mobile_of(w, sender) else {
            return;
        };
        if bad_member(w, from, self.guild) {
            return;
        }
        if info.button_id == 1 {
            GuildGump::display_to(w, from, self.guild);
        }
    }
}

/// The rows of a list gump after its header.
fn build_list_rows(b: &mut DynamicGumpBuilder, radio: bool, names: &[String]) {
    for (i, name) in names.iter().enumerate() {
        let i = i as i32;
        if i % 11 == 0 {
            if i != 0 {
                b.add_button_type(300, 370, 4005, 4007, 0, GumpButtonType::Page, i / 11 + 1);
                b.add_html_localized(335, 370, 300, 35, 1011066, false, false); // Next page
            }
            b.add_page(i / 11 + 1);
            if i != 0 {
                b.add_button_type(20, 370, 4014, 4016, 0, GumpButtonType::Page, i / 11);
                b.add_html_localized(55, 370, 300, 35, 1011067, false, false); // Previous page
            }
        }
        if radio {
            b.add_radio(20, 35 + i % 11 * 30, 208, 209, false, i);
        }
        b.add_label(if radio { 55 } else { 20 }, 35 + i % 11 * 30, 0, name);
    }
}

// ---- GuildTitlePrompt ----

fn guild_title_prompt(w: &mut World, from: MobileId, target: MobileId, g: GuildId) {
    set_prompt(
        w,
        from,
        Prompt {
            on_cancel: Some(Rc::new(move |w: &mut World, from| {
                if bad_leader(w, from, g) {
                    return;
                }
                if muo_content::mx::deleted(w, target) || !gc::is_member(w, g, target) {
                    return;
                }
                GuildmasterGump::display_to(w, from, g);
            })),
            on_response: Some(Rc::new(move |w: &mut World, from, text: &str| {
                if bad_leader(w, from, g) {
                    return;
                }
                if muo_content::mx::deleted(w, target) || !gc::is_member(w, g, target) {
                    return;
                }
                let s: String = text.trim().chars().take(20).collect();
                if !s.is_empty() {
                    muo_server::mobile::part1::props_b::set_guild_title(w, target, Some(s));
                }
                GuildmasterGump::display_to(w, from, g);
            })),
        },
    );
}

// ---- GuildWarAdminGump ----

pub struct GuildWarAdminGump {
    guild: GuildId,
    enemies: bool,
    invitations: bool,
    declarations: bool,
}

impl GuildWarAdminGump {
    pub fn display_to(w: &mut World, from: MobileId, g: GuildId) {
        if !begin(w, from, g) {
            return;
        }
        let d = data(w, g);
        let gump = Self {
            guild: g,
            enemies: d.is_some_and(|d| !d.enemies.is_empty()),
            invitations: d.is_some_and(|d| !d.war_invitations.is_empty()),
            declarations: d.is_some_and(|d| !d.war_declarations.is_empty()),
        };
        send(w, from, DynamicGumpObj::new(20, 30, gump));
    }
}

impl DynamicGumpLogic for GuildWarAdminGump {
    const TYPE_NAME: &'static str = "Server.Gumps.GuildWarAdminGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        b.set_no_move();
        b.add_page(0);
        b.add_background(0, 0, 550, 440, 5054);
        b.add_background(10, 10, 530, 420, 3000);

        hl(b, 20, 10, 510, 35, 1011105); // <center>WAR FUNCTIONS</center>

        b.add_button(20, 40, 4005, 4007, 1);
        hl(b, 55, 40, 400, 30, 1011099); // Declare war through guild name search.

        if self.enemies {
            b.add_button(20, 160 + 30, 4005, 4007, 2);
            hl(b, 55, 160, 400, 30, 1011103); // Declare peace.
        } else {
            hl(b, 20, 160, 400, 30, 1013033); // No current wars
        }

        if self.invitations {
            b.add_button(20, 190, 4005, 4007, 3);
            hl(b, 55, 190, 400, 30, 1011100); // Accept war invitations.
            b.add_button(20, 190, 4005, 4007, 4);
            hl(b, 55, 190, 400, 30, 1011101); // Reject war invitations.
        } else {
            hl(b, 20, 190, 400, 30, 1018012); // No current invitations received for war.
        }

        if self.declarations {
            b.add_button(20, 220, 4005, 4007, 5);
            hl(b, 55, 220, 400, 30, 1011102); // Rescind your war declarations.
        } else {
            hl(b, 20, 220, 400, 30, 1013055); // No current war declarations
        }

        b.add_button(20, 400, 4005, 4007, 6);
        hl(b, 55, 400, 400, 35, 1011104); // Return to the previous menu.
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let Some(from) = mobile_of(w, sender) else {
            return;
        };
        let g = self.guild;
        if bad_leader(w, from, g) {
            return;
        }
        match info.button_id {
            1 => {
                muo_content::mx::msg(w, from, 1018001); // Declare war through search - Enter Guild Name:
                guild_declare_war_prompt(w, from, g);
            }
            2 => GuildDeclarePeaceGump::display_to(w, from, g),
            3 => GuildAcceptWarGump::display_to(w, from, g),
            4 => GuildRejectWarGump::display_to(w, from, g),
            5 => GuildRescindDeclarationGump::display_to(w, from, g),
            6 => GuildmasterGump::display_to(w, from, g),
            _ => {}
        }
    }
}

// ---- GuildWarGump ----

pub struct GuildWarGump {
    guild: GuildId,
    enemies: Vec<String>,
    declared: Vec<String>,
    invites: Vec<String>,
}

impl GuildWarGump {
    pub fn display_to(w: &mut World, from: MobileId, g: GuildId) {
        if !begin(w, from, g) {
            return;
        }
        let names = |w: &World, f: fn(&GuildData) -> &Vec<GuildId>| -> Vec<String> {
            data(w, g)
                .map(|d| f(d).iter().map(|x| gc::name(w, *x)).collect())
                .unwrap_or_default()
        };
        let gump = Self {
            guild: g,
            enemies: names(w, |d| &d.enemies),
            declared: names(w, |d| &d.war_declarations),
            invites: names(w, |d| &d.war_invitations),
        };
        send(w, from, DynamicGumpObj::new(20, 30, gump));
    }
}

impl DynamicGumpLogic for GuildWarGump {
    const TYPE_NAME: &'static str = "Server.Gumps.GuildWarGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        b.set_no_move();
        b.add_page(0);
        b.add_background(0, 0, 550, 440, 5054);
        b.add_background(10, 10, 530, 420, 3000);

        hl(b, 20, 10, 500, 35, 1011133); // <center>WARFARE STATUS</center>

        b.add_button(20, 400, 4005, 4007, 1);
        hl(b, 55, 400, 300, 35, 1011120); // Return to the main menu.

        b.add_page(1);

        b.add_button_type(375, 375, 5224, 5224, 0, GumpButtonType::Page, 2);
        hl(b, 410, 373, 100, 25, 1011066); // Next page

        hl(b, 20, 45, 400, 20, 1011134); // We are at war with:

        list_or(b, &self.enemies, 1013033); // No current wars

        b.add_page(2);

        b.add_button_type(375, 375, 5224, 5224, 0, GumpButtonType::Page, 3);
        hl(b, 410, 373, 100, 25, 1011066); // Next page

        b.add_button_type(30, 375, 5223, 5223, 0, GumpButtonType::Page, 1);
        hl(b, 65, 373, 150, 25, 1011067); // Previous page

        hl(b, 20, 45, 400, 20, 1011136); // Guilds that we have declared war on:

        list_or(b, &self.declared, 1018012); // No current invitations received for war.

        b.add_page(3);

        b.add_button_type(30, 375, 5223, 5223, 0, GumpButtonType::Page, 2);
        hl(b, 65, 373, 150, 25, 1011067); // Previous page

        hl(b, 20, 45, 400, 20, 1011135); // Guilds that have declared war on us:

        list_or(b, &self.invites, 1013055); // No current war declarations
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let Some(from) = mobile_of(w, sender) else {
            return;
        };
        if bad_member(w, from, self.guild) {
            return;
        }
        if info.button_id == 1 {
            GuildGump::display_to(w, from, self.guild);
        }
    }
}

fn list_or(b: &mut DynamicGumpBuilder, names: &[String], empty: i32) {
    if names.is_empty() {
        hl(b, 20, 65, 400, 20, empty);
    } else {
        for (i, n) in names.iter().enumerate() {
            b.add_html(20, 65 + i as i32 * 20, 300, 20, n, false, false);
        }
    }
}

// ---- GuildWebsitePrompt ----

fn guild_website_prompt(w: &mut World, mobile: MobileId, g: GuildId) {
    set_prompt(
        w,
        mobile,
        Prompt {
            on_cancel: Some(Rc::new(move |w: &mut World, _from| {
                if !bad_leader(w, mobile, g) {
                    GuildmasterGump::display_to(w, mobile, g);
                }
            })),
            on_response: Some(Rc::new(move |w: &mut World, _from, text: &str| {
                if bad_leader(w, mobile, g) {
                    return;
                }
                let s: String = text.trim().chars().take(50).collect();
                if !s.is_empty()
                    && let Some(d) = gc::data_mut(w, g)
                {
                    d.website = Some(s);
                }
                GuildmasterGump::display_to(w, mobile, g);
            })),
        },
    );
}

// ---- RecruitTarget ----

fn guild_recruit_target(w: &mut World, mobile: MobileId, g: GuildId) {
    let mut t = Target::new(w, 10, false, TargetFlags::NONE);
    t.handlers.on_target = Some(Rc::new(move |w: &mut World, _from, targeted: Targeted| {
        if bad_member(w, mobile, g) {
            return;
        }
        let Targeted::Entity(muo_server::ids::EntityId::Mobile(m)) = targeted else {
            return;
        };

        let guild_faction = leader_of(w, g).and_then(|l| faction_of(w, l));
        let target_faction = faction_of(w, m);
        let d = data(w, g);

        if !muo_content::mx::player(w, m) {
            muo_content::mx::msg(w, mobile, 501161); // You may only recruit players into the guild.
        } else if !muo_content::mx::alive(w, m) {
            muo_content::mx::msg(w, mobile, 501162); // Only the living may be recruited.
        } else if gc::is_member(w, g, m) {
            muo_content::mx::msg(w, mobile, 501163); // They are already a guildmember!
        } else if d.is_some_and(|d| d.candidates.contains(&m)) {
            muo_content::mx::msg(w, mobile, 501164); // They are already a candidate.
        } else if d.is_some_and(|d| d.accepted.contains(&m)) {
            // They have already been accepted for membership, and merely need to use the Guildstone to gain full membership.
            muo_content::mx::msg(w, mobile, 501165);
        } else if w.mobile(m).is_some_and(|x| x.guild.is_some()) {
            muo_content::mx::msg(w, mobile, 501166); // You can only recruit candidates who are not already in a guild.
        } else if guild_faction != target_faction {
            if guild_faction.is_none() {
                muo_content::mx::msg(w, mobile, 1013027); // That player cannot join a non-faction guild.
            } else if target_faction.is_none() {
                muo_content::mx::msg(w, mobile, 1013026); // That player must be in a faction before joining this guild.
            } else {
                muo_content::mx::msg(w, mobile, 1013028); // That person has a different faction affiliation.
            }
        } else if faction_leaving(w, m) {
            muo_content::mx::send_message(
                w,
                mobile,
                "That person is quitting their faction and so you may not recruit them.",
            );
        } else if access(w, mobile) >= AccessLevel::GameMaster || leader_of(w, g) == Some(mobile) {
            if let Some(d) = gc::data_mut(w, g) {
                d.accepted.push(m);
            }
        } else if let Some(d) = gc::data_mut(w, g) {
            d.candidates.push(m);
        }
    }));
    t.handlers.on_target_finish = Some(Rc::new(move |w: &mut World, _from| {
        if bad_member(w, mobile, g) {
            return;
        }
        GuildGump::display_to(w, mobile, g);
    }));
    let id = add_target(w, t);
    set_mobile_target(w, mobile, Some(id));
}

/// `GuildGump.DisplayTo` for callers outside the guild gumps (the guildstone).
pub fn open_guild_gump(w: &mut World, from: MobileId, g: GuildId) {
    GuildGump::display_to(w, from, g);
}

#[cfg(test)]
mod tests {
    use super::*;
    use muo_content::gumps::base::Gump;

    fn layout_of(g: &mut dyn Gump) -> String {
        g.core_mut().serial = muo_primitives::Serial::new(1);
        let p = g.compile();
        let len = i32::from_be_bytes(p[19..23].try_into().unwrap());
        let raw_len = i32::from_be_bytes(p[23..27].try_into().unwrap()) as usize;
        let mut out = vec![0u8; raw_len];
        muo_io::compression::STANDARD
            .unpack(&mut out, &p[27..19 + 4 + len as usize])
            .unwrap();
        String::from_utf8_lossy(&out)
            .trim_end_matches('\0')
            .to_owned()
    }

    #[test]
    fn a_list_gump_pages_eleven_rows_with_radios() {
        let names: Vec<String> = (0..12).map(|i| format!("m{i}")).collect();
        let list = vec![MobileId(muo_primitives::Serial::new(0x1)); 12];
        let mut g = DynamicGumpObj::new(
            20,
            30,
            DeclareFealtyGump {
                guild: GuildId(1),
                list,
                names,
            },
        );
        let l = layout_of(&mut g);
        assert!(l.starts_with(
            "{ nomove }{ page 0 }{ resizepic 0 0 5054 550 440 }{ resizepic 10 10 3000 530 420 }"
        ));
        assert!(l.contains("{ page 1 }") && l.contains("{ page 2 }"));
        // row 0 and the first row of page 2, with the radio at x 20 and the label at x 55
        assert!(l.contains("{ radio 20 35 208 209 0 0 }"));
        assert!(l.contains("{ radio 20 35 208 209 0 11 }"));
        // the Next page button leads page 1 to 2 (type page, param 2)
        assert!(l.contains("{ button 300 370 4005 4007 0 2 0 }"));
    }

    #[test]
    fn the_war_status_gump_has_three_pages() {
        let mut g = DynamicGumpObj::new(
            20,
            30,
            GuildWarGump {
                guild: GuildId(1),
                enemies: vec!["A".into()],
                declared: vec![],
                invites: vec![],
            },
        );
        let l = layout_of(&mut g);
        assert!(l.contains("{ page 3 }"));
        assert!(!l.contains("{ page 4 }"));
    }

    /// A leader standing at their guildstone, and `n` other players.
    fn guild_with_stone(n: usize) -> (World, MobileId, GuildId, Vec<MobileId>) {
        let mut w = muo_server::map::test_support::world();
        let at = muo_primitives::Point3D::new(1500, 1500, 0);
        let mk = |w: &mut World| {
            let m = muo_content::player_mobile::new_player_mobile(w);
            muo_server::mobile::base::move_to_world(w, m, at, muo_server::map::MapId::FELUCCA);
            m
        };
        let leader = mk(&mut w);
        let g = muo_content::misc::guild_class::new_guild(&mut w, leader, "Test Guild", "TST");
        let stone = w.new_item(&muo_server::item::ITEM_BASE);
        muo_server::item::ops::move_to_world_map(
            &mut w,
            stone,
            at,
            Some(muo_server::map::MapId::FELUCCA),
        );
        gc::data_mut(&mut w, g).unwrap().guildstone = Some(stone);
        let others = (0..n).map(|_| mk(&mut w)).collect();
        (w, leader, g, others)
    }

    /// GuildMobileListGump copies `guild.Candidates`: a row means the candidate shown there, whatever the list became.
    #[test]
    fn admin_candidates_index_the_shown_list() {
        let (mut w, leader, g, c) = guild_with_stone(3);
        gc::data_mut(&mut w, g).unwrap().candidates = c.clone();
        let mut gump = GuildAdminCandidatesGump {
            guild: g,
            list: c.clone(),
            names: vec![],
        };
        // the first candidate leaves behind the gump's back: [b, c]; row 1 is still b
        gc::data_mut(&mut w, g).unwrap().candidates.remove(0);
        gump.respond(&mut w, leader, &RelayInfo::new(2, vec![1], vec![])); // Refuse
        assert_eq!(data(&w, g).unwrap().candidates, vec![c[2]]);
    }

    /// Same copy for DeclareFealtyGump (`guild.Members` is copied by GuildMobileListGump): row 1 stays who was shown.
    #[test]
    fn declare_fealty_indexes_the_shown_list() {
        let (mut w, leader, g, c) = guild_with_stone(2);
        let shown = vec![leader, c[0], c[1]];
        gc::data_mut(&mut w, g).unwrap().members = shown.clone();
        let mut gump = DeclareFealtyGump {
            guild: g,
            list: shown,
            names: vec![],
        };
        gc::data_mut(&mut w, g).unwrap().members.remove(0);
        gump.respond(&mut w, c[0], &RelayInfo::new(1, vec![1], vec![]));
        assert_eq!(w.mobile(c[0]).unwrap().guild_fealty, Some(c[0]));
    }
}
