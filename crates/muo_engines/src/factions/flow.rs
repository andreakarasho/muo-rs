//! The rest of `Faction`: joining (`OnJoinAccepted`, `JoinGuilded`, `JoinAlone`), the
//! account checks, `ProcessTick`, the commands (`FactionElection`, `FactionCommander`, `FactionItemReset`,
//! `FactionReset`, `FactionTownReset`, `FactionKick/Ban/Unban`), `BeginBroadcast`, `BeginHonorLeadership`, and
//! the faction generator.
//!
//! The old `Guild` class is not here: what the join flow asks of it is a [`GuildHooks`] in `w.resources`. With
//! no hook set a mobile that has a guild is treated as the sole member and guildmaster of a regular guild.

use std::rc::Rc;

use muo_primitives::DateTime;
use muo_server::World;
use muo_server::commands::CommandSystem;
use muo_server::guild::{GuildId, GuildType};
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::LootType;
use muo_server::mobile::AccessLevel;
use muo_server::mobile::part2::messages as mm;
use muo_server::prompt;
use muo_server::target::{self, TargetFlags, Targeted};

use muo_content::player_mobile;

use super::items;
use super::tables::{FACTION_DEFS, TOWN_DEFS};
use super::{FACET, find, player_state, st, st_mut};

/// What the join flow asks of a mobile's `Guild`.
pub struct GuildInfo {
    pub id: GuildId,
    pub leader: Option<MobileId>,
    pub ty: GuildType,
    /// `Enemies.Count`.
    pub enemies: usize,
    pub has_alliance: bool,
    pub members: Vec<MobileId>,
}

#[derive(Default)]
pub struct GuildHooks {
    /// `mob.Guild as Guild`. Unset: a mobile with a guild is its sole member and leader.
    pub guild_of: Option<fn(&World, MobileId) -> Option<GuildInfo>>,
    /// `guild.RemoveMember(mob)`. Unset: the mobile's guild is cleared.
    pub remove_member: Option<fn(&mut World, GuildId, MobileId)>,
}

fn guild_of(w: &World, m: MobileId) -> Option<GuildInfo> {
    if let Some(f) = w.resources.get::<GuildHooks>().and_then(|h| h.guild_of) {
        return f(w, m);
    }
    let id = w.mobile(m)?.guild?;
    Some(GuildInfo {
        id,
        leader: Some(m),
        ty: muo_server::guild::guild(w, id).map_or(GuildType::Regular, |g| g.ty),
        enemies: 0,
        has_alliance: false,
        members: vec![m],
    })
}

fn guild_remove_member(w: &mut World, g: GuildId, m: MobileId) {
    if let Some(f) = w
        .resources
        .get::<GuildHooks>()
        .and_then(|h| h.remove_member)
    {
        f(w, g, m);
    } else {
        muo_server::mobile::part1::props_b::set_guild(w, m, None);
    }
}

fn msg(w: &mut World, m: MobileId, n: i32) {
    mm::send_localized_message(w, m, n, "", 0x3B2);
}

fn account_of(w: &World, m: MobileId) -> Option<muo_server::iaccount::AccountId> {
    w.mobile(m)?.account
}

/// `AlreadyHasCharInFaction(mob)`.
fn already_has_char_in_faction(w: &World, mob: MobileId) -> bool {
    let Some(a) = account_of(w, mob).and_then(|id| muo_content::accounting::account(w, id)) else {
        return false;
    };
    a.mobiles.iter().flatten().any(|&c| find(w, c).is_some())
}

/// `IsFactionBanned(mob)`.
pub fn is_faction_banned(w: &World, mob: MobileId) -> bool {
    account_of(w, mob)
        .and_then(|id| muo_content::accounting::account(w, id))
        .is_some_and(|a| a.get_tag("FactionBanned").is_some())
}

