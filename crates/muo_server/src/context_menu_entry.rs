//! Context menus (`ContextMenu`, `ContextMenuEntry`).

use std::fmt;
use std::rc::Rc;

use crate::ids::{EntityId, MobileId};
use crate::packets::menu::CmeFlags;
use crate::world::World;

/// `OnClick(Mobile from, IEntity target)`.
pub type ContextClick = Rc<dyn Fn(&mut World, MobileId, EntityId)>;

/// `ContextMenuEntry`. A subclass is an entry with `on_click` (and `non_local_use`) set.
#[derive(Clone)]
pub struct ContextMenuEntry {
    pub flags: CmeFlags,
    pub number: i32,
    /// Tiles; -1 is no maximum.
    pub range: i32,
    /// A1-R5-G5-B5.
    pub color: i32,
    pub enabled: bool,
    /// `NonLocalUse`.
    pub non_local_use: bool,
    pub on_click: Option<ContextClick>,
}

impl fmt::Debug for ContextMenuEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ContextMenuEntry")
            .field("number", &self.number)
            .field("range", &self.range)
            .field("enabled", &self.enabled)
            .finish_non_exhaustive()
    }
}

impl ContextMenuEntry {
    /// `new ContextMenuEntry(number, range)`; the usual default range is -1.
    pub fn new(number: i32, range: i32) -> ContextMenuEntry {
        ContextMenuEntry {
            flags: CmeFlags::NONE,
            // Legacy code support.
            number: if number <= 0x7FFF {
                3_000_000 + number
            } else {
                number
            },
            range,
            color: 0xFFFF,
            enabled: true,
            non_local_use: false,
            on_click: None,
        }
    }

    /// `OnClick(from, target)`: the base method does nothing.
    pub fn click(&self, w: &mut World, from: MobileId, target: EntityId) {
        if let Some(f) = self.on_click.clone() {
            f(w, from, target);
        }
    }
}

/// `ContextMenu`.
#[derive(Clone, Debug)]
pub struct ContextMenu {
    pub from: MobileId,
    pub target: EntityId,
    pub entries: Vec<ContextMenuEntry>,
    pub requires_new_packet: bool,
}

impl ContextMenu {
    pub fn new(from: MobileId, target: EntityId, entries: Vec<ContextMenuEntry>) -> ContextMenu {
        let requires_new_packet = entries
            .iter()
            .any(|e| !(3_000_000..=3_032_767).contains(&e.number));
        ContextMenu {
            from,
            target,
            entries,
            requires_new_packet,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_numbers_are_shifted() {
        assert_eq!(ContextMenuEntry::new(0x7FFF, -1).number, 3_032_767);
        assert_eq!(ContextMenuEntry::new(0x8000, -1).number, 0x8000);
    }

    #[test]
    fn new_packet_is_required_outside_the_legacy_range() {
        let m = MobileId(muo_primitives::Serial::new(1));
        let t = EntityId::Mobile(m);
        let old = ContextMenu::new(m, t, vec![ContextMenuEntry::new(3_000_001, -1)]);
        let new = ContextMenu::new(m, t, vec![ContextMenuEntry::new(3_032_768, -1)]);
        assert!(!old.requires_new_packet);
        assert!(new.requires_new_packet);
    }
}
