//! The young player gumps, `PetResurrectGump`, `PolymorphGump`, `SelectAddonDirectionGump`, `TithingGump` and
//! `ToTAdminGump`.

use std::rc::Rc;

use muo_content::gumps::base::{
    LegacyGumpLogic, LegacyGumpObj, RelayInfo, StaticGumpLogic, StaticGumpObj,
};
use muo_content::gumps::builder::{GumpStrings, HtmlStyle, StaticGumpBuilder};
use muo_content::gumps::layout::GumpButtonType;
use muo_primitives::MessageType;
use muo_primitives::html::TextAlignment;
use muo_server::World;
use muo_server::commands::{CommandEventArgs, CommandSystem};
use muo_server::ids::{ItemId, MobileId};
use muo_server::mobile::AccessLevel;
use muo_server::net_state::NetStateId;

use super::common::{mobile_of, n0, send};
use super::hooks::hooks;

// ---- Young player gumps ----

pub struct YoungDungeonWarningGump;

/// `new YoungDungeonWarningGump()`.
pub fn new_young_dungeon_warning() -> StaticGumpObj<YoungDungeonWarningGump> {
    StaticGumpObj::new(150, 200, YoungDungeonWarningGump)
}

impl StaticGumpLogic for YoungDungeonWarningGump {
    const TYPE_NAME: &'static str = "Server.Gumps.YoungDungeonWarningGump";
    fn singleton(&self) -> bool {
        true
    }
    fn build_layout(&self, b: &mut StaticGumpBuilder) {
        b.add_background(0, 0, 250, 170, 0x12E);
        b.add_background(10, 10, 230, 150, 0x1400);

        // Warning: monsters may attack you on site down here in the dungeons!
        b.add_html_localized_color(20, 25, 210, 70, 1018030, 0x7FFF, false, false);

        b.add_button(163, 125, 0xF8, 0xF9, 0);
    }
}

pub struct YoungDeathNoticeGump;

/// `new YoungDeathNoticeGump()`.
pub fn new_young_death_notice() -> StaticGumpObj<YoungDeathNoticeGump> {
    StaticGumpObj::new(100, 15, YoungDeathNoticeGump)
}

impl StaticGumpLogic for YoungDeathNoticeGump {
    const TYPE_NAME: &'static str = "Server.Gumps.YoungDeathNoticeGump";
    fn singleton(&self) -> bool {
        true
    }
    fn build_layout(&self, b: &mut StaticGumpBuilder) {
        b.set_no_close();

        b.add_background(25, 10, 425, 444, 0x12E);
        b.add_background(35, 20, 405, 424, 0x1400);
        b.add_alpha_region(25, 10, 425, 444);

        b.add_html_localized_color(190, 34, 120, 20, 1046287, 0x7D00, false, false); // You have died.

        // As a ghost you cannot interact with the world. You cannot touch items nor can you use them.
        b.add_html_localized_color(50, 60, 380, 40, 1046288, 0x7FFF, false, false);
        // You can pass through doors as though they do not exist.  However, you cannot pass through walls.
        b.add_html_localized_color(50, 105, 380, 45, 1046289, 0x7FFF, false, false);
        // Since you are a new player, any items you had on your person at the time of your death will be in your backpack upon resurrection.
        b.add_html_localized_color(50, 150, 380, 60, 1046291, 0x7FFF, false, false);
        // To be resurrected you must find a healer in town or wandering in the wilderness.  Some powerful players may also be able to resurrect you.
        b.add_html_localized_color(50, 214, 380, 65, 1046292, 0x7FFF, false, false);
        // While you are still in young status, you will be transported to the nearest healer (along with your items) at the time of your death.
        b.add_html_localized_color(50, 279, 380, 65, 1046293, 0x7FFF, false, false);
        // To rejoin the world of the living simply walk near one of the NPC healers, and they will resurrect you as long as you are not marked as a criminal.
        b.add_html_localized_color(50, 339, 380, 70, 1046294, 0x7FFF, false, false);

        b.add_button(363, 409, 0xF8, 0xF9, 0);
    }
}

pub struct RenounceYoungGump;

/// `Keywords.EventSink_Speech` 0x0035: the gump unless one is open.
pub fn configure_renounce_young(w: &mut World) {
    muo_content::hooks::hooks_mut(w).renounce_young_gump = Some(|w, from| {
        let open = w
            .mobile(from)
            .and_then(|m| m.net_state)
            .is_some_and(|ns| muo_content::gumps::has_gump::<RenounceYoungGump>(w, ns));
        if !open {
            send(w, from, new_renounce_young());
        }
    });
}

