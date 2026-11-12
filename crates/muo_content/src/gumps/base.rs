//! The gump base: `BaseGump`, `DynamicGump`, `StaticGump`, `RelayInfo`, `GumpColors`, the invalid-response check
//! and the packed gump packet writer.
//!
//! A gump class is a plain struct that implements [`DynamicGumpLogic`] or [`StaticGumpLogic`] (or the legacy
//! [`LegacyGumpLogic`]); [`DynamicGumpObj`]/[`StaticGumpObj`]/[`LegacyGumpObj`] wrap it with the state `BaseGump`
//! holds. A type test (`gump is MyGump`), `HasGump<MyGump>()` and `FindGump<MyGump>()` are
//! [`super::system::find_gump`] over the logic's own type.
//!
//! The serial is given when the gump is first sent (not in the constructor: nothing observes it before the
//! send).

use std::any::Any;

use muo_primitives::hash::compute_hash32;
use muo_primitives::{Point2D, Serial};
use muo_server::World;
use muo_server::net_state::NetStateId;

use super::builder::{DynamicGumpBuilder, GumpBuilder, GumpStrings, StaticGumpBuilder};
use super::legacy::LegacyLayout;

/// `GumpTextColors`.
pub mod text_colors {
    pub const GOLD: &str = "#ffc959";
    pub const WHITE: &str = "#ffffff";
    pub const BLUE: &str = "#1D63DC";
    pub const LIGHT_BLUE: &str = "#7799EE";
    pub const GREEN: &str = "#80E732";
    pub const ORANGE: &str = "#F28B00";
    pub const PURPLE: &str = "#F3ACF6";
    pub const RED: &str = "#B52B2D";
    pub const BRIGHT_RED: &str = "#FF0000";
    pub const LIGHT_GREEN: &str = "#E6FFC0";
    pub const YELLOW: &str = "#FFFFBB";
    pub const LIGHT_GRAY: &str = "#CCCCCC";
}

/// `GumpHues`.
pub mod hues {
    pub const BLACK: i32 = 0;
    pub const WHITE: i32 = 2049;
}

/// `BaseGump.GetTypeId(type)`.
pub fn get_type_id(full_name: &str) -> i32 {
    const PRIME_MULTI: i32 = 0x108B_76F1;
    let hash = compute_hash32(full_name) as i32;
    // Virtue Gump
    if hash == 461 {
        hash.wrapping_mul(PRIME_MULTI)
    } else {
        hash
    }
}

/// The state every `BaseGump` has.
#[derive(Clone, Debug, Default)]
pub struct GumpCore {
    pub type_id: i32,
    /// 0 until the gump is sent.
    pub serial: Serial,
    pub x: i32,
    pub y: i32,
    pub has_visual_elements: bool,
    pub switches: i32,
    pub text_entries: i32,
}

impl GumpCore {
    pub fn new(type_name: &str, x: i32, y: i32) -> Self {
        Self {
            type_id: get_type_id(type_name),
            x,
            y,
            ..Self::default()
        }
    }
}

/// `RelayInfo`: what a gump response carried.
#[derive(Clone, Debug, Default)]
pub struct RelayInfo {
    pub button_id: i32,
    pub switches: Vec<i32>,
    text_entries: Vec<(u16, String)>,
}

impl RelayInfo {
    pub fn new(button_id: i32, switches: Vec<i32>, text_entries: Vec<(u16, String)>) -> Self {
        Self {
            button_id,
            switches,
            text_entries,
        }
    }

    /// `IsSwitched`.
    pub fn is_switched(&self, switch_id: i32) -> bool {
        self.switches.contains(&switch_id)
    }

    /// `GetTextEntry`: `None` if the client sent nothing for the entry.
    pub fn get_text_entry(&self, entry_id: i32) -> Option<&str> {
        let id = u16::try_from(entry_id).ok()?;
        self.text_entries
            .iter()
            .find(|(i, _)| *i == id)
            .map(|(_, s)| s.as_str())
    }
}