/// `JoinGuilded(mob, guild)`.
pub fn join_guilded(w: &mut World, f: usize, mob: MobileId, guild: GuildId) {
    if player_mobile::young(w, mob) {
        guild_remove_member(w, guild, mob);
        // You have been kicked out of your guild!
        // Young players may not remain in a guild which is allied with a faction.
        msg(w, mob, 1042283);
    } else if already_has_char_in_faction(w, mob) {
        guild_remove_member(w, guild, mob);
        msg(w, mob, 1005281); // You have been kicked out of your guild due to factional overlap
    } else if is_faction_banned(w, mob) {
        guild_remove_member(w, guild, mob);
        msg(w, mob, 1005052); // You are currently banned from the faction system
    } else {
        super::add_member(w, f, mob);
        // You are now joining a faction:
        mm::send_localized_message_affix(
            w,
            mob,
            1042756,
            true,
            &format!(" {}", FACTION_DEFS[f].friendly_name),
            "",
            0x3B2,
        );
    }
}

/// `JoinAlone(mob)`.
pub fn join_alone(w: &mut World, f: usize, mob: MobileId) {
    super::add_member(w, f, mob);
    msg(w, mob, 1005058); // You have joined the faction
}

/// `OnJoinAccepted(mob)`.
pub fn on_join_accepted(w: &mut World, f: usize, pm: MobileId) {
    if player_mobile::data(w, pm).is_none() {
        return; // sanity
    }
    let leaving = player_state(w, pm).is_some_and(|(_, p)| p.is_leaving());
    if player_mobile::young(w, pm) {
        msg(w, pm, 1010104); // You cannot join a faction as a young player
    } else if leaving {
        // You cannot use the faction stone until you have finished quitting your current faction
        msg(w, pm, 1005051);
    } else if already_has_char_in_faction(w, pm) {
        // You cannot join a faction because you already declared your allegiance with another character
        msg(w, pm, 1005059);
    } else if is_faction_banned(w, pm) {
        msg(w, pm, 1005052); // You are currently banned from the faction system
    } else if let Some(guild) = guild_of(w, pm) {
        if guild.leader != Some(pm) {
            // You cannot join a faction because you are in a guild and not the guildmaster
            msg(w, pm, 1005057);
        } else if guild.ty != GuildType::Regular {
            // You cannot join a faction because your guild is an Order or Chaos type.
            msg(w, pm, 1042161);
        } else if !muo_content::misc::guild::new_guild_system(w) && guild.enemies > 0 {
            msg(w, pm, 1005056); // You cannot join a faction with active Wars
        } else if muo_content::misc::guild::new_guild_system(w) && guild.has_alliance {
            // Your guild cannot join a faction while in alliance with non-factioned guilds.
            msg(w, pm, 1080454);
        } else if !super::can_handle_influx(w, f, guild.members.len()) {
            // In the interest of faction stability, this faction declines to accept new members for now.
            msg(w, pm, 1018031);
        } else {
            for member in guild.members.clone() {
                if player_mobile::data(w, member).is_none() {
                    continue;
                }
                join_guilded(w, f, member, guild.id);
            }
        }
    } else if !super::can_handle_influx(w, f, 1) {
        // In the interest of faction stability, this faction declines to accept new members for now.
        msg(w, pm, 1018031);
    } else {
        join_alone(w, f, pm);
    }
}

/// `BeginBroadcast(from)`.
pub fn begin_broadcast(w: &mut World, f: usize, from: MobileId) {
    msg(w, from, 1010265); // Enter Faction Message
    let p = prompt::SimplePrompt::new(
        Some(Rc::new(move |w: &mut World, from, text: &str| {
            super::end_broadcast(w, f, from, text)
        })),
        false,
    );
    let pid = prompt::add_prompt(w, p.into_prompt());
    muo_server::mobile::part1::props_b::set_prompt(w, from, Some(pid));
}

