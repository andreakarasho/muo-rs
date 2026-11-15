//! The ML quest givers on a `BaseCreature` base: one constructor per class.

use super::npc_support::*;
use super::quester::{self, Quester};
use muo_content::creature::{BASE_CREATURE, BASE_CREATURE_C};
use muo_content::hooks::rnd;
use muo_primitives::{Body, utility};
use muo_server::ids::MobileId;
use muo_server::type_registry::TypeRegistry;
use muo_server::world::World;

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Ben`.
    BEN, BEN_C = "Server.Engines.MLQuests.Definitions.Ben" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Frederic`.
    FREDERIC, FREDERIC_C = "Server.Engines.MLQuests.Definitions.Frederic" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Leon`.
    LEON, LEON_C = "Server.Engines.MLQuests.Definitions.Leon" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Andros`.
    ANDROS, ANDROS_C = "Server.Engines.MLQuests.Definitions.Andros" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Kia`.
    KIA, KIA_C = "Server.Engines.MLQuests.Definitions.Kia" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Emerillo`.
    EMERILLO, EMERILLO_C = "Server.Engines.MLQuests.Definitions.Emerillo" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}
fn emerillo_shout(w: &mut World, id: MobileId, pm: MobileId) {
    quester::tell(w, Quester::Mobile(id), pm, 1074222);
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Nythalia`.
    NYTHALIA, NYTHALIA_C = "Server.Engines.MLQuests.Definitions.Nythalia" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}

fn jamal_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &JAMAL, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the Fisherman".to_string()));
    set_body(w, id, Body(400));
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    let hh = hair_hue(w, id);
    assign_random_facial_hair_hue(w, id, hh);
    add_item_hue(w, id, "Shirt", Some(0x1BB));
    let v2 = utility::random_neutral_hue(&mut w.rng);
    add_item_hue(w, id, "ShortPants", Some(v2));
    let v3 = utility::random_animal_hue(&mut w.rng);
    add_item_hue(w, id, "ThighBoots", Some(v3));
    add_item_hue(w, id, "Backpack", None);
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Jamal`.
    JAMAL, JAMAL_C = "Server.Engines.MLQuests.Definitions.Jamal" : BASE_CREATURE / BASE_CREATURE_C, ctor jamal_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        default_name: |_w, _id| Some("Jamal"),
    }
    m {
    }
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Iosep`.
    IOSEP, IOSEP_C = "Server.Engines.MLQuests.Definitions.Iosep" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}
fn iosep_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = rnd::bool(w);
    let v2 = if v1 { 1074209 } else { 1074215 };
    quester::tell(w, Quester::Mobile(id), pm, v2);
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Aurelia`.
    AURELIA, AURELIA_C = "Server.Engines.MLQuests.Definitions.Aurelia" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.SkeletonOfSzandor`.
    SKELETON_OF_SZANDOR, SKELETON_OF_SZANDOR_C = "Server.Engines.MLQuests.Definitions.SkeletonOfSzandor" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Saril`.
    SARIL, SARIL_C = "Server.Engines.MLQuests.Definitions.Saril" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}
fn saril_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = rnd::bool(w);
    let v2 = if v1 { 1074186 } else { 1074183 };
    quester::tell(w, Quester::Mobile(id), pm, v2);
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Cailla`.
    CAILLA, CAILLA_C = "Server.Engines.MLQuests.Definitions.Cailla" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}
fn cailla_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = rnd::bool(w);
    let v2 = if v1 { 1074187 } else { 1074210 };
    quester::tell(w, Quester::Mobile(id), pm, v2);
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Tamm`.
    TAMM, TAMM_C = "Server.Engines.MLQuests.Definitions.Tamm" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}
fn tamm_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = rnd::bool(w);
    let v2 = if v1 { 1074213 } else { 1074187 };
    quester::tell(w, Quester::Mobile(id), pm, v2);
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Landy`.
    LANDY, LANDY_C = "Server.Engines.MLQuests.Definitions.Landy" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}
fn landy_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = rnd::bool(w);
    let v2 = if v1 { 1074211 } else { 1074218 };
    quester::tell(w, Quester::Mobile(id), pm, v2);
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Alejaha`.
    ALEJAHA, ALEJAHA_C = "Server.Engines.MLQuests.Definitions.Alejaha" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}
fn alejaha_shout(w: &mut World, id: MobileId, pm: MobileId) {
    quester::tell(w, Quester::Mobile(id), pm, 1074223);
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Mielan`.
    MIELAN, MIELAN_C = "Server.Engines.MLQuests.Definitions.Mielan" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}
fn mielan_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = rnd::bool(w);
    let v2 = if v1 { 1074219 } else { 1074223 };
    quester::tell(w, Quester::Mobile(id), pm, v2);
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Ciala`.
    CIALA, CIALA_C = "Server.Engines.MLQuests.Definitions.Ciala" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}
fn ciala_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = rnd::bool(w);
    let v2 = if v1 { 1074206 } else { 1074186 };
    quester::tell(w, Quester::Mobile(id), pm, v2);
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Aniel`.
    ANIEL, ANIEL_C = "Server.Engines.MLQuests.Definitions.Aniel" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}
fn aniel_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = rnd::bool(w);
    let v2 = if v1 { 1074220 } else { 1074222 };
    quester::tell(w, Quester::Mobile(id), pm, v2);
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Aulan`.
    AULAN, AULAN_C = "Server.Engines.MLQuests.Definitions.Aulan" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}
fn aulan_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v2 = rnd::below(w, 3);
    let v1 = match v2 {
        0 => 1074188,
        1 => 1074191,
        _ => 1074195,
    };
    quester::tell(w, Quester::Mobile(id), pm, v1);
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Brinnae`.
    BRINNAE, BRINNAE_C = "Server.Engines.MLQuests.Definitions.Brinnae" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}
fn brinnae_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = rnd::bool(w);
    let v2 = if v1 { 1074212 } else { 1074210 };
    quester::tell(w, Quester::Mobile(id), pm, v2);
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Caelas`.
    CAELAS, CAELAS_C = "Server.Engines.MLQuests.Definitions.Caelas" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}
