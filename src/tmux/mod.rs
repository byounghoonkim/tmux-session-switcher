use std::fmt::Display;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use regex::Regex;
use serde_json;

const TMUX: &str = "tmux";

pub mod favorite;
pub mod previous;
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
        "#{window_marked_flag}|"
    );

    let all_windows = Command::new(TMUX)
        .args(["list-windows", "-a", "-F", fields])
        .output()
        .expect("Failed to execute tmux command")
        .stdout;

    let all_windows = String::from_utf8_lossy(&all_windows);

    let mut windows = Vec::new();
    let re = Regex::new(r"([^|]+)\|([^|]+)\|([^|]+)\|([^|]+)\|([^|]+)").unwrap();
    for line in all_windows.lines() {
        if let Some(captures) = re.captures(line) {
            windows.push(window::Window {
                session_name: captures[1].to_string(),
                index: captures[2].to_string(),
                name: captures[3].to_string(),
                active: &captures[4] == "1" && &captures[1] == current_session,
                marked: &captures[5] == "1",
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

fn get_previous_window_path() -> PathBuf {
    let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push(".config");
    path.push("tmux-session-switcher");

    if !path.exists() {
        fs::create_dir_all(&path).expect("Failed to create config directory");
    }

    path.push("previous_window.json");
    path
}

pub(crate) fn save_previous_window(session_name: &str, index: &str, name: &str) {
    let previous_window = previous::PreviousWindow {
        session_name: session_name.to_string(),
        index: index.to_string(),
        name: name.to_string(),
    };

    let path = get_previous_window_path();
    let json = serde_json::to_string_pretty(&previous_window)
        .expect("Failed to serialize previous window");

    fs::write(path, json).expect("Failed to write previous window file");
}

pub(crate) fn load_previous_window() -> Option<previous::PreviousWindow> {
    let path = get_previous_window_path();

    if !path.exists() {
        return None;
    }

    let contents = fs::read_to_string(path).ok()?;
    serde_json::from_str(&contents).ok()
}
