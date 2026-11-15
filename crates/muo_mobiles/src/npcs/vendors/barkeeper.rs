//! The player barkeeper, `BarkeeperGump` and `BarkeeperTitleGump`.
//! `PlayerBarkeeper` is a `BaseVendor` (`[SerializationGenerator(2, false)]`); `BarkeeperRumor` is its own
//! generated serializer (encoded version 0).
//!
//! The gump classes `BarkeeperHumanGump`/`BarkeeperNonHumanGump` are file-local; their client type ids are hashed
//! from the plain names.

use std::rc::Rc;

use muo_content::creature::CreatureType;
use muo_content::gumps::GumpButtonType;
use muo_content::gumps::base::{RelayInfo, StaticGumpLogic, StaticGumpObj};
use muo_content::gumps::builder::{GumpStrings, StaticGumpBuilder};
use muo_content::gumps::system::close_gump;
use muo_content::hooks::rnd;
use muo_content::vendor::{
    BASE_VENDOR, BASE_VENDOR_C, BASE_VENDOR_CLASS, ShoeType, VendorClass, data, init_body,
    init_outfit, load_sb_info, new_vendor,
};
use muo_content::{factory, mx};
use muo_io::{GenericReader, GenericWriter};
use muo_multis::house;
use muo_primitives::{MessageType, utility};
use muo_server::World;
use muo_server::context_menu_entry::ContextMenuEntry;
use muo_server::events::SpeechEventArgs;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::mobile::part1::{props_a, props_b};
use muo_server::mobile::part2::{items, messages};
use muo_server::mobile::{AccessLevel, MobileType};
use muo_server::net_state::NetStateId;
use muo_server::prompt::{Prompt, add_prompt};
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::timer::{self, TimerHandle};
use muo_server::type_registry::TypeRegistry;

use crate::npcs::townfolk::crier::{Owner, get_random_entry};

/// `PlayerBarkeeper.RumorCount`.
pub const RUMOR_COUNT: usize = 3;

/// `BarkeeperRumor`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Rumor {
    pub message: Option<String>,
    pub keyword: Option<String>,
}

/// What `PlayerBarkeeper` adds to a `BaseVendor`.
#[derive(Default)]
pub struct BarkeeperData {
    pub house: Option<ItemId>,
    pub owner: Option<MobileId>,
    /// `_rumors`: `None` is null, a slot `None` is a null rumor.
    pub rumors: Option<Vec<Option<Rumor>>>,
    pub tip_message: Option<String>,
    news_timer: TimerHandle,
}

pub fn bdata(w: &World, id: MobileId) -> Option<&BarkeeperData> {
    w.mobile(id)?.ext.get::<BarkeeperData>()
}

fn bdata_mut(w: &mut World, id: MobileId) -> Option<&mut BarkeeperData> {
    w.mobile_mut(id)?.ext.get_mut::<BarkeeperData>()
}

/// `m is PlayerBarkeeper`.
pub fn is_barkeeper(w: &World, id: MobileId) -> bool {
    bdata(w, id).is_some()
}

/// `House = value` (`OnHouseChanged`: the old house forgets the barkeeper, the new one lists it).
pub fn set_house(w: &mut World, id: MobileId, value: Option<ItemId>) {
    let old = bdata(w, id).and_then(|d| d.house);
    if old == value {
        return;
    }
    if let Some(d) = bdata_mut(w, id) {
        d.house = value;
    }
    if let Some(h) = old.and_then(|h| house::data_mut(w, h)) {
        h.player_barkeepers.retain(|&v| v != id);
    }
    if let Some(h) = value.and_then(|h| house::data_mut(w, h)) {
        h.player_barkeepers.push(id);
    }
}

/// `IsOwner(from)`.
pub fn is_owner(w: &World, id: MobileId, from: MobileId) -> bool {
    !mx::deleted(w, from)
        && !mx::deleted(w, id)
        && (mx::access_level(w, from) > AccessLevel::GameMaster
            || bdata(w, id).and_then(|d| d.owner) == Some(from))
}

// ---- construction and the vendor class ----

/// `new PlayerBarkeeper(owner, house)`.
pub fn make(w: &mut World, owner: Option<MobileId>, house_id: Option<ItemId>) -> MobileId {
    let id = new_vendor(w, &PLAYER_BARKEEPER, Some("the barkeeper"));
    if let Some(m) = w.mobile_mut(id) {
        m.ext.insert(BarkeeperData::default());
    }
    if let Some(d) = bdata_mut(w, id) {
        d.owner = owner;
    }
    set_house(w, id, house_id);
    load_sb_info(w, id);
    id
}

fn ctor(w: &mut World, v: &[muo_content::props::Value]) -> EntityId {
    use muo_content::props::Value;
    let owner = match v.first() {
        Some(Value::Entity(Some(EntityId::Mobile(m)))) => Some(*m),
        _ => None,
    };
    let house = match v.get(1) {
        Some(Value::Entity(Some(EntityId::Item(i)))) => Some(*i),
        _ => None,
    };
    EntityId::Mobile(make(w, owner, house))
}

fn add_hued(w: &mut World, id: MobileId, name: &str, hue: i32) {
    muo_content::vendor::add_hued(w, id, name, hue);
}