fn caelas_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = rnd::bool(w);
    let v2 = if v1 { 1074204 } else { 1074201 };
    quester::tell(w, Quester::Mobile(id), pm, v2);
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Clehin`.
    CLEHIN, CLEHIN_C = "Server.Engines.MLQuests.Definitions.Clehin" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}
fn clehin_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = rnd::bool(w);
    let v2 = if v1 { 1074211 } else { 1074186 };
    quester::tell(w, Quester::Mobile(id), pm, v2);
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Cloorne`.
    CLOORNE, CLOORNE_C = "Server.Engines.MLQuests.Definitions.Cloorne" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}
fn cloorne_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = rnd::bool(w);
    let v2 = if v1 { 1074185 } else { 1074186 };
    quester::tell(w, Quester::Mobile(id), pm, v2);
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Salaenih`.
    SALAENIH, SALAENIH_C = "Server.Engines.MLQuests.Definitions.Salaenih" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}
fn salaenih_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = rnd::bool(w);
    let v2 = if v1 { 1074207 } else { 1074209 };
    quester::tell(w, Quester::Mobile(id), pm, v2);
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Vilo`.
    VILO, VILO_C = "Server.Engines.MLQuests.Definitions.Vilo" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}
fn vilo_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = rnd::bool(w);
    let v2 = if v1 { 1074210 } else { 1074220 };
    quester::tell(w, Quester::Mobile(id), pm, v2);
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Tholef`.
    THOLEF, THOLEF_C = "Server.Engines.MLQuests.Definitions.Tholef" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}
fn tholef_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = rnd::bool(w);
    let v2 = if v1 { 1074209 } else { 1074184 };
    quester::tell(w, Quester::Mobile(id), pm, v2);
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Tillanil`.
    TILLANIL, TILLANIL_C = "Server.Engines.MLQuests.Definitions.Tillanil" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}
fn tillanil_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = rnd::bool(w);
    let v2 = if v1 { 1074187 } else { 1074222 };
    quester::tell(w, Quester::Mobile(id), pm, v2);
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Waelian`.
    WAELIAN, WAELIAN_C = "Server.Engines.MLQuests.Definitions.Waelian" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}
fn waelian_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = rnd::bool(w);
    let v2 = if v1 { 1074221 } else { 1074201 };
    quester::tell(w, Quester::Mobile(id), pm, v2);
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Sleen`.
    SLEEN, SLEEN_C = "Server.Engines.MLQuests.Definitions.Sleen" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}
fn sleen_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = rnd::bool(w);
    let v2 = if v1 { 1074200 } else { 1074206 };
    quester::tell(w, Quester::Mobile(id), pm, v2);
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Unoelil`.
    UNOELIL, UNOELIL_C = "Server.Engines.MLQuests.Definitions.Unoelil" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}
fn unoelil_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = rnd::bool(w);
    let v2 = if v1 { 1074209 } else { 1074186 };
    quester::tell(w, Quester::Mobile(id), pm, v2);
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Anolly`.
    ANOLLY, ANOLLY_C = "Server.Engines.MLQuests.Definitions.Anolly" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Jusae`.
    JUSAE, JUSAE_C = "Server.Engines.MLQuests.Definitions.Jusae" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}
fn jusae_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = rnd::bool(w);
    let v2 = if v1 { 1074210 } else { 1074213 };
    quester::tell(w, Quester::Mobile(id), pm, v2);
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Cillitha`.
    CILLITHA, CILLITHA_C = "Server.Engines.MLQuests.Definitions.Cillitha" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}
fn cillitha_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = rnd::bool(w);
    let v2 = if v1 { 1074223 } else { 1074213 };
    quester::tell(w, Quester::Mobile(id), pm, v2);
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Lohn`.
    LOHN, LOHN_C = "Server.Engines.MLQuests.Definitions.Lohn" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}
fn lohn_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = rnd::bool(w);
    let v2 = if v1 { 1074209 } else { 1074187 };
    quester::tell(w, Quester::Mobile(id), pm, v2);
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Olla`.
    OLLA, OLLA_C = "Server.Engines.MLQuests.Definitions.Olla" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}
fn olla_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = rnd::bool(w);
    let v2 = if v1 { 1074185 } else { 1074187 };
    quester::tell(w, Quester::Mobile(id), pm, v2);
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Thallary`.
    THALLARY, THALLARY_C = "Server.Engines.MLQuests.Definitions.Thallary" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}
fn thallary_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = rnd::bool(w);
    let v2 = if v1 { 1074221 } else { 1074212 };
    quester::tell(w, Quester::Mobile(id), pm, v2);
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Ahie`.
    AHIE, AHIE_C = "Server.Engines.MLQuests.Definitions.Ahie" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}
