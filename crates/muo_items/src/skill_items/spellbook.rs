//! `Spellbook` and its six subclasses (MysticSpellbook, NecromancerSpellbook,
//! SpellweavingBook, BookOfBushido, BookOfChivalry, BookOfNinjitsu). State: `muo_content::items::ext_bags::SpellbookData`
//! (so `items::attr` reaches the attribute bags); the class constants (`SpellbookType`, `BookOffset`, `BookCount`) are
//! [`SpellbookClass`]. [`configure`] sets the hooks other systems call (scroll to book, the 0xBF/0x1C spell packets,
//! slayer scalars, `ICraftable`).
//!
//! `OnCraft` of a Magery 80+ crafter calls `BaseRunicTool.ApplyAttributesTo(spellbook, ..)`: it runs through
//! [`SpellbookHooks::apply_runic`] (set in `wiring.rs`).

use std::collections::HashMap;

use muo_content::aos::{AosAttribute, AttrKind};
use muo_content::context_menus::ContextMenuHooks;
use muo_content::craft;
use muo_content::items::equip_hooks::{CraftArgs, hooks as equip_hooks};
use muo_content::items::ext_bags::SpellbookData;
use muo_content::items::slayer::{self, SlayerName};
use muo_content::items::{attr, skill_bonuses_add_to, skill_bonuses_remove};
use muo_content::network::incoming_game::GameHooks;
use muo_content::player_mobile::hooks::design_check;
use muo_content::props::{Ctor, Kind, Param, Value, add_ctors};
use muo_content::spells::registry::{self, Registered};
use muo_content::utilities::types::{Numeric, PropType};
use muo_io::{GenericReader, GenericWriter, Result};
use muo_server::World;
use muo_server::ext::Ext;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, LootType, base, deps, ops};
use muo_server::layer::Layer;
use muo_server::mobile::part2::{effects, messages};
use muo_server::property_list::PropertyList;
use muo_server::type_registry::{Phase, TypeRegistry};

use super::{hue, read_version, write_version};

/// `SpellbookType`.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(i32)]
pub enum SpellbookType {
    Invalid = -1,
    Regular = 0,
    Necromancer,
    Paladin,
    Ninja,
    Samurai,
    Arcanist,
    Mystic,
}

/// `BookQuality.Exceptional`.
pub const QUALITY_EXCEPTIONAL: i32 = 1;

/// The virtuals of a spellbook class that are not `Item` virtuals.
#[derive(Copy, Clone)]
pub struct SpellbookClass {
    pub spellbook_type: SpellbookType,
    pub book_offset: i32,
    pub book_count: fn(&World) -> i32,
}

pub static SPELLBOOK_CLASS: SpellbookClass = SpellbookClass {
    spellbook_type: SpellbookType::Regular,
    book_offset: 0,
    book_count: |_| 64,
};

pub fn class(w: &World, id: ItemId) -> &'static SpellbookClass {
    w.item(id)
        .and_then(|i| i.ty.family::<SpellbookClass>())
        .unwrap_or(&SPELLBOOK_CLASS)
}

pub fn data(w: &World, id: ItemId) -> Option<&SpellbookData> {
    w.item(id)?.ext.get::<SpellbookData>()
}

pub fn data_mut(w: &mut World, id: ItemId) -> Option<&mut SpellbookData> {
    w.item_mut(id)?.ext.get_mut::<SpellbookData>()
}

/// `item is Spellbook`.
pub fn is_spellbook(w: &World, id: ItemId) -> bool {
    w.item(id).is_some_and(|i| i.ty.is(&SPELLBOOK))
}

pub fn content(w: &World, id: ItemId) -> u64 {
    data(w, id).map_or(0, |d| d.content)
}

/// `Content = value`: the setter counts the spells.
pub fn set_content(w: &mut World, id: ItemId, value: u64) {
    if let Some(d) = data_mut(w, id) {
        d.content = value;
        d.spell_count = value.count_ones() as i32;
    }
    ops::invalidate_properties(w, id);
}

fn set_crafter(w: &mut World, id: ItemId, value: Option<String>) {
    if let Some(d) = data_mut(w, id)
        && d.crafter != value
    {
        d.crafter = value;
        ops::invalidate_properties(w, id);
    }
}

/// `Quality = value`.
pub fn set_quality(w: &mut World, id: ItemId, value: i32) {
    if let Some(d) = data_mut(w, id)
        && d.quality != value
    {
        d.quality = value;
        ops::invalidate_properties(w, id);
    }
}

