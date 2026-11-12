//! `ExportWSC`, `SignParser`, `Dupe`, `ClearCommands`, `SaveStabilityCommand` and `GenCommands`.

use muo_primitives::dotnet::IgnoreCase;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;
use std::rc::Rc;

use muo_content::props::Value;
use muo_primitives::{Point3D, Rectangle2D};
use muo_server::commands::{CommandEventArgs, CommandSystem};
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::ItemDelta;
use muo_server::map::MapId;
use muo_server::mobile::AccessLevel;
use muo_server::mobile::part2::messages::send_message;
use muo_server::target::{TargetFlags, Targeted, begin_target};
use muo_server::world::World;

use crate::hooks::CommandHooks;
use crate::logging::{self, format_entity};

type Handler = fn(&mut World, &mut CommandEventArgs);

pub fn configure(w: &mut World) {
    let mut reg = |name: &str, level: AccessLevel, aliases: &[&str], f: Handler| {
        CommandSystem::register(w, name, level, aliases, Rc::new(f));
    };
    reg("ExportWSC", AccessLevel::Developer, &[], export_on_command);
    reg("SignGen", AccessLevel::Developer, &[], |w, e| {
        sign_parse(w, e.mobile)
    });
    reg("Dupe", AccessLevel::GameMaster, &[], |w, e| {
        dupe_command(w, e, false)
    });
    reg("DupeInBag", AccessLevel::GameMaster, &[], |w, e| {
        dupe_command(w, e, true)
    });
    reg(
        "ClearFacet",
        AccessLevel::Owner,
        &[],
        clear_facet_on_command,
    );
    reg("ClearAll", AccessLevel::Owner, &[], clear_all_on_command);
    reg("ClearXY", AccessLevel::Owner, &[], clear_xy_on_command);
    reg(
        "SaveStability",
        AccessLevel::Administrator,
        &[],
        save_stability_on_command,
    );
    reg(
        "GenCommands",
        AccessLevel::Developer,
        &["GenCmdWeb"],
        crate::help_info::gen_commands_on_command,
    );
}

// ---- ExportWSC -------------------------------------------------------------------------------------------------

/// `[Usage("ExportWSC <file-path>")]`
fn export_on_command(w: &mut World, e: &mut CommandEventArgs) {
    let from = e.mobile;
    if e.arguments.is_empty() {
        send_message(w, from, "Usage: [ExportWSC <file-path>]");
        return;
    }
    let path = e.get_string(0);
    if let Some(dir) = Path::new(&path).parent() {
        let _ = fs::create_dir_all(dir);
    }
    let mut out = String::new();
    let mut remove: Vec<ItemId> = Vec::new();
    let mut count = 0;
    send_message(
        w,
        from,
        &format!("Exporting all static items to \"{path}\"..."),
    );
    send_message(
        w,
        from,
        "This will delete all static items in the world.  Please make a backup.",
    );
    for (id, item) in &w.items {
        let id = *id;
        let is_static = [
            "Server.Items.Static",
            "Server.Items.BaseFloor",
            "Server.Items.BaseWall",
        ]
        .iter()
        .any(|c| muo_content::props::is_class(w, EntityId::Item(id), c));
        if is_static && muo_server::item::ops::root_parent(w, id).is_none() {
            let _ = write!(
                out,
                "SECTION WORLDITEM {count}\n{{\nSERIAL {}\nNAME #\nNAME2 #\nID {}\nX {}\nY {}\nZ {}\nCOLOR {}\nCONT -1\nTYPE 0\nAMOUNT 1\nWEIGHT 255\nOWNER -1\nSPAWN -1\nVALUE 1\n}}\n\n",
                item.serial,
                item.item_id,
                item.location.x,
                item.location.y,
                item.location.z,
                item.hue
            );
            count += 1;
            remove.push(id);
        }
    }
    // `new StreamWriter(path)` throws on a bad path; the command handler's caller logs it, nothing is deleted.
    if let Err(err) = fs::write(&path, out) {
        send_message(w, from, &err.to_string());
        return;
    }
    for id in remove {
        muo_server::item::base::delete(w, id);
    }
    send_message(
        w,
        from,
        &format!("Export complete. Exported {count} statics."),
    );
}

