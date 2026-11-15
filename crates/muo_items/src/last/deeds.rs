//! `HairRestylingDeed`, `BarkeepContract` and `DragonBardingDeed`.

use muo_content::craft::{self, CraftResource};
use muo_content::gumps::base::{DynamicGumpLogic, DynamicGumpObj, RelayInfo};
use muo_content::gumps::builder::DynamicGumpBuilder;
use muo_content::gumps::send_gump_to_mobile;
use muo_content::items::equip_hooks::CraftArgs;
use muo_content::props::Value;
use muo_primitives::TimeSpan;
use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, LootType, deps, ops};
use muo_server::mobile::AccessLevel;
use muo_server::net_state::NetStateId;
use muo_server::serialization_ext::ReadEntityExt;
use muo_server::target::{self, TargetFlags, Targeted};
use muo_server::timer;
use muo_server::type_registry::{Phase, TypeRegistry};

use crate::simple::hue;

fn in_pack(w: &mut World, id: ItemId, from: MobileId) -> bool {
    let pack = muo_server::mobile::part1::props_b::backpack(w, from);
    ops::is_child_of(w, id, pack.map(EntityId::Item))
}

fn delete(w: &mut World, id: ItemId) {
    if let Some(ty) = w.item(id).map(|i| i.ty) {
        (ty.delete)(w, id);
    }
}

fn msg(w: &mut World, to: MobileId, number: i32) {
    deps::send_localized_message(w, to, number);
}

// ---- HairRestylingDeed ----

/// `ElvenArray`: `cliloc_F, cliloc_M, ItemID_F, ItemID_M, gump_img_F, gump_img_M` (row 0 is the padding).
const ELVEN: [[i32; 6]; 10] = [
    [0, 0, 0, 0, 0, 0],
    [1011064, 1011064, 0, 0, 0, 0],                     // bald
    [1074386, 1074386, 0x2fc0, 0x2fc0, 0xedf5, 0xc6e5], // long feather
    [1074387, 1074387, 0x2fc1, 0x2fc1, 0xedf6, 0xc6e6], // short
    [1074388, 1074388, 0x2fc2, 0x2fc2, 0xedf7, 0xc6e7], // mullet
    [1074391, 1074391, 0x2fce, 0x2fce, 0xeddc, 0xc6cc], // knob
    [1074392, 1074392, 0x2fcf, 0x2fcf, 0xeddd, 0xc6cd], // braided
    [1074394, 1074394, 0x2fd1, 0x2fd1, 0xeddf, 0xc6cf], // spiked
    [1074389, 1074385, 0x2fcc, 0x2fbf, 0xedda, 0xc6e4], // flower, mid-long
    [1074393, 1074390, 0x2fd0, 0x2fcd, 0xedde, 0xc6cb], // buns, long
];

/// `HumanArray`.
const HUMAN: [[i32; 6]; 11] = [
    [0, 0, 0, 0, 0, 0],
    [1011064, 1011064, 0, 0, 0, 0],                     // bald
    [1011052, 1011052, 0x203B, 0x203B, 0xed1c, 0xC60C], // Short
    [1011053, 1011053, 0x203C, 0x203C, 0xed1d, 0xc60d], // Long
    [1011054, 1011054, 0x203D, 0x203D, 0xed1e, 0xc60e], // Ponytail
    [1011055, 1011055, 0x2044, 0x2044, 0xed27, 0xC60F], // Mohawk
    [1011047, 1011047, 0x2045, 0x2045, 0xED26, 0xED26], // Pageboy
    [1074393, 1011048, 0x2046, 0x2048, 0xed28, 0xEDE5], // Buns, Receding
    [1011049, 1011049, 0x2049, 0x2049, 0xede6, 0xede6], // 2-tails
    [1011050, 1011050, 0x204A, 0x204A, 0xED29, 0xED29], // Topknot
    [1011396, 1011396, 0x2047, 0x2047, 0xed25, 0xc618], // Curly
];

