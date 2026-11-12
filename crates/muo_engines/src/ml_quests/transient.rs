//! `QuestGiverItem`, `TransientQuestGiverItem` and the `TransientItem` quest items:
//! APersonalLetterAddressedToAhie, CrateForSledge, SpeckledPoisonSac and the Lost Items BatteredBucket.
//! `TransientItem` itself is `muo_spells::spellweaving::items`.

use muo_primitives::{MessageType, TimeSpan};
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, LootType, ops};
use muo_server::mobile::part2::messages as mm;
use muo_server::property_list::PropertyList;
use muo_server::type_registry::TypeRegistry;
use muo_spells::spellweaving::items::{TRANSIENT_ITEM, init_transient};

use super::engine;
use super::item_support::add_quest_item_property;
use super::quester::{Quester, quests_of};

/// `TransientItem(itemID, lifeSpan)` of class `ty`.
pub fn transient_new(
    w: &mut World,
    ty: &'static ItemType,
    item_id: i32,
    life_span: TimeSpan,
) -> ItemId {
    let id = ops::new_item_with_id(w, ty, item_id);
    init_transient(w, id, life_span);
    id
}

// ---- QuestGiverItem --------------------------------------------------------------------------------------

/// The `IQuestGiver` half both classes share: `AddNameProperties`.
fn quest_giver_name_properties(w: &World, id: ItemId, list: &mut PropertyList) {
    add_quest_item_property(w, id, list);
    if !quests_of(w, Quester::Item(id)).is_empty() {
        list.add(1072269); // Quest Giver
    }
}

/// `OnDoubleClick`, the same in both classes.
fn quest_giver_double_click(w: &mut World, id: ItemId, from: MobileId) {
    let loc = ops::get_world_location(w, id);
    let in_range = w
        .mobile(from)
        .is_some_and(|m| (m.ty.in_range_point3d)(w, from, loc, 2));
    if !in_range {
        mm::local_overhead_message_number(w, from, MessageType::REGULAR, 0x3B2, 1019045, ""); // I can't reach that.
    } else if !{
        let pack = muo_server::mobile::part1::props_b::backpack(w, from)
            .map(muo_server::ids::EntityId::Item);
        ops::is_child_of(w, id, pack)
    } {
        mm::send_localized_message(w, from, 1042593, "", 0x3B2); // That is not in your backpack.
    } else if engine::enabled(w)
        && !quests_of(w, Quester::Item(id)).is_empty()
        && muo_content::player_mobile::data(w, from).is_some()
    {
        engine::on_double_click(w, Quester::Item(id), from);
    }
}

fn quest_giver_after_delete(w: &mut World, id: ItemId) {
    (ITEM_BASE.on_after_delete)(w, id);
    if engine::enabled(w) {
        engine::handle_quester_deletion(w, Quester::Item(id));
    }
}

/// `[SerializationGenerator(0, false)] abstract QuestGiverItem : Item, IQuestGiver`.
pub static QUEST_GIVER_ITEM: ItemType = ItemType {
    full_name: "Server.Engines.MLQuests.Items.QuestGiverItem",
    parent: Some(&ITEM_BASE),
    construct: None,
    nontransferable: |_, _| true,
    add_name_properties: |w, id, list| {
        (ITEM_BASE.add_name_properties)(w, id, list);
        quest_giver_name_properties(w, id, list);
    },
    on_double_click: quest_giver_double_click,
    on_after_delete: quest_giver_after_delete,
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        r.read_i32()?;
        Ok(())
    },
    ..ITEM_BASE
};

