//! `Tapestry` and `DecorativeWeapon` (the two-part tapestries, swords and double axes) and the ankhs.
//!
//! Each of those is two items that follow each other: the main item and its `InternalItem`. They move, change map
//! and delete together; the pair is [`Pair`] in both `Item::ext`s and each class says where the other half stands
//! (its template prop `pairOffset`). Nested class names are `Outer+InternalItem` like `Type.FullName`.
//!
//! `Ankhs.Resurrect` shows `new ResurrectGump(m, ResurrectMessage.VirtueShrine)` and the tithe entry shows
//! `new TithingGump(from)`: both gumps are the gumps crate's, through [`AnkhHooks`].

use muo_content::props::{Ctor, Kind, Param, Value, add_ctors};
use muo_content::utilities::types::PropType;
use muo_io::{GenericReader, GenericWriter};
use muo_primitives::{Point3D, utility};
use muo_server::World;
use muo_server::context_menu_entry::ContextMenuEntry;
use muo_server::ext::Ext;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, base, deps, ops};
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::type_registry::{Phase, TypeRegistry};

/// The other half of a pair.
#[derive(Clone, Copy, Debug, Default)]
struct Pair {
    other: Option<ItemId>,
}

pub(crate) fn other(w: &World, id: ItemId) -> Option<ItemId> {
    w.item(id)?.ext.get::<Pair>()?.other
}

fn set_other(w: &mut World, id: ItemId, o: Option<ItemId>) {
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(Pair { other: o });
    }
}

/// Where the other half stands relative to this one (`_item.Location = new Point3D(X + dx, Y + dy, Z)`): the
/// class's `pairOffset` prop `[dx, dy]`.
fn pair_offset(w: &World, id: ItemId) -> Option<(i32, i32)> {
    let v = muo_content::templates::props::item(w, id, "pairOffset")?.as_array()?;
    let at = |i: usize| {
        v.get(i)
            .and_then(|x| x.as_i64())
            .and_then(|x| i32::try_from(x).ok())
    };
    Some((at(0)?, at(1)?))
}

fn on_location_change(w: &mut World, id: ItemId, _old: Point3D) {
    let (Some(o), Some((dx, dy))) = (other(w, id), pair_offset(w, id)) else {
        return;
    };
    let Some(l) = w.item(id).map(|i| i.location) else {
        return;
    };
    if let Some(ty) = w.item(o).map(|i| i.ty) {
        (ty.set_location)(w, o, Point3D::new(l.x + dx, l.y + dy, l.z));
    }
}

fn on_map_change(w: &mut World, id: ItemId) {
    let Some(o) = other(w, id) else { return };
    let map = w.item(id).and_then(|i| i.map);
    ops::set_map(w, o, map);
}

fn on_after_delete(w: &mut World, id: ItemId) {
    base::on_after_delete_base(w, id);
    if let Some(o) = other(w, id)
        && let Some(ty) = w.item(o).map(|i| i.ty)
    {
        (ty.delete)(w, o);
    }
}

fn write_pair(w: &World, id: ItemId, out: &mut dyn GenericWriter, encoded: bool) {
    (ITEM_BASE.serialize)(w, id, out);
    crate::simple::write_version(out, encoded);
    out.write_item(w, other(w, id));
}

fn read_pair(
    w: &mut World,
    id: ItemId,
    r: &mut dyn GenericReader,
    encoded: bool,
) -> muo_io::Result<()> {
    (ITEM_BASE.deserialize)(w, id, r)?;
    let _version = crate::simple::read_version(r, encoded)?;
    let o = r.read_item(w)?;
    set_other(w, id, o);
    Ok(())
}

/// Makes a pair: the main item (`main_gid`, not movable) and its inner item (`inner_gid`, `inner_movable`).
fn new_pair(
    w: &mut World,
    main: &'static ItemType,
    main_gid: i32,
    inner: &'static ItemType,
    inner_gid: i32,
    inner_movable: bool,
) -> ItemId {
    let id = ops::new_item_with_id(w, main, main_gid);
    ops::set_movable(w, id, false);
    let o = ops::new_item_with_id(w, inner, inner_gid);
    ops::set_movable(w, o, inner_movable);
    set_other(w, id, Some(o));
    set_other(w, o, Some(id));
    id
}

