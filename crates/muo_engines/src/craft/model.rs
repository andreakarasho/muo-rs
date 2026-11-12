//! `CraftSystem`, `CraftItem` (data), `CraftRes`, `CraftSubRes`, `CraftSubResCol`, `CraftGroup`, `CraftContext` and
//! `CraftSkill`: the data of a craft system and its builder (`AddCraft`, `AddRes`, ...).
//!
//! A `Type` is a full type name (`"Server.Items.IronIngot"`), resolved against the type registry when used. A craft
//! system is built once (`Def*.InitCraftList`) and then read-only; what changes per player is [`CraftContext`].

use std::collections::HashMap;

pub use muo_data::expansion::Expansion;
pub use muo_io::text::TextDefinition as Td;
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
pub use muo_server::skills::SkillName;

/// `CraftECA`.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum CraftEca {
    ChanceMinusSixty,
    FiftyPercentChanceMinusTenPercent,
    ChanceMinusSixtyToFortyFive,
}

/// `BeverageType`.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum BeverageType {
    Ale,
    Cider,
    Liquor,
    Milk,
    Wine,
    Water,
}

/// `CraftSkill`.
#[derive(Copy, Clone, Debug)]
pub struct CraftSkill {
    pub skill: SkillName,
    pub min_skill: f64,
    pub max_skill: f64,
}

/// `CraftRes`. `name` empty means no name: the label of the item type (see [`super::item_label`]).
#[derive(Clone, Debug)]
pub struct CraftRes {
    pub item_type: &'static str,
    pub name: Td,
    pub amount: i32,
    pub message: Td,
}

/// `CraftSubRes`.
#[derive(Clone, Debug)]
pub struct CraftSubRes {
    pub item_type: &'static str,
    pub name: Td,
    pub required_skill: f64,
    pub generic_name_number: i32,
    pub message: Td,
}

/// `CraftSubResCol`.
#[derive(Clone, Debug, Default)]
pub struct SubResCol {
    pub init: bool,
    pub res_type: &'static str,
    pub name: Td,
    pub items: Vec<CraftSubRes>,
}

impl SubResCol {
    /// `SearchFor(type)`.
    pub fn search_for(&self, ty: &str) -> Option<&CraftSubRes> {
        self.items.iter().find(|s| s.item_type == ty)
    }
}

/// `CraftGroup`: the items are indexes into [`CraftSystem::items`].
#[derive(Clone, Debug)]
pub struct CraftGroup {
    pub name: Td,
    pub items: Vec<usize>,
}

/// `CraftItem`.
#[derive(Clone, Debug)]
pub struct CraftItem {
    pub item_type: &'static str,
    pub group_name: Td,
    pub name: Td,
    /// `_itemId`, -1 until `ItemId` is first read.
    pub item_id: i32,
    pub resources: Vec<CraftRes>,
    pub skills: Vec<CraftSkill>,
    pub force_non_exceptional: bool,
    pub required_expansion: Expansion,
    pub recipe: Option<i32>,
    pub required_beverage: BeverageType,
    pub mana: i32,
    pub need_mana_label: Td,
    pub hits: i32,
    pub need_hits_label: Td,
    pub stam: i32,
    pub need_stam_label: Td,
    pub use_sub_res2: bool,
    pub use_all_res: bool,
    pub need_heat: bool,
    pub need_oven: bool,
    pub need_mill: bool,
    pub item_hue: i32,
}

impl CraftItem {
    fn new(item_type: &'static str, group: Td, name: Td, item_id: i32) -> CraftItem {
        CraftItem {
            item_type,
            group_name: group,
            name,
            item_id,
            resources: Vec::new(),
            skills: Vec::new(),
            force_non_exceptional: false,
            required_expansion: Expansion::None,
            recipe: None,
            required_beverage: BeverageType::Water,
            mana: 0,
            need_mana_label: Td::EMPTY,
            hits: 0,
            need_hits_label: Td::EMPTY,
            stam: 0,
            need_stam_label: Td::EMPTY,
            use_sub_res2: false,
            use_all_res: false,
            need_heat: false,
            need_oven: false,
            need_mill: false,
            item_hue: 0,
        }
    }

    /// `NameNumber`.
    pub fn name_number(&self) -> i32 {
        self.name.number
    }

    /// `NameString`.
    pub fn name_string(&self) -> &str {
        self.name.string.as_deref().unwrap_or("")
    }
}

