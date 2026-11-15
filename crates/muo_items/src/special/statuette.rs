//! `MonsterStatuette`, `HeritageToken`, `BaseImprisonedMobile`, `GrizzledMareStatuette` and the
//! `TurnOnGump`/`TurnOffGump` of the gump toggle items.
//!
//! The gumps `HeritageTokenGump` and `ConfirmBreakCrystalGump` live in muo_gumps and are reached through
//! [`SpecialHooks`](super::hooks::SpecialHooks); `BaseImprisonedMobile.Summon`/`Release` are called by that gump through
//! [`summon`] and [`release`].

use muo_content::gumps::base::{Gump, RelayInfo, StaticGumpLogic, StaticGumpObj};
use muo_content::gumps::builder::StaticGumpBuilder;
use muo_content::gumps::send_gump_to_mobile;
use muo_content::props::{Ctor, Kind, Param, Value, add_ctors};
use muo_content::utilities::types::PropType;
use muo_primitives::{Point3D, utility};
use muo_server::World;
use muo_server::effects;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, LootType, ops};
use muo_server::mobile::AccessLevel;
use muo_server::net_state::NetStateId;
use muo_server::type_registry::TypeRegistry;

use super::hooks::hooks;
use super::util::{access_level, in_pack, msg, reward_year_label};

// ---- the toggle gumps ----

/// `IGumpToggleItem`: how a toggle gump reads and writes its item.
#[derive(Clone, Copy)]
pub struct Toggle {
    pub turned_on: fn(&World, ItemId) -> bool,
    pub set_turned_on: fn(&mut World, ItemId, bool),
}

/// `TurnOnGump` (`on`) or `TurnOffGump`.
pub struct TurnOnOffGump<const ON: bool> {
    item: ItemId,
    toggle: Toggle,
}

/// `from.SendGump(new TurnOnGump(item))` / `TurnOffGump` by the item's current state.
pub fn send_toggle_gump(w: &mut World, from: MobileId, item: ItemId, toggle: Toggle) {
    let gump: Box<dyn Gump> = if (toggle.turned_on)(w, item) {
        Box::new(StaticGumpObj::new(
            150,
            200,
            TurnOnOffGump::<false> { item, toggle },
        ))
    } else {
        Box::new(StaticGumpObj::new(
            150,
            200,
            TurnOnOffGump::<true> { item, toggle },
        ))
    };
    send_gump_to_mobile(w, from, gump, false);
}

impl<const ON: bool> StaticGumpLogic for TurnOnOffGump<ON> {
    const TYPE_NAME: &'static str = if ON {
        "Server.Items.TurnOnGump"
    } else {
        "Server.Items.TurnOffGump"
    };

    fn build_layout(&self, b: &mut StaticGumpBuilder) {
        b.add_background(0, 0, 300, 150, 0xA28);
        b.add_button(40, 53, 0xFA5, 0xFA7, 1);
        b.add_html_localized(80, 55, 65, 35, 1011036, false, false); // OKAY
        b.add_button(150, 53, 0xFA5, 0xFA7, 0);
        b.add_html_localized(190, 55, 100, 35, 1011012, false, false); // CANCEL
        // Activate this item / Deactivate this item
        b.add_html_localized(
            45,
            20,
            300,
            35,
            if ON { 1011034 } else { 1011035 },
            false,
            false,
        );
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let Some(from) = w.net_states.get(&sender).and_then(|n| n.mobile) else {
            return;
        };
        if info.button_id != 1 {
            msg(w, from, 502694); // Cancelled action.
            return;
        }
        let new_value = !(self.toggle.turned_on)(w, self.item);
        (self.toggle.set_turned_on)(w, self.item, new_value);
        if new_value && !ops::is_locked_down(w, self.item) {
            msg(w, from, 502693); // Remember, this only works when locked down.
        }
    }
}

// ---- MonsterStatuette ----

/// `MonsterStatuetteType` numbers.
pub mod kind {
    pub const CROCODILE: i32 = 0;
    pub const SLIME: i32 = 27;
    pub const RED_DEATH: i32 = 29;
    pub const HALLOWEEN_GHOUL: i32 = 38;
}

/// `MonsterStatuetteInfo`: label, item id, sounds.
struct Info {
    label: i32,
    item_id: i32,
    sounds: &'static [i32],
}

macro_rules! info_base {
    ($label:expr, $id:expr, $s:expr) => {
        Info {
            label: $label,
            item_id: $id,
            sounds: &[$s, $s + 1, $s + 2, $s + 3, $s + 4],
        }
    };
}

