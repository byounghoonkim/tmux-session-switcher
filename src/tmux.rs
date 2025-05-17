use regex::Regex;

use std::process::Command;

const TMUX: &str = "tmux";

pub(crate) struct Window {
    pub(crate) session_name: String,
    pub(crate) index: String,
    pub(crate) name: String,
    pub(crate) actvie: bool,
    pub(crate) last_flag: bool,
    pub(crate) marked: bool,
}

pub(crate) fn get_all_windows(current_session: &str) -> Vec<Window> {
    let fields = concat!(
        "#{session_name}|",
        "#{window_index}|",
        "#{window_name}|",
        "#{window_active}|",
        "#{window_last_flag}|",
        "#{window_marked_flag}|"
    );

    let all_windows = Command::new(TMUX)
        .args(["list-windows", "-a", "-F", fields])
        .output()
        .expect("Failed to execute tmux command")
        .stdout;

    let all_windows = String::from_utf8_lossy(&all_windows);

    let mut windows = Vec::new();
    let re = Regex::new(r"([^|]+)\|([^|]+)\|([^|]+)\|([^|]+)\|([^|]+)\|([^|]+)").unwrap();
    for line in all_windows.lines() {
        if let Some(captures) = re.captures(line) {
            windows.push(Window {
                session_name: captures[1].to_string(),
                index: captures[2].to_string(),
                name: captures[3].to_string(),
                actvie: &captures[4] == "1" && &captures[1] == current_session,
                last_flag: &captures[5] == "1",
                marked: &captures[6] == "1",
            });
        }
    }

    windows
}

pub(crate) fn switch_window(selected_window: &Window) {
    Command::new(TMUX)
        .args([
            "switch",
            "-t",
            &format!("{}:{}", selected_window.session_name, selected_window.index,),
        ])
        .status()
        .expect("Failed to execute tmux switch");
}

pub(crate) fn get_current_session() -> String {
    let current_session = Command::new(TMUX)
        .args(["display-message", "-p", "#S"])
        .output()
        .expect("Failed to execute tmux command")
        .stdout;
    String::from_utf8_lossy(&current_session).trim().to_string()
}
