// Server.Engines.MLQuests.Definitions.GrandpaCharley (Data/Templates/Mobiles/ml_quests/npcs.json): the rest of
// the constructor after the template. Hair and beard share one hue, 0x3B2 or 0x3B3.

fn on_spawn(m) {
    let h = 0x3B2 + random_below(2);
    mlq_random_hair(m, h);
    mlq_facial_hair(m, 0x203E, h);
    mlq_skill_range(m, "ItemID", 80.0, 90.0);
    mlq_equip(m, "WideBrimHat", random_hue("randomNondyedHue"));
    mlq_equip(m, "FancyShirt", random_hue("randomNondyedHue"));
    mlq_equip(m, "LongPants", random_hue("randomNondyedHue"));
    mlq_equip(m, "Sandals", random_hue("randomNeutralHue"));
    mlq_equip(m, "ShepherdsCrook");
    mlq_equip(m, "Backpack");
}
