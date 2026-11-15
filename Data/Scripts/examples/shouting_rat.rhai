// Examples.ShoutingRat (Data/Templates/Mobiles/examples.json): a rat that shouts when it dies and leaves extra
// gold in its corpse. Make one with `[add Examples.ShoutingRat`.
//
// Only functions run; code outside a function is never evaluated. Edit this file and type `[reloadscripts`.

// After the template is applied, before the rat is placed in the world.
fn on_spawn(m) {
    m.hue = 0x21;
}

// After the corpse is made. Returning false keeps the default death behavior (loot, fame, karma, ...).
fn on_death(m, corpse) {
    say(m, "Squeak! You will pay for this!");
    if type_of(corpse) == "Item" {
        add_item(corpse, "Server.Items.Gold", random(50, 100));
    }
    false
}
