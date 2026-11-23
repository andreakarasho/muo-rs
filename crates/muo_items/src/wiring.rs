//! Hooks of `muo_content` whose targets are item classes of this crate (see `muo_content::wiring`).

use muo_content::loot_pack::LootHooks;
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::type_registry::{Phase, TypeRegistry};

use crate::skill_items::instruments;

/// `instrument.Slayer = SlayerGroup.GetLootSlayerType(creature type)`; a weapon goes to the content wiring.
fn pack_slayer(w: &mut World, item: ItemId, from: MobileId) {
    if instruments::is_instrument(w, item) {
        let name = w.mobile(from).map_or("", |m| m.ty.full_name);
        let s = muo_content::items::slayer::loot_slayer_type(w, name) as i32;
        if let Some(d) = instruments::data_mut(w, item) {
            d.slayer = s;
        }
    } else {
        muo_content::wiring::pack_slayer(w, item, from);
    }
}

pub fn configure(w: &mut World) {
    use muo_content::items::slayer::SlayerName;
    w.resources
        .get_mut::<muo_content::creature::packing::PackHooks>()
        .set_loot_slayer = Some(pack_slayer);
    // `LootPack` Mutate of a BaseInstrument: Regular quality and a slayer (none: the caller deletes it).
    w.resources.get_mut::<LootHooks>().apply_instrument = Some(|w, item, from| {
        let slayer = if w.core_aos() {
            muo_content::items::slayer::random_slayer(w)
        } else {
            let name = w.mobile(from).map_or("", |m| m.ty.full_name);
            muo_content::items::slayer::loot_slayer_type(w, name)
        };
        if slayer == SlayerName::None {
            return false;
        }
        if let Some(d) = instruments::data_mut(w, item) {
            d.quality = 1; // InstrumentQuality.Regular
            d.slayer = slayer as i32;
        }
        true
    });
    // `BaseRunicTool.ApplyAttributesTo(spellbook, count, min, max)` (a Magery 80+ crafter's book).
    w.resources
        .get_mut::<crate::skill_items::spellbook::SpellbookHooks>()
        .apply_runic = Some(|w, item, count, min, max| {
        muo_content::items::runic::apply_to_spellbook(w, item, false, 0, count, min, max)
    });
    w.resources
        .get_mut::<muo_content::special_systems::test_center::TestCenterHooks>()
        .make_special = Some(|w, s| {
        use muo_content::special_systems::test_center::Special;
        match s {
            Special::PotionKeg { effect, hue } => {
                crate::skill_items::potion_keg::new_test_keg(w, effect, hue)
            }
            Special::Runebook => {
                let book = crate::skill_items::runebook::new_runebook(w, 10);
                crate::skill_items::runebook::set_cur_charges(w, book, 10); // CurCharges = MaxCharges
                Some(book)
            }
            Special::UnredyableDyeTub { hue } => {
                Some(crate::skill_items::dyes::new_unredyable_tub(w, hue))
            }
            _ => None,
        }
    });
    muo_content::spells::hooks::hooks_mut(w).resolve_recall_source = Some(resolve_recall_source);
    let choppables = &mut muo_content::items::common::hooks_mut(w).choppables;
    choppables.push(muo_content::items::addon_container::chop_addon_family);
    choppables.push(crate::containers::strong_box::strong_box_chop);
    w.resources
        .get_mut::<muo_content::hooks::ContentHooks>()
        .random_talisman =
        Some(|w| crate::talismans::random_talisman_new(w, &crate::talismans::BASE_TALISMAN));
    w.resources
        .get_mut::<muo_content::hooks::ContentHooks>()
        .new_staff_robe =
        Some(|w, level| crate::suits::staff_robe_new(w, &crate::suits::STAFF_ROBE, level));
    let h = w.resources.get_mut::<muo_content::hooks::WorldHooks>();
    h.waypoint_next = Some(crate::misc::basic::way_point_next);
    w.resources
        .get_mut::<muo_content::hooks::DeathHooks>()
        .pack_treasure_map = Some(|w, id, level| {
        let map = w.mobile(id).and_then(|m| m.map);
        let item = crate::maps::treasure::treasure_map_new(
            w,
            &crate::maps::treasure::TREASURE_MAP,
            level,
            map,
        );
        muo_content::creature::setters::pack_item(w, id, Some(item));
    });
}

