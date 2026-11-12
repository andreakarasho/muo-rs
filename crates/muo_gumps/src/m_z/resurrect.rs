//! `ResurrectGump`.

use muo_content::gumps::base::{DynamicGumpLogic, DynamicGumpObj, RelayInfo};
use muo_content::gumps::builder::DynamicGumpBuilder;
use muo_content::misc::aos_helpers::scale;
use muo_content::player_mobile::tables::{MurderTable, VirtueTable};
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::layer::Layer;
use muo_server::net_state::NetStateId;
use muo_server::skills::SkillName;

use super::common::mobile_of;

/// `ResurrectMessage`.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ResurrectMessage {
    ChaosShrine = 0,
    VirtueShrine = 1,
    Healer = 2,
    Generic = 3,
}

pub const SHORT_MURDERS_FOR_STAT_LOSS: i32 = 5;

pub struct ResurrectGump {
    from_sacrifice: bool,
    healer: Option<MobileId>,
    hits_scalar: f64,
    resurrect_message: ResurrectMessage,
    price: i32,
    /// An `OnResponse` override that runs before the base (AnkhResurrectGump); false ends the response.
    prelude: Option<fn(&mut World, MobileId, i32) -> bool>,
}

/// `ResurrectGump.TryGiveStatLoss(player)`.
pub fn try_give_stat_loss(w: &mut World, player: MobileId) {
    let short_term_murders = w
        .resources
        .get::<MurderTable>()
        .and_then(|t| t.0.get(&player))
        .map_or(0, |c| c.short_term_murders);

    if muo_content::core::aos(w) || short_term_murders < SHORT_MURDERS_FOR_STAT_LOSS {
        return;
    }

    let loss = ((100.0
        - (4.0 + f64::from(short_term_murders) / f64::from(SHORT_MURDERS_FOR_STAT_LOSS)))
        / 100.0)
        .clamp(0.85, 0.95); // 5 to 15% loss
    let (raw_str, raw_int, raw_dex) = w
        .mobile(player)
        .map_or((0, 0, 0), |m| (m.str, m.int, m.dex));
    let loss_str = (f64::from(raw_str) * loss) as i32;
    let loss_int = (f64::from(raw_int) * loss) as i32;
    let loss_dex = (f64::from(raw_dex) * loss) as i32;

    use muo_server::mobile::part1::props_b as pb;
    if loss_str >= 10 {
        pb::set_raw_str(w, player, loss_str);
    }

    if loss_int >= 10 {
        pb::set_raw_int(w, player, loss_int);
    }

    if loss_dex >= 10 {
        pb::set_raw_dex(w, player, loss_dex);
    }

    let count = w
        .mobile(player)
        .and_then(|m| m.skills.as_ref())
        .map_or(0, |s| s.len());
    for s in 0..count {
        let Some(skill) = SkillName::from_i32(s as i32) else {
            continue;
        };
        let skill_loss =
            (f64::from(muo_server::skills::base_fixed(w, player, skill)) * loss) as i32;
        if skill_loss >= 350 {
            muo_server::skills::set_base_fixed(w, player, skill, skill_loss);
        }
    }
}

/// `new ResurrectGump(healer, msg, fromSacrifice, hitsScalar, price)`.
pub fn new(
    healer: Option<MobileId>,
    msg: ResurrectMessage,
    from_sacrifice: bool,
    hits_scalar: f64,
    price: i32,
) -> DynamicGumpObj<ResurrectGump> {
    DynamicGumpObj::new(
        100,
        0,
        ResurrectGump {
            from_sacrifice,
            healer,
            hits_scalar,
            resurrect_message: msg,
            price,
            prelude: None,
        },
    )
}

