//! The Summoning quest's items and mobiles (namespace `Server.Engines.Quests.Doom`).
//!
//! `ChylothStaff` derives `BlackStaff` and `SummoningAltar` derives `AbbatoirAddon` (both `muo_items` classes this crate
//! cannot reach): they stand on `Item` and `BaseAddon`.

use muo_content::gumps::{
    DynamicGumpBuilder, DynamicGumpLogic, DynamicGumpObj, RelayInfo, send_gump_to_mobile,
};
use muo_content::items::addon;
use muo_primitives::{Direction, Point3D, TimeSpan};
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, LootType, effect_item, ops};
use muo_server::layer::Layer;
use muo_server::map::MapId;
use muo_server::mobile::MobileType;
use muo_server::net_state::NetStateId;
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};

use super::core::{Quest, can_offer_quest, has_quest, load, with_quest};
use super::gumps::WHITE;
use super::quester::{BASE_QUESTER, QUESTER_C, QUESTER_OPS, QUESTER_VC, QuesterOps};
use super::summoning::*;
use super::util::{public_overhead, say_to_number, set_body};
use crate::{quest_item, quester};

fn quest(w: &World, player: MobileId) -> Option<Quest> {
    load(w, player).filter(|q| q.is(&QUEST))
}

fn is_player(w: &World, m: MobileId) -> bool {
    w.mobile(m)
        .is_some_and(|m| m.ty.is(&muo_content::player_mobile::PLAYER_MOBILE))
}

/// `Effects.SendLocationParticles(EffectItem.Create(loc, map, EffectItem.DefaultDuration), 0x3728, 10, 10, effect)`.
fn puff(w: &mut World, loc: Point3D, map: Option<MapId>, effect: i32) {
    let e = effect_item::create_targeted(w, loc, map, effect_item::DEFAULT_DURATION);
    muo_server::effects::send_location_particles(w, e, 0x3728, 10, 10, effect);
}

// ---- the plain items ----

quest_item! {
    /// `ChylothShroud`.
    CHYLOTH_SHROUD = "Server.Engines.Quests.Doom.ChylothShroud" : ITEM_BASE, |w| {
        let id = ops::new_item_with_id(w, &CHYLOTH_SHROUD, 0x204E);
        muo_server::item::base::set_hue(w, id, 0x846);
        muo_server::item::base::set_layer(w, id, Layer::OuterTorso);
        id
    };
}

quest_item! {
    /// `GoldenSkull`.
    GOLDEN_SKULL = "Server.Engines.Quests.Doom.GoldenSkull" : ITEM_BASE, |w| {
        let graphic = muo_primitives::utility::random_from(&mut w.rng, 0x1AE2, 3);
        let id = ops::new_item_with_id(w, &GOLDEN_SKULL, graphic);
        muo_server::item::base::set_hue(w, id, 0x8A5);
        ops::set_loot_type(w, id, LootType::Blessed);
        id
    };
    default_weight: |_, _| 1.0,
    label_number: |_, _| 1061619, // a golden skull
}

quest_item! {
    /// `GrandGrimoire`.
    GRAND_GRIMOIRE = "Server.Engines.Quests.Doom.GrandGrimoire" : ITEM_BASE, |w| {
        let id = ops::new_item_with_id(w, &GRAND_GRIMOIRE, 0xEFA);
        muo_server::item::base::set_hue(w, id, 0x835);
        muo_server::item::base::set_layer(w, id, Layer::OneHanded);
        ops::set_loot_type(w, id, LootType::Blessed);
        id
    };
    default_weight: |_, _| 1.0,
    label_number: |_, _| 1060801, // The Grand Grimoire
}

// ---- SummoningAltar ----

/// `SummoningAltar` (`AbbatoirAddon`): `[SerializationGenerator(0, false)]`, with `Daemon`.
pub static SUMMONING_ALTAR: ItemType = ItemType {
    full_name: "Server.Engines.Quests.Doom.SummoningAltar",
    parent: Some(&muo_items::addons::leaves::ABBATOIR_ADDON),
    construct: Some(|w| {
        let id = addon::new_addon(w, &SUMMONING_ALTAR);
        muo_items::addons::leaves::abbatoir_components(w, id);
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(AltarData::default());
        }
        id
    }),
    serialize: |w, id, out| {
        (muo_items::addons::leaves::ABBATOIR_ADDON.serialize)(w, id, out);
        out.write_i32(0);
        out.write_mobile(w, altar_daemon(w, id));
    },
    deserialize: |w, id, r| {
        (muo_items::addons::leaves::ABBATOIR_ADDON.deserialize)(w, id, r)?;
        r.read_i32()?;
        let daemon = r.read_mobile(w)?;
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(AltarData { daemon });
        }
        check_daemon(w, id); // [AfterDeserialization]
        Ok(())
    },
    ..muo_items::addons::leaves::ABBATOIR_ADDON
};

