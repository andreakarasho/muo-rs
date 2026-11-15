//! Helpers the table class constructors call: one per property/method a constructor sets, named after it.

pub use muo_content::creature::setters::pack_item;
use muo_server::world::World;

pub use muo_content::creature::enums::*;
pub use muo_content::creature::packing::*;
pub use muo_content::creature::setters::*;
pub use muo_content::hooks::rnd;
pub use muo_content::loot_pack;
pub use muo_server::mobile::ResistanceType;
pub use muo_server::skills::SkillName;

pub use muo_content::creature::data::read;
pub use muo_content::creature::init_ops::*;
use muo_server::ids::MobileId;

/// The per-world tables of single monster classes (class-wide state).
#[derive(Default)]
pub struct MobileTables {
    /// Ilhenir `m_Table`.
    pub cacophonic: std::collections::HashSet<MobileId>,
    /// KhaldunRevenant `m_Set`.
    pub khaldun: std::collections::HashSet<MobileId>,
    /// RaiJu `m_Table`.
    pub rai_ju: std::collections::HashSet<MobileId>,
    /// Satyr `m_Suppressed`.
    pub satyr: std::collections::HashMap<MobileId, muo_server::timer::TimerHandle>,
    /// MeerMage `m_Table`.
    pub meer_mage: std::collections::HashMap<MobileId, muo_server::timer::TimerHandle>,
    pub swoop: std::collections::HashMap<MobileId, muo_server::timer::TimerHandle>,
    pub flurry: std::collections::HashMap<MobileId, muo_server::timer::TimerHandle>,
    pub chlorophyll: std::collections::HashMap<MobileId, muo_server::timer::TimerHandle>,
    pub snow: std::collections::HashMap<MobileId, muo_server::timer::TimerHandle>,
    /// DemonKnight's re-entry guard.
    pub demon_knight_in_here: bool,
}

pub fn tables(w: &mut World) -> &mut MobileTables {
    w.resources.get_mut::<MobileTables>()
}
