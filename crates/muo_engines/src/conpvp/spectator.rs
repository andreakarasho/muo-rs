//! `DuelContext.ArenaMoongate`: the spectator gate that appears where the duel was
//! challenged for ten seconds. A `ConfirmationMoongate` with its own warning text; it is never saved.

use muo_io::text::TextDefinition;
use muo_items::misc::moongates::{
    CONFIRMATION_MOONGATE, MoongateData, check_gate_base, confirmation_class, confirmation_edit,
    confirmation_moongate_new, use_gate_base,
};
use muo_primitives::{Point3D, TimeSpan};
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::ItemType;
use muo_server::map::MapId;
use muo_server::type_registry::TypeRegistry;

use super::arena;
use super::flow::check_combat;

const COMBAT_TEXT: &str =
    "You have recently been in combat with another player and cannot use this moongate.";

/// `m_Teleporter`.
#[derive(Clone, Copy, Debug, Default)]
struct GateExit(Option<ItemId>);

fn check_gate(w: &mut World, id: ItemId, m: MobileId, range: i32) {
    if check_combat(w, m) {
        muo_server::item::deps::send_message_hue(w, m, 0x22, COMBAT_TEXT);
    } else {
        check_gate_base(w, id, m, range);
    }
}

fn use_gate(w: &mut World, id: ItemId, m: MobileId) {
    if check_combat(w, m) {
        muo_server::item::deps::send_message_hue(w, m, 0x22, COMBAT_TEXT);
        return;
    }
    let tp = w
        .item(id)
        .and_then(|i| i.ext.get::<GateExit>())
        .and_then(|g| g.0);
    if let Some(tp) = tp.filter(|t| !muo_server::item::ops::is_deleted(w, *t)) {
        arena::teleporter_register(w, tp, m);
    }
    use_gate_base(w, id, m);
}

static ARENA_MOONGATE_CLASS: muo_items::misc::moongates::MoongateClass =
    confirmation_class(check_gate, use_gate);

/// `DuelContext.ArenaMoongate`.
pub static ARENA_MOONGATE: ItemType = ItemType {
    full_name: "Server.Engines.ConPVP.DuelContext+ArenaMoongate",
    parent: Some(&CONFIRMATION_MOONGATE),
    construct: None,
    family: Some(&ARENA_MOONGATE_CLASS),
    default_name: |w, id| {
        crate::template_keys::item_name(w, id, CONFIRMATION_MOONGATE.default_name)
    },
    skip_serialization: |_, _| true,
    ..CONFIRMATION_MOONGATE
};

/// `new ArenaMoongate(target, map, tp)`.
pub fn new_arena_moongate(
    w: &mut World,
    target: Point3D,
    map: Option<MapId>,
    tp: Option<ItemId>,
) -> ItemId {
    let id = confirmation_moongate_new(w, &ARENA_MOONGATE, target, map);
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(GateExit(tp));
    }
    muo_server::item::base::set_item_id(w, id, 0x1FD4);
    if let Some(d) = w.item_mut(id).and_then(|i| i.ext.get_mut::<MoongateData>()) {
        d.dispellable = false;
    }
    confirmation_edit(w, id, |d| {
        d.gump_width = 300;
        d.gump_height = 150;
        d.message_color = 0xFFC000;
        d.message = Some(TextDefinition::of_text(
            "Are you sure you wish to spectate this duel?",
        ));
        d.title_color = 0x7800;
        d.title_number = 1062051; // Gate Warning
    });
    muo_server::timer::start(w, TimeSpan::from_secs(10), TimeSpan::ZERO, 1, move |w| {
        if let Some(ty) = w.item(id).map(|i| i.ty) {
            (ty.delete)(w, id);
        }
    });
    id
}

/// `Appear(loc, map)`.
pub fn appear(w: &mut World, id: ItemId, loc: Point3D, map: Option<MapId>) {
    muo_server::effects::play_sound(w, loc, map, 0x20E);
    muo_server::item::ops::move_to_world_map(w, id, loc, map);
}

/// Registers the class.
pub fn register(types: &mut TypeRegistry) {
    types.register_item(&ARENA_MOONGATE, &[]);
}