/// `BeginHonorLeadership(from)`.
pub fn begin_honor_leadership(w: &mut World, from: MobileId) {
    msg(w, from, 502090); // Click on the player whom you wish to honor.
    target::begin_target(w, from, 12, false, TargetFlags::NONE, |w, from, obj| {
        let recv = match obj {
            Targeted::Entity(EntityId::Mobile(m)) => Some(m),
            _ => None,
        };
        super::honor_leadership(w, from, recv);
    });
}

// ---- ProcessTick ----

/// `Faction.ProcessTick()`: every 30 seconds.
pub fn process_tick(w: &mut World) {
    let sigils = st(w).map_or_else(Vec::new, |s| s.sigils.clone());
    for sg in sigils {
        let now = w.now;
        let Some(d) = items::sigil(w, sg) else {
            continue;
        };
        let grace_start = d.grace_start;
        if !items::is_being_corrupted(w, sg)
            && grace_start != DateTime::MIN
            && grace_start + items::sigil_corruption_grace(w) < now
        {
            let d = items::sigil(w, sg).expect("sigil");
            let (corrupted, lm) = (d.corrupted, d.last_monolith);
            let lm_faction = lm.and_then(|m| items::monolith_faction(w, m));
            if lm.is_some_and(|m| items::is_stronghold_monolith(w, m))
                && (corrupted.is_none() || lm_faction != corrupted)
            {
                items::sigil_set_corrupting(w, sg, lm_faction);
                if let Some(d) = items::sigil_mut(w, sg) {
                    d.corruption_start = now;
                }
            } else {
                items::sigil_set_corrupting(w, sg, None);
                if let Some(d) = items::sigil_mut(w, sg) {
                    d.corruption_start = DateTime::MIN;
                }
            }
            if let Some(d) = items::sigil_mut(w, sg) {
                d.grace_start = DateTime::MIN;
            }
        }

        // `sigil.LastMonolith?.Sigil == null`
        let lm = items::sigil(w, sg).and_then(|d| d.last_monolith);
        if lm.and_then(|m| items::monolith_sigil(w, m)).is_none() {
            let stolen = items::sigil(w, sg).map_or(DateTime::MIN, |d| d.last_stolen);
            if stolen + items::SIGIL_RETURN_PERIOD < now {
                items::return_home(w, sg);
            }
            continue;
        }

        let d = items::sigil(w, sg).expect("sigil");
        let (corruption_start, purification_start) = (d.corruption_start, d.purification_start);
        if items::is_being_corrupted(w, sg)
            && corruption_start + items::sigil_corruption_period(w) < now
        {
            let c = items::sigil(w, sg).and_then(|d| d.corrupting);
            items::sigil_set_corrupted(w, sg, c);
        } else if items::is_purifying(w, sg)
            && purification_start + items::SIGIL_PURIFICATION_PERIOD < now
            && let Some(d) = items::sigil_mut(w, sg)
        {
            d.purification_start = DateTime::MIN;
        }

        items::sigil_set_corrupting(w, sg, None);
        if let Some(d) = items::sigil_mut(w, sg) {
            d.corruption_start = DateTime::MIN;
            d.grace_start = DateTime::MIN;
        }
    }
}

// ---- resets ----

fn reset_towns_and_sigils(w: &mut World) {
    let monoliths = st(w).map_or_else(Vec::new, |s| s.monoliths.clone());
    for m in monoliths {
        items::monolith_set_sigil(w, m, None);
    }
    for t in 0..TOWN_DEFS.len() {
        {
            let s = st_mut(w);
            s.towns[t].silver = 0;
        }
        super::set_town_sheriff(w, t, None);
        super::set_town_finance(w, t, None);
        st_mut(w).towns[t].tax = 0;
        super::town::capture(w, t, None);
    }
    let sigils = st(w).map_or_else(Vec::new, |s| s.sigils.clone());
    for sg in sigils {
        items::sigil_set_corrupted(w, sg, None);
        items::sigil_set_corrupting(w, sg, None);
        if let Some(d) = items::sigil_mut(w, sg) {
            d.last_stolen = DateTime::MIN;
            d.grace_start = DateTime::MIN;
            d.corruption_start = DateTime::MIN;
            d.purification_start = DateTime::MIN;
            d.last_monolith = None;
        }
        items::return_home(w, sg);
    }
}

