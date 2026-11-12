//! `[LanguageStatistics` writes `languages.txt`, how many accounts (by their first
//! character with a language) use each client language. Names keep the file's U+200E marks.

use muo_server::World;
use muo_server::commands::{CommandEventArgs, CommandSystem};
use muo_server::mobile::AccessLevel;
use muo_server::mobile::part2::messages::send_message;

/// `Environment.NewLine` (`StreamWriter.WriteLine`).
const NL: &str = if cfg!(windows) { "\r\n" } else { "\n" };

/// `InternationalCode`: the code, the language and the country, and their local names where known.
struct Code(
    &'static str,
    &'static str,
    &'static str,
    Option<(&'static str, &'static str)>,
);

const DEFAULT_LOCAL_NAMES: bool = false;
const SHOW_ALTERNATIVES: bool = true;
const COUNT_ACCOUNTS: bool = true; // will consider only first character's valid language

static CODES: &[Code] = &[
    Code(
        "ARA",
        "Arabic",
        "Saudi Arabia",
        Some(("العربية", "السعودية")),
    ),
    Code("ARI", "Arabic", "Iraq", Some(("العربية", "العراق"))),
    Code("ARE", "Arabic", "Egypt", Some(("العربية", "مصر"))),
    Code("ARL", "Arabic", "Libya", Some(("العربية", "ليبيا"))),
    Code("ARG", "Arabic", "Algeria", Some(("العربية", "الجزائر"))),
    Code("ARM", "Arabic", "Morocco", Some(("العربية", "المغرب"))),
    Code("ART", "Arabic", "Tunisia", Some(("العربية", "تونس"))),
    Code("ARO", "Arabic", "Oman", Some(("العربية", "عمان"))),
    Code("ARY", "Arabic", "Yemen", Some(("العربية", "اليمن"))),
    Code("ARS", "Arabic", "Syria", Some(("العربية", "سورية"))),
    Code("ARJ", "Arabic", "Jordan", Some(("العربية", "الأردن"))),
    Code("ARB", "Arabic", "Lebanon", Some(("العربية", "لبنان"))),
    Code("ARK", "Arabic", "Kuwait", Some(("العربية", "الكويت"))),
    Code("ARU", "Arabic", "U.A.E.", Some(("العربية", "الامارات"))),
    Code("ARH", "Arabic", "Bahrain", Some(("العربية", "البحرين"))),
    Code("ARQ", "Arabic", "Qatar", Some(("العربية", "قطر"))),
    Code(
        "BGR",
        "Bulgarian",
        "Bulgaria",
        Some(("Български", "България")),
    ),
    Code("CAT", "Catalan", "Spain", Some(("Català", "Espanya"))),
    Code("CHT", "Chinese", "Taiwan", Some(("台語", "臺灣"))),
    Code("CHS", "Chinese", "PRC", Some(("中文", "中国"))),
    Code("ZHH", "Chinese", "Hong Kong", Some(("中文", "香港"))),
    Code("ZHI", "Chinese", "Singapore", Some(("中文", "新加坡"))),
    Code("ZHM", "Chinese", "Macau", Some(("中文", "澳門"))),
    Code(
        "CSY",
        "Czech",
        "Czech Republic",
        Some(("Čeština", "Česká republika")),
    ),
    Code("DAN", "Danish", "Denmark", Some(("Dansk", "Danmark"))),
    Code("DEU", "German", "Germany", Some(("Deutsch", "Deutschland"))),
    Code(
        "DES",
        "German",
        "Switzerland",
        Some(("Deutsch", "der Schweiz")),
    ),
    Code("DEA", "German", "Austria", Some(("Deutsch", "Österreich"))),
    Code(
        "DEL",
        "German",
        "Luxembourg",
        Some(("Deutsch", "Luxembourg")),
    ),
    Code(
        "DEC",
        "German",
        "Liechtenstein",
        Some(("Deutsch", "Liechtenstein")),
    ),
    Code("ELL", "Greek", "Greece", Some(("Ελληνικά", "Ελλάδα"))),
    Code("ENU", "English", "United States", None),
    Code("ENG", "English", "United Kingdom", None),
    Code("ENA", "English", "Australia", None),
    Code("ENC", "English", "Canada", None),
    Code("ENZ", "English", "New Zealand", None),
    Code("ENI", "English", "Ireland", None),
    Code("ENS", "English", "South Africa", None),
    Code("ENJ", "English", "Jamaica", None),
    Code("ENB", "English", "Caribbean", None),
    Code("ENL", "English", "Belize", None),
    Code("ENT", "English", "Trinidad", None),
    Code("ENW", "English", "Zimbabwe", None),
    Code("ENP", "English", "Philippines", None),
    Code(
        "ESP",
        "Spanish",
        "Spain (Traditional Sort)",
        Some(("Español", "España (tipo tradicional)")),
    ),
    Code("ESM", "Spanish", "Mexico", Some(("Español", "México"))),
    Code(
        "ESN",
        "Spanish",
        "Spain (International Sort)",
        Some(("Español", "España (tipo internacional)")),
    ),
    Code(
        "ESG",
        "Spanish",
        "Guatemala",
        Some(("Español", "Guatemala")),
    ),
    Code(
        "ESC",
        "Spanish",
        "Costa Rica",
        Some(("Español", "Costa Rica")),
    ),
    Code("ESA", "Spanish", "Panama", Some(("Español", "Panama"))),
    Code(
        "ESD",
        "Spanish",
        "Dominican Republic",
        Some(("Español", "Republica Dominicana")),
    ),
    Code(
        "ESV",
        "Spanish",
        "Venezuela",
        Some(("Español", "Venezuela")),
    ),
    Code("ESO", "Spanish", "Colombia", Some(("Español", "Colombia"))),
    Code("ESR", "Spanish", "Peru", Some(("Español", "Peru"))),
    Code(
        "ESS",
        "Spanish",
        "Argentina",
        Some(("Español", "Argentina")),
    ),
    Code("ESF", "Spanish", "Ecuador", Some(("Español", "Ecuador"))),
    Code("ESL", "Spanish", "Chile", Some(("Español", "Chile"))),
    Code("ESY", "Spanish", "Uruguay", Some(("Español", "Uruguay"))),
    Code("ESZ", "Spanish", "Paraguay", Some(("Español", "Paraguay"))),
    Code("ESB", "Spanish", "Bolivia", Some(("Español", "Bolivia"))),
    Code(
        "ESE",
        "Spanish",
        "El Salvador",
        Some(("Español", "El Salvador")),
    ),
    Code("ESH", "Spanish", "Honduras", Some(("Español", "Honduras"))),
    Code(
        "ESI",
        "Spanish",
        "Nicaragua",
        Some(("Español", "Nicaragua")),
    ),
    Code(
        "ESU",
        "Spanish",
        "Puerto Rico",
        Some(("Español", "Puerto Rico")),
    ),
    Code("FIN", "Finnish", "Finland", Some(("Suomi", "Suomi"))),
    Code("FRA", "French", "France", Some(("Français", "France"))),
    Code("FRB", "French", "Belgium", Some(("Français", "Belgique"))),
    Code("FRC", "French", "Canada", Some(("Français", "Canada"))),
    Code("FRS", "French", "Switzerland", Some(("Français", "Suisse"))),
    Code(
        "FRL",
        "French",
        "Luxembourg",
        Some(("Français", "Luxembourg")),
    ),
    Code("FRM", "French", "Monaco", Some(("Français", "Monaco"))),
    Code("HEB", "Hebrew", "Israel", Some(("עִבְרִית", "ישׂראל"))),
    Code(
        "HUN",
        "Hungarian",
        "Hungary",
        Some(("Magyar", "Magyarország")),
    ),
    Code("ISL", "Icelandic", "Iceland", Some(("Íslenska", "Ísland"))),
    Code("ITA", "Italian", "Italy", Some(("Italiano", "Italia"))),
    Code(
        "ITS",
        "Italian",
        "Switzerland",
        Some(("Italiano", "Svizzera")),
    ),
    Code("JPN", "Japanese", "Japan", Some(("日本語", "日本"))),
    Code(
        "KOR",
        "Korean (Extended Wansung)",
        "Korea",
        Some(("한국어", "한국")),
    ),
    Code(
        "NLD",
        "Dutch",
        "Netherlands",
        Some(("Nederlands", "Nederland")),
    ),
    Code("NLB", "Dutch", "Belgium", Some(("Nederlands", "België"))),
    Code(
        "NOR",
        "Norwegian",
        "Norway (Bokmål)",
        Some(("Norsk", "Norge (Bokmål)")),
    ),
    Code(
        "NON",
        "Norwegian",
        "Norway (Nynorsk)",
        Some(("Norsk", "Norge (Nynorsk)")),
    ),
    Code("PLK", "Polish", "Poland", Some(("Polski", "Polska"))),
    Code("PTB", "Portuguese", "Brazil", Some(("Português", "Brasil"))),
    Code(
        "PTG",
        "Portuguese",
        "Portugal",
        Some(("Português", "Brasil")),
    ),
    Code(
        "ROM",
        "Romanian",
        "Romania",
        Some(("Limba Română", "România")),
    ),
    Code("RUS", "Russian", "Russia", Some(("Русский", "Россия"))),
    Code("HRV", "Croatian", "Croatia", Some(("Hrvatski", "Hrvatska"))),
    Code(
        "SRL",
        "Serbian",
        "Serbia (Latin)",
        Some(("Srpski", "Srbija i Crna Gora")),
    ),
    Code(
        "SRB",
        "Serbian",
        "Serbia (Cyrillic)",
        Some(("Српски", "Србија и Црна Гора")),
    ),
    Code(
        "SKY",
        "Slovak",
        "Slovakia",
        Some(("Slovenčina", "Slovensko")),
    ),
    Code("SQI", "Albanian", "Albania", Some(("Shqip", "Shqipëria"))),
    Code("SVE", "Swedish", "Sweden", Some(("Svenska", "Sverige"))),
    Code("SVF", "Swedish", "Finland", Some(("Svenska", "Finland"))),
    Code("THA", "Thai", "Thailand", Some(("ภาษาไทย", "ประเทศไทย"))),
    Code("TRK", "Turkish", "Turkey", Some(("Türkçe", "Türkiye"))),
    Code("URP", "Urdu", "Pakistan", Some(("اردو", "پاکستان"))),
    Code(
        "IND",
        "Indonesian",
        "Indonesia",
        Some(("Bahasa Indonesia", "Indonesia")),
    ),
    Code(
        "UKR",
        "Ukrainian",
        "Ukraine",
        Some(("Українська", "Украина")),
    ),
    Code(
        "BEL",
        "Belarusian",
        "Belarus",
        Some(("Беларускі", "Беларусь")),
    ),
    Code(
        "SLV",
        "Slovene",
        "Slovenia",
        Some(("Slovenščina", "Slovenija")),
    ),
    Code("ETI", "Estonian", "Estonia", Some(("Eesti", "Eesti"))),
    Code("LVI", "Latvian", "Latvia", Some(("Latviešu", "Latvija"))),
    Code(
        "LTH",
        "Lithuanian",
        "Lithuania",
        Some(("Lietuvių", "Lietuva")),
    ),
    Code(
        "LTC",
        "Classic Lithuanian",
        "Lithuania",
        Some(("Lietuviškai", "Lietuva")),
    ),
    Code("FAR", "Farsi", "Iran", Some(("فارسى", "ايران"))),
    Code(
        "VIT",
        "Vietnamese",
        "Viet Nam",
        Some(("tiếng Việt", "Việt Nam")),
    ),
    Code("HYE", "Armenian", "Armenia", Some(("Հայերէն", "Հայաստան"))),
    Code(
        "AZE",
        "Azeri",
        "Azerbaijan (Latin)",
        Some(("Azərbaycanca", "Azərbaycan")),
    ),
    Code(
        "AZE",
        "Azeri",
        "Azerbaijan (Cyrillic)",
        Some(("Азәрбајҹанҹа", "Азәрбајҹан")),
    ),
    Code("EUQ", "Basque", "Spain", Some(("Euskera", "Espainia"))),
    Code(
        "MKI",
        "Macedonian",
        "Macedonia",
        Some(("Македонски", "Македонија")),
    ),
    Code(
        "AFK",
        "Afrikaans",
        "South Africa",
        Some(("Afrikaans", "Republiek van Suid-Afrika")),
    ),
    Code(
        "KAT",
        "Georgian",
        "Georgia",
        Some(("ქართული", "საკარტველო")),
    ),
    Code(
        "FOS",
        "Faeroese",
        "Faeroe Islands",
        Some(("Føroyska", "Føroya")),
    ),
    Code("HIN", "Hindi", "India", Some(("हिन्दी", "भारत"))),
    Code(
        "MSL",
        "Malay",
        "Malaysia",
        Some(("Bahasa melayu", "Malaysia")),
    ),
    Code(
        "MSB",
        "Malay",
        "Brunei Darussalam",
        Some(("Bahasa melayu", "Negara Brunei Darussalam")),
    ),
    Code("KAZ", "Kazak", "Kazakstan", Some(("Қазақ", "Қазақстан"))),
    Code("SWK", "Swahili", "Kenya", Some(("Kiswahili", "Kenya"))),
    Code(
        "UZB",
        "Uzbek",
        "Uzbekistan (Latin)",
        Some(("O'zbek", "O'zbekiston")),
    ),
    Code(
        "UZB",
        "Uzbek",
        "Uzbekistan (Cyrillic)",
        Some(("Ўзбек", "Ўзбекистон")),
    ),
    Code("TAT", "Tatar", "Tatarstan", Some(("Татарча", "Татарстан"))),
    Code("BEN", "Bengali", "India", Some(("বাংলা", "ভারত"))),
    Code("PAN", "Punjabi", "India", Some(("ਪੰਜਾਬੀ", "ਭਾਰਤ"))),
    Code("GUJ", "Gujarati", "India", Some(("ગુજરાતી", "ભારત"))),
    Code("ORI", "Oriya", "India", Some(("ଓଡ଼ିଆ", "ଭାରତ"))),
    Code("TAM", "Tamil", "India", Some(("தமிழ்", "இந்தியா"))),
    Code("TEL", "Telugu", "India", Some(("తెలుగు", "భారత"))),
    Code("KAN", "Kannada", "India", Some(("ಕನ್ನಡ", "ಭಾರತ"))),
    Code("MAL", "Malayalam", "India", Some(("മലയാളം", "ഭാരത"))),
    Code("ASM", "Assamese", "India", Some(("অসমিয়া", "Bhārat"))),
    Code("MAR", "Marathi", "India", Some(("मराठी", "भारत"))),
    Code("SAN", "Sanskrit", "India", Some(("संस्कृत", "भारतम्"))),
    Code("KOK", "Konkani", "India", Some(("कोंकणी", "भारत"))),
];

