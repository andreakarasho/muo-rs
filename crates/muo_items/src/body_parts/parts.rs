//! `BonePile`, `RibCage` and `Head`. `BonePile` and `RibCage` are `IScissorable` (cut into
//! bones); `Head` keeps who it belonged to, what kind of kill it was, the bounty target and when it was carved.

use muo_content::items::interfaces::register_scissorable;
use muo_content::props::{Ctor, Kind, Param, Value, add_ctors};
use muo_content::utilities::types::PropType;
use muo_primitives::DateTime;
use muo_server::World;
use muo_server::ext::Ext;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, deps, ops};
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::type_registry::{Phase, TypeRegistry};

use crate::simple::{random_from, random_min_max};

fn scissor_bones(w: &mut World, id: ItemId, from: MobileId, min: i32, max: i32) -> bool {
    if ops::is_deleted(w, id) || !deps::can_see(w, from, id) {
        return false;
    }
    let per = random_min_max(w, min, max);
    if let Some(bone) = muo_content::factory::make_item(w, "Server.Items.Bone")
        && let Some(ty) = w.item(id).map(|i| i.ty)
    {
        (ty.scissor_helper)(w, id, from, bone, per, true);
    }
    true
}

macro_rules! bones {
    ($($name:ident = $full:literal, base $base:expr, count $count:expr, weight $weight:expr;)*) => {$(
        #[doc = concat!("`", $full, "`.")]
        pub static $name: ItemType = ItemType {
            full_name: $full,
            parent: Some(&ITEM_BASE),
            construct: Some(|w| {
                let g = $base + random_from(w, 0, $count);
                let id = ops::new_item_with_id(w, &$name, g);
                ops::set_stackable(w, id, false);
                id
            }),
            default_weight: |_, _| $weight,
            serialize: |w, id, out| {
                (ITEM_BASE.serialize)(w, id, out);
                out.write_i32(0);
            },
            deserialize: |w, id, r| {
                (ITEM_BASE.deserialize)(w, id, r)?;
                let _version = r.read_i32()?;
                Ok(())
            },
            ..ITEM_BASE
        };
    )*};
}

bones! {
    BONE_PILE = "Server.Items.BonePile", base 0x1B09, count 8, weight 10.0;
    RIB_CAGE = "Server.Items.RibCage", base 0x1B17, count 2, weight 5.0;
}

/// `HeadType`.
pub static HEAD_TYPES: &[(&str, i64)] = &[("Regular", 0), ("Duel", 1), ("Tournament", 2)];

/// What `Head` adds.
#[derive(Clone, Debug)]
pub struct HeadData {
    pub player_name: Option<String>,
    pub head_type: i32,
    pub bounty_target: Option<MobileId>,
    pub carved_time: DateTime,
}

fn data(w: &World, id: ItemId) -> Option<&HeadData> {
    w.item(id)?.ext.get::<HeadData>()
}

/// `new Head(headType, playerName)`.
pub fn new_head(w: &mut World, head_type: i32, player_name: Option<String>) -> ItemId {
    let id = ops::new_item_with_id(w, &HEAD, 0x1DA0);
    let now = w.now;
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(HeadData {
            player_name,
            head_type,
            bounty_target: None,
            carved_time: now,
        });
    }
    id
}

/// `Head.BountyTarget = target`.
pub fn set_bounty_target(w: &mut World, id: ItemId, target: Option<MobileId>) {
    if let Some(d) = w.item_mut(id).and_then(|i| i.ext.get_mut::<HeadData>()) {
        d.bounty_target = target;
    }
}