/// `MonsterStatuetteInfo._table`, in enum order.
static TABLE: [Info; 40] = [
    info_base!(1041249, 0x20DA, 660),
    info_base!(1041250, 0x20D3, 357),
    info_base!(1041251, 0x20D6, 362),
    info_base!(1041252, 0x20D7, 268),
    info_base!(1041253, 0x20D8, 367),
    info_base!(1041254, 0x20D9, 372),
    info_base!(1041255, 0x20F5, 158),
    info_base!(1041256, 0x20F8, 1001),
    info_base!(1041257, 0x20DE, 417),
    info_base!(1041258, 0x20DF, 427),
    info_base!(1041259, 0x20E0, 1114),
    info_base!(1041260, 0x20E3, 437),
    info_base!(1041261, 0x20E7, 1165),
    info_base!(1041262, 0x20E9, 461),
    info_base!(1041263, 0x2103, 120),
    info_base!(1041264, 0x20EC, 471),
    info_base!(1041265, 0x20F6, 1011),
    info_base!(1049742, 0x2133, 634),
    info_base!(1049743, 0x20FA, 442),
    info_base!(1049744, 0x20F9, 422),
    info_base!(1049768, 0x20F4, 377),
    info_base!(1049769, 0x20F3, 838),
    info_base!(1049770, 0x2122, 229),
    info_base!(1063488, 0x3FFE, 168),
    info_base!(1070819, 0x25BA, 138),
    Info {
        label: 1080520,
        item_id: 0x25BB,
        sounds: &[0x289, 0x28A, 0x28B],
    },
    info_base!(1080521, 0x2590, 0x300),
    info_base!(1015246, 0x20E8, 456),
    info_base!(1029747, 0x2613, 0x1BF),
    Info {
        label: 1094932,
        item_id: 0x2617,
        sounds: &[],
    },
    info_base!(1029668, 0x25C4, 1170),
    info_base!(1029641, 0x25A9, 639),
    info_base!(1029645, 0x25AD, 634),
    info_base!(1029642, 0x25aa, 634),
    info_base!(1029643, 0x25ab, 639),
    info_base!(1031651, 0x2D83, 0xA8),
    info_base!(1031657, 0x2D89, 0x596),
    info_base!(1096928, 0x4688, 0x69),
    info_base!(1076782, 0x2109, 0x482),
    info_base!(1097968, 0x4A98, 0x669),
];

/// `MonsterStatuetteInfo.GetInfo(type)`: an unknown type is the first row.
fn get_info(kind: i32) -> &'static Info {
    TABLE
        .get(usize::try_from(kind).unwrap_or(usize::MAX))
        .unwrap_or(&TABLE[0])
}

/// The fields of `MonsterStatuette`.
#[derive(Clone, Copy, Debug, Default)]
pub struct StatuetteData {
    pub kind: i32,
    pub turned_on: bool,
    pub is_reward_item: bool,
}

fn sd(w: &World, id: ItemId) -> StatuetteData {
    w.item(id)
        .and_then(|i| i.ext.get::<StatuetteData>())
        .copied()
        .unwrap_or_default()
}

fn sd_mut(w: &mut World, id: ItemId) -> Option<&mut StatuetteData> {
    w.item_mut(id)?.ext.get_mut::<StatuetteData>()
}

/// `GetStatuetteHue(type, fallback)`.
pub fn get_statuette_hue(w: &mut World, kind: i32, fallback: i32) -> i32 {
    match kind {
        kind::SLIME => utility::random_from(&mut w.rng, 2201, 24),
        kind::RED_DEATH => 0x21,
        kind::HALLOWEEN_GHOUL => 0xF4,
        _ => fallback,
    }
}

/// `MonsterStatuette(type)`.
pub fn monster_statuette_new(w: &mut World, kind: i32) -> ItemId {
    monster_statuette_new_ty(w, &MONSTER_STATUETTE, kind)
}

/// `MonsterStatuette(type)` for the dynamic class `ty` (a derived class passes its own).
pub fn monster_statuette_new_ty(w: &mut World, ty: &'static ItemType, kind: i32) -> ItemId {
    let id = ops::new_item_with_id(w, ty, get_info(kind).item_id);
    ops::set_loot_type(w, id, LootType::Blessed);
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(StatuetteData {
            kind,
            turned_on: false,
            is_reward_item: false,
        });
    }
    let hue = get_statuette_hue(w, kind, 0);
    super::hue(w, id, hue);
    id
}

/// `Type = value`.
pub fn set_type(w: &mut World, id: ItemId, kind: i32) {
    if let Some(d) = sd_mut(w, id) {
        d.kind = kind;
    }
    let Some(ty) = w.item(id).map(|i| i.ty) else {
        return;
    };
    (ty.set_item_id)(w, id, get_info(kind).item_id);
    let cur = w.item(id).map_or(0, |i| i.hue);
    let hue = get_statuette_hue(w, kind, cur);
    super::hue(w, id, hue);
    ops::invalidate_properties(w, id);
}

