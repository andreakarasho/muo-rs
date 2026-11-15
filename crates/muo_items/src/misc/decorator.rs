//! `InteriorDecorator`.

use muo_content::gumps::base::{DynamicGumpLogic, DynamicGumpObj, RelayInfo};
use muo_content::gumps::builder::DynamicGumpBuilder;
use muo_content::gumps::send_gump_to_mobile;
use muo_content::items::{addon, addon_container, flip};
use muo_primitives::Point3D;
use muo_server::World;
use muo_server::ext::Ext;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, TotalType, base, deps, ops};
use muo_server::net_state::NetStateId;
use muo_server::target::{self, TargetCancelType, Targeted};
use muo_server::type_registry::TypeRegistry;

use super::{read_version, write_version};

/// `DecorateCommand`.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum DecorateCommand {
    #[default]
    None = 0,
    Turn = 1,
    Up = 2,
    Down = 3,
}

impl DecorateCommand {
    fn from_i32(n: i32) -> Self {
        match n {
            1 => Self::Turn,
            2 => Self::Up,
            3 => Self::Down,
            _ => Self::None,
        }
    }
}

fn command(w: &World, id: ItemId) -> DecorateCommand {
    w.item(id)
        .and_then(|i| i.ext.get::<DecorateCommand>())
        .copied()
        .unwrap_or_default()
}

/// `InteriorDecorator.Command = value`.
pub fn set_command(w: &mut World, id: ItemId, value: DecorateCommand) {
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(value);
    }
    ops::invalidate_properties(w, id);
}

/// `InteriorDecorator()`.
pub fn interior_decorator_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.InteriorDecorator")
}

/// `InteriorDecorator.CheckUse(from)`.
pub fn check_use(w: &mut World, from: MobileId) -> bool {
    use muo_multis::house::access;
    if access::find_house_at_mobile(w, from).is_some_and(|h| access::is_co_owner(w, h, from)) {
        return true;
    }
    deps::send_localized_message(w, from, 502092); // You must be in your house to do this.
    false
}

fn begin_decorator_target(w: &mut World, from: MobileId, decorator: ItemId) {
    let mut t = target::Target::new(w, -1, false, target::TargetFlags::NONE);
    t.check_los = false;
    let on_target = std::rc::Rc::new(move |w: &mut World, from: MobileId, targeted: Targeted| {
        decorator_target(w, decorator, from, targeted)
    });
    t.handlers.on_target = Some(on_target.clone());
    t.handlers.on_target_not_accessible = Some(on_target);
    t.handlers.on_target_cancel = Some(std::rc::Rc::new(
        move |w: &mut World, from: MobileId, kind: TargetCancelType| {
            if kind == TargetCancelType::Canceled
                && let Some(ns) = w.mobile(from).and_then(|m| m.net_state)
            {
                muo_content::gumps::close_gump::<InteriorDecoratorGump>(w, ns);
            }
        },
    ));
    let tid = target::add_target(w, t);
    target::set_mobile_target(w, from, Some(tid));
}

/// The addon an addon component, container component or addon container belongs to, and how many components.
fn addon_of(w: &World, item: ItemId) -> Option<(ItemId, usize)> {
    if addon::is_component(w, item) {
        let a = addon::component_addon(w, item)?;
        return Some((a, addon::components(w, a).len()));
    }
    if addon_container::is_container_component(w, item) {
        let a = addon_container::component_addon(w, item)?;
        return Some((a, addon_container::components(w, a).len()));
    }
    if addon_container::is_container_addon(w, item) {
        return Some((item, addon_container::components(w, item).len()));
    }
    None
}

fn has_flippable_addon(w: &World, a: ItemId) -> bool {
    let Some(ty) = w.item(a).map(|i| i.ty) else {
        return false;
    };
    w.resources
        .get::<flip::AddonFlips>()
        .is_some_and(|f| f.0.contains_key(ty.full_name))
}

