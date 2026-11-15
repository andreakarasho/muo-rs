//! `DisguiseKit`, `DisguisePersistence` and the specialized craft books (`GlassblowingBook`, `MasonryBook`,
//! `SandMiningBook`, `StoneMiningBook`).
//!
//! `DisguisePersistence` is the `GenericPersistence` named "Disguises" (priority 10) over [`DisguiseTimers`]; the
//! `PlayerMobile` hooks `disguise_start_timer`/`disguise_stop_timer` (login, logout) and the spells' `is_disguised`
//! and `disguise_stop_timer` are set in [`configure`].

use indexmap::IndexMap;
use muo_content::gumps::base::{DynamicGumpLogic, DynamicGumpObj, RelayInfo};
use muo_content::gumps::builder::DynamicGumpBuilder;
use muo_content::gumps::layout::GumpButtonType;
use muo_content::player_mobile::{self, NpcGuild, PlayerFlag};
use muo_io::{GenericReader, GenericWriter, Result};
use muo_primitives::TimeSpan;
use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, base, deps, ops};
use muo_server::net_state::NetStateId;
use muo_server::persistence::{GenericPayload, GenericPersistence};
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::skills::SkillName;
use muo_server::timer::{self, TimerHandle};
use muo_server::type_registry::{Phase, TypeRegistry};

use super::{read_version, write_version};

// ---- DisguisePersistence ----

/// `DisguisePersistence.Timers`.
#[derive(Default)]
pub struct DisguiseTimers {
    timers: IndexMap<MobileId, TimerHandle>,
}

fn timers(w: &mut World) -> &mut DisguiseTimers {
    w.resources.get_mut::<DisguiseTimers>()
}

/// `DisguisePersistence.IsDisguised(m)`.
pub fn is_disguised(w: &World, m: MobileId) -> bool {
    w.resources
        .get::<DisguiseTimers>()
        .is_some_and(|t| t.timers.contains_key(&m))
}

/// `DisguisePersistence.CreateTimer(m, delay)`: a timer that is not started until `StartTimer`.
pub fn create_timer(w: &mut World, m: MobileId, delay: TimeSpan) {
    if w.mobile(m).is_some() && !is_disguised(w, m) {
        // InternalTimer.OnTick
        let h = timer::create(w, delay, TimeSpan::ZERO, 1, move |w| {
            muo_server::mobile::part1::props_b::set_name_mod(w, m, None);
            if player_mobile::data(w, m).is_some() {
                player_mobile::set_hair_mods(w, m, -1, -1);
            }
            remove_timer(w, m);
        });
        timers(w).timers.insert(m, h);
    }
}

/// `DisguisePersistence.StartTimer(m)`.
pub fn start_timer(w: &mut World, m: MobileId) {
    if let Some(h) = timers(w).timers.get(&m).copied() {
        timer::begin(w, h);
    }
}

/// `DisguisePersistence.StopTimer(m)`: the remaining time becomes the delay.
pub fn stop_timer(w: &mut World, m: MobileId) {
    let Some(h) = timers(w).timers.get(&m).copied() else {
        return;
    };
    let remaining = (h.next(w) - w.now).max(TimeSpan::ZERO);
    timer::set_delay(w, h, remaining);
    timer::stop(w, h);
}

/// `DisguisePersistence.RemoveTimer(m)` (also the player death and deleted events).
pub fn remove_timer(w: &mut World, m: MobileId) {
    if let Some(h) = timers(w).timers.shift_remove(&m) {
        timer::release(w, h);
    }
}

/// `DisguisePersistence.TimeRemaining(m)`.
pub fn time_remaining(w: &World, m: MobileId) -> TimeSpan {
    w.resources
        .get::<DisguiseTimers>()
        .and_then(|t| t.timers.get(&m))
        .map_or(TimeSpan::ZERO, |h| h.next(w) - w.now)
}

/// `DisguisePersistence`: the payload of the "Disguises" persistence.
pub struct DisguisePayload;

impl GenericPayload for DisguisePayload {
    fn name(&self) -> &str {
        "Disguises"
    }