/// `HasSpell(spellID)`.
pub fn has_spell(w: &World, id: ItemId, spell_id: i32) -> bool {
    let c = class(w, id);
    let s = spell_id - c.book_offset;
    s >= 0 && s < (c.book_count)(w) && content(w, id) & (1u64 << s) != 0
}

/// `Slayer` and `Slayer2` (`ISlayer`).
pub fn slayers(w: &World, id: ItemId) -> Option<(i32, i32)> {
    data(w, id).map(|d| (d.slayer, d.slayer2))
}

/// `new Spellbook(content, itemID)` of class `ty`.
pub fn new_spellbook(w: &mut World, ty: &'static ItemType, content: u64, item_id: i32) -> ItemId {
    let id = ops::new_item_with_id(w, ty, item_id);
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(SpellbookData::default());
    }
    (ty.set_layer)(w, id, Layer::OneHanded);
    ops::set_loot_type(w, id, LootType::Blessed);
    // The setter is calculating the spell count
    set_content(w, id, content);
    id
}

// ---- finding a book ----

/// `Spellbook._table`: the books last found for each mobile.
#[derive(Default)]
struct BookTable(HashMap<MobileId, Vec<ItemId>>);

/// `GetTypeForSpell(spellID)`.
pub fn type_for_spell(spell_id: i32) -> SpellbookType {
    match spell_id {
        0..=63 => SpellbookType::Regular,
        100..=116 => SpellbookType::Necromancer,
        200..=209 => SpellbookType::Paladin,
        400..=405 => SpellbookType::Samurai,
        500..=507 => SpellbookType::Ninja,
        600..=616 => SpellbookType::Arcanist,
        677..=692 => SpellbookType::Mystic,
        _ => SpellbookType::Invalid,
    }
}

/// `Find(from, spellID)`.
pub fn find(w: &mut World, from: MobileId, spell_id: i32) -> Option<ItemId> {
    find_typed(w, from, spell_id, type_for_spell(spell_id))
}

/// `Find(from, spellID, type)`.
pub fn find_typed(
    w: &mut World,
    from: MobileId,
    spell_id: i32,
    ty: SpellbookType,
) -> Option<ItemId> {
    let deleted = w.mobile(from).is_none_or(|m| m.deleted);
    if deleted {
        w.resources.get_mut::<BookTable>().0.remove(&from);
        return None;
    }
    let mut search_again = false;
    let cached = w.resources.get_mut::<BookTable>().0.remove(&from);
    let mut list = match cached {
        Some(l) => {
            search_again = true;
            l
        }
        None => find_all_spellbooks(w, from),
    };
    let mut book = find_in_list(w, &mut list, from, spell_id, ty);
    if book.is_none() && search_again {
        list = find_all_spellbooks(w, from);
        book = find_in_list(w, &mut list, from, spell_id, ty);
    }
    w.resources.get_mut::<BookTable>().0.insert(from, list);
    book
}

fn find_in_list(
    w: &World,
    list: &mut Vec<ItemId>,
    from: MobileId,
    spell_id: i32,
    ty: SpellbookType,
) -> Option<ItemId> {
    let pack = w.mobile(from).and_then(|m| m.backpack);
    let mut i = list.len();
    while i > 0 {
        i -= 1;
        let book = list[i];
        let ok = w.item(book).is_some_and(|b| {
            !b.deleted()
                && (b.parent == Some(EntityId::Mobile(from))
                    || pack.is_some_and(|p| b.parent == Some(EntityId::Item(p))))
        }) && validate(w, book, spell_id, ty);
        if ok {
            return Some(book);
        }
        list.remove(i);
    }
    None
}

/// `FindAllSpellbooks(from)`.
pub fn find_all_spellbooks(w: &World, from: MobileId) -> Vec<ItemId> {
    let mut list = Vec::new();
    if let Some(b) = find_equipped(w, from) {
        list.push(b);
    }
    if let Some(pack) = w.mobile(from).and_then(|m| m.backpack) {
        for &i in w.item(pack).map(|p| p.items.as_slice()).unwrap_or(&[]) {
            if is_spellbook(w, i) {
                list.push(i);
            }
        }
    }
    list
}

/// `FindEquippedSpellbook(from)`.
pub fn find_equipped(w: &World, from: MobileId) -> Option<ItemId> {
    deps::find_item_on_layer(w, from, Layer::OneHanded).filter(|&i| is_spellbook(w, i))
}

/// `ValidateSpellbook(book, spellID, type)`.
pub fn validate(w: &World, book: ItemId, spell_id: i32, ty: SpellbookType) -> bool {
    class(w, book).spellbook_type == ty && (spell_id == -1 || has_spell(w, book, spell_id))
}

// ---- requests from the client ----

