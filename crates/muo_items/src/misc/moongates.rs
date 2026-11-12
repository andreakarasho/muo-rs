//! `PublicMoongate`, `MoonstoneGate`, `Moonstone` and, because `MoonstoneGate` derives from it, `Moongate` (with
//! `MoongateConfirmGump`).
//!
//! `Sigil.ExistsOn` is the spell hook (set by the factions engine); `Party.Get(_caster)` of the MoonstoneGate is `SpellHooks::party_contains`.

use muo_content::gumps::GumpButtonType;
use muo_content::gumps::base::{DynamicGumpLogic, DynamicGumpObj, RelayInfo};
use muo_content::gumps::builder::DynamicGumpBuilder;
use muo_content::gumps::send_gump_to_mobile;
use muo_data::expansion::{Expansion, MapSelectionFlags};
use muo_io::{GenericReader, GenericWriter};
use muo_net::client_version::ClientFlags;
use muo_primitives::{MessageType, Point3D, TimeSpan, utility};
use muo_server::World;
use muo_server::commands::{CommandEventArgs, CommandSystem};
use muo_server::ext::Ext;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, base, deps, messages, ops};
use muo_server::map::MapId;
use muo_server::mobile::AccessLevel;
use muo_server::net_state::NetStateId;
use muo_server::packets::message::MessageType as ItemMessage;
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::type_registry::{Phase, TypeRegistry};

use super::{read_version, write_version};

fn msg(w: &mut World, to: MobileId, number: i32) {
    deps::send_localized_message(w, to, number);
}

fn msg_hue(w: &mut World, to: MobileId, number: i32, hue: i32) {
    muo_server::mobile::part2::messages::send_localized_message(w, to, number, "", hue);
}

fn sigil_exists_on(w: &World, m: MobileId) -> bool {
    muo_content::spells::hooks::hooks(w)
        .and_then(|h| h.sigil_exists_on)
        .is_some_and(|f| f(w, m))
}

fn faction_facet(w: &World) -> Option<MapId> {
    muo_content::player_mobile::hooks::faction_facet(w)
}

fn map_flag(map: MapId) -> MapSelectionFlags {
    match map {
        MapId::FELUCCA => MapSelectionFlags::FELUCCA,
        MapId::TRAMMEL => MapSelectionFlags::TRAMMEL,
        MapId::ILSHENAR => MapSelectionFlags::ILSHENAR,
        MapId::MALAS => MapSelectionFlags::MALAS,
        MapId::TOKUNO => MapSelectionFlags::TOKUNO,
        MapId::TER_MUR => MapSelectionFlags::TER_MUR,
        _ => MapSelectionFlags::empty(),
    }
}

fn core_maps(w: &World) -> MapSelectionFlags {
    w.core_expansion_info()
        .map_or(MapSelectionFlags::empty(), |i| i.map_selection_flags)
}

// ---- Moongate ----

/// What `Moongate` adds to an `Item`.
#[derive(Clone, Debug, Default)]
pub struct MoongateData {
    pub target: Point3D,
    pub target_map: Option<MapId>,
    pub dispellable: bool,
    /// `MoonstoneGate._caster`.
    pub caster: Option<MobileId>,
}

/// `item is Moongate { Dispellable: false }`.
pub fn not_dispellable(w: &World, id: ItemId) -> bool {
    w.item(id)
        .and_then(|i| i.ext.get::<MoongateData>())
        .is_some_and(|d| !d.dispellable)
}

/// `Dispellable = value`.
pub fn set_dispellable(w: &mut World, id: ItemId, value: bool) {
    if let Some(d) = w.item_mut(id).and_then(|i| i.ext.get_mut::<MoongateData>()) {
        d.dispellable = value;
    }
}

fn mg(w: &World, id: ItemId) -> MoongateData {
    w.item(id)
        .and_then(|i| i.ext.get::<MoongateData>())
        .cloned()
        .unwrap_or_default()
}

fn mg_edit(w: &mut World, id: ItemId, f: impl FnOnce(&mut MoongateData)) {
    if let Some(d) = w.item_mut(id).and_then(|i| i.ext.get_mut::<MoongateData>()) {
        f(d);
    }
}

/// The virtuals of `Moongate` a subclass overrides.
pub struct MoongateClass {
    pub show_felucca_warning: bool,
    /// `ShowFeluccaWarning => Core.AOS` (GateTravelMoongate): true when the era is AOS or later.
    pub felucca_warning_is_aos: bool,
    pub check_gate: fn(&mut World, ItemId, MobileId, i32),
    pub use_gate: fn(&mut World, ItemId, MobileId),
    /// `BeginConfirmation(from)`.
    pub begin_confirmation: fn(&mut World, ItemId, MobileId),
}

pub static MOONGATE_CLASS: MoongateClass = MoongateClass {
    show_felucca_warning: false,
    felucca_warning_is_aos: false,
    check_gate: check_gate_base,
    use_gate: use_gate_base,
    begin_confirmation,
};

/// `ShowFeluccaWarning`.
pub fn felucca_warning(w: &World, id: ItemId) -> bool {
    let c = class(w, id);
    c.show_felucca_warning || (c.felucca_warning_is_aos && muo_content::core::aos(w))
}

fn class(w: &World, id: ItemId) -> &'static MoongateClass {
    w.item(id)
        .and_then(|i| i.ty.family::<MoongateClass>())
        .unwrap_or(&MOONGATE_CLASS)
}

/// `Moongate(target, targetMap, dispellable)`.
pub fn moongate_new(
    w: &mut World,
    ty: &'static ItemType,
    target: Point3D,
    target_map: Option<MapId>,
    dispellable: bool,
) -> ItemId {
    let id = ops::new_item_with_id(w, ty, 0xF6C);
    ops::set_movable(w, id, false);
    ops::set_light(w, id, muo_server::light_type::LightType::Circle300);
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(MoongateData {
            target,
            target_map,
            dispellable,
            caster: None,
        });
    }
    id
}

pub fn check_gate_base(w: &mut World, id: ItemId, m: MobileId, range: i32) {
    let hidden_player = w
        .mobile(m)
        .is_some_and(|x| x.hidden && x.access_level == AccessLevel::Player);
    if hidden_player && muo_content::core::ml(w) {
        muo_server::mobile::base::revealing_action(w, m);
    }
    muo_server::timer::start(w, TimeSpan::from_secs(1), TimeSpan::ZERO, 1, move |w| {
        delay_callback(w, id, m, range)
    });
}

fn client_flag(w: &World, m: MobileId, flag: ClientFlags) -> bool {
    w.mobile(m)
        .and_then(|x| x.net_state)
        .and_then(|ns| w.net_state(ns))
        .is_some_and(|n| n.caps.has_flag(flag))
}

pub fn use_gate_base(w: &mut World, id: ItemId, m: MobileId) {
    let d = mg(w, id);
    let tm = d.target_map;
    let young = w.mobile(m).is_some_and(|x| x.player) && muo_content::player_mobile::young(w, m);
    let murderer = muo_server::mobile::base::murderer(w, m);
    let no_flag = |f: ClientFlags, map: MapId| tm == Some(map) && !client_flag(w, m, f);
    if sigil_exists_on(w, m) {
        msg(w, m, 1061632); // You can't do that while carrying the sigil.
    } else if tm == Some(MapId::FELUCCA) && young {
        msg(w, m, 1049543); // You decide against traveling to Felucca while you are still young.
    } else if (murderer && tm != Some(MapId::FELUCCA))
        || no_flag(ClientFlags::TOKUNO, MapId::TOKUNO)
        || no_flag(ClientFlags::MALAS, MapId::MALAS)
        || no_flag(ClientFlags::ILSHENAR, MapId::ILSHENAR)
        || no_flag(ClientFlags::TER_MUR, MapId::TER_MUR)
    {
        msg(w, m, 1019004); // You are not allowed to travel there.
    } else if w.mobile(m).is_some_and(|x| x.spell.is_some()) {
        msg(w, m, 1049616); // You are too busy to do that at the moment.
    } else if let Some(map) = tm.filter(|t| *t != MapId::INTERNAL) {
        muo_content::creature::pets::teleport_pets(w, m, d.target, map, false);
        muo_server::mobile::base::move_to_world(w, m, d.target, map);
        let (level, hidden) = w
            .mobile(m)
            .map_or((AccessLevel::Player, false), |x| (x.access_level, x.hidden));
        if level == AccessLevel::Player || !hidden {
            muo_content::mx::play_sound(w, m, 0x1FE);
        }
        // OnGateUsed: nothing
    } else {
        deps::send_message(w, m, "This moongate does not seem to go anywhere.");
    }
}