    fn serialize(&mut self, w: &World, writer: &mut dyn GenericWriter) {
        let empty = DisguiseTimers::default();
        let t = w.resources.get::<DisguiseTimers>().unwrap_or(&empty);
        writer.write_encoded_int(t.timers.len() as i32);
        for (m, h) in &t.timers {
            writer.write_mobile(w, Some(*m));
            writer.write_time_span_ticks((h.next(w) - w.now).ticks());
            writer.write_opt_string(w.mobile(*m).and_then(|x| x.name_mod.as_deref()));
        }
    }

    fn deserialize(&mut self, w: &mut World, reader: &mut dyn GenericReader) -> Result<()> {
        let count = reader.read_encoded_int()?;
        for _ in 0..count.max(0) {
            let m = reader.read_mobile(w)?;
            let delay = TimeSpan::from_ticks(reader.read_time_span_ticks()?);
            let name = reader.read_string()?;
            if let Some(m) = m {
                create_timer(w, m, delay);
                muo_server::mobile::part1::props_b::set_name_mod(w, m, name);
            }
        }
        Ok(())
    }
}

// ---- DisguiseKit ----

fn kit_valid_use(w: &mut World, kit: ItemId, from: MobileId) -> bool {
    let pack = w.mobile(from).and_then(|m| m.backpack);
    let pm =
        player_mobile::data(w, from).map(|d| (d.npc_guild, d.npc_guild == NpcGuild::ThievesGuild));
    let kills = w.mobile(from).map_or(0, |m| m.kills);
    let suspend = w
        .resources
        .get::<muo_content::skills::stealing::StealingSettings>()
        .copied()
        .unwrap_or_default()
        .suspend_on_murder;
    if !ops::is_child_of(w, kit, pack.map(EntityId::Item)) {
        deps::send_localized_message(w, from, 1042001);
    } else if !pm.is_some_and(|p| p.1) {
        deps::send_localized_message(w, from, 501702);
    } else if suspend && kills > 0 {
        deps::send_localized_message(w, from, 501703); // could be perma-red: intentional
    } else if !muo_server::mobile::part1::combat::can_begin_action(
        w,
        from,
        muo_spells::fifth::incognito::KEY,
    ) {
        deps::send_localized_message(w, from, 501704);
    } else if muo_content::spells::hooks::hooks(w)
        .and_then(|h| h.sigil_exists_on)
        .is_some_and(|f| f(w, from))
    {
        deps::send_localized_message(w, from, 1010465); // You cannot disguise yourself while holding a sigil
    } else if muo_content::spells::helper::under_transformation(w, from) {
        deps::send_localized_message(w, from, 1061634);
    } else if w
        .mobile(from)
        .is_some_and(|m| m.body_mod.0 == 183 || m.body_mod.0 == 184)
    {
        deps::send_localized_message(w, from, 1040002);
    } else if !muo_server::mobile::part1::combat::can_begin_action(
        w,
        from,
        muo_spells::seventh::polymorph::KEY,
    ) || muo_server::mobile::part1::props_b::is_body_mod(w, from)
    {
        deps::send_localized_message(w, from, 501705);
    } else {
        return true;
    }
    false
}

fn kit_double_click(w: &mut World, id: ItemId, from: MobileId) {
    if kit_valid_use(w, id, from) {
        display_gump(w, from, id, true, false);
    }
}

/// `DisguiseGump.DisplayTo(from, kit, startAtHair, used)`.
fn display_gump(w: &mut World, from: MobileId, kit: ItemId, start_at_hair: bool, used: bool) {
    if w.mobile(from).and_then(|m| m.net_state).is_none() || w.item(kit).is_none_or(|i| i.deleted())
    {
        return;
    }
    let female = w
        .mobile(from)
        .is_some_and(|m| m.female || m.cur_body().is_female());
    let g = DynamicGumpObj::new(
        50,
        50,
        DisguiseGump {
            from,
            kit,
            start_at_hair,
            used,
            female,
        },
    );
    super::hue_pickers::send(w, from, g);
}

