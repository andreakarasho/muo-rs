//! The lumberjacking harvest system, and the lumberjacking branches of `HarvestTarget`.

use muo_io::text::TextDefinition as Td;
use muo_primitives::TimeSpan;
use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{deps, ops};
use muo_server::skills::SkillName;
use muo_server::target::Targeted;

use super::*;

const TREE_TILES: &[i32] = &[
    0x0CCA, 0x0CCB, 0x0CCC, 0x0CCD, 0x0CD0, 0x0CD3, 0x0CD6, 0x0CD8, 0x0CDA, 0x0CDD, 0x0CE0, 0x0CE3,
    0x0CE6, 0x0CF8, 0x0CFB, 0x0CFE, 0x0D01, 0x0D41, 0x0D42, 0x0D43, 0x0D44, 0x0D57, 0x0D58, 0x0D59,
    0x0D5A, 0x0D5B, 0x0D6E, 0x0D6F, 0x0D70, 0x0D71, 0x0D72, 0x0D84, 0x0D85, 0x0D86, 0x12B5, 0x12B6,
    0x12B7, 0x12B8, 0x12B9, 0x12BA, 0x12BB, 0x12BC, 0x12BD, 0x0CCE, 0x0CCF, 0x0CD1, 0x0CD2, 0x0CD4,
    0x0CD5, 0x0CD7, 0x0CD9, 0x0CDB, 0x0CDC, 0x0CDE, 0x0CDF, 0x0CE1, 0x0CE2, 0x0CE4, 0x0CE5, 0x0CE7,
    0x0CE8, 0x0CF9, 0x0CFA, 0x0CFC, 0x0CFD, 0x0CFF, 0x0D00, 0x0D02, 0x0D03, 0x0D45, 0x0D46, 0x0D47,
    0x0D48, 0x0D49, 0x0D4A, 0x0D4B, 0x0D4C, 0x0D4D, 0x0D4E, 0x0D4F, 0x0D50, 0x0D51, 0x0D52, 0x0D53,
    0x0D5C, 0x0D5D, 0x0D5E, 0x0D5F, 0x0D60, 0x0D61, 0x0D62, 0x0D63, 0x0D64, 0x0D65, 0x0D66, 0x0D67,
    0x0D68, 0x0D69, 0x0D73, 0x0D74, 0x0D75, 0x0D76, 0x0D77, 0x0D78, 0x0D79, 0x0D7A, 0x0D7B, 0x0D7C,
    0x0D7D, 0x0D7E, 0x0D7F, 0x0D87, 0x0D88, 0x0D89, 0x0D8A, 0x0D8B, 0x0D8C, 0x0D8D, 0x0D8E, 0x0D8F,
    0x0D90, 0x0D95, 0x0D96, 0x0D97, 0x0D99, 0x0D9A, 0x0D9B, 0x0D9D, 0x0D9E, 0x0D9F, 0x0DA1, 0x0DA2,
    0x0DA3, 0x0DA5, 0x0DA6, 0x0DA7, 0x0DA9, 0x0DAA, 0x0DAB, 0x12BE, 0x12BF, 0x12C0, 0x12C1, 0x12C2,
    0x12C3, 0x12C4, 0x12C5, 0x12C6, 0x12C7,
];

/// `new Lumberjacking()`.
pub fn create(w: &mut World) -> HarvestSystem {
    let (aos, ml) = (muo_content::core::aos(w), muo_content::core::ml(w));
    let mut lumber = HarvestDefinition {
        bank_width: 4,
        bank_height: 3,
        min_total: 20,
        max_total: 45,
        min_respawn: TimeSpan::from_mins(20),
        max_respawn: TimeSpan::from_mins(30),
        skill: SkillName::Lumberjacking,
        land_tiles: Vec::new(),
        static_tiles: TREE_TILES.to_vec(),
        max_range: 2,
        consumed_per_harvest: 10,
        consumed_per_felucca_harvest: 20,
        effect_actions: vec![13],
        effect_sounds: vec![0x13E],
        effect_counts: if aos { vec![1] } else { vec![1, 2, 2, 2, 3] },
        effect_delay: TimeSpan::from_millis(1600),
        effect_sound_delay: TimeSpan::from_millis(900),
        no_resources_message: Td::of_number(500493), // There's not enough wood here to harvest.
        fail_message: Td::of_number(500495), // You hack at the tree for a while, but fail to produce any useable wood.
        out_of_range_message: Td::of_number(500446), // That is too far away.
        pack_full_message: Td::of_number(500497), // You can't place any wood into your backpack!
        tool_broke_message: Td::of_number(500499), // You broke your axe.
        ..Default::default()
    };
    let vein = |chance, fb_chance, primary, fallback| HarvestVein {
        vein_chance: chance,
        chance_to_fallback: fb_chance,
        primary,
        fallback,
    };
    if ml {
        lumber.resources = vec![
            HarvestResource::new(0.0, 0.0, 100.0, 1072540, &["Server.Items.Log"]),
            HarvestResource::new(65.0, 25.0, 105.0, 1072541, &["Server.Items.OakLog"]),
            HarvestResource::new(80.0, 40.0, 120.0, 1072542, &["Server.Items.AshLog"]),
            HarvestResource::new(95.0, 55.0, 135.0, 1072543, &["Server.Items.YewLog"]),
            HarvestResource::new(100.0, 60.0, 140.0, 1072544, &["Server.Items.HeartwoodLog"]),
            HarvestResource::new(100.0, 60.0, 140.0, 1072545, &["Server.Items.BloodwoodLog"]),
            HarvestResource::new(100.0, 60.0, 140.0, 1072546, &["Server.Items.FrostwoodLog"]),
        ];
        lumber.set_veins(vec![
            vein(490, 0.0, 0, None),    // Ordinary Logs
            vein(300, 0.5, 1, Some(0)), // Oak
            vein(100, 0.5, 2, Some(0)), // Ash
            vein(50, 0.5, 3, Some(0)),  // Yew
            vein(30, 0.5, 4, Some(0)),  // Heartwood
            vein(20, 0.5, 5, Some(0)),  // Bloodwood
            vein(10, 0.5, 6, Some(0)),  // Frostwood
        ]);
        let b = |req, chance, msg: i32, ty: Option<&'static str>| BonusResource {
            req_skill: req,
            chance,
            message: if msg == 0 {
                Td::EMPTY
            } else {
                Td::of_number(msg)
            },
            ty,
        };
        lumber.bonus_resources = Some(vec![
            b(0.0, 83.9, 0, None), // Nothing
            b(100.0, 10.0, 1072548, Some("Server.Items.BarkFragment")),
            b(100.0, 3.0, 1072550, Some("Server.Items.LuminescentFungi")),
            b(100.0, 2.0, 1072547, Some("Server.Items.SwitchItem")),
            b(100.0, 1.0, 1072549, Some("Server.Items.ParasiticPlant")),
            b(100.0, 0.1, 1072551, Some("Server.Items.BrilliantAmber")),
        ]);
    } else {
        lumber.resources = vec![HarvestResource::new(
            0.0,
            0.0,
            100.0,
            500498,
            &["Server.Items.Log"],
        )];
        lumber.set_veins(vec![vein(1000, 0.0, 0, None)]);
    }
    lumber.race_bonus = ml;
    lumber.randomize_veins = ml;
    HarvestSystem {
        kind: Kind::Lumberjacking,
        name: "Lumberjacking",
        definitions: vec![lumber],
    }
}

