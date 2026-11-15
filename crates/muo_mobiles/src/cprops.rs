//! Constant `CreatureType` members of the creature classes, read from their template `"props"` (the member name in
//! camelCase). A class whose member was a constant points it here; without the prop anywhere in its class chain a
//! member answers what `BASE_CREATURE_C` answers. See `muo_content::templates::creature_props`.
//!
//! Enum props are names (`"favoriteFood": ["Meat", "Fish"]`, `"hideType": "Spined"`). They are parsed once per
//! template table into [`EnumCache`]; a read is a lookup by class name. The cache is rebuilt by the creature
//! constructors of this crate after a `[ReloadTemplates` (until then a read parses the prop itself).

use std::collections::HashMap;
use std::rc::Rc;

use serde_json::Value;

use muo_content::ai::opposition_group::{self as og, OppositionGroup};
use muo_content::creature::BASE_CREATURE_C;
use muo_content::creature::enums::{FoodType, HideType, MeatType, PackInstinct, ScaleType};
use muo_content::items::weapon::Ability;
use muo_content::templates::props::{self, as_bool, as_f64, as_i32};
use muo_content::templates::{self, Kind, TemplateTable};
use muo_primitives::TimeSpan;
use muo_server::ids::MobileId;
use muo_server::mobile::MobileType;
use muo_server::poison::PoisonId;
use muo_server::world::World;

macro_rules! bool_prop {
    ($($f:ident = $key:literal;)*) => {$(
        pub fn $f(w: &World, id: MobileId) -> bool {
            as_bool(props::mobile(w, id, $key)).unwrap_or_else(|| (BASE_CREATURE_C.$f)(w, id))
        }
    )*};
}

macro_rules! int_prop {
    ($($f:ident = $key:literal;)*) => {$(
        pub fn $f(w: &World, id: MobileId) -> i32 {
            as_i32(props::mobile(w, id, $key)).unwrap_or_else(|| (BASE_CREATURE_C.$f)(w, id))
        }
    )*};
}

macro_rules! f64_prop {
    ($($f:ident = $key:literal;)*) => {$(
        pub fn $f(w: &World, id: MobileId) -> f64 {
            as_f64(props::mobile(w, id, $key)).unwrap_or_else(|| (BASE_CREATURE_C.$f)(w, id))
        }
    )*};
}

/// A delay in seconds.
macro_rules! secs_prop {
    ($($f:ident = $key:literal;)*) => {$(
        pub fn $f(w: &World, id: MobileId) -> TimeSpan {
            match as_f64(props::mobile(w, id, $key)) {
                Some(s) => TimeSpan::from_secs_f64(s),
                None => (BASE_CREATURE_C.$f)(w, id),
            }
        }
    )*};
}

bool_prop! {
    bard_immune = "bardImmune";
    unprovokable = "unprovokable";
    uncalmable = "uncalmable";
    area_peace_immune = "areaPeaceImmune";
    auto_dispel = "autoDispel";
    initial_innocent = "initialInnocent";
    delete_corpse_on_death = "deleteCorpseOnDeath";
    reacquire_on_movement = "reacquireOnMovement";
    can_anger_on_tame = "canAngerOnTame";
    death_adder_charmable = "deathAdderCharmable";
    stat_loss_after_tame = "statLossAfterTame";
    override_bonding_reqs = "overrideBondingReqs";
    is_scared_of_scary_things = "isScaredOfScaryThings";
    is_scary_to_pets = "isScaryToPets";
    always_attackable = "alwaysAttackable";
    can_heal = "canHeal";
    can_heal_owner = "canHealOwner";
    subdue_before_tame = "subdueBeforeTame";
    reduce_speed_with_damage = "reduceSpeedWithDamage";
    delete_on_release = "deleteOnRelease";
    disallow_all_moves = "disallowAllMoves";
    allow_male_rider = "allowMaleRider";
    allow_female_rider = "allowFemaleRider";
    allow_male_tamer = "allowMaleTamer";
    allow_female_tamer = "allowFemaleTamer";
    is_bondable = "isBondable";
    no_house_restrictions = "noHouseRestrictions";
    has_mana_override = "hasManaOverride";
    has_aura = "hasAura";
    is_house_summonable = "isHouseSummonable";
    breath_immune = "breathImmune";
}

