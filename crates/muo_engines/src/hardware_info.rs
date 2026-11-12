//! Hardware info: packet 0xD9 (the client's hardware report, kept on the account) and the `[HWInfo`
//! command that shows it to a game master.
//!
//! The report opens as a properties gump over its `HardwareInfo` table, hosted by the targeted player.

use std::rc::Rc;

use muo_content::accounting::{self, HardwareInfo};
use muo_content::props::{self, Kind, PropDef, Value};
use muo_io::{Result, SpanReader};
use muo_net::net_state::NetState;
use muo_primitives::DateTime;
use muo_server::World;
use muo_server::commands::{CommandEventArgs, CommandSystem};
use muo_server::ids::{EntityId, MobileId};
use muo_server::mobile::AccessLevel;
use muo_server::target::{self, TargetFlags, Targeted};

/// The report of the account of the mobile `e` (`HardwareInfo` is no entity: its table reads its host, the player).
fn info_of(w: &World, e: EntityId) -> Option<&HardwareInfo> {
    let EntityId::Mobile(m) = e else { return None };
    accounting::account(w, w.mobile(m)?.account?)?
        .hardware_info
        .as_ref()
}

fn info_mut(w: &mut World, e: EntityId) -> std::result::Result<&mut HardwareInfo, String> {
    let EntityId::Mobile(m) = e else {
        return Err("That is not a mobile.".into());
    };
    let id = w
        .mobile(m)
        .and_then(|m| m.account)
        .ok_or("That mobile has no account.")?;
    accounting::account_mut(w, id)
        .and_then(|a| a.hardware_info.as_mut())
        .ok_or_else(|| "That account has no hardware information.".to_string())
}

/// A `[CommandProperty(AccessLevel.GameMaster)] { get; private set; }` member (reflection still writes it).
macro_rules! hw_prop {
    ($name:literal, int, $f:ident) => {
        PropDef {
            name: $name,
            kind: props::kinds::INT,
            read: AccessLevel::GameMaster,
            write: AccessLevel::GameMaster,
            get: |w, e| info_of(w, e).map_or(Value::Null, |i| Value::Int(i64::from(i.$f))),
            set: Some(|w, e, v| info_mut(w, e).map(|i| i.$f = v.i32())),
        }
    };
    ($name:literal, str, $f:ident) => {
        PropDef {
            name: $name,
            kind: props::kinds::STR,
            read: AccessLevel::GameMaster,
            write: AccessLevel::GameMaster,
            get: |w, e| info_of(w, e).map_or(Value::Null, |i| Value::Str(i.$f.clone())),
            set: Some(|w, e, v| info_mut(w, e).map(|i| i.$f = v.string().unwrap_or_default())),
        }
    };
}

/// `HardwareInfo`'s `[CommandProperty]` members.
static HARDWARE_INFO: &[PropDef] = &[
    hw_prop!("CpuModel", int, cpu_model),
    hw_prop!("CpuClockSpeed", int, cpu_clock_speed),
    hw_prop!("CpuQuantity", int, cpu_quantity),
    hw_prop!("OSMajor", int, os_major),
    hw_prop!("OSMinor", int, os_minor),
    hw_prop!("OSRevision", int, os_revision),
    hw_prop!("InstanceID", int, instance_id),
    hw_prop!("ScreenWidth", int, screen_width),
    hw_prop!("ScreenHeight", int, screen_height),
    hw_prop!("ScreenDepth", int, screen_depth),
    hw_prop!("PhysicalMemory", int, physical_memory),
    hw_prop!("CpuManufacturer", int, cpu_manufacturer),
    hw_prop!("CpuFamily", int, cpu_family),
    hw_prop!("VCVendorID", int, vc_vendor_id),
    hw_prop!("VCDeviceID", int, vc_device_id),
    hw_prop!("VCMemory", int, vc_memory),
    hw_prop!("DXMajor", int, dx_major),
    hw_prop!("DXMinor", int, dx_minor),
    hw_prop!("VCDescription", str, vc_description),
    hw_prop!("Language", str, language),
    hw_prop!("Distribution", int, distribution),
    hw_prop!("ClientsRunning", int, clients_running),
    hw_prop!("ClientsInstalled", int, clients_installed),
    hw_prop!("PartialInstalled", int, partial_installed),
    hw_prop!("Unknown", str, unknown),
    PropDef {
        name: "TimeReceived",
        kind: Kind::DateTime,
        read: AccessLevel::GameMaster,
        write: AccessLevel::GameMaster,
        get: |w, e| info_of(w, e).map_or(Value::Null, |i| Value::DateTime(i.time_received)),
        set: Some(|w, e, v| match v {
            Value::DateTime(t) => info_mut(w, e).map(|i| i.time_received = t),
            _ => Ok(()),
        }),
    },
];

