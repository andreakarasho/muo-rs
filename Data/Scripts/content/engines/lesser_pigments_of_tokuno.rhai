// Server.Items.LesserPigmentsOfTokuno (Data/Templates/Items/engines/misc.json): one use of a random type
// (`(LesserPigmentType)Utility.Random(0, 11)`), or the type it is made with.

fn on_spawn(item) {
    on_spawn(item, random_below(11));
}

fn on_spawn(item, kind) {
    set_prop(item, "Type", kind);
}
