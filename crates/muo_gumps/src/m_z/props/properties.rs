//! `PropertiesGump` (and `StackEntry`).
//!
//! Every `Set*Gump` gets a [`PropertiesContext`] so it can call `SendPropertiesGump()` and `OnValueChanged`: the gump's state, which is cheap to clone.

use std::cell::RefCell;
use std::rc::Rc;

use muo_content::gumps::base::{LegacyGumpLogic, LegacyGumpObj, RelayInfo};
use muo_primitives::html::center_rgb;
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::net_state::NetStateId;

use super::host::{ListEntry, PropKind, PropObj, PropValue};
use super::setters;
use crate::m_z::common::{mobile_of, send};
use crate::m_z::hooks::props_host;
use muo_content::gumps::props_config::*;
use muo_content::gumps::props_layout::*;

const NAME_WIDTH: i32 = 107;
const VALUE_WIDTH: i32 = 128;

const MAX_ENTRIES_PER_PAGE: usize = 15;

const TYPE_WIDTH: i32 = NAME_WIDTH + OFFSET_SIZE + VALUE_WIDTH;

const TOTAL_WIDTH: i32 =
    OFFSET_SIZE + NAME_WIDTH + OFFSET_SIZE + VALUE_WIDTH + OFFSET_SIZE + SET_WIDTH + OFFSET_SIZE;

/// `PropertiesGump.BackWidth`.
pub const BACK_WIDTH: i32 = BORDER_SIZE + TOTAL_WIDTH + BORDER_SIZE;

/// `StackEntry`.
#[derive(Clone)]
pub struct StackEntry {
    pub object: PropObj,
    pub prop: super::host::PropInfo,
}

/// The `Stack<StackEntry>` every gump of one browsing session shares.
pub type PropsStack = Rc<RefCell<Vec<StackEntry>>>;

/// What a subclass of `PropertiesGump` overrides (`SpawnPropsGump`): `ShowAttribute`, the extra Apply button
/// (`Initialize`, `TotalHeight`) and its response. Only the gumps the subclass itself builds keep it: the base gump's
/// own Previous/Next/Closed and drill-in build plain `PropertiesGump`s.
pub trait PropsVariant {
    /// The gump's type name (its type id is what the client keys it by).
    fn type_name(&self) -> &'static str;
    /// `ShowAttribute(name)`.
    fn show_attribute(&self, name: &str) -> bool;
    /// `OnResponse` case 3 (Apply), after the access check.
    fn on_apply(&self, w: &mut World, from: MobileId, object: &PropObj);
}

/// What a `PropertiesGump` is: `m_Mobile`, `m_Object`, `m_Stack`, `m_List`, `m_Page`.
#[derive(Clone)]
pub struct PropertiesContext {
    pub mobile: MobileId,
    /// `m_Object`; `None` is null (`m_Type` is null too).
    pub object: Option<PropObj>,
    pub stack: Option<PropsStack>,
    pub list: Rc<Vec<ListEntry>>,
    pub page: usize,
    /// The subclass this is, if any.
    pub variant: Option<Rc<dyn PropsVariant>>,
}

impl PropertiesContext {
    /// `SendPropertiesGump()`.
    pub fn send(&self, w: &mut World) {
        let g = new_page(
            w,
            self.mobile,
            self.object.clone(),
            self.stack.clone(),
            self.list.clone(),
            self.page,
            self.variant.clone(),
        );
        send(w, self.mobile, g);
    }

    /// `OnValueChanged(obj, prop)`: a value-type property changed in place, so its owner's property is set again.
    pub fn on_value_changed(&self, w: &mut World, obj: &PropObj, prop: &super::host::PropInfo) {
        let Some(stack) = &self.stack else { return };
        if stack.borrow().is_empty() {
            return;
        }

        if prop.nullable {
            return; // !prop.PropertyType.IsValueType
        }

        let Some(peek) = stack.borrow().last().cloned() else {
            return;
        };

        if peek.prop.can_write
            && let Some(host) = props_host(w)
        {
            let value = match obj {
                PropObj::Other(o) => PropValue::Opaque(o.clone()),
                PropObj::Entity(_) => return,
            };
            let _ = host.set_value(w, &peek.object, &peek.prop, &value);
        }
    }
}

