shop("SBScribe", |s| {
    s.buys("ScribesPen", 4);
    s.buys("BrownBook", 7);
    s.buys("TanBook", 7);
    s.buys("BlueBook", 7);
    s.buys("BlankScroll", 3);
    s.sells("ScribesPen", 8, 20, 0x0FBF, #{ hue: 0 });
    s.sells("BlankScroll", 5, 999, 0x0E34, #{ hue: 0 });
    s.sells("ScribesPen", 8, 20, 0x0FC0, #{ hue: 0 });
    s.sells("BrownBook", 15, 10, 0x0FEF, #{ hue: 0 });
    s.sells("TanBook", 15, 10, 0x0FF0, #{ hue: 0 });
    s.sells("BlueBook", 15, 10, 0x0FF2, #{ hue: 0 });
});
