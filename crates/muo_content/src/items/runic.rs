//! `BaseRunicTool`'s static `ApplyAttributesTo` for weapons, armor, hats, jewelry and spellbooks (loot, quest rewards).
//! The tool-instance overloads (a crafter's runic tool) are the craft system's: they pass `runic = true`.

use muo_server::World;
use muo_server::ids::ItemId;
use muo_server::item::ops;
use muo_server::skills::SkillName;

use super::weapon_combat::{elemental_damage_hue, get_damage_types};
use super::{armor, attr, ext_bags, slayer, weapon};
use crate::aos::{
    AosArmorAttribute as AA, AosAttribute as A, AosElementAttribute as AE,
    AosWeaponAttribute as AW, AttrKind, Attrs,
};
use crate::craft::CraftResource;
use crate::factory::item_is;
use crate::hooks::rnd;

/// The roll's shared state: `m_IsRunicTool`, `m_LuckChance`, `m_Props`, and the call's `min`/`max`.
struct Rt {
    runic: bool,
    luck: i32,
    min: i32,
    max: i32,
    props: u32,
}

/// `Scale(min, max, low, high)`.
fn scale(w: &mut World, rt: &Rt, low: i32, high: i32) -> i32 {
    let mut percent = if rt.runic {
        rnd::min_max(w, rt.min, rt.max)
    } else {
        // Behold, the worst system ever!
        let v = rnd::min_max(w, 0, 10000);
        let mut v = 100 - f64::from(v).sqrt() as i32;
        if crate::loot_pack::check_luck_chance(w, rt.luck) {
            v += 10;
        }
        if v < rt.min {
            v = rt.min;
        } else if v > rt.max {
            v = rt.max;
        }
        v
    };
    percent = percent.wrapping_mul(10000 + 10000 / ((high - low).abs() + 1));
    low + (high - low).wrapping_mul(percent) / 1000001
}

/// `ApplyAttribute(bag, min, max, attr, low, high, scale)`.
#[allow(clippy::too_many_arguments)] // ApplyAttribute(bag, min, max, attr, low, high, scale)
fn put(
    w: &mut World,
    rt: &Rt,
    id: ItemId,
    kind: AttrKind,
    mask: u32,
    low: i32,
    high: i32,
    sc: i32,
) {
    let v = scale(w, rt, low / sc, high / sc) * sc;
    if kind == AttrKind::Attributes && mask == A::CastSpeed as u32 {
        let cur = attr(w, id, kind, mask);
        super::set_attr(w, id, kind, mask, cur + v);
    } else {
        super::set_attr(w, id, kind, mask, v);
    }
    if kind == AttrKind::Attributes && mask == A::SpellChanneling as u32 {
        let cur = attr(w, id, kind, A::CastSpeed as u32);
        super::set_attr(w, id, kind, A::CastSpeed as u32, cur - 1);
    }
}

/// `GetUniqueRandom(count)`: -1 when every property is taken.
fn unique(w: &mut World, rt: &mut Rt, count: i32) -> i32 {
    let avail: Vec<i32> = (0..count).filter(|i| rt.props & (1 << i) == 0).collect();
    if avail.is_empty() {
        return -1;
    }
    let v = avail[rnd::below(w, avail.len() as i32) as usize];
    rt.props |= 1 << v;
    v
}

const BONUS_SKILLS: [SkillName; 28] = [
    SkillName::Swords,
    SkillName::Fencing,
    SkillName::Macing,
    SkillName::Archery,
    SkillName::Wrestling,
    SkillName::Parry,
    SkillName::Tactics,
    SkillName::Anatomy,
    SkillName::Healing,
    SkillName::Magery,
    SkillName::Meditation,
    SkillName::EvalInt,
    SkillName::MagicResist,
    SkillName::AnimalTaming,
    SkillName::AnimalLore,
    SkillName::Veterinary,
    SkillName::Musicianship,
    SkillName::Provocation,
    SkillName::Discordance,
    SkillName::Peacemaking,
    SkillName::Chivalry,
    SkillName::Focus,
    SkillName::Necromancy,
    SkillName::Stealing,
    SkillName::Stealth,
    SkillName::SpiritSpeak,
    SkillName::Bushido,
    SkillName::Ninjitsu,
];
const BOOK_SKILLS: [SkillName; 4] = [
    SkillName::Magery,
    SkillName::Meditation,
    SkillName::EvalInt,
    SkillName::MagicResist,
];

