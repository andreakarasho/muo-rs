//! The pre-binary account loader: `Accounts.DeserializeXml` and the `XmlElement` constructors of `Account`,
//! `AccountComment` and `AccountTag`.
//!
//! A server whose `Accounts/accounts.xml` exists loads only that file (the binary files are not read). RunUO and
//! ServUO passwords (`newSecureCryptPassword`, `newCryptPassword`, `cryptPassword`) are adopted with their
//! algorithm; a plain `password` is rehashed with the configured one. An account that fails to load is skipped with
//! a warning.

use std::path::Path;

use muo_primitives::{DateTime, Serial, TimeSpan};
use muo_server::World;
use muo_server::iaccount::AccountId;
use muo_server::ids::MobileId;
use muo_server::mobile::AccessLevel;
use roxmltree::Node;
use tracing::warn;

use super::security::PasswordProtectionAlgorithm;
use super::{Account, AccountComment, AccountHooks, AccountTag, accounts, check_young, is_young};

/// `node["name"]`: the first child element with the name.
fn child<'a, 'i>(node: Node<'a, 'i>, name: &str) -> Option<Node<'a, 'i>> {
    node.children()
        .find(|c| c.is_element() && c.tag_name().name() == name)
}

/// `GetElementsByTagName(name)`: every descendant element with the name, in document order.
fn descendants<'a, 'i>(node: Node<'a, 'i>, name: &'a str) -> impl Iterator<Item = Node<'a, 'i>> {
    node.descendants()
        .filter(move |d| d.is_element() && *d != node && d.tag_name().name() == name)
}

/// `XmlNode.InnerText`.
fn inner_text(node: Node) -> String {
    node.descendants()
        .filter(|d| d.is_text())
        .filter_map(|d| d.text())
        .collect()
}

/// `Utility.GetText(node, default)`.
fn text(node: Option<Node>, default: Option<&str>) -> Option<String> {
    node.map(inner_text).or_else(|| default.map(str::to_owned))
}

/// `Utility.GetAttribute(node, name, default)`.
fn attribute(node: Node, name: &str, default: Option<&str>) -> Option<String> {
    node.attribute(name)
        .map(str::to_owned)
        .or_else(|| default.map(str::to_owned))
}

/// `Utility.GetXMLInt32`: `XmlConvert.ToInt32` (trims white space), else `int.TryParse`, else the default.
fn xml_int32(s: Option<&str>, default: i32) -> i32 {
    s.and_then(|s| s.trim().parse().ok()).unwrap_or(default)
}

/// `Utility.GetXMLUInt32`.
fn xml_uint32(s: Option<&str>, default: u32) -> u32 {
    s.and_then(|s| s.trim().parse().ok()).unwrap_or(default)
}

/// `Utility.GetXMLDateTime`: an ISO 8601 date and time (taken as UTC), else the default.
fn xml_date_time(s: Option<&str>, default: DateTime) -> DateTime {
    s.and_then(|s| DateTime::parse_iso(s.trim()))
        .unwrap_or(default)
}

/// `Utility.GetXMLTimeSpan`: `XmlConvert.ToTimeSpan`, an ISO 8601 duration such as `P1DT2H3M4.5S`.
fn xml_time_span(s: Option<&str>, default: TimeSpan) -> TimeSpan {
    s.and_then(|s| parse_duration(s.trim())).unwrap_or(default)
}

fn parse_duration(s: &str) -> Option<TimeSpan> {
    let (neg, rest) = match s.strip_prefix('-') {
        Some(r) => (true, r),
        None => (false, s),
    };
    let rest = rest.strip_prefix('P')?;
    let (date, time) = match rest.split_once('T') {
        Some((d, t)) => (d, t),
        None => (rest, ""),
    };
    if date.is_empty() && time.is_empty() {
        return None;
    }
    let mut total = 0f64;
    let mut number = String::new();
    for c in date.chars() {
        match c {
            '0'..='9' | '.' => number.push(c),
            'D' => total += number.parse::<f64>().ok()? * 86400.0,
            'Y' | 'M' => return None, // years and months have no fixed length
            _ => return None,
        }
        if matches!(c, 'D') {
            number.clear();
        }
    }
    if !number.is_empty() {
        return None;
    }
    for c in time.chars() {
        match c {
            '0'..='9' | '.' => number.push(c),
            'H' => total += number.parse::<f64>().ok()? * 3600.0,
            'M' => total += number.parse::<f64>().ok()? * 60.0,
            'S' => total += number.parse::<f64>().ok()?,
            _ => return None,
        }
        if matches!(c, 'H' | 'M' | 'S') {
            number.clear();
        }
    }
    if !number.is_empty() {
        return None;
    }
    let span = TimeSpan::from_secs_f64(total);
    Some(if neg { -span } else { span })
}