// ---- SignParser ------------------------------------------------------------------------------------------------

struct SignEntry {
    text: String,
    location: Point3D,
    item_id: i32,
    map: i32,
}

/// `SignParser.Parse(from)`.
pub fn sign_parse(w: &mut World, from: MobileId) {
    let cfg = w.core.base_directory.join("Data/signs.cfg");
    if !cfg.exists() {
        send_message(w, from, &format!("{} not found!", cfg.display()));
        return;
    }
    send_message(w, from, "Generating signs, please wait.");
    let Ok(text) = fs::read_to_string(&cfg) else {
        return;
    };
    let ti = |s: &str| muo_primitives::parse::to_i32(s);
    let mut list = Vec::new();
    for line in text.lines() {
        let split: Vec<&str> = line.split(' ').collect();
        let skip: usize = split.iter().take(5).map(|s| s.len() + 1).sum();
        if split.len() < 5 || skip > line.len() {
            // A short line or bad range start: the command stops here, nothing is generated and no
            // completion message is sent.
            return;
        }
        list.push(SignEntry {
            text: line.get(skip..).unwrap_or("").to_string(),
            location: Point3D::new(ti(split[2]), ti(split[3]), ti(split[4])),
            item_id: ti(split[1]),
            map: ti(split[0]),
        });
    }
    for e in &list {
        let maps: &[MapId] = match e.map {
            0 => &[MapId::FELUCCA, MapId::TRAMMEL],
            1 => &[MapId::FELUCCA],
            2 => &[MapId::TRAMMEL],
            3 => &[MapId::ILSHENAR],
            4 => &[MapId::MALAS],
            5 => &[MapId::TOKUNO],
            _ => &[],
        };
        for map in maps {
            add_static(w, e.item_id, e.location, *map, &e.text);
        }
    }
    send_message(w, from, "Sign generating complete.");
}

/// `SignParser.Add_Static`.
pub fn add_static(w: &mut World, item_id: i32, location: Point3D, map: MapId, name: &str) {
    let existing: Vec<ItemId> =
        muo_server::map::enumerate::items_at(w, Some(map), location.x, location.y)
            .filter(|i| {
                muo_content::props::is_class(w, EntityId::Item(*i), "Server.Items.Sign")
                    && w.item(*i)
                        .is_some_and(|x| x.location.z == location.z && x.item_id == item_id)
            })
            .collect();
    for i in existing {
        muo_server::item::base::delete(w, i);
    }
    let sign = if let Some(number) = name.strip_prefix('#') {
        muo_content::factory::make_item_args(
            w,
            "Server.Items.LocalizedSign",
            &[
                Value::Int(i64::from(item_id)),
                Value::Int(i64::from(muo_primitives::parse::to_i32(number))),
            ],
        )
    } else {
        let s = muo_content::factory::make_item_args(
            w,
            "Server.Items.Sign",
            &[Value::Int(i64::from(item_id))],
        );
        if let Some(s) = s {
            muo_server::item::ops::set_name(w, s, Some(name.to_string()));
        }
        s
    };
    let Some(sign) = sign else {
        // no Sign class registered: nothing to place
        return;
    };
    if map == MapId::MALAS {
        let hue = match (location.x, location.y) {
            (x, y) if x >= 965 && y >= 502 && x <= 1012 && y <= 537 => Some(0x47E),
            (x, y) if x >= 1960 && y >= 1278 && x < 2106 && y < 1413 => Some(0x44E),
            _ => None,
        };
        if let Some(h) = hue
            && let Some(ty) = w.item(sign).map(|i| i.ty)
        {
            (ty.set_hue)(w, sign, h);
        }
    }
    muo_server::item::ops::move_to_world_map(w, sign, location, Some(map));
}

// ---- Dupe ------------------------------------------------------------------------------------------------------