fn delete_item(w: &mut World, i: ItemId) {
    if let Some(t) = w.item(i).map(|i| i.ty) {
        (t.delete)(w, i);
    }
}

fn delete_backpack(w: &mut World, id: MobileId) {
    if let Some(p) = props_b::backpack(w, id) {
        delete_item(w, p);
    }
}

pub static PLAYER_BARKEEPER_V: VendorClass = VendorClass {
    init_sb_info: |w, id| {
        let ok = w.mobile(id).is_some_and(|m| {
            matches!(
                m.title.as_deref(),
                Some("the waiter" | "the barkeeper" | "the baker" | "the innkeeper" | "the chef")
            )
        });
        if ok {
            vec![super::sb::named(w, id, "SBPlayerBarkeeper")]
        } else {
            Vec::new()
        }
    },
    is_active_buyer: |_, _| false,
    is_active_seller: |w, id| data(w, id).is_some_and(|d| !d.sell_info.is_empty()),
    get_gender: |_, _| false,
    shoe_type: |w, _| {
        if rnd::bool(w) {
            ShoeType::ThighBoots
        } else {
            ShoeType::Boots
        }
    },
    init_outfit: |w, id| {
        init_outfit(w, id);
        let h = utility::random_bright_hue(&mut w.rng);
        add_hued(w, id, "Server.Items.HalfApron", h);
        delete_backpack(w, id);
    },
    init_body: |w, id| {
        init_body(w, id);
        let body = w.mobile(id).map_or(0, |m| m.cur_body().0);
        let hue = if body == 0x340 || body == 0x402 {
            0
        } else {
            0x83F4
        }; // hue is not random
        muo_server::mobile::base::set_hue(w, id, hue);
        delete_backpack(w, id);
    },
    ..BASE_VENDOR_CLASS
};

/// `CheckGold(from, dropped)`.
fn check_gold(w: &mut World, id: MobileId, from: MobileId, dropped: ItemId) -> bool {
    if !factory::item_is(w, dropped, "Server.Items.Gold") {
        return false;
    }
    let amount = w.item(dropped).map_or(0, |i| i.amount);
    let ns = w.mobile(from).and_then(|m| m.net_state);
    let say = |w: &mut World, t: &str| {
        messages::private_overhead_message(w, id, MessageType::REGULAR, 0x3B2, false, t, ns)
    };
    if amount > 50 {
        say(w, "I cannot accept so large a tip!");
    } else {
        let tip = bdata(w, id).and_then(|d| d.tip_message.clone());
        match tip.map(|t| t.trim().to_owned()).filter(|t| !t.is_empty()) {
            None => say(
                w,
                "It would not be fair of me to take your money and not offer you information in return.",
            ),
            Some(tip) => {
                let d = mx::get_direction_to(w, id, from);
                mx::set_direction(w, id, d);
                say(w, &tip);
                delete_item(w, dropped);
                return true;
            }
        }
    }
    false
}

pub static PLAYER_BARKEEPER_C: CreatureType = CreatureType {
    subclass: Some(&PLAYER_BARKEEPER_V),
    disallow_all_moves: |_, _| true,
    no_house_restrictions: |_, _| true,
    check_gold,
    ..BASE_VENDOR_C
};

fn handles_on_speech(w: &World, id: MobileId, from: MobileId) -> bool {
    mx::in_range(w, id, from, 3) || (BASE_VENDOR.handles_on_speech)(w, id, from)
}

fn say_public(w: &mut World, id: MobileId, text: &str) {
    messages::public_overhead_message(
        w,
        id,
        MessageType::REGULAR,
        0x3B2,
        false,
        text,
        true,
        AccessLevel::Player,
    );
}

fn on_speech(w: &mut World, id: MobileId, e: &mut SpeechEventArgs) {
    (BASE_VENDOR.on_speech)(w, id, e);
    if e.handled || !mx::in_range(w, id, e.mobile, 3) {
        return;
    }
    let no_timer = bdata(w, id).is_some_and(|d| d.news_timer == TimerHandle::default());
    if no_timer && e.has_keyword(0x30) {
        // *news*
        match get_random_entry(w, Owner::Global) {
            None => messages::public_overhead_message_number(
                w,
                id,
                MessageType::REGULAR,
                0x3B2,
                1005643,
                "",
                true,
            ), // I have no news at this time.
            Some(tce) => {
                let n = tce.lines.len() as i32;
                let mut index = 0usize;
                let h = timer::start(
                    w,
                    muo_primitives::TimeSpan::from_secs(1),
                    muo_primitives::TimeSpan::from_secs(3),
                    n,
                    move |w| {
                        // `_newsTimer.Index` counts the ticks done so far.
                        if index >= tce.lines.len() {
                            let mut t = bdata_mut(w, id)
                                .map(|d| std::mem::take(&mut d.news_timer))
                                .unwrap_or_default();
                            t.cancel(w);
                        } else {
                            say_public(w, id, &tce.lines[index]);
                            index += 1;
                        }
                    },
                );
                if let Some(d) = bdata_mut(w, id) {
                    d.news_timer = h;
                }
                messages::public_overhead_message_number(
                    w,
                    id,
                    MessageType::REGULAR,
                    0x3B2,
                    502978,
                    "",
                    true,
                ); // Some of the latest news!
            }
        }
    }
    let rumors = bdata(w, id).and_then(|d| d.rumors.clone());
    let Some(rumors) = rumors else { return };
    for rumor in rumors.iter().flatten() {
        let Some(keyword) = rumor
            .keyword
            .as_deref()
            .map(str::trim)
            .filter(|k| !k.is_empty())
        else {
            continue;
        };
        if keyword.to_uppercase() == e.speech.to_uppercase() {
            let Some(message) = rumor
                .message
                .as_deref()
                .map(str::trim)
                .filter(|m| !m.is_empty())
            else {
                continue;
            };
            say_public(w, id, message);
        }
    }
}