/// `Enum.TryParse<AccessLevel>(text, ignoreCase: true)`: a name or a number; anything else leaves `Player`.
fn parse_access_level(s: &str) -> AccessLevel {
    let s = s.trim();
    if let Ok(n) = s.parse::<u8>() {
        return AccessLevel::from_i32(i32::from(n)).unwrap_or_default();
    }
    [
        AccessLevel::Player,
        AccessLevel::Counselor,
        AccessLevel::GameMaster,
        AccessLevel::Seer,
        AccessLevel::Administrator,
        AccessLevel::Developer,
        AccessLevel::Owner,
    ]
    .into_iter()
    .find(|l| format!("{l:?}").eq_ignore_ascii_case(s))
    .unwrap_or_default()
}

/// `Enum.TryParse<PasswordProtectionAlgorithm>(text, ignoreCase: true)`; failure is `None`.
fn parse_algorithm(s: Option<&str>) -> PasswordProtectionAlgorithm {
    let Some(s) = s.map(str::trim) else {
        return PasswordProtectionAlgorithm::None;
    };
    if let Ok(n) = s.parse::<i32>() {
        return PasswordProtectionAlgorithm::from_i32(n);
    }
    s.parse().unwrap_or(PasswordProtectionAlgorithm::None)
}

/// `new AccountComment(XmlElement)`.
fn comment(node: Node, now: DateTime) -> AccountComment {
    AccountComment {
        added_by: attribute(node, "addedBy", Some("empty")).unwrap_or_default(),
        last_modified: xml_date_time(node.attribute("lastModified"), now),
        content: inner_text(node),
    }
}

/// `new AccountTag(XmlElement)`.
fn tag(node: Node) -> AccountTag {
    AccountTag {
        name: attribute(node, "name", Some("empty")).unwrap_or_default(),
        value: Some(inner_text(node)),
    }
}

/// `Account(XmlElement)` up to its `Accounts.Add`. `Err` is a load failure the caller skips.
fn account_from(w: &mut World, node: Node) -> Result<AccountId, String> {
    let now = w.now;
    let serial = accounts(w).new_account_serial();
    let mut a = Account::blank(serial, now);
    a.username = text(child(node, "username"), Some("empty")).unwrap_or_default();
    a.password_algorithm = parse_algorithm(text(child(node, "passwordAlgorithm"), None).as_deref());

    // Backward compatibility with RunUO/ServUO
    let mut plain_password = None;
    if a.password_algorithm == PasswordProtectionAlgorithm::None {
        let upgrade = |a: &mut Account, name: &str, algorithm: PasswordProtectionAlgorithm| {
            let Some(p) = text(child(node, name), None) else {
                return false;
            };
            a.password_algorithm = algorithm;
            a.password = Some(p.replace('-', ""));
            true
        };
        let upgraded = upgrade(
            &mut a,
            "newSecureCryptPassword",
            PasswordProtectionAlgorithm::Sha2,
        ) || upgrade(
            &mut a,
            "newCryptPassword",
            PasswordProtectionAlgorithm::Sha1,
        ) || upgrade(&mut a, "cryptPassword", PasswordProtectionAlgorithm::Md5);
        // Automatically upgrade plain passwords to current algorithm.
        if !upgraded {
            plain_password = Some(
                text(child(node, "password"), None).ok_or("account has no password")?, // an account without a password is a bad record
            );
        }
    } else {
        a.password = text(child(node, "password"), None);
    }

    a.access_level =
        parse_access_level(&text(child(node, "accessLevel"), Some("Player")).unwrap_or_default());
    a.flags = xml_int32(text(child(node, "flags"), Some("0")).as_deref(), 0);
    a.created = xml_date_time(text(child(node, "created"), None).as_deref(), now);
    a.last_login = xml_date_time(text(child(node, "lastLogin"), None).as_deref(), now);
    a.total_gold = xml_int32(text(child(node, "totalGold"), Some("0")).as_deref(), 0);
    a.total_plat = xml_int32(text(child(node, "totalPlat"), Some("0")).as_deref(), 0);

    // LoadMobiles: seven slots; a character that does not exist is an empty slot
    a.mobiles = vec![None; 7];
    if let Some(chars) = child(node, "chars") {
        for ele in descendants(chars, "char") {
            let index = xml_int32(attribute(ele, "index", Some("0")).as_deref(), 0);
            let s = xml_uint32(Some(&inner_text(ele)), 0);
            if index >= 0 && (index as usize) < a.mobiles.len() {
                a.mobiles[index as usize] = w.find_mobile(Serial::new(s));
            }
        }
    }
    // LoadComments, LoadTags
    if let Some(comments) = child(node, "comments") {
        a.comments = descendants(comments, "comment")
            .map(|c| comment(c, now))
            .collect();
    }
    if let Some(tags) = child(node, "tags") {
        a.tags = descendants(tags, "tag").map(tag).collect();
    }
    // LoadAddressList: at most `count` addresses
    if let Some(list) = child(node, "addressList") {
        let count = xml_int32(attribute(list, "count", Some("0")).as_deref(), 0).max(0) as usize;
        a.login_ips = descendants(list, "ip")
            .filter_map(|ip| muo_primitives::dotnet::parse_ip_address(inner_text(ip).trim()))
            .take(count)
            .collect();
    }
    let total_game_time = xml_time_span(
        text(child(node, "totalGameTime"), None).as_deref(),
        TimeSpan::ZERO,
    );
    let mobiles: Vec<MobileId> = a.mobiles.iter().flatten().copied().collect();
    let id = accounts(w).add(a);
    if let Some(plain) = plain_password {
        super::set_password(w, id, &plain);
    }
    for m in &mobiles {
        if let Some(mm) = w.mobile_mut(*m) {
            mm.account = Some(id);
        }
    }
    let mut total = total_game_time;
    if total == TimeSpan::ZERO {
        let f = w.resources.get::<AccountHooks>().map(|h| h.game_time);
        for m in &mobiles {
            if let Some(t) = f.and_then(|f| f(w, *m)) {
                total += t;
            }
        }
    }
    if let Some(a) = super::account_mut(w, id) {
        a.total_game_time = total;
    }
    if is_young(w, id) {
        check_young(w, id);
    }
    Ok(id)
}