/// `[Usage("Dupe [amount]")]` / `[Usage("DupeInBag <count>")]`
fn dupe_command(w: &mut World, e: &mut CommandEventArgs, in_bag: bool) {
    let mut amount = 1;
    if e.length() >= 1 {
        amount = e.get_int32(0);
    }
    let amount = if amount > 0 { amount } else { 1 };
    begin_target(
        w,
        e.mobile,
        15,
        false,
        TargetFlags::NONE,
        move |w, from, o| dupe_on_target(w, from, o, in_bag, amount),
    );
    send_message(w, e.mobile, "What do you wish to dupe?");
}

fn dupe_on_target(w: &mut World, from: MobileId, o: Targeted, in_bag: bool, amount: i32) {
    let Targeted::Entity(EntityId::Item(copy)) = o else {
        send_message(w, from, "You can only dupe items.");
        return;
    };
    let level = w
        .mobile(from)
        .map_or(AccessLevel::Player, |m| m.access_level);
    let text = format!(
        "{level:?} {} duping {} (inBag={}; amount={amount})",
        format_entity(w, EntityId::Mobile(from)),
        format_entity(w, EntityId::Item(copy)),
        if in_bag { "True" } else { "False" }
    );
    logging::write_line(w, from, &text);
    let pack: Option<ItemId> = if in_bag {
        match w.item(copy).and_then(|i| i.parent) {
            Some(EntityId::Item(c)) if w.item(c).is_some_and(|x| x.cont.is_some()) => Some(c),
            Some(EntityId::Mobile(m)) => w.mobile(m).and_then(|x| x.backpack),
            _ => None,
        }
    } else {
        w.mobile(from).and_then(|x| x.backpack)
    };
    if do_dupe(w, copy, amount, Some(from), pack) {
        send_message(w, from, "Duping done.");
    } else {
        send_message(w, from, "Duping Error!");
    }
}

/// `src.GetType().GetConstructor(out paramCount)`: a way to make an item of the class of `src` with every argument
/// defaulted. `None` when the class has no constructor with zero required parameters.
fn construct_like(w: &mut World, src: ItemId) -> Option<ItemId> {
    let ty = w.item(src)?.ty;
    if let Some(ctors) = muo_content::props::ctors_of(w, ty.full_name) {
        for c in ctors {
            if c.params.iter().all(|p| p.default.is_some()) {
                let values: Vec<Value> = c
                    .params
                    .iter()
                    .map(|p| p.default.clone().unwrap_or(Value::Null))
                    .collect();
                return match (c.make)(w, &values) {
                    EntityId::Item(i) => Some(i),
                    _ => None,
                };
            }
        }
        return None;
    }
    ty.construct.map(|f| f(w))
}

/// `Dupe.DoDupe(src, amount, from, pack)`.
pub fn do_dupe(
    w: &mut World,
    src: ItemId,
    amount: i32,
    from: Option<MobileId>,
    pack: Option<ItemId>,
) -> bool {
    if let Some(f) = from {
        send_message(w, f, &format!("Duping {amount}..."));
    }
    if construct_like(w, src)
        .map(|i| muo_server::item::base::delete(w, i))
        .is_none()
    {
        if let Some(f) = from {
            send_message(
                w,
                f,
                "Unable to dupe. Item must have a constructor with zero required parameters.",
            );
        }
        return false;
    }
    for _ in 0..amount {
        if let Some(new_item) = do_dupe_one(w, src, from) {
            if let Some(pack) = pack {
                if let Some(ty) = w.item(pack).map(|i| i.ty) {
                    (ty.drop_item)(w, pack, new_item);
                }
            } else if let Some(f) = from {
                let (loc, map) = w.mobile(f).map(|m| (m.location, m.map)).unwrap_or_default();
                muo_server::item::ops::move_to_world_map(w, new_item, loc, map);
            }
        }
    }
    true
}

