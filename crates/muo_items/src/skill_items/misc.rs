//! `FireHorn` and `RecipeScroll`.

use muo_content::props::{Ctor, Kind, Param, Value, add_ctors};
use muo_content::utilities::types::{Numeric, PropType};
use muo_io::text::TextDefinition;
use muo_primitives::{Point3D, TimeSpan, utility};
use muo_server::World;
use muo_server::ext::Ext;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, base, deps, ops};
use muo_server::packets::message::MessageType;
use muo_server::property_list::PropertyList;
use muo_server::skills::SkillName;
use muo_server::target::{self, TargetFlags, Targeted};
use muo_server::type_registry::{Phase, TypeRegistry};

use super::{read_version, write_version};

// ---- FireHorn ----

const ACTION: &str = "FireHorn";

fn sulfurous_ash(w: &World) -> Option<&'static ItemType> {
    super::item_type(w, "Server.Items.SulfurousAsh")
}

/// `CheckUse(from)`.
fn check_use(w: &mut World, horn: ItemId, from: MobileId) -> bool {
    if !base::is_accessible_to(w, horn, from) {
        return false;
    }
    let loc = ops::get_world_location(w, horn);
    let same_map = w.mobile(from).map(|m| m.map) == w.item(horn).map(|i| i.map);
    if !same_map || !deps::mobile_in_range(w, from, loc, 2) {
        deps::local_overhead_message(w, from, MessageType::REGULAR, 0x3B2, 1019045); // I can't reach that.
        return false;
    }
    if !muo_server::mobile::part1::combat::can_begin_action(w, from, ACTION) {
        deps::send_localized_message(w, from, 1049615); // You must take a moment to catch your breath.
        return false;
    }
    let need = if muo_content::core::aos(w) { 4 } else { 15 };
    let pack = w.mobile(from).and_then(|m| m.backpack);
    // `Backpack?.GetAmount(..) < need` is false without a backpack.
    if let (Some(p), Some(ash)) = (pack, sulfurous_ash(w))
        && muo_server::item::container::get_amount(w, p, ash, true) < need
    {
        deps::send_localized_message(w, from, 1049617); // You do not have enough sulfurous ash.
        return false;
    }
    true
}

fn horn_double_click(w: &mut World, id: ItemId, from: MobileId) {
    if check_use(w, id, from) {
        deps::send_localized_message(w, from, 1049620); // Select an area to incinerate.
        let range = if muo_content::core::aos(w) { 3 } else { 2 };
        target::begin_target(
            w,
            from,
            range,
            true,
            TargetFlags::HARMFUL,
            move |w, from, t| {
                if w.item(id).is_none_or(|i| i.deleted()) {
                    return;
                }
                let loc = match t {
                    Targeted::Entity(EntityId::Item(i)) => ops::get_world_location(w, i),
                    Targeted::Entity(EntityId::Mobile(m)) => {
                        w.mobile(m).map_or_else(Point3D::default, |x| x.location)
                    }
                    Targeted::Land(l) => l.location,
                    Targeted::Static(s) => s.location,
                };
                use_horn(w, id, from, loc);
            },
        );
    }
}

