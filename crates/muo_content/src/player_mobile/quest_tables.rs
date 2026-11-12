//! Each quest system's `QuestTypes` slot, `TypeReferenceTable` and the layout
//! its `ChildDeserialize` (and its objectives' and conversations') read. See `quest_record.rs`.

use super::quest_record::Tok;

/// A quest system: its full name, its `TypeReferenceTable` and its `ChildDeserialize` layout.
pub struct SystemDef {
    pub name: &'static str,
    pub table: &'static [&'static str],
    pub child: &'static [Tok],
}

/// `QuestSystem.QuestTypes`, in order.
pub static SYSTEMS: &[SystemDef] = &[
    SystemDef {
        name: "Server.Engines.Quests.Doom.TheSummoningQuest",
        table: &[
            "Server.Engines.Quests.Doom.AcceptConversation",
            "Server.Engines.Quests.Doom.CollectBonesObjective",
            "Server.Engines.Quests.Doom.VanquishDaemonConversation",
            "Server.Engines.Quests.Doom.VanquishDaemonObjective",
        ],
        child: &[Tok::E, Tok::Ser, Tok::B],
    },
    SystemDef {
        name: "Server.Engines.Quests.Necro.DarkTidesQuest",
        table: &[
            "Server.Engines.Quests.Necro.AcceptConversation",
            "Server.Engines.Quests.Necro.AnimateMaabusCorpseObjective",
            "Server.Engines.Quests.Necro.BankerConversation",
            "Server.Engines.Quests.Necro.CashBankCheckObjective",
            "Server.Engines.Quests.Necro.FetchAbraxusScrollObjective",
            "Server.Engines.Quests.Necro.FindBankObjective",
            "Server.Engines.Quests.Necro.FindCallingScrollObjective",
            "Server.Engines.Quests.Necro.FindCityOfLightObjective",
            "Server.Engines.Quests.Necro.FindCrystalCaveObjective",
            "Server.Engines.Quests.Necro.FindMaabusCorpseObjective",
            "Server.Engines.Quests.Necro.FindMaabusTombObjective",
            "Server.Engines.Quests.Necro.FindMardothAboutKronusObjective",
            "Server.Engines.Quests.Necro.FindMardothAboutVaultObjective",
            "Server.Engines.Quests.Necro.FindMardothEndObjective",
            "Server.Engines.Quests.Necro.FindVaultOfSecretsObjective",
            "Server.Engines.Quests.Necro.FindWellOfTearsObjective",
            "Server.Engines.Quests.Necro.HorusConversation",
            "Server.Engines.Quests.Necro.LostCallingScrollConversation",
            "Server.Engines.Quests.Necro.MaabasConversation",
            "Server.Engines.Quests.Necro.MardothEndConversation",
            "Server.Engines.Quests.Necro.MardothKronusConversation",
            "Server.Engines.Quests.Necro.MardothVaultConversation",
            "Server.Engines.Quests.Necro.RadarConversation",
            "Server.Engines.Quests.Necro.ReadAbraxusScrollConversation",
            "Server.Engines.Quests.Necro.ReadAbraxusScrollObjective",
            "Server.Engines.Quests.Necro.ReanimateMaabusConversation",
            "Server.Engines.Quests.Necro.RetrieveAbraxusScrollObjective",
            "Server.Engines.Quests.Necro.ReturnToCrystalCaveObjective",
            "Server.Engines.Quests.Necro.SecondHorusConversation",
            "Server.Engines.Quests.Necro.SpeakCavePasswordObjective",
            "Server.Engines.Quests.Necro.UseCallingScrollObjective",
            "Server.Engines.Quests.Necro.VaultOfSecretsConversation",
            "Server.Engines.Quests.Necro.FindHorusAboutRewardObjective",
            "Server.Engines.Quests.Necro.HealConversation",
            "Server.Engines.Quests.Necro.HorusRewardConversation",
        ],
        child: &[Tok::E],
    },
    SystemDef {
        name: "Server.Engines.Quests.Haven.UzeraanTurmoilQuest",
        table: &[
            "Server.Engines.Quests.Haven.AcceptConversation",
            "Server.Engines.Quests.Haven.UzeraanTitheConversation",
            "Server.Engines.Quests.Haven.UzeraanFirstTaskConversation",
            "Server.Engines.Quests.Haven.UzeraanReportConversation",
            "Server.Engines.Quests.Haven.SchmendrickConversation",
            "Server.Engines.Quests.Haven.UzeraanScrollOfPowerConversation",
            "Server.Engines.Quests.Haven.DryadConversation",
            "Server.Engines.Quests.Haven.UzeraanFertileDirtConversation",
            "Server.Engines.Quests.Haven.UzeraanDaemonBloodConversation",
            "Server.Engines.Quests.Haven.UzeraanDaemonBoneConversation",
            "Server.Engines.Quests.Haven.BankerConversation",
            "Server.Engines.Quests.Haven.RadarConversation",
            "Server.Engines.Quests.Haven.LostScrollOfPowerConversation",
            "Server.Engines.Quests.Haven.LostFertileDirtConversation",
            "Server.Engines.Quests.Haven.DryadAppleConversation",
            "Server.Engines.Quests.Haven.LostDaemonBloodConversation",
            "Server.Engines.Quests.Haven.LostDaemonBoneConversation",
            "Server.Engines.Quests.Haven.FindUzeraanBeginObjective",
            "Server.Engines.Quests.Haven.TitheGoldObjective",
            "Server.Engines.Quests.Haven.FindUzeraanFirstTaskObjective",
            "Server.Engines.Quests.Haven.KillHordeMinionsObjective",
            "Server.Engines.Quests.Haven.FindUzeraanAboutReportObjective",
            "Server.Engines.Quests.Haven.FindSchmendrickObjective",
            "Server.Engines.Quests.Haven.FindApprenticeObjective",
            "Server.Engines.Quests.Haven.ReturnScrollOfPowerObjective",
            "Server.Engines.Quests.Haven.FindDryadObjective",
            "Server.Engines.Quests.Haven.ReturnFertileDirtObjective",
            "Server.Engines.Quests.Haven.GetDaemonBloodObjective",
            "Server.Engines.Quests.Haven.ReturnDaemonBloodObjective",
            "Server.Engines.Quests.Haven.GetDaemonBoneObjective",
            "Server.Engines.Quests.Haven.ReturnDaemonBoneObjective",
            "Server.Engines.Quests.Haven.CashBankCheckObjective",
            "Server.Engines.Quests.Haven.FewReagentsConversation",
        ],
        child: &[Tok::E, Tok::B],
    },
    SystemDef {
        name: "Server.Engines.Quests.Collector.CollectorQuest",
        table: &[
            "Server.Engines.Quests.Collector.DontOfferConversation",
            "Server.Engines.Quests.Collector.DeclineConversation",
            "Server.Engines.Quests.Collector.AcceptConversation",
            "Server.Engines.Quests.Collector.ElwoodDuringFishConversation",
            "Server.Engines.Quests.Collector.ReturnPearlsConversation",
            "Server.Engines.Quests.Collector.AlbertaPaintingConversation",
            "Server.Engines.Quests.Collector.AlbertaStoolConversation",
            "Server.Engines.Quests.Collector.AlbertaEndPaintingConversation",
            "Server.Engines.Quests.Collector.AlbertaAfterPaintingConversation",
            "Server.Engines.Quests.Collector.ElwoodDuringPainting1Conversation",
            "Server.Engines.Quests.Collector.ElwoodDuringPainting2Conversation",
            "Server.Engines.Quests.Collector.ReturnPaintingConversation",
            "Server.Engines.Quests.Collector.GabrielAutographConversation",
            "Server.Engines.Quests.Collector.GabrielNoSheetMusicConversation",
            "Server.Engines.Quests.Collector.NoSheetMusicConversation",
            "Server.Engines.Quests.Collector.GetSheetMusicConversation",
            "Server.Engines.Quests.Collector.GabrielSheetMusicConversation",
            "Server.Engines.Quests.Collector.GabrielIgnoreConversation",
            "Server.Engines.Quests.Collector.ElwoodDuringAutograph1Conversation",
            "Server.Engines.Quests.Collector.ElwoodDuringAutograph2Conversation",
            "Server.Engines.Quests.Collector.ElwoodDuringAutograph3Conversation",
            "Server.Engines.Quests.Collector.ReturnAutographConversation",
            "Server.Engines.Quests.Collector.TomasToysConversation",
            "Server.Engines.Quests.Collector.TomasDuringCollectingConversation",
            "Server.Engines.Quests.Collector.ReturnImagesConversation",
            "Server.Engines.Quests.Collector.ElwoodDuringToys1Conversation",
            "Server.Engines.Quests.Collector.ElwoodDuringToys2Conversation",
            "Server.Engines.Quests.Collector.ElwoodDuringToys3Conversation",
            "Server.Engines.Quests.Collector.FullEndConversation",
            "Server.Engines.Quests.Collector.FishPearlsObjective",
            "Server.Engines.Quests.Collector.ReturnPearlsObjective",
            "Server.Engines.Quests.Collector.FindAlbertaObjective",
            "Server.Engines.Quests.Collector.SitOnTheStoolObjective",
            "Server.Engines.Quests.Collector.ReturnPaintingObjective",
            "Server.Engines.Quests.Collector.FindGabrielObjective",
            "Server.Engines.Quests.Collector.FindSheetMusicObjective",
            "Server.Engines.Quests.Collector.ReturnSheetMusicObjective",
            "Server.Engines.Quests.Collector.ReturnAutographObjective",
            "Server.Engines.Quests.Collector.FindTomasObjective",
            "Server.Engines.Quests.Collector.CaptureImagesObjective",
            "Server.Engines.Quests.Collector.ReturnImagesObjective",
            "Server.Engines.Quests.Collector.ReturnToysObjective",
            "Server.Engines.Quests.Collector.MakeRoomObjective",
        ],
        child: &[Tok::E],
    },
    SystemDef {
        name: "Server.Engines.Quests.Hag.WitchApprenticeQuest",
        table: &[
            "Server.Engines.Quests.Hag.FindApprenticeObjective",
            "Server.Engines.Quests.Hag.FindGrizeldaAboutMurderObjective",
            "Server.Engines.Quests.Hag.KillImpsObjective",
            "Server.Engines.Quests.Hag.FindZeefzorpulObjective",
            "Server.Engines.Quests.Hag.ReturnRecipeObjective",
            "Server.Engines.Quests.Hag.FindIngredientObjective",
            "Server.Engines.Quests.Hag.ReturnIngredientsObjective",
            "Server.Engines.Quests.Hag.DontOfferConversation",
            "Server.Engines.Quests.Hag.AcceptConversation",
            "Server.Engines.Quests.Hag.HagDuringCorpseSearchConversation",
            "Server.Engines.Quests.Hag.ApprenticeCorpseConversation",
            "Server.Engines.Quests.Hag.MurderConversation",
            "Server.Engines.Quests.Hag.HagDuringImpSearchConversation",
            "Server.Engines.Quests.Hag.ImpDeathConversation",
            "Server.Engines.Quests.Hag.ZeefzorpulConversation",
            "Server.Engines.Quests.Hag.RecipeConversation",
            "Server.Engines.Quests.Hag.HagDuringIngredientsConversation",
            "Server.Engines.Quests.Hag.BlackheartFirstConversation",
            "Server.Engines.Quests.Hag.BlackheartNoPirateConversation",
            "Server.Engines.Quests.Hag.BlackheartPirateConversation",
            "Server.Engines.Quests.Hag.EndConversation",
            "Server.Engines.Quests.Hag.RecentlyFinishedConversation",
        ],
        child: &[Tok::E],
    },
    SystemDef {
        name: "Server.Engines.Quests.Naturalist.StudyOfSolenQuest",
        table: &[
            "Server.Engines.Quests.Naturalist.StudyNestsObjective",
            "Server.Engines.Quests.Naturalist.ReturnToNaturalistObjective",
            "Server.Engines.Quests.Naturalist.DontOfferConversation",
            "Server.Engines.Quests.Naturalist.AcceptConversation",
            "Server.Engines.Quests.Naturalist.NaturalistDuringStudyConversation",
            "Server.Engines.Quests.Naturalist.EndConversation",
            "Server.Engines.Quests.Naturalist.SpecialEndConversation",
            "Server.Engines.Quests.Naturalist.FullBackpackConversation",
        ],
        child: &[Tok::E, Tok::Ser],
    },
    SystemDef {
        name: "Server.Engines.Quests.Matriarch.SolenMatriarchQuest",
        table: &[
            "Server.Engines.Quests.Matriarch.DontOfferConversation",
            "Server.Engines.Quests.Matriarch.AcceptConversation",
            "Server.Engines.Quests.Matriarch.DuringKillInfiltratorsConversation",
            "Server.Engines.Quests.Matriarch.GatherWaterConversation",
            "Server.Engines.Quests.Matriarch.DuringWaterGatheringConversation",
            "Server.Engines.Quests.Matriarch.ProcessFungiConversation",
            "Server.Engines.Quests.Matriarch.DuringFungiProcessConversation",
            "Server.Engines.Quests.Matriarch.FullBackpackConversation",
            "Server.Engines.Quests.Matriarch.EndConversation",
            "Server.Engines.Quests.Matriarch.KillInfiltratorsObjective",
            "Server.Engines.Quests.Matriarch.ReturnAfterKillsObjective",
            "Server.Engines.Quests.Matriarch.GatherWaterObjective",
            "Server.Engines.Quests.Matriarch.ReturnAfterWaterObjective",
            "Server.Engines.Quests.Matriarch.ProcessFungiObjective",
            "Server.Engines.Quests.Matriarch.GetRewardObjective",
        ],
        child: &[Tok::E, Tok::B],
    },
    SystemDef {
        name: "Server.Engines.Quests.Ambitious.AmbitiousQueenQuest",
        table: &[
            "Server.Engines.Quests.Ambitious.DontOfferConversation",
            "Server.Engines.Quests.Ambitious.AcceptConversation",
            "Server.Engines.Quests.Ambitious.DuringKillQueensConversation",
            "Server.Engines.Quests.Ambitious.GatherFungiConversation",
            "Server.Engines.Quests.Ambitious.DuringFungiGatheringConversation",
            "Server.Engines.Quests.Ambitious.EndConversation",
            "Server.Engines.Quests.Ambitious.FullBackpackConversation",
            "Server.Engines.Quests.Ambitious.End2Conversation",
            "Server.Engines.Quests.Ambitious.KillQueensObjective",
            "Server.Engines.Quests.Ambitious.ReturnAfterKillsObjective",
            "Server.Engines.Quests.Ambitious.GatherFungiObjective",
            "Server.Engines.Quests.Ambitious.GetRewardObjective",
        ],
        child: &[Tok::E, Tok::B],
    },
    SystemDef {
        name: "Server.Engines.Quests.Ninja.EminosUndertakingQuest",
        table: &[
            "Server.Engines.Quests.Ninja.AcceptConversation",
            "Server.Engines.Quests.Ninja.FindZoelConversation",
            "Server.Engines.Quests.Ninja.RadarConversation",
            "Server.Engines.Quests.Ninja.EnterCaveConversation",
            "Server.Engines.Quests.Ninja.SneakPastGuardiansConversation",
            "Server.Engines.Quests.Ninja.NeedToHideConversation",
            "Server.Engines.Quests.Ninja.UseTeleporterConversation",
            "Server.Engines.Quests.Ninja.GiveZoelNoteConversation",
            "Server.Engines.Quests.Ninja.LostNoteConversation",
            "Server.Engines.Quests.Ninja.GainInnInformationConversation",
            "Server.Engines.Quests.Ninja.ReturnFromInnConversation",
            "Server.Engines.Quests.Ninja.SearchForSwordConversation",
            "Server.Engines.Quests.Ninja.HallwayWalkConversation",
            "Server.Engines.Quests.Ninja.ReturnSwordConversation",
            "Server.Engines.Quests.Ninja.SlayHenchmenConversation",
            "Server.Engines.Quests.Ninja.ContinueSlayHenchmenConversation",
            "Server.Engines.Quests.Ninja.GiveEminoSwordConversation",
            "Server.Engines.Quests.Ninja.LostSwordConversation",
            "Server.Engines.Quests.Ninja.EarnGiftsConversation",
            "Server.Engines.Quests.Ninja.EarnLessGiftsConversation",
            "Server.Engines.Quests.Ninja.FindEminoBeginObjective",
            "Server.Engines.Quests.Ninja.FindZoelObjective",
            "Server.Engines.Quests.Ninja.EnterCaveObjective",
            "Server.Engines.Quests.Ninja.SneakPastGuardiansObjective",
            "Server.Engines.Quests.Ninja.UseTeleporterObjective",
            "Server.Engines.Quests.Ninja.GiveZoelNoteObjective",
            "Server.Engines.Quests.Ninja.GainInnInformationObjective",
            "Server.Engines.Quests.Ninja.ReturnFromInnObjective",
            "Server.Engines.Quests.Ninja.SearchForSwordObjective",
            "Server.Engines.Quests.Ninja.HallwayWalkObjective",
            "Server.Engines.Quests.Ninja.ReturnSwordObjective",
            "Server.Engines.Quests.Ninja.SlayHenchmenObjective",
            "Server.Engines.Quests.Ninja.GiveEminoSwordObjective",
        ],
        child: &[Tok::E, Tok::B],
    },
    SystemDef {
        name: "Server.Engines.Quests.Samurai.HaochisTrialsQuest",
        table: &[
            "Server.Engines.Quests.Samurai.AcceptConversation",
            "Server.Engines.Quests.Samurai.RadarConversation",
            "Server.Engines.Quests.Samurai.FirstTrialIntroConversation",
            "Server.Engines.Quests.Samurai.FirstTrialKillConversation",
            "Server.Engines.Quests.Samurai.GainKarmaConversation",
            "Server.Engines.Quests.Samurai.SecondTrialIntroConversation",
            "Server.Engines.Quests.Samurai.SecondTrialAttackConversation",
            "Server.Engines.Quests.Samurai.ThirdTrialIntroConversation",
            "Server.Engines.Quests.Samurai.ThirdTrialKillConversation",
            "Server.Engines.Quests.Samurai.FourthTrialIntroConversation",
            "Server.Engines.Quests.Samurai.FourthTrialCatsConversation",
            "Server.Engines.Quests.Samurai.FifthTrialIntroConversation",
            "Server.Engines.Quests.Samurai.FifthTrialReturnConversation",
            "Server.Engines.Quests.Samurai.LostSwordConversation",
            "Server.Engines.Quests.Samurai.SixthTrialIntroConversation",
            "Server.Engines.Quests.Samurai.SeventhTrialIntroConversation",
            "Server.Engines.Quests.Samurai.EndConversation",
            "Server.Engines.Quests.Samurai.FindHaochiObjective",
            "Server.Engines.Quests.Samurai.FirstTrialIntroObjective",
            "Server.Engines.Quests.Samurai.FirstTrialKillObjective",
            "Server.Engines.Quests.Samurai.FirstTrialReturnObjective",
            "Server.Engines.Quests.Samurai.SecondTrialIntroObjective",
            "Server.Engines.Quests.Samurai.SecondTrialAttackObjective",
            "Server.Engines.Quests.Samurai.SecondTrialReturnObjective",
            "Server.Engines.Quests.Samurai.ThirdTrialIntroObjective",
            "Server.Engines.Quests.Samurai.ThirdTrialKillObjective",
            "Server.Engines.Quests.Samurai.ThirdTrialReturnObjective",
            "Server.Engines.Quests.Samurai.FourthTrialIntroObjective",
            "Server.Engines.Quests.Samurai.FourthTrialCatsObjective",
            "Server.Engines.Quests.Samurai.FourthTrialReturnObjective",
            "Server.Engines.Quests.Samurai.FifthTrialIntroObjective",
            "Server.Engines.Quests.Samurai.FifthTrialReturnObjective",
            "Server.Engines.Quests.Samurai.SixthTrialIntroObjective",
            "Server.Engines.Quests.Samurai.SixthTrialReturnObjective",
            "Server.Engines.Quests.Samurai.SeventhTrialIntroObjective",
            "Server.Engines.Quests.Samurai.SeventhTrialReturnObjective",
        ],
        child: &[Tok::E, Tok::B],
    },
    SystemDef {
        name: "Server.Engines.Quests.Zento.TerribleHatchlingsQuest",
        table: &[
            "Server.Engines.Quests.Zento.AcceptConversation",
            "Server.Engines.Quests.Zento.DirectionConversation",
            "Server.Engines.Quests.Zento.TakeCareConversation",
            "Server.Engines.Quests.Zento.EndConversation",
            "Server.Engines.Quests.Zento.FirstKillObjective",
            "Server.Engines.Quests.Zento.SecondKillObjective",
            "Server.Engines.Quests.Zento.ThirdKillObjective",
            "Server.Engines.Quests.Zento.ReturnObjective",
        ],
        child: &[Tok::E],
    },
];

