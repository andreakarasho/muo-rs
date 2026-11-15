//! The Engines/ML Quests/Items classes that sit on a base another crate owns (jewelry, tools, food, gems, books,
//! pigments, clothing, containers, teleporters) and the reward bags / craftsman satchels. See also `transient.rs`.

use muo_content::hooks::rnd;
use muo_content::items::cont::{BACKPACK, BAG, BASE_CONTAINER, WOODEN_BOX};
use muo_content::loot;
use muo_io::text::TextDefinition;
use muo_primitives::{MessageType, Point3D, utility};
use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, ops};
use muo_server::map::MapId;
use muo_server::mobile::part2::messages as mm;
use muo_server::property_list::PropertyList;
use muo_server::type_registry::TypeRegistry;

use super::engine;
use super::item_support::{add_quest_item_property, hue, leaf};

/// `new Parent()` then the class swapped to `ty`: the base constructor of a class this crate extends. A class that
/// changes none of the parent's virtuals the constructor calls (all of these) builds the same object.
fn new_as(w: &mut World, parent: &'static ItemType, ty: &'static ItemType) -> ItemId {
    let id = (parent.construct.expect("a constructible parent"))(w);
    if let Some(i) = w.item_mut(id) {
        i.ty = ty;
    }
    id
}

/// One concrete class on a ready-made parent: `construct` is the parent's, then the class's item template.
macro_rules! sub {
    (
        $(#[$m:meta])*
        $name:ident = $full:literal : $parent:ident;
        $($f:ident : $v:expr,)*
    ) => {
        leaf! {
            $(#[$m])*
            $name = $full : $parent, v0;
            @props |w| {
                let id = new_as(w, &$parent, &$name);
                muo_content::templates::apply_item_as(w, id, $full);
                id
            };
            $($f: $v,)*
        }
    };
}

/// The class's item template applied to `id`, made by its parent's constructor.
fn with_template(w: &mut World, id: ItemId) -> ItemId {
    if let Some(ty) = w.item(id).map(|i| i.ty) {
        muo_content::templates::apply_item_as(w, id, ty.full_name);
    }
    id
}

fn quest_name_properties(w: &World, id: ItemId, list: &mut PropertyList) {
    (ITEM_BASE.add_name_properties)(w, id, list);
    add_quest_item_property(w, id, list);
}

// ---- AnOldNecklace, AnOldRing, ReginasRing, TransparentHeart ----------------------------------------------

sub! {
    /// `Server.Items.AnOldNecklace`.
    AN_OLD_NECKLACE = "Server.Items.AnOldNecklace" : NECKLACE;
}
use muo_content::items::leaves::{GOLD_RING, NECKLACE};

sub! {
    /// `Server.Items.AnOldRing`.
    AN_OLD_RING = "Server.Items.AnOldRing" : GOLD_RING;
}

sub! {
    /// `Server.Items.ReginasRing`.
    REGINAS_RING = "Server.Items.ReginasRing" : SILVER_RING;
    add_name_properties: quest_name_properties,
}
use muo_items::jewels::leaves::{GOLD_EARRINGS, SILVER_RING};

sub! {
    /// `Server.Items.TransparentHeart`.
    TRANSPARENT_HEART = "Server.Items.TransparentHeart" : GOLD_EARRINGS;
}

// ---- AndrosGratitude -------------------------------------------------------------------------------------

/// `AndrosGratitude(): base(10)`, then the template.
fn andros_gratitude_new(w: &mut World) -> ItemId {
    let id = muo_items::skill_items::tools::new_smith_hammer(w, Some(10));
    if let Some(i) = w.item_mut(id) {
        i.ty = &ANDROS_GRATITUDE;
    }
    with_template(w, id)
}

leaf! {
    /// `Server.Items.AndrosGratitude`.
    ANDROS_GRATITUDE = "Server.Items.AndrosGratitude" : SMITH_HAMMER, v0;
    @props andros_gratitude_new;
}
use muo_items::skill_items::tools::SMITH_HAMMER;

// ---- Bleach ------------------------------------------------------------------------------------------------

fn bleach_new(w: &mut World) -> ItemId {
    let id = muo_engines_pigments::base_new(w, &BLEACH, 10);
    with_template(w, id)
}
use crate::tokuno::pigments as muo_engines_pigments;
use crate::tokuno::pigments::PIGMENTS_OF_TOKUNO;

leaf! {
    /// `Server.Items.Bleach`.
    BLEACH = "Server.Items.Bleach" : PIGMENTS_OF_TOKUNO, v0;
    @props bleach_new;
}

// ---- food ------------------------------------------------------------------------------------------------------

leaf! {
    /// `Server.Items.MiniatureMushroom`.
    MINIATURE_MUSHROOM = "Server.Items.MiniatureMushroom" : FOOD, v0;
    @props |w| {
        let id = muo_content::items::food::new_food(w, &MINIATURE_MUSHROOM, 0, 1);
        with_template(w, id)
    };
}
use muo_content::items::food::{CHICKEN_LEG, FISH_STEAK, FOOD};

sub! {
    /// `Server.Items.PeppercornFishsteak`.
    PEPPERCORN_FISHSTEAK = "Server.Items.PeppercornFishsteak" : FISH_STEAK;
}

sub! {
    /// `Server.Items.PixieLeg` (`PixieLeg(int amount = 1)`).
    PIXIE_LEG = "Server.Items.PixieLeg" : CHICKEN_LEG;
}

// ---- RedLeatherBook -----------------------------------------------------------------------------------------

sub! {
    /// `Server.Items.RedLeatherBook`.
    RED_LEATHER_BOOK = "Server.Items.RedLeatherBook" : BLUE_BOOK;
}
use muo_items::books::BLUE_BOOK;

// ---- PrismaticAmber ----------------------------------------------------------------------------------------

/// `DestroyItem(from)`.
fn amber_destroy(w: &mut World, id: ItemId, from: MobileId) {
    mm::send_localized_message(w, from, 500424, "", 0x3B2); // You destroyed the item.
    if let Some(ty) = w.item(id).map(|i| i.ty) {
        (ty.delete)(w, id);
    }
}

sub! {
    /// `Server.Items.PrismaticAmber`.
    PRISMATIC_AMBER = "Server.Items.PrismaticAmber" : AMBER;
    add_name_properties: |w, id, list| {
        (ITEM_BASE.add_name_properties)(w, id, list);
        list.add(1075269); // Destroyed when dropped
    },
    drop_to_world: |w, id, from, p| {
        let ret = (AMBER.drop_to_world)(w, id, from, p);
        if ret {
            amber_destroy(w, id, from);
        }
        ret
    },
    drop_to_mobile: |w, id, from, target, p| {
        let ret = (AMBER.drop_to_mobile)(w, id, from, target, p);
        if ret {
            amber_destroy(w, id, from);
        }
        ret
    },
    drop_to_item: |w, id, from, target, p| {
        let ret = (AMBER.drop_to_item)(w, id, from, target, p);
        // `Parent != from.Backpack`
        let pack = muo_server::mobile::part1::props_b::backpack(w, from).map(EntityId::Item);
        if ret && w.item(id).and_then(|i| i.parent) != pack {
            amber_destroy(w, id, from);
        }
        ret
    },
}
use muo_items::gems::AMBER;

// ---- GustarShroud (NewHavenSkillTraining) --------------------------------------------------------------------

leaf! {
    /// `Server.Engines.MLQuests.Definitions.GustarShroud`.
    GUSTAR_SHROUD = "Server.Engines.MLQuests.Definitions.GustarShroud" : BASE_OUTER_TORSO, v0;
    @props |w| {
        let id = muo_content::items::clothing::new_clothing(w, &GUSTAR_SHROUD, 0, 0);
        with_template(w, id)
    };
    default_name: |w, id| crate::template_keys::item_name(w, id, BASE_OUTER_TORSO.default_name),
}
use muo_content::items::clothing::BASE_OUTER_TORSO;

// ---- GiftForArielle -----------------------------------------------------------------------------------------

leaf! {
    /// `Server.Items.GiftForArielle`.
    GIFT_FOR_ARIELLE = "Server.Items.GiftForArielle" : BASE_CONTAINER, v0;
    @template template_container;
    default_gump_id: |w, id| crate::template_keys::item_int(w, id, "defaultGumpId", BASE_CONTAINER.default_gump_id),
    add_name_properties: quest_name_properties,
}

// ---- PrismaticCrystal ---------------------------------------------------------------------------------------

/// `PrismaticCrystal.OnDoubleClick`.
fn prismatic_crystal_double_click(w: &mut World, id: ItemId, from: MobileId) {
    if muo_content::player_mobile::data(w, from).is_none() {
        return;
    }
    let Some(pack) = muo_server::mobile::part1::props_b::backpack(w, from) else {
        return;
    };

    let loc = ops::get_world_location(w, id);
    if !w
        .mobile(from)
        .is_some_and(|m| (m.ty.in_range_point3d)(w, from, loc, 2))
    {
        mm::local_overhead_message_number(w, from, MessageType::REGULAR, 0x3B2, 1019045, ""); // I can't reach that.
        return;
    }

    let doing = engine::find_quest(w, "UnfadingMemoriesPartOne")
        .is_some_and(|q| engine::is_doing_quest(w, from, q));
    let has_amber =
        muo_content::factory::find_item_by_type(w, pack, "Server.Items.PrismaticAmber").is_some();
    // `FindItemByType<PrismaticAmber>(false)`: not recursive
    let has_amber = has_amber
        && w.item(pack).is_some_and(|p| {
            p.items
                .iter()
                .any(|&i| engine::item_is(w, i, "PrismaticAmber"))
        });
    if !doing || has_amber {
        mm::send_localized_message(w, from, 1075464, "", 0x3B2); // You already have as many of those as you need.
        return;
    }

    let Some(amber) = muo_content::factory::make_item(w, "Server.Items.PrismaticAmber") else {
        return;
    };
    if muo_server::mobile::part2::items::place_in_backpack(w, from, amber) {
        engine::mark_quest_item(w, from, amber);
        if let Some(ty) = w.item(id).map(|i| i.ty) {
            (ty.delete)(w, id);
        }
    } else {
        mm::send_localized_message(w, from, 502385, "", 0x3B2); // Your pack cannot hold this item.
        if let Some(ty) = w.item(amber).map(|i| i.ty) {
            (ty.delete)(w, amber);
        }
    }
}

leaf! {
    /// `Server.Items.PrismaticCrystal`.
    PRISMATIC_CRYSTAL = "Server.Items.PrismaticCrystal" : ITEM_BASE, v0;
    @template template_new;
    on_double_click: prismatic_crystal_double_click,
}

// ---- RewardBags ----------------------------------------------------------------------------------------------

/// `RewardBag.Enhance(loot)`.
pub fn enhance(w: &mut World, loot: ItemId) {
    use muo_content::factory::item_is;
    let apply = w
        .resources
        .get::<muo_content::creature::packing::PackHooks>()
        .and_then(|h| h.apply_aos_attributes);
    let is_weapon = item_is(w, loot, "Server.Items.BaseWeapon");
    let is_armor = item_is(w, loot, "Server.Items.BaseArmor");
    let is_jewel = item_is(w, loot, "Server.Items.BaseJewel");
    if is_weapon {
        let n = rnd::min_max(w, 1, 5);
        if let Some(f) = apply {
            f(w, loot, n, 10, 80);
        }
        return;
    }
    if is_armor {
        let n = rnd::min_max(w, 1, 5);
        if let Some(f) = apply {
            f(w, loot, n, 10, 80);
        }
    }
    if is_jewel {
        let n = rnd::min_max(w, 1, 5);
        if let Some(f) = apply {
            f(w, loot, n, 10, 80);
        }
    }
}

fn drop_into(w: &mut World, c: ItemId, item: ItemId) {
    if let Some(ty) = w.item(c).map(|i| i.ty) {
        (ty.drop_item)(w, c, item);
    }
}

/// `RewardBag.Fill(c, itemCount, talismanChance)`.
pub fn fill(w: &mut World, c: ItemId, item_count: i32, talisman_chance: f64) {
    let h = utility::random_nondyed_hue(&mut w.rng);
    hue(w, c, h);

    let mut done = 0;
    if rnd::double(w) < talisman_chance {
        if let Some(t) = muo_content::factory::make_item(w, "Server.Items.RandomTalisman") {
            drop_into(w, c, t);
        }
        done += 1;
    }

    while done < item_count {
        let loot = match rnd::below(w, 5) {
            0 => loot::random_weapon(w, false, true, false),
            1 => loot::random_armor(w, false, true),
            2 => loot::random_ranged_weapon(w, false, true, false),
            3 => loot::random_jewelry(w),
            _ => loot::random_hat(w, false), // 4
        };
        done += 1;
        let Some(loot) = loot else { continue };
        enhance(w, loot);
        drop_into(w, c, loot);
    }
}

/// `Fill(this, itemCount, talismanChance)` with the class's props `rewardItemCount` and `rewardTalismanChance`.
fn fill_from_props(w: &mut World, id: ItemId) {
    use muo_content::templates::props;
    let count = props::as_i32(props::item(w, id, "rewardItemCount")).unwrap_or(0);
    let chance = props::as_f64(props::item(w, id, "rewardTalismanChance")).unwrap_or(0.0);
    fill(w, id, count, chance);
}

/// A reward bag: its template (the graphic) then [`fill_from_props`].
fn reward_bag_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = super::item_support::template_container(w, ty);
    fill_from_props(w, id);
    id
}

macro_rules! bag {
    ($(#[$m:meta])* $name:ident = $full:literal : $parent:ident) => {
        leaf! {
            $(#[$m])*
            $name = $full : $parent, v0;
            construct: Some(|w| reward_bag_new(w, &$name)),
        }
    };
}

bag!(/// `Server.Engines.MLQuests.Items.SmallBagOfTrinkets`.
    SMALL_BAG_OF_TRINKETS = "Server.Engines.MLQuests.Items.SmallBagOfTrinkets" : BAG);
bag!(/// `Server.Engines.MLQuests.Items.BagOfTrinkets`.
    BAG_OF_TRINKETS = "Server.Engines.MLQuests.Items.BagOfTrinkets" : BAG);
bag!(/// `Server.Engines.MLQuests.Items.BagOfTreasure`.
    BAG_OF_TREASURE = "Server.Engines.MLQuests.Items.BagOfTreasure" : BAG);
bag!(/// `Server.Engines.MLQuests.Items.LargeBagOfTreasure`.
    LARGE_BAG_OF_TREASURE = "Server.Engines.MLQuests.Items.LargeBagOfTreasure" : BAG);

// ---- CraftmansSatchel ------------------------------------------------------------------------------------------

/// `BaseCraftmansSatchel.AddBaseLoot(params Type[][] lootSets)`.
fn add_base_loot(w: &mut World, id: ItemId, sets: &[&[&str]]) {
    let set = sets[rnd::below(w, sets.len() as i32) as usize]; // lootSets.RandomElement()
    let Some(item) = loot::construct_any(w, set) else {
        return;
    };
    enhance(w, item);
    drop_into(w, id, item);
}

/// `BaseCraftmansSatchel.AddRecipe(system)`.
fn add_recipe(w: &mut World, id: ItemId, system: &str) {
    let recipes = crate::craft::system(w, system)
        .map(|s| s.recipes.clone())
        .unwrap_or_default();
    // RandomRecipe(): -1 for no recipes, else a random element
    if recipes.is_empty() {
        return;
    }
    let recipe_id = recipes[rnd::below(w, recipes.len() as i32) as usize];
    let make = w
        .resources
        .get::<super::defs::special::QuestItemHooks>()
        .and_then(|h| h.recipe_scroll);
    if let Some(item) = make.and_then(|f| f(w, recipe_id)) {
        drop_into(w, id, item);
    }
}

const TALISMAN: &[&str] = &["Server.Items.RandomTalisman"];

/// `BaseCraftmansSatchel()`: the class's template (it inherits the `BaseCraftmansSatchel` one).
fn satchel_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    super::item_support::template_container(w, ty)
}

leaf! {
    /// `Server.Engines.MLQuests.Items.BaseCraftmansSatchel` (abstract).
    BASE_CRAFTMANS_SATCHEL = "Server.Engines.MLQuests.Items.BaseCraftmansSatchel" : BACKPACK, v0;
    construct: None,
}

leaf! {
    /// `Server.Engines.MLQuests.Items.TailorSatchel`.
    TAILOR_SATCHEL = "Server.Engines.MLQuests.Items.TailorSatchel" : BASE_CRAFTMANS_SATCHEL, v0;
    construct: Some(|w| {
        let id = satchel_new(w, &TAILOR_SATCHEL);
        add_base_loot(w, id, &[loot::ML_ARMOR_TYPES, loot::JEWELRY_TYPES, TALISMAN]);
        if rnd::bool(w) {
            add_recipe(w, id, "Tailoring");
        }
        id
    }),
}

leaf! {
    /// `Server.Engines.MLQuests.Items.BlacksmithSatchel`.
    BLACKSMITH_SATCHEL = "Server.Engines.MLQuests.Items.BlacksmithSatchel" : BASE_CRAFTMANS_SATCHEL, v0;
    construct: Some(|w| {
        let id = satchel_new(w, &BLACKSMITH_SATCHEL);
        add_base_loot(w, id, &[loot::ML_WEAPON_TYPES, loot::JEWELRY_TYPES, TALISMAN]);
        if rnd::bool(w) {
            add_recipe(w, id, "Blacksmithy");
        }
        id
    }),
}

leaf! {
    /// `Server.Engines.MLQuests.Items.TinkerSatchel`.
    TINKER_SATCHEL = "Server.Engines.MLQuests.Items.TinkerSatchel" : BASE_CRAFTMANS_SATCHEL, v0;
    construct: Some(|w| {
        let id = satchel_new(w, &TINKER_SATCHEL);
        add_base_loot(
            w,
            id,
            &[loot::ML_ARMOR_TYPES, loot::ML_WEAPON_TYPES, loot::ML_RANGED_WEAPON_TYPES, loot::JEWELRY_TYPES, TALISMAN],
        );
        if rnd::bool(w) {
            match rnd::below(w, 6) {
                0 => add_recipe(w, id, "Inscription"),
                1 => add_recipe(w, id, "Alchemy"),
                _ => {} // TODO: AddNonArtifactRecipe for cases 2 to 5
            }
        }
        id
    }),
}

leaf! {
    /// `Server.Engines.MLQuests.Items.FletchingSatchel`.
    FLETCHING_SATCHEL = "Server.Engines.MLQuests.Items.FletchingSatchel" : BASE_CRAFTMANS_SATCHEL, v0;
    construct: Some(|w| {
        let id = satchel_new(w, &FLETCHING_SATCHEL);
        add_base_loot(w, id, &[loot::ML_RANGED_WEAPON_TYPES, loot::JEWELRY_TYPES, TALISMAN]);
        if rnd::bool(w) {
            add_recipe(w, id, "BowFletching");
        }
        id
    }),
}

leaf! {
    /// `Server.Engines.MLQuests.Items.CarpentrySatchel`.
    CARPENTRY_SATCHEL = "Server.Engines.MLQuests.Items.CarpentrySatchel" : BASE_CRAFTMANS_SATCHEL, v0;
    construct: Some(|w| {
        let id = satchel_new(w, &CARPENTRY_SATCHEL);
        add_base_loot(
            w,
            id,
            &[loot::ML_ARMOR_TYPES, loot::ML_WEAPON_TYPES, loot::ML_RANGED_WEAPON_TYPES, loot::JEWELRY_TYPES, TALISMAN],
        );
        if rnd::bool(w) {
            add_recipe(w, id, "Carpentry");
        }
        id
    }),
}

// ---- RewardStrongbox (RewardBags) ----------------------------------------------------------------------------

leaf! {
    /// `Server.Engines.MLQuests.Items.RewardStrongbox`.
    REWARD_STRONGBOX = "Server.Engines.MLQuests.Items.RewardStrongbox" : WOODEN_BOX, v0;
    construct: Some(|w| reward_bag_new(w, &REWARD_STRONGBOX)),
}

// ---- Teleporters ---------------------------------------------------------------------------------------------

use muo_items::misc::teleporter::{self as tele, TELEPORTER, TELEPORTER_CLASS, TeleporterClass};

/// What `MLQuestTeleporter` / `TicketTeleporter` add to a `Teleporter`: the `Type` as the hash a save holds, and the
/// `TextDefinition`.
#[derive(Clone, Debug, Default)]
pub struct QuestTeleporterData {
    /// `_questType` / `_ticketType`: `Write(Type)` writes the XXH3 hash of the full name.
    pub type_hash: Option<u64>,
    /// `_message`.
    pub message: Option<TextDefinition>,
}

fn qdata(w: &World, id: ItemId) -> QuestTeleporterData {
    w.item(id)
        .and_then(|i| i.ext.get::<QuestTeleporterData>())
        .cloned()
        .unwrap_or_default()
}

/// `MLQuestTeleporter(pointDest, mapDest, questType, message)` / `TicketTeleporter(..)` of class `ty`.
pub fn quest_teleporter_new(
    w: &mut World,
    ty: &'static ItemType,
    point_dest: Point3D,
    map_dest: Option<MapId>,
    type_full_name: Option<&str>,
    message: Option<TextDefinition>,
) -> ItemId {
    let id = tele::teleporter_new(w, ty, point_dest, map_dest, false);
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(QuestTeleporterData {
            type_hash: type_full_name.map(muo_primitives::hash::compute_hash64),
            message,
        });
    }
    id
}

fn serialize_quest_teleporter(
    parent: &'static ItemType,
    w: &World,
    id: ItemId,
    out: &mut dyn muo_io::GenericWriter,
) {
    (parent.serialize)(w, id, out);
    out.write_i32(1);
    let d = qdata(w, id);
    out.write_type_hash(d.type_hash);
    out.write_text_definition(d.message.as_ref());
}

fn deserialize_quest_teleporter(
    parent: &'static ItemType,
    w: &mut World,
    id: ItemId,
    r: &mut dyn muo_io::GenericReader,
) -> muo_io::Result<()> {
    (parent.deserialize)(w, id, r)?;
    let _version = r.read_i32()?;
    // Both versions read the same: `Deserialize(reader, version)` reads the type by its name, then the message.
    let type_hash = match r.read_type()? {
        None => None,
        Some(muo_io::serialization::TypeRef::Hash(h)) => Some(h),
        Some(muo_io::serialization::TypeRef::Name(n)) => {
            Some(muo_primitives::hash::compute_hash64(&n))
        }
    };
    let message = r.read_text_definition()?;
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(QuestTeleporterData { type_hash, message });
    }
    Ok(())
}

