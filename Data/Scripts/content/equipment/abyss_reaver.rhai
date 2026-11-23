// AbyssReaver: rolled throwing bonus and damage increase. The class replays no template of its own (its parent
// Cyclone's replay runs this hook), so the constant slayer is set here too.
fn on_spawn(w) {
    skill_bonus(w, 0, "Throwing", random(5, 10).to_float());
    set_attr(w, "Attributes", "WeaponDamage", random(25, 35));
    set_slayer(w, "Exorcism");
}