/// `OpenSpellbookRequest(from, typeID)`.
pub fn open_spellbook_request(w: &mut World, from: MobileId, type_id: i32) {
    if !design_check(w, from) {
        return; // They are customizing
    }
    let ty = match type_id {
        2 => SpellbookType::Necromancer,
        3 => SpellbookType::Paladin,
        4 => SpellbookType::Ninja,
        5 => SpellbookType::Samurai,
        6 => SpellbookType::Arcanist,
        7 => SpellbookType::Mystic,
        _ => SpellbookType::Regular,
    };
    if let Some(book) = find_typed(w, from, -1, ty) {
        display_to(w, book, from);
    }
}

/// `TargetedSpell(from, target, spellId)`.
pub fn targeted_spell(w: &mut World, from: MobileId, _target: Option<EntityId>, spell_id: i16) {
    if !design_check(w, from) {
        return; // They are customizing
    }
    let spell_id = i32::from(spell_id);
    let book = find(w, from, spell_id);
    if !book.is_some_and(|b| has_spell(w, b, spell_id)) {
        deps::send_localized_message(w, from, 500015); // You do not have that spell!
        return;
    }
    cast_found(w, from, spell_id, false);
}

fn cast_found(w: &mut World, from: MobileId, spell_id: i32, disabled_message: bool) {
    if let Some(mv) = registry::registry(w).special_move(spell_id) {
        muo_content::spells::special_move::set_current_move(w, from, Some(mv));
    } else {
        match registry::new_spell_by_id(w, spell_id, from, None) {
            Some(sp) => {
                muo_content::spells::spell::cast(w, sp);
            }
            None if disabled_message => deps::send_localized_message(w, from, 502345), // temporarily disabled
            None => {}
        }
    }
}

/// `CastSpellRequest(from, spellID, item)`.
pub fn cast_spell_request(w: &mut World, from: MobileId, spell_id: i32, item: Option<ItemId>) {
    if !design_check(w, from) {
        return; // They are customizing
    }
    // Early rejection by spell ID before instantiation
    let player_below_admin = w.mobile(from).is_some_and(|m| {
        m.ty.is(&muo_content::player_mobile::PLAYER_MOBILE)
            && m.access_level < muo_server::mobile::AccessLevel::Administrator
    });
    if player_below_admin
        && let Some(Registered::Spell(t)) = registry::registry(w).type_of(spell_id)
        && let Some(f) = muo_content::spells::hooks::hooks(w).and_then(|h| h.spell_blocked_reason)
        && let Some(reason) = f(w, t.full_name)
    {
        messages::send_message_hue(w, from, 0x22, reason.as_deref().unwrap_or(""));
        return;
    }
    let mut book = item.filter(|&i| is_spellbook(w, i));
    if !book.is_some_and(|b| has_spell(w, b, spell_id)) {
        book = find(w, from, spell_id);
    }
    if book.is_some_and(|b| has_spell(w, b, spell_id)) {
        cast_found(w, from, spell_id, true);
    } else {
        deps::send_localized_message(w, from, 500015); // You do not have that spell!
    }
}

// ---- display ----

/// `DisplayTo(to)`.
pub fn display_to(w: &mut World, id: ItemId, to: MobileId) {
    // The client must know about the spellbook or it will crash!
    let Some(ns) = deps::net_state_of(w, to) else {
        return;
    };
    if w.cannot_send_packets(ns) {
        return;
    }
    match w.item(id).and_then(|i| i.parent) {
        None => {
            if let Some(ty) = w.item(id).map(|i| i.ty) {
                (ty.send_world_packet_to)(w, id, ns, &[]);
            }
        }
        Some(EntityId::Item(_)) => {
            muo_server::packets::container::send_container_content_update(w, ns, id)
        }
        Some(EntityId::Mobile(_)) => muo_server::packets::equipment::send_equip_update(w, ns, id),
    }
    let (serial, graphic, offset, bits) = {
        let Some(i) = w.item(id) else { return };
        (
            i.serial,
            i.item_id,
            class(w, id).book_offset + 1,
            content(w, id),
        )
    };
    muo_server::packets::container::send_display_spellbook(w, ns, serial);
    let aos = muo_content::core::aos(w);
    muo_server::packets::container::send_spellbook_content(
        w, ns, aos, serial, graphic, offset, bits,
    );
}