fn on_after_delete(w: &mut World, id: MobileId) {
    (BASE_VENDOR.on_after_delete)(w, id);
    set_house(w, id, None);
}

fn on_before_death(w: &mut World, id: MobileId) -> bool {
    if !(BASE_VENDOR.on_before_death)(w, id) {
        return false;
    }
    if let Some(shoes) = items::find_item_on_layer(w, id, muo_server::layer::Layer::Shoes)
        && factory::item_is(w, shoes, "Server.Items.Sandals")
    {
        muo_server::item::base::set_hue(w, shoes, 0);
    }
    true
}

fn get_context_menu_entries(
    w: &mut World,
    id: MobileId,
    from: MobileId,
    list: &mut Vec<ContextMenuEntry>,
) {
    (BASE_VENDOR.get_context_menu_entries)(w, id, from, list);
    if is_owner(w, id, from) && mx::in_los(w, from, id) {
        let mut e = ContextMenuEntry::new(6151, 12);
        e.on_click = Some(Rc::new(move |w, from, target| {
            if let EntityId::Mobile(v) = target
                && is_barkeeper(w, v)
            {
                begin_management(w, v, from);
            }
        }));
        list.push(e);
    }
}

// ---- management ----

fn private_say(w: &mut World, id: MobileId, from: MobileId, text: &str) {
    let ns = w.mobile(from).and_then(|m| m.net_state);
    messages::private_overhead_message(w, id, MessageType::REGULAR, 0x3B2, false, text, ns);
}

fn private_say_number(w: &mut World, id: MobileId, from: MobileId, number: i32) {
    let ns = w.mobile(from).and_then(|m| m.net_state);
    messages::private_overhead_message_number(w, id, MessageType::REGULAR, 0x3B2, number, "", ns);
}

fn set_prompt(w: &mut World, from: MobileId, p: Prompt) {
    let pid = add_prompt(w, p);
    props_b::set_prompt(w, from, Some(pid));
}

/// A prompt whose cancel answers an empty string; the text is cut to 130 characters.
fn text_prompt(f: impl Fn(&mut World, MobileId, String) + 'static) -> Prompt {
    let f = Rc::new(f);
    let g = f.clone();
    Prompt {
        on_response: Some(Rc::new(move |w, from, text| {
            f(w, from, text.chars().take(130).collect());
        })),
        on_cancel: Some(Rc::new(move |w, from| g(w, from, String::new()))),
    }
}

/// `BeginManagement(from)`.
pub fn begin_management(w: &mut World, id: MobileId, from: MobileId) {
    if !is_owner(w, id, from) {
        return;
    }
    display_gump(w, from, id);
}

/// `Dismiss()`.
pub fn dismiss(w: &mut World, id: MobileId) {
    muo_server::mobile::base::delete(w, id);
}

/// `BeginChangeRumor(from, index)`.
pub fn begin_change_rumor(w: &mut World, id: MobileId, from: MobileId, index: usize) {
    if index >= RUMOR_COUNT {
        return;
    }
    set_prompt(
        w,
        from,
        text_prompt(move |w, from, text| end_change_rumor(w, id, from, index, &text)),
    );
    private_say(
        w,
        id,
        from,
        "Say what news you would like me to tell our guests.",
    );
}

/// The `Rumors ??= new BarkeeperRumor[RumorCount]; Rumors[index]` of the End methods. `None` when the stored
/// array is shorter (nothing happens then).
fn rumor_slot(w: &mut World, id: MobileId, index: usize) -> Option<&mut Option<Rumor>> {
    let d = bdata_mut(w, id)?;
    let r = d.rumors.get_or_insert_with(|| vec![None; RUMOR_COUNT]);
    r.get_mut(index)
}

/// `EndChangeRumor(from, index, text)`.
pub fn end_change_rumor(w: &mut World, id: MobileId, from: MobileId, index: usize, text: &str) {
    if index >= RUMOR_COUNT {
        return;
    }
    let Some(slot) = rumor_slot(w, id, index) else {
        return;
    };
    match slot {
        None => {
            *slot = Some(Rumor {
                message: Some(text.to_owned()),
                keyword: None,
            })
        }
        Some(r) => r.message = Some(text.to_owned()),
    }
    set_prompt(
        w,
        from,
        text_prompt(move |w, from, text| end_change_keyword(w, id, from, index, &text)),
    );
    // What keyword should a guest say to me to get this news?
    private_say_number(w, id, from, 1079267);
}

/// `EndChangeKeyword(from, index, text)`.
pub fn end_change_keyword(w: &mut World, id: MobileId, from: MobileId, index: usize, text: &str) {
    if index >= RUMOR_COUNT {
        return;
    }
    let Some(slot) = rumor_slot(w, id, index) else {
        return;
    };
    match slot {
        None => {
            *slot = Some(Rumor {
                message: None,
                keyword: Some(text.to_owned()),
            })
        }
        Some(r) => r.keyword = Some(text.to_owned()),
    }
    private_say(w, id, from, "I'll pass on the message.");
}

