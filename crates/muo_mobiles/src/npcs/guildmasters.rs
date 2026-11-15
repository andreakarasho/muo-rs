//! Guildmasters: `BaseGuildmaster` (a `BaseVendor` that signs players up for an NPC
//! guild) and its twelve classes.

use muo_content::creature::{BASE_CREATURE_C, CreatureType};
use muo_content::mx;
use muo_content::player_mobile::{self, NpcGuild};
use muo_content::vendor::{BASE_VENDOR, BASE_VENDOR_C, BASE_VENDOR_CLASS, VendorClass};
use muo_primitives::TimeSpan;
use muo_primitives::dotnet::InvariantCase;
use muo_server::context_menu_entry::ContextMenuEntry;
use muo_server::events::SpeechEventArgs;
use muo_server::ids::{ItemId, MobileId};
use muo_server::mobile::MobileType;
use muo_server::mobile::part2::speech;
use muo_server::packets::message::AffixType;
use muo_server::skills::SkillName;
use muo_server::type_registry::TypeRegistry;
use muo_server::world::World;

use crate::template_keys_npcs as keys;

/// The virtuals `BaseGuildmaster` adds.
pub struct GuildmasterClass {
    pub join_cost: i32,
    pub join_age: TimeSpan,
    pub join_game_age: TimeSpan,
    pub quit_age: TimeSpan,
    pub quit_game_age: TimeSpan,
    /// `CheckCustomReqs(pm)`.
    pub check_custom_reqs: fn(&mut World, MobileId, MobileId) -> bool,
    /// `SayWelcomeTo(m)`.
    pub say_welcome_to: fn(&mut World, MobileId, MobileId),
}

pub static BASE_GUILDMASTER_CLASS: GuildmasterClass = GuildmasterClass {
    join_cost: 500,
    join_age: TimeSpan::ZERO,
    join_game_age: TimeSpan::from_days(2),
    quit_age: TimeSpan::from_days(7),
    quit_game_age: TimeSpan::from_days(4),
    check_custom_reqs: |_, _, _| true,
    // Welcome to the guild! Thou shalt find that fellow members shall grant thee lower prices in shops.
    say_welcome_to: |w, id, m| speech::say_to_number(w, id, m, 1008054, ""),
};

fn gclass(w: &World, id: MobileId) -> &'static GuildmasterClass {
    muo_content::vendor::subclass::<GuildmasterClass>(w, id).unwrap_or(&BASE_GUILDMASTER_CLASS)
}

fn guild(w: &World, id: MobileId) -> NpcGuild {
    muo_content::vendor::class(w, id).map_or(NpcGuild::None, |c| (c.npc_guild)(w, id))
}

fn is_player(w: &World, m: MobileId) -> bool {
    player_mobile::data(w, m).is_some()
}

fn say_to(w: &mut World, id: MobileId, to: MobileId, n: i32) {
    speech::say_to_number(w, id, to, n, "");
}

/// `WasNamed(speech)`: the speech starts with the vendor's name, ignoring case.
fn was_named(w: &World, id: MobileId, speech: &str) -> bool {
    let name = w.mobile(id).and_then(|m| (m.ty.name)(w, id));
    name.is_some_and(|n| speech.lower_inv().starts_with(&n.lower_inv()))
}

fn handles_on_speech(w: &World, id: MobileId, from: MobileId) -> bool {
    mx::in_range_point(w, from, mx::location(w, id), 2)
        || (BASE_VENDOR.handles_on_speech)(w, id, from)
}

/// `SayPriceTo(m)`: the guild price as a localized affix.
fn say_price_to(w: &mut World, id: MobileId, m: MobileId) {
    let cost = gclass(w, id).join_cost;
    let name = w
        .mobile(id)
        .and_then(|x| (x.ty.name)(w, id))
        .unwrap_or_default();
    let Some(ns) = w.mobile(m).and_then(|x| x.net_state) else {
        return;
    };
    let (serial, body, hue) = w.mobile(id).map_or_else(Default::default, |x| {
        (x.serial, x.cur_body().body_id(), x.speech_hue)
    });
    muo_server::packets::message::send_message_localized_affix(
        w,
        ns,
        serial,
        body,
        muo_server::packets::message::MessageType::REGULAR,
        hue,
        3,
        1008052,
        &name,
        AffixType::APPEND,
        &cost.to_string(),
        "",
    );
}

