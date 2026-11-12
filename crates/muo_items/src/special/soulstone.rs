//! `SoulStone` and its gumps (`SelectSkillGump`, `ConfirmSkillGump`,
//! `ConfirmTransferGump`, `ConfirmRemovalGump`, `ErrorGump`), `SoulstoneFragment`, `BlueSoulstone`, `RedSoulstone`.
//!
//! `RedSoulstone`'s year label comes from the veteran reward system (`RewardHooks`, set by the rewards engine).

use muo_content::aos::skill_label;
use muo_content::gumps::base::{
    DynamicGumpLogic, DynamicGumpObj, RelayInfo, StaticGumpLogic, StaticGumpObj,
};
use muo_content::gumps::builder::{DynamicGumpBuilder, StaticGumpBuilder};
use muo_content::gumps::{GumpButtonType, close_gump, send_gump_to_mobile};
use muo_content::items::house_deps;
use muo_content::props::{Ctor, Kind, Param, Value, add_ctors};
use muo_content::utilities::types::{Numeric, PropType};
use muo_io::{GenericReader, GenericWriter};
use muo_primitives::{Point3D, TimeSpan, utility};
use muo_server::World;
use muo_server::effects::{self, EffectLayer, Targeted};
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, LootType, effect_item, ops};
use muo_server::light_type::LightType;
use muo_server::net_state::NetStateId;
use muo_server::property_list::{PropertyList, cliloc_arg};
use muo_server::skills::{self, SkillLock, SkillName};
use muo_server::type_registry::TypeRegistry;

use super::hooks::hooks;
use super::util::{account_username, msg, overhead};

/// The fields of `SoulStone` (`[SerializableField]` 0..6).
#[derive(Clone, Debug)]
pub struct SoulData {
    pub last_user_name: Option<String>,
    /// `SecureLevel` (an int enum).
    pub level: i32,
    pub active_item_id: i32,
    pub inactive_item_id: i32,
    pub account: Option<String>,
    pub skill: SkillName,
    pub skill_value: f64,
}

/// `SoulstoneFragment._usesRemaining`.
#[derive(Clone, Copy, Debug)]
pub struct FragmentData {
    pub uses_remaining: i32,
}

/// `RedSoulstone._isRewardItem`.
#[derive(Clone, Copy, Debug, Default)]
pub struct RedData {
    pub is_reward_item: bool,
}

fn data(w: &World, id: ItemId) -> Option<&SoulData> {
    w.item(id)?.ext.get::<SoulData>()
}

fn data_mut(w: &mut World, id: ItemId) -> Option<&mut SoulData> {
    w.item_mut(id)?.ext.get_mut::<SoulData>()
}

fn is_fragment(w: &World, id: ItemId) -> bool {
    w.item(id).is_some_and(|i| i.ty.is(&SOULSTONE_FRAGMENT))
}

/// `IsEmpty`.
pub fn is_empty(w: &World, id: ItemId) -> bool {
    data(w, id).is_none_or(|d| d.skill_value <= 0.0)
}

fn skill_value(w: &World, id: ItemId) -> f64 {
    data(w, id).map_or(0.0, |d| d.skill_value)
}

fn skill_of(w: &World, id: ItemId) -> SkillName {
    data(w, id).map_or(SkillName::Alchemy, |d| d.skill)
}

/// `SkillValue = value`: the change refreshes the properties and `ItemID`.
pub fn set_skill_value(w: &mut World, id: ItemId, value: f64) {
    let Some(d) = data_mut(w, id) else { return };
    if d.skill_value == value {
        return;
    }
    d.skill_value = value;
    let item_id = if value <= 0.0 {
        d.inactive_item_id
    } else {
        d.active_item_id
    };
    ops::invalidate_properties(w, id);
    if let Some(ty) = w.item(id).map(|i| i.ty) {
        (ty.set_item_id)(w, id, item_id);
    }
}

/// `LastUserName = value`.
pub fn set_last_user_name(w: &mut World, id: ItemId, value: Option<String>) {
    if let Some(d) = data_mut(w, id)
        && d.last_user_name != value
    {
        d.last_user_name = value;
        ops::invalidate_properties(w, id);
    }
}

/// `ActiveItemID = value`.
pub fn set_active_item_id(w: &mut World, id: ItemId, value: i32) {
    let empty = is_empty(w, id);
    if let Some(d) = data_mut(w, id) {
        d.active_item_id = value;
    }
    if !empty && let Some(ty) = w.item(id).map(|i| i.ty) {
        (ty.set_item_id)(w, id, value);
    }
}

/// `InactiveItemID = value`.
pub fn set_inactive_item_id(w: &mut World, id: ItemId, value: i32) {
    let empty = is_empty(w, id);
    if let Some(d) = data_mut(w, id) {
        d.inactive_item_id = value;
    }
    if empty && let Some(ty) = w.item(id).map(|i| i.ty) {
        (ty.set_item_id)(w, id, value);
    }
}

