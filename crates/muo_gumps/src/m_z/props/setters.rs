//! The gumps that set one property: `SetGump`, `SetObjectGump`, `SetObjectTarget`, `SetPoint2DGump`,
//! `SetPoint3DGump`, `SetTimeSpanGump`, `SetListOptionGump` and `SetCustomEnumGump`.

use std::rc::Rc;

use muo_content::gumps::base::{LegacyGumpLogic, LegacyGumpObj, RelayInfo};
use muo_primitives::parse::{to_f64, to_i32, to_u32};
use muo_primitives::{Point2D, Point3D, Serial, TimeSpan};
use muo_server::World;
use muo_server::ids::{EntityId, MobileId};
use muo_server::net_state::NetStateId;
use muo_server::prompt::{Prompt, add_prompt};
use muo_server::target::{Target, TargetFlags, Targeted, add_target, set_mobile_target};

use super::host::{PropInfo, PropObj, PropType, PropValue, PropsHost};
use super::properties::{self, PropertiesContext};
use crate::m_z::common::{mobile_of, send};
use crate::m_z::hooks::props_host;
use muo_content::gumps::props_config::*;
use muo_content::gumps::props_layout::*;

const EXCEPTION_MESSAGE: &str = "An exception was caught. The property may not have changed.";

/// The fields every `Set*Gump` keeps: `m_Property`, `m_Mobile`, `m_Object`, `m_PropertiesGump`.
#[derive(Clone)]
pub struct Subject {
    pub prop: PropInfo,
    pub mobile: MobileId,
    pub object: PropObj,
    pub props: PropertiesContext,
}

impl Subject {
    /// The `try { LogChangeProperty; SetValue; OnValueChanged } catch { SendMessage(..) }` every setter has.
    fn apply(&self, w: &mut World, host: &dyn PropsHost, value: &PropValue) {
        host.log_change(w, self.mobile, &self.object, &self.prop.name, value);
        if host.set_value(w, &self.object, &self.prop, value).is_ok() {
            self.props.on_value_changed(w, &self.object, &self.prop);
        } else {
            muo_content::mx::send_message(w, self.mobile, EXCEPTION_MESSAGE);
        }
    }
}

// ---- SetGump ----

const SET_ENTRY_WIDTH: i32 = 212;
const SET_TOTAL_WIDTH: i32 = OFFSET_SIZE + SET_ENTRY_WIDTH + OFFSET_SIZE + SET_WIDTH + OFFSET_SIZE;

pub struct SetGump {
    s: Subject,
}

/// `new SetGump(prop, from, o, propertiesGump)`.
pub fn new_set(
    w: &mut World,
    prop: PropInfo,
    from: MobileId,
    o: PropObj,
    props: PropertiesContext,
) -> LegacyGumpObj<SetGump> {
    let can_null = prop.nullable;
    let can_dye = prop.hued;

    let initial_text = props_host(w).map_or_else(String::new, |h| h.text_value(w, &o, &prop));

    let row_count = 2 + i32::from(can_null) + i32::from(can_dye);

    let name = prop.name.clone();
    let mut g = LegacyGumpObj::new(
        GUMP_OFFSET_X,
        GUMP_OFFSET_Y,
        SetGump {
            s: Subject {
                prop,
                mobile: from,
                object: o,
                props,
            },
        },
    );
    let l = &mut g.layout;
    l.add_page(0);

    let (mut x, mut y) = add_props_frame(l, SET_TOTAL_WIDTH, row_count);
    add_props_entry_label(l, &mut x, &mut y, SET_ENTRY_WIDTH, &name);
    next_row(&mut x, &mut y);
    add_props_entry_text_input(
        l,
        &mut x,
        &mut y,
        SET_ENTRY_WIDTH,
        0,
        &initial_text,
        Some(1),
    );

    if can_null {
        next_row(&mut x, &mut y);
        add_props_entry_button(
            l,
            &mut x,
            &mut y,
            SET_ENTRY_WIDTH,
            "Null",
            Some(2),
            TEXT_HUE,
        );
    }

    if can_dye {
        next_row(&mut x, &mut y);
        add_props_entry_button(
            l,
            &mut x,
            &mut y,
            SET_ENTRY_WIDTH,
            "Hue Picker",
            Some(3),
            TEXT_HUE,
        );
    }
    g
}

impl LegacyGumpLogic for SetGump {
    const TYPE_NAME: &'static str = "Server.Gumps.SetGump";

