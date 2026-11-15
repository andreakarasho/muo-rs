//! The tool half of the house placement tool: the `HousePlacementTool` item, its category and
//! list gumps, `NewHousePlacementTarget` and `HousePlacementEntry.OnPlacement`/`ConstructHouse`
//! (the entry tables are `placement_entry.rs`).
//!
//! A `HouseFoundation` house is built through `HouseDeps::construct_foundation`, which `foundation::install` sets.

use muo_content::factory::find_item_by_type;
use muo_content::gumps::builder::{DynamicGumpBuilder, StaticGumpBuilder};
use muo_content::gumps::layout::GumpButtonType;
use muo_content::gumps::{
    DynamicGumpLogic, DynamicGumpObj, RelayInfo, StaticGumpLogic, StaticGumpObj,
    send_gump_to_mobile,
};
use muo_content::items::house_deps::{WarningGump, deps};
use muo_content::vendor::banker;
use muo_primitives::Point3D;
use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, ops};
use muo_server::mobile::AccessLevel;
use muo_server::net_state::NetStateId;
use muo_server::region;
use muo_server::target::{Target, TargetFlags, add_target, set_mobile_target};
use muo_server::type_registry::TypeRegistry;

use crate::classes::*;
use crate::contest::*;
use crate::house::access::{has_account_house, msg};
use crate::house::doors::remove_keys;
use crate::placement::{self, HousePlacementResult};
use crate::placement_entry::{self, Entry};
use crate::preview::new_preview_house;

const LABEL_COLOR: i32 = 0x7FFF;
const LABEL_HUE: i32 = 0x480;

fn alive(w: &mut World, m: MobileId) -> bool {
    muo_server::mobile::part2::stats::check_alive(w, m, true)
}

fn has_tool(w: &World, m: MobileId) -> bool {
    w.mobile(m)
        .and_then(|x| x.backpack)
        .and_then(|p| find_item_by_type(w, p, "Server.Items.HousePlacementTool"))
        .is_some()
}

/// `Server.Items.HousePlacementTool`.
pub static HOUSE_PLACEMENT_TOOL: ItemType = ItemType {
    full_name: "Server.Items.HousePlacementTool",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let id = ops::new_item_with_id(w, &HOUSE_PLACEMENT_TOOL, 0);
        muo_content::templates::apply_item_as(w, id, HOUSE_PLACEMENT_TOOL.full_name);
        id
    }),
    default_weight: |w, id| {
        crate::template_props::item_float(w, id, "defaultWeight", ITEM_BASE.default_weight)
    },
    label_number: |w, id| {
        crate::template_props::item_int(w, id, "labelNumber", ITEM_BASE.label_number)
    },
    on_double_click: |w, id, from| {
        let pack = w.mobile(from).and_then(|m| m.backpack);
        let in_pack = match pack {
            Some(p) => ops::is_child_of(w, id, Some(EntityId::Item(p))),
            None => ops::is_child_of(w, id, None),
        };
        if !in_pack {
            msg(w, from, 1042001); // That must be in your pack for you to use it.
            return;
        }
        send_gump_to_mobile(
            w,
            from,
            Box::new(StaticGumpObj::new(50, 50, CategoryGump)),
            true,
        );
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        r.read_i32()?;
        Ok(())
    },
    ..ITEM_BASE
};

/// Which entry table a list gump shows.
#[derive(Copy, Clone, Debug)]
pub enum Tables {
    Houses,
    Two,
    Three,
}

fn table(w: &World, t: Tables) -> &'static [Entry] {
    match t {
        Tables::Houses => placement_entry::houses(w),
        Tables::Two => placement_entry::two_story(),
        Tables::Three => placement_entry::three_story(),
    }
}

// ---- HousePlacementCategoryGump ----

struct CategoryGump;

