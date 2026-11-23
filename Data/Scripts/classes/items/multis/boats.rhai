item("SmallBoat", |c| {
    c.boat_ids = [0, 1, 2, 3];
    c.hold_distance = 4;
    c.tiller_man_distance = -4;
    c.starboard_offset = [2, 0];
    c.port_offset = [-2, 0];
    c.mark_offset = [0, 1, 3];
});

item("SmallDragonBoat", |c| {
    c.boat_ids = [4, 5, 6, 7];
    c.hold_distance = 4;
    c.tiller_man_distance = -4;
    c.starboard_offset = [2, 0];
    c.port_offset = [-2, 0];
    c.mark_offset = [0, 1, 3];
});

item("MediumBoat", |c| {
    c.boat_ids = [8, 9, 10, 11];
    c.hold_distance = 4;
    c.tiller_man_distance = -5;
    c.starboard_offset = [2, 0];
    c.port_offset = [-2, 0];
    c.mark_offset = [0, 1, 3];
});

item("MediumDragonBoat", |c| {
    c.boat_ids = [12, 13, 14, 15];
    c.hold_distance = 4;
    c.tiller_man_distance = -5;
    c.starboard_offset = [2, 0];
    c.port_offset = [-2, 0];
    c.mark_offset = [0, 1, 3];
});

item("LargeBoat", |c| {
    c.boat_ids = [16, 17, 18, 19];
    c.hold_distance = 5;
    c.tiller_man_distance = -5;
    c.starboard_offset = [2, -1];
    c.port_offset = [-2, -1];
    c.mark_offset = [0, 0, 3];
});

item("LargeDragonBoat", |c| {
    c.boat_ids = [20, 21, 22, 23];
    c.hold_distance = 5;
    c.tiller_man_distance = -5;
    c.starboard_offset = [2, -1];
    c.port_offset = [-2, -1];
    c.mark_offset = [0, 0, 3];
});

item("SmallDockedBoat", |c| {
    c.inherits = "Server.Multis.BaseDockedBoat";
    c.multi_id = 0;
    c.offset = [0, 0, 0];
});

item("SmallDockedDragonBoat", |c| {
    c.inherits = "Server.Multis.BaseDockedBoat";
    c.multi_id = 4;
    c.offset = [0, 0, 0];
});

item("MediumDockedBoat", |c| {
    c.inherits = "Server.Multis.BaseDockedBoat";
    c.multi_id = 8;
    c.offset = [0, 0, 0];
});

item("MediumDockedDragonBoat", |c| {
    c.inherits = "Server.Multis.BaseDockedBoat";
    c.multi_id = 12;
    c.offset = [0, 0, 0];
});

item("LargeDockedBoat", |c| {
    c.inherits = "Server.Multis.BaseDockedBoat";
    c.multi_id = 16;
    c.offset = [0, -1, 0];
});

item("LargeDockedDragonBoat", |c| {
    c.inherits = "Server.Multis.BaseDockedBoat";
    c.multi_id = 20;
    c.offset = [0, -1, 0];
});

item("Server.Multis.BaseDockedBoat", |c| {
    c.on_create(|i| {
        i.item_id = 0x14F4;
        i.loot_type = "Blessed";
    });
});
