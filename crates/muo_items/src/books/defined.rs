//! The books with a fixed text: their `BookContent` (title, author, pages) is the `"bookContent"` prop of the
//! class template (`Data/Scripts/classes/items/books/defined.json`), the name some of them show the `"nameText"` prop.

use muo_server::type_registry::TypeRegistry;

use super::{BASE_BOOK, BLUE_BOOK, RED_BOOK, defined_book};

defined_book! {
    /// `Server.Items.BlackthornWelcomeBook`.
    BLACKTHORN_WELCOME_BOOK = "Server.Items.BlackthornWelcomeBook" : RED_BOOK;
}

defined_book! {
    /// `Server.Items.DrakovsJournal`.
    DRAKOVS_JOURNAL = "Server.Items.DrakovsJournal" : BLUE_BOOK;
}

defined_book! {
    /// `Server.Items.FropozJournal`.
    FROPOZ_JOURNAL = "Server.Items.FropozJournal" : RED_BOOK;
}

defined_book! {
    /// `Server.Items.KaburJournal`.
    KABUR_JOURNAL = "Server.Items.KaburJournal" : RED_BOOK;
}

defined_book! {
    /// `Server.Items.GrammarOfOrcish`.
    GRAMMAR_OF_ORCISH = "Server.Items.GrammarOfOrcish" : BASE_BOOK;
}

defined_book! {
    /// `Server.Items.CallToAnarchy`.
    CALL_TO_ANARCHY = "Server.Items.CallToAnarchy" : BASE_BOOK;
}

defined_book! {
    /// `Server.Items.ArmsAndWeaponsPrimer`.
    ARMS_AND_WEAPONS_PRIMER = "Server.Items.ArmsAndWeaponsPrimer" : BASE_BOOK;
}

defined_book! {
    /// `Server.Items.SongOfSamlethe`.
    SONG_OF_SAMLETHE = "Server.Items.SongOfSamlethe" : BASE_BOOK;
}

defined_book! {
    /// `Server.Items.TaleOfThreeTribes`.
    TALE_OF_THREE_TRIBES = "Server.Items.TaleOfThreeTribes" : BASE_BOOK;
}

defined_book! {
    /// `Server.Items.GuideToGuilds`.
    GUIDE_TO_GUILDS = "Server.Items.GuideToGuilds" : BASE_BOOK;
}

defined_book! {
    /// `Server.Items.BirdsOfBritannia`.
    BIRDS_OF_BRITANNIA = "Server.Items.BirdsOfBritannia" : BASE_BOOK;
}

defined_book! {
    /// `Server.Items.BritannianFlora`.
    BRITANNIAN_FLORA = "Server.Items.BritannianFlora" : BASE_BOOK;
}

defined_book! {
    /// `Server.Items.ChildrenTalesVol2`.
    CHILDREN_TALES_VOL2 = "Server.Items.ChildrenTalesVol2" : BASE_BOOK;
}

defined_book! {
    /// `Server.Items.TalesOfVesperVol1`.
    TALES_OF_VESPER_VOL1 = "Server.Items.TalesOfVesperVol1" : BASE_BOOK;
}

defined_book! {
    /// `Server.Items.DeceitDungeonOfHorror`.
    DECEIT_DUNGEON_OF_HORROR = "Server.Items.DeceitDungeonOfHorror" : BASE_BOOK;
}

defined_book! {
    /// `Server.Items.DimensionalTravel`.
    DIMENSIONAL_TRAVEL = "Server.Items.DimensionalTravel" : BASE_BOOK;
}

defined_book! {
    /// `Server.Items.EthicalHedonism`.
    ETHICAL_HEDONISM = "Server.Items.EthicalHedonism" : BASE_BOOK;
}

defined_book! {
    /// `Server.Items.MyStory`.
    MY_STORY = "Server.Items.MyStory" : BASE_BOOK;
}

defined_book! {
    /// `Server.Items.DiversityOfOurLand`.
    DIVERSITY_OF_OUR_LAND = "Server.Items.DiversityOfOurLand" : BASE_BOOK;
}

defined_book! {
    /// `Server.Items.QuestOfVirtues`.
    QUEST_OF_VIRTUES = "Server.Items.QuestOfVirtues" : BASE_BOOK;
}

defined_book! {
    /// `Server.Items.RegardingLlamas`.
    REGARDING_LLAMAS = "Server.Items.RegardingLlamas" : BASE_BOOK;
}

