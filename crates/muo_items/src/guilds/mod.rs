//! Guild items: `GuildDeed`, `GuildTeleporter`, `Guildstone` and `GuildstoneDeed`.
//!
//! Houses are `muo_multis::house`, the guild is `muo_content::misc::guild_class`.
//!
//! The faction check of the accepted-invitation branch of `Guildstone.OnDoubleClick` (old guild system only) is
//! [`GuildstoneHooks`], set by the factions engine.
//! `Guildstone.OnChop` is [`guildstone_chop`], found through `CommonItemHooks::choppables`.

pub mod props;
use muo_content::props::{Ctor, Kind, Param, Value, add_ctors};
use muo_content::utilities::types::PropType;
use muo_io::GenericWriter;
use muo_primitives::{Point3D, utility};
use muo_server::World;
use muo_server::ext::Ext;
use muo_server::guild::GuildId;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, LootType, base, deps, ops};
use muo_server::mobile::AccessLevel;
use muo_server::mobile::part1::props_b;
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::target::TargetFlags;
use muo_server::type_registry::TypeRegistry;

use crate::misc::{read_version, write_version};

type JoinCheckFn = fn(&mut World, Option<MobileId>, MobileId) -> bool;
/// What `Guildstone` asks of the factions engine. Unset: no faction state, so the check passes.
#[derive(Default)]
pub struct GuildstoneHooks {
    /// `(guild leader, joining mobile)`: false when the member may not join.
    pub join_check: Option<JoinCheckFn>,
}

#[cfg(test)]
mod tests;

fn in_backpack(w: &World, item: ItemId, m: MobileId) -> bool {
    match w.mobile(m).and_then(|m| m.backpack) {
        Some(p) => ops::is_child_of(w, item, Some(EntityId::Item(p))),
        None => ops::is_child_of(w, item, None),
    }
}

fn house_at_mobile(w: &World, m: MobileId) -> Option<ItemId> {
    muo_multis::house::access::find_house_at_mobile(w, m)
}

fn is_owner(w: &World, house: ItemId, m: MobileId) -> bool {
    muo_multis::house::access::is_owner(w, house, m)
}

fn find_guildstone(w: &World, house: ItemId) -> Option<ItemId> {
    muo_multis::house::storage::find_guildstone(w, house)
}

fn delete_item(w: &mut World, item: ItemId) {
    if let Some(ty) = w.item(item).map(|i| i.ty) {
        (ty.delete)(w, item);
    }
}

// ---- GuildDeed ----

/// The checks of `GuildDeed.OnDoubleClick` and its prompt response, in order: the message to send, or None
/// when the guild can be founded.
fn guild_deed_problem(w: &World, deed: ItemId, from: MobileId) -> Option<i32> {
    if !in_backpack(w, deed, from) {
        return Some(1042001); // That must be in your pack for you to use it.
    }
    if w.mobile(from).is_some_and(|m| m.guild.is_some()) {
        return Some(501137); // You must resign from your current guild before founding another!
    }
    let Some(house) = house_at_mobile(w, from) else {
        return Some(501138); // You can only place a guildstone in a house.
    };
    if find_guildstone(w, house).is_some() {
        return Some(501142); // Only one guildstone may reside in a given house.
    }
    if !is_owner(w, house, from) {
        return Some(501141); // You can only place a guildstone in a house you own!
    }
    None
}

/// `new InternalPrompt(deed)`.
fn guild_deed_prompt(id: ItemId) -> muo_server::prompt::Prompt {
    muo_server::prompt::Prompt {
        on_response: Some(std::rc::Rc::new(move |w: &mut World, from, text: &str| {
            guild_deed_response(w, id, from, text)
        })),
        // `InternalPrompt.OnCancel`
        on_cancel: Some(std::rc::Rc::new(|w: &mut World, from| {
            deps::send_localized_message(w, from, 501145); // Placement of guildstone cancelled.
        })),
    }
}