/// `DisguiseGump.DisguiseEntry`: `(itemID, gumpID, offsetX, offsetY, name)`.
type Entry = Option<(i32, i32, i32, i32, i32)>;

const HAIR_ENTRIES: [Entry; 10] = [
    Some((8251, 50700, 0, 5, 1011052)),  // Short
    Some((8261, 60710, 0, 3, 1011047)),  // Pageboy
    Some((8252, 60708, 0, -5, 1011053)), // Long
    Some((8264, 60901, 0, 5, 1011048)),  // Receding
    Some((8253, 60702, 0, -5, 1011054)), // Ponytail
    Some((8265, 60707, 0, -5, 1011049)), // 2-tails
    Some((8260, 50703, 0, 5, 1011055)),  // Mohawk
    Some((8266, 60713, 0, 10, 1011050)), // Topknot
    None,
    Some((0, 0, 0, 0, 1011051)), // None
];

const BEARD_ENTRIES: [Entry; 8] = [
    Some((8269, 50906, 0, 0, 1011401)),   // Vandyke
    Some((8257, 50808, 0, -2, 1011062)),  // Mustache
    Some((8255, 50802, 0, 0, 1011060)),   // Short beard
    Some((8268, 50905, 0, -10, 1011061)), // Long beard
    Some((8267, 50904, 0, 0, 1011060)),   // Short beard
    Some((8254, 50801, 0, -10, 1011061)), // Long beard
    None,
    Some((0, 0, 0, 0, 1011051)), // None
];

/// `DisguiseGump`.
pub struct DisguiseGump {
    from: MobileId,
    kit: ItemId,
    start_at_hair: bool,
    used: bool,
    /// `_from.Female || _from.Body.IsFemale` when the gump was made.
    female: bool,
}

fn draw_entries(
    b: &mut DynamicGumpBuilder,
    index: i32,
    page: i32,
    next_page: i32,
    entries: &[Entry],
    next_number: i32,
) {
    b.add_page(page);
    if next_page != -1 {
        b.add_button_type(
            155,
            320,
            250 + index * 2,
            251 + index * 2,
            0,
            GumpButtonType::Page,
            next_page,
        );
        b.add_html_localized(180, 320, 150, 35, next_number, false, false);
    }
    for (i, e) in entries.iter().enumerate() {
        let Some((_, gump_id, ox, oy, number)) = *e else {
            continue;
        };
        let x = (i as i32 % 2) * 205;
        let y = (i as i32 / 2) * 55;
        if gump_id != 0 {
            b.add_background(220 + x, 60 + y, 50, 50, 2620);
            b.add_image(153 + x + ox, 15 + y + oy, gump_id, 0);
        }
        b.add_html_localized(140 + x, 72 + y, 80, 35, number, false, false);
        b.add_radio(118 + x, 73 + y, 208, 209, false, i as i32 * 2 + index);
    }
}