/// `RemoveRumor(index)`.
pub fn remove_rumor(w: &mut World, id: MobileId, index: usize) {
    if index < RUMOR_COUNT
        && let Some(d) = bdata_mut(w, id)
        && let Some(r) = d.rumors.as_mut()
        && let Some(slot) = r.get_mut(index)
    {
        *slot = None;
    }
}

/// `BeginChangeTip(from)`.
pub fn begin_change_tip(w: &mut World, id: MobileId, from: MobileId) {
    set_prompt(
        w,
        from,
        text_prompt(move |w, from, text| end_change_tip(w, id, from, &text)),
    );
    // Say what you want me to tell guests when they give me a good tip.
    private_say_number(w, id, from, 1079268);
}

/// `EndChangeTip(from, text)`.
pub fn end_change_tip(w: &mut World, id: MobileId, from: MobileId, text: &str) {
    if let Some(d) = bdata_mut(w, id) {
        d.tip_message = Some(text.to_owned());
    }
    private_say(
        w,
        id,
        from,
        "I'll say that to anyone who gives me a good tip.",
    );
}

/// `RemoveTip()`.
pub fn remove_tip(w: &mut World, id: MobileId) {
    if let Some(d) = bdata_mut(w, id) {
        d.tip_message = None;
    }
}

/// `BeginChangeTitle(from)`.
pub fn begin_change_title(w: &mut World, id: MobileId, from: MobileId) {
    muo_gumps::m_z::common::send(
        w,
        from,
        StaticGumpObj::new(0, 0, BarkeeperTitleGump { barkeeper: id }),
    );
}

/// `EndChangeTitle(title)`.
pub fn end_change_title(w: &mut World, id: MobileId, title: Option<&str>) {
    props_a::set_title(w, id, title.map(str::to_owned));
    load_sb_info(w, id);
}

/// `ChangeGender()`.
pub fn change_gender(w: &mut World, id: MobileId) {
    let female = !w.mobile(id).is_some_and(|m| m.female);
    props_b::set_female(w, id, female);
    if female {
        props_b::set_body(w, id, muo_primitives::Body(401));
        let n = muo_content::misc::small::random_name(w, "female");
        crate::common::name(w, id, Some(n));
        props_b::set_facial_hair_item_id(w, id, 0);
    } else {
        props_b::set_body(w, id, muo_primitives::Body(400));
        let n = muo_content::misc::small::random_name(w, "male");
        crate::common::name(w, id, Some(n));
    }
}

// ---- serialization ----

fn serialize(w: &World, id: MobileId, out: &mut dyn GenericWriter) {
    (BASE_VENDOR.serialize)(w, id, out);
    out.write_i32(2);
    let Some(d) = bdata(w, id) else { return };
    out.write_item(w, d.house);
    out.write_mobile(w, d.owner);
    let rumors = d.rumors.as_deref().unwrap_or(&[]);
    out.write_encoded_int(rumors.len() as i32);
    for r in rumors {
        // A null slot would throw in the generated serializer; it is saved as an empty rumor.
        let r = r.clone().unwrap_or_default();
        out.write_encoded_int(0);
        out.write_opt_string(r.message.as_deref());
        out.write_opt_string(r.keyword.as_deref());
    }
    out.write_opt_string(d.tip_message.as_deref());
}

fn deserialize(w: &mut World, id: MobileId, r: &mut dyn GenericReader) -> muo_io::Result<()> {
    (BASE_VENDOR.deserialize)(w, id, r)?;
    let version = r.read_i32()?;
    // `_house = ...` is a plain field write: the house's barkeeper list is not touched.
    let mut d = BarkeeperData {
        house: r.read_item(w)?,
        owner: r.read_mobile(w)?,
        ..BarkeeperData::default()
    };
    let n = r.read_encoded_int()?.max(0) as usize;
    let mut rumors = Vec::with_capacity(n.min(1024));
    for _ in 0..n {
        if version < 2 {
            if r.read_bool()? {
                let message = r.read_string()?;
                let keyword = r.read_string()?;
                rumors.push(Some(Rumor { message, keyword }));
            } else {
                rumors.push(None);
            }
        } else {
            let _v = r.read_encoded_int()?;
            let message = r.read_string()?;
            let keyword = r.read_string()?;
            rumors.push(Some(Rumor { message, keyword }));
        }
    }
    d.rumors = Some(rumors);
    d.tip_message = r.read_string()?;
    if let Some(m) = w.mobile_mut(id) {
        m.ext.insert(d);
    }
    Ok(())
}

pub static PLAYER_BARKEEPER: MobileType = MobileType {
    full_name: "Server.Mobiles.PlayerBarkeeper",
    parent: Some(&BASE_VENDOR),
    construct: None,
    family: Some(&PLAYER_BARKEEPER_C),
    serialize,
    deserialize,
    handles_on_speech,
    on_speech,
    on_after_delete,
    on_before_death,
    get_context_menu_entries,
    ..BASE_VENDOR
};

// ---- the gumps ----