// ---- BellOfTheDead ----

/// `BellOfTheDead`'s `_chyloth`, `_dragon` and `Summoning`.
#[derive(Clone, Default)]
pub struct BellData {
    pub chyloth: Option<MobileId>,
    pub dragon: Option<MobileId>,
    pub summoning: bool,
}

fn bell(w: &World, id: ItemId) -> BellData {
    w.item(id)
        .and_then(|i| i.ext.get::<BellData>())
        .cloned()
        .unwrap_or_default()
}

fn set_bell(w: &mut World, id: ItemId, f: impl FnOnce(&mut BellData)) {
    if let Some(i) = w.item_mut(id) {
        if i.ext.get::<BellData>().is_none() {
            i.ext.insert(BellData::default());
        }
        if let Some(d) = i.ext.get_mut::<BellData>() {
            f(d);
        }
    }
}

fn alive_mobile(w: &World, m: Option<MobileId>) -> bool {
    m.is_some_and(|m| !muo_content::mx::deleted(w, m))
}

/// `BeginSummon(from)`.
fn begin_summon(w: &mut World, bell_id: ItemId, from: MobileId) {
    let b = bell(w, bell_id);
    if alive_mobile(w, b.chyloth) {
        muo_content::mx::msg(w, from, 1050010);
    } else if alive_mobile(w, b.dragon) {
        muo_content::mx::msg(w, from, 1050017);
    } else if !b.summoning {
        set_bell(w, bell_id, |d| d.summoning = true);
        let (loc, map) = (
            ops::get_world_location(w, bell_id),
            w.item(bell_id).and_then(|i| i.map),
        );
        muo_server::effects::play_sound(w, loc, map, 0x100);
        muo_server::timer::start(
            w,
            TimeSpan::from_secs_f64(8.0),
            TimeSpan::ZERO,
            1,
            move |w| end_summon(w, bell_id, from),
        );
    }
}

/// `EndSummon(from)`.
fn end_summon(w: &mut World, bell_id: ItemId, from: MobileId) {
    let b = bell(w, bell_id);
    if alive_mobile(w, b.chyloth) {
        muo_content::mx::msg(w, from, 1050010);
    } else if alive_mobile(w, b.dragon) {
        muo_content::mx::msg(w, from, 1050017);
    } else if b.summoning {
        set_bell(w, bell_id, |d| d.summoning = false);
        let mut loc = ops::get_world_location(w, bell_id);
        let map = w.item(bell_id).and_then(|i| i.map);
        loc.z -= 16;
        puff(w, loc, map, 2023);
        muo_server::effects::play_sound(w, loc, map, 0x1FE);
        let Some(chyloth) =
            muo_content::factory::make_mobile(w, "Server.Engines.Quests.Doom.Chyloth")
        else {
            return;
        };
        // Direction = (Direction)(7 & (4 + (int)from.GetDirectionTo(loc)))
        let to = muo_server::mobile::part2::effects::get_direction_to(w, from, loc.x, loc.y, false);
        let dir = Direction::from_bits_retain(7 & (4 + to.bits()));
        muo_content::mx::set_direction(w, chyloth, dir);
        muo_server::mobile::base::move_to_world(w, chyloth, loc, map.unwrap_or(MapId::INTERNAL));
        set_bell(w, bell_id, |d| d.chyloth = Some(chyloth));
        set_chyloth(w, chyloth, |c| {
            c.bell = Some(bell_id);
            c.angry_at = Some(from);
        });
        begin_give_warning(w, chyloth);
        begin_remove(w, chyloth, TimeSpan::from_secs(40));
    }
}