/// `ApplySkillBonus(skills, min, max, index, low, high)`.
fn skill_bonus(w: &mut World, rt: &Rt, id: ItemId, book: bool, index: u32, low: i32, high: i32) {
    let mut possible: Vec<SkillName> = if book {
        BOOK_SKILLS.to_vec()
    } else {
        BONUS_SKILLS.to_vec()
    };
    let mut count = if w.core_se() || book {
        possible.len() as i32
    } else {
        possible.len() as i32 - 2
    };
    let aos = crate::core::aos(w);
    let sk = loop {
        let sk = possible[rnd::below(w, count) as usize];
        count -= 1;
        possible.retain(|s| *s != sk);
        let found = (0..5).any(|i| {
            w.item(id)
                .and_then(|it| super::attrs(it, AttrKind::SkillBonuses))
                .and_then(|b| b.skill_bonus(aos, i))
                .is_some_and(|(s, _)| s == sk)
        });
        if !(found && count > 0) {
            break sk;
        }
    };
    let bonus = scale(w, rt, low, high);
    super::set_attr(
        w,
        id,
        AttrKind::SkillBonuses,
        1 << index,
        Attrs::skill_bonus_value(sk, f64::from(bonus)),
    );
}

type Row = (AttrKind, u32, i32, i32, i32);
const fn pa(a: A, low: i32, high: i32, sc: i32) -> Row {
    (AttrKind::Attributes, a as u32, low, high, sc)
}
const fn pw(a: AW, low: i32, high: i32, sc: i32) -> Row {
    (AttrKind::Weapon, a as u32, low, high, sc)
}
const fn pr(a: AA, low: i32, high: i32, sc: i32) -> Row {
    (AttrKind::Armor, a as u32, low, high, sc)
}
const fn pe(a: AE, low: i32, high: i32) -> Row {
    (AttrKind::Element, a as u32, low, high, 1)
}
fn row(w: &mut World, rt: &Rt, id: ItemId, r: Row) {
    put(w, rt, id, r.0, r.1, r.2, r.3, r.4);
}

/// `ApplyAttributesTo(BaseWeapon, isRunicTool, luckChance, attributeCount, min, max)`.
pub fn apply_to_weapon(
    w: &mut World,
    id: ItemId,
    runic: bool,
    luck: i32,
    count: i32,
    min: i32,
    max: i32,
) {
    let mut rt = Rt {
        runic,
        luck,
        min,
        max,
        props: 0,
    };
    if item_is(w, id, "Server.Items.BaseRanged") {
        rt.props |= 1 << 2; // ranged weapons cannot be ubws or mageweapon
    }
    const TAIL: [Row; 20] = [
        pa(A::WeaponDamage, 1, 50, 1),
        pa(A::DefendChance, 1, 15, 1),
        pa(A::CastSpeed, 1, 1, 1),
        pa(A::AttackChance, 1, 15, 1),
        pa(A::Luck, 1, 100, 1),
        pa(A::WeaponSpeed, 5, 30, 5),
        pa(A::SpellChanneling, 1, 1, 1),
        pw(AW::HitDispel, 2, 50, 2),
        pw(AW::HitLeechHits, 2, 50, 2),
        pw(AW::HitLowerAttack, 2, 50, 2),
        pw(AW::HitLowerDefend, 2, 50, 2),
        pw(AW::HitLeechMana, 2, 50, 2),
        pw(AW::HitLeechStam, 2, 50, 2),
        pw(AW::LowerStatReq, 10, 100, 10),
        pw(AW::ResistPhysicalBonus, 1, 15, 1),
        pw(AW::ResistFireBonus, 1, 15, 1),
        pw(AW::ResistColdBonus, 1, 15, 1),
        pw(AW::ResistPoisonBonus, 1, 15, 1),
        pw(AW::ResistEnergyBonus, 1, 15, 1),
        pw(AW::DurabilityBonus, 10, 100, 10),
    ];
    const AREA: [AW; 5] = [
        AW::HitPhysicalArea,
        AW::HitFireArea,
        AW::HitColdArea,
        AW::HitPoisonArea,
        AW::HitEnergyArea,
    ];
    const SPELL: [AW; 4] = [
        AW::HitMagicArrow,
        AW::HitHarm,
        AW::HitFireball,
        AW::HitLightning,
    ];
    for _ in 0..count {
        match unique(w, &mut rt, 25) {
            -1 => break,
            0 => {
                let a = AREA[rnd::below(w, 5) as usize];
                row(w, &rt, id, pw(a, 2, 50, 2));
            }
            1 => {
                let a = SPELL[rnd::below(w, 4) as usize];
                row(w, &rt, id, pw(a, 2, 50, 2));
            }
            2 => {
                if rnd::below(w, 2) == 0 {
                    row(w, &rt, id, pw(AW::UseBestSkill, 1, 1, 1));
                } else {
                    row(w, &rt, id, pw(AW::MageWeapon, 1, 10, 1));
                }
            }
            n @ 3..=22 => row(w, &rt, id, TAIL[n as usize - 3]),
            23 => {
                let s = slayer::random_slayer(w);
                if let Some(d) = weapon::data_mut(w, id) {
                    d.slayer = s as i32;
                }
                ops::invalidate_properties(w, id);
            }
            _ => elemental_damages(w, id, true),
        }
    }
}

