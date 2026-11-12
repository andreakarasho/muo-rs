//! Staff suits: `BaseSuit` (an `Item` on `Layer.OuterTorso` that only staff of its level can keep)
//! and its robes and suits. `AccessLevel` is `SuitData` in `Item::ext`; `OnAccessLevelChanged` is the one virtual
//! of the class, in [`SuitClass`].

use muo_io::{GenericReader, IoError, Result};
use muo_server::World;
use muo_server::ext::Ext;
use muo_server::ids::{EntityId, ItemId};
use muo_server::item::{ITEM_BASE, ItemType, LootType, base, deps, ops};
use muo_server::layer::Layer;
use muo_server::mobile::AccessLevel;
use muo_server::type_registry::TypeRegistry;

/// What `BaseSuit` adds to an `Item`.
#[derive(Clone, Debug)]
pub struct SuitData {
    pub access_level: AccessLevel,
}

/// The virtual of the suit classes that is not an `Item` virtual.
#[derive(Copy, Clone)]
pub struct SuitClass {
    /// `OnAccessLevelChanged(oldAccessLevel, accessLevel)`.
    pub on_access_level_changed: fn(&mut World, ItemId, AccessLevel, AccessLevel),
}

/// `BaseSuit`' defaults (the method is empty).
pub static BASE_SUIT_CLASS: SuitClass = SuitClass {
    on_access_level_changed: |_, _, _, _| {},
};

pub fn data(w: &World, id: ItemId) -> Option<&SuitData> {
    w.item(id)?.ext.get::<SuitData>()
}

fn class(w: &World, id: ItemId) -> &'static SuitClass {
    w.item(id)
        .and_then(|i| i.ty.family::<SuitClass>())
        .unwrap_or(&BASE_SUIT_CLASS)
}

/// `AccessLevel`.
pub fn access_level(w: &World, id: ItemId) -> AccessLevel {
    data(w, id).map_or(AccessLevel::Player, |d| d.access_level)
}

/// `AccessLevel` setter: invalidates properties and runs `OnAccessLevelChanged`.
pub fn set_access_level(w: &mut World, id: ItemId, value: AccessLevel) {
    let Some(d) = w.item_mut(id).and_then(|i| i.ext.get_mut::<SuitData>()) else {
        return;
    };
    let old = d.access_level;
    if old != value {
        d.access_level = value;
        ops::invalidate_properties(w, id);
        (class(w, id).on_access_level_changed)(w, id, old, value);
    }
}

/// `BaseSuit(level, hue, itemID)` for class `ty`.
pub fn new_suit(
    w: &mut World,
    ty: &'static ItemType,
    level: AccessLevel,
    hue: i32,
    item_id: i32,
) -> ItemId {
    let id = ops::new_item_with_id(w, ty, item_id);
    (ty.set_hue)(w, id, hue);
    ops::set_movable(w, id, false);
    ops::set_loot_type(w, id, LootType::Newbied);
    (ty.set_layer)(w, id, Layer::OuterTorso);
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(SuitData {
            access_level: level,
        });
    }
    id
}

/// `Validate()`: staff below the suit's level lose it.
pub fn validate(w: &mut World, id: ItemId) -> bool {
    let level = access_level(w, id);
    let Some(EntityId::Mobile(m)) = ops::root_parent(w, id) else {
        return true;
    };
    if w.mobile(m).is_none_or(|m| m.access_level >= level) {
        return true;
    }
    if let Some(ty) = w.item(id).map(|i| i.ty) {
        (ty.delete)(w, id);
    }
    false
}

fn read_access_level(r: &mut dyn GenericReader) -> Result<AccessLevel> {
    let v = r.read_enum(4)? as i32;
    AccessLevel::from_i32(v)
        .ok_or_else(|| IoError::InvalidData(format!("AccessLevel: no value {v}")))
}

/// `Server.Items.BaseSuit` (abstract; `[SerializationGenerator(1, false)]`: version int 1, then `AccessLevel`).
pub static BASE_SUIT: ItemType = ItemType {
    full_name: "Server.Items.BaseSuit",
    parent: Some(&ITEM_BASE),
    construct: None,
    family: Some(&BASE_SUIT_CLASS),
    default_weight: |_, _| 1.0,
    on_single_click: |w, id, from| {
        if validate(w, id) {
            base::on_single_click_base(w, id, from);
        }
    },
    on_double_click: |w, id, from| {
        if validate(w, id) {
            base::on_double_click_base(w, id, from);
        }
    },
    verify_move: |w, id, from| {
        w.mobile(from)
            .is_some_and(|m| m.access_level >= access_level(w, id))
    },
    on_equip: |w, id, from| {
        if w.mobile(from)
            .is_some_and(|m| m.access_level < access_level(w, id))
        {
            deps::send_message(w, from, "You may not wear this.");
            return false;
        }
        true
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(1);
        out.write_enum(4, i64::from(access_level(w, id) as i32));
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let version = r.read_i32()?;
        if version < 1 {
            // `Deserialize(reader, version)`: `AccessLevel = (AccessLevel)reader.ReadInt()` (the setter runs).
            let v = r.read_i32()?;
            let level = AccessLevel::from_i32(v)
                .ok_or_else(|| IoError::InvalidData(format!("AccessLevel: no value {v}")))?;
            if let Some(i) = w.item_mut(id) {
                i.ext = Ext::new(SuitData {
                    access_level: AccessLevel::Player,
                });
            }
            set_access_level(w, id, level);
        } else {
            // The current layout assigns the field.
            let level = read_access_level(r)?;
            if let Some(i) = w.item_mut(id) {
                i.ext = Ext::new(SuitData {
                    access_level: level,
                });
            }
        }
        Ok(())
    },
    ..ITEM_BASE
};