/// `Server.Engines.Quests.Doom.BellOfTheDead`.
pub static BELL_OF_THE_DEAD: ItemType = ItemType {
    full_name: "Server.Engines.Quests.Doom.BellOfTheDead",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let id = ops::new_item_with_id(w, &BELL_OF_THE_DEAD, 0);
        muo_content::templates::apply_item_as(w, id, BELL_OF_THE_DEAD.full_name);
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(BellData::default());
        }
        id
    }),
    label_number: |w, id| {
        crate::template_keys::item_int(w, id, "labelNumber", ITEM_BASE.label_number)
    },
    on_double_click: |w, id, from| {
        if muo_content::mx::in_range_point(w, from, ops::get_world_location(w, id), 2) {
            begin_summon(w, id, from);
        } else {
            super::util::local_overhead_number(w, from, 0x3B2, 1019045); // I can't reach that.
        }
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(0);
        let b = bell(w, id);
        out.write_mobile(w, b.chyloth);
        out.write_mobile(w, b.dragon);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        r.read_i32()?;
        let chyloth = r.read_mobile(w)?;
        let dragon = r.read_mobile(w)?;
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(BellData {
                chyloth,
                dragon,
                summoning: false,
            });
        }
        // [AfterDeserialization(false)]: _chyloth?.Delete()
        if let Some(c) = chyloth {
            muo_server::mobile::base::delete(w, c);
        }
        Ok(())
    },
    ..ITEM_BASE
};

// ---- Chyloth ----

/// `Chyloth`'s `Bell` and `AngryAt` (not saved).
#[derive(Clone, Default)]
pub struct ChylothData {
    pub bell: Option<ItemId>,
    pub angry_at: Option<MobileId>,
}

fn set_chyloth(w: &mut World, id: MobileId, f: impl FnOnce(&mut ChylothData)) {
    if let Some(m) = w.mobile_mut(id) {
        if m.ext.get::<ChylothData>().is_none() {
            m.ext.insert(ChylothData::default());
        }
        if let Some(d) = m.ext.get_mut::<ChylothData>() {
            f(d);
        }
    }
}

fn chyloth(w: &World, id: MobileId) -> ChylothData {
    w.mobile(id)
        .and_then(|m| m.ext.get::<ChylothData>())
        .cloned()
        .unwrap_or_default()
}

/// `BeginGiveWarning()`.
fn begin_give_warning(w: &mut World, id: MobileId) {
    if muo_content::mx::deleted(w, id) || chyloth(w, id).angry_at.is_none() {
        return;
    }
    muo_server::timer::start(w, TimeSpan::from_secs(4), TimeSpan::ZERO, 1, move |w| {
        end_give_warning(w, id)
    });
}

/// `EndGiveWarning()`.
fn end_give_warning(w: &mut World, id: MobileId) {
    let Some(angry) = chyloth(w, id)
        .angry_at
        .filter(|_| !muo_content::mx::deleted(w, id))
    else {
        return;
    };
    let name = w
        .mobile(angry)
        .and_then(|m| (m.ty.name)(w, angry))
        .unwrap_or_default();
    public_overhead(w, id, 0x3B2, 1050013, &name);
    public_overhead(w, id, 0x3B2, 1050014, ""); // Why have you disturbed me, mortal?!?
    begin_summon_dragon(w, id);
}

/// `BeginSummonDragon()`.
fn begin_summon_dragon(w: &mut World, id: MobileId) {
    if muo_content::mx::deleted(w, id) || chyloth(w, id).angry_at.is_none() {
        return;
    }
    muo_server::timer::start(w, TimeSpan::from_secs(30), TimeSpan::ZERO, 1, move |w| {
        end_summon_dragon(w, id)
    });
}

/// `BeginRemove(delay)`.
fn begin_remove(w: &mut World, id: MobileId, delay: TimeSpan) {
    muo_server::timer::start(w, delay, TimeSpan::ZERO, 1, move |w| end_remove(w, id));
}

/// `EndRemove()`.
fn end_remove(w: &mut World, id: MobileId) {
    if muo_content::mx::deleted(w, id) {
        return;
    }
    let (loc, map) = (
        muo_content::mx::location(w, id),
        muo_content::mx::map(w, id),
    );
    puff(w, loc, map, 2023);
    muo_server::effects::play_sound(w, loc, map, 0x1FE);
    muo_server::mobile::base::delete(w, id);
}

const OFFSETS: [i32; 16] = [-1, -1, -1, 0, -1, 1, 0, -1, 0, 1, 1, -1, 1, 0, 1, 1];