/// `new RenounceYoungGump()`.
pub fn new_renounce_young() -> StaticGumpObj<RenounceYoungGump> {
    StaticGumpObj::new(150, 50, RenounceYoungGump)
}

impl StaticGumpLogic for RenounceYoungGump {
    const TYPE_NAME: &'static str = "Server.Gumps.RenounceYoungGump";
    fn singleton(&self) -> bool {
        true
    }
    fn build_layout(&self, b: &mut StaticGumpBuilder) {
        b.add_background(0, 0, 450, 330, 0x12E);
        b.add_background(10, 10, 430, 310, 0x1400);

        b.add_html_localized_color(0, 24, 450, 20, 1013004, 0x7FFF, false, false); // <center> Renouncing 'Young Player' Status</center>

        /* As a 'Young' player, you are currently under a system of protection that prevents
         * you from being attacked by other players and certain monsters. [..] press CANCEL. */
        b.add_html_localized_color(25, 50, 380, 210, 1013005, 0x7FFF, false, false);

        b.add_button(363, 285, 0xF8, 0xF9, 1); // OKAY
        b.add_button(283, 285, 0xF1, 0xF2, 0); // CANCEL
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let Some(from) = mobile_of(w, sender) else {
            return;
        };

        if info.button_id != 1 {
            muo_content::mx::msg(w, from, 502086); // You have chosen not to renounce your `Young' player status.
            return;
        }

        if let Some(account) = w.mobile(from).and_then(|m| m.account) {
            // You have chosen to renounce your `Young' player status.
            muo_content::accounting::remove_young_status(w, account, 502085);
        }
    }
}

// ---- PetResurrectGump ----

pub struct PetResurrectGump {
    hits_scalar: f64,
    pet: MobileId,
    /// `_pet.Name`, read when the layout is compiled (right after the gump is made).
    pet_name: String,
}

/// `new PetResurrectGump(from, pet, hitsScalar)`.
pub fn new_pet_resurrect(
    w: &World,
    pet: MobileId,
    hits_scalar: f64,
) -> StaticGumpObj<PetResurrectGump> {
    StaticGumpObj::new(
        50,
        50,
        PetResurrectGump {
            hits_scalar,
            pet,
            pet_name: w
                .mobile(pet)
                .and_then(|m| (m.ty.name)(w, pet))
                .unwrap_or_default(),
        },
    )
}

impl StaticGumpLogic for PetResurrectGump {
    const TYPE_NAME: &'static str = "Server.Gumps.PetResurrectGump";
    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut StaticGumpBuilder) {
        b.add_page(0);

        b.add_background(10, 10, 265, 140, 0x242C);

        b.add_item(205, 40, 0x4, 0);
        b.add_item(227, 40, 0x5, 0);

        b.add_item(180, 78, 0xCAE, 0);
        b.add_item(195, 90, 0xCAD, 0);
        b.add_item(218, 95, 0xCB0, 0);

        // <div align=center>Wilt thou sanctify the resurrection of:</div>
        b.add_html_localized(30, 30, 150, 75, 1049665, false, false);
        b.add_html_placeholder(30, 70, 150, 25, "petName", true, false);

        b.add_button(40, 105, 0x81A, 0x81B, 0x1); // Okay
        b.add_button(110, 105, 0x819, 0x818, 0x2); // Cancel
    }

    fn build_strings(&self, strings: &mut GumpStrings) {
        strings.set_html_text_styled(
            "petName",
            &self.pet_name,
            HtmlStyle {
                align: TextAlignment::Center,
                ..HtmlStyle::NONE
            },
        );
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        use muo_content::creature::data::read;
        let pet = self.pet;
        let (bonded, dead_pet) = read(w, pet, (false, false), |d| (d.is_bonded, d.is_dead_pet));
        if muo_content::mx::deleted(w, pet) || !bonded || !dead_pet {
            return;
        }

        let Some(from) = mobile_of(w, sender) else {
            return;
        };

        if info.button_id != 1 {
            return;
        }

        let (loc, map, region) = match w.mobile(pet) {
            Some(m) => (m.location, m.map, m.region),
            None => return,
        };
        let fits = match map {
            Some(map) => {
                muo_server::map::fit::can_fit(w, map, loc.x, loc.y, loc.z, 16, false, false, true)
            }
            None => false,
        };
        if !fits {
            muo_content::mx::msg(w, from, 503256); // You fail to resurrect the creature.
            return;
        }

        if muo_server::region::is_part_of_name(w, region, "Khaldun", false) {
            // The veil of death in this area is too strong and resists thy efforts to restore life.
            muo_content::mx::msg(w, from, 1010395);
            return;
        }

        muo_content::mx::play_sound(w, pet, 0x214);
        muo_content::mx::fixed_effect(w, pet, 0x376A, 10, 16);
        muo_content::creature::misc::resurrect_pet(w, pet);

        let master = muo_content::creature::control::control_master(w, pet);
        let decrease_amount = if master == Some(from) { 0.1 } else { 0.2 };

        let count = w
            .mobile(pet)
            .and_then(|m| m.skills.as_ref())
            .map_or(0, |s| s.len());
        for i in 0..count {
            // Decrease all skills on pet.
            if let Some(skill) = muo_server::skills::SkillName::from_i32(i as i32) {
                let base = muo_server::skills::base(w, pet, skill);
                muo_server::skills::set_base(w, pet, skill, base - decrease_amount);
            }
        }

        let still_dead = muo_content::creature::data::read(w, pet, false, |d| d.is_dead_pet);
        if !still_dead && self.hits_scalar > 0.0 {
            let hits = (f64::from(muo_content::mx::hits_max(w, pet)) * self.hits_scalar) as i32;
            muo_server::mobile::part1::props_b::set_hits(w, pet, hits);
        }
    }
}