int_prop! {
    default_hue = "defaultHue";
    aura_range = "auraRange";
    aura_fire_damage = "auraFireDamage";
    aura_cold_damage = "auraColdDamage";
}

f64_prop! {
    dispel_difficulty = "dispelDifficulty";
    dispel_focus = "dispelFocus";
    hit_poison_chance = "hitPoisonChance";
    weapon_ability_chance = "weaponAbilityChance";
    treasure_map_chance = "treasureMapChance";
}

secs_prop! {
    mount_ability_delay = "mountAbilityDelay";
    reacquire_delay = "reacquireDelay";
    aura_interval = "auraInterval";
}

/// `"hitPoison": <level>` or `null`.
pub fn hit_poison(w: &mut World, id: MobileId) -> Option<PoisonId> {
    match props::mobile(w, id, "hitPoison") {
        Some(Value::Null) => None,
        Some(v) => v.as_u64().and_then(|n| u8::try_from(n).ok()).map(PoisonId),
        None => (BASE_CREATURE_C.hit_poison)(w, id),
    }
}

// ---- loot ----

fn loot_pack(w: &World, name: &str) -> Option<&'static muo_content::loot_pack::LootPack> {
    use muo_content::loot_pack as lp;
    Some(match name {
        "Poor" => lp::poor(w),
        "Meager" => lp::meager(w),
        "Average" => lp::average(w),
        "Rich" => lp::rich(w),
        "FilthyRich" => lp::filthy_rich(w),
        "UltraRich" => lp::ultra_rich(w),
        "SuperBoss" => lp::super_boss(w),
        "Gems" => &lp::GEMS,
        "LowScrolls" => &lp::LOW_SCROLLS,
        "MedScrolls" => &lp::MED_SCROLLS,
        "HighScrolls" => &lp::HIGH_SCROLLS,
        "Potions" => &lp::POTIONS,
        "AosRich" => &lp::AOS_RICH,
        "AosFilthyRich" => &lp::AOS_FILTHY_RICH,
        "AosUltraRich" => &lp::AOS_ULTRA_RICH,
        "AosSuperBoss" => &lp::AOS_SUPER_BOSS,
        "MlRich" => &lp::ML_RICH,
        _ => return None,
    })
}

/// `"loot"` with the forms the uniform `templates::creature_props::generate_loot` lacks, run in order:
/// `[pack, n]` where `n` is a number or `{"min": a, "max": b}` / `{"from": a, "count": n}` (drawn then),
/// `{"era": "ML", "loot": [..]}` (only from that era on) and `{"pack": "Weapon"}` (`PackItem(Loot.RandomWeapon())`).
pub fn generate_loot(w: &mut World, id: MobileId) {
    let Some(list) = props::mobile(w, id, "loot")
        .and_then(Value::as_array)
        .cloned()
    else {
        return (BASE_CREATURE_C.generate_loot)(w, id);
    };
    loot_list(w, id, &list);
}

