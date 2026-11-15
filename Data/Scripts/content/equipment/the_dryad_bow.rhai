// TheDryadBow: one skill of six at +5, or +10 one time in four.
fn on_spawn(bow) {
    let skill = pick(["Archery", "Healing", "MagicResist", "Peacemaking", "Chivalry", "Ninjitsu"]);
    let bonus = if random_below(4) == 0 { 10.0 } else { 5.0 };
    skill_bonus(bow, 0, skill, bonus);
}
