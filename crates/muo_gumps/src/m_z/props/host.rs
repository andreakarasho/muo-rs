//! What the props gumps ask of the property registry.
//!
//! There is no reflection: the registry that describes every class's `[CommandProperty]`s implements [`PropsHost`] and installs itself in
//! `GumpHooks::props`. The host answers the questions about properties (list, get, set, parse)
//! that the gumps ask.
//!
//! `props::configure` installs the registry host (`registry::RegistryHost`) in `GumpHooks::props`.

use std::any::Any;
use std::rc::Rc;

use muo_primitives::{Point2D, Point3D, Serial, TimeSpan};
use muo_server::World;
use muo_server::ids::{EntityId, MobileId};
use muo_server::mobile::AccessLevel;
use muo_server::target::Targeted;

/// An `object` a props gump shows: an item or mobile, or anything else the registry can describe
/// (a `[PropertyObject]` struct, say).
#[derive(Clone)]
pub enum PropObj {
    Entity(EntityId),
    Other(Rc<dyn Any>),
}

/// A type the gumps pass around: its name (for messages) and the registry's token for it.
#[derive(Clone)]
pub struct PropType {
    pub name: String,
    /// `type == typeof(System.Type)`.
    pub is_system_type: bool,
    pub token: Rc<dyn Any>,
}

/// A `PropertyInfo` that has a `[CommandProperty]`.
#[derive(Clone)]
pub struct PropInfo {
    pub name: String,
    /// `CPA.ReadOnly`.
    pub read_only: bool,
    /// `CPA.WriteLevel`.
    pub write_level: AccessLevel,
    /// `PropertyInfo.CanWrite`.
    pub can_write: bool,
    /// `CPA.CanModify`.
    pub cpa_can_modify: bool,
    /// `!PropertyType.IsValueType`: the property can be set to null.
    pub nullable: bool,
    /// `IsDefined(typeof(HueAttribute))`.
    pub hued: bool,
    /// The registry's token for the property.
    pub token: Rc<dyn Any>,
}

impl PropInfo {
    /// `cpa?.ReadOnly == false && access >= cpa.WriteLevel && (prop.CanWrite || cpa.CanModify)`.
    pub fn can_modify(&self, access: AccessLevel) -> bool {
        !self.read_only && access >= self.write_level && (self.can_write || self.cpa_can_modify)
    }
}

/// One row of `PropertiesGump.m_List`.
#[derive(Clone)]
pub enum ListEntry {
    /// `null`: a blank separator row.
    Blank,
    /// A `Type`: the name of the class the next properties belong to.
    Type(String),
    Prop(PropInfo),
}

/// A value to store in a property.
#[derive(Clone)]
pub enum PropValue {
    Null,
    Point3D(Point3D),
    Point2D(Point2D),
    TimeSpan(TimeSpan),
    Hue(i32),
    /// Text for the registry to parse for the property's type (`Types.TryParse`).
    Text(String),
    /// One of the values a [`PropKind::List`] offers.
    Opaque(Rc<dyn Any>),
}

/// Which gump `PropertiesGump` opens for a property, decided on the value's run-time type (or the declared type of
/// a null value), checks in a fixed order.
#[derive(Clone)]
pub enum PropKind {
    /// `Item`/`Mobile`: `SetObjectGump` with the type to assign.
    Entity(PropType),
    /// `System.Type`: a target that stores the targeted object's type.
    Type(PropType),
    Point3D,
    Point2D,
    TimeSpan,
    /// `[CustomEnum]`: the names; the registry parses the picked name.
    CustomEnum(Vec<String>),
    /// An enum, a bool, a `Poison` or a `Map`: the names and the values they set.
    List(Vec<String>, Vec<PropValue>),
    /// `Skills`: the skills gump of the mobile.
    Skills,
    /// `TextDefinition`, a string or anything `Parse`able.
    Text,
    /// `[PropertyObject]`: drill into the value.
    PropertyObject,
    Other,
}

/// The reflection answers.
#[allow(clippy::result_unit_err)] // `Err` is a failed call; it carries nothing
pub trait PropsHost {
    /// `BaseCommand.IsAccessible(from, obj)`.
    fn is_accessible(&self, w: &World, from: MobileId, obj: &PropObj) -> bool;