    fn on_response(&mut self, w: &mut World, _sender: NetStateId, info: &RelayInfo) {
        let s = self.s.clone();
        let Some(host) = props_host(w) else { return };
        let mut to_set = PropValue::Null;
        let should_set;
        let mut should_send = true;

        match info.button_id {
            1 => {
                let text = info.get_text_entry(0).map(str::trim);

                if let Some(text) = text.filter(|t| !t.is_empty()) {
                    match host.parse(w, &s.prop, text) {
                        Ok(v) => {
                            to_set = v;
                            should_set = true;
                        }
                        // `Types.TryParse`'s error is ignored: a value that does not parse sets null.
                        Err(()) => should_set = true,
                    }
                } else {
                    should_set = false;
                }
            }
            2 => {
                // Null
                should_set = true;
            }
            3 => {
                // Hue Picker
                should_set = false;
                should_send = false;

                send_hue_picker(w, &s, host.clone());
            }
            _ => should_set = false,
        }

        if should_set {
            s.apply(w, host.as_ref(), &to_set);
        }

        if should_send {
            s.props.send(w);
        }
    }
}

/// `m_Mobile.SendHuePicker(new InternalPicker(..))`.
fn send_hue_picker(w: &mut World, s: &Subject, host: Rc<dyn PropsHost>) {
    let item_id = host.hued_item_id(w, &s.object);
    let mut picker = muo_server::hue_picker::HuePicker::new(w, item_id);
    let s2 = s.clone();
    picker.on_response = Some(Rc::new(move |w: &mut World, hue: i32| {
        s2.apply(w, host.as_ref(), &PropValue::Hue(hue));
        s2.props.send(w);
    }));
    if let Some(ns) = w.mobile(s.mobile).and_then(|m| m.net_state) {
        muo_server::hue_picker::send_to(w, ns, picker);
    }
}

// ---- SetObjectGump / SetObjectTarget ----

pub struct SetObjectGump {
    s: Subject,
    ty: PropType,
}

/// `new SetObjectGump(prop, mobile, o, type, propertiesGump)`.
pub fn new_set_object(
    w: &mut World,
    prop: PropInfo,
    mobile: MobileId,
    o: PropObj,
    ty: PropType,
    props: PropertiesContext,
) -> LegacyGumpObj<SetObjectGump> {
    let initial_text = props_host(w).map_or_else(String::new, |h| h.value_to_string(w, &o, &prop));
    let name = prop.name.clone();

    let mut g = LegacyGumpObj::new(
        GUMP_OFFSET_X,
        GUMP_OFFSET_Y,
        SetObjectGump {
            s: Subject {
                prop,
                mobile,
                object: o,
                props,
            },
            ty,
        },
    );
    let l = &mut g.layout;
    l.add_page(0);

    let (mut x, mut y) = add_props_frame(l, SET_TOTAL_WIDTH, 5);
    add_props_entry_label(l, &mut x, &mut y, SET_ENTRY_WIDTH, &name);
    next_row(&mut x, &mut y);
    add_props_entry_button(
        l,
        &mut x,
        &mut y,
        SET_ENTRY_WIDTH,
        &initial_text,
        Some(1),
        TEXT_HUE,
    );
    next_row(&mut x, &mut y);
    add_props_entry_button(
        l,
        &mut x,
        &mut y,
        SET_ENTRY_WIDTH,
        "Change by Serial",
        Some(2),
        TEXT_HUE,
    );
    next_row(&mut x, &mut y);
    add_props_entry_button(
        l,
        &mut x,
        &mut y,
        SET_ENTRY_WIDTH,
        "Nullify",
        Some(3),
        TEXT_HUE,
    );
    next_row(&mut x, &mut y);
    add_props_entry_button(
        l,
        &mut x,
        &mut y,
        SET_ENTRY_WIDTH,
        "View Properties",
        Some(4),
        TEXT_HUE,
    );
    g
}