fn ahie_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = rnd::bool(w);
    let v2 = if v1 { 1074206 } else { 1074203 };
    quester::tell(w, Quester::Mobile(id), pm, v2);
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Tyeelor`.
    TYEELOR, TYEELOR_C = "Server.Engines.MLQuests.Definitions.Tyeelor" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Athailon`.
    ATHAILON, ATHAILON_C = "Server.Engines.MLQuests.Definitions.Athailon" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.ElderTaellia`.
    ELDER_TAELLIA, ELDER_TAELLIA_C = "Server.Engines.MLQuests.Definitions.ElderTaellia" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.ElderMallew`.
    ELDER_MALLEW, ELDER_MALLEW_C = "Server.Engines.MLQuests.Definitions.ElderMallew" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.ElderAbbein`.
    ELDER_ABBEIN, ELDER_ABBEIN_C = "Server.Engines.MLQuests.Definitions.ElderAbbein" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.ElderVicaie`.
    ELDER_VICAIE, ELDER_VICAIE_C = "Server.Engines.MLQuests.Definitions.ElderVicaie" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.ElderJothan`.
    ELDER_JOTHAN, ELDER_JOTHAN_C = "Server.Engines.MLQuests.Definitions.ElderJothan" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.ElderAlethanian`.
    ELDER_ALETHANIAN, ELDER_ALETHANIAN_C = "Server.Engines.MLQuests.Definitions.ElderAlethanian" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Rebinil`.
    REBINIL, REBINIL_C = "Server.Engines.MLQuests.Definitions.Rebinil" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Aluniol`.
    ALUNIOL, ALUNIOL_C = "Server.Engines.MLQuests.Definitions.Aluniol" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Olaeni`.
    OLAENI, OLAENI_C = "Server.Engines.MLQuests.Definitions.Olaeni" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Bolaevin`.
    BOLAEVIN, BOLAEVIN_C = "Server.Engines.MLQuests.Definitions.Bolaevin" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.LorekeeperAneen`.
    LOREKEEPER_ANEEN, LOREKEEPER_ANEEN_C = "Server.Engines.MLQuests.Definitions.LorekeeperAneen" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Daelas`.
    DAELAS, DAELAS_C = "Server.Engines.MLQuests.Definitions.Daelas" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Alelle`.
    ALELLE, ALELLE_C = "Server.Engines.MLQuests.Definitions.Alelle" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.LorekeeperNillaen`.
    LOREKEEPER_NILLAEN, LOREKEEPER_NILLAEN_C = "Server.Engines.MLQuests.Definitions.LorekeeperNillaen" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.LorekeeperRyal`.
    LOREKEEPER_RYAL, LOREKEEPER_RYAL_C = "Server.Engines.MLQuests.Definitions.LorekeeperRyal" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}
fn lorekeeper_ryal_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = rnd::bool(w);
    let v2 = if v1 { 1074204 } else { 1074200 };
    quester::tell(w, Quester::Mobile(id), pm, v2);
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Braen`.
    BRAEN, BRAEN_C = "Server.Engines.MLQuests.Definitions.Braen" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}
fn braen_shout(w: &mut World, id: MobileId, pm: MobileId) {
    quester::tell(w, Quester::Mobile(id), pm, 1074187);
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.ElderAcob`.
    ELDER_ACOB, ELDER_ACOB_C = "Server.Engines.MLQuests.Definitions.ElderAcob" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}
fn elder_acob_shout(w: &mut World, id: MobileId, pm: MobileId) {
    quester::tell(w, Quester::Mobile(id), pm, 1074197);
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.LorekeeperCalendor`.
    LOREKEEPER_CALENDOR, LOREKEEPER_CALENDOR_C = "Server.Engines.MLQuests.Definitions.LorekeeperCalendor" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}
fn lorekeeper_calendor_shout(w: &mut World, id: MobileId, pm: MobileId) {
    quester::tell(w, Quester::Mobile(id), pm, 1074204);
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.LorekeeperSiarra`.
    LOREKEEPER_SIARRA, LOREKEEPER_SIARRA_C = "Server.Engines.MLQuests.Definitions.LorekeeperSiarra" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}
fn lorekeeper_siarra_shout(w: &mut World, id: MobileId, pm: MobileId) {
    quester::tell(w, Quester::Mobile(id), pm, 1074206);
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Enigma`.
    ENIGMA, ENIGMA_C = "Server.Engines.MLQuests.Definitions.Enigma" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Sledge`.
    SLEDGE, SLEDGE_C = "Server.Engines.MLQuests.Definitions.Sledge" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}
fn sledge_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = [1074188, 1074195][rnd::below(w, 2) as usize];
    quester::tell(w, Quester::Mobile(id), pm, v1);
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Patricus`.
    PATRICUS, PATRICUS_C = "Server.Engines.MLQuests.Definitions.Patricus" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Belulah`.
    BELULAH, BELULAH_C = "Server.Engines.MLQuests.Definitions.Belulah" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}
fn belulah_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = rnd::from(w, 1074205, 2);
    quester::tell(w, Quester::Mobile(id), pm, v1);
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Evan`.
    EVAN, EVAN_C = "Server.Engines.MLQuests.Definitions.Evan" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Regina`.
    REGINA, REGINA_C = "Server.Engines.MLQuests.Definitions.Regina" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}

fn grandpa_charley_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &GRANDPA_CHARLEY, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the farmer".to_string()));
    set_body(w, id, Body(400));
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    let v2 = rnd::below(w, 2);
    let v3 = 0x3B2 + v2;
    assign_random_hair_hue(w, id, v3);
    set_facial_hair_item_id(w, id, 0x203E);
    set_facial_hair_hue(w, id, v3);
    set_skill_range(w, id, SkillName::ItemID, 80.0, 90.0);
    let v4 = utility::random_nondyed_hue(&mut w.rng);
    add_item_hue(w, id, "WideBrimHat", Some(v4));
    let v5 = utility::random_nondyed_hue(&mut w.rng);
    add_item_hue(w, id, "FancyShirt", Some(v5));
    let v6 = utility::random_nondyed_hue(&mut w.rng);
    add_item_hue(w, id, "LongPants", Some(v6));
    let v7 = utility::random_neutral_hue(&mut w.rng);
    add_item_hue(w, id, "Sandals", Some(v7));
    add_item_hue(w, id, "ShepherdsCrook", None);
    add_item_hue(w, id, "Backpack", None);
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.GrandpaCharley`.
    GRANDPA_CHARLEY, GRANDPA_CHARLEY_C = "Server.Engines.MLQuests.Definitions.GrandpaCharley" : BASE_CREATURE / BASE_CREATURE_C, ctor grandpa_charley_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        default_name: |_w, _id| Some("Grandpa Charley"),
        can_teach: |_w, _id| true,
    }
    m {
    }
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Jelrice`.
    JELRICE, JELRICE_C = "Server.Engines.MLQuests.Definitions.Jelrice" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}
fn jelrice_shout(w: &mut World, id: MobileId, pm: MobileId) {
    quester::tell(w, Quester::Mobile(id), pm, 1074221);
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Yorus`.
    YORUS, YORUS_C = "Server.Engines.MLQuests.Definitions.Yorus" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}