/// `Dupe.DoDupe(src, c, args, from)`.
fn do_dupe_one(w: &mut World, src: ItemId, from: Option<MobileId>) -> Option<ItemId> {
    let new_item = construct_like(w, src)?;
    let src_ty = w.item(src)?.ty;
    (src_ty.dupe)(w, src, new_item);
    muo_server::item::base::update_totals(w, new_item);
    muo_server::item::ops::invalidate_properties(w, new_item);
    muo_server::item::ops::delta(w, new_item, ItemDelta::UPDATE);
    if let Some(f) = from {
        let level = w.mobile(f).map_or(AccessLevel::Player, |m| m.access_level);
        let text = format!(
            "{level:?} {} duped {} creating {}",
            format_entity(w, EntityId::Mobile(f)),
            format_entity(w, EntityId::Item(src)),
            format_entity(w, EntityId::Item(new_item))
        );
        logging::write_line(w, f, &text);
    }
    // Recurse for items that have items
    let stale: Vec<ItemId> = w
        .item(new_item)
        .map(|i| i.items.clone())
        .unwrap_or_default();
    for j in stale.into_iter().rev() {
        muo_server::item::base::remove_item(w, new_item, j);
        muo_server::item::base::delete(w, j);
    }
    let children: Vec<ItemId> = w.item(src).map(|i| i.items.clone()).unwrap_or_default();
    for child in children {
        if construct_like(w, child)
            .map(|i| muo_server::item::base::delete(w, i))
            .is_none()
        {
            continue;
        }
        if let Some(sub) = do_dupe_one(w, child, from) {
            muo_server::item::base::add_item(w, new_item, sub);
        }
    }
    Some(new_item)
}

// ---- ClearCommands ---------------------------------------------------------------------------------------------

/// `GetObjects(predicate)`: loose items and non-player mobiles.
fn get_objects(w: &World, map: Option<Option<MapId>>) -> Vec<EntityId> {
    let mut list = Vec::new();
    for (id, item) in &w.items {
        if item.parent.is_none() && map.is_none_or(|m| item.map == m) {
            list.push(EntityId::Item(*id));
        }
    }
    for (id, m) in &w.mobiles {
        if !m.player && map.is_none_or(|mm| m.map == mm) {
            list.push(EntityId::Mobile(*id));
        }
    }
    list
}

fn clear_facet_on_command(w: &mut World, e: &mut CommandEventArgs) {
    let from = e.mobile;
    let map = w.mobile(from).and_then(|m| m.map);
    let list = get_objects(w, Some(map));
    // `map.Name` throws for a null map.
    let Some(name) = map
        .and_then(|m| w.maps.get(m))
        .map(|m| m.name().to_string())
    else {
        return;
    };
    delete_objects(w, list, from, &name);
}

fn clear_all_on_command(w: &mut World, e: &mut CommandEventArgs) {
    let list = get_objects(w, None);
    delete_objects(w, list, e.mobile, "globally");
}

fn clear_xy_on_command(w: &mut World, e: &mut CommandEventArgs) {
    let from = e.mobile;
    let p = |i: usize| {
        e.arguments
            .get(i)
            .and_then(|s| s.trim().parse::<i32>().ok())
    };
    let (Some(mut x1), Some(mut y1), Some(mut x2), Some(mut y2)) = (p(0), p(1), p(2), p(3)) else {
        send_message(w, from, "Format: ClearXY x1 y1 x2 y2");
        return;
    };
    if e.arguments.len() != 4 {
        send_message(w, from, "Format: ClearXY x1 y1 x2 y2");
        return;
    }
    let map = w.mobile(from).and_then(|m| m.map);
    if x2 < x1 {
        std::mem::swap(&mut x1, &mut x2);
    }
    if y2 < y1 {
        std::mem::swap(&mut y1, &mut y2);
    }
    let rect = Rectangle2D::new(x1, y1, x2 - x1 + 1, y2 - y1 + 1);
    let mut list: Vec<EntityId> = Vec::new();
    for i in muo_server::map::enumerate::items_in_bounds(w, map, rect, true) {
        list.push(EntityId::Item(i));
    }
    for m in muo_server::map::enumerate::mobiles_in_bounds(w, map, rect, true) {
        if !w.mobile(m).is_some_and(|x| x.player) {
            list.push(EntityId::Mobile(m));
        }
    }
    let Some(name) = map
        .and_then(|m| w.maps.get(m))
        .map(|m| m.name().to_string())
    else {
        return;
    };
    delete_objects(
        w,
        list,
        from,
        &format!("({x1}, {y1}) to ({x2}, {y2}) in {name}"),
    );
}

fn plural(n: usize) -> &'static str {
    if n == 1 { "" } else { "s" }
}