/// `new AnkhResurrectGump(owner, msg)`: `prelude(w, from, button)` is the override's part before `base.OnResponse`.
pub fn new_with_prelude(
    owner: MobileId,
    msg: ResurrectMessage,
    prelude: fn(&mut World, MobileId, i32) -> bool,
) -> DynamicGumpObj<ResurrectGump> {
    let mut g = new(Some(owner), msg, false, 0.0, 0);
    g.logic.prelude = Some(prelude);
    // the gump is the AnkhResurrectGump subclass
    g.core.type_id = muo_content::gumps::base::get_type_id(g.logic.gump_type_name());
    g
}

/// `new ResurrectGump(healer, hitsScalar)`.
pub fn new_with_scalar(
    healer: Option<MobileId>,
    hits_scalar: f64,
) -> DynamicGumpObj<ResurrectGump> {
    new(healer, ResurrectMessage::Generic, false, hits_scalar, 0)
}

/// `new ResurrectGump(healer, price)`.
pub fn new_with_price(healer: Option<MobileId>, price: i32) -> DynamicGumpObj<ResurrectGump> {
    new(healer, ResurrectMessage::Generic, false, 0.0, price)
}

impl DynamicGumpLogic for ResurrectGump {
    const TYPE_NAME: &'static str = "Server.Gumps.ResurrectGump";

    fn gump_type_name(&self) -> &'static str {
        if self.prelude.is_some() {
            "Server.Items.AnkhOfSacrificeComponent+AnkhResurrectGump"
        } else {
            Self::TYPE_NAME
        }
    }
    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        if self.price > 0 {
            self.build_priced_layout(b);
            return;
        }

        self.build_default_layout(b);
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        if let Some(p) = self.prelude {
            let Some(from) = mobile_of(w, sender) else {
                return;
            };
            if !p(w, from, info.button_id) {
                return;
            }
        }
        if self.price > 0 {
            if info.button_id != 2 || info.switches.first() != Some(&1) {
                return;
            }
        } else if info.button_id != 1 {
            return;
        }

        let Some(from) = mobile_of(w, sender) else {
            return;
        };

        let (loc, map) = w
            .mobile(from)
            .map_or((Default::default(), None), |m| (m.location, m.map));
        let fits = map.is_some_and(|map| {
            muo_server::map::fit::can_fit(w, map, loc.x, loc.y, loc.z, 16, false, false, true)
        });
        if !fits {
            muo_content::mx::msg(w, from, 502391); // Thou can not be resurrected there!
            return;
        }

        if self.price > 0 {
            if muo_content::vendor::banker::withdraw(w, from, self.price) {
                // ~1_AMOUNT~ gold has been withdrawn from your bank to cover the price of the healing.
                muo_content::mx::msg_args(w, from, 1060021, &self.price.to_string());

                // You have ~1_AMOUNT~ gold in cash remaining in your bank box.
                let balance = muo_content::vendor::banker::get_balance(w, from);
                muo_content::mx::msg_args(w, from, 1060022, &balance.to_string());
            } else {
                // Unfortunately, you do not have enough cash in your bank to cover the cost of the healing.
                muo_content::mx::msg(w, from, 1060020);
                return;
            }
        }

        muo_content::mx::play_sound(w, from, 0x214);
        muo_content::mx::fixed_effect(w, from, 0x376A, 10, 16);

        if let Some(ty) = w.mobile(from).map(|m| m.ty) {
            (ty.resurrect)(w, from);
        }

        if let Some(healer) = self.healer.filter(|&h| h != from) {
            // VirtueSystem.GetLevel(healer, VirtueName.Compassion)
            let v = w
                .resources
                .get::<VirtueTable>()
                .and_then(|t| t.0.get(&healer))
                .and_then(|c| c.values.as_ref())
                .and_then(|v| v.get(2).copied())
                .unwrap_or(0);
            let level = if v < 4000 {
                0
            } else if v >= 21000 {
                // GetMaxAmount(Compassion)
                3
            } else {
                (v + 9999) / 10000
            };

            let hits_max = muo_content::mx::hits_max(w, from);
            let hits = match level {
                1 => Some(scale(hits_max, 20)), // Seeker
                2 => Some(scale(hits_max, 40)), // Follower
                3 => Some(scale(hits_max, 80)), // Knight
                _ => None,
            };
            if let Some(hits) = hits {
                muo_server::mobile::part1::props_b::set_hits(w, from, hits);
            }
        }

        if muo_content::player_mobile::data(w, from).is_some() {
            if self.from_sacrifice {
                muo_content::player_mobile::tables::virtues_mut(w, from).available_resurrects -= 1;

                let pack = muo_server::mobile::part1::props_b::backpack(w, from);
                let corpse = w.mobile(from).and_then(|m| m.corpse);

                if let (Some(pack), Some(corpse)) = (pack, corpse) {
                    let items = w.item(corpse).map(|c| c.items.clone()).unwrap_or_default();
                    for &item in items.iter().rev() {
                        let (layer, movable) = w
                            .item(item)
                            .map_or((Layer::Invalid, false), |i| (i.layer, i.movable()));
                        if layer != Layer::Hair
                            && layer != Layer::FacialHair
                            && movable
                            && let Some(ty) = w.item(pack).map(|i| i.ty)
                        {
                            (ty.drop_item)(w, pack, item);
                        }
                    }
                }
            }

            try_give_stat_loss(w, from);
        }

        let fame = w.mobile(from).map_or(0, |m| m.fame);
        if fame > 0 {
            let amount = fame / 10;

            muo_content::misc::titles::award_fame(w, from, -amount, true);
        }

        if muo_content::mx::alive(w, from) && self.hits_scalar > 0.0 {
            let hits = (f64::from(muo_content::mx::hits_max(w, from)) * self.hits_scalar) as i32;
            muo_server::mobile::part1::props_b::set_hits(w, from, hits);
        }
    }
}

