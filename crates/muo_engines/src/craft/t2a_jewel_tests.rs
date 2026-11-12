//! T2A jewel gem crafting: `BaseJewel.OnCraft` against the craft context's pending
//! gem (`GemSelectTarget` sets it; the craft spends it).

use muo_content::items::equip_hooks::CraftArgs;
use muo_content::items::jewel::{self, GemType};
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::ops;
use muo_server::mobile::MOBILE_BASE;

use super::*;

fn world() -> World {
    let mut w = World::new();
    muo_content::register(&mut w.types);
    muo_items::register(&mut w.types);
    crate::register(&mut w.types);
    // T2A jewelry takes the gems from the crafter's backpack
    muo_content::items::equip_hooks::hooks_mut(&mut w).consume_gems =
        Some(muo_items::gems::consume_gems);
    w
}

fn crafter(w: &mut World) -> (MobileId, ItemId) {
    let m = w.new_mobile(&MOBILE_BASE);
    let pack = muo_content::factory::make_item(w, "Server.Items.Backpack").unwrap();
    muo_server::mobile::part2::items::add_item(w, m, pack);
    w.mobile_mut(m).unwrap().backpack = Some(pack);
    (m, pack)
}

fn stack(w: &mut World, pack: ItemId, name: &str, amount: i32) {
    let item = muo_content::factory::make_item(w, name).unwrap();
    ops::set_amount(w, item, amount);
    muo_server::item::container::drop_item_default(w, pack, item);
}

fn amount_of(w: &World, pack: ItemId, name: &str) -> i32 {
    w.item(pack)
        .unwrap()
        .items
        .iter()
        .filter(|&&i| muo_content::factory::item_is(w, i, name))
        .map(|&i| w.item(i).unwrap().amount)
        .sum()
}

/// `item.OnCraft(1, false, player, system, typeof(IronIngot), null, recipe, 0)` with the pending gem in the context.
fn craft_ring(
    w: &mut World,
    m: MobileId,
    pending: (GemType, i32),
) -> (ItemId, Option<(GemType, i32)>) {
    set_context(
        w,
        "Tinkering",
        m,
        CraftContext {
            pending_gem: Some(pending),
            ..CraftContext::default()
        },
    );
    let ring = muo_content::factory::make_item(w, "Server.Items.GoldRing").unwrap();
    let mut args = CraftArgs::new(1, false, m, muo_content::craft::CraftResource::Iron);
    args.craft_system = "Tinkering";
    args.pending_gem = context_of(w, "Tinkering", m).and_then(|c| c.pending_gem);
    on_craft(w, ring, &args);
    (
        ring,
        context_of(w, "Tinkering", m).and_then(|c| c.pending_gem),
    )
}

#[test]
fn on_craft_consumes_entire_targeted_gem_stack_and_names_by_count() {
    let mut w = world();
    let (player, pack) = crafter(&mut w);
    stack(&mut w, pack, "Server.Items.IronIngot", 10);
    stack(&mut w, pack, "Server.Items.Diamond", 50); // a stack of 50 diamonds

    let (ring, pending) = craft_ring(&mut w, player, (GemType::Diamond, 50));

    assert_eq!(amount_of(&w, pack, "Server.Items.Diamond"), 0); // all 50 consumed
    let d = jewel::data(&w, ring).unwrap();
    assert_eq!((d.gem_type, d.gem_count), (GemType::Diamond, 50));
    // pending state cleared so the next craft starts fresh
    assert_eq!(pending, None);
}

#[test]
fn on_craft_when_gems_unavailable_at_craft_time_crafts_plain_piece() {
    let mut w = world();
    let (player, pack) = crafter(&mut w);
    stack(&mut w, pack, "Server.Items.IronIngot", 10);
    // deliberately do NOT add any diamonds

    let (ring, pending) = craft_ring(&mut w, player, (GemType::Diamond, 5));

    let d = jewel::data(&w, ring).unwrap();
    assert_eq!((d.gem_type, d.gem_count), (GemType::None, 0));
    assert_eq!(pending, None);
}
