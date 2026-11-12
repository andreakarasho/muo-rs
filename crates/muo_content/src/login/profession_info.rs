//! Profession info (the part character creation reads): `Data/Professions/<era>/
//! prof.txt`, loaded when the first character is created or a profession is asked for.

use muo_data::expansion::get_era_folder;
use muo_primitives::dotnet::{IgnoreCase, InvariantCase};
use muo_primitives::parse::{to_bool, to_i32};
use muo_server::World;
use muo_server::skills::{SkillName, SkillTable};

/// `ProfessionInfo`.
#[derive(Clone, Debug, Default)]
pub struct ProfessionInfo {
    pub id: i32,
    pub name: String,
    pub name_id: i32,
    pub desc_id: i32,
    pub top_level: bool,
    pub gump_id: i32,
    pub skills: Vec<(SkillName, u8)>,
    pub stats: [u8; 3],
}

/// `ProfessionInfo._professions`; slot 0 is "Advanced Skills".
#[derive(Default)]
pub struct Professions {
    loaded: bool,
    list: Vec<Option<ProfessionInfo>>,
}

impl Professions {
    pub fn len(&self) -> usize {
        self.list.len()
    }

    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }
}

/// `ProfessionInfo.VerifyProfession`.
pub fn verify_profession(w: &mut World, prof_index: i32) -> bool {
    ensure_loaded(w);
    prof_index > 0 && (prof_index as usize) < w.resources.get_mut::<Professions>().list.len()
}

/// `ProfessionInfo.GetProfession`.
pub fn get_profession(w: &mut World, prof_index: i32) -> Option<ProfessionInfo> {
    if !verify_profession(w, prof_index) {
        return None;
    }
    w.resources.get_mut::<Professions>().list[prof_index as usize].clone()
}

fn ensure_loaded(w: &mut World) {
    if w.resources.get_mut::<Professions>().loaded {
        return;
    }
    let skill_names: Vec<(i32, String)> = w
        .resources
        .get::<SkillTable>()
        .map(|t| {
            t.info
                .iter()
                .map(|s| (s.skill_id, s.profession_skill_name.clone()))
                .collect()
        })
        .unwrap_or_default();
    let parent = w.core.base_directory.join("Data/Professions");
    // `Core.FindDataFile("prof.txt", false)`, else the era folder's
    let file = w
        .config
        .data_directories()
        .find("prof.txt")
        .or_else(|| get_era_folder(&parent, w.expansion()).map(|f| f.join("prof.txt")));
    let list = file
        .and_then(|f| std::fs::read_to_string(f).ok())
        .map(|text| parse(&text, &skill_names))
        .unwrap_or_else(|| vec![None]);
    let p = w.resources.get_mut::<Professions>();
    p.list = list;
    p.loaded = true;
}

/// `Enum.TryParse<SkillName>(name)` (case-sensitive names, surrounding spaces ignored), then the profession
/// skill names of the skill table.
fn try_skill_name(name: &str, skill_names: &[(i32, String)]) -> Option<SkillName> {
    let n = name.trim();
    if let Ok(v) = n.parse::<i32>() {
        return SkillName::from_i32(v);
    }
    if let Some(s) = (0..SkillName::COUNT as i32)
        .filter_map(SkillName::from_i32)
        .find(|s| format!("{s:?}") == n)
    {
        return Some(s);
    }
    let lower = name.lower_inv().replace(' ', "");
    if lower.is_empty() {
        return None;
    }
    skill_names
        .iter()
        .find(|(_, p)| lower == p.lower_inv())
        .and_then(|(id, _)| SkillName::from_i32(*id))
}

