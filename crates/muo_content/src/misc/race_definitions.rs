//! The three registered races (Human, Elf, Gargoyle).

use muo_data::expansion::Expansion;
use muo_primitives::{BuiltInRng, utility};
use muo_server::World;
use muo_server::race::{Race, RaceOps, Races};

fn between(v: i32, lo: i32, hi: i32) -> bool {
    (lo..=hi).contains(&v)
}

/// `RandomElement()` of a non-empty array.
fn random_element(rng: &mut BuiltInRng, list: &[i32]) -> i32 {
    list[utility::random(rng, list.len() as i32) as usize]
}

fn first_if_unlisted(list: &[i32], hue: i32) -> i32 {
    if list.contains(&hue) { hue } else { list[0] }
}

// ---- Human ----

fn human_validate_hair(female: bool, id: i32) -> bool {
    if id == 0 {
        return true;
    }
    if (female && id == 0x2048) || (!female && id == 0x2046) {
        return false; // Buns & Receding Hair
    }
    between(id, 0x203B, 0x203D) || between(id, 0x2044, 0x204A)
}

fn human_random_hair(rng: &mut BuiltInRng, female: bool) -> i32 {
    // Random hair doesn't include baldness
    match utility::random(rng, 9) {
        0 => 0x203B, // Short
        1 => 0x203C, // Long
        2 => 0x203D, // Pony Tail
        3 => 0x2044, // Mohawk
        4 => 0x2045, // Pageboy
        5 => 0x2047, // Afro
        6 => 0x2049, // Pig tails
        7 => 0x204A, // Krisna
        _ => {
            if female {
                0x2046
            } else {
                0x2048
            }
        }
    }
}

fn human_validate_facial_hair(female: bool, id: i32) -> bool {
    if id == 0 {
        return true;
    }
    if female {
        return false;
    }
    between(id, 0x203E, 0x2041) || between(id, 0x204B, 0x204D)
}

fn human_random_facial_hair(rng: &mut BuiltInRng, female: bool) -> i32 {
    if female {
        return 0;
    }
    let rand = utility::random(rng, 7);
    (if rand < 4 { 0x203E } else { 0x2047 }) + rand
}

fn human_random_skin_hue(rng: &mut BuiltInRng) -> i32 {
    utility::random_from(rng, 1002, 57) | 0x8000
}

fn human_random_hair_hue(rng: &mut BuiltInRng) -> i32 {
    utility::random_from(rng, 1102, 48)
}

static HUMAN_OPS: RaceOps = RaceOps {
    validate_hair: human_validate_hair,
    random_hair: human_random_hair,
    validate_facial_hair: human_validate_facial_hair,
    random_facial_hair: human_random_facial_hair,
    clip_skin_hue: |hue| hue.clamp(1002, 1058),
    random_skin_hue: human_random_skin_hue,
    clip_hair_hue: |hue| hue.clamp(1102, 1149),
    random_hair_hue: human_random_hair_hue,
};

// ---- Elf ----

const ELF_SKIN_HUES: [i32; 32] = [
    0x0BF, 0x24D, 0x24E, 0x24F, 0x353, 0x361, 0x367, 0x374, 0x375, 0x376, 0x381, 0x382, 0x383,
    0x384, 0x385, 0x389, 0x3DE, 0x3E5, 0x3E6, 0x3E8, 0x3E9, 0x430, 0x4A7, 0x4DE, 0x51D, 0x53F,
    0x579, 0x76B, 0x76C, 0x76D, 0x835, 0x903,
];

const ELF_HAIR_HUES: [i32; 51] = [
    0x034, 0x035, 0x036, 0x037, 0x038, 0x039, 0x058, 0x08E, 0x08F, 0x090, 0x091, 0x092, 0x101,
    0x159, 0x15A, 0x15B, 0x15C, 0x15D, 0x15E, 0x128, 0x12F, 0x1BD, 0x1E4, 0x1F3, 0x207, 0x211,
    0x239, 0x251, 0x26C, 0x2C3, 0x2C9, 0x31D, 0x31E, 0x31F, 0x320, 0x321, 0x322, 0x323, 0x324,
    0x325, 0x326, 0x369, 0x386, 0x387, 0x388, 0x389, 0x38A, 0x59D, 0x6B8, 0x725, 0x853,
];

fn elf_validate_hair(female: bool, id: i32) -> bool {
    if id == 0 {
        return true;
    }
    if (female && matches!(id, 0x2FCD | 0x2FBF)) || (!female && matches!(id, 0x2FCC | 0x2FD0)) {
        return false;
    }
    between(id, 0x2FBF, 0x2FC2) || between(id, 0x2FCC, 0x2FD1)
}