/// The class named by a saved hash among the item classes (`Type.Name` of an `IsInstanceOfType` test).
fn type_name_for_hash(w: &World, hash: u64) -> Option<String> {
    w.types
        .find_item_by_hash(hash)
        .map(|t| super::model::short_name(t.full_name).to_owned())
}

/// `_ticketType.IsInstanceOfType(item)`.
fn is_instance_of_hash(w: &World, item: ItemId, hash: u64) -> bool {
    let mut t = w.item(item).map(|i| i.ty);
    while let Some(ty) = t {
        if muo_primitives::hash::compute_hash64(ty.full_name) == hash {
            return true;
        }
        t = ty.parent;
    }
    false
}

fn mlq_can_teleport(w: &mut World, id: ItemId, m: MobileId) -> bool {
    if !(TELEPORTER_CLASS.can_teleport)(w, id, m) {
        return false;
    }
    let d = qdata(w, id);
    let Some(hash) = d.type_hash else { return true };
    if muo_content::player_mobile::data(w, m).is_none() {
        return false;
    }
    // `context?.IsDoingQuest(_questType) == true || context?.HasDoneQuest(_questType) == true`
    if let Some(q) = engine::find_quest_by_hash(w, hash)
        && (engine::is_doing_quest(w, m, q) || engine::has_done_quest(w, m, q).is_some())
    {
        return true;
    }
    muo_content::text::send_message_to(w, d.message.as_ref(), m);
    false
}