fn get_properties(w: &World, id: ItemId, list: &mut PropertyList) {
    base::get_properties_base(w, id, list);
    let Some(d) = data(w, id) else { return };
    if d.quality == QUALITY_EXCEPTIONAL {
        list.add(1063341); // exceptional
    }
    if let Some(t) = &d.engraved_text {
        list.add_str(1072305, t); // Engraved: ~1_INSCRIPTION~
    }
    if let Some(c) = &d.crafter {
        list.add_str(1050043, c); // crafted by ~1_NAME~
    }
    let aos = muo_content::core::aos(w);
    muo_content::aos::add_skill_bonus_properties(&d.skill_bonuses, list, aos);
    for s in [d.slayer, d.slayer2] {
        if let Some(name) = SlayerName::from_i32(s)
            && let Some((entry, _)) = slayer::entry_by_name(w, name)
        {
            list.add(slayer::title(w, entry));
        }
    }
    muo_content::aos::add_attribute_properties(
        &d.attributes,
        list,
        aos,
        muo_content::core::ml(w),
        0,
        0,
        0,
    );
    list.add_int(1042886, d.spell_count); // ~1_NUMBERS_OF_SPELLS~ Spells
}

fn on_single_click(w: &mut World, id: ItemId, from: MobileId) {
    base::on_single_click_base(w, id, from);
    let Some(d) = data(w, id) else { return };
    let (crafter, count) = (d.crafter.clone(), d.spell_count);
    if let Some(c) = crafter {
        ops::label_to_number(w, id, from, 1050043, &c); // crafted by ~1_NAME~
    }
    ops::label_to_number(w, id, from, 1042886, &count.to_string());
}

fn on_double_click(w: &mut World, id: ItemId, from: MobileId) {
    let pack = w.mobile(from).and_then(|m| m.backpack);
    let parent = w.item(id).and_then(|i| i.parent);
    if parent == Some(EntityId::Mobile(from))
        || pack.is_some_and(|p| parent == Some(EntityId::Item(p)))
    {
        display_to(w, id, from);
    } else {
        // The spellbook must be in your backpack (and not in a container within) to open.
        deps::send_localized_message(w, from, 500207);
    }
}

// ---- equipping, trading, dropping ----

fn allow_secure_trade(
    w: &mut World,
    id: ItemId,
    from: MobileId,
    to: MobileId,
    new_owner: MobileId,
    accepted: bool,
) -> bool {
    if let Some(f) = equip_hooks(w).and_then(|h| h.ethic_check_trade)
        && !f(w, from, to, new_owner, id)
    {
        return false;
    }
    base::allow_secure_trade_base(w, id, from, to, new_owner, accepted)
}

fn can_equip(w: &mut World, id: ItemId, from: MobileId) -> bool {
    if let Some(f) = equip_hooks(w).and_then(|h| h.ethic_check_equip)
        && !f(w, from, id)
    {
        return false;
    }
    muo_server::mobile::part1::combat::can_begin_action(w, from, "Server.Items.BaseWeapon")
        && base::can_equip_base(w, id, from)
}

/// `OnDragDrop(from, dropped)`: a scroll of this book's kind is read into it.
fn on_drag_drop(w: &mut World, id: ItemId, from: MobileId, dropped: ItemId) -> bool {
    if !super::scrolls::is_spell_scroll(w, dropped) {
        return false;
    }
    let sid = super::scrolls::spell_id(w, dropped).unwrap_or(0);
    if type_for_spell(sid) != class(w, id).spellbook_type {
        return false;
    }
    if has_spell(w, id, sid) {
        deps::send_localized_message(w, from, 500179); // That spell is already present in that spellbook.
        return false;
    }
    let c = class(w, id);
    let val = sid - c.book_offset;
    if val >= 0 && val < (c.book_count)(w) {
        if let Some(d) = data_mut(w, id) {
            d.content |= 1u64 << val;
            d.spell_count += 1;
        }
        ops::invalidate_properties(w, id);
        base::consume(w, dropped, 1);
        let loc = ops::get_world_location(w, id);
        deps::send_sound(w, from, 0x249, loc);
        return w.item(dropped).is_none_or(|i| i.deleted());
    }
    false
}

/// The `AddToSpellbookEntry.InternalTarget.OnTarget` body for a `Spellbook` target (`Spellbook book`'s own
/// `OnDragDrop` does the type check, `HasSpell`, `Content |=`, `Consume` and the 0x249 sound).
fn add_scroll_to_book(w: &mut World, from: MobileId, scroll: ItemId, book: ItemId) -> bool {
    if !is_spellbook(w, book) {
        return false;
    }
    // `from.CheckAlive() && !m_Scroll.Deleted && m_Scroll.Movable && m_Scroll.Amount >= 1 && m_Scroll.CheckItemUse(from)`
    if !muo_server::mobile::part2::stats::check_alive(w, from, true) {
        return true;
    }
    let usable = w
        .item(scroll)
        .is_some_and(|i| !i.deleted() && i.movable() && i.amount >= 1);
    let use_ok = usable
        && w.item(scroll)
            .map(|i| i.ty)
            .is_some_and(|ty| (ty.check_item_use)(w, scroll, from, scroll));
    if use_ok {
        on_drag_drop(w, book, from, scroll);
    }
    true
}

