//! `BaseWand` (a `BaseBashing` weapon with charges that casts a spell or targets) and
//! its eleven wands, `WandTarget`. State: `WeaponData::{wand_effect, charges}`; the wand constants are the weapon
//! `WeaponClass` of `BaseWand`, inherited as `..BASE_WAND_CLASS`. [`configure`] sets the spell hooks
//! (`wand_consume_charge`, `wand_charges`) the spell base calls.

use std::rc::Rc;

use muo_content::aos::{AosAttribute, AosWeaponAttribute, AttrKind};
use muo_content::craft::CraftResource;
use muo_content::items::weapon::{
    self, Ability, BASE_BASHING, BASE_BASHING_CLASS, WeaponClass, new_weapon, set_resource,
};
use muo_content::items::{core_uotd, same, set_attr};
use muo_content::spells::{registry, spell};
use muo_io::IoError;
use muo_primitives::{TimeSpan, utility};
use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ItemType, LootType, deps, ops};
use muo_server::packets::equipment::{EquipInfoAttribute, send_display_equipment_info};
use muo_server::property_list::PropertyList;
use muo_server::target::{self, TargetFlags, Targeted};
use muo_server::timer;
use muo_server::type_registry::{Phase, TypeRegistry};

use muo_content::spells::hooks::SpellHooks;

/// `BeginAction<BaseWand>()`'s key.
const WAND_ACTION: &str = "Server.Items.BaseWand";

/// `WandEffect`.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(i32)]
pub enum WandEffect {
    Clumsiness = 0,
    Identification = 1,
    Healing = 2,
    Feeblemindedness = 3,
    Weakness = 4,
    MagicArrow = 5,
    Harming = 6,
    Fireball = 7,
    GreaterHealing = 8,
    Lightning = 9,
    ManaDraining = 10,
}

impl WandEffect {
    pub fn from_i32(v: i32) -> Option<WandEffect> {
        use WandEffect::*;
        Some(match v {
            0 => Clumsiness,
            1 => Identification,
            2 => Healing,
            3 => Feeblemindedness,
            4 => Weakness,
            5 => MagicArrow,
            6 => Harming,
            7 => Fireball,
            8 => GreaterHealing,
            9 => Lightning,
            10 => ManaDraining,
            _ => return None,
        })
    }

    /// The `GetProperties` line: "~name~ charges: ~1_val~".
    fn properties_number(self) -> i32 {
        use WandEffect::*;
        match self {
            Clumsiness => 1017326,
            Identification => 1017350,
            Healing => 1017329,
            Feeblemindedness => 1017327,
            Weakness => 1017328,
            MagicArrow => 1060492,
            Harming => 1017334,
            Fireball => 1060487,
            GreaterHealing => 1017330,
            Lightning => 1060491,
            ManaDraining => 1017339,
        }
    }

    /// The `OnSingleClick` equip info entry.
    fn equip_info_number(self) -> i32 {
        use WandEffect::*;
        match self {
            Clumsiness => 3002011,
            Identification => 1044063,
            Healing => 3002014,
            Feeblemindedness => 3002013,
            Weakness => 3002018,
            MagicArrow => 3002015,
            Harming => 3002022,
            Fireball => 3002028,
            GreaterHealing => 3002039,
            Lightning => 3002040,
            ManaDraining => 3002041,
        }
    }

    /// `WandEffectText`.
    fn text(self) -> &'static str {
        use WandEffect::*;
        match self {
            Clumsiness => "clumsiness",
            Identification => "identification",
            Healing => "healing",
            Feeblemindedness => "feeblemindedness",
            Weakness => "weakness",
            MagicArrow => "magic arrow",
            Harming => "harming",
            Fireball => "fireball",
            GreaterHealing => "greater healing",
            Lightning => "lightning",
            ManaDraining => "mana draining",
        }
    }
}

/// `WandEffect`.
pub fn wand_effect(w: &World, id: ItemId) -> Option<WandEffect> {
    weapon::data(w, id).and_then(|d| WandEffect::from_i32(d.wand_effect))
}