fn mlq_properties(w: &World, id: ItemId, list: &mut PropertyList) {
    (TELEPORTER_CLASS.add_properties)(w, id, list);
    if let Some(h) = qdata(w, id).type_hash {
        let name = engine::find_quest_by_hash(w, h)
            .map(|q| engine::quest(w, q).class.name().to_owned())
            .or_else(|| type_name_for_hash(w, h));
        if let Some(n) = name {
            list.add_text(&format!("Required quest: {n}"));
        }
    }
}

pub static MLQ_TELEPORTER_CLASS: TeleporterClass = TeleporterClass {
    can_teleport: mlq_can_teleport,
    add_properties: mlq_properties,
    ..TELEPORTER_CLASS
};

/// `Server.Engines.MLQuests.Items.MLQuestTeleporter`.
pub static ML_QUEST_TELEPORTER: ItemType = ItemType {
    full_name: "Server.Engines.MLQuests.Items.MLQuestTeleporter",
    parent: Some(&TELEPORTER),
    construct: Some(|w| {
        quest_teleporter_new(w, &ML_QUEST_TELEPORTER, Point3D::ZERO, None, None, None)
    }),
    family: Some(&MLQ_TELEPORTER_CLASS),
    serialize: |w, id, out| serialize_quest_teleporter(&TELEPORTER, w, id, out),
    deserialize: |w, id, r| deserialize_quest_teleporter(&TELEPORTER, w, id, r),
    ..TELEPORTER
};

