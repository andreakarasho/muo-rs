//! The Treasures of Tokuno gumps: `ToTTurnInGump` and `ToTRedeemGump`
//! (both `BaseImageTileButtonsGump`s, 2 x 5 tiles), with `ItemTileButtonInfo`, `TypeTileButtonInfo` and
//! `PigmentsTileButtonInfo`.

use muo_content::gumps::base::{LegacyGumpLogic, LegacyGumpObj, RelayInfo};
use muo_content::gumps::{close_gump, has_gump, send_gump_to_mobile};
use muo_content::player_mobile as pmob;
use muo_gumps::a_l::base_image_tile_buttons::{ImageTileButtonInfo, adjusted_button, build};
use muo_io::text::TextDefinition;
use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::net_state::NetStateId;

use super::pigments::{self, get_info};
use super::{Era, ITEMS_PER_REWARD, reward_era};

/// `TypeTileButtonInfo`: class, graphic, hue, label cliloc, tooltip.
pub struct TypeTile(pub &'static str, pub i32, pub i32, pub i32, pub i32);

const fn t(ty: &'static str, item_id: i32, label: i32, tip: i32) -> TypeTile {
    TypeTile(ty, item_id, 0, label, tip)
}

/// `ToTRedeemGump.NormalRewards`: ToT One and Two have the same ten, Three has nine.
static NORMAL_ONE: [TypeTile; 10] = [
    t("Server.Items.SwordsOfProsperity", 0x27A9, 1070963, 1071002),
    t("Server.Items.SwordOfTheStampede", 0x27A2, 1070964, 1070978),
    t("Server.Items.WindsEdge", 0x27A3, 1070965, 1071003),
    t("Server.Items.DarkenedSky", 0x27AD, 1070966, 1071004),
    t("Server.Items.TheHorselord", 0x27A5, 1070967, 1071005),
    t("Server.Items.RuneBeetleCarapace", 0x277D, 1070968, 1071006),
    t("Server.Items.KasaOfTheRajin", 0x2798, 1070969, 1071007),
    t("Server.Items.Stormgrip", 0x2792, 1070970, 1071008),
    TypeTile(
        "Server.Items.TomeOfLostKnowledge",
        0x0EFA,
        0x530,
        1070971,
        1071009,
    ),
    t("Server.Items.PigmentsOfTokuno", 0x0EFF, 1070933, 1071011),
];

/// `PigmentRewards` as `PigmentType` numbers.
static PIGMENTS_ONE: [i32; 10] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
static PIGMENTS_TWO: [i32; 9] = [11, 12, 13, 14, 15, 16, 17, 18, 19];

fn normal_rewards(era: Era) -> &'static [TypeTile] {
    match era {
        Era::ToTOne | Era::ToTTwo => &NORMAL_ONE,
        Era::ToTThree => &NORMAL_ONE[..9],
        Era::None => &[],
    }
}

fn pigment_rewards(era: Era) -> &'static [i32] {
    match era {
        Era::ToTOne | Era::ToTThree => &PIGMENTS_ONE,
        Era::ToTTwo => &PIGMENTS_TWO,
        Era::None => &[],
    }
}

/// `TreasuresOfTokuno.GreaterArtifacts` for `era`.
pub fn greater_artifacts(era: Era) -> Vec<&'static str> {
    normal_rewards(era).iter().map(|t| t.0).collect()
}

fn mobile_of(w: &World, ns: NetStateId) -> Option<MobileId> {
    w.net_states.get(&ns).and_then(|n| n.mobile)
}

fn say_count(w: &mut World, collector: MobileId, pm: MobileId) {
    let turned = pmob::data(w, pm).map_or(0, |d| d.tot_items_turned_in);
    // You have turned in ~1_COUNT~ minor artifacts. Turn in ~2_NUM~ to receive a reward.
    muo_server::mobile::part2::speech::say_to_number(
        w,
        collector,
        pm,
        1070981,
        &format!("{turned}\t{ITEMS_PER_REWARD}"),
    );
}

fn say(w: &mut World, collector: MobileId, pm: MobileId, number: i32) {
    muo_server::mobile::part2::speech::say_to_number(w, collector, pm, number, "");
}

fn turned_in(w: &World, pm: MobileId) -> i32 {
    pmob::data(w, pm).map_or(0, |d| d.tot_items_turned_in)
}

fn in_collector_range(w: &World, pm: MobileId, collector: MobileId) -> bool {
    muo_content::mx::in_range_point(w, pm, muo_content::mx::location(w, collector), 7)
}

/// `HandleCancel` of both gumps.
fn handle_cancel(w: &mut World, collector: MobileId, sender: NetStateId, redeem: bool) {
    let Some(pm) = mobile_of(w, sender) else {
        return;
    };
    if pmob::data(w, pm).is_none() || !in_collector_range(w, pm, collector) {
        return;
    }
    let turned = turned_in(w, pm);
    if turned == 0 {
        say(w, collector, pm, 1071013); // Bring me 10 of the lost treasures of Tokuno and I will reward you with a valuable item.
    } else if turned < ITEMS_PER_REWARD {
        // This case should ALWAYS be true with the turn-in gump, just a sanity check.
        say_count(w, collector, pm);
    } else if !redeem {
        say(w, collector, pm, 1070982); // When you wish to choose your reward, you have but to approach me again.
    } else {
        say(w, collector, pm, 1070982);
    }
}

