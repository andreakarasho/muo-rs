// Server.Mobiles.GrimmochDrummel (Data/Templates/Mobiles/khaldun/mobiles.json): the pack after the template.
// 40 arrows, a fire horn 3 times in 100 and one of his journals (`Loot.RandomGrimmochJournal()`) one time in 3.

fn on_spawn(m) {
    pack_item(m, "Server.Items.Arrow", 40);
    if random_below(100) < 3 {
        pack_item(m, "Server.Items.FireHorn", 1);
    }
    if random_below(3) < 1 {
        let journal = pick([
            "Server.Items.GrimmochJournal1", "Server.Items.GrimmochJournal2", "Server.Items.GrimmochJournal3",
            "Server.Items.GrimmochJournal6", "Server.Items.GrimmochJournal7", "Server.Items.GrimmochJournal11",
            "Server.Items.GrimmochJournal14", "Server.Items.GrimmochJournal17", "Server.Items.GrimmochJournal23"
        ]);
        pack_item(m, journal, 1);
    }
}
