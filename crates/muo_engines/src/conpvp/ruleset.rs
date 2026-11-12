//! Rulesets: the option tree (per expansion), the bit array of a ruleset, the
//! default rulesets and the flavors.

use std::sync::{Arc, OnceLock};

use muo_server::World;

/// `RulesetLayout`: a node of the option tree. `parent`/`children` are indexes into [`RulesetLayout::nodes`].
#[derive(Clone, Debug)]
pub struct LayoutNode {
    pub title: String,
    pub description: String,
    pub options: Vec<String>,
    pub children: Vec<usize>,
    pub parent: Option<usize>,
    pub offset: usize,
    pub total_length: usize,
}

/// The whole tree (the last node is `Root`, "Rules"), its `Defaults` and its `Flavors`.
#[derive(Debug)]
pub struct RulesetLayout {
    pub nodes: Vec<LayoutNode>,
    pub defaults: Vec<Ruleset>,
    pub flavors: Vec<Ruleset>,
}

struct Builder {
    nodes: Vec<LayoutNode>,
}

impl Builder {
    fn leaf(&mut self, title: &str, description: &str, options: &[&str]) -> usize {
        self.node(title, description, &[], options)
    }

    fn node(
        &mut self,
        title: &str,
        description: &str,
        children: &[usize],
        options: &[&str],
    ) -> usize {
        let id = self.nodes.len();
        for &c in children {
            self.nodes[c].parent = Some(id);
        }
        self.nodes.push(LayoutNode {
            title: title.to_owned(),
            description: description.to_owned(),
            options: options.iter().map(|s| (*s).to_owned()).collect(),
            children: children.to_vec(),
            parent: None,
            offset: 0,
            total_length: 0,
        });
        id
    }
}

const SPELL_CIRCLES: [(&str, [&str; 8]); 8] = [
    (
        "1st Circle",
        [
            "Reactive Armor",
            "Clumsy",
            "Create Food",
            "Feeblemind",
            "Heal",
            "Magic Arrow",
            "Night Sight",
            "Weaken",
        ],
    ),
    (
        "2nd Circle",
        [
            "Agility",
            "Cunning",
            "Cure",
            "Harm",
            "Magic Trap",
            "Untrap",
            "Protection",
            "Strength",
        ],
    ),
    (
        "3rd Circle",
        [
            "Bless",
            "Fireball",
            "Magic Lock",
            "Poison",
            "Telekinesis",
            "Teleport",
            "Unlock Spell",
            "Wall of Stone",
        ],
    ),
    (
        "4th Circle",
        [
            "Arch Cure",
            "Arch Protection",
            "Curse",
            "Fire Field",
            "Greater Heal",
            "Lightning",
            "Mana Drain",
            "Recall",
        ],
    ),
    (
        "5th Circle",
        [
            "Blade Spirits",
            "Dispel Field",
            "Incognito",
            "Magic Reflection",
            "Mind Blast",
            "Paralyze",
            "Poison Field",
            "Summon Creature",
        ],
    ),
    (
        "6th Circle",
        [
            "Dispel",
            "Energy Bolt",
            "Explosion",
            "Invisibility",
            "Mark",
            "Mass Curse",
            "Paralyze Field",
            "Reveal",
        ],
    ),
    (
        "7th Circle",
        [
            "Chain Lightning",
            "Energy Field",
            "Flame Strike",
            "Gate Travel",
            "Mana Vampire",
            "Mass Dispel",
            "Meteor Swarm",
            "Polymorph",
        ],
    ),
    (
        "8th Circle",
        [
            "Earthquake",
            "Energy Vortex",
            "Resurrection",
            "Air Elemental",
            "Summon Daemon",
            "Earth Elemental",
            "Fire Elemental",
            "Water Elemental",
        ],
    ),
];

const POTIONS: [&str; 8] = [
    "Agility",
    "Cure",
    "Explosion",
    "Heal",
    "Nightsight",
    "Poison",
    "Refresh",
    "Strength",
];

