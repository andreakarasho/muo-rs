//! State of the item families whose classes live in `muo_items` but whose attribute bags `items::attr`/`set_attr`
//! must reach (`BaseQuiver` and `BaseTalisman` have `AosAttributes`; talismans also `AosSkillBonuses`). The logic
//! is in `muo_items::{quivers, talismans}`.

use muo_io::text::TextDefinition;
use muo_server::ids::MobileId;
use muo_server::timer::TimerHandle;

use crate::aos::Attrs;

/// `BaseQuiver`' fields. `quality` is a `ClothingQuality` value.
#[derive(Clone, Debug, Default)]
pub struct QuiverData {
    pub attributes: Attrs,
    pub lower_ammo_cost: i32,
    pub weight_reduction: i32,
    pub damage_increase: i32,
    pub crafter: Option<String>,
    pub quality: i32,
    pub capacity: i32,
}

/// `TalismanAttribute`: a class (by full name), a display name and an amount.
#[derive(Clone, Debug, Default)]
pub struct TalismanAttr {
    pub type_name: Option<String>,
    pub name: Option<TextDefinition>,
    pub amount: i32,
}

impl TalismanAttr {
    /// `IsEmpty => _type == null`.
    pub fn is_empty(&self) -> bool {
        self.type_name.is_none()
    }
}

/// `BaseTalisman`' fields.
#[derive(Clone, Debug, Default)]
pub struct TalismanData {
    pub attributes: Attrs,
    pub skill_bonuses: Attrs,
    pub protection: TalismanAttr,
    pub killer: TalismanAttr,
    pub summoner: TalismanAttr,
    /// `TalismanRemoval`, by value.
    pub removal: i32,
    /// `SkillName`, by value.
    pub skill: i32,
    pub success_bonus: i32,
    pub exceptional_bonus: i32,
    pub max_charges: i32,
    pub charges: i32,
    pub max_charge_time: i32,
    pub charge_time: i32,
    pub blessed: bool,
    /// `TalismanSlayerName`, by value.
    pub slayer: i32,
    /// `_creature` (not serialized) and `_timerToken`.
    pub creature: Option<MobileId>,
    pub timer: TimerHandle,
}

/// `Spellbook`' fields; the `AosAttributes` and `AosSkillBonuses` bags
/// are here so that `items::attr` reaches them. `quality` is a `BookQuality`, `slayer`/`slayer2` `SlayerName`s.
#[derive(Clone, Debug, Default)]
pub struct SpellbookData {
    pub quality: i32,
    pub engraved_text: Option<String>,
    pub crafter: Option<String>,
    pub slayer: i32,
    pub slayer2: i32,
    pub attributes: Attrs,
    pub skill_bonuses: Attrs,
    pub content: u64,
    pub spell_count: i32,
}