impl StaticGumpLogic for CategoryGump {
    const TYPE_NAME: &'static str = "Server.Items.HousePlacementCategoryGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut StaticGumpBuilder) {
        b.add_page(0);
        b.add_background(0, 0, 270, 145, 5054);
        b.add_image_tiled(10, 10, 250, 125, 2624);
        b.add_alpha_region(10, 10, 250, 125);
        b.add_html_localized_color(10, 10, 250, 20, 1060239, LABEL_COLOR, false, false); // <CENTER>HOUSE PLACEMENT TOOL</CENTER>
        b.add_button(10, 110, 4017, 4019, 0);
        b.add_html_localized_color(45, 110, 150, 20, 3000363, LABEL_COLOR, false, false); // Close
        b.add_button(10, 40, 4005, 4007, 1);
        b.add_html_localized_color(45, 40, 200, 20, 1060390, LABEL_COLOR, false, false); // Classic Houses
        b.add_button(10, 60, 4005, 4007, 2);
        b.add_html_localized_color(45, 60, 200, 20, 1060391, LABEL_COLOR, false, false); // 2-Story Customizable Houses
        b.add_button(10, 80, 4005, 4007, 3);
        b.add_html_localized_color(45, 80, 200, 20, 1060392, LABEL_COLOR, false, false); // 3-Story Customizable Houses
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let Some(from) = w.net_mobile(sender) else {
            return;
        };
        if !alive(w, from) || !has_tool(w, from) {
            return;
        }
        match info.button_id {
            1 => show_list(w, from, Tables::Houses),
            2 => show_list(w, from, Tables::Two),
            3 => show_list(w, from, Tables::Three),
            _ => {}
        }
    }
}

fn send_list(w: &mut World, from: MobileId, entries: Tables) {
    // `build_layout` takes no world: the balance, table and vendor-system flag are read here.
    let balance = banker::get_balance(w, from);
    let new_vendor = crate::house::new_vendor_system(w);
    let table: &'static [Entry] = table(w, entries);
    let gump = ListGump {
        from,
        entries,
        balance,
        table,
        new_vendor,
    };
    send_gump_to_mobile(w, from, Box::new(DynamicGumpObj::new(50, 50, gump)), true);
}

// ---- HousePlacementListGump ----

struct ListGump {
    from: MobileId,
    entries: Tables,
    balance: i32,
    table: &'static [Entry],
    new_vendor: bool,
}

impl DynamicGumpLogic for ListGump {
    const TYPE_NAME: &'static str = "Server.Items.HousePlacementListGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        let (balance, entries, new_vendor) = (self.balance, self.table, self.new_vendor);
        b.add_page(0);
        b.add_background(0, 0, 520, 420, 5054);
        b.add_image_tiled(10, 10, 500, 20, 2624);
        b.add_alpha_region(10, 10, 500, 20);
        b.add_html_localized_color(10, 10, 500, 20, 1060239, LABEL_COLOR, false, false); // <CENTER>HOUSE PLACEMENT TOOL</CENTER>
        b.add_image_tiled(10, 40, 500, 20, 2624);
        b.add_alpha_region(10, 40, 500, 20);
        b.add_html_localized_color(50, 40, 225, 20, 1060235, LABEL_COLOR, false, false); // House Description
        b.add_html_localized_color(275, 40, 75, 20, 1060236, LABEL_COLOR, false, false); // Storage
        b.add_html_localized_color(350, 40, 75, 20, 1060237, LABEL_COLOR, false, false); // Lockdowns
        b.add_html_localized_color(425, 40, 75, 20, 1060034, LABEL_COLOR, false, false); // Cost
        b.add_image_tiled(10, 70, 500, 280, 2624);
        b.add_alpha_region(10, 70, 500, 280);
        b.add_image_tiled(10, 360, 500, 20, 2624);
        b.add_alpha_region(10, 360, 500, 20);
        b.add_html_localized_color(10, 360, 250, 20, 1060645, LABEL_COLOR, false, false); // Bank Balance:
        b.add_label(250, 360, LABEL_HUE, &balance.to_string());
        b.add_image_tiled(10, 390, 500, 20, 2624);
        b.add_alpha_region(10, 390, 500, 20);
        b.add_button(10, 390, 4017, 4019, 0);
        b.add_html_localized_color(50, 390, 100, 20, 3000363, LABEL_COLOR, false, false); // Close

