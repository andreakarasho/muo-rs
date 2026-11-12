//! The enums of `BaseCreature` and `AIType`.

use bitflags::bitflags;

macro_rules! int_enum {
    ($(#[$m:meta])* $name:ident { $($v:ident = $n:expr),+ $(,)? }) => {
        $(#[$m])*
        #[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd)]
        #[repr(i32)]
        pub enum $name { #[default] $($v = $n),+ }
        impl $name {
            /// The value for a raw integer (one without a name is `None`).
            pub const fn from_i32(v: i32) -> Option<$name> {
                match v { $($n => Some($name::$v),)+ _ => None }
            }
        }
    };
}

int_enum!(FightMode { None = 0, Aggressor = 1, Strongest = 2, Weakest = 3, Closest = 4, Evil = 5 });

impl FightMode {
    /// `FightMode.Closest`, the constructor default.
    pub const DEFAULT: FightMode = FightMode::Closest;
}

int_enum!(OrderType {
    None = 0, Come = 1, Drop = 2, Follow = 3, Friend = 4, Unfriend = 5, Guard = 6, Attack = 7, Patrol = 8,
    Release = 9, Stay = 10, Stop = 11, Transfer = 12, Rename = 13
});

int_enum!(AIType {
    AiUseDefault = 0, AiMelee = 1, AiAnimal = 2, AiArcher = 3, AiHealer = 4, AiVendor = 5, AiMage = 6,
    AiBerserk = 7, AiPredator = 8, AiThief = 9
});

int_enum!(ScaleType { Red = 0, Yellow = 1, Black = 2, Green = 3, White = 4, Blue = 5, All = 6 });
int_enum!(MeatType { Ribs = 0, Bird = 1, LambLeg = 2 });
int_enum!(HideType { Regular = 0, Spined = 1, Horned = 2, Barbed = 3 });
int_enum!(SpeedLevel { None = 0, VerySlow = 1, Slow = 2, Medium = 3, Fast = 4, VeryFast = 5 });
int_enum!(
    /// `BaseCreature.Allegiance`.
    Allegiance { None = 0, Ally = 1, Enemy = 2 }
);
int_enum!(
    /// `BaseCreature.TeachResult`.
    TeachResult {
        Success = 0, Failure = 1, KnowsMoreThanMe = 2, KnowsWhatIKnow = 3, SkillNotRaisable = 4,
        NotEnoughFreePoints = 5
    }
);

bitflags! {
    #[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
    pub struct FoodType: i32 {
        const MEAT = 0x1;
        const FRUITS_AND_VEGGIES = 0x2;
        const GRAINS_AND_HAY = 0x4;
        const FISH = 0x8;
        const EGGS = 0x10;
        const GOLD = 0x20;
        const LEATHER = 0x40;
        const METAL = 0x80;
    }

    #[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
    pub struct PackInstinct: i32 {
        const CANINE = 0x1;
        const OSTARD = 0x2;
        const FELINE = 0x4;
        const ARACHNID = 0x8;
        const DAEMON = 0x10;
        const BEAR = 0x20;
        const EQUINE = 0x40;
        const BULL = 0x80;
    }
}

/// `BaseCreature.MaxLoyalty` and the other constants.
pub const MAX_LOYALTY: i32 = 100;
pub const LOYALTY_INCREASE_PER_FOOD: i32 = 10;
pub const MAX_LOYALTY_INCREASE: i32 = MAX_LOYALTY / LOYALTY_INCREASE_PER_FOOD;
pub const MAX_OWNERS: usize = 5;
pub const DEFAULT_RANGE_PERCEPTION: i32 = 16;
pub const SHOUT_RANGE: i32 = 8;
pub const FALLBACK_ACTIVE_SPEED: f64 = 0.25;
pub const FALLBACK_PASSIVE_SPEED: f64 = 0.5;
/// `HerdingMoveSpeed`.
pub const HERDING_MOVE_SPEED: f64 = 0.3;