/// `BarkeeperGump.DisplayTo(from, barkeeper)`.
pub fn display_gump(w: &mut World, from: MobileId, barkeeper: MobileId) {
    let ns = w.mobile(from).and_then(|m| m.net_state);
    if let Some(ns) = ns {
        close_gump::<BarkeeperTitleGump>(w, ns);
        close_gump::<BarkeeperGump<true>>(w, ns);
        close_gump::<BarkeeperGump<false>>(w, ns);
    }
    let body = w.mobile(barkeeper).map_or(0, |m| m.cur_body().0);
    if body != 0x340 && body != 0x402 {
        muo_gumps::m_z::common::send_singleton(
            w,
            from,
            StaticGumpObj::new(0, 0, BarkeeperGump::<true>::new(w, barkeeper)),
        );
    } else {
        muo_gumps::m_z::common::send_singleton(
            w,
            from,
            StaticGumpObj::new(0, 0, BarkeeperGump::<false>::new(w, barkeeper)),
        );
    }
}

/// `BarkeeperHumanGump` (`MODIFY`) and `BarkeeperNonHumanGump`.
pub struct BarkeeperGump<const MODIFY: bool> {
    barkeeper: MobileId,
    /// `_barkeeper.Rumors ?? EmptyRumors` and `TipMessage`, read when the gump is made (it is compiled at once).
    rumors: Vec<Option<Rumor>>,
    tip: Option<String>,
}

impl<const MODIFY: bool> BarkeeperGump<MODIFY> {
    fn new(w: &World, barkeeper: MobileId) -> Self {
        let d = bdata(w, barkeeper);
        Self {
            barkeeper,
            rumors: d
                .and_then(|d| d.rumors.clone())
                .unwrap_or_else(|| vec![None; RUMOR_COUNT]),
            tip: d.and_then(|d| d.tip_message.clone()),
        }
    }
}

fn get_button_id(kind: i32, index: i32) -> i32 {
    1 + index * 6 + kind
}

fn back_button(b: &mut StaticGumpBuilder, page: Option<i32>) {
    match page {
        Some(p) => b.add_button_type(338, 437, 4014, 4016, 0, GumpButtonType::Page, p),
        None => b.add_button(338, 437, 4014, 4016, 0),
    }
    b.add_html_localized(290, 440, 35, 40, 1005007, false, false); // Back
}

/// The background of both barkeeper gumps (`RenderBackground`): the common frame.
fn frame(b: &mut StaticGumpBuilder) {
    b.add_background(30, 40, 585, 410, 5054);
    for (x, y, id) in [
        (30, 40, 9251),
        (180, 40, 9251),
        (30, 40, 9253),
        (30, 130, 9253),
        (598, 40, 9255),
        (598, 130, 9255),
        (30, 433, 9257),
        (180, 433, 9257),
        (30, 40, 9250),
        (598, 40, 9252),
        (598, 433, 9258),
        (30, 433, 9256),
    ] {
        b.add_image(x, y, id, 0);
    }
    for (x, y, id) in [
        (30, 40, 6816),
        (30, 125, 6817),
        (30, 233, 6817),
        (30, 341, 6817),
        (580, 40, 6814),
        (588, 125, 6815),
        (588, 233, 6815),
        (588, 341, 6815),
    ] {
        b.add_item(x, y, id, 0);
    }
}

