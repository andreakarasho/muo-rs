shop("SBSwordWeapon", |s| {
    s.buys("Broadsword", 17);
    s.buys("Cutlass", 12);
    s.buys("Katana", 16);
    s.buys("Kryss", 16);
    s.buys("Longsword", 27);
    s.buys("Scimitar", 18);
    s.buys("ThinLongsword", 13);
    s.buys("VikingSword", 27);
    s.era("AOS", |s| {
        s.buys("Scythe", 19);
        s.buys("BoneHarvester", 17);
        s.buys("Scepter", 18);
        s.buys("BladedStaff", 16);
        s.buys("Pike", 19);
        s.buys("DoubleBladedStaff", 17);
        s.buys("Lance", 17);
        s.buys("CrescentBlade", 18);
    });
    s.sells("Cutlass", 24, 20, 0x1441, #{ hue: 0 });
    s.sells("Katana", 33, 20, 0x13FF, #{ hue: 0 });
    s.sells("Kryss", 32, 20, 0x1401, #{ hue: 0 });
    s.sells("Broadsword", 35, 20, 0x0F5E, #{ hue: 0 });
    s.sells("Longsword", 55, 20, 0x0F61, #{ hue: 0 });
    s.sells("ThinLongsword", 27, 20, 0x13B8, #{ hue: 0 });
    s.sells("VikingSword", 55, 20, 0x13B9, #{ hue: 0 });
    s.sells("Scimitar", 36, 20, 0x13B6, #{ hue: 0 });
    s.era("AOS", |s| {
        s.sells("BoneHarvester", 35, 20, 0x26BB, #{ hue: 0 });
        s.sells("CrescentBlade", 37, 20, 0x26C1, #{ hue: 0 });
        s.sells("DoubleBladedStaff", 35, 20, 0x26BF, #{ hue: 0 });
        s.sells("Lance", 34, 20, 0x26C0, #{ hue: 0 });
        s.sells("Pike", 39, 20, 0x26BE, #{ hue: 0 });
        s.sells("Scythe", 39, 20, 0x26BA, #{ hue: 0 });
    });
});