/// `SoulStone(account, inactiveItemID, activeItemID)`.
pub fn soulstone_new(
    w: &mut World,
    ty: &'static ItemType,
    account: Option<String>,
    inactive_item_id: i32,
    active_item_id: i32,
) -> ItemId {
    let id = ops::new_item_with_id(w, ty, inactive_item_id);
    ops::set_light(w, id, LightType::Circle300);
    ops::set_loot_type(w, id, LootType::Blessed);
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(SoulData {
            last_user_name: None,
            level: 0,
            active_item_id,
            inactive_item_id,
            account,
            skill: SkillName::Alchemy,
            skill_value: 0.0,
        });
    }
    id
}

/// `CheckCombat(m, time)`.
fn check_combat(w: &World, m: MobileId, time: TimeSpan) -> bool {
    w.mobile(m).is_some_and(|mm| {
        mm.aggressed
            .iter()
            .any(|info| w.now - info.last_combat_time < time)
    })
}

/// `CheckUse(from)`, with `SoulstoneFragment`'s override.
fn check_use(w: &mut World, id: ItemId, from: MobileId) -> bool {
    let ok = check_use_base(w, id, from);
    if ok && is_fragment(w, id) && fragment_uses(w, id) <= 0 {
        msg(w, from, 1070975); // That soulstone fragment has no more uses.
        return false;
    }
    ok
}

fn check_use_base(w: &mut World, id: ItemId, from: MobileId) -> bool {
    if ops::is_deleted(w, id) {
        return false;
    }
    let ty = w.item(id).map(|i| i.ty).expect("item");
    if !(ty.is_accessible_to)(w, id, from) {
        return false;
    }
    let item_map = w.item(id).and_then(|i| i.map);
    let from_map = w.mobile(from).and_then(|m| m.map);
    let loc = ops::get_world_location(w, id);
    if from_map != item_map || !muo_content::mx::in_range_point(w, from, loc, 2) {
        overhead(w, from, 0x3B2, 1019045); // I can't reach that.
        return false;
    }
    if let Some(account) = data(w, id).and_then(|d| d.account.clone())
        && account_username(w, from).as_deref() != Some(account.as_str())
    {
        // This is an Account Bound Soulstone, and your character is not bound to it.  You cannot use this Soulstone.
        msg(w, from, 1070714);
        return false;
    }
    if check_combat(w, from, TimeSpan::from_secs(120)) {
        // You must wait two minutes after engaging in combat before you can use a Soulstone.
        msg(w, from, 1070727);
        return false;
    }
    if muo_server::mobile::base::criminal(w, from) {
        // You must wait two minutes after committing a criminal act before you can use a Soulstone.
        msg(w, from, 1070728);
        return false;
    }
    if muo_server::mobile::base::get_logout_delay(w, from) > TimeSpan::ZERO {
        // In order to use your Soulstone, you must be in a safe log-out location.
        msg(w, from, 1070729);
        return false;
    }
    if !muo_content::mx::alive(w, from) {
        msg(w, from, 1070730); // You may not use a Soulstone while your character is dead.
        return false;
    }
    if hooks(w)
        .and_then(|h| h.sigil_exists_on)
        .is_some_and(|f| f(w, from))
    {
        // You may not use a Soulstone while your character has a faction town sigil.
        msg(w, from, 1070731);
        return false;
    }
    let casting = w
        .mobile(from)
        .and_then(|m| m.spell)
        .is_some_and(|sp| muo_content::spells::spell::is_casting(w, sp));
    if casting {
        msg(w, from, 1070733); // You may not use a Soulstone while your character is casting a spell.
        return false;
    }
    if muo_server::mobile::part1::props_b::poisoned(w, from) {
        msg(w, from, 1070734); // You may not use a Soulstone while your character is poisoned.
        return false;
    }
    if muo_server::mobile::base::paralyzed(w, from) {
        msg(w, from, 1070735); // You may not use a Soulstone while your character is paralyzed.
        return false;
    }
    if muo_content::player_mobile::data(w, from).is_some_and(|p| p.accelerated_start > w.now) {
        // You may not use a soulstone while your character is under the effects of a Scroll of Alacrity.
        msg(w, from, 1078115);
        return false;
    }
    true
}

fn on_double_click(w: &mut World, id: ItemId, from: MobileId) {
    if !check_use(w, id, from) {
        return;
    }
    if let Some(ns) = super::util::net_state(w, from) {
        close_gump::<SelectSkillGump>(w, ns);
        close_gump::<ConfirmSkillGump>(w, ns);
        close_gump::<ConfirmTransferGump>(w, ns);
        close_gump::<ConfirmRemovalGump>(w, ns);
        close_gump::<ErrorGump>(w, ns);
    }
    if is_empty(w, id) {
        SelectSkillGump::display_to(w, from, id);
    } else {
        ConfirmTransferGump::display_to(w, from, id);
    }
}

