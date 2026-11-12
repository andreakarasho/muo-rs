//! `BaseVendor` (the class, its serializer, the outfit and body defaults, `LoadSBInfo`,
//! restock) with GenericBuy/GenericSell/VendorBuybackPack in the sibling modules and the `Banker` statics.
//!
//! A concrete vendor is a `MobileType` under [`BASE_VENDOR`] whose `CreatureType` has `subclass:
//! Some(&VendorClass)`; [`class`] reads it. The vendor's own state is [`VendorData`] in `Mobile::ext` next to
//! `CreatureData`. `SBInfos` of a class is `VendorClass::init_sb_info`, called on construction, on load and on a
//! map change, as `LoadSBInfo` does (so a random choice in it is made again each time).
//!
//! Systems this does not own are `fn` fields of [`VendorHooks`] in `w.resources`: bulk orders (`SupportsBulkOrders`
//! etc. are class fields, the gumps are hooks), factions' `Town.Tax`, the item classes that take a constructor
//! argument (`BroadcastCrystal(500)`, `Pitcher(BeverageType)`) and the preset maps. An unset hook behaves as
//! if the system were absent.

pub mod banker;
pub mod buy;
pub mod packs;
pub mod trade;

pub use buy::{BuyInfo, BuyKind, SbInfo, SbInstance, SellInfo};
pub use packs::{DISPLAY_CACHE, VENDOR_BUYBACK_PACK};

use muo_primitives::{DateTime, TimeSpan, utility};
use muo_server::ids::{ItemId, MobileId};
use muo_server::layer::Layer;
use muo_server::mobile::MobileType;
use muo_server::mobile::part1::{props_a, props_b};
use muo_server::mobile::part2::{items, stats};
use muo_server::race::{RaceOps, Races};
use muo_server::timer::{self, TimerHandle};
use muo_server::world::World;

use crate::creature::enums::{AIType, FightMode};
use crate::creature::setters::{new_creature, pack_gold_range};
use crate::creature::{BASE_CREATURE, BASE_CREATURE_C, CreatureType, ctype, speeds};
use crate::factory;
use crate::hooks::rnd;
use crate::player_mobile::NpcGuild;

/// `VendorShoeType`.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum ShoeType {
    None,
    #[default]
    Shoes,
    Boots,
    Sandals,
    ThighBoots,
}

/// `BuyListLimit`: one (not all) of the buy packets counts entries in a byte.
pub const BUY_LIST_LIMIT: usize = 250;
/// `MaxSell`.
pub const MAX_SELL: i32 = 500;

/// What `BaseVendor` adds to a `BaseCreature`.
pub struct VendorData {
    pub last_restock: DateTime,
    pub next_trick_or_treat: DateTime,
    /// `_buyInfo`, the entries of every `SBInfo` in order.
    pub buy_info: Vec<BuyInfo>,
    /// `_sellInfo`, one per `SBInfo`.
    pub sell_info: Vec<SellInfo>,
    /// `_buybackPurgeToken`.
    pub buyback_purge: TimerHandle,
}

impl VendorData {
    pub fn new(now: DateTime) -> VendorData {
        VendorData {
            last_restock: now,
            next_trick_or_treat: DateTime::MIN,
            buy_info: Vec::new(),
            sell_info: Vec::new(),
            buyback_purge: TimerHandle::default(),
        }
    }
}

pub fn data(w: &World, id: MobileId) -> Option<&VendorData> {
    w.mobile(id)?.ext.get::<VendorData>()
}

pub fn data_mut(w: &mut World, id: MobileId) -> Option<&mut VendorData> {
    w.mobile_mut(id)?.ext.get_mut::<VendorData>()
}

/// `m is BaseVendor`.
pub fn is_vendor(w: &World, id: MobileId) -> bool {
    data(w, id).is_some()
}

/// The `BaseVendor` statics: `Configure` reads two settings.
#[derive(Default)]
pub struct VendorStatics {
    pub enable_vendor_buy_opl: bool,
    pub vendor_invulnerable: bool,
}

/// `ContentFeatureFlags` the vendors read.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Feature {
    VendorPurchase,
    VendorSell,
    BulkOrders,
}

