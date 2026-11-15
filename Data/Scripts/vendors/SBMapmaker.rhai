shop("SBMapmaker", |s| {
    s.buys("BlankScroll", 3);
    s.buys("MapmakersPen", 4);
    s.buys("BlankMap", 2);
    s.buys("CityMap", 3);
    s.buys("LocalMap", 3);
    s.buys("WorldMap", 3);
    s.buys("Server.Items.PresetMapEntry", 3);
    s.sells("BlankMap", 5, 40, 0x14EC, #{ hue: 0 });
    s.sells("MapmakersPen", 8, 20, 0x0FBF, #{ hue: 0 });
    s.sells("BlankScroll", 12, 40, 0x0EF3, #{ hue: 0 });
    s.preset_maps();
});
