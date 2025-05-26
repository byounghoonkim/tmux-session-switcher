use std::fmt::Display;
use std::process::Command;

use regex::Regex;

const TMUX: &str = "tmux";

pub mod favorite;
pub mod window;

pub(crate) trait Item: Display + SortPriority + Switchable {}

pub(crate) trait SortPriority {
    fn sort_priority(&self) -> f32;
}

pub(crate) trait Switchable {
    fn switch_window(&self);
}

pub(crate) fn get_running_windows(current_session: &str) -> Vec<window::Window> {
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
            windows.push(window::Window {
                session_name: captures[1].to_string(),
                index: captures[2].to_string(),
                name: captures[3].to_string(),
                active: &captures[4] == "1" && &captures[1] == current_session,
                last_flag: &captures[5] == "1",
                marked: &captures[6] == "1",
            });
        }
    }

    windows
}

pub(crate) fn get_current_session() -> String {
    let current_session = Command::new(TMUX)
        .args(["display-message", "-p", "#S"])
        .output()
        .expect("Failed to execute tmux command")
        .stdout;
    String::from_utf8_lossy(&current_session).trim().to_string()
}

pub(crate) fn create_new_window(current_session: &str, title: &str) {
    Command::new(TMUX)
        .args(["new-window", "-t", current_session, "-n", title])
        .status()
        .expect("Failed to create new window");
}