type ConstructWithArgFn = fn(&mut World, &str, i32) -> Option<ItemId>;
type CreateSmallBodFn = fn(&mut World, i32, MobileId) -> Option<ItemId>;
type BodTurnInFn = fn(&mut World, MobileId, MobileId, ItemId) -> Option<bool>;
/// Calls into systems that are not part of the vendor code. Unset: the system is absent.
#[derive(Default)]
pub struct VendorHooks {
    /// `ContentFeatureFlags.X`. Unset: enabled.
    pub feature_enabled: Option<fn(&World, Feature) -> bool>,
    /// `Town.FromRegion(Region)?.Tax` of the vendor. Unset: no town (the scalar is then 0, see
    /// [`get_price_scalar`]).
    pub town_tax: Option<fn(&World, MobileId) -> Option<i32>>,
    /// `Type.CreateInstance<IEntity>(arg)` for a class with a one-integer constructor (`BroadcastCrystal(charges)`,
    /// `Pitcher(BeverageType)`). Unset: the parameterless constructor.
    pub construct_with_arg: Option<ConstructWithArgFn>,
    /// `new PresetMap(PresetMapEntry.Table[index])`.
    pub make_preset_map: Option<fn(&mut World, usize) -> Option<ItemId>>,
    /// `PresetMapEntry.Table[i].Name` (a cliloc) for every entry.
    pub preset_map_names: Option<fn(&World) -> Vec<i32>>,
    /// For a `BaseBeverage` item: `Some(IsEmpty || Content == Milk)`; `None` for any other item.
    pub beverage_is_empty_or_milk: Option<fn(&World, ItemId) -> Option<bool>>,
    /// The tinker guildmaster's `RechargeEntry` (6271, 6) of the weapon engraving tool (Core.ML).
    pub recharge_entry: Option<
        fn(&mut World, MobileId, MobileId) -> muo_server::context_menu_entry::ContextMenuEntry,
    >,
    /// `m.SendGump(new ResurrectGump(healer, price), true)` when `price > 0`, else
    /// `new ResurrectGump(m, ResurrectMessage.Healer)`.
    pub resurrect_gump: Option<fn(&mut World, MobileId, MobileId, i32)>,
    /// `SmallSmithBOD.CreateRandomFor(from)` (kind 0) / `SmallTailorBOD.CreateRandomFor(from)` (kind 1).
    pub create_small_bod: Option<CreateSmallBodFn>,
    /// `ContentFeatureFlags.BulkOrders` accept gump: `new LargeBODAcceptGump(bod)` / `SmallBODAcceptGump`.
    pub send_bod_accept_gump: Option<fn(&mut World, MobileId, ItemId)>,
    /// The bulk order half of `BaseVendor.OnDragDrop(from, dropped)`: `(vendor, from, dropped)`; `None` when
    /// `dropped` is not a deed (the base drop applies), else the return value. Unset: the base drop.
    pub bod_turn_in: Option<BodTurnInFn>,
}

pub fn hooks(w: &World) -> Option<&VendorHooks> {
    w.resources.get::<VendorHooks>()
}

pub fn feature(w: &World, f: Feature) -> bool {
    hooks(w)
        .and_then(|h| h.feature_enabled)
        .is_none_or(|g| g(w, f))
}

/// The virtuals of `BaseVendor` (and the ones its subclasses override), inherited with `..PARENT_CLASS`.
pub struct VendorClass {
    /// `InitSBInfo()` after `SBInfos.Clear()`: the `SBInfo` instances in order.
    pub init_sb_info: fn(&mut World, MobileId) -> Vec<SbInstance>,
    /// `IsActiveVendor`.
    pub is_active_vendor: fn(&World, MobileId) -> bool,
    /// `IsActiveBuyer` (response to vendor SELL); defaults to `IsActiveVendor`.
    pub is_active_buyer: fn(&World, MobileId) -> bool,
    /// `IsActiveSeller` (response to vendor BUY); defaults to `IsActiveVendor`.
    pub is_active_seller: fn(&World, MobileId) -> bool,
    pub npc_guild: fn(&World, MobileId) -> NpcGuild,
    pub shoe_type: fn(&mut World, MobileId) -> ShoeType,
    pub restock_delay: fn(&World, MobileId) -> TimeSpan,
    pub get_gender: fn(&mut World, MobileId) -> bool,
    pub get_random_hue: fn(&mut World, MobileId) -> i32,
    pub get_shoe_hue: fn(&mut World, MobileId) -> i32,
    pub get_hair_hue: fn(&mut World, MobileId) -> i32,
    pub init_body: fn(&mut World, MobileId),
    pub init_outfit: fn(&mut World, MobileId),
    pub is_valid_bulk_order: fn(&World, MobileId, ItemId) -> bool,
    pub create_bulk_order: fn(&mut World, MobileId, MobileId, bool) -> Option<ItemId>,
    pub supports_bulk_orders: fn(&World, MobileId, MobileId) -> bool,
    pub get_next_bulk_order: fn(&mut World, MobileId, MobileId) -> TimeSpan,
    pub on_successful_bulk_order_receive: fn(&mut World, MobileId, MobileId),
    /// `VendorBuy(from)`.
    pub vendor_buy: fn(&mut World, MobileId, MobileId),
    /// `VendorSell(from)`.
    pub vendor_sell: fn(&mut World, MobileId, MobileId),
    /// `OnBuyItems(buyer, list)`.
    pub on_buy_items:
        fn(&mut World, MobileId, MobileId, &[muo_server::vendor::BuyItemResponse]) -> bool,
    /// `OnSellItems(seller, list)`.
    pub on_sell_items:
        fn(&mut World, MobileId, MobileId, &[muo_server::vendor::SellItemResponse]) -> bool,
    /// `CheckVendorAccess(from)`.
    pub check_vendor_access: fn(&World, MobileId, MobileId) -> bool,
    /// `GetPriceScalar()`.
    pub get_price_scalar: fn(&World, MobileId) -> i32,
    /// `TurnToTokuno()`: renames a vendor `CheckTokuno` finds is not named for Tokuno.
    pub turn_to_tokuno: fn(&mut World, MobileId),
    /// `CheckMorph()`.
    pub check_morph: fn(&mut World, MobileId),
    /// The class table of the first subclass of `BaseVendor` that has one (`BaseHealer`, `BaseGuildmaster`...),
    /// inherited with `..PARENT_CLASS`; read through [`subclass`].
    pub subclass: Option<&'static (dyn std::any::Any + Send + Sync)>,
}