// ---- ToTTurnInGump ----

pub struct ToTTurnInGump {
    collector: MobileId,
    items: Vec<ItemId>,
}

/// `new ToTTurnInGump(collector, buttons)`.
pub fn turn_in_gump(
    w: &World,
    collector: MobileId,
    buttons: Vec<ItemId>,
) -> LegacyGumpObj<ToTTurnInGump> {
    let infos: Vec<ImageTileButtonInfo> = buttons.iter().map(|&i| item_tile(w, i)).collect();
    let mut g = LegacyGumpObj::new(
        10,
        10,
        ToTTurnInGump {
            collector,
            items: buttons,
        },
    );
    // Click a minor artifact to give it to Ihara Soko.
    build(
        &mut g.layout,
        &TextDefinition::of_number(1071012),
        &infos,
        2,
        5,
    );
    g
}

/// `new ItemTileButtonInfo(item)`: the item's graphic and hue, its name or else its label number.
fn item_tile(w: &World, i: ItemId) -> ImageTileButtonInfo {
    let Some(it) = w.item(i) else {
        return ImageTileButtonInfo::new(0, 0, TextDefinition::EMPTY);
    };
    let label = match it.name.as_deref() {
        Some(n) if !n.is_empty() => TextDefinition::of_text(n),
        _ => TextDefinition::of_number((it.ty.label_number)(w, i)),
    };
    ImageTileButtonInfo::new(it.item_id, it.hue, label)
}

/// `ToTTurnInGump.FindRedeemableItems(m)`: the backpack items that count as minor artifacts.
pub fn find_redeemable_items(w: &World, m: MobileId) -> Option<Vec<ItemId>> {
    let pack = w.mobile(m)?.backpack?;
    let mut out = Vec::new();
    for item in muo_server::item::enumerable::find_items(w, pack, true, |_, _| true) {
        let Some(it) = w.item(item) else { continue };
        if !super::in_names(it.ty, super::lists::LESSER_ARTIFACTS_TOTAL) {
            continue;
        }
        if super::in_names(it.ty, &["Server.Items.ChestOfHeirlooms"]) {
            let ok = muo_content::items::cont::ext(w, item)
                .is_some_and(|d| d.locked && d.trap_level == 10);
            if !ok {
                continue;
            }
        }
        if super::in_names(it.ty, &["Server.Items.PigmentsOfTokuno"])
            && pigments::pigment_type(w, item) != 0
        {
            continue;
        }
        out.push(item);
    }
    Some(out)
}

impl LegacyGumpLogic for ToTTurnInGump {
    const TYPE_NAME: &'static str = "Server.Gumps.ToTTurnInGump";

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        match adjusted_button(info.button_id, self.items.len()) {
            Some(i) => self.handle_button(w, sender, self.items[i]),
            None => handle_cancel(w, self.collector, sender, false),
        }
    }
}

impl ToTTurnInGump {
    fn handle_button(&self, w: &mut World, sender: NetStateId, item: ItemId) {
        let Some(pm) = mobile_of(w, sender) else {
            return;
        };
        let collector = self.collector;
        let pack = w.mobile(pm).and_then(|m| m.backpack).map(EntityId::Item);
        if pmob::data(w, pm).is_none()
            || !muo_server::item::ops::is_child_of(w, item, pack)
            || !in_collector_range(w, pm, collector)
        {
            return;
        }
        if let Some(t) = w.item(item).map(|i| i.ty) {
            (t.delete)(w, item);
        }
        let turned = {
            let Some(d) = pmob::data_mut(w, pm) else {
                return;
            };
            d.tot_items_turned_in += 1;
            d.tot_items_turned_in
        };
        if turned >= ITEMS_PER_REWARD {
            // Congratulations! You have turned in enough minor treasures to earn a greater reward.
            say(w, collector, pm, 1070980);
            close_turn_in(w, pm); // Sanity
            if !has_redeem(w, pm) {
                send_redeem(w, pm, collector, false);
            }
        } else {
            say_count(w, collector, pm);
            let buttons = find_redeemable_items(w, pm);
            close_turn_in(w, pm); // Sanity
            if let Some(b) = buttons.filter(|b| !b.is_empty()) {
                let g = turn_in_gump(w, collector, b);
                send_gump_to_mobile(w, pm, Box::new(g), false);
            }
        }
    }
}

fn net_state(w: &World, pm: MobileId) -> Option<NetStateId> {
    w.mobile(pm).and_then(|m| m.net_state)
}

