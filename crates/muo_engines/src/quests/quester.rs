//! `TalkEntry` and `BaseQuester`, the base of the quest givers.
//!
//! A quester is a `BaseVendor` that sells nothing, cannot be hurt or moved and talks. What differs per class is
//! [`QuesterOps`] (`OnTalk`, `CanTalkTo`, `TalkNumber`, `GetAutoTalkRange`), kept in the mobile's `Ext` as
//! [`QuesterData`] and set by the class's constructor and its deserializer (the `quester!` macro).

use muo_content::creature::CreatureType;
use muo_content::vendor::{BASE_VENDOR, BASE_VENDOR_C, BASE_VENDOR_CLASS, VendorClass};
use muo_primitives::Point3D;
use muo_server::World;
use muo_server::context_menu_entry::ContextMenuEntry;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::mobile::MobileType;

/// The virtuals `BaseQuester` leaves to its subclasses.
pub struct QuesterOps {
    /// `TalkNumber`.
    pub talk_number: i32,
    /// `OnTalk(player, contextMenu)`: `(world, quester, player, contextMenu)`.
    pub on_talk: fn(&mut World, MobileId, MobileId, bool),
    /// `CanTalkTo(to)`.
    pub can_talk_to: fn(&mut World, MobileId, MobileId) -> bool,
    /// `GetAutoTalkRange(m)`.
    pub auto_talk_range: fn(&World, MobileId, MobileId) -> i32,
    /// The class's own saved fields, written after its version (a `[SerializableField]`).
    pub save: fn(&World, MobileId, &mut dyn muo_io::GenericWriter),
    /// ... and read back.
    pub load: fn(&mut World, MobileId, &mut dyn muo_io::GenericReader) -> muo_io::Result<()>,
}

/// `BaseQuester`'s defaults: talk 6146, anyone, no automatic talking.
pub const QUESTER_OPS: QuesterOps = QuesterOps {
    talk_number: 6146, // Talk
    on_talk: |_, _, _, _| {},
    can_talk_to: |_, _, _| true,
    auto_talk_range: |_, _, _| -1,
    save: |_, _, _| {},
    load: |_, _, _| Ok(()),
};

/// A quester's `QuesterOps`.
#[derive(Clone, Copy)]
pub struct QuesterData {
    pub ops: &'static QuesterOps,
}

/// The ops of quester `id`; `None` for a mobile that is not a quester.
pub fn ops(w: &World, id: MobileId) -> Option<&'static QuesterOps> {
    w.mobile(id)?.ext.get::<QuesterData>().map(|d| d.ops)
}

/// `quester.CanTalkTo(to)`.
pub fn can_talk_to(w: &mut World, quester: MobileId, to: MobileId) -> bool {
    ops(w, quester).is_some_and(|o| (o.can_talk_to)(w, quester, to))
}

/// `quester.OnTalk(player, contextMenu)`.
pub fn on_talk(w: &mut World, quester: MobileId, player: MobileId, context_menu: bool) {
    if let Some(o) = ops(w, quester) {
        (o.on_talk)(w, quester, player, context_menu);
    }
}

fn is_player(w: &World, m: MobileId) -> bool {
    w.mobile(m)
        .is_some_and(|m| m.ty.is(&muo_content::player_mobile::PLAYER_MOBILE))
}

/// `BaseQuester.AddCustomContextEntries(from, ref list)`.
fn add_custom_context_entries(
    w: &mut World,
    quester: MobileId,
    from: MobileId,
    list: &mut Vec<ContextMenuEntry>,
) {
    (BASE_VENDOR_C.add_custom_context_entries)(w, quester, from, list);

    let Some(o) = ops(w, quester) else { return };
    if muo_content::mx::alive(w, from)
        && is_player(w, from)
        && o.talk_number > 0
        && (o.can_talk_to)(w, quester, from)
    {
        // TalkEntry
        let mut e = ContextMenuEntry::new(o.talk_number, -1);
        e.on_click = Some(std::rc::Rc::new(
            |w: &mut World, from: MobileId, target: EntityId| {
                let EntityId::Mobile(q) = target else { return };
                if muo_server::mobile::part2::stats::check_alive(w, from, true)
                    && is_player(w, from)
                    && can_talk_to(w, q, from)
                {
                    on_talk(w, q, from, true);
                }
            },
        ));
        list.push(e);
    }
}