pub struct PropertiesGump {
    pub ctx: PropertiesContext,
}

fn new_gump(ctx: PropertiesContext) -> LegacyGumpObj<PropertiesGump> {
    let mut g = LegacyGumpObj::new(GUMP_OFFSET_X, GUMP_OFFSET_Y, PropertiesGump { ctx });
    if let Some(v) = &g.logic.ctx.variant {
        g.core.type_id = muo_content::gumps::base::get_type_id(v.type_name());
    }
    g
}

/// `new PropertiesGump(mobile, o)`.
pub fn new(w: &mut World, mobile: MobileId, o: PropObj) -> LegacyGumpObj<PropertiesGump> {
    new_with_parent(w, mobile, o, None, None)
}

/// `new PropertiesGump(mobile, o, stack, parent)`.
pub fn new_with_parent(
    w: &mut World,
    mobile: MobileId,
    o: PropObj,
    stack: Option<PropsStack>,
    parent: Option<StackEntry>,
) -> LegacyGumpObj<PropertiesGump> {
    new_full(w, mobile, o, stack, parent, None)
}

/// `new SpawnPropsGump(mobile, o, ..)`-style: the first constructor of a subclass.
pub fn new_variant(
    w: &mut World,
    mobile: MobileId,
    o: PropObj,
    variant: Rc<dyn PropsVariant>,
) -> LegacyGumpObj<PropertiesGump> {
    new_full(w, mobile, o, None, None, Some(variant))
}

fn new_full(
    w: &mut World,
    mobile: MobileId,
    o: PropObj,
    stack: Option<PropsStack>,
    parent: Option<StackEntry>,
    variant: Option<Rc<dyn PropsVariant>>,
) -> LegacyGumpObj<PropertiesGump> {
    let mut list = props_host(w).map_or_else(Vec::new, |h| h.build_list(w, mobile, &o));
    if let Some(v) = &variant {
        // `BuildList`: `if (ShowAttribute(item.Name))` filters the properties, never the type rows
        list.retain(|e| !matches!(e, ListEntry::Prop(p) if !v.show_attribute(&p.name)));
    }

    let mut stack = stack;
    if let Some(parent) = parent {
        stack
            .get_or_insert_with(|| Rc::new(RefCell::new(Vec::new())))
            .borrow_mut()
            .push(parent);
    }

    let mut g = new_gump(PropertiesContext {
        mobile,
        object: Some(o),
        stack,
        list: Rc::new(list),
        page: 0,
        variant,
    });
    initialize(w, &mut g, 0);
    g
}

/// `new PropertiesGump(mobile, o, stack, list, page)`.
pub fn new_page(
    w: &mut World,
    mobile: MobileId,
    o: Option<PropObj>,
    stack: Option<PropsStack>,
    list: Rc<Vec<ListEntry>>,
    page: usize,
    variant: Option<Rc<dyn PropsVariant>>,
) -> LegacyGumpObj<PropertiesGump> {
    let mut g = new_gump(PropertiesContext {
        mobile,
        object: o,
        stack,
        list,
        page,
        variant,
    });
    initialize(w, &mut g, page);
    g
}