// ---- PolymorphGump ----

/// `PolymorphEntry`.
pub struct PolymorphEntry {
    pub art_id: i32,
    pub body_id: i32,
    pub loc_number: i32,
    pub x: i32,
    pub y: i32,
}

const fn pe(art_id: i32, body_id: i32, loc_number: i32, x: i32, y: i32) -> PolymorphEntry {
    PolymorphEntry {
        art_id,
        body_id,
        loc_number,
        x,
        y,
    }
}

pub const CHICKEN: PolymorphEntry = pe(8401, 0xD0, 1015236, 15, 10);
pub const DOG: PolymorphEntry = pe(8405, 0xD9, 1015237, 17, 10);
pub const WOLF: PolymorphEntry = pe(8426, 0xE1, 1015238, 18, 10);
pub const PANTHER: PolymorphEntry = pe(8473, 0xD6, 1015239, 20, 14);
pub const GORILLA: PolymorphEntry = pe(8437, 0x1D, 1015240, 23, 10);
pub const BLACK_BEAR: PolymorphEntry = pe(8399, 0xD3, 1015241, 22, 10);
pub const GRIZZLY_BEAR: PolymorphEntry = pe(8411, 0xD4, 1015242, 22, 12);
pub const POLAR_BEAR: PolymorphEntry = pe(8417, 0xD5, 1015243, 26, 10);
pub const HUMAN_MALE: PolymorphEntry = pe(8397, 0x190, 1015244, 29, 8);
pub const HUMAN_FEMALE: PolymorphEntry = pe(8398, 0x191, 1015254, 29, 10);
pub const SLIME: PolymorphEntry = pe(8424, 0x33, 1015246, 5, 10);
pub const ORC: PolymorphEntry = pe(8416, 0x11, 1015247, 29, 10);
pub const LIZARD_MAN: PolymorphEntry = pe(8414, 0x21, 1015248, 26, 10);
pub const GARGOYLE: PolymorphEntry = pe(8409, 0x04, 1015249, 22, 10);
pub const OGRE: PolymorphEntry = pe(8415, 0x01, 1015250, 24, 9);
pub const TROLL: PolymorphEntry = pe(8425, 0x36, 1015251, 25, 9);
pub const ETTIN: PolymorphEntry = pe(8408, 0x02, 1015252, 25, 8);
pub const DAEMON: PolymorphEntry = pe(8403, 0x09, 1015253, 25, 8);

/// `PolymorphGump.Categories`: the localized name and the entries.
const POLY_CATEGORIES: [(i32, [&PolymorphEntry; 9]); 2] = [
    (
        1015235, // Animals
        [
            &CHICKEN,
            &DOG,
            &WOLF,
            &PANTHER,
            &GORILLA,
            &BLACK_BEAR,
            &GRIZZLY_BEAR,
            &POLAR_BEAR,
            &HUMAN_MALE,
        ],
    ),
    (
        1015245, // Monsters
        [
            &SLIME,
            &ORC,
            &LIZARD_MAN,
            &GARGOYLE,
            &OGRE,
            &TROLL,
            &ETTIN,
            &DAEMON,
            &HUMAN_FEMALE,
        ],
    ),
];

const NEW_POLY_ENTRIES: [&PolymorphEntry; 18] = [
    &CHICKEN,
    &DOG,
    &WOLF,
    &PANTHER,
    &GORILLA,
    &BLACK_BEAR,
    &GRIZZLY_BEAR,
    &POLAR_BEAR,
    &HUMAN_MALE,
    &HUMAN_FEMALE,
    &SLIME,
    &ORC,
    &LIZARD_MAN,
    &GARGOYLE,
    &OGRE,
    &TROLL,
    &ETTIN,
    &DAEMON,
];

