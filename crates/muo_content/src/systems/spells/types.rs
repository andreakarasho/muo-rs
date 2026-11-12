//! Spell types: `DisturbType`, `SpellCircle`, `SpellState`, `SpellInfo` and the reagents.

use muo_server::skills::SkillName;

/// `SpellState`.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum SpellState {
    #[default]
    None = 0,
    /// We are in the process of casting (waiting `GetCastDelay()` and doing animations). Casting may be interrupted.
    Casting = 1,
    /// Casting completed, but the full spell sequence isn't. Usually waiting for a target response. Some actions are
    /// restricted in this state (using skills for example).
    Sequencing = 2,
}

/// `DisturbType`.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum DisturbType {
    #[default]
    Unspecified,
    EquipRequest,
    UseRequest,
    Hurt,
    Kill,
    NewCast,
}

/// `SpellCircle`.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub enum SpellCircle {
    #[default]
    First,
    Second,
    Third,
    Fourth,
    Fifth,
    Sixth,
    Seventh,
    Eighth,
}

/// `TravelCheckType`.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TravelCheckType {
    RecallFrom,
    RecallTo,
    GateFrom,
    GateTo,
    Mark,
    TeleportFrom,
    TeleportTo,
}

/// `SpellInfo`: what the spell's constructor passed to `base(caster, scroll, new SpellInfo(..))`. Reagents are item
/// class full names (see [`reagent`]); `amounts` empty means one of each.
#[derive(Copy, Clone, Debug)]
pub struct SpellInfo {
    pub name: &'static str,
    pub mantra: &'static str,
    pub action: i32,
    pub left_hand_effect: i32,
    pub right_hand_effect: i32,
    pub allow_town: bool,
    pub reagents: &'static [&'static str],
    pub amounts: &'static [i32],
    /// `Core.AOS ? a : b` in the info: the (action, left hand, right hand) used before AOS.
    pub pre_aos: Option<(i32, i32, i32)>,
}

impl SpellInfo {
    /// The action, honoring a `Core.AOS` switch.
    pub fn action_in(&self, aos: bool) -> i32 {
        match self.pre_aos {
            Some(p) if !aos => p.0,
            _ => self.action,
        }
    }

    /// The (left, right) hand effects, honoring a `Core.AOS` switch.
    pub fn hand_effects_in(&self, aos: bool) -> (i32, i32) {
        match self.pre_aos {
            Some(p) if !aos => (p.1, p.2),
            _ => (self.left_hand_effect, self.right_hand_effect),
        }
    }

    /// The action and hand effect the info takes when `Core.AOS` is false.
    pub const fn pre_aos(mut self, action: i32, hand_effect: i32) -> Self {
        self.pre_aos = Some((action, hand_effect, hand_effect));
        self
    }

    /// `new SpellInfo(name, mantra, regs)`: action 16, no hand effect, allowed in town.
    pub const fn new(
        name: &'static str,
        mantra: &'static str,
        reagents: &'static [&'static str],
    ) -> Self {
        Self {
            name,
            mantra,
            action: 16,
            left_hand_effect: 0,
            right_hand_effect: 0,
            allow_town: true,
            reagents,
            amounts: &[],
            pre_aos: None,
        }
    }

    pub const fn action(mut self, action: i32) -> Self {
        self.action = action;
        self
    }

    /// `handEffect`: the same effect on both hands.
    pub const fn hand_effect(mut self, effect: i32) -> Self {
        self.left_hand_effect = effect;
        self.right_hand_effect = effect;
        self
    }

    pub const fn hand_effects(mut self, left: i32, right: i32) -> Self {
        self.left_hand_effect = left;
        self.right_hand_effect = right;
        self
    }

    pub const fn allow_town(mut self, allow: bool) -> Self {
        self.allow_town = allow;
        self
    }

    pub const fn amounts(mut self, amounts: &'static [i32]) -> Self {
        self.amounts = amounts;
        self
    }

    /// `Amounts[i]`.
    pub fn amount(&self, i: usize) -> i32 {
        self.amounts.get(i).copied().unwrap_or(1)
    }

    /// `Amounts` as a vector, one per reagent.
    pub fn amounts_vec(&self) -> Vec<i32> {
        (0..self.reagents.len()).map(|i| self.amount(i)).collect()
    }
}

/// `Reagent`: the item class of each reagent, by full name.
pub mod reagent {
    pub const BLACK_PEARL: &str = "Server.Items.BlackPearl";
    pub const BLOODMOSS: &str = "Server.Items.Bloodmoss";
    pub const GARLIC: &str = "Server.Items.Garlic";
    pub const GINSENG: &str = "Server.Items.Ginseng";
    pub const MANDRAKE_ROOT: &str = "Server.Items.MandrakeRoot";
    pub const NIGHTSHADE: &str = "Server.Items.Nightshade";
    pub const SULFUROUS_ASH: &str = "Server.Items.SulfurousAsh";
    pub const SPIDERS_SILK: &str = "Server.Items.SpidersSilk";
    pub const BAT_WING: &str = "Server.Items.BatWing";
    pub const GRAVE_DUST: &str = "Server.Items.GraveDust";
    pub const DAEMON_BLOOD: &str = "Server.Items.DaemonBlood";
    pub const NOX_CRYSTAL: &str = "Server.Items.NoxCrystal";
    pub const PIG_IRON: &str = "Server.Items.PigIron";
    pub const BONE: &str = "Server.Items.Bone";
    pub const FERTILE_DIRT: &str = "Server.Items.FertileDirt";
    pub const DRAGONS_BLOOD: &str = "Server.Items.DragonsBlood";
    pub const DAEMON_BONE: &str = "Server.Items.DaemonBone";

    /// `Reagent.Types`, in declaration order.
    pub const TYPES: [&str; 17] = [
        BLACK_PEARL,
        BLOODMOSS,
        GARLIC,
        GINSENG,
        MANDRAKE_ROOT,
        NIGHTSHADE,
        SULFUROUS_ASH,
        SPIDERS_SILK,
        BAT_WING,
        GRAVE_DUST,
        DAEMON_BLOOD,
        NOX_CRYSTAL,
        PIG_IRON,
        BONE,
        FERTILE_DIRT,
        DRAGONS_BLOOD,
        DAEMON_BONE,
    ];
}

/// `SkillName` of the default `CastSkill`/`DamageSkill`.
pub const DEFAULT_CAST_SKILL: SkillName = SkillName::Magery;
pub const DEFAULT_DAMAGE_SKILL: SkillName = SkillName::EvalInt;