/// The `VendorClass::subclass` table of type `T` of mobile `id`.
pub fn subclass<T: std::any::Any>(w: &World, id: MobileId) -> Option<&'static T> {
    class(w, id)?.subclass?.downcast_ref()
}

/// The `VendorClass` of mobile `id`.
pub fn class(w: &World, id: MobileId) -> Option<&'static VendorClass> {
    ctype(w, id)?.subclass?.downcast_ref()
}

fn cls(w: &World, id: MobileId) -> &'static VendorClass {
    class(w, id).unwrap_or(&BASE_VENDOR_CLASS)
}

pub static BASE_VENDOR_CLASS: VendorClass = VendorClass {
    init_sb_info: |_, _| Vec::new(),
    is_active_vendor: |_, _| true,
    is_active_buyer: |w, id| (cls(w, id).is_active_vendor)(w, id),
    is_active_seller: |w, id| (cls(w, id).is_active_vendor)(w, id),
    npc_guild: |_, _| NpcGuild::None,
    shoe_type: |_, _| ShoeType::Shoes,
    restock_delay: |_, _| TimeSpan::from_hours(1),
    get_gender: |w, _| rnd::bool(w),
    get_random_hue,
    get_shoe_hue,
    get_hair_hue,
    init_body,
    init_outfit,
    is_valid_bulk_order: |_, _, _| false,
    create_bulk_order: |_, _, _, _| None,
    supports_bulk_orders: |_, _, _| false,
    get_next_bulk_order: |_, _, _| TimeSpan::ZERO,
    on_successful_bulk_order_receive: |_, _, _| {},
    vendor_buy: trade::vendor_buy,
    vendor_sell: trade::vendor_sell,
    on_buy_items: trade::on_buy_items,
    on_sell_items: trade::on_sell_items,
    check_vendor_access,
    get_price_scalar,
    turn_to_tokuno,
    check_morph: check_morph_base,
    subclass: None,
};

/// `IsActiveVendor`, `IsActiveBuyer`, `IsActiveSeller`.
pub fn is_active_vendor(w: &World, id: MobileId) -> bool {
    (cls(w, id).is_active_vendor)(w, id)
}
pub fn is_active_buyer(w: &World, id: MobileId) -> bool {
    (cls(w, id).is_active_buyer)(w, id)
}
pub fn is_active_seller(w: &World, id: MobileId) -> bool {
    (cls(w, id).is_active_seller)(w, id)
}

fn statics(w: &World) -> Option<&VendorStatics> {
    w.resources.get::<VendorStatics>()
}

/// `BaseVendor`'s `CreatureType`.
pub static BASE_VENDOR_C: CreatureType = CreatureType {
    can_teach: |_, _| true,
    bard_immune: |_, _| true,
    player_range_sensitive: |_, _| true,
    is_invulnerable: |w, _| statics(w).is_some_and(|s| s.vendor_invulnerable),
    add_custom_context_entries: trade::add_custom_context_entries,
    subclass: Some(&BASE_VENDOR_CLASS),
    ..BASE_CREATURE_C
};

/// `Server.Mobiles.BaseVendor` (abstract).
pub static BASE_VENDOR: MobileType = MobileType {
    full_name: "Server.Mobiles.BaseVendor",
    parent: Some(&BASE_CREATURE),
    construct: None,
    family: Some(&BASE_VENDOR_C),
    show_fame_title: |_, _| false,
    on_after_spawn: |w, id| check_morph(w, id),
    on_map_change,
    on_delete,
    on_drag_drop: |w, id, from, dropped| {
        // BaseVendor.OnDragDrop: a bulk order deed is turned in; anything else is the creature's drop.
        let turn_in = hooks(w).and_then(|h| h.bod_turn_in);
        match turn_in.and_then(|f| f(w, id, from, dropped)) {
            Some(r) => r,
            None => (BASE_CREATURE.on_drag_drop)(w, id, from, dropped),
        }
    },
    serialize: |w, id, out| {
        (BASE_CREATURE.serialize)(w, id, out);
        out.write_i32(2);
    },
    deserialize: |w, id, r| {
        (BASE_CREATURE.deserialize)(w, id, r)?;
        let tick = w.now;
        if let Some(m) = w.mobile_mut(id)
            && m.ext.get::<VendorData>().is_none()
        {
            m.ext.insert(VendorData::new(tick));
        }
        let version = r.read_i32()?;
        if version < 2 {
            // Version 1 persisted which buy entries had grown restock amounts; restock is transient now.
            while r.read_encoded_int()? > 0 {
                r.read_encoded_int()?;
            }
        }
        after_deserialization(w, id);
        Ok(())
    },
    ..BASE_CREATURE
};