/// `Initialize(page)`.
fn initialize(w: &mut World, g: &mut LegacyGumpObj<PropertiesGump>, page: usize) {
    let ctx = g.logic.ctx.clone();
    let host = props_host(w);
    let host = host.as_deref();

    let index_on_page = page * MAX_ENTRIES_PER_PAGE;

    // i32: a page past the end whose previous row is blank counts -1 rows
    let mut entry_count = ctx
        .list
        .len()
        .saturating_sub(index_on_page)
        .min(MAX_ENTRIES_PER_PAGE) as i32;
    let last_index = (index_on_page as i64) + i64::from(entry_count) - 1;
    if last_index >= 0
        && (last_index as usize) < ctx.list.len()
        && matches!(ctx.list[last_index as usize], ListEntry::Blank)
    {
        entry_count -= 1;
    }

    let access = muo_content::mx::access_level(w, ctx.mobile);
    let l = &mut g.layout;
    l.add_page(0);

    let (mut x, mut y) = add_props_frame(l, TOTAL_WIDTH, entry_count + 1);

    // m_Type != null && m_Object is ISerializable ? "{Name} ({Serial})" : m_Type?.Name, centered
    let title = match (&ctx.object, host) {
        (Some(o), Some(h)) => {
            let name = h.type_name(w, o);
            Some(match h.serial(w, o) {
                Some(serial) => center_rgb(&format!("{name} ({serial})"), 0xFAFAFA),
                None => center_rgb(&name, 0xFAFAFA),
            })
        }
        _ => None,
    };

    let mut header = Header::new(
        title.as_deref(),
        (page > 0).then_some(1),
        ((page + 1) * MAX_ENTRIES_PER_PAGE < ctx.list.len()).then_some(2),
    );
    header.next_nav = Nav {
        kind: muo_content::gumps::GumpButtonType::Reply,
        param: 1,
    };
    add_props_header(l, TOTAL_WIDTH, &mut x, &mut y, &header);

    let mut index = page * MAX_ENTRIES_PER_PAGE;
    let mut i = 0;
    while i < entry_count && index < ctx.list.len() {
        next_row(&mut x, &mut y);

        match &ctx.list[index] {
            ListEntry::Blank => add_props_entry_blank(l, x, y, TOTAL_WIDTH),
            ListEntry::Type(name) => add_props_entry_type(l, &mut x, &mut y, TYPE_WIDTH, name),
            ListEntry::Prop(prop) => {
                let can_modify = prop.can_modify(access);
                let value = match (&ctx.object, host) {
                    (Some(o), Some(h)) => h.value_to_string(w, o, prop),
                    _ => String::new(),
                };
                add_props_entry_name_value(
                    l,
                    &mut x,
                    &mut y,
                    NAME_WIDTH,
                    VALUE_WIDTH,
                    &prop.name,
                    &value,
                    can_modify.then_some(i + 3),
                );
            }
        }

        i += 1;
        index += 1;
    }

    if ctx.variant.is_some() {
        // SpawnPropsGump.Initialize: the Apply button under the (base) total height
        let total_height = OFFSET_SIZE + (ENTRY_HEIGHT + OFFSET_SIZE) * (entry_count + 1);
        l.add_button(
            BACK_WIDTH / 3,
            BORDER_SIZE * 2 + total_height,
            5204,
            5205,
            3,
        );
    }
}

impl LegacyGumpLogic for PropertiesGump {
    const TYPE_NAME: &'static str = "Server.Gumps.PropertiesGump";

    fn gump_type_name(&self) -> &'static str {
        self.ctx
            .variant
            .as_ref()
            .map_or(Self::TYPE_NAME, |v| v.type_name())
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let Some(from) = mobile_of(w, sender) else {
            return;
        };
        let ctx = self.ctx.clone();
        let host = props_host(w);

        // BaseCommand.IsAccessible(from, m_Object)
        let accessible = match (&host, &ctx.object) {
            (Some(h), Some(o)) => h.is_accessible(w, from, o),
            _ => false,
        };
        if !accessible {
            muo_content::mx::send_message(w, from, "You may no longer access their properties.");
            return;
        }
        let (Some(host), Some(object)) = (host, ctx.object.clone()) else {
            return;
        };

        if info.button_id == 3
            && let Some(v) = &ctx.variant
        {
            v.on_apply(w, from, &object);
            return;
        }