/// `Charges`.
pub fn charges(w: &World, id: ItemId) -> i32 {
    weapon::data(w, id).map_or(0, |d| d.charges)
}

/// `Charges` setter (`[InvalidateProperties]`).
pub fn set_charges(w: &mut World, id: ItemId, value: i32) {
    if let Some(d) = weapon::data_mut(w, id)
        && d.charges != value
    {
        d.charges = value;
        ops::invalidate_properties(w, id);
    }
}

/// `new BaseWand(effect, minCharges, maxCharges)` for class `ty`: the arguments are the class's `wandEffect`,
/// `minCharges` and `maxCharges` (`n`, or `[preML, ML]`) props.
pub fn new_wand(w: &mut World, ty: &'static ItemType) -> ItemId {
    // `base(0xDF2 + Utility.Random(4))`
    let graphic = 0xDF2 + utility::random(&mut w.rng, 4);
    let id = new_weapon(w, ty, graphic);
    let effect = crate::class_props::str_of(w, id, "wandEffect")
        .and_then(|n| (0..=10).filter_map(WandEffect::from_i32).find(|e| format!("{e:?}") == n));
    let min_charges = muo_content::templates::item_props::int(w, id, "minCharges", 0);
    let ml = muo_content::core::ml(w);
    let max_charges =
        muo_content::templates::item_props::ml_int(w, id, "maxCharges", muo_content::items::same(0))
            .get(ml);
    if let Some(d) = weapon::data_mut(w, id) {
        d.wand_effect = effect.map_or(0, |e| e as i32);
    }
    let n = utility::random_min_max(&mut w.rng, min_charges, max_charges);
    if let Some(d) = weapon::data_mut(w, id) {
        d.charges = n;
    }
    set_attr(
        w,
        id,
        AttrKind::Attributes,
        AosAttribute::SpellChanneling as u32,
        1,
    );
    set_attr(
        w,
        id,
        AttrKind::Attributes,
        AosAttribute::CastSpeed as u32,
        -1,
    );
    let mage = utility::random_min_max(&mut w.rng, 1, 10);
    set_attr(
        w,
        id,
        AttrKind::Weapon,
        AosWeaponAttribute::MageWeapon as u32,
        mage,
    );
    set_resource(w, id, CraftResource::None);
    id
}

/// `GetUseDelay`: 4 seconds; `IDWand` has none.
pub fn use_delay(w: &World, id: ItemId) -> TimeSpan {
    if w.item(id).is_some_and(|i| i.ty.is(&ID_WAND)) {
        TimeSpan::ZERO
    } else {
        TimeSpan::from_secs(4)
    }
}

/// `ApplyDelayTo(from)`: the wand lock for `GetUseDelay`; `ReleaseWandLock_Callback` is empty.
pub fn apply_delay_to(w: &mut World, id: ItemId, from: MobileId) {
    muo_server::mobile::part1::combat::begin_action(w, from, WAND_ACTION);
    let delay = use_delay(w, id);
    timer::start(w, delay, TimeSpan::ZERO, 1, move |w| {
        muo_server::mobile::part1::combat::end_action(w, from, WAND_ACTION);
    });
}

/// `ConsumeCharge(from)`.
pub fn consume_charge(w: &mut World, id: ItemId, from: MobileId) {
    set_charges(w, id, charges(w, id) - 1);
    if charges(w, id) == 0 {
        deps::send_localized_message(w, from, 1019073); // This item is out of charges.
    }
    apply_delay_to(w, id, from);
}

/// `OnWandTarget(from, o)`: true for every wand but `IDWand`.
fn on_wand_target(w: &mut World, id: ItemId, from: MobileId, o: Targeted) -> bool {
    if !w.item(id).is_some_and(|i| i.ty.is(&ID_WAND)) {
        return true;
    }
    let Targeted::Entity(EntityId::Item(item)) = o else {
        return false;
    };
    if w.item(item).is_none() {
        return false;
    }
    if let Some(d) = weapon::data_mut(w, item) {
        d.identified = true;
    } else if let Some(d) = muo_content::items::armor::data_mut(w, item) {
        d.identified = true;
    }
    if !muo_content::core::aos(w)
        && let Some(ty) = w.item(item).map(|i| i.ty)
    {
        (ty.on_single_click)(w, item, from);
    }
    true
}

