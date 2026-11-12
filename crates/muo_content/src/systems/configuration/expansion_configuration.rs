//! `ExpansionConfiguration`: the expansion the shard runs.

use muo_server::World;
use muo_server::mobile::VisibleDamageType;
use muo_server::mobile::part1::props_a::PropsAStatics;
use muo_server::mobile::part1::props_b;
use muo_server::mobile::part2::statics::Part2Statics;

use crate::core;

/// Set by `Configure` for AOS clients: the single click of everything is overridden to check the object
/// property list (`IncomingEntityPackets.SingleClickProps`, read by the entity packet handlers).
pub use crate::network::incoming_game::SingleClickProps;

/// `T2ACraftSystem.Enabled`.
// Read by `Inscribe`, `Lore` and the T2A craft menus.
#[derive(Default)]
pub struct T2aCraftSystemEnabled(pub bool);

/// `ExpansionConfiguration.Configure()`.
pub fn configure(w: &mut World) {
    let aos = core::aos(w);
    let uotd = core::expansion(w) >= muo_data::expansion::Expansion::UOTD;

    // ObjectPropertyList.Enabled
    let opl = w.config.get_setting("opl.enable", aos);
    w.core.opl_enabled = opl;
    let visible_damage = w.config.get_setting("visibleDamage", aos);
    w.resources.get_mut::<PropsAStatics>().visible_damage_type = if visible_damage {
        VisibleDamageType::Related
    } else {
        VisibleDamageType::None
    };
    let guild_click = w.config.get_setting("guildClickMessage", !aos);
    let ascii_click = w.config.get_setting("asciiClickMessage", !aos);
    let s = w.resources.get_mut::<Part2Statics>();
    s.guild_click_message = guild_click;
    s.ascii_click_message = ascii_click;
    let t2a = w.config.get_setting("t2aCraftMenus", !uotd);
    w.resources.get_mut::<T2aCraftSystemEnabled>().0 = t2a;

    let delay = w
        .config
        .get_setting("actionDelay", if aos { 1000 } else { 500 });
    w.resources.get_mut::<PropsAStatics>().action_delay = delay;

    if aos {
        if opl {
            // single click for everything is overridden to check object property list
            w.resources.get_mut::<SingleClickProps>().0 = true;
        }
        props_b::set_aos_status_handler(w, Some(crate::misc::aos_helpers::get_status));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use muo_data::expansion::Expansion;

    #[test]
    fn aos_defaults() {
        let mut w = World::new();
        w.core.expansion = Some(Expansion::AOS);
        configure(&mut w);
        assert!(w.core.opl_enabled);
        assert_eq!(
            w.resources.get_mut::<PropsAStatics>().visible_damage_type,
            VisibleDamageType::Related
        );
        assert_eq!(w.resources.get_mut::<PropsAStatics>().action_delay, 1000);
        assert!(!w.resources.get_mut::<Part2Statics>().guild_click_message);
        assert!(w.resources.get_mut::<SingleClickProps>().0);
        assert!(props_b::aos_status_handler(&w).is_some());
    }

    #[test]
    fn pre_aos_defaults() {
        let mut w = World::new();
        w.core.expansion = Some(Expansion::T2A);
        configure(&mut w);
        assert!(!w.core.opl_enabled);
        assert_eq!(
            w.resources.get_mut::<PropsAStatics>().visible_damage_type,
            VisibleDamageType::None
        );
        assert_eq!(w.resources.get_mut::<PropsAStatics>().action_delay, 500);
        assert!(w.resources.get_mut::<Part2Statics>().ascii_click_message);
        assert!(!w.resources.get_mut::<SingleClickProps>().0);
        assert!(props_b::aos_status_handler(&w).is_none());
    }
}