pub fn turned_on(w: &World, id: ItemId) -> bool {
    sd(w, id).turned_on
}

pub fn set_turned_on(w: &mut World, id: ItemId, v: bool) {
    if let Some(d) = sd_mut(w, id)
        && d.turned_on != v
    {
        d.turned_on = v;
        ops::invalidate_properties(w, id);
    }
}

/// `IsRewardItem = value`.
pub fn set_is_reward_item(w: &mut World, id: ItemId, v: bool) {
    if let Some(d) = sd_mut(w, id) {
        d.is_reward_item = v;
    }
    ops::invalidate_properties(w, id);
}

const TOGGLE: Toggle = Toggle {
    turned_on,
    set_turned_on,
};

fn has_access(w: &World, id: ItemId, mob: MobileId) -> bool {
    access_level(w, mob) >= AccessLevel::GameMaster
        || muo_multis::house::access::find_house_at_item(w, id)
            .is_some_and(|h| muo_multis::house::access::is_owner(w, h, mob))
}

/// `Server.Items.MonsterStatuette`.
pub static MONSTER_STATUETTE: ItemType = ItemType {
    full_name: "Server.Items.MonsterStatuette",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| monster_statuette_new(w, kind::CROCODILE)),
    label_number: |w, id| get_info(sd(w, id).kind).label,
    default_weight: crate::class_props::default_weight,
    handles_on_movement: |w, id| turned_on(w, id) && ops::is_locked_down(w, id),
    on_movement: |w, id, m, old| {
        let loc = w.item(id).map_or_else(Point3D::default, |i| i.location);
        let hidden = w.mobile(m).is_some_and(|x| x.hidden);
        let mloc = w.mobile(m).map_or_else(Point3D::default, |x| x.location);
        if turned_on(w, id)
            && ops::is_locked_down(w, id)
            && (!hidden || access_level(w, m) == AccessLevel::Player)
            && utility::in_range(mloc.x, mloc.y, loc.x, loc.y, 2)
            && !utility::in_range(old.x, old.y, loc.x, loc.y, 2)
        {
            let sounds = get_info(sd(w, id).kind).sounds;
            if let Some(&s) = utility::random_element(&mut w.rng, sounds) {
                let map = w.item(id).and_then(|i| i.map);
                effects::play_sound(w, loc, map, s);
            }
        }
        (ITEM_BASE.on_movement)(w, id, m, old);
    },
    get_properties: |w, id, list| {
        (ITEM_BASE.get_properties)(w, id, list);
        if muo_server::item::deps::core_ml(w)
            && sd(w, id).is_reward_item
            && let Some(n) = reward_year_label(w, id)
        {
            list.add(n); // X Year Veteran Reward
        }
        list.add(if sd(w, id).turned_on { 502695 } else { 502696 }); // turned on / turned off
    },
    on_double_click: |w, id, from| {
        if !has_access(w, id, from) {
            msg(w, from, 502691); // You must be the owner to use this.
        } else {
            send_toggle_gump(w, from, id, TOGGLE);
        }
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(0);
        let d = sd(w, id);
        out.write_enum(4, i64::from(d.kind));
        out.write_bool(d.turned_on);
        out.write_bool(d.is_reward_item);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        let kind = r.read_enum(4)? as i32;
        let turned_on = r.read_bool()?;
        let is_reward_item = r.read_bool()?;
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(StatuetteData {
                kind,
                turned_on,
                is_reward_item,
            });
        }
        Ok(())
    },
    ..ITEM_BASE
};

// ---- HeritageToken ----

/// `Server.Items.HeritageToken`.
pub static HERITAGE_TOKEN: ItemType = ItemType {
    full_name: "Server.Items.HeritageToken",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| crate::simple::from_template(w, &HERITAGE_TOKEN, "Server.Items.HeritageToken")),
    default_weight: crate::class_props::default_weight,
    label_number: crate::class_props::label_number, // A Heritage Token
    on_double_click: |w, id, from| {
        if in_pack(w, id, from) {
            if let Some(f) = hooks(w).and_then(|h| h.heritage_token_gump) {
                f(w, from, id);
            }
        } else {
            msg(w, from, 1062334); // This item must be in your backpack to be used.
        }
    },
    get_properties: |w, id, list| {
        (ITEM_BASE.get_properties)(w, id, list);
        list.add_localized_number(1070998, 1076595); // Use this to redeem<br>Your Heritage Items
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_encoded_int(0);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = r.read_encoded_int()?;
        Ok(())
    },
    ..ITEM_BASE
};

// ---- BaseImprisonedMobile, GrizzledMareStatuette ----