fn parse(text: &str, skill_names: &[(i32, String)]) -> Vec<Option<ProfessionInfo>> {
    let mut max_prof = 0;
    let mut profs = Vec::new();
    let mut lines = text.lines();
    while let Some(line) = lines.next() {
        let line = line.trim();
        if line.is_empty() || !line.lower_inv().starts_with("begin") {
            continue;
        }
        let mut prof = ProfessionInfo::default();
        let (mut total_stats, mut total_skill) = (0, 0);
        for line in lines.by_ref() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if line.lower_inv().starts_with("end") {
                if prof.id > 0 && total_stats >= 80 && total_skill >= 100 {
                    fix_skills(&mut prof);
                    profs.push(prof);
                }
                break;
            }
            let cols: Vec<&str> = line.split('\t').filter(|c| !c.is_empty()).collect();
            if cols.len() < 2 {
                continue;
            }
            let key = cols[0].lower_inv();
            let value = cols[1].trim_matches('"');
            if key == "type" && !value.eq_ic("profession") {
                break;
            }
            match key.as_str() {
                "truename" => prof.name = value.to_owned(),
                "nameid" => prof.name_id = to_i32(value),
                "descid" => prof.desc_id = to_i32(value),
                "desc" => {
                    prof.id = to_i32(value);
                    max_prof = max_prof.max(prof.id);
                }
                "toplevel" => prof.top_level = to_bool(value),
                "gump" => prof.gump_id = to_i32(value),
                "skill" => {
                    let Some(name) = try_skill_name(value, skill_names) else {
                        continue;
                    };
                    let v: u8 = cols.get(2).and_then(|c| c.trim().parse().ok()).unwrap_or(0);
                    if prof.skills.len() < 4 {
                        prof.skills.push((name, v));
                    }
                    total_skill += i32::from(v);
                }
                "stat" => {
                    // `Enum.TryParse<StatType>` is case-sensitive: Str 1, Dex 2, Int 4; index is `stat >> 1`
                    let idx = match value.trim() {
                        "Str" => 0,
                        "Dex" => 1,
                        "Int" => 2,
                        _ => continue,
                    };
                    let v: u8 = cols.get(2).and_then(|c| c.trim().parse().ok()).unwrap_or(0);
                    prof.stats[idx] = v;
                    total_stats += i32::from(v);
                }
                _ => {}
            }
        }
    }
    let mut list: Vec<Option<ProfessionInfo>> = vec![None; max_prof as usize + 1];
    for p in profs {
        let id = p.id as usize;
        list[id] = Some(p);
    }
    list[0] = Some(ProfessionInfo {
        name: "Advanced Skills".to_owned(),
        ..ProfessionInfo::default()
    });
    list
}

/// `FixSkills`: the array is four long and keeps only up to the last skill that is not `(Alchemy, 0)`. Here the
/// list holds only the skills given, so trailing `(Alchemy, 0)` entries are the ones dropped.
fn fix_skills(p: &mut ProfessionInfo) {
    while p
        .skills
        .last()
        .is_some_and(|s| *s == (SkillName::Alchemy, 0))
    {
        p.skills.pop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_profession_block() {
        let text = "Begin\r\n\tName\t\t\tPaladin\r\n\tTrueName\t\t\"Paladin\"\r\n\tDesc\t\t\t5\r\n\tType\t\t\tProfession\r\n\tSkill\t\t\tChivalry\t\t50\r\n\tSkill\t\t\tTactics \t\t50\r\n\tSkill\t\t\tWrestling\t\t0\r\n\tStat\t\t\tStr\t\t\t30\r\n\tStat\t\t\tDex\t\t\t25\r\n\tStat\t\t\tInt\t\t\t25\r\nEnd\r\n";
        let list = parse(text, &[]);
        let p = list[5].as_ref().unwrap();
        assert_eq!(p.name, "Paladin");
        assert_eq!(p.stats, [30, 25, 25]);
        assert_eq!(
            p.skills,
            vec![
                (SkillName::Chivalry, 50),
                (SkillName::Tactics, 50),
                (SkillName::Wrestling, 0)
            ]
        );
        assert_eq!(list[0].as_ref().unwrap().name, "Advanced Skills");
    }

    /// `Utility.ToInt32` takes "0x" hex and `ToBoolean` takes "on"/"enabled"; stat names are case-sensitive.
    #[test]
    fn numbers_booleans_and_stats_parse_as_utility_does() {
        let text = "Begin
	Type	Profession
	Desc	0x10
	Gump	0x1F
	TopLevel	on
	Skill	Alchemy	100
	Stat	str	10
	Stat	Str	40
	Stat	Dex	40
End
";
        let list = parse(text, &[]);
        let p = list[16].as_ref().unwrap();
        assert_eq!((p.gump_id, p.top_level, p.stats), (31, true, [40, 40, 0]));
    }
}