/// `new T()` of a paired class: both halves are made from their templates (graphic, `Movable`) and linked.
fn new_pair_of(
    w: &mut World,
    main: &'static ItemType,
    main_name: &str,
    inner: &'static ItemType,
    inner_name: &str,
) -> ItemId {
    let id = crate::simple::from_template(w, main, main_name);
    let o = crate::simple::from_template(w, inner, inner_name);
    set_other(w, id, Some(o));
    set_other(w, o, Some(id));
    id
}

macro_rules! pairs {
    ($($main:ident = $mfull:literal, $menc:expr;
       $inner:ident = $ifull:literal, $ienc:expr;)*) => {$(

        #[doc = concat!("`", $mfull, "`.")]
        pub static $main: ItemType = ItemType {
            full_name: $mfull,
            parent: Some(&ITEM_BASE),
            construct: Some(|w| new_pair_of(w, &$main, $mfull, &$inner, $ifull)),
            on_location_change,
            on_map_change,
            on_after_delete,
            serialize: |w, id, out| write_pair(w, id, out, $menc),
            deserialize: |w, id, r| read_pair(w, id, r, $menc),
            ..ITEM_BASE
        };

        #[doc = concat!("`", $ifull, "` (the nested `InternalItem`).")]
        pub static $inner: ItemType = ItemType {
            full_name: $ifull,
            parent: Some(&ITEM_BASE),
            on_location_change,
            on_map_change,
            on_after_delete,
            serialize: |w, id, out| write_pair(w, id, out, $ienc),
            deserialize: |w, id, r| read_pair(w, id, r, $ienc),
            ..ITEM_BASE
        };
    )*};
}

pairs! {
    TAPESTRY1N = "Server.Items.Tapestry1N", false;
    TAPESTRY1N_INNER = "Server.Items.Tapestry1N+InternalItem", true;
    TAPESTRY2N = "Server.Items.Tapestry2N", false;
    TAPESTRY2N_INNER = "Server.Items.Tapestry2N+InternalItem", false;
    TAPESTRY2W = "Server.Items.Tapestry2W", false;
    TAPESTRY2W_INNER = "Server.Items.Tapestry2W+InternalItem", false;
    TAPESTRY3N = "Server.Items.Tapestry3N", false;
    TAPESTRY3N_INNER = "Server.Items.Tapestry3N+InternalItem", false;
    TAPESTRY3W = "Server.Items.Tapestry3W", false;
    TAPESTRY3W_INNER = "Server.Items.Tapestry3W+InternalItem", false;
    TAPESTRY4N = "Server.Items.Tapestry4N", false;
    TAPESTRY4N_INNER = "Server.Items.Tapestry4N+InternalItem", false;
    TAPESTRY4W = "Server.Items.Tapestry4W", false;
    TAPESTRY4W_INNER = "Server.Items.Tapestry4W+InternalItem", false;
    TAPESTRY5N = "Server.Items.Tapestry5N", false;
    TAPESTRY5N_INNER = "Server.Items.Tapestry5N+InternalItem", false;
    TAPESTRY5W = "Server.Items.Tapestry5W", false;
    TAPESTRY5W_INNER = "Server.Items.Tapestry5W+InternalItem", false;
    TAPESTRY6N = "Server.Items.Tapestry6N", false;
    TAPESTRY6N_INNER = "Server.Items.Tapestry6N+InternalItem", false;
    TAPESTRY6W = "Server.Items.Tapestry6W", false;
    TAPESTRY6W_INNER = "Server.Items.Tapestry6W+InternalItem", false;
    DECORATIVE_SWORD_NORTH = "Server.Items.DecorativeSwordNorth", false;
    DECORATIVE_SWORD_NORTH_INNER = "Server.Items.DecorativeSwordNorth+InternalItem", false;
    DECORATIVE_SWORD_WEST = "Server.Items.DecorativeSwordWest", true;
    DECORATIVE_SWORD_WEST_INNER = "Server.Items.DecorativeSwordWest+InternalItem", true;
    DECORATIVE_DAXE_NORTH = "Server.Items.DecorativeDAxeNorth", true;
    DECORATIVE_DAXE_NORTH_INNER = "Server.Items.DecorativeDAxeNorth+InternalItem", true;
    DECORATIVE_DAXE_WEST = "Server.Items.DecorativeDAxeWest", true;
    DECORATIVE_DAXE_WEST_INNER = "Server.Items.DecorativeDAxeWest+InternalItem", true;
}

