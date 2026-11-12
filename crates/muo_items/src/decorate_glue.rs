//! The item classes of this crate for `[Decorate` (`Construct()` and
//! `FindItem`/`Generate`): the constructors the overload tables cannot call, the per-class parameter blocks, and the
//! door pairing. `muo_commands::decorate` owns the file reading and placement; these are its `decorate_*` hooks.

use muo_commands::hooks::{CommandHooks, enum_parse, parse_error};
use muo_content::props::{self, item_ty};
use muo_io::TextDefinition;
use muo_primitives::parse::{to_bool, to_f64, to_i32};
use muo_primitives::{Point2D, Point3D, Rectangle2D, TimeSpan};
use muo_server::World;
use muo_server::ids::ItemId;
use muo_server::item::ops;
use muo_server::map::enumerate;
use muo_server::type_registry::{Phase, TypeRegistry};

use crate::containers::fillable;
use crate::lights::base as light;
use crate::misc::{gadgets, teleporter};
use crate::skill_items::runebook;

/// The text after `=` of a `Key=value` parameter.
fn after_eq(p: &str) -> Option<&str> {
    p.find('=').map(|i| &p[i + 1..])
}

/// `StartsWithOrdinal(key)` + `IndexOfOrdinal('=')`: the value of a parameter, `None` when it has no `=`.
fn val<'a>(p: &'a str, key: &str) -> Option<&'a str> {
    if p.starts_with(key) {
        after_eq(p)
    } else {
        None
    }
}

fn text_of(s: Option<&str>) -> TextDefinition {
    // `TextDefinition.Of(string)`: a null string is `new TextDefinition(0, null)`, an empty definition.
    TextDefinition {
        number: 0,
        string: s.map(str::to_string),
    }
}

/// `TimeSpan.Parse(s)`: throws on bad text.
fn timespan(s: &str) -> Result<TimeSpan, String> {
    TimeSpan::try_parse(s).ok_or_else(|| parse_error(s))
}

/// `Map.Parse(s)`: throws when no map matches.
fn map_parse(w: &World, s: &str) -> Result<muo_server::map::MapId, String> {
    w.maps.parse(s).ok_or_else(|| parse_error(s))
}

/// `Point3D.Parse(s)`: throws on bad text.
fn point3d(s: &str) -> Result<Point3D, String> {
    Point3D::try_parse(s).ok_or_else(|| parse_error(s))
}

/// `Point2D.Parse(s)`: throws on bad text.
fn point2d(s: &str) -> Result<Point2D, String> {
    Point2D::try_parse(s).ok_or_else(|| parse_error(s))
}

/// `new HintItem/WarningItem/SerpentPillar/...(..)` for the classes with no table constructor.
fn construct(
    w: &mut World,
    full: &str,
    item_id: i32,
    ps: &[String],
) -> Option<Result<ItemId, String>> {
    construct_checked(w, full, item_id, ps).transpose()
}

fn construct_checked(
    w: &mut World,
    full: &str,
    item_id: i32,
    ps: &[String],
) -> Result<Option<ItemId>, String> {
    Ok(match full {
        "Server.Items.HintItem" | "Server.Items.WarningItem" => {
            let hint = full == "Server.Items.HintItem";
            let (
                mut range,
                mut message_number,
                mut message_string,
                mut hint_number,
                mut hint_string,
            ) = (0, 0, None, 0, None);
            let mut reset_delay = TimeSpan::ZERO;
            for p in ps {
                if let Some(v) = val(p, "Range") {
                    range = to_i32(v);
                } else if let Some(v) = val(p, "WarningString") {
                    message_string = Some(v.to_string());
                } else if let Some(v) = val(p, "WarningNumber") {
                    message_number = to_i32(v);
                } else if let Some(v) = val(p, "HintString").filter(|_| hint) {
                    hint_string = Some(v.to_string());
                } else if let Some(v) = val(p, "HintNumber").filter(|_| hint) {
                    hint_number = to_i32(v);
                } else if let Some(v) = val(p, "ResetDelay") {
                    reset_delay = timespan(v)?;
                }
            }
            let number = TextDefinition {
                number: message_number,
                string: None,
            };
            let id = if hint {
                gadgets::hint_item_new(
                    w,
                    item_id,
                    range,
                    number,
                    TextDefinition {
                        number: hint_number,
                        string: None,
                    },
                )
            } else {
                let ty = item_ty(w, "Server.Items.WarningItem");
                gadgets::warning_item_new(w, ty, item_id, range, number)
            };
            // `wi.WarningMessage = messageString;`: assigns even a null string, which replaces the number with an
            // empty definition (the data files' `WarningNumber` messages are not used).
            gadgets::set_warning_message(w, id, text_of(message_string.as_deref()));
            if hint {
                gadgets::set_hint_message(w, id, text_of(hint_string.as_deref()));
            }
            gadgets::set_reset_delay(w, id, reset_delay);
            Some(id)
        }
        "Server.Items.SerpentPillar" => {
            let (mut word, mut dest) = (None, Rectangle2D::default());
            for p in ps {
                if let Some(v) = val(p, "Word") {
                    word = Some(v.to_string());
                } else if let Some(v) = val(p, "DestStart") {
                    dest.start = point2d(v)?;
                } else if let Some(v) = val(p, "DestEnd") {
                    dest.end = point2d(v)?;
                }
            }
            let ty = item_ty(w, "Server.Items.SerpentPillar");
            // `new SerpentPillar(word, destination)`: `active` defaults to true.
            Some(gadgets::serpent_pillar_new(w, ty, word, dest, true))
        }
        "Server.Items.MarkContainer" => {
            let (mut bone, mut locked, mut map) =
                (false, false, Some(muo_server::map::MapId::MALAS));
            for p in ps {
                if p == "Bone" {
                    bone = true;
                } else if p == "Locked" {
                    locked = true;
                } else if let Some(v) = val(p, "TargetMap") {
                    map = Some(map_parse(w, v)?);
                }
            }
            Some(crate::last::mark::decorate_new(w, bone, locked, map))
        }
        // `Campfire()` is public but not `[Constructible]`: Decorate's `CreateInstance` still finds it.
        "Server.Items.Campfire" => Some(crate::skill_items::camping::new_campfire(w)),
        _ => None,
    })
}