/// `CheckHarvest` (both overloads) beyond the base's tool check: the axe must be worn.
pub fn check_harvest(w: &mut World, from: MobileId, tool: ItemId) -> bool {
    if w.item(tool)
        .is_none_or(|i| i.parent != Some(EntityId::Mobile(from)))
    {
        deps::send_localized_message(w, from, 500487); // The axe must be equipped for any serious wood chopping.
        return false;
    }
    true
}

pub fn on_bad_harvest_target(w: &mut World, from: MobileId, to: ToHarvest) {
    match to {
        ToHarvest::Mobile(m) => {
            // You can only skin dead creatures.
            let ns = w.mobile(from).and_then(|x| x.net_state);
            muo_server::mobile::part2::messages::private_overhead_message_number(
                w,
                m,
                muo_primitives::MessageType::REGULAR,
                0x3B2,
                500450,
                "",
                ns,
            );
        }
        ToHarvest::Item(i) | ToHarvest::Static(i) => {
            // Use this on corpses to carve away meat and hide
            ops::label_to_number(w, i, from, 500464, "");
        }
        ToHarvest::StaticTarget(_) | ToHarvest::Land(_) => {
            deps::send_localized_message(w, from, 500489); // You can't use an axe on that.
        }
    }
}

/// `HarvestTarget.OnTarget`'s lumberjacking branches (IChoppable, IAxe, ICarvable, furniture); true if handled.
pub fn on_target_special(w: &mut World, from: MobileId, tool: ItemId, targeted: Targeted) -> bool {
    let Targeted::Entity(EntityId::Item(item)) = targeted else {
        return false;
    };
    if super::state_hooks(w)
        .and_then(|h| h.chop)
        .is_some_and(|f| f(w, item, from))
    {
        return true;
    }
    // `targeted is IAxe obj && m_Tool is BaseAxe axe`
    if let Some(f) = super::state_hooks(w).and_then(|h| h.axe_target)
        && let Some(handled) = f(w, item, from, tool)
    {
        return handled;
    }
    // `targeted is ICarvable carvable` then `carvable.Carve(from, m_Tool)`
    let carve = w
        .resources
        .get::<muo_content::targets::TargetHooks>()
        .and_then(|h| h.carvable_carve);
    if carve.is_some_and(|f| f(w, EntityId::Item(item), from, tool)) {
        return true;
    }
    if super::state_hooks(w)
        .and_then(|h| h.is_furniture)
        .is_some_and(|f| f(w, item))
    {
        destroy_furniture(w, from, item);
        return true;
    }
    false
}

fn destroy_furniture(w: &mut World, from: MobileId, item: ItemId) {
    let loc = ops::get_world_location(w, item);
    if !mx::in_range_point(w, from, loc, 3) {
        deps::send_localized_message(w, from, 500446); // That is too far away.
        return;
    }
    let pack = deps::find_item_on_layer(w, from, muo_server::layer::Layer::Backpack);
    let in_pack = pack.is_some_and(|p| ops::is_child_of(w, item, Some(EntityId::Item(p))));
    if !in_pack && !w.item(item).is_some_and(|i| i.movable()) {
        deps::send_localized_message(w, from, 500462); // You can't destroy that while it is here.
        return;
    }
    deps::send_localized_message(w, from, 500461); // You destroy the item.
    let map = w.item(item).and_then(|i| i.map);
    muo_server::effects::play_sound(w, loc, map, 0x3B3);
    let Some(ty) = w.item(item).map(|i| i.ty) else {
        return;
    };
    if ty.is(&muo_server::item::container::CONTAINER) {
        if ty.is(&muo_content::items::cont::TRAPPABLE_CONTAINER) {
            let f = muo_content::items::cont::class(w, item).execute_trap;
            f(w, item, from);
        }
        (ty.destroy)(w, item);
    } else {
        (ty.delete)(w, item);
    }
}