fn too_young(w: &World, id: MobileId, pm: MobileId) -> bool {
    let c = gclass(w, id);
    let created = w.mobile(pm).map_or(w.now, |m| m.created);
    player_mobile::game_time(w, pm) < c.join_game_age || created + c.join_age > w.now
}

fn on_speech(w: &mut World, id: MobileId, e: &mut SpeechEventArgs) {
    let from = e.mobile;
    if !e.handled
        && is_player(w, from)
        && mx::in_range_point(w, from, mx::location(w, id), 2)
        && was_named(w, id, &e.speech)
    {
        let g = guild(w, id);
        let pm_guild = player_mobile::data(w, from).map_or(NpcGuild::None, |d| d.npc_guild);
        if e.has_keyword(0x0004) {
            // *join* | *member*
            if pm_guild == g {
                say_to(w, id, from, 501047); // Thou art already a member of our guild.
            } else if pm_guild != NpcGuild::None {
                say_to(w, id, from, 501046); // Thou must resign from thy other guild first.
            } else if too_young(w, id, from) {
                say_to(w, id, from, 501048); // You are too young to join my guild...
            } else if (gclass(w, id).check_custom_reqs)(w, id, from) {
                say_price_to(w, id, from);
            }
            e.handled = true;
        } else if e.has_keyword(0x0005) {
            // *resign* | *quit*
            let c = gclass(w, id);
            let (joined, game) = player_mobile::data(w, from)
                .map_or((w.now, TimeSpan::ZERO), |d| {
                    (d.npc_guild_join_time, d.npc_guild_game_time)
                });
            if pm_guild != g {
                say_to(w, id, from, 501052); // Thou dost not belong to my guild!
            } else if joined + c.quit_age > w.now
                || game + c.quit_game_age > player_mobile::game_time(w, from)
            {
                say_to(w, id, from, 501053); // You just joined my guild! You must wait a week to resign.
            } else {
                say_to(w, id, from, 501054); // I accept thy resignation.
                if let Some(d) = player_mobile::data_mut(w, from) {
                    d.npc_guild = NpcGuild::None;
                }
            }
            e.handled = true;
        }
    }
    (BASE_VENDOR.on_speech)(w, id, e);
}

fn on_gold_given(w: &mut World, id: MobileId, from: MobileId, dropped: ItemId) -> bool {
    let amount = w.item(dropped).map_or(0, |i| i.amount);
    if is_player(w, from) && amount == gclass(w, id).join_cost {
        let g = guild(w, id);
        let pm_guild = player_mobile::data(w, from).map_or(NpcGuild::None, |d| d.npc_guild);
        if pm_guild == g {
            say_to(w, id, from, 501047);
        } else if pm_guild != NpcGuild::None {
            say_to(w, id, from, 501046);
        } else if too_young(w, id, from) {
            say_to(w, id, from, 501048);
        } else if (gclass(w, id).check_custom_reqs)(w, id, from) {
            (gclass(w, id).say_welcome_to)(w, id, from);
            let (now, gt) = (w.now, player_mobile::game_time(w, from));
            if let Some(d) = player_mobile::data_mut(w, from) {
                d.npc_guild = g;
                d.npc_guild_join_time = now;
                d.npc_guild_game_time = gt;
            }
            if let Some(t) = w.item(dropped).map(|i| i.ty) {
                (t.delete)(w, dropped);
            }
            return true;
        }
        return false;
    }
    (BASE_CREATURE_C.on_gold_given)(w, id, from, dropped)
}

static BASE_GUILDMASTER_V: VendorClass = VendorClass {
    init_sb_info: |_, _| Vec::new(),
    is_active_vendor: |_, _| false,
    subclass: Some(&BASE_GUILDMASTER_CLASS),
    ..BASE_VENDOR_CLASS
};
static BASE_GUILDMASTER_C: CreatureType = CreatureType {
    on_gold_given,
    subclass: Some(&BASE_GUILDMASTER_V),
    ..BASE_VENDOR_C
};

