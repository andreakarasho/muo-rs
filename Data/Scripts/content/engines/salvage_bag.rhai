// Server.Items.SalvageBag (Data/Templates/Items/engines/misc.json): `SalvageBag()` is a random blue bag,
// `SalvageBag(hue)` takes the hue.

fn on_spawn(bag) {
    on_spawn(bag, random_hue("randomBlueHue"));
}

fn on_spawn(bag, hue) {
    bag.hue = hue;
}