impl RulesetLayout {
    /// The tree of `Root` for an expansion (`Core.AOS`, `Core.SE`, `Core.ML`), without defaults and flavors.
    pub fn build(aos: bool, se: bool, ml: bool) -> RulesetLayout {
        let mut b = Builder { nodes: Vec::new() };
        let mut entries = Vec::new();

        let circles: Vec<usize> = SPELL_CIRCLES
            .iter()
            .map(|(t, o)| b.leaf(t, "Spells", o))
            .collect();
        entries.push(b.node("Spells", "Spells", &circles, &[]));

        if aos {
            entries.push(b.leaf(
                "Chivalry",
                "Chivalry",
                &[
                    "Cleanse by Fire",
                    "Close Wounds",
                    "Consecrate Weapon",
                    "Dispel Evil",
                    "Divine Fury",
                    "Enemy of One",
                    "Holy Light",
                    "Noble Sacrifice",
                    "Remove Curse",
                    "Sacred Journey",
                ],
            ));
            entries.push(b.leaf(
                "Necromancy",
                "Necromancy",
                &[
                    "Animate Dead",
                    "Blood Oath",
                    "Corpse Skin",
                    "Curse Weapon",
                    "Evil Omen",
                    "Horrific Beast",
                    "Lich Form",
                    "Mind Rot",
                    "Pain Spike",
                    "Poison Strike",
                    "Strangle",
                    "Summon Familiar",
                    "Vampiric Embrace",
                    "Vengeful Spirit",
                    "Wither",
                    "Wraith Form",
                ],
            ));
            if se {
                entries.push(b.leaf(
                    "Bushido",
                    "Bushido",
                    &[
                        "Confidence",
                        "Counter Attack",
                        "Evasion",
                        "Honorable Execution",
                        "Lightning Strike",
                        "Momentum Strike",
                    ],
                ));
                entries.push(b.leaf(
                    "Ninjitsu",
                    "Ninjitsu",
                    &[
                        "Animal Form",
                        "Backstab",
                        "Death Strike",
                        "Focus Attack",
                        "Ki Attack",
                        "Mirror Image",
                        "Shadow Jump",
                        "Suprise Attack",
                    ],
                ));
                if ml {
                    entries.push(b.leaf(
                        "Spellweaving",
                        "Spellweaving",
                        &[
                            "Arcane Circle",
                            "Arcane Empowerment",
                            "Attune Weapon",
                            "Dryad Allure",
                            "Essence of Wind",
                            "Ethereal Voyage",
                            "Gift of Life",
                            "Gift of Renewal",
                            "Immolating Weapon",
                            "Nature's Fury",
                            "Reaper Form",
                            "Summon Fey",
                            "Summon Fiend",
                            "Thunderstorm",
                            "Wildfire",
                            "Word of Death",
                        ],
                    ));
                }
            }
        }

        let combat: &[&str] = if aos && se {
            &[
                "Stun",
                "Disarm",
                "Armor Ignore",
                "Bleed Attack",
                "Concussion Blow",
                "Crushing Blow",
                "Disarm",
                "Dismount",
                "Double Strike",
                "Infectious Strike",
                "Mortal Strike",
                "Moving Shot",
                "Paralyzing Blow",
                "Shadow Strike",
                "Whirlwind Attack",
                "Riding Swipe",
                "Frenzied Whirlwind",
                "Block",
                "Defense Mastery",
                "Nerve Strike",
                "Talon Strike",
                "Feint",
                "Dual Wield",
                "Double Shot",
                "Armor Pierce",
            ]
        } else if aos {
            &[
                "Stun",
                "Disarm",
                "Armor Ignore",
                "Bleed Attack",
                "Concussion Blow",
                "Crushing Blow",
                "Disarm",
                "Dismount",
                "Double Strike",
                "Infectious Strike",
                "Mortal Strike",
                "Moving Shot",
                "Paralyzing Blow",
                "Shadow Strike",
                "Whirlwind Attack",
            ]
        } else {
            &[
                "Stun",
                "Disarm",
                "Concussion Blow",
                "Crushing Blow",
                "Paralyzing Blow",
            ]
        };
        entries.push(b.leaf("Combat Abilities", "Combat Abilities", combat));

        entries.push(b.leaf(
            "Skills",
            "Skills",
            &[
                "Anatomy",
                "Detect Hidden",
                "Evaluating Intelligence",
                "Hiding",
                "Poisoning",
                "Snooping",
                "Stealing",
                "Spirit Speak",
                "Stealth",
            ],
        ));

        if aos {
            entries.push(b.leaf(
                "Weapons",
                "Weapons",
                &["Magical", "Melee", "Ranged", "Poisoned", "Wrestling"],
            ));
            entries.push(b.leaf("Armor", "Armor", &["Magical", "Shields"]));
        } else {
            entries.push(b.leaf(
                "Weapons",
                "Weapons",
                &[
                    "Magical",
                    "Melee",
                    "Ranged",
                    "Poisoned",
                    "Wrestling",
                    "Runics",
                ],
            ));
            entries.push(b.leaf("Armor", "Armor", &["Magical", "Shields", "Colored"]));
        }

        let potions = b.leaf("Potions", "Potions", &POTIONS);
        let items: &[&str] = if se {
            &[
                "Bandages",
                "Wands",
                "Trapped Containers",
                "Bolas",
                "Mounts",
                "Orange Petals",
                "Shurikens",
                "Fukiya Darts",
                "Fire Horns",
            ]
        } else {
            &[
                "Bandages",
                "Wands",
                "Trapped Containers",
                "Bolas",
                "Mounts",
                "Orange Petals",
                "Fire Horns",
            ]
        };
        entries.push(b.node("Items", "Items", &[potions], items));

        let root = b.node("Rules", "Rules", &entries, &[]);
        let mut layout = RulesetLayout {
            nodes: b.nodes,
            defaults: Vec::new(),
            flavors: Vec::new(),
        };
        let mut offset = 0;
        layout.compute_offsets(root, &mut offset);
        layout
    }