fn yorus_shout(w: &mut World, id: MobileId, pm: MobileId) {
    quester::tell(w, Quester::Mobile(id), pm, 1074218);
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Drithen`.
    DRITHEN, DRITHEN_C = "Server.Engines.MLQuests.Definitions.Drithen" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}
fn drithen_shout(w: &mut World, id: MobileId, pm: MobileId) {
    quester::tell(w, Quester::Mobile(id), pm, 1074188);
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Aernya`.
    AERNYA, AERNYA_C = "Server.Engines.MLQuests.Definitions.Aernya" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Gorrow`.
    GORROW, GORROW_C = "Server.Engines.MLQuests.Definitions.Gorrow" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}
fn gorrow_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = [1074200, 1074203][rnd::below(w, 2) as usize];
    quester::tell(w, Quester::Mobile(id), pm, v1);
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.MasterGnosos`.
    MASTER_GNOSOS, MASTER_GNOSOS_C = "Server.Engines.MLQuests.Definitions.MasterGnosos" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}
fn master_gnosos_shout(w: &mut World, id: MobileId, pm: MobileId) {
    quester::tell(w, Quester::Mobile(id), pm, 1074186);
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Dimethro`.
    DIMETHRO, DIMETHRO_C = "Server.Engines.MLQuests.Definitions.Dimethro" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}
fn dimethro_shout(w: &mut World, id: MobileId, pm: MobileId) {
    quester::tell(w, Quester::Mobile(id), pm, 1078128);
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Churchill`.
    CHURCHILL, CHURCHILL_C = "Server.Engines.MLQuests.Definitions.Churchill" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}
fn churchill_shout(w: &mut World, id: MobileId, pm: MobileId) {
    quester::tell(w, Quester::Mobile(id), pm, 1078141);
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Recaro`.
    RECARO, RECARO_C = "Server.Engines.MLQuests.Definitions.Recaro" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}
fn recaro_shout(w: &mut World, id: MobileId, pm: MobileId) {
    quester::tell(w, Quester::Mobile(id), pm, 1078187);
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.AldenArmstrong`.
    ALDEN_ARMSTRONG, ALDEN_ARMSTRONG_C = "Server.Engines.MLQuests.Definitions.AldenArmstrong" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}
fn alden_armstrong_shout(w: &mut World, id: MobileId, pm: MobileId) {
    quester::tell(w, Quester::Mobile(id), pm, 1078136);
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Jockles`.
    JOCKLES, JOCKLES_C = "Server.Engines.MLQuests.Definitions.Jockles" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}
fn jockles_shout(w: &mut World, id: MobileId, pm: MobileId) {
    quester::tell(w, Quester::Mobile(id), pm, 1078135);
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.TylAriadne`.
    TYL_ARIADNE, TYL_ARIADNE_C = "Server.Engines.MLQuests.Definitions.TylAriadne" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}
fn tyl_ariadne_shout(w: &mut World, id: MobileId, pm: MobileId) {
    quester::tell(w, Quester::Mobile(id), pm, 1078140);
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Alefian`.
    ALEFIAN, ALEFIAN_C = "Server.Engines.MLQuests.Definitions.Alefian" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}
fn alefian_shout(w: &mut World, id: MobileId, pm: MobileId) {
    quester::tell(w, Quester::Mobile(id), pm, 1078130);
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Gustar`.
    GUSTAR, GUSTAR_C = "Server.Engines.MLQuests.Definitions.Gustar" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}
fn gustar_shout(w: &mut World, id: MobileId, pm: MobileId) {
    quester::tell(w, Quester::Mobile(id), pm, 1078126);
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Jillian`.
    JILLIAN, JILLIAN_C = "Server.Engines.MLQuests.Definitions.Jillian" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}
fn jillian_shout(w: &mut World, id: MobileId, pm: MobileId) {
    quester::tell(w, Quester::Mobile(id), pm, 1078129);
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Kaelynna`.
    KAELYNNA, KAELYNNA_C = "Server.Engines.MLQuests.Definitions.Kaelynna" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}
fn kaelynna_shout(w: &mut World, id: MobileId, pm: MobileId) {
    quester::tell(w, Quester::Mobile(id), pm, 1078125);
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Mithneral`.
    MITHNERAL, MITHNERAL_C = "Server.Engines.MLQuests.Definitions.Mithneral" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}
fn mithneral_shout(w: &mut World, id: MobileId, pm: MobileId) {
    quester::tell(w, Quester::Mobile(id), pm, 1078127);
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.AndreasVesalius`.
    ANDREAS_VESALIUS, ANDREAS_VESALIUS_C = "Server.Engines.MLQuests.Definitions.AndreasVesalius" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}
fn andreas_vesalius_shout(w: &mut World, id: MobileId, pm: MobileId) {
    quester::tell(w, Quester::Mobile(id), pm, 1078138);
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Avicenna`.
    AVICENNA, AVICENNA_C = "Server.Engines.MLQuests.Definitions.Avicenna" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}
fn avicenna_shout(w: &mut World, id: MobileId, pm: MobileId) {
    quester::tell(w, Quester::Mobile(id), pm, 1078137);
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.SarsmeaSmythe`.
    SARSMEA_SMYTHE, SARSMEA_SMYTHE_C = "Server.Engines.MLQuests.Definitions.SarsmeaSmythe" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}
fn sarsmea_smythe_shout(w: &mut World, id: MobileId, pm: MobileId) {
    quester::tell(w, Quester::Mobile(id), pm, 1078139);
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Chiyo`.
    CHIYO, CHIYO_C = "Server.Engines.MLQuests.Definitions.Chiyo" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}
fn chiyo_shout(w: &mut World, id: MobileId, pm: MobileId) {
    quester::tell(w, Quester::Mobile(id), pm, 1078165);
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Jun`.
    JUN, JUN_C = "Server.Engines.MLQuests.Definitions.Jun" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}