fn get_properties(w: &World, id: ItemId, list: &mut PropertyList) {
    (ITEM_BASE.get_properties)(w, id, list);
    let Some(d) = data(w, id) else { return };
    if d.skill_value > 0.0 {
        // Skill stored: ~1_skillname~ ~2_skillamount~
        list.add_str(
            1070721,
            &format!(
                "{}\t{:.1}",
                cliloc_arg(i64::from(skill_label(d.skill))),
                d.skill_value
            ),
        );
    }
    match &d.last_user_name {
        Some(n) => list.add_str(1041602, n), // Owner: ~1_val~
        None => list.add_localized_number(1041602, 1074235),
    }
}

/// `SoulStone.Deserialize`'s pre-version-4 layout.
fn read_legacy(w: &mut World, id: ItemId, r: &mut dyn GenericReader) -> muo_io::Result<()> {
    let last_user_name = r.read_string()?;
    let level = r.read_i32()?;
    let active_item_id = r.read_i32()?;
    let inactive_item_id = r.read_i32()?;
    let account = r.read_string()?;
    r.read_date_time_ticks()?; // m_NextUse - Not used
    let skill = SkillName::from_i32(r.read_encoded_int()?).unwrap_or_default();
    let skill_value = r.read_f64()?;
    set_data(
        w,
        id,
        SoulData {
            last_user_name,
            level,
            active_item_id,
            inactive_item_id,
            account,
            skill,
            skill_value,
        },
    );
    Ok(())
}

fn set_data(w: &mut World, id: ItemId, d: SoulData) {
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(d);
    }
}

fn soul_serialize(w: &World, id: ItemId, out: &mut dyn GenericWriter) {
    (ITEM_BASE.serialize)(w, id, out);
    out.write_encoded_int(4);
    let d = data(w, id).cloned().unwrap_or(SoulData {
        last_user_name: None,
        level: 0,
        active_item_id: 0,
        inactive_item_id: 0,
        account: None,
        skill: SkillName::Alchemy,
        skill_value: 0.0,
    });
    out.write_opt_string(d.last_user_name.as_deref());
    out.write_enum(4, i64::from(d.level));
    out.write_i32(d.active_item_id);
    out.write_i32(d.inactive_item_id);
    out.write_opt_string(d.account.as_deref());
    out.write_enum(4, d.skill as i64);
    out.write_f64(d.skill_value);
}

fn soul_deserialize(w: &mut World, id: ItemId, r: &mut dyn GenericReader) -> muo_io::Result<()> {
    (ITEM_BASE.deserialize)(w, id, r)?;
    let version = r.read_encoded_int()?;
    if version < 4 {
        return read_legacy(w, id, r);
    }
    let last_user_name = r.read_string()?;
    let level = r.read_enum(4)? as i32;
    let active_item_id = r.read_i32()?;
    let inactive_item_id = r.read_i32()?;
    let account = r.read_string()?;
    let skill = SkillName::from_i32(r.read_enum(4)? as i32).unwrap_or_default();
    let skill_value = r.read_f64()?;
    set_data(
        w,
        id,
        SoulData {
            last_user_name,
            level,
            active_item_id,
            inactive_item_id,
            account,
            skill,
            skill_value,
        },
    );
    Ok(())
}

/// `Server.Items.SoulStone`.
pub static SOULSTONE: ItemType = ItemType {
    full_name: "Server.Items.SoulStone",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| soulstone_new(w, &SOULSTONE, None, 0x2A93, 0x2A94)),
    label_number: |_, _| 1030899, // soulstone
    get_properties,
    on_double_click,
    serialize: soul_serialize,
    deserialize: soul_deserialize,
    ..ITEM_BASE
};

// ---- SoulstoneFragment ----

fn fragment_uses(w: &World, id: ItemId) -> i32 {
    w.item(id)
        .and_then(|i| i.ext.get::<FragmentData>())
        .map_or(0, |d| d.uses_remaining)
}

/// `UsesRemaining` of a fragment (`IUsesRemaining`).
pub fn set_fragment_uses(w: &mut World, id: ItemId, value: i32) {
    if let Some(d) = w.item_mut(id).and_then(|i| i.ext.get_mut::<FragmentData>()) {
        d.uses_remaining = value;
    }
    ops::invalidate_properties(w, id);
}

/// `SoulstoneFragment(usesRemaining, account)`.
pub fn soulstone_fragment_new(
    w: &mut World,
    uses_remaining: i32,
    account: Option<String>,
) -> ItemId {
    let inactive = utility::random_from(&mut w.rng, 0x2AA1, 9);
    let id = soulstone_new(w, &SOULSTONE_FRAGMENT, account, inactive, 0x2A94);
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(FragmentData { uses_remaining });
    }
    id
}

/// `Server.Items.SoulstoneFragment`.
pub static SOULSTONE_FRAGMENT: ItemType = ItemType {
    full_name: "Server.Items.SoulstoneFragment",
    parent: Some(&SOULSTONE),
    construct: Some(|w| soulstone_fragment_new(w, 5, None)),
    label_number: |_, _| 1071000, // soulstone fragment
    get_properties: |w, id, list| {
        (SOULSTONE.get_properties)(w, id, list);
        list.add_int(1060584, fragment_uses(w, id)); // uses remaining: ~1_val~
    },
    serialize: |w, id, out| {
        (SOULSTONE.serialize)(w, id, out);
        out.write_encoded_int(0);
        out.write_encoded_int(fragment_uses(w, id));
    },
    deserialize: |w, id, r| {
        (SOULSTONE.deserialize)(w, id, r)?;
        let _version = r.read_encoded_int()?;
        let uses_remaining = r.read_encoded_int()?;
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(FragmentData { uses_remaining });
        }
        Ok(())
    },
    ..SOULSTONE
};

