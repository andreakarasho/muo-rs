//! `PacketLoggingCommand`, and the `Firewall` command's reach into the firewall and the ban channel
//! (`FirewallCommand`).

use std::net::IpAddr;
use std::rc::Rc;
use std::time::Duration;

use muo_content::network::bans::BanReportQueue;
use muo_content::network::bans::firewall::{self, FirewallEntry};
use muo_net::bans::ban_reasons;
use muo_primitives::TimeSpan;
use muo_server::commands::CommandEventArgs;
use muo_server::mobile::AccessLevel;
use muo_server::world::World;

use crate::generic::implementors::register_command;
use crate::generic::{
    BaseCommand, CmdSpec, CommandSupport as S, Obj, ObjectTypes as O, Run, as_mobile,
};
use crate::hooks::CommandHooks;
use crate::logging::{format_entity, write_line};

/// `PacketLoggingCommand`.
pub struct PacketLoggingCommand {
    spec: CmdSpec,
}

impl PacketLoggingCommand {
    fn new() -> Self {
        Self {
            spec: CmdSpec::new(
                &["PacketLogging"],
                AccessLevel::Developer,
                S::ALL_MOBILES,
                O::Mobiles,
                "PacketLogging <on|off>",
                "Enables or disables packet logging for a particular user until they disconnect.",
            ),
        }
    }
}

impl BaseCommand for PacketLoggingCommand {
    fn spec(&self) -> &CmdSpec {
        &self.spec
    }

    fn execute(&self, w: &mut World, run: &mut Run, e: &CommandEventArgs, obj: Obj) {
        let Some(first) = e.arguments.first() else {
            return run.log_failure("Format: PacketLogging <on|off>");
        };
        let from = e.mobile;
        let enable = muo_primitives::parse::to_bool(first);

        let Some(pm) = as_mobile(obj).filter(|m| w.mobile(*m).is_some_and(|m| m.player)) else {
            return run.log_failure("That is not a player.");
        };
        let Some(ns) = w.mobile(pm).and_then(|m| m.net_state) else {
            return run.log_failure("The player is not connected.");
        };
        let level = |w: &World, m| w.mobile(m).map_or(AccessLevel::Player, |x| x.access_level);
        if from != pm && level(w, from) < level(w, pm) {
            return run.log_failure("You do not have the required access level to do this.");
        }
        w.set_packet_logging(ns, enable);
        let enabled = if enable { "enabled" } else { "disabled" };
        let username = w
            .mobile(pm)
            .and_then(|m| m.account)
            .and_then(|a| muo_content::accounting::account(w, a))
            .map(|a| a.username.clone())
            .unwrap_or_default();
        let desc = w
            .net_state(ns)
            .map(|d| d.description.clone())
            .unwrap_or_default();
        let text = format!(
            "{:?} {} {enabled} packet logging for {username} ({desc})",
            level(w, from),
            format_entity(w, muo_server::ids::EntityId::Mobile(from)),
        );
        write_line(w, from, &text);
        run.add_response(&format!(
            "Packet logging has been {enabled} for {username} ({desc})"
        ));
    }
}

/// `Firewall.Add(new SingleIpFirewallEntry(address)); BanChannel.Report(address, TimeSpan.Zero, BanReasons.Manual)`.
fn firewall_add(w: &mut World, address: IpAddr) -> Result<(), String> {
    firewall::add(w, FirewallEntry::single(address), TimeSpan::ZERO, true);
    w.resources
        .get_mut::<BanReportQueue>()
        .push(address, Duration::ZERO, ban_reasons::MANUAL);
    Ok(())
}

/// `NetworkCommands.Configure` (`TargetCommands.Register(new PacketLoggingCommand())`).
pub fn configure(w: &mut World) {
    register_command(w, Rc::new(PacketLoggingCommand::new()));
    w.resources.get_mut::<CommandHooks>().firewall_add = Some(firewall_add);
}
