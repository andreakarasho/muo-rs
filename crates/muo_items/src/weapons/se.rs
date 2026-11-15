//! Weapon classes of se; see weapons/mod.rs.

use super::*;

weapon_leaf! {
    /// `Server.Items.Bokuto`.
    BOKUTO, BOKUTO_CLASS, bokuto_new = "Server.Items.Bokuto" : BASE_SWORD / BASE_SWORD_CLASS, json (), v0;
}

weapon_leaf! {
    /// `Server.Items.Daisho`.
    DAISHO, DAISHO_CLASS, daisho_new = "Server.Items.Daisho" : BASE_SWORD / BASE_SWORD_CLASS, json (), v0;
    @json;
}

weapon_leaf! {
    /// `Server.Items.Kama`.
    KAMA, KAMA_CLASS, kama_new = "Server.Items.Kama" : BASE_KNIFE / BASE_KNIFE_CLASS, json (), v0;
    @json;
}

weapon_leaf! {
    /// `Server.Items.Lajatang`.
    LAJATANG, LAJATANG_CLASS, lajatang_new = "Server.Items.Lajatang" : BASE_KNIFE / BASE_KNIFE_CLASS, json (), v0;
    @json;
}

weapon_leaf! {
    /// `Server.Items.NoDachi`.
    NO_DACHI, NO_DACHI_CLASS, no_dachi_new = "Server.Items.NoDachi" : BASE_SWORD / BASE_SWORD_CLASS, json (), v0;
    @json;
}

weapon_leaf! {
    /// `Server.Items.Nunchaku`.
    NUNCHAKU, NUNCHAKU_CLASS, nunchaku_new = "Server.Items.Nunchaku" : BASE_BASHING / BASE_BASHING_CLASS, json (), v0;
}

weapon_leaf! {
    /// `Server.Items.Sai`.
    SAI, SAI_CLASS, sai_new = "Server.Items.Sai" : BASE_KNIFE / BASE_KNIFE_CLASS, json (), v0;
    @json;
}

weapon_leaf! {
    /// `Server.Items.Tekagi`.
    TEKAGI, TEKAGI_CLASS, tekagi_new = "Server.Items.Tekagi" : BASE_KNIFE / BASE_KNIFE_CLASS, json (), v0;
    @json;
}

weapon_leaf! {
    /// `Server.Items.Tessen`.
    TESSEN, TESSEN_CLASS, tessen_new = "Server.Items.Tessen" : BASE_BASHING / BASE_BASHING_CLASS, json (), v0;
    @json;
}

weapon_leaf! {
    /// `Server.Items.Tetsubo`.
    TETSUBO, TETSUBO_CLASS, tetsubo_new = "Server.Items.Tetsubo" : BASE_BASHING / BASE_BASHING_CLASS, json (), v0;
    @json;
}

weapon_leaf! {
    /// `Server.Items.Wakizashi`.
    WAKIZASHI, WAKIZASHI_CLASS, wakizashi_new = "Server.Items.Wakizashi" : BASE_SWORD / BASE_SWORD_CLASS, json (), v0;
    @json;
}

weapon_leaf! {
    /// `Server.Items.Yumi`.
    YUMI, YUMI_CLASS, yumi_new = "Server.Items.Yumi" : BASE_RANGED / BASE_RANGED_CLASS, json (), v0;
    @json;
}

pub fn register(types: &mut TypeRegistry) {
    types.register_item(&BOKUTO, &[]);
    types.register_item(&DAISHO, &[]);
    types.register_item(&KAMA, &[]);
    types.register_item(&LAJATANG, &[]);
    types.register_item(&NO_DACHI, &[]);
    types.register_item(&NUNCHAKU, &[]);
    types.register_item(&SAI, &[]);
    types.register_item(&TEKAGI, &[]);
    types.register_item(&TESSEN, &[]);
    types.register_item(&TETSUBO, &[]);
    types.register_item(&WAKIZASHI, &[]);
    types.register_item(&YUMI, &[]);
}