/// `BaseGump`.
pub trait Gump: Any {
    fn core(&self) -> &GumpCore;
    fn core_mut(&mut self) -> &mut GumpCore;
    /// The gump class's full name (`Server.Gumps.WhoGump`), which the client's type id is a hash of.
    fn type_name(&self) -> &'static str;
    /// `Singleton`: only one instance of this gump can be open at a time per player.
    fn singleton(&self) -> bool {
        false
    }
    /// `Compile`: the 0xDD packet.
    fn compile(&mut self) -> Vec<u8>;
    /// `OnResponse`.
    fn on_response(&mut self, _w: &mut World, _sender: NetStateId, _info: &RelayInfo) {}
    /// `OnServerClose`.
    fn on_server_close(&mut self, _w: &mut World, _owner: NetStateId) {}
    /// Legacy gumps know their buttons: a response naming another one is invalid.
    fn has_button(&self, _button_id: i32) -> bool {
        true
    }
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// `OutgoingGumpPackets.WritePacked`: the length of the rest, the length when unpacked, the zlib stream.
pub fn write_packed(out: &mut Vec<u8>, span: &[u8]) {
    if span.is_empty() {
        out.extend_from_slice(&0i32.to_be_bytes());
        return;
    }
    let mut dest = vec![0u8; muo_io::compression::Level::compress_bound(span.len())];
    let packed = muo_io::compression::STANDARD.pack(&mut dest, span);
    if packed == 0 {
        tracing::warn!("Gump compression failed");
        out.extend_from_slice(&4i32.to_be_bytes());
        out.extend_from_slice(&0i32.to_be_bytes());
        return;
    }
    out.extend_from_slice(&((4 + packed) as i32).to_be_bytes());
    out.extend_from_slice(&(span.len() as i32).to_be_bytes());
    out.extend_from_slice(&dest[..packed]);
}

/// The 0xDD header: id, a length slot, serial, type id, x, y.
fn begin_packet(core: &GumpCore) -> Vec<u8> {
    let mut out = vec![0xDD, 0, 0];
    out.extend_from_slice(&core.serial.value().to_be_bytes());
    out.extend_from_slice(&core.type_id.to_be_bytes());
    out.extend_from_slice(&core.x.to_be_bytes());
    out.extend_from_slice(&core.y.to_be_bytes());
    out
}

/// `WritePacketLength`.
fn end_packet(mut out: Vec<u8>) -> Vec<u8> {
    let len = out.len() as u16;
    out[1..3].copy_from_slice(&len.to_be_bytes());
    out
}

/// `BaseGump.GetItemGraphicOffset(itemId)`.
pub fn get_item_graphic_offset(w: &World, item_id: usize) -> Point2D {
    let (width, height) = muo_server::item::bounds::size(w, item_id);
    let (width, height) = (i32::from(width), i32::from(height));
    let (mut x, mut y) = (0, 0);
    if width > 44 {
        x -= (width - 44) / 2;
    } else if width < 44 {
        x += (44 - width) / 2;
    }
    if height > 44 {
        y -= height - 44;
    } else if height < 44 {
        y += 44 - height;
    }
    Point2D::new(x, y)
}

/// `BaseGump.GetGumpOffsetForItemGraphic`.
pub fn get_gump_offset_for_item_graphic(
    w: &World,
    item_id: usize,
    relative_x: i32,
    relative_y: i32,
) -> Point2D {
    let o = get_item_graphic_offset(w, item_id);
    Point2D::new(
        relative_x * 22 - relative_y * 22 + o.x,
        relative_x * 22 + relative_y * 22 + o.y,
    )
}

// ---- DynamicGump ----

/// What a `DynamicGump` subclass supplies.
pub trait DynamicGumpLogic: 'static {
    const TYPE_NAME: &'static str;
    /// The type name of this instance: a subclass sharing the logic overrides it (and sets `core.type_id`).
    fn gump_type_name(&self) -> &'static str {
        Self::TYPE_NAME
    }
    fn singleton(&self) -> bool {
        false
    }
    /// `BuildLayout(ref DynamicGumpBuilder)`.
    fn build_layout(&self, builder: &mut DynamicGumpBuilder);
    fn on_response(&mut self, _w: &mut World, _sender: NetStateId, _info: &RelayInfo) {}
    fn on_server_close(&mut self, _w: &mut World, _owner: NetStateId) {}
}

/// `DynamicGump`.
pub struct DynamicGumpObj<T> {
    pub core: GumpCore,
    pub logic: T,
}

impl<T: DynamicGumpLogic> DynamicGumpObj<T> {
    pub fn new(x: i32, y: i32, logic: T) -> Self {
        Self {
            core: GumpCore::new(T::TYPE_NAME, x, y),
            logic,
        }
    }
}