impl<const MODIFY: bool> StaticGumpLogic for BarkeeperGump<MODIFY> {
    const TYPE_NAME: &'static str = if MODIFY {
        "Server.Gumps.BarkeeperHumanGump"
    } else {
        "Server.Gumps.BarkeeperNonHumanGump"
    };

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut StaticGumpBuilder) {
        // RenderBackground
        b.add_page(0);
        frame(b);
        b.add_background(183, 25, 280, 30, 5054);
        b.add_image(180, 25, 10460, 0);
        b.add_image(434, 25, 10460, 0);
        b.add_image(560, 20, 1417, 0);
        b.add_html_localized(223, 32, 200, 40, 1078366, false, false); // BARKEEP CUSTOMIZATION MENU
        b.add_background(243, 433, 150, 30, 5054);
        b.add_image(240, 433, 10460, 0);
        b.add_image(375, 433, 10460, 0);

        // RenderCategories
        b.add_page(1);
        b.add_button_type(130, 120, 4005, 4007, 0, GumpButtonType::Page, 2);
        b.add_html_localized(170, 120, 200, 40, 1078352, false, false); // Message Control
        b.add_button_type(130, 200, 4005, 4007, 0, GumpButtonType::Page, 8);
        b.add_html_localized(170, 200, 200, 40, 1078353, false, false); // Customize your barkeep
        b.add_button_type(130, 280, 4005, 4007, 0, GumpButtonType::Page, 3);
        b.add_html_localized(170, 280, 200, 40, 1078354, false, false); // Dismiss your barkeep
        back_button(b, None);
        b.add_item(574, 43, 5360, 0);

        // RenderMessageManagement
        b.add_page(2);
        b.add_button_type(130, 120, 4005, 4007, 0, GumpButtonType::Page, 4);
        b.add_html_localized(170, 120, 380, 20, 1078355, false, false); // Add or change a message and keyword
        b.add_button_type(130, 200, 4005, 4007, 0, GumpButtonType::Page, 5);
        b.add_html_localized(170, 200, 380, 20, 1078356, false, false); // Remove a message and keyword from your barkeep
        b.add_button_type(130, 280, 4005, 4007, 0, GumpButtonType::Page, 6);
        b.add_html_localized(170, 280, 380, 20, 1078357, false, false); // Add or change your barkeeper's tip message
        b.add_button_type(130, 360, 4005, 4007, 0, GumpButtonType::Page, 7);
        b.add_html_localized(170, 360, 380, 20, 1078358, false, false); // Delete your barkeepers tip message
        back_button(b, Some(1));
        b.add_item(580, 46, 4030, 0);

        // RenderDismissConfirmation
        b.add_page(3);
        b.add_html_localized(170, 160, 380, 20, 1078359, false, false); // Are you sure you want to dismiss your barkeeper?
        b.add_button(205, 280, 4005, 4007, get_button_id(0, 0));
        b.add_html_localized(240, 280, 100, 20, 1046362, false, false); // Yes
        b.add_button(395, 280, 4005, 4007, 0);
        b.add_html_localized(430, 280, 100, 20, 1046363, false, false); // No
        back_button(b, Some(1));
        b.add_item(574, 43, 5360, 0);
        b.add_item(584, 34, 6579, 0);

        // RenderMessageManagement_Message_AddOrChange (page 4) and _Remove (page 5)
        for (page, header_x, header_w, header, kind) in
            [(4, 250, 500, 1078360, 1), (5, 190, 500, 1078362, 2)]
        {
            b.add_page(page);
            b.add_html_localized(header_x, 60, header_w, 25, header, false, false);
            for i in 0..RUMOR_COUNT as i32 {
                b.add_html(100, 70 + i * 120, 50, 20, "Message", false, false);
                b.add_html_placeholder(
                    100,
                    90 + i * 120,
                    450,
                    40,
                    &format!("r{i}msg"),
                    true,
                    false,
                );
                b.add_html_localized(100, 130 + i * 120, 50, 20, 1078361, false, false); // Keyword
                b.add_html_placeholder(
                    100,
                    150 + i * 120,
                    450,
                    40,
                    &format!("r{i}key"),
                    true,
                    false,
                );
                b.add_button(60, 90 + i * 120, 4005, 4007, get_button_id(kind, i));
            }
            back_button(b, Some(2));
            b.add_item(580, 46, 4030, 0);
        }

        // RenderMessageManagement_Tip_AddOrChange (page 6) and _Remove (page 7)
        for (page, header, kind) in [(6, 1078363, 3), (7, 1078364, 4)] {
            b.add_page(page);
            b.add_html_localized(250, 95, 500, 20, header, false, false);
            b.add_html(100, 190, 50, 20, "Message", false, false);
            b.add_html_placeholder(100, 210, 450, 40, "tipMsg", true, false);
            b.add_button(60, 210, 4005, 4007, get_button_id(kind, 0));
            back_button(b, Some(2));
            b.add_item(580, 46, 4030, 0);
        }

        // RenderAppearanceCategories
        b.add_page(8);
        b.add_button(130, 120, 4005, 4007, get_button_id(5, 0));
        b.add_html(170, 120, 120, 20, "Title", false, false);
        if MODIFY {
            b.add_button(130, 200, 4005, 4007, get_button_id(5, 1));
            b.add_html_localized(170, 200, 120, 20, 1077829, false, false); // Appearance
            b.add_button(130, 280, 4005, 4007, get_button_id(5, 2));
            b.add_html_localized(170, 280, 120, 20, 1078365, false, false); // Male / Female
        }
        back_button(b, Some(1));
        b.add_item(580, 44, 4033, 0);
    }

    fn build_strings(&self, strings: &mut GumpStrings) {
        for (i, rumor) in self.rumors.iter().enumerate() {
            let msg = rumor.as_ref().and_then(|r| r.message.as_deref());
            let key = rumor.as_ref().and_then(|r| r.keyword.as_deref());
            strings.set_string_slot(&format!("r{i}msg"), msg.unwrap_or("No current message"));
            strings.set_string_slot(&format!("r{i}key"), key.unwrap_or("None"));
        }
        strings.set_string_slot(
            "tipMsg",
            self.tip.as_deref().unwrap_or("No current tip message"),
        );
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let Some(from) = muo_gumps::m_z::common::mobile_of(w, sender) else {
            return;
        };
        let id = self.barkeeper;
        if !is_owner(w, id, from) {
            return;
        }
        let index = info.button_id - 1;
        if index < 0 {
            return;
        }
        let (index, kind) = (index / 6, index % 6);
        match kind {
            0 => {
                if index == 0 {
                    dismiss(w, id); // Dismiss
                }
            }
            1 => begin_change_rumor(w, id, from, index as usize),
            2 => remove_rumor(w, id, index as usize),
            3 => begin_change_tip(w, id, from),
            4 => remove_tip(w, id),
            5 => match index {
                0 => begin_change_title(w, id, from),
                1 => muo_gumps::m_z::player_vendor::customize_display_to(w, from, id),
                2 => change_gender(w, id),
                _ => {}
            },
            _ => {}
        }
    }
}

// ---- BarkeeperTitleGump ----

