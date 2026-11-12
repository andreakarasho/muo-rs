//! Systems the commands call that live in crates above this one: a `fn` pointer in `w.resources`, set by the owning crate's `register`. Unset means the system is
//! absent and the command says so or goes on without it.

use muo_server::ids::{EntityId, MobileId};
use muo_server::world::World;

type FirewallAddFn = fn(&mut World, std::net::IpAddr) -> Result<(), String>;
/// The replies of `FactionKickCommand`: `(true, text)` is an `AddResponse`, `(false, text)` a `LogFailure`.
type FactionKickFn = fn(&mut World, MobileId, MobileId, u8) -> Vec<(bool, String)>;
type InterfaceGumpFn = fn(&mut World, MobileId, Vec<EntityId>, Vec<String>);
type DecorateDoorProbeFn =
    fn(&World, muo_server::ids::ItemId) -> Option<(muo_primitives::Point3D, i32)>;
type WarningGumpFn = fn(&mut World, MobileId, &str, (i32, i32), Box<dyn FnOnce(&mut World, bool)>);
type SextantReverseLookupFn = fn(
    &mut World,
    muo_server::map::MapId,
    i32,
    i32,
    i32,
    i32,
    bool,
    bool,
) -> muo_primitives::Point3D;
type NewTeleporterFn = fn(
    &mut World,
    muo_primitives::Point3D,
    Option<muo_server::map::MapId>,
) -> Option<muo_server::ids::ItemId>;
type DecorateConstructFn =
    fn(&mut World, &str, i32, &[String]) -> Option<Result<muo_server::ids::ItemId, String>>;
type DecorateApplyFn =
    fn(&mut World, muo_server::ids::ItemId, i32, &[String], bool) -> Result<bool, String>;