    /// `Root`'s index.
    pub fn root_id(&self) -> usize {
        self.nodes.len() - 1
    }

    fn compute_offsets(&mut self, n: usize, offset: &mut usize) -> usize {
        self.nodes[n].offset = *offset;
        *offset += self.nodes[n].options.len();
        self.nodes[n].total_length += self.nodes[n].options.len();
        for c in self.nodes[n].children.clone() {
            let t = self.compute_offsets(c, offset);
            self.nodes[n].total_length += t;
        }
        self.nodes[n].total_length
    }

    /// `Root.TotalLength`.
    pub fn total_length(&self) -> usize {
        self.nodes[self.root_id()].total_length
    }

    /// `FindByTitle(title)`.
    pub fn find_by_title(&self, title: &str) -> Option<usize> {
        self.find_by_title_from(self.root_id(), title)
    }

    fn find_by_title_from(&self, n: usize, title: &str) -> Option<usize> {
        if self.nodes[n].title == title {
            return Some(n);
        }
        self.nodes[n]
            .children
            .iter()
            .find_map(|&c| self.find_by_title_from(c, title))
    }

    /// `FindByIndex(index)`: "Description: Option".
    pub fn find_by_index(&self, index: usize) -> Option<String> {
        self.find_by_index_from(self.root_id(), index)
    }

    fn find_by_index_from(&self, n: usize, index: usize) -> Option<String> {
        let node = &self.nodes[n];
        if index >= node.offset && index < node.offset + node.options.len() {
            return Some(format!(
                "{}: {}",
                node.description,
                node.options[index - node.offset]
            ));
        }
        node.children
            .iter()
            .find_map(|&c| self.find_by_index_from(c, index))
    }

    /// `FindByOption(title, option, ref index)`: the node and the option's index in it. A node whose title matches
    /// but has no such option lets its children (and later siblings) match any title.
    pub fn find_by_option(&self, title: Option<&str>, option: &str) -> Option<(usize, usize)> {
        self.find_by_option_from(self.root_id(), title, option)
    }