fn on_after_duped(w: &mut World, id: ItemId, new_item: ItemId) {
    let Some(src) = data(w, id) else { return };
    let (a, s) = (src.attributes.clone(), src.skill_bonuses.clone());
    if let Some(d) = data_mut(w, new_item) {
        d.attributes = a;
        d.skill_bonuses = s;
    }
}

fn stat_bonuses(w: &World, id: ItemId) -> (i32, i32, i32) {
    (
        attr(w, id, AttrKind::Attributes, AosAttribute::BonusStr as u32),
        attr(w, id, AttrKind::Attributes, AosAttribute::BonusDex as u32),
        attr(w, id, AttrKind::Attributes, AosAttribute::BonusInt as u32),
    )
}

fn check_stat_timers(w: &mut World, m: MobileId) {
    if let Some(ty) = w.mobile(m).map(|m| m.ty) {
        (ty.check_stat_timers)(w, m);
    }
}

fn on_added(w: &mut World, id: ItemId, parent: EntityId) {
    if muo_content::core::aos(w)
        && let EntityId::Mobile(from) = parent
    {
        skill_bonuses_add_to(w, id, from);
        let (s, d, i) = stat_bonuses(w, id);
        muo_content::items::add_stat_mods(w, id, from, s, d, i);
        check_stat_timers(w, from);
    }
}

fn on_removed(w: &mut World, id: ItemId, parent: EntityId) {
    if muo_content::core::aos(w)
        && let EntityId::Mobile(from) = parent
    {
        skill_bonuses_remove(w, id);
        muo_content::items::remove_stat_mods(w, id, from);
        check_stat_timers(w, from);
    }
}

// ---- ICraftable ----

/// A property-count table: `n` copies of each count, in order.
const fn rep<const N: usize>(runs: [(i32, usize); 4]) -> [i32; N] {
    let mut out = [0; N];
    let (mut i, mut r) = (0, 0);
    while r < 4 {
        let mut k = 0;
        while k < runs[r].1 {
            out[i] = runs[r].0;
            i += 1;
            k += 1;
        }
        r += 1;
    }
    out
}

const LEGEND: [i32; 52] = rep([(0, 21), (1, 15), (2, 10), (3, 6)]);
const ELDER: [i32; 34] = rep([(0, 15), (1, 10), (2, 6), (3, 3)]);
const GRAND: [i32; 20] = rep([(0, 10), (1, 6), (2, 3), (3, 1)]);
const MASTER: [i32; 10] = rep([(0, 6), (1, 3), (2, 1), (3, 0)]);
const ADEPT: [i32; 4] = rep([(0, 3), (1, 1), (2, 0), (3, 0)]);

/// `OnCraft(quality, makersMark, from, ..)`.
pub fn on_craft(w: &mut World, id: ItemId, a: &CraftArgs) -> i32 {
    let magery = muo_server::skills::base_fixed(w, a.from, muo_server::skills::SkillName::Magery);
    if magery >= 800 {
        let (counts, min, max): (&[i32], i32, i32) = if magery >= 1000 {
            let c: &[i32] = if magery >= 1200 {
                &LEGEND
            } else if magery >= 1100 {
                &ELDER
            } else {
                &GRAND
            };
            (c, 55, 75)
        } else if magery >= 900 {
            (&MASTER, 25, 45)
        } else {
            (&ADEPT, 0, 15)
        };
        let n = muo_primitives::utility::random(&mut w.rng, counts.len() as i32) as usize;
        if let Some(f) = w
            .resources
            .get::<SpellbookHooks>()
            .and_then(|h| h.apply_runic)
        {
            f(w, id, counts[n], min, max);
        }
    }
    if a.makers_mark {
        let name = muo_server::mobile::part1::props_b::raw_name(w, a.from);
        set_crafter(w, id, name);
    }
    set_quality(w, id, a.quality - 1);
    a.quality
}

type ApplyRunicFn = fn(&mut World, ItemId, i32, i32, i32);
/// `BaseRunicTool.ApplyAttributesTo(spellbook, true, 0, propertyCount, minIntensity, maxIntensity)`: set by the
/// runic tool code.
#[derive(Default)]
pub struct SpellbookHooks {
    pub apply_runic: Option<ApplyRunicFn>,
}

