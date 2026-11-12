//! Startup cleanup: 2.5 s after start, items nobody can reach (on `Map.Internal` at the origin, without a
//! parent, and of a class that is known to leak) and orphaned bank boxes are deleted, and worn hair items become
//! the virtual hair.

use muo_content::items::is_named;
use muo_primitives::{Point3D, TimeSpan};
use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::layer::Layer;
use muo_server::map::MapId;
use muo_server::mobile::AccessLevel;

/// `Cleanup.Initialize`: `Timer.StartTimer(TimeSpan.FromSeconds(2.5), Run)`.
pub fn initialize(w: &mut World) {
    muo_server::timer::start(w, TimeSpan::from_millis(2500), TimeSpan::default(), 1, run);
}

/// `Cleanup.Run`.
pub fn run(w: &mut World) {
    let mut items: Vec<ItemId> = Vec::new();
    let mut valid: Vec<ItemId> = Vec::new();
    let mut hair_cleanup: Vec<MobileId> = Vec::new();
    let mut boxes = 0;

    let ids: Vec<ItemId> = w.items.keys().copied().collect();
    for id in ids {
        let Some(item) = w.item(id) else { continue };
        let ty = item.ty;
        if item.map.is_none() {
            items.push(id);
            continue;
        }
        if is_named(ty, "Server.Items.CommodityDeed") {
            if let Some(c) = muo_items::deeds::deed_classes::commodity_of(w, id) {
                valid.push(c);
            }
            continue;
        }
        if is_named(ty, "Server.Multis.BaseHouse") {
            let h = muo_multis::house::d(w, id);
            for r in &h.relocated_entities {
                if let EntityId::Item(i) = r.entity {
                    valid.push(i);
                }
            }
            for v in &h.vendor_inventories {
                valid.extend(v.items.iter().copied());
            }
        } else if muo_server::item::containers::is_bank_box(w, id) {
            if muo_server::item::containers::owner(w, id).is_none() || item.items.is_empty() {
                items.push(id);
                boxes += 1;
            }
            continue;
        } else if matches!(item.layer, Layer::Hair | Layer::FacialHair)
            && let Some(EntityId::Mobile(root)) = muo_server::item::ops::root_parent(w, id)
        {
            if item.parent != Some(EntityId::Mobile(root))
                && w.mobile(root)
                    .is_some_and(|m| m.access_level == AccessLevel::Player)
            {
                items.push(id);
                continue;
            }
            if item.parent == Some(EntityId::Mobile(root)) {
                hair_cleanup.push(root);
                continue;
            }
        }
        if item.parent.is_some() || item.map != Some(MapId::INTERNAL) || item.held_by.is_some() {
            continue;
        }
        if item.location != Point3D::ZERO {
            continue;
        }
        if !is_buggable(w, id) {
            continue;
        }
        items.push(id);
    }

    for v in &valid {
        items.retain(|i| i != v);
    }

    if !items.is_empty() {
        if boxes > 0 {
            tracing::info!(
                "Cleanup: Detected {} inaccessible items, including {} bank boxes, removing..",
                items.len(),
                boxes
            );
        } else {
            tracing::info!(
                "Cleanup: Detected {} inaccessible items, removing..",
                items.len()
            );
        }
        for i in items {
            muo_server::item::base::delete(w, i);
        }
    }
    if !hair_cleanup.is_empty() {
        tracing::info!(
            "Cleanup: Detected {} hair and facial hair items being worn, converting to their virtual counterparts..",
            hair_cleanup.len()
        );
        for m in hair_cleanup {
            muo_server::mobile::part2::items::convert_hair(w, m);
        }
    }
}

/// `Cleanup.IsBuggable`.
pub fn is_buggable(w: &World, id: ItemId) -> bool {
    let Some(ty) = w.item(id).map(|i| i.ty) else {
        return false;
    };
    if is_named(ty, "Server.Items.Fists") {
        return false;
    }
    (is_named(ty, "Server.Items.BaseJewel") && muo_content::core::aos(w))
        || muo_items::commodity::deedable(w, id).is_some()
        || [
            "Server.Multis.BaseBoat",
            "Server.Items.Fish",
            "Server.Items.BigFish",
            "Server.Items.Food",
            "Server.Items.CookableFood",
            "Server.Items.SpecialFishingNet",
            "Server.Items.BaseMagicFish",
            "Server.Items.Shoes",
            "Server.Items.Sandals",
            "Server.Items.Boots",
            "Server.Items.ThighBoots",
            "Server.Items.TreasureMap",
            "Server.Items.MessageInABottle",
            "Server.Items.BaseArmor",
            "Server.Items.BaseWeapon",
            "Server.Items.BaseClothing",
            "Server.Items.SkullPole",
            "Server.Items.EvilIdolSkull",
            "Server.Items.MonsterStatuette",
            "Server.Items.Pier",
            "Server.Items.ArtifactLargeVase",
            "Server.Items.ArtifactVase",
            "Server.Items.MinotaurStatueDeed",
            "Server.Items.SwampTile",
            "Server.Items.WallBlood",
            "Server.Items.TatteredAncientMummyWrapping",
            "Server.Items.LavaTile",
            "Server.Items.DemonSkull",
            "Server.Items.Web",
            "Server.Items.WaterTile",
            "Server.Items.WindSpirit",
            "Server.Items.DirtPatch",
            "Server.Items.Futon",
        ]
        .iter()
        .any(|n| is_named(ty, n))
}

#[cfg(test)]
mod tests {
    use super::*;
    use muo_server::item::{ITEM_BASE, ItemType};

    fn class(name: &'static str) -> &'static ItemType {
        Box::leak(Box::new(ItemType {
            full_name: name,
            parent: Some(&ITEM_BASE),
            ..ITEM_BASE
        }))
    }

    #[test]
    fn a_stray_known_leak_goes_and_other_items_stay() {
        let mut w = muo_server::map::test_support::world();
        let fish = w.new_item(class("Server.Items.Fish"));
        let plain = w.new_item(class("Server.Items.Foo"));
        run(&mut w);
        assert!(
            w.item(fish).is_none(),
            "an unowned Fish on the internal map is removed"
        );
        assert!(w.item(plain).is_some());
    }
}
