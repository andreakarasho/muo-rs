shop("SBAnimalTrainer", |s| {
    s.sells("Cat", 132, 10, 0x00C9, #{ name: "a cat", control_slots: 1, hue: 0 });
    s.sells("Dog", 170, 10, 0x00D9, #{ name: "a dog", control_slots: 1, hue: 0 });
    s.sells("Horse", 550, 10, 0x00CC, #{ name: "a horse", control_slots: 1, hue: 0 });
    s.sells("PackHorse", 631, 10, 0x0123, #{ name: "a pack horse", control_slots: 1, hue: 0 });
    s.sells("PackLlama", 565, 10, 0x0124, #{ name: "a pack llama", control_slots: 1, hue: 0 });
    s.sells("Rabbit", 106, 10, 0x00CD, #{ name: "a rabbit", control_slots: 1, hue: 0 });
    s.era("AOS", |s| {}, |s| {
        s.sells("Eagle", 402, 10, 0x0005, #{ name: "an eagle", control_slots: 1, hue: 0 });
        s.sells("BrownBear", 855, 10, 0x00D3, #{ name: "a brown bear", control_slots: 1, hue: 0 });
        s.sells("GrizzlyBear", 1767, 10, 0x00D4, #{ name: "a grizzly bear", control_slots: 1, hue: 0 });
        s.sells("Panther", 1271, 10, 0x00D6, #{ name: "a panther", control_slots: 1, hue: 0 });
        s.sells("TimberWolf", 768, 10, 0x00E1, #{ name: "a timber wolf", control_slots: 1, hue: 0 });
        s.sells("Rat", 107, 10, 0x00EE, #{ name: "a rat", control_slots: 1, hue: 0 });
    });
});