fn delete_objects(w: &mut World, list: Vec<EntityId>, from: MobileId, facets: &str) {
    if list.is_empty() {
        send_message(w, from, "There were no objects found to delete.");
        return;
    }
    let n = list.len();
    let level = w
        .mobile(from)
        .map_or(AccessLevel::Player, |m| m.access_level);
    let text = format!(
        "{level:?} {} started cleaning {facets} ({n} object{})",
        format_entity(w, EntityId::Mobile(from)),
        plural(n)
    );
    logging::write_line(w, from, &text);
    let Some(gump) = w.resources.get_mut::<CommandHooks>().warning_gump else {
        // The gump is `muo_gumps`' (`CommandHooks::warning_gump`); a build without it deletes nothing.
        send_message(w, from, "The confirmation gump is not available.");
        return;
    };
    // `DeleteObjectsNoticeGump`: width 360, height 260.
    let content = format!(
        "You are about to delete {n} object{} from {facets}. Do you really wish to continue?",
        plural(n)
    );
    gump(
        w,
        from,
        &content,
        (360, 260),
        Box::new(move |w, okay| delete_list_callback(w, from, okay, &list)),
    );
}

/// `ClearCommands.DeleteList_Callback`.
pub fn delete_list_callback(w: &mut World, from: MobileId, okay: bool, list: &[EntityId]) {
    if !okay {
        send_message(w, from, "You have chosen not to delete those objects.");
        return;
    }
    let n = list.len();
    let level = w
        .mobile(from)
        .map_or(AccessLevel::Player, |m| m.access_level);
    let text = format!(
        "{level:?} {} deleting {n} object{}",
        format_entity(w, EntityId::Mobile(from)),
        plural(n)
    );
    logging::write_line(w, from, &text);
    for e in list {
        match e {
            EntityId::Item(i) => muo_server::item::base::delete(w, *i),
            EntityId::Mobile(m) => muo_server::mobile::base::delete(w, *m),
        }
    }
    send_message(
        w,
        from,
        &format!("You have deleted {n} object{}.", plural(n)),
    );
}

// ---- SaveStability ---------------------------------------------------------------------------------------------

/// The one check that may run (`SaveStability._current`).
#[derive(Default)]
struct StabilityState {
    current: Option<muo_server::timer::TimerHandle>,
}

struct StabilityRun {
    from: MobileId,
    delay: muo_primitives::TimeSpan,
    entities: Vec<EntityId>,
    hashes: Vec<u64>,
    /// (type name, total, changed), in first-seen order.
    by_type: indexmap::IndexMap<&'static str, (i32, i32)>,
    index: usize,
    comparing: bool,
}

/// `Hash(entity, writer)`: the 64 bit hash of the serialized bytes; 0 for a deleted entity.
fn hash_entity(w: &World, e: EntityId) -> u64 {
    let mut bytes = muo_io::SaveWriter::new(true);
    match e {
        EntityId::Item(i) => {
            let Some(it) = w.item(i).filter(|x| !x.deleted()) else {
                return 0;
            };
            (it.ty.serialize)(w, i, &mut bytes);
        }
        EntityId::Mobile(m) => {
            let Some(mm) = w.mobile(m).filter(|x| !x.deleted) else {
                return 0;
            };
            (mm.ty.serialize)(w, m, &mut bytes);
        }
    }
    muo_primitives::hash::compute_hash64_bytes(bytes.as_slice())
}

fn entity_type_name(w: &World, e: EntityId) -> &'static str {
    match e {
        EntityId::Item(i) => w.item(i).map_or("", |x| x.ty.full_name),
        EntityId::Mobile(m) => w.mobile(m).map_or("", |x| x.ty.full_name),
    }
}

/// `SaveStability.SnapshotEntities(stride)`: every Nth entity, mobiles then items (persistence priority order is
/// mobiles first in `Persistence.EntityPersistences`; the order only decides which entities a stride samples).
pub fn snapshot_entities(w: &World, stride: i32) -> Vec<EntityId> {
    let mut entities: Vec<EntityId> = Vec::new();
    let mut i = 0usize;
    for m in w.mobiles.keys() {
        if i.is_multiple_of(stride as usize) {
            entities.push(EntityId::Mobile(*m));
        }
        i += 1;
    }
    for it in w.items.keys() {
        if i.is_multiple_of(stride as usize) {
            entities.push(EntityId::Item(*it));
        }
        i += 1;
    }
    entities
}