/// Objectives and conversations by full name: the layout `ChildDeserialize` reads (base classes first).
pub static CLASSES: &[(&str, &[Tok])] = &[
    (
        "Server.Engines.Quests.Ambitious.AcceptConversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Ambitious.DontOfferConversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Ambitious.DuringFungiGatheringConversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Ambitious.DuringKillQueensConversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Ambitious.End2Conversation",
        &[Tok::E],
    ),
    ("Server.Engines.Quests.Ambitious.EndConversation", &[Tok::E]),
    (
        "Server.Engines.Quests.Ambitious.FullBackpackConversation",
        &[Tok::E, Tok::B, Tok::B, Tok::B],
    ),
    (
        "Server.Engines.Quests.Ambitious.GatherFungiConversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Ambitious.GatherFungiObjective",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Ambitious.GetRewardObjective",
        &[Tok::E, Tok::B, Tok::B, Tok::B],
    ),
    (
        "Server.Engines.Quests.Ambitious.KillQueensObjective",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Ambitious.ReturnAfterKillsObjective",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Collector.AcceptConversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Collector.AlbertaAfterPaintingConversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Collector.AlbertaEndPaintingConversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Collector.AlbertaPaintingConversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Collector.AlbertaStoolConversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Collector.CaptureImagesObjective",
        &[Tok::E, Tok::ListEB],
    ),
    (
        "Server.Engines.Quests.Collector.DeclineConversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Collector.DontOfferConversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Collector.ElwoodDuringAutograph1Conversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Collector.ElwoodDuringAutograph2Conversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Collector.ElwoodDuringAutograph3Conversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Collector.ElwoodDuringFishConversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Collector.ElwoodDuringPainting1Conversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Collector.ElwoodDuringPainting2Conversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Collector.ElwoodDuringToys1Conversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Collector.ElwoodDuringToys2Conversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Collector.ElwoodDuringToys3Conversation",
        &[Tok::E],
    ),
    // Not in CollectorQuest's TypeReferenceTable: a saved EndConversation is written by name.
    ("Server.Engines.Quests.Collector.EndConversation", &[Tok::E]),
    (
        "Server.Engines.Quests.Collector.FindAlbertaObjective",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Collector.FindGabrielObjective",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Collector.FindSheetMusicObjective",
        &[Tok::E, Tok::E],
    ),
    (
        "Server.Engines.Quests.Collector.FindTomasObjective",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Collector.FishPearlsObjective",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Collector.FullEndConversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Collector.GabrielAutographConversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Collector.GabrielIgnoreConversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Collector.GabrielNoSheetMusicConversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Collector.GabrielSheetMusicConversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Collector.GetSheetMusicConversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Collector.MakeRoomObjective",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Collector.NoSheetMusicConversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Collector.ReturnAutographConversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Collector.ReturnAutographObjective",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Collector.ReturnImagesConversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Collector.ReturnImagesObjective",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Collector.ReturnPaintingConversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Collector.ReturnPaintingObjective",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Collector.ReturnPearlsConversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Collector.ReturnPearlsObjective",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Collector.ReturnSheetMusicObjective",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Collector.ReturnToysObjective",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Collector.SitOnTheStoolObjective",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Collector.TomasDuringCollectingConversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Collector.TomasToysConversation",
        &[Tok::E],
    ),
    ("Server.Engines.Quests.Doom.AcceptConversation", &[Tok::E]),
    (
        "Server.Engines.Quests.Doom.CollectBonesObjective",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Doom.VanquishDaemonConversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Doom.VanquishDaemonObjective",
        &[Tok::E, Tok::Ser, Tok::Ser],
    ),
    ("Server.Engines.Quests.Hag.AcceptConversation", &[Tok::E]),
    (
        "Server.Engines.Quests.Hag.ApprenticeCorpseConversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Hag.BlackheartFirstConversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Hag.BlackheartNoPirateConversation",
        &[Tok::E, Tok::B, Tok::B],
    ),
    (
        "Server.Engines.Quests.Hag.BlackheartPirateConversation",
        &[Tok::E, Tok::B],
    ),
    ("Server.Engines.Quests.Hag.DontOfferConversation", &[Tok::E]),
    ("Server.Engines.Quests.Hag.EndConversation", &[Tok::E]),
    (
        "Server.Engines.Quests.Hag.FindApprenticeObjective",
        &[Tok::FindApprentice],
    ),
    (
        "Server.Engines.Quests.Hag.FindGrizeldaAboutMurderObjective",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Hag.FindIngredientObjective",
        &[Tok::E, Tok::ListE, Tok::B],
    ),
    (
        "Server.Engines.Quests.Hag.FindZeefzorpulObjective",
        &[Tok::E, Tok::P3],
    ),
    (
        "Server.Engines.Quests.Hag.HagDuringCorpseSearchConversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Hag.HagDuringImpSearchConversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Hag.HagDuringIngredientsConversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Hag.ImpDeathConversation",
        &[Tok::E, Tok::P3],
    ),
    (
        "Server.Engines.Quests.Hag.KillImpsObjective",
        &[Tok::E, Tok::I],
    ),
    ("Server.Engines.Quests.Hag.MurderConversation", &[Tok::E]),
    (
        "Server.Engines.Quests.Hag.RecentlyFinishedConversation",
        &[Tok::E],
    ),
    ("Server.Engines.Quests.Hag.RecipeConversation", &[Tok::E]),
    (
        "Server.Engines.Quests.Hag.ReturnIngredientsObjective",
        &[Tok::E],
    ),
    ("Server.Engines.Quests.Hag.ReturnRecipeObjective", &[Tok::E]),
    (
        "Server.Engines.Quests.Hag.ZeefzorpulConversation",
        &[Tok::E],
    ),
    ("Server.Engines.Quests.Haven.AcceptConversation", &[Tok::E]),
    ("Server.Engines.Quests.Haven.BankerConversation", &[Tok::E]),
    (
        "Server.Engines.Quests.Haven.CashBankCheckObjective",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Haven.DryadAppleConversation",
        &[Tok::E],
    ),
    ("Server.Engines.Quests.Haven.DryadConversation", &[Tok::E]),
    (
        "Server.Engines.Quests.Haven.FewReagentsConversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Haven.FindApprenticeObjective",
        &[Tok::E],
    ),
    ("Server.Engines.Quests.Haven.FindDryadObjective", &[Tok::E]),
    (
        "Server.Engines.Quests.Haven.FindSchmendrickObjective",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Haven.FindUzeraanAboutReportObjective",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Haven.FindUzeraanBeginObjective",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Haven.FindUzeraanFirstTaskObjective",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Haven.GetDaemonBloodObjective",
        &[Tok::E, Tok::B],
    ),
    (
        "Server.Engines.Quests.Haven.GetDaemonBoneObjective",
        &[Tok::E, Tok::Ser],
    ),
    (
        "Server.Engines.Quests.Haven.KillHordeMinionsObjective",
        &[Tok::E, Tok::E],
    ),
    (
        "Server.Engines.Quests.Haven.LostDaemonBloodConversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Haven.LostDaemonBoneConversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Haven.LostFertileDirtConversation",
        &[Tok::E, Tok::B],
    ),
    (
        "Server.Engines.Quests.Haven.LostScrollOfPowerConversation",
        &[Tok::E, Tok::B],
    ),
    ("Server.Engines.Quests.Haven.RadarConversation", &[Tok::E]),
    (
        "Server.Engines.Quests.Haven.ReturnDaemonBloodObjective",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Haven.ReturnDaemonBoneObjective",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Haven.ReturnFertileDirtObjective",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Haven.ReturnScrollOfPowerObjective",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Haven.SchmendrickConversation",
        &[Tok::E],
    ),
    ("Server.Engines.Quests.Haven.TitheGoldObjective", &[Tok::E]),
    (
        "Server.Engines.Quests.Haven.UzeraanDaemonBloodConversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Haven.UzeraanDaemonBoneConversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Haven.UzeraanFertileDirtConversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Haven.UzeraanFirstTaskConversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Haven.UzeraanReportConversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Haven.UzeraanScrollOfPowerConversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Haven.UzeraanTitheConversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Matriarch.AcceptConversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Matriarch.DontOfferConversation",
        &[Tok::E, Tok::B],
    ),
    (
        "Server.Engines.Quests.Matriarch.DuringFungiProcessConversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Matriarch.DuringKillInfiltratorsConversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Matriarch.DuringWaterGatheringConversation",
        &[Tok::E],
    ),
    ("Server.Engines.Quests.Matriarch.EndConversation", &[Tok::E]),
    (
        "Server.Engines.Quests.Matriarch.FullBackpackConversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Matriarch.GatherWaterConversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Matriarch.GatherWaterObjective",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Matriarch.GetRewardObjective",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Matriarch.KillInfiltratorsObjective",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Matriarch.ProcessFungiConversation",
        &[Tok::E, Tok::B],
    ),
    (
        "Server.Engines.Quests.Matriarch.ProcessFungiObjective",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Matriarch.ReturnAfterKillsObjective",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Matriarch.ReturnAfterWaterObjective",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Naturalist.AcceptConversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Naturalist.DontOfferConversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Naturalist.EndConversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Naturalist.FullBackpackConversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Naturalist.NaturalistDuringStudyConversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Naturalist.ReturnToNaturalistObjective",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Naturalist.SpecialEndConversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Naturalist.StudyNestsObjective",
        &[Tok::E, Tok::ListE, Tok::B],
    ),
    ("Server.Engines.Quests.Necro.AcceptConversation", &[Tok::E]),
    (
        "Server.Engines.Quests.Necro.AnimateMaabusCorpseObjective",
        &[Tok::E],
    ),
    ("Server.Engines.Quests.Necro.BankerConversation", &[Tok::E]),
    (
        "Server.Engines.Quests.Necro.CashBankCheckObjective",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Necro.FetchAbraxusScrollObjective",
        &[Tok::E],
    ),
    ("Server.Engines.Quests.Necro.FindBankObjective", &[Tok::E]),
    (
        "Server.Engines.Quests.Necro.FindCallingScrollObjective",
        &[Tok::E, Tok::E, Tok::B, Tok::B],
    ),
    (
        "Server.Engines.Quests.Necro.FindCityOfLightObjective",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Necro.FindCrystalCaveObjective",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Necro.FindHorusAboutRewardObjective",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Necro.FindMaabusCorpseObjective",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Necro.FindMaabusTombObjective",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Necro.FindMardothAboutKronusObjective",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Necro.FindMardothAboutVaultObjective",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Necro.FindMardothEndObjective",
        &[Tok::E, Tok::B],
    ),
    (
        "Server.Engines.Quests.Necro.FindVaultOfSecretsObjective",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Necro.FindWellOfTearsObjective",
        &[Tok::E],
    ),
    ("Server.Engines.Quests.Necro.HealConversation", &[Tok::E]),
    ("Server.Engines.Quests.Necro.HorusConversation", &[Tok::E]),
    (
        "Server.Engines.Quests.Necro.HorusRewardConversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Necro.LostCallingScrollConversation",
        &[Tok::E, Tok::B],
    ),
    ("Server.Engines.Quests.Necro.MaabasConversation", &[Tok::E]),
    (
        "Server.Engines.Quests.Necro.MardothEndConversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Necro.MardothKronusConversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Necro.MardothVaultConversation",
        &[Tok::E],
    ),
    ("Server.Engines.Quests.Necro.RadarConversation", &[Tok::E]),
    (
        "Server.Engines.Quests.Necro.ReadAbraxusScrollConversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Necro.ReadAbraxusScrollObjective",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Necro.ReanimateMaabusConversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Necro.RetrieveAbraxusScrollObjective",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Necro.ReturnToCrystalCaveObjective",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Necro.SecondHorusConversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Necro.SpeakCavePasswordObjective",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Necro.UseCallingScrollObjective",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Necro.VaultOfSecretsConversation",
        &[Tok::E],
    ),
    ("Server.Engines.Quests.Ninja.AcceptConversation", &[Tok::E]),
    (
        "Server.Engines.Quests.Ninja.ContinueSlayHenchmenConversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Ninja.EarnGiftsConversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Ninja.EarnLessGiftsConversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Ninja.EnterCaveConversation",
        &[Tok::E],
    ),
    ("Server.Engines.Quests.Ninja.EnterCaveObjective", &[Tok::E]),
    (
        "Server.Engines.Quests.Ninja.FindEminoBeginObjective",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Ninja.FindZoelConversation",
        &[Tok::E],
    ),
    ("Server.Engines.Quests.Ninja.FindZoelObjective", &[Tok::E]),
    (
        "Server.Engines.Quests.Ninja.GainInnInformationConversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Ninja.GainInnInformationObjective",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Ninja.GiveEminoSwordConversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Ninja.GiveEminoSwordObjective",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Ninja.GiveZoelNoteConversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Ninja.GiveZoelNoteObjective",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Ninja.HallwayWalkConversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Ninja.HallwayWalkObjective",
        &[Tok::E, Tok::B],
    ),
    (
        "Server.Engines.Quests.Ninja.LostNoteConversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Ninja.LostSwordConversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Ninja.NeedToHideConversation",
        &[Tok::E],
    ),
    ("Server.Engines.Quests.Ninja.RadarConversation", &[Tok::E]),
    (
        "Server.Engines.Quests.Ninja.ReturnFromInnConversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Ninja.ReturnFromInnObjective",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Ninja.ReturnSwordConversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Ninja.ReturnSwordObjective",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Ninja.SearchForSwordConversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Ninja.SearchForSwordObjective",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Ninja.SlayHenchmenConversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Ninja.SlayHenchmenObjective",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Ninja.SneakPastGuardiansConversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Ninja.SneakPastGuardiansObjective",
        &[Tok::E, Tok::B],
    ),
    (
        "Server.Engines.Quests.Ninja.UseTeleporterConversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Ninja.UseTeleporterObjective",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Samurai.AcceptConversation",
        &[Tok::E],
    ),
    ("Server.Engines.Quests.Samurai.EndConversation", &[Tok::E]),
    (
        "Server.Engines.Quests.Samurai.FifthTrialIntroConversation",
        &[Tok::E, Tok::B],
    ),
    (
        "Server.Engines.Quests.Samurai.FifthTrialIntroObjective",
        &[Tok::E, Tok::B],
    ),
    (
        "Server.Engines.Quests.Samurai.FifthTrialReturnConversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Samurai.FifthTrialReturnObjective",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Samurai.FindHaochiObjective",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Samurai.FirstTrialIntroConversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Samurai.FirstTrialIntroObjective",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Samurai.FirstTrialKillConversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Samurai.FirstTrialKillObjective",
        &[Tok::E, Tok::E, Tok::E],
    ),
    (
        "Server.Engines.Quests.Samurai.FirstTrialReturnObjective",
        &[Tok::E, Tok::B],
    ),
    (
        "Server.Engines.Quests.Samurai.FourthTrialCatsConversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Samurai.FourthTrialCatsObjective",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Samurai.FourthTrialIntroConversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Samurai.FourthTrialIntroObjective",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Samurai.FourthTrialReturnObjective",
        &[Tok::E, Tok::B],
    ),
    (
        "Server.Engines.Quests.Samurai.GainKarmaConversation",
        &[Tok::E, Tok::B],
    ),
    (
        "Server.Engines.Quests.Samurai.LostSwordConversation",
        &[Tok::E],
    ),
    ("Server.Engines.Quests.Samurai.RadarConversation", &[Tok::E]),
    (
        "Server.Engines.Quests.Samurai.SecondTrialAttackConversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Samurai.SecondTrialAttackObjective",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Samurai.SecondTrialIntroConversation",
        &[Tok::E, Tok::B],
    ),
    (
        "Server.Engines.Quests.Samurai.SecondTrialIntroObjective",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Samurai.SecondTrialReturnObjective",
        &[Tok::E, Tok::B],
    ),
    (
        "Server.Engines.Quests.Samurai.SeventhTrialIntroConversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Samurai.SeventhTrialIntroObjective",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Samurai.SeventhTrialReturnObjective",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Samurai.SixthTrialIntroConversation",
        &[Tok::E, Tok::B],
    ),
    (
        "Server.Engines.Quests.Samurai.SixthTrialIntroObjective",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Samurai.SixthTrialReturnObjective",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Samurai.ThirdTrialIntroConversation",
        &[Tok::E, Tok::B],
    ),
    (
        "Server.Engines.Quests.Samurai.ThirdTrialIntroObjective",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Samurai.ThirdTrialKillConversation",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Samurai.ThirdTrialKillObjective",
        &[Tok::E],
    ),
    (
        "Server.Engines.Quests.Samurai.ThirdTrialReturnObjective",
        &[Tok::E],
    ),
    ("Server.Engines.Quests.Zento.AcceptConversation", &[Tok::E]),
    (
        "Server.Engines.Quests.Zento.DirectionConversation",
        &[Tok::E],
    ),
    ("Server.Engines.Quests.Zento.EndConversation", &[Tok::E]),
    ("Server.Engines.Quests.Zento.FirstKillObjective", &[Tok::E]),
    ("Server.Engines.Quests.Zento.ReturnObjective", &[Tok::E]),
    ("Server.Engines.Quests.Zento.SecondKillObjective", &[Tok::E]),
    (
        "Server.Engines.Quests.Zento.TakeCareConversation",
        &[Tok::E],
    ),
    ("Server.Engines.Quests.Zento.ThirdKillObjective", &[Tok::E]),
];