fn jun_shout(w: &mut World, id: MobileId, pm: MobileId) {
    quester::tell(w, Quester::Mobile(id), pm, 1078175);
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Walker`.
    WALKER, WALKER_C = "Server.Engines.MLQuests.Definitions.Walker" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}
fn walker_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = [1078213, 1078212, 1078214][rnd::below(w, 3) as usize];
    quester::tell(w, Quester::Mobile(id), pm, v1);
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.JacobWaltz`.
    JACOB_WALTZ, JACOB_WALTZ_C = "Server.Engines.MLQuests.Definitions.JacobWaltz" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}
fn jacob_waltz_shout(w: &mut World, id: MobileId, pm: MobileId) {
    quester::tell(w, Quester::Mobile(id), pm, 1078124);
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Andric`.
    ANDRIC, ANDRIC_C = "Server.Engines.MLQuests.Definitions.Andric" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}
fn andric_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = [1074205, 1074213][rnd::below(w, 2) as usize];
    quester::tell(w, Quester::Mobile(id), pm, v1);
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Kashiel`.
    KASHIEL, KASHIEL_C = "Server.Engines.MLQuests.Definitions.Kashiel" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Asandos`.
    ASANDOS, ASANDOS_C = "Server.Engines.MLQuests.Definitions.Asandos" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}
fn asandos_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = [1074205, 1074213][rnd::below(w, 2) as usize];
    quester::tell(w, Quester::Mobile(id), pm, v1);
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Clairesse`.
    CLAIRESSE, CLAIRESSE_C = "Server.Engines.MLQuests.Definitions.Clairesse" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}
fn clairesse_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = [1074205, 1074213][rnd::below(w, 2) as usize];
    quester::tell(w, Quester::Mobile(id), pm, v1);
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Gervis`.
    GERVIS, GERVIS_C = "Server.Engines.MLQuests.Definitions.Gervis" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}
fn gervis_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = [1074205, 1074213, 1074211][rnd::below(w, 3) as usize];
    quester::tell(w, Quester::Mobile(id), pm, v1);
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Mugg`.
    MUGG, MUGG_C = "Server.Engines.MLQuests.Definitions.Mugg" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}
fn mugg_shout(w: &mut World, id: MobileId, pm: MobileId) {
    quester::tell(w, Quester::Mobile(id), pm, 1074211);
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Lowel`.
    LOWEL, LOWEL_C = "Server.Engines.MLQuests.Definitions.Lowel" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}
fn lowel_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = [1074205, 1074213][rnd::below(w, 2) as usize];
    quester::tell(w, Quester::Mobile(id), pm, v1);
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Lyle`.
    LYLE, LYLE_C = "Server.Engines.MLQuests.Definitions.Lyle" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}
fn lyle_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = [1074205, 1074213][rnd::below(w, 2) as usize];
    quester::tell(w, Quester::Mobile(id), pm, v1);
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Nibbet`.
    NIBBET, NIBBET_C = "Server.Engines.MLQuests.Definitions.Nibbet" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Norton`.
    NORTON, NORTON_C = "Server.Engines.MLQuests.Definitions.Norton" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}
fn norton_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = [1074205, 1074213, 1074211][rnd::below(w, 3) as usize];
    quester::tell(w, Quester::Mobile(id), pm, v1);
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Sadrah`.
    SADRAH, SADRAH_C = "Server.Engines.MLQuests.Definitions.Sadrah" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}
fn sadrah_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = [1074205, 1074213, 1074211][rnd::below(w, 3) as usize];
    quester::tell(w, Quester::Mobile(id), pm, v1);
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Hargrove`.
    HARGROVE, HARGROVE_C = "Server.Engines.MLQuests.Definitions.Hargrove" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}
fn hargrove_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = [1074213, 1074211][rnd::below(w, 2) as usize];
    quester::tell(w, Quester::Mobile(id), pm, v1);
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Beotham`.
    BEOTHAM, BEOTHAM_C = "Server.Engines.MLQuests.Definitions.Beotham" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}
fn beotham_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = [1074187, 1074184][rnd::below(w, 2) as usize];
    quester::tell(w, Quester::Mobile(id), pm, v1);
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Danoel`.
    DANOEL, DANOEL_C = "Server.Engines.MLQuests.Definitions.Danoel" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}
fn danoel_shout(w: &mut World, id: MobileId, pm: MobileId) {
    quester::tell(w, Quester::Mobile(id), pm, 1074197);
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Tallinin`.
    TALLININ, TALLININ_C = "Server.Engines.MLQuests.Definitions.Tallinin" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}
fn tallinin_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = [1074188, 1074211][rnd::below(w, 2) as usize];
    quester::tell(w, Quester::Mobile(id), pm, v1);
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Tiana`.
    TIANA, TIANA_C = "Server.Engines.MLQuests.Definitions.Tiana" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}
fn tiana_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = rnd::below(w, 2);
    quester::tell(w, Quester::Mobile(id), pm, 1074214 + v1);
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.LorekeeperOolua`.
    LOREKEEPER_OOLUA, LOREKEEPER_OOLUA_C = "Server.Engines.MLQuests.Definitions.LorekeeperOolua" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}
fn lorekeeper_oolua_shout(w: &mut World, id: MobileId, pm: MobileId) {
    quester::tell(w, Quester::Mobile(id), pm, 1074187);
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.LorekeeperRollarn`.
    LOREKEEPER_ROLLARN, LOREKEEPER_ROLLARN_C = "Server.Engines.MLQuests.Definitions.LorekeeperRollarn" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}
