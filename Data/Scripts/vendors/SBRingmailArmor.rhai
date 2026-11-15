shop("SBRingmailArmor", |s| {
    s.buys("RingmailArms", 42);
    s.buys("RingmailChest", 60);
    s.buys("RingmailGloves", 26);
    s.buys("RingmailLegs", 45);
    s.sells("RingmailChest", 121, 20, 0x13EC, #{ hue: 0 });
    s.sells("RingmailLegs", 90, 20, 0x13F0, #{ hue: 0 });
    s.sells("RingmailArms", 85, 20, 0x13EE, #{ hue: 0 });
    s.sells("RingmailGloves", 93, 20, 0x13EB, #{ hue: 0 });
});