/// `SaveStabilityReport`: (type name, total, changed) per type in first-seen order.
pub struct SaveStabilityReport {
    pub checked: i32,
    pub changed: i32,
    pub by_type: indexmap::IndexMap<&'static str, (i32, i32)>,
}

/// `SaveStability.CaptureHashes(entities)`: the hash of each entity's serialized bytes; 0 for a deleted one.
pub fn capture_hashes(w: &World, entities: &[EntityId]) -> Vec<u64> {
    entities.iter().map(|&e| hash_entity(w, e)).collect()
}

fn is_live(w: &World, e: EntityId) -> bool {
    match e {
        EntityId::Item(i) => w.item(i).is_some_and(|x| !x.deleted()),
        EntityId::Mobile(m) => w.mobile(m).is_some_and(|x| !x.deleted),
    }
}

/// `SaveStability.Compare(entities, captured)`: re-hashes every live entity and reports, per type, how many differ.
pub fn compare(w: &World, entities: &[EntityId], captured: &[u64]) -> SaveStabilityReport {
    let mut report = SaveStabilityReport {
        checked: 0,
        changed: 0,
        by_type: indexmap::IndexMap::new(),
    };
    for (i, &e) in entities.iter().enumerate() {
        if !is_live(w, e) || captured[i] == 0 {
            continue;
        }
        report.checked += 1;
        let stats = report
            .by_type
            .entry(entity_type_name(w, e))
            .or_insert((0, 0));
        stats.0 += 1;
        if hash_entity(w, e) != captured[i] {
            stats.1 += 1;
            report.changed += 1;
        }
    }
    report
}

/// `[Usage("SaveStability [delaySeconds=60] [sampleStride=1] | SaveStability cancel")]`
fn save_stability_on_command(w: &mut World, e: &mut CommandEventArgs) {
    let from = e.mobile;
    if e.length() > 0 && e.get_string(0).eq_ic("cancel") {
        let cur = w.resources.get_mut::<StabilityState>().current.take();
        match cur {
            Some(mut h) => {
                h.cancel(w);
                send_message(w, from, "Save stability check cancelled.");
            }
            None => send_message(w, from, "No save stability check is running."),
        }
        return;
    }
    if w.resources.get_mut::<StabilityState>().current.is_some() {
        send_message(
            w,
            from,
            "A save stability check is already running. Use [SaveStability cancel to stop it.",
        );
        return;
    }
    let delay_seconds = if e.length() > 0 { e.get_int32(0) } else { 60 };
    let stride = if e.length() > 1 { e.get_int32(1) } else { 1 };
    if delay_seconds < 1 || stride < 1 {
        send_message(
            w,
            from,
            "Usage: [SaveStability [delaySeconds] [sampleStride]",
        );
        return;
    }
    let entities = snapshot_entities(w, stride);
    send_message(
        w,
        from,
        &format!(
            "Save stability: hashing {} entities across ticks...",
            entities.len()
        ),
    );
    let n = entities.len();
    let mut run = StabilityRun {
        from,
        delay: muo_primitives::TimeSpan::from_secs_f64(f64::from(delay_seconds)),
        entities,
        hashes: vec![0; n],
        by_type: indexmap::IndexMap::new(),
        index: 0,
        comparing: false,
    };
    let h = muo_server::timer::start(
        w,
        muo_primitives::TimeSpan::ZERO,
        muo_primitives::TimeSpan::ZERO,
        0,
        move |w| stability_step(w, &mut run),
    );
    w.resources.get_mut::<StabilityState>().current = Some(h);
}