/// `BaseQuester.OnMovement(m, oldLocation)`.
fn on_movement(w: &mut World, quester: MobileId, m: MobileId, old: Point3D) {
    if !(muo_content::mx::alive(w, m) && is_player(w, m)) {
        return;
    }
    let Some(o) = ops(w, quester) else { return };
    let range = (o.auto_talk_range)(w, quester, m);
    if muo_content::mx::alive(w, m)
        && range >= 0
        && muo_content::mx::in_range(w, quester, m, range)
        && !muo_content::mx::in_range_point(w, quester, old, range)
        && (o.can_talk_to)(w, quester, m)
    {
        (o.on_talk)(w, quester, m, false);
    }
}

/// `BaseQuester.GetNewContainer()`: `new Bag { Hue = QuestSystem.RandomBrightHue() }`.
pub fn get_new_container(w: &mut World) -> ItemId {
    let bag = muo_content::factory::make_item(w, "Server.Items.Bag").expect("Bag is registered");
    let hue = super::core::random_bright_hue(w);
    set_hue(w, bag, hue)
}

/// `BaseQuester.SetHue(item, hue)`.
pub fn set_hue(w: &mut World, item: ItemId, hue: i32) -> ItemId {
    if let Some(f) = w.item(item).map(|i| i.ty.set_hue) {
        f(w, item, hue);
    }
    item
}

/// `BaseQuester.FocusTo(to)`.
pub fn focus_to(w: &mut World, quester: MobileId, to: MobileId) {
    super::core::focus_to(w, quester, to);
}

/// `BaseQuester`'s `VendorClass`: not an active vendor.
pub static QUESTER_VC: VendorClass = VendorClass {
    is_active_vendor: |_, _| false,
    ..BASE_VENDOR_CLASS
};

/// `BaseQuester`'s `CreatureType`.
pub static QUESTER_C: CreatureType = CreatureType {
    can_teach: |_, _| false,
    is_invulnerable: |_, _| true,
    disallow_all_moves: |_, _| true,
    add_custom_context_entries,
    subclass: Some(&QUESTER_VC),
    ..BASE_VENDOR_C
};

