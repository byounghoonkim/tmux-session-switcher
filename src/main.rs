use regex::Regex;
use std::process::Command;

fn main() {
    // Get the current tmux session
    let current_session = Command::new("tmux")
        .args(["display-message", "-p", "#S"])
        .output()
        .expect("Failed to execute tmux command")
        .stdout;
    let current_session = String::from_utf8_lossy(&current_session).trim().to_string();

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
            windows.push((
                captures[1].to_string(),
                captures[2].to_string(),
                captures[3].to_string(),
                captures[4].to_string(),
                captures[5].to_string(),
                captures[6].to_string(),
            ));
        }
    }

    // Build window lists
    let mut windows_lists = String::new();
    let mut recent_windows = String::new();
    let mut active_window = String::new();

    for window in &windows {
        let mut windows_list = format!("{} - {} - {}", window.0, window.1, window.2);

        if window.0 == current_session && window.3 == "1" {
            windows_list.push_str(" 🟢");
        } else if window.4 == "1" {
            windows_list.push_str("  ⃝");
        }
        if window.5 == "1" {
            windows_list.push_str(" ♥️");
        }
        windows_list.push('\n');

        if window.0 == current_session && window.3 == "1" {
            active_window = windows_list.clone();
        } else if window.4 == "1" || window.5 == "1" {
            recent_windows.push_str(&windows_list);
        } else {
            windows_lists.push_str(&windows_list);
        }
    }

    windows_lists = format!("{}{}{}", active_window, recent_windows, windows_lists);

    // Use fzf-tmux to select a window
    let select_window = Command::new("sh")
        .arg("-c")
        .arg(format!(
            "echo '{}' | fzf-tmux -p 80,36 --border-label ' Select window ' --prompt '⚡' --bind 'tab:down,btab:up'",
            windows_lists
        ))
        .output()
        .expect("Failed to execute fzf-tmux")
        .stdout;
    let select_window = String::from_utf8_lossy(&select_window).trim().to_string();

    if select_window.is_empty() {
        return;
    }

    // Parse the selected window
    let re_select = Regex::new(r"([\w\-]+)\s-\s(\d+)\s-").unwrap();
    if let Some(captures) = re_select.captures(&select_window) {
        let session_name = &captures[1];
        let window_index = &captures[2];

        // Switch to the selected window
        Command::new("tmux")
            .args([
                "switch",
                "-t",
                &format!("{}:{}", session_name, window_index),
            ])
            .status()
            .expect("Failed to execute tmux switch");
    }
}
