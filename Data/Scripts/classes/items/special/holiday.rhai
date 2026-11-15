item("GiftBox", |c| {
    c.weight = 2.0;
});

item("GreenStocking", |c| {
    c.gump_id = 259;
    c.drop_sound = 66;
    c.on_create(|i| {
        switch random_below(2) {
            0 => {
                i.item_id = 0x2BD9;
            },
            1 => {
                i.item_id = 0x2BDA;
            }
        }
    });
});

item("RedStocking", |c| {
    c.gump_id = 259;
    c.drop_sound = 66;
    c.on_create(|i| {
        switch random_below(2) {
            0 => {
                i.item_id = 0x2BDB;
            },
            1 => {
                i.item_id = 0x2BDC;
            }
        }
    });
});

item("SnowPileDeco", |c| {
    c.label = 1095253;
    c.weight = 2.0;
});

item("Snowman", |c| {
    c.weight = 10.0;
});

item("IcyPatch", |c| {
    c.label = 1095159;
    c.weight = 5.0;
});

item("SnowGlobe", |c| {
    c.weight = 1.0;
    c.on_create(|i| {
        i.item_id = 0xE2F;
        i.loot_type = "Blessed";
        i.light = "Circle150";
    });
});

item("PottedPlantDeed", |c| {
    c.label = 1041114;
    c.weight = 1.0;
    c.on_create(|i| {
        i.item_id = 0x14F0;
        i.loot_type = "Blessed";
    });
});

item("BadCard", |c| {
    c.label = 1041428;
    c.on_create(|i| {
        i.item_id = 0x14EF;
    });
});

item("SnowStatueDeed", |c| {
    c.label = 1114296;
    c.weight = 1.0;
    c.on_create(|i| {
        i.item_id = 0x14F0;
        i.loot_type = "Blessed";
    });
});

item("HolidayBell", |c| {
    c.on_create(|i| {
        i.item_id = 0x1C12;
        i.loot_type = "Blessed";
    });
});

item("Spam", |c| {
    c.on_create(|i| {
        i.item_id = 0x1044;
        i.stackable = false;
        i.loot_type = "Blessed";
    });
});

item("GiftBoxRectangle", |c| {
    c.gump_id = 286;
    c.on_create(|i| {
        if coin() {
            i.item_id = 0x46A5;
        } else {
            i.item_id = 0x46A6;
        }
        switch random_below(20) {
            0 => {
                i.hue = 0x672;
            },
            1 => {
                i.hue = 0x454;
            },
            2 => {
                i.hue = 0x507;
            },
            3 => {
                i.hue = 0x4AC;
            },
            4 => {
                i.hue = 0x504;
            },
            5 => {
                i.hue = 0x84B;
            },
            6 => {
                i.hue = 0x495;
            },
            7 => {
                i.hue = 0x97C;
            },
            8 => {
                i.hue = 0x493;
            },
            9 => {
                i.hue = 0x4A8;
            },
            10 => {
                i.hue = 0x494;
            },
            11 => {
                i.hue = 0x4AA;
            },
            12 => {
                i.hue = 0xB8B;
            },
            13 => {
                i.hue = 0x84F;
            },
            14 => {
                i.hue = 0x491;
            },
            15 => {
                i.hue = 0x851;
            },
            16 => {
                i.hue = 0x503;
            },
            17 => {
                i.hue = 0xB8C;
            },
            18 => {
                i.hue = 0x4AB;
            },
            19 => {
                i.hue = 0x84B;
            }
        }
    });
});

item("GiftBoxCube", |c| {
    c.gump_id = 283;
    c.on_create(|i| {
        i.item_id = 0x46A2;
        switch random_below(20) {
            0 => {
                i.hue = 0x672;
            },
            1 => {
                i.hue = 0x454;
            },
            2 => {
                i.hue = 0x507;
            },
            3 => {
                i.hue = 0x4AC;
            },
            4 => {
                i.hue = 0x504;
            },
            5 => {
                i.hue = 0x84B;
            },
            6 => {
                i.hue = 0x495;
            },
            7 => {
                i.hue = 0x97C;
            },
            8 => {
                i.hue = 0x493;
            },
            9 => {
                i.hue = 0x4A8;
            },
            10 => {
                i.hue = 0x494;
            },
            11 => {
                i.hue = 0x4AA;
            },
            12 => {
                i.hue = 0xB8B;
            },
            13 => {
                i.hue = 0x84F;
            },
            14 => {
                i.hue = 0x491;
            },
            15 => {
                i.hue = 0x851;
            },
            16 => {
                i.hue = 0x503;
            },
            17 => {
                i.hue = 0xB8C;
            },
            18 => {
                i.hue = 0x4AB;
            },
            19 => {
                i.hue = 0x84B;
            }
        }
    });
});

