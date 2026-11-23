// Server.Items.MetalPigmentsOfTokuno (Data/Templates/Items/engines/misc.json): `RandomHue()`, one of 29 metal
// hues from 0x961 or none.

fn on_spawn(item) {
    let a = random_below(30);
    item.hue = if a != 0 { a + 0x960 } else { 0 };
}