/// `GetElementalDamages(weapon, randomizeOrder)`.
pub fn elemental_damages(w: &mut World, id: ItemId, randomize: bool) {
    let mut total = get_damage_types(w, id, None).phys;
    let mut attrs = [AE::Cold, AE::Energy, AE::Fire, AE::Poison];
    if randomize {
        for i in 0..attrs.len() {
            let rand = rnd::below(w, attrs.len() as i32) as usize;
            attrs.swap(i, rand);
        }
    }
    for a in attrs {
        // AssignElementalDamage
        if total <= 0 {
            total = 0;
            continue;
        }
        let random = rnd::below(w, total / 10 + 1) * 10;
        super::set_attr(w, id, AttrKind::Element, a as u32, random);
        total -= random;
    }
    let hue = elemental_damage_hue(w, id);
    muo_server::item::base::set_hue(w, id, hue);
}

/// `ApplyAttributesTo(BaseArmor, ..)`.
pub fn apply_to_armor(
    w: &mut World,
    id: ItemId,
    runic: bool,
    luck: i32,
    count: i32,
    min: i32,
    max: i32,
) {
    let mut rt = Rt {
        runic,
        luck,
        min,
        max,
        props: 0,
    };
    let shield = armor::is_shield(w, id);
    let (base_count, base_offset) = if shield { (7, 0) } else { (20, 4) };
    if !shield && armor::meditation_allowance(w, id) == armor::ArmorMeditationAllowance::All {
        rt.props |= 1 << 3; // remove mage armor
    }
    let res = armor::data(w, id).map_or(CraftResource::None, |d| d.resource);
    if (CraftResource::RegularLeather as i32..=CraftResource::BarbedLeather as i32)
        .contains(&(res as i32))
    {
        rt.props |= (1 << 0) | (1 << 2); // no lower requirements, no durability bonus on leather
    }
    let races = w.item(id).map_or(0, |i| (i.ty.required_races)(w, id));
    if races == muo_server::race::ALLOW_ELVES_ONLY {
        rt.props |= 1 << 7; // elves have night sight
    }
    for _ in 0..count {
        let random = unique(w, &mut rt, base_count);
        if random == -1 {
            break;
        }
        match random + base_offset {
            0 => row(w, &rt, id, pa(A::SpellChanneling, 1, 1, 1)),
            1 => row(w, &rt, id, pa(A::DefendChance, 1, 15, 1)),
            2 => {
                if w.core_ml() {
                    row(w, &rt, id, pa(A::ReflectPhysical, 1, 15, 1));
                } else {
                    row(w, &rt, id, pa(A::AttackChance, 1, 15, 1));
                }
            }
            3 => row(w, &rt, id, pa(A::CastSpeed, 1, 1, 1)),
            4 => row(w, &rt, id, pr(AA::LowerStatReq, 10, 100, 10)),
            5 => row(w, &rt, id, pr(AA::SelfRepair, 1, 5, 1)),
            6 => row(w, &rt, id, pr(AA::DurabilityBonus, 10, 100, 10)),
            7 => row(w, &rt, id, pr(AA::MageArmor, 1, 1, 1)),
            8 => row(w, &rt, id, pa(A::RegenHits, 1, 2, 1)),
            9 => row(w, &rt, id, pa(A::RegenStam, 1, 3, 1)),
            10 => row(w, &rt, id, pa(A::RegenMana, 1, 2, 1)),
            11 => row(w, &rt, id, pa(A::NightSight, 1, 1, 1)),
            12 => row(w, &rt, id, pa(A::BonusHits, 1, 5, 1)),
            13 => row(w, &rt, id, pa(A::BonusStam, 1, 8, 1)),
            14 => row(w, &rt, id, pa(A::BonusMana, 1, 8, 1)),
            15 => row(w, &rt, id, pa(A::LowerManaCost, 1, 8, 1)),
            16 => row(w, &rt, id, pa(A::LowerRegCost, 1, 20, 1)),
            17 => row(w, &rt, id, pa(A::Luck, 1, 100, 1)),
            18 => row(w, &rt, id, pa(A::ReflectPhysical, 1, 15, 1)),
            n @ 19..=23 => {
                // ApplyResistance: Physical, Fire, Cold, Poison, Energy bonus.
                let v = scale(w, &rt, 1, 15);
                if let Some(d) = armor::data_mut(w, id) {
                    match n {
                        19 => d.physical_bonus += v,
                        20 => d.fire_bonus += v,
                        21 => d.cold_bonus += v,
                        22 => d.poison_bonus += v,
                        _ => d.energy_bonus += v,
                    }
                }
                ops::invalidate_properties(w, id);
            }
            _ => {}
        }
    }
}