// ---- Ankhs ----

/// What `Ankhs` reaches in the gumps crate.
#[derive(Default)]
pub struct AnkhHooks {
    /// `m.SendGump(new ResurrectGump(m, ResurrectMessage.VirtueShrine))`.
    pub resurrect_gump: Option<fn(&mut World, MobileId)>,
    /// `from.SendGump(new TithingGump(from))`.
    pub tithing_gump: Option<fn(&mut World, MobileId)>,
}

/// `Ankhs.Resurrect(m, item)`.
pub fn resurrect(w: &mut World, m: MobileId, item: ItemId) {
    if muo_server::mobile::base::alive(w, m) {
        return;
    }
    if !deps::mobile_in_range(w, m, ops::get_world_location(w, item), 2) {
        deps::send_localized_message(w, m, 500446); // That is too far away.
        return;
    }
    let (loc, map) = w
        .mobile(m)
        .map_or((Point3D::new(0, 0, 0), None), |x| (x.location, x.map));
    let fits = map.is_some_and(|map| {
        muo_server::map::fit::can_fit(w, map, loc.x, loc.y, loc.z, 16, false, false, true)
    });
    if fits {
        if let Some(f) = w
            .resources
            .get::<AnkhHooks>()
            .and_then(|h| h.resurrect_gump)
        {
            f(w, m);
        }
    } else {
        deps::send_localized_message(w, m, 502391); // Thou can not be resurrected there!
    }
}

/// `Ankhs.LockKarmaEntry(mobile.KarmaLocked)` when `Core.UOTD && !Core.AOS` and `from` is a player.
pub fn karma_lock_entry(w: &World, from: MobileId) -> Option<ContextMenuEntry> {
    use muo_content::player_mobile::{self, PlayerFlag};
    let d = player_mobile::data(w, from)?;
    if !(muo_content::items::core_uotd(w) && !muo_content::core::aos(w)) {
        return None;
    }
    let mut e = ContextMenuEntry::new(
        if d.flag(PlayerFlag::KARMA_LOCKED) {
            6197
        } else {
            6196
        },
        2,
    );
    e.on_click = Some(std::rc::Rc::new(|w, from, target| {
        let EntityId::Item(item) = target else { return };
        if player_mobile::data(w, from).is_none() {
            return;
        }
        if !deps::mobile_in_range(w, from, ops::get_world_location(w, item), 2) {
            deps::send_localized_message(w, from, 500446); // That is too far away.
        }
        let locked = player_mobile::data(w, from).is_some_and(|d| d.flag(PlayerFlag::KARMA_LOCKED));
        if let Some(d) = player_mobile::data_mut(w, from) {
            d.set_flag(PlayerFlag::KARMA_LOCKED, !locked);
        }
        // Your karma has been locked. Your karma can no longer be raised. / unlocked. Your karma can be raised again.
        deps::send_localized_message(w, from, if locked { 1060191 } else { 1060192 });
    }));
    Some(e)
}

fn ankh_entries(w: &mut World, id: ItemId, from: MobileId, list: &mut Vec<ContextMenuEntry>) {
    (ITEM_BASE.get_context_menu_entries)(w, id, from, list);
    let alive = muo_server::mobile::base::alive(w, from);
    if let Some(e) = karma_lock_entry(w, from) {
        list.push(e);
    }
    let mut res = ContextMenuEntry::new(6195, 2);
    res.enabled = alive;
    res.on_click = Some(std::rc::Rc::new(|w, from, target| {
        if let EntityId::Item(item) = target {
            resurrect(w, from, item);
        }
    }));
    list.push(res);
    if muo_content::core::aos(w) {
        let mut tithe = ContextMenuEntry::new(6198, 2);
        tithe.enabled = alive;
        tithe.on_click = Some(std::rc::Rc::new(|w, from, _| {
            if muo_server::mobile::part2::stats::check_alive(w, from, true)
                && let Some(f) = w.resources.get::<AnkhHooks>().and_then(|h| h.tithing_gump)
            {
                f(w, from);
            }
        }));
        list.push(tithe);
    }
}