/// `EndSummonDragon()`.
fn end_summon_dragon(w: &mut World, id: MobileId) {
    let Some(angry) = chyloth(w, id)
        .angry_at
        .filter(|_| !muo_content::mx::deleted(w, id))
    else {
        return;
    };
    let Some(map) = muo_content::mx::map(w, angry) else {
        return;
    };
    let in_doom = w
        .mobile(angry)
        .is_some_and(|m| muo_server::region::is_part_of_name(w, m.region, "Doom", false));
    if !in_doom {
        return;
    }
    public_overhead(w, id, 0x3B2, 1050015, ""); // Feel the wrath of my legions!!!
    // A wee bit crazy, aren't we?
    muo_server::mobile::part2::messages::public_overhead_message(
        w,
        id,
        muo_primitives::MessageType::REGULAR,
        0x3B2,
        false,
        "MUHAHAHAHA HAHAH HAHA",
        false,
        muo_server::mobile::AccessLevel::Player,
    );

    let Some(dragon) = muo_content::factory::make_mobile(w, "Server.Mobiles.SkeletalDragon") else {
        return;
    };
    let offset = muo_content::hooks::rnd::below(w, 8) * 2;
    let aloc = muo_content::mx::location(w, angry);
    let mut found = false;
    let mut i = 0;
    while i < OFFSETS.len() {
        let x = aloc.x + OFFSETS[(offset as usize + i) % OFFSETS.len()];
        let y = aloc.y + OFFSETS[(offset as usize + i + 1) % OFFSETS.len()];
        if muo_server::map::fit::can_spawn_mobile_at(w, map, x, y, aloc.z) {
            muo_server::mobile::base::move_to_world(w, dragon, Point3D::new(x, y, aloc.z), map);
            found = true;
            break;
        }
        let z = w.maps.get_average_z(map, x, y);
        if muo_server::map::fit::can_spawn_mobile_at(w, map, x, y, z) {
            muo_server::mobile::base::move_to_world(w, dragon, Point3D::new(x, y, z), map);
            found = true;
            break;
        }
        i += 2;
    }
    if !found {
        muo_server::mobile::base::move_to_world(w, dragon, aloc, map);
    }
    muo_content::mx::set_combatant(w, dragon, Some(angry));
    if let Some(b) = chyloth(w, id).bell {
        set_bell(w, b, |d| d.dragon = Some(dragon));
    }
}

/// `Chyloth.TeleportToFerry(from)`.
pub fn teleport_to_ferry(w: &mut World, from: MobileId) {
    let loc = Point3D::new(408, 251, 2);
    let map = MapId::MALAS;
    puff(w, loc, Some(map), 2023);
    muo_server::effects::play_sound(w, loc, Some(map), 0x1FE);
    muo_content::creature::pets::teleport_pets(w, from, loc, map, false);
    muo_server::mobile::base::move_to_world(w, from, loc, map);
}

static CHYLOTH_OPS: QuesterOps = QuesterOps {
    can_talk_to: |_, _, _| false,
    ..QUESTER_OPS
};

quester! {
    /// `Server.Engines.Quests.Doom.Chyloth`.
    CHYLOTH, CHYLOTH_C, CHYLOTH_V = "Server.Engines.Quests.Doom.Chyloth" : BASE_QUESTER / QUESTER_C / QUESTER_VC,
        ops CHYLOTH_OPS, encoded false, title Some("the Ferryman"), extra |_, _| {};
    v {
        init_body: |w, id| {
            muo_server::mobile::part2::stats::init_stats(w, id, 100, 100, 25);
            muo_server::mobile::base::set_hue(w, id, 0x8455);
            set_body(w, id, 0x190);
        },
        init_outfit: |w, id| {
            super::util::equip_new(w, id, "Server.Engines.Quests.Doom.ChylothShroud");
            super::util::equip_new(w, id, "Server.Engines.Quests.Doom.ChylothStaff");
        },
    }
    c { default_name: |_, _| Some("Chyloth"), }
    m {
        on_drag_drop: |w, id, from, dropped| {
            if super::util::item_is(w, dropped, "Server.Engines.Quests.Doom.GoldenSkull") {
                muo_server::item::base::delete(w, dropped);
                let name = w.mobile(from).and_then(|m| (m.ty.name)(w, from)).unwrap_or_default();
                public_overhead(w, id, 0x3B2, 1050046, &name);
                begin_remove(w, id, TimeSpan::from_secs(4));
                if let Some(p) = crate::party::get(w, from) {
                    let members: Vec<MobileId> =
                        crate::party::state(w, p).map(|s| s.members.iter().map(|m| m.mobile).collect()).unwrap_or_default();
                    for member in members {
                        let in_doom = w.mobile(member).is_some_and(|m| {
                            m.map == Some(MapId::MALAS) && muo_server::region::is_part_of_name(w, m.region, "Doom", false)
                        });
                        if member != from && in_doom {
                            if chyloth(w, id).angry_at == Some(member) {
                                set_chyloth(w, id, |c| c.angry_at = None);
                            }
                            party_gump_display(w, member, from);
                        }
                    }
                }
                if chyloth(w, id).angry_at == Some(from) {
                    set_chyloth(w, id, |c| c.angry_at = None);
                }
                teleport_to_ferry(w, from);
                return false;
            }
            (BASE_QUESTER.on_drag_drop)(w, id, from, dropped)
        },
    }
}