/// `Moongate.ValidateUse(from, message)`.
fn validate_use(w: &mut World, id: ItemId, from: MobileId, message: bool) -> bool {
    let (gone, from_map, gate_map) = (
        w.mobile(from).is_none_or(|m| m.deleted) || ops::is_deleted(w, id),
        w.mobile(from).map(|m| m.map),
        w.item(id).map(|i| i.map),
    );
    if gone {
        return false;
    }
    let loc = w.item(id).map_or_else(Point3D::default, |i| i.location);
    if from_map != gate_map || !deps::mobile_in_range(w, from, loc, 1) {
        if message {
            msg(w, from, 500446); // That is too far away.
        }
        return false;
    }
    true
}

/// `Moongate.IsInTown(p, map)`.
pub fn is_in_town(w: &World, p: Point3D, map: Option<MapId>) -> bool {
    let Some(map) = map else { return false };
    let r = muo_server::region::find(w, p, Some(map));
    muo_content::regions::get_guarded(w, r)
        .is_some_and(|g| !muo_content::regions::guarded::is_disabled(w, g))
}

pub fn begin_confirmation(w: &mut World, id: ItemId, from: MobileId) {
    let d = mg(w, id);
    let (loc, map) = w
        .mobile(from)
        .map_or((Point3D::default(), None), |m| (m.location, m.map));
    if (is_in_town(w, loc, map) && !is_in_town(w, d.target, d.target_map))
        || (map != Some(MapId::FELUCCA)
            && d.target_map == Some(MapId::FELUCCA)
            && felucca_warning(w, id))
    {
        let (level, hidden) = w
            .mobile(from)
            .map_or((AccessLevel::Player, false), |m| (m.access_level, m.hidden));
        if level == AccessLevel::Player || !hidden {
            muo_server::mobile::part2::effects::send_sound_at(w, from, 0x20E, loc);
        }
        if w.mobile(from).and_then(|m| m.net_state).is_some() && !ops::is_deleted(w, id) {
            let from_felucca_warning = map != Some(MapId::FELUCCA)
                && d.target_map == Some(MapId::FELUCCA)
                && felucca_warning(w, id);
            let g = MoongateConfirmGump {
                gate: id,
                from,
                aos: muo_content::core::aos(w),
                felucca_warning: from_felucca_warning,
            };
            let (x, y) = if g.aos { (110, 100) } else { (20, 30) };
            send_gump_to_mobile(w, from, Box::new(DynamicGumpObj::new(x, y, g)), false);
        }
    } else {
        end_confirmation(w, id, from);
    }
}

pub fn end_confirmation(w: &mut World, id: ItemId, from: MobileId) {
    if !validate_use(w, id, from, true) {
        return;
    }
    (class(w, id).use_gate)(w, id, from);
}

fn delay_callback(w: &mut World, id: ItemId, from: MobileId, range: i32) {
    let loc = w.item(id).map_or_else(Point3D::default, |i| i.location);
    if !validate_use(w, id, from, false) || !deps::mobile_in_range(w, from, loc, range) {
        return;
    }
    if mg(w, id).target_map.is_some() {
        (class(w, id).begin_confirmation)(w, id, from);
    } else {
        deps::send_message(w, from, "This moongate does not seem to go anywhere.");
    }
}

/// `MoongateConfirmGump`.
pub struct MoongateConfirmGump {
    gate: ItemId,
    from: MobileId,
    aos: bool,
    felucca_warning: bool,
}

impl DynamicGumpLogic for MoongateConfirmGump {
    const TYPE_NAME: &'static str = "Server.Items.MoongateConfirmGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        b.add_page(0);
        if self.aos {
            b.set_no_close();
            b.add_background(0, 0, 420, 280, 5054);
            b.add_image_tiled(10, 10, 400, 20, 2624);
            b.add_alpha_region(10, 10, 400, 20);
            b.add_html_localized_color(10, 10, 400, 20, 1062051, 30720, false, false); // Gate Warning
            b.add_image_tiled(10, 40, 400, 200, 2624);
            b.add_alpha_region(10, 40, 400, 200);
            if self.felucca_warning {
                // This Gate goes to Felucca... Continue to enter the gate, Cancel to stay here
                b.add_html_localized_color(10, 40, 400, 200, 1062050, 32512, false, true);
            } else {
                // Dost thou wish to step into the moongate? Continue to enter the gate, Cancel to stay here
                b.add_html_localized_color(10, 40, 400, 200, 1062049, 32512, false, true);
            }
            b.add_image_tiled(10, 250, 400, 20, 2624);
            b.add_alpha_region(10, 250, 400, 20);
            b.add_button(10, 250, 4005, 4007, 1);
            b.add_html_localized_color(40, 250, 170, 20, 1011036, 32767, false, false); // OKAY
            b.add_button(210, 250, 4005, 4007, 0);
            b.add_html_localized_color(240, 250, 170, 20, 1011012, 32767, false, false); // CANCEL
        } else {
            b.add_background(0, 0, 420, 400, 5054);
            b.add_background(10, 10, 400, 380, 3000);
            b.add_html_localized(20, 40, 380, 60, 1062049, false, false);
            b.add_html_localized(55, 110, 290, 20, 1011012, false, false); // CANCEL
            b.add_button(20, 110, 4005, 4007, 0);
            b.add_html_localized(55, 140, 290, 40, 1011011, false, false); // CONTINUE
            b.add_button(20, 140, 4005, 4007, 1);
        }
    }

    fn on_response(&mut self, w: &mut World, _sender: NetStateId, info: &RelayInfo) {
        if info.button_id == 1 {
            end_confirmation(w, self.gate, self.from);
        }
    }
}

fn serialize_moongate(w: &World, id: ItemId, out: &mut dyn GenericWriter) {
    (ITEM_BASE.serialize)(w, id, out);
    write_version(out, false);
    let d = mg(w, id);
    out.write_point3d(d.target);
    out.write_map(d.target_map);
    out.write_bool(d.dispellable);
}

fn deserialize_moongate(
    w: &mut World,
    id: ItemId,
    r: &mut dyn GenericReader,
) -> muo_io::Result<()> {
    (ITEM_BASE.deserialize)(w, id, r)?;
    read_version(r, false)?;
    let d = MoongateData {
        target: r.read_point3d()?,
        target_map: r.read_map()?,
        dispellable: r.read_bool()?,
        caster: None,
    };
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(d);
    }
    Ok(())
}

item_leaf_raw! {
    /// `Server.Items.Moongate` `[SerializationGenerator(0, false)]`.
    MOONGATE = "Server.Items.Moongate" : ITEM_BASE;
    construct: Some(|w| moongate_new(w, &MOONGATE, Point3D::default(), None, true)),
    family: Some(&MOONGATE_CLASS),
    on_double_click: |w, id, from| {
        if !w.mobile(from).is_some_and(|m| m.player) {
            return;
        }
        let loc = ops::get_world_location(w, id);
        if deps::mobile_in_range(w, from, loc, 1) {
            (class(w, id).check_gate)(w, id, from, 1);
        } else {
            msg(w, from, 500446); // That is too far away.
        }
    },
    on_move_over: |w, id, m| {
        if w.mobile(m).is_some_and(|x| x.player) {
            (class(w, id).check_gate)(w, id, m, 0);
        }
        true
    },
    serialize: |w, id, out| serialize_moongate(w, id, out),
    deserialize: |w, id, r| deserialize_moongate(w, id, r),
}