fn craftable(w: &mut World, id: ItemId, a: &CraftArgs) -> Option<i32> {
    is_spellbook(w, id).then(|| on_craft(w, id, a))
}

// ---- save ----

fn serialize(w: &World, id: ItemId, out: &mut dyn GenericWriter) {
    (ITEM_BASE.serialize)(w, id, out);
    out.write_i32(6);
    let d = data(w, id).cloned().unwrap_or_default();
    out.write_enum(4, i64::from(d.quality));
    out.write_opt_string(d.engraved_text.as_deref());
    out.write_opt_string(d.crafter.as_deref());
    out.write_enum(4, i64::from(d.slayer));
    out.write_enum(4, i64::from(d.slayer2));
    d.attributes.serialize(out);
    d.skill_bonuses.serialize(out);
    out.write_u64(d.content);
    out.write_i32(d.spell_count);
}

fn deserialize(w: &mut World, id: ItemId, r: &mut dyn GenericReader) -> Result<()> {
    use muo_server::serialization_ext::ReadEntityExt;
    (ITEM_BASE.deserialize)(w, id, r)?;
    let version = r.read_i32()?;
    let mut d = SpellbookData::default();
    if version < 6 {
        // `Deserialize(reader, version)`: the crafter was a Mobile whose name is copied on a zero-delay timer.
        d.quality = i32::from(r.read_u8()?);
        d.engraved_text = r.read_string()?;
        d.crafter = r
            .read_mobile(w)?
            .and_then(|m| muo_server::mobile::part1::props_b::raw_name(w, m));
        d.slayer = r.read_i32()?;
        d.slayer2 = r.read_i32()?;
    } else {
        d.quality = r.read_enum(4)? as i32;
        d.engraved_text = r.read_string()?;
        d.crafter = r.read_string()?;
        d.slayer = r.read_enum(4)? as i32;
        d.slayer2 = r.read_enum(4)? as i32;
    }
    d.attributes = muo_content::aos::Attrs::deserialize(r)?;
    d.skill_bonuses = muo_content::aos::Attrs::deserialize(r)?;
    d.content = r.read_u64()?;
    d.spell_count = r.read_i32()?;
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(d);
    }
    // The bonuses of a worn book.
    if let Some(EntityId::Mobile(m)) = w.item(id).and_then(|i| i.parent) {
        if muo_content::core::aos(w) {
            skill_bonuses_add_to(w, id, m);
        }
        let (s, d, i) = stat_bonuses(w, id);
        muo_content::items::add_stat_mods(w, id, m, s, d, i);
        check_stat_timers(w, m);
    }
    Ok(())
}

/// `Server.Items.Spellbook`.
pub static SPELLBOOK: ItemType = ItemType {
    full_name: "Server.Items.Spellbook",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| new_spellbook(w, &SPELLBOOK, 0, 0xEFA)),
    family: Some(&SPELLBOOK_CLASS),
    default_weight: crate::class_props::default_weight,
    display_weight: |_, _| false,
    display_loot_type: |w, _| muo_content::core::aos(w),
    allow_equipped_cast: |_, _, _| true,
    allow_secure_trade,
    can_equip,
    on_drag_drop,
    on_after_duped,
    on_added,
    on_removed,
    get_properties,
    on_single_click,
    on_double_click,
    serialize,
    deserialize,
    ..ITEM_BASE
};

const ULONG: Kind = Kind::Plain(PropType::Numeric(Numeric::ULong));
const INT: Kind = Kind::Plain(PropType::Numeric(Numeric::Int));
static SPELLBOOK_CTOR: [Ctor; 1] = [Ctor {
    access: None,
    params: &[
        Param {
            name: "content",
            kind: ULONG,
            default: Some(Value::Int(0)),
            ty: "",
        },
        Param {
            name: "itemID",
            kind: INT,
            default: Some(Value::Int(0xEFA)),
            ty: "",
        },
    ],
    make: |w, v| EntityId::Item(new_spellbook(w, &SPELLBOOK, v[0].i64() as u64, v[1].i32())),
}];