/// `Use(from, loc)`.
fn use_horn(w: &mut World, horn: ItemId, from: MobileId, loc: Point3D) {
    if !check_use(w, horn, from) {
        return;
    }
    let aos = muo_content::core::aos(w);
    muo_server::mobile::part1::combat::begin_action(w, from, ACTION);
    let delay = if aos {
        TimeSpan::from_secs(6)
    } else {
        TimeSpan::from_secs(12)
    };
    muo_server::timer::start(w, delay, TimeSpan::ZERO, 1, move |w| {
        muo_server::mobile::part1::combat::end_action(w, from, ACTION);
        deps::send_localized_message(w, from, 1049621); // You catch your breath.
    });

    let music = muo_server::skills::fixed(w, from, SkillName::Musicianship);
    let suc_chance = 500 + (music - 775) * 2;
    let d_suc_chance = f64::from(suc_chance) / 1000.0;
    if !muo_server::mobile::part2::stats::check_skill_chance(
        w,
        from,
        SkillName::Musicianship,
        d_suc_chance,
    ) {
        deps::send_localized_message(w, from, 1049618); // The horn emits a pathetic squeak.
        muo_server::mobile::part2::effects::play_sound(w, from, 0x18A);
        return;
    }

    let ash_count = if aos { 4 } else { 15 };
    if let (Some(pack), Some(ash)) = (w.mobile(from).and_then(|m| m.backpack), sulfurous_ash(w)) {
        muo_server::item::container::consume_up_to(w, pack, ash, ash_count, true);
    }
    muo_server::mobile::part2::effects::play_sound(w, from, 0x15F);
    muo_server::effects::send_moving_effect_to_point(w, from, loc, 0x36D4, 5, 0, false, true, 0, 0);

    let mut player_vs_player = false;
    let mut targets = std::collections::VecDeque::new();
    let map = w.mobile(from).and_then(|m| m.map);
    let near: Vec<MobileId> =
        muo_server::map::enumerate::mobiles_in_range(w, map, loc.x, loc.y, 2).collect();
    for m in near {
        if from != m
            && muo_content::spells::helper::valid_indirect_target(w, from, m)
            && muo_content::mx::can_be_harmful_quiet(w, from, m)
            && (!aos || muo_content::mx::in_los(w, from, m))
        {
            targets.push_back(m);
            if w.mobile(m).is_some_and(|x| x.player) {
                player_vs_player = true;
            }
        }
    }

    if !targets.is_empty() {
        let prov = muo_server::skills::fixed(w, from, SkillName::Provocation);
        let disc = muo_server::skills::fixed(w, from, SkillName::Discordance);
        let peace = muo_server::skills::fixed(w, from, SkillName::Peacemaking);
        let (min_damage, max_damage);
        if aos {
            let music_scaled = music + (music - 900).max(0) * 2;
            let prov_scaled = prov + (prov - 900).max(0) * 2;
            let disc_scaled = disc + (disc - 900).max(0) * 2;
            let peace_scaled = peace + (peace - 900).max(0) * 2;
            let weight_avg = (music_scaled + prov_scaled * 3 + disc_scaled * 3 + peace_scaled) / 80;
            let avg_damage = if player_vs_player {
                weight_avg / 3
            } else {
                weight_avg / 2
            };
            min_damage = avg_damage * 9 / 10;
            max_damage = avg_damage * 10 / 9;
        } else {
            let mut total = prov + disc / 5 + peace / 5;
            if player_vs_player {
                total /= 3;
            }
            max_damage = total * 2 / 30;
            min_damage = max_damage * 7 / 10;
        }
        let mut damage = f64::from(utility::random_min_max(&mut w.rng, min_damage, max_damage));
        if aos && targets.len() > 1 {
            damage = damage * 2.0 / targets.len() as f64;
        } else if !aos {
            damage /= targets.len() as f64;
        }
        while let Some(m) = targets.pop_front() {
            let mut to_deal = damage;
            if !aos && muo_content::mx::check_skill(w, m, SkillName::MagicResist, 0.0, 120.0) {
                to_deal *= 0.5;
                deps::send_localized_message(w, m, 501783); // You feel yourself resisting magical energy.
            }
            muo_content::mx::do_harmful(w, from, m);
            muo_content::spells::helper::damage_aos(
                w,
                None,
                TimeSpan::ZERO,
                m,
                Some(from),
                to_deal,
                muo_content::spells::helper::SpellDamageSplit {
                    fire: 100,
                    ..Default::default()
                },
            );
            muo_server::effects::send_target_effect(w, m, 0x3709, 10, 30, 0, 0);
        }
    }

    let break_chance = if aos { 0.01 } else { 0.16 };
    if utility::random_double(&mut w.rng) < break_chance {
        deps::send_localized_message(w, from, 1049619); // The fire horn crumbles in your hands.
        base::delete(w, horn);
    }
}

/// `Server.Items.FireHorn`; `[SerializationGenerator(0)]`.
pub static FIRE_HORN: ItemType = ItemType {
    full_name: "Server.Items.FireHorn",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let id = ops::new_item_with_id(w, &FIRE_HORN, 0xFC7);
        super::hue(w, id, 0x466);
        id
    }),
    default_weight: |_, _| 1.0,
    label_number: |_, _| 1060456, // fire horn
    on_double_click: horn_double_click,
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        write_version(out, true);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        read_version(r, true)?;
        Ok(())
    },
    ..ITEM_BASE
};

// ---- RecipeScroll ----

/// What the craft engine answers about a recipe.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Learn {
    /// 1044153: "You don't have the required skills to attempt this item."
    Unskilled,
    /// The recipe was acquired; the text goes in 1073451.
    Learned,
}