fn reset_faction_items(w: &mut World, f: usize) {
    let list: Vec<_> = st(w).map_or_else(Vec::new, |s| s.factions[f].items.clone());
    for fi in list {
        let Some(item) = fi.item else { continue };
        if fi.expiration == DateTime::MIN {
            if let Some(t) = w.item(item).map(|i| i.ty) {
                (t.delete)(w, item);
            }
        } else {
            super::detach(w, f, item);
        }
    }
}

/// `FactionTownReset_OnCommand`.
pub fn faction_town_reset(w: &mut World) {
    reset_towns_and_sigils(w);
    for f in 0..FACTION_DEFS.len() {
        reset_faction_items(w, f);
    }
}

/// `FactionReset_OnCommand`.
pub fn faction_reset(w: &mut World) {
    reset_towns_and_sigils(w);
    for f in 0..FACTION_DEFS.len() {
        let members: Vec<MobileId> = st(w).map_or_else(Vec::new, |s| {
            s.factions[f].members.iter().map(|p| p.mobile).collect()
        });
        for m in members {
            super::remove_member(w, m);
        }
        reset_faction_items(w, f);
        let traps: Vec<ItemId> = st(w).map_or_else(Vec::new, |s| s.factions[f].traps.clone());
        for t in traps {
            if let Some(ty) = w.item(t).map(|i| i.ty) {
                (ty.delete)(w, t);
            }
        }
    }
}

/// `FactionItemReset_OnCommand`.
pub fn faction_item_reset(w: &mut World, from: MobileId) {
    let mut hues = Vec::new();
    for d in FACTION_DEFS.iter() {
        hues.push(d.hue_primary);
        hues.push(d.hue_secondary);
    }
    let mut count = 0;
    let ids: Vec<ItemId> = w.items.keys().copied().collect();
    for id in ids {
        let Some(i) = w.item(id) else { continue };
        let is_faction_item = i.ty.is(&muo_content::items::armor::BASE_ARMOR)
            || i.ty.is(&muo_content::items::clothing::BASE_CLOTHING)
            || i.ty.is(&muo_content::items::weapon::BASE_WEAPON);
        // `item is IFactionItem && item is not HoodedShroudOfShadows`
        if !is_faction_item || i.ty.full_name == "Server.Items.HoodedShroudOfShadows" {
            continue;
        }
        if super::item_has_state(w, id) || i.loot_type != LootType::Blessed {
            continue;
        }
        if hues.contains(&i.hue) {
            count += 1; // `fci.FactionItemState = null`
        }
    }
    mm::send_message(w, from, &format!("{count} items reset"));
}

// ---- commands ----

fn target_mobile(obj: Targeted) -> Option<MobileId> {
    match obj {
        Targeted::Entity(EntityId::Mobile(m)) => Some(m),
        _ => None,
    }
}

fn faction_commander_command(w: &mut World, from: MobileId) {
    mm::send_message(
        w,
        from,
        "Target a player to make them the faction commander.",
    );
    target::begin_target(
        w,
        from,
        -1,
        false,
        TargetFlags::NONE,
        |w, from, obj| match target_mobile(obj).filter(|&m| player_mobile::data(w, m).is_some()) {
            Some(targ) => match find(w, targ) {
                Some(f) => {
                    super::set_commander(w, f, Some(targ));
                    mm::send_message(w, from, "You have appointed them as the faction commander.");
                }
                None => mm::send_message(w, from, "They are not in a faction."),
            },
            None => mm::send_message(w, from, "That is not a player."),
        },
    );
}

