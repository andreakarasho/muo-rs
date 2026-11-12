//! What the skill handlers read of classes other crates own (houses, food and potions, armor and weapons,
//! trappable containers, corpses, factions, buffs...). [`SkillHooks`] holds an optional table; each method of
//! [`SkillItemFns`] says in its documentation what an absent system answers (the default body): the answer when the
//! class is not in the world.

use muo_primitives::Point3D;
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::map::MapId;
use muo_server::poison::PoisonId;

/// The facts of a weapon `ArmsLore` reads.
#[derive(Copy, Clone, Debug, Default)]
pub struct WeaponFacts {
    pub hit_points: i32,
    pub max_hit_points: i32,
    pub min_damage: i32,
    pub max_damage: i32,
    /// `weap.Layer == Layer.OneHanded`.
    pub one_handed: bool,
    pub kind: WeaponKind,
    /// `weap.Poison != null && weap.PoisonCharges > 0`.
    pub poisoned: bool,
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum WeaponKind {
    #[default]
    Other,
    Slashing,
    Piercing,
    Bashing,
    Ranged,
}

/// The facts of armor `ArmsLore` reads.
#[derive(Copy, Clone, Debug, Default)]
pub struct ArmorFacts {
    pub hit_points: i32,
    pub max_hit_points: i32,
    pub armor_rating: f64,
}

/// The armor a mobile wears, for Stealth: `(MaterialType, BodyPosition, MageArmor != 0)` per piece.
pub type WornArmor = (i32, i32, bool);

/// One field per hook, set by the crate that owns the class; any number of owners fill their own fields of the same
/// table (`hooks_mut(w).fns.find_house_at = Some(..)`), so none overwrites another. A field left `None` answers the
/// default after `=`, the answer when the class is not in the world.
macro_rules! skill_item_fns {
    ($( $(#[$m:meta])* fn $name:ident($w:ident : $wt:ty $(, $a:ident : $t:ty)*) -> $r:ty = $d:expr; )*) => {
        #[derive(Clone, Copy, Default)]
        pub struct SkillItemFns { $( pub $name: Option<fn($wt $(, $t)*) -> $r>, )* }
        impl SkillItemFns { $(
            $(#[$m])*
            #[allow(unused_variables)]
            pub fn $name(&self, $w: $wt $(, $a: $t)*) -> $r {
                match self.$name { Some(f) => f($w $(, $a)*), None => $d }
            }
        )* }
    };
}

skill_item_fns! {
    /// `BaseHouse.FindHouseAt(m)`.
    fn find_house_at(w: &World, m: MobileId) -> Option<ItemId> = None;
    /// `BaseHouse.FindHouseAt(p, map, height)`.
    fn find_house_at_point(w: &World, p: Point3D, map: Option<MapId>, height: i32) -> Option<ItemId> = None;
    fn house_is_friend(w: &World, house: ItemId, m: MobileId) -> bool = false;
    /// `item is Food` then `food.Poison != null`.
    fn food_poisoned(w: &World, item: ItemId) -> Option<bool> = None;
    /// `item is BasePotion`: its `LabelNumber`.
    fn potion_label(w: &World, item: ItemId) -> Option<i32> = None;
    /// `item is PotionKeg`: `(Held, LabelNumber)`.
    fn potion_keg(w: &World, item: ItemId) -> Option<(i32, i32)> = None;
    /// `item is IIdentifiable`: sets `Identified`; false when it isn't one.
    fn set_identified(w: &mut World, item: ItemId) -> bool = false;
    fn is_identifiable(w: &World, item: ItemId) -> bool = false;
    fn weapon_facts(w: &World, item: ItemId) -> Option<WeaponFacts> = None;
    fn armor_facts(w: &World, item: ItemId) -> Option<ArmorFacts> = None;
    /// `targeted is SwampDragon pet && pet.HasBarding`: `(BardingHP, BardingMaxHP)`.
    fn swamp_dragon_barding(w: &World, m: MobileId) -> Option<(i32, i32)> = None;
    /// `TrappableContainer`: `(TrapType != None, TrapPower)`.
    fn trappable_container(w: &World, item: ItemId) -> Option<(bool, i32)> = None;
    /// Disarm a `TrappableContainer` (`TrapPower = 0; TrapLevel = 0; TrapType = None`).
    fn disarm_trappable_container(w: &mut World, item: ItemId) -> () = ();
    /// `(cont as TrappableContainer)?.ExecuteTrap(from) == true`.
    fn execute_trap(w: &mut World, item: ItemId, from: MobileId) -> bool = false;
    /// `ILockpickable.Picker?.Name`: `Some(None)` is a lockpickable nobody picked.
    fn lockpickable_picker(w: &World, item: ItemId) -> Option<Option<String>> = None;
    /// `Corpse`: `(forensicist, amount is human body, killer name, looters)`.
    fn corpse_facts(w: &World, item: ItemId) -> Option<CorpseFacts> = None;
    fn set_corpse_forensicist(w: &mut World, item: ItemId, name: String) -> () = ();
    /// `container.DisplayTo(from)`.
    fn display_container_to(w: &mut World, item: ItemId, from: MobileId) -> () = ();
    /// `PlayerMobile.NpcGuild == NpcGuild.ThievesGuild`.
    fn is_thieves_guild_member(w: &World, m: MobileId) -> bool = false;
    /// `corpse.Channeled`.
    fn corpse_channeled(w: &World, item: ItemId) -> bool = false;
    /// `corpse.Channeled = true; corpse.Hue = 0x835`.
    fn channel_corpse(w: &mut World, item: ItemId) -> () = ();
    /// `item is BaseFactionTrap`.
    fn is_faction_trap(w: &World, item: ItemId) -> bool = false;
    /// DetectHidden's reveal of a faction trap; true if one was revealed.
    fn detect_faction_trap(w: &mut World, src: MobileId, item: ItemId) -> bool = false;
    /// RemoveTrap's whole `BaseFactionTrap` branch.
    fn disarm_faction_trap(w: &mut World, from: MobileId, item: ItemId) -> () = ();
    /// `item is BaseTrap { Visible: false }`.
    fn is_hidden_trap(w: &World, item: ItemId) -> bool = false;
    fn set_trap_visible(w: &mut World, item: ItemId, visible: bool) -> () = ();
    /// `house.IsInside(m)`.
    fn is_inside_mobile(w: &World, house: ItemId, m: MobileId) -> bool = false;
    /// The armor worn, for `Stealth.GetArmorRating` under AOS.
    fn worn_armor(w: &World, m: MobileId) -> Vec<WornArmor> = Vec::new();
    /// `m.ArmorRating` (non-AOS).
    fn armor_rating(w: &World, m: MobileId) -> f64 = 0.0;
    /// `Meditation.CheckOkayHolding(item)` for a non-null item: a Spellbook, Runebook, or (AOS) a weapon/armor with
    /// SpellChanneling.
    fn okay_to_hold_while_meditating(w: &World, item: ItemId) -> bool = false;
    /// `InvisibilitySpell.StopTimer(m)`.
    fn stop_invisibility_timer(w: &mut World, m: MobileId) -> () = ();
    /// `Faction.InSkillLoss(from)`.
    fn faction_in_skill_loss(w: &World, m: MobileId) -> bool = false;
    /// `item is BasePoisonPotion`: its `Poison`, `MinPoisoningSkill` and `MaxPoisoningSkill`.
    fn poison_potion(w: &World, item: ItemId) -> Option<PoisonPotionFacts> = None;
    /// `item is FukiyaDarts or Shuriken`: its `UsesRemaining`.
    fn poisonable_throwing(w: &World, item: ItemId) -> Option<i32> = None;
    /// `darts.Poison = poison; darts.PoisonCharges = charges` (FukiyaDarts, Shuriken).
    fn poison_throwing(w: &mut World, item: ItemId, poison: PoisonId, charges: i32) -> () = ();
    /// `item is BaseBook`: its facts.
    fn book_facts(w: &World, item: ItemId) -> Option<BookFacts> = None;
    /// `Inscribe.Copy(src, dst)`: title, author and the lines of the pages both have.
    fn copy_book(w: &mut World, src: ItemId, dst: ItemId) -> () = ();
    /// `item is BlankScroll`.
    fn is_blank_scroll(w: &World, item: ItemId) -> bool = false;
    /// `item is RecallRune`.
    fn is_recall_rune(w: &World, item: ItemId) -> bool = false;
    /// `StealableArtifacts.GetStealableInstance(item) != null`.
    fn stealable_instance(w: &World, item: ItemId) -> bool = false;
    /// `si.Item = null` of that instance.
    fn clear_stealable_instance(w: &mut World, item: ItemId) -> () = ();
    /// `item is Sigil`.
    fn is_sigil(w: &World, item: ItemId) -> bool = false;
    /// The whole `toSteal is Sigil sig` branch of `StealingTarget.TryStealItem` (faction, disguise and transformation
    /// checks, the skill check, `GraceStart`, `LastMonolith.Sigil = null`, `LastStolen`), with its messages. Returns
    /// the sigil when it was stolen.
    fn steal_sigil(w: &mut World, thief: MobileId, sigil: ItemId) -> Option<ItemId> = None;
}

#[derive(Clone, Debug, Default)]
pub struct CorpseFacts {
    pub forensicist: Option<String>,
    pub is_human_body: bool,
    pub killer_name: Option<String>,
    pub looter_names: Vec<String>,
}

/// The table, set by the owning crates.
#[derive(Default)]
pub struct SkillHooks {
    pub fns: SkillItemFns,
}

/// The registered hooks (a copy of the table), or the inert ones.
pub fn ops(w: &World) -> SkillItemFns {
    w.resources
        .get::<SkillHooks>()
        .map(|h| h.fns)
        .unwrap_or_default()
}

pub fn hooks(w: &mut World) -> &SkillHooks {
    w.resources.get_mut::<SkillHooks>()
}

/// `BasePoisonPotion`'s numbers.
#[derive(Copy, Clone, Debug)]
pub struct PoisonPotionFacts {
    pub poison: muo_server::poison::PoisonId,
    pub min_skill: f64,
    pub max_skill: f64,
}

/// What `Inscribe` reads of a `BaseBook`.
#[derive(Copy, Clone, Debug, Default)]
pub struct BookFacts {
    /// `Inscribe.IsEmpty(book)`: no page has a non-empty line.
    pub is_empty: bool,
    pub writable: bool,
}
