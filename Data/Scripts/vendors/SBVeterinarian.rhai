shop("SBVeterinarian", |s| {
    s.buys("Bandage", 1);
    s.sells("Bandage", 6, 20, 0x0E21, #{ hue: 0 });
    s.sells("PackHorse", 616, 10, 0x0123, #{ name: "a pack horse", control_slots: 1, hue: 0 });
    s.sells("PackLlama", 523, 10, 0x0124, #{ name: "a pack llama", control_slots: 1, hue: 0 });
    s.sells("Dog", 158, 10, 0x00D9, #{ name: "a dog", control_slots: 1, hue: 0 });
    s.sells("Cat", 131, 10, 0x00C9, #{ name: "a cat", control_slots: 1, hue: 0 });
});