fn faction_election_command(w: &mut World, from: MobileId) {
    mm::send_message(
        w,
        from,
        "Target a faction stone to open its election properties.",
    );
    target::begin_target(w, from, -1, false, TargetFlags::NONE, |w, from, obj| {
        let stone = match obj {
            Targeted::Entity(EntityId::Item(i))
                if w.item(i).is_some_and(|x| x.ty.is(&items::FACTION_STONE)) =>
            {
                Some(i)
            }
            _ => None,
        };
        match stone {
            Some(s) => match items::stone_faction(w, s) {
                Some(f) => super::gumps::send_election_management_gump(w, from, f),
                None => mm::send_message(w, from, "That stone has no faction assigned."),
            },
            None => mm::send_message(w, from, "That is not a faction stone."),
        }
    });
}

/// `FactionKickCommand.Execute`: the reply text for `muo_commands`. `kind` is 0 kick, 1 ban, 2 unban.
pub fn faction_kick(
    w: &mut World,
    _from: MobileId,
    mob: MobileId,
    kind: u8,
) -> Vec<(bool, String)> {
    let mut out: Vec<(bool, String)> = Vec::new();
    match kind {
        0 => {
            if find(w, mob).is_some() {
                super::remove_member(w, mob);
                mm::send_message(w, mob, "You have been kicked from your faction.");
                out.push((true, "They have been kicked from their faction.".into()));
            } else {
                out.push((false, "They are not in a faction.".into()));
            }
        }
        1 => {
            let Some(acct) = account_of(w, mob) else {
                return vec![(false, "They have no assigned account.".into())];
            };
            let banned = muo_content::accounting::account(w, acct)
                .is_some_and(|a| a.get_tag("FactionBanned").is_some());
            if !banned {
                if let Some(a) = muo_content::accounting::accounts(w).get_mut(acct) {
                    a.set_tag("FactionBanned", "true");
                }
                out.push((
                    true,
                    "The account has been banned from joining factions.".into(),
                ));
            } else {
                out.push((
                    true,
                    "The account is already banned from joining factions.".into(),
                ));
            }
            let chars: Vec<MobileId> = muo_content::accounting::account(w, acct)
                .map_or_else(Vec::new, |a| a.mobiles.iter().flatten().copied().collect());
            for c in chars {
                if find(w, c).is_some() {
                    super::remove_member(w, c);
                    mm::send_message(w, c, "You have been kicked from your faction.");
                    out.push((true, "They have been kicked from their faction.".into()));
                }
            }
        }
        _ => {
            let Some(acct) = account_of(w, mob) else {
                return vec![(false, "They have no assigned account.".into())];
            };
            let banned = muo_content::accounting::account(w, acct)
                .is_some_and(|a| a.get_tag("FactionBanned").is_some());
            if !banned {
                out.push((
                    true,
                    "The account is not already banned from joining factions.".into(),
                ));
            } else {
                if let Some(a) = muo_content::accounting::accounts(w).get_mut(acct) {
                    a.remove_tag("FactionBanned");
                }
                out.push((true, "The account may now freely join factions.".into()));
            }
        }
    }
    out
}

/// `Generator.GenerateFactions_OnCommand`.
pub fn generate_factions(w: &mut World, from: MobileId) {
    super::enable(w);
    for (f, def) in FACTION_DEFS.iter().enumerate() {
        generate_faction(w, f);
        mm::send_message(w, from, &format!("Generated {}", def.friendly_name));
    }
    for (t, def) in TOWN_DEFS.iter().enumerate() {
        generate_town(w, t);
        mm::send_message(w, from, &format!("Generated {}", def.friendly_name));
    }
    mm::send_message(w, from, "Faction generation completed.");
}

/// `CheckExistence(loc, facet, type)`.
fn check_existence(w: &World, loc: (i32, i32, i32), is: impl Fn(&World, ItemId) -> bool) -> bool {
    muo_server::map::enumerate::items_at(w, Some(FACET), loc.0, loc.1).any(|i| is(w, i))
}