/// `BoneMachete.OnTicketUsed(from)`, the only `ITicket`.
fn on_ticket_used(w: &mut World, ticket: ItemId, from: MobileId) {
    if !engine::item_is(w, ticket, "BoneMachete") {
        return;
    }
    muo_items::weapons::special::bone_machete_on_ticket_used(w, ticket, from);
}

fn ticket_can_teleport(w: &mut World, id: ItemId, m: MobileId) -> bool {
    if !(TELEPORTER_CLASS.can_teleport)(w, id, m) {
        return false;
    }
    let d = qdata(w, id);
    let Some(hash) = d.type_hash else { return true };

    // `m.Backpack?.FindItemByType(_ticketType, false) ?? m.Items.Find(item => _ticketType.IsInstanceOfType(item))`
    let pack = muo_server::mobile::part1::props_b::backpack(w, m);
    let mut ticket = pack.and_then(|p| {
        w.item(p).and_then(|p| {
            p.items
                .iter()
                .copied()
                .find(|&i| is_instance_of_hash(w, i, hash))
        })
    });
    if ticket.is_none() {
        ticket = w.mobile(m).and_then(|mm| {
            mm.items
                .iter()
                .copied()
                .find(|&i| is_instance_of_hash(w, i, hash))
        });
    }
    let Some(ticket) = ticket else {
        muo_content::text::send_message_to(w, d.message.as_ref(), m);
        return false;
    };
    on_ticket_used(w, ticket, m);
    true
}