fn decorator_target(w: &mut World, decorator: ItemId, from: MobileId, targeted: Targeted) {
    use muo_multis::house::{access, storage};
    if let Targeted::Entity(EntityId::Item(item)) = targeted
        && check_use(w, from)
    {
        let house = access::find_house_at_mobile(w, from);
        let mut decorable_component = false;
        let addon_info = addon_of(w, item);
        let cmd = command(w, decorator);
        if let Some((a, count)) = addon_info {
            if count == 1 && muo_content::core::se(w) {
                decorable_component = true;
            }
            if cmd == DecorateCommand::Turn && has_flippable_addon(w, a) {
                decorable_component = true;
            }
        }
        let house = house.filter(|h| access::is_co_owner(w, *h, from));
        let Some(h) = house else {
            deps::send_localized_message(w, from, 502092); // You must be in your house to do this.
            begin_decorator_target(w, from, decorator);
            return;
        };
        if w.item(item).is_some_and(|i| i.parent.is_some()) || !access::is_inside_item(w, h, item) {
            deps::send_localized_message(w, from, 1042270); // That is not in your house.
        } else if !storage::has_locked_down_item(w, h, item)
            && !storage::has_secure_item(w, h, item)
            && !decorable_component
        {
            if addon::is_component(w, item) && cmd == DecorateCommand::Up {
                deps::send_localized_message(w, from, 1042274); // You cannot raise it up any higher.
            } else if addon::is_component(w, item) && cmd == DecorateCommand::Down {
                deps::send_localized_message(w, from, 1042275); // You cannot lower it down any further.
            } else {
                deps::send_localized_message(w, from, 1042271); // That is not locked down.
            }
        } else if muo_content::factory::item_is(w, item, "Server.Items.VendorRentalContract") {
            deps::send_localized_message(w, from, 1062491); // You cannot use the house decorator on that object.
        } else if ops::get_total(w, item, TotalType::Weight) + ops::pile_weight(w, item) > 100 {
            deps::send_localized_message(w, from, 1042272); // That is too heavy.
        } else {
            match cmd {
                DecorateCommand::Up => up(w, item, from),
                DecorateCommand::Down => down(w, item, from),
                DecorateCommand::Turn => turn(w, item, from),
                DecorateCommand::None => {}
            }
        }
    }
    begin_decorator_target(w, from, decorator);
}

fn turn(w: &mut World, item: ItemId, from: MobileId) {
    if let Some((a, _)) = addon_of(w, item)
        && has_flippable_addon(w, a)
    {
        flip::flip_addon(w, from, a);
        return;
    }
    if !flip::flip(w, item) {
        deps::send_localized_message(w, from, 1042273); // You cannot turn that.
    }
}

fn floor_z(w: &mut World, item: ItemId) -> i32 {
    let Some((loc, Some(map))) = w.item(item).map(|i| (i.location, i.map)) else {
        return i32::MIN;
    };
    let mut z = i32::MIN;
    for tile in muo_server::map::tiles::static_and_multi_tiles(w, map, loc.x, loc.y) {
        let data = w.maps.tile_data.item(tile.id as usize);
        let top = i32::from(tile.z);
        let (surface, impassable) = (
            data.flags.contains(muo_data::tile_data::TileFlag::SURFACE),
            data.flags
                .contains(muo_data::tile_data::TileFlag::IMPASSABLE),
        );
        if surface && !impassable && top > z && top <= loc.z {
            z = top;
        }
    }
    z
}

fn up(w: &mut World, item: ItemId, from: MobileId) {
    let fz = floor_z(w, item);
    let loc = w.item(item).map_or_else(Point3D::default, |i| i.location);
    if fz > i32::MIN && loc.z < fz + 15 {
        ops::set_location(w, item, Point3D::new(loc.x, loc.y, loc.z + 1));
    } else {
        deps::send_localized_message(w, from, 1042274); // You cannot raise it up any higher.
    }
}

fn down(w: &mut World, item: ItemId, from: MobileId) {
    let fz = floor_z(w, item);
    let loc = w.item(item).map_or_else(Point3D::default, |i| i.location);
    if fz > i32::MIN && loc.z > floor_z(w, item) {
        ops::set_location(w, item, Point3D::new(loc.x, loc.y, loc.z - 1));
    } else {
        deps::send_localized_message(w, from, 1042275); // You cannot lower it down any further.
    }
}

/// `InteriorDecorator.InteriorDecoratorGump`.
pub struct InteriorDecoratorGump {
    decorator: ItemId,
    command: DecorateCommand,
}