// ---- BlueSoulstone, RedSoulstone ----

/// `Server.Items.BlueSoulstone`.
pub static BLUE_SOULSTONE: ItemType = ItemType {
    full_name: "Server.Items.BlueSoulstone",
    parent: Some(&SOULSTONE),
    construct: Some(|w| soulstone_new(w, &BLUE_SOULSTONE, None, 0x2ADC, 0x2ADD)),
    serialize: |w, id, out| {
        (SOULSTONE.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (SOULSTONE.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        Ok(())
    },
    ..SOULSTONE
};

/// `BlueSoulstone.Flip()`.
pub fn blue_flip(w: &mut World, id: ItemId) {
    let Some(ty) = w.item(id).map(|i| i.ty) else {
        return;
    };
    let cur = (ty.item_id)(w, id);
    let next = match cur {
        0x2ADC => 0x2AEC,
        0x2ADD => 0x2AED,
        0x2AEC => 0x2ADC,
        0x2AED => 0x2ADD,
        other => other,
    };
    (ty.set_item_id)(w, id, next);
}

fn red_is_reward(w: &World, id: ItemId) -> bool {
    w.item(id)
        .and_then(|i| i.ext.get::<RedData>())
        .is_some_and(|d| d.is_reward_item)
}

/// `soulstone.Account = value`.
pub fn set_account(w: &mut World, id: ItemId, value: Option<String>) {
    if let Some(mut d) = data(w, id).cloned() {
        d.account = value;
        set_data(w, id, d);
    }
    ops::invalidate_properties(w, id);
}

/// `RedSoulstone.IsRewardItem = value`.
pub fn set_red_is_reward_item(w: &mut World, id: ItemId, value: bool) {
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(RedData {
            is_reward_item: value,
        });
    }
    ops::invalidate_properties(w, id);
}

/// `Server.Items.RedSoulstone`.
pub static RED_SOULSTONE: ItemType = ItemType {
    full_name: "Server.Items.RedSoulstone",
    parent: Some(&SOULSTONE),
    construct: Some(|w| {
        let id = soulstone_new(w, &RED_SOULSTONE, None, 0x32F3, 0x32F4);
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(RedData::default());
        }
        id
    }),
    get_properties: |w, id, list| {
        (SOULSTONE.get_properties)(w, id, list);
        if red_is_reward(w, id) {
            list.add(1076217); // 1st Year Veteran Reward
        }
    },
    serialize: |w, id, out| {
        (SOULSTONE.serialize)(w, id, out);
        out.write_i32(0);
        out.write_bool(red_is_reward(w, id));
    },
    deserialize: |w, id, r| {
        (SOULSTONE.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        let is_reward_item = r.read_bool()?;
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(RedData { is_reward_item });
        }
        Ok(())
    },
    ..SOULSTONE
};

// ---- gumps ----

fn mobile_of(w: &World, ns: NetStateId) -> Option<MobileId> {
    w.net_states.get(&ns).and_then(|n| n.mobile)
}

/// The frame every gump of the file draws.
fn frame(b: &mut DynamicGumpBuilder) {
    b.add_page(0);
    b.add_background(0, 0, 520, 440, 0x13BE);
    b.add_image_tiled(10, 10, 500, 20, 0xA40);
    b.add_image_tiled(10, 40, 500, 360, 0xA40);
    b.add_image_tiled(10, 410, 500, 20, 0xA40);
    b.add_alpha_region(10, 10, 500, 420);
}

fn html(b: &mut DynamicGumpBuilder, x: i32, y: i32, w: i32, h: i32, number: i32) {
    b.add_html_localized_color(x, y, w, h, number, 0x7FFF, false, false);
}

fn html_scroll(b: &mut DynamicGumpBuilder, x: i32, y: i32, w: i32, h: i32, number: i32) {
    b.add_html_localized_color(x, y, w, h, number, 0x7FFF, false, true);
}

fn send<G: muo_content::gumps::Gump + 'static>(w: &mut World, to: MobileId, g: G) {
    send_gump_to_mobile(w, to, Box::new(g), false);
}