impl ResurrectGump {
    fn build_default_layout(&self, b: &mut DynamicGumpBuilder) {
        b.add_page(0);

        b.add_background(0, 0, 400, 350, 2600);

        b.add_html_localized(0, 20, 400, 35, 1011022, false, false); // <center>Resurrection</center>

        /* It is possible for you to be resurrected here by this healer. Do you wish to try?<br>
         * CONTINUE - You chose to try to come back to life now.<br>
         * CANCEL - You prefer to remain a ghost for now. */
        b.add_html_localized(
            50,
            55,
            300,
            140,
            1011023 + self.resurrect_message as i32,
            true,
            true,
        );

        b.add_button(200, 227, 4005, 4007, 0);
        b.add_html_localized(235, 230, 110, 35, 1011012, false, false); // CANCEL

        b.add_button(65, 227, 4005, 4007, 1);
        b.add_html_localized(100, 230, 110, 35, 1011011, false, false); // CONTINUE
    }

    fn build_priced_layout(&self, b: &mut DynamicGumpBuilder) {
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
        b.add_html_localized_color(65, 145, 300, 25, 1060015, 0x7FFF, false, false); // Grudgingly pay the money

        b.add_radio(30, 175, 9727, 9730, false, 0);
        b.add_html_localized_color(65, 178, 300, 25, 1060016, 0x7FFF, false, false); // I'd rather stay dead, you scoundrel!!!

        // Wishing to rejoin the living, are you?  I can restore your body... for a price of course...
        b.add_html_localized_color(30, 20, 360, 35, 1060017, 0x7FFF, false, false);

        // Do you accept the fee, which will be withdrawn from your bank?
        b.add_html_localized_color(30, 105, 345, 40, 1060018, 0x5B2D, false, false);

        b.add_image(65, 72, 5605, 0);

        b.add_image_tiled(80, 90, 200, 1, 9107);
        b.add_image_tiled(95, 92, 200, 1, 9157);

        b.add_label(90, 70, 1645, &self.price.to_string());
        b.add_html_localized_color(140, 70, 100, 25, 1023823, 0x7FFF, false, false); // gold coins

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
}