fn display_gump(w: &mut World, from: MobileId, decorator: ItemId) {
    if w.mobile(from).and_then(|m| m.net_state).is_none() || ops::is_deleted(w, decorator) {
        return;
    }
    let g = InteriorDecoratorGump {
        decorator,
        command: command(w, decorator),
    };
    send_gump_to_mobile(w, from, Box::new(DynamicGumpObj::new(150, 50, g)), false);
}

impl DynamicGumpLogic for InteriorDecoratorGump {
    const TYPE_NAME: &'static str = "Server.Items.InteriorDecorator+InteriorDecoratorGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        let pick = |c: DecorateCommand| if self.command == c { 2154 } else { 2152 };
        b.add_background(0, 0, 200, 200, 2600);
        b.add_button(50, 45, pick(DecorateCommand::Turn), 2154, 1);
        b.add_html_localized(90, 50, 70, 40, 1018323, false, false); // Turn
        b.add_button(50, 95, pick(DecorateCommand::Up), 2154, 2);
        b.add_html_localized(90, 100, 70, 40, 1018324, false, false); // Up
        b.add_button(50, 145, pick(DecorateCommand::Down), 2154, 3);
        b.add_html_localized(90, 150, 70, 40, 1018325, false, false); // Down
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let Some(from) = w.net_states.get(&sender).and_then(|n| n.mobile) else {
            return;
        };
        let cmd = match info.button_id {
            1 => DecorateCommand::Turn,
            2 => DecorateCommand::Up,
            3 => DecorateCommand::Down,
            _ => DecorateCommand::None,
        };
        if cmd != DecorateCommand::None {
            set_command(w, self.decorator, cmd);
            display_gump(w, from, self.decorator);
            begin_decorator_target(w, from, self.decorator);
        } else {
            target::cancel_mobile_target(w, from); // `Target.Cancel(sender.Mobile)` also sends the cancel packet
        }
    }
}

item_leaf_raw! {
    /// `Server.Items.InteriorDecorator`.
    INTERIOR_DECORATOR = "Server.Items.InteriorDecorator" : ITEM_BASE;
    construct: Some(|w| interior_decorator_new(w, &INTERIOR_DECORATOR)),
    default_weight: |_, _| 1.0,
    label_number: |_, _| 1041280, // an interior decorator
    get_properties: |w, id, list| {
        base::get_properties_base(w, id, list);
        let c = command(w, id);
        if c != DecorateCommand::None {
            list.add(1018322 + c as i32); // Turn/Up/Down
        }
    },
    on_double_click: |w, id, from| {
        if !check_use(w, from) {
            return;
        }
        let has = w.mobile(from).and_then(|m| m.net_state).is_some_and(|ns| {
            muo_content::gumps::has_gump::<InteriorDecoratorGump>(w, ns)
        });
        if !has {
            display_gump(w, from, id);
        }
        if command(w, id) != DecorateCommand::None {
            begin_decorator_target(w, from, id);
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

static INTERIOR_DECORATOR_PROPS: &[muo_content::props::PropDef] = &[muo_content::props::PropDef {
    name: "Command",
    kind: muo_content::props::Kind::Plain(muo_content::utilities::types::PropType::Enum(&[
        ("None", 0),
        ("Turn", 1),
        ("Up", 2),
        ("Down", 3),
    ])),
    read: muo_server::mobile::AccessLevel::GameMaster,
    write: muo_server::mobile::AccessLevel::GameMaster,
    get: |w, e| match e {
        EntityId::Item(i) => muo_content::props::Value::Enum(command(w, i) as i64),
        _ => muo_content::props::Value::Null,
    },
    set: Some(|w, e, v| {
        if let EntityId::Item(i) = e {
            set_command(w, i, DecorateCommand::from_i32(v.i32()));
        }
        Ok(())
    }),
}];

/// Registers the classes of this file.
pub fn register(types: &mut TypeRegistry) {
    types.register_item(&INTERIOR_DECORATOR, &[]);
    types.register_phase_default(
        muo_server::type_registry::Phase::Configure,
        "Server.Items.InteriorDecorator.Configure",
        |w| {
            muo_content::props::add(
                w,
                "Server.Items.InteriorDecorator",
                INTERIOR_DECORATOR_PROPS,
            )
        },
    );
}
