//! The tracking skill gumps: `TrackWhatGump` and `TrackWhoGump`.

use muo_content::ai::transfer::{DEFAULT_ITEM_ID, shrink_lookup};
use muo_content::gumps::base::{
    DynamicGumpLogic, DynamicGumpObj, RelayInfo, StaticGumpLogic, StaticGumpObj,
};
use muo_content::gumps::builder::{DynamicGumpBuilder, StaticGumpBuilder};
use muo_content::gumps::close_gump;
use muo_content::skills::tracking::{self, TrackingHooks};
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::net_state::NetStateId;

use super::common::*;

pub struct TrackWhatGump;

pub fn new_what() -> StaticGumpObj<TrackWhatGump> {
    StaticGumpObj::new(20, 30, TrackWhatGump)
}

impl StaticGumpLogic for TrackWhatGump {
    const TYPE_NAME: &'static str = "Server.SkillHandlers.TrackWhatGump";

    fn build_layout(&self, b: &mut StaticGumpBuilder) {
        b.add_page(0);
        b.add_background(0, 0, 440, 135, 5054);
        b.add_background(10, 10, 420, 75, 2620);
        b.add_background(10, 85, 420, 25, 3000);
        // Animals, Monsters, Human NPCs, Players
        for (i, (item, cliloc)) in [
            (9682, 1018087),
            (9607, 1018088),
            (8454, 1018089),
            (8455, 1018090),
        ]
        .into_iter()
        .enumerate()
        {
            let x = 20 + 100 * i as i32;
            b.add_item(x, 20, item, 0);
            b.add_button(x, 110, 4005, 4007, i as i32 + 1);
            b.add_html_localized(x, 90, 100, 20, cliloc, false, false);
        }
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        if let Some(pm) = mobile_of(w, sender) {
            tracking::track_what_response(w, pm, info.button_id);
        }
    }
}

pub struct TrackWhoGump {
    targets: Vec<MobileId>,
    graphics: Vec<i32>,
    names: Vec<Option<String>>,
    range: i32,
}

pub fn new_who(w: &mut World, targets: &[MobileId], range: i32) -> DynamicGumpObj<TrackWhoGump> {
    let mut graphics = Vec::new();
    let mut names = Vec::new();
    for &m in targets {
        let body = muo_server::mobile::part1::props_b::body(w, m).0; // `m.Body`: the body mod when there is one
        graphics.push(shrink_lookup(w, body, DEFAULT_ITEM_ID));
        names.push(w.mobile(m).and_then(|x| (x.ty.name)(w, m)));
    }
    DynamicGumpObj::new(
        20,
        30,
        TrackWhoGump {
            targets: targets.to_vec(),
            graphics,
            names,
            range,
        },
    )
}

impl DynamicGumpLogic for TrackWhoGump {
    const TYPE_NAME: &'static str = "Server.SkillHandlers.TrackWhoGump";

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        let n = self.targets.len() as i32;
        b.add_page(0);
        b.add_background(0, 0, 440, 155, 5054);
        b.add_background(10, 10, 420, 75, 2620);
        b.add_background(10, 85, 420, 45, 3000);
        if n > 4 {
            b.add_background(0, 155, 440, 155, 5054);
            b.add_background(10, 165, 420, 75, 2620);
            b.add_background(10, 240, 420, 45, 3000);
            if n > 8 {
                b.add_background(0, 310, 440, 155, 5054);
                b.add_background(10, 320, 420, 75, 2620);
                b.add_background(10, 395, 420, 45, 3000);
            }
        }
        for i in 0..n {
            let (x, row) = (20 + i % 4 * 100, i / 4 * 155);
            b.add_item(x, 20 + row, self.graphics[i as usize], 0);
            b.add_button(x, 130 + row, 4005, 4007, i + 1);
            if let Some(name) = &self.names[i as usize] {
                b.add_html(x, 90 + row, 90, 40, name, false, false);
            }
        }
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        if let Some(pm) = mobile_of(w, sender) {
            tracking::track_who_response(w, pm, &self.targets, self.range, info.button_id);
        }
    }
}

/// Sets the hooks `Tracking.OnUse` and `TrackWhoGump.DisplayTo` open the gumps through.
pub fn configure(w: &mut World) {
    let h = w.resources.get_mut::<TrackingHooks>();
    h.show_track_what_gump = Some(|w, pm| {
        if let Some(ns) = w.mobile(pm).and_then(|m| m.net_state) {
            close_gump::<TrackWhoGump>(w, ns);
            close_gump::<TrackWhatGump>(w, ns);
        }
        send(w, pm, new_what());
    });
    h.show_track_who_gump = Some(|w, pm, mobs, range| {
        let g = new_who(w, mobs, range);
        send(w, pm, g);
    });
}