fn loot_list(w: &mut World, id: MobileId, list: &[Value]) {
    use muo_content::hooks::rnd;
    for e in list {
        if let Some([name, n]) = e.as_array().map(Vec::as_slice) {
            let count = match n {
                Value::Object(o) => {
                    let g = |k: &str| as_i32(o.get(k)).unwrap_or(0);
                    if o.contains_key("from") {
                        rnd::from(w, g("from"), g("count"))
                    } else {
                        rnd::min_max(w, g("min"), g("max"))
                    }
                }
                n => as_i32(Some(n)).unwrap_or(0),
            };
            match name.as_str().and_then(|s| loot_pack(w, s)) {
                Some(p) => muo_content::creature::setters::add_loot_n(w, id, p, count),
                None => tracing::warn!("Templates: a bad \"loot\" entry {e}"),
            }
        } else if let Some(era) = e.get("era").and_then(Value::as_str) {
            let at = (0..64)
                .filter_map(muo_data::expansion::Expansion::from_index)
                .find(|x| format!("{x:?}") == era);
            if at.is_some_and(|a| w.expansion() >= a) {
                let inner = e
                    .get("loot")
                    .and_then(Value::as_array)
                    .cloned()
                    .unwrap_or_default();
                loot_list(w, id, &inner);
            }
        } else if e.get("pack").and_then(Value::as_str) == Some("Weapon") {
            let it = muo_content::loot::random_weapon(w, false, false, false);
            muo_content::creature::setters::pack_item(w, id, it);
        } else {
            tracing::warn!("Templates: a bad \"loot\" entry {e}");
        }
    }
}

// ---- enum props ----

/// The enum props of one template (its `inherits` chain included), parsed.
#[derive(Default, Clone, Copy)]
pub struct Enums {
    favorite_food: Option<FoodType>,
    pack_instinct: Option<PackInstinct>,
    meat_type: Option<MeatType>,
    hide_type: Option<HideType>,
    scale_type: Option<ScaleType>,
    opposition_group: Option<Option<&'static OppositionGroup>>,
    weapon_ability: Option<Option<Ability>>,
}

/// [`Enums`] by template name, and the table they were parsed from (held, so its address is not reused).
pub struct EnumCache {
    table: Rc<TemplateTable>,
    by_class: HashMap<String, Enums>,
}

fn parse(get: impl Fn(&str) -> Option<Value>) -> Enums {
    Enums {
        favorite_food: get("favoriteFood").and_then(|v| flags(&v, FoodType::from_name)),
        pack_instinct: get("packInstinct").and_then(|v| flags(&v, PackInstinct::from_name)),
        meat_type: get("meatType").and_then(|v| named(&v, MeatType::from_i32)),
        hide_type: get("hideType").and_then(|v| named(&v, HideType::from_i32)),
        scale_type: get("scaleType").and_then(|v| named(&v, ScaleType::from_i32)),
        opposition_group: get("oppositionGroup").map(|v| group(&v)),
        weapon_ability: get("weaponAbility").map(|v| ability(&v)),
    }
}

/// `["FruitsAndVeggies", "GrainsAndHay"]`: the flag names in CamelCase.
fn flags<T: std::ops::BitOr<Output = T> + Default>(
    v: &Value,
    from: fn(&str) -> Option<T>,
) -> Option<T> {
    let mut out = T::default();
    for n in v.as_array()? {
        out = out | from(&screaming(n.as_str()?))?;
    }
    Some(out)
}

fn screaming(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 4);
    for (i, c) in s.chars().enumerate() {
        if c.is_ascii_uppercase() && i > 0 {
            out.push('_');
        }
        out.push(c.to_ascii_uppercase());
    }
    out
}

fn named<T: std::fmt::Debug>(v: &Value, from: fn(i32) -> Option<T>) -> Option<T> {
    let s = v.as_str()?;
    (0..64).filter_map(from).find(|x| format!("{x:?}") == s)
}

const GROUPS: [(&str, &OppositionGroup); 3] = [
    ("TerathansAndOphidians", &og::TERATHANS_AND_OPHIDIANS),
    ("SavagesAndOrcs", &og::SAVAGES_AND_ORCS),
    ("FeyAndUndead", &og::FEY_AND_UNDEAD),
];

fn group(v: &Value) -> Option<&'static OppositionGroup> {
    let s = v.as_str()?;
    GROUPS.iter().find(|(n, _)| *n == s).map(|(_, g)| *g)
}