impl<T: DynamicGumpLogic> Gump for DynamicGumpObj<T> {
    fn core(&self) -> &GumpCore {
        &self.core
    }
    fn core_mut(&mut self) -> &mut GumpCore {
        &mut self.core
    }
    fn type_name(&self) -> &'static str {
        self.logic.gump_type_name()
    }
    fn singleton(&self) -> bool {
        self.logic.singleton()
    }

    fn compile(&mut self) -> Vec<u8> {
        let mut out = begin_packet(&self.core);
        let mut b = GumpBuilder::new_dynamic();
        self.logic.build_layout(&mut b);
        b.finalize_layout();

        self.core.has_visual_elements = b.has_visual_elements();
        self.core.switches = b.switches();
        self.core.text_entries = b.text_entries();

        write_packed(&mut out, b.layout_data());
        out.extend_from_slice(&b.strings_count.to_be_bytes());
        write_packed(&mut out, b.strings_data());
        end_packet(out)
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        self.logic.on_response(w, sender, info);
    }
    fn on_server_close(&mut self, w: &mut World, owner: NetStateId) {
        self.logic.on_server_close(w, owner);
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

// ---- StaticGump ----

/// What a `StaticGump<TSelf>` subclass supplies.
pub trait StaticGumpLogic: 'static {
    const TYPE_NAME: &'static str;
    fn singleton(&self) -> bool {
        false
    }
    /// `BuildLayout(ref StaticGumpBuilder)`: the same for every instance of the class.
    fn build_layout(&self, builder: &mut StaticGumpBuilder);
    /// `BuildStrings(ref GumpStringsBuilder)`: fills the placeholders for this instance.
    fn build_strings(&self, _strings: &mut GumpStrings) {}
    fn on_response(&mut self, _w: &mut World, _sender: NetStateId, _info: &RelayInfo) {}
    fn on_server_close(&mut self, _w: &mut World, _owner: NetStateId) {}
}

/// `StaticGump<TSelf>`.
pub struct StaticGumpObj<T> {
    pub core: GumpCore,
    pub logic: T,
}

impl<T: StaticGumpLogic> StaticGumpObj<T> {
    pub fn new(x: i32, y: i32, logic: T) -> Self {
        Self {
            core: GumpCore::new(T::TYPE_NAME, x, y),
            logic,
        }
    }
}

impl<T: StaticGumpLogic> Gump for StaticGumpObj<T> {
    fn core(&self) -> &GumpCore {
        &self.core
    }
    fn core_mut(&mut self) -> &mut GumpCore {
        &mut self.core
    }
    fn type_name(&self) -> &'static str {
        T::TYPE_NAME
    }
    fn singleton(&self) -> bool {
        self.logic.singleton()
    }

    fn compile(&mut self) -> Vec<u8> {
        let mut out = begin_packet(&self.core);
        let mut b = StaticGumpBuilder::new_static();
        self.logic.build_layout(&mut b);
        b.finalize_layout();

        self.core.has_visual_elements = b.has_visual_elements();
        self.core.switches = b.switches();
        self.core.text_entries = b.text_entries();
        let static_count = b.strings_count;

        if b.has_slots() {
            let mut strings = GumpStrings::new();
            self.logic.build_strings(&mut strings);
            b.finalize_strings(&strings); // Modifies the layout
            write_packed(&mut out, b.layout_data());
            out.extend_from_slice(&(static_count + strings.count).to_be_bytes());
            let mut all = b.strings_data().to_vec();
            all.extend_from_slice(strings.data());
            write_packed(&mut out, &all);
        } else {
            write_packed(&mut out, b.layout_data());
            out.extend_from_slice(&static_count.to_be_bytes());
            write_packed(&mut out, b.strings_data());
        }
        end_packet(out)
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        self.logic.on_response(w, sender, info);
    }
    fn on_server_close(&mut self, w: &mut World, owner: NetStateId) {
        self.logic.on_server_close(w, owner);
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

// ---- legacy Gump ----

/// What a `Gump` subclass supplies besides the entries it adds to its [`LegacyLayout`].
pub trait LegacyGumpLogic: 'static {
    const TYPE_NAME: &'static str;
    fn singleton(&self) -> bool {
        false
    }
    /// The type name of this instance: a subclass sharing the logic overrides it.
    fn gump_type_name(&self) -> &'static str {
        Self::TYPE_NAME
    }
    fn on_response(&mut self, _w: &mut World, _sender: NetStateId, _info: &RelayInfo) {}
    fn on_server_close(&mut self, _w: &mut World, _owner: NetStateId) {}
}

/// `Gump`.
pub struct LegacyGumpObj<T> {
    pub core: GumpCore,
    pub layout: LegacyLayout,
    pub logic: T,
}