    fn find_by_option_from(
        &self,
        n: usize,
        mut title: Option<&str>,
        option: &str,
    ) -> Option<(usize, usize)> {
        let node = &self.nodes[n];
        if title.is_none_or(|t| node.title == t) {
            if let Some(i) = node.options.iter().position(|o| o == option) {
                return Some((n, i));
            }
            title = None;
        }
        node.children
            .iter()
            .find_map(|&c| self.find_by_option_from(c, title, option))
    }
}

/// `Ruleset`: a bit array over a layout.
#[derive(Clone, Debug)]
pub struct Ruleset {
    pub layout: Arc<RulesetLayout>,
    pub options: Vec<bool>,
    pub title: Option<String>,
    pub changed: bool,
    /// `Base`: an index into `layout.defaults`.
    pub base: Option<usize>,
    /// `Flavors`: indexes into `layout.flavors`.
    pub flavors: Vec<usize>,
}

static ROOT: OnceLock<Arc<RulesetLayout>> = OnceLock::new();

/// `RulesetLayout.Root` for the world's expansion, built once (with its default rulesets and flavors).
pub fn root(w: &World) -> Arc<RulesetLayout> {
    ROOT.get_or_init(|| {
        build_root(
            muo_content::core::aos(w),
            muo_content::core::se(w),
            muo_content::core::ml(w),
        )
    })
    .clone()
}

impl Default for Ruleset {
    /// A ruleset over the layout built so far (every expansion's options when none was built: only tests get here).
    fn default() -> Self {
        Ruleset::new(ROOT.get_or_init(|| build_root(true, true, true)).clone())
    }
}

const M5X_OFF: [&str; 31] = [
    "Wall of Stone",
    "Fire Field",
    "Poison Field",
    "Energy Field",
    "Reactive Armor",
    "Protection",
    "Teleport",
    "Wall of Stone",
    "Arch Protection",
    "Recall",
    "Blade Spirits",
    "Incognito",
    "Magic Reflection",
    "Paralyze",
    "Summon Creature",
    "Invisibility",
    "Mark",
    "Paralyze Field",
    "Energy Field",
    "Gate Travel",
    "Polymorph",
    "Energy Vortex",
    "Air Elemental",
    "Summon Daemon",
    "Earth Elemental",
    "Fire Elemental",
    "Water Elemental",
    "Earthquake",
    "Meteor Swarm",
    "Chain Lightning",
    "Resurrection",
];

fn spells_off(r: &mut Ruleset, skip: &[&str]) {
    for o in M5X_OFF.iter().filter(|o| !skip.contains(o)) {
        r.set_option("Spells", o, false);
    }
}