/// `(description, title, vendor)`.
const TITLES: &[(i32, Option<&str>, bool)] = &[
    (1076083, Some("the alchemist"), false),
    (1077244, Some("the animal tamer"), false),
    (1078440, Some("the apothecary"), false),
    (1078441, Some("the artist"), false),
    (1078442, Some("the baker"), true),
    (1073298, Some("the bard"), false),
    (1076102, Some("the barkeep"), true),
    (1078443, Some("the beggar"), false),
    (1073297, Some("the blacksmith"), false),
    (1078444, Some("the bounty hunter"), false),
    (1078446, Some("the brigand"), false),
    (1078447, Some("the butler"), false),
    (1060774, Some("the carpenter"), false),
    (1078448, Some("the chef"), true),
    (1078449, Some("the commander"), false),
    (1078450, Some("the curator"), false),
    (1078451, Some("the drunkard"), false),
    (1078452, Some("the farmer"), false),
    (1078453, Some("the fisherman"), false),
    (1078454, Some("the gambler"), false),
    (1078455, Some("the gypsy"), false),
    (1075996, Some("the herald"), false),
    (1076107, Some("the herbalist"), false),
    (1078465, Some("the hermit"), false),
    (1078466, Some("the innkeeper"), true),
    (1078467, Some("the jailor"), false),
    (1078468, Some("the jester"), false),
    (1078469, Some("the librarian"), false),
    (1073292, Some("the mage"), false),
    (1078470, Some("the mercenary"), false),
    (1060775, Some("the merchant"), false),
    (1078472, Some("the messenger"), false),
    (1076093, Some("the miner"), false),
    (1078475, Some("the monk"), false),
    (1078476, Some("the noble"), false),
    (1073290, Some("the paladin"), false),
    (1078479, Some("the peasant"), false),
    (1078480, Some("the pirate"), false),
    (1078481, Some("the prisoner"), false),
    (1078482, Some("the prophet"), false),
    (1078484, Some("the ranger"), false),
    (1078487, Some("the sage"), false),
    (1078488, Some("the sailor"), false),
    (1078489, Some("the scholar"), false),
    (1060773, Some("the scribe"), false),
    (1078490, Some("the sentry"), false),
    (1060795, Some("the servant"), false),
    (1078491, Some("the shepherd"), false),
    (1078492, Some("the soothsayer"), false),
    (1078493, Some("the stoic"), false),
    (1078494, Some("the storyteller"), false),
    (1076134, Some("the tailor"), false),
    (1076096, Some("the thief"), false),
    (1076137, Some("the tinker"), false),
    (1076097, Some("the town crier"), false),
    (1073291, Some("the treasure hunter"), false),
    (1076112, Some("the waiter"), true),
    (1077242, Some("the warrior"), false),
    (1078496, Some("the watchman"), false),
    (1078495, None, false), // No Title
];

const TITLE_PAGES: usize = TITLES.len().div_ceil(20);

/// `BarkeeperTitleGump`.
pub struct BarkeeperTitleGump {
    barkeeper: MobileId,
}

impl StaticGumpLogic for BarkeeperTitleGump {
    const TYPE_NAME: &'static str = "Server.Gumps.BarkeeperTitleGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut StaticGumpBuilder) {
        b.add_page(0);
        frame(b);
        b.add_image(560, 20, 1417, 0);
        b.add_item(580, 44, 4033, 0);
        b.add_background(183, 25, 280, 30, 5054);
        b.add_image(180, 25, 10460, 0);
        b.add_image(434, 25, 10460, 0);
        b.add_html_localized(223, 32, 200, 40, 1078366, false, false); // BARKEEP CUSTOMIZATION MENU
        b.add_background(243, 433, 150, 30, 5054);
        b.add_image(240, 433, 10460, 0);
        b.add_image(375, 433, 10460, 0);
        b.add_image(80, 398, 2151, 0);
        b.add_item(72, 406, 2543, 0);
        b.add_html_localized(110, 412, 180, 25, 1078445, false, false); // sells food and drink

        for page in 0..TITLE_PAGES {
            let current = page as i32 + 1;
            b.add_page(current);
            if TITLE_PAGES == 3 && (1..=3).contains(&current) {
                let cliloc = match current {
                    1 => 1078439, // Page 1 of 3
                    2 => 1078464, // Page 2 of 3
                    _ => 1078483, // Page 3 of 3
                };
                b.add_html_localized(430, 70, 180, 25, cliloc, false, false);
            } else {
                // Page ~1_CUR~ of ~2_MAX~
                b.add_html_localized_args(
                    430,
                    70,
                    180,
                    25,
                    1153561,
                    &format!("{current}\t{TITLE_PAGES}"),
                    0x7FFF,
                    false,
                    false,
                );
            }

            let mut count = 0;
            let mut i = page * 20;
            while count < 20 && i < TITLES.len() {
                let (desc, _, vendor) = TITLES[i];
                let (y_offset, x_offset) = (count % 10, count / 10);
                b.add_button(
                    80 + x_offset * 260,
                    100 + y_offset * 30,
                    4005,
                    4007,
                    2 + i as i32,
                );
                b.add_html_localized(
                    120 + x_offset * 260,
                    100 + y_offset * 30,
                    if vendor { 148 } else { 180 },
                    25,
                    desc,
                    true,
                    false,
                );
                if vendor {
                    b.add_image(270 + x_offset * 260, 98 + y_offset * 30, 2151, 0);
                    b.add_item(262 + x_offset * 260, 106 + y_offset * 30, 2543, 0);
                }
                count += 1;
                i += 1;
            }

            // `currentPage + 1 % _pageCount` binds as `currentPage + (1 % _pageCount)`.
            b.add_button_type(
                340,
                400,
                4005,
                4007,
                0,
                GumpButtonType::Page,
                current + 1 % TITLE_PAGES as i32,
            );
            b.add_html_localized(380, 400, 180, 25, 1078456, false, false); // More Job Titles
            b.add_button(338, 437, 4014, 4016, 1);
            b.add_html_localized(290, 440, 35, 40, 1005007, false, false); // Back
        }
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let id = self.barkeeper;
        if mx::deleted(w, id) {
            return;
        }
        let mut button = info.button_id;
        if button <= 0 {
            return;
        }
        button -= 1;
        let Some(from) = muo_gumps::m_z::common::mobile_of(w, sender) else {
            return;
        };
        if button <= 0 {
            display_gump(w, from, id); // CancelChangeTitle
            return;
        }
        button -= 1;
        if (button as usize) < TITLES.len() {
            end_change_title(w, id, TITLES[button as usize].1);
        }
    }
}

