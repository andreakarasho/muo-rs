//! Bulk order books: `BulkOrderBook`, its entries (`BOBSmallEntry`, `BOBLargeEntry` with
//! `BOBLargeSubEntry`, saved by `BOBEntries : GenericEntityPersistence<IBOBEntry>("BOBEntries", 3, 0x1,
//! 0x7FFFFFFF)`), `BOBFilter` (the player's copy is in `PlayerData`), `BOBGump`, `BOBFilterGump` and `BODBuyGump`.
//!
//! `PlayerVendor` is not in a crate this one depends on: `BobHooks` answers `RootParent is PlayerVendor`,
//! `GetVendorItem(book)?.IsForSale` and `HoldGold += price` (set in `special_wiring`).

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::rc::Rc;

use indexmap::IndexMap;
use muo_content::gumps::{
    DynamicGumpBuilder, DynamicGumpLogic, DynamicGumpObj, GumpStrings, RelayInfo, StaticGumpLogic,
    StaticGumpObj, send_gump_to_mobile,
};
use muo_content::player_mobile::{self, BobFilter, PlayerFlag};
use muo_io::serialization::index::{EntityFileWriter, IdxTypeRef, TypeTable, anchored_time_shift};
use muo_io::serialization::persistence::{bin_path, load_indexes};
use muo_io::{GenericReader, GenericWriter, Result, SaveReader, SaveWriter, SeekOrigin};
use muo_primitives::hash::compute_hash64;
use muo_primitives::{DateTime, Serial};
use muo_server::World;
use muo_server::ext::Ext;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, LootType, TotalType, container, deps, ops};
use muo_server::layer::Layer;
use muo_server::net_state::NetStateId;
use muo_server::persistence::{LoadContext, PersistError, Persistence, Snapshot};
use muo_server::property_list::PropertyList;
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::type_registry::TypeRegistry;

use super::{
    BOD_SMITH, BOD_TAILOR, BulkGenericType, LargeBulkEntry, SmallBulkEntry, classify, data,
    in_pack, is_large, mat, new_large_with, new_small_with,
};

const SMALL_NAME: &str = "Server.Engines.BulkOrders.BOBSmallEntry";
const LARGE_NAME: &str = "Server.Engines.BulkOrders.BOBLargeEntry";

// ---- the entries ----

/// `BOBLargeSubEntry`.
#[derive(Clone, Debug, Default)]
pub struct SubEntry {
    pub item_type: Option<&'static str>,
    pub amount_cur: i32,
    pub number: i32,
    pub graphic: i32,
}

#[derive(Clone, Debug)]
pub enum BobKind {
    Small {
        item_type: Option<&'static str>,
        amount_cur: i32,
        number: i32,
        graphic: i32,
    },
    Large {
        entries: Vec<SubEntry>,
    },
}

/// `BaseBOBEntry` with `BOBSmallEntry` / `BOBLargeEntry`.
#[derive(Clone, Debug)]
pub struct BobEntry {
    pub created: DateTime,
    pub require_exceptional: bool,
    pub deed_type: i32,
    pub material: i32,
    pub amount_max: i32,
    pub price: i32,
    pub kind: BobKind,
}

impl BobEntry {
    fn blank(created: DateTime, large: bool) -> BobEntry {
        BobEntry {
            created,
            require_exceptional: false,
            deed_type: 0,
            material: 0,
            amount_max: 0,
            price: 0,
            kind: if large {
                BobKind::Large {
                    entries: Vec::new(),
                }
            } else {
                BobKind::Small {
                    item_type: None,
                    amount_cur: 0,
                    number: 0,
                    graphic: 0,
                }
            },
        }
    }

    pub fn is_large(&self) -> bool {
        matches!(self.kind, BobKind::Large { .. })
    }

    /// How many table rows the entry takes.
    pub fn rows(&self) -> usize {
        match &self.kind {
            BobKind::Large { entries } => entries.len(),
            BobKind::Small { .. } => 1,
        }
    }
}

/// `BOBEntries`' table.
#[derive(Default)]
pub struct BobState {
    pub entries: IndexMap<Serial, BobEntry>,
    last_serial: u32,
}

impl BobState {
    /// `BOBEntries.NewBOBEntry`.
    fn new_serial(&mut self) -> Serial {
        self.last_serial += 1;
        Serial::new(self.last_serial)
    }
}

fn state(w: &World) -> Option<&BobState> {
    w.resources.get::<BobState>()
}

fn state_mut(w: &mut World) -> &mut BobState {
    w.resources.get_mut::<BobState>()
}

pub fn entry(w: &World, s: Serial) -> Option<&BobEntry> {
    state(w)?.entries.get(&s)
}

/// `new BOBSmallEntry(bod)` / `new BOBLargeEntry(bod)`: registered, its serial returned.
pub fn add_from_deed(w: &mut World, deed: ItemId) -> Serial {
    let d = data(w, deed).cloned().unwrap_or_default();
    let now = w.now;
    let large = is_large(w, deed);
    let mut e = BobEntry::blank(now, large);
    e.require_exceptional = d.require_exceptional;
    e.deed_type = if super::is_tailor(w, deed) {
        BOD_TAILOR
    } else {
        BOD_SMITH
    };
    e.material = d.material;
    e.amount_max = d.amount_max;
    e.kind = if large {
        BobKind::Large {
            entries: d
                .entries
                .iter()
                .map(|x| SubEntry {
                    item_type: x.details.type_name,
                    amount_cur: x.amount,
                    number: x.details.number,
                    graphic: x.details.graphic,
                })
                .collect(),
        }
    } else {
        BobKind::Small {
            item_type: d.type_name,
            amount_cur: d.amount_cur,
            number: d.number,
            graphic: d.graphic,
        }
    };
    let st = state_mut(w);
    let s = st.new_serial();
    st.entries.insert(s, e);
    s
}

/// `Reconstruct()`.
pub fn reconstruct(w: &mut World, e: &BobEntry) -> Option<ItemId> {
    let tailor = e.deed_type == BOD_TAILOR;
    if e.deed_type != BOD_SMITH && !tailor {
        return None;
    }
    let hue = if tailor { 0x483 } else { 0x44E };
    match &e.kind {
        BobKind::Small {
            item_type,
            amount_cur,
            number,
            graphic,
        } => {
            let ty = if tailor {
                &super::SMALL_TAILOR_BOD
            } else {
                &super::SMALL_SMITH_BOD
            };
            Some(new_small_with(
                w,
                ty,
                hue,
                *amount_cur,
                e.amount_max,
                *item_type,
                *number,
                *graphic,
                e.require_exceptional,
                e.material,
            ))
        }
        BobKind::Large { entries } => {
            let ty = if tailor {
                &super::LARGE_TAILOR_BOD
            } else {
                &super::LARGE_SMITH_BOD
            };
            let list = entries
                .iter()
                .map(|s| LargeBulkEntry {
                    amount: s.amount_cur,
                    details: SmallBulkEntry {
                        type_name: s.item_type,
                        number: s.number,
                        graphic: s.graphic,
                    },
                })
                .collect();
            Some(new_large_with(
                w,
                ty,
                hue,
                e.amount_max,
                e.require_exceptional,
                e.material,
                list,
            ))
        }
    }
}

// ---- entry serialization ----

fn write_entry(e: &BobEntry, out: &mut dyn GenericWriter) {
    out.write_encoded_int(1);
    out.write_bool(e.require_exceptional);
    out.write_enum(4, i64::from(e.deed_type));
    out.write_enum(4, i64::from(e.material));
    out.write_i32(e.amount_max);
    out.write_i32(e.price);
    out.write_encoded_int(1);
    match &e.kind {
        BobKind::Small {
            item_type,
            amount_cur,
            number,
            graphic,
        } => {
            out.write_type(*item_type);
            out.write_encoded_int(*amount_cur);
            out.write_encoded_int(*number);
            out.write_encoded_int(*graphic);
        }
        BobKind::Large { entries } => {
            out.write_encoded_int(entries.len() as i32);
            for s in entries {
                out.write_encoded_int(0);
                out.write_type(s.item_type);
                out.write_encoded_int(s.amount_cur);
                out.write_encoded_int(s.number);
                out.write_encoded_int(s.graphic);
            }
        }
    }
}