// ---- ConfirmationMoongate ----

/// The fields `ConfirmationMoongate` adds to a `Moongate`.
#[derive(Clone, Debug, Default)]
pub struct ConfirmationData {
    pub message: Option<muo_io::text::TextDefinition>,
    pub gump_width: i32,
    pub gump_height: i32,
    pub title_color: i32,
    pub message_color: i32,
    pub title_number: i32,
}

/// The `ConfirmationMoongate` fields of a gate.
pub fn confirmation(w: &World, id: ItemId) -> Option<&ConfirmationData> {
    w.item(id)?.ext.get::<ConfirmationData>()
}

/// Edits the `ConfirmationMoongate` fields of a gate.
pub fn confirmation_edit(w: &mut World, id: ItemId, f: impl FnOnce(&mut ConfirmationData)) {
    if let Some(d) = w
        .item_mut(id)
        .and_then(|i| i.ext.get_mut::<ConfirmationData>())
    {
        f(d);
    }
}

/// `ConfirmationMoongate.BeginConfirmation`: a `WarningGump` when the gate has a message, else the base.
fn confirmation_begin(w: &mut World, id: ItemId, from: MobileId) {
    let Some(d) = confirmation(w, id).cloned() else {
        return begin_confirmation(w, id, from);
    };
    if d.gump_width > 0
        && d.gump_height > 0
        && d.title_number > 0
        && d.message.as_ref().is_some_and(|m| !m.is_empty())
    {
        // Warning_Callback
        let cb: muo_gumps::m_z::warning::WarningCallback =
            std::rc::Rc::new(move |w: &mut World, okay| {
                if okay {
                    end_confirmation(w, id, from);
                }
            });
        let g = muo_gumps::m_z::warning::new_full(
            d.title_number,
            d.title_color,
            d.message.unwrap_or_default(),
            d.message_color,
            d.gump_width,
            d.gump_height,
            Some(cb),
            true,
        );
        send_gump_to_mobile(w, from, Box::new(g), true);
    } else {
        begin_confirmation(w, id, from);
    }
}

static CONFIRMATION_MOONGATE_CLASS: MoongateClass = MoongateClass {
    show_felucca_warning: false,
    felucca_warning_is_aos: false,
    check_gate: check_gate_base,
    use_gate: use_gate_base,
    begin_confirmation: confirmation_begin,
};

/// The virtuals of a `ConfirmationMoongate` subclass: its own `CheckGate`/`UseGate`, the confirmation of the base.
pub const fn confirmation_class(
    check_gate: fn(&mut World, ItemId, MobileId, i32),
    use_gate: fn(&mut World, ItemId, MobileId),
) -> MoongateClass {
    MoongateClass {
        show_felucca_warning: false,
        felucca_warning_is_aos: false,
        check_gate,
        use_gate,
        begin_confirmation: confirmation_begin,
    }
}

/// `ConfirmationMoongate(target, targetMap)` for the class `ty` (a subclass passes its own).
pub fn confirmation_moongate_new(
    w: &mut World,
    ty: &'static ItemType,
    target: Point3D,
    target_map: Option<MapId>,
) -> ItemId {
    let id = moongate_new(w, ty, target, target_map, true);
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(ConfirmationData::default());
    }
    id
}

/// `ConfirmationMoongate`'s serializer: the moongate, then version 1 and the six fields.
pub fn serialize_confirmation(w: &World, id: ItemId, out: &mut dyn GenericWriter) {
    serialize_moongate(w, id, out);
    out.write_i32(1);
    let d = confirmation(w, id).cloned().unwrap_or_default();
    out.write_text_definition(d.message.as_ref());
    out.write_i32(d.gump_width);
    out.write_i32(d.gump_height);
    out.write_i32(d.title_color);
    out.write_i32(d.message_color);
    out.write_i32(d.title_number);
}

/// `ConfirmationMoongate.Deserialize`. (Version 0 goes to the legacy `Deserialize(reader, version)`, which calls the
/// base `Deserialize` again and reads the fields as encoded ints; that is kept.)
pub fn deserialize_confirmation(
    w: &mut World,
    id: ItemId,
    r: &mut dyn GenericReader,
) -> muo_io::Result<()> {
    deserialize_moongate(w, id, r)?;
    let version = r.read_i32()?;
    let d = if version < 1 {
        deserialize_moongate(w, id, r)?;
        let (gump_width, gump_height) = (r.read_encoded_int()?, r.read_encoded_int()?);
        let (title_color, message_color) = (r.read_encoded_int()?, r.read_encoded_int()?);
        let title_number = r.read_encoded_int()?;
        let number = r.read_encoded_int()?;
        let message = r.read_string()?;
        ConfirmationData {
            message: Some(if number > 0 {
                muo_io::text::TextDefinition::of_number(number)
            } else {
                muo_io::text::TextDefinition::of(0, message.as_deref())
            }),
            gump_width,
            gump_height,
            title_color,
            message_color,
            title_number,
        }
    } else {
        ConfirmationData {
            message: r.read_text_definition()?,
            gump_width: r.read_i32()?,
            gump_height: r.read_i32()?,
            title_color: r.read_i32()?,
            message_color: r.read_i32()?,
            title_number: r.read_i32()?,
        }
    };
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(d);
    }
    Ok(())
}

item_leaf_raw! {
    /// `Server.Items.ConfirmationMoongate` `[SerializationGenerator(1, false)]`.
    CONFIRMATION_MOONGATE = "Server.Items.ConfirmationMoongate" : MOONGATE;
    construct: Some(|w| confirmation_moongate_new(w, &CONFIRMATION_MOONGATE, Point3D::default(), None)),
    family: Some(&CONFIRMATION_MOONGATE_CLASS),
    serialize: |w, id, out| serialize_confirmation(w, id, out),
    deserialize: |w, id, r| deserialize_confirmation(w, id, r),
}

// ---- MoonstoneGate ----

fn party_of_caster(w: &World, caster: Option<MobileId>, m: MobileId) -> bool {
    let Some(c) = caster else { return false };
    m == c
        || muo_content::spells::hooks::hooks(w)
            .and_then(|h| h.party_contains)
            .is_some_and(|f| f(w, c, m))
}

fn moonstone_check_gate(w: &mut World, id: ItemId, m: MobileId, range: i32) {
    if muo_server::mobile::base::murderer(w, m) {
        return;
    }
    if party_of_caster(w, mg(w, id).caster, m) {
        check_gate_base(w, id, m, range);
    }
}

fn moonstone_use_gate(w: &mut World, id: ItemId, m: MobileId) {
    if muo_server::mobile::base::murderer(w, m) {
        return;
    }
    if party_of_caster(w, mg(w, id).caster, m) {
        use_gate_base(w, id, m);
    }
}

static MOONSTONE_GATE_CLASS: MoongateClass = MoongateClass {
    show_felucca_warning: false,
    felucca_warning_is_aos: false,
    check_gate: moonstone_check_gate,
    use_gate: moonstone_use_gate,
    begin_confirmation,
};

