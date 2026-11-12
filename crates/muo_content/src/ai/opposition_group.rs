//! `OppositionGroup`: groups of creature classes that are enemies of each other. A class
//! belongs to a group when it is, or derives from, a listed class (`IsAssignableFrom`).

use muo_server::ids::MobileId;
use muo_server::world::World;

/// `new OppositionGroup(Type[][])`, the types as class names in `Server.Mobiles`.
pub struct OppositionGroup {
    types: &'static [&'static [&'static str]],
}

impl OppositionGroup {
    pub const fn new(types: &'static [&'static [&'static str]]) -> Self {
        Self { types }
    }

    /// `IndexOf(obj)`: the index of the first group containing the mobile's class, or -1.
    pub fn index_of(&self, w: &World, m: MobileId) -> i32 {
        let Some(mob) = w.mobile(m) else { return -1 };
        for (i, group) in self.types.iter().enumerate() {
            let mut t = Some(mob.ty);
            while let Some(ty) = t {
                if group
                    .iter()
                    .any(|n| ty.full_name.strip_prefix("Server.Mobiles.") == Some(*n))
                {
                    return i as i32;
                }
                t = ty.parent;
            }
        }
        -1
    }

    /// `IsEnemy(from, target)`.
    pub fn is_enemy(&self, w: &World, from: MobileId, target: MobileId) -> bool {
        let (a, b) = (self.index_of(w, from), self.index_of(w, target));
        a != -1 && b != -1 && a != b
    }
}

pub static TERATHANS_AND_OPHIDIANS: OppositionGroup = OppositionGroup::new(&[
    &[
        "TerathanAvenger",
        "TerathanDrone",
        "TerathanMatriarch",
        "TerathanWarrior",
    ],
    &[
        "OphidianArchmage",
        "OphidianKnight",
        "OphidianMage",
        "OphidianMatriarch",
        "OphidianWarrior",
    ],
]);

pub static SAVAGES_AND_ORCS: OppositionGroup = OppositionGroup::new(&[
    &[
        "Orc",
        "OrcBomber",
        "OrcBrute",
        "OrcCaptain",
        "OrcishLord",
        "OrcishMage",
        "SpawnedOrcishLord",
    ],
    &["Savage", "SavageRider", "SavageRidgeback", "SavageShaman"],
]);

pub static FEY_AND_UNDEAD: OppositionGroup = OppositionGroup::new(&[
    &[
        "Centaur",
        "EtherealWarrior",
        "Kirin",
        "LordOaks",
        "Pixie",
        "Silvani",
        "Unicorn",
        "Wisp",
        "Treefellow",
        "MLDryad",
        "Satyr",
    ],
    &[
        "AncientLich",
        "Bogle",
        "LichLord",
        "Shade",
        "Spectre",
        "Wraith",
        "BoneKnight",
        "Ghoul",
        "Mummy",
        "SkeletalKnight",
        "Skeleton",
        "Zombie",
        "ShadowKnight",
        "DarknightCreeper",
        "RevenantLion",
        "LadyOfTheSnow",
        "RottingCorpse",
        "SkeletalDragon",
        "Lich",
    ],
]);

#[cfg(test)]
mod tests {
    use super::*;
    use muo_server::mobile::{MOBILE_BASE, MobileType};

    static ORC: MobileType = MobileType {
        full_name: "Server.Mobiles.Orc",
        parent: Some(&MOBILE_BASE),
        ..MOBILE_BASE
    };
    static ORC_CHIEF: MobileType = MobileType {
        full_name: "Server.Mobiles.OrcChief",
        parent: Some(&ORC),
        ..ORC
    };
    static SAVAGE: MobileType = MobileType {
        full_name: "Server.Mobiles.Savage",
        parent: Some(&MOBILE_BASE),
        ..MOBILE_BASE
    };
    static WOLF: MobileType = MobileType {
        full_name: "Server.Mobiles.Wolf",
        parent: Some(&MOBILE_BASE),
        ..MOBILE_BASE
    };

    #[test]
    fn orcs_and_savages_oppose_and_subclasses_count() {
        let mut w = World::new();
        let (o, c, s, wolf) = (
            w.new_mobile(&ORC),
            w.new_mobile(&ORC_CHIEF),
            w.new_mobile(&SAVAGE),
            w.new_mobile(&WOLF),
        );
        let g = &SAVAGES_AND_ORCS;
        assert!(
            g.is_enemy(&w, o, s) && g.is_enemy(&w, s, c),
            "a subclass of Orc is in the orc group"
        );
        assert!(!g.is_enemy(&w, o, c), "same group");
        assert!(
            !g.is_enemy(&w, o, wolf) && !g.is_enemy(&w, wolf, s),
            "a class in no group is nobody's enemy"
        );
        assert_eq!(g.index_of(&w, c), 0);
        assert_eq!(g.index_of(&w, wolf), -1);
    }
}