/// `ChylothPartyGump`.
pub struct ChylothPartyLogic {
    leader: MobileId,
    /// `SetStringSlot("leader", _leader.Name)`.
    leader_name: String,
}

fn party_gump_display(w: &mut World, to: MobileId, leader: MobileId) {
    let leader_name = w
        .mobile(leader)
        .and_then(|m| (m.ty.name)(w, leader))
        .unwrap_or_default();
    send_gump_to_mobile(
        w,
        to,
        Box::new(DynamicGumpObj::new(
            150,
            50,
            ChylothPartyLogic {
                leader,
                leader_name,
            },
        )),
        false,
    );
}

impl DynamicGumpLogic for ChylothPartyLogic {
    const TYPE_NAME: &'static str = "Server.Engines.Quests.Doom.ChylothPartyGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        // a static gump would fill the "leader" string slot with the leader's name; drawn dynamically here
        b.set_no_close();
        b.add_page(0);
        b.add_image(0, 0, 3600, 0);
        b.add_image_tiled(0, 14, 15, 200, 3603);
        b.add_image_tiled(380, 14, 14, 200, 3605);
        b.add_image(0, 201, 3606, 0);
        b.add_image_tiled(15, 201, 370, 16, 3607);
        b.add_image_tiled(15, 0, 370, 16, 3601);
        b.add_image(380, 0, 3602, 0);
        b.add_image(380, 201, 3608, 0);
        b.add_image_tiled(15, 15, 365, 190, 2624);
        b.add_radio(30, 140, 9727, 9730, true, 1);
        b.add_html_localized_color(65, 145, 300, 25, 1050050, WHITE, false, false); // Yes, let's go!
        b.add_radio(30, 175, 9727, 9730, false, 0);
        b.add_html_localized_color(65, 178, 300, 25, 1050049, WHITE, false, false); // No thanks, I'd rather stay here.
        b.add_html_localized_color(30, 20, 360, 35, 1050047, WHITE, false, false);
        b.add_html_localized_color(30, 105, 345, 40, 1050048, 0x5B2D, false, false); // Do you wish to accept their invitation at this time?
        b.add_image(65, 72, 5605, 0);
        b.add_image_tiled(80, 90, 200, 1, 9107);
        b.add_image_tiled(95, 92, 200, 1, 9157);
        b.add_label(90, 70, 1645, &self.leader_name);
        b.add_button(290, 175, 247, 248, 2);
        b.add_image_tiled(15, 14, 365, 1, 9107);
        b.add_image_tiled(380, 14, 1, 190, 9105);
        b.add_image_tiled(15, 205, 365, 1, 9107);
        b.add_image_tiled(15, 14, 1, 190, 9105);
        b.add_image_tiled(0, 0, 395, 1, 9157);
        b.add_image_tiled(394, 0, 1, 217, 9155);
        b.add_image_tiled(0, 216, 395, 1, 9157);
        b.add_image_tiled(0, 0, 1, 217, 9155);
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let Some(member) = w.net_states.get(&sender).and_then(|n| n.mobile) else {
            return;
        };
        let name = w
            .mobile(member)
            .and_then(|m| (m.ty.name)(w, member))
            .unwrap_or_default();
        if info.button_id == 2 && info.is_switched(1) {
            let in_doom = w
                .mobile(member)
                .is_some_and(|m| muo_server::region::is_part_of_name(w, m.region, "Doom", false));
            if in_doom {
                muo_server::mobile::part2::messages::send_localized_message(
                    w,
                    self.leader,
                    1050054,
                    &name,
                    0x3B2,
                );
                teleport_to_ferry(w, member);
            } else {
                muo_content::mx::msg(w, member, 1050051); // The invitation has been revoked.
            }
        } else {
            muo_content::mx::msg(w, member, 1050052); // You have declined their invitation.
            muo_server::mobile::part2::messages::send_localized_message(
                w,
                self.leader,
                1050053,
                &name,
                0x3B2,
            );
        }
    }
}

