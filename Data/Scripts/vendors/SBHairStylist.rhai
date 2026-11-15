shop("SBHairStylist", |s| {
    s.buys("HairDye", 30);
    s.buys("SpecialBeardDye", 250000);
    s.buys("SpecialHairDye", 250000);
    s.sells("SpecialBeardDye", 500000, 20, 0x0E26, #{ name: "special beard dye", hue: 0 });
    s.sells("SpecialHairDye", 500000, 20, 0x0E26, #{ name: "special hair dye", hue: 0 });
    s.sells("HairDye", 60, 20, 0x0EFF, #{ name: "1041060", hue: 0 });
});