fn guild_deed_double_click(w: &mut World, id: ItemId, from: MobileId) {
    if muo_content::misc::guild::new_guild_system(w) {
        return;
    }
    if let Some(msg) = guild_deed_problem(w, id, from) {
        deps::send_localized_message(w, from, msg);
        return;
    }
    deps::send_localized_message(w, from, 1013060); // Enter new guild name (40 characters max):
    let prompt = guild_deed_prompt(id);
    let pid = muo_server::prompt::add_prompt(w, prompt);
    muo_server::mobile::part1::props_b::set_prompt(w, from, Some(pid));
}

fn guild_deed_response(w: &mut World, deed: ItemId, from: MobileId, text: &str) {
    if ops::is_deleted(w, deed) {
        return;
    }
    if let Some(msg) = guild_deed_problem(w, deed, from) {
        deps::send_localized_message(w, from, msg);
        return;
    }
    delete_item(w, deed);
    let text: String = text.chars().take(40).collect();
    let guild = muo_content::misc::guild_class::new_guild(w, from, &text, "none");
    props_b::set_guild(w, from, Some(guild));
    props_b::set_guild_title(w, from, Some("Guildmaster".to_string()));
    let stone = guildstone_new(
        w,
        &GUILDSTONE,
        Some(guild),
        Some(text.clone()),
        Some("none".to_string()),
    );
    let (loc, map) = w
        .mobile(from)
        .map_or((Point3D::ZERO, None), |m| (m.location, m.map));
    ops::move_to_world_map(w, stone, loc, map);
    if let Some(d) = muo_content::misc::guild_class::data_mut(w, guild) {
        d.guildstone = Some(stone);
    }
}

/// `Server.Items.GuildDeed`.
pub static GUILD_DEED: ItemType = ItemType {
    full_name: "Server.Items.GuildDeed",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| ops::new_item_with_id(w, &GUILD_DEED, 0x14F0)),
    default_weight: |_, _| 1.0,
    label_number: |_, _| 1041055, // a guild deed
    on_double_click: guild_deed_double_click,
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        write_version(out, false);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        read_version(r, false)?;
        Ok(())
    },
    ..ITEM_BASE
};

// ---- GuildTeleporter ----

#[derive(Clone, Debug, Default)]
struct TeleporterData {
    stone: Option<ItemId>,
}

fn teleporter_stone(w: &World, id: ItemId) -> Option<ItemId> {
    w.item(id)?.ext.get::<TeleporterData>()?.stone
}

/// `GuildTeleporter(stone)`.
pub fn guild_teleporter_new(w: &mut World, ty: &'static ItemType, stone: Option<ItemId>) -> ItemId {
    let id = ops::new_item_with_id(w, ty, 0x1869);
    ops::set_loot_type(w, id, LootType::Blessed);
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(TeleporterData { stone });
    }
    id
}

fn guild_teleporter_double_click(w: &mut World, id: ItemId, from: MobileId) {
    if muo_content::misc::guild::new_guild_system(w) {
        return;
    }
    if !in_backpack(w, id, from) {
        deps::send_localized_message(w, from, 1042001); // That must be in your pack for you to use it.
        return;
    }
    let stone = teleporter_stone(w, id)
        .filter(|s| w.item(*s).is_some_and(|i| i.ty.is(&GUILDSTONE)) && !ops::is_deleted(w, *s));
    let guild = stone.and_then(|s| stone_guild(w, s));
    let linked = guild.is_some_and(|g| {
        muo_content::misc::guild_class::data(w, g).and_then(|d| d.teleporter) == Some(id)
    });
    let (Some(stone), Some(guild), true) = (stone, guild, linked) else {
        deps::send_localized_message(w, from, 501197); // This teleporting object can not determine what guildstone to teleport
        return;
    };
    let Some(house) = house_at_mobile(w, from) else {
        deps::send_localized_message(w, from, 501138); // You can only place a guildstone in a house.
        return;
    };
    if !is_owner(w, house, from) {
        deps::send_localized_message(w, from, 501141); // You can only place a guildstone in a house you own!
    } else if find_guildstone(w, house).is_some_and(|s| s != stone) {
        deps::send_localized_message(w, from, 501142); // Only one guildstone may reside in a given house.
    } else {
        let (loc, map) = w
            .mobile(from)
            .map_or((Point3D::ZERO, None), |m| (m.location, m.map));
        ops::move_to_world_map(w, stone, loc, map);
        delete_item(w, id);
        if let Some(d) = muo_content::misc::guild_class::data_mut(w, guild) {
            d.teleporter = None;
        }
    }
}

