// Golem(summoned, scalar): every stat scaled by `scalar` (passed as its decimal text). A summoned golem is
// hued, less fire resistant, has 10 fame and karma and packs nothing. Body and control slots are in the template.
fn sc(v, s) {
    (v.to_float() * s).to_int()
}

fn on_spawn(g, summoned, scalar) {
    let s = parse_float(scalar);
    let summoned = summoned != 0;
    if summoned {
        set_hue(g, 2101);
    }
    set_str_range(g, sc(251, s), sc(350, s));
    set_dex_range(g, sc(76, s), sc(100, s));
    set_int_range(g, sc(101, s), sc(150, s));
    set_hits_range(g, sc(151, s), sc(210, s));
    set_damage(g, sc(13, s), sc(24, s));
    set_damage_type(g, "Physical", 100);
    set_resistance_range(g, "Physical", sc(35, s), sc(55, s));
    if summoned {
        set_resistance_range(g, "Fire", sc(50, s), sc(60, s));
    } else {
        set_resistance(g, "Fire", sc(100, s));
    }
    set_resistance_range(g, "Cold", sc(10, s), sc(30, s));
    set_resistance_range(g, "Poison", sc(10, s), sc(25, s));
    set_resistance_range(g, "Energy", sc(30, s), sc(40, s));
    set_skill_range(g, "MagicResist", 150.1 * s, 190.0 * s);
    set_skill_range(g, "Tactics", 60.1 * s, 100.0 * s);
    set_skill_range(g, "Wrestling", 60.1 * s, 100.0 * s);
    if summoned {
        set_fame(g, 10);
        set_karma(g, 10);
    } else {
        set_fame(g, 3500);
        set_karma(g, -3500);
        pack_new_amount(g, "Server.Items.IronIngot", random(13, 21));
        if random_double() < 0.1 {
            pack_new(g, "Server.Items.PowerCrystal");
        }
        if random_double() < 0.15 {
            pack_new(g, "Server.Items.ClockworkAssembly");
        }
        if random_double() < 0.2 {
            pack_new(g, "Server.Items.ArcaneGem");
        }
        if random_double() < 0.25 {
            pack_new(g, "Server.Items.Gears");
        }
    }
}