/// One concrete suit: the constructor's `(level, hue, graphic)`; `ctor` is whether it is `[Constructible]`.
macro_rules! suit_leaves {
    ($($(#[$m:meta])* $name:ident, $new:ident = $full:literal, $level:expr, hue $hue:expr, graphic $gid:expr, $ctor:ident;)*) => {
        $(
            /// `new T()`.
            pub fn $new(w: &mut World, ty: &'static ItemType) -> ItemId {
                new_suit(w, ty, $level, $hue, $gid)
            }

            $(#[$m])*
            pub static $name: ItemType = ItemType {
                full_name: $full,
                parent: Some(&BASE_SUIT),
                construct: suit_leaves!(@construct $ctor, $new, $name),
                serialize: |w, id, out| {
                    (BASE_SUIT.serialize)(w, id, out);
                    out.write_i32(0); // [SerializationGenerator(0, false)]
                },
                deserialize: |w, id, r| {
                    (BASE_SUIT.deserialize)(w, id, r)?;
                    let _version = r.read_i32()?;
                    Ok(())
                },
                ..BASE_SUIT
            };
        )*

        /// Every suit class of this file.
        pub static LEAVES: &[&ItemType] = &[$(&$name),*];
    };
    (@construct yes, $new:ident, $name:ident) => { Some(|w| $new(w, &$name)) };
    (@construct no, $new:ident, $name:ident) => { None };
}

suit_leaves! {
    /// `Server.Items.AdminRobe` (`// [Constructible] // Deprecated, use StaffRobe`).
    ADMIN_ROBE, admin_robe_new = "Server.Items.AdminRobe", AccessLevel::Administrator, hue 0x0, graphic 0x204F, no;
    /// `Server.Items.CounselorRobe` (deprecated).
    COUNSELOR_ROBE, counselor_robe_new = "Server.Items.CounselorRobe", AccessLevel::Counselor, hue 0x3, graphic 0x204F, no;
    /// `Server.Items.DeathShroud`.
    DEATH_SHROUD, death_shroud_new = "Server.Items.DeathShroud", AccessLevel::GameMaster, hue 0x0, graphic 0x204E, yes;
    /// `Server.Items.DupreSuit`.
    DUPRE_SUIT, dupre_suit_new = "Server.Items.DupreSuit", AccessLevel::GameMaster, hue 0x0, graphic 0x2050, yes;
    /// `Server.Items.GMRobe` (deprecated).
    GM_ROBE, gm_robe_new = "Server.Items.GMRobe", AccessLevel::GameMaster, hue 0x26, graphic 0x204F, no;
    /// `Server.Items.LordBlackthorneSuit`.
    LORD_BLACKTHORNE_SUIT, lord_blackthorne_suit_new = "Server.Items.LordBlackthorneSuit", AccessLevel::GameMaster, hue 0x0, graphic 0x2043, yes;
    /// `Server.Items.LordBritishSuit`.
    LORD_BRITISH_SUIT, lord_british_suit_new = "Server.Items.LordBritishSuit", AccessLevel::GameMaster, hue 0x0, graphic 0x2042, yes;
    /// `Server.Items.SeerRobe` (deprecated).
    SEER_ROBE, seer_robe_new = "Server.Items.SeerRobe", AccessLevel::Seer, hue 0x1D3, graphic 0x204F, no;
}

/// `StaffRobe.GetStaffRobeHue(accessLevel)` (Player has no hue).
pub fn staff_robe_hue(level: AccessLevel) -> i32 {
    match level {
        AccessLevel::Counselor => 0x3,
        AccessLevel::GameMaster => 0x26,
        AccessLevel::Seer => 0x1D3,
        AccessLevel::Administrator => 0x488,
        AccessLevel::Developer => 11,
        AccessLevel::Owner => 0x54F,
        // shortcut: Player has no hue; 0 here, as nothing builds one.
        AccessLevel::Player => 0,
    }
}

/// `StaffRobe(level)`.
pub fn staff_robe_new(w: &mut World, ty: &'static ItemType, level: AccessLevel) -> ItemId {
    new_suit(w, ty, level, staff_robe_hue(level), 0x204F)
}

/// `Server.Items.StaffRobe` (`[SerializationGenerator(0)]`: encoded version; no `[Constructible]`).
pub static STAFF_ROBE_CLASS: SuitClass = SuitClass {
    on_access_level_changed: |w, id, old, new| {
        let old_hue = staff_robe_hue(old);
        if w.item(id).is_some_and(|i| i.hue == old_hue)
            && let Some(ty) = w.item(id).map(|i| i.ty)
        {
            (ty.set_hue)(w, id, staff_robe_hue(new));
        }
    },
};

/// `Server.Items.StaffRobe`.
pub static STAFF_ROBE: ItemType = ItemType {
    full_name: "Server.Items.StaffRobe",
    parent: Some(&BASE_SUIT),
    construct: None,
    family: Some(&STAFF_ROBE_CLASS),
    serialize: |w, id, out| {
        (BASE_SUIT.serialize)(w, id, out);
        out.write_encoded_int(0);
    },
    deserialize: |w, id, r| {
        (BASE_SUIT.deserialize)(w, id, r)?;
        let _version = r.read_encoded_int()?;
        Ok(())
    },
    ..BASE_SUIT
};

/// Registers the classes of this file.
pub fn register(types: &mut TypeRegistry) {
    for t in LEAVES {
        types.register_item(t, &[]);
    }
    types.register_item(&STAFF_ROBE, &[]);
}