/// The particles and sounds of both transfers.
fn transfer_effects(w: &mut World, from: MobileId) {
    let (loc, map) = (
        muo_content::mx::location(w, from),
        muo_content::mx::map(w, from),
    );
    let fx = effect_item::create_targeted(w, loc, map, effect_item::DEFAULT_DURATION);
    effects::send_location_particles(w, fx, 0, 0, 0, 5060);
    effects::play_sound(w, loc, map, 0x243);
    effects::send_moving_particles_layer(
        w,
        Targeted::point(Point3D::new(loc.x - 6, loc.y - 6, loc.z + 15), map),
        from,
        0x36D4,
        7,
        0,
        false,
        true,
        0x497,
        0,
        9502,
        1,
        0,
        255,
        0x100,
    );
    effects::send_target_particles_full(
        w,
        from,
        0x375A,
        35,
        90,
        0x00,
        0x00,
        9502,
        EffectLayer(255),
        0x100,
    );
}

/// The skills with a base above zero, as `_from.Skills` held them when the gump was built (the layout is compiled on
/// the send, so a snapshot is the same thing).
fn skills_with_base(w: &mut World, from: MobileId) -> Vec<SkillName> {
    let n = skills_len(w, from);
    (0..n as i32)
        .filter_map(SkillName::from_i32)
        .filter(|&s| skills::base(w, from, s) > 0.0)
        .collect()
}

fn skills_len(w: &World, from: MobileId) -> usize {
    w.mobile(from)
        .and_then(|m| m.skills.as_ref())
        .map_or(0, |s| s.len())
}

pub struct SelectSkillGump {
    stone: ItemId,
    skills: Vec<SkillName>,
}

impl SelectSkillGump {
    fn display_to(w: &mut World, from: MobileId, stone: ItemId) {
        if super::util::net_state(w, from).is_none()
            || super::util::deleted(w, stone)
            || !is_empty(w, stone)
        {
            return;
        }
        let skills = skills_with_base(w, from);
        send(
            w,
            from,
            DynamicGumpObj::new(50, 50, SelectSkillGump { stone, skills }),
        );
    }
}

impl DynamicGumpLogic for SelectSkillGump {
    const TYPE_NAME: &'static str = "Server.Items.SoulStone+SelectSkillGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        frame(b);
        // Which skill do you wish to transfer to the Soulstone?
        html(b, 10, 12, 500, 20, 1061087);
        b.add_button(10, 410, 0xFB1, 0xFB2, 0);
        html(b, 45, 412, 450, 20, 1060051); // CANCEL
        for (n, &name) in self.skills.iter().enumerate() {
            let n = n as i32;
            let p = n % 30;
            if p == 0 {
                let page = n / 30;
                if page > 0 {
                    b.add_button_type(260, 380, 0xFA5, 0xFA6, 0, GumpButtonType::Page, page + 1);
                    html(b, 305, 382, 200, 20, 1011066); // Next page
                }
                b.add_page(page + 1);
                if page > 0 {
                    b.add_button_type(10, 380, 0xFAE, 0xFAF, 0, GumpButtonType::Page, page);
                    html(b, 55, 382, 200, 20, 1011067); // Previous page
                }
            }
            let x = if p % 2 == 0 { 10 } else { 260 };
            let y = p / 2 * 20 + 40;
            b.add_button(x, y, 0xFA5, 0xFA6, name as i32 + 1);
            html(b, x + 45, y + 2, 200, 20, skill_label(name));
        }
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        if info.button_id == 0 || !is_empty(w, self.stone) {
            return;
        }
        let Some(from) = mobile_of(w, sender) else {
            return;
        };
        let i_skill = info.button_id - 1;
        if i_skill < 0 || i_skill as usize >= skills_len(w, from) {
            return;
        }
        let Some(name) = SkillName::from_i32(i_skill) else {
            return;
        };
        if skills::base(w, from, name) <= 0.0 || !check_use(w, self.stone, from) {
            return;
        }
        ConfirmSkillGump::display_to(w, from, self.stone, name);
    }
}

pub struct ConfirmSkillGump {
    skill: SkillName,
    stone: ItemId,
    base: f64,
    cap: f64,
}

impl ConfirmSkillGump {
    fn display_to(w: &mut World, from: MobileId, stone: ItemId, skill: SkillName) {
        if super::util::net_state(w, from).is_none()
            || super::util::deleted(w, stone)
            || !is_empty(w, stone)
        {
            return;
        }
        let (base, cap) = (skills::base(w, from, skill), skills::cap(w, from, skill));
        send(
            w,
            from,
            DynamicGumpObj::new(
                50,
                50,
                ConfirmSkillGump {
                    skill,
                    stone,
                    base,
                    cap,
                },
            ),
        );
    }
}

