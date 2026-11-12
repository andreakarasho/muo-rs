fish_leaves! {
    /// `Server.Items.AlbinoCourtesanFish`.
    ALBINO_COURTESAN_FISH = "Server.Items.AlbinoCourtesanFish", 1074592, |_: &mut World| 0x3B04, 0;
    /// `Server.Items.AlbinoFrog`.
    ALBINO_FROG = "Server.Items.AlbinoFrog", 1073824, |_: &mut World| 0x3B0D, 0x47E;
    /// `Server.Items.BritainCrownFish`.
    BRITAIN_CROWN_FISH = "Server.Items.BritainCrownFish", 1074589, |_: &mut World| 0x3AFF, 0;
    /// `Server.Items.FandancerFish`.
    FANDANCER_FISH = "Server.Items.FandancerFish", 1074591, |_: &mut World| 0x3B02, 0;
    /// `Server.Items.GoldenBroadtail`.
    GOLDEN_BROADTAIL = "Server.Items.GoldenBroadtail", 1073828, |_: &mut World| 0x3B03, 0;
    /// `Server.Items.Jellyfish`.
    JELLYFISH = "Server.Items.Jellyfish", 1074593, |_: &mut World| 0x3B0E, 0;
    /// `Server.Items.KillerFrog`.
    KILLER_FROG = "Server.Items.KillerFrog", 1073825, |_: &mut World| 0x3B0D, 0;
    /// `Server.Items.LongClawCrab`.
    LONG_CLAW_CRAB = "Server.Items.LongClawCrab", 1073827, |_: &mut World| 0x3AFC, 0x527;
    /// `Server.Items.MakotoCourtesanFish`.
    MAKOTO_COURTESAN_FISH = "Server.Items.MakotoCourtesanFish", 1073835, |_: &mut World| 0x3AFD, 0;
    /// `Server.Items.MinocBlueFish`.
    MINOC_BLUE_FISH = "Server.Items.MinocBlueFish", 1073829, |_: &mut World| 0x3AFE, 0;
    /// `Server.Items.NujelmHoneyFish`.
    NUJELM_HONEY_FISH = "Server.Items.NujelmHoneyFish", 1073830, |_: &mut World| 0x3B06, 0;
    /// `Server.Items.PurpleFrog`.
    PURPLE_FROG = "Server.Items.PurpleFrog", 1073823, |_: &mut World| 0x3B0D, 0x4FA;
    /// `Server.Items.RedDartFish`.
    RED_DART_FISH = "Server.Items.RedDartFish", 1073834, |_: &mut World| 0x3B00, 0;
    /// `Server.Items.Shrimp`.
    SHRIMP = "Server.Items.Shrimp", 1074596, |_: &mut World| 0x3B14, 0;
    /// `Server.Items.SmallMouthSuckerFin`.
    SMALL_MOUTH_SUCKER_FIN = "Server.Items.SmallMouthSuckerFin", 1074590, |_: &mut World| 0x3B01, 0;
    /// `Server.Items.SpeckledCrab`.
    SPECKLED_CRAB = "Server.Items.SpeckledCrab", 1073826, |_: &mut World| 0x3AFC, 0;
    /// `Server.Items.SpinedScratcherFish`.
    SPINED_SCRATCHER_FISH = "Server.Items.SpinedScratcherFish", 1073832, |_: &mut World| 0x3B05, 0;
    /// `Server.Items.SpottedBuccaneer`.
    SPOTTED_BUCCANEER = "Server.Items.SpottedBuccaneer", 1073833, |_: &mut World| 0x3B09, 0;
    /// `Server.Items.VesperReefTiger`.
    VESPER_REEF_TIGER = "Server.Items.VesperReefTiger", 1073836, |_: &mut World| 0x3B08, 0;
    /// `Server.Items.YellowFinBluebelly`.
    YELLOW_FIN_BLUEBELLY = "Server.Items.YellowFinBluebelly", 1073831, |_: &mut World| 0x3B07, 0;
    /// `Server.Items.BrineShrimp`.
    BRINE_SHRIMP = "Server.Items.BrineShrimp", 1074415, |_: &mut World| 0x3B11, 0;
    /// `Server.Items.Coral`.
    CORAL = "Server.Items.Coral", 1074588, |w: &mut World| { let ids = [0x3AF9,0x3AFA,0x3AFB]; *utility::random_element(&mut w.rng, &ids).unwrap_or(&ids[0]) }, 0;
    /// `Server.Items.FullMoonFish`.
    FULL_MOON_FISH = "Server.Items.FullMoonFish", 1074597, |_: &mut World| 0x3B15, 0;
    /// `Server.Items.SeaHorseFish`.
    SEA_HORSE_FISH = "Server.Items.SeaHorseFish", 1074414, |_: &mut World| 0x3B10, 0;
    /// `Server.Items.StrippedFlakeFish`.
    STRIPPED_FLAKE_FISH = "Server.Items.StrippedFlakeFish", 1074595, |_: &mut World| 0x3B0A, 0;
    /// `Server.Items.StrippedSosarianSwill`.
    STRIPPED_SOSARIAN_SWILL = "Server.Items.StrippedSosarianSwill", 1074594, |_: &mut World| 0x3B0A, 0;
}

/// Every fish.
pub static FISH: &[&ItemType] = &[&ALBINO_COURTESAN_FISH, &ALBINO_FROG, &BRITAIN_CROWN_FISH, &FANDANCER_FISH, &GOLDEN_BROADTAIL, &JELLYFISH, &KILLER_FROG, &LONG_CLAW_CRAB, &MAKOTO_COURTESAN_FISH, &MINOC_BLUE_FISH, &NUJELM_HONEY_FISH, &PURPLE_FROG, &RED_DART_FISH, &SHRIMP, &SMALL_MOUTH_SUCKER_FIN, &SPECKLED_CRAB, &SPINED_SCRATCHER_FISH, &SPOTTED_BUCCANEER, &VESPER_REEF_TIGER, &YELLOW_FIN_BLUEBELLY, &BRINE_SHRIMP, &CORAL, &FULL_MOON_FISH, &SEA_HORSE_FISH, &STRIPPED_FLAKE_FISH, &STRIPPED_SOSARIAN_SWILL];