/// `DoWandTarget(from, o)`.
pub fn do_wand_target(w: &mut World, id: ItemId, from: MobileId, o: Targeted) {
    let parent_is_from = w.item(id).and_then(|i| i.parent) == Some(EntityId::Mobile(from));
    let ground = matches!(o, Targeted::Land(_) | Targeted::Static(_));
    if ops::is_deleted(w, id) || charges(w, id) <= 0 || !parent_is_from || ground {
        return;
    }
    if on_wand_target(w, id, from, o) {
        consume_charge(w, id, from);
    }
}

/// `OnWandUse(from)` of a wand with no spell: `from.Target = new WandTarget(this)` (`Target(6, false, None)`).
fn begin_wand_target(w: &mut World, id: ItemId, from: MobileId) {
    let mut t = target::Target::new(w, 6, false, TargetFlags::NONE);
    t.handlers.on_target = Some(Rc::new(move |w: &mut World, from, targeted| {
        do_wand_target(w, id, from, targeted)
    }));
    let tid = target::add_target(w, t);
    target::set_mobile_target(w, from, Some(tid));
}

/// `Cast(new XSpell(from, this))`: `Movable = false; spell.Cast(); Movable = m`.
fn cast_wand_spell(w: &mut World, id: ItemId, from: MobileId, spell_name: &str) {
    let movable = w.item(id).is_some_and(|i| i.movable());
    ops::set_movable(w, id, false);
    if let Some(s) = registry::new_spell_by_name(w, spell_name, from, Some(id)) {
        spell::cast(w, s);
    }
    ops::set_movable(w, id, movable);
}

/// `OnDoubleClick(from)`; `on_use` is the class's `OnWandUse`.
fn double_click(
    w: &mut World,
    id: ItemId,
    from: MobileId,
    on_use: fn(&mut World, ItemId, MobileId),
) {
    if !muo_server::mobile::part1::combat::can_begin_action(w, from, WAND_ACTION) {
        deps::send_localized_message(w, from, 1070860); // You must wait a moment for the wand to recharge.
        return;
    }
    if w.item(id).and_then(|i| i.parent) == Some(EntityId::Mobile(from)) {
        if charges(w, id) > 0 {
            on_use(w, id, from);
        } else {
            deps::send_localized_message(w, from, 1019073); // This item is out of charges.
        }
    } else {
        deps::send_localized_message(w, from, 502641); // You must equip this item to use it.
    }
}

fn get_properties(w: &World, id: ItemId, list: &mut PropertyList) {
    (BASE_BASHING.get_properties)(w, id, list);
    if let Some(e) = wand_effect(w, id) {
        list.add_str(e.properties_number(), &charges(w, id).to_string());
    }
}

fn on_single_click(w: &mut World, id: ItemId, from: MobileId) {
    if !core_uotd(w) {
        on_single_click_pre_uotd(w, id, from);
        return;
    }
    let Some(ty) = w.item(id).map(|i| i.ty) else {
        return;
    };
    let mut attrs = Vec::new();
    if (ty.display_loot_type)(w, id) {
        match w.item(id).map(|i| i.loot_type) {
            Some(LootType::Blessed) => attrs.push(EquipInfoAttribute::new(1038021, -1)), // blessed
            Some(LootType::Cursed) => attrs.push(EquipInfoAttribute::new(1049643, -1)),  // cursed
            _ => {}
        }
    }
    let (identified, crafter) =
        weapon::data(w, id).map_or((false, None), |d| (d.identified, d.crafter.clone()));
    if !identified {
        attrs.push(EquipInfoAttribute::new(1038000, -1)); // Unidentified
    } else if let Some(e) = wand_effect(w, id) {
        attrs.push(EquipInfoAttribute::new(
            e.equip_info_number(),
            charges(w, id),
        ));
    }
    let name = ops::name(w, id);
    let number = match &name {
        None => 1017085,
        Some(n) => {
            ops::label_to(w, id, from, n);
            1041000
        }
    };
    if attrs.is_empty() && crafter.is_none() && name.is_some() {
        return;
    }
    let serial = w.item(id).map(|i| i.serial);
    if let (Some(ns), Some(serial)) = (deps::net_state_of(w, from), serial) {
        send_display_equipment_info(w, ns, serial, number, crafter.as_deref(), false, &attrs);
    }
}

