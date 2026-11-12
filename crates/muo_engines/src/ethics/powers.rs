//! The ethics powers. The Hero and Evil powers of a kind are the
//! same code with other texts, so one table row per power names its side.

use muo_content::spells::helper::{self as spell_helper, Point3dRef};
use muo_primitives::{Direction, Serial, TimeSpan};
use muo_server::World;
use muo_server::effects::{EffectLayer, play_sound as effects_play_sound};
use muo_server::ids::{EntityId, MobileId};
use muo_server::mobile::part2::{effects as fx, enumerators, messages as mm};
use muo_server::mobile::{StatType, base as mb};
use muo_server::mods::StatMod;
use muo_server::target::{self, TargetFlags, Targeted};

use super::{EthicKind, begin_shield, is_shielded, player_mobile, say, st, st_mut};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum PowerKind {
    Sense,
    Item,
    Familiar,
    Blade,
    Bless,
    Shield,
    Steed,
    Word,
}

/// A `Power` and its `PowerDefinition` (the name and description are not used by any code).
#[derive(Debug)]
pub struct Power {
    pub side: EthicKind,
    pub kind: PowerKind,
    pub name: &'static str,
    pub power: i32,
    pub phrase: &'static str,
}

const fn p(
    side: EthicKind,
    kind: PowerKind,
    name: &'static str,
    power: i32,
    phrase: &'static str,
) -> Power {
    Power {
        side,
        kind,
        name,
        power,
        phrase,
    }
}

use EthicKind::{Evil, Hero};
use PowerKind::*;

pub static HERO_POWERS: [Power; 8] = [
    p(Hero, Sense, "Holy Sense", 0, "Drewrok Erstok"),
    p(Hero, Item, "Holy Item", 5, "Vidda K'balc"),
    p(Hero, Familiar, "Summon Familiar", 5, "Trubechs Vingir"),
    p(Hero, Blade, "Holy Blade", 10, "Erstok Reyam"),
    p(Hero, Bless, "Bless", 15, "Erstok Ontawl"),
    p(Hero, Shield, "Holy Shield", 20, "Erstok K'blac"),
    p(Hero, Steed, "Holy Steed", 30, "Trubechs Yeliab"),
    p(Hero, Word, "Holy Word", 100, "Erstok Oostrac"),
];

pub static EVIL_POWERS: [Power; 8] = [
    p(Evil, Sense, "Unholy Sense", 0, "Drewrok Velgo"),
    p(Evil, Item, "Unholy Item", 5, "Vidda K'balc"),
    p(Evil, Familiar, "Summon Familiar", 5, "Trubechs Vingir"),
    p(Evil, Blade, "Vile Blade", 10, "Velgo Reyam"),
    p(Evil, Bless, "Blight", 15, "Velgo Ontawl"),
    p(Evil, Shield, "Unholy Shield", 20, "Velgo K'blac"),
    p(Evil, Steed, "Unholy Steed", 30, "Trubechs Yeliab"),
    p(Evil, Word, "Unholy Word", 100, "Velgo Oostrac"),
];

fn hero_side(side: EthicKind) -> bool {
    side == EthicKind::Hero
}

impl Power {
    /// `CheckInvoke(from)`.
    pub fn check_invoke(&self, w: &mut World, pl: Serial) -> bool {
        let Some(m) = player_mobile(w, pl) else {
            return false;
        };
        if !muo_server::mobile::part2::stats::check_alive(w, m, true) {
            return false;
        }
        if super::power(w, pl) < self.power {
            say(w, m, "You lack the power to invoke this ability.");
            return false;
        }
        true
    }

    /// `FinishInvoke(from)`.
    pub fn finish_invoke(&self, w: &mut World, pl: Serial) {
        if let Some(p) = st_mut(w).player_mut(pl) {
            p.power -= self.power;
        }
    }

    /// `BeginInvoke(from)`.
    pub fn begin_invoke(&'static self, w: &mut World, pl: Serial) {
        let Some(m) = player_mobile(w, pl) else {
            return;
        };
        match self.kind {
            Sense => self.sense(w, pl, m),
            Item => {
                target::begin_target(w, m, 12, false, TargetFlags::NONE, move |w, _from, obj| {
                    self.item_target(w, pl, m, obj)
                });
                mm::send_message(w, m, "Which item do you wish to imbue?");
            }
            Familiar => self.summon_familiar(w, pl, m),
            Blade | Word => {}
            Bless => {
                target::begin_target(w, m, 12, true, TargetFlags::NONE, move |w, _from, obj| {
                    self.area_target(w, pl, m, obj)
                });
                let text = if hero_side(self.side) {
                    "Where do you wish to bless?"
                } else {
                    "Where do you wish to blight?"
                };
                mm::send_message(w, m, text);
            }
            Shield => {
                if is_shielded(w, pl) {
                    let text = if hero_side(self.side) {
                        "You are already under the protection of a holy shield."
                    } else {
                        "You are already under the protection of an unholy shield."
                    };
                    say(w, m, text);
                    return;
                }
                begin_shield(w, pl);
                let text = if hero_side(self.side) {
                    "You are now under the protection of a holy shield."
                } else {
                    "You are now under the protection of an unholy shield."
                };
                say(w, m, text);
                self.finish_invoke(w, pl);
            }
            Steed => self.summon_steed(w, pl, m),
        }
    }