/// `new SetObjectTarget(prop, mobile, o, type, propertiesGump)` and `Mobile.Target = ..`.
pub fn begin_set_object_target(
    w: &mut World,
    prop: PropInfo,
    mobile: MobileId,
    o: PropObj,
    ty: PropType,
    props: PropertiesContext,
) {
    let s = Subject {
        prop,
        mobile,
        object: o,
        props,
    };
    let mut t = Target::new(w, -1, false, TargetFlags::NONE);

    let (s1, ty1) = (s.clone(), ty.clone());
    t.handlers.on_target = Some(Rc::new(
        move |w: &mut World, _from: MobileId, targeted: Targeted| {
            let Some(host) = props_host(w) else { return };
            match host.assign_targeted(w, s1.mobile, &s1.object, &s1.prop, &ty1, targeted) {
                Ok(true) => s1.props.on_value_changed(w, &s1.object, &s1.prop),
                Ok(false) => {
                    let msg = format!(
                        "That cannot be assigned to a property of type : {}",
                        ty1.name
                    );
                    muo_content::mx::send_message(w, s1.mobile, &msg);
                }
                Err(()) => muo_content::mx::send_message(w, s1.mobile, EXCEPTION_MESSAGE),
            }
        },
    ));

    t.handlers.on_target_finish = Some(Rc::new(move |w: &mut World, from: MobileId| {
        if ty.is_system_type {
            s.props.send(w);
        } else {
            let g = new_set_object(
                w,
                s.prop.clone(),
                s.mobile,
                s.object.clone(),
                ty.clone(),
                s.props.clone(),
            );
            send(w, from, g);
        }
    }));

    let id = add_target(w, t);
    set_mobile_target(w, mobile, Some(id));
}

impl LegacyGumpLogic for SetObjectGump {
    const TYPE_NAME: &'static str = "Server.Gumps.SetObjectGump";

    fn on_response(&mut self, w: &mut World, _sender: NetStateId, info: &RelayInfo) {
        let s = self.s.clone();
        let ty = self.ty.clone();
        let Some(host) = props_host(w) else { return };
        let mut should_send = true;
        let mut view_props: Option<PropObj> = None;

        match info.button_id {
            0 => {
                // closed
                s.props.send(w);
                should_send = false;
            }
            1 => {
                // Change by Target
                begin_set_object_target(
                    w,
                    s.prop.clone(),
                    s.mobile,
                    s.object.clone(),
                    ty.clone(),
                    s.props.clone(),
                );
                should_send = false;
            }
            2 => {
                // Change by Serial
                should_send = false;

                muo_content::mx::send_message(w, s.mobile, "Enter the serial you wish to find:");
                begin_serial_prompt(w, &s, &ty);
            }
            3 => {
                // Nullify
                s.apply(w, host.as_ref(), &PropValue::Null);
            }
            4 => {
                // View Properties
                match host.value_object(w, &s.object, &s.prop) {
                    None => {
                        muo_content::mx::send_message(
                            w,
                            s.mobile,
                            "The property is null and so you cannot view its properties.",
                        );
                    }
                    Some(obj) if !host.is_accessible(w, s.mobile, &obj) => {
                        muo_content::mx::send_message(
                            w,
                            s.mobile,
                            "You may not view their properties.",
                        );
                    }
                    Some(obj) => view_props = Some(obj),
                }
            }
            _ => {}
        }

        if should_send {
            let g = new_set_object(
                w,
                s.prop.clone(),
                s.mobile,
                s.object.clone(),
                ty.clone(),
                s.props.clone(),
            );
            send(w, s.mobile, g);
        }

        if let Some(obj) = view_props {
            let g = properties::new(w, s.mobile, obj);
            send(w, s.mobile, g);
        }
    }
}

/// `m_Mobile.Prompt = new InternalPrompt(..)`.
fn begin_serial_prompt(w: &mut World, s: &Subject, ty: &PropType) {
    let (s1, ty1) = (s.clone(), ty.clone());
    let (s2, ty2) = (s.clone(), ty.clone());
    let prompt = Prompt {
        on_cancel: Some(Rc::new(move |w: &mut World, _from: MobileId| {
            let g = new_set_object(
                w,
                s1.prop.clone(),
                s1.mobile,
                s1.object.clone(),
                ty1.clone(),
                s1.props.clone(),
            );
            send(w, s1.mobile, g);
        })),
        on_response: Some(Rc::new(
            move |w: &mut World, _from: MobileId, text: &str| {
                let (s, ty) = (&s2, &ty2);
                if let Some(host) = props_host(w) {
                    let serial = Serial::new(to_u32(text));
                    match host.assign_serial(w, s.mobile, &s.object, &s.prop, ty, serial) {
                        Ok(None) => muo_content::mx::send_message(
                            w,
                            s.mobile,
                            "No object with that serial was found.",
                        ),
                        Ok(Some(false)) => {
                            let msg = format!(
                                "The object with that serial could not be assigned to a property of type : {}",
                                ty.name
                            );
                            muo_content::mx::send_message(w, s.mobile, &msg);
                        }
                        Ok(Some(true)) => s.props.on_value_changed(w, &s.object, &s.prop),
                        Err(()) => muo_content::mx::send_message(w, s.mobile, EXCEPTION_MESSAGE),
                    }
                }
                let g = new_set_object(
                    w,
                    s.prop.clone(),
                    s.mobile,
                    s.object.clone(),
                    ty.clone(),
                    s.props.clone(),
                );
                send(w, s.mobile, g);
            },
        )),
    };
    let id = add_prompt(w, prompt);
    muo_server::mobile::part1::props_b::set_prompt(w, s.mobile, Some(id));
}