impl Code {
    /// `InternationalCode.GetName`.
    fn name(&self) -> String {
        match self.3 {
            Some((ll, kl)) => {
                let (lang, country) = if DEFAULT_LOCAL_NAMES {
                    (ll, kl)
                } else {
                    (self.1, self.2)
                };
                let mut s = format!("{lang}\u{200e} - {country}");
                if SHOW_ALTERNATIVES {
                    let (lang, country) = if DEFAULT_LOCAL_NAMES {
                        (self.1, self.2)
                    } else {
                        (ll, kl)
                    };
                    s += &format!("\u{200e} \u{3010}{lang}\u{200e} - {country}\u{200e}\u{3011}");
                }
                s
            }
            None => format!("{}\u{200e} - {}", self.1, self.2),
        }
    }
}

/// `GetFormattedInfo`: the first row of that code, or `Unknown code`.
fn formatted_info(code: &str) -> String {
    if code.chars().count() == 3
        && let Some(c) = CODES.iter().find(|c| c.0 == code)
    {
        return c.name();
    }
    format!("Unknown code {code}")
}

pub fn configure(w: &mut World) {
    CommandSystem::register(
        w,
        "LanguageStatistics",
        AccessLevel::Administrator,
        &[],
        std::rc::Rc::new(on_command),
    );
}