        for (i, entry) in entries.iter().enumerate() {
            let (page, index) = (i as i32 / 14 + 1, i as i32 % 14);
            if index == 0 {
                if page > 1 {
                    b.add_button_type(450, 390, 4005, 4007, 0, GumpButtonType::Page, page);
                    b.add_html_localized_color(
                        400,
                        390,
                        100,
                        20,
                        3000406,
                        LABEL_COLOR,
                        false,
                        false,
                    ); // Next
                }
                b.add_page(page);
                if page > 1 {
                    b.add_button_type(200, 390, 4014, 4016, 0, GumpButtonType::Page, page - 1);
                    b.add_html_localized_color(
                        250,
                        390,
                        100,
                        20,
                        3000405,
                        LABEL_COLOR,
                        false,
                        false,
                    ); // Previous
                }
            }
            let y = 70 + index * 20;
            b.add_button(10, y, 4005, 4007, 1 + i as i32);
            b.add_html_localized_color(
                50,
                y,
                225,
                20,
                entry.description,
                LABEL_COLOR,
                false,
                false,
            );
            b.add_label(275, y, LABEL_HUE, &entry.storage(new_vendor).to_string());
            b.add_label(350, y, LABEL_HUE, &entry.lockdowns(new_vendor).to_string());
            b.add_label(425, y, LABEL_HUE, &entry.cost.to_string());
        }
    }

    fn on_response(&mut self, w: &mut World, _sender: NetStateId, info: &RelayInfo) {
        let from = self.from;
        if !alive(w, from) || !has_tool(w, from) {
            return;
        }
        let entries = table(w, self.entries);
        let index = info.button_id - 1;
        if index < 0 || index as usize >= entries.len() {
            send_gump_to_mobile(
                w,
                from,
                Box::new(StaticGumpObj::new(50, 50, CategoryGump)),
                true,
            );
        } else if w
            .mobile(from)
            .is_some_and(|m| m.access_level < AccessLevel::GameMaster)
            && has_account_house(w, from)
        {
            msg(w, from, 501271); // You already own a house, you may not place another!
        } else {
            begin_target(w, from, self.entries, index as usize);
        }
    }
}

/// Opens the list gump for `from`: `new HousePlacementListGump(from, entries)`.
pub fn show_list(w: &mut World, from: MobileId, tables: Tables) {
    send_list(w, from, tables);
}

// ---- NewHousePlacementTarget ----

fn begin_target(w: &mut World, from: MobileId, tables: Tables, index: usize) {
    let entry = &table(w, tables)[index];
    let mut t = Target::new_multi(
        w,
        entry.multi_id,
        entry.offset(),
        14,
        true,
        TargetFlags::NONE,
    );
    let placed = std::rc::Rc::new(std::cell::Cell::new(false));
    let p2 = placed.clone();
    t.handlers.on_target = Some(std::rc::Rc::new(move |w, from, targeted| {
        let Some(p) = crate::deeds::target_point(w, targeted) else {
            return;
        };
        if !alive(w, from) || !has_tool(w, from) {
            return;
        }
        let map = w.mobile(from).and_then(|m| m.map);
        let reg = region::find(w, p, map);
        let gm = w
            .mobile(from)
            .is_some_and(|m| m.access_level >= AccessLevel::GameMaster);
        if gm || placement::allow_housing(w, reg, from, p) {
            let entry = &table(w, tables)[index];
            p2.set(on_placement(w, entry, from, p));
        } else {
            crate::deeds::housing_refused(w, from, reg);
        }
    }));
    t.handlers.on_target_finish = Some(std::rc::Rc::new(move |w, from| {
        if !alive(w, from) || !has_tool(w, from) {
            return;
        }
        if !placed.get() {
            show_list(w, from, tables);
        }
    }));
    let id = add_target(w, t);
    set_mobile_target(w, from, Some(id));
}