/// `LayoutArray`: `bgX, bgY, htmlX, htmlY, imgX, imgY, butX, butY`.
const LAYOUT: [[i32; 8]; 11] = [
    [0, 0, 0, 0, 0, 0, 0, 0],
    [425, 280, 342, 295, 0, 0, 310, 292],
    [235, 60, 150, 75, 168, 20, 118, 73],
    [235, 115, 150, 130, 168, 70, 118, 128],
    [235, 170, 150, 185, 168, 130, 118, 183],
    [235, 225, 150, 240, 168, 185, 118, 238],
    [425, 60, 342, 75, 358, 18, 310, 73],
    [425, 115, 342, 130, 358, 75, 310, 128],
    [425, 170, 342, 185, 358, 125, 310, 183],
    [425, 225, 342, 240, 358, 185, 310, 238],
    [235, 280, 150, 295, 168, 245, 118, 292], // slot 10, Curly - N/A for elfs.
];

fn racial_data(human: bool) -> &'static [[i32; 6]] {
    if human { &HUMAN } else { &ELVEN }
}

/// `HairRestylingDeedGump`.
struct HairRestylingGump {
    deed: ItemId,
    from: MobileId,
    human: bool,
    female: bool,
}

impl DynamicGumpLogic for HairRestylingGump {
    const TYPE_NAME: &'static str = "Server.Items.HairRestylingDeed+HairRestylingDeedGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        b.add_background(100, 10, 400, 385, 0xA28);
        b.add_html_localized(100, 25, 400, 35, 1013008, false, false);
        b.add_button(175, 340, 0xFA5, 0xFA7, 0x0); // CANCEL
        b.add_html_localized(210, 342, 90, 35, 1011012, false, false); // <CENTER>HAIRSTYLE SELECTION MENU</center>
        let data = racial_data(self.human);
        for (i, row) in data.iter().enumerate().skip(1) {
            let l = &LAYOUT[i];
            b.add_html_localized(
                l[2],
                l[3],
                if i == 1 { 125 } else { 80 },
                if i == 1 { 70 } else { 35 },
                if self.female { row[0] } else { row[1] },
                false,
                false,
            );
            if l[4] != 0 {
                b.add_background(l[0], l[1], 50, 50, 0xA3C);
                b.add_image(l[4], l[5], if self.female { row[4] } else { row[5] }, 0);
            }
            b.add_button(l[6], l[7], 0xFA5, 0xFA7, i as i32);
        }
    }

    fn on_response(&mut self, w: &mut World, _sender: NetStateId, info: &RelayInfo) {
        let from = self.from;
        if !muo_server::mobile::base::alive(w, from) || ops::is_deleted(w, self.deed) {
            return;
        }
        if !(1..=10).contains(&info.button_id) {
            return;
        }
        // `_from.Race` and `pm.Female` are read now, not as the gump was laid out.
        let human =
            muo_server::mobile::part1::props_a::race(w, from) == muo_server::race::RaceId(0);
        let female = w.mobile(from).is_some_and(|m| m.female);
        let data = racial_data(human);
        // The elven table has no row 10; an index there would be out of range.
        let Some(row) = data.get(info.button_id as usize) else {
            return;
        };
        if muo_content::factory::mobile_is(w, from, "Server.Mobiles.PlayerMobile") {
            muo_content::player_mobile::set_hair_mods(w, from, -1, -1); // clear any hairmods (disguise kit, incognito)
            let item = if female { row[2] } else { row[3] };
            muo_server::mobile::part1::props_b::set_hair_item_id(w, from, item);
            delete(w, self.deed);
        }
    }
}

/// `Server.Items.HairRestylingDeed`.
pub static HAIR_RESTYLING_DEED: ItemType = ItemType {
    full_name: "Server.Items.HairRestylingDeed",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let id = ops::new_item_with_id(w, &HAIR_RESTYLING_DEED, 0x14F0);
        ops::set_loot_type(w, id, LootType::Blessed);
        id
    }),
    default_weight: crate::class_props::default_weight,
    label_number: crate::class_props::label_number, // a coupon for a free hair restyling
    on_double_click: |w, id, from| {
        if !in_pack(w, id, from) {
            msg(w, from, 1042001); // That must be in your pack...
            return;
        }
        if w.mobile(from).is_none_or(|m| m.net_state.is_none()) || ops::is_deleted(w, id) {
            return;
        }
        let human =
            muo_server::mobile::part1::props_a::race(w, from) == muo_server::race::RaceId(0);
        let female = w.mobile(from).is_some_and(|m| m.female);
        let gump = DynamicGumpObj::new(
            50,
            50,
            HairRestylingGump {
                deed: id,
                from,
                human,
                female,
            },
        );
        send_gump_to_mobile(w, from, Box::new(gump), false);
    },
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