/// `new PolymorphSpell(caster, scroll, body).Cast()`, through the hook the spells crate sets.
fn cast_polymorph(w: &mut World, caster: MobileId, scroll: Option<ItemId>, body_id: i32) {
    if let Some(f) = hooks(w).and_then(|h| h.polymorph_cast) {
        f(w, caster, scroll, body_id);
    }
}

pub struct PolymorphGump {
    scroll: Option<ItemId>,
}

/// `new PolymorphGump(scroll)`.
pub fn new_polymorph(scroll: Option<ItemId>) -> StaticGumpObj<PolymorphGump> {
    StaticGumpObj::new(50, 50, PolymorphGump { scroll })
}

impl StaticGumpLogic for PolymorphGump {
    const TYPE_NAME: &'static str = "Server.Gumps.PolymorphGump";

    fn build_layout(&self, b: &mut StaticGumpBuilder) {
        b.add_page(0);
        b.add_background(0, 0, 585, 393, 5054);
        b.add_background(195, 36, 387, 275, 3000);
        b.add_html_localized(0, 0, 510, 18, 1015234, false, false); // <center>Polymorph Selection Menu</center>
        b.add_html_localized(60, 355, 150, 18, 1011036, false, false); // OKAY
        b.add_button_type(25, 355, 4005, 4007, 1, GumpButtonType::Reply, 1);
        b.add_html_localized(320, 355, 150, 18, 1011012, false, false); // CANCEL
        b.add_button_type(285, 355, 4005, 4007, 0, GumpButtonType::Reply, 2);

        let mut y = 35;
        for (i, (loc, _)) in POLY_CATEGORIES.iter().enumerate() {
            b.add_html_localized(5, y, 150, 25, *loc, true, false);
            b.add_button_type(155, y, 4005, 4007, 0, GumpButtonType::Page, i as i32 + 1);
            y += 25;
        }

        for (i, (_, entries)) in POLY_CATEGORIES.iter().enumerate() {
            b.add_page(i as i32 + 1);

            for (c, entry) in entries.iter().enumerate() {
                let (y_offset, x_offset) = ((c / 3) as i32, (c % 3) as i32);
                let x = 198 + x_offset * 129;
                let y = 38 + y_offset * 67;

                b.add_html_localized(x, y, 100, 18, entry.loc_number, false, false);
                b.add_item(x + 20, y + 25, entry.art_id, 0);
                b.add_radio(x, y + 20, 210, 211, false, ((c as i32) << 8) + i as i32);
            }
        }
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        if info.button_id != 1 || info.switches.is_empty() {
            return;
        }

        // Math.DivRem(switch, 256, out cat)
        let (ent, cat) = (info.switches[0] / 256, info.switches[0] % 256);

        if cat < 0 || cat as usize >= POLY_CATEGORIES.len() {
            return;
        }

        let entries = &POLY_CATEGORIES[cat as usize].1;

        if ent >= 0
            && (ent as usize) < entries.len()
            && let Some(from) = mobile_of(w, sender)
        {
            cast_polymorph(w, from, self.scroll, entries[ent as usize].body_id);
        }
    }
}

pub struct NewPolymorphGump {
    scroll: Option<ItemId>,
}

/// `new NewPolymorphGump(scroll)`.
pub fn new_new_polymorph(scroll: Option<ItemId>) -> StaticGumpObj<NewPolymorphGump> {
    StaticGumpObj::new(0, 0, NewPolymorphGump { scroll })
}

impl StaticGumpLogic for NewPolymorphGump {
    const TYPE_NAME: &'static str = "Server.Gumps.NewPolymorphGump";