fn move_to(w: &mut World, item: ItemId, loc: (i32, i32, i32)) {
    muo_server::item::ops::move_to_world_map(
        w,
        item,
        muo_primitives::Point3D::new(loc.0, loc.1, loc.2),
        Some(FACET),
    );
}

/// `Generate(Town)`.
pub fn generate_town(w: &mut World, t: usize) {
    let def = TOWN_DEFS[t];
    if !check_existence(w, def.monolith, items::is_town_monolith) {
        let mono = items::new_monolith(w, false, Some(t), None);
        move_to(w, mono, def.monolith);
        let exists = st(w).is_some_and(|s| {
            s.sigils.iter().any(|&sg| {
                w.item(sg).is_some_and(|i| !i.deleted())
                    && items::sigil(w, sg).and_then(|d| d.town) == Some(t)
            })
        });
        if !exists {
            let sg = items::new_sigil(w, Some(t));
            items::monolith_set_sigil(w, mono, Some(sg));
        }
    }
    if !check_existence(w, def.town_stone, |w, i| {
        w.item(i).is_some_and(|x| x.ty.is(&items::TOWN_STONE))
    }) {
        let stone = items::new_town_stone(w, Some(t));
        move_to(w, stone, def.town_stone);
    }
}

/// `Generate(Faction)`.
pub fn generate_faction(w: &mut World, f: usize) {
    let sh = &FACTION_DEFS[f].stronghold;
    if !check_existence(w, sh.join_stone, |w, i| {
        w.item(i).is_some_and(|x| x.ty.is(&items::JOIN_STONE))
    }) {
        let s = items::new_join_stone(w, Some(f));
        move_to(w, s, sh.join_stone);
    }
    if !check_existence(w, sh.faction_stone, |w, i| {
        w.item(i).is_some_and(|x| x.ty.is(&items::FACTION_STONE))
    }) {
        let s = items::new_faction_stone(w, Some(f));
        move_to(w, s, sh.faction_stone);
    }
    for (i, &loc) in sh.monoliths.iter().enumerate() {
        if !check_existence(w, loc, items::is_stronghold_monolith) {
            let m = items::new_monolith(w, true, Some(i), Some(f));
            move_to(w, m, loc);
        }
    }
}

/// `Faction.Configure` / `Generator.Configure` / `Town.Configure`: the commands.
pub fn register_commands(w: &mut World) {
    use muo_server::mobile::AccessLevel as A;
    CommandSystem::register(
        w,
        "FactionElection",
        A::GameMaster,
        &[],
        Rc::new(|w, e| faction_election_command(w, e.mobile)),
    );
    CommandSystem::register(
        w,
        "FactionCommander",
        A::Administrator,
        &[],
        Rc::new(|w, e| faction_commander_command(w, e.mobile)),
    );
    CommandSystem::register(
        w,
        "FactionItemReset",
        A::Administrator,
        &[],
        Rc::new(|w, e| faction_item_reset(w, e.mobile)),
    );
    CommandSystem::register(
        w,
        "FactionReset",
        A::Administrator,
        &[],
        Rc::new(|w, _| faction_reset(w)),
    );
    CommandSystem::register(
        w,
        "FactionTownReset",
        A::Administrator,
        &[],
        Rc::new(|w, _| faction_town_reset(w)),
    );
    CommandSystem::register(
        w,
        "GenerateFactions",
        A::Developer,
        &["FactionGen", "GenFactions"],
        Rc::new(|w, e| generate_factions(w, e.mobile)),
    );
    CommandSystem::register(
        w,
        "GrantTownSilver",
        A::Administrator,
        &[],
        Rc::new(super::town::grant_town_silver),
    );
    w.resources
        .get_mut::<muo_commands::hooks::CommandHooks>()
        .faction_kick = Some(faction_kick);
    w.resources
        .get_mut::<muo_items::guilds::GuildstoneHooks>()
        .join_check = Some(guildstone_join_check);
    let _ = AccessLevel::Player;
}