impl DynamicGumpLogic for ConfirmSkillGump {
    const TYPE_NAME: &'static str = "Server.Items.SoulStone+ConfirmSkillGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        frame(b);
        html(b, 10, 12, 500, 20, 1070709); // <CENTER>Confirm Soulstone Transfer</CENTER>
        html_scroll(b, 10, 42, 500, 110, 1061067);
        html(b, 10, 200, 390, 20, 1062297); // Skill Chosen:
        html(b, 210, 200, 390, 20, skill_label(self.skill));
        html(b, 10, 220, 390, 20, 1062298); // Current Value:
        b.add_label(210, 220, 0x481, &format!("{:.1}", self.base));
        html(b, 10, 240, 390, 20, 1062299); // Current Cap:
        b.add_label(210, 240, 0x481, &format!("{:.1}", self.cap));
        html(b, 10, 260, 390, 20, 1062300); // New Value:
        b.add_label(210, 260, 0x481, "0.0");
        b.add_button(10, 360, 0xFA5, 0xFA6, 2);
        html(b, 45, 362, 450, 20, 1070720); // Activate the stone.  I am ready to transfer the skill points to it.
        b.add_button(10, 380, 0xFA5, 0xFA6, 1);
        html(b, 45, 382, 450, 20, 1062279); // No, let me make another selection.
        b.add_button(10, 410, 0xFB1, 0xFB2, 0);
        html(b, 45, 412, 450, 20, 1060051); // CANCEL
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        if info.button_id == 0 || !is_empty(w, self.stone) {
            return;
        }
        let Some(from) = mobile_of(w, sender) else {
            return;
        };
        if !check_use(w, self.stone, from) {
            return;
        }
        if info.button_id == 1 {
            // Is asking for another selection
            SelectSkillGump::display_to(w, from, self.stone);
            return;
        }
        if skills::base(w, from, self.skill) <= 0.0 {
            return;
        }
        if skills::lock(w, from, self.skill) != SkillLock::Down {
            // Unable to Transfer Selected Skill to Soulstone: the skill may be locked or set to raise.
            ErrorGump::display_to(w, from, self.stone, 1070710, 1070711);
            return;
        }
        let base = skills::base(w, from, self.skill);
        if let Some(d) = data_mut(w, self.stone) {
            d.skill = self.skill;
        }
        ops::invalidate_properties(w, self.stone);
        set_skill_value(w, self.stone, base);
        skills::set_base(w, from, self.skill, 0.0);
        msg(w, from, 1070712); // You have successfully transferred your skill points into the Soulstone.
        let name = muo_server::mobile::part1::props_b::raw_name(w, from);
        set_last_user_name(w, self.stone, name);
        transfer_effects(w, from);
    }
}

pub struct ConfirmTransferGump {
    stone: ItemId,
    skill: SkillName,
    base: f64,
    cap: f64,
    stored: f64,
}

impl ConfirmTransferGump {
    fn display_to(w: &mut World, from: MobileId, stone: ItemId) {
        if super::util::net_state(w, from).is_none()
            || super::util::deleted(w, stone)
            || is_empty(w, stone)
        {
            return;
        }
        let skill = skill_of(w, stone);
        let g = ConfirmTransferGump {
            stone,
            skill,
            base: skills::base(w, from, skill),
            cap: skills::cap(w, from, skill),
            stored: skill_value(w, stone),
        };
        send(w, from, DynamicGumpObj::new(50, 50, g));
    }
}

impl DynamicGumpLogic for ConfirmTransferGump {
    const TYPE_NAME: &'static str = "Server.Items.SoulStone+ConfirmTransferGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        frame(b);
        html(b, 10, 12, 500, 20, 1070709); // <CENTER>Confirm Soulstone Transfer</CENTER>
        html_scroll(b, 10, 42, 500, 110, 1061067);
        html(b, 10, 200, 390, 20, 1070718); // Skill Stored:
        html(b, 210, 200, 390, 20, skill_label(self.skill));
        html(b, 10, 220, 390, 20, 1062298); // Current Value:
        b.add_label(210, 220, 0x481, &format!("{:.1}", self.base));
        html(b, 10, 240, 390, 20, 1062299); // Current Cap:
        b.add_label(210, 240, 0x481, &format!("{:.1}", self.cap));
        html(b, 10, 260, 390, 20, 1062300); // New Value:
        b.add_label(210, 260, 0x481, &format!("{:.1}", self.stored));
        b.add_button(10, 360, 0xFA5, 0xFA6, 2);
        html(b, 45, 362, 450, 20, 1070719); // Activate the stone.  I am ready to retrieve the skill points from it.
        b.add_button(10, 380, 0xFA5, 0xFA6, 1);
        html(b, 45, 382, 450, 20, 1070723); // Remove all skill points from this stone and DO NOT absorb them.
        b.add_button(10, 410, 0xFB1, 0xFB2, 0);
        html(b, 45, 412, 450, 20, 1060051); // CANCEL
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        if info.button_id == 0 || is_empty(w, self.stone) {
            return;
        }
        let Some(from) = mobile_of(w, sender) else {
            return;
        };
        if !check_use(w, self.stone, from) {
            return;
        }
        if info.button_id == 1 {
            // Remove skill points
            ConfirmRemovalGump::display_to(w, from, self.stone);
            return;
        }
        let stone = self.stone;
        let stone_skill = skill_of(w, stone);
        let skill_value = skill_value(w, stone);
        let from_base_fixed = skills::base_fixed(w, from, stone_skill);
        let skills_cap = muo_server::mobile::part1::props_a::skills_cap(w, from);
        let skills_total = skills::total(w, from);