    fn build_layout(&self, b: &mut StaticGumpBuilder) {
        b.add_page(0);

        b.add_background(0, 0, 520, 404, 0x13BE);
        b.add_image_tiled(10, 10, 500, 20, 0xA40);
        b.add_image_tiled(10, 40, 500, 324, 0xA40);
        b.add_image_tiled(10, 374, 500, 20, 0xA40);
        b.add_alpha_region(10, 10, 500, 384);

        b.add_html_localized_color(14, 12, 500, 20, 1015234, 0x7FFF, false, false); // <center>Polymorph Selection Menu</center>

        b.add_button(10, 374, 0xFB1, 0xFB2, 0);
        b.add_html_localized_color(45, 376, 450, 20, 1060051, 0x7FFF, false, false); // CANCEL

        for (i, entry) in NEW_POLY_ENTRIES.iter().enumerate() {
            let i = i as i32;
            let (page, pos) = (i / 10 + 1, i % 10);

            if pos == 0 {
                if page > 1 {
                    b.add_button_type(400, 374, 0xFA5, 0xFA7, 0, GumpButtonType::Page, page);
                    b.add_html_localized_color(440, 376, 60, 20, 1043353, 0x7FFF, false, false); // Next
                }

                b.add_page(page);

                if page > 1 {
                    b.add_button_type(300, 374, 0xFAE, 0xFB0, 0, GumpButtonType::Page, 1);
                    b.add_html_localized_color(340, 376, 60, 20, 1011393, 0x7FFF, false, false); // Back
                }
            }

            let (y_offset, x_offset) = (pos / 2, pos % 2);
            let x = x_offset * 250 + 14;
            let y = y_offset * 64 + 44;

            b.add_image_tiled_button(
                x,
                y,
                0x918,
                0x919,
                i + 1,
                GumpButtonType::Reply,
                0,
                entry.art_id,
                0x0,
                entry.x,
                entry.y,
                -1,
            );
            b.add_html_localized_color(x + 84, y, 250, 60, entry.loc_number, 0x7FFF, false, false);
        }
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let idx = info.button_id - 1;

        if idx < 0 || idx as usize >= NEW_POLY_ENTRIES.len() {
            return;
        }

        if let Some(from) = mobile_of(w, sender) {
            cast_polymorph(w, from, self.scroll, NEW_POLY_ENTRIES[idx as usize].body_id);
        }
    }
}

// ---- SelectAddonDirectionGump ----

/// What a `SelectAddonDirectionGump<T>` subclass supplies.
pub trait SelectAddonDirection: 'static {
    const TYPE_NAME: &'static str;
    /// `SelectionNumber`.
    const SELECTION_NUMBER: i32;
}

/// `SelectAddonDirectionGump<T>`: the choice is South or East of an `IDirectionAddonDeed`.
pub struct SelectAddonDirectionGump<S: SelectAddonDirection> {
    deed: ItemId,
    _kind: std::marker::PhantomData<S>,
}

/// `new T(deed)`.
pub fn new_select_addon_direction<S: SelectAddonDirection>(
    deed: ItemId,
) -> StaticGumpObj<SelectAddonDirectionGump<S>> {
    StaticGumpObj::new(
        60,
        63,
        SelectAddonDirectionGump {
            deed,
            _kind: std::marker::PhantomData,
        },
    )
}

impl<S: SelectAddonDirection> StaticGumpLogic for SelectAddonDirectionGump<S> {
    const TYPE_NAME: &'static str = S::TYPE_NAME;

    fn build_layout(&self, b: &mut StaticGumpBuilder) {
        b.add_page(0);

        b.add_background(0, 0, 273, 324, 0x13BE);
        b.add_image_tiled(10, 10, 253, 20, 0xA40);
        b.add_image_tiled(10, 40, 253, 244, 0xA40);
        b.add_image_tiled(10, 294, 253, 20, 0xA40);
        b.add_alpha_region(10, 10, 253, 304);

        b.add_button(10, 294, 0xFB1, 0xFB2, 0);

        b.add_html_localized_color(45, 296, 450, 20, 1060051, 0x7FFF, false, false); // CANCEL
        b.add_html_localized_color(14, 12, 273, 20, S::SELECTION_NUMBER, 0x7FFF, false, false);

        b.add_page(1);

        b.add_button(19, 49, 0x845, 0x846, 1);
        b.add_html_localized_color(44, 47, 213, 20, 1075386, 0x7FFF, false, false); // South
        b.add_button(19, 73, 0x845, 0x846, 2);
        b.add_html_localized_color(44, 71, 213, 20, 1075387, 0x7FFF, false, false); // East
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        if muo_server::item::ops::is_deleted(w, self.deed) || info.button_id == 0 {
            return;
        }
        let Some(from) = mobile_of(w, sender) else {
            return;
        };
        // `_deed.East = ButtonID != 1; _deed.SendTarget(from)`.
        if let Some(f) = hooks(w).and_then(|h| h.addon_deed_choose) {
            f(w, self.deed, info.button_id != 1, from);
        }
    }
}

// ---- TithingGump ----

// TODO: What's the maximum?
const MAX_TITHE_AMOUNT: i32 = 100000;

pub struct TithingGump {
    from: MobileId,
    offer: i32,
    total_gold: i32,
}

/// `new TithingGump(from)` with the offer a response left (`BuildStrings` clamped it to what is held).
pub fn new_tithing(w: &World, from: MobileId, offer: i32) -> StaticGumpObj<TithingGump> {
    let total_gold = muo_server::mobile::part1::props_a::total_gold(w, from);
    StaticGumpObj::new(
        160,
        40,
        TithingGump {
            from,
            offer: offer.clamp(0, total_gold.max(0)),
            total_gold,
        },
    )
}