macro_rules! books {
    ($($name:ident, $new:ident, $ctor:ident = $full:literal, $class:ident { $ty:ident, $off:expr, $count:expr }, $gid:expr,
        content $content:expr, layer $layer:expr, hue $hue:expr, encoded $enc:expr;)*) => {$(
        pub static $class: SpellbookClass = SpellbookClass {
            spellbook_type: SpellbookType::$ty,
            book_offset: $off,
            book_count: $count,
        };
        #[doc = concat!("`new ", $full, "(content)`.")]
        pub fn $new(w: &mut World, content: u64) -> ItemId {
            let id = new_spellbook(w, &$name, content, $gid);
            let layer: fn(&World) -> Layer = $layer;
            let l = layer(w);
            (SPELLBOOK.set_layer)(w, id, l);
            if $hue != 0 {
                hue(w, id, $hue);
            }
            id
        }
        static $ctor: [Ctor; 1] = [Ctor {
            access: None,
            params: &[Param { name: "content", kind: ULONG, default: Some(Value::Int($content)), ty: "" }],
            make: |w, v| EntityId::Item($new(w, v[0].i64() as u64)),
        }];
        #[doc = concat!("`", $full, "`.")]
        pub static $name: ItemType = ItemType {
            full_name: $full,
            parent: Some(&SPELLBOOK),
            construct: Some(|w| $new(w, $content)),
            family: Some(&$class),
            serialize: |w, id, out| {
                (SPELLBOOK.serialize)(w, id, out);
                write_version(out, $enc);
            },
            deserialize: |w, id, r| {
                (SPELLBOOK.deserialize)(w, id, r)?;
                read_version(r, $enc)?;
                Ok(())
            },
            ..SPELLBOOK
        };
    )*};
}

fn one_handed(_: &World) -> Layer {
    Layer::OneHanded
}
fn ml_one_handed(w: &World) -> Layer {
    if muo_content::core::ml(w) {
        Layer::OneHanded
    } else {
        Layer::Invalid
    }
}

books! {
    MYSTIC_SPELLBOOK, new_mystic, MYSTIC_CTOR = "Server.Items.MysticSpellbook", MYSTIC_CLASS { Mystic, 677, |_| 16 }, 0x2D9D,
        content 0, layer one_handed, hue 0, encoded false;
    NECROMANCER_SPELLBOOK, new_necromancer, NECRO_CTOR = "Server.Items.NecromancerSpellbook", NECRO_CLASS { Necromancer, 100, |w| if muo_content::core::se(w) { 17 } else { 16 } }, 0x2253,
        content 0, layer ml_one_handed, hue 0, encoded false;
    SPELLWEAVING_BOOK, new_spellweaving, WEAVING_CTOR = "Server.Items.SpellweavingBook", WEAVING_CLASS { Arcanist, 600, |_| 16 }, 0x2D50,
        content 0, layer one_handed, hue 0x8A2, encoded true;
    BOOK_OF_BUSHIDO, new_bushido, BUSHIDO_CTOR = "Server.Items.BookOfBushido", BUSHIDO_CLASS { Samurai, 400, |_| 6 }, 0x238C,
        content 0x3F, layer ml_one_handed, hue 0, encoded false;
    BOOK_OF_CHIVALRY, new_chivalry, CHIVALRY_CTOR = "Server.Items.BookOfChivalry", CHIVALRY_CLASS { Paladin, 200, |_| 10 }, 0x2252,
        content 0x3FF, layer ml_one_handed, hue 0, encoded false;
    BOOK_OF_NINJITSU, new_ninjitsu, NINJITSU_CTOR = "Server.Items.BookOfNinjitsu", NINJITSU_CLASS { Ninja, 500, |_| 8 }, 0x23A0,
        content 0xFF, layer ml_one_handed, hue 0, encoded false;
}

static ALL: [&ItemType; 7] = [
    &SPELLBOOK,
    &MYSTIC_SPELLBOOK,
    &NECROMANCER_SPELLBOOK,
    &SPELLWEAVING_BOOK,
    &BOOK_OF_BUSHIDO,
    &BOOK_OF_CHIVALRY,
    &BOOK_OF_NINJITSU,
];

/// Every spellbook class of this file.
pub fn all() -> &'static [&'static ItemType] {
    &ALL
}