/// `Server.Items.GuildTeleporter`.
pub static GUILD_TELEPORTER: ItemType = ItemType {
    full_name: "Server.Items.GuildTeleporter",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| guild_teleporter_new(w, &GUILD_TELEPORTER, None)),
    default_weight: |_, _| 1.0,
    label_number: |_, _| 1041054, // guildstone teleporter
    display_loot_type: |_, _| false,
    on_double_click: guild_teleporter_double_click,
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        write_version(out, false);
        out.write_item(w, teleporter_stone(w, id));
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        read_version(r, false)?;
        let stone = r.read_item(w)?;
        if let Some(i) = w.item_mut(id) {
            i.ext = Ext::new(TeleporterData { stone });
        }
        Ok(())
    },
    ..ITEM_BASE
};

// ---- Guildstone, GuildstoneDeed ----

/// The fields of `Guildstone` and `GuildstoneDeed`.
#[derive(Clone, Debug, Default)]
pub struct StoneData {
    pub guild_name: Option<String>,
    pub guild_abbrev: Option<String>,
    pub guild: Option<GuildId>,
}

fn stone(w: &World, id: ItemId) -> StoneData {
    w.item(id)
        .and_then(|i| i.ext.get::<StoneData>())
        .cloned()
        .unwrap_or_default()
}

/// `Guildstone.Guild`.
pub fn stone_guild(w: &World, id: ItemId) -> Option<GuildId> {
    stone(w, id).guild
}

/// `Guildstone(g, guildName, abbrev)`: `Movable = false`, the graphic by `Guild.NewGuildSystem`.
pub fn guildstone_new(
    w: &mut World,
    ty: &'static ItemType,
    guild: Option<GuildId>,
    name: Option<String>,
    abbrev: Option<String>,
) -> ItemId {
    let graphic = if muo_content::misc::guild::new_guild_system(w) {
        0xED6
    } else {
        0xED4
    };
    let id = ops::new_item_with_id(w, ty, graphic);
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(StoneData {
            guild_name: name,
            guild_abbrev: abbrev,
            guild,
        });
    }
    ops::set_movable(w, id, false);
    id
}

/// The name and abbreviation of a live guild, as `GetProperties` shows them.
fn live_guild_names(w: &World, guild: Option<GuildId>) -> Option<(String, String)> {
    let g = muo_server::guild::guild(w, guild?).filter(|g| !g.disbanded)?;
    let name = g.name.trim();
    let name = if name.is_empty() { "(unnamed)" } else { name };
    Some((name.to_string(), g.abbreviation.trim().to_string()))
}

fn add_guild_property(w: &World, id: ItemId, list: &mut muo_server::property_list::PropertyList) {
    let s = stone(w, id);
    if let Some((name, abbr)) = live_guild_names(w, s.guild) {
        list.add_str(
            1060802,
            &format!(
                "{} [{}]",
                muo_primitives::utility::fix_html(&name),
                muo_primitives::utility::fix_html(&abbr)
            ),
        );
    } else if let (Some(n), Some(a)) = (&s.guild_name, &s.guild_abbrev) {
        list.add_str(
            1060802,
            &format!("{} [{}]", utility::fix_html(n), utility::fix_html(a)),
        );
    }
}

fn serialize_stone_fields(w: &World, id: ItemId, out: &mut dyn GenericWriter) {
    let s = stone(w, id);
    out.write_opt_string(s.guild_name.as_deref());
    out.write_opt_string(s.guild_abbrev.as_deref());
    out.write_guild(w, s.guild);
}

/// `GuildGump.DisplayTo(from, guild)`.
fn display_guild_gump(w: &mut World, from: MobileId, guild: GuildId) {
    muo_gumps::a_l::guilds::GuildGump::display_to(w, from, guild);
}

/// `Guildstone.Deed`: `new GuildstoneDeed(Guild, _guildName, _guildAbbrev)`.
fn stone_deed(w: &mut World, id: ItemId) -> ItemId {
    let s = stone(w, id);
    guildstone_deed_new(w, &GUILDSTONE_DEED, s.guild, s.guild_name, s.guild_abbrev)
}