fn build_root(aos: bool, se: bool, ml: bool) -> Arc<RulesetLayout> {
    let base = Arc::new(RulesetLayout::build(aos, se, ml));
    let mk = |title: &str| {
        let mut r = Ruleset::new(base.clone());
        r.title = Some(title.to_owned());
        r
    };

    let defaults = if !aos {
        let mut m5x = mk("Mage 5x");
        m5x.set_option_range("Spells", true);
        spells_off(&mut m5x, &[]);
        m5x.set_option("Weapons", "Wrestling", true);
        m5x.set_option("Skills", "Anatomy", true);
        m5x.set_option("Skills", "Detect Hidden", true);
        m5x.set_option("Skills", "Evaluating Intelligence", true);
        m5x.set_option("Items", "Trapped Containers", true);

        let mut m7x = mk("Mage 7x");
        m7x.set_option_range("Spells", true);
        spells_off(&mut m7x, &[]);
        m7x.set_option("Combat Abilities", "Stun", true);
        m7x.set_option("Skills", "Anatomy", true);
        m7x.set_option("Skills", "Detect Hidden", true);
        m7x.set_option("Skills", "Poisoning", true);
        m7x.set_option("Skills", "Evaluating Intelligence", true);
        m7x.set_option("Weapons", "Wrestling", true);
        m7x.set_option("Potions", "Refresh", true);
        m7x.set_option("Items", "Trapped Containers", true);
        m7x.set_option("Items", "Bandages", true);

        let mut s7x = mk("Standard 7x");
        s7x.set_option_range("Spells", true);
        spells_off(&mut s7x, &["Reactive Armor", "Protection"]);
        s7x.set_option_range("Combat Abilities", true);
        s7x.set_option("Skills", "Anatomy", true);
        s7x.set_option("Skills", "Detect Hidden", true);
        s7x.set_option("Skills", "Poisoning", true);
        s7x.set_option("Skills", "Evaluating Intelligence", true);
        s7x.set_option_range("Weapons", true);
        s7x.set_option("Weapons", "Runics", false);
        s7x.set_option_range("Armor", true);
        s7x.set_option("Potions", "Refresh", true);
        s7x.set_option("Items", "Bandages", true);
        s7x.set_option("Items", "Trapped Containers", true);
        vec![m5x, m7x, s7x]
    } else {
        let mut all = mk("Standard All Skills");
        all.set_option_range("Spells", true);
        spells_off(&mut all, &["Reactive Armor", "Protection"]);
        all.set_option_range("Necromancy", true);
        for o in [
            "Summon Familiar",
            "Vengeful Spirit",
            "Animate Dead",
            "Wither",
            "Poison Strike",
        ] {
            all.set_option("Necromancy", o, false);
        }
        all.set_option_range("Chivalry", true);
        for o in ["Sacred Journey", "Enemy of One", "Noble Sacrifice"] {
            all.set_option("Chivalry", o, false);
        }
        all.set_option_range("Combat Abilities", true);
        all.set_option("Combat Abilities", "Paralyzing Blow", false);
        all.set_option("Combat Abilities", "Shadow Strike", false);
        for o in [
            "Anatomy",
            "Detect Hidden",
            "Poisoning",
            "Spirit Speak",
            "Evaluating Intelligence",
        ] {
            all.set_option("Skills", o, true);
        }
        all.set_option_range("Weapons", true);
        all.set_option("Weapons", "Poisoned", false);
        all.set_option_range("Armor", true);
        all.set_option_range("Ninjitsu", true);
        for o in [
            "Animal Form",
            "Mirror Image",
            "Backstab",
            "Suprise Attack",
            "Shadow Jump",
        ] {
            all.set_option("Ninjitsu", o, false);
        }
        all.set_option_range("Bushido", true);
        all.set_option_range("Spellweaving", true);
        for o in [
            "Gift of Life",
            "Summon Fey",
            "Summon Fiend",
            "Nature's Fury",
        ] {
            all.set_option("Spellweaving", o, false);
        }
        all.set_option("Potions", "Refresh", true);
        all.set_option("Items", "Bandages", true);
        all.set_option("Items", "Trapped Containers", true);
        vec![all]
    };

    let mut pots = mk("Potions");
    pots.set_option_range("Potions", true);
    pots.set_option("Potions", "Explosion", false);
    let mut para = mk("Paralyze");
    para.set_option("Spells", "Paralyze", true);
    para.set_option("Spells", "Paralyze Field", true);
    para.set_option("Combat Abilities", "Paralyzing Blow", true);
    let mut fields = mk("Fields");
    for o in [
        "Wall of Stone",
        "Fire Field",
        "Poison Field",
        "Energy Field",
        "Wildfire",
    ] {
        fields.set_option("Spells", o, true);
    }
    let mut area = mk("Area Effect");
    for o in ["Earthquake", "Meteor Swarm", "Chain Lightning"] {
        area.set_option("Spells", o, true);
    }
    area.set_option("Necromancy", "Wither", true);
    area.set_option("Necromancy", "Poison Strike", true);
    let mut summons = mk("Summons");
    for o in [
        "Blade Spirits",
        "Energy Vortex",
        "Air Elemental",
        "Summon Daemon",
        "Earth Elemental",
        "Fire Elemental",
        "Water Elemental",
    ] {
        summons.set_option("Spells", o, true);
    }
    for o in ["Summon Familiar", "Vengeful Spirit", "Animate Dead"] {
        summons.set_option("Necromancy", o, true);
    }
    summons.set_option("Ninjitsu", "Mirror Image", true);
    for o in ["Summon Fey", "Summon Fiend", "Nature's Fury"] {
        summons.set_option("Spellweaving", o, true);
    }

    // Defaults and flavors hold the bare tree (the same offsets); the tree the world sees carries them.
    Arc::new(RulesetLayout {
        nodes: base.nodes.clone(),
        defaults,
        flavors: vec![pots, para, fields, area, summons],
    })
}