fn read_type(w: &World, r: &mut dyn GenericReader) -> Result<Option<&'static str>> {
    Ok(ReadEntityExt::read_type(r, w)?.map(|t| t.full_name()))
}

fn read_sub(w: &World, r: &mut dyn GenericReader) -> Result<SubEntry> {
    let _version = r.read_encoded_int()?;
    Ok(SubEntry {
        item_type: read_type(w, r)?,
        amount_cur: r.read_encoded_int()?,
        number: r.read_encoded_int()?,
        graphic: r.read_encoded_int()?,
    })
}

/// The generated `Deserialize` of `BaseBOBEntry` and its subclass: a version below 1 reads the old layout (the
/// base one seeks back over its version byte, as `Deserialize(reader, 0)` does).
fn read_entry(w: &World, r: &mut dyn GenericReader, e: &mut BobEntry) -> Result<()> {
    let v = r.read_encoded_int()?;
    if v < 1 {
        r.seek(-1, SeekOrigin::Current)?;
    } else {
        e.require_exceptional = r.read_bool()?;
        e.deed_type = r.read_enum(4)? as i32;
        e.material = r.read_enum(4)? as i32;
        e.amount_max = r.read_i32()?;
        e.price = r.read_i32()?;
    }
    let v = r.read_encoded_int()?;
    match &mut e.kind {
        BobKind::Small {
            item_type,
            amount_cur,
            number,
            graphic,
        } => {
            if v < 1 {
                *item_type = read_type(w, r)?;
                e.require_exceptional = r.read_bool()?;
                e.deed_type = r.read_encoded_int()?;
                e.material = r.read_encoded_int()?;
                *amount_cur = r.read_encoded_int()?;
                e.amount_max = r.read_encoded_int()?;
                *number = r.read_encoded_int()?;
                *graphic = r.read_encoded_int()?;
                e.price = r.read_encoded_int()?;
            } else {
                *item_type = read_type(w, r)?;
                *amount_cur = r.read_encoded_int()?;
                *number = r.read_encoded_int()?;
                *graphic = r.read_encoded_int()?;
            }
        }
        BobKind::Large { entries } => {
            if v < 1 {
                e.require_exceptional = r.read_bool()?;
                e.deed_type = r.read_encoded_int()?;
                e.material = r.read_encoded_int()?;
                e.amount_max = r.read_encoded_int()?;
                e.price = r.read_encoded_int()?;
            }
            let n = r.read_encoded_int()?;
            entries.clear();
            for _ in 0..n.max(0) {
                entries.push(read_sub(w, r)?);
            }
        }
    }
    Ok(())
}

// ---- persistence ----

struct Span {
    serial: Serial,
    position: i64,
    length: i32,
}

#[derive(Default)]
pub struct BobPersistence {
    spans: Vec<Span>,
    anchored_shift: i64,
    estimated: u64,
}

impl Persistence for BobPersistence {
    fn name(&self) -> &str {
        "BOBEntries"
    }

    fn deserialize_indexes(
        &mut self,
        w: &mut World,
        ctx: &mut LoadContext<'_>,
    ) -> std::result::Result<(), PersistError> {
        self.spans.clear();
        for (_, idx) in load_indexes(ctx.save_path, "BOBEntries")? {
            if idx.version >= 5 {
                self.anchored_shift = anchored_time_shift(idx.anchor_ticks, w.now.ticks());
            }
            for r in &idx.records {
                let name = match r.ty {
                    IdxTypeRef::Table(i) => idx.type_table.get(usize::from(i)).cloned(),
                    IdxTypeRef::Hash(h) => [SMALL_NAME, LARGE_NAME]
                        .into_iter()
                        .find(|n| compute_hash64(n) == h)
                        .map(str::to_owned),
                    IdxTypeRef::Legacy(_) => None,
                };
                let large = match name.as_deref() {
                    Some(SMALL_NAME) => false,
                    Some(LARGE_NAME) => true,
                    _ => continue,
                };
                let created = r.created_ticks.map_or(w.now, DateTime::from_ticks);
                let s = state_mut(w);
                s.last_serial = s.last_serial.max(r.serial.value());
                s.entries.insert(r.serial, BobEntry::blank(created, large));
                self.spans.push(Span {
                    serial: r.serial,
                    position: r.position,
                    length: r.length,
                });
            }
        }
        Ok(())
    }

    fn deserialize(
        &mut self,
        w: &mut World,
        ctx: &mut LoadContext<'_>,
    ) -> std::result::Result<(), PersistError> {
        let spans = std::mem::take(&mut self.spans);
        let Ok(bytes) = std::fs::read(bin_path(ctx.save_path, "BOBEntries")) else {
            for s in spans {
                state_mut(w).entries.shift_remove(&s.serial);
            }
            return Ok(());
        };
        self.estimated = bytes.len() as u64;
        let mut reader = SaveReader::new(&bytes)
            .with_types_db(ctx.types_db)
            .with_anchored_time_shift_ticks(self.anchored_shift);
        let mut delete_all = false;
        let mut to_delete = Vec::new();
        for span in &spans {
            if span.length == 0 {
                to_delete.push(span.serial);
                continue;
            }
            let result = reader
                .seek(span.position, SeekOrigin::Begin)
                .map_err(|e| e.to_string())
                .and_then(|_| {
                    let outcome = catch_unwind(AssertUnwindSafe(|| {
                        let mut e = state(w)
                            .and_then(|s| s.entries.get(&span.serial).cloned())
                            .ok_or_else(|| muo_io::IoError::InvalidData("no entry".into()))?;
                        read_entry(w, &mut reader, &mut e)?;
                        state_mut(w).entries.insert(span.serial, e);
                        Ok::<(), muo_io::IoError>(())
                    }));
                    match outcome {
                        Ok(Ok(())) => {}
                        Ok(Err(e)) => return Err(e.to_string()),
                        Err(_) => return Err("panic".into()),
                    }
                    let done = reader.position() - span.position as u64;
                    if done != span.length as u64 {
                        return Err(format!(
                            "Serialized object was {} bytes, but {done} bytes deserialized",
                            span.length
                        ));
                    }
                    Ok(())
                });
            if let Err(error) = result {
                ctx.prompt.write_line(&format!(
                    "***** Bad deserialize of IBOBEntry ({}) *****",
                    span.serial
                ));
                ctx.prompt.write_line(&error);
                if !delete_all {
                    ctx.prompt
                        .write("Delete the object and continue? (y/n/a): ");
                    let pressed = ctx.prompt.read_line()?;
                    if pressed.eq_ignore_ascii_case("a") {
                        delete_all = true;
                    } else if !pressed.eq_ignore_ascii_case("y") {
                        return Err(PersistError::Failed("Deserialization failed.".into()));
                    }
                }
                to_delete.push(span.serial);
            }
        }
        for s in to_delete {
            state_mut(w).entries.shift_remove(&s);
        }
        Ok(())
    }

    fn serialize(&mut self, w: &World) -> std::result::Result<Snapshot, PersistError> {
        let mut types = TypeTable::new();
        let mut bytes = SaveWriter::new(true);
        let mut records = Vec::new();
        if let Some(s) = state(w) {
            for (serial, e) in &s.entries {
                let index = types.register(if e.is_large() { LARGE_NAME } else { SMALL_NAME })?;
                let start = bytes.position() as usize;
                write_entry(e, &mut bytes);
                records.push((
                    index,
                    *serial,
                    e.created.ticks(),
                    start,
                    bytes.position() as usize,
                ));
            }
        }
        let mut file = EntityFileWriter::new(w.save_start_time.ticks(), types.names(), &[]);
        let data = bytes.as_slice();
        for (index, serial, created, start, end) in records {
            file.add_record(index, serial, created, &data[start..end])?;
        }
        let (idx, bin) = file.finish();
        self.estimated = bin.len() as u64;
        Ok(Snapshot {
            name: "BOBEntries".to_owned(),
            idx: Some(idx),
            bin,
        })
    }