/// `BaseVendor.Configure`.
pub fn configure(w: &mut World) {
    let opl = w.config.get_setting("opl.enableForVendorBuy", true);
    let lbr = w.expansion() >= muo_data::expansion::Expansion::LBR;
    let inv = w.config.get_setting("vendor.isInvulnerable", lbr);
    let s = w.resources.get_mut::<VendorStatics>();
    s.enable_vendor_buy_opl = opl;
    s.vendor_invulnerable = inv;
}

/// `BaseVendor.Initialize`: every vendor in the world checks its morph, and the vendor packets reach
/// `OnBuyItems`/`OnSellItems`.
pub fn initialize(w: &mut World) {
    let vendors: Vec<MobileId> = w
        .mobiles
        .iter()
        .filter(|(_, m)| m.ext.get::<VendorData>().is_some())
        .map(|(&id, _)| id)
        .collect();
    for id in vendors {
        check_morph(w, id);
    }
    let h = w
        .resources
        .get_mut::<crate::network::incoming_game::GameHooks>();
    h.vendor_buy_items = Some(|w, vendor, from, list| match class(w, vendor) {
        Some(c) => (c.on_buy_items)(w, vendor, from, list),
        None => false,
    });
    h.vendor_sell_items = Some(|w, vendor, from, list| match class(w, vendor) {
        Some(c) => (c.on_sell_items)(w, vendor, from, list),
        None => false,
    });
    let ch = w
        .resources
        .get_mut::<crate::items::common::CommonItemHooks>();
    ch.banker_deposit_up_to = Some(|w, from, amount, use_checks, reserve| {
        banker::deposit_up_to(w, from, amount, use_checks, reserve)
    });
}

pub fn register(types: &mut muo_server::type_registry::TypeRegistry) {
    use muo_server::type_registry::{DEFAULT_CALL_PRIORITY, Phase};
    types.register_item(&VENDOR_BUYBACK_PACK, &[]);
    types.register_item(&DISPLAY_CACHE, &[]);
    types.register_phase(
        Phase::Configure,
        "BaseVendor.Configure",
        DEFAULT_CALL_PRIORITY,
        configure,
    );
    types.register_phase(
        Phase::Initialize,
        "BaseVendor.Initialize",
        DEFAULT_CALL_PRIORITY,
        initialize,
    );
}

// ---- construction ----

/// `new BaseVendor(title)` for class `ty`: `base(AIType.AI_Vendor, FightMode.None, 2)`, `LoadSBInfo()`, the title,
/// body and outfit, the speeds and the two packs the client needs.
pub fn new_vendor(w: &mut World, ty: &'static MobileType, title: Option<&str>) -> MobileId {
    let id = new_creature(w, ty, AIType::AiVendor, FightMode::None, 2, 1);
    let now = w.now;
    if let Some(m) = w.mobile_mut(id) {
        m.ext.insert(VendorData::new(now));
    }
    load_sb_info(w, id);
    props_a::set_title(w, id, title.map(str::to_owned));
    (cls(w, id).init_body)(w, id);
    (cls(w, id).init_outfit)(w, id);
    speeds::set_speed(w, id, 0.5, 2.0, true);

    // these packs MUST exist, or the client will crash when the packets are sent
    if let Some(p) = factory::make_item(w, "Server.Items.VendorBuybackPack") {
        items::add_item(w, id, p);
    }
    add_resale_pack(w, id);
    let now = w.now;
    if let Some(d) = data_mut(w, id) {
        d.last_restock = now;
    }
    id
}

/// `AddItem(new Backpack { Layer = Layer.ShopResale, Movable = false, Visible = false })`.
pub fn add_resale_pack(w: &mut World, id: MobileId) -> Option<ItemId> {
    let pack = factory::make_item(w, "Server.Items.Backpack")?;
    (w.item(pack)?.ty.set_layer)(w, pack, Layer::ShopResale);
    muo_server::item::ops::set_movable(w, pack, false);
    muo_server::item::ops::set_visible(w, pack, false);
    items::add_item(w, id, pack);
    Some(pack)
}

/// `LoadSBInfo()`.
pub fn load_sb_info(w: &mut World, id: MobileId) {
    let now = w.now;
    if let Some(d) = data_mut(w, id) {
        d.last_restock = now;
    }
    let old = data_mut(w, id)
        .map(|d| std::mem::take(&mut d.buy_info))
        .unwrap_or_default();
    for mut b in old {
        if let Some(e) = b.display.take() {
            buy::delete_entity(w, e);
        }
    }
    if let Some(d) = data_mut(w, id) {
        d.sell_info.clear();
    }

    let sbs = (cls(w, id).init_sb_info)(w, id);

    if let Some(d) = data_mut(w, id) {
        for sb in sbs {
            d.buy_info.extend(sb.buy);
            d.sell_info.push(sb.sell);
        }
    }
}