fn ankh_on_movement(w: &mut World, id: ItemId, m: MobileId, old: Point3D) {
    let Some(i) = w.item(id) else { return };
    let Some(loc) = w.mobile(m).map(|x| x.location) else {
        return;
    };
    let (l, parent) = (i.location, i.parent.is_some());
    if !parent
        && utility::in_range(l.x, l.y, loc.x, loc.y, 1)
        && !utility::in_range(l.x, l.y, old.x, old.y, 1)
    {
        resurrect(w, m, id);
    }
}

fn ankh_set_hue(w: &mut World, id: ItemId, value: i32) {
    base::set_hue_base(w, id, value);
    if let Some(o) = other(w, id)
        && w.item(o).is_some_and(|i| i.hue != value)
        && let Some(ty) = w.item(o).map(|i| i.ty)
    {
        (ty.set_hue)(w, o, value);
    }
}

macro_rules! ankhs {
    ($($main:ident = $mfull:literal;
       $inner:ident = $ifull:literal;)*) => {$(

        #[doc = concat!("`", $mfull, "`.")]
        pub static $main: ItemType = ItemType {
            full_name: $mfull,
            parent: Some(&ITEM_BASE),
            construct: Some(|w| new_pair_of(w, &$main, $mfull, &$inner, $ifull)),
            handles_on_movement: |_, _| true,
            on_movement: ankh_on_movement,
            get_context_menu_entries: ankh_entries,
            on_double_click_dead: |w, id, m| resurrect(w, m, id),
            set_hue: ankh_set_hue,
            on_location_change,
            on_map_change,
            on_after_delete,
            serialize: |w, id, out| write_pair(w, id, out, false),
            deserialize: |w, id, r| read_pair(w, id, r, false),
            ..ITEM_BASE
        };

        #[doc = concat!("`", $ifull, "` (the nested `InternalItem`).")]
        pub static $inner: ItemType = ItemType {
            full_name: $ifull,
            parent: Some(&ITEM_BASE),
            handles_on_movement: |_, _| true,
            on_movement: ankh_on_movement,
            get_context_menu_entries: ankh_entries,
            on_double_click_dead: |w, id, m| resurrect(w, m, id),
            set_hue: ankh_set_hue,
            on_location_change,
            on_map_change,
            on_after_delete,
            serialize: |w, id, out| write_pair(w, id, out, false),
            deserialize: |w, id, r| read_pair(w, id, r, false),
            ..ITEM_BASE
        };
    )*};
}

/// `new AnkhWest(bloodied)` / `new AnkhNorth(bloodied)`.
pub fn new_ankh_bloodied(w: &mut World, north: bool, bloodied: bool) -> ItemId {
    let (main, inner, mg, ig) = if north {
        (
            &ANKH_NORTH,
            &ANKH_NORTH_INNER,
            if bloodied { 0x1E5D } else { 0x4 },
            if bloodied { 0x1E5C } else { 0x5 },
        )
    } else {
        (
            &ANKH_WEST,
            &ANKH_WEST_INNER,
            if bloodied { 0x1D98 } else { 0x3 },
            if bloodied { 0x1D97 } else { 0x2 },
        )
    };
    new_pair(w, main, mg, inner, ig, false)
}

ankhs! {
    ANKH_WEST = "Server.Items.AnkhWest";
    ANKH_WEST_INNER = "Server.Items.AnkhWest+InternalItem";
    ANKH_NORTH = "Server.Items.AnkhNorth";
    ANKH_NORTH_INNER = "Server.Items.AnkhNorth+InternalItem";
}

static BLOODIED: [Param; 1] = [Param {
    name: "bloodied",
    kind: Kind::Plain(PropType::Bool),
    default: Some(Value::Bool(false)),
    ty: "",
}];
static C_ANKH_WEST: [Ctor; 1] = [Ctor {
    access: None,
    params: &BLOODIED,
    make: |w, v| EntityId::Item(new_ankh_bloodied(w, false, v[0].bool())),
}];
static C_ANKH_NORTH: [Ctor; 1] = [Ctor {
    access: None,
    params: &BLOODIED,
    make: |w, v| EntityId::Item(new_ankh_bloodied(w, true, v[0].bool())),
}];

fn configure(w: &mut World) {
    add_ctors(w, "Server.Items.AnkhWest", &C_ANKH_WEST);
    add_ctors(w, "Server.Items.AnkhNorth", &C_ANKH_NORTH);
}