// ---- BarkeepContract ----

/// `new PlayerBarkeeper(from, house) { Direction = from.Direction & Direction.Mask }` moved to the owner's place.
fn place_barkeeper(w: &mut World, from: MobileId, house: Option<ItemId>, contract: ItemId) {
    let args = [
        Value::Entity(Some(EntityId::Mobile(from))),
        Value::Entity(house.map(EntityId::Item)),
    ];
    let Some(EntityId::Mobile(v)) =
        crate::misc::gadgets::create_with_args_pub(w, "Server.Mobiles.PlayerBarkeeper", &args)
    else {
        return; // no PlayerBarkeeper constructor registered: nothing to place
    };
    let dir = w.mobile(from).map(|m| m.direction.bits() & 7);
    if let Some(d) = dir {
        muo_server::mobile::part1::props_b::set_direction(
            w,
            v,
            muo_primitives::Direction::from_bits_retain(d),
        );
    }
    if let Some((loc, Some(map))) = w.mobile(from).map(|m| (m.location, m.map)) {
        muo_server::mobile::base::move_to_world(w, v, loc, map);
    }
    delete(w, contract);
}

/// `Server.Items.BarkeepContract`.
pub static BARKEEP_CONTRACT: ItemType = ItemType {
    full_name: "Server.Items.BarkeepContract",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let id = ops::new_item_with_id(w, &BARKEEP_CONTRACT, 0x14F0);
        ops::set_loot_type(w, id, LootType::Blessed);
        id
    }),
    default_weight: crate::class_props::default_weight,
    default_name: crate::class_props::default_name,
    on_double_click: |w, id, from| {
        use muo_multis::house::{access, storage};
        if !in_pack(w, id, from) {
            msg(w, from, 1042001); // That must be in your pack for you to use it.
        } else if w
            .mobile(from)
            .is_some_and(|m| m.access_level >= AccessLevel::GameMaster)
        {
            msg(w, from, 503248); // Your godly powers allow you to place this vendor whereever you wish.
            let h = access::find_house_at_mobile(w, from);
            place_barkeeper(w, from, h, id);
        } else {
            let house = access::find_house_at_mobile(w, from);
            match house {
                Some(h) if access::is_owner(w, h, from) => {
                    if !storage::can_place_new_barkeep(w, h) {
                        msg(w, from, 1062490); // That action would exceed the maximum number of barkeeps for this house.
                        return;
                    }
                    let (loc, map) = w
                        .mobile(from)
                        .map_or((muo_primitives::Point3D::default(), None), |m| {
                            (m.location, m.map)
                        });
                    let (vendor, contract) = w
                        .resources
                        .get::<crate::misc::dyes::PlayerVendorHooks>()
                        .and_then(|x| x.is_there_vendor)
                        .map_or((false, false), |f| f(w, loc, map));
                    if vendor {
                        msg(w, from, 1062677); // You cannot place a vendor or barkeep at this location.
                    } else if contract {
                        msg(w, from, 1062678); // You cannot place a vendor or barkeep on top of a rental contract!
                    } else {
                        place_barkeeper(w, from, Some(h), id);
                    }
                }
                _ => muo_server::mobile::part2::messages::local_overhead_message(
                    w,
                    from,
                    muo_primitives::MessageType::REGULAR,
                    0x3B2,
                    false,
                    "You are not the full owner of this house.",
                ),
            }
        }
    },
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

// ---- DragonBardingDeed ----