fn after_deserialization(w: &mut World, id: MobileId) {
    load_sb_info(w, id);
    let has_pack = items::find_item_on_layer(w, id, Layer::ShopBuy)
        .and_then(|i| w.item(i))
        .is_some_and(|i| i.ty.is(&VENDOR_BUYBACK_PACK));
    if !has_pack {
        // Deleting during deserialization is unsafe; replace on the first tick.
        timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 1, move |w| {
            if w.mobile(id).is_some_and(|m| !m.deleted) {
                let _ = buy_pack(w, id);
            }
        });
    }
    // `if (IsParagon) IsParagon = false`
    if crate::creature::read(w, id, false, |d| d.is_paragon) {
        crate::creature::overrides::set_is_paragon(w, id, false);
    }
}

// ---- body and outfit ----

fn race_ops(w: &World, id: MobileId) -> Option<&'static RaceOps> {
    let races = w.resources.get::<Races>()?;
    let race = w
        .mobile(id)
        .and_then(|m| m.race)
        .and_then(|r| races.get(r))
        .or_else(|| races.default_race())?;
    Some(race.ops)
}

fn human_ops(w: &World) -> Option<&'static RaceOps> {
    Some(w.resources.get::<Races>()?.default_race()?.ops)
}

/// `Race.RandomHairHue()` of the vendor's race.
pub fn race_random_hair_hue(w: &mut World, id: MobileId) -> i32 {
    match race_ops(w, id) {
        Some(o) => (o.random_hair_hue)(&mut w.rng),
        None => 0,
    }
}

/// `Utility.AssignRandomHair(m, hue)`.
pub fn assign_random_hair(w: &mut World, id: MobileId, hue: i32) {
    let female = w.mobile(id).is_some_and(|m| m.female);
    if let Some(o) = race_ops(w, id) {
        let v = (o.random_hair)(&mut w.rng, female);
        props_b::set_hair_item_id(w, id, v);
    }
    props_b::set_hair_hue(w, id, hue);
}

/// `Utility.AssignRandomHair(m)`: the item, then `Race.RandomHairHue()`.
pub fn assign_random_hair_random_hue(w: &mut World, id: MobileId) {
    let female = w.mobile(id).is_some_and(|m| m.female);
    if let Some(o) = race_ops(w, id) {
        let v = (o.random_hair)(&mut w.rng, female);
        props_b::set_hair_item_id(w, id, v);
        let h = (o.random_hair_hue)(&mut w.rng);
        props_b::set_hair_hue(w, id, h);
    }
}

/// `HairItemID = Race.RandomHair(Female)`: the race's random hair for the current `Female`.
pub fn roll_hair_item(w: &mut World, id: MobileId) {
    let female = w.mobile(id).is_some_and(|m| m.female);
    if let Some(o) = race_ops(w, id) {
        let v = (o.random_hair)(&mut w.rng, female);
        props_b::set_hair_item_id(w, id, v);
    }
}

/// `HairHue = Race.RandomHairHue()`.
pub fn roll_hair_hue(w: &mut World, id: MobileId) {
    let h = race_random_hair_hue(w, id);
    props_b::set_hair_hue(w, id, h);
}

/// `Race.RandomFacialHair(this);` as a statement: the roll is made and discarded.
pub fn roll_facial_hair(w: &mut World, id: MobileId) {
    let female = w.mobile(id).is_some_and(|m| m.female);
    if let Some(o) = race_ops(w, id) {
        let _ = (o.random_facial_hair)(&mut w.rng, female);
    }
}

/// `Utility.AssignRandomFacialHair(m, hue)`.
pub fn assign_random_facial_hair(w: &mut World, id: MobileId, hue: i32) {
    let female = w.mobile(id).is_some_and(|m| m.female);
    if let Some(o) = race_ops(w, id) {
        let v = (o.random_facial_hair)(&mut w.rng, female);
        props_b::set_facial_hair_item_id(w, id, v);
    }
    props_b::set_facial_hair_hue(w, id, hue);
}

/// `Race.Human.RandomSkinHue()`.
pub fn human_random_skin_hue(w: &mut World) -> i32 {
    match human_ops(w) {
        Some(o) => (o.random_skin_hue)(&mut w.rng),
        None => 0,
    }
}

/// `InitBody()`.
pub fn init_body(w: &mut World, id: MobileId) {
    stats::init_stats(w, id, 100, 100, 25);

    let hue = utility::random_dyed_hue(&mut w.rng);
    if let Some(m) = w.mobile_mut(id) {
        m.speech_hue = hue;
    }
    let skin = human_random_skin_hue(w);
    muo_server::mobile::base::set_hue(w, id, skin);

    let female = (cls(w, id).get_gender)(w, id);
    props_b::set_female(w, id, female);
    let (body, kind) = if female {
        (0x191, "female")
    } else {
        (0x190, "male")
    };
    props_b::set_body(w, id, muo_primitives::Body(body));
    let name = crate::misc::small::random_name(w, kind);
    set_name(w, id, Some(name));
}