        match info.button_id {
            0 => {
                // Closed
                let popped = ctx.stack.as_ref().and_then(|s| s.borrow_mut().pop());
                if let Some(entry) = popped {
                    let g = new_with_parent(w, from, entry.object, ctx.stack.clone(), None);
                    send(w, from, g);
                }
            }
            1 => {
                // Previous
                if ctx.page > 0 {
                    let g = new_page(
                        w,
                        from,
                        ctx.object.clone(),
                        ctx.stack.clone(),
                        ctx.list.clone(),
                        ctx.page - 1,
                        None,
                    );
                    send(w, from, g);
                }
            }
            2 => {
                // Next
                if (ctx.page + 1) * MAX_ENTRIES_PER_PAGE < ctx.list.len() {
                    let g = new_page(
                        w,
                        from,
                        ctx.object.clone(),
                        ctx.stack.clone(),
                        ctx.list.clone(),
                        ctx.page + 1,
                        None,
                    );
                    send(w, from, g);
                }
            }
            button => {
                let index = ctx.page as i64 * MAX_ENTRIES_PER_PAGE as i64 + i64::from(button - 3);

                if index < 0 || index >= ctx.list.len() as i64 {
                    return;
                }

                let ListEntry::Prop(prop) = ctx.list[index as usize].clone() else {
                    return;
                };

                let access = muo_content::mx::access_level(w, from);
                if !prop.can_modify(access) {
                    return;
                }

                match host.kind(w, &object, &prop) {
                    PropKind::Entity(ty) => {
                        let g = setters::new_set_object(w, prop, from, object, ty, ctx);
                        send(w, from, g);
                    }
                    PropKind::Type(ty) => {
                        setters::begin_set_object_target(w, prop, from, object, ty, ctx);
                    }
                    PropKind::Point3D => {
                        let g = setters::new_set_point3d(w, prop, from, object, ctx);
                        send(w, from, g);
                    }
                    PropKind::Point2D => {
                        let g = setters::new_set_point2d(w, prop, from, object, ctx);
                        send(w, from, g);
                    }
                    PropKind::TimeSpan => {
                        let g = setters::new_set_time_span(w, prop, from, object, ctx);
                        send(w, from, g);
                    }
                    PropKind::CustomEnum(names) => {
                        let g = setters::new_set_custom_enum(prop, from, object, ctx, names);
                        send(w, from, g);
                    }
                    PropKind::List(names, values) => {
                        let g =
                            setters::new_set_list_option(prop, from, object, ctx, names, values);
                        send(w, from, g);
                    }
                    PropKind::Skills => {
                        if let Some(mobile) = host.as_mobile(&object) {
                            let g = new_page(
                                w,
                                from,
                                Some(PropObj::Entity(muo_server::ids::EntityId::Mobile(mobile))),
                                ctx.stack.clone(),
                                ctx.list.clone(),
                                ctx.page,
                                None,
                            );
                            send(w, from, g);
                            let g = crate::m_z::skills::new_skills(w, from, mobile, None);
                            send(w, from, g);
                        }
                    }
                    // Must stay ahead of [PropertyObject]: TextDefinition carries that attribute, but Number and
                    // String are get-only so drilling in is a dead end.
                    PropKind::Text => {
                        let g = setters::new_set(w, prop, from, object, ctx);
                        send(w, from, g);
                    }
                    PropKind::PropertyObject => match host.value_object(w, &object, &prop) {
                        Some(obj) => {
                            let g = new_with_parent(
                                w,
                                from,
                                obj,
                                ctx.stack.clone(),
                                Some(StackEntry { object, prop }),
                            );
                            send(w, from, g);
                        }
                        None => {
                            let g = new_page(
                                w,
                                from,
                                ctx.object.clone(),
                                ctx.stack.clone(),
                                ctx.list.clone(),
                                ctx.page,
                                None,
                            );
                            send(w, from, g);
                        }
                    },
                    PropKind::Other => {}
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::any::Any;

    use muo_primitives::{Point2D, Point3D, Serial, TimeSpan};
    use muo_server::ids::EntityId;
    use muo_server::mobile::AccessLevel;
    use muo_server::target::Targeted;

    use super::*;
    use crate::m_z::hooks::hooks_mut;
    use crate::m_z::props::host::{PropInfo, PropType, PropsHost};
    use crate::m_z::tests::layout_of;

    /// Two groups of two properties, the last one read only.
    struct Mock;

    thread_local! {
        /// The tick counts the mock was asked to set.
        static SET_TICKS: std::cell::RefCell<Vec<i64>> = const { std::cell::RefCell::new(Vec::new()) };
    }

    fn prop(name: &str, can_write: bool) -> PropInfo {
        PropInfo {
            name: name.to_owned(),
            read_only: !can_write,
            write_level: AccessLevel::Player,
            can_write,
            cpa_can_modify: false,
            nullable: false,
            hued: false,
            token: Rc::new(()) as Rc<dyn Any>,
        }
    }

    impl PropsHost for Mock {
        fn is_accessible(&self, _: &World, _: MobileId, _: &PropObj) -> bool {
            true
        }
        fn type_name(&self, _: &World, _: &PropObj) -> String {
            "Mock".into()
        }
        fn serial(&self, _: &World, _: &PropObj) -> Option<Serial> {
            Some(Serial::new(0x4000_0001))
        }
        fn build_list(&self, _: &mut World, _: MobileId, _: &PropObj) -> Vec<ListEntry> {
            vec![
                ListEntry::Type("Mock".into()),
                ListEntry::Prop(prop("Amount", true)),
                ListEntry::Prop(prop("Name", true)),
                ListEntry::Blank,
                ListEntry::Type("Base".into()),
                ListEntry::Prop(prop("Serial", false)),
            ]
        }
        fn value_to_string(&self, _: &mut World, _: &PropObj, p: &PropInfo) -> String {
            format!("v-{}", p.name)
        }
        fn kind(&self, _: &mut World, _: &PropObj, _: &PropInfo) -> PropKind {
            PropKind::Text
        }
        fn value_object(&self, _: &mut World, _: &PropObj, _: &PropInfo) -> Option<PropObj> {
            None
        }
        fn as_mobile(&self, _: &PropObj) -> Option<MobileId> {
            None
        }
        fn text_value(&self, _: &mut World, _: &PropObj, _: &PropInfo) -> String {
            String::new()
        }
        fn point2d_value(&self, _: &mut World, _: &PropObj, _: &PropInfo) -> Point2D {
            Point2D::ZERO
        }
        fn point3d_value(&self, _: &mut World, _: &PropObj, _: &PropInfo) -> Point3D {
            Point3D::ZERO
        }
        fn time_span_value(&self, _: &mut World, _: &PropObj, _: &PropInfo) -> TimeSpan {
            TimeSpan::ZERO
        }
        fn parse(&self, _: &World, _: &PropInfo, text: &str) -> Result<PropValue, ()> {
            Ok(PropValue::Text(text.to_owned()))
        }
        fn set_value(
            &self,
            _: &mut World,
            _: &PropObj,
            _: &PropInfo,
            v: &PropValue,
        ) -> Result<(), ()> {
            if let PropValue::TimeSpan(t) = v {
                SET_TICKS.with(|s| s.borrow_mut().push(t.ticks()));
            }
            Ok(())
        }
        fn set_direct(
            &self,
            _: &mut World,
            _: MobileId,
            _: &PropObj,
            _: &PropInfo,
            _: &PropValue,
        ) -> Result<String, ()> {
            Ok("Property has been set.".into())
        }
        fn set_custom_enum(
            &self,
            _: &mut World,
            _: MobileId,
            _: &PropObj,
            _: &PropInfo,
            _: &str,
        ) -> Result<String, ()> {
            Ok(String::new())
        }
        fn log_change(&self, _: &mut World, _: MobileId, _: &PropObj, _: &str, _: &PropValue) {}
        fn hued_item_id(&self, _: &World, _: &PropObj) -> i32 {
            0
        }
        fn assign_targeted(
            &self,
            _: &mut World,
            _: MobileId,
            _: &PropObj,
            _: &PropInfo,
            _: &PropType,
            _: Targeted,
        ) -> Result<bool, ()> {
            Ok(true)
        }
        fn assign_serial(
            &self,
            _: &mut World,
            _: MobileId,
            _: &PropObj,
            _: &PropInfo,
            _: &PropType,
            _: Serial,
        ) -> Result<Option<bool>, ()> {
            Ok(None)
        }
    }

    #[test]
    fn rows_are_numbered_from_three_and_read_only_rows_have_no_button() {
        let mut w = World::new();
        hooks_mut(&mut w).props = Some(Rc::new(Mock));
        let viewer = MobileId(Serial::new(1));
        let obj = PropObj::Entity(EntityId::Mobile(viewer));
        let mut g = new(&mut w, viewer, obj);
        let l = layout_of(&mut g);

        // the title row: "Mock (0x40000001)" centered in 0xFAFAFA
        assert!(l.starts_with("{ page 0 }{ resizepic 0 0 5054 "), "{l}");
        // 107 + 128 wide name/value cells; the set arrow of the first modifiable row (list index 1 -> button 3 + i)
        let buttons = l.matches("{ button ").count();
        // Amount and Name are modifiable (i = 1, 2 on the page -> buttons 4, 5); Serial is read only; no prev/next
        assert_eq!(buttons, 2, "{l}");
        assert!(l.contains(" 1 0 4 }") && l.contains(" 1 0 5 }"), "{l}");
        assert!(!l.contains(" 1 0 3 }"), "the type row has no button: {l}");
        // the blank separator row is BackGumpID + 4
        assert!(l.contains("{ gumppictiled 10 95 259 20 5058 }"), "{l}");
    }

    /// `SetTimeSpanGump.OnResponse`: `TimeSpan.FromHours(huge or NaN)` throws and is caught, so nothing is set.
    #[test]
    fn a_time_span_that_overflows_is_not_set() {
        use muo_content::gumps::base::Gump;
        let mut w = World::new();
        hooks_mut(&mut w).props = Some(Rc::new(Mock));
        let viewer = MobileId(Serial::new(1));
        let obj = PropObj::Entity(EntityId::Mobile(viewer));
        let ctx = new(&mut w, viewer, obj.clone()).logic.ctx.clone();
        let mut g =
            super::super::setters::new_set_time_span(&mut w, prop("Delay", true), viewer, obj, ctx);
        let ns = NetStateId(1);
        let press = |w: &mut World, g: &mut dyn Gump, button: i32, hours: &str| {
            let info = RelayInfo::new(
                button,
                vec![],
                vec![(0, hours.into()), (1, "0".into()), (2, "0".into())],
            );
            g.on_response(w, ns, &info);
        };
        SET_TICKS.with(|s| s.borrow_mut().clear());
        press(&mut w, &mut g, 3, "1e300");
        press(&mut w, &mut g, 3, "NaN");
        assert!(SET_TICKS.with(|s| s.borrow().is_empty()));
        press(&mut w, &mut g, 3, "2");
        assert_eq!(
            SET_TICKS.with(|s| s.borrow().clone()),
            vec![2 * 36_000_000_000]
        );
    }

    #[test]
    fn a_context_is_resent_on_the_same_page() {
        let mut w = World::new();
        hooks_mut(&mut w).props = Some(Rc::new(Mock));
        let viewer = MobileId(Serial::new(1));
        let obj = PropObj::Entity(EntityId::Mobile(viewer));
        let g = new(&mut w, viewer, obj);
        assert_eq!(g.logic.ctx.page, 0);
        assert_eq!(g.logic.ctx.list.len(), 6);
    }
}