    /// `HolySense` / `UnholySense`. The opposition is `Ethic.Evil` / `Ethic.Hero`, null: the power fails before
    /// any message (nothing happens, and `FinishInvoke` is not reached).
    fn sense(&self, _w: &mut World, _pl: Serial, _m: MobileId) {
        let opposition: Option<Serial> = if hero_side(self.side) {
            super::evil()
        } else {
            super::hero()
        };
        let Some(_opposition) = opposition else {
            return;
        };
        // unreachable while the ethic statics are null
    }

    /// `HolyItem.Power_OnTarget` / `UnholyItem.Power_OnTarget`.
    fn item_target(&self, w: &mut World, pl: Serial, m: MobileId, obj: Targeted) {
        let Targeted::Entity(EntityId::Item(item)) = obj else {
            say(w, m, "You may not imbue that.");
            return;
        };
        if w.item(item).and_then(|i| i.parent) != Some(EntityId::Mobile(m)) {
            say(w, m, "You may only imbue items you are wearing.");
            return;
        }
        if w.item(item).is_some_and(|i| i.saved_flags & 0x300 != 0) {
            let text = if hero_side(self.side) {
                "That has already beem imbued."
            } else {
                "That has already been imbued."
            };
            say(w, m, text);
            return;
        }
        let can_imbue = w.item(item).is_some_and(|i| {
            (i.ty.is(&muo_content::items::armor::BASE_ARMOR)
                || i.ty.is(&muo_content::items::clothing::BASE_CLOTHING)
                || i.ty.is(&muo_content::items::weapon::BASE_WEAPON)
                || muo_content::factory::item_is(w, item, "Server.Items.Spellbook"))
                && i.name.is_none()
        });
        if can_imbue {
            if !self.check_invoke(w, pl) {
                return;
            }
            // `Ethic.Hero.Definition.PrimaryHue`: Hero / Evil are null, so the power fails here.
            let ethic = if hero_side(self.side) {
                super::hero()
            } else {
                super::evil()
            };
            if ethic.is_none() {
                return;
            }
            muo_server::item::base::set_hue(w, item, self.side.primary_hue());
            if let Some(i) = w.item_mut(item) {
                i.saved_flags |= if hero_side(self.side) { 0x100 } else { 0x200 };
            }
            fx::fixed_effect(w, m, 0x375A, 10, 20);
            fx::play_sound(w, m, 0x209);
            self.finish_invoke(w, pl);
        } else {
            say(w, m, "You may not imbue that.");
        }
    }

    /// `SummonFamiliar.BeginInvoke`.
    fn summon_familiar(&self, w: &mut World, pl: Serial, m: MobileId) {
        let (name, ctor): (&str, fn(&mut World) -> MobileId) = if hero_side(self.side) {
            ("holy", super::mobiles::new_holy_familiar)
        } else {
            ("unholy", super::mobiles::new_unholy_familiar)
        };
        let familiar = st(w).and_then(|s| s.player(pl)).and_then(|p| p.familiar);
        if familiar.is_some_and(|f| w.mobile(f).is_none())
            && let Some(p) = st_mut(w).player_mut(pl)
        {
            p.familiar = None;
        }
        if st(w)
            .and_then(|s| s.player(pl))
            .and_then(|p| p.familiar)
            .is_some()
        {
            let article = if hero_side(self.side) { "a" } else { "an" };
            say(
                w,
                m,
                &format!("You already have {article} {name} familiar."),
            );
            return;
        }
        let (followers, max) = w
            .mobile(m)
            .map_or((0, 0), |x| (x.followers, x.followers_max));
        if followers + 1 > max {
            mm::send_localized_message(w, m, 1049645, "", 0x3B2); // You have too many followers to summon that creature.
            return;
        }
        let creature = ctor(w);
        let loc = w.mobile(m).map(|x| x.location).unwrap_or_default();
        if muo_content::creature::control::summon(
            w,
            creature,
            true,
            m,
            loc,
            0x217,
            TimeSpan::from_hours(1),
            None,
        ) {
            if let Some(p) = st_mut(w).player_mut(pl) {
                p.familiar = Some(creature);
            }
            self.finish_invoke(w, pl);
        }
    }

