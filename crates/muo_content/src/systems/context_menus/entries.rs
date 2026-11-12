//! The context menu entry classes: AddToParty, AddToSpellbook, Eat, EjectPlayer, OpenBackpack, OpenBank,
//! Paperdoll and Teach.

use std::rc::Rc;

use muo_server::World;
use muo_server::context_menu_entry::ContextMenuEntry;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::mobile::base as mb;
use muo_server::mobile::part2::{messages, stats};
use muo_server::skills::SkillName;
use muo_server::target::{self, TargetFlags, Targeted};

use super::ContextMenuHooks;

fn hooks(w: &mut World) -> &ContextMenuHooks {
    w.resources.get_mut::<ContextMenuHooks>()
}

fn send(w: &mut World, to: MobileId, number: i32) {
    messages::send_localized_message(w, to, number, "", 0x3B2);
}

fn entry(
    number: i32,
    range: i32,
    click: impl Fn(&mut World, MobileId, EntityId) + 'static,
) -> ContextMenuEntry {
    let mut e = ContextMenuEntry::new(number, range);
    e.on_click = Some(Rc::new(click));
    e
}

/// `AddToPartyEntry`.
pub fn add_to_party() -> ContextMenuEntry {
    entry(197, 12, |w, from, target| {
        let EntityId::Mobile(target) = target else {
            return;
        };
        let (get, capacity) = {
            let h = hooks(w);
            (h.party_get, h.party_capacity.unwrap_or(10))
        };
        let p = get.and_then(|f| f(w, from));
        let mp = get.and_then(|f| f(w, target));

        if from == target {
            send(w, from, 1005439); // You cannot add yourself to a party.
        } else if p.is_some_and(|p| p.leader != from) {
            send(w, from, 1005453); // You may only add members to the party if you are the leader.
        } else if p.is_some_and(|p| p.members + p.candidates >= capacity) {
            send(w, from, 1008095); // You may only have 10 in your party (this includes candidates).
        } else if !w.mobile(target).is_some_and(|m| m.player) {
            send(w, from, 1005444); // The creature ignores your offer.
        } else if mp.is_some() && mp == p {
            send(w, from, 1005440); // This person is already in your party!
        } else if mp.is_some() {
            send(w, from, 1005441); // This person is already in a party!
        } else if let Some(invite) = hooks(w).party_invite {
            invite(w, from, target);
        }
    })
}

/// `AddToSpellbookEntry`: `Number` 6144 is the legacy range's top, so it keeps the 3,000,000 offset.
pub fn add_to_spellbook() -> ContextMenuEntry {
    entry(6144, 3, |w, from, target| {
        let EntityId::Item(scroll) = target else {
            return;
        };
        if !stats::check_alive(w, from, true) {
            return;
        }
        let is_scroll = hooks(w).is_spell_scroll.is_some_and(|f| f(w, scroll));
        if !is_scroll {
            return;
        }
        // InternalTarget: range 3, no ground, no flags.
        target::begin_target(
            w,
            from,
            3,
            false,
            TargetFlags::NONE,
            move |w, from, targeted| {
                if let Targeted::Entity(EntityId::Item(book)) = targeted
                    && let Some(add) = hooks(w).add_scroll_to_book
                {
                    add(w, from, scroll, book);
                }
            },
        );
    })
}

/// `EatEntry`.
pub fn eat() -> ContextMenuEntry {
    entry(6135, 1, |w, from, target| {
        let EntityId::Item(food) = target else { return };
        if !stats::check_alive(w, from, true) {
            return;
        }
        let (is_food, eat) = {
            let h = hooks(w);
            (h.is_food, h.food_eat)
        };
        if !is_food.is_some_and(|f| f(w, food)) {
            return;
        }
        let ok = w.item(food).is_some_and(|i| !i.deleted() && i.movable())
            && check_item_use(w, food, from);
        if ok && let Some(eat) = eat {
            eat(w, food, from);
        }
    })
}

/// `item.CheckItemUse(from)`.
fn check_item_use(w: &mut World, item: ItemId, from: MobileId) -> bool {
    let Some(ty) = w.item(item).map(|i| i.ty) else {
        return false;
    };
    (ty.check_item_use)(w, item, from, item)
}

/// `EjectPlayerEntry`.
pub fn eject_player() -> ContextMenuEntry {
    entry(6206, 12, |w, from, target| {
        let EntityId::Mobile(target) = target else {
            return;
        };
        if let Some(eject) = hooks(w).eject_from_house {
            eject(w, from, target);
        }
    })
}

/// `OpenBackpackEntry`.
pub fn open_backpack() -> ContextMenuEntry {
    entry(6145, -1, |w, from, _| {
        if let Some(pack) = w.mobile(from).and_then(|m| m.backpack) {
            mb::use_item(w, from, pack);
        }
    })
}

/// `OpenBankEntry`.
pub fn open_bank() -> ContextMenuEntry {
    entry(6105, 12, |w, from, target| {
        let EntityId::Mobile(banker) = target else {
            return;
        };
        if !stats::check_alive(w, from, true) {
            return;
        }
        if w.mobile(from).is_some_and(|m| m.criminal) {
            messages::say_number(w, banker, 500378, ""); // Thou art a criminal and cannot access thy bank box.
        } else if let Some(bank) = muo_server::mobile::part1::props_b::bank_box(w, from) {
            muo_server::item::containers::open(w, bank);
        }
    })
}

/// `PaperdollEntry`.
pub fn paperdoll() -> ContextMenuEntry {
    entry(6123, 18, |w, from, target| {
        let EntityId::Mobile(m) = target else { return };
        let Some(ty) = w.mobile(m).map(|x| x.ty) else {
            return;
        };
        if (ty.can_paperdoll_be_opened_by)(w, m, from) {
            (ty.display_paperdoll_to)(w, m, from);
        }
    })
}

/// `TeachEntry(skill, enabled)`: `Number` is 6000 + the skill.
pub fn teach(skill: SkillName, enabled: bool) -> ContextMenuEntry {
    let mut e = entry(6000 + skill as i32, -1, move |w, from, target| {
        if !stats::check_alive(w, from, true) {
            return;
        }
        let EntityId::Mobile(bc) = target else { return };
        if let Some(t) = hooks(w).creature_teach {
            t(w, bc, skill, from);
        }
    });
    e.enabled = enabled;
    e
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_and_ranges() {
        let e = paperdoll();
        assert_eq!((e.number, e.range), (3_006_123, 18));
        assert_eq!((open_bank().number, open_bank().range), (3_006_105, 12));
        assert_eq!(
            (open_backpack().number, open_backpack().range),
            (3_006_145, -1)
        );
        assert_eq!((eat().number, eat().range), (3_006_135, 1));
        assert_eq!(
            (add_to_spellbook().number, add_to_spellbook().range),
            (3_006_144, 3)
        );
        assert_eq!(
            (eject_player().number, eject_player().range),
            (3_006_206, 12)
        );
        // The cliloc offset 0197 is decimal 197, not octal.
        let t = teach(SkillName::Anatomy, false);
        assert_eq!(t.number, 3_006_001);
        assert!(!t.enabled);
    }
}