/// `Server.Items.Head`; `[SerializationGenerator(2, false)]`.
pub static HEAD: ItemType = ItemType {
    full_name: "Server.Items.Head",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| new_head(w, 0, None)),
    default_name: |w, id| {
        let d = data(w, id)?;
        let Some(name) = &d.player_name else {
            return (ITEM_BASE.default_name)(w, id);
        };
        Some(match d.head_type {
            1 => format!("the head of {name}, taken in a duel"),
            2 => format!("the head of {name}, taken in a tournament"),
            _ => format!("the head of {name}"),
        })
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(2);
        let (name, kind, target, carved) = data(w, id)
            .map(|d| {
                (
                    d.player_name.clone(),
                    d.head_type,
                    d.bounty_target,
                    d.carved_time,
                )
            })
            .unwrap_or((None, 0, None, DateTime::MIN));
        out.write_opt_string(name.as_deref());
        out.write_enum(4, i64::from(kind));
        out.write_mobile(w, target);
        out.write_date_time_ticks(carved.ticks());
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let version = r.read_i32()?;
        let d = if version >= 2 {
            let name = r.read_string()?;
            let kind = r.read_enum(4)? as i32;
            let target = r.read_mobile(w)?;
            let carved =
                DateTime::try_from_ticks(r.read_date_time_ticks()?).unwrap_or(DateTime::MIN);
            HeadData {
                player_name: name,
                head_type: kind,
                bounty_target: target,
                carved_time: carved,
            }
        } else if version == 1 {
            // V1Content: the name and the type as an enum (an int)
            let name = r.read_string()?;
            let kind = r.read_enum(4)? as i32;
            HeadData {
                player_name: name,
                head_type: kind,
                bounty_target: None,
                carved_time: DateTime::MIN,
            }
        } else {
            // `Deserialize(reader, version)`: pre-codegen data: the name and the type as an encoded int
            let name = r.read_string()?;
            let kind = r.read_encoded_int()?;
            HeadData {
                player_name: name,
                head_type: kind,
                bounty_target: None,
                carved_time: DateTime::MIN,
            }
        };
        if let Some(i) = w.item_mut(id) {
            i.ext = Ext::new(d);
        }
        Ok(())
    },
    ..ITEM_BASE
};

const KIND: Kind = Kind::Plain(PropType::Enum(HEAD_TYPES));
static HEAD_P1: [Param; 1] = [Param {
    name: "playerName",
    kind: Kind::Plain(PropType::Str),
    default: None,
    ty: "",
}];
static HEAD_P2: [Param; 2] = [
    Param {
        name: "headType",
        kind: KIND,
        default: Some(Value::Enum(0)),
        ty: "HeadType",
    },
    Param {
        name: "playerName",
        kind: Kind::Plain(PropType::Str),
        default: Some(Value::Null),
        ty: "",
    },
];
static C_HEAD: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &HEAD_P1,
        make: |w, v| EntityId::Item(new_head(w, 0, v[0].string())),
    },
    Ctor {
        access: None,
        params: &HEAD_P2,
        make: |w, v| EntityId::Item(new_head(w, v[0].i32(), v[1].string())),
    },
];

fn configure(w: &mut World) {
    register_scissorable(w, &BONE_PILE, |w, id, from, _| {
        scissor_bones(w, id, from, 10, 15)
    });
    register_scissorable(w, &RIB_CAGE, |w, id, from, _| {
        scissor_bones(w, id, from, 3, 5)
    });
    add_ctors(w, "Server.Items.Head", &C_HEAD);
}

/// Registers the classes.
pub fn register(types: &mut TypeRegistry) {
    for t in [&BONE_PILE, &RIB_CAGE, &HEAD] {
        types.register_item(t, &[]);
    }
    types.register_phase_default(
        Phase::Configure,
        "Server.Items.BonePile.Configure",
        configure,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use muo_io::{SaveReader, SaveWriter};

    #[test]
    fn head_names_follow_the_kind() {
        let mut w = World::new();
        let h = new_head(&mut w, 1, Some("Lord".to_string()));
        assert_eq!(
            (HEAD.default_name)(&w, h).as_deref(),
            Some("the head of Lord, taken in a duel")
        );
        let mut out = SaveWriter::new(true);
        (HEAD.serialize)(&w, h, &mut out);
        let bytes = out.into_vec();
        let h2 = w.add_item(muo_server::item::Item::new_for_load(
            muo_primitives::Serial::new(0x4000_0100),
            &HEAD,
        ));
        let mut r = SaveReader::new(&bytes);
        (HEAD.deserialize)(&mut w, h2, &mut r).unwrap();
        assert_eq!(r.remaining(), 0);
        assert_eq!(data(&w, h2).unwrap().head_type, 1);
    }

    #[test]
    fn bone_pile_graphic_is_in_range() {
        let mut w = World::new();
        for _ in 0..20 {
            let id = (BONE_PILE.construct.unwrap())(&mut w);
            assert!((0x1B09..0x1B09 + 8).contains(&w.item(id).unwrap().item_id));
        }
    }
}
