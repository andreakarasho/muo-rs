//! `ConfirmBreakCrystalGump` (`BaseImprisonedMobile`'s confirmation).
//!

use muo_content::creature::control;
use muo_content::gumps::base::{LegacyGumpLogic, LegacyGumpObj, RelayInfo};
use muo_server::World;
use muo_server::effects;
use muo_server::ids::{ItemId, MobileId};
use muo_server::net_state::NetStateId;
use muo_server::skills::{self, SkillName};

use super::base_confirm::{self, ConfirmHandler};
use super::hooks::hooks;

pub struct ConfirmBreakCrystalGump {
    item: ItemId,
}

/// `new ConfirmBreakCrystalGump(item)`.
pub fn new(item: ItemId) -> LegacyGumpObj<ConfirmBreakCrystalGump> {
    base_confirm::build(ConfirmBreakCrystalGump { item })
}

impl LegacyGumpLogic for ConfirmBreakCrystalGump {
    const TYPE_NAME: &'static str = "Server.Gumps.ConfirmBreakCrystalGump";

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        base_confirm::on_response(self, w, sender, info);
    }
}

impl ConfirmHandler for ConfirmBreakCrystalGump {
    // This statuette will be destroyed when its trapped creature is summoned. The creature will be bonded to you
    // but will disappear if released. <br><br>Do you wish to proceed?
    fn label_number(&self) -> i32 {
        1075084
    }

    fn confirm(&mut self, w: &mut World, from: MobileId) {
        let item = self.item;
        if w.item(item).is_none_or(|i| i.deleted()) {
            return;
        }
        let Some(summon_fn) = hooks(w).and_then(|h| h.imprisoned_summon) else {
            return;
        };
        let Some(summon) = summon_fn(w, item) else {
            return;
        };

        if !control::set_control_master(w, summon, Some(from)) {
            muo_server::mobile::part1::lifecycle::delete(w, summon);
        } else {
            muo_content::mx::msg(w, from, 1049666); // Your pet has bonded with you!

            let (loc, map) = (
                muo_content::mx::location(w, from),
                muo_content::mx::map(w, from),
            );
            if let Some(map) = map {
                muo_server::mobile::base::move_to_world(w, summon, loc, map);
            }
            control::set_is_bonded(w, summon, true);

            for s in [
                SkillName::Wrestling,
                SkillName::Tactics,
                SkillName::MagicResist,
                SkillName::Anatomy,
            ] {
                skills::set_base(w, summon, s, 100.0);
            }

            let (loc, map) = (
                muo_content::mx::location(w, summon),
                muo_content::mx::map(w, summon),
            );
            let sound = w.mobile(summon).map_or(0, |m| m.base_sound_id);
            effects::play_sound(w, loc, map, sound);
            let e = muo_server::item::effect_item::create_targeted(
                w,
                loc,
                map,
                muo_server::item::effect_item::DEFAULT_DURATION,
            );
            effects::send_location_particles(w, e, 0x3728, 1, 10, 0x26B6);

            if let Some(release) = hooks(w).and_then(|h| h.imprisoned_release) {
                release(w, item, from, summon);
            }
            muo_server::item::base::delete(w, item);
        }
    }
}