/// `Guildstone.OnChop(from)` (`IChoppable`): true when `id` is a guildstone.
pub fn guildstone_chop(w: &mut World, id: ItemId, from: MobileId) -> bool {
    if !w.item(id).is_some_and(|i| i.ty.is(&GUILDSTONE)) {
        return false;
    }
    if !muo_content::misc::guild::new_guild_system(w) {
        return true;
    }
    let Some(house) = muo_multis::house::access::find_house_at_item(w, id) else {
        return true;
    };
    let in_addons = muo_multis::house::data(w, house).is_some_and(|h| h.addons.contains(&id));
    if is_owner(w, house, from) && in_addons {
        if let Some(h) = muo_multis::house::data_mut(w, house) {
            h.addons.retain(|a| *a != id);
        }
        let loc = ops::get_world_location(w, id);
        let map = w.item(id).and_then(|i| i.map);
        muo_server::effects::play_sound(w, loc, map, 0x3B3);
        deps::send_localized_message(w, from, 500461); // You destroy the item.
        let deed = stone_deed(w, id);
        delete_item(w, id);
        deps::add_to_backpack(w, from, deed);
    }
    true
}

/// `Server.Items.Guildstone`: `[SerializationGenerator(4, false)]`.
pub static GUILDSTONE: ItemType = ItemType {
    full_name: "Server.Items.Guildstone",
    parent: Some(&ITEM_BASE),
    // The two `Guildstone` constructors are not `[Constructible]`: `[add Guildstone` builds nothing.
    construct: None,
    label_number: |_, _| 1041429, // a guildstone
    get_properties: |w, id, list| {
        base::get_properties_base(w, id, list);
        add_guild_property(w, id, list);
    },
    on_single_click: |w, id, from| {
        base::on_single_click_base(w, id, from);
        let s = stone(w, id);
        if let Some((name, _)) = live_guild_names(w, s.guild) {
            ops::label_to(w, id, from, &name);
        } else if let Some(n) = &s.guild_name {
            ops::label_to(w, id, from, n);
        }
    },
    on_after_delete: |w, id| {
        let s = stone(w, id);
        let live = s
            .guild
            .and_then(|g| muo_server::guild::guild(w, g))
            .is_some_and(|g| !g.disbanded);
        if !muo_content::misc::guild::new_guild_system(w)
            && live
            && let Some(g) = s.guild
        {
            muo_content::misc::guild_class::disband(w, g);
        }
    },
    on_double_click: |w, id, from| {
        if muo_content::misc::guild::new_guild_system(w) {
            return;
        }
        let s = stone(w, id);
        let live = s
            .guild
            .and_then(|g| muo_server::guild::guild(w, g))
            .is_some_and(|g| !g.disbanded);
        let loc = ops::get_world_location(w, id);
        if !live {
            delete_item(w, id);
        } else if !deps::mobile_in_range(w, from, loc, 2) {
            deps::send_localized_message(w, from, 500446); // That is too far away.
        } else {
            use muo_content::misc::guild_class as gc;
            let Some(guild) = s.guild else { return };
            let accepted = gc::data(w, guild).is_some_and(|d| d.accepted.contains(&from));
            if accepted {
                // `PlayerState.Find(..).Faction`, `IsLeaving`, `Leaving`: the factions engine's. Unset, both
                // sides have no faction and the member joins.
                let leader = gc::data(w, guild).and_then(|d| d.leader);
                if let Some(f) = w
                    .resources
                    .get::<GuildstoneHooks>()
                    .and_then(|h| h.join_check)
                    && !f(w, leader, from)
                {
                    return;
                }
                if let Some(d) = gc::data_mut(w, guild) {
                    d.accepted.retain(|m| *m != from);
                }
                gc::add_member(w, guild, from);
                display_guild_gump(w, from, guild);
            } else {
                let is_member = gc::is_member(w, guild, from);
                let gm = w
                    .mobile(from)
                    .is_some_and(|m| m.access_level >= AccessLevel::GameMaster);
                if !gm && !is_member {
                    if let Some(ns) = deps::net_state_of(w, from) {
                        let (serial, graphic) =
                            w.item(id).map_or((muo_primitives::Serial::new(0), 0), |i| {
                                (i.serial, i.item_id)
                            });
                        // You are not a member ...
                        muo_server::packets::message::send_message_localized(
                            w,
                            ns,
                            serial,
                            graphic,
                            muo_server::packets::message::MessageType::REGULAR,
                            0x3B2,
                            3,
                            501158,
                            "",
                            "",
                        );
                    }
                } else {
                    display_guild_gump(w, from, guild);
                }
            }
        }
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(4);
        serialize_stone_fields(w, id, out);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let version = r.read_i32()?;
        let s = if version < 4 {
            r.read_bool()?; // Before Change Over
            StoneData {
                guild_name: r.read_string()?,
                guild_abbrev: r.read_string()?,
                guild: r.read_guild(w)?,
            }
        } else {
            StoneData {
                guild_name: r.read_string()?,
                guild_abbrev: r.read_string()?,
                guild: r.read_guild(w)?,
            }
        };
        let guild = s.guild;
        if let Some(i) = w.item_mut(id) {
            i.ext = Ext::new(s);
        }
        // [AfterDeserialization(false)]
        muo_server::timer::start(
            w,
            muo_primitives::TimeSpan::ZERO,
            muo_primitives::TimeSpan::ZERO,
            1,
            move |w| {
                let new_system = muo_content::misc::guild::new_guild_system(w);
                if new_system && w.item(id).is_some_and(|i| i.item_id == 0xED4) {
                    crate::misc::item_id(w, id, 0xED6);
                }
                if !new_system && guild.is_none() {
                    delete_item(w, id);
                }
            },
        );
        Ok(())
    },
    ..ITEM_BASE
};