// ---- Victoria ----

static VICTORIA_OPS: QuesterOps = QuesterOps {
    talk_number: 6159, // Ask about Chyloth
    can_talk_to: |w, _, to| !has_quest(w, to) && can_offer_quest(w, to, &QUEST),
    on_talk: |w, victoria, player, _| {
        if !has_quest(w, player) && can_offer_quest(w, player, &QUEST) {
            let d = muo_content::mx::get_direction_to(w, victoria, player);
            muo_content::mx::set_direction(w, victoria, d);
            new_quest(w, victoria, player).send_offer(w);
        }
    },
    ..QUESTER_OPS
};

quester! {
    /// `Server.Engines.Quests.Doom.Victoria`.
    VICTORIA, VICTORIA_C, VICTORIA_V = "Server.Engines.Quests.Doom.Victoria" : BASE_QUESTER / QUESTER_C / QUESTER_VC,
        ops VICTORIA_OPS, encoded false, title Some("the Sorceress"), extra |w, id| {
            if let Some(m) = w.mobile_mut(id) {
                m.ext.insert(VictoriaData::default());
            }
        };
    v {
        is_active_vendor: |_, _| true,
        init_sb_info: |w, id| vec![muo_mobiles::npcs::vendors::sb::SBMAGE.new_for(w, id)],
        init_body: |w, id| {
            muo_server::mobile::part2::stats::init_stats(w, id, 100, 100, 25);
            muo_server::mobile::part1::props_b::set_female(w, id, true);
            muo_server::mobile::base::set_hue(w, id, 0x8835);
            set_body(w, id, 0x191);
        },
        init_outfit: |w, id| {
            super::util::equip_new(w, id, "Server.Engines.Quests.Doom.GrandGrimoire");
            for (name, hue) in [("Sandals", 0x455), ("SkullCap", 0x455), ("PlainDress", 0x455)] {
                if let Some(it) = muo_content::factory::make_item(w, &format!("Server.Items.{name}")) {
                    super::quester::set_hue(w, it, hue);
                    if !muo_server::mobile::base::equip_item(w, id, it) {
                        muo_server::item::base::delete(w, it);
                    }
                }
            }
            muo_server::mobile::part1::props_b::set_hair_item_id(w, id, 0x203C);
            muo_server::mobile::part1::props_b::set_hair_hue(w, id, 0x482);
        },
    }
    c { default_name: |_, _| Some("Victoria"), disallow_all_moves: |_, _| false, }
    m {
        click_title: |_, _| true,
        on_drag_drop: |w, victoria, from, dropped| {
            let fallback = |w: &mut World| (BASE_QUESTER.on_drag_drop)(w, victoria, from, dropped);
            if !is_player(w, from) || quest(w, from).is_none() || !super::util::item_is(w, dropped, "Server.Items.DaemonBone") {
                return fallback(w);
            }
            let qs = quest(w, from).expect("checked");
            let obj = qs.find_objective(&COLLECT_BONES_OBJECTIVE);
            match obj.filter(|&i| !qs.objectives[i].completed(&qs)) {
                Some(i) => {
                    let need = qs.objectives[i].max_progress(&qs) - qs.objectives[i].cur_progress;
                    let have = super::util::amount_of(w, dropped);
                    if have < need {
                        with_quest(w, from, |w, q| {
                            let v = q.objectives[i].cur_progress + have;
                            q.set_progress(w, i, v);
                        });
                        muo_server::item::base::delete(w, dropped);
                        with_quest(w, from, |w, q| super::gumps::log_updated_display(w, q));
                    } else {
                        with_quest(w, from, |w, q| q.complete_objective(w, i));
                        muo_server::item::base::consume(w, dropped, need);
                        if !super::util::deleted_item(w, dropped) {
                            say_to_number(w, victoria, from, 1050038);
                        }
                    }
                }
                None => say_to_number(w, victoria, from, 1050038),
            }
            false
        },
    }
}

/// The mobile classes of this folder.
pub static MOBILES: &[&MobileType] = &[&CHYLOTH, &VICTORIA];

/// The item classes of this folder.
pub static ITEMS: &[&ItemType] = &[
    &BELL_OF_THE_DEAD,
    &CHYLOTH_SHROUD,
    &GOLDEN_SKULL,
    &GRAND_GRIMOIRE,
    &SUMMONING_ALTAR,
];