/// `OnSingleClickPreUOTD(from)`.
pub fn on_single_click_pre_uotd(w: &mut World, id: ItemId, from: MobileId) {
    let Some(ty) = w.item(id).map(|i| i.ty) else {
        return;
    };
    let is_magic_item = charges(w, id) > 0;
    let name = ops::name(w, id).unwrap_or_else(|| {
        let article_an = ops::item_data(w, id)
            .flags
            .contains(muo_data::tile_data::TileFlag::ARTICLE_AN);
        let text = muo_content::items::equip_hooks::lower_text(w, (ty.label_number)(w, id));
        format!("{} {text}", if article_an { "an" } else { "a" })
    });
    let identified = weapon::data(w, id).is_some_and(|d| d.identified);
    if is_magic_item && !identified {
        ops::label_to(w, id, from, &format!("an unidentified {name}"));
        return;
    }
    let text = match (is_magic_item, wand_effect(w, id)) {
        (true, Some(e)) => format!("{name} of {}", e.text()),
        _ => name,
    };
    ops::label_to(w, id, from, &text);
}

/// `BaseWand`' constants (the same as `MagicWand`).
pub static BASE_WAND_CLASS: WeaponClass = WeaponClass {
    default_weight: Some(1.0),
    primary_ability: Some(Ability::Dismount),
    secondary_ability: Some(Ability::Disarm),
    aos_str_req: 5,
    aos_min_damage: same(9),
    aos_max_damage: same(11),
    aos_speed: 40,
    ml_speed: 2.75,
    old_str_req: 0,
    old_min_damage: 2,
    old_max_damage: 6,
    old_speed: 35,
    init_min_hits: 31,
    init_max_hits: 110,
    ..BASE_BASHING_CLASS
};

/// `Server.Items.BaseWand` (abstract; `[SerializationGenerator(1, false)]`: version int 1, the effect, the charges).
pub static BASE_WAND: ItemType = ItemType {
    full_name: "Server.Items.BaseWand",
    parent: Some(&BASE_BASHING),
    construct: None,
    family: Some(&BASE_WAND_CLASS),
    on_double_click: |w, id, from| double_click(w, id, from, begin_wand_target),
    get_properties,
    on_single_click,
    serialize: |w, id, out| {
        (BASE_BASHING.serialize)(w, id, out);
        out.write_i32(1);
        out.write_enum(
            4,
            i64::from(weapon::data(w, id).map_or(0, |d| d.wand_effect)),
        );
        out.write_i32(charges(w, id));
    },
    deserialize: |w, id, r| {
        (BASE_BASHING.deserialize)(w, id, r)?;
        let version = r.read_i32()?;
        // `Deserialize(reader, version)` (version 0) reads both as plain ints.
        let effect = if version < 1 {
            r.read_i32()?
        } else {
            r.read_enum(4)? as i32
        };
        WandEffect::from_i32(effect)
            .ok_or_else(|| IoError::InvalidData(format!("WandEffect: no value {effect}")))?;
        let charges = r.read_i32()?;
        if let Some(d) = weapon::data_mut(w, id) {
            d.wand_effect = effect;
            d.charges = charges;
        }
        Ok(())
    },
    ..BASE_BASHING
};