fn elf_random_hair(rng: &mut BuiltInRng, female: bool) -> i32 {
    // Random hair doesn't include baldness
    match utility::random(rng, 8) {
        0 => 0x2FC0, // Long Feather
        1 => 0x2FC1, // Short
        2 => 0x2FC2, // Mullet
        3 => 0x2FCE, // Knob
        4 => 0x2FCF, // Braided
        5 => 0x2FD1, // Spiked
        6 => {
            if female { 0x2FCC } else { 0x2FBF } // Flower or Mid-long
        }
        _ => {
            if female {
                0x2FD0
            } else {
                0x2FCD
            }
        }
    }
}

static ELF_OPS: RaceOps = RaceOps {
    validate_hair: elf_validate_hair,
    random_hair: elf_random_hair,
    validate_facial_hair: |_, id| id == 0,
    random_facial_hair: |_, _| 0,
    clip_skin_hue: |hue| first_if_unlisted(&ELF_SKIN_HUES, hue),
    random_skin_hue: |rng| random_element(rng, &ELF_SKIN_HUES) | 0x8000,
    clip_hair_hue: |hue| first_if_unlisted(&ELF_HAIR_HUES, hue),
    random_hair_hue: |rng| random_element(rng, &ELF_HAIR_HUES),
};

// ---- Gargoyle ----

const HORN_HUES: [i32; 18] = [
    0x709, 0x70B, 0x70D, 0x70F, 0x711, 0x763, 0x765, 0x768, 0x76B, 0x6F3, 0x6F1, 0x6EF, 0x6E4,
    0x6E2, 0x6E0, 0x709, 0x70B, 0x70D,
];

fn gargoyle_validate_hair(female: bool, id: i32) -> bool {
    if !female {
        return between(id, 0x4258, 0x425F);
    }
    matches!(
        id,
        0x4261 | 0x4262 | 0x4273..=0x4275 | 0x42B0 | 0x42B1 | 0x42AA | 0x42AB
    )
}

fn gargoyle_random_hair(rng: &mut BuiltInRng, female: bool) -> i32 {
    if utility::random(rng, 9) == 0 {
        return 0;
    }
    if !female {
        return 0x4258 + utility::random(rng, 8);
    }
    match utility::random(rng, 9) {
        0 => 0x4261,
        1 => 0x4262,
        2 => 0x4273,
        3 => 0x4274,
        4 => 0x4275,
        5 => 0x42B0,
        6 => 0x42B1,
        7 => 0x42AA,
        8 => 0x42AB,
        _ => 0,
    }
}

fn gargoyle_random_facial_hair(rng: &mut BuiltInRng, female: bool) -> i32 {
    if female {
        return 0;
    }
    // Utility.RandomList(0, 0x42AD, 0x42AE, 0x42AF, 0x42B0)
    [0, 0x42AD, 0x42AE, 0x42AF, 0x42B0][utility::random(rng, 5) as usize]
}

static GARGOYLE_OPS: RaceOps = RaceOps {
    validate_hair: gargoyle_validate_hair,
    random_hair: gargoyle_random_hair,
    validate_facial_hair: |female, id| !female && between(id, 0x42AD, 0x42B0),
    random_facial_hair: gargoyle_random_facial_hair,
    clip_skin_hue: |hue| hue,
    random_skin_hue: |rng| utility::random_from(rng, 1755, 25) | 0x8000,
    clip_hair_hue: |hue| first_if_unlisted(&HORN_HUES, hue),
    random_hair_hue: |rng| random_element(rng, &HORN_HUES),
};

/// `RaceDefinitions.Configure`: races 0 to 2 are reserved for core use.
pub fn configure(w: &mut World) {
    let races = w.resources.get_mut::<Races>();
    races.register(Race::new(
        0,
        0,
        "Human",
        "Humans",
        400,
        401,
        402,
        403,
        Expansion::None,
        &HUMAN_OPS,
    ));
    races.register(Race::new(
        1,
        1,
        "Elf",
        "Elves",
        605,
        606,
        607,
        608,
        Expansion::ML,
        &ELF_OPS,
    ));
    races.register(Race::new(
        2,
        2,
        "Gargoyle",
        "Gargoyles",
        666,
        667,
        402,
        403,
        Expansion::SA,
        &GARGOYLE_OPS,
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registers_three_races_and_validates_hair() {
        let mut w = World::new();
        configure(&mut w);
        let races = w.resources.get::<Races>().unwrap();
        assert_eq!(races.all_races.len(), 3);
        let elf = races
            .try_parse("elves")
            .map(|r| races.get(r).unwrap())
            .unwrap();
        assert!((elf.ops.validate_hair)(false, 0x2FC0));
        assert!(!(elf.ops.validate_hair)(true, 0x2FBF));
        assert_eq!((elf.ops.clip_skin_hue)(1), 0x0BF);
        let human = races.default_race().unwrap();
        assert_eq!((human.ops.clip_skin_hue)(5000), 1058);
        assert!(!(human.ops.validate_hair)(true, 0x2048));
        assert!((races
            .get(muo_server::race::RaceId(2))
            .unwrap()
            .ops
            .validate_hair)(false, 0x425A));
    }
}
