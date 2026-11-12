//! The props gumps: the shared layout of staff gumps (`PropsConfig`, the `AddProps*`
//! helpers), the properties gump and the gumps that set one property.

pub mod host;
pub mod properties;
pub mod registry;
pub mod setters;

pub use host::{ListEntry, PropInfo, PropKind, PropObj, PropType, PropValue, PropsHost};
pub use properties::{PropertiesContext, PropertiesGump, StackEntry};

use std::rc::Rc;

use muo_commands::hooks::CommandHooks;
use muo_server::World;
use muo_server::ids::{EntityId, MobileId};

/// Installs the registry as the props gumps' host and the commands' `new PropertiesGump(from, entity)` hook.
pub fn configure(w: &mut World) {
    super::hooks::hooks_mut(w).props = Some(Rc::new(registry::RegistryHost));
    let h = w.resources.get_mut::<CommandHooks>();
    h.properties_gump = Some(open_properties_gump);
    h.properties_gump_target = Some(open_target_gump);
}

/// `from.SendGump(new PropertiesGump(from, o))` for a land or static target.
fn open_target_gump(w: &mut World, from: MobileId, t: muo_server::target::Targeted) {
    let g = properties::new(w, from, PropObj::Other(Rc::new(t)));
    super::common::send(w, from, g);
}

/// `from.SendGump(new PropertiesGump(from, entity))`.
fn open_properties_gump(w: &mut World, from: MobileId, e: EntityId) {
    let g = properties::new(w, from, PropObj::Entity(e));
    super::common::send(w, from, g);
}
