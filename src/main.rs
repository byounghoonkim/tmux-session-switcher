use std::cmp::Ordering::{Greater, Less};
use std::process::Command;

use regex::Regex;

fn main() {
    let current_session = get_current_session();
    let mut windows = get_all_windows(&current_session);
    sort_windows(&mut windows);
    if let Some(sw) = select_window(&windows) {
        switch_window(sw);
    }
}

fn sort_windows(windows: &mut [Window]) {
    // Sort windows by active, marked, last and others
    windows.sort_by(|a, b| {
        if a.actvie && !b.actvie {
            return Less;
        } else if !a.actvie && b.actvie {
            return Greater;
        }
        if a.marked && !b.marked {
            return Less;
        } else if !a.marked && b.marked {
            return Greater;
        }
        if a.last_flag && !b.last_flag {
            return Less;
        } else if !a.last_flag && b.last_flag {
            return Greater;
        }
        std::cmp::Ordering::Equal
    });
}

fn select_window(windows: &[Window]) -> Option<&Window> {
    let select_result = Command::new("sh")
        .arg("-c")
        .arg(format!(
            "echo '{}' | fzf-tmux -p 80,36 --border-label ' Select window ' --prompt '⚡' --bind 'tab:down,btab:up'",
            windows
                .iter()
                .map(|w| w.to_string())
                .collect::<String>()
        ))
        .output()
        .expect("Failed to execute fzf-tmux")
        .stdout;
    let select_result = String::from_utf8_lossy(&select_result).trim().to_string();
    if select_result.is_empty() {
        return None;
    }
    let selected_window = windows
        .iter()
        .find(|w| w.to_string().trim() == select_result)
        .expect("Selected window not found");

    Some(selected_window)
}

fn switch_window(selected_window: &Window) {
    // Switch to the selected window
    Command::new("tmux")
        .args([
            "switch",
            "-t",
            &format!("{}:{}", selected_window.session_name, selected_window.index,),
        ])
        .status()
        .expect("Failed to execute tmux switch");
}

struct Window {
    session_name: String,
    index: String,
    name: String,
    actvie: bool,
    last_flag: bool,
    marked: bool,
}

//convert from trait Window to string
impl std::fmt::Display for Window {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{:15} - {:3} - {}{}{}{}",
            self.session_name,
            self.index,
            self.name,
            if self.actvie { " 🟢" } else { "" },
            if self.last_flag { "  ⃝" } else { "" },
            if self.marked { " ♥️" } else { "" },
        )
    }
}

fn get_all_windows(current_session: &str) -> Vec<Window> {
    // Get all tmux windows
    let all_windows = Command::new("tmux")
        .args([
            "list-windows",
            "-a",
            "-F",
            "#{session_name}|#{window_index}|#{window_name}|#{window_active}|#{window_last_flag}|#{window_marked_flag}",
        ])
        .output()
        .expect("Failed to execute tmux command")
        .stdout;

    let all_windows = String::from_utf8_lossy(&all_windows);

    // Parse windows
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

fn get_current_session() -> String {
    let current_session = Command::new("tmux")
        .args(["display-message", "-p", "#S"])
        .output()
        .expect("Failed to execute tmux command")
        .stdout;
    let current_session = String::from_utf8_lossy(&current_session).trim().to_string();
    current_session
}