item("GiftBoxCylinder", |c| {
    c.gump_id = 284;
    c.on_create(|i| {
        i.item_id = 0x46A3;
        switch random_below(20) {
            0 => {
                i.hue = 0x672;
            },
            1 => {
                i.hue = 0x454;
            },
            2 => {
                i.hue = 0x507;
            },
            3 => {
                i.hue = 0x4AC;
            },
            4 => {
                i.hue = 0x504;
            },
            5 => {
                i.hue = 0x84B;
            },
            6 => {
                i.hue = 0x495;
            },
            7 => {
                i.hue = 0x97C;
            },
            8 => {
                i.hue = 0x493;
            },
            9 => {
                i.hue = 0x4A8;
            },
            10 => {
                i.hue = 0x494;
            },
            11 => {
                i.hue = 0x4AA;
            },
            12 => {
                i.hue = 0xB8B;
            },
            13 => {
                i.hue = 0x84F;
            },
            14 => {
                i.hue = 0x491;
            },
            15 => {
                i.hue = 0x851;
            },
            16 => {
                i.hue = 0x503;
            },
            17 => {
                i.hue = 0xB8C;
            },
            18 => {
                i.hue = 0x4AB;
            },
            19 => {
                i.hue = 0x84B;
            }
        }
    });
});

item("GiftBoxOctogon", |c| {
    c.gump_id = 285;
    c.on_create(|i| {
        i.item_id = 0x46A4;
        switch random_below(20) {
            0 => {
                i.hue = 0x672;
            },
            1 => {
                i.hue = 0x454;
            },
            2 => {
                i.hue = 0x507;
            },
            3 => {
                i.hue = 0x4AC;
            },
            4 => {
                i.hue = 0x504;
            },
            5 => {
                i.hue = 0x84B;
            },
            6 => {
                i.hue = 0x495;
            },
            7 => {
                i.hue = 0x97C;
            },
            8 => {
                i.hue = 0x493;
            },
            9 => {
                i.hue = 0x4A8;
            },
            10 => {
                i.hue = 0x494;
            },
            11 => {
                i.hue = 0x4AA;
            },
            12 => {
                i.hue = 0xB8B;
            },
            13 => {
                i.hue = 0x84F;
            },
            14 => {
                i.hue = 0x491;
            },
            15 => {
                i.hue = 0x851;
            },
            16 => {
                i.hue = 0x503;
            },
            17 => {
                i.hue = 0xB8C;
            },
            18 => {
                i.hue = 0x4AB;
            },
            19 => {
                i.hue = 0x84B;
            }
        }
    });
});

item("GiftBoxAngel", |c| {
    c.gump_id = 287;
    c.on_create(|i| {
        i.item_id = 0x46A7;
        switch random_below(20) {
            0 => {
                i.hue = 0x672;
            },
            1 => {
                i.hue = 0x454;
            },
            2 => {
                i.hue = 0x507;
            },
            3 => {
                i.hue = 0x4AC;
            },
            4 => {
                i.hue = 0x504;
            },
            5 => {
                i.hue = 0x84B;
            },
            6 => {
                i.hue = 0x495;
            },
            7 => {
                i.hue = 0x97C;
            },
            8 => {
                i.hue = 0x493;
            },
            9 => {
                i.hue = 0x4A8;
            },
            10 => {
                i.hue = 0x494;
            },
            11 => {
                i.hue = 0x4AA;
            },
            12 => {
                i.hue = 0xB8B;
            },
            13 => {
                i.hue = 0x84F;
            },
            14 => {
                i.hue = 0x491;
            },
            15 => {
                i.hue = 0x851;
            },
            16 => {
                i.hue = 0x503;
            },
            17 => {
                i.hue = 0xB8C;
            },
            18 => {
                i.hue = 0x4AB;
            },
            19 => {
                i.hue = 0x84B;
            }
        }
    });
});

item("GiftBoxNeon", |c| {
    c.gump_id = 282;
    c.on_create(|i| {
        if coin() {
            i.item_id = 0x232A;
        } else {
            i.item_id = 0x232B;
        }
        switch random_below(6) {
            0 => {
                i.hue = 0x438;
            },
            1 => {
                i.hue = 0x424;
            },
            2 => {
                i.hue = 0x433;
            },
            3 => {
                i.hue = 0x445;
            },
            4 => {
                i.hue = 0x42B;
            },
            5 => {
                i.hue = 0x448;
            }
        }
    });
});

item("SnowStatuePegasus", |c| {
    c.on_create(|i| {
        i.item_id = 0x456E;
    });
});

item("SnowStatueSeahorse", |c| {
    c.on_create(|i| {
        i.item_id = 0x4578;
    });
});

item("SnowStatueMermaid", |c| {
    c.on_create(|i| {
        i.item_id = 0x457A;
    });
});

item("SnowStatueGriffon", |c| {
    c.on_create(|i| {
        i.item_id = 0x457C;
    });
});