/// The faction check of `Guildstone.OnDoubleClick` for an accepted invitation: false when `from` may not join
/// (the guild leader's faction differs, or `from` is leaving his); otherwise `targetState.Leaving` becomes the
/// leader's.
fn guildstone_join_check(w: &mut World, leader: Option<MobileId>, from: MobileId) -> bool {
    let guild_state = leader
        .and_then(|l| super::player_state(w, l))
        .map(|(f, p)| (f, p.leaving));
    let target_state = super::player_state(w, from).map(|(f, p)| (f, p.is_leaving()));
    if guild_state.map(|g| g.0) != target_state.map(|t| t.0) || target_state.is_some_and(|t| t.1) {
        return false;
    }
    if let (Some((_, leaving)), Some((f, _))) = (guild_state, target_state)
        && let Some(p) = st_mut(w).factions[f]
            .members
            .iter_mut()
            .find(|p| p.mobile == from)
    {
        p.leaving = leaving;
    }
    true
}

/// `m.Guild is Guild guild ? guild.Leader : null`.
pub fn guild_leader_of(w: &World, m: MobileId) -> Option<MobileId> {
    guild_of(w, m).and_then(|g| g.leader)
}

/// `LeaveFactionGump.OnResponse`, continue.
pub fn leave_faction(w: &mut World, from: MobileId) {
    let leave_days = super::LEAVE_PERIOD.total_days();
    let now = w.now;
    let set_leaving = |w: &mut World, m: MobileId| -> bool {
        match find(w, m) {
            Some(f) => {
                if let Some(p) = st_mut(w).factions[f]
                    .members
                    .iter_mut()
                    .find(|p| p.mobile == m)
                {
                    p.leaving = now;
                }
                true
            }
            None => false,
        }
    };
    match guild_of(w, from) {
        None => {
            if set_leaving(w, from) {
                if leave_days == 3.0 {
                    msg(w, from, 1005065); // You will be removed from the faction in 3 days
                } else {
                    mm::send_message(
                        w,
                        from,
                        &format!("You will be removed from the faction in {leave_days} days."),
                    );
                }
            }
        }
        Some(g) if g.leader != Some(from) => msg(w, from, 1005061), // You cannot quit the faction because you are not the guild master
        Some(g) => {
            msg(w, from, 1042285); // Your guild is now quitting the faction.
            for m in g.members {
                if set_leaving(w, m) {
                    if leave_days == 3.0 {
                        msg(w, m, 1005060); // Your guild will quit the faction in 3 days
                    } else {
                        mm::send_message(
                            w,
                            m,
                            &format!("Your guild will quit the faction in {leave_days} days."),
                        );
                    }
                }
            }
        }
    }
}

// ---- Keywords ----