impl StaticGumpLogic for TithingGump {
    const TYPE_NAME: &'static str = "Server.Gumps.TithingGump";

    fn build_layout(&self, b: &mut StaticGumpBuilder) {
        b.add_page(0);

        b.add_image(30, 30, 102, 0);

        // May your wealth bring blessings to those in need, if tithed upon this most sacred site.
        b.add_html_localized_color(95, 100, 120, 100, 1060198, 0, false, false);

        b.add_html_localized(57, 274, 50, 20, 3000311, false, false); // Gold:
        b.add_label_placeholder(87, 274, 53, "goldOffer");

        b.add_html_localized(137, 274, 50, 20, 1079251, false, false); // Tithe:
        b.add_label_placeholder(172, 274, 53, "titheOffer");

        b.add_button(105, 230, 5220, 5220, 2);
        b.add_button(113, 230, 5222, 5222, 2);
        b.add_label(108, 228, 0, "<");
        b.add_label(112, 228, 0, "<");

        b.add_button(127, 230, 5223, 5223, 1);
        b.add_label(131, 228, 0, "<");

        b.add_button(147, 230, 5224, 5224, 3);
        b.add_label(153, 228, 0, ">");

        b.add_button(168, 230, 5220, 5220, 4);
        b.add_button(176, 230, 5222, 5222, 4);
        b.add_label(172, 228, 0, ">");
        b.add_label(176, 228, 0, ">");

        b.add_button(217, 272, 4023, 4024, 5);
    }

    fn build_strings(&self, strings: &mut GumpStrings) {
        strings.set_string_slot("goldOffer", &n0(i64::from(self.total_gold - self.offer)));
        strings.set_string_slot("titheOffer", &n0(i64::from(self.offer)));
    }

    fn on_response(&mut self, w: &mut World, _sender: NetStateId, info: &RelayInfo) {
        let from = self.from;
        match info.button_id {
            0 => {
                // You have decided to tithe no gold to the shrine.
                muo_server::mobile::part2::messages::local_overhead_message_number(
                    w,
                    from,
                    MessageType::REGULAR,
                    0x3B2,
                    1060193,
                    "",
                );
            }
            1..=4 => {
                let offer = match info.button_id {
                    1 => self.offer - 100,
                    2 => 0,
                    3 => self.offer + 100,
                    _ => muo_server::mobile::part1::props_a::total_gold(w, from),
                };
                send(w, from, new_tithing(w, from, offer));
            }
            5 => {
                let total_gold = muo_server::mobile::part1::props_a::total_gold(w, from);
                let mut offer = self.offer.clamp(0, total_gold.max(0));

                let points = hooks(w)
                    .and_then(|h| h.tithing_points)
                    .map_or(0, |f| f(w, from));
                if points + offer > MAX_TITHE_AMOUNT {
                    offer = MAX_TITHE_AMOUNT - points;
                }
                self.offer = offer;

                if offer <= 0 {
                    // You have decided to tithe no gold to the shrine.
                    muo_server::mobile::part2::messages::local_overhead_message_number(
                        w,
                        from,
                        MessageType::REGULAR,
                        0x3B2,
                        1060193,
                        "",
                    );
                    return;
                }

                let pack = muo_server::mobile::part1::props_b::backpack(w, from);

                // TODO: At some point this was changed on OSI to only work from bank/account funds
                let consumed = pack.is_some_and(|p| {
                    muo_server::item::container::consume_total(
                        w,
                        p,
                        &muo_content::items::gold::GOLD,
                        offer,
                        true,
                        None,
                    )
                });
                if consumed {
                    // You tithe gold to the shrine as a sign of devotion.
                    muo_server::mobile::part2::messages::local_overhead_message_number(
                        w,
                        from,
                        MessageType::REGULAR,
                        0x3B2,
                        1060195,
                        "",
                    );
                    if let Some(f) = hooks(w).and_then(|h| h.add_tithing_points) {
                        f(w, from, offer);
                    }

                    muo_content::mx::play_sound(w, from, 0x243);
                    muo_content::mx::play_sound(w, from, 0x2E6);
                } else {
                    // You do not have enough gold to tithe that amount!
                    muo_server::mobile::part2::messages::local_overhead_message_number(
                        w,
                        from,
                        MessageType::REGULAR,
                        0x3B2,
                        1060194,
                        "",
                    );
                }
            }
            _ => {}
        }
    }
}

// ---- ToTAdminGump ----