/// `ConstructHouse(from)`: the house class of the entry, built for `from`.
pub fn construct_house(w: &mut World, e: &Entry, from: MobileId) -> Option<ItemId> {
    let id = e.multi_id;
    Some(match e.class {
        "HouseFoundation" => {
            let f = deps(w)?.construct_foundation?;
            return f(w, from, id, e.storage, e.lockdowns);
        }
        "SmallOldHouse" => new_small_old_house(w, from, id),
        "SmallShop" => new_small_shop(w, from, id),
        "TwoStoryHouse" => new_two_story_house(w, from, id),
        "GuildHouse" => new_guild_house(w, from),
        "Tower" => new_tower(w, from),
        "Keep" => new_keep(w, from),
        "Castle" => new_castle(w, from),
        "LargePatioHouse" => new_large_patio_house(w, from),
        "LargeMarbleHouse" => new_large_marble_house(w, from),
        "SmallTower" => new_small_tower(w, from),
        "LogCabin" => new_log_cabin(w, from),
        "SandStonePatio" => new_sand_stone_patio(w, from),
        "TwoStoryVilla" => new_two_story_villa(w, from),
        "TrinsicKeep" => new_trinsic_keep(w, from),
        "GothicRoseCastle" => new_gothic_rose_castle(w, from),
        "ElsaCastle" => new_elsa_castle(w, from),
        "Spires" => new_spires(w, from),
        "CastleOfOceania" => new_castle_of_oceania(w, from),
        "FeudalCastle" => new_feudal_castle(w, from),
        "RobinsNest" => new_robins_nest(w, from),
        "TraditionalKeep" => new_traditional_keep(w, from),
        "VillaCrowley" => new_villa_crowley(w, from),
        "DarkthornKeep" => new_darkthorn_keep(w, from),
        "SandalwoodKeep" => new_sandalwood_keep(w, from),
        "CasaMoga" => new_casa_moga(w, from),
        "RobinsRoost" => new_robins_roost(w, from),
        "Camelot" => new_camelot(w, from),
        "LacrimaeInCaelo" => new_lacrimae_in_caelo(w, from),
        "OkinawaSweetDreamCastle" => new_okinawa_sweet_dream_castle(w, from),
        "TheSandstoneCastle" => new_the_sandstone_castle(w, from),
        "GrimswindSisters" => new_grimswind_sisters(w, from),
        _ => return None,
    })
}

type Callback = Box<dyn FnOnce(&mut World, bool)>;

fn warn(w: &mut World, to: MobileId, g: WarningGump, cb: Callback) {
    if let Some(f) = deps(w).and_then(|x| x.warning_gump) {
        f(w, to, g, cb);
    }
}

/// `OnPlacement(from, p)`: true when the preview was shown.
pub fn on_placement(w: &mut World, e: &'static Entry, from: MobileId, p: Point3D) -> bool {
    if !alive(w, from) || !has_tool(w, from) {
        return false;
    }
    let o = e.offset();
    let center = Point3D::new(p.x - o.x, p.y - o.y, p.z - o.z);
    let (res, to_move) = placement::check(w, from, e.multi_id, center, e.direction);
    match res {
        HousePlacementResult::Valid => {
            if !is_gm(w, from) && has_account_house(w, from) {
                msg(w, from, 501271); // You already own a house, you may not place another!
            } else {
                msg(w, from, 1011576); // This is a valid location.
                let prev = new_preview_house(w, e.multi_id);
                let mcl = muo_server::item::multi::components(w, prev);
                let mut ban =
                    Point3D::new(center.x + mcl.min().x, center.y + mcl.max().y + 1, center.z);
                for entry in &mcl.list {
                    let id = i32::from(entry.item_id);
                    if (0xBA3..=0xC0E).contains(&id) {
                        ban = Point3D::new(
                            center.x + i32::from(entry.offset_x),
                            center.y + i32::from(entry.offset_y),
                            center.z,
                        );
                        break;
                    }
                }
                for ent in to_move {
                    match ent {
                        EntityId::Mobile(m) => {
                            muo_server::mobile::part1::lifecycle::set_location(w, m, ban)
                        }
                        EntityId::Item(i) => ops::set_location(w, i, ban),
                    }
                }
                let map = w.mobile(from).and_then(|m| m.map);
                ops::move_to_world_map(w, prev, center, map);
                /* You are about to place a new house. Placing this house will condemn any and all of your other
                 * houses that you may have. All of your houses on all shards will be affected. In addition, you will
                 * not be able to place another house or have one transferred to you for one (1) real-life week.
                 * Once you accept these terms, these effects cannot be reversed. ... */
                let g = WarningGump {
                    name: "Server.Items.HousePlacementEntry+CondemnWarningGump",
                    header: 1060635,
                    header_color: 0x7800,
                    content: 1049583,
                    width: 420,
                    height: 280,
                };
                warn(
                    w,
                    from,
                    g,
                    Box::new(move |w, okay| placement_warning_callback(w, e, from, okay, prev)),
                );
                return true;
            }
        }
        r => refusal(w, from, r),
    }
    false
}