/// `ApplyAttributesTo(BaseHat, ..)`.
pub fn apply_to_hat(
    w: &mut World,
    id: ItemId,
    runic: bool,
    luck: i32,
    count: i32,
    min: i32,
    max: i32,
) {
    let mut rt = Rt {
        runic,
        luck,
        min,
        max,
        props: 0,
    };
    const ROWS: [Row; 19] = [
        pa(A::ReflectPhysical, 1, 15, 1),
        pa(A::RegenHits, 1, 2, 1),
        pa(A::RegenStam, 1, 3, 1),
        pa(A::RegenMana, 1, 2, 1),
        pa(A::NightSight, 1, 1, 1),
        pa(A::BonusHits, 1, 5, 1),
        pa(A::BonusStam, 1, 8, 1),
        pa(A::BonusMana, 1, 8, 1),
        pa(A::LowerManaCost, 1, 8, 1),
        pa(A::LowerRegCost, 1, 20, 1),
        pa(A::Luck, 1, 100, 1),
        pr(AA::LowerStatReq, 10, 100, 10),
        pr(AA::SelfRepair, 1, 5, 1),
        pr(AA::DurabilityBonus, 10, 100, 10),
        pe(AE::Physical, 1, 15),
        pe(AE::Fire, 1, 15),
        pe(AE::Cold, 1, 15),
        pe(AE::Poison, 1, 15),
        pe(AE::Energy, 1, 15),
    ];
    for _ in 0..count {
        let random = unique(w, &mut rt, 19);
        if random == -1 {
            break;
        }
        row(w, &rt, id, ROWS[random as usize]);
    }
}

