shop("SBArchitect", |s| {
    s.buys("InteriorDecorator", 5000);
    s.era("AOS", |s| {
        s.buys("HousePlacementTool", 301);
    });
    s.sells("InteriorDecorator", 10001, 20, 0x0FC1, #{ name: "1041280", hue: 0 });
    s.era("AOS", |s| {
        s.sells("HousePlacementTool", 627, 20, 0x14F6, #{ name: "1060651", hue: 0 });
    });
});
