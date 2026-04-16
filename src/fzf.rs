use std::cmp::Ordering::Greater;
use std::cmp::Ordering::Less;
use std::fmt::Display;
use std::io::Write;
use std::process::Command;

use super::tmux::SortPriority;
use crate::picker::PickerConfig;

fn get_terminal_width() -> u16 {
    if let Some((terminal_size::Width(width), _)) = terminal_size::terminal_size() {
        std::cmp::min(width, 80)
    } else {
        80
    }
}

fn shell_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

/// ratatui border 문자열을 tmux display-popup -b 옵션 값으로 변환
fn to_tmux_border(border: &str) -> &'static str {
    match border {
        "rounded" => "rounded",
        "double" => "double",
        "bold" => "heavy",
        "sharp" => "single",
        "none" => "none",
        _ => "single",
    }
}

pub(crate) fn sort_by_priority<T: SortPriority + ?Sized>(items: &mut [Box<T>]) {
    items.sort_by(|a, b| {
        if a.sort_priority() > b.sort_priority() {
            return Greater;
        } else if a.sort_priority() < b.sort_priority() {
            return Less;
        }
        std::cmp::Ordering::Equal
    });
}

pub(crate) enum SelectItemReturn<'a, T> {
    None,
    Item(&'a T),
    NewWindowTitle(String),
}

pub(crate) enum PickerOutput {
    Cancelled,
    Selected(usize),
    New(String),
}

/// 아이템 문자열 목록을 받아 tmux display-popup으로 TUI 피커를 실행하고 결과를 반환한다.
/// tmux 세션 외부에서 호출하면 panic한다.
pub(crate) fn invoke_picker(
    item_strings: &[String],
    title: &str,
    border: &str,
    layout: &str,
    theme: &str,
) -> PickerOutput {
    let config = PickerConfig {
        items: item_strings.to_vec(),
        title: title.to_string(),
        border: border.to_string(),
        layout: layout.to_string(),
        theme: theme.to_string(),
    };

    // 아이템을 temp file에 직렬화
    let mut items_file = tempfile::NamedTempFile::new().expect("Failed to create items temp file");
    serde_json::to_writer(&items_file, &config).expect("Failed to serialize picker config");
    items_file.flush().expect("Failed to flush items temp file");
    let items_path = items_file.path().to_string_lossy().to_string();

    // 결과를 받을 temp file 생성 (inner process가 여기에 씀)
    let result_file = tempfile::NamedTempFile::new().expect("Failed to create result temp file");
    let result_path = result_file.path().to_string_lossy().to_string();

    // 현재 실행 파일 경로로 display-popup 실행
    let exe = std::env::current_exe().expect("Failed to get current executable path");
    let height = std::cmp::min(item_strings.len() + 6, 40);
    let width = get_terminal_width();
    let popup_cmd = format!(
        "{} internal-picker {} {}",
        shell_quote(&exe.to_string_lossy()),
        shell_quote(&items_path),
        shell_quote(&result_path),
    );

    Command::new("tmux")
        .args([
            "display-popup",
            "-EE",
            "-w",
            &width.to_string(),
            "-h",
            &height.to_string(),
            "-b",
            to_tmux_border(border),
            "-T",
            &format!(" {} ", title),
            &popup_cmd,
        ])
        .status()
        .expect("Failed to run tmux display-popup");

    // result file 읽기
    let raw = std::fs::read_to_string(result_file.path()).unwrap_or_default();
    let raw = raw.trim();

    if raw.is_empty() {
        return PickerOutput::Cancelled;
    }
    if let Some(title) = raw.strip_prefix("new:") {
        return PickerOutput::New(title.to_string());
    }
    if let Ok(idx) = raw.parse::<usize>() {
        return PickerOutput::Selected(idx);
    }
    PickerOutput::Cancelled
}

fn invoke_fzf(
    item_strings: &[String],
    title: &str,
    border: &str,
    layout: &str,
) -> PickerOutput {
    use std::process::Stdio;

    let height = std::cmp::min(item_strings.len() + 5, 40);
    let width = get_terminal_width();

    let input: String = item_strings.iter().cloned().collect();

    let mut child = Command::new("fzf")
        .args([
            "--tmux",
            &format!("{},{}", width, height),
            &format!("--layout={}", layout),
            &format!("--border={}", border),
            "--border-label",
            &format!(" {} ", title),
            "--prompt",
            "⚡",
            "--bind",
            "tab:down,btab:up",
            "--print-query",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn fzf");

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(input.as_bytes()).ok();
    }

    let output = child.wait_with_output().expect("Failed to wait on fzf");
    let result = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if result.is_empty() {
        return PickerOutput::Cancelled;
    }

    match result.split_once('\n') {
        Some((_, selected)) => {
            let selected = selected.trim();
            if let Some(idx) = item_strings.iter().position(|s| s.trim() == selected) {
                PickerOutput::Selected(idx)
            } else {
                PickerOutput::Cancelled
            }
        }
        None => {
            // Only query line — unmatched query, user wants new window
            let query = result.trim();
            if !query.is_empty() {
                PickerOutput::New(query.to_string())
            } else {
                PickerOutput::Cancelled
            }
        }
    }
}

pub(crate) fn dispatch_picker(
    item_strings: &[String],
    title: &str,
    border: &str,
    layout: &str,
    use_fzf: bool,
    theme: &str,
) -> PickerOutput {
    if use_fzf {
        invoke_fzf(item_strings, title, border, layout)
    } else {
        invoke_picker(item_strings, title, border, layout, theme)
    }
}

pub(crate) fn select_item<'a, T: Display + ?Sized>(
    items: &'a [Box<T>],
    title: &str,
    border: &str,
    layout: &str,
    use_fzf: bool,
    theme: &str,
) -> SelectItemReturn<'a, Box<T>> {
    let item_strings: Vec<String> = items.iter().map(|w| w.to_string()).collect();

    match dispatch_picker(&item_strings, title, border, layout, use_fzf, theme) {
        PickerOutput::Cancelled => SelectItemReturn::None,
        PickerOutput::Selected(idx) => {
            if let Some(item) = items.get(idx) {
                SelectItemReturn::Item(item)
            } else {
                SelectItemReturn::None
            }
        }
        PickerOutput::New(title) => SelectItemReturn::NewWindowTitle(title),
    }
}
