shop("SBSmithTools", |s| {
    s.buys("Tongs", 7);
    s.buys("IronIngot", 4);
    s.sells("IronIngot", 5, 16, 0x1BF2, #{ hue: 0 });
    s.sells("Tongs", 13, 14, 0x0FBB, #{ hue: 0 });
});
