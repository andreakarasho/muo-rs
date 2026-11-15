shop("SBMiller", |s| {
    s.buys("SackFlour", 1);
    s.buys("SheafOfHay", 1);
    s.sells("SackFlour", 3, 20, 0x1039, #{ hue: 0 });
    s.sells("SheafOfHay", 2, 20, 0x0F36, #{ hue: 0 });
});