const ABILITIES: [Ability; 31] = {
    use Ability::*;
    [
        ArmorIgnore,
        BleedAttack,
        ConcussionBlow,
        CrushingBlow,
        Disarm,
        Dismount,
        DoubleStrike,
        InfectiousStrike,
        MortalStrike,
        MovingShot,
        ParalyzingBlow,
        ShadowStrike,
        WhirlwindAttack,
        RidingSwipe,
        FrenziedWhirlwind,
        Block,
        DefenseMastery,
        NerveStrike,
        TalonStrike,
        Feint,
        DualWield,
        DoubleShot,
        ArmorPierce,
        Bladeweave,
        ForceArrow,
        LightningArrow,
        PsychicAttack,
        SerpentArrow,
        ForceOfNature,
        InfusedThrow,
        MysticArc,
    ]
};

fn ability(v: &Value) -> Option<Ability> {
    let s = v.as_str()?;
    ABILITIES.iter().copied().find(|a| format!("{a:?}") == s)
}

/// Parses the enum props of every template of `t`.
fn build(t: Rc<TemplateTable>) -> EnumCache {
    let by_class = t
        .mobiles
        .keys()
        .map(|name| {
            let e = parse(|key| {
                t.chain(Kind::Mobile, name)
                    .find_map(|tm| tm.props.get(key))
                    .cloned()
            });
            (name.clone(), e)
        })
        .collect();
    EnumCache { table: t, by_class }
}

/// Rebuilds the cache when the loaded table is not the one it was parsed from.
pub fn refresh(w: &mut World) {
    let t = templates::table(w);
    let fresh = w
        .resources
        .get::<EnumCache>()
        .is_some_and(|c| Rc::ptr_eq(&c.table, &t));
    if !fresh {
        let c = build(t);
        w.resources.insert(c);
    }
}

/// `new T()` of a creature class whose constructor is its template ([`templates::new_creature`]).
pub fn new_creature(w: &mut World, ty: &'static MobileType) -> MobileId {
    refresh(w);
    templates::new_creature(w, ty)
}

/// `new T()` of a mount class: the `BaseMount(body, itemID, ..)` arguments are the props `mountBody` and
/// `mountItemId`, the rest is the template.
pub fn new_mount(w: &mut World, ty: &'static MobileType) -> MobileId {
    refresh(w);
    let t = templates::table(w);
    let a = t.creature_args(ty.full_name);
    let prop = |key: &str| {
        std::iter::successors(Some(ty), |t| t.parent)
            .find_map(|c| {
                t.chain(Kind::Mobile, c.full_name)
                    .find_map(|tm| tm.props.get(key))
            })
            .and_then(|v| as_i32(Some(v)))
            .unwrap_or(0)
    };
    let (body, item) = (prop("mountBody"), prop("mountItemId"));
    let id = muo_content::creature::mount::new_mount(
        w,
        ty,
        body,
        item,
        a.ai,
        a.fight_mode,
        a.range_perception,
        a.range_fight,
    );
    templates::apply_mobile(w, id);
    id
}

/// The parsed enum `field` of the mobile's class chain; `parse_one` for a stale cache.
fn lookup<T: Copy>(
    w: &World,
    id: MobileId,
    key: &str,
    field: fn(&Enums) -> Option<T>,
) -> Option<T> {
    let ty = w.mobile(id)?.ty;
    let loaded = templates::loaded(w)?;
    match w.resources.get::<EnumCache>() {
        Some(c) if std::ptr::eq(&*c.table, loaded) => std::iter::successors(Some(ty), |t| t.parent)
            .find_map(|t| c.by_class.get(t.full_name).and_then(field)),
        _ => {
            let v = props::mobile(w, id, key)?.clone();
            field(&parse(|k| (k == key).then(|| v.clone())))
        }
    }
}

pub fn favorite_food(w: &World, id: MobileId) -> FoodType {
    lookup(w, id, "favoriteFood", |e| e.favorite_food)
        .unwrap_or_else(|| (BASE_CREATURE_C.favorite_food)(w, id))
}

pub fn pack_instinct(w: &World, id: MobileId) -> PackInstinct {
    lookup(w, id, "packInstinct", |e| e.pack_instinct)
        .unwrap_or_else(|| (BASE_CREATURE_C.pack_instinct)(w, id))
}