        // If we have, say, 88.4 in our skill and the stone holds 100, we need 11.6 free points. Also, if we're
        // below our skillcap by, say, 8.2 points, we only need 11.6 - 8.2 = 3.4 points.
        let mut required_amount =
            (skill_value * 10.0) as i32 - from_base_fixed - (skills_cap - skills_total);

        let n = skills_len(w, from) as i32;
        let mut cannot_absorb = false;
        if skills::lock(w, from, stone_skill) != SkillLock::Up {
            cannot_absorb = true;
        } else if required_amount > 0 {
            let mut available = 0;
            for i in 0..n {
                let Some(s) = SkillName::from_i32(i) else {
                    continue;
                };
                if skills::lock(w, from, s) != SkillLock::Down {
                    continue;
                }
                available += skills::base_fixed(w, from, s);
            }
            if required_amount > available {
                cannot_absorb = true;
            }
        }
        if cannot_absorb {
            // Unable to Absorb Selected Skill from Soulstone: locked, set to lower, or at the total cap.
            ErrorGump::display_to(w, from, stone, 1070717, 1070716);
            return;
        }
        if skill_value > skills::cap(w, from, stone_skill) {
            // The amount stored exceeds your individual skill cap for that skill.
            ErrorGump::display_to(w, from, stone, 1070717, 1070715);
            return;
        }
        if skills::base(w, from, stone_skill) >= skill_value {
            // Wrong message?!
            ErrorGump::display_to(w, from, stone, 1070717, 1070802);
            return;
        }
        if muo_content::player_mobile::data(w, from).is_some_and(|p| p.accelerated_start > w.now) {
            // Wrong message?!
            ErrorGump::display_to(w, from, stone, 1070717, 1078115);
            return;
        }
        if required_amount > 0 {
            for i in 0..n {
                let Some(s) = SkillName::from_i32(i) else {
                    continue;
                };
                if skills::lock(w, from, s) != SkillLock::Down {
                    continue;
                }
                let fixed = skills::base_fixed(w, from, s);
                if required_amount >= fixed {
                    required_amount -= fixed;
                    skills::set_base(w, from, s, 0.0);
                } else {
                    skills::set_base_fixed(w, from, s, fixed - required_amount);
                    break;
                }
            }
        }
        skills::set_base(w, from, stone_skill, skill_value);
        set_skill_value(w, stone, 0.0);
        msg(w, from, 1070713); // You have successfully absorbed the Soulstone's skill points.
        let name = muo_server::mobile::part1::props_b::raw_name(w, from);
        set_last_user_name(w, stone, name);
        transfer_effects(w, from);
        if is_fragment(w, stone) {
            let left = fragment_uses(w, stone) - 1;
            set_fragment_uses(w, stone, left);
            if left <= 0 {
                msg(w, from, 1070974); // You have used up your soulstone fragment.
            }
        }
    }
}

pub struct ConfirmRemovalGump {
    stone: ItemId,
}

impl ConfirmRemovalGump {
    fn display_to(w: &mut World, from: MobileId, stone: ItemId) {
        if super::util::net_state(w, from).is_none()
            || super::util::deleted(w, stone)
            || is_empty(w, stone)
        {
            return;
        }
        send(
            w,
            from,
            StaticGumpObj::new(50, 50, ConfirmRemovalGump { stone }),
        );
    }
}

impl StaticGumpLogic for ConfirmRemovalGump {
    const TYPE_NAME: &'static str = "Server.Items.SoulStone+ConfirmRemovalGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut StaticGumpBuilder) {
        b.add_page(0);
        b.add_background(0, 0, 520, 440, 0x13BE);
        b.add_image_tiled(10, 10, 500, 20, 0xA40);
        b.add_image_tiled(10, 40, 500, 360, 0xA40);
        b.add_image_tiled(10, 410, 500, 20, 0xA40);
        b.add_alpha_region(10, 10, 500, 420);
        b.add_html_localized_color(10, 12, 500, 20, 1070725, 0x7FFF, false, false); // Confirm Soulstone Skill Removal
        b.add_html_localized_color(10, 42, 500, 110, 1070724, 0x7FFF, false, true); // WARNING!..
        b.add_button(10, 380, 0xFA5, 0xFA6, 1);
        b.add_html_localized_color(45, 382, 450, 20, 1052072, 0x7FFF, false, false); // Continue
        b.add_button(10, 410, 0xFB1, 0xFB2, 0);
        b.add_html_localized_color(45, 412, 450, 20, 1060051, 0x7FFF, false, false); // CANCEL
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        if info.button_id == 0 || is_empty(w, self.stone) {
            return;
        }
        let Some(from) = mobile_of(w, sender) else {
            return;
        };
        if !check_use(w, self.stone, from) {
            return;
        }
        set_skill_value(w, self.stone, 0.0);
        msg(w, from, 1070726); // You have successfully deleted the Soulstone's skill points.
    }
}

pub struct ErrorGump {
    stone: ItemId,
    title: i32,
    message: i32,
}