/// `HardwareInfo.Configure`: the packet and `[HWInfo`.
pub fn configure(w: &mut World) {
    muo_content::network::register_packet(w, 0xD9, 0x10C, false, false, on_receive);
    CommandSystem::register(
        w,
        "HWInfo",
        AccessLevel::GameMaster,
        &[],
        Rc::new(hw_info_on_command),
    );
}

/// The body of `HardwareInfo.OnReceive` after the packet id.
pub fn parse(body: &[u8], now: DateTime) -> Result<HardwareInfo> {
    let mut r = SpanReader::new(body);
    r.read_byte()?; // 1: <4.0.1a, 2>=4.0.1a
    Ok(HardwareInfo {
        instance_id: r.read_i32()?,
        os_major: r.read_i32()?,
        os_minor: r.read_i32()?,
        os_revision: r.read_i32()?,
        cpu_manufacturer: i32::from(r.read_byte()?),
        cpu_family: r.read_i32()?,
        cpu_model: r.read_i32()?,
        cpu_clock_speed: r.read_i32()?,
        cpu_quantity: i32::from(r.read_byte()?),
        physical_memory: r.read_i32()?,
        screen_width: r.read_i32()?,
        screen_height: r.read_i32()?,
        screen_depth: r.read_i32()?,
        dx_major: i32::from(r.read_i16()?),
        dx_minor: i32::from(r.read_i16()?),
        vc_description: r.read_little_uni_safe(Some(64))?,
        vc_vendor_id: r.read_i32()?,
        vc_device_id: r.read_i32()?,
        vc_memory: r.read_i32()?,
        distribution: i32::from(r.read_byte()?),
        clients_running: i32::from(r.read_byte()?),
        clients_installed: i32::from(r.read_byte()?),
        partial_installed: i32::from(r.read_byte()?),
        language: r.read_little_uni_safe(Some(4))?,
        unknown: r.read_ascii_safe(Some(64))?,
        time_received: now,
    })
}

/// `HardwareInfo.OnReceive`: `acct.HardwareInfo = info` for the connection's account.
fn on_receive(w: &mut World, ns: &mut NetState, body: &[u8]) {
    let Ok(info) = parse(body, w.now) else { return };
    let Some(id) = ns
        .account
        .as_deref()
        .and_then(|name| accounting::accounts(w).find(name))
    else {
        return;
    };
    if let Some(a) = accounting::account_mut(w, id) {
        a.hardware_info = Some(info);
    }
}

/// `HWInfo_OnCommand`.
fn hw_info_on_command(w: &mut World, e: &mut CommandEventArgs) {
    begin(w, e.mobile);
    muo_content::mx::send_message(
        w,
        e.mobile,
        "Target a player to view their hardware information.",
    );
}

fn begin(w: &mut World, from: MobileId) {
    target::begin_target(w, from, -1, false, TargetFlags::NONE, on_target);
}