/// `MoonstoneGate(loc, map, targetMap, caster, hue)`.
pub fn moonstone_gate_new(
    w: &mut World,
    loc: Point3D,
    map: Option<MapId>,
    target_map: Option<MapId>,
    caster: MobileId,
    hue: i32,
) -> ItemId {
    let id = moongate_new(w, &MOONSTONE_GATE, loc, target_map, true);
    ops::move_to_world_map(w, id, loc, map);
    mg_edit(w, id, |d| {
        d.dispellable = false;
        d.caster = Some(caster);
    });
    super::hue(w, id, hue);
    muo_server::timer::start(w, TimeSpan::from_secs(30), TimeSpan::ZERO, 1, move |w| {
        if let Some(ty) = w.item(id).map(|i| i.ty) {
            (ty.delete)(w, id);
        }
    });
    muo_server::effects::play_sound(w, loc, map, 0x20E);
    id
}

item_leaf_raw! {
    /// `Server.Items.MoonstoneGate` `[SerializationGenerator(0, false)]`.
    MOONSTONE_GATE = "Server.Items.MoonstoneGate" : MOONGATE;
    construct: None,
    family: Some(&MOONSTONE_GATE_CLASS),
    skip_serialization: |_, _| true,
    serialize: |w, id, out| {
        serialize_moongate(w, id, out);
        write_version(out, false);
    },
    deserialize: |w, id, r| {
        deserialize_moongate(w, id, r)?;
        read_version(r, false)?;
        Ok(())
    },
}

// ---- HornOfRetreatMoongate ----

/// `HornOfRetreatMoongate.UseGate`: only the caster, never from a jail, and the gate goes with the trip.
fn horn_gate_use_gate(w: &mut World, id: ItemId, m: MobileId) {
    if muo_content::player_mobile::hooks::in_jail_region(w, m) {
        msg(w, m, 1114345); // You'll need a better jailbreak plan than that!
    } else if mg(w, id).caster == Some(m) {
        use_gate_base(w, id, m);
        if let Some(ty) = w.item(id).map(|i| i.ty) {
            (ty.delete)(w, id);
        }
    }
}

static HORN_GATE_CLASS: MoongateClass = MoongateClass {
    show_felucca_warning: false,
    felucca_warning_is_aos: false,
    check_gate: check_gate_base,
    use_gate: horn_gate_use_gate,
    // `BeginConfirmation(from)` => `EndConfirmation(from)`
    begin_confirmation: end_confirmation,
};

/// `HornOfRetreatMoongate(destLoc, destMap, caster, hue)`: gone after ten seconds. The caller moves it into the world.
pub fn horn_gate_new(
    w: &mut World,
    dest: Point3D,
    dest_map: Option<MapId>,
    caster: MobileId,
    hue: i32,
) -> ItemId {
    let id = moongate_new(w, &HORN_OF_RETREAT_MOONGATE, dest, dest_map, false);
    mg_edit(w, id, |d| d.caster = Some(caster));
    super::hue(w, id, hue);
    muo_server::timer::start(w, TimeSpan::from_secs(10), TimeSpan::ZERO, 1, move |w| {
        if let Some(ty) = w.item(id).map(|i| i.ty) {
            (ty.delete)(w, id);
        }
    });
    id
}

item_leaf_raw! {
    /// `Server.Engines.Quests.HornOfRetreatMoongate` `[SerializationGenerator(0, false)]`.
    HORN_OF_RETREAT_MOONGATE = "Server.Engines.Quests.HornOfRetreatMoongate" : MOONGATE;
    construct: None,
    family: Some(&HORN_GATE_CLASS),
    label_number: |_, _| 1049114, // Sanctuary Gate
    skip_serialization: |_, _| true,
    serialize: |w, id, out| {
        serialize_moongate(w, id, out);
        write_version(out, false);
    },
    deserialize: |w, id, r| {
        deserialize_moongate(w, id, r)?;
        read_version(r, false)?;
        Ok(())
    },
}

// ---- Moonstone ----

fn moonstone_type(w: &World, id: ItemId) -> i32 {
    w.item(id)
        .and_then(|i| i.ext.get::<i32>())
        .copied()
        .unwrap_or(0)
}

fn target_map_of(w: &World, id: ItemId) -> MapId {
    if moonstone_type(w, id) == 0 {
        MapId::FELUCCA
    } else {
        MapId::TRAMMEL
    }
}

/// `Moonstone(type)`: 0 Felucca, 1 Trammel.
pub fn moonstone_new(w: &mut World, ty: i32) -> ItemId {
    let id = ops::new_item_with_id(w, &MOONSTONE, 0xF8B);
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(ty);
    }
    id
}

fn moonstone_double_click(w: &mut World, id: ItemId, from: MobileId) {
    let pack = w.mobile(from).and_then(|m| m.backpack).map(EntityId::Item);
    if !ops::is_child_of(w, id, pack) {
        msg(w, from, 1042001); // That must be in your pack for you to use it.
        return;
    }
    let target = target_map_of(w, id);
    let (map, loc) = w
        .mobile(from)
        .map_or((None, Point3D::default()), |m| (m.map, m.location));
    let mounted = muo_server::mobile::part1::props_b::mounted(w, from);
    let human = muo_content::creature::misc::body_is_human(w, from);
    if mounted {
        msg(w, from, 1005399); // You can not bury a stone while you sit on a mount.
    } else if !human {
        msg(w, from, 1005400); // You can not bury a stone in this form.
    } else if sigil_exists_on(w, from) {
        msg(w, from, 1061632); // You can't do that while carrying the sigil.
    } else if map == Some(target) || (map != Some(MapId::TRAMMEL) && map != Some(MapId::FELUCCA)) {
        msg(w, from, 1005401); // You cannot bury the stone here.
    } else if w.mobile(from).is_some_and(|m| m.player) && muo_content::player_mobile::young(w, from)
    {
        msg(w, from, 1049543); // You decide against traveling to Felucca while you are still young.
    } else if muo_server::mobile::base::murderer(w, from) {
        // The magic of the stone cannot be evoked by someone with blood on their hands.
        msg(w, from, 1005402);
    } else if w.mobile(from).is_some_and(|m| m.criminal) {
        msg(w, from, 1005403); // The magic of the stone cannot be evoked by the lawless.
    } else if !muo_server::region::is_default(w, muo_server::region::find(w, loc, map))
        || !muo_server::region::is_default(w, muo_server::region::find(w, loc, Some(target)))
    {
        msg(w, from, 1005401);
    } else if !muo_server::map::fit::can_fit_point(w, target, loc, 16) {
        msg(w, from, 1005408); // Something is blocking the facet gate exit.
    } else {
        ops::set_movable(w, id, false);
        ops::move_to_world_map(w, id, loc, map);
        muo_content::mx::animate(w, from, 32, 5, 1, true, false, 0);
        settle_timer(w, id, loc, map, target, from);
    }
}

fn settle_timer(
    w: &mut World,
    stone: ItemId,
    loc: Point3D,
    map: Option<MapId>,
    target: MapId,
    caster: MobileId,
) {
    let mut count = 0;
    let handle = std::rc::Rc::new(std::cell::Cell::new(
        muo_server::timer::TimerHandle::default(),
    ));
    let h2 = handle.clone();
    let h = muo_server::timer::start(
        w,
        TimeSpan::from_secs_f64(2.5),
        TimeSpan::from_secs(1),
        0,
        move |w| {
            count += 1;
            let stop = |w: &mut World| {
                let mut hh = h2.get();
                hh.cancel(w);
            };
            if count == 1 {
                messages::public_overhead_message_number(
                    w,
                    stone,
                    ItemMessage::REGULAR,
                    0x3B2,
                    1005414,
                    "",
                ); // The stone settles into the ground.
            } else if count >= 10 {
                let l = w.item(stone).map_or(Point3D::default(), |i| i.location);
                ops::set_location(w, stone, Point3D::new(l.x, l.y, l.z - 1));
                if count == 16 {
                    let blocked =
                        !muo_server::region::is_default(w, muo_server::region::find(w, loc, map))
                            || !muo_server::region::is_default(
                                w,
                                muo_server::region::find(w, loc, Some(target)),
                            )
                            || !muo_server::map::fit::can_fit_point(w, target, loc, 16);
                    if blocked {
                        ops::set_movable(w, stone, true);
                        deps::add_to_backpack(w, caster, stone);
                        stop(w);
                        return;
                    }
                    let mut hue = w.item(stone).map_or(0, |i| i.hue);
                    if hue == 0 {
                        hue = utility::random_bird_hue(&mut w.rng);
                    }
                    moonstone_gate_new(w, loc, Some(target), map, caster, hue);
                    moonstone_gate_new(w, loc, map, Some(target), caster, hue);
                    if let Some(ty) = w.item(stone).map(|i| i.ty) {
                        (ty.delete)(w, stone);
                    }
                    stop(w);
                }
            }
        },
    );
    handle.set(h);
}

