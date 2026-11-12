//! The first-boot console questions (server and expansion). Input and output go through [`PromptIo`] so tests can
//! script them.
//!
//! // shortcut: no console colors (`Utility.PushColor`); a `PromptIo` that wants them can add them in `write`.

use crate::expansion::{Expansion, ExpansionTable, MapSelectionFlags};
use std::io::{BufRead, Write};
use std::net::SocketAddr;
use std::path::Path;

pub trait PromptIo {
    /// One line of input without its newline; `None` at end of input (`ConsoleInputHandler.ReadLine()` null).
    fn read_line(&mut self) -> Option<String>;
    fn write(&mut self, text: &str);

    fn write_line(&mut self, text: &str) {
        self.write(text);
        self.write("\n");
    }
}

/// stdin and stdout.
pub struct StdioPrompt;

impl PromptIo for StdioPrompt {
    fn read_line(&mut self) -> Option<String> {
        let mut line = String::new();
        match std::io::stdin().lock().read_line(&mut line) {
            Ok(0) | Err(_) => None,
            Ok(_) => Some(line.trim_end_matches(['\r', '\n']).to_string()),
        }
    }

    fn write(&mut self, text: &str) {
        let mut out = std::io::stdout();
        let _ = out.write_all(text.as_bytes());
        let _ = out.flush();
    }
}

fn blank(s: &Option<String>) -> bool {
    s.as_deref().is_none_or(|s| s.trim().is_empty())
}

/// `GetDataDirectories`: existing directories until a blank line.
pub fn get_data_directories(io: &mut dyn PromptIo) -> Vec<String> {
    io.write_line("Please enter the absolute path to your ClassicUO or Ultima Online data:");
    let mut dirs = Vec::new();
    loop {
        io.write(&format!(
            "{}> ",
            if dirs.is_empty() {
                " "
            } else {
                "[enter to finish]"
            }
        ));
        let line = io.read_line();
        if blank(&line) {
            break;
        }
        let dir = line.unwrap_or_default();
        if Path::new(&dir).is_dir() {
            io.write_line(&format!("Added {dir}."));
            dirs.push(dir);
        } else {
            io.write_line(&format!("{dir} does not exist."));
        }
    }
    dirs
}

/// `GetListeners`: `ip[:port]` entries (port 2593 by default) until a blank line; nothing entered is `0.0.0.0:2593`.
pub fn get_listeners(io: &mut dyn PromptIo) -> Vec<SocketAddr> {
    io.write_line("Please enter the IP and ports to listen:");
    io.write_line(" - Only enter IP addresses directly bound to this machine");
    io.write_line(" - To listen to all IP addresses enter 0.0.0.0");
    let mut ips: Vec<SocketAddr> = Vec::new();
    loop {
        io.write(&format!(
            "[{}]> ",
            if ips.is_empty() {
                "0.0.0.0:2593"
            } else {
                "enter to finish"
            }
        ));
        let line = io.read_line();
        let ip = if blank(&line) {
            if !ips.is_empty() {
                break;
            }
            SocketAddr::from(([0, 0, 0, 0], 2593))
        } else {
            let mut text = line.unwrap_or_default();
            if !text.contains(':') {
                text.push_str(":2593");
            }
            match text.parse() {
                Ok(ip) => ip,
                Err(_) => {
                    io.write_line(&format!("{text} is not a valid IP or port."));
                    continue;
                }
            }
        };
        io.write_line(&format!("Added {ip}."));
        ips.push(ip);
    }
    ips
}

/// `GetServerName`: blank is `muo`.
pub fn get_server_name(io: &mut dyn PromptIo) -> String {
    io.write_line("Please enter the name of your shard:");
    io.write("[muo]> ");
    let line = io.read_line();
    let name = if blank(&line) {
        "muo".to_string()
    } else {
        line.unwrap_or_default()
    };
    io.write_line(&format!("Server name set to {name}."));
    name
}

/// `GetExpansion`: a number, a name, or blank for the newest.
pub fn get_expansion(io: &mut dyn PromptIo, table: &ExpansionTable) -> Expansion {
    for (i, info) in table.table.iter().enumerate() {
        let name = Expansion::from_index(info.id)
            .map_or_else(|| info.id.to_string(), |e| e.name().to_string());
        io.write_line(&format!(" - {i:>2}: {name} ({})", info.name));
    }
    let max = table
        .table
        .last()
        .and_then(|i| Expansion::from_index(i.id))
        .unwrap_or_default();
    loop {
        io.write(&format!("[enter for {max}]> "));
        let line = io.read_line();
        let expansion = if blank(&line) {
            max
        } else {
            let input = line.unwrap_or_default();
            let numbered = input
                .trim()
                .parse::<i32>()
                .ok()
                .filter(|n| *n >= 0 && (*n as usize) < table.table.len());
            match numbered
                .and_then(Expansion::from_index)
                .or_else(|| Expansion::parse(&input))
            {
                Some(e) => e,
                None => {
                    io.write_line(&format!("{input} is an invalid expansion option."));
                    continue;
                }
            }
        };
        io.write_line(&format!(
            "Expansion set to {}.",
            table.get_info(expansion.index()).name
        ));
        return expansion;
    }
}