// ---- SetPoint2DGump / SetPoint3DGump ----

/// The point a target gives (`targeted is IPoint3D`).
fn targeted_point(w: &World, t: Targeted) -> Option<Point3D> {
    match t {
        Targeted::Entity(EntityId::Mobile(m)) => w.mobile(m).map(|m| m.location),
        Targeted::Entity(EntityId::Item(i)) => w.item(i).map(|i| i.location),
        Targeted::Land(l) => Some(l.location),
        Targeted::Static(s) => Some(s.location),
    }
}

pub struct SetPoint2DGump {
    s: Subject,
}

const COORD_WIDTH_2D: i32 = 105;
const ENTRY_WIDTH_2D: i32 = COORD_WIDTH_2D + OFFSET_SIZE + COORD_WIDTH_2D;
const TOTAL_WIDTH_2D: i32 = OFFSET_SIZE + ENTRY_WIDTH_2D + OFFSET_SIZE + SET_WIDTH + OFFSET_SIZE;

/// One `X:`/`Y:`/`Z:` cell.
fn add_coord(
    l: &mut muo_content::gumps::legacy::LegacyLayout,
    x: i32,
    y: i32,
    width: i32,
    label: &str,
    id: i32,
    v: i32,
) {
    l.add_image_tiled(x, y, width, ENTRY_HEIGHT, ENTRY_GUMP_ID);
    l.add_label_cropped(
        x + TEXT_OFFSET_X,
        y,
        width - TEXT_OFFSET_X,
        ENTRY_HEIGHT,
        TEXT_HUE,
        label,
    );
    l.add_text_entry(
        x + 16,
        y,
        width - 16,
        ENTRY_HEIGHT,
        TEXT_HUE,
        id,
        &v.to_string(),
    );
}

/// `new SetPoint2DGump(prop, mobile, o, propertiesGump)`.
pub fn new_set_point2d(
    w: &mut World,
    prop: PropInfo,
    mobile: MobileId,
    o: PropObj,
    props: PropertiesContext,
) -> LegacyGumpObj<SetPoint2DGump> {
    let p = props_host(w).map_or(Point2D::new(0, 0), |h| h.point2d_value(w, &o, &prop));
    let name = prop.name.clone();

    let mut g = LegacyGumpObj::new(
        GUMP_OFFSET_X,
        GUMP_OFFSET_Y,
        SetPoint2DGump {
            s: Subject {
                prop,
                mobile,
                object: o,
                props,
            },
        },
    );
    let l = &mut g.layout;
    l.add_page(0);

    let (mut x, mut y) = add_props_frame(l, TOTAL_WIDTH_2D, 4);
    add_props_entry_label(l, &mut x, &mut y, ENTRY_WIDTH_2D, &name);
    next_row(&mut x, &mut y);
    add_props_entry_button(
        l,
        &mut x,
        &mut y,
        ENTRY_WIDTH_2D,
        "Use your location",
        Some(1),
        TEXT_HUE,
    );
    next_row(&mut x, &mut y);
    add_props_entry_button(
        l,
        &mut x,
        &mut y,
        ENTRY_WIDTH_2D,
        "Target a location",
        Some(2),
        TEXT_HUE,
    );
    next_row(&mut x, &mut y);

    add_coord(l, x, y, COORD_WIDTH_2D, "X:", 0, p.x);
    x += COORD_WIDTH_2D + OFFSET_SIZE;

    add_coord(l, x, y, COORD_WIDTH_2D, "Y:", 1, p.y);
    x += COORD_WIDTH_2D + OFFSET_SIZE;

    if SET_GUMP_ID != 0 {
        l.add_image_tiled(x, y, SET_WIDTH, ENTRY_HEIGHT, SET_GUMP_ID);
    }

    l.add_button(
        x + SET_OFFSET_X,
        y + SET_OFFSET_Y,
        SET_BUTTON_ID1,
        SET_BUTTON_ID2,
        3,
    );
    g
}