const TOT_INFO: [&str; 4] = [
    // Opening Message
    "<center>Treasures of Tokuno Admin</center><br>\
-Use the gems to switch eras<br>\
-Drop era and Reward era can be changed separately<br>\
-Drop era can be deactivated, Reward era is always activated",
    // Treasures of Tokuno 1 message
    "<center>Treasures of Tokuno 1</center><br>\
-10 charge Bleach Pigment can drop as a Lesser Artifact<br>\
-50 charge Neon Pigments available as a reward<br>",
    // Treasures of Tokuno 2 message
    "<center>Treasures of Tokuno 2</center><br>\
-30 types of 1 charge Metallic Pigments drop as Lesser Artifacts<br>\
-1 charge Bleach Pigment can drop as a Lesser Artifact<br>\
-10 charge Greater Metallic Pigments available as a reward",
    // Treasures of Tokuno 3 message
    "<center>Treasures of Tokuno 3</center><br>\
-10 types of 1 charge Fresh Pigments drop as Lesser Artifacts<br>\
-1 charge Bleach Pigment can drop as a Lesser Artifact<br>\
-Leurocian's Mempo Of Fortune can drop as a Lesser Artifact",
];

/// `Enum.GetValues(typeof(TreasuresOfTokunoEra)).Length - 1`.
const TOT_ERAS: i32 = 3;

pub struct ToTAdminGump;

/// `new ToTAdminGump()` with `TreasuresOfTokuno.DropEra` and `RewardEra` (0 is `None`).
pub fn new_tot_admin(drop_era: i32, reward_era: i32) -> LegacyGumpObj<ToTAdminGump> {
    let mut g = LegacyGumpObj::new(30, 50, ToTAdminGump);
    let l = &mut g.layout;
    l.closable = true;
    l.disposable = true;
    l.draggable = true;
    l.resizable = false;

    l.add_page(0);
    l.add_background(0, 0, 320, 75 + TOT_ERAS * 25, 9200);
    l.add_image_tiled(25, 18, 270, 10, 9267);
    l.add_label(75, 5, 54, "Treasures of Tokuno Admin");
    l.add_label(10, 25, 54, "ToT Era");
    l.add_label(90, 25, 54, "Drop Era");
    l.add_label(195, 25, 54, "Reward Era");
    l.add_label(287, 25, 54, "Info");

    l.add_background(320, 0, 200, 150, 9200);
    l.add_image_tiled(325, 5, 190, 140, 2624);
    l.add_alpha_region(325, 5, 190, 140);

    // SetupToTEras
    let is_activated = drop_era != 0;
    let button = if is_activated { 2361 } else { 2360 };
    l.add_button(75, 50, button, button, 1);
    l.add_label(
        90,
        45,
        if is_activated { 167 } else { 137 },
        if is_activated {
            "Activated"
        } else {
            "Deactivated"
        },
    );

    for i in 0..TOT_ERAS {
        let y_offset = i * 25;

        let is_this_drop_era = drop_era - 1 == i;
        let is_this_reward_era = reward_era - 1 == i;
        let drop_button_id = if is_this_drop_era { 2361 } else { 2360 };
        let reward_button_id = if is_this_reward_era { 2361 } else { 2360 };

        l.add_label(10, 70 + y_offset, 2100, &format!("ToT {}", i + 1));
        l.add_button(75, 75 + y_offset, drop_button_id, drop_button_id, 2 + i * 2);
        l.add_label(
            90,
            70 + y_offset,
            if is_this_drop_era { 167 } else { 137 },
            if is_this_drop_era {
                "Active"
            } else {
                "Inactive"
            },
        );
        l.add_button(
            180,
            75 + y_offset,
            reward_button_id,
            reward_button_id,
            2 + i * 2 + 1,
        );
        l.add_label(
            195,
            70 + y_offset,
            if is_this_reward_era { 167 } else { 137 },
            if is_this_reward_era {
                "Active"
            } else {
                "Inactive"
            },
        );

        l.add_button_type(
            285,
            70 + y_offset,
            4005,
            4006,
            i,
            GumpButtonType::Page,
            2 + i,
        );
    }

    for (i, info) in TOT_INFO.iter().enumerate() {
        l.add_page(1 + i as i32);
        l.add_html(330, 10, 180, 130, info, false, true);
    }
    g
}