/// `Accounts.DeserializeXml(filePath)`.
pub fn deserialize_xml(w: &mut World, path: &Path) -> Result<usize, String> {
    let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let doc = roxmltree::Document::parse_with_options(
        &text,
        roxmltree::ParsingOptions {
            allow_dtd: true,
            ..Default::default()
        },
    )
    .map_err(|e| e.to_string())?;
    let root = doc.root();
    let Some(accounts_node) = child(root, "accounts") else {
        return Err("Unable to load xml file".into());
    };
    let mut loaded = 0;
    for node in descendants(accounts_node, "account") {
        match account_from(w, node) {
            Ok(id) => {
                super::schedule_after_deserialization(w, id);
                loaded += 1;
            }
            Err(e) => warn!("Account instance load failed: {e}"),
        }
    }
    Ok(loaded)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::IpAddr;

    const XML: &str = r#"<?xml version="1.0"?>
<accounts>
  <account>
    <username>bob</username>
    <password>hunter2</password>
    <accessLevel>GameMaster</accessLevel>
    <flags>1</flags>
    <created>2010-05-06T07:08:09Z</created>
    <totalGold>12</totalGold>
    <addressList count="2"><ip>1.2.3.4</ip><ip>bad</ip><ip>5.6.7.8</ip></addressList>
    <comments><comment addedBy="admin" lastModified="2011-01-01T00:00:00Z">hello</comment></comments>
    <tags><tag name="Foo">bar</tag></tags>
    <totalGameTime>PT1H30M</totalGameTime>
  </account>
  <account>
    <username>old</username>
    <newSecureCryptPassword>AB-CD-EF</newSecureCryptPassword>
  </account>
  <account>
    <username>nopass</username>
  </account>
</accounts>"#;

    #[test]
    fn loads_accounts_from_the_old_xml() {
        let mut w = World::new();
        let dir = std::env::temp_dir().join(format!("muo-xml-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("accounts.xml");
        std::fs::write(&path, XML).unwrap();
        assert_eq!(deserialize_xml(&mut w, &path).unwrap(), 2);

        let bob = super::super::get_account(&w, "bob").unwrap();
        let a = super::super::account(&w, bob).unwrap();
        assert_eq!(a.access_level, AccessLevel::GameMaster);
        assert_eq!(a.flags, 1);
        assert_eq!(a.total_gold, 12);
        assert_eq!(
            a.created,
            DateTime::from_ymd_hms(2010, 5, 6, 7, 8, 9).unwrap()
        );
        // a plain password was rehashed with the configured algorithm
        assert_ne!(a.password.as_deref(), Some("hunter2"));
        assert_eq!(
            a.login_ips,
            [
                "1.2.3.4".parse::<IpAddr>().unwrap(),
                "5.6.7.8".parse().unwrap()
            ]
        );
        assert_eq!(
            (a.comments.len(), a.comments[0].added_by.as_str()),
            (1, "admin")
        );
        assert_eq!(a.tags[0].name, "Foo");
        assert_eq!(a.total_game_time, TimeSpan::from_mins(90));
        assert_eq!(a.mobiles.len(), 7);

        let old = super::super::get_account(&w, "old").unwrap();
        let o = super::super::account(&w, old).unwrap();
        assert_eq!(o.password_algorithm, PasswordProtectionAlgorithm::Sha2);
        assert_eq!(o.password.as_deref(), Some("ABCDEF"));
        assert!(super::super::get_account(&w, "nopass").is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn durations_parse_like_xmlconvert() {
        assert_eq!(parse_duration("PT1H30M"), Some(TimeSpan::from_mins(90)));
        assert_eq!(parse_duration("P1DT2H"), Some(TimeSpan::from_hours(26)));
        assert_eq!(parse_duration("-PT5S"), Some(-TimeSpan::from_secs(5)));
        assert_eq!(parse_duration("PT1.5S"), Some(TimeSpan::from_millis(1500)));
        assert_eq!(parse_duration("garbage"), None);
        assert_eq!(parse_duration("P"), None);
    }
}