fn is(w: &World, id: ItemId, class: &str) -> bool {
    props::is_class(w, muo_server::ids::EntityId::Item(id), class)
}

/// The `item is X` chain of `Construct()` for this crate's classes.
fn apply(
    w: &mut World,
    item: ItemId,
    item_id: i32,
    ps: &[String],
    mag: bool,
) -> Result<bool, String> {
    if is(w, item, "Server.Items.BaseAddon") {
        // Components with a zero offset take the item id (a MaabusCoffin is the engines' block).
        if !is(w, item, "Server.Engines.Quests.Necro.MaabusCoffin") && item_id > 0 {
            for c in muo_content::items::addon::components(w, item) {
                if muo_content::items::addon::component_offset(w, c) == Point3D::ZERO {
                    muo_server::item::ops::set_item_id(w, c, item_id);
                }
            }
        }
        return Ok(!is(w, item, "Server.Engines.Quests.Necro.MaabusCoffin"));
    }
    if light::is_light(w, item) {
        let unlit = ps.iter().any(|p| p == "Unlit");
        let unprotected = ps.iter().any(|p| p == "Unprotected");
        // Make light never run out of fuel.
        light::set_duration(w, item, TimeSpan::ZERO);
        if !unlit {
            light::ignite(w, item);
        }
        if !unprotected {
            light::set_protected(w, item, true);
        }
        if item_id > 0 {
            muo_server::item::ops::set_item_id(w, item, item_id);
        }
        return Ok(true);
    }
    if is(w, item, "Server.Items.RecallRune") {
        for p in ps {
            if let Some(v) = val(p, "Description") {
                runebook::set_rune_description(w, item, Some(v.to_string()));
            } else if let Some(v) = val(p, "Marked") {
                runebook::set_marked(w, item, to_bool(v));
            } else if let Some(v) = val(p, "TargetMap") {
                runebook::set_target_map(w, item, Some(map_parse(w, v)?));
            } else if let Some(v) = val(p, "Target") {
                runebook::set_target(w, item, point3d(v)?);
            }
        }
        return Ok(true);
    }
    if teleporter::data(w, item).is_some() {
        return apply_teleporter(w, item, item_id, ps, mag);
    }
    if is(w, item, "Server.Items.FillableContainer") {
        for p in ps {
            if let Some(v) = val(p, "ContentType") {
                // `(FillableContentType)Enum.Parse(.., true)`
                let at = enum_parse(fillable::content_type_names(), v)?;
                fillable::set_content_type(w, item, at);
            }
        }
        if item_id > 0 {
            muo_server::item::ops::set_item_id(w, item, item_id);
        }
        return Ok(true);
    }
    Ok(false)
}