    fn estimated_size(&self) -> u64 {
        self.estimated
    }
}

// ---- the book ----

/// What `BulkOrderBook` adds to an `Item`.
#[derive(Clone, Debug, Default)]
pub struct BookData {
    pub item_count: i32,
    pub level: i32,
    pub book_name: Option<String>,
    pub filter: BobFilter,
    pub entries: Vec<Serial>,
}

pub fn book(w: &World, id: ItemId) -> Option<&BookData> {
    w.item(id)?.ext.get::<BookData>()
}

fn book_mut(w: &mut World, id: ItemId) -> Option<&mut BookData> {
    w.item_mut(id)?.ext.get_mut::<BookData>()
}

type IsForSaleFn = fn(&World, MobileId, ItemId) -> Option<bool>;
/// `PlayerVendor` and what the book needs of it.
#[derive(Default)]
pub struct BobHooks {
    /// `RootParent as PlayerVendor`.
    pub root_vendor: Option<fn(&World, ItemId) -> Option<MobileId>>,
    /// `pv.GetVendorItem(book)?.IsForSale`: `None` when the vendor does not sell the book.
    pub is_for_sale: Option<IsForSaleFn>,
    /// `pv.HoldGold += price`.
    pub add_hold_gold: Option<fn(&mut World, MobileId, i32)>,
}

fn root_vendor(w: &World, book: ItemId) -> Option<MobileId> {
    w.resources
        .get::<BobHooks>()?
        .root_vendor
        .and_then(|f| f(w, book))
}

/// `GetVendorItem(book)`: `Some(IsForSale)` if the vendor has the book.
fn vendor_item_for_sale(w: &World, pv: MobileId, book: ItemId) -> Option<bool> {
    w.resources
        .get::<BobHooks>()?
        .is_for_sale
        .and_then(|f| f(w, pv, book))
}

fn new_book(w: &mut World) -> ItemId {
    let id = ops::new_item_with_id(w, &BULK_ORDER_BOOK, 0x2259);
    ops::set_loot_type(w, id, LootType::Blessed);
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(BookData {
            level: 4, /* SecureLevel.CoOwners */
            ..Default::default()
        });
    }
    id
}

/// `InvalidateContainers(parent)`.
fn invalidate_containers(w: &mut World, mut parent: Option<EntityId>) {
    while let Some(p) = parent {
        match p {
            EntityId::Item(i) => {
                ops::invalidate_properties(w, i);
                parent = w.item(i).and_then(|x| x.parent);
            }
            EntityId::Mobile(m) => {
                muo_server::mobile::part2::properties::invalidate_properties(w, m);
                return;
            }
        }
    }
}

/// `InvalidateItems()`.
fn invalidate_items(w: &mut World, id: ItemId) {
    if let Some(EntityId::Mobile(m)) = ops::root_parent(w, id) {
        muo_server::mobile::base::update_totals(w, m);
        let parent = w.item(id).and_then(|i| i.parent);
        invalidate_containers(w, parent);
    }
}

/// `AddEntry(entry)`.
fn add_entry(w: &mut World, id: ItemId, entry: Serial) {
    if let Some(b) = book_mut(w, id) {
        b.entries.push(entry);
    }
    ops::invalidate_properties(w, id);
}

/// `RemoveEntry(entry)`: also forgets the entry (`Delete`).
fn remove_entry(w: &mut World, id: ItemId, entry: Serial) {
    if let Some(b) = book_mut(w, id) {
        b.entries.retain(|e| *e != entry);
    }
    ops::invalidate_properties(w, id);
}

fn on_double_click(w: &mut World, id: ItemId, from: MobileId) {
    let loc = ops::get_world_location(w, id);
    let in_range = w
        .mobile(from)
        .is_some_and(|m| (m.ty.in_range_point3d)(w, from, loc, 2));
    if !in_range {
        muo_server::mobile::part2::messages::local_overhead_message_number(
            w,
            from,
            muo_primitives::MessageType::REGULAR,
            0x3B2,
            1019045,
            "",
        ); // I can't reach that.
    } else if book(w, id).is_none_or(|b| b.entries.is_empty()) {
        deps::send_localized_message(w, from, 1062381); // The book is empty.
    } else if player_mobile::data(w, from).is_some() {
        send_bob_gump(w, from, id, 0, None);
    }
}

fn on_double_click_secure_trade(w: &mut World, id: ItemId, from: MobileId) {
    let loc = ops::get_world_location(w, id);
    let in_range = w
        .mobile(from)
        .is_some_and(|m| (m.ty.in_range_point3d)(w, from, loc, 2));
    if !in_range {
        deps::send_localized_message(w, from, 500446); // That is too far away.
    } else if book(w, id).is_none_or(|b| b.entries.is_empty()) {
        deps::send_localized_message(w, from, 1062381); // The book is empty.
    } else {
        send_bob_gump(w, from, id, 0, None);
        // The other side of the trade sees the book too.
        let other = muo_server::item::secure_trade_container::trade_of(w, id).and_then(|t| {
            let (a, b) = deps::trade_parties(w, t);
            if a == Some(from) {
                b
            } else if b == Some(from) {
                a
            } else {
                None
            }
        });
        if let Some(o) = other {
            send_bob_gump(w, o, id, 0, None);
        }
    }
}

fn on_drag_drop(w: &mut World, id: ItemId, from: MobileId, dropped: ItemId) -> bool {
    if !super::is_bod(w, dropped) {
        deps::send_localized_message(w, from, 1062388); // That is not a bulk order deed.
        return false;
    }
    if !in_pack(w, id, from) {
        deps::send_localized_message(w, from, 1062385); // You must have the book in your backpack to add deeds to it.
        return false;
    }
    let Some(pack) = deps::find_item_on_layer(w, from, Layer::Backpack) else {
        return false;
    };
    if !container::check_hold_items(w, pack, Some(from), dropped, true, true) {
        return false;
    }
    let count = book(w, id).map_or(0, |b| b.entries.len());
    if count < 500 {
        let entry = add_from_deed(w, dropped);
        add_entry(w, id, entry);
        let (n, item_count) =
            book(w, id).map_or((0, 0), |b| (b.entries.len() as i32, b.item_count));
        if n / 5 > item_count {
            if let Some(b) = book_mut(w, id) {
                b.item_count += 1;
            }
            invalidate_items(w, id);
        }
        let loc = ops::get_world_location(w, id);
        deps::send_sound(w, from, 0x42, loc);
        deps::send_localized_message(w, from, 1062386); // Deed added to book.
        if player_mobile::data(w, from).is_some() {
            send_bob_gump(w, from, id, 0, None);
        }
        if let Some(ty) = w.item(dropped).map(|i| i.ty) {
            (ty.delete)(w, dropped);
        }
        return true;
    }
    deps::send_localized_message(w, from, 1062387); // The book is full of deeds.
    false
}

fn get_total(w: &World, id: ItemId, t: TotalType) -> i32 {
    let total = (ITEM_BASE.get_total)(w, id, t);
    if t == TotalType::Items {
        book(w, id).map_or(0, |b| b.item_count)
    } else {
        total
    }
}

fn get_properties(w: &World, id: ItemId, list: &mut PropertyList) {
    (ITEM_BASE.get_properties)(w, id, list);
    let Some(b) = book(w, id) else { return };
    list.add_int(1062344, b.entries.len() as i32); // Deeds in book: ~1_val~
    if let Some(n) = b.book_name.as_deref().filter(|n| !n.is_empty()) {
        list.add_str(1062481, n); // Book Name: ~1_val~
    }
}