/// `LanguageStatistics_OnCommand`.
fn on_command(w: &mut World, e: &mut CommandEventArgs) {
    // A `Dictionary<string, counter>` (insertion order), sorted by count descending, then code.
    let mut counts: indexmap::IndexMap<String, i32> = indexmap::IndexMap::new();
    if COUNT_ACCOUNTS {
        for a in crate::accounting::accounts(w).ids() {
            for m in crate::accounting::mobiles(w, a) {
                if let Some(lang) = w.mobile(m).and_then(|m| m.language.clone()) {
                    *counts.entry(lang.to_uppercase()).or_insert(0) += 1;
                    break;
                }
            }
        }
    } else {
        for m in w.mobiles.values() {
            if let (true, Some(lang)) = (m.player, &m.language) {
                *counts.entry(lang.to_uppercase()).or_insert(0) += 1;
            }
        }
    }
    let mut text = format!(
        "Language statistics. Numbers show how many {} use the specified language.{NL}",
        if COUNT_ACCOUNTS {
            "accounts"
        } else {
            "playermobile"
        }
    );
    text += &format!("{}{NL}{NL}", "=".repeat(100));
    let mut list: Vec<(String, i32)> = counts.into_iter().collect();
    list.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    for (code, n) in list {
        text += &format!("{}\u{200e} : {n}{NL}", formatted_info(&code));
    }
    if let Err(err) = std::fs::write("languages.txt", text) {
        tracing::warn!("LanguageStatistics: {err}");
    }
    send_message(w, e.mobile, "Languages list generated.");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_follow_the_expected_formats() {
        assert_eq!(formatted_info("ENU"), "English\u{200e} - United States");
        assert!(
            formatted_info("ARA").starts_with("Arabic\u{200e} - Saudi Arabia\u{200e} \u{3010}")
        );
        assert_eq!(formatted_info("XX"), "Unknown code XX");
    }
}
