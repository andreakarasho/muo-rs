item("VacationWafer", |c| {
    c.label = 1074431;
    c.on_create(|i| {
        i.item_id = 0x973;
    });
});

item("AlbinoCourtesanFish", |c| {
    c.label = 1074592;
    c.on_create(|i| {
        i.item_id = 0x3B04;
    });
});

item("AlbinoFrog", |c| {
    c.label = 1073824;
    c.on_create(|i| {
        i.item_id = 0x3B0D;
        i.hue = 0x47E;
    });
});

item("BritainCrownFish", |c| {
    c.label = 1074589;
    c.on_create(|i| {
        i.item_id = 0x3AFF;
    });
});

item("FandancerFish", |c| {
    c.label = 1074591;
    c.on_create(|i| {
        i.item_id = 0x3B02;
    });
});

item("GoldenBroadtail", |c| {
    c.label = 1073828;
    c.on_create(|i| {
        i.item_id = 0x3B03;
    });
});

item("Jellyfish", |c| {
    c.label = 1074593;
    c.on_create(|i| {
        i.item_id = 0x3B0E;
    });
});

item("KillerFrog", |c| {
    c.label = 1073825;
    c.on_create(|i| {
        i.item_id = 0x3B0D;
    });
});

item("LongClawCrab", |c| {
    c.label = 1073827;
    c.on_create(|i| {
        i.item_id = 0x3AFC;
        i.hue = 0x527;
    });
});

item("MakotoCourtesanFish", |c| {
    c.label = 1073835;
    c.on_create(|i| {
        i.item_id = 0x3AFD;
    });
});

item("MinocBlueFish", |c| {
    c.label = 1073829;
    c.on_create(|i| {
        i.item_id = 0x3AFE;
    });
});

item("NujelmHoneyFish", |c| {
    c.label = 1073830;
    c.on_create(|i| {
        i.item_id = 0x3B06;
    });
});

item("PurpleFrog", |c| {
    c.label = 1073823;
    c.on_create(|i| {
        i.item_id = 0x3B0D;
        i.hue = 0x4FA;
    });
});

item("RedDartFish", |c| {
    c.label = 1073834;
    c.on_create(|i| {
        i.item_id = 0x3B00;
    });
});

item("Shrimp", |c| {
    c.label = 1074596;
    c.on_create(|i| {
        i.item_id = 0x3B14;
    });
});

item("SmallMouthSuckerFin", |c| {
    c.label = 1074590;
    c.on_create(|i| {
        i.item_id = 0x3B01;
    });
});

item("SpeckledCrab", |c| {
    c.label = 1073826;
    c.on_create(|i| {
        i.item_id = 0x3AFC;
    });
});

item("SpinedScratcherFish", |c| {
    c.label = 1073832;
    c.on_create(|i| {
        i.item_id = 0x3B05;
    });
});

item("SpottedBuccaneer", |c| {
    c.label = 1073833;
    c.on_create(|i| {
        i.item_id = 0x3B09;
    });
});

item("VesperReefTiger", |c| {
    c.label = 1073836;
    c.on_create(|i| {
        i.item_id = 0x3B08;
    });
});

item("YellowFinBluebelly", |c| {
    c.label = 1073831;
    c.on_create(|i| {
        i.item_id = 0x3B07;
    });
});

item("BrineShrimp", |c| {
    c.label = 1074415;
    c.on_create(|i| {
        i.item_id = 0x3B11;
    });
});

item("FullMoonFish", |c| {
    c.label = 1074597;
    c.on_create(|i| {
        i.item_id = 0x3B15;
    });
});

item("SeaHorseFish", |c| {
    c.label = 1074414;
    c.on_create(|i| {
        i.item_id = 0x3B10;
    });
});

item("StrippedFlakeFish", |c| {
    c.label = 1074595;
    c.on_create(|i| {
        i.item_id = 0x3B0A;
    });
});

item("StrippedSosarianSwill", |c| {
    c.label = 1074594;
    c.on_create(|i| {
        i.item_id = 0x3B0A;
    });
});

item("AquariumFood", |c| {
    c.label = 1074819;
    c.weight = 1.0;
    c.on_create(|i| {
        i.item_id = 0xEFC;
    });
});

item("FishBones", |c| {
    c.label = 1074601;
    c.weight = 1.0;
    c.on_create(|i| {
        i.item_id = 0x3B0C;
    });
});

item("IslandStatue", |c| {
    c.label = 1074600;
    c.weight = 1.0;
    c.on_create(|i| {
        i.item_id = 0x3B0F;
    });
});

item("Shell", |c| {
    c.label = 1074598;
    c.weight = 1.0;
    c.on_create(|i| {
        switch random_below(2) {
            0 => {
                i.item_id = 0x3B12;
            },
            1 => {
                i.item_id = 0x3B13;
            }
        }
    });
});

item("ToyBoat", |c| {
    c.label = 1074363;
    c.weight = 1.0;
    c.on_create(|i| {
        i.item_id = 0x14F4;
    });
});

item("Coral", |c| {
    c.label = 1074588;
    c.on_create(|i| {
        switch random_below(3) {
            0 => {
                i.item_id = 0x3AF9;
            },
            1 => {
                i.item_id = 0x3AFA;
            },
            2 => {
                i.item_id = 0x3AFB;
            }
        }
    });
});
