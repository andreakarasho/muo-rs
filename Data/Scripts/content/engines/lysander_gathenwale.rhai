// Server.Mobiles.LysanderGathenwale (Data/Templates/Mobiles/khaldun/mobiles.json): the pack after the template,
// 30 of one random reagent (`Loot.RandomReagent()`).

fn on_spawn(m) {
    let reagent = pick([
        "Server.Items.BlackPearl", "Server.Items.Bloodmoss", "Server.Items.Garlic", "Server.Items.Ginseng",
        "Server.Items.MandrakeRoot", "Server.Items.Nightshade", "Server.Items.SulfurousAsh", "Server.Items.SpidersSilk"
    ]);
    pack_item(m, reagent, 30);
}