fn on_single_click(w: &mut World, id: ItemId, from: MobileId) {
    (ITEM_BASE.on_single_click)(w, id, from);
    let Some(b) = book(w, id).cloned() else {
        return;
    };
    ops::label_to_number(w, id, from, 1062344, &b.entries.len().to_string()); // Deeds in book: ~1_val~
    if let Some(n) = b.book_name.as_deref().filter(|n| !n.is_empty()) {
        ops::label_to_number(w, id, from, 1062481, n);
    }
}

fn get_context_menu_entries(
    w: &mut World,
    id: ItemId,
    from: MobileId,
    list: &mut Vec<muo_server::context_menu_entry::ContextMenuEntry>,
) {
    (ITEM_BASE.get_context_menu_entries)(w, id, from, list);
    // `from.CheckAlive()` without its message: the context menu only reads.
    if muo_server::mobile::base::alive(w, from) && in_pack(w, id, from) {
        let mut e = muo_server::context_menu_entry::ContextMenuEntry::new(6216, -1);
        e.on_click = Some(Rc::new(|w: &mut World, from, target| {
            let EntityId::Item(book) = target else { return };
            if !muo_server::mobile::part2::stats::check_alive(w, from, true)
                || !in_pack(w, book, from)
            {
                return;
            }
            // NameBookPrompt
            let p = muo_server::prompt::SimplePrompt::new(
                Some(Rc::new(move |w: &mut World, from, text: &str| {
                    let mut text = text.to_owned();
                    if text.chars().count() > 40 {
                        text = text.chars().take(40).collect();
                    }
                    if muo_server::mobile::part2::stats::check_alive(w, from, true)
                        && in_pack(w, book, from)
                    {
                        let name = muo_primitives::utility::fix_html(text.trim());
                        if let Some(b) = book_mut(w, book) {
                            b.book_name = Some(name);
                        }
                        ops::invalidate_properties(w, book);
                        deps::send_localized_message(w, from, 1062480); // The bulk order book's name has been changed.
                    }
                })),
                false,
            );
            let pid = muo_server::prompt::add_prompt(w, p.into_prompt());
            muo_server::mobile::part1::props_b::set_prompt(w, from, Some(pid));
            deps::send_localized_message(w, from, 1062479); // Type in the new name of the book:
        }));
        list.push(e);
    }
    if let Some(e) = muo_content::items::common::hooks(w)
        .and_then(|h| h.secure_level_entry)
        .and_then(|f| f(w, from, id))
    {
        list.push(e);
    }
}

/// `OnAfterDuped(newItem)`: the filter and every entry, recreated.
fn on_after_duped(w: &mut World, id: ItemId, new_item: ItemId) {
    (ITEM_BASE.on_after_duped)(w, id, new_item);
    if !w.item(new_item).is_some_and(|i| i.ty.is(&BULK_ORDER_BOOK)) {
        return;
    }
    let Some(b) = book(w, id).cloned() else {
        return;
    };
    if let Some(n) = book_mut(w, new_item) {
        n.filter = b.filter.clone();
    }
    for s in b.entries {
        let Some(e) = entry(w, s).cloned() else {
            continue;
        };
        let Some(bod) = reconstruct(w, &e) else {
            continue;
        };
        let ne = add_from_deed(w, bod);
        add_entry(w, new_item, ne);
        if let Some(ty) = w.item(bod).map(|i| i.ty) {
            (ty.delete)(w, bod);
        }
    }
}

fn serialize(w: &World, id: ItemId, out: &mut dyn GenericWriter) {
    (ITEM_BASE.serialize)(w, id, out);
    let b = book(w, id).cloned().unwrap_or_default();
    out.write_i32(3);
    out.write_i32(b.item_count);
    out.write_enum(4, i64::from(b.level));
    out.write_opt_string(b.book_name.as_deref());
    b.filter.serialize(out);
    out.write_encoded_int(b.entries.len() as i32);
    for s in &b.entries {
        out.write_serial(*s);
    }
}

fn deserialize(w: &mut World, id: ItemId, r: &mut dyn GenericReader) -> Result<()> {
    (ITEM_BASE.deserialize)(w, id, r)?;
    let version = r.read_i32()?;
    let mut b = BookData {
        item_count: r.read_i32()?,
        ..Default::default()
    };
    b.level = if version < 3 {
        r.read_i32()?
    } else {
        r.read_enum(4)? as i32
    };
    b.book_name = r.read_string()?;
    b.filter = BobFilter::deserialize(r)?;
    let count = r.read_encoded_int()?;
    if version < 3 {
        // `Deserialize(reader, version)`: the entries are inside the book
        let mut entries = Vec::new();
        for _ in 0..count.max(0) {
            let kind = r.read_encoded_int()?;
            let large = match kind {
                0 => true,
                1 => false,
                _ => continue,
            };
            let mut e = BobEntry::blank(w.now, large);
            read_entry(w, r, &mut e)?;
            let st = state_mut(w);
            let s = st.new_serial();
            st.entries.insert(s, e);
            entries.push(s);
        }
        b.entries = entries;
    } else {
        for _ in 0..count.max(0) {
            let s = r.read_serial()?;
            if state(w).is_some_and(|st| st.entries.contains_key(&s)) {
                b.entries.push(s);
            }
        }
    }
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(b);
    }
    Ok(())
}

/// `Server.Engines.BulkOrders.BulkOrderBook`, `[SerializationGenerator(3, false)]`.
pub static BULK_ORDER_BOOK: ItemType = ItemType {
    full_name: "Server.Engines.BulkOrders.BulkOrderBook",
    parent: Some(&ITEM_BASE),
    construct: Some(new_book),
    default_weight: |w, id| {
        crate::template_keys::item_float(w, id, "defaultWeight", ITEM_BASE.default_weight)
    },
    on_double_click,
    on_double_click_secure_trade,
    on_drag_drop,
    get_total,
    get_properties,
    on_single_click,
    get_context_menu_entries,
    on_after_duped,
    serialize,
    deserialize,
    ..ITEM_BASE
};

// ---- BOBGump ----

const LABEL: i32 = 0x7FFF;

/// `GetMaterialName`: a cliloc number, or 0 for `TextDefinition.Empty`.
pub fn material_name(w: &World, material: i32, deed_type: i32, item_type: Option<&str>) -> i32 {
    match deed_type {
        BOD_SMITH => match material {
            mat::DULL_COPPER..=mat::VALORITE => 1018332 + (material - mat::DULL_COPPER),
            _ => 1062226,
        },
        BOD_TAILOR => match material {
            mat::SPINED => 1062236,
            mat::HORNED => 1062237,
            mat::BARBED => 1062238,
            _ if classify(w, BOD_TAILOR, item_type) == BulkGenericType::Leather => 1062235,
            _ => 1044286,
        },
        _ => 0,
    }
}

/// `CheckFilter(mat, amountMax, isLarge, reqExc, deedType, itemType)`.
#[allow(clippy::too_many_arguments)]
fn check_filter_raw(
    w: &World,
    f: &BobFilter,
    material: i32,
    amount_max: i32,
    is_large: bool,
    req_exc: bool,
    deed_type: i32,
    item_type: Option<&str>,
) -> bool {
    if f.is_default() {
        return true;
    }
    if (f.quality == 1 && req_exc) || (f.quality == 2 && !req_exc) {
        return false;
    }
    if (f.quantity == 1 && amount_max != 10)
        || (f.quantity == 2 && amount_max != 15)
        || (f.quantity == 3 && amount_max != 20)
    {
        return false;
    }
    if (f.ty == 1 && is_large) || (f.ty == 2 && !is_large) {
        return false;
    }
    let generic = || classify(w, deed_type, item_type);
    match f.material {
        1 => deed_type == BOD_SMITH,
        2 => deed_type == BOD_TAILOR,
        3 => material == mat::NONE && generic() == BulkGenericType::Iron,
        4 => material == mat::DULL_COPPER,
        5 => material == mat::SHADOW_IRON,
        6 => material == mat::COPPER,
        7 => material == mat::BRONZE,
        8 => material == mat::GOLD,
        9 => material == mat::AGAPITE,
        10 => material == mat::VERITE,
        11 => material == mat::VALORITE,
        12 => material == mat::NONE && generic() == BulkGenericType::Cloth,
        13 => material == mat::NONE && generic() == BulkGenericType::Leather,
        14 => material == mat::SPINED,
        15 => material == mat::HORNED,
        16 => material == mat::BARBED,
        _ => true,
    }
}