item_leaf_raw! {
    /// `Server.Items.Moonstone` `[SerializationGenerator(1, false)]`.
    MOONSTONE = "Server.Items.Moonstone" : ITEM_BASE;
    construct: None,
    default_weight: |_, _| 1.0,
    label_number: |w, id| 1041490 + moonstone_type(w, id),
    on_single_click: |w, id, from| {
        let pack = w.mobile(from).and_then(|m| m.backpack).map(EntityId::Item);
        if ops::is_child_of(w, id, pack) {
            let h = utility::random_bird_hue(&mut w.rng);
            super::hue(w, id, h);
            if let Some(ty) = w.item(id).map(|i| i.ty) {
                (ty.process_delta)(w, id);
            }
            msg(w, from, 1005398); // The stone's substance shifts as you examine it.
        }
        base::on_single_click_base(w, id, from);
    },
    on_double_click: moonstone_double_click,
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(1);
        out.write_enum(4, i64::from(moonstone_type(w, id)));
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let version = r.read_i32()?;
        let ty = if version < 1 { r.read_i32()? } else { r.read_enum(4)? as i32 };
        if let Some(i) = w.item_mut(id) {
            i.ext = Ext::new(ty);
        }
        Ok(())
    },
}

// ---- PublicMoongate ----

/// `PMEntry`.
pub struct PmEntry {
    pub location: Point3D,
    pub number: i32,
}

const fn e(x: i32, y: i32, z: i32, number: i32) -> PmEntry {
    PmEntry {
        location: Point3D::new(x, y, z),
        number,
    }
}

/// `PMList`.
pub struct PmList {
    pub number: i32,
    pub sel_number: i32,
    pub map: MapId,
    pub entries: &'static [PmEntry],
}

/// Where an entry sends you: Magincia's z is `Map.GetAverageZ(3563, 2139)` of its facet (5 on the stock maps), the
/// others are fixed.
fn entry_location(w: &mut World, list: &PmList, entry: &PmEntry) -> Point3D {
    let mut l = entry.location;
    if entry.number == 1012010 {
        l.z = w.maps.get_average_z(list.map, l.x, l.y);
    }
    l
}

static TRAMMEL_ENTRIES: [PmEntry; 9] = [
    e(4467, 1283, 5, 1012003),
    e(1336, 1997, 5, 1012004),
    e(1499, 3771, 5, 1012005),
    e(771, 752, 5, 1012006),
    e(2701, 692, 5, 1012007),
    e(1828, 2948, -20, 1012008),
    e(643, 2067, 5, 1012009),
    e(3563, 2139, 5, 1012010),
    e(3450, 2677, 25, 1078098),
];
static FELUCCA_ENTRIES: [PmEntry; 9] = [
    e(4467, 1283, 5, 1012003),
    e(1336, 1997, 5, 1012004),
    e(1499, 3771, 5, 1012005),
    e(771, 752, 5, 1012006),
    e(2701, 692, 5, 1012007),
    e(1828, 2948, -20, 1012008),
    e(643, 2067, 5, 1012009),
    e(3563, 2139, 5, 1012010),
    e(2711, 2234, 0, 1019001),
];
static ILSHENAR_ENTRIES: [PmEntry; 9] = [
    e(1215, 467, -13, 1012015),
    e(722, 1366, -60, 1012016),
    e(744, 724, -28, 1012017),
    e(281, 1016, 0, 1012018),
    e(987, 1011, -32, 1012019),
    e(1174, 1286, -30, 1012020),
    e(1532, 1340, -3, 1012021),
    e(528, 216, -45, 1012022),
    e(1721, 218, 96, 1019000),
];
static MALAS_ENTRIES: [PmEntry; 2] = [e(1015, 527, -65, 1060641), e(1997, 1386, -85, 1060642)];
static TOKUNO_ENTRIES: [PmEntry; 3] = [
    e(1169, 998, 41, 1063412),
    e(802, 1204, 25, 1063413),
    e(270, 628, 15, 1063414),
];
static TERMUR_ENTRIES: [PmEntry; 1] = [e(850, 3525, -38, 1113603)];
static TERMUR_EODON_ENTRIES: [PmEntry; 2] = [e(850, 3525, -38, 1113603), e(719, 1863, 40, 1156262)];

pub static TRAMMEL: PmList = PmList {
    number: 1012000,
    sel_number: 1012012,
    map: MapId::TRAMMEL,
    entries: &TRAMMEL_ENTRIES,
};
pub static FELUCCA: PmList = PmList {
    number: 1012001,
    sel_number: 1012013,
    map: MapId::FELUCCA,
    entries: &FELUCCA_ENTRIES,
};
pub static ILSHENAR: PmList = PmList {
    number: 1012002,
    sel_number: 1012014,
    map: MapId::ILSHENAR,
    entries: &ILSHENAR_ENTRIES,
};
pub static MALAS: PmList = PmList {
    number: 1060643,
    sel_number: 1062039,
    map: MapId::MALAS,
    entries: &MALAS_ENTRIES,
};
pub static TOKUNO: PmList = PmList {
    number: 1063258,
    sel_number: 1063415,
    map: MapId::TOKUNO,
    entries: &TOKUNO_ENTRIES,
};
pub static TER_MUR: PmList = PmList {
    number: 1113602,
    sel_number: 1113604,
    map: MapId::TER_MUR,
    entries: &TERMUR_ENTRIES,
};
pub static TER_MUR_EODON: PmList = PmList {
    number: 1113602,
    sel_number: 1113604,
    map: MapId::TER_MUR,
    entries: &TERMUR_EODON_ENTRIES,
};

type Lists = &'static [&'static PmList];

static NO_TRAMMEL: Lists = &[&FELUCCA];
static T2A: Lists = &[&TRAMMEL, &FELUCCA];
static T2A_YOUNG: Lists = &[&TRAMMEL];
static LBR: Lists = &[&TRAMMEL, &FELUCCA, &ILSHENAR];
static LBR_YOUNG: Lists = &[&TRAMMEL, &ILSHENAR];
static AOS: Lists = &[&TRAMMEL, &FELUCCA, &ILSHENAR, &MALAS];
static AOS_YOUNG: Lists = &[&TRAMMEL, &ILSHENAR, &MALAS];
static SE: Lists = &[&TRAMMEL, &FELUCCA, &ILSHENAR, &MALAS, &TOKUNO];
static SE_YOUNG: Lists = &[&TRAMMEL, &ILSHENAR, &MALAS, &TOKUNO];
static SA: Lists = &[&TRAMMEL, &FELUCCA, &ILSHENAR, &MALAS, &TOKUNO, &TER_MUR];
static SA_YOUNG: Lists = &[&TRAMMEL, &ILSHENAR, &MALAS, &TOKUNO, &TER_MUR];
static TOL: Lists = &[
    &TRAMMEL,
    &FELUCCA,
    &ILSHENAR,
    &MALAS,
    &TOKUNO,
    &TER_MUR_EODON,
];
static TOL_YOUNG: Lists = &[&TRAMMEL, &ILSHENAR, &MALAS, &TOKUNO, &TER_MUR_EODON];
static RED: Lists = &[&FELUCCA];
static SIGIL: Lists = &[&FELUCCA];