impl DynamicGumpLogic for DisguiseGump {
    const TYPE_NAME: &'static str = "Server.Items.DisguiseGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        b.add_page(0);
        b.add_background(100, 10, 400, 385, 2600);
        b.add_html_localized(100, 25, 400, 35, 1011045, false, false); // <center>THIEF DISGUISE KIT</center>
        b.add_button(140, 353, 4005, 4007, 0);
        b.add_html_localized(172, 355, 90, 35, 1011036, false, false); // OKAY
        b.add_button(257, 353, 4005, 4007, 1);
        b.add_html_localized(289, 355, 90, 35, 1011046, false, false); // APPLY
        if self.female {
            draw_entries(b, 0, 1, -1, &HAIR_ENTRIES, -1);
        } else if self.start_at_hair {
            draw_entries(b, 0, 1, 2, &HAIR_ENTRIES, 1011056);
            draw_entries(b, 1, 2, 1, &BEARD_ENTRIES, 1011059);
        } else {
            draw_entries(b, 1, 1, 2, &BEARD_ENTRIES, 1011059);
            draw_entries(b, 0, 2, 1, &HAIR_ENTRIES, 1011056);
        }
    }

    fn on_response(&mut self, w: &mut World, _sender: NetStateId, info: &RelayInfo) {
        let from = self.from;
        if info.button_id == 0 {
            if self.used {
                deps::send_localized_message(w, from, 501706); // Disguises wear off after 2 hours.
            } else {
                deps::send_localized_message(w, from, 501707); // You're looking good.
            }
            return;
        }
        let Some(&switched) = info.switches.first() else {
            return;
        };
        let (ty, index) = (switched % 2, switched / 2);
        let hair = ty == 0;
        let entries: &[Entry] = if hair { &HAIR_ENTRIES } else { &BEARD_ENTRIES };
        if index >= 0 && (index as usize) < entries.len() {
            let Some((item_id, ..)) = entries[index as usize] else {
                return;
            };
            if !kit_valid_use(w, self.kit, from) {
                return;
            }
            let female_now = w
                .mobile(from)
                .is_some_and(|m| m.female || m.cur_body().is_female());
            if !hair && female_now {
                return;
            }
            let gender = if w.mobile(from).is_some_and(|m| m.female) {
                "female"
            } else {
                "male"
            };
            let name = muo_content::misc::small::random_name(w, gender);
            muo_server::mobile::part1::props_b::set_name_mod(w, from, Some(name));
            if player_mobile::data(w, from).is_some() {
                if hair {
                    player_mobile::set_hair_mods(w, from, item_id, -2);
                } else {
                    player_mobile::set_hair_mods(w, from, -2, item_id);
                }
            }
            display_gump(w, from, self.kit, hair, true);
            remove_timer(w, from);
            create_timer(w, from, TimeSpan::from_hours(2));
            start_timer(w, from);
        }
    }
}

/// `Server.Items.DisguiseKit`.
pub static DISGUISE_KIT: ItemType = ItemType {
    full_name: "Server.Items.DisguiseKit",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| crate::simple::from_template(w, &DISGUISE_KIT, "Server.Items.DisguiseKit")),
    default_weight: crate::class_props::default_weight,
    label_number: crate::class_props::label_number, // a disguise kit
    on_double_click: kit_double_click,
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        write_version(out, false);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        read_version(r, false)?;
        Ok(())
    },
    ..ITEM_BASE
};

/// `Server.Items.DisguisePersistenceTimers` (`[TypeAlias("Server.Items.DisguisePersistance")]`): the legacy item
/// that only reads its record into the persistence and deletes itself.
pub static DISGUISE_PERSISTENCE_TIMERS: ItemType = ItemType {
    full_name: "Server.Items.DisguisePersistenceTimers",
    parent: Some(&ITEM_BASE),
    construct: None,
    serialize: |_, _, _| {},
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        let count = r.read_i32()?;
        for _ in 0..count.max(0) {
            let m = r.read_mobile(w)?;
            let delay = TimeSpan::from_ticks(r.read_time_span_ticks()?);
            let name = r.read_string()?;
            if let Some(m) = m {
                create_timer(w, m, delay);
                muo_server::mobile::part1::props_b::set_name_mod(w, m, name);
            }
        }
        deps::delay_delete(w, id);
        Ok(())
    },
    ..ITEM_BASE
};

// ---- Specialized books ----

/// One of the four books that teach a crafting specialty to a grandmaster.
struct BookClass {
    graphic: i32,
    label: i32,
    skill: SkillName,
    /// The refusal when the skill is below 100 (or the reader is not a player).
    too_low: i32,
    flag: PlayerFlag,
    learned: i32,
}

macro_rules! books {
    ($($name:ident = $full:literal, $class:ident { $($f:ident : $v:expr),* $(,)? };)*) => {$(
        static $class: BookClass = BookClass { $($f: $v,)* };
        #[doc = concat!("`", $full, "`.")]
        pub static $name: ItemType = ItemType {
            full_name: $full,
            parent: Some(&ITEM_BASE),
            construct: Some(|w| ops::new_item_with_id(w, &$name, $class.graphic)),
            default_weight: |_, _| 1.0,
            label_number: |_, _| $class.label,
            on_double_click: |w, id, from| book_double_click(w, id, from, &$class),
            serialize: |w, id, out| {
                (ITEM_BASE.serialize)(w, id, out);
                write_version(out, false);
            },
            deserialize: |w, id, r| {
                (ITEM_BASE.deserialize)(w, id, r)?;
                read_version(r, false)?;
                Ok(())
            },
            ..ITEM_BASE
        };
    )*};
}