fn set_name(w: &mut World, id: MobileId, value: Option<String>) {
    if let Some(f) = w.mobile(id).map(|m| m.ty.set_name) {
        f(w, id, value);
    }
}

/// `GetRandomHue()`.
pub fn get_random_hue(w: &mut World, _id: MobileId) -> i32 {
    match rnd::below(w, 5) {
        0 => utility::random_blue_hue(&mut w.rng),
        1 => utility::random_green_hue(&mut w.rng),
        2 => utility::random_red_hue(&mut w.rng),
        3 => utility::random_yellow_hue(&mut w.rng),
        _ => utility::random_neutral_hue(&mut w.rng),
    }
}

/// `GetShoeHue()`.
pub fn get_shoe_hue(w: &mut World, _id: MobileId) -> i32 {
    if rnd::double(w) < 0.1 {
        0
    } else {
        utility::random_neutral_hue(&mut w.rng)
    }
}

/// `GetHairHue()`: `Race.RandomHairHue()`.
pub fn get_hair_hue(w: &mut World, id: MobileId) -> i32 {
    race_random_hair_hue(w, id)
}

/// `AddItem(new T(hue))` of a clothing class: the hue argument is read before the constructor runs.
pub fn add_hued(w: &mut World, id: MobileId, full_name: &str, hue: i32) -> Option<ItemId> {
    let it = factory::make_item(w, full_name)?;
    muo_server::item::base::set_hue(w, it, hue);
    items::add_item(w, id, it);
    Some(it)
}

/// `InitOutfit()`.
pub fn init_outfit(w: &mut World, id: MobileId) {
    let c = cls(w, id);
    let top = match rnd::below(w, 3) {
        0 => "Server.Items.FancyShirt",
        1 => "Server.Items.Doublet",
        _ => "Server.Items.Shirt",
    };
    let h = (c.get_random_hue)(w, id);
    add_hued(w, id, top, h);

    let shoes = match (c.shoe_type)(w, id) {
        ShoeType::Shoes => "Server.Items.Shoes",
        ShoeType::Boots => "Server.Items.Boots",
        ShoeType::Sandals => "Server.Items.Sandals",
        _ => "Server.Items.ThighBoots",
    };
    let h = (c.get_shoe_hue)(w, id);
    add_hued(w, id, shoes, h);

    let hair_hue = (c.get_hair_hue)(w, id);
    assign_random_hair(w, id, hair_hue);
    assign_random_facial_hair(w, id, hair_hue);

    let female = w.mobile(id).is_some_and(|m| m.female);
    if female {
        let name = match rnd::below(w, 6) {
            0 => "Server.Items.ShortPants",
            1 | 2 => "Server.Items.Kilt",
            _ => "Server.Items.Skirt",
        };
        let h = (c.get_random_hue)(w, id);
        add_hued(w, id, name, h);
    } else {
        let name = if rnd::bool(w) {
            "Server.Items.LongPants"
        } else {
            "Server.Items.ShortPants"
        };
        let h = (c.get_random_hue)(w, id);
        add_hued(w, id, name, h);
    }

    pack_gold_range(w, id, 100, 200);
}

// ---- morphing ----

fn region_is_part_of(w: &World, id: MobileId, name: &str) -> bool {
    w.mobile(id)
        .is_some_and(|m| muo_server::region::is_part_of_name(w, m.region, name, false))
}

/// `CheckMorph()`.
pub fn check_morph(w: &mut World, id: MobileId) {
    (cls(w, id).check_morph)(w, id);
}

/// `BaseVendor.CheckMorph()`.
pub fn check_morph_base(w: &mut World, id: MobileId) {
    if check_gargoyle(w, id) {
        return;
    }
    if check_necromancer(w, id) {
        return;
    }
    check_tokuno(w, id);
}

/// `IsTokunoVendor`.
pub fn is_tokuno_vendor(w: &World, id: MobileId) -> bool {
    w.mobile(id)
        .is_some_and(|m| m.map == Some(muo_server::map::MapId::TOKUNO))
}

/// `IsTerMurVendor`.
pub fn is_ter_mur_vendor(w: &World, id: MobileId) -> bool {
    w.mobile(id)
        .is_some_and(|m| m.map == Some(muo_server::map::MapId::TER_MUR))
}

/// `CheckTokuno()`.
pub fn check_tokuno(w: &mut World, id: MobileId) -> bool {
    if !is_tokuno_vendor(w, id) {
        return false;
    }
    let (female, name) = match w.mobile(id) {
        Some(m) => (m.female, (m.ty.name)(w, id).unwrap_or_default()),
        None => return false,
    };
    let kind = if female {
        "tokuno female"
    } else {
        "tokuno male"
    };
    let known = crate::misc::small::get_name_list(w, kind).is_some_and(|l| l.contains_name(&name));
    if !known {
        (cls(w, id).turn_to_tokuno)(w, id);
    }
    true
}