/// `PublicMoongate()`.
pub fn public_moongate_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = ops::new_item_with_id(w, ty, 0xF6C);
    ops::set_movable(w, id, false);
    ops::set_light(w, id, muo_server::light_type::LightType::Circle300);
    id
}

fn pm_use_gate(w: &mut World, id: ItemId, m: MobileId) -> bool {
    if w.mobile(m).is_some_and(|x| x.criminal) {
        msg_hue(w, m, 1005561, 0x22); // Thou'rt a criminal and cannot escape so easily.
        return false;
    }
    if muo_content::spells::helper::check_combat(w, m) {
        msg_hue(w, m, 1005564, 0x22); // Wouldst thou flee during the heat of battle??
        return false;
    }
    if w.mobile(m).is_some_and(|x| x.spell.is_some()) {
        msg(w, m, 1049616); // You are too busy to do that at the moment.
        return false;
    }
    display_gump(w, m, id);
    let (level, hidden, loc, map) = w.mobile(m).map_or(
        (AccessLevel::Player, false, Point3D::default(), None),
        |x| (x.access_level, x.hidden, x.location, x.map),
    );
    if !hidden || level == AccessLevel::Player {
        muo_server::effects::play_sound(w, loc, map, 0x20E);
    }
    true
}

/// `MoongateGump`: `_lists` has the active map first, `_tab_order` the original order.
pub struct MoongateGump {
    gate: ItemId,
    lists: Vec<&'static PmList>,
    tab_order: Vec<&'static PmList>,
}

fn display_gump(w: &mut World, m: MobileId, gate: ItemId) {
    if w.mobile(m).and_then(|x| x.net_state).is_none() || ops::is_deleted(w, gate) {
        return;
    }
    let expansion = w.expansion();
    let young = w.mobile(m).is_some_and(|x| x.player) && muo_content::player_mobile::young(w, m);
    let check: Lists = if !w.mobile(m).is_some_and(|x| x.player) {
        SE
    } else if sigil_exists_on(w, m) {
        SIGIL
    } else if muo_server::mobile::base::murderer(w, m) {
        RED
    } else {
        let f = |flag: ClientFlags| client_flag(w, m, flag);
        let pick = |a: Lists, b: Lists| if young { b } else { a };
        if expansion >= Expansion::TOL && f(ClientFlags::TER_MUR) {
            pick(TOL, TOL_YOUNG)
        } else if expansion >= Expansion::SA && f(ClientFlags::TER_MUR) {
            pick(SA, SA_YOUNG)
        } else if expansion >= Expansion::SE && f(ClientFlags::TOKUNO) {
            pick(SE, SE_YOUNG)
        } else if expansion >= Expansion::AOS && f(ClientFlags::MALAS) {
            pick(AOS, AOS_YOUNG)
        } else if expansion >= Expansion::LBR && f(ClientFlags::ILSHENAR) {
            pick(LBR, LBR_YOUNG)
        } else if expansion >= Expansion::T2A && f(ClientFlags::TRAMMEL) {
            pick(T2A, T2A_YOUNG)
        } else {
            NO_TRAMMEL
        }
    };
    let available = core_maps(w);
    let filtered: Vec<&'static PmList> = check
        .iter()
        .copied()
        .filter(|l| available.contains(map_flag(l.map)))
        .collect();
    if filtered.is_empty() {
        return;
    }
    let tab_order = filtered.clone();
    let mut lists = filtered;
    let here = w.mobile(m).and_then(|x| x.map);
    if let Some(i) = lists.iter().position(|l| Some(l.map) == here) {
        lists.swap(i, 0);
    }
    send_gump_to_mobile(
        w,
        m,
        Box::new(DynamicGumpObj::new(
            100,
            100,
            MoongateGump {
                gate,
                lists,
                tab_order,
            },
        )),
        false,
    );
}

impl DynamicGumpLogic for MoongateGump {
    const TYPE_NAME: &'static str = "Server.Items.MoongateGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        let index_of = |list: &Vec<&'static PmList>, l: &PmList| {
            list.iter().position(|x| std::ptr::eq(*x, l)).unwrap_or(0) as i32
        };
        b.add_page(0);
        b.add_background(0, 0, 380, 280, 5054);
        b.add_button(10, 210, 4005, 4007, 1);
        b.add_html_localized(45, 210, 140, 25, 1011036, false, false); // OKAY
        b.add_button(10, 235, 4005, 4007, 0);
        b.add_html_localized(45, 235, 140, 25, 1011012, false, false); // CANCEL
        b.add_html_localized(5, 5, 200, 20, 1012011, false, false); // Pick your destination:
        // Tab buttons on page 0, in the original order so the active map appears at its expected slot.
        for (i, l) in self.tab_order.iter().enumerate() {
            b.add_button_type(
                10,
                35 + i as i32 * 25,
                2117,
                2118,
                0,
                GumpButtonType::Page,
                index_of(&self.lists, l) + 1,
            );
            b.add_html_localized(30, 35 + i as i32 * 25, 150, 20, l.number, false, false);
        }
        for (index, list) in self.lists.iter().enumerate() {
            let offset = index_of(&self.tab_order, list);
            b.add_page(index as i32 + 1);
            b.add_button_type(
                10,
                35 + offset * 25,
                2117,
                2118,
                0,
                GumpButtonType::Page,
                index as i32 + 1,
            );
            b.add_html_localized(30, 35 + offset * 25, 150, 20, list.sel_number, false, false);
            for (i, entry) in list.entries.iter().enumerate() {
                b.add_radio(
                    200,
                    35 + i as i32 * 25,
                    210,
                    211,
                    false,
                    index as i32 * 100 + i as i32,
                );
                b.add_html_localized(225, 35 + i as i32 * 25, 150, 20, entry.number, false, false);
            }
        }
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        if info.button_id == 0 || ops::is_deleted(w, self.gate) {
            return;
        }
        let Some(from) = w.net_states.get(&sender).and_then(|n| n.mobile) else {
            return;
        };
        if w.mobile(from).and_then(|m| m.map).is_none() {
            return;
        }
        let Some(&switch_id) = info.switches.first() else {
            return;
        };
        let (list_index, list_entry) = (switch_id / 100, switch_id % 100);
        if list_index < 0 || list_index as usize >= self.lists.len() {
            return;
        }
        let list = self.lists[list_index as usize];
        if list_entry < 0 || list_entry as usize >= list.entries.len() {
            return;
        }
        let entry = &list.entries[list_entry as usize];
        let dest = entry_location(w, list, entry);
        let gate_loc = ops::get_world_location(w, self.gate);
        let gate_map = w.item(self.gate).and_then(|i| i.map);
        let (fmap, floc, player) = w
            .mobile(from)
            .map_or((None, Point3D::default(), false), |m| {
                (m.map, m.location, m.player)
            });
        if !deps::mobile_in_range(w, from, gate_loc, 1) || fmap != gate_map {
            msg(w, from, 1019002); // You are too far away to use the gate.
        } else if (player
            && muo_server::mobile::base::murderer(w, from)
            && list.map != MapId::FELUCCA)
            || (sigil_exists_on(w, from) && Some(list.map) != faction_facet(w))
        {
            msg(w, from, 1019004); // You are not allowed to travel there.
        } else if w.mobile(from).is_some_and(|m| m.criminal) {
            msg_hue(w, from, 1005561, 0x22); // Thou'rt a criminal and cannot escape so easily.
        } else if muo_content::spells::helper::check_combat(w, from) {
            msg_hue(w, from, 1005564, 0x22); // Wouldst thou flee during the heat of battle??
        } else if w.mobile(from).is_some_and(|m| m.spell.is_some()) {
            msg(w, from, 1049616); // You are too busy to do that at the moment.
        } else if fmap == Some(list.map) && utility::in_range_3d(floc, dest, 1) {
            msg(w, from, 1019003); // You are already there.
        } else {
            muo_content::creature::pets::teleport_pets(w, from, dest, list.map, false);
            muo_content::mx::set_combatant(w, from, None);
            muo_content::mx::set_warmode(w, from, false);
            muo_server::mobile::part1::props_b::set_hidden(w, from, true);
            muo_server::mobile::base::move_to_world(w, from, dest, list.map);
            muo_server::effects::play_sound(w, dest, Some(list.map), 0x1FE);
        }
    }
}

