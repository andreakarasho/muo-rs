//! Hooks of this crate's engines whose other end is in `muo_gumps` (so the engine modules themselves stay free of it).

use muo_content::gumps::send_gump_to_mobile;
use muo_gumps::m_z::resurrect::{ResurrectMessage, new};
use muo_server::World;

pub fn configure(w: &mut World) {
    // `SacrificeVirtue.Resurrect`: `new ResurrectGump(from, fromSacrifice: true)`
    w.resources
        .get_mut::<crate::virtues::ChampionHooks>()
        .sacrifice_resurrect_gump = Some(|w, from| {
        let g = new(Some(from), ResurrectMessage::Generic, true, 0.0, 0);
        send_gump_to_mobile(w, from, Box::new(g), false);
    });
}