impl LegacyGumpLogic for SetPoint2DGump {
    const TYPE_NAME: &'static str = "Server.Gumps.SetPoint2DGump";

    fn on_response(&mut self, w: &mut World, _sender: NetStateId, info: &RelayInfo) {
        let s = self.s.clone();
        let Some(host) = props_host(w) else { return };
        let mut to_set = Point2D::new(0, 0);
        let (should_set, should_send);

        match info.button_id {
            1 => {
                // Current location
                let loc = w.mobile(s.mobile).map_or(Point3D::ZERO, |m| m.location);
                to_set = Point2D::new(loc.x, loc.y);
                should_set = true;
                should_send = true;
            }
            2 => {
                // Pick location
                begin_point_target(w, &s, false);
                should_set = false;
                should_send = false;
            }
            3 => {
                // Use values
                to_set = Point2D::new(
                    to_i32(info.get_text_entry(0).unwrap_or("")),
                    to_i32(info.get_text_entry(1).unwrap_or("")),
                );
                should_set = true;
                should_send = true;
            }
            _ => {
                should_set = false;
                should_send = true;
            }
        }

        if should_set {
            s.apply(w, host.as_ref(), &PropValue::Point2D(to_set));
        }

        if should_send {
            s.props.send(w);
        }
    }
}

pub struct SetPoint3DGump {
    s: Subject,
}

const COORD_WIDTH_3D: i32 = 70;
const ENTRY_WIDTH_3D: i32 =
    COORD_WIDTH_3D + OFFSET_SIZE + COORD_WIDTH_3D + OFFSET_SIZE + COORD_WIDTH_3D;
const TOTAL_WIDTH_3D: i32 = OFFSET_SIZE + ENTRY_WIDTH_3D + OFFSET_SIZE + SET_WIDTH + OFFSET_SIZE;

/// `new SetPoint3DGump(prop, mobile, o, propertiesGump)`.
pub fn new_set_point3d(
    w: &mut World,
    prop: PropInfo,
    mobile: MobileId,
    o: PropObj,
    props: PropertiesContext,
) -> LegacyGumpObj<SetPoint3DGump> {
    let p = props_host(w).map_or(Point3D::ZERO, |h| h.point3d_value(w, &o, &prop));
    let name = prop.name.clone();

    let mut g = LegacyGumpObj::new(
        GUMP_OFFSET_X,
        GUMP_OFFSET_Y,
        SetPoint3DGump {
            s: Subject {
                prop,
                mobile,
                object: o,
                props,
            },
        },
    );
    let l = &mut g.layout;
    l.add_page(0);

    let (mut x, mut y) = add_props_frame(l, TOTAL_WIDTH_3D, 4);
    add_props_entry_label(l, &mut x, &mut y, ENTRY_WIDTH_3D, &name);
    next_row(&mut x, &mut y);
    add_props_entry_button(
        l,
        &mut x,
        &mut y,
        ENTRY_WIDTH_3D,
        "Use your location",
        Some(1),
        TEXT_HUE,
    );
    next_row(&mut x, &mut y);
    add_props_entry_button(
        l,
        &mut x,
        &mut y,
        ENTRY_WIDTH_3D,
        "Target a location",
        Some(2),
        TEXT_HUE,
    );
    next_row(&mut x, &mut y);

    add_coord(l, x, y, COORD_WIDTH_3D, "X:", 0, p.x);
    x += COORD_WIDTH_3D + OFFSET_SIZE;

    add_coord(l, x, y, COORD_WIDTH_3D, "Y:", 1, p.y);
    x += COORD_WIDTH_3D + OFFSET_SIZE;

    add_coord(l, x, y, COORD_WIDTH_3D, "Z:", 2, p.z);
    x += COORD_WIDTH_3D + OFFSET_SIZE;

    if SET_GUMP_ID != 0 {
        l.add_image_tiled(x, y, SET_WIDTH, ENTRY_HEIGHT, SET_GUMP_ID);
    }

    l.add_button(
        x + SET_OFFSET_X,
        y + SET_OFFSET_Y,
        SET_BUTTON_ID1,
        SET_BUTTON_ID2,
        3,
    );
    g
}

impl LegacyGumpLogic for SetPoint3DGump {
    const TYPE_NAME: &'static str = "Server.Gumps.SetPoint3DGump";

