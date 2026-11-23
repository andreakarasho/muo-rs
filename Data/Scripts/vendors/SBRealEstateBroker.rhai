shop("SBRealEstateBroker", |s| {
    s.buys("ScribesPen", 4);
    s.buys("BlankScroll", 2);
    s.sells("BlankScroll", 5, 20, 0x0E34, #{ hue: 0 });
    s.sells("ScribesPen", 8, 20, 0x0FBF, #{ hue: 0 });
});