pub fn meat_type(w: &World, id: MobileId) -> MeatType {
    lookup(w, id, "meatType", |e| e.meat_type).unwrap_or_else(|| (BASE_CREATURE_C.meat_type)(w, id))
}

pub fn hide_type(w: &World, id: MobileId) -> HideType {
    lookup(w, id, "hideType", |e| e.hide_type).unwrap_or_else(|| (BASE_CREATURE_C.hide_type)(w, id))
}

pub fn scale_type(w: &mut World, id: MobileId) -> ScaleType {
    match lookup(w, id, "scaleType", |e| e.scale_type) {
        Some(s) => s,
        None => (BASE_CREATURE_C.scale_type)(w, id),
    }
}

/// `"oppositionGroup": "FeyAndUndead"` (or `null`).
pub fn opposition_group(w: &World, id: MobileId) -> Option<&'static OppositionGroup> {
    lookup(w, id, "oppositionGroup", |e| e.opposition_group)
        .unwrap_or_else(|| (BASE_CREATURE_C.opposition_group)(w, id))
}

/// `"weaponAbility": "BleedAttack"` (or `null`).
pub fn get_weapon_ability(w: &mut World, id: MobileId) -> Option<Ability> {
    match lookup(w, id, "weaponAbility", |e| e.weapon_ability) {
        Some(a) => a,
        None => (BASE_CREATURE_C.get_weapon_ability)(w, id),
    }
}

/// Builds the cache once the templates are loaded.
pub(crate) fn register(types: &mut muo_server::type_registry::TypeRegistry) {
    types.register_phase(
        muo_server::type_registry::Phase::Configure,
        "muo_mobiles.CreatureProps.Configure",
        1,
        |w| {
            if templates::loaded(w).is_some() {
                refresh(w);
            }
        },
    );
}

/// The `"<class>: <key>"` of every enum prop of `t` that does not name a value.
#[cfg(test)]
pub(crate) fn bad_enum_props(t: &TemplateTable) -> Vec<String> {
    let mut bad = Vec::new();
    for (name, tm) in &t.mobiles {
        let e = parse(|k| tm.props.get(k).cloned());
        let ok =
            |key: &str, parsed: bool| !tm.props.get(key).is_some_and(|v| !v.is_null() && !parsed);
        let checks = [
            ("favoriteFood", e.favorite_food.is_some()),
            ("packInstinct", e.pack_instinct.is_some()),
            ("meatType", e.meat_type.is_some()),
            ("hideType", e.hide_type.is_some()),
            ("scaleType", e.scale_type.is_some()),
            ("oppositionGroup", e.opposition_group.flatten().is_some()),
            ("weaponAbility", e.weapon_ability.flatten().is_some()),
        ];
        for (key, parsed) in checks {
            if !ok(key, parsed) {
                bad.push(format!("{name}: {key}"));
            }
        }
    }
    bad
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enum_names_parse() {
        let e = parse(|k| match k {
            "favoriteFood" => Some(serde_json::json!(["FruitsAndVeggies", "GrainsAndHay"])),
            "packInstinct" => Some(serde_json::json!(["Daemon", "Equine"])),
            "hideType" => Some(serde_json::json!("Spined")),
            "oppositionGroup" => Some(serde_json::json!("FeyAndUndead")),
            "weaponAbility" => Some(serde_json::json!("BleedAttack")),
            _ => None,
        });
        assert_eq!(
            e.favorite_food,
            Some(FoodType::FRUITS_AND_VEGGIES | FoodType::GRAINS_AND_HAY)
        );
        assert_eq!(
            e.pack_instinct,
            Some(PackInstinct::DAEMON | PackInstinct::EQUINE)
        );
        assert_eq!(e.hide_type, Some(HideType::Spined));
        assert!(
            e.opposition_group
                .flatten()
                .is_some_and(|g| std::ptr::eq(g, &og::FEY_AND_UNDEAD))
        );
        assert_eq!(e.weapon_ability, Some(Some(Ability::BleedAttack)));
        assert_eq!(e.meat_type, None);
    }
}