/// `SkillTeleporter`, `KeywordTeleporter`, `InteractionTeleporter` and `Teleporter` blocks, which share their
/// `PointDest`, `MapDest`, effect, sound and delay parameters.
fn apply_teleporter(
    w: &mut World,
    item: ItemId,
    item_id: i32,
    ps: &[String],
    mag: bool,
) -> Result<bool, String> {
    use teleporter::flags::{CREATURES, DEST_EFFECT, SOURCE_EFFECT};
    let skill = is(w, item, "Server.Items.SkillTeleporter");
    let keyword = is(w, item, "Server.Items.KeywordTeleporter");
    let interaction = !mag && is(w, item, "Server.Items.InteractionTeleporter");
    if interaction {
        muo_server::item::ops::set_item_id(w, item, item_id);
    }
    for p in ps {
        // The `else if` order matters: `RequiredFixedPoint` before `Required`, `MapDest` before `Name`...
        if skill && let Some(v) = val(p, "Skill") {
            // `Enum.Parse(typeof(SkillName), v, true)`
            let names: Vec<String> = (0..muo_server::skills::SkillName::COUNT as i32)
                .map(|i| {
                    muo_server::skills::SkillName::from_i32(i)
                        .map_or_else(String::new, |s| format!("{s:?}"))
                })
                .collect();
            let names: Vec<&str> = names.iter().map(String::as_str).collect();
            let i = enum_parse(&names, v)?;
            if let Some(d) = teleporter::data_mut(w, item) {
                d.skill = i;
            }
        } else if skill && let Some(v) = val(p, "RequiredFixedPoint") {
            if let Some(d) = teleporter::data_mut(w, item) {
                d.required = f64::from(to_i32(v)) * 0.1;
            }
        } else if skill && let Some(v) = val(p, "Required") {
            if let Some(d) = teleporter::data_mut(w, item) {
                d.required = to_f64(v);
            }
        } else if skill && let Some(v) = val(p, "MessageString") {
            if let Some(d) = teleporter::data_mut(w, item) {
                d.message = text_of(Some(v));
            }
        } else if skill && let Some(v) = val(p, "MessageNumber") {
            if let Some(d) = teleporter::data_mut(w, item) {
                d.message = TextDefinition {
                    number: to_i32(v),
                    string: None,
                };
            }
        } else if keyword && let Some(v) = val(p, "Substring") {
            if let Some(d) = teleporter::data_mut(w, item) {
                d.substring = Some(v.to_string());
            }
        } else if keyword && let Some(v) = val(p, "Keyword") {
            if let Some(d) = teleporter::data_mut(w, item) {
                d.keyword = to_i32(v);
            }
        } else if keyword && let Some(v) = val(p, "Range") {
            if let Some(d) = teleporter::data_mut(w, item) {
                d.range = to_i32(v);
            }
        } else if let Some(v) = val(p, "PointDest") {
            let t = point3d(v)?;
            if let Some(d) = teleporter::data_mut(w, item) {
                d.point_dest = t;
            }
        } else if let Some(v) = val(p, "MapDest") {
            let m = Some(map_parse(w, v)?);
            if let Some(d) = teleporter::data_mut(w, item) {
                d.map_dest = m;
            }
        } else if !interaction && let Some(v) = val(p, "Creatures") {
            teleporter::set_flag(w, item, CREATURES, to_bool(v));
        } else if let Some(v) = val(p, "SourceEffect") {
            teleporter::set_flag(w, item, SOURCE_EFFECT, to_bool(v));
        } else if let Some(v) = val(p, "DestEffect") {
            teleporter::set_flag(w, item, DEST_EFFECT, to_bool(v));
        } else if let Some(v) = val(p, "SoundID") {
            if let Some(d) = teleporter::data_mut(w, item) {
                d.sound_id = to_i32(v);
            }
        } else if let Some(v) = val(p, "Delay") {
            if let Some(d) = teleporter::data_mut(w, item) {
                d.delay = timespan(v)?;
            }
        } else if interaction && let Some(v) = val(p, "Name") {
            ops::set_name(w, item, Some(v.to_string()));
        }
    }
    if !interaction && item_id > 0 {
        muo_server::item::ops::set_item_id(w, item, item_id);
    }
    Ok(true)
}

/// `FindItem` for a door: its closed location and graphic.
fn door_probe(w: &World, id: ItemId) -> Option<(Point3D, i32)> {
    use crate::construction::doors;
    if !doors::is_door(w, id) {
        return None;
    }
    let i = w.item(id)?;
    Some(if doors::is_open(w, id) {
        let o = doors::offset(w, id);
        (
            Point3D::new(i.location.x - o.x, i.location.y - o.y, i.location.z - o.z),
            doors::closed_id_of(w, id),
        )
    } else {
        (i.location, i.item_id)
    })
}

/// After `MoveToWorld`: a door pairs with a door of its type one tile away on the same level.
fn placed(w: &mut World, item: ItemId, loc: Point3D, extra: &str) {
    use crate::construction::doors;
    if !doors::is_door(w, item) {
        if is(w, item, "Server.Items.MarkContainer") {
            crate::last::mark::decorate_set_target(w, item, extra);
        }
        return;
    }
    let (ty, z, map) = match w.item(item) {
        Some(i) => (i.ty.full_name, i.location.z, i.map),
        None => return,
    };
    let near: Vec<ItemId> = enumerate::items_in_range(w, map, loc.x, loc.y, 1).collect();
    for link in near {
        if link != item
            && doors::is_door(w, link)
            && w.item(link)
                .is_some_and(|l| l.location.z == z && l.ty.full_name == ty)
        {
            doors::set_link(w, item, Some(link));
            doors::set_link(w, link, Some(item));
            break;
        }
    }
}

fn configure(w: &mut World) {
    let h = w.resources.get_mut::<CommandHooks>();
    h.decorate_construct.push(construct);
    h.decorate_apply.push(apply);
    h.decorate_door_probe = Some(door_probe);
    h.decorate_placed = Some(placed);
}

pub fn register(types: &mut TypeRegistry) {
    types.register_phase_default(
        Phase::Configure,
        "Server.Items.DecorateGlue.Configure",
        configure,
    );
}