/// `BaseVendor.TurnToTokuno()`.
pub fn turn_to_tokuno(w: &mut World, id: MobileId) {
    let Some(female) = w.mobile(id).map(|m| m.female) else {
        return;
    };
    let kind = if female {
        "tokuno female"
    } else {
        "tokuno male"
    };
    let n = crate::misc::small::random_name(w, kind);
    set_name(w, id, Some(n));
}

/// `CheckGargoyle()`.
pub fn check_gargoyle(w: &mut World, id: MobileId) -> bool {
    if w.mobile(id).map(|m| m.map) != Some(Some(muo_server::map::MapId::ILSHENAR)) {
        return false;
    }
    if !region_is_part_of(w, id, "Gargoyle City") {
        return false;
    }
    let (body, hue) = w.mobile(id).map_or((0, 0), |m| {
        (
            muo_server::mobile::part1::props_b::body(w, id).body_id(),
            (m.ty.hue)(w, id),
        )
    });
    if body != 0x2F6 || (hue & 0x8000) == 0 {
        turn_to_gargoyle(w, id);
    }
    true
}

/// `CheckNecromancer()`.
pub fn check_necromancer(w: &mut World, id: MobileId) -> bool {
    if w.mobile(id).map(|m| m.map) != Some(Some(muo_server::map::MapId::MALAS)) {
        return false;
    }
    if !region_is_part_of(w, id, "Umbra") {
        return false;
    }
    if w.mobile(id).map_or(0, |m| (m.ty.hue)(w, id)) != 0x83E8 {
        turn_to_necromancer(w, id);
    }
    true
}

/// `GetRandomNecromancerHue()`.
pub fn get_random_necromancer_hue(w: &mut World) -> i32 {
    match rnd::below(w, 20) {
        0 => 0,
        1 => 0x4E9,
        _ => {
            let i = rnd::below(w, 2);
            [0x485, 0x497][i as usize]
        }
    }
}

/// `TurnToNecromancer()`.
pub fn turn_to_necromancer(w: &mut World, id: MobileId) {
    let list: Vec<ItemId> = w.mobile(id).map(|m| m.items.clone()).unwrap_or_default();
    for item in list {
        let hit = w.item(item).is_some_and(|i| {
            i.ty.is(&crate::items::clothing::BASE_CLOTHING)
                || i.ty.is(&crate::items::weapon::BASE_WEAPON)
                || i.ty.is(&crate::items::armor::BASE_ARMOR)
                || hooks(w).is_some() && factory_is_tool(w, item)
        });
        if hit {
            let h = get_random_necromancer_hue(w);
            muo_server::item::base::set_hue(w, item, h);
        }
    }
    props_b::set_hair_hue(w, id, 0);
    props_b::set_facial_hair_hue(w, id, 0);
    muo_server::mobile::base::set_hue(w, id, 0x83E8);
}

fn factory_is_tool(w: &World, item: ItemId) -> bool {
    factory::item_is(w, item, "Server.Items.BaseTool")
}

/// `TurnToGargoyle()`.
pub fn turn_to_gargoyle(w: &mut World, id: MobileId) {
    let list: Vec<ItemId> = w.mobile(id).map(|m| m.items.clone()).unwrap_or_default();
    for item in list {
        if w.item(item)
            .is_some_and(|i| i.ty.is(&crate::items::clothing::BASE_CLOTHING))
            && let Some(t) = w.item(item).map(|i| i.ty)
        {
            (t.delete)(w, item);
        }
    }
    props_b::set_hair_item_id(w, id, 0);
    props_b::set_facial_hair_item_id(w, id, 0);
    props_b::set_body(w, id, muo_primitives::Body(0x2F6));
    let hue = utility::random_bright_hue(&mut w.rng) | 0x8000;
    muo_server::mobile::base::set_hue(w, id, hue);
    let n = crate::misc::small::random_name(w, "gargoyle vendor");
    set_name(w, id, Some(n));
    capitalize_title(w, id);
}

/// `CapitalizeTitle()`: `Title = Title.Capitalize()`.
pub fn capitalize_title(w: &mut World, id: MobileId) {
    let t = w.mobile(id).and_then(|m| m.title.clone());
    if let Some(t) = t {
        props_a::set_title(w, id, Some(muo_io::text::helpers::capitalize(&t)));
    }
}

fn on_map_change(w: &mut World, id: MobileId, old: Option<muo_server::map::MapId>) {
    (BASE_CREATURE.on_map_change)(w, id, old);
    check_morph(w, id);
    load_sb_info(w, id);
    // LoadSBInfo re-anchored the restock grid. Not done there since it also runs on deserialize.
    cancel_and_purge_buyback(w, id);
}

fn on_delete(w: &mut World, id: MobileId) {
    if let Some(d) = data_mut(w, id) {
        let mut t = std::mem::take(&mut d.buyback_purge);
        t.cancel(w);
    }
    (BASE_CREATURE.on_delete)(w, id);
}

// ---- the buyback pack and restock ----