/// One concrete wand: the effect and the charge range of its constructor (`max_ml`, `max` are `Core.ML ? a : b`).
macro_rules! wand_leaves {
    ($($(#[$m:meta])* $name:ident, $new:ident = $full:literal, $spell:expr;)*) => {
        $(
            /// `new T()`: `base(effect, minCharges, maxCharges)` from the class's props.
            pub fn $new(w: &mut World, ty: &'static ItemType) -> ItemId {
                new_wand(w, ty)
            }

            $(#[$m])*
            pub static $name: ItemType = ItemType {
                full_name: $full,
                parent: Some(&BASE_WAND),
                construct: Some(|w| $new(w, &$name)),
                on_double_click: |w, id, from| {
                    fn on_use(w: &mut World, id: ItemId, from: MobileId) {
                        let spell: Option<&'static str> = $spell;
                        match spell {
                            Some(n) => cast_wand_spell(w, id, from, n),
                            None => begin_wand_target(w, id, from),
                        }
                    }
                    double_click(w, id, from, on_use)
                },
                serialize: |w, id, out| {
                    (BASE_WAND.serialize)(w, id, out);
                    out.write_i32(0); // [SerializationGenerator(0, false)]
                },
                deserialize: |w, id, r| {
                    (BASE_WAND.deserialize)(w, id, r)?;
                    let _version = r.read_i32()?;
                    Ok(())
                },
                ..BASE_WAND
            };
        )*

        /// Every wand class.
        pub static LEAVES: &[&ItemType] = &[$(&$name),*];
    };
}


wand_leaves! {
    /// `Server.Items.ClumsyWand`.
    CLUMSY_WAND, clumsy_wand_new = "Server.Items.ClumsyWand", Some("Server.Spells.First.ClumsySpell");
    /// `Server.Items.FeebleWand`.
    FEEBLE_WAND, feeble_wand_new = "Server.Items.FeebleWand", Some("Server.Spells.First.FeeblemindSpell");
    /// `Server.Items.FireballWand`.
    FIREBALL_WAND, fireball_wand_new = "Server.Items.FireballWand", Some("Server.Spells.Third.FireballSpell");
    /// `Server.Items.GreaterHealWand`.
    GREATER_HEAL_WAND, greater_heal_wand_new = "Server.Items.GreaterHealWand", Some("Server.Spells.Fourth.GreaterHealSpell");
    /// `Server.Items.HarmWand`.
    HARM_WAND, harm_wand_new = "Server.Items.HarmWand", Some("Server.Spells.Second.HarmSpell");
    /// `Server.Items.HealWand`.
    HEAL_WAND, heal_wand_new = "Server.Items.HealWand", Some("Server.Spells.First.HealSpell");
    /// `Server.Items.IDWand` (`GetUseDelay => TimeSpan.Zero`, `OnWandTarget` identifies; no spell).
    ID_WAND, id_wand_new = "Server.Items.IDWand", None;
    /// `Server.Items.LightningWand`.
    LIGHTNING_WAND, lightning_wand_new = "Server.Items.LightningWand", Some("Server.Spells.Fourth.LightningSpell");
    /// `Server.Items.MagicArrowWand`.
    MAGIC_ARROW_WAND, magic_arrow_wand_new = "Server.Items.MagicArrowWand", Some("Server.Spells.First.MagicArrowSpell");
    /// `Server.Items.ManaDrainWand`.
    MANA_DRAIN_WAND, mana_drain_wand_new = "Server.Items.ManaDrainWand", Some("Server.Spells.Fourth.ManaDrainSpell");
    /// `Server.Items.WeaknessWand`.
    WEAKNESS_WAND, weakness_wand_new = "Server.Items.WeaknessWand", Some("Server.Spells.First.WeakenSpell");
}

/// Sets the `SpellHooks` the spell base calls on a wand.
fn configure(w: &mut World) {
    let h = w.resources.get_mut::<SpellHooks>();
    h.wand_consume_charge = Some(consume_charge);
    h.wand_charges = Some(charges);
}

/// Registers the classes of this file.
pub fn register(types: &mut TypeRegistry) {
    for t in LEAVES {
        types.register_item(t, &[]);
    }
    types.register_phase_default(
        Phase::Configure,
        "Server.Items.BaseWand.Configure",
        configure,
    );
}