pub(crate) fn register(types: &mut TypeRegistry) {
    types.register_mobile(&PLAYER_BARKEEPER, &[]);
    types.register_phase_default(
        muo_server::type_registry::Phase::Configure,
        "PlayerBarkeeper.Configure",
        |w| {
            use muo_content::props::{Ctor, Kind, Param};
            use muo_content::utilities::types::PropType;
            static CTORS: &[Ctor] = &[Ctor {
                access: None,
                params: &[
                    Param {
                        name: "owner",
                        kind: Kind::Plain(PropType::Entity),
                        default: None,
                        ty: "Mobile",
                    },
                    Param {
                        name: "house",
                        kind: Kind::Plain(PropType::Entity),
                        default: None,
                        ty: "BaseHouse",
                    },
                ],
                make: ctor,
            }];
            muo_content::props::add_ctors(w, "Server.Mobiles.PlayerBarkeeper", CTORS);
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use muo_content::gumps::base::Gump;

    fn world() -> World {
        let mut w = World::new();
        w.resources
            .get_mut::<muo_content::creature::speeds::NpcSpeeds>()
            .register(muo_content::creature::speeds::SpeedClassEntry {
                level: "Medium".into(),
                active: 0.2,
                passive: 0.4,
                active_move: 0.0,
                passive_move: 0.0,
                types: vec![],
            });
        w
    }

    #[test]
    fn the_title_decides_what_the_barkeeper_sells() {
        let mut w = world();
        let b = make(&mut w, None, None);
        assert_eq!(w.mobile(b).unwrap().title.as_deref(), Some("the barkeeper"));
        assert!(muo_content::vendor::is_active_seller(&w, b));
        assert!(!muo_content::vendor::is_active_buyer(&w, b));
        end_change_title(&mut w, b, Some("the thief"));
        assert!(!muo_content::vendor::is_active_seller(&w, b));
        end_change_title(&mut w, b, Some("the chef"));
        assert!(muo_content::vendor::is_active_seller(&w, b));
        assert_eq!(TITLES.len(), 60);
        assert_eq!(TITLE_PAGES, 3);
    }

    #[test]
    fn the_gumps_compile_with_their_rumors_in_the_placeholders() {
        let mut w = world();
        let b = make(&mut w, None, None);
        let owner = w.new_mobile(&muo_server::mobile::MOBILE_BASE);
        end_change_rumor(&mut w, b, owner, 0, "news");
        let mut human = StaticGumpObj::new(0, 0, BarkeeperGump::<true>::new(&w, b));
        let mut other = StaticGumpObj::new(0, 0, BarkeeperGump::<false>::new(&w, b));
        let (h, o) = (human.compile(), other.compile());
        assert_eq!(h[0], 0xDD);
        // The human gump has two more buttons (Appearance, Male / Female) and their labels.
        assert!(h.len() > o.len());
        let mut title = StaticGumpObj::new(0, 0, BarkeeperTitleGump { barkeeper: b });
        assert_eq!(title.compile()[0], 0xDD);
    }

    #[test]
    fn a_rumor_slot_beyond_the_array_does_nothing() {
        let mut w = world();
        let b = make(&mut w, None, None);
        let owner = w.new_mobile(&muo_server::mobile::MOBILE_BASE);
        bdata_mut(&mut w, b).unwrap().rumors = Some(Vec::new()); // as loaded from a null array
        end_change_rumor(&mut w, b, owner, 2, "x");
        remove_rumor(&mut w, b, 2);
        assert_eq!(bdata(&w, b).unwrap().rumors, Some(Vec::new()));
    }

    #[test]
    fn only_the_owner_or_a_higher_level_manages() {
        let mut w = world();
        let owner = w.new_mobile(&muo_server::mobile::MOBILE_BASE);
        let other = w.new_mobile(&muo_server::mobile::MOBILE_BASE);
        let b = make(&mut w, Some(owner), None);
        assert!(is_owner(&w, b, owner));
        assert!(!is_owner(&w, b, other));
        w.mobile_mut(other).unwrap().access_level = AccessLevel::Administrator;
        assert!(is_owner(&w, b, other));
    }
}