impl LegacyGumpLogic for ToTAdminGump {
    const TYPE_NAME: &'static str = "Server.Gumps.ToTAdminGump";
    fn singleton(&self) -> bool {
        true
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let button = info.button_id;
        let Some(from) = mobile_of(w, sender) else {
            return;
        };
        let (set_drop, set_reward) = match hooks(w) {
            Some(h) => (h.tot_set_drop_era, h.tot_set_reward_era),
            None => (None, None),
        };

        if button == 1 {
            if let Some(f) = set_drop {
                f(w, 0);
            }
            muo_content::mx::send_message(
                w,
                from,
                "Treasures of Tokuno Drops have been deactivated",
            );
        } else if button >= 2 {
            if button % 2 == 0 {
                let selected = button / 2;
                if let Some(f) = set_drop {
                    f(w, selected);
                }
                muo_content::mx::send_message(
                    w,
                    from,
                    &format!("Treasures of Tokuno {selected} Drops have been enabled"),
                );
            } else {
                let selected = (button - 1) / 2;
                if let Some(f) = set_reward {
                    f(w, selected);
                }
                muo_content::mx::send_message(
                    w,
                    from,
                    &format!("Treasures of Tokuno {selected} Rewards have been enabled"),
                );
            }
        }
    }
}

/// `IDirectionAddonDeed`: `deed.East = east; deed.SendTarget(from)`.
pub fn select_addon_direction_configure(w: &mut World) {
    w.resources
        .get_mut::<super::hooks::MzHooks>()
        .addon_deed_choose = Some(muo_content::items::addon::choose_direction_of);
}

/// `PlayerMobile+ServerLockdownNoticeGump : StaticNoticeGump`.
struct ServerLockdownNoticeGump(String);

impl super::warning::StaticNotice for ServerLockdownNoticeGump {
    const TYPE_NAME: &'static str = "Server.Mobiles.PlayerMobile+ServerLockdownNoticeGump";
    const WIDTH: i32 = 300;
    const HEIGHT: i32 = 140;

    fn content(&self) -> Option<String> {
        Some(self.0.clone())
    }
}

/// The gumps `PlayerMobile` sends: the lockdown notice, the young death notice, and the `ResurrectGump` it closes.
pub fn player_mobile_gumps_configure(w: &mut World) {
    use muo_content::player_mobile::hooks::SystemHooks;
    let h = w.resources.get_mut::<SystemHooks>();
    h.lockdown_notice_gump = Some(|w, m, notice| {
        let g =
            super::warning::new_static_notice(ServerLockdownNoticeGump(notice.to_owned()), None);
        send(w, m, g);
    });
    h.young_death_notice_gump = Some(|w, m| {
        if w.mobile(m).is_some_and(|x| x.net_state.is_some()) {
            send(w, m, new_young_death_notice());
        }
    });
    // `Alive ? gumps.Close<ResurrectGump>() : gumps.Has<ResurrectGump>()`
    h.resurrect_gump_close = Some(|w, m, alive| {
        let Some(ns) = w.mobile(m).and_then(|x| x.net_state) else {
            return false;
        };
        if alive {
            muo_content::gumps::close_gump::<super::resurrect::ResurrectGump>(w, ns);
            false
        } else {
            muo_content::gumps::has_gump::<super::resurrect::ResurrectGump>(w, ns)
        }
    });
}

/// `ToTAdminGump.Configure`.
pub fn tot_admin_configure(w: &mut World) {
    CommandSystem::register(
        w,
        "ToTAdmin",
        AccessLevel::Administrator,
        &[],
        Rc::new(tot_admin_on_command),
    );
}

/// `ToTAdmin_OnCommand`.
fn tot_admin_on_command(w: &mut World, e: &mut CommandEventArgs) {
    let (drop_era, reward_era) = match hooks(w) {
        Some(h) => (h.tot_drop_era, h.tot_reward_era),
        None => (None, None),
    };
    let drop_era = drop_era.map_or(0, |f| f(w));
    let reward_era = reward_era.map_or(0, |f| f(w));
    send(w, e.mobile, new_tot_admin(drop_era, reward_era));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::m_z::tests::{sections, strings_of};
    use muo_primitives::Serial;

    #[test]
    fn tithing_shows_the_gold_left_and_the_offer_with_thousands_separators() {
        let from = MobileId(Serial::new(1));
        let mut g = StaticGumpObj::new(
            160,
            40,
            TithingGump {
                from,
                offer: 500,
                total_gold: 12345,
            },
        );
        let (layout, strings) = sections(&mut g);
        let s = strings_of(&strings);
        assert!(
            s.contains(&"11,845".to_owned()) && s.contains(&"500".to_owned()),
            "{s:?}"
        );
        assert!(
            layout.contains("{ button 217 272 4023 4024 1 0 5 }"),
            "{layout}"
        );
        assert!(
            layout.contains("{ button 105 230 5220 5220 1 0 2 }{ button 113 230 5222 5222 1 0 2 }"),
            "{layout}"
        );
    }
}
