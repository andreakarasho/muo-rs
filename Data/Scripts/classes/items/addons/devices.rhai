item("ElvenSpinningWheelSouthAddon", |c| {
    c.label = 1031737;
    c.addon_deed = "Server.Items.ElvenSpinningWheelSouthDeed";
    c.on_create(|i| {
        i.addon_components = [[11839, 0, 0, 0]];
    });
});

item("ElvenSpinningWheelSouthDeed", |c| {
    c.label = 1072878;
    c.deed_addon = "Server.Items.ElvenSpinningWheelSouthAddon";
});

item("ElvenSpinningWheelEastAddon", |c| {
    c.label = 1031737;
    c.addon_deed = "Server.Items.ElvenSpinningWheelEastDeed";
    c.on_create(|i| {
        i.addon_components = [[11837, 0, 0, 0]];
    });
});

item("ElvenSpinningWheelEastDeed", |c| {
    c.label = 1073393;
    c.deed_addon = "Server.Items.ElvenSpinningWheelEastAddon";
});

item("SpinningWheelSouthAddon", |c| {
    c.label = 1024117;
    c.addon_deed = "Server.Items.SpinningWheelSouthDeed";
    c.on_create(|i| {
        i.addon_components = [[4117, 0, 0, 0]];
    });
});

item("SpinningWheelSouthDeed", |c| {
    c.label = 1044342;
    c.deed_addon = "Server.Items.SpinningWheelSouthAddon";
});

item("SpinningWheelEastAddon", |c| {
    c.label = 1024117;
    c.addon_deed = "Server.Items.SpinningWheelEastDeed";
    c.on_create(|i| {
        i.addon_components = [[4121, 0, 0, 0]];
    });
});

item("SpinningWheelEastDeed", |c| {
    c.label = 1044341;
    c.deed_addon = "Server.Items.SpinningWheelEastAddon";
});

item("LoomSouthAddon", |c| {
    c.addon_deed = "Server.Items.LoomSouthDeed";
    c.on_create(|i| {
        i.addon_components = [[4193, 0, 0, 0], [4194, 1, 0, 0]];
    });
});

item("LoomSouthDeed", |c| {
    c.label = 1044344;
    c.deed_addon = "Server.Items.LoomSouthAddon";
});

item("LoomEastAddon", |c| {
    c.addon_deed = "Server.Items.LoomEastDeed";
    c.on_create(|i| {
        i.addon_components = [[4192, 0, 0, 0], [4191, 0, 1, 0]];
    });
});

item("LoomEastDeed", |c| {
    c.label = 1044343;
    c.deed_addon = "Server.Items.LoomEastAddon";
});

item("FlourMillSouthAddon", |c| {
    c.addon_deed = "Server.Items.FlourMillSouthDeed";
    c.on_create(|i| {
        i.addon_components = [[6444, 0, -1, 0], [6446, 0, 0, 0], [6448, 0, 1, 0]];
    });
});

item("FlourMillSouthDeed", |c| {
    c.label = 1044348;
    c.deed_addon = "Server.Items.FlourMillSouthAddon";
});

item("FlourMillEastAddon", |c| {
    c.addon_deed = "Server.Items.FlourMillEastDeed";
    c.on_create(|i| {
        i.addon_components = [[6432, -1, 0, 0], [6434, 0, 0, 0], [6436, 1, 0, 0]];
    });
});

item("FlourMillEastDeed", |c| {
    c.label = 1044347;
    c.deed_addon = "Server.Items.FlourMillEastAddon";
});

item("WaterTroughSouthAddon", |c| {
    c.addon_deed = "Server.Items.WaterTroughSouthDeed";
    c.on_create(|i| {
        i.addon_components = [[2883, 0, 0, 0], [2884, 1, 0, 0]];
    });
});

item("WaterTroughSouthDeed", |c| {
    c.label = 1044350;
    c.deed_addon = "Server.Items.WaterTroughSouthAddon";
});

item("WaterTroughEastAddon", |c| {
    c.addon_deed = "Server.Items.WaterTroughEastDeed";
    c.on_create(|i| {
        i.addon_components = [[2881, 0, 0, 0], [2882, 0, 1, 0]];
    });
});