/// The fields of `DragonBardingDeed`.
#[derive(Clone, Debug, Default)]
pub struct BardingData {
    pub crafted_by: Option<String>,
    pub exceptional: bool,
    pub resource: CraftResource,
}

pub fn bd(w: &World, id: ItemId) -> BardingData {
    w.item(id)
        .and_then(|i| i.ext.get::<BardingData>())
        .cloned()
        .unwrap_or_default()
}

/// `DragonBardingDeed.Resource`.
pub fn barding_resource(w: &World, id: ItemId) -> CraftResource {
    bd(w, id).resource
}

pub fn set_bd(w: &mut World, id: ItemId, d: BardingData) {
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(d);
    }
    ops::invalidate_properties(w, id);
}

/// `Resource = value`: `OnResourceChanged` sets `Hue = CraftResources.GetHue(newValue)`.
pub fn set_resource(w: &mut World, id: ItemId, value: CraftResource) {
    let mut d = bd(w, id);
    if d.resource == value {
        return;
    }
    d.resource = value;
    set_bd(w, id, d);
    let h = craft::hue(w, value);
    hue(w, id, h);
}

type PetStateFn = fn(&World, MobileId) -> Option<(bool, bool, Option<MobileId>)>;
type ApplyFn = fn(&mut World, MobileId, bool, Option<String>, CraftResource, i32);
/// What `DragonBardingDeed.OnTarget` asks of the swamp dragon: `(HasBarding, Controlled, ControlMaster)`, and
/// `pet.BardingExceptional = ..; pet.BardingCraftedBy = ..; pet.BardingHP = pet.BardingMaxHP;
/// pet.BardingResource = ..; pet.HasBarding = true; pet.Hue = hue`. Unset: no mobile is a swamp dragon.
#[derive(Default)]
pub struct BardingHooks {
    pub pet_state: Option<PetStateFn>,
    pub apply: Option<ApplyFn>,
}

fn on_target(w: &mut World, id: ItemId, from: MobileId, t: Targeted) {
    if ops::is_deleted(w, id) {
        return;
    }
    let pet = match t {
        Targeted::Entity(EntityId::Mobile(m)) => Some(m),
        _ => None,
    };
    let state = pet.and_then(|p| {
        w.resources
            .get::<BardingHooks>()
            .and_then(|h| h.pet_state)
            .and_then(|f| f(w, p))
    });
    match (pet, state) {
        (Some(_), Some((false, controlled, master))) if controlled && master == Some(from) => {
            if !in_pack(w, id, from) {
                msg(w, from, 1060640); // The item must be in your backpack to use it.
                return;
            }
            let pet = pet.expect("pet");
            let d = bd(w, id);
            let h = w.item(id).map_or(0, |i| i.hue);
            if let Some(f) = w.resources.get::<BardingHooks>().and_then(|h| h.apply) {
                f(w, pet, d.exceptional, d.crafted_by, d.resource, h);
            }
            delete(w, id);
            // You place the barding on your swamp dragon.  Use a bladed item on your dragon to remove the armor.
            msg(w, from, 1053027);
        }
        (Some(_), Some((false, _, _))) => msg(w, from, 1053026), // You can only put barding on a tamed swamp dragon that you own.
        _ => msg(w, from, 1053025), // That is not an unarmored swamp dragon.
    }
}