    /// `HolySteed.BeginInvoke` / `UnholySteed.BeginInvoke`.
    fn summon_steed(&self, w: &mut World, pl: Serial, m: MobileId) {
        let steed = st(w).and_then(|s| s.player(pl)).and_then(|p| p.steed);
        if steed.is_some_and(|s| w.mobile(s).is_none())
            && let Some(p) = st_mut(w).player_mut(pl)
        {
            p.steed = None;
        }
        if st(w)
            .and_then(|s| s.player(pl))
            .and_then(|p| p.steed)
            .is_some()
        {
            let text = if hero_side(self.side) {
                "You already have a holy steed."
            } else {
                "You already have an unholy steed."
            };
            say(w, m, text);
            return;
        }
        let (followers, max) = w
            .mobile(m)
            .map_or((0, 0), |x| (x.followers, x.followers_max));
        if followers + 1 > max {
            mm::send_localized_message(w, m, 1049645, "", 0x3B2); // You have too many followers to summon that creature.
            return;
        }
        let steed = if hero_side(self.side) {
            super::mobiles::new_holy_steed(w)
        } else {
            super::mobiles::new_unholy_steed(w)
        };
        let loc = w.mobile(m).map(|x| x.location).unwrap_or_default();
        if muo_content::creature::control::summon(
            w,
            steed,
            true,
            m,
            loc,
            0x217,
            TimeSpan::from_hours(1),
            None,
        ) {
            if let Some(p) = st_mut(w).player_mut(pl) {
                p.steed = Some(steed);
            }
            self.finish_invoke(w, pl);
        }
    }

    /// `Bless.Power_OnTarget` / `Blight.Power_OnTarget`.
    fn area_target(&self, w: &mut World, pl: Serial, m: MobileId, obj: Targeted) {
        let p: Point3dRef = obj.into();
        if !self.check_invoke(w, pl) {
            return;
        }
        let blight = !hero_side(self.side);
        let p = spell_helper::get_surface_top(w, p);
        let mut functioned = false;

        for mob in enumerators::get_mobiles_in_range(w, m, 6) {
            if blight {
                if mob == m || !spell_helper::valid_indirect_target(w, m, mob) {
                    continue;
                }
            } else if mob != m && spell_helper::valid_indirect_target(w, m, mob) {
                // as written: Bless skips everyone it may validly target but the caster
                continue;
            }
            let name = if blight { "Holy Curse" } else { "Holy Bless" };
            if mb::get_stat_mod(w, mob, name, false).is_some() {
                continue;
            }
            if blight {
                if !mb::can_be_harmful_msg(w, m, mob, false) {
                    continue;
                }
                mb::do_harmful(w, m, mob, true);
            } else {
                if !mb::can_be_beneficial_msg(w, m, mob, false) {
                    continue;
                }
                mb::do_beneficial(w, m, mob);
            }
            let offset = if blight { -10 } else { 10 };
            let sm = StatMod::new(
                w,
                StatType::ALL,
                Some(name),
                offset,
                TimeSpan::from_mins(30),
                None,
            );
            mb::add_stat_mod(w, mob, sm);
            if blight {
                fx::fixed_particles(w, mob, 0x374A, 10, 15, 5028, EffectLayer::Waist);
                fx::play_sound(w, mob, 0x1FB);
            } else {
                fx::fixed_particles(w, mob, 0x373A, 10, 15, 5018, EffectLayer::Waist);
                fx::play_sound(w, mob, 0x1EA);
            }
            functioned = true;
        }

        if functioned {
            spell_helper::turn(w, m, Some(p));
            let at = spell_helper::point_of(w, p);
            let map = w.mobile(m).and_then(|x| x.map);
            effects_play_sound(w, at, map, if blight { 0x1FB } else { 0x299 });
            say(
                w,
                m,
                if blight {
                    "You curse the area."
                } else {
                    "You consecrate the area."
                },
            );
            self.finish_invoke(w, pl);
        } else {
            fx::fixed_effect(w, m, 0x3735, 6, 30);
            fx::play_sound(w, m, 0x5C);
        }
    }
}

/// `HolySense`' direction word (`GetDirectionTo(...) switch`).
pub fn direction_word(d: Direction) -> &'static str {
    match d.bits() & Direction::MASK.bits() {
        x if x == Direction::EAST.bits() => "east",
        x if x == Direction::NORTH.bits() => "north",
        x if x == Direction::SOUTH.bits() => "south",
        x if x == Direction::UP.bits() => "north-west",
        x if x == Direction::DOWN.bits() => "south-east",
        x if x == Direction::LEFT.bits() => "south-west",
        x if x == Direction::RIGHT.bits() => "north-east",
        _ => "west",
    }
}
