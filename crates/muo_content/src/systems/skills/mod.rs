//! Skills: skill checks and gain, the anti-macro system, and the skill handlers (`SkillInfo.Table[..]
//! .Callback = OnUse`).
//!
//! What the skill handlers reach of items other crates own is [`SkillItemFns`], [`bard::InstrumentOps`],
//! [`animal::AnimalTamingHooks`] and [`SkillGumpHooks`].

pub mod animal;
pub mod anti_macro;
pub mod bard;
pub mod common;
pub mod hooks;
pub mod inscribe;
pub mod item_ops;
pub mod lore;
pub mod misc_skills;
pub mod poisoning;
pub mod skill_check;
pub mod skill_events;
pub mod skills_info;
pub mod stealing;
pub mod stealth;
pub mod tracking;

#[cfg(test)]
mod skills2_tests;

use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::type_registry::{Phase, TypeRegistry};

/// Gumps a skill opens that the Gumps/Craft crates draw.
#[derive(Default)]
pub struct SkillGumpHooks {
    /// `T2ACraftSystem.ShowMenu(m, DefCartography.CraftSystem, null)`.
    pub show_cartography_menu: Option<fn(&mut World, MobileId)>,
    /// `T2ACraftSystem.ShowMenu(m, DefInscription.CraftSystem, null)`.
    pub show_inscription_menu: Option<fn(&mut World, MobileId)>,
    /// The T2A Inscribe target on a `RecallRune`: `system.CanCraft(from, null, typeof(Runebook))` (its message when
    /// it is above zero) then `system.CreateItem(from, typeof(Runebook), null, null, itemDef)`.
    pub craft_runebook: Option<fn(&mut World, MobileId)>,
}

pub use hooks::{SkillHooks, SkillItemFns};
pub use skill_events::subscribe_skill_used;

/// Every skill handler's `Initialize()`.
pub fn initialize_handlers(w: &mut World) {
    lore::initialize(w);
    stealth::initialize(w);
    animal::initialize(w);
    bard::initialize(w);
    poisoning::initialize(w);
    inscribe::initialize(w);
    stealing::initialize(w);
    animal::configure(w);
    bard::configure(w);
    misc_skills::initialize(w);
    tracking::initialize(w);
}

pub fn register(t: &mut TypeRegistry) {
    t.register_phase_default(
        Phase::Configure,
        "Server.SkillsInfo.Configure",
        skills_info::configure,
    );
    t.register_phase_default(
        Phase::Configure,
        "Server.Misc.SkillCheck.Configure",
        skill_check::configure,
    );
    t.register_phase_default(
        Phase::Configure,
        "Server.Misc.AntiMacroSystem.Configure",
        anti_macro::configure,
    );
    t.register_phase_default(
        Phase::Configure,
        "Server.SkillHandlers.ItemOps.Configure",
        item_ops::configure,
    );
    t.register_phase_default(
        Phase::Configure,
        "Server.SkillHandlers.Tracking.Configure",
        tracking::configure,
    );
    t.register_phase_default(
        Phase::Configure,
        "Server.SkillHandlers.Stealing.Configure",
        stealing::configure,
    );
    t.register_phase_default(
        Phase::Configure,
        "Server.SkillHandlers.Snooping.Configure",
        snooping_configure,
    );
    t.register_phase_default(
        Phase::Initialize,
        "Server.Misc.SkillCheck.Initialize",
        skill_check::initialize,
    );
    t.register_phase_default(
        Phase::Initialize,
        "Server.Misc.AntiMacroSystem.Initialize",
        anti_macro::initialize,
    );
    t.register_phase_default(
        Phase::Initialize,
        "Server.SkillHandlers.Initialize",
        initialize_handlers,
    );
}

/// `Snooping.Configure()`: `Container.SnoopHandler = Container_Snoop`.
fn snooping_configure(w: &mut World) {
    w.resources.get_mut::<SnoopHandler>().0 = Some(misc_skills::container_snoop);
    w.resources
        .get_mut::<muo_server::item::container::ContainerStatics>()
        .snoop_handler = Some(misc_skills::container_snoop);
}

/// `Container.SnoopHandler`: the container code calls this for a snoop.
// Kept for readers of this crate; `ContainerStatics::snoop_handler` is the slot the container code reads.
#[derive(Default)]
pub struct SnoopHandler(pub Option<fn(&mut World, muo_server::ids::ItemId, MobileId)>);