item("WaterTroughEastDeed", |c| {
    c.label = 1044349;
    c.deed_addon = "Server.Items.WaterTroughEastAddon";
});

item("TrainingDummySouthAddon", |c| {
    c.addon_deed = "Server.Items.TrainingDummySouthDeed";
    c.on_create(|i| {
        i.addon_components = [[#{ itemId: 4208, type: "Server.Items.TrainingDummy" }, 0, 0, 0]];
    });
});

item("TrainingDummySouthDeed", |c| {
    c.label = 1044336;
    c.deed_addon = "Server.Items.TrainingDummySouthAddon";
});

item("TrainingDummyEastAddon", |c| {
    c.addon_deed = "Server.Items.TrainingDummyEastDeed";
    c.on_create(|i| {
        i.addon_components = [[#{ itemId: 4212, type: "Server.Items.TrainingDummy" }, 0, 0, 0]];
    });
});

item("TrainingDummyEastDeed", |c| {
    c.label = 1044335;
    c.deed_addon = "Server.Items.TrainingDummyEastAddon";
});

item("PickpocketDipSouthAddon", |c| {
    c.addon_deed = "Server.Items.PickpocketDipSouthDeed";
    c.on_create(|i| {
        i.addon_components = [[#{ itemId: 7872, type: "Server.Items.PickpocketDip" }, 0, 0, 0]];
    });
});

item("PickpocketDipSouthDeed", |c| {
    c.label = 1044338;
    c.deed_addon = "Server.Items.PickpocketDipSouthAddon";
});

item("PickpocketDipEastAddon", |c| {
    c.addon_deed = "Server.Items.PickpocketDipEastDeed";
    c.on_create(|i| {
        i.addon_components = [[#{ itemId: 7875, type: "Server.Items.PickpocketDip" }, 0, 0, 0]];
    });
});

item("PickpocketDipEastDeed", |c| {
    c.label = 1044337;
    c.deed_addon = "Server.Items.PickpocketDipEastAddon";
});

item("ArcheryButteDeed", |c| {
    c.label = 1024106;
    c.deed_addon = "Server.Items.ArcheryButteAddon";
});

item("DartBoardSouthDeed", |c| {
    c.label = 1044325;
    c.deed_addon = "Server.Items.DartBoardSouthAddon";
});

item("DartBoardEastDeed", |c| {
    c.label = 1044326;
    c.deed_addon = "Server.Items.DartBoardEastAddon";
});

item("DartBoardEastAddon", |c| {
    c.addon_deed = "Server.Items.DartBoardEastDeed";
    c.on_create(|i| {
        i.addon_components = [[#{ itemId: 7727, type: "Server.Items.DartBoard" }, 0, 0, 0]];
    });
});

item("DartBoardSouthAddon", |c| {
    c.addon_deed = "Server.Items.DartBoardSouthDeed";
    c.on_create(|i| {
        i.addon_components = [[#{ itemId: 7726, type: "Server.Items.DartBoard" }, 0, 0, 0]];
    });
});

item("RejuvinationAnkhWest", |c| {
    c.on_create(|i| {
        i.addon_components = [
            [#{ itemId: 3, type: "Server.Items.RejuvinationAddonComponent" }, 0, 0, 0],
            [#{ itemId: 2, type: "Server.Items.RejuvinationAddonComponent" }, 0, 1, 0],
        ];
    });
});

item("RejuvinationAnkhNorth", |c| {
    c.on_create(|i| {
        i.addon_components = [
            [#{ itemId: 4, type: "Server.Items.RejuvinationAddonComponent" }, 0, 0, 0],
            [#{ itemId: 5, type: "Server.Items.RejuvinationAddonComponent" }, 1, 0, 0],
        ];
    });
});

item("ArcheryButteAddon", |c| {
    c.addon_deed = "Server.Items.ArcheryButteDeed";
    c.on_create(|i| {
        i.addon_components = [[#{ itemId: 4106, type: "Server.Items.ArcheryButte" }, 0, 0, 0]];
    });
});