/// `Server.Engines.Quests.BaseQuester` (abstract): `[SerializationGenerator(0, false)]`, so a plain int version.
pub static BASE_QUESTER: MobileType = MobileType {
    full_name: "Server.Engines.Quests.BaseQuester",
    parent: Some(&BASE_VENDOR),
    construct: None,
    family: Some(&QUESTER_C),
    click_title: |_, _| false,
    can_be_damaged: |_, _| false,
    on_movement,
    serialize: |w, id, out| {
        (BASE_VENDOR.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (BASE_VENDOR.deserialize)(w, id, r)?;
        r.read_i32()?;
        Ok(())
    },
    ..BASE_VENDOR
};

/// One quester class: a `VendorClass`, a `CreatureType` and a `MobileType`, each `..PARENT` plus the lines given.
/// `encoded` is whether the class's generated serializer writes its version as an encoded int. The concrete form
/// has the `[Constructible]` constructor: `new_vendor` with `title`, then `extra` (the rest of the class's constructor);
/// the `abstract` form has none.
#[macro_export]
macro_rules! quester {
    (
        $(#[$meta:meta])*
        $name:ident, $cname:ident, $vname:ident = $full:literal : $parent:ident / $pc:ident / $pv:ident,
        ops $ops:expr, encoded $enc:literal, title $title:expr, extra $extra:expr;
        v { $($vf:ident : $vv:expr,)* }
        c { $($cf:ident : $cv:expr,)* }
        m { $($mf:ident : $mv:expr,)* }
    ) => {
        $crate::quester!(@define $(#[$meta])* $name, $cname, $vname = $full : $parent / $pc / $pv,
            ops $ops, encoded $enc,
            construct Some(|w| {
                let id = muo_content::vendor::new_vendor(w, &$name, $title);
                if let Some(m) = w.mobile_mut(id) {
                    m.ext.insert($crate::quests::quester::QuesterData { ops: &$ops });
                }
                let extra: fn(&mut muo_server::World, muo_server::ids::MobileId) = $extra;
                extra(w, id);
                id
            }), make None;
            v { $($vf : $vv,)* } c { $($cf : $cv,)* } m { $($mf : $mv,)* });
    };
    (
        $(#[$meta:meta])*
        $name:ident, $cname:ident, $vname:ident = $full:literal : $parent:ident / $pc:ident / $pv:ident,
        ops $ops:expr, encoded $enc:literal, title $title:expr, extra $extra:expr, makeonly;
        v { $($vf:ident : $vv:expr,)* }
        c { $($cf:ident : $cv:expr,)* }
        m { $($mf:ident : $mv:expr,)* }
    ) => {
        $crate::quester!(@define $(#[$meta])* $name, $cname, $vname = $full : $parent / $pc / $pv,
            ops $ops, encoded $enc,
            construct None,
            make Some(|w| {
                let id = muo_content::vendor::new_vendor(w, &$name, $title);
                if let Some(m) = w.mobile_mut(id) {
                    m.ext.insert($crate::quests::quester::QuesterData { ops: &$ops });
                }
                let extra: fn(&mut muo_server::World, muo_server::ids::MobileId) = $extra;
                extra(w, id);
                id
            });
            v { $($vf : $vv,)* } c { $($cf : $cv,)* } m { $($mf : $mv,)* });
    };
    (
        $(#[$meta:meta])*
        $name:ident, $cname:ident, $vname:ident = $full:literal : $parent:ident / $pc:ident / $pv:ident,
        ops $ops:expr, encoded $enc:literal, abstract;
        v { $($vf:ident : $vv:expr,)* }
        c { $($cf:ident : $cv:expr,)* }
        m { $($mf:ident : $mv:expr,)* }
    ) => {
        $crate::quester!(@define $(#[$meta])* $name, $cname, $vname = $full : $parent / $pc / $pv,
            ops $ops, encoded $enc, construct None, make None;
            v { $($vf : $vv,)* } c { $($cf : $cv,)* } m { $($mf : $mv,)* });
    };
    (@define
        $(#[$meta:meta])*
        $name:ident, $cname:ident, $vname:ident = $full:literal : $parent:ident / $pc:ident / $pv:ident,
        ops $ops:expr, encoded $enc:literal, construct $construct:expr, make $make:expr;
        v { $($vf:ident : $vv:expr,)* }
        c { $($cf:ident : $cv:expr,)* }
        m { $($mf:ident : $mv:expr,)* }
    ) => {
        pub static $vname: muo_content::vendor::VendorClass = muo_content::vendor::VendorClass { $($vf: $vv,)* ..$pv };
        pub static $cname: muo_content::creature::CreatureType =
            muo_content::creature::CreatureType { subclass: Some(&$vname), $($cf: $cv,)* ..$pc };
        $(#[$meta])*
        pub static $name: muo_server::mobile::MobileType = muo_server::mobile::MobileType {
            full_name: $full,
            parent: Some(&$parent),
            construct: $construct,
            make: $make,
            family: Some(&$cname),
            serialize: |w, id, out| {
                ($parent.serialize)(w, id, out);
                if $enc {
                    out.write_encoded_int(0);
                } else {
                    out.write_i32(0);
                }
                ($ops.save)(w, id, out);
            },
            deserialize: |w, id, r| {
                ($parent.deserialize)(w, id, r)?;
                if let Some(m) = w.mobile_mut(id) {
                    m.ext.insert($crate::quests::quester::QuesterData { ops: &$ops });
                }
                if $enc {
                    r.read_encoded_int()?;
                } else {
                    r.read_i32()?;
                }
                ($ops.load)(w, id, r)
            },
            $($mf: $mv,)*
            ..$parent
        };
    };
}