/// `gumps.Close<ToTTurnInGump>()`.
pub fn close_turn_in(w: &mut World, pm: MobileId) {
    if let Some(ns) = net_state(w, pm) {
        close_gump::<ToTTurnInGump>(w, ns);
    }
}

/// `gumps.Close<ToTRedeemGump>()`.
pub fn close_redeem(w: &mut World, pm: MobileId) {
    if let Some(ns) = net_state(w, pm) {
        close_gump::<ToTRedeemGump>(w, ns);
    }
}

pub fn has_turn_in(w: &World, pm: MobileId) -> bool {
    net_state(w, pm).is_some_and(|ns| has_gump::<ToTTurnInGump>(w, ns))
}

pub fn has_redeem(w: &World, pm: MobileId) -> bool {
    net_state(w, pm).is_some_and(|ns| has_gump::<ToTRedeemGump>(w, ns))
}

// ---- ToTRedeemGump ----

enum Reward {
    Type(&'static str),
    Pigment(i32),
}

pub struct ToTRedeemGump {
    collector: MobileId,
    rewards: Vec<Reward>,
}

/// `new ToTRedeemGump(collector, pigments)`.
pub fn redeem_gump(w: &World, collector: MobileId, pigments: bool) -> LegacyGumpObj<ToTRedeemGump> {
    let era = reward_era(w);
    let (header, infos, rewards): (i32, Vec<ImageTileButtonInfo>, Vec<Reward>) = if pigments {
        let list = pigment_rewards(era);
        let infos = list
            .iter()
            .map(|&p| {
                let (h, label) = get_info(p);
                ImageTileButtonInfo::new(0xEFF, h, TextDefinition::of_number(label))
            })
            .collect();
        (
            1070986,
            infos,
            list.iter().map(|&p| Reward::Pigment(p)).collect(),
        )
    } else {
        let list = normal_rewards(era);
        let infos = list
            .iter()
            .map(|t| {
                let mut b = ImageTileButtonInfo::new(t.1, t.2, TextDefinition::of_number(t.3));
                b.localized_tooltip = t.4;
                b
            })
            .collect();
        (
            1070985,
            infos,
            list.iter().map(|t| Reward::Type(t.0)).collect(),
        )
    };
    let mut g = LegacyGumpObj::new(10, 10, ToTRedeemGump { collector, rewards });
    build(
        &mut g.layout,
        &TextDefinition::of_number(header),
        &infos,
        2,
        5,
    );
    g
}

/// `gumps.Send(new ToTRedeemGump(collector, pigments))`.
pub fn send_redeem(w: &mut World, pm: MobileId, collector: MobileId, pigments: bool) {
    let g = redeem_gump(w, collector, pigments);
    send_gump_to_mobile(w, pm, Box::new(g), false);
}

impl LegacyGumpLogic for ToTRedeemGump {
    const TYPE_NAME: &'static str = "Server.Gumps.ToTRedeemGump";

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        match adjusted_button(info.button_id, self.rewards.len()) {
            Some(i) => self.handle_button(w, sender, i),
            None => handle_cancel(w, self.collector, sender, true),
        }
    }
}

impl ToTRedeemGump {
    fn handle_button(&self, w: &mut World, sender: NetStateId, index: usize) {
        let Some(pm) = mobile_of(w, sender) else {
            return;
        };
        let collector = self.collector;
        if pmob::data(w, pm).is_none()
            || !in_collector_range(w, pm, collector)
            || turned_in(w, pm) < ITEMS_PER_REWARD
        {
            return;
        }
        let item = match &self.rewards[index] {
            Reward::Pigment(p) => Some(pigments::pigments_new(w, *p, None)),
            Reward::Type("Server.Items.PigmentsOfTokuno") => {
                // Special case of course.
                close_turn_in(w, pm); // Sanity
                let g = redeem_gump(w, collector, true);
                send_gump_to_mobile(w, pm, Box::new(g), true);
                return;
            }
            Reward::Type(name) => muo_content::factory::make_item(w, name),
        };
        let Some(item) = item else { return }; // Sanity
        if muo_server::mobile::part2::items::add_to_backpack(w, pm, item) {
            if let Some(d) = pmob::data_mut(w, pm) {
                d.tot_items_turned_in -= ITEMS_PER_REWARD;
            }
            let it = w.item(item);
            let args = match it.and_then(|i| i.name.clone()) {
                Some(n) if !n.is_empty() => n,
                _ => {
                    let ty = it.map(|i| i.ty);
                    format!("#{}", ty.map_or(0, |t| (t.label_number)(w, item)))
                }
            };
            // You have earned the gratitude of the Empire. I have placed the ~1_OBJTYPE~ in your backpack.
            muo_server::mobile::part2::speech::say_to_number(w, collector, pm, 1070984, &args);
        } else {
            if let Some(t) = w.item(item).map(|i| i.ty) {
                (t.delete)(w, item);
            }
            say(w, collector, pm, 500722); // You don't have enough room in your backpack!
            say(w, collector, pm, 1070982); // When you wish to choose your reward, you have but to approach me again.
        }
    }
}