impl Ruleset {
    /// `new Ruleset(layout)`.
    pub fn new(layout: Arc<RulesetLayout>) -> Ruleset {
        Ruleset {
            options: vec![false; layout.total_length()],
            layout,
            title: None,
            changed: false,
            base: None,
            flavors: Vec::new(),
        }
    }

    /// `ApplyDefault(newDefault)`: `newDefault` is an index into `layout.defaults`.
    pub fn apply_default(&mut self, idx: usize) {
        self.base = Some(idx);
        self.changed = false;
        self.options = self.layout.defaults[idx].options.clone();
        self.apply_flavors();
    }

    /// `ApplyFlavorsTo(this)`.
    pub fn apply_flavors(&mut self) {
        for &f in &self.flavors {
            for (o, v) in self.options.iter_mut().zip(&self.layout.flavors[f].options) {
                *o |= *v;
            }
        }
    }

    /// `AddFlavor(flavor)`.
    pub fn add_flavor(&mut self, f: usize) {
        if self.flavors.contains(&f) {
            return;
        }
        self.flavors.push(f);
        for (o, v) in self.options.iter_mut().zip(&self.layout.flavors[f].options) {
            *o |= *v;
        }
    }

    /// `RemoveFlavor(flavor)`: the flavor's bits are cleared.
    pub fn remove_flavor(&mut self, f: usize) {
        let Some(p) = self.flavors.iter().position(|x| *x == f) else {
            return;
        };
        self.flavors.remove(p);
        for (o, v) in self.options.iter_mut().zip(&self.layout.flavors[f].options) {
            if *v {
                *o = false;
            }
        }
    }

    /// `SetOptionRange(title, value)`.
    pub fn set_option_range(&mut self, title: &str, value: bool) {
        let Some(n) = self.layout.find_by_title(title) else {
            return;
        };
        let node = &self.layout.nodes[n];
        for i in 0..node.total_length {
            self.options[i + node.offset] = value;
        }
        self.changed = true;
    }

    /// `GetOption(title, option)`: true when no layout names the option.
    pub fn get_option(&self, title: &str, option: &str) -> bool {
        match self.layout.find_by_option(Some(title), option) {
            None => true,
            Some((n, i)) => self.options[self.layout.nodes[n].offset + i],
        }
    }

    /// `SetOption(title, option, value)`.
    pub fn set_option(&mut self, title: &str, option: &str, value: bool) {
        let Some((n, i)) = self.layout.find_by_option(Some(title), option) else {
            return;
        };
        let at = self.layout.nodes[n].offset + i;
        self.options[at] = value;
        self.changed = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pvp_ruleset_layout_and_defaults() {
        let l = build_root(true, true, true);
        assert_eq!(l.nodes[l.root_id()].title, "Rules");
        let mut r = Ruleset::new(l.clone());
        r.apply_default(0);
        assert!(!r.get_option("Spells", "Recall"));
        assert!(r.get_option("Spells", "Heal"));
        assert!(r.get_option("Potions", "Refresh"));
        assert!(!r.get_option("Combat Abilities", "Paralyzing Blow"));
        assert!(
            r.get_option("Nonexistent", "Option"),
            "unnamed options are allowed"
        );
        r.add_flavor(1);
        assert!(r.get_option("Combat Abilities", "Paralyzing Blow"));
        r.remove_flavor(1);
        assert!(!r.get_option("Spells", "Paralyze"));
        assert!(l.find_by_index(0).unwrap().starts_with("Spells: "));
    }
}