/// The virtuals of a `CraftSystem` subclass.
pub struct SystemVt {
    /// `GetChanceAtMin(item)`.
    pub chance_at_min: fn(&CraftItem) -> f64,
    /// `CanCraft(from, tool, itemType)`: 0 or the cliloc of what is wrong.
    pub can_craft: fn(&mut World, MobileId, Option<ItemId>, Option<&str>) -> i32,
    /// `PlayCraftEffect(from)`.
    pub play_craft_effect: fn(&mut World, MobileId),
    /// `PlayEndingEffect(from, failed, lostMaterial, toolBroken, quality, makersMark, item)`: the message number.
    pub play_ending_effect: fn(&mut World, MobileId, Ending, &CraftItem) -> i32,
    /// `ConsumeOnFailure(from, resourceType, craftItem)`.
    pub consume_on_failure: fn(&mut World, MobileId, &str, &CraftItem) -> bool,
    /// `RetainsColorFrom(item, type)`.
    pub retains_color_from: fn(&World, &CraftItem, &str) -> bool,
    /// `RequiresTool`.
    pub requires_tool: fn(&World) -> bool,
}

/// The flags `PlayEndingEffect` takes.
#[derive(Copy, Clone, Debug)]
pub struct Ending {
    pub failed: bool,
    pub lost_material: bool,
    pub tool_broken: bool,
    pub quality: i32,
    pub makers_mark: bool,
}

/// `CraftSystem`.
pub struct CraftSystem {
    /// The class name without `Def`: `Blacksmithy`, `Tailoring`..
    pub name: &'static str,
    pub main_skill: SkillName,
    pub min_craft_effect: i32,
    pub max_craft_effect: i32,
    pub delay: f64,
    pub gump_title: Td,
    pub eca: CraftEca,
    pub resmelt: bool,
    pub repair: bool,
    pub mark_option: bool,
    pub can_enhance: bool,
    pub items: Vec<CraftItem>,
    pub groups: Vec<CraftGroup>,
    pub sub_res: SubResCol,
    pub sub_res2: SubResCol,
    pub recipes: Vec<i32>,
    pub rare_recipes: Vec<i32>,
    pub vt: SystemVt,
}

impl CraftSystem {
    pub fn new(
        name: &'static str,
        main_skill: SkillName,
        (min_craft_effect, max_craft_effect, delay): (i32, i32, f64),
        gump_title: Td,
        eca: CraftEca,
        vt: SystemVt,
    ) -> CraftSystem {
        CraftSystem {
            name,
            main_skill,
            min_craft_effect,
            max_craft_effect,
            delay,
            gump_title,
            eca,
            resmelt: false,
            repair: false,
            mark_option: false,
            can_enhance: false,
            items: Vec::new(),
            groups: Vec::new(),
            sub_res: SubResCol::default(),
            sub_res2: SubResCol::default(),
            recipes: Vec::new(),
            rare_recipes: Vec::new(),
            vt,
        }
    }

    /// `CraftItems.SearchFor(type)`.
    pub fn search_for(&self, ty: &str) -> Option<usize> {
        self.items.iter().position(|c| c.item_type == ty)
    }

    /// `AddCraft(typeItem, group, name, [itemId], [skillToMake], minSkill, maxSkill, typeRes, nameRes, amount, message)`.
    #[allow(clippy::too_many_arguments)]
    pub fn add_craft(
        &mut self,
        type_item: &'static str,
        group: Td,
        name: Td,
        item_id: Option<i32>,
        skill_to_make: Option<SkillName>,
        min_skill: f64,
        max_skill: f64,
        type_res: &'static str,
        name_res: Td,
        amount: i32,
        message: Td,
    ) -> usize {
        let skill = skill_to_make.unwrap_or(self.main_skill);
        let mut item = CraftItem::new(type_item, group.clone(), name, item_id.unwrap_or(-1));
        item.resources.push(CraftRes {
            item_type: type_res,
            name: name_res,
            amount,
            message,
        });
        item.skills.push(CraftSkill {
            skill,
            min_skill,
            max_skill,
        });
        let index = self.items.len();
        // DoGroup: the group is found by its number (when set) or its string.
        let found = self.groups.iter().position(|g| {
            (g.name.number != 0 && g.name.number == group.number)
                || (g.name.string.is_some() && g.name.string == group.string)
        });
        match found {
            Some(g) => self.groups[g].items.push(index),
            None => self.groups.push(CraftGroup {
                name: group,
                items: vec![index],
            }),
        }
        self.items.push(item);
        index
    }

    pub fn set_item_hue(&mut self, index: usize, hue: i32) {
        self.items[index].item_hue = hue;
    }

    pub fn set_mana_req(&mut self, index: usize, mana: i32, cliloc: i32) {
        let c = &mut self.items[index];
        c.mana = mana;
        c.need_mana_label = Td::of(cliloc, Some("You lack the required mana to make that."));
    }

    pub fn set_stam_req(&mut self, index: usize, stam: i32, cliloc: i32) {
        let c = &mut self.items[index];
        c.stam = stam;
        c.need_stam_label = Td::of(cliloc, Some("You lack the required stamina to make that."));
    }

    pub fn set_hits_req(&mut self, index: usize, hits: i32, cliloc: i32) {
        let c = &mut self.items[index];
        c.hits = hits;
        c.need_hits_label = Td::of(
            cliloc,
            Some("You lack the required hit points to make that."),
        );
    }

    pub fn set_use_all_res(&mut self, index: usize, v: bool) {
        self.items[index].use_all_res = v;
    }

