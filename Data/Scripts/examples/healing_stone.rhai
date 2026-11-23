// Examples.HealingStone (Data/Templates/Items/examples.json): double-click to heal, then it recharges for ten
// seconds. Make one with `[add Examples.HealingStone`.

fn on_use(stone, from) {
    if stone.hue == 0x3B2 {
        send_message(from, "The stone is still recharging.");
        return true;
    }
    from.hits = from.hits + random(10, 20);
    send_message(from, "The stone glows and your wounds close.");
    stone.hue = 0x3B2; // grey while it recharges
    timer(10000, "recharge", stone);
    true // handled: the item's own double-click does not run
}

// Called by the timer. If the stone was deleted meanwhile, setting its hue does nothing.
fn recharge(stone) {
    stone.hue = 1153;
}