/// `Server.Mobiles.BaseGuildmaster` (abstract).
pub static BASE_GUILDMASTER: MobileType = MobileType {
    full_name: "Server.Mobiles.BaseGuildmaster",
    parent: Some(&BASE_VENDOR),
    construct: None,
    family: Some(&BASE_GUILDMASTER_C),
    click_title: |_, _| false,
    handles_on_speech,
    on_speech,
    // `[SerializationGenerator(0, false)]`: a plain int version.
    serialize: |w, id, out| {
        (BASE_VENDOR.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (BASE_VENDOR.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        Ok(())
    },
    ..BASE_VENDOR
};

/// The members of a guildmaster class that are its template's props (`template_keys_npcs`).
static TEMPLATE_GUILDMASTER_V: VendorClass = VendorClass {
    init_sb_info: super::vendors::sb::from_props,
    npc_guild: keys::npc_guild,
    shoe_type: keys::shoe_type,
    get_shoe_hue: keys::shoe_hue,
    init_outfit: keys::init_outfit,
    ..BASE_GUILDMASTER_V
};

/// One guildmaster class, whose constructor is its template (`npcTitleByGender`, `skills`). `v`: fields of its
/// `VendorClass` (over the props-reading members), `g`: of its `GuildmasterClass`, `c`: of its `CreatureType`,
/// `m`: of its `MobileType`.
macro_rules! guildmaster {
    ($name:ident, $ctor:ident = $full:literal;
     v { $($vf:ident : $vv:expr,)* } g { $($gf:ident : $gv:expr,)* } c { $($cf:ident : $cv:expr,)* } m { $($mf:ident : $mv:expr,)* }) => {
        fn $ctor(w: &mut World) -> MobileId {
            super::vendors::npc::new_template_vendor(w, &$name)
        }
        mod $ctor {
            use super::*;
            pub static G: GuildmasterClass = GuildmasterClass { $($gf: $gv,)* ..BASE_GUILDMASTER_CLASS };
            pub static V: VendorClass = VendorClass {
                $($vf: $vv,)*
                subclass: Some(&G),
                ..TEMPLATE_GUILDMASTER_V
            };
            pub static C: CreatureType = CreatureType { $($cf: $cv,)* subclass: Some(&V), ..BASE_GUILDMASTER_C };
        }
        pub static $name: MobileType = MobileType {
            full_name: $full,
            parent: Some(&BASE_GUILDMASTER),
            construct: Some($ctor),
            family: Some(&$ctor::C),
            serialize: |w, id, out| {
                (BASE_GUILDMASTER.serialize)(w, id, out);
                out.write_i32(0);
            },
            deserialize: |w, id, r| {
                (BASE_GUILDMASTER.deserialize)(w, id, r)?;
                let _version = r.read_i32()?;
                Ok(())
            },
            $($mf: $mv,)*
            ..BASE_GUILDMASTER
        };
    };
}

guildmaster!(BARD_GUILDMASTER, bard_guildmaster_new = "Server.Mobiles.BardGuildmaster";
    v {} g {} c {} m {});

guildmaster!(BLACKSMITH_GUILDMASTER, blacksmith_guildmaster_new = "Server.Mobiles.BlacksmithGuildmaster";
    v {
        is_active_vendor: |_, _| true,
    }
    g {} c {} m { click_title: |_, _| true, });

guildmaster!(FISHER_GUILDMASTER, fisher_guildmaster_new = "Server.Mobiles.FisherGuildmaster";
    v {} g {} c {} m {});

guildmaster!(HEALER_GUILDMASTER, healer_guildmaster_new = "Server.Mobiles.HealerGuildmaster";
    v {} g {} c {} m {});

guildmaster!(MAGE_GUILDMASTER, mage_guildmaster_new = "Server.Mobiles.MageGuildmaster";
    v {} g {} c {} m {});

guildmaster!(MERCHANT_GUILDMASTER, merchant_guildmaster_new = "Server.Mobiles.MerchantGuildmaster";
    v {} g {} c {} m {});

guildmaster!(MINER_GUILDMASTER, miner_guildmaster_new = "Server.Mobiles.MinerGuildmaster";
    v {} g {} c {} m {});

guildmaster!(RANGER_GUILDMASTER, ranger_guildmaster_new = "Server.Mobiles.RangerGuildmaster";
    v {} g {} c {} m {});

guildmaster!(TAILOR_GUILDMASTER, tailor_guildmaster_new = "Server.Mobiles.TailorGuildmaster";
    v {} g {} c {} m {});

fn thief_check_custom_reqs(w: &mut World, id: MobileId, pm: MobileId) -> bool {
    if player_mobile::young(w, pm) {
        say_to(w, id, pm, 502089); // You cannot be a member of the Thieves' Guild while you are Young.
        return false;
    }
    if w.mobile(pm).is_some_and(|m| m.kills > 0) {
        say_to(w, id, pm, 501050); // This guild is for cunning thieves, not oafish cutthroats.
        return false;
    }
    if muo_server::skills::base(w, pm, SkillName::Stealing) < 60.0 {
        say_to(w, id, pm, 501051); // You must be at least a journeyman pickpocket to join this elite organization.
        return false;
    }
    true
}

fn thief_on_speech(w: &mut World, id: MobileId, e: &mut SpeechEventArgs) {
    let from = e.mobile;
    if !e.handled
        && is_player(w, from)
        && mx::in_range_point(w, from, mx::location(w, id), 2)
        && e.has_keyword(0x1F)
    {
        // *disguise*
        if player_mobile::data(w, from).is_some_and(|d| d.npc_guild == NpcGuild::ThievesGuild) {
            say_to(w, id, from, 501839); // That particular item costs 700 gold pieces.
        } else {
            say_to(w, id, from, 501838); // I don't know what you're talking about.
        }
        e.handled = true;
    }
    (BASE_GUILDMASTER.on_speech)(w, id, e);
}

fn thief_on_gold_given(w: &mut World, id: MobileId, from: MobileId, dropped: ItemId) -> bool {
    let amount = w.item(dropped).map_or(0, |i| i.amount);
    if is_player(w, from)
        && amount == 700
        && player_mobile::data(w, from).is_some_and(|d| d.npc_guild == NpcGuild::ThievesGuild)
    {
        if let Some(kit) = muo_content::factory::make_item(w, "Server.Items.DisguiseKit") {
            muo_server::mobile::part2::items::add_to_backpack(w, from, kit);
        }
        if let Some(t) = w.item(dropped).map(|i| i.ty) {
            (t.delete)(w, dropped);
        }
        return true;
    }
    on_gold_given(w, id, from, dropped)
}

guildmaster!(THIEF_GUILDMASTER, thief_guildmaster_new = "Server.Mobiles.ThiefGuildmaster";
    v {}
    g {
        join_age: TimeSpan::from_days(7),
        check_custom_reqs: thief_check_custom_reqs,
        // Welcome to the guild! Stay to the shadows, friend.
        say_welcome_to: |w, id, m| say_to(w, id, m, 1008053),
    }
    c { on_gold_given: thief_on_gold_given, }
    m { on_speech: thief_on_speech, });

fn tinker_context_entries(
    w: &mut World,
    id: MobileId,
    from: MobileId,
    list: &mut Vec<ContextMenuEntry>,
) {
    // RechargeEntry (6271, 6) of the weapon engraving tool; Core.ML only.
    if w.core_ml()
        && mx::alive(w, from)
        && let Some(f) = muo_content::vendor::hooks(w).and_then(|h| h.recharge_entry)
    {
        list.push(f(w, id, from));
    }
    (BASE_GUILDMASTER_C.add_custom_context_entries)(w, id, from, list);
}

guildmaster!(TINKER_GUILDMASTER, tinker_guildmaster_new = "Server.Mobiles.TinkerGuildmaster";
    v {} g {} c { add_custom_context_entries: tinker_context_entries, } m {});

guildmaster!(WARRIOR_GUILDMASTER, warrior_guildmaster_new = "Server.Mobiles.WarriorGuildmaster";
    v {} g {} c {} m {});

pub(crate) fn register(types: &mut TypeRegistry) {
    for t in all_types() {
        types.register_mobile(t, &[]);
    }
}

fn all_types() -> Vec<&'static MobileType> {
    vec![
        &BARD_GUILDMASTER,
        &BLACKSMITH_GUILDMASTER,
        &FISHER_GUILDMASTER,
        &HEALER_GUILDMASTER,
        &MAGE_GUILDMASTER,
        &MERCHANT_GUILDMASTER,
        &MINER_GUILDMASTER,
        &RANGER_GUILDMASTER,
        &TAILOR_GUILDMASTER,
        &THIEF_GUILDMASTER,
        &TINKER_GUILDMASTER,
        &WARRIOR_GUILDMASTER,
    ]
}

#[cfg(test)]
pub(crate) fn all() -> Vec<&'static MobileType> {
    all_types()
}