/// What a `BaseImprisonedMobile` subclass overrides: the class of the creature `Summon` makes.
pub struct ImprisonedClass {
    pub summon: &'static str,
}

/// `BaseImprisonedMobile.OnDoubleClick`.
fn imprisoned_double_click(w: &mut World, id: ItemId, from: MobileId) {
    if in_pack(w, id, from) {
        if let Some(f) = hooks(w).and_then(|h| h.break_crystal_gump) {
            f(w, from, id);
        }
    } else {
        msg(w, from, 1042001); // That must be in your pack for you to use it.
    }
}

/// `BaseImprisonedMobile.Summon`: a new creature of the statuette's class.
pub fn summon(w: &mut World, id: ItemId) -> Option<MobileId> {
    let class = w.item(id)?.ty.family::<ImprisonedClass>()?;
    muo_content::factory::make_mobile(w, class.summon)
}

/// `BaseImprisonedMobile.Release(from, summon)`: nothing in the base class.
pub fn release(_w: &mut World, _id: ItemId, _from: MobileId, _summon: MobileId) {}

static GRIZZLED_MARE_CLASS: ImprisonedClass = ImprisonedClass {
    summon: "Server.Mobiles.GrizzledMare",
};

/// `Server.Items.BaseImprisonedMobile` (abstract).
pub static BASE_IMPRISONED_MOBILE: ItemType = ItemType {
    full_name: "Server.Items.BaseImprisonedMobile",
    parent: Some(&ITEM_BASE),
    construct: None,
    on_double_click: imprisoned_double_click,
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        Ok(())
    },
    ..ITEM_BASE
};

/// `Server.Items.GrizzledMareStatuette`.
pub static GRIZZLED_MARE_STATUETTE: ItemType = ItemType {
    full_name: "Server.Items.GrizzledMareStatuette",
    parent: Some(&BASE_IMPRISONED_MOBILE),
    construct: Some(|w| crate::simple::from_template(w, &GRIZZLED_MARE_STATUETTE, "Server.Items.GrizzledMareStatuette")),
    family: Some(&GRIZZLED_MARE_CLASS),
    default_weight: crate::class_props::default_weight,
    label_number: crate::class_props::label_number, // Grizzled Mare Statuette
    serialize: |w, id, out| {
        (BASE_IMPRISONED_MOBILE.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (BASE_IMPRISONED_MOBILE.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        Ok(())
    },
    ..BASE_IMPRISONED_MOBILE
};

static TYPE_NAMES: [(&str, i64); 40] = [
    ("Crocodile", 0),
    ("Daemon", 1),
    ("Dragon", 2),
    ("EarthElemental", 3),
    ("Ettin", 4),
    ("Gargoyle", 5),
    ("Gorilla", 6),
    ("Lich", 7),
    ("Lizardman", 8),
    ("Ogre", 9),
    ("Orc", 10),
    ("Ratman", 11),
    ("Skeleton", 12),
    ("Troll", 13),
    ("Cow", 14),
    ("Zombie", 15),
    ("Llama", 16),
    ("Ophidian", 17),
    ("Reaper", 18),
    ("Mongbat", 19),
    ("Gazer", 20),
    ("FireElemental", 21),
    ("Wolf", 22),
    ("PhillipsWoodenSteed", 23),
    ("Seahorse", 24),
    ("Harrower", 25),
    ("Efreet", 26),
    ("Slime", 27),
    ("PlagueBeast", 28),
    ("RedDeath", 29),
    ("Spider", 30),
    ("OphidianArchMage", 31),
    ("OphidianWarrior", 32),
    ("OphidianKnight", 33),
    ("OphidianMage", 34),
    ("DreadHorn", 35),
    ("Minotaur", 36),
    ("BlackCat", 37),
    ("HalloweenGhoul", 38),
    ("Santa", 39),
];

static MONSTER_STATUETTE_CTORS: [Ctor; 1] = [Ctor {
    access: None,
    params: &[Param {
        name: "type",
        kind: Kind::Plain(PropType::Enum(&TYPE_NAMES)),
        default: Some(Value::Enum(0)),
        ty: "MonsterStatuetteType",
    }],
    make: |w, v| EntityId::Item(monster_statuette_new(w, v[0].i32())),
}];

/// Configure phase: the `[Constructible]` overloads.
pub(super) fn configure(w: &mut World) {
    add_ctors(w, "Server.Items.MonsterStatuette", &MONSTER_STATUETTE_CTORS);
}

pub fn register(types: &mut TypeRegistry) {
    for t in [
        &MONSTER_STATUETTE,
        &HERITAGE_TOKEN,
        &BASE_IMPRISONED_MOBILE,
        &GRIZZLED_MARE_STATUETTE,
    ] {
        types.register_item(t, &[]);
    }
}