fn deed_double_click(w: &mut World, id: ItemId, from: MobileId) {
    if !in_backpack(w, id, from) {
        deps::send_localized_message(w, from, 1042001); // That must be in your pack for you to use it.
        return;
    }
    let owner = house_at_mobile(w, from).is_some_and(|h| is_owner(w, h, from));
    if owner {
        deps::send_localized_message(w, from, 1062838); // Where would you like to place this decoration?
        muo_server::target::begin_target(
            w,
            from,
            -1,
            true,
            TargetFlags::NONE,
            move |w, from, t| deed_placement(w, id, from, t),
        );
    } else {
        deps::send_localized_message(w, from, 502092); // You must be in your house to do this.
    }
}

fn deed_placement(
    w: &mut World,
    id: ItemId,
    from: MobileId,
    targeted: muo_server::target::Targeted,
) {
    use muo_server::target::Targeted;
    let loc = match targeted {
        Targeted::Static(s) => s.location,
        Targeted::Land(l) => l.location,
        Targeted::Entity(EntityId::Item(i)) => ops::get_world_location(w, i),
        Targeted::Entity(EntityId::Mobile(m)) => w.mobile(m).map_or(Point3D::ZERO, |m| m.location),
    };
    if ops::is_deleted(w, id) {
        return;
    }
    if !in_backpack(w, id, from) {
        deps::send_localized_message(w, from, 1042001); // That must be in your pack for you to use it.
        return;
    }
    let map = w.mobile(from).and_then(|m| m.map);
    let house = muo_multis::house::access::find_house_at_point(w, loc, map, 16);
    match house.filter(|h| is_owner(w, *h, from)) {
        Some(house) => {
            let s = stone(w, id);
            let addon = guildstone_new(w, &GUILDSTONE, s.guild, s.guild_name, s.guild_abbrev);
            ops::move_to_world_map(w, addon, loc, map);
            if let Some(h) = muo_multis::house::data_mut(w, house) {
                h.addons.push(addon);
            }
            delete_item(w, id);
        }
        None => deps::send_localized_message(w, from, 1042036), // That location is not in your house.
    }
}