/// `Run.Step`: hashes (or compares) entities under a 20 ms budget per tick.
fn stability_step(w: &mut World, run: &mut StabilityRun) {
    let started = std::time::Instant::now();
    while run.index < run.entities.len() {
        let e = run.entities[run.index];
        if run.comparing {
            let captured = run.hashes[run.index];
            let live = match e {
                EntityId::Item(i) => w.item(i).is_some_and(|x| !x.deleted()),
                EntityId::Mobile(m) => w.mobile(m).is_some_and(|x| !x.deleted),
            };
            if live && captured != 0 {
                let name = entity_type_name(w, e);
                let stats = run.by_type.entry(name).or_insert((0, 0));
                stats.0 += 1;
                if hash_entity(w, e) != captured {
                    stats.1 += 1;
                }
            }
        } else {
            run.hashes[run.index] = hash_entity(w, e);
        }
        run.index += 1;
        if started.elapsed() >= std::time::Duration::from_millis(20) {
            return;
        }
    }
    // The pass is done: stop this timer.
    if let Some(mut h) = muo_server::timer::current(w) {
        h.cancel(w);
    }
    if !run.comparing {
        send_message(
            w,
            run.from,
            &format!(
                "Save stability: captured. Comparing again in {:.0}s.",
                run.delay.total_seconds()
            ),
        );
        run.comparing = true;
        run.index = 0;
        let (from, delay) = (run.from, run.delay);
        let entities = std::mem::take(&mut run.entities);
        let hashes = std::mem::take(&mut run.hashes);
        let mut next = StabilityRun {
            from,
            delay,
            entities,
            hashes,
            by_type: indexmap::IndexMap::new(),
            index: 0,
            comparing: true,
        };
        let h = muo_server::timer::start(w, delay, muo_primitives::TimeSpan::ZERO, 0, move |w| {
            stability_step(w, &mut next)
        });
        w.resources.get_mut::<StabilityState>().current = Some(h);
        return;
    }
    stability_finish(w, run);
}

