// GlacialStaff: rolled hit harm, mage weapon and cold damage.
fn on_spawn(staff) {
    set_attr(staff, "WeaponAttributes", "HitHarm", 5 * random(1, 5));
    set_attr(staff, "WeaponAttributes", "MageWeapon", random(5, 10));
    set_attr(staff, "ElementAttributes", "Cold", 20 + 5 * random(0, 6));
}