/// `GuildstoneDeed(g, guildName, abbrev)`.
pub fn guildstone_deed_new(
    w: &mut World,
    ty: &'static ItemType,
    guild: Option<GuildId>,
    name: Option<String>,
    abbrev: Option<String>,
) -> ItemId {
    let id = ops::new_item_with_id(w, ty, 0x14F0);
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(StoneData {
            guild_name: name,
            guild_abbrev: abbrev,
            guild,
        });
    }
    id
}

/// `Server.Items.GuildstoneDeed`; `[Flippable(0x14F0, 0x14EF)]`.
pub static GUILDSTONE_DEED: ItemType = ItemType {
    full_name: "Server.Items.GuildstoneDeed",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| guildstone_deed_new(w, &GUILDSTONE_DEED, None, None, None)), // the all-default constructor
    default_weight: |_, _| 1.0,
    label_number: |_, _| 1041233, // deed to a guildstone
    get_properties: |w, id, list| {
        base::get_properties_base(w, id, list);
        add_guild_property(w, id, list);
    },
    on_double_click: deed_double_click,
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        write_version(out, false);
        serialize_stone_fields(w, id, out);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        read_version(r, false)?;
        let s = StoneData {
            guild_name: r.read_string()?,
            guild_abbrev: r.read_string()?,
            guild: r.read_guild(w)?,
        };
        if let Some(i) = w.item_mut(id) {
            i.ext = Ext::new(s);
        }
        Ok(())
    },
    ..ITEM_BASE
};

/// `[Flippable(0x14F0, 0x14EF)]` of `GuildstoneDeed`.
pub static FLIPPABLE: &[(&ItemType, &[i32])] = &[(&GUILDSTONE_DEED, &[0x14F0, 0x14EF])];

/// The classes of this folder.
pub static ALL: &[&ItemType] = &[
    &GUILD_DEED,
    &GUILD_TELEPORTER,
    &GUILDSTONE,
    &GUILDSTONE_DEED,
];

fn never_parses(_: &str) -> Option<muo_content::utilities::types::Parsed> {
    None
}

/// `[Constructible] GuildstoneDeed(Guild g = null, string guildName = null, string abbrev = null)`; a `Guild`
/// cannot be parsed from `[add` text, so only the forms that leave it out build anything.
static GUILDSTONE_DEED_CTORS: [Ctor; 1] = [Ctor {
    access: None,
    params: &[
        Param {
            name: "g",
            kind: Kind::Plain(PropType::Parsable(never_parses)),
            default: Some(Value::Null),
            ty: "Guild",
        },
        Param {
            name: "guildName",
            kind: Kind::Plain(PropType::Str),
            default: Some(Value::Null),
            ty: "",
        },
        Param {
            name: "abbrev",
            kind: Kind::Plain(PropType::Str),
            default: Some(Value::Null),
            ty: "",
        },
    ],
    make: |w, v| {
        EntityId::Item(guildstone_deed_new(
            w,
            &GUILDSTONE_DEED,
            None,
            v[1].string(),
            v[2].string(),
        ))
    },
}];

/// `[Constructible] GuildTeleporter(Item stone = null)`.
static GUILD_TELEPORTER_CTORS: [Ctor; 1] = [Ctor {
    access: None,
    params: &[Param {
        name: "stone",
        kind: Kind::Plain(PropType::Entity),
        default: Some(Value::Null),
        ty: "Item",
    }],
    make: |w, v| {
        let stone = match v[0].entity() {
            Some(EntityId::Item(i)) => Some(i),
            _ => None,
        };
        EntityId::Item(guild_teleporter_new(w, &GUILD_TELEPORTER, stone))
    },
}];

/// `Guild.GuildProps_OnCommand`'s ending: `from.SendGump(new PropertiesGump(from, g))`, and the guild gump for a
/// game master when `NewGuildSystem`. A guild is not an entity: its table (`props::GUILD`) reads the guild of its
/// stone, so the gump hosts it over `Guild.Guildstone`.
fn show_guild_props(w: &mut World, from: MobileId, g: GuildId) {
    let stone = muo_content::misc::guild_class::data(w, g).and_then(|d| d.guildstone);
    let obj = match stone {
        Some(s) => muo_gumps::m_z::props::registry::SubObj::object(
            EntityId::Item(s),
            props::GUILD,
            "Guild",
        ),
        // no stone: the table reads the guild by `sub_key`, with the viewer as the (unused) owner
        None => muo_gumps::m_z::props::registry::SubObj::keyed(
            EntityId::Mobile(from),
            props::GUILD,
            "Guild",
            g.0 as usize,
        ),
    };
    let gump = muo_gumps::m_z::props::properties::new(w, from, obj);
    muo_gumps::m_z::common::send(w, from, gump);
    if muo_content::misc::guild::new_guild_system(w)
        && muo_content::mx::access_level(w, from) >= AccessLevel::GameMaster
        && muo_content::player_mobile::data(w, from).is_some()
    {
        muo_gumps::a_l::new_guilds::open_info_gump(w, from, g);
    }
}