    fn on_response(&mut self, w: &mut World, _sender: NetStateId, info: &RelayInfo) {
        let s = self.s.clone();
        let Some(host) = props_host(w) else { return };
        let mut to_set = Point3D::ZERO;
        let (should_set, should_send);

        match info.button_id {
            1 => {
                // Current location
                to_set = w.mobile(s.mobile).map_or(Point3D::ZERO, |m| m.location);
                should_set = true;
                should_send = true;
            }
            2 => {
                // Pick location
                begin_point_target(w, &s, true);
                should_set = false;
                should_send = false;
            }
            3 => {
                // Use values
                to_set = Point3D::new(
                    to_i32(info.get_text_entry(0).unwrap_or("")),
                    to_i32(info.get_text_entry(1).unwrap_or("")),
                    to_i32(info.get_text_entry(2).unwrap_or("")),
                );
                should_set = true;
                should_send = true;
            }
            _ => {
                should_set = false;
                should_send = true;
            }
        }

        if should_set {
            s.apply(w, host.as_ref(), &PropValue::Point3D(to_set));
        }

        if should_send {
            s.props.send(w);
        }
    }
}

/// `m_Mobile.Target = new InternalTarget(..)` (range -1, ground allowed).
fn begin_point_target(w: &mut World, s: &Subject, three_d: bool) {
    let mut t = Target::new(w, -1, true, TargetFlags::NONE);
    let s1 = s.clone();
    t.handlers.on_target = Some(Rc::new(
        move |w: &mut World, _from: MobileId, targeted: Targeted| {
            let Some(p) = targeted_point(w, targeted) else {
                return;
            };
            let Some(host) = props_host(w) else { return };
            let value = if three_d {
                PropValue::Point3D(p)
            } else {
                PropValue::Point2D(Point2D::new(p.x, p.y))
            };
            s1.apply(w, host.as_ref(), &value);
        },
    ));
    let s2 = s.clone();
    t.handlers.on_target_finish = Some(Rc::new(move |w: &mut World, _from: MobileId| {
        s2.props.send(w)
    }));
    let id = add_target(w, t);
    set_mobile_target(w, s.mobile, Some(id));
}

// ---- SetTimeSpanGump ----

pub struct SetTimeSpanGump {
    s: Subject,
}

/// `new SetTimeSpanGump(prop, mobile, o, propertiesGump)`.
pub fn new_set_time_span(
    w: &mut World,
    prop: PropInfo,
    mobile: MobileId,
    o: PropObj,
    props: PropertiesContext,
) -> LegacyGumpObj<SetTimeSpanGump> {
    let ts = props_host(w).map_or(TimeSpan::ZERO, |h| h.time_span_value(w, &o, &prop));
    let name = prop.name.clone();

    let mut g = LegacyGumpObj::new(
        GUMP_OFFSET_X,
        GUMP_OFFSET_Y,
        SetTimeSpanGump {
            s: Subject {
                prop,
                mobile,
                object: o,
                props,
            },
        },
    );
    let l = &mut g.layout;
    l.add_page(0);

    add_props_frame(l, SET_TOTAL_WIDTH, 7);

    let add_rect = |l: &mut muo_content::gumps::legacy::LegacyLayout,
                    index: i32,
                    s: &str,
                    button: i32,
                    text: i32| {
        let mut x = BORDER_SIZE + OFFSET_SIZE;
        let y = BORDER_SIZE + OFFSET_SIZE + index * (ENTRY_HEIGHT + OFFSET_SIZE);

        l.add_image_tiled(x, y, SET_ENTRY_WIDTH, ENTRY_HEIGHT, ENTRY_GUMP_ID);
        l.add_label_cropped(
            x + TEXT_OFFSET_X,
            y,
            SET_ENTRY_WIDTH - TEXT_OFFSET_X,
            ENTRY_HEIGHT,
            TEXT_HUE,
            s,
        );

        if text != -1 {
            l.add_text_entry(
                x + 16 + TEXT_OFFSET_X,
                y,
                SET_ENTRY_WIDTH - TEXT_OFFSET_X - 16,
                ENTRY_HEIGHT,
                TEXT_HUE,
                text,
                "",
            );
        }

        x += SET_ENTRY_WIDTH + OFFSET_SIZE;

        if SET_GUMP_ID != 0 {
            l.add_image_tiled(x, y, SET_WIDTH, ENTRY_HEIGHT, SET_GUMP_ID);
        }

        if button != 0 {
            l.add_button(
                x + SET_OFFSET_X,
                y + SET_OFFSET_Y,
                SET_BUTTON_ID1,
                SET_BUTTON_ID2,
                button,
            );
        }
    };

    add_rect(l, 0, &name, 0, -1);
    add_rect(l, 1, &ts.to_string(), 0, -1);
    add_rect(l, 2, "Zero", 1, -1);
    add_rect(l, 3, "From H:M:S", 2, -1);
    add_rect(l, 4, "H:", 3, 0);
    add_rect(l, 5, "M:", 4, 1);
    add_rect(l, 6, "S:", 5, 2);
    g
}