    pub fn set_need_heat(&mut self, index: usize, v: bool) {
        self.items[index].need_heat = v;
    }

    pub fn set_need_oven(&mut self, index: usize, v: bool) {
        self.items[index].need_oven = v;
    }

    pub fn set_need_mill(&mut self, index: usize, v: bool) {
        self.items[index].need_mill = v;
    }

    pub fn set_beverage_type(&mut self, index: usize, v: BeverageType) {
        self.items[index].required_beverage = v;
    }

    pub fn set_needed_expansion(&mut self, index: usize, v: Expansion) {
        self.items[index].required_expansion = v;
    }

    pub fn set_use_sub_res2(&mut self, index: usize, v: bool) {
        self.items[index].use_sub_res2 = v;
    }

    pub fn force_non_exceptional(&mut self, index: usize) {
        self.items[index].force_non_exceptional = true;
    }

    /// `AddRes(index, type, name, amount, message)`.
    pub fn add_res(&mut self, index: usize, ty: &'static str, name: Td, amount: i32, message: Td) {
        self.items[index].resources.push(CraftRes {
            item_type: ty,
            name,
            amount,
            message,
        });
    }

    /// `AddSkill(index, skill, min, max)`.
    pub fn add_skill(&mut self, index: usize, skill: SkillName, min_skill: f64, max_skill: f64) {
        self.items[index].skills.push(CraftSkill {
            skill,
            min_skill,
            max_skill,
        });
    }

    fn add_recipe_base(&mut self, index: usize, id: i32) {
        // CraftItem.AddRecipe: a second recipe on an item is refused.
        if self.items[index].recipe.is_none() {
            self.items[index].recipe = Some(id);
        }
    }

    pub fn add_recipe(&mut self, index: usize, id: i32) {
        self.add_recipe_base(index, id);
        self.recipes.push(id);
    }

    pub fn add_rare_recipe(&mut self, index: usize, id: i32) {
        self.add_recipe_base(index, id);
        self.rare_recipes.push(id);
    }

    pub fn add_quest_recipe(&mut self, index: usize, id: i32) {
        self.add_recipe_base(index, id);
    }

    pub fn set_sub_res(&mut self, ty: &'static str, name: Td) {
        self.sub_res.res_type = ty;
        self.sub_res.name = name;
        self.sub_res.init = true;
    }

    pub fn add_sub_res(
        &mut self,
        ty: &'static str,
        name: Td,
        req_skill: f64,
        generic_name: i32,
        message: Td,
    ) {
        self.sub_res.items.push(CraftSubRes {
            item_type: ty,
            name,
            required_skill: req_skill,
            generic_name_number: generic_name,
            message,
        });
    }

    pub fn set_sub_res2(&mut self, ty: &'static str, name: Td) {
        self.sub_res2.res_type = ty;
        self.sub_res2.name = name;
        self.sub_res2.init = true;
    }

    pub fn add_sub_res2(
        &mut self,
        ty: &'static str,
        name: Td,
        req_skill: f64,
        generic_name: i32,
        message: Td,
    ) {
        self.sub_res2.items.push(CraftSubRes {
            item_type: ty,
            name,
            required_skill: req_skill,
            generic_name_number: generic_name,
            message,
        });
    }
}

/// `CraftMarkOption`.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Default)]
pub enum CraftMarkOption {
    #[default]
    MarkItem,
    DoNotMark,
    PromptForMark,
}

/// `CraftContext`: per player and system. `items` are indexes into the system's items (most recent first).
#[derive(Clone, Debug)]
pub struct CraftContext {
    pub items: Vec<usize>,
    pub last_resource_index: i32,
    pub last_resource_index2: i32,
    pub last_group_index: i32,
    pub do_not_color: bool,
    pub mark_option: CraftMarkOption,
    /// T2A: last hue used for hue-aware crafting.
    pub last_hue: i32,
    /// T2A jewelry: `PendingGemType` and `PendingGemCount`.
    pub pending_gem: Option<(muo_content::items::jewel::GemType, i32)>,
}

impl Default for CraftContext {
    fn default() -> Self {
        CraftContext {
            items: Vec::new(),
            last_resource_index: -1,
            last_resource_index2: -1,
            last_group_index: -1,
            do_not_color: false,
            mark_option: CraftMarkOption::MarkItem,
            last_hue: -1,
            pending_gem: None,
        }
    }
}

impl CraftContext {
    /// `LastMade`.
    pub fn last_made(&self) -> Option<usize> {
        self.items.first().copied()
    }

    /// `OnMade(item)`: most recent first, ten at most.
    pub fn on_made(&mut self, item: usize) {
        self.items.retain(|&i| i != item);
        if self.items.len() == 10 {
            self.items.remove(9);
        }
        self.items.insert(0, item);
    }
}

/// `Dictionary<Mobile, CraftContext>` of one system.
pub type Contexts = HashMap<MobileId, CraftContext>;