/// A row of the gump's list: what the layout and the responses read of an entry.
#[derive(Clone)]
struct Row {
    serial: Serial,
    e: BobEntry,
    /// `CheckFilter(entry)`.
    visible: bool,
    /// `GetMaterialName(..)` per sub-entry (one for a small entry).
    names: Vec<i32>,
}

struct BobGump {
    from: MobileId,
    book: ItemId,
    page: usize,
    /// `List`: the serials that passed the filter when it was built.
    list: Vec<Serial>,
    rows: Vec<Row>,
    can_drop: bool,
    can_buy: bool,
    can_price: bool,
    filter_default: bool,
    own_filter: bool,
}

/// `new BOBGump(from, book)` shown on `page`; `list` is the carried `List` (`None` builds it, `ResetList`).
fn send_bob_gump(
    w: &mut World,
    from: MobileId,
    book_id: ItemId,
    page: usize,
    list: Option<Vec<Serial>>,
) {
    let own = player_mobile::data(w, from).is_some_and(|d| d.flag(PlayerFlag::USE_OWN_FILTER));
    let filter = if own {
        player_mobile::data(w, from)
            .map(|d| d.bob_filter.clone())
            .unwrap_or_default()
    } else {
        book(w, book_id)
            .map(|b| b.filter.clone())
            .unwrap_or_default()
    };
    let all = book(w, book_id)
        .map(|b| b.entries.clone())
        .unwrap_or_default();
    let visible = |w: &World, s: Serial| -> bool {
        let Some(e) = entry(w, s) else { return false };
        match &e.kind {
            BobKind::Large { entries } => check_filter_raw(
                w,
                &filter,
                e.material,
                e.amount_max,
                true,
                e.require_exceptional,
                e.deed_type,
                entries.first().and_then(|x| x.item_type),
            ),
            BobKind::Small { item_type, .. } => check_filter_raw(
                w,
                &filter,
                e.material,
                e.amount_max,
                false,
                e.require_exceptional,
                e.deed_type,
                *item_type,
            ),
        }
    };
    let list = list.unwrap_or_else(|| all.iter().copied().filter(|&s| visible(w, s)).collect());
    let rows = list
        .iter()
        .filter_map(|&s| {
            let e = entry(w, s)?.clone();
            let names = match &e.kind {
                BobKind::Large { entries } => entries
                    .iter()
                    .map(|x| material_name(w, e.material, e.deed_type, x.item_type))
                    .collect(),
                BobKind::Small { item_type, .. } => {
                    vec![material_name(w, e.material, e.deed_type, *item_type)]
                }
            };
            Some(Row {
                serial: s,
                visible: visible(w, s),
                e,
                names,
            })
        })
        .collect();
    let can_drop = in_pack(w, book_id, from);
    let pv = root_vendor(w, book_id);
    let mut can_buy = pv.is_some();
    if let Some(pv) = pv {
        can_buy = vendor_item_for_sale(w, pv, book_id) == Some(false);
    }
    let g = BobGump {
        from,
        book: book_id,
        page,
        list,
        rows,
        can_drop,
        can_buy,
        can_price: can_drop || pv.is_some(),
        filter_default: filter.is_default(),
        own_filter: own,
    };
    // `SendTo`: the filter gump is closed first.
    if let Some(ns) = deps::net_state_of(w, from) {
        muo_content::gumps::close_gump::<BobFilterGump>(w, ns);
    }
    let x = (624 - if g.can_price { 600 } else { 516 }) / 2;
    send_gump_to_mobile(w, from, Box::new(DynamicGumpObj::new(x, 24, g)), false);
}

impl BobGump {
    fn index_for_page(&self, page: usize) -> usize {
        let mut index = 0;
        for _ in 0..page {
            index += self.count_for_index(index);
        }
        index
    }

    fn count_for_index(&self, index: usize) -> usize {
        let (mut slots, mut count) = (0, 0);
        for r in self.rows.iter().skip(index) {
            if r.visible {
                let add = r.e.rows();
                if slots + add > 10 {
                    break;
                }
                slots += add;
            }
            count += 1;
        }
        count
    }

    fn page_for_index(&self, index: usize, size_dropped: usize) -> usize {
        if index == 0 {
            return 0;
        }
        let (mut count, mut page) = (0, 0);
        let mut i = 0;
        while i < index && i < self.rows.len() {
            let r = &self.rows[i];
            i += 1;
            if !r.visible {
                continue;
            }
            let add = r.e.rows();
            count += add;
            if count > 10 {
                page += 1;
                count = add;
            }
        }
        // now we are on the page of the bod preceding the dropped one
        i += 1;
        if count + size_dropped > 10 {
            while i < self.rows.len() && count <= 10 {
                if self.rows[i].visible {
                    count += self.rows[i].e.rows();
                }
                i += 1;
            }
            if count > 10 {
                page += 1;
            }
        }
        page
    }
}

impl DynamicGumpLogic for BobGump {
    const TYPE_NAME: &'static str = "Server.Engines.BulkOrders.BOBGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        let index = self.index_for_page(self.page);
        let count = self.count_for_index(index);
        let mut table_index = 0;
        let (can_drop, can_buy, can_price) = (self.can_drop, self.can_buy, self.can_price);
        let width = if can_price { 600 } else { 516 };

        b.add_page(0);
        b.add_background(10, 10, width, 439, 5054);
        b.add_image_tiled(18, 20, width - 17, 420, 2624);
        if can_price {
            b.add_image_tiled(573, 64, 24, 352, 200);
            b.add_image_tiled(493, 64, 78, 352, 1416);
        }
        if can_drop {
            b.add_image_tiled(24, 64, 32, 352, 1416);
        }
        b.add_image_tiled(58, 64, 36, 352, 200);
        b.add_image_tiled(96, 64, 133, 352, 1416);
        b.add_image_tiled(231, 64, 80, 352, 200);
        b.add_image_tiled(313, 64, 100, 352, 1416);
        b.add_image_tiled(415, 64, 76, 352, 200);

        let range = index..(index + count).min(self.rows.len());
        for r in &self.rows[range.clone()] {
            if !r.visible {
                continue;
            }
            b.add_image_tiled(
                24,
                94 + table_index * 32,
                if can_price { 573 } else { 489 },
                2,
                2624,
            );
            table_index += r.e.rows() as i32;
        }

        b.add_alpha_region(18, 20, width - 17, 420);
        b.add_image(5, 5, 10460, 0);
        b.add_image(width - 15, 5, 10460, 0);
        b.add_image(5, 424, 10460, 0);
        b.add_image(width - 15, 424, 10460, 0);

        let hl = |b: &mut DynamicGumpBuilder, x, y, wd, h, n, c| {
            b.add_html_localized_color(x, y, wd, h, n, c, false, false)
        };
        hl(
            b,
            if can_price { 266 } else { 224 },
            32,
            200,
            32,
            1062220,
            LABEL,
        ); // Bulk Order Book
        hl(b, 63, 64, 200, 32, 1062213, LABEL); // Type
        hl(b, 147, 64, 200, 32, 1062214, LABEL); // Item
        hl(b, 246, 64, 200, 32, 1062215, LABEL); // Quality
        hl(b, 336, 64, 200, 32, 1062216, LABEL); // Material
        hl(b, 429, 64, 200, 32, 1062217, LABEL); // Amount
        b.add_button(35, 32, 4005, 4007, 1);
        hl(b, 70, 32, 200, 32, 1062476, LABEL); // Set Filter