/// `BuyPack`: the vendor's `VendorBuybackPack`, replacing a legacy plain pack on the layer.
pub fn buy_pack(w: &mut World, id: MobileId) -> Option<ItemId> {
    let existing = items::find_item_on_layer(w, id, Layer::ShopBuy);
    if let Some(e) = existing {
        if w.item(e).is_some_and(|i| i.ty.is(&VENDOR_BUYBACK_PACK)) {
            return Some(e);
        }
        // Legacy pack; deleting it stays linear even with a huge backlog.
        if let Some(t) = w.item(e).map(|i| i.ty) {
            (t.delete)(w, e);
        }
    }
    let pack = packs::new_buyback_pack(w);
    items::add_item(w, id, pack);
    Some(pack)
}

/// `BuybackPurgeScheduled`.
pub fn buyback_purge_scheduled(w: &World, id: MobileId) -> bool {
    data(w, id).is_some_and(|d| d.buyback_purge.running(w))
}

/// `GetNextBuybackPurge(lastRestock, restockDelay, now)`: the next restock boundary strictly after `now`.
pub fn get_next_buyback_purge(
    last_restock: DateTime,
    restock_delay: TimeSpan,
    now: DateTime,
) -> DateTime {
    if restock_delay <= TimeSpan::ZERO {
        return now;
    }
    // A backwards clock adjustment can future-date the anchor past RestockDelay.
    let last = if last_restock > now {
        now
    } else {
        last_restock
    };
    let next = last + restock_delay;
    if next > now {
        return next;
    }
    // Restock is lazy and may be hours overdue; stay on its grid.
    let periods = now.since(last).ticks() / restock_delay.ticks() + 1;
    last + TimeSpan::from_ticks(restock_delay.ticks() * periods)
}

/// `AddToBuyback(item, capacity)`.
pub fn add_to_buyback(w: &mut World, id: MobileId, item: ItemId, capacity: i32) {
    let Some(pack) = buy_pack(w, id) else { return };
    packs::add_buyback(w, pack, item, capacity);
    let deleted = w.item(item).is_none_or(|i| i.deleted());
    if !deleted && !buyback_purge_scheduled(w, id) {
        let last = data(w, id).map_or(w.now, |d| d.last_restock);
        let delay = (cls(w, id).restock_delay)(w, id);
        let now = w.now;
        let delay = get_next_buyback_purge(last, delay, now).since(now);
        let h = timer::start(w, delay, TimeSpan::ZERO, 1, move |w| {
            cancel_and_purge_buyback(w, id);
        });
        if let Some(d) = data_mut(w, id) {
            d.buyback_purge = h;
        }
    }
}

/// `CancelAndPurgeBuyback()`.
pub fn cancel_and_purge_buyback(w: &mut World, id: MobileId) {
    if let Some(d) = data_mut(w, id) {
        let mut t = std::mem::take(&mut d.buyback_purge);
        t.cancel(w);
    }
    // PurgeBuyback: (FindItemOnLayer(Layer.ShopBuy) as VendorBuybackPack)?.Purge()
    if let Some(p) = items::find_item_on_layer(w, id, Layer::ShopBuy)
        && w.item(p).is_some_and(|i| i.ty.is(&VENDOR_BUYBACK_PACK))
    {
        packs::purge(w, p);
    }
}

/// `Restock()`.
pub fn restock(w: &mut World, id: MobileId) {
    let now = w.now;
    if let Some(d) = data_mut(w, id) {
        d.last_restock = now;
    }
    cancel_and_purge_buyback(w, id);
    if let Some(d) = data_mut(w, id) {
        for b in &mut d.buy_info {
            b.on_restock();
        }
    }
}

/// `CheckVendorAccess(from)`.
pub fn check_vendor_access(w: &World, id: MobileId, from: MobileId) -> bool {
    use crate::systems::regions::{get_guarded, guarded};
    let (Some(vm), Some(fm)) = (w.mobile(id), w.mobile(from)) else {
        return true;
    };
    let ok = |region| {
        get_guarded(w, region).map(|g| guarded::check_vendor_access(w, g, from)) != Some(false)
    };
    ok(vm.region) || (vm.region != fm.region && ok(fm.region))
}

/// `GetPriceScalar()`: `100 + Town.FromRegion(Region)?.Tax ?? 0` binds as `(100 + tax?) ?? 0`, so a vendor outside
/// a faction town has scalar 0.
pub fn get_price_scalar(w: &World, id: MobileId) -> i32 {
    hooks(w)
        .and_then(|h| h.town_tax)
        .and_then(|f| f(w, id))
        .map_or(0, |t| 100 + t)
}

/// `UpdateBuyInfo()`.
pub fn update_buy_info(w: &mut World, id: MobileId) {
    let scalar = (cls(w, id).get_price_scalar)(w, id);
    if let Some(d) = data_mut(w, id) {
        for b in &mut d.buy_info {
            b.price_scalar = scalar;
        }
    }
}

/// `BaseHouse.NewVendorSystem`: `Core.AOS`.
pub fn new_vendor_system(w: &World) -> bool {
    w.core_aos()
}

#[cfg(test)]
mod buyback_tests;
#[cfg(test)]
mod tests;