/// `RecallSpellTarget.OnTarget`'s `o is RecallRune / Runebook / Key (to a boat) / HouseRaffleDeed` chain.
fn resolve_recall_source(
    w: &mut World,
    e: muo_server::ids::EntityId,
    to_boat: bool,
) -> muo_content::spells::targeting::RecallSource {
    use crate::skill_items::runebook as rb;
    use muo_content::spells::targeting::RecallSource as R;
    let muo_server::ids::EntityId::Item(id) = e else {
        return R::Invalid;
    };
    let Some(ty) = w.item(id).map(|i| i.ty) else {
        return R::Invalid;
    };
    if ty.is(&rb::RECALL_RUNE) {
        return match rb::rune(w, id) {
            Some(r) if r.marked => R::Location {
                loc: r.target,
                map: r.target_map,
                check_multi: true,
            },
            _ => R::RuneNotMarked,
        };
    }
    if rb::is_runebook(w, id) {
        let Some(b) = rb::book(w, id) else {
            return R::Invalid;
        };
        return match usize::try_from(b.default_index)
            .ok()
            .and_then(|i| b.entries.get(i))
        {
            Some(en) => R::Location {
                loc: en.location,
                map: en.map,
                check_multi: true,
            },
            None => R::RunebookNoDefault,
        };
    }
    if to_boat
        && muo_content::items::gold::is_key(w, id)
        && muo_content::items::gold::key_value(w, id) != 0
        && let Some(boat) = muo_content::items::gold::key_link(w, id)
        && muo_multis::boats::is_boat(w, boat)
        && w.item(boat).is_some_and(|i| !i.deleted())
        && muo_multis::boats::check_key(w, boat, muo_content::items::gold::key_value(w, id))
    {
        let map = w.item(boat).and_then(|i| i.map);
        return R::Location {
            loc: muo_multis::boats::get_marked_location(w, boat),
            map,
            check_multi: false,
        };
    }
    if ty.is(&crate::special::raffle::HOUSE_RAFFLE_DEED)
        && let Some(d) = crate::special::raffle::deed(w, id)
        && d.plot_location != muo_primitives::Point3D::ZERO
        && d.plot_facet
            .is_some_and(|m| m != muo_server::map::MapId::INTERNAL)
    {
        return R::Location {
            loc: d.plot_location,
            map: d.plot_facet,
            check_multi: true,
        };
    }
    R::Invalid
}

pub fn register(types: &mut TypeRegistry) {
    types.register_phase_default(Phase::Configure, "muo_items.Wiring.Configure", configure);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skill_items::runebook as rb;
    use muo_content::spells::targeting::RecallSource as R;
    use muo_server::ids::EntityId;

    #[test]
    fn recall_resolves_runes_and_runebooks() {
        let mut w = crate::test_world();
        let rune = rb::new_rune(&mut w);
        assert_eq!(
            resolve_recall_source(&mut w, EntityId::Item(rune), true),
            R::RuneNotMarked
        );
        rb::set_marked(&mut w, rune, true);
        rb::set_target(&mut w, rune, muo_primitives::Point3D::new(5, 6, 7));
        assert!(matches!(
            resolve_recall_source(&mut w, EntityId::Item(rune), true),
            R::Location {
                check_multi: true,
                ..
            }
        ));
        let book = rb::new_runebook(&mut w, 6);
        assert_eq!(
            resolve_recall_source(&mut w, EntityId::Item(book), true),
            R::RunebookNoDefault
        );
        let other = (crate::skill_items::tools::ALL[0].construct.unwrap())(&mut w);
        assert_eq!(
            resolve_recall_source(&mut w, EntityId::Item(other), true),
            R::Invalid
        );
    }
}