/// `Server.Items.DragonBardingDeed` (`[TypeAlias("Server.Items.DragonBarding")]`, `[SerializationGenerator(2, false)]`).
pub static DRAGON_BARDING_DEED: ItemType = ItemType {
    full_name: "Server.Items.DragonBardingDeed",
    parent: Some(&ITEM_BASE),
    construct: None,
    make: Some(|w| {
        let id = ops::new_item_with_id(w, &DRAGON_BARDING_DEED, 0x14F0);
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(BardingData::default());
        }
        id
    }),
    default_weight: crate::class_props::default_weight,
    label_number: |w, id| {
        if bd(w, id).exceptional {
            1053181
        } else {
            1053012
        }
    }, // dragon barding deed
    get_properties: |w, id, list| {
        (ITEM_BASE.get_properties)(w, id, list);
        let d = bd(w, id);
        if d.exceptional
            && let Some(c) = d.crafted_by
        {
            list.add_str(1050043, &c); // crafted by ~1_NAME~
        }
    },
    on_double_click: |w, id, from| {
        if in_pack(w, id, from) {
            target::begin_target(w, from, 6, false, TargetFlags::NONE, move |w, from, t| {
                on_target(w, id, from, t)
            });
            msg(w, from, 1053024); // Select the swamp dragon you wish to place the barding on.
        } else {
            msg(w, from, 1042001); // That must be in your pack for you to use it.
        }
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(2);
        let d = bd(w, id);
        out.write_opt_string(d.crafted_by.as_deref());
        out.write_bool(d.exceptional);
        out.write_enum(4, d.resource as i64);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let version = r.read_i32()?;
        let mut d = BardingData::default();
        if version >= 2 {
            d.crafted_by = r.read_string()?;
            d.exceptional = r.read_bool()?;
            d.resource = CraftResource::from_i32(r.read_enum(4)? as i32).unwrap_or_default();
        } else {
            d.exceptional = r.read_bool()?;
            let crafter = r.read_mobile(w)?;
            if version < 1 {
                r.read_i32()?;
            }
            d.resource = CraftResource::from_i32(r.read_i32()?).unwrap_or_default();
            // `Timer.DelayCall((item, crafter) => item._craftedBy = crafter?.RawName, this, crafter)`
            timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 1, move |w| {
                let name = crafter.and_then(|c| muo_server::mobile::part1::props_b::raw_name(w, c));
                let mut d = bd(w, id);
                d.crafted_by = name;
                set_bd(w, id, d);
            });
        }
        set_bd(w, id, d);
        Ok(())
    },
    ..ITEM_BASE
};

/// `DragonBardingDeed.OnCraft`.
fn on_craft(w: &mut World, id: ItemId, a: &CraftArgs) -> Option<i32> {
    if !w.item(id).is_some_and(|i| i.ty.is(&DRAGON_BARDING_DEED)) {
        return None;
    }
    let mut d = bd(w, id);
    d.exceptional = a.quality >= 2;
    if a.makers_mark {
        d.crafted_by = muo_server::mobile::part1::props_b::raw_name(w, a.from);
    }
    set_bd(w, id, d);
    set_resource(w, id, a.resource);
    if a.do_not_color {
        hue(w, id, 0);
    }
    Some(a.quality)
}

fn configure(w: &mut World) {
    muo_content::craft::hooks_mut(w).craftables.push(on_craft);
}

/// Registers the classes of this file.
pub fn register(types: &mut TypeRegistry) {
    types.register_item(&HAIR_RESTYLING_DEED, &[]);
    types.register_item(&BARKEEP_CONTRACT, &[]);
    types.register_item(&DRAGON_BARDING_DEED, &["Server.Items.DragonBarding"]);
    types.register_phase_default(
        Phase::Configure,
        "Server.Items.DragonBardingDeed.Configure",
        configure,
    );
}

#[cfg(test)]
mod hair_restyling_tests {
    use super::*;

    /// `_from.Race` and `pm.Female` are read when the button is pressed, not when the gump was laid out.
    #[test]
    fn the_hair_restyling_gump_rereads_race_and_sex() {
        let mut w = World::new();
        let from = w.new_mobile(&muo_content::player_mobile::PLAYER_MOBILE);
        let deed = ops::new_item_with_id(&mut w, &HAIR_RESTYLING_DEED, 0x14F0);
        // laid out for a human man; by the time of the answer they are an elven woman
        let mut g = HairRestylingGump {
            deed,
            from,
            human: true,
            female: false,
        };
        muo_server::mobile::part1::props_a::set_race(
            &mut w,
            from,
            Some(muo_server::race::RaceId(1)),
        );
        w.mobile_mut(from).unwrap().female = true;
        g.on_response(&mut w, NetStateId(1), &RelayInfo::new(8, vec![], vec![]));
        assert_eq!(w.mobile(from).unwrap().hair_item_id, ELVEN[8][2]);
    }
}
