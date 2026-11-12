//! Server.Spells.Chivalry
pub mod cleanse_by_fire;
pub mod close_wounds;
pub mod consecrate_weapon;
pub mod dispel_evil;
pub mod divine_fury;
pub mod enemy_of_one;
pub mod holy_light;
pub mod noble_sacrifice;
pub mod paladin;
pub mod remove_curse;
pub mod sacred_journey;

use muo_content::items::weapon_ability;
use muo_content::player_mobile::buff::{BuffIcon, remove_buff};
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::mobile::base as mb;

use crate::common::is_player_mobile;

/// The curse removal common to Remove Curse and Noble Sacrifice: the three stat curses, then (when `paralyzed` is
/// taken by the caller) the spell and weapon effects. Returns whether anything was removed by the effect calls.
pub fn remove_stat_curses(w: &mut World, m: MobileId) -> bool {
    let mut any = false;
    for name in [
        "[Magic] Str Curse",
        "[Magic] Dex Curse",
        "[Magic] Int Curse",
    ] {
        if let Some(i) = mb::get_stat_mod(w, m, name, false)
            && w.mobile(m).is_some_and(|x| x.stat_mods[i].offset < 0)
        {
            mb::remove_stat_mod_by_name(w, m, name);
            any = true;
        }
    }
    any
}

/// `EvilOmenSpell.EndEffect`, `StrangleSpell.RemoveCurse`, `CorpseSkinSpell.RemoveCurse`, `CurseSpell.RemoveEffect`,
/// `MortalStrike.EndWound`, `MindRotSpell.ClearMindRotScalar`, `BloodOathSpell.RemoveCurse` and
/// `SpellPlagueSpell.RemoveEffect`, in that order; true if any of them did something. Each runs even after one did.
pub fn end_curse_effects(w: &mut World, m: MobileId) -> bool {
    let mut send_effect = false;
    send_effect = crate::necromancy::evil_omen::end_effect(w, m) || send_effect;
    send_effect = crate::necromancy::strangle::remove_curse(w, m) || send_effect;
    send_effect = crate::necromancy::corpse_skin::remove_curse(w, m) || send_effect;
    send_effect = crate::fourth::curse::remove_effect(w, m) || send_effect;
    send_effect = weapon_ability::end_wound(w, m) || send_effect;
    send_effect = crate::necromancy::mind_rot::clear_mind_rot_scalar(w, m) || send_effect;
    send_effect = crate::necromancy::blood_oath::remove_curse(w, m) || send_effect;
    send_effect = crate::mysticism::spell_plague::remove_effect(w, m) || send_effect;
    send_effect
}

/// The buff icons Remove Curse and Noble Sacrifice take off a player.
pub fn remove_curse_buffs(w: &mut World, m: MobileId) {
    if is_player_mobile(w, m) {
        for icon in [
            BuffIcon::Clumsy,
            BuffIcon::FeebleMind,
            BuffIcon::Weaken,
            BuffIcon::Curse,
            BuffIcon::MassCurse,
            BuffIcon::MortalStrike,
            BuffIcon::Strangle,
            BuffIcon::EvilOmen,
        ] {
            remove_buff(w, m, icon);
        }
    }
}