fn ticket_properties(w: &World, id: ItemId, list: &mut PropertyList) {
    (TELEPORTER_CLASS.add_properties)(w, id, list);
    if let Some(h) = qdata(w, id).type_hash
        && let Some(n) = type_name_for_hash(w, h)
    {
        list.add_text(&format!("Required ticket: {n}"));
    }
}

pub static TICKET_TELEPORTER_CLASS: TeleporterClass = TeleporterClass {
    can_teleport: ticket_can_teleport,
    add_properties: ticket_properties,
    ..TELEPORTER_CLASS
};

/// `Server.Engines.MLQuests.Items.TicketTeleporter`.
pub static TICKET_TELEPORTER: ItemType = ItemType {
    full_name: "Server.Engines.MLQuests.Items.TicketTeleporter",
    parent: Some(&TELEPORTER),
    construct: Some(|w| {
        quest_teleporter_new(w, &TICKET_TELEPORTER, Point3D::ZERO, None, None, None)
    }),
    family: Some(&TICKET_TELEPORTER_CLASS),
    serialize: |w, id, out| serialize_quest_teleporter(&TELEPORTER, w, id, out),
    deserialize: |w, id, r| deserialize_quest_teleporter(&TELEPORTER, w, id, r),
    ..TELEPORTER
};

