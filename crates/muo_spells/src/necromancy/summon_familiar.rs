//! The Summon Familiar spell, `SummonFamiliarEntry` and `SummonFamiliarGump`.

use indexmap::IndexMap;
use muo_content::creature::control;
use muo_content::factory;
use muo_content::gumps::base::{DynamicGumpLogic, DynamicGumpObj, RelayInfo};
use muo_content::gumps::builder::DynamicGumpBuilder;
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_content::spells::{SpellInfo, reagent};
use muo_content::text::add_html_text_builder;
use muo_gumps::m_z::common::draw_builder;
use muo_io::text::TextDefinition;
use muo_primitives::TimeSpan;
use muo_server::World;
use muo_server::effects::EffectLayer;
use muo_server::ids::MobileId;
use muo_server::mobile::part2::{effects, messages as mm};
use muo_server::net_state::NetStateId;
use muo_server::skills::{self, SkillName};

use super::necromancer::{NECRO_BASE, NECROMANCER_SPELL, NecroType};

static SUMMON_FAMILIAR: NecroType = NecroType {
    required_skill: 30.0,
    required_mana: 17,
    ..NECRO_BASE
};

/// `SummonFamiliarSpell`: Kal Xen Bal.
pub static SUMMON_FAMILIAR_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Necromancy.SummonFamiliarSpell",
    parent: Some(&NECROMANCER_SPELL),
    info: SpellInfo::new(
        "Summon Familiar",
        "Kal Xen Bal",
        &[
            reagent::BAT_WING,
            reagent::GRAVE_DUST,
            reagent::DAEMON_BLOOD,
        ],
    )
    .action(203)
    .hand_effect(9031),
    cast_delay_base: |_, _| TimeSpan::from_secs(2),
    on_cast,
    check_cast,
    family: Some(&SUMMON_FAMILIAR),
    ..NECROMANCER_SPELL
};

/// `SummonFamiliarEntry`.
pub struct SummonFamiliarEntry {
    pub type_name: &'static str,
    pub name: i32,
    pub req_necromancy: f64,
    pub req_spirit_speak: f64,
}

/// `Entries`.
pub static ENTRIES: [SummonFamiliarEntry; 5] = [
    SummonFamiliarEntry {
        type_name: "Server.Mobiles.HordeMinionFamiliar",
        name: 1060146,
        req_necromancy: 30.0,
        req_spirit_speak: 30.0,
    }, // Horde Minion
    SummonFamiliarEntry {
        type_name: "Server.Mobiles.ShadowWispFamiliar",
        name: 1060142,
        req_necromancy: 50.0,
        req_spirit_speak: 50.0,
    }, // Shadow Wisp
    SummonFamiliarEntry {
        type_name: "Server.Mobiles.DarkWolfFamiliar",
        name: 1060143,
        req_necromancy: 60.0,
        req_spirit_speak: 60.0,
    }, // Dark Wolf
    SummonFamiliarEntry {
        type_name: "Server.Mobiles.DeathAdder",
        name: 1060145,
        req_necromancy: 80.0,
        req_spirit_speak: 80.0,
    }, // Death Adder
    SummonFamiliarEntry {
        type_name: "Server.Mobiles.VampireBatFamiliar",
        name: 1060144,
        req_necromancy: 100.0,
        req_spirit_speak: 100.0,
    }, // Vampire Bat
];

/// `Table`.
#[derive(Default)]
struct Table(IndexMap<MobileId, MobileId>);

fn table(w: &mut World) -> &mut IndexMap<MobileId, MobileId> {
    &mut w.resources.get_mut::<Table>().0
}

/// The `PlayerDeletedEvent` handler.
pub fn init(w: &mut World) {
    muo_content::player_mobile::subscribe_deleted(w, remove_effects);
}

/// `RemoveEffects(m)`.
pub fn remove_effects(w: &mut World, m: MobileId) {
    if let Some(summon) = table(w).shift_remove(&m)
        && let Some(ty) = w.mobile(summon).map(|x| x.ty)
    {
        (ty.delete)(w, summon);
    }
}