item_leaf_raw! {
    /// `Server.Items.PublicMoongate` `[SerializationGenerator(0, false)]`.
    PUBLIC_MOONGATE = "Server.Items.PublicMoongate" : ITEM_BASE;
    construct: Some(|w| public_moongate_new(w, &PUBLIC_MOONGATE)),
    force_show_properties: |w, _| deps::opl_enabled(w),
    handles_on_movement: |_, _| true,
    on_double_click: |w, id, from| {
        if !w.mobile(from).is_some_and(|m| m.player) {
            return;
        }
        let loc = ops::get_world_location(w, id);
        if deps::mobile_in_range(w, from, loc, 1) {
            pm_use_gate(w, id, from);
        } else {
            msg(w, from, 500446); // That is too far away.
        }
    },
    on_move_over: |w, id, m| {
        // Changed so criminals are not blocked by it.
        if w.mobile(m).is_some_and(|x| x.player) {
            pm_use_gate(w, id, m);
        }
        true
    },
    on_movement: |w, id, m, old| {
        if !w.mobile(m).is_some_and(|x| x.ty.is(&muo_content::player_mobile::PLAYER_MOBILE)) {
            return;
        }
        let loc = w.item(id).map_or_else(Point3D::default, |i| i.location);
        let now = w.mobile(m).map_or_else(Point3D::default, |x| x.location);
        if !utility::in_range_3d(now, loc, 1) && utility::in_range_3d(old, loc, 1)
            && let Some(ns) = w.mobile(m).and_then(|x| x.net_state) {
                muo_content::gumps::close_gump::<MoongateGump>(w, ns);
            }
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        write_version(out, false);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        read_version(r, false)?;
        Ok(())
    },
}

/// `PublicMoongate.MoonGen`: removes all the old moongates and makes one per entry of the lists the core
/// expansion has maps for.
fn moon_gen_command(w: &mut World, _e: &mut CommandEventArgs) {
    let old: Vec<ItemId> = w
        .items
        .iter()
        .filter(|(_, i)| i.ty.is(&PUBLIC_MOONGATE) && !i.deleted())
        .map(|(id, _)| *id)
        .collect();
    for i in &old {
        if let Some(ty) = w.item(*i).map(|x| x.ty) {
            (ty.delete)(w, *i);
        }
    }
    if !old.is_empty() {
        w.broadcast(0x35, true, &format!("{} moongates removed.", old.len()));
    }
    let available = core_maps(w);
    let mut count = 0;
    for (list, flag) in [
        (&TRAMMEL, MapSelectionFlags::TRAMMEL),
        (&FELUCCA, MapSelectionFlags::FELUCCA),
        (&ILSHENAR, MapSelectionFlags::ILSHENAR),
        (&MALAS, MapSelectionFlags::MALAS),
        (&TOKUNO, MapSelectionFlags::TOKUNO),
    ] {
        if !available.contains(flag) {
            continue;
        }
        for entry in list.entries {
            let item = public_moongate_new(w, &PUBLIC_MOONGATE);
            let at = entry_location(w, list, entry);
            ops::move_to_world_map(w, item, at, Some(list.map));
            if entry.number == 1060642 {
                // Umbra
                super::hue(w, item, 0x497);
            }
        }
        count += list.entries.len();
    }
    w.broadcast(0x35, true, &format!("{count} moongates generated."));
}

fn moongates_configure(w: &mut World) {
    // Usage: MoonGen; Description: Generates public moongates. Removes all old moongates.
    CommandSystem::register(
        w,
        "MoonGen",
        AccessLevel::Developer,
        &[],
        std::rc::Rc::new(moon_gen_command),
    );
    muo_content::spells::hooks::hooks_mut(w).moongate_not_dispellable =
        Some(|w, id| w.item(id).is_some_and(|i| i.ty.is(&MOONGATE)) && !mg(w, id).dispellable);
    let d = muo_content::props::add;
    let _ = d;
}

static MOONSTONE_CTORS: [muo_content::props::Ctor; 1] = [muo_content::props::Ctor {
    access: None,
    params: &[muo_content::props::Param {
        name: "type",
        kind: muo_content::props::Kind::Plain(muo_content::utilities::types::PropType::Enum(&[
            ("Felucca", 0),
            ("Trammel", 1),
        ])),
        default: None,
        ty: "MoonstoneType",
    }],
    make: |w, v| EntityId::Item(moonstone_new(w, v[0].i32())),
}];

static MOONGATE_CTORS: [muo_content::props::Ctor; 3] = [
    muo_content::props::Ctor {
        access: None,
        params: &[muo_content::props::Param {
            name: "dispellable",
            kind: muo_content::props::Kind::Plain(muo_content::utilities::types::PropType::Bool),
            default: Some(muo_content::props::Value::Bool(true)),
            ty: "",
        }],
        make: |w, v| {
            EntityId::Item(moongate_new(
                w,
                &MOONGATE,
                Point3D::default(),
                None,
                v[0].bool(),
            ))
        },
    },
    muo_content::props::Ctor {
        access: None,
        params: &[
            muo_content::props::Param {
                name: "target",
                kind: muo_content::props::Kind::Plain(
                    muo_content::utilities::types::PropType::IPoint3D,
                ),
                default: None,
                ty: "",
            },
            muo_content::props::Param {
                name: "targetMap",
                kind: muo_content::props::Kind::Map,
                default: Some(muo_content::props::Value::Map(None)),
                ty: "",
            },
            muo_content::props::Param {
                name: "dispellable",
                kind: muo_content::props::Kind::Plain(
                    muo_content::utilities::types::PropType::Bool,
                ),
                default: Some(muo_content::props::Value::Bool(true)),
                ty: "",
            },
        ],
        make: |w, v| {
            let map = match &v[1] {
                muo_content::props::Value::Map(m) => *m,
                _ => None,
            };
            EntityId::Item(moongate_new(w, &MOONGATE, v[0].point(), map, v[2].bool()))
        },
    },
    // `Moongate()` is the first overload with its defaults: no separate entry
    muo_content::props::Ctor {
        access: None,
        params: &[],
        make: |w, _| EntityId::Item(moongate_new(w, &MOONGATE, Point3D::default(), None, true)),
    },
];

/// `ConfirmationMoongate()` is `this(Point3D.Zero)`; `(target, targetMap = null)`.
static CONFIRMATION_CTORS: [muo_content::props::Ctor; 2] = [
    muo_content::props::Ctor {
        access: None,
        params: &[],
        make: |w, _| {
            EntityId::Item(confirmation_moongate_new(
                w,
                &CONFIRMATION_MOONGATE,
                Point3D::default(),
                None,
            ))
        },
    },
    muo_content::props::Ctor {
        access: None,
        params: &[
            muo_content::props::Param {
                name: "target",
                kind: muo_content::props::Kind::Plain(
                    muo_content::utilities::types::PropType::IPoint3D,
                ),
                default: None,
                ty: "",
            },
            muo_content::props::Param {
                name: "targetMap",
                kind: muo_content::props::Kind::Map,
                default: Some(muo_content::props::Value::Map(None)),
                ty: "",
            },
        ],
        make: |w, v| {
            let map = match &v[1] {
                muo_content::props::Value::Map(m) => *m,
                _ => None,
            };
            EntityId::Item(confirmation_moongate_new(
                w,
                &CONFIRMATION_MOONGATE,
                v[0].point(),
                map,
            ))
        },
    },
];

