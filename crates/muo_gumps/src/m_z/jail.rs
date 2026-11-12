//! `JailRecordGump`.

use muo_content::gumps::base::{StaticGumpLogic, StaticGumpObj};
use muo_content::gumps::builder::{GumpStrings, StaticGumpBuilder};
use muo_content::jail::{JailHooks, JailRecord, JailRecordGumpText, jail_record_gump_text};
use muo_server::World;

use super::common::send;

pub struct JailRecordGump(JailRecordGumpText);

impl StaticGumpLogic for JailRecordGump {
    const TYPE_NAME: &'static str = "Server.Systems.JailSystem.JailRecordGump";

    fn build_layout(&self, b: &mut StaticGumpBuilder) {
        b.set_no_resize();
        b.add_page(0);
        b.add_background(0, 0, 330, 300, 9200);
        b.add_alpha_region(10, 10, 310, 280);
        b.add_html(
            20,
            20,
            360,
            30,
            "<BASEFONT size=6 color=#FF6600>JAIL RECORD</BASEFONT>",
            false,
            false,
        );
        b.add_html_placeholder(20, 60, 360, 25, "playerName", false, false);
        b.add_html_placeholder(20, 80, 360, 25, "jailCount", false, false);
        b.add_html_placeholder(20, 100, 360, 25, "lastJailed", false, false);
        b.add_html_placeholder(20, 120, 290, 25, "jailReason", false, false);
        b.add_html_placeholder(20, 150, 360, 25, "jailStatus", false, false);
        b.add_html_placeholder(20, 170, 360, 25, "jailTime", false, false);
        for (y, t) in [
            (200, "If you believe you were jailed in error,"),
            (220, "please contact staff through normal channels."),
            (240, "Each time you are jailed, the wait increases."),
            (260, "Follow all shard rules to avoid future jail time."),
        ] {
            b.add_html(
                20,
                y,
                360,
                25,
                &format!("<BASEFONT size=4 color=#CCCCCC>{t}</BASEFONT>"),
                false,
                false,
            );
        }
    }

    fn build_strings(&self, s: &mut GumpStrings) {
        let t = &self.0;
        for (k, (text, color, size)) in [
            ("playerName", &t.player_name),
            ("jailCount", &t.jail_count),
            ("lastJailed", &t.last_jailed),
            ("jailReason", &t.jail_reason),
            ("jailStatus", &t.jail_status),
        ] {
            s.set_html_text(k, text, color, i32::from(*size));
        }
        if t.jail_time.0.is_empty() {
            s.set_string_slot("jailTime", "");
        } else {
            s.set_html_text(
                "jailTime",
                &t.jail_time.0,
                t.jail_time.1,
                i32::from(t.jail_time.2),
            );
        }
    }
}

/// `new JailRecordGump(player, record)`.
pub fn new(
    w: &World,
    player: muo_server::ids::MobileId,
    record: &JailRecord,
) -> StaticGumpObj<JailRecordGump> {
    let name = w
        .mobile(player)
        .and_then(|m| (m.ty.name)(w, player))
        .unwrap_or_default();
    StaticGumpObj::new(
        0,
        0,
        JailRecordGump(jail_record_gump_text(&name, record, w.now)),
    )
}

pub fn configure(w: &mut World) {
    w.resources.get_mut::<JailHooks>().show_jail_record_gump = Some(|w, to, player, record| {
        let g = new(w, player, record);
        send(w, to, g);
    });
}