/// `Unregister(master, summoned)`.
pub fn unregister(w: &mut World, master: Option<MobileId>, summoned: MobileId) {
    if let Some(master) = master
        && table(w).get(&master) == Some(&summoned)
    {
        table(w).shift_remove(&master);
    }
}

/// `Table.TryGetValue(master, out var bc)`, read-only.
pub fn familiar_of(w: &World, master: MobileId) -> Option<MobileId> {
    w.resources.get::<Table>()?.0.get(&master).copied()
}

/// `Table.GetValueOrDefault(m)?.Deleted == false`.
fn has_familiar(w: &mut World, m: MobileId) -> bool {
    table(w)
        .get(&m)
        .copied()
        .is_some_and(|f| w.mobile(f).is_some_and(|x| !x.deleted))
}

fn check_cast(w: &mut World, sp: SpellId) -> bool {
    let Some(caster) = spell::caster(w, sp) else {
        return false;
    };
    if has_familiar(w, caster) {
        mm::send_localized_message(w, caster, 1061605, "", 0x3B2); // You already have a familiar.
        return false;
    }

    (NECROMANCER_SPELL.check_cast)(w, sp)
}

fn on_cast(w: &mut World, sp: SpellId) {
    if spell::check_sequence(w, sp)
        && let Some(caster) = spell::caster(w, sp)
    {
        let necro = skills::value(w, caster, SkillName::Necromancy);
        let spirit = skills::value(w, caster, SkillName::SpiritSpeak);
        let gump = Box::new(new_gump_with(caster, sp, necro, spirit));
        muo_content::gumps::system::send_gump_to_mobile(w, caster, gump, true);
    }

    spell::finish_sequence(w, sp);
}

// ---- SummonFamiliarGump ----

const ENABLED_COLOR16: i32 = 0x0F20;
const DISABLED_COLOR16: i32 = 0x262A;
const ENABLED_COLOR32: i32 = 0x18CD00;
const DISABLED_COLOR32: i32 = 0x4A8B52;

/// `SummonFamiliarGump`.
pub struct SummonFamiliarGump {
    from: MobileId,
    spell: SpellId,
    necro: f64,
    spirit: f64,
}

impl DynamicGumpLogic for SummonFamiliarGump {
    const TYPE_NAME: &'static str = "Server.Spells.Necromancy.SummonFamiliarGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        b.add_page(0);

        b.add_background(10, 10, 250, 178, 9270);
        b.add_alpha_region(20, 20, 230, 158);

        b.add_image(220, 20, 10464, 0);
        b.add_image(220, 72, 10464, 0);
        b.add_image(220, 124, 10464, 0);

        b.add_item(188, 16, 6883, 0);
        b.add_item(198, 168, 6881, 0);
        b.add_item(8, 15, 6882, 0);
        b.add_item(2, 168, 6880, 0);

        b.add_html_localized_color(30, 26, 200, 20, 1060147, ENABLED_COLOR16, false, false); // Chose thy familiar...

