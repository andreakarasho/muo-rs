//! The Create Food spell.

use muo_content::factory;
use muo_content::spells::magery::{FIRST_CIRCLE, MAGERY_SPELL};
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_content::spells::{SpellInfo, reagent};
use muo_primitives::utility;
use muo_server::World;
use muo_server::effects::EffectLayer;
use muo_server::mobile::part2::{effects, messages as mm};

/// `CreateFoodSpell`: In Mani Ylem.
pub static CREATE_FOOD_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.First.CreateFoodSpell",
    parent: Some(&MAGERY_SPELL),
    info: SpellInfo::new(
        "Create Food",
        "In Mani Ylem",
        &[reagent::GARLIC, reagent::GINSENG, reagent::MANDRAKE_ROOT],
    )
    .action(224)
    .hand_effect(9011),
    on_cast,
    family: Some(&FIRST_CIRCLE),
    ..MAGERY_SPELL
};

const FOODS: [&str; 10] = [
    "Server.Items.Grapes",
    "Server.Items.Ham",
    "Server.Items.CheeseWedge",
    "Server.Items.Muffins",
    "Server.Items.FishSteak",
    "Server.Items.Ribs",
    "Server.Items.CookedBird",
    "Server.Items.Sausage",
    "Server.Items.Apple",
    "Server.Items.Peach",
];

fn on_cast(w: &mut World, sp: SpellId) {
    if spell::check_sequence(w, sp) {
        let Some(caster) = spell::caster(w, sp) else {
            return;
        };
        let pick = utility::random(&mut w.rng, 10) as usize;
        let class = FOODS[pick.min(9)];
        if let Some(food) = factory::make_item(w, class) {
            muo_server::mobile::part2::items::add_to_backpack(w, caster, food);
            let label = w.item(food).map_or(0, |i| (i.ty.label_number)(w, food));
            let name = muo_content::items::equip_hooks::hooks(w)
                .and_then(|h| h.localized_text)
                .and_then(|f| f(w, label))
                .unwrap_or_default(); // a missing cliloc is null, interpolated as ""
            // You magically create food in your backpack:
            mm::send_localized_message_affix(
                w,
                caster,
                1042695,
                true,
                &format!(" {name}"),
                "",
                0x3B2,
            );
            effects::fixed_particles(w, caster, 0, 10, 5, 2003, EffectLayer::RightHand);
            effects::play_sound(w, caster, 0x1E2);
        }
    }
    spell::finish_sequence(w, sp);
}
