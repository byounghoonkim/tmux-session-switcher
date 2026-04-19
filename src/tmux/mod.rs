use std::fmt::Display;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::OnceLock;

use regex::Regex;

const TMUX: &str = "tmux";

pub mod favorite;
pub mod previous;
pub mod window;

pub(crate) trait Item: Display + SortPriority + Switchable {
    fn session_name(&self) -> String;
    fn index(&self) -> String;
    fn name(&self) -> String;
}

pub(crate) trait SortPriority {
    fn sort_priority(&self) -> f32;
}

pub(crate) trait Switchable {
    fn switch_window(&self);
}

static WINDOW_RE: OnceLock<Regex> = OnceLock::new();

fn run_command(args: &[&str]) -> Result<String, String> {
    let output = Command::new(TMUX)
        .args(args)
        .output()
        .map_err(|e| format!("Failed to run tmux {}: {}", args.first().unwrap_or(&""), e))?;
    if !output.status.success() {
        return Err(format!(
            "tmux {} exited with status {}",
            args.first().unwrap_or(&""),
            output.status
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub(crate) fn format_window_base(session: &str, index: &str, name: &str) -> String {
    format!("{:15} - {:>3} - {}", session, index, name)
}

pub(crate) fn get_running_windows(current_session: &str) -> Result<Vec<window::Window>, String> {
    let fields = concat!(
        "#{session_name}|",
        "#{window_index}|",
        "#{window_name}|",
        "#{window_active}|",
        "#{window_marked_flag}|",
        "#{window_bell_flag}|"
    );

    let raw = run_command(&["list-windows", "-a", "-F", fields])?;

    let re = WINDOW_RE.get_or_init(|| {
        Regex::new(r"([^|]+)\|([^|]+)\|([^|]+)\|([^|]+)\|([^|]+)\|([^|]+)").unwrap()
    });

    let mut windows = Vec::new();
    for line in raw.lines() {
        if let Some(captures) = re.captures(line) {
            windows.push(window::Window {
                session_name: captures[1].to_string(),
                index: captures[2].to_string(),
                name: captures[3].to_string(),
                active: &captures[4] == "1" && &captures[1] == current_session,
                marked: &captures[5] == "1",
                bell: &captures[6] == "1",
            });
        }
    }

    Ok(windows)
}

pub(crate) fn get_current_session() -> String {
    let current_session = Command::new(TMUX)
        .args(["display-message", "-p", "#S"])
        .output()
        .expect("Failed to execute tmux command")
        .stdout;
    String::from_utf8_lossy(&current_session).trim().to_string()
}

pub(crate) fn get_current_window() -> (String, String, String, String) {
    let fields = "#{session_name}|#{window_index}|#{window_name}|#{pane_current_path}";
    let output = Command::new(TMUX)
        .args(["display-message", "-p", "-F", fields])
        .output()
        .expect("Failed to execute tmux command")
        .stdout;
    let output = String::from_utf8_lossy(&output).trim().to_string();
    let parts: Vec<&str> = output.splitn(4, '|').collect();
    (
        parts.first().unwrap_or(&"").to_string(),
        parts.get(1).unwrap_or(&"").to_string(),
        parts.get(2).unwrap_or(&"").to_string(),
        parts.get(3).unwrap_or(&"").to_string(),
    )
}

pub(crate) fn create_new_window(current_session: &str, title: &str) {
    Command::new(TMUX)
        .args(["new-window", "-t", current_session, "-n", title])
        .status()
        .expect("Failed to create new window");
}

fn get_previous_window_path() -> PathBuf {
    let mut path = crate::utils::get_config_dir();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_running_windows_returns_result() {
        // Compile-time check: the return type must be Result<Vec<window::Window>, String>.
        let _: fn(&str) -> Result<Vec<window::Window>, String> = get_running_windows;
    }

    #[test]
    fn test_format_window_base_pads_session() {
        let result = format_window_base("main", "3", "editor");
        // "main" = 4 chars, padded to 15 = 11 trailing spaces, then " - ", then "3" right-aligned in 3 = "  3"
        assert_eq!(result, "main            -   3 - editor");
    }

    #[test]
    fn test_format_window_base_long_session() {
        let result = format_window_base("verylongsessionname", "10", "term");
        // long session overflows :15 padding, index "10" gets 1 leading space in :>3
        assert_eq!(result, "verylongsessionname -  10 - term");
    }
}