/// `HWInfo_OnTarget`.
fn on_target(w: &mut World, from: MobileId, targeted: Targeted) {
    let player = match targeted {
        Targeted::Entity(EntityId::Mobile(m)) if muo_content::mx::player(w, m) => Some(m),
        _ => None,
    };
    let Some(m) = player else {
        begin(w, from);
        muo_content::mx::send_message(w, from, "That is not a player. Try again.");
        return;
    };
    let Some(acct) = w.mobile(m).and_then(|x| x.account) else {
        muo_content::mx::send_message(w, from, "No account has been attached to that player.");
        return;
    };
    let Some(_) = accounting::account(w, acct).and_then(|a| a.hardware_info.as_ref()) else {
        muo_content::mx::send_message(
            w,
            from,
            "No hardware information for that account was found.",
        );
        return;
    };
    let access = muo_content::mx::access_level(w, from);
    let line = format!(
        "{:?} {} viewing hardware info of {}",
        access,
        muo_commands::logging::format_entity(w, EntityId::Mobile(from)),
        muo_commands::logging::format_entity(w, EntityId::Mobile(m)),
    );
    muo_commands::logging::write_line(w, from, &line);
    let gump = muo_gumps::m_z::props::properties::new(
        w,
        from,
        muo_gumps::m_z::props::registry::SubObj::object(
            EntityId::Mobile(m),
            HARDWARE_INFO,
            "HardwareInfo",
        ),
    );
    muo_gumps::m_z::common::send(w, from, gump);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A 0xD9 body (after the id) built field by field.
    pub(super) fn body() -> Vec<u8> {
        let mut b = vec![2u8];
        for v in [7_i32, 10, 0, 19045] {
            b.extend(v.to_be_bytes()); // instance id, os major/minor/revision
        }
        b.push(1); // cpu manufacturer
        for v in [6_i32, 165, 3600] {
            b.extend(v.to_be_bytes()); // cpu family/model/clock
        }
        b.push(8); // cpu quantity
        for v in [16384_i32, 1920, 1080, 32] {
            b.extend(v.to_be_bytes()); // memory, screen
        }
        b.extend(12i16.to_be_bytes());
        b.extend(1i16.to_be_bytes());
        let mut desc = "GPU"
            .encode_utf16()
            .flat_map(u16::to_le_bytes)
            .collect::<Vec<_>>();
        desc.resize(128, 0);
        b.extend(desc);
        for v in [0x10DE_i32, 0x2204, 24576] {
            b.extend(v.to_be_bytes());
        }
        b.extend([3, 2, 1, 0]); // distribution, running, installed, partial
        let mut lang = "ENU"
            .encode_utf16()
            .flat_map(u16::to_le_bytes)
            .collect::<Vec<_>>();
        lang.resize(8, 0);
        b.extend(lang);
        let mut unknown = b"x".to_vec();
        unknown.resize(64, 0);
        b.extend(unknown);
        assert_eq!(b.len(), 0x10C - 1);
        b
    }

    #[test]
    fn parses_the_report() {
        let now = DateTime::MIN;
        let i = parse(&body(), now).unwrap();
        assert_eq!((i.instance_id, i.os_major, i.os_revision), (7, 10, 19045));
        assert_eq!(
            (
                i.cpu_manufacturer,
                i.cpu_family,
                i.cpu_clock_speed,
                i.cpu_quantity
            ),
            (1, 6, 3600, 8)
        );
        assert_eq!(
            (
                i.physical_memory,
                i.screen_width,
                i.screen_height,
                i.screen_depth
            ),
            (16384, 1920, 1080, 32)
        );
        assert_eq!((i.dx_major, i.dx_minor), (12, 1));
        assert_eq!(
            (i.vc_description.as_str(), i.vc_vendor_id, i.vc_memory),
            ("GPU", 0x10DE, 24576)
        );
        assert_eq!(
            (
                i.distribution,
                i.clients_running,
                i.clients_installed,
                i.partial_installed
            ),
            (3, 2, 1, 0)
        );
        assert_eq!((i.language.as_str(), i.unknown.as_str()), ("ENU", "x"));
        assert!(parse(&body()[..100], now).is_err());
    }
}

#[cfg(test)]
mod props_tests {
    use super::*;

    /// `[HWInfo` gump members read and write the report of the player's account.
    #[test]
    fn hardware_info_table_reads_and_writes_the_players_account() {
        let mut w = World::new();
        accounting::initialize(&mut w);
        let acct = accounting::create_account(&mut w, "hwtest", "pw");
        let m = w.new_mobile(&muo_server::mobile::MOBILE_BASE);
        w.mobile_mut(m).unwrap().account = Some(acct);
        let e = EntityId::Mobile(m);
        let get = |w: &mut World, n: &str| {
            (HARDWARE_INFO.iter().find(|d| d.name == n).unwrap().get)(w, e)
        };
        assert_eq!(get(&mut w, "OSMajor"), Value::Null); // no report yet
        let mut i = parse(&super::tests::body(), DateTime::MIN).unwrap();
        i.vc_description = "GPU".into();
        accounting::account_mut(&mut w, acct).unwrap().hardware_info = Some(i);
        assert_eq!(get(&mut w, "OSMajor"), Value::Int(10));
        assert_eq!(get(&mut w, "VCDescription"), Value::Str("GPU".into()));
        let set = HARDWARE_INFO
            .iter()
            .find(|d| d.name == "ScreenWidth")
            .unwrap()
            .set
            .unwrap();
        set(&mut w, e, Value::Int(800)).unwrap();
        assert_eq!(get(&mut w, "ScreenWidth"), Value::Int(800));
    }
}