fn stability_finish(w: &mut World, run: &StabilityRun) {
    w.resources.get_mut::<StabilityState>().current = None;
    let (mut checked, mut changed) = (0, 0);
    let mut ranked: Vec<(&'static str, (i32, i32))> = Vec::new();
    for (name, stats) in &run.by_type {
        checked += stats.0;
        changed += stats.1;
        if stats.1 > 0 {
            ranked.push((name, *stats));
        }
    }
    // List.Sort: an unstable introsort; `net_sort` keeps its order of equal counts.
    muo_primitives::dotnet::net_sort(&mut ranked, |a, b| b.1.1.cmp(&a.1.1));
    let from = run.from;
    let secs = run.delay.total_seconds();
    send_message(
        w,
        from,
        &format!("Save stability: {changed} of {checked} entities changed over {secs:.0}s."),
    );
    tracing::info!(
        "Save stability: {changed} of {checked} entities changed over {secs}s ({} types)",
        ranked.len()
    );
    for (shown, (name, (total, ch))) in ranked.iter().enumerate() {
        let percent = f64::from(*ch) * 100.0 / f64::from(*total);
        tracing::info!("  {name}: {ch}/{total} ({percent:.1}%)");
        if shown < 25 {
            let short = name.rsplit(['.', '+']).next().unwrap_or(name);
            send_message(w, from, &format!("  {short}: {ch}/{total} ({percent:.1}%)"));
        }
    }
    if ranked.len() > 25 {
        send_message(
            w,
            from,
            &format!("  ...{} more types in the log.", ranked.len() - 25),
        );
    }
}

/// Save stability tests.
#[cfg(test)]
mod save_stability_tests {
    use super::*;
    use muo_content::creature::enums::{AIType, FightMode};
    use muo_content::creature::setters::new_creature;
    use muo_content::creature::{BASE_CREATURE, BASE_CREATURE_C};
    use muo_server::item::{ITEM_BASE, ItemType};
    use muo_server::mobile::MobileType;

    /// Serializes the current time, the classic way a record drifts between saves with no mutation.
    static CLOCK_STAMPED: ItemType = ItemType {
        full_name: "Server.Tests.ClockStampedItem",
        parent: Some(&ITEM_BASE),
        serialize: |w, id, out| {
            (ITEM_BASE.serialize)(w, id, out);
            out.write_date_time_ticks(w.now.ticks());
        },
        ..ITEM_BASE
    };

    static STUB_C: muo_content::creature::CreatureType =
        muo_content::creature::CreatureType { ..BASE_CREATURE_C };
    static CREATURE_STUB: MobileType = MobileType {
        full_name: "Server.Tests.CreatureStub",
        parent: Some(&BASE_CREATURE),
        family: Some(&STUB_C),
        ..BASE_CREATURE
    };

    fn world() -> World {
        let mut w = muo_server::map::test_support::world();
        muo_content::ai::configure_for_tests(&mut w);
        muo_content::misc::guild_persistence::GuildsPersistence::register(&mut w); // World's own, registered by its crate
        w
    }

    fn creature_stub(w: &mut World, x: i32, y: i32) -> MobileId {
        let c = new_creature(
            w,
            &CREATURE_STUB,
            AIType::AiAnimal,
            FightMode::Closest,
            10,
            1,
        );
        w.mobile_mut(c).unwrap().body = muo_primitives::Body(0xEE); // rat
        muo_server::mobile::base::move_to_world(w, c, Point3D::new(x, y, 0), MapId::FELUCCA);
        c
    }

    #[test]
    fn stable_entities_hash_identically_after_time_passes() {
        let mut w = world();
        let bag = muo_server::item::container::new_container(
            &mut w,
            &muo_server::item::container::CONTAINER,
            0xE76,
        );
        let gold = muo_content::items::gold::new_gold(&mut w, 100);
        let dagger = (muo_content::items::leaves::KATANA.construct.unwrap())(&mut w);
        muo_server::item::base::add_item(&mut w, bag, gold);
        muo_server::item::base::add_item(&mut w, bag, dagger);
        muo_server::item::ops::move_to_world_map(
            &mut w,
            bag,
            Point3D::new(1450, 1450, 0),
            Some(MapId::FELUCCA),
        );
        let creature = creature_stub(&mut w, 1451, 1450);

        let mut entities = vec![EntityId::Item(bag), EntityId::Mobile(creature)];
        entities.extend(
            w.item(bag)
                .unwrap()
                .items
                .iter()
                .map(|&i| EntityId::Item(i)),
        );

        let captured = capture_hashes(&w, &entities);
        w.now += muo_primitives::TimeSpan::from_mins(5);

        let report = compare(&w, &entities, &captured);

        assert_eq!(report.checked as usize, entities.len());
        assert_eq!(report.changed, 0);
    }

    #[test]
    fn clock_derived_record_is_reported_by_type() {
        let mut w = world();
        let item = muo_server::item::ops::new_item_with_id(&mut w, &CLOCK_STAMPED, 0x1F03);
        muo_server::item::ops::move_to_world_map(
            &mut w,
            item,
            Point3D::new(1452, 1450, 0),
            Some(MapId::FELUCCA),
        );

        let entities = vec![EntityId::Item(item)];
        let captured = capture_hashes(&w, &entities);

        // Same tick: the drift is invisible, which is why the command waits in real time.
        assert_eq!(compare(&w, &entities, &captured).changed, 0);

        w.now += muo_primitives::TimeSpan::from_mins(1);
        let report = compare(&w, &entities, &captured);

        assert_eq!(report.changed, 1);
        assert_eq!(report.by_type["Server.Tests.ClockStampedItem"], (1, 1));
    }

    #[test]
    fn snapshot_covers_every_registered_entity_persistence() {
        let mut w = world();
        let item = muo_server::item::container::new_container(
            &mut w,
            &muo_server::item::container::CONTAINER,
            0xE76,
        );
        muo_server::item::ops::move_to_world_map(
            &mut w,
            item,
            Point3D::new(1453, 1450, 0),
            Some(MapId::FELUCCA),
        );
        let mobile = creature_stub(&mut w, 1454, 1450);

        let snapshot = snapshot_entities(&w, 1);

        assert!(snapshot.contains(&EntityId::Item(item)));
        assert!(snapshot.contains(&EntityId::Mobile(mobile)));

        let names: Vec<String> = w.persistence.iter().map(|p| p.name().to_string()).collect();
        assert!(names.iter().any(|n| n == "Items"));
        assert!(names.iter().any(|n| n == "Mobiles"));
        assert!(names.iter().any(|n| n == "Guilds"), "{names:?}");
    }
}