books! {
    // Crafting glass with Glassblowing
    GLASSBLOWING_BOOK = "Server.Items.GlassblowingBook", GLASSBLOWING { graphic: 0xFF4, label: 1153528, skill: SkillName::Alchemy,
        too_low: 1080042, flag: PlayerFlag::GLASSBLOWING, learned: 1111702 };
    // Making valuables with Stonecrafting
    MASONRY_BOOK = "Server.Items.MasonryBook", MASONRY { graphic: 0xFBE, label: 1153527, skill: SkillName::Carpentry,
        too_low: 1080043, flag: PlayerFlag::MASONRY, learned: 1080044 };
    // Find Glass-Quality Sand
    SAND_MINING_BOOK = "Server.Items.SandMiningBook", SAND_MINING { graphic: 0xFF4, label: 1153531, skill: SkillName::Mining,
        too_low: 1080041, flag: PlayerFlag::SAND_MINING, learned: 1080065 };
    // Mining For Quality Stone
    STONE_MINING_BOOK = "Server.Items.StoneMiningBook", STONE_MINING { graphic: 0xFBE, label: 1153530, skill: SkillName::Mining,
        too_low: 1080041, flag: PlayerFlag::STONE_MINING, learned: 1080045 };
}

fn book_double_click(w: &mut World, id: ItemId, from: MobileId, c: &BookClass) {
    let pack = w.mobile(from).and_then(|m| m.backpack);
    let is_player = player_mobile::data(w, from).is_some();
    if !ops::is_child_of(w, id, pack.map(EntityId::Item)) {
        deps::send_localized_message(w, from, 1042001); // That must be in your pack for you to use it.
    } else if !is_player || muo_server::skills::base(w, from, c.skill) < 100.0 {
        deps::send_localized_message(w, from, c.too_low);
    } else if player_mobile::data(w, from).is_some_and(|d| d.flag(c.flag)) {
        deps::send_localized_message(w, from, 1080066); // You have already learned this information.
    } else {
        if let Some(d) = player_mobile::data_mut(w, from) {
            d.set_flag(c.flag, true);
        }
        deps::send_localized_message(w, from, c.learned);
        base::delete(w, id);
    }
}

fn configure(w: &mut World) {
    let h = w
        .resources
        .get_mut::<muo_content::player_mobile::hooks::SystemHooks>();
    h.disguise_start_timer = Some(start_timer);
    h.disguise_stop_timer = Some(stop_timer);
    h.disguise_time_remaining = Some(time_remaining);
    let s = muo_content::spells::hooks::hooks_mut(w);
    s.disguise_stop_timer = Some(stop_timer);
    s.is_disguised = Some(is_disguised);
    muo_content::player_mobile::subscribe_death(w, remove_timer);
    muo_content::player_mobile::subscribe_deleted(w, remove_timer);
    w.persistence
        .register(10, Box::new(GenericPersistence::new(DisguisePayload)));
}

/// Registers the classes and the "Disguises" persistence.
pub fn register(types: &mut TypeRegistry) {
    types.register_item(
        &DISGUISE_PERSISTENCE_TIMERS,
        &["Server.Items.DisguisePersistance"],
    );
    for t in [
        &DISGUISE_KIT,
        &GLASSBLOWING_BOOK,
        &MASONRY_BOOK,
        &SAND_MINING_BOOK,
        &STONE_MINING_BOOK,
    ] {
        types.register_item(t, &[]);
    }
    types.register_phase_default(
        Phase::Configure,
        "Server.Items.DisguisePersistence.Configure",
        configure,
    );
}
