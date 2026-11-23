item("MapItem", |c| {
    c.weight = 1.0;
    c.on_create(|i| {
        i.item_id = 0x14EC;
    });
});

item("BlankMap", |c| {});

item("CityMap", |c| {
    c.label = 1015231;
    c.on_create(|i| {
        i.map_display = [0, 0, 5119, 4095, 400, 400];
    });
});

item("LocalMap", |c| {
    c.label = 1015230;
    c.on_create(|i| {
        i.map_display = [0, 0, 5119, 4095, 400, 400];
    });
});

item("SeaChart", |c| {
    c.label = 1015232;
    c.on_create(|i| {
        i.map_display = [0, 0, 5119, 4095, 400, 400];
    });
});

item("WorldMap", |c| {
    c.label = 1015233;
    c.on_create(|i| {
        i.map_display = [0, 0, 5119, 4095, 400, 400];
    });
});

item("IndecipherableMap", |c| {
    c.label = 1070799;
    c.on_create(|i| {
        if chance(0.2) {
            i.hue = 0x965;
        } else {
            i.hue = 0x961;
        }
    });
});

item("WeatherMap", |c| {
    c.name = "weather map";
    c.on_create(|i| {
        i.map_display = [0, 0, 5119, 4095, 400, 400];
    });
});