impl LegacyGumpLogic for SetTimeSpanGump {
    const TYPE_NAME: &'static str = "Server.Gumps.SetTimeSpanGump";

    fn on_response(&mut self, w: &mut World, _sender: NetStateId, info: &RelayInfo) {
        let s = self.s.clone();
        let Some(host) = props_host(w) else { return };

        let h = info.get_text_entry(0);
        let m = info.get_text_entry(1);
        let sec = info.get_text_entry(2);

        let mut to_set = TimeSpan::ZERO;
        let (should_set, should_send);

        match info.button_id {
            1 => {
                // Zero
                should_set = true;
                should_send = true;
            }
            2 => {
                // From H:M:S
                let mut ok = false;
                if let (Some(h), Some(m), Some(sec)) = (h, m, sec)
                    && let Some(ts) = TimeSpan::try_parse(&format!("{h}:{m}:{sec}"))
                {
                    to_set = ts;
                    ok = true;
                }
                should_set = ok;
                should_send = ok;
            }
            3..=5 => {
                // From H, M or S
                let (text, ticks_per): (Option<&str>, f64) = match info.button_id {
                    3 => (h, 36_000_000_000.0),
                    4 => (m, 600_000_000.0),
                    _ => (sec, 10_000_000.0),
                };
                // TimeSpan.FromHours/Minutes/Seconds throw (caught: nothing set, no gump) on NaN and overflow
                let ticks = text.map(|t| to_f64(t) * ticks_per);
                match ticks
                    .filter(|t| !t.is_nan() && *t <= i64::MAX as f64 && *t >= i64::MIN as f64)
                {
                    Some(t) => {
                        to_set = TimeSpan::from_ticks(t as i64);
                        should_set = true;
                        should_send = true;
                    }
                    None => {
                        should_set = false;
                        should_send = false;
                    }
                }
            }
            _ => {
                should_set = false;
                should_send = true;
            }
        }

        if should_set {
            s.apply(w, host.as_ref(), &PropValue::TimeSpan(to_set));
        }

        if should_send {
            s.props.send(w);
        }
    }
}

// ---- SetListOptionGump / SetCustomEnumGump ----

const LIST_ENTRY_COUNT: usize = 13;

/// `SetListOptionGump`'s layout, which `SetCustomEnumGump` shares.
fn build_list_layout(
    g_layout: &mut muo_content::gumps::legacy::LegacyLayout,
    prop_name: &str,
    names: &[String],
) {
    let l = g_layout;
    let pages = names.len().div_ceil(LIST_ENTRY_COUNT) as i32;
    let mut index = 0usize;

    let add_rect =
        |l: &mut muo_content::gumps::legacy::LegacyLayout, index: i32, s: &str, button: i32| {
            let mut x = BORDER_SIZE + OFFSET_SIZE;
            let y = BORDER_SIZE + OFFSET_SIZE + (index + 1) * (ENTRY_HEIGHT + OFFSET_SIZE);

            l.add_image_tiled(x, y, SET_ENTRY_WIDTH, ENTRY_HEIGHT, ENTRY_GUMP_ID);
            l.add_label_cropped(
                x + TEXT_OFFSET_X,
                y,
                SET_ENTRY_WIDTH - TEXT_OFFSET_X,
                ENTRY_HEIGHT,
                TEXT_HUE,
                s,
            );

            x += SET_ENTRY_WIDTH + OFFSET_SIZE;

            if SET_GUMP_ID != 0 {
                l.add_image_tiled(x, y, SET_WIDTH, ENTRY_HEIGHT, SET_GUMP_ID);
            }

            if button != 0 {
                l.add_button(
                    x + SET_OFFSET_X,
                    y + SET_OFFSET_Y,
                    SET_BUTTON_ID1,
                    SET_BUTTON_ID2,
                    button,
                );
            }
        };

    for page in 1..=pages {
        l.add_page(page);

        let start = (page as usize - 1) * LIST_ENTRY_COUNT;
        let count = (names.len() - start).min(LIST_ENTRY_COUNT);

        let (mut x, mut y) = add_props_frame(l, SET_TOTAL_WIDTH, count as i32 + 2);
        let mut header = Header::new(None, (page > 1).then_some(0), (page < pages).then_some(0));
        header.prev_nav = Nav::page(page - 1);
        header.next_nav = Nav::page(page + 1);
        add_props_header(l, SET_TOTAL_WIDTH, &mut x, &mut y, &header);

        add_rect(l, 0, prop_name, 0);

        for i in 0..count {
            let name = &names[index];
            index += 1;
            add_rect(l, i as i32 + 1, name, index as i32);
        }
    }
}