impl<T: LegacyGumpLogic> LegacyGumpObj<T> {
    pub fn new(x: i32, y: i32, logic: T) -> Self {
        Self {
            core: GumpCore::new(T::TYPE_NAME, x, y),
            layout: LegacyLayout::default(),
            logic,
        }
    }
}

impl<T: LegacyGumpLogic> Gump for LegacyGumpObj<T> {
    fn core(&self) -> &GumpCore {
        &self.core
    }
    fn core_mut(&mut self) -> &mut GumpCore {
        &mut self.core
    }
    fn type_name(&self) -> &'static str {
        self.logic.gump_type_name()
    }
    fn singleton(&self) -> bool {
        self.logic.singleton()
    }

    fn compile(&mut self) -> Vec<u8> {
        let c = self.layout.compile();
        self.core.text_entries = c.text_entries;
        self.core.switches = c.switches;
        self.core.has_visual_elements = c.has_visual_elements;

        let mut out = begin_packet(&self.core);
        write_packed(&mut out, &c.layout);
        out.extend_from_slice(&(c.strings_count as i32).to_be_bytes());
        write_packed(&mut out, &c.strings);
        end_packet(out)
    }

    fn has_button(&self, button_id: i32) -> bool {
        button_id == 0 || self.layout.has_button(button_id) // 0 is always 'close'
    }
    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        self.logic.on_response(w, sender, info);
    }
    fn on_server_close(&mut self, w: &mut World, owner: NetStateId) {
        self.logic.on_server_close(w, owner);
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Hello;
    impl DynamicGumpLogic for Hello {
        const TYPE_NAME: &'static str = "Server.Gumps.Hello";
        fn build_layout(&self, b: &mut DynamicGumpBuilder) {
            b.add_page(0);
            b.add_background(0, 0, 100, 50, 5054);
            b.add_label(5, 5, 0x480, "hello");
            b.add_button(5, 25, 4005, 4007, 1);
        }
    }

    /// Reads back a packet: header fields, then the two zlib sections.
    fn unpack_section(data: &[u8], at: &mut usize) -> Vec<u8> {
        let len = i32::from_be_bytes(data[*at..*at + 4].try_into().unwrap());
        *at += 4;
        if len == 0 {
            return Vec::new();
        }
        let raw_len = i32::from_be_bytes(data[*at..*at + 4].try_into().unwrap()) as usize;
        let packed = &data[*at + 4..*at + len as usize];
        *at += len as usize;
        let mut out = vec![0u8; raw_len];
        muo_io::compression::STANDARD
            .unpack(&mut out, packed)
            .unwrap();
        out
    }

    #[test]
    fn a_dynamic_gump_compiles_to_the_0xdd_packet() {
        let mut g = DynamicGumpObj::new(50, 60, Hello);
        g.core.serial = Serial::new(7);
        let p = g.compile();
        assert_eq!(p[0], 0xDD);
        assert_eq!(u16::from_be_bytes([p[1], p[2]]) as usize, p.len());
        assert_eq!(&p[3..7], &7u32.to_be_bytes());
        assert_eq!(&p[7..11], &get_type_id("Server.Gumps.Hello").to_be_bytes());
        assert_eq!(&p[11..19], &[0, 0, 0, 50, 0, 0, 0, 60]);
        let mut at = 19;
        let layout = unpack_section(&p, &mut at);
        assert_eq!(
            String::from_utf8_lossy(&layout),
            "{ page 0 }{ resizepic 0 0 5054 100 50 }{ text 5 5 1152 0 }{ button 5 25 4005 4007 1 0 1 }\0"
        );
        let count = i32::from_be_bytes(p[at..at + 4].try_into().unwrap());
        at += 4;
        assert_eq!(count, 1);
        let strings = unpack_section(&p, &mut at);
        assert_eq!(strings, [0, 5, 0, b'h', 0, b'e', 0, b'l', 0, b'l', 0, b'o']);
        assert_eq!(at, p.len());
        assert!(g.core.has_visual_elements);
    }

    #[test]
    fn the_virtue_gump_type_id_is_remapped() {
        assert_ne!(get_type_id("Server.Gumps.Hello"), 0);
    }

    #[test]
    fn relay_info_finds_entries_and_switches() {
        let r = RelayInfo::new(3, vec![1, 5], vec![(2, "abc".into())]);
        assert!(r.is_switched(5) && !r.is_switched(2));
        assert_eq!(r.get_text_entry(2), Some("abc"));
        assert_eq!(r.get_text_entry(3), None);
    }
}