fn is_gm(w: &World, m: MobileId) -> bool {
    w.mobile(m)
        .is_some_and(|x| x.access_level >= AccessLevel::GameMaster)
}

fn refusal(w: &mut World, from: MobileId, r: HousePlacementResult) {
    match r {
        HousePlacementResult::BadItem
        | HousePlacementResult::BadLand
        | HousePlacementResult::BadStatic
        | HousePlacementResult::BadRegionHidden
        | HousePlacementResult::NoSurface => {
            // The house could not be created here.  Either something is blocking the house, or the house would not
            // be on valid terrain.
            msg(w, from, 1043287);
        }
        HousePlacementResult::BadRegion => msg(w, from, 501265), // Housing cannot be created in this area.
        HousePlacementResult::BadRegionTemp => msg(w, from, 501270), // Lord British has decreed a 'no build' period
        HousePlacementResult::BadRegionRaffle => msg(w, from, 1150493), // You must have a deed for this plot of land
        HousePlacementResult::InvalidCastleKeep => msg(w, from, 1061122), // Castles and keeps cannot be created here.
        HousePlacementResult::Valid => {}
    }
}

/// `PlacementWarning_Callback(from, okay, prevHouse)`.
fn placement_warning_callback(
    w: &mut World,
    e: &'static Entry,
    from: MobileId,
    okay: bool,
    prev: ItemId,
) {
    if !alive(w, from) || !has_tool(w, from) {
        return;
    }
    let delete = |w: &mut World, id: ItemId| {
        if let Some(ty) = w.item(id).map(|i| i.ty) {
            (ty.delete)(w, id);
        }
    };
    if !okay {
        delete(w, prev);
        return;
    }
    if w.item(prev).is_none_or(|i| i.deleted()) {
        // Too much time has passed and the test house you created has been deleted.  Please try again!
        let g = WarningGump {
            name: "Server.Items.HousePlacementEntry+HousePlacementTimeoutNoticeGump",
            header: 1060635,
            header_color: 0x7800,
            content: 1060647,
            width: 320,
            height: 180,
        };
        warn(w, from, g, Box::new(|_, _| {}));
        return;
    }
    let center = w.item(prev).map_or(Point3D::ZERO, |i| i.location);
    delete(w, prev);

    let (res, to_move) = placement::check(w, from, e.multi_id, center, e.direction);
    match res {
        HousePlacementResult::Valid => {
            if !is_gm(w, from) && has_account_house(w, from) {
                msg(w, from, 501271); // You already own a house, you may not place another!
                return;
            }
            let Some(house) = construct_house(w, e, from) else {
                return;
            };
            if let Some(h) = crate::house::data_mut(w, house) {
                h.price = e.cost;
            }
            if is_gm(w, from) {
                let text = format!(
                    "{} gold would have been withdrawn from your bank if you were not a GM.",
                    e.cost
                );
                muo_server::mobile::part2::messages::send_message(w, from, &text);
            } else if banker::withdraw(w, from, e.cost) {
                // ~1_AMOUNT~ gold has been withdrawn from your bank box.
                crate::house::access::msg_args(w, from, 1060398, &e.cost.to_string());
            } else {
                remove_keys(w, house, from);
                if let Some(ty) = w.item(house).map(|i| i.ty) {
                    (ty.delete)(w, house);
                }
                // You do not have the funds available in your bank box to purchase this house.  Try placing a
                // smaller house, or adding gold or checks to your bank box.
                msg(w, from, 1060646);
                return;
            }
            let map = w.mobile(from).and_then(|m| m.map);
            ops::move_to_world_map(w, house, center, map);
            let ban = crate::house::ban_location(w, house);
            for ent in to_move {
                match ent {
                    EntityId::Mobile(m) => {
                        muo_server::mobile::part1::lifecycle::set_location(w, m, ban)
                    }
                    EntityId::Item(i) => ops::set_location(w, i, ban),
                }
            }
        }
        r => refusal(w, from, r),
    }
}

pub fn register(types: &mut TypeRegistry) {
    types.register_item(&HOUSE_PLACEMENT_TOOL, &[]);
}