defined_book! {
    /// `Server.Items.TalkingToWisps`.
    TALKING_TO_WISPS = "Server.Items.TalkingToWisps" : BASE_BOOK;
}

defined_book! {
    /// `Server.Items.TamingDragons`.
    TAMING_DRAGONS = "Server.Items.TamingDragons" : BASE_BOOK;
}

defined_book! {
    /// `Server.Items.BoldStranger`.
    BOLD_STRANGER = "Server.Items.BoldStranger" : BASE_BOOK;
}

defined_book! {
    /// `Server.Items.BurningOfTrinsic`.
    BURNING_OF_TRINSIC = "Server.Items.BurningOfTrinsic" : BASE_BOOK;
}

defined_book! {
    /// `Server.Items.TheFight`.
    THE_FIGHT = "Server.Items.TheFight" : BASE_BOOK;
}

defined_book! {
    /// `Server.Items.LifeOfATravellingMinstrel`.
    LIFE_OF_A_TRAVELLING_MINSTREL = "Server.Items.LifeOfATravellingMinstrel" : BASE_BOOK;
}

defined_book! {
    /// `Server.Items.MajorTradeAssociation`.
    MAJOR_TRADE_ASSOCIATION = "Server.Items.MajorTradeAssociation" : BASE_BOOK;
}

defined_book! {
    /// `Server.Items.RankingsOfTrades`.
    RANKINGS_OF_TRADES = "Server.Items.RankingsOfTrades" : BASE_BOOK;
}

defined_book! {
    /// `Server.Items.WildGirlOfTheForest`.
    WILD_GIRL_OF_THE_FOREST = "Server.Items.WildGirlOfTheForest" : BASE_BOOK;
}

defined_book! {
    /// `Server.Items.TreatiseOnAlchemy`.
    TREATISE_ON_ALCHEMY = "Server.Items.TreatiseOnAlchemy" : BASE_BOOK;
}

defined_book! {
    /// `Server.Items.VirtueBook`.
    VIRTUE_BOOK = "Server.Items.VirtueBook" : BASE_BOOK;
}

defined_book! {
    /// `Server.Items.NewAquariumBook`.
    NEW_AQUARIUM_BOOK = "Server.Items.NewAquariumBook" : BLUE_BOOK;
}

defined_book! {
    /// `Server.Items.TranslatedGargoyleJournal`.
    TRANSLATED_GARGOYLE_JOURNAL = "Server.Items.TranslatedGargoyleJournal" : BLUE_BOOK;
}

/// Every class of this file.
pub static ALL: &[&muo_server::item::ItemType] = &[
    &BLACKTHORN_WELCOME_BOOK,
    &DRAKOVS_JOURNAL,
    &FROPOZ_JOURNAL,
    &KABUR_JOURNAL,
    &GRAMMAR_OF_ORCISH,
    &CALL_TO_ANARCHY,
    &ARMS_AND_WEAPONS_PRIMER,
    &SONG_OF_SAMLETHE,
    &TALE_OF_THREE_TRIBES,
    &GUIDE_TO_GUILDS,
    &BIRDS_OF_BRITANNIA,
    &BRITANNIAN_FLORA,
    &CHILDREN_TALES_VOL2,
    &TALES_OF_VESPER_VOL1,
    &DECEIT_DUNGEON_OF_HORROR,
    &DIMENSIONAL_TRAVEL,
    &ETHICAL_HEDONISM,
    &MY_STORY,
    &DIVERSITY_OF_OUR_LAND,
    &QUEST_OF_VIRTUES,
    &REGARDING_LLAMAS,
    &TALKING_TO_WISPS,
    &TAMING_DRAGONS,
    &BOLD_STRANGER,
    &BURNING_OF_TRINSIC,
    &THE_FIGHT,
    &LIFE_OF_A_TRAVELLING_MINSTREL,
    &MAJOR_TRADE_ASSOCIATION,
    &RANKINGS_OF_TRADES,
    &WILD_GIRL_OF_THE_FOREST,
    &TREATISE_ON_ALCHEMY,
    &VIRTUE_BOOK,
    &NEW_AQUARIUM_BOOK,
    &TRANSLATED_GARGOYLE_JOURNAL,
];

/// Registers the classes of this file.
pub fn register(types: &mut TypeRegistry) {
    for t in ALL {
        types.register_item(t, &[]);
    }
}