/// `[GuildProps [serial|abbreviation|name]`.
fn guild_props_on_command(w: &mut World, e: &mut muo_server::commands::CommandEventArgs) {
    use muo_server::target::{self, Target, Targeted};
    let arg = e.arg_string.trim().to_owned();
    let from = e.mobile;
    if arg.is_empty() {
        let t = Target::simple(w, -1, TargetFlags::NONE, true, |w, from, o| {
            let entity = match o {
                Targeted::Entity(e) => Some(e),
                _ => None,
            };
            if !muo_content::targets::is_accessible(w, from, entity) {
                muo_server::mobile::part2::messages::send_localized_message(
                    w, from, 500447, "", 0x3B2,
                ); // That is not accessible.
                return;
            }
            let mut g = None;
            match entity {
                Some(EntityId::Item(i)) if w.item(i).is_some_and(|x| x.ty.is(&GUILDSTONE)) => {
                    let sg = stone_guild(w, i);
                    if sg.is_none_or(|g| muo_content::misc::guild_class::disbanded(w, g)) {
                        muo_content::mx::send_message(
                            w,
                            from,
                            "The guild associated with that Guildstone no longer exists",
                        );
                        return;
                    }
                    g = sg;
                }
                Some(EntityId::Mobile(m)) => g = muo_content::misc::guild_class::mobile_guild(w, m),
                _ => {}
            }
            let Some(g) = g else {
                muo_content::mx::send_message(w, from, "That is not in a guild!");
                return;
            };
            show_guild_props(w, from, g);
        });
        let id = target::add_target(w, t);
        target::set_mobile_target(w, from, Some(id));
        return;
    }
    let g = match arg.parse::<u32>() {
        Ok(id) => muo_server::guild::find_guild(w, muo_primitives::Serial::new(id)),
        Err(_) => muo_server::guild::world_find_by_abbrev(w, &arg)
            .or_else(|| muo_server::guild::world_find_by_name(w, &arg)),
    };
    if let Some(g) = g {
        show_guild_props(w, from, g);
    }
}

pub(crate) fn guilds_configure(w: &mut World) {
    muo_server::commands::CommandSystem::register(
        w,
        "GuildProps",
        AccessLevel::Counselor,
        &[],
        std::rc::Rc::new(guild_props_on_command),
    );
    add_ctors(w, "Server.Items.GuildstoneDeed", &GUILDSTONE_DEED_CTORS);
    add_ctors(w, "Server.Items.GuildTeleporter", &GUILD_TELEPORTER_CTORS);
    muo_content::items::common::hooks_mut(w)
        .choppables
        .push(guildstone_chop);
    // `BaseHouse.FindGuildstone`: the first `Guildstone` in the house's bounds.
    muo_content::items::house_deps::deps_mut(w).find_guildstone = Some(|w, map, bounds| {
        muo_server::map::enumerate::items_in_bounds(w, Some(map), bounds, false)
            .find(|&i| w.item(i).is_some_and(|x| x.ty.is(&GUILDSTONE)))
    });
}

/// Registers the classes of this folder.
pub fn register(types: &mut TypeRegistry) {
    types.register_phase_default(
        muo_server::type_registry::Phase::Configure,
        "muo_items.GuildProps.Configure",
        props::configure,
    );
    types.register_phase_default(
        muo_server::type_registry::Phase::Configure,
        "Server.Items.Guildstone.Configure",
        guilds_configure,
    );
    for t in ALL {
        types.register_item(t, &[]);
    }
}