        let fx = if can_price { 470 } else { 386 };
        if self.filter_default {
            hl(b, fx, 32, 120, 32, 1062475, 16927); // Using No Filter
        } else if self.own_filter {
            hl(b, fx, 32, 120, 32, 1062451, 16927); // Using Your Filter
        } else {
            hl(b, fx, 32, 120, 32, 1062230, 16927); // Using Book Filter
        }

        b.add_button(375, 416, 4017, 4018, 0);
        hl(b, 410, 416, 120, 20, 1011441, LABEL); // EXIT

        if can_drop {
            hl(b, 26, 64, 50, 32, 1062212, LABEL); // Drop
        }
        if can_price {
            hl(b, 516, 64, 200, 32, 1062218, LABEL); // Price
            if can_buy {
                hl(b, 576, 64, 200, 32, 1062219, LABEL); // Buy
            } else {
                hl(b, 576, 64, 200, 32, 1062227, LABEL); // Set
                b.add_button(450, 416, 4005, 4007, 4);
                b.add_html(
                    485,
                    416,
                    120,
                    20,
                    "<BASEFONT COLOR=#FFFFFF>Price all</BASEFONT>",
                    false,
                    false,
                );
            }
        }

        table_index = 0;
        if self.page > 0 {
            b.add_button(75, 416, 4014, 4016, 2);
            hl(b, 110, 416, 150, 20, 1011067, LABEL); // Previous page
        }
        if self.index_for_page(self.page + 1) < self.rows.len() {
            b.add_button(225, 416, 4005, 4007, 3);
            hl(b, 260, 416, 150, 20, 1011066, LABEL); // Next page
        }

        for (i, r) in self
            .rows
            .iter()
            .enumerate()
            .take(range.end)
            .skip(range.start)
        {
            if !r.visible {
                continue;
            }
            let e = &r.e;
            let n = (i * 2) as i32;
            let exceptional = if e.require_exceptional {
                1060636
            } else {
                1011542
            };
            match &e.kind {
                BobKind::Large { entries } => {
                    let mut y = 96 + table_index * 32;
                    if can_drop {
                        b.add_button(35, y + 2, 5602, 5606, 5 + n);
                    }
                    if can_drop || (can_buy && e.price > 0) {
                        b.add_button(579, y + 2, 2117, 2118, 6 + n);
                        b.add_label(495, y, 1152, &e.price.to_string());
                    }
                    hl(b, 61, y, 50, 32, 1062225, LABEL); // Large
                    for (j, sub) in entries.iter().enumerate() {
                        hl(b, 103, y, 130, 32, sub.number, LABEL);
                        hl(b, 235, y, 80, 20, exceptional, LABEL);
                        if r.names[j] != 0 {
                            hl(b, 316, y, 100, 20, r.names[j], 1152);
                        }
                        b.add_label(
                            421,
                            y,
                            1152,
                            &format!("{} / {}", sub.amount_cur, e.amount_max),
                        );
                        table_index += 1;
                        y += 32;
                    }
                }
                BobKind::Small {
                    amount_cur, number, ..
                } => {
                    let y = 96 + table_index * 32;
                    table_index += 1;
                    if can_drop {
                        b.add_button(35, y + 2, 5602, 5606, 5 + n);
                    }
                    if can_drop || (can_buy && e.price > 0) {
                        b.add_button(579, y + 2, 2117, 2118, 6 + n);
                        b.add_label(495, y, 1152, &e.price.to_string());
                    }
                    hl(b, 61, y, 50, 32, 1062224, LABEL); // Small
                    hl(b, 103, y, 130, 32, *number, LABEL);
                    hl(b, 235, y, 80, 20, exceptional, LABEL);
                    if r.names[0] != 0 {
                        hl(b, 316, y, 100, 20, r.names[0], 1152);
                    }
                    b.add_label(421, y, 1152, &format!("{} / {}", amount_cur, e.amount_max));
                }
            }
        }
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let from = self.from;
        if w.net_states.get(&sender).and_then(|n| n.mobile) != Some(from) {
            return;
        }
        let index = info.button_id;
        match index {
            0 => {}
            1 => send_filter_gump(w, from, self.book),
            2 => {
                if self.page > 0 {
                    send_bob_gump(w, from, self.book, self.page - 1, Some(self.list.clone()));
                }
            }
            3 => {
                if self.index_for_page(self.page + 1) < self.rows.len() {
                    send_bob_gump(w, from, self.book, self.page + 1, Some(self.list.clone()));
                }
            }
            4 => {
                if in_pack(w, self.book, from) {
                    self.begin_price_prompt(w, None);
                    deps::send_message(w, from, "Type in a price for all deeds in the book:");
                }
            }
            _ => self.entry_button(w, index - 5),
        }
    }
}

impl BobGump {
    fn begin_price_prompt(&self, w: &mut World, entry: Option<Serial>) {
        let (from, book_id, list, page) = (self.from, self.book, self.list.clone(), self.page);
        let p = muo_server::prompt::SimplePrompt::new(
            Some(Rc::new(move |w: &mut World, from, text: &str| {
                if let Some(e) = entry
                    && !book(w, book_id).is_some_and(|b| b.entries.contains(&e))
                {
                    deps::send_localized_message(w, from, 1062382); // The deed selected is not available.
                    return;
                }
                let price = muo_primitives::parse::to_i32(text); // Utility.ToInt32
                if !(0..=250_000_000).contains(&price) {
                    deps::send_localized_message(w, from, 1062390); // The price you requested is outrageous!
                    return;
                }
                let targets: Vec<Serial> = match entry {
                    Some(e) => vec![e],
                    None => list.clone(),
                };
                for s in targets {
                    if entry.is_none() && !book(w, book_id).is_some_and(|b| b.entries.contains(&s))
                    {
                        continue;
                    }
                    if let Some(e) = state_mut(w).entries.get_mut(&s) {
                        e.price = price;
                    }
                }
                deps::send_localized_message(w, from, 1062384); // Deed price set.
                send_bob_gump(w, from, book_id, page, Some(list.clone()));
            })),
            false,
        );
        let pid = muo_server::prompt::add_prompt(w, p.into_prompt());
        muo_server::mobile::part1::props_b::set_prompt(w, from, Some(pid));
    }