/// Registers the classes.
pub fn register(types: &mut TypeRegistry) {
    for t in [
        &TAPESTRY1N,
        &TAPESTRY1N_INNER,
        &TAPESTRY2N,
        &TAPESTRY2N_INNER,
        &TAPESTRY2W,
        &TAPESTRY2W_INNER,
        &TAPESTRY3N,
        &TAPESTRY3N_INNER,
        &TAPESTRY3W,
        &TAPESTRY3W_INNER,
        &TAPESTRY4N,
        &TAPESTRY4N_INNER,
        &TAPESTRY4W,
        &TAPESTRY4W_INNER,
        &TAPESTRY5N,
        &TAPESTRY5N_INNER,
        &TAPESTRY5W,
        &TAPESTRY5W_INNER,
        &TAPESTRY6N,
        &TAPESTRY6N_INNER,
        &TAPESTRY6W,
        &TAPESTRY6W_INNER,
        &DECORATIVE_SWORD_NORTH,
        &DECORATIVE_SWORD_NORTH_INNER,
        &DECORATIVE_SWORD_WEST,
        &DECORATIVE_SWORD_WEST_INNER,
        &DECORATIVE_DAXE_NORTH,
        &DECORATIVE_DAXE_NORTH_INNER,
        &DECORATIVE_DAXE_WEST,
        &DECORATIVE_DAXE_WEST_INNER,
        &ANKH_WEST,
        &ANKH_WEST_INNER,
    ] {
        types.register_item(t, &[]);
    }
    types.register_item(&ANKH_NORTH, &["Server.Items.AnkhEast"]);
    types.register_item(&ANKH_NORTH_INNER, &["Server.Items.AnkhEast+InternalItem"]);
    types.register_phase_default(Phase::Configure, "Server.Items.Ankhs.Configure", configure);
}

#[cfg(test)]
mod tests {
    use super::*;
    use muo_io::{SaveReader, SaveWriter};

    fn save(w: &World, id: ItemId) -> Vec<u8> {
        let mut out = SaveWriter::new(true);
        (w.item(id).unwrap().ty.serialize)(w, id, &mut out);
        out.into_vec()
    }

    #[test]
    fn a_tapestry_is_two_items_that_save_each_other() {
        let mut w = crate::test_world();
        let id = (TAPESTRY3N.construct.unwrap())(&mut w);
        let o = other(&w, id).unwrap();
        assert_eq!(
            (w.item(id).unwrap().item_id, w.item(o).unwrap().item_id),
            (0xFD6, 0xFD5)
        );
        assert!(!w.item(id).unwrap().movable() && w.item(o).unwrap().movable());
        assert_eq!(other(&w, o), Some(id));
        let bytes = save(&w, id);
        let mut want = SaveWriter::new(true);
        (ITEM_BASE.serialize)(&w, id, &mut want);
        want.write_i32(0);
        want.write_item(&w, Some(o));
        assert_eq!(bytes, want.into_vec());
        let id2 = w.add_item(muo_server::item::Item::new_for_load(
            muo_primitives::Serial::new(0x4000_0100),
            &TAPESTRY3N,
        ));
        let mut r = SaveReader::new(&bytes);
        (TAPESTRY3N.deserialize)(&mut w, id2, &mut r).unwrap();
        assert_eq!((r.remaining(), other(&w, id2)), (0, Some(o)));
    }

    #[test]
    fn the_encoded_version_classes_write_one_byte() {
        let mut w = crate::test_world();
        let id = (TAPESTRY1N.construct.unwrap())(&mut w);
        let o = other(&w, id).unwrap();
        let mut want = SaveWriter::new(true);
        (ITEM_BASE.serialize)(&w, o, &mut want);
        want.write_encoded_int(0);
        want.write_item(&w, Some(id));
        assert_eq!(save(&w, o), want.into_vec());
    }

    #[test]
    fn a_bloodied_ankh_uses_the_bloodied_graphics() {
        let mut w = crate::test_world();
        let id = new_ankh_bloodied(&mut w, true, true);
        let o = other(&w, id).unwrap();
        assert_eq!(
            (w.item(id).unwrap().item_id, w.item(o).unwrap().item_id),
            (0x1E5D, 0x1E5C)
        );
    }
}