        for (i, entry) in ENTRIES.iter().enumerate() {
            let enabled =
                self.necro >= entry.req_necromancy && self.spirit >= entry.req_spirit_speak;

            b.add_button(27, 53 + i as i32 * 21, 9702, 9703, i as i32 + 1);

            let name = TextDefinition::of_number(entry.name);
            draw_builder(
                b,
                add_html_text_builder(
                    Some(&name),
                    false,
                    false,
                    if enabled {
                        ENABLED_COLOR16
                    } else {
                        DISABLED_COLOR16
                    },
                    if enabled {
                        ENABLED_COLOR32
                    } else {
                        DISABLED_COLOR32
                    },
                ),
                50,
                51 + i as i32 * 21,
                150,
                20,
            );
        }
    }

    fn on_response(&mut self, w: &mut World, _sender: NetStateId, info: &RelayInfo) {
        let from = self.from;
        let index = info.button_id - 1;

        if index < 0 || index as usize >= ENTRIES.len() {
            mm::send_localized_message(w, from, 1061825, "", 0x3B2); // You decide not to summon a familiar.
            return;
        }

        let entry = &ENTRIES[index as usize];

        let necro = skills::value(w, from, SkillName::Necromancy);
        let spirit = skills::value(w, from, SkillName::SpiritSpeak);

        let duel_blocks = muo_content::spells::hooks::hooks(w)
            .and_then(|h| h.duel_allow_spell_cast)
            .is_some_and(|f| !f(w, from, self.spell));

        if duel_blocks {
            // not allowed in the duel
        } else if has_familiar(w, from) {
            mm::send_localized_message(w, from, 1061605, "", 0x3B2); // You already have a familiar.
        } else if necro < entry.req_necromancy || spirit < entry.req_spirit_speak {
            // That familiar requires ~1_NECROMANCY~ Necromancy and ~2_SPIRIT~ Spirit Speak.
            mm::send_localized_message(
                w,
                from,
                1061606,
                &format!("{:.1}\t{:.1}", entry.req_necromancy, entry.req_spirit_speak),
                0x3B2,
            );
            let g = Box::new(new_gump_with(from, self.spell, necro, spirit));
            muo_content::gumps::system::send_gump_to_mobile(w, from, g, true);
        } else if let Some(bc) = factory::make_mobile(w, entry.type_name) {
            // TODO: Is this right?
            let base = skills::base(w, from, SkillName::MagicResist);
            skills::set_base(w, bc, SkillName::MagicResist, base);

            let loc = w.mobile(from).map(|m| m.location).unwrap_or_default();
            if control::summon(w, bc, true, from, loc, -1, TimeSpan::from_days(1), None) {
                effects::fixed_particles(w, from, 0x3728, 1, 10, 9910, EffectLayer::Head);
                let snd = w
                    .mobile(bc)
                    .map(|x| x.ty)
                    .map_or(0, |x| (x.get_idle_sound)(w, bc));
                effects::play_sound(w, bc, snd);

                table(w).insert(from, bc);
            }
        } else {
            mm::send_message(w, from, "That familiar has not yet been defined.");
            let g = Box::new(new_gump_with(from, self.spell, necro, spirit));
            muo_content::gumps::system::send_gump_to_mobile(w, from, g, true);
        }
    }
}

fn new_gump_with(
    from: MobileId,
    sp: SpellId,
    necro: f64,
    spirit: f64,
) -> DynamicGumpObj<SummonFamiliarGump> {
    DynamicGumpObj::new(
        200,
        100,
        SummonFamiliarGump {
            from,
            spell: sp,
            necro,
            spirit,
        },
    )
}

/// Tests of a familiar whose master is deleted.
#[cfg(test)]
mod behavior_tests {
    use super::*;
    use muo_content::creature::BASE_CREATURE;
    use muo_content::creature::control::{set_control_master, set_summoned};
    use muo_content::creature::enums::{AIType, FightMode};
    use muo_content::creature::setters::new_creature;

    #[test]
    fn deleting_a_familiar_unregisters_it() {
        let mut w = muo_server::map::test_support::world();
        muo_content::ai::configure_for_tests(&mut w);
        crate::init_content_hooks(&mut w); // the hook BaseCreature.Delete calls to unregister a summon
        let caster = muo_content::player_mobile::new_player_mobile(&mut w);
        let familiar = new_creature(
            &mut w,
            &BASE_CREATURE,
            AIType::AiAnimal,
            FightMode::Closest,
            10,
            1,
        );
        set_control_master(&mut w, familiar, Some(caster));
        set_summoned(&mut w, familiar, true);
        table(&mut w).insert(caster, familiar);

        let ty = w.mobile(familiar).unwrap().ty;
        (ty.delete)(&mut w, familiar);

        assert!(!table(&mut w).contains_key(&caster));
    }
}