/// `Keywords.EventSink_Speech`.
pub fn on_speech(w: &mut World, e: &mut muo_server::events::SpeechEventArgs) {
    let from = e.mobile;
    for &k in &e.keywords.clone() {
        match k {
            0x00E4 => {
                // *i wish to access the city treasury*
                let region = muo_server::mobile::part1::props_b::region(w, from);
                let Some(t) = super::town::from_region(w, region) else { continue };
                if !super::town::is_finance(w, t, from) || !muo_server::mobile::base::alive(w, from) {
                    continue;
                }
                if items::faction_gump_exists(w, from) {
                    msg(w, from, 1042160); // You already have a faction menu open.
                } else if let Some(o) = super::town::owner(w, t)
                    && player_mobile::data(w, from).is_some()
                {
                    super::gumps::send_finance_gump(w, from, o, t);
                }
            }
            0x00ED => {
                // *i am sheriff*
                let region = muo_server::mobile::part1::props_b::region(w, from);
                let Some(t) = super::town::from_region(w, region) else { continue };
                if !super::town::is_sheriff(w, t, from) || !muo_server::mobile::base::alive(w, from) {
                    continue;
                }
                if items::faction_gump_exists(w, from) {
                    msg(w, from, 1042160);
                } else if let Some(o) = super::town::owner(w, t) {
                    super::gumps::send_sheriff_gump(w, from, o, t);
                }
            }
            0x00EF => {
                // *you are fired*
                let region = muo_server::mobile::part1::props_b::region(w, from);
                let Some(t) = super::town::from_region(w, region) else { continue };
                if super::town::is_finance(w, t, from) || super::town::is_sheriff(w, t, from) {
                    super::town::begin_order_firing(w, t, from);
                }
            }
            0x00E5 => {
                // *i wish to resign as finance minister*
                if let Some(t) = player_state(w, from).and_then(|(_, p)| p.finance) {
                    super::set_town_finance(w, t, None);
                    msg(w, from, 1005081); // You have been fired as Finance Minister
                }
            }
            0x00EE => {
                // *i wish to resign as sheriff*
                if let Some(t) = player_state(w, from).and_then(|(_, p)| p.sheriff) {
                    super::set_town_sheriff(w, t, None);
                    msg(w, from, 1010270); // You have been fired as Sheriff
                }
            }
            0x00E9 => {
                // *what is my faction term status*
                let Some((_, p)) = player_state(w, from) else { continue };
                if p.is_leaving() {
                    let leaving = p.leaving;
                    if super::check_leave_timer(w, from) {
                        continue;
                    }
                    let remaining = leaving + super::LEAVE_PERIOD - w.now;
                    let n0 = muo_content::items::gold::n0;
                    if remaining.total_days() >= 1.0 {
                        // Your term of service will come to an end in ~1_DAYS~ days.
                        mm::send_localized_message(w, from, 1042743, &n0(remaining.total_days().round() as i64), 0x3B2);
                    } else if remaining.total_hours() >= 1.0 {
                        mm::send_localized_message(w, from, 1042741, &n0(remaining.total_hours().round() as i64), 0x3B2);
                    } else {
                        msg(w, from, 1042742); // Your term of service will come to an end in less than one hour.
                    }
                } else {
                    msg(w, from, 1042233); // You are not in the process of quitting the faction.
                }
            }
            0x00EA => {
                // *message faction*
                let Some(f) = find(w, from) else { continue };
                if !super::is_commander(w, f, from) {
                    continue;
                }
                let player = w.mobile(from).is_some_and(|m| m.access_level == AccessLevel::Player);
                if player && !st(w).is_some_and(|s| s.factions[f].message_ready(w.now)) {
                    msg(w, from, 1010264); // The required time has not yet passed since the last message was sent
                } else {
                    begin_broadcast(w, f, from);
                }
            }
            0x00EC => {
                // *showscore*
                if let Some((_, p)) = player_state(w, from) {
                    let at_keyword = p.kill_points;
                    muo_server::timer::start(w, muo_primitives::TimeSpan::ZERO, muo_primitives::TimeSpan::ZERO, 1, move |w| {
                        // `ShowScore_Sandbox` reads `pl.KillPoints` when the timer runs, not when the word was said.
                        let kp = player_state(w, from).map_or(at_keyword, |(_, p)| p.kill_points);
                        let hue = w.mobile(from).map_or(0, |m| m.speech_hue);
                        mm::public_overhead_message(
                            w,
                            from,
                            muo_primitives::MessageType::REGULAR,
                            hue,
                            true,
                            &muo_content::items::gold::n0(kp as i64),
                            true,
                            AccessLevel::Player,
                        );
                    });
                }
            }
            0x0178
                // i honor your leadership
                if find(w, from).is_some() => {
                    begin_honor_leadership(w, from);
                }
            _ => {}
        }
    }
}
