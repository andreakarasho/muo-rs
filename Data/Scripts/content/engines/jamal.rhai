// Server.Engines.MLQuests.Definitions.Jamal (Data/Templates/Mobiles/ml_quests/npcs.json): the rest of the
// constructor after the template. The facial hair takes the hue of the random hair the template gave.

fn on_spawn(m) {
    mlq_random_facial_hair(m, mlq_hair_hue(m));
    mlq_equip(m, "Shirt", 0x1BB);
    mlq_equip(m, "ShortPants", random_hue("randomNeutralHue"));
    mlq_equip(m, "ThighBoots", random_hue("randomAnimalHue"));
    mlq_equip(m, "Backpack");
}