/// `OutputSelectedMaps`.
pub fn output_selected_maps(
    io: &mut dyn PromptIo,
    table: &ExpansionTable,
    expansion: Expansion,
    selected: MapSelectionFlags,
) {
    io.write_line("Selected maps:");
    let mut i = 0;
    for flag in MapSelectionFlags::from_expansion(table, expansion) {
        io.write_line(&format!(
            "{}. {} [{}]",
            i + 1,
            MapSelectionFlags::name_of(flag),
            if selected.includes(flag) { "*" } else { "" }
        ));
        i += 1;
    }
    io.write_line("");
    io.write_line(&format!("[1-{i} and enter to toggle, or enter to finish]"));
}

/// `GetSelectedMaps`: toggles the expansion's maps by number until a blank line.
pub fn get_selected_maps(
    io: &mut dyn PromptIo,
    table: &ExpansionTable,
    expansion: Expansion,
) -> MapSelectionFlags {
    let expansion_maps = table.get_info(expansion.index()).map_selection_flags;
    let mut selected = expansion_maps;
    loop {
        output_selected_maps(io, table, expansion, selected);
        let line = io.read_line().map(|l| l.trim_end().to_string());
        if blank(&line) {
            break;
        }
        let Ok(n) = line.unwrap_or_default().parse::<i32>() else {
            io.write_line("You need to choose a number, or press ENTER on its own to accept");
            continue;
        };
        if !(1..=31).contains(&n) {
            io.write_line("That number was not an option. Please try again...");
            continue;
        }
        let flag = MapSelectionFlags::from_bits_retain(1 << (n - 1));
        if !expansion_maps.includes(flag) {
            io.write_line("That number was not an option. Please try again...");
            continue;
        }
        selected.toggle(flag);
    }
    io.write_line("These maps will be populated and moongates will not lead to other maps: ");
    io.write_line(&selected.to_comma_delimited_string());
    io.write_line(&format!(
        "To change the selected maps, modify {}.",
        crate::expansion::EXPANSION_CONFIGURATION_PATH
    ));
    selected
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::expansion::ExpansionInfo;
    use std::collections::VecDeque;

    #[derive(Default)]
    pub(crate) struct Script {
        pub input: VecDeque<String>,
        pub output: String,
    }

    impl Script {
        pub(crate) fn new(lines: &[&str]) -> Script {
            Script {
                input: lines.iter().map(|s| s.to_string()).collect(),
                output: String::new(),
            }
        }
    }

    impl PromptIo for Script {
        fn read_line(&mut self) -> Option<String> {
            self.input.pop_front()
        }

        fn write(&mut self, text: &str) {
            self.output.push_str(text);
        }
    }

    pub(crate) fn table() -> ExpansionTable {
        ExpansionTable {
            table: (0..12)
                .map(|i| ExpansionInfo {
                    id: i,
                    name: format!("Expansion {i}"),
                    map_selection_flags: if i >= 2 {
                        MapSelectionFlags::FELUCCA | MapSelectionFlags::TRAMMEL
                    } else {
                        MapSelectionFlags::FELUCCA
                    },
                    ..Default::default()
                })
                .collect(),
        }
    }

    #[test]
    fn listeners() {
        let mut io = Script::new(&[""]);
        assert_eq!(
            get_listeners(&mut io),
            vec!["0.0.0.0:2593".parse().unwrap()]
        );
        let mut io = Script::new(&["bad ip", "127.0.0.1", "[::1]:9", ""]);
        assert_eq!(
            get_listeners(&mut io),
            vec![
                "127.0.0.1:2593".parse().unwrap(),
                "[::1]:9".parse().unwrap()
            ]
        );
        assert!(io.output.contains("bad ip:2593 is not a valid IP or port."));
    }

    #[test]
    fn data_directories_and_server_name() {
        let here = std::env::temp_dir().to_string_lossy().to_string();
        let mut io = Script::new(&["/definitely/not/here", &here, ""]);
        assert_eq!(get_data_directories(&mut io), vec![here]);
        assert_eq!(get_server_name(&mut Script::new(&["  "])), "muo");
        assert_eq!(get_server_name(&mut Script::new(&["My Shard"])), "My Shard");
        assert_eq!(get_server_name(&mut Script::default()), "muo");
    }

    #[test]
    fn expansion_choice() {
        let t = table();
        assert_eq!(get_expansion(&mut Script::new(&[""]), &t), Expansion::EJ);
        assert_eq!(get_expansion(&mut Script::new(&["7"]), &t), Expansion::ML);
        assert_eq!(
            get_expansion(&mut Script::new(&["12", "nope", "SE"]), &t),
            Expansion::SE
        );
    }

    #[test]
    fn map_selection_toggles() {
        let t = table();
        // ML offers Felucca and Trammel: toggle Trammel off, ask for Malas (not offered) and garbage, accept
        let mut io = Script::new(&["2", "4", "x", "99", ""]);
        let got = get_selected_maps(&mut io, &t, Expansion::ML);
        assert_eq!(got, MapSelectionFlags::FELUCCA);
        assert!(io.output.contains("2. Trammel [*]") && io.output.contains("2. Trammel []"));
        assert!(io.output.contains("You need to choose a number"));
    }
}