/// `GetSlayerDamageScalar(defender)` of `Spell`.
fn slayer_damage_scalar(w: &mut World, caster: MobileId, defender: MobileId) -> f64 {
    let entry =
        |w: &World, s: i32| SlayerName::from_i32(s).and_then(|n| slayer::entry_by_name(w, n));
    let mut scalar = 1.0;
    if let Some(book) = find_equipped(w, caster) {
        let (s1, s2) = slayers(w, book).unwrap_or((0, 0));
        if entry(w, s1).is_some_and(|(e, _)| slayer::slays(w, e, defender))
            || entry(w, s2).is_some_and(|(e, _)| slayer::slays(w, e, defender))
        {
            effects::fixed_effect(w, defender, 0x37B9, 10, 5);
            scalar = 2.0;
        }
        let silver = SlayerName::Silver as i32;
        let ctx = muo_content::spells::helper::get_context(w, defender);
        // Every necromancer transformation other than horrific beast take an additional 25% damage
        if (s1 == silver || s2 == silver)
            && ctx.is_some_and(|c| c.ty != "Server.Spells.Necromancy.HorrificBeastSpell")
        {
            scalar += 0.25;
        }
        if scalar != 1.0 {
            return scalar;
        }
    }
    let def = find_equipped(w, defender)
        .and_then(|b| slayers(w, b))
        .or_else(|| {
            let weapon = muo_content::items::weapon_hooks::weapon_item(w, defender)?;
            muo_content::items::weapon::data(w, weapon).map(|d| (d.slayer, d.slayer2))
        });
    if let Some((s1, s2)) = def
        && (entry(w, s1).is_some_and(|(_, g)| slayer::opposition_super_slays(w, g, caster))
            || entry(w, s2).is_some_and(|(_, g)| slayer::opposition_super_slays(w, g, caster)))
    {
        scalar = 2.0;
    }
    scalar
}

fn configure(w: &mut World) {
    w.resources.get_mut::<ContextMenuHooks>().add_scroll_to_book = Some(add_scroll_to_book);
    {
        let h = w.resources.get_mut::<GameHooks>();
        h.open_spellbook_request = Some(open_spellbook_request);
        h.cast_spell_request = Some(cast_spell_request);
        h.targeted_spell = Some(targeted_spell);
    }
    w.resources
        .get_mut::<muo_content::items::weapon_hooks::WeaponHooks>()
        .equipped_spellbook_slayers = Some(|w, m| find_equipped(w, m).and_then(|b| slayers(w, b)));
    w.resources
        .get_mut::<muo_content::spells::hooks::SpellHooks>()
        .slayer_damage_scalar = Some(slayer_damage_scalar);
    w.resources
        .get_mut::<muo_content::skills::SkillHooks>()
        .fns
        .okay_to_hold_while_meditating = Some(|w, item| {
        is_spellbook(w, item)
            || ((muo_content::items::weapon::is_weapon(w, item)
                || muo_content::items::armor::is_armor(w, item))
                && attr(
                    w,
                    item,
                    AttrKind::Attributes,
                    AosAttribute::SpellChanneling as u32,
                ) != 0)
    });
    craft::hooks_mut(w).craftables.push(craftable);
    add_ctors(w, "Server.Items.Spellbook", &SPELLBOOK_CTOR);
    for (name, t) in [
        ("Server.Items.MysticSpellbook", &MYSTIC_CTOR),
        ("Server.Items.NecromancerSpellbook", &NECRO_CTOR),
        ("Server.Items.SpellweavingBook", &WEAVING_CTOR),
        ("Server.Items.BookOfBushido", &BUSHIDO_CTOR),
        ("Server.Items.BookOfChivalry", &CHIVALRY_CTOR),
        ("Server.Items.BookOfNinjitsu", &NINJITSU_CTOR),
    ] {
        add_ctors(w, name, t);
    }
    command(w);
}

fn command(w: &mut World) {
    use muo_server::commands::{CommandEventArgs, CommandSystem};
    use muo_server::mobile::AccessLevel;
    CommandSystem::register(
        w,
        "AllSpells",
        AccessLevel::GameMaster,
        &[],
        std::rc::Rc::new(|w: &mut World, e: &mut CommandEventArgs| {
            // Usage: AllSpells; Description: Completely fills a targeted spellbook with scrolls.
            all_spells_target(w, e.mobile);
            messages::send_message(w, e.mobile, "Target the spellbook to fill.");
        }),
    );
}

fn all_spells_target(w: &mut World, from: MobileId) {
    use muo_server::target::{TargetFlags, Targeted};
    muo_server::target::begin_target(w, from, -1, false, TargetFlags::NONE, |w, from, t| {
        if let Targeted::Entity(EntityId::Item(book)) = t
            && is_spellbook(w, book)
        {
            let count = (class(w, book).book_count)(w);
            set_content(
                w,
                book,
                if count == 64 {
                    u64::MAX
                } else {
                    (1u64 << count) - 1
                },
            );
            messages::send_message(w, from, "The spellbook has been filled.");
        } else {
            all_spells_target(w, from);
            messages::send_message(w, from, "That is not a spellbook. Try again.");
        }
    });
}

/// Registers the spellbook classes.
pub fn register(types: &mut TypeRegistry) {
    for t in ALL {
        types.register_item(t, &[]);
    }
    types.register_phase_default(
        Phase::Configure,
        "Server.Items.Spellbook.Configure",
        configure,
    );
}
