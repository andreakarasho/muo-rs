shop("SBMetalShields", |s| {
    s.buys("Server.Items.Buckler", 25);
    s.buys("BronzeShield", 33);
    s.buys("MetalShield", 60);
    s.buys("MetalKiteShield", 62);
    s.buys("HeaterShield", 115);
    s.buys("WoodenKiteShield", 35);
    s.sells("BronzeShield", 66, 20, 0x1B72, #{ hue: 0 });
    s.sells("Server.Items.Buckler", 50, 20, 0x1B73, #{ hue: 0 });
    s.sells("MetalKiteShield", 123, 20, 0x1B74, #{ hue: 0 });
    s.sells("HeaterShield", 231, 20, 0x1B76, #{ hue: 0 });
    s.sells("WoodenKiteShield", 70, 20, 0x1B78, #{ hue: 0 });
    s.sells("MetalShield", 121, 20, 0x1B7B, #{ hue: 0 });
});