/// `fn(w, from, text)` style hooks.
#[derive(Default)]
pub struct CommandHooks {
    /// `new PropertiesGump(from, entity)` sent to `from` (`muo_gumps`).
    pub properties_gump: Option<fn(&mut World, MobileId, EntityId)>,
    /// `new PropertiesGump(from, o)` for a land or static target.
    pub properties_gump_target: Option<fn(&mut World, MobileId, muo_server::target::Targeted)>,
    /// `StaticWarningGump<T>`: a Continue / Cancel gump with HTML `content`, size `(width, height)`; the callback
    /// gets the button pressed.
    pub warning_gump: Option<WarningGumpFn>,
    /// `new MessageSentGump(to, fromName, text)` (the `[message` popup).
    pub message_sent_gump: Option<fn(&mut World, MobileId, &str, &str)>,
    /// `new BanDurationGump(account)` for the mobile's account.
    pub ban_duration_gump: Option<fn(&mut World, MobileId, MobileId)>,
    /// `Firewall.Add(new SingleIpFirewallEntry(address))` and `BanChannel.Report(..)`; `Err` is the exception text.
    pub firewall_add: Option<FirewallAddFn>,
    /// `HouseSign.Owner.RefreshDecay()` for a house sign item; `None` when it is not a sign, `Some(false)` when the
    /// sign has no house.
    pub refresh_house: Option<fn(&mut World, muo_server::ids::ItemId) -> Option<bool>>,
    /// `BaseHouse.AllHouses` search for the house an item is locked down or secured in, opening its props gump.
    pub trace_lockdown: Option<fn(&mut World, MobileId, muo_server::ids::ItemId) -> bool>,
    /// `FactionKickCommand`: kick / ban / unban a mobile from its faction (0, 1, 2); the replies.
    pub faction_kick: Option<FactionKickFn>,
    /// `new AddGump(search, 0, matches, ..)` (`categorized` false) or `new CategorizedAddGump(from)` (true).
    pub add_gump: Option<fn(&mut World, MobileId, &str, bool)>,
    /// `[interface`: `InterfaceGump(from, list, 0, null)` over the found objects with the `view` columns.
    pub interface_gump: Option<InterfaceGumpFn>,
    /// `GoGump.DisplayTo(from)` (`[go` with no arguments).
    pub go_gump: Option<fn(&mut World, MobileId)>,
    /// `new ClientGump(from, targ.NetState)`: the target mobile's connection.
    pub client_gump: Option<fn(&mut World, MobileId, MobileId)>,
    /// `new SkillsGump(from, mobile)` (`[skills`).
    pub skills_gump: Option<fn(&mut World, MobileId, MobileId)>,
    /// `new StuckMenu(from, mobile, false)`.
    pub stuck_menu: Option<fn(&mut World, MobileId, MobileId)>,
    /// `Localization.Clear()`.
    pub clear_localization: Option<fn(&mut World)>,
    /// `PageQueue.GetEntry(m)?.Handler`: the staff member handling a mobile's help page.
    pub page_handler: Option<fn(&World, MobileId) -> Option<MobileId>>,
    /// `Sextant.ReverseLookup(map, xLong, yLat, xMins, yMins, xEast, ySouth)`; `Point3D.Zero` on failure.
    pub sextant_reverse_lookup: Option<SextantReverseLookupFn>,
    /// `new StaffRobe(accessLevel)`; unset, a default `StaffRobe` is made.
    pub new_staff_robe:
        Option<fn(&mut World, muo_server::mobile::AccessLevel) -> Option<muo_server::ids::ItemId>>,
    /// `DesignContext.Check(m)`: false while the mobile is customizing a house.
    pub design_context_check: Option<fn(&mut World, MobileId) -> bool>,
    /// `Spawner.Reset()` for a spawner item (Advanced Search, Reset).
    pub spawner_reset: Option<fn(&mut World, muo_server::ids::ItemId)>,
    /// `ISpawner { Running: true }.Respawn()` for an entity: does nothing when it is not a running spawner.
    pub spawner_respawn: Option<fn(&mut World, EntityId)>,
    /// `BaseSpawner.OnDoubleClick(from)`: the spawner's own gump.
    pub spawner_open: Option<fn(&mut World, MobileId, muo_server::ids::ItemId)>,
    /// `AdvancedSearchThreadWorker.HandleValidInternal`: the entities a CommodityDeed, a KeyRing or a BaseHouse
    /// holds on the internal map legitimately (so the search can hide them).
    pub valid_internal_held: Option<fn(&World, EntityId) -> Vec<EntityId>>,
    /// `pm.ShortTermMurders = n` (`PlayerMurderSystem.ManuallySetShortTermMurders`).
    pub set_short_term_murders: Option<fn(&mut World, MobileId, i32)>,
    /// `PlayerVendor.House != null`: an internalized vendor of a house.
    pub vendor_has_house: Option<fn(&World, MobileId) -> bool>,
    /// `new Teleporter(destination, destinationMap)`.
    pub new_teleporter: Option<NewTeleporterFn>,
    /// `DesignInsertCommand.ProcessInsert` after the item checks: finds the customizable house the item is in,
    /// inserts the item's graphic into its current and design states and deletes the item.
    pub design_insert:
        Option<fn(&mut World, muo_server::ids::ItemId) -> crate::teleporters::InsertOutcome>,
    /// `Decorate`: the classes whose constructor the generic overload tables cannot call (`new HintItem(..)`, a
    /// `BaseBeverage` with its `Content`, `new Cannon(direction)`, `new SerpentPillar(word, destination)`,
    /// `new MarkContainer(bone, locked)`...). Gets the class full name, the entry's item id and its `(..)`
    /// parameters; `None` when the hook does not know the class, `Some(Err)` is a constructor exception.
    /// Several crates add one each; the first that knows the class answers.
    pub decorate_construct: Vec<DecorateConstructFn>,
    /// `Decorate`: the per-class parameter blocks of `Construct` (addon components, lights, spawners, runes,
    /// teleporters, fillable containers). `true` when the item's class was one of them (the generic `ItemID`
    /// assignment is then skipped, as in the `else if` chain); the flag says `DecorateMag` ran (it has no
    /// `WalkingRange` or `InteractionTeleporter`).
    /// Several crates add one each; the first that handles the item answers `true`. `Err` is the exception a
    /// `Parse` of the parameters threw (`Point3D.Parse`, `TimeSpan.Parse`, `Map.Parse`, `Enum.Parse`).
    pub decorate_apply: Vec<DecorateApplyFn>,
    /// `Decorate.FindItem` for a door: `(p, bdItemID)` of a `BaseDoor` (its closed location and graphic), `None`
    /// for anything else.
    pub decorate_door_probe: Option<DecorateDoorProbeFn>,
    /// `Decorate.Generate` after `MoveToWorld`: links a door to its pair, sets a `MarkContainer`'s target from the
    /// entry's extra text.
    pub decorate_placed:
        Option<fn(&mut World, muo_server::ids::ItemId, muo_primitives::Point3D, &str)>,
    /// `PlayerMurderSystem.BountiesEnabled`.
    pub bounties_enabled: Option<fn(&World) -> bool>,
}

/// The exception text of a `Parse` (`Point3D.Parse`, `TimeSpan.Parse`, `Map.Parse`) that was given `text`.
pub fn parse_error(text: &str) -> String {
    format!("The input string '{text}' was not in a correct format.")
}

/// `(T)Enum.Parse(typeof(T), text, true)` for an enum whose members are `names` by value: a name (ignoring case) or a
/// number; `Err` is the `ArgumentException` otherwise.
pub fn enum_parse(names: &[&str], text: &str) -> Result<i32, String> {
    let t = text.trim();
    if t.is_empty() {
        return Err("Must specify valid information for parsing in the string.".to_string());
    }
    if let Ok(n) = t.parse::<i32>() {
        return Ok(n);
    }
    names
        .iter()
        .position(|n| n.eq_ignore_ascii_case(t))
        .map(|i| i as i32)
        .ok_or_else(|| format!("Requested value '{t}' was not found."))
}

#[cfg(test)]
mod parse_tests {
    use super::*;

    #[test]
    fn enum_parse_names_numbers_and_failures() {
        let names = ["North", "East", "South", "West"];
        assert_eq!(enum_parse(&names, " south "), Ok(2));
        assert_eq!(enum_parse(&names, "WEST"), Ok(3));
        assert_eq!(enum_parse(&names, "7"), Ok(7));
        assert!(enum_parse(&names, "up").unwrap_err().contains("'up'"));
        assert!(enum_parse(&names, "").is_err());
    }
}