/// `ApplyAttributesTo(BaseJewel, ..)`.
pub fn apply_to_jewel(
    w: &mut World,
    id: ItemId,
    runic: bool,
    luck: i32,
    count: i32,
    min: i32,
    max: i32,
) {
    let mut rt = Rt {
        runic,
        luck,
        min,
        max,
        props: 0,
    };
    const ROWS: [Row; 19] = [
        pe(AE::Physical, 1, 15),
        pe(AE::Fire, 1, 15),
        pe(AE::Cold, 1, 15),
        pe(AE::Poison, 1, 15),
        pe(AE::Energy, 1, 15),
        pa(A::WeaponDamage, 1, 25, 1),
        pa(A::DefendChance, 1, 15, 1),
        pa(A::AttackChance, 1, 15, 1),
        pa(A::BonusStr, 1, 8, 1),
        pa(A::BonusDex, 1, 8, 1),
        pa(A::BonusInt, 1, 8, 1),
        pa(A::EnhancePotions, 5, 25, 5),
        pa(A::CastSpeed, 1, 1, 1),
        pa(A::CastRecovery, 1, 3, 1),
        pa(A::LowerManaCost, 1, 8, 1),
        pa(A::LowerRegCost, 1, 20, 1),
        pa(A::Luck, 1, 100, 1),
        pa(A::SpellDamage, 1, 12, 1),
        pa(A::NightSight, 1, 1, 1),
    ];
    for _ in 0..count {
        match unique(w, &mut rt, 24) {
            -1 => break,
            n @ 0..=18 => row(w, &rt, id, ROWS[n as usize]),
            n => skill_bonus(w, &rt, id, false, n as u32 - 19, 1, 15),
        }
    }
}

/// `ApplyAttributesTo(Spellbook, ..)`.
pub fn apply_to_spellbook(
    w: &mut World,
    id: ItemId,
    runic: bool,
    luck: i32,
    count: i32,
    min: i32,
    max: i32,
) {
    let mut rt = Rt {
        runic,
        luck,
        min,
        max,
        props: 0,
    };
    for _ in 0..count {
        match unique(w, &mut rt, 16) {
            -1 => break,
            0..=3 => {
                row(w, &rt, id, pa(A::BonusInt, 1, 8, 1));
                rt.props |= 0xF;
            }
            4 => row(w, &rt, id, pa(A::BonusMana, 1, 8, 1)),
            5 => row(w, &rt, id, pa(A::CastSpeed, 1, 1, 1)),
            6 => row(w, &rt, id, pa(A::CastRecovery, 1, 3, 1)),
            7 => row(w, &rt, id, pa(A::SpellDamage, 1, 12, 1)),
            n @ 8..=11 => skill_bonus(w, &rt, id, true, n as u32 - 8, 1, 15),
            12 => row(w, &rt, id, pa(A::LowerRegCost, 1, 20, 1)),
            13 => row(w, &rt, id, pa(A::LowerManaCost, 1, 8, 1)),
            14 => row(w, &rt, id, pa(A::RegenMana, 1, 2, 1)),
            _ => {
                let s = slayer::random_slayer(w);
                if let Some(d) = w
                    .item_mut(id)
                    .and_then(|i| i.ext.get_mut::<ext_bags::SpellbookData>())
                {
                    d.slayer = s as i32;
                }
                ops::invalidate_properties(w, id);
            }
        }
    }
}

/// The loot hook and the quests' `apply_runic_attributes`: whichever kind of item it is.
pub fn apply_to_item(
    w: &mut World,
    id: ItemId,
    runic: bool,
    luck: i32,
    count: i32,
    min: i32,
    max: i32,
) {
    if item_is(w, id, "Server.Items.BaseWeapon") {
        apply_to_weapon(w, id, runic, luck, count, min, max);
    } else if item_is(w, id, "Server.Items.BaseArmor") {
        apply_to_armor(w, id, runic, luck, count, min, max);
    } else if item_is(w, id, "Server.Items.BaseJewel") {
        apply_to_jewel(w, id, runic, luck, count, min, max);
    } else if item_is(w, id, "Server.Items.BaseHat") {
        apply_to_hat(w, id, runic, luck, count, min, max);
    } else if w
        .item(id)
        .is_some_and(|i| i.ext.get::<ext_bags::SpellbookData>().is_some())
    {
        apply_to_spellbook(w, id, runic, luck, count, min, max);
    }
}