pub struct SetListOptionGump {
    s: Subject,
    values: Vec<PropValue>,
}

/// `new SetListOptionGump(prop, mobile, o, propertiesGump, names, values)`.
pub fn new_set_list_option(
    prop: PropInfo,
    mobile: MobileId,
    o: PropObj,
    props: PropertiesContext,
    names: Vec<String>,
    values: Vec<PropValue>,
) -> LegacyGumpObj<SetListOptionGump> {
    let name = prop.name.clone();
    let mut g = LegacyGumpObj::new(
        GUMP_OFFSET_X,
        GUMP_OFFSET_Y,
        SetListOptionGump {
            s: Subject {
                prop,
                mobile,
                object: o,
                props,
            },
            values,
        },
    );
    build_list_layout(&mut g.layout, &name, &names);
    g
}

/// The end of both list gumps' responses: the message `SetDirect` returned, then the properties gump.
fn finish_list_set(w: &mut World, s: &Subject, result: Result<String, ()>) {
    match result {
        Ok(message) => {
            muo_content::mx::send_message(w, s.mobile, &message);
            if message == "Property has been set." {
                s.props.on_value_changed(w, &s.object, &s.prop);
            }
        }
        Err(()) => muo_content::mx::send_message(w, s.mobile, EXCEPTION_MESSAGE),
    }
}

impl LegacyGumpLogic for SetListOptionGump {
    const TYPE_NAME: &'static str = "Server.Gumps.SetListOptionGump";

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let s = self.s.clone();
        let Some(from) = mobile_of(w, sender) else {
            return;
        };
        let Some(host) = props_host(w) else { return };

        if !host.is_accessible(w, from, &s.object) {
            muo_content::mx::send_message(w, from, "You may no longer access their properties.");
            return;
        }

        let index = info.button_id - 1;

        if index >= 0 && (index as usize) < self.values.len() {
            let result = host.set_direct(
                w,
                s.mobile,
                &s.object,
                &s.prop,
                &self.values[index as usize],
            );
            finish_list_set(w, &s, result);
        }

        s.props.send(w);
    }
}

pub struct SetCustomEnumGump {
    s: Subject,
    names: Vec<String>,
}

/// `new SetCustomEnumGump(prop, mobile, o, propertiesGump, names)`.
pub fn new_set_custom_enum(
    prop: PropInfo,
    mobile: MobileId,
    o: PropObj,
    props: PropertiesContext,
    names: Vec<String>,
) -> LegacyGumpObj<SetCustomEnumGump> {
    let name = prop.name.clone();
    let mut g = LegacyGumpObj::new(
        GUMP_OFFSET_X,
        GUMP_OFFSET_Y,
        SetCustomEnumGump {
            s: Subject {
                prop,
                mobile,
                object: o,
                props,
            },
            names: names.clone(),
        },
    );
    build_list_layout(&mut g.layout, &name, &names);
    g
}

impl LegacyGumpLogic for SetCustomEnumGump {
    const TYPE_NAME: &'static str = "Server.Gumps.SetCustomEnumGump";

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let s = self.s.clone();
        let Some(from) = mobile_of(w, sender) else {
            return;
        };
        let Some(host) = props_host(w) else { return };

        if !host.is_accessible(w, from, &s.object) {
            muo_content::mx::send_message(w, from, "You may no longer access their properties.");
            return;
        }

        let index = info.button_id - 1;

        if index >= 0 && (index as usize) < self.names.len() {
            let result =
                host.set_custom_enum(w, s.mobile, &s.object, &s.prop, &self.names[index as usize]);
            finish_list_set(w, &s, result);
        }

        s.props.send(w);
    }
}
