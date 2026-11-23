// Examples.RandomBlade (Data/Templates/Items/examples.json): a longsword whose magic properties are rolled when it
// is made. Fixed values belong in the template ("attributes": { ... }); a script is for values that are computed.
// Make one with `[add Examples.RandomBlade`.

fn on_spawn(blade) {
    set_attr(blade, "Attributes", "WeaponDamage", random(20, 40));
    set_attr(blade, "WeaponAttributes", "HitLightning", random(10, 30));
    if random(1, 100) <= 25 {
        skill_bonus(blade, 0, "Swords", 5.0);
        blade.name = "a gleaming blade";
    }
}