fn lorekeeper_rollarn_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = [1074196, 1074197][rnd::below(w, 2) as usize];
    quester::tell(w, Quester::Mobile(id), pm, v1);
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Dallid`.
    DALLID, DALLID_C = "Server.Engines.MLQuests.Definitions.Dallid" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}
fn dallid_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = [1074185, 1074195][rnd::below(w, 2) as usize];
    quester::tell(w, Quester::Mobile(id), pm, v1);
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Canir`.
    CANIR, CANIR_C = "Server.Engines.MLQuests.Definitions.Canir" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}
fn canir_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = [1074193, 1074186][rnd::below(w, 2) as usize];
    quester::tell(w, Quester::Mobile(id), pm, v1);
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Yellienir`.
    YELLIENIR, YELLIENIR_C = "Server.Engines.MLQuests.Definitions.Yellienir" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.ElderOnallan`.
    ELDER_ONALLAN, ELDER_ONALLAN_C = "Server.Engines.MLQuests.Definitions.ElderOnallan" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}
fn elder_onallan_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = [1074217, 1074218][rnd::below(w, 2) as usize];
    quester::tell(w, Quester::Mobile(id), pm, v1);
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Aeluva`.
    AELUVA, AELUVA_C = "Server.Engines.MLQuests.Definitions.Aeluva" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}
fn aeluva_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = rnd::from(w, 1074206, 2);
    quester::tell(w, Quester::Mobile(id), pm, v1);
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Koole`.
    KOOLE, KOOLE_C = "Server.Engines.MLQuests.Definitions.Koole" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}
fn koole_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = [1074186, 1074218][rnd::below(w, 2) as usize];
    quester::tell(w, Quester::Mobile(id), pm, v1);
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Synaeva`.
    SYNAEVA, SYNAEVA_C = "Server.Engines.MLQuests.Definitions.Synaeva" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}
fn synaeva_shout(w: &mut World, id: MobileId, pm: MobileId) {
    quester::tell(w, Quester::Mobile(id), pm, 1074223);
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.ElderBrae`.
    ELDER_BRAE, ELDER_BRAE_C = "Server.Engines.MLQuests.Definitions.ElderBrae" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}
fn elder_brae_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = [1074215, 1074218][rnd::below(w, 2) as usize];
    quester::tell(w, Quester::Mobile(id), pm, v1);
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.LorekeeperBroolol`.
    LOREKEEPER_BROOLOL, LOREKEEPER_BROOLOL_C = "Server.Engines.MLQuests.Definitions.LorekeeperBroolol" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}
fn lorekeeper_broolol_shout(w: &mut World, id: MobileId, pm: MobileId) {
    quester::tell(w, Quester::Mobile(id), pm, 1074200);
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Emilio`.
    EMILIO, EMILIO_C = "Server.Engines.MLQuests.Definitions.Emilio" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}

creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Thalia`.
    THALIA, THALIA_C = "Server.Engines.MLQuests.Definitions.Thalia" : BASE_CREATURE / BASE_CREATURE_C, template, encoded false;
    m {
    }
}

/// Registers the generated mobile classes.
pub fn register(types: &mut TypeRegistry) {
    types.register_mobile(&BEN, &[]);
    types.register_mobile(&FREDERIC, &[]);
    types.register_mobile(&LEON, &[]);
    types.register_mobile(&ANDROS, &[]);
    types.register_mobile(&KIA, &[]);
    types.register_mobile(&EMERILLO, &[]);
    types.register_mobile(&NYTHALIA, &[]);
    types.register_mobile(&JAMAL, &[]);
    types.register_mobile(&IOSEP, &[]);
    types.register_mobile(&AURELIA, &[]);
    types.register_mobile(&SKELETON_OF_SZANDOR, &[]);
    types.register_mobile(&SARIL, &[]);
    types.register_mobile(&CAILLA, &[]);
    types.register_mobile(&TAMM, &[]);
    types.register_mobile(&LANDY, &[]);
    types.register_mobile(&ALEJAHA, &[]);
    types.register_mobile(&MIELAN, &[]);
    types.register_mobile(&CIALA, &[]);
    types.register_mobile(&ANIEL, &[]);
    types.register_mobile(&AULAN, &[]);
    types.register_mobile(&BRINNAE, &[]);
    types.register_mobile(&CAELAS, &[]);
    types.register_mobile(&CLEHIN, &[]);
    types.register_mobile(&CLOORNE, &[]);
    types.register_mobile(&SALAENIH, &[]);
    types.register_mobile(&VILO, &[]);
    types.register_mobile(&THOLEF, &[]);
    types.register_mobile(&TILLANIL, &[]);
    types.register_mobile(&WAELIAN, &[]);
    types.register_mobile(&SLEEN, &[]);
    types.register_mobile(&UNOELIL, &[]);
    types.register_mobile(&ANOLLY, &[]);
    types.register_mobile(&JUSAE, &[]);
    types.register_mobile(&CILLITHA, &[]);
    types.register_mobile(&LOHN, &[]);
    types.register_mobile(&OLLA, &[]);
    types.register_mobile(&THALLARY, &[]);
    types.register_mobile(&AHIE, &[]);
    types.register_mobile(&TYEELOR, &[]);
    types.register_mobile(&ATHAILON, &[]);
    types.register_mobile(&ELDER_TAELLIA, &[]);
    types.register_mobile(&ELDER_MALLEW, &[]);
    types.register_mobile(&ELDER_ABBEIN, &[]);
    types.register_mobile(&ELDER_VICAIE, &[]);
    types.register_mobile(&ELDER_JOTHAN, &[]);
    types.register_mobile(&ELDER_ALETHANIAN, &[]);
    types.register_mobile(&REBINIL, &[]);
    types.register_mobile(&ALUNIOL, &[]);
    types.register_mobile(&OLAENI, &[]);
    types.register_mobile(&BOLAEVIN, &[]);
    types.register_mobile(&LOREKEEPER_ANEEN, &[]);
    types.register_mobile(&DAELAS, &[]);
    types.register_mobile(&ALELLE, &[]);
    types.register_mobile(&LOREKEEPER_NILLAEN, &[]);
    types.register_mobile(&LOREKEEPER_RYAL, &[]);
    types.register_mobile(&BRAEN, &[]);
    types.register_mobile(&ELDER_ACOB, &[]);
    types.register_mobile(&LOREKEEPER_CALENDOR, &[]);
    types.register_mobile(&LOREKEEPER_SIARRA, &[]);
    types.register_mobile(&ENIGMA, &[]);
    types.register_mobile(&SLEDGE, &[]);
    types.register_mobile(&PATRICUS, &[]);
    types.register_mobile(&BELULAH, &[]);
    types.register_mobile(&EVAN, &[]);
    types.register_mobile(&REGINA, &[]);
    types.register_mobile(&GRANDPA_CHARLEY, &[]);
    types.register_mobile(&JELRICE, &[]);
    types.register_mobile(&YORUS, &[]);
    types.register_mobile(&DRITHEN, &[]);
    types.register_mobile(&AERNYA, &[]);
    types.register_mobile(&GORROW, &[]);
    types.register_mobile(&MASTER_GNOSOS, &[]);
    types.register_mobile(&DIMETHRO, &[]);
    types.register_mobile(&CHURCHILL, &[]);
    types.register_mobile(&RECARO, &[]);
    types.register_mobile(&ALDEN_ARMSTRONG, &[]);
    types.register_mobile(&JOCKLES, &[]);
    types.register_mobile(&TYL_ARIADNE, &[]);
    types.register_mobile(&ALEFIAN, &[]);
    types.register_mobile(&GUSTAR, &[]);
    types.register_mobile(&JILLIAN, &[]);
    types.register_mobile(&KAELYNNA, &[]);
    types.register_mobile(&MITHNERAL, &[]);
    types.register_mobile(&ANDREAS_VESALIUS, &[]);
    types.register_mobile(&AVICENNA, &[]);
    types.register_mobile(&SARSMEA_SMYTHE, &[]);
    types.register_mobile(&CHIYO, &[]);
    types.register_mobile(&JUN, &[]);
    types.register_mobile(&WALKER, &[]);
    types.register_mobile(&JACOB_WALTZ, &[]);
    types.register_mobile(&ANDRIC, &[]);
    types.register_mobile(&KASHIEL, &[]);
    types.register_mobile(&ASANDOS, &[]);
    types.register_mobile(&CLAIRESSE, &[]);
    types.register_mobile(&GERVIS, &[]);
    types.register_mobile(&MUGG, &[]);
    types.register_mobile(&LOWEL, &[]);
    types.register_mobile(&LYLE, &[]);
    types.register_mobile(&NIBBET, &[]);
    types.register_mobile(&NORTON, &[]);
    types.register_mobile(&SADRAH, &[]);
    types.register_mobile(&HARGROVE, &[]);
    types.register_mobile(&BEOTHAM, &[]);
    types.register_mobile(&DANOEL, &[]);
    types.register_mobile(&TALLININ, &[]);
    types.register_mobile(&TIANA, &[]);
    types.register_mobile(&LOREKEEPER_OOLUA, &[]);
    types.register_mobile(&LOREKEEPER_ROLLARN, &[]);
    types.register_mobile(&DALLID, &[]);
    types.register_mobile(&CANIR, &[]);
    types.register_mobile(&YELLIENIR, &[]);
    types.register_mobile(&ELDER_ONALLAN, &[]);
    types.register_mobile(&AELUVA, &[]);
    types.register_mobile(&KOOLE, &[]);
    types.register_mobile(&SYNAEVA, &[]);
    types.register_mobile(&ELDER_BRAE, &[]);
    types.register_mobile(&LOREKEEPER_BROOLOL, &[]);
    types.register_mobile(&EMILIO, &[]);
    types.register_mobile(&THALIA, &[]);
}

/// `Shout(pm)` overrides: class full name -> function.
#[allow(clippy::type_complexity)] // the callback type, spelled out
pub static SHOUTS: &[(&str, fn(&mut World, MobileId, MobileId))] = &[
    (
        "Server.Engines.MLQuests.Definitions.Emerillo",
        emerillo_shout,
    ),
    ("Server.Engines.MLQuests.Definitions.Iosep", iosep_shout),
    ("Server.Engines.MLQuests.Definitions.Saril", saril_shout),
    ("Server.Engines.MLQuests.Definitions.Cailla", cailla_shout),
    ("Server.Engines.MLQuests.Definitions.Tamm", tamm_shout),
    ("Server.Engines.MLQuests.Definitions.Landy", landy_shout),
    ("Server.Engines.MLQuests.Definitions.Alejaha", alejaha_shout),
    ("Server.Engines.MLQuests.Definitions.Mielan", mielan_shout),
    ("Server.Engines.MLQuests.Definitions.Ciala", ciala_shout),
    ("Server.Engines.MLQuests.Definitions.Aniel", aniel_shout),
    ("Server.Engines.MLQuests.Definitions.Aulan", aulan_shout),
    ("Server.Engines.MLQuests.Definitions.Brinnae", brinnae_shout),
    ("Server.Engines.MLQuests.Definitions.Caelas", caelas_shout),
    ("Server.Engines.MLQuests.Definitions.Clehin", clehin_shout),
    ("Server.Engines.MLQuests.Definitions.Cloorne", cloorne_shout),
    (
        "Server.Engines.MLQuests.Definitions.Salaenih",
        salaenih_shout,
    ),
    ("Server.Engines.MLQuests.Definitions.Vilo", vilo_shout),
    ("Server.Engines.MLQuests.Definitions.Tholef", tholef_shout),
    (
        "Server.Engines.MLQuests.Definitions.Tillanil",
        tillanil_shout,
    ),
    ("Server.Engines.MLQuests.Definitions.Waelian", waelian_shout),
    ("Server.Engines.MLQuests.Definitions.Sleen", sleen_shout),
    ("Server.Engines.MLQuests.Definitions.Unoelil", unoelil_shout),
    ("Server.Engines.MLQuests.Definitions.Jusae", jusae_shout),
    (
        "Server.Engines.MLQuests.Definitions.Cillitha",
        cillitha_shout,
    ),
    ("Server.Engines.MLQuests.Definitions.Lohn", lohn_shout),
    ("Server.Engines.MLQuests.Definitions.Olla", olla_shout),
    (
        "Server.Engines.MLQuests.Definitions.Thallary",
        thallary_shout,
    ),
    ("Server.Engines.MLQuests.Definitions.Ahie", ahie_shout),
    (
        "Server.Engines.MLQuests.Definitions.LorekeeperRyal",
        lorekeeper_ryal_shout,
    ),
    ("Server.Engines.MLQuests.Definitions.Braen", braen_shout),
    (
        "Server.Engines.MLQuests.Definitions.ElderAcob",
        elder_acob_shout,
    ),
    (
        "Server.Engines.MLQuests.Definitions.LorekeeperCalendor",
        lorekeeper_calendor_shout,
    ),
    (
        "Server.Engines.MLQuests.Definitions.LorekeeperSiarra",
        lorekeeper_siarra_shout,
    ),
    ("Server.Engines.MLQuests.Definitions.Sledge", sledge_shout),
    ("Server.Engines.MLQuests.Definitions.Belulah", belulah_shout),
    ("Server.Engines.MLQuests.Definitions.Jelrice", jelrice_shout),
    ("Server.Engines.MLQuests.Definitions.Yorus", yorus_shout),
    ("Server.Engines.MLQuests.Definitions.Drithen", drithen_shout),
    ("Server.Engines.MLQuests.Definitions.Gorrow", gorrow_shout),
    (
        "Server.Engines.MLQuests.Definitions.MasterGnosos",
        master_gnosos_shout,
    ),
    (
        "Server.Engines.MLQuests.Definitions.Dimethro",
        dimethro_shout,
    ),
    (
        "Server.Engines.MLQuests.Definitions.Churchill",
        churchill_shout,
    ),
    ("Server.Engines.MLQuests.Definitions.Recaro", recaro_shout),
    (
        "Server.Engines.MLQuests.Definitions.AldenArmstrong",
        alden_armstrong_shout,
    ),
    ("Server.Engines.MLQuests.Definitions.Jockles", jockles_shout),
    (
        "Server.Engines.MLQuests.Definitions.TylAriadne",
        tyl_ariadne_shout,
    ),
    ("Server.Engines.MLQuests.Definitions.Alefian", alefian_shout),
    ("Server.Engines.MLQuests.Definitions.Gustar", gustar_shout),
    ("Server.Engines.MLQuests.Definitions.Jillian", jillian_shout),
    (
        "Server.Engines.MLQuests.Definitions.Kaelynna",
        kaelynna_shout,
    ),
    (
        "Server.Engines.MLQuests.Definitions.Mithneral",
        mithneral_shout,
    ),
    (
        "Server.Engines.MLQuests.Definitions.AndreasVesalius",
        andreas_vesalius_shout,
    ),
    (
        "Server.Engines.MLQuests.Definitions.Avicenna",
        avicenna_shout,
    ),
    (
        "Server.Engines.MLQuests.Definitions.SarsmeaSmythe",
        sarsmea_smythe_shout,
    ),
    ("Server.Engines.MLQuests.Definitions.Chiyo", chiyo_shout),
    ("Server.Engines.MLQuests.Definitions.Jun", jun_shout),
    ("Server.Engines.MLQuests.Definitions.Walker", walker_shout),
    (
        "Server.Engines.MLQuests.Definitions.JacobWaltz",
        jacob_waltz_shout,
    ),
    ("Server.Engines.MLQuests.Definitions.Andric", andric_shout),
    ("Server.Engines.MLQuests.Definitions.Asandos", asandos_shout),
    (
        "Server.Engines.MLQuests.Definitions.Clairesse",
        clairesse_shout,
    ),
    ("Server.Engines.MLQuests.Definitions.Gervis", gervis_shout),
    ("Server.Engines.MLQuests.Definitions.Mugg", mugg_shout),
    ("Server.Engines.MLQuests.Definitions.Lowel", lowel_shout),
    ("Server.Engines.MLQuests.Definitions.Lyle", lyle_shout),
    ("Server.Engines.MLQuests.Definitions.Norton", norton_shout),
    ("Server.Engines.MLQuests.Definitions.Sadrah", sadrah_shout),
    (
        "Server.Engines.MLQuests.Definitions.Hargrove",
        hargrove_shout,
    ),
    ("Server.Engines.MLQuests.Definitions.Beotham", beotham_shout),
    ("Server.Engines.MLQuests.Definitions.Danoel", danoel_shout),
    (
        "Server.Engines.MLQuests.Definitions.Tallinin",
        tallinin_shout,
    ),
    ("Server.Engines.MLQuests.Definitions.Tiana", tiana_shout),
    (
        "Server.Engines.MLQuests.Definitions.LorekeeperOolua",
        lorekeeper_oolua_shout,
    ),
    (
        "Server.Engines.MLQuests.Definitions.LorekeeperRollarn",
        lorekeeper_rollarn_shout,
    ),
    ("Server.Engines.MLQuests.Definitions.Dallid", dallid_shout),
    ("Server.Engines.MLQuests.Definitions.Canir", canir_shout),
    (
        "Server.Engines.MLQuests.Definitions.ElderOnallan",
        elder_onallan_shout,
    ),
    ("Server.Engines.MLQuests.Definitions.Aeluva", aeluva_shout),
    ("Server.Engines.MLQuests.Definitions.Koole", koole_shout),
    ("Server.Engines.MLQuests.Definitions.Synaeva", synaeva_shout),
    (
        "Server.Engines.MLQuests.Definitions.ElderBrae",
        elder_brae_shout,
    ),
    (
        "Server.Engines.MLQuests.Definitions.LorekeeperBroolol",
        lorekeeper_broolol_shout,
    ),
];