/// `[SerializationGenerator(0, false)] abstract TransientQuestGiverItem : TransientItem, IQuestGiver`.
pub static TRANSIENT_QUEST_GIVER_ITEM: ItemType = ItemType {
    full_name: "Server.Engines.MLQuests.Items.TransientQuestGiverItem",
    parent: Some(&TRANSIENT_ITEM),
    construct: None,
    nontransferable: |_, _| true,
    // HandleInvalidTransfer(from) is empty
    handle_invalid_transfer: |_, _, _| {},
    add_name_properties: |w, id, list| {
        (ITEM_BASE.add_name_properties)(w, id, list);
        quest_giver_name_properties(w, id, list);
    },
    on_double_click: quest_giver_double_click,
    on_after_delete: quest_giver_after_delete,
    serialize: |w, id, out| {
        (TRANSIENT_ITEM.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (TRANSIENT_ITEM.deserialize)(w, id, r)?;
        r.read_i32()?;
        Ok(())
    },
    ..TRANSIENT_ITEM
};

// ---- the concrete classes --------------------------------------------------------------------------------

/// One concrete class derived from `TransientItem` (or `TransientQuestGiverItem`): "parent part + version int 0".
macro_rules! transient_leaf {
    (
        $(#[$m:meta])*
        $name:ident = $full:literal : $parent:ident, graphic $gid:expr, life $life:expr, blessed;
        $($f:ident : $v:expr,)*
    ) => {
        $(#[$m])*
        pub static $name: ItemType = ItemType {
            full_name: $full,
            parent: Some(&$parent),
            construct: Some(|w| {
                let id = transient_new(w, &$name, $gid, $life);
                ops::set_loot_type(w, id, LootType::Blessed);
                id
            }),
            $($f: $v,)*
            serialize: |w, id, out| {
                ($parent.serialize)(w, id, out);
                out.write_i32(0);
            },
            deserialize: |w, id, r| {
                ($parent.deserialize)(w, id, r)?;
                r.read_i32()?;
                Ok(())
            },
            ..$parent
        };
    };
}

transient_leaf! {
    /// `Server.Items.APersonalLetterAddressedToAhie`.
    A_PERSONAL_LETTER_ADDRESSED_TO_AHIE = "Server.Items.APersonalLetterAddressedToAhie" : TRANSIENT_ITEM,
        graphic 0x14ED, life TimeSpan::from_mins(30), blessed;
    label_number: |_, _| 1073128, // A personal letter addressed to: Ahie
    add_name_properties: |w, id, list| {
        (ITEM_BASE.add_name_properties)(w, id, list);
        add_quest_item_property(w, id, list);
    },
}

transient_leaf! {
    /// `Server.Items.CrateForSledge`.
    CRATE_FOR_SLEDGE = "Server.Items.CrateForSledge" : TRANSIENT_ITEM,
        graphic 0x1FFF, life TimeSpan::from_hours(1), blessed;
    label_number: |_, _| 1074520, // Crate for Sledge
    add_name_properties: |w, id, list| {
        (ITEM_BASE.add_name_properties)(w, id, list);
        add_quest_item_property(w, id, list);
    },
}

transient_leaf! {
    /// `Server.Items.SpeckledPoisonSac`.
    SPECKLED_POISON_SAC = "Server.Items.SpeckledPoisonSac" : TRANSIENT_ITEM,
        graphic 0x23A, life TimeSpan::from_hours(1), blessed;
    label_number: |_, _| 1073133, // Speckled Poison Sac
}

transient_leaf! {
    /// `Server.Engines.MLQuests.Definitions.BatteredBucket`.
    BATTERED_BUCKET = "Server.Engines.MLQuests.Definitions.BatteredBucket" : TRANSIENT_QUEST_GIVER_ITEM,
        graphic 0x2004, life TimeSpan::from_mins(10), blessed;
    // Original label, doesn't fit the expiration message well: LabelNumber 1073129
    default_name: |_, _| Some("battered bucket".to_string()),
}

/// Registers the item classes of this file.
pub fn register(types: &mut TypeRegistry) {
    types.register_item(&QUEST_GIVER_ITEM, &[]);
    types.register_item(&TRANSIENT_QUEST_GIVER_ITEM, &[]);
    types.register_item(&A_PERSONAL_LETTER_ADDRESSED_TO_AHIE, &[]);
    types.register_item(&CRATE_FOR_SLEDGE, &[]);
    types.register_item(&SPECKLED_POISON_SAC, &[]);
    types.register_item(&BATTERED_BUCKET, &[]);
}