/// Registers the item classes of this file.
pub fn register(types: &mut TypeRegistry) {
    for t in [
        &AN_OLD_NECKLACE,
        &AN_OLD_RING,
        &REGINAS_RING,
        &TRANSPARENT_HEART,
        &ANDROS_GRATITUDE,
        &BLEACH,
        &MINIATURE_MUSHROOM,
        &PEPPERCORN_FISHSTEAK,
        &PIXIE_LEG,
        &RED_LEATHER_BOOK,
        &PRISMATIC_AMBER,
        &GUSTAR_SHROUD,
        &GIFT_FOR_ARIELLE,
        &PRISMATIC_CRYSTAL,
        &SMALL_BAG_OF_TRINKETS,
        &BAG_OF_TRINKETS,
        &BAG_OF_TREASURE,
        &LARGE_BAG_OF_TREASURE,
        &REWARD_STRONGBOX,
        &BASE_CRAFTMANS_SATCHEL,
        &TAILOR_SATCHEL,
        &BLACKSMITH_SATCHEL,
        &TINKER_SATCHEL,
        &FLETCHING_SATCHEL,
        &CARPENTRY_SATCHEL,
        &ML_QUEST_TELEPORTER,
        &TICKET_TELEPORTER,
    ] {
        types.register_item(t, &[]);
    }
}