    /// `obj.GetType().Name`.
    fn type_name(&self, w: &World, obj: &PropObj) -> String;

    /// `(obj as ISerializable)?.Serial`.
    fn serial(&self, w: &World, obj: &PropObj) -> Option<Serial>;

    /// `PropertiesGump.BuildList()`: the groups of properties `viewer` may read, sorted.
    fn build_list(&self, w: &mut World, viewer: MobileId, obj: &PropObj) -> Vec<ListEntry>;

    /// `PropertiesGump.ValueToString(obj, prop)`.
    fn value_to_string(&self, w: &mut World, obj: &PropObj, prop: &PropInfo) -> String;

    /// The routing type of `prop` on `obj`.
    fn kind(&self, w: &mut World, obj: &PropObj, prop: &PropInfo) -> PropKind;

    /// `prop.GetValue(obj)` when it is an object that has properties; `None` for null.
    fn value_object(&self, w: &mut World, obj: &PropObj, prop: &PropInfo) -> Option<PropObj>;

    /// `obj as Mobile`.
    fn as_mobile(&self, obj: &PropObj) -> Option<MobileId>;

    /// `SetGump`'s initial text: "" for null, `TextDefinition.GetValue()`, else `ToString()`.
    fn text_value(&self, w: &mut World, obj: &PropObj, prop: &PropInfo) -> String;

    /// `prop.GetValue(obj)` as an `IPoint2D`/`IPoint3D`/`TimeSpan`; the default when it is not one.
    fn point2d_value(&self, w: &mut World, obj: &PropObj, prop: &PropInfo) -> Point2D;
    fn point3d_value(&self, w: &mut World, obj: &PropObj, prop: &PropInfo) -> Point3D;
    fn time_span_value(&self, w: &mut World, obj: &PropObj, prop: &PropInfo) -> TimeSpan;

    /// `Types.TryParse(prop.PropertyType, text, out value)`; `Err` where it threw.
    fn parse(&self, w: &World, prop: &PropInfo, text: &str) -> Result<PropValue, ()>;

    /// `prop.SetValue(obj, value)`; `Err` where it threw.
    fn set_value(
        &self,
        w: &mut World,
        obj: &PropObj,
        prop: &PropInfo,
        value: &PropValue,
    ) -> Result<(), ()>;

    /// `Properties.SetDirect(from, obj, obj, prop, prop.Name, value, true)`: the message it returns.
    fn set_direct(
        &self,
        w: &mut World,
        from: MobileId,
        obj: &PropObj,
        prop: &PropInfo,
        value: &PropValue,
    ) -> Result<String, ()>;

    /// `Properties.SetDirect` for a `[CustomEnum]` property with the picked name (`Parse` or `Enum.Parse`).
    fn set_custom_enum(
        &self,
        w: &mut World,
        from: MobileId,
        obj: &PropObj,
        prop: &PropInfo,
        name: &str,
    ) -> Result<String, ()>;

    /// `CommandLogging.LogChangeProperty(from, obj, propertyName, value.ToString())`; `Null` logs "(null)" and a
    /// `Text` is the value's `ToString()` already.
    fn log_change(
        &self,
        w: &mut World,
        from: MobileId,
        obj: &PropObj,
        property_name: &str,
        value: &PropValue,
    );

    /// `((IHued)obj).HuedItemID`.
    fn hued_item_id(&self, w: &World, obj: &PropObj) -> i32;

    /// `SetObjectTarget.OnTarget`: stores what `from` targeted if it can be assigned to a property of `ty`
    /// (an addon component stands for its addon; a `System.Type` property takes the targeted object's type).
    /// `Err` carries nothing: the registry did not accept it; `Ok(false)` is "cannot be assigned".
    fn assign_targeted(
        &self,
        w: &mut World,
        from: MobileId,
        obj: &PropObj,
        prop: &PropInfo,
        ty: &PropType,
        targeted: Targeted,
    ) -> Result<bool, ()>;

    /// `World.FindEntity(serial)`, assigned if its class is a `ty`: `Ok(None)` when no entity has the serial.
    /// `Ok(Some(false))` is a wrong class.
    fn assign_serial(
        &self,
        w: &mut World,
        from: MobileId,
        obj: &PropObj,
        prop: &PropInfo,
        ty: &PropType,
        serial: Serial,
    ) -> Result<Option<bool>, ()>;
}