// ---- GateTravelMoongate ----

/// `GateTravelMoongate.LinkedGate`.
#[derive(Clone, Copy, Debug, Default)]
pub struct LinkedGate(pub Option<ItemId>);

/// `GateTravelMoongate.UseGate(m)`: goes to wherever the linked gate is now.
fn gate_travel_use_gate(w: &mut World, id: ItemId, m: MobileId) {
    let linked = w
        .item(id)
        .and_then(|i| i.ext.get::<LinkedGate>())
        .and_then(|l| l.0)
        .filter(|&l| !ops::is_deleted(w, l));
    let Some(linked) = linked else {
        deps::send_message(w, m, "The other gate no longer exists.");
        return;
    };

    let (target, target_map) = w
        .item(linked)
        .map_or((Point3D::default(), None), |i| (i.location, i.map));

    // TODO: boat permissions
    mg_edit(w, id, |d| {
        d.target = target;
        d.target_map = target_map;
    });

    use_gate_base(w, id, m);
}

static GATE_TRAVEL_MOONGATE_CLASS: MoongateClass = MoongateClass {
    show_felucca_warning: false,
    felucca_warning_is_aos: true, // ShowFeluccaWarning => Core.AOS
    check_gate: check_gate_base,
    use_gate: gate_travel_use_gate,
    begin_confirmation,
};

/// `new GateTravelMoongate(target, map)` moved to `(at, at_map)`, as the spell does.
pub fn gate_travel_moongate_new(
    w: &mut World,
    target: Point3D,
    target_map: Option<MapId>,
    at: Point3D,
    at_map: Option<MapId>,
) -> ItemId {
    let id = moongate_new(w, &GATE_TRAVEL_MOONGATE, target, target_map, true);
    // Map = map; the item is not in the world yet, so the location comes with MoveToWorld below.
    if felucca_warning_aos(w)
        && target_map == Some(MapId::FELUCCA)
        && let Some(f) = w.item(id).map(|i| i.ty.set_item_id)
    {
        f(w, id, 0xDDA);
    }
    mg_edit(w, id, |d| d.dispellable = true);
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(LinkedGate::default());
    }
    let life = if muo_content::core::t2a(w) { 30 } else { 10 };
    muo_server::timer::start(w, TimeSpan::from_secs(life), TimeSpan::ZERO, 1, move |w| {
        if let Some(ty) = w.item(id).map(|i| i.ty) {
            (ty.delete)(w, id);
        }
    });
    ops::move_to_world_map(w, id, at, at_map);
    id
}

fn felucca_warning_aos(w: &World) -> bool {
    muo_content::core::aos(w)
}

/// `a.LinkedGate = b; b.LinkedGate = a`.
pub fn gate_travel_moongates_link(w: &mut World, a: ItemId, b: ItemId) {
    for (x, y) in [(a, b), (b, a)] {
        if let Some(i) = w.item_mut(x) {
            i.ext.insert(LinkedGate(Some(y)));
        }
    }
}

item_leaf_raw! {
    /// `Server.Spells.Seventh.GateTravelMoongate` `[SerializationGenerator(0)]`.
    GATE_TRAVEL_MOONGATE = "Server.Spells.Seventh.GateTravelMoongate" : MOONGATE;
    construct: None,
    family: Some(&GATE_TRAVEL_MOONGATE_CLASS),
    skip_serialization: |_, _| true,
    serialize: |w, id, out| {
        serialize_moongate(w, id, out);
        write_version(out, true);
    },
    deserialize: |w, id, r| {
        deserialize_moongate(w, id, r)?;
        read_version(r, true)?;
        Ok(())
    },
}

fn gate_travel_hooks(w: &mut World) {
    let h = muo_content::spells::hooks::hooks_mut(w);
    h.new_gate_travel_moongate = Some(|w, target, target_map, at, at_map| {
        Some(gate_travel_moongate_new(w, target, target_map, at, at_map))
    });
    h.link_gate_travel_moongates = Some(gate_travel_moongates_link);
}

pub(crate) fn moongates_tables(w: &mut World) {
    muo_content::props::add_ctors(w, "Server.Items.Moonstone", &MOONSTONE_CTORS);
    muo_content::props::add_ctors(w, "Server.Items.Moongate", &MOONGATE_CTORS[..2]);
    muo_content::props::add_ctors(w, "Server.Items.ConfirmationMoongate", &CONFIRMATION_CTORS);
}

/// The classes of this file.
pub static ALL: &[&ItemType] = &[
    &MOONGATE,
    &CONFIRMATION_MOONGATE,
    &MOONSTONE_GATE,
    &GATE_TRAVEL_MOONGATE,
    &MOONSTONE,
    &PUBLIC_MOONGATE,
];

/// Registers the classes of this file.
pub fn register(types: &mut TypeRegistry) {
    for t in ALL {
        types.register_item(t, &[]);
    }
    // the quest's gate: made by the horn only (no constructor)
    types.register_item(&HORN_OF_RETREAT_MOONGATE, &[]);
    types.register_phase_default(
        Phase::Configure,
        "Server.Items.PublicMoongate.Configure",
        moongates_configure,
    );
    types.register_phase_default(
        Phase::Configure,
        "Server.Spells.Seventh.GateTravelMoongate.Hooks",
        gate_travel_hooks,
    );
    types.register_phase_default(
        Phase::Configure,
        "Server.Items.Moonstone.Constructors",
        moongates_tables,
    );
    let _ = (MessageType::REGULAR, ItemMessage::REGULAR);
}

#[cfg(test)]
mod gate_travel_tests {
    use super::*;
    use muo_data::expansion::Expansion;

    #[test]
    fn a_gate_travel_pair_sends_the_walker_to_the_other_gate_and_stops_when_it_is_gone() {
        let mut w = muo_server::map::test_support::world();
        w.core.expansion = Some(Expansion::AOS);
        let a = gate_travel_moongate_new(
            &mut w,
            Point3D::new(300, 300, 0),
            Some(MapId::FELUCCA),
            Point3D::new(100, 100, 0),
            Some(MapId::FELUCCA),
        );
        let b = gate_travel_moongate_new(
            &mut w,
            Point3D::new(100, 100, 0),
            Some(MapId::FELUCCA),
            Point3D::new(300, 300, 0),
            Some(MapId::FELUCCA),
        );
        gate_travel_moongates_link(&mut w, a, b);
        assert_eq!(w.item(a).unwrap().location, Point3D::new(100, 100, 0));
        assert_eq!(
            w.item(a).unwrap().item_id,
            0xDDA,
            "AOS: the gate to Felucca shows the warning graphic"
        );

        let walker = muo_content::player_mobile::new_player_mobile(&mut w);
        muo_server::mobile::base::move_to_world(
            &mut w,
            walker,
            Point3D::new(100, 100, 0),
            MapId::FELUCCA,
        );
        gate_travel_use_gate(&mut w, a, walker);
        assert_eq!(
            w.mobile(walker).unwrap().location,
            Point3D::new(300, 300, 0)
        );

        // The gates live for 30 seconds; the other one is gone after that.
        let ty = w.item(b).unwrap().ty;
        (ty.delete)(&mut w, b);
        gate_travel_use_gate(&mut w, a, walker);
        assert_eq!(
            w.mobile(walker).unwrap().location,
            Point3D::new(300, 300, 0),
            "stayed"
        );
        assert!(
            ops::is_deleted(&w, b) && !ops::is_deleted(&w, a),
            "only the deleted one is gone"
        );
        // The dispel check and the spell's hooks find the class by its type name.
        assert_eq!(
            w.item(a).unwrap().ty.full_name,
            "Server.Spells.Seventh.GateTravelMoongate"
        );
        assert!(!not_dispellable(&w, a));
    }
}