/// `QuestType` / `TicketType` as a property: the `Type` the saved hash names (null when no class has that hash).
pub fn type_prop_get(w: &World, id: ItemId, quest: bool) -> muo_content::props::Value {
    use muo_content::props::Value;
    let Some(h) = qdata(w, id).type_hash else {
        return Value::Null;
    };
    let full = if quest {
        engine::find_quest_by_hash(w, h).map(|q| engine::quest(w, q).class.full_name)
    } else {
        w.types.find_item_by_hash(h).map(|t| t.full_name)
    };
    full.map_or(Value::Null, |n| Value::Text(n.to_owned()))
}

/// `QuestType = value` / `TicketType = value` (`[InvalidateProperties]`); the type is kept as the hash of its full name.
pub fn type_prop_set(
    w: &mut World,
    id: ItemId,
    quest: bool,
    v: &muo_content::props::Value,
) -> Result<(), String> {
    let hash = match v.string() {
        None => None,
        Some(s) => {
            let full = if quest {
                engine::find_quest_by_full_name(w, &s)
                    .or_else(|| engine::find_quest(w, &s))
                    .map(|q| engine::quest(w, q).class.full_name)
            } else {
                muo_content::props::class_by_name(w, &s).filter(|n| {
                    w.types
                        .find_item_by_hash(muo_primitives::hash::compute_hash64(n))
                        .is_some()
                })
            };
            Some(muo_primitives::hash::compute_hash64(
                full.ok_or("No type with that name was found.")?,
            ))
        }
    };
    if let Some(d) = w
        .item_mut(id)
        .and_then(|i| i.ext.get_mut::<QuestTeleporterData>())
    {
        d.type_hash = hash;
    }
    muo_server::item::ops::invalidate_properties(w, id);
    Ok(())
}