/// `Recipe.Recipes` and `CraftItem.GetSuccessChance`, set by `muo_engines::craft`.
#[derive(Default)]
pub struct RecipeHooks {
    /// `Recipe.TextDefinition`; `None` when there is no such recipe.
    pub text: Option<fn(&World, i32) -> Option<TextDefinition>>,
    /// `chance = r.CraftItem.GetSuccessChance(pm, null, r.CraftSystem, false, out all); all && chance >= 0` then
    /// `pm.AcquireRecipe(r)`.
    pub learn: Option<fn(&mut World, MobileId, i32) -> Learn>,
}

/// `RecipeScroll._recipeID`.
#[derive(Clone, Copy, Debug, Default)]
pub struct RecipeData(pub i32);

pub fn recipe_id(w: &World, id: ItemId) -> i32 {
    w.item(id)
        .and_then(|i| i.ext.get::<RecipeData>())
        .map_or(0, |d| d.0)
}

/// `new RecipeScroll(recipeID)`.
pub fn new_recipe_scroll(w: &mut World, recipe_id: i32) -> ItemId {
    let id = ops::new_item_with_id(w, &RECIPE_SCROLL, 0x2831);
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(RecipeData(recipe_id));
    }
    id
}

fn recipe_text(w: &World, recipe: i32) -> Option<TextDefinition> {
    let f = w.resources.get::<RecipeHooks>().and_then(|h| h.text)?;
    f(w, recipe)
}

fn scroll_double_click(w: &mut World, id: ItemId, from: MobileId) {
    let loc = ops::get_world_location(w, id);
    if !deps::mobile_in_range(w, from, loc, 2) {
        deps::local_overhead_message(w, from, MessageType::REGULAR, 0x3B2, 1019045); // I can't reach that.
        return;
    }
    let recipe = recipe_id(w, id);
    let Some(text) = recipe_text(w, recipe) else {
        return;
    };
    if !w
        .mobile(from)
        .is_some_and(|m| m.ty.is(&muo_content::player_mobile::PLAYER_MOBILE))
    {
        return;
    }
    if muo_content::player_mobile::has_recipe(w, from, recipe) {
        deps::send_localized_message(w, from, 1073427); // You already know this recipe.
        return;
    }
    let result = match w.resources.get::<RecipeHooks>().and_then(|h| h.learn) {
        Some(f) => f(w, from, recipe),
        None => Learn::Unskilled,
    };
    match result {
        Learn::Learned => {
            // You have learned a new recipe: ~1_RECIPE~
            muo_server::mobile::part2::messages::send_localized_message(
                w,
                from,
                1073451,
                &text.to_string(),
                0x3B2,
            );
            muo_content::player_mobile::acquire_recipe(w, from, recipe);
            base::delete(w, id);
        }
        Learn::Unskilled => deps::send_localized_message(w, from, 1044153), // You don't have the required skills to attempt this item.
    }
}

fn scroll_properties(w: &World, id: ItemId, list: &mut PropertyList) {
    base::get_properties_base(w, id, list);
    if let Some(td) = recipe_text(w, recipe_id(w, id)) {
        if td.number > 0 {
            list.add_localized_number(1049644, td.number); // [~1_stuff~]
        } else {
            list.add_str(1049644, td.string.as_deref().unwrap_or(""));
        }
    }
}

/// `Server.Items.RecipeScroll`.
pub static RECIPE_SCROLL: ItemType = ItemType {
    full_name: "Server.Items.RecipeScroll",
    parent: Some(&ITEM_BASE),
    construct: None,
    label_number: |_, _| 1074560, // recipe scroll
    get_properties: scroll_properties,
    on_double_click: scroll_double_click,
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        write_version(out, false);
        out.write_i32(recipe_id(w, id));
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        read_version(r, false)?;
        let n = r.read_i32()?;
        if let Some(i) = w.item_mut(id) {
            i.ext = Ext::new(RecipeData(n));
        }
        Ok(())
    },
    ..ITEM_BASE
};

const INT: Kind = Kind::Plain(PropType::Numeric(Numeric::Int));
static SCROLL_CTOR: [Ctor; 1] = [Ctor {
    access: None,
    params: &[Param {
        name: "recipeID",
        kind: INT,
        default: None,
        ty: "",
    }],
    make: |w, v| EntityId::Item(new_recipe_scroll(w, v[0].i32())),
}];

fn configure(w: &mut World) {
    add_ctors(w, "Server.Items.RecipeScroll", &SCROLL_CTOR);
    let _ = Value::Null;
}

/// Registers the classes.
pub fn register(types: &mut TypeRegistry) {
    types.register_item(&FIRE_HORN, &[]);
    types.register_item(&RECIPE_SCROLL, &[]);
    types.register_phase_default(
        Phase::Configure,
        "Server.Items.RecipeScroll.Configure",
        configure,
    );
}