impl ErrorGump {
    fn display_to(w: &mut World, from: MobileId, stone: ItemId, title: i32, message: i32) {
        if super::util::net_state(w, from).is_none() || super::util::deleted(w, stone) {
            return;
        }
        send(
            w,
            from,
            DynamicGumpObj::new(
                50,
                50,
                ErrorGump {
                    stone,
                    title,
                    message,
                },
            ),
        );
    }
}

impl DynamicGumpLogic for ErrorGump {
    const TYPE_NAME: &'static str = "Server.Items.SoulStone+ErrorGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        frame(b);
        html(b, 10, 12, 500, 20, self.title);
        html_scroll(b, 10, 42, 500, 110, self.message);
        b.add_button(10, 380, 0xFA5, 0xFA6, 1);
        html(b, 45, 382, 450, 20, 1052072); // Continue
        b.add_button(10, 410, 0xFB1, 0xFB2, 0);
        html(b, 45, 412, 450, 20, 1060051); // CANCEL
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        if info.button_id == 0 {
            return;
        }
        let Some(from) = mobile_of(w, sender) else {
            return;
        };
        if !check_use(w, self.stone, from) {
            return;
        }
        if is_empty(w, self.stone) {
            SelectSkillGump::display_to(w, from, self.stone);
        } else {
            ConfirmTransferGump::display_to(w, from, self.stone);
        }
    }
}

// ---- registration ----

const INT: Kind = Kind::Plain(PropType::Numeric(Numeric::Int));
const STR: Kind = Kind::Plain(PropType::Str);

static SOULSTONE_CTORS: [Ctor; 1] = [Ctor {
    access: None,
    params: &[
        Param {
            name: "account",
            kind: STR,
            default: Some(Value::Null),
            ty: "",
        },
        Param {
            name: "inactiveItemID",
            kind: INT,
            default: Some(Value::Int(0x2A93)),
            ty: "",
        },
        Param {
            name: "activeItemID",
            kind: INT,
            default: Some(Value::Int(0x2A94)),
            ty: "",
        },
    ],
    make: |w, v| {
        EntityId::Item(soulstone_new(
            w,
            &SOULSTONE,
            v[0].string(),
            v[1].i32(),
            v[2].i32(),
        ))
    },
}];

static FRAGMENT_CTORS: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[Param {
            name: "account",
            kind: STR,
            default: None,
            ty: "",
        }],
        make: |w, v| EntityId::Item(soulstone_fragment_new(w, 5, v[0].string())),
    },
    Ctor {
        access: None,
        params: &[
            Param {
                name: "usesRemaining",
                kind: INT,
                default: Some(Value::Int(5)),
                ty: "",
            },
            Param {
                name: "account",
                kind: STR,
                default: Some(Value::Null),
                ty: "",
            },
        ],
        make: |w, v| EntityId::Item(soulstone_fragment_new(w, v[0].i32(), v[1].string())),
    },
];

static BLUE_CTORS: [Ctor; 1] = [Ctor {
    access: None,
    params: &[Param {
        name: "account",
        kind: STR,
        default: Some(Value::Null),
        ty: "",
    }],
    make: |w, v| {
        EntityId::Item(soulstone_new(
            w,
            &BLUE_SOULSTONE,
            v[0].string(),
            0x2ADC,
            0x2ADD,
        ))
    },
}];

static RED_CTORS: [Ctor; 1] = [Ctor {
    access: None,
    params: &[Param {
        name: "account",
        kind: STR,
        default: Some(Value::Null),
        ty: "",
    }],
    make: |w, v| {
        let id = soulstone_new(w, &RED_SOULSTONE, v[0].string(), 0x32F3, 0x32F4);
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(RedData::default());
        }
        EntityId::Item(id)
    },
}];

/// Configure phase: `[Constructible]` overloads, `ISecurable`, the `BlueSoulstone.Flip`.
pub(super) fn configure(w: &mut World) {
    add_ctors(w, "Server.Items.SoulStone", &SOULSTONE_CTORS);
    add_ctors(w, "Server.Items.SoulstoneFragment", &FRAGMENT_CTORS);
    add_ctors(w, "Server.Items.BlueSoulstone", &BLUE_CTORS);
    add_ctors(w, "Server.Items.RedSoulstone", &RED_CTORS);
    house_deps::deps_mut(w).securables.push(|w, id| {
        w.item(id)
            .is_some_and(|i| i.ty.is(&SOULSTONE))
            .then(|| data(w, id).map_or(0, |d| d.level) as u8)
    });
    house_deps::deps_mut(w)
        .securable_setters
        .push(|w, id, level| {
            let is = w.item(id).is_some_and(|i| i.ty.is(&SOULSTONE));
            if is && let Some(d) = data_mut(w, id) {
                d.level = i32::from(level);
            }
            is
        });
    muo_content::items::flip::register_own_flip(w, "Server.Items.BlueSoulstone", blue_flip);
}

pub fn register(types: &mut TypeRegistry) {
    for t in [
        &SOULSTONE,
        &SOULSTONE_FRAGMENT,
        &BLUE_SOULSTONE,
        &RED_SOULSTONE,
    ] {
        types.register_item(t, &[]);
    }
}