    /// The default arm of `OnResponse`: a drop or a price/buy button of the entry at `index / 2`.
    fn entry_button(&mut self, w: &mut World, index: i32) {
        let from = self.from;
        let kind = index % 2;
        let index = index / 2;
        if index < 0 || index as usize >= self.rows.len() {
            return;
        }
        let index = index as usize;
        let (serial, entry) = (self.rows[index].serial, self.rows[index].e.clone());
        if !book(w, self.book).is_some_and(|b| b.entries.contains(&serial)) {
            deps::send_localized_message(w, from, 1062382); // The deed selected is not available.
            return;
        }
        let size = entry.rows();
        if in_pack(w, self.book, from) {
            if kind == 0 {
                // Drop
                let Some(item) = reconstruct(w, &entry) else {
                    return;
                };
                let pack = deps::find_item_on_layer(w, from, Layer::Backpack);
                let weight = ops::pile_weight(w, item);
                let ok = pack.is_some_and(|p| {
                    container::check_hold_default(w, p, Some(from), item, true, true, 0, weight)
                });
                if !ok {
                    deps::send_localized_message(w, from, 503204); // You do not have room in your backpack for this
                    send_bob_gump(w, from, self.book, self.page, None);
                } else {
                    muo_server::mobile::part2::items::add_to_backpack(w, from, item);
                    deps::send_localized_message(w, from, 1045152); // The bulk order deed has been placed in your backpack.
                    remove_entry(w, self.book, serial);
                    let (n, item_count) = book(w, self.book)
                        .map_or((0, 0), |b| (b.entries.len() as i32, b.item_count));
                    if n / 5 < item_count {
                        if let Some(b) = book_mut(w, self.book) {
                            b.item_count -= 1;
                        }
                        invalidate_items(w, self.book);
                    }
                    if n > 0 {
                        let page = self.page_for_index(index, size);
                        send_bob_gump(w, from, self.book, page, None);
                    } else {
                        deps::send_localized_message(w, from, 1062381); // The book is empty.
                    }
                }
            } else {
                // Set Price | Buy
                self.begin_price_prompt(w, Some(serial));
                deps::send_localized_message(w, from, 1062383); // Type in a price for the deed:
            }
        } else if let Some(pv) = root_vendor(w, self.book) {
            if vendor_item_for_sale(w, pv, self.book) != Some(false) {
                return;
            }
            let price = entry.price;
            if price == 0 {
                deps::send_localized_message(w, from, 1062382); // The deed selected is not available.
            } else if book(w, self.book).is_some_and(|b| !b.entries.is_empty()) {
                let page = self.page_for_index(index, size);
                send_buy_gump(w, from, self.book, serial, price, page, self.list.clone());
            } else {
                deps::send_localized_message(w, from, 1062381); // The book is empty
            }
        }
    }
}

// ---- BOBFilterGump ----

const MATERIAL_FILTERS: [(i32, i32); 18] = [
    (1044067, 1),
    (1062226, 3),
    (1018332, 4),
    (1018333, 5),
    (1018334, 6),
    (1018335, 7),
    (0, 0),
    (1018336, 8),
    (1018337, 9),
    (1018338, 10),
    (1018339, 11),
    (0, 0),
    (1044094, 2),
    (1044286, 12),
    (1062235, 13),
    (1062236, 14),
    (1062237, 15),
    (1062238, 16),
];
const TYPE_FILTERS: [(i32, i32); 3] = [(1062229, 0), (1062224, 1), (1062225, 2)];
const QUALITY_FILTERS: [(i32, i32); 3] = [(1062229, 0), (1011542, 1), (1060636, 2)];
const AMOUNT_FILTERS: [(i32, i32); 4] = [(1062229, 0), (1049706, 1), (1016007, 2), (1062239, 3)];

fn filters(kind: usize) -> &'static [(i32, i32)] {
    match kind {
        0 => &TYPE_FILTERS,
        1 => &QUALITY_FILTERS,
        2 => &MATERIAL_FILTERS,
        _ => &AMOUNT_FILTERS,
    }
}

struct BobFilterGump {
    from: MobileId,
    book: ItemId,
    f: BobFilter,
    own: bool,
}

fn current_filter(w: &World, from: MobileId, book_id: ItemId) -> (BobFilter, bool) {
    let own = player_mobile::data(w, from).is_some_and(|d| d.flag(PlayerFlag::USE_OWN_FILTER));
    let f = if own {
        player_mobile::data(w, from)
            .map(|d| d.bob_filter.clone())
            .unwrap_or_default()
    } else {
        book(w, book_id)
            .map(|b| b.filter.clone())
            .unwrap_or_default()
    };
    (f, own)
}

/// `new BOBFilterGump(from, book)`.
fn send_filter_gump(w: &mut World, from: MobileId, book_id: ItemId) {
    let (f, own) = current_filter(w, from, book_id);
    send_gump_to_mobile(
        w,
        from,
        Box::new(DynamicGumpObj::new(
            12,
            24,
            BobFilterGump {
                from,
                book: book_id,
                f,
                own,
            },
        )),
        false,
    );
}

#[allow(clippy::too_many_arguments)]
fn add_filter_list(
    b: &mut DynamicGumpBuilder,
    x: i32,
    y: i32,
    x_offsets: &[i32],
    y_offset: i32,
    list: &[(i32, i32)],
    x_widths: &[i32],
    value: i32,
    filter_index: i32,
) {
    for (i, &(number, v)) in list.iter().enumerate() {
        if number == 0 {
            continue;
        }
        let len = x_offsets.len();
        let selected = v == value || (i % len == 0 && value == 0);
        let px = x + x_offsets[i % len];
        let py = y + (i / len) as i32 * y_offset;
        b.add_html_localized_color(
            px + 35,
            py,
            x_widths[i % len],
            32,
            number,
            if selected { 16927 } else { LABEL },
            false,
            false,
        );
        b.add_button(px, py, 4005, 4007, 4 + filter_index + i as i32 * 4);
    }
}

impl DynamicGumpLogic for BobFilterGump {
    const TYPE_NAME: &'static str = "Server.Engines.BulkOrders.BOBFilterGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        let f = &self.f;
        let hl = |b: &mut DynamicGumpBuilder, x, y, wd, h, n, c| {
            b.add_html_localized_color(x, y, wd, h, n, c, false, false)
        };
        b.add_page(0);
        b.add_background(10, 10, 600, 439, 5054);
        b.add_image_tiled(18, 20, 583, 420, 2624);
        b.add_alpha_region(18, 20, 583, 420);
        b.add_image(5, 5, 10460, 0);
        b.add_image(585, 5, 10460, 0);
        b.add_image(5, 424, 10460, 0);
        b.add_image(585, 424, 10460, 0);

        hl(b, 270, 32, 200, 32, 1062223, LABEL); // Filter Preference
        hl(b, 26, 64, 120, 32, 1062228, LABEL); // Bulk Order Type
        add_filter_list(
            b,
            25,
            96,
            &[0, 75, 170],
            40,
            &TYPE_FILTERS,
            &[50, 50, 70, 50],
            f.ty,
            0,
        );
        hl(b, 320, 64, 50, 32, 1062215, LABEL); // Quality
        add_filter_list(
            b,
            320,
            96,
            &[0, 75, 170],
            40,
            &QUALITY_FILTERS,
            &[50, 50, 70, 50],
            f.quality,
            1,
        );
        hl(b, 26, 160, 120, 32, 1062232, LABEL); // Material Type
        add_filter_list(
            b,
            25,
            192,
            &[0, 105, 210, 305, 390, 485],
            40,
            &MATERIAL_FILTERS,
            &[80, 50, 50, 50, 50, 50],
            f.material,
            2,
        );
        hl(b, 26, 320, 120, 32, 1062217, LABEL); // Amount
        add_filter_list(
            b,
            25,
            352,
            &[0, 75, 180, 275],
            40,
            &AMOUNT_FILTERS,
            &[50, 50, 70, 50],
            f.quantity,
            3,
        );

        hl(
            b,
            75,
            416,
            120,
            32,
            1062477,
            if self.own { LABEL } else { 16927 },
        ); // Set Book Filter
        b.add_button(40, 416, 4005, 4007, 1);
        hl(
            b,
            235,
            416,
            120,
            32,
            1062478,
            if self.own { 16927 } else { LABEL },
        ); // Set Your Filter
        b.add_button(200, 416, 4005, 4007, 2);
        hl(b, 405, 416, 120, 32, 1062231, LABEL); // Clear Filter
        b.add_button(370, 416, 4005, 4007, 3);
        hl(b, 540, 416, 50, 32, 1011046, LABEL); // APPLY
        b.add_button(505, 416, 4017, 4018, 0);
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let from = self.from;
        if w.net_states.get(&sender).and_then(|n| n.mobile) != Some(from) {
            return;
        }
        let book_id = self.book;
        let set_own = |w: &mut World, v: bool| {
            if let Some(d) = player_mobile::data_mut(w, from) {
                d.set_flag(PlayerFlag::USE_OWN_FILTER, v);
            }
        };
        // `f` is whichever filter is in use right now.
        let edit = |w: &mut World, g: &dyn Fn(&mut BobFilter)| {
            let (_, own) = current_filter(w, from, book_id);
            if own {
                if let Some(d) = player_mobile::data_mut(w, from) {
                    g(&mut d.bob_filter);
                }
            } else if let Some(b) = book_mut(w, book_id) {
                g(&mut b.filter);
            }
        };
        match info.button_id {
            0 => send_bob_gump(w, from, book_id, 0, None),
            1 => {
                set_own(w, false);
                send_filter_gump(w, from, book_id);
            }
            2 => {
                set_own(w, true);
                send_filter_gump(w, from, book_id);
            }
            3 => {
                edit(w, &|f| f.clear());
                send_filter_gump(w, from, book_id);
            }
            i => {
                let i = i - 4;
                let kind = (i % 4) as usize;
                let index = (i / 4) as usize;
                let list = filters(kind);
                if let Some(&(number, v)) = list.get(index) {
                    if number == 0 {
                        return;
                    }
                    edit(w, &move |f| match kind {
                        0 => f.ty = v,
                        1 => f.quality = v,
                        2 => f.material = v,
                        _ => f.quantity = v,
                    });
                    send_filter_gump(w, from, book_id);
                }
            }
        }
    }
}

// ---- BODBuyGump ----

struct BodBuyGump {
    book: ItemId,
    entry: Serial,
    price: i32,
    page: usize,
    list: Vec<Serial>,
}

fn send_buy_gump(
    w: &mut World,
    from: MobileId,
    book_id: ItemId,
    entry: Serial,
    price: i32,
    page: usize,
    list: Vec<Serial>,
) {
    send_gump_to_mobile(
        w,
        from,
        Box::new(StaticGumpObj::new(
            100,
            200,
            BodBuyGump {
                book: book_id,
                entry,
                price,
                page,
                list,
            },
        )),
        false,
    );
}

fn n0(v: i32) -> String {
    // `{price:N0}`
    let s = v.abs().to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    if v < 0 { format!("-{out}") } else { out }
}

impl StaticGumpLogic for BodBuyGump {
    const TYPE_NAME: &'static str = "Server.Engines.BulkOrders.BODBuyGump";

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        b.add_page(0);
        b.add_background(100, 10, 300, 150, 5054);
        b.add_html_localized(125, 20, 250, 24, 1019070, false, false); // You have agreed to purchase:
        b.add_html_localized(125, 45, 250, 24, 1045151, false, false); // a bulk order deed
        b.add_html_localized(125, 70, 250, 24, 1019071, false, false); // for the amount of:
        b.add_label_placeholder(125, 95, 0, "price");
        b.add_button(250, 130, 4005, 4007, 1);
        b.add_html_localized(282, 130, 100, 24, 1011012, false, false); // CANCEL
        b.add_button(120, 130, 4005, 4007, 2);
        b.add_html_localized(152, 130, 100, 24, 1011036, false, false); // OKAY
    }

    fn build_strings(&self, s: &mut GumpStrings) {
        s.set_string_slot("price", &n0(self.price));
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let Some(pm) = w
            .net_states
            .get(&sender)
            .and_then(|n| n.mobile)
            .filter(|&m| player_mobile::data(w, m).is_some())
        else {
            return;
        };
        if info.button_id != 2 {
            deps::send_localized_message(w, pm, 503207); // Cancelled purchase.
            return;
        }
        let book_id = self.book;
        let Some(pv) = root_vendor(w, book_id) else {
            deps::send_localized_message(w, pm, 1062382); // The deed selected is not available.
            return;
        };
        if !book(w, book_id).is_some_and(|b| b.entries.contains(&self.entry)) {
            muo_server::mobile::part2::speech::say_to_number(w, pv, pm, 1062382, ""); // The deed selected is not available.
            return;
        }
        let price = if vendor_item_for_sale(w, pv, book_id) == Some(false) {
            entry(w, self.entry).map_or(0, |e| e.price)
        } else {
            0
        };
        if price != self.price {
            muo_content::mx::say_to(
                w,
                pv,
                pm,
                "The price has been been changed. If you like, you may offer to purchase the item again.",
            );
            return;
        }
        if price == 0 {
            muo_server::mobile::part2::speech::say_to_number(w, pv, pm, 1062382, "");
            return;
        }
        let Some(e) = entry(w, self.entry).cloned() else {
            return;
        };
        let Some(item) = reconstruct(w, &e) else {
            return;
        };
        let name = w
            .mobile(pm)
            .and_then(|m| (m.ty.name)(w, pm))
            .unwrap_or_default();
        muo_server::mobile::part2::messages::say(w, pv, false, &name);
        let pack = deps::find_item_on_layer(w, pm, Layer::Backpack);
        let weight = ops::pile_weight(w, item);
        let room = pack.is_some_and(|p| {
            container::check_hold_default(w, p, Some(pm), item, true, true, 0, weight)
        });
        let delete = |w: &mut World, item: ItemId| {
            if let Some(ty) = w.item(item).map(|i| i.ty) {
                (ty.delete)(w, item);
            }
        };
        if !room {
            muo_server::mobile::part2::speech::say_to_number(w, pv, pm, 503204, ""); // You do not have room in your backpack for this
            send_bob_gump(w, pm, book_id, self.page, Some(self.list.clone()));
            delete(w, item);
            return;
        }
        let pack = pack.expect("room implies a pack");
        let paid =
            container::consume_total(w, pack, &muo_content::items::gold::GOLD, price, true, None)
                || muo_content::vendor::banker::withdraw(w, pm, price);
        if paid {
            remove_entry(w, book_id, self.entry);
            if let Some(f) = w.resources.get::<BobHooks>().and_then(|h| h.add_hold_gold) {
                f(w, pv, price);
            }
            muo_server::mobile::part2::items::add_to_backpack(w, pm, item);
            deps::send_localized_message(w, pm, 1045152); // The bulk order deed has been placed in your backpack.
            let (n, item_count) =
                book(w, book_id).map_or((0, 0), |b| (b.entries.len() as i32, b.item_count));
            if n / 5 < item_count {
                if let Some(b) = book_mut(w, book_id) {
                    b.item_count -= 1;
                }
                invalidate_items(w, book_id);
            }
            if n > 0 {
                send_bob_gump(w, pm, book_id, self.page, None);
            } else {
                deps::send_localized_message(w, pm, 1062381); // The book is empty.
            }
        } else {
            muo_server::mobile::part2::speech::say_to_number(w, pv, pm, 503205, ""); // You cannot afford this item.
            delete(w, item);
        }
    }
}

// ---- wiring ----

pub fn configure(w: &mut World) {
    w.persistence
        .register(3, Box::new(BobPersistence::default()));
}

pub fn register(types: &mut TypeRegistry) {
    types.register_item(&BULK_ORDER_BOOK, &[]);
    // `BulkOrderBook : ISecurable`: the house's secure menu reads and sets its `Level`.
    types.register_phase_default(
        muo_server::type_registry::Phase::Configure,
        "Server.Items.BulkOrderBook.Securable",
        |w| {
            let d = muo_content::items::house_deps::deps_mut(w);
            d.securables.push(|w, id| {
                w.item(id)
                    .is_some_and(|i| i.ty.is(&BULK_ORDER_BOOK))
                    .then(|| book(w, id).map_or(0, |b| b.level) as u8)
            });
            d.securable_setters.push(|w, id, level| {
                if !w.item(id).is_some_and(|i| i.ty.is(&BULK_ORDER_BOOK)) {
                    return false;
                }
                if let Some(b) = book_mut(w, id) {
                    b.level = i32::from(level);
                }
                true
            });
        },
    );
}

#[cfg(test)]
pub fn write_entry_for_test(e: &BobEntry, out: &mut dyn GenericWriter) {
    write_entry(e, out);
}

#[cfg(test)]
pub fn read_entry_for_test(w: &World, r: &mut dyn GenericReader, e: &mut BobEntry) -> Result<()> {
    read_entry(w, r, e)
}
