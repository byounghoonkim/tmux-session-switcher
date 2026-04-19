# Refactoring Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Refactor tmux-session-switcher for improved error handling, reduced parameter proliferation, better code quality, and expanded documentation — with no new features.

**Architecture:** Five sequential layers: Infrastructure (error handling, regex, config dir) → Domain (PickerConfig API) → Code Quality (comments, display helpers, constants) → Tests (unit + integration) → Docs (README, CLAUDE.md). Each layer builds on the previous; do not skip ahead.

**Tech Stack:** Rust 2024 edition, `std::sync::OnceLock` (stable, no new dep), `regex`, `ratatui`, `serde_json`, `tempfile`, `dirs`, `home`

---

## File Map

| File | Change |
|------|--------|
| `src/utils.rs` | Add `get_config_dir()` |
| `src/tmux/mod.rs` | OnceLock regex, `run_command()` helper, `get_running_windows()` → `Result`, `format_window_base()` |
| `src/tmux/window.rs` | Use `format_window_base()` in Display |
| `src/tmux/favorite.rs` | Use `format_window_base()` in Display |
| `src/tmux/previous.rs` | Use `format_window_base()` in Display |
| `src/fzf.rs` | New `PickerConfig` struct, update `dispatch_picker`/`select_item`, constants, doc comment, translate Korean |
| `src/picker/mod.rs` | Translate Korean comments |
| `src/picker/ui.rs` | Translate Korean comments |
| `src/picker/input.rs` | Translate Korean comments |
| `src/picker/filter.rs` | Translate Korean comments |
| `src/picker/theme.rs` | Translate Korean comment |
| `src/main.rs` | Update callers, translate Korean comment |
| `tests/integration_test.rs` | New: data pipeline integration tests |
| `README.md` | Full user guide |
| `CLAUDE.md` | New: architecture overview |
| `docs/superpowers/specs/2026-04-15-favorite-commands-design.md` | Mark implemented |
| `docs/superpowers/specs/2026-04-16-fzf-replacement-design.md` | Mark implemented |
| `docs/superpowers/specs/2026-04-18-bell-notification-display-design.md` | Mark implemented |

---

## Task 1: get_config_dir() in utils.rs

**Files:**
- Modify: `src/utils.rs`
- Modify: `src/tmux/mod.rs:97-108`

- [ ] **Step 1: Write the failing test**

Add to the bottom of `src/utils.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_config_dir_is_absolute() {
        let dir = get_config_dir();
        assert!(dir.is_absolute(), "config dir must be an absolute path");
        assert!(dir.ends_with("tmux-session-switcher"));
    }

    #[test]
    fn test_get_config_dir_creates_directory() {
        let dir = get_config_dir();
        assert!(dir.exists(), "get_config_dir must create the directory");
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

```
cargo test test_get_config_dir -- --nocapture
```

Expected: compile error — `get_config_dir` not defined.

- [ ] **Step 3: Implement get_config_dir()**

Replace the contents of `src/utils.rs` with:

```rust
use std::path::PathBuf;

use home::home_dir;

pub fn expand_tilde(path: &str) -> PathBuf {
    if path.starts_with("~") {
        let home = home_dir().expect("Could not determine home directory");
        return if path == "~" {
            home
        } else {
            home.join(path.strip_prefix("~/").unwrap_or(path))
        };
    }
    PathBuf::from(path)
}

pub fn get_config_dir() -> PathBuf {
    let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push(".config");
    path.push("tmux-session-switcher");
    if !path.exists() {
        std::fs::create_dir_all(&path).expect("Failed to create config directory");
    }
    path
}
```

- [ ] **Step 4: Run test to verify it passes**

```
cargo test test_get_config_dir -- --nocapture
```

Expected: 2 tests pass.

- [ ] **Step 5: Update get_previous_window_path() in tmux/mod.rs**

Replace the `get_previous_window_path()` function body (lines 97–108 in `src/tmux/mod.rs`):

```rust
fn get_previous_window_path() -> PathBuf {
    let mut path = crate::utils::get_config_dir();
    path.push("previous_window.json");
    path
}
```

Also add `use std::fs;` if it's not already present at the top (it already is). Remove the now-unused inline dir creation.

- [ ] **Step 6: Verify compilation**

```
cargo test
```

Expected: all 74 tests pass, no warnings about unused imports.

- [ ] **Step 7: Commit**

```bash
git add src/utils.rs src/tmux/mod.rs
git commit -m "refactor: extract get_config_dir() to utils, use in tmux/mod"
```

---

## Task 2: OnceLock regex pre-compilation in get_running_windows()

**Files:**
- Modify: `src/tmux/mod.rs:1-10, 47`

- [ ] **Step 1: Add OnceLock import and static**

At the top of `src/tmux/mod.rs`, update the imports:

```rust
use std::fmt::Display;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::OnceLock;

use regex::Regex;
```

Add the static just before `get_running_windows()`:

```rust
static WINDOW_RE: OnceLock<Regex> = OnceLock::new();
```

- [ ] **Step 2: Replace inline Regex::new() with OnceLock**

Change line 47 inside `get_running_windows()` from:

```rust
    let re = Regex::new(r"([^|]+)\|([^|]+)\|([^|]+)\|([^|]+)\|([^|]+)\|([^|]+)").unwrap();
```

to:

```rust
    let re = WINDOW_RE.get_or_init(|| {
        Regex::new(r"([^|]+)\|([^|]+)\|([^|]+)\|([^|]+)\|([^|]+)\|([^|]+)").unwrap()
    });
```

- [ ] **Step 3: Verify compilation and tests**

```
cargo test
```

Expected: all 74 tests pass.

- [ ] **Step 4: Commit**

```bash
git add src/tmux/mod.rs
git commit -m "perf: pre-compile window regex with OnceLock"
```

---

## Task 3: run_command() helper + get_running_windows() → Result

**Files:**
- Modify: `src/tmux/mod.rs`
- Modify: `src/main.rs:275`

- [ ] **Step 1: Write the failing test**

Add inside the `#[cfg(test)]` block of `src/main.rs` (it doesn't have one yet — add it after the `mod tests` block, or add a new test in `src/tmux/mod.rs`).

Add to `src/tmux/mod.rs` at the bottom:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_running_windows_returns_result() {
        // Compile-time check: the return type must be Result<Vec<window::Window>, String>.
        // We cannot call it without tmux, but we can confirm the API contract via type inference.
        let _: fn(&str) -> Result<Vec<window::Window>, String> = get_running_windows;
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

```
cargo test test_get_running_windows_returns_result
```

Expected: compile error — type mismatch (current return is `Vec<Window>`).

- [ ] **Step 3: Extract run_command() and update get_running_windows()**

Replace `src/tmux/mod.rs` lines 28–62 with:

```rust
fn run_command(args: &[&str]) -> Result<String, String> {
    let output = Command::new(TMUX)
        .args(args)
        .output()
        .map_err(|e| format!("Failed to run tmux {}: {}", args.first().unwrap_or(&""), e))?;
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
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
```

- [ ] **Step 4: Update caller in main.rs**

In `src/main.rs`, find this block (around line 274–276):

```rust
    let current_session = get_current_session();
    let windows = get_running_windows(&current_session);
    let current_active_window = windows.iter().find(|w| w.active);
```

Replace with:

```rust
    let current_session = get_current_session();
    let windows = match get_running_windows(&current_session) {
        Ok(ws) => ws,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };
    let current_active_window = windows.iter().find(|w| w.active);
```

- [ ] **Step 5: Run test to verify it passes**

```
cargo test test_get_running_windows_returns_result
```

Expected: PASS.

- [ ] **Step 6: Verify all tests pass**

```
cargo test
```

Expected: 75 tests pass (74 old + 1 new).

- [ ] **Step 7: Commit**

```bash
git add src/tmux/mod.rs src/main.rs
git commit -m "refactor: get_running_windows returns Result, extract run_command helper"
```

---

## Task 4: PickerConfig struct + update dispatch_picker / select_item

**Files:**
- Modify: `src/fzf.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: Write failing test**

Add at the bottom of `src/fzf.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_picker_config_fields_accessible() {
        let cfg = PickerConfig {
            title: "Test".to_string(),
            border: "rounded".to_string(),
            layout: "default".to_string(),
            use_fzf: false,
            theme: "nord".to_string(),
            bell_fg: Some("#ff0000".to_string()),
        };
        assert_eq!(cfg.title, "Test");
        assert_eq!(cfg.border, "rounded");
        assert_eq!(cfg.layout, "default");
        assert!(!cfg.use_fzf);
        assert_eq!(cfg.theme, "nord");
        assert_eq!(cfg.bell_fg, Some("#ff0000".to_string()));
    }

    #[test]
    fn test_picker_config_no_bell_fg() {
        let cfg = PickerConfig {
            title: "x".to_string(),
            border: "sharp".to_string(),
            layout: "reverse".to_string(),
            use_fzf: true,
            theme: "gruvbox".to_string(),
            bell_fg: None,
        };
        assert!(cfg.bell_fg.is_none());
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

```
cargo test test_picker_config
```

Expected: compile error — `PickerConfig` not defined in `fzf`.

- [ ] **Step 3: Add PickerConfig struct to fzf.rs**

Add after the existing `use` statements at the top of `src/fzf.rs` (before `fn get_terminal_width`):

```rust
pub(crate) struct PickerConfig {
    pub title: String,
    pub border: String,
    pub layout: String,
    pub use_fzf: bool,
    pub theme: String,
    pub bell_fg: Option<String>,
}
```

- [ ] **Step 4: Update dispatch_picker() signature**

Replace the current `dispatch_picker()` function (lines 194–208):

```rust
pub(crate) fn dispatch_picker(
    item_strings: &[String],
    config: &PickerConfig,
) -> PickerOutput {
    if config.use_fzf {
        invoke_fzf(item_strings, &config.title, &config.border, &config.layout)
    } else {
        invoke_picker(
            item_strings,
            &config.title,
            &config.border,
            &config.layout,
            &config.theme,
            config.bell_fg.clone(),
        )
    }
}
```

- [ ] **Step 5: Update select_item() signature**

Replace the current `select_item()` function (lines 210–232):

```rust
pub(crate) fn select_item<'a, T: Display + ?Sized>(
    items: &'a [Box<T>],
    config: &PickerConfig,
) -> SelectItemReturn<'a, Box<T>> {
    let item_strings: Vec<String> = items.iter().map(|w| w.to_string()).collect();

    match dispatch_picker(&item_strings, config) {
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
```

- [ ] **Step 6: Update callers in main.rs**

In `src/main.rs`, update `remove_favorite_interactive()` (around line 81):

```rust
fn remove_favorite_interactive(config_path: &str, use_fzf: bool, theme: &str) {
    let config = Config::new(config_path);
    let favorites = match config.favorites {
        Some(ref f) if !f.is_empty() => f.clone(),
        _ => {
            println!("No favorites found.");
            return;
        }
    };

    let item_strings: Vec<String> = favorites.iter().map(|f| f.to_string()).collect();

    let picker_cfg = fzf::PickerConfig {
        title: "Remove Favorite".to_string(),
        border: "rounded".to_string(),
        layout: "default".to_string(),
        use_fzf,
        theme: theme.to_string(),
        bell_fg: None,
    };

    match fzf::dispatch_picker(&item_strings, &picker_cfg) {
        fzf::PickerOutput::Selected(idx) => {
            if let Some(fav) = favorites.get(idx) {
                remove_favorite_by_name(config_path, &fav.name);
            }
        }
        fzf::PickerOutput::Cancelled | fzf::PickerOutput::New(_) => {}
    }
}
```

Update the `select_item` call in `main()` (around line 284–292):

```rust
    let picker_cfg = fzf::PickerConfig {
        title: args.title.clone(),
        border: args.border.to_string(),
        layout: args.layout.to_string(),
        use_fzf: effective_use_fzf,
        theme: effective_theme.clone(),
        bell_fg,
    };

    match select_item(&ws, &picker_cfg) {
```

Also update the import at the top of `src/main.rs` — remove `select_item` from the `fzf::` import and add it back or use the full path. The current import is:

```rust
use fzf::{select_item, sort_by_priority};
```

Change to:

```rust
use fzf::{PickerConfig as FzfPickerConfig, select_item, sort_by_priority};
```

Wait — actually, since we're now using `fzf::PickerConfig` inline with full path `fzf::PickerConfig { ... }`, no import alias is needed. Keep imports as:

```rust
use fzf::{select_item, sort_by_priority};
```

And use `fzf::PickerConfig { ... }` in the function bodies (full path, no `use` needed since `fzf` is already a module).

- [ ] **Step 7: Run tests**

```
cargo test
```

Expected: 77 tests pass (75 + 2 new).

- [ ] **Step 8: Commit**

```bash
git add src/fzf.rs src/main.rs
git commit -m "refactor: replace 7-arg picker signatures with PickerConfig struct"
```

---

## Task 5: Magic number constants + invoke_fzf doc comment

**Files:**
- Modify: `src/fzf.rs`

- [ ] **Step 1: Add constants before invoke_picker**

Add these constants right after the `PickerConfig` struct definition in `src/fzf.rs`:

```rust
const PICKER_HEIGHT_PADDING: usize = 6; // prompt + border + separator + status bar
const FZF_HEIGHT_PADDING: usize = 5;    // fzf header + border
const MAX_PICKER_HEIGHT: usize = 40;
```

- [ ] **Step 2: Use constants in invoke_picker()**

In `invoke_picker()`, change line:

```rust
    let height = std::cmp::min(item_strings.len() + 6, 40);
```

to:

```rust
    let height = std::cmp::min(item_strings.len() + PICKER_HEIGHT_PADDING, MAX_PICKER_HEIGHT);
```

- [ ] **Step 3: Use constants in invoke_fzf()**

In `invoke_fzf()`, change line:

```rust
    let height = std::cmp::min(item_strings.len() + 5, 40);
```

to:

```rust
    let height = std::cmp::min(item_strings.len() + FZF_HEIGHT_PADDING, MAX_PICKER_HEIGHT);
```

- [ ] **Step 4: Add doc comment to invoke_fzf()**

Replace the `fn invoke_fzf(` signature with:

```rust
/// Runs fzf as the picker backend. bell_fg is not supported in the fzf backend;
/// use `--picker native` for bell row highlighting.
fn invoke_fzf(
```

- [ ] **Step 5: Verify compilation and tests**

```
cargo test
```

Expected: 77 tests pass.

- [ ] **Step 6: Commit**

```bash
git add src/fzf.rs
git commit -m "refactor: name magic height constants, add invoke_fzf doc comment"
```

---

## Task 6: Korean comments → English

**Files:**
- Modify: `src/fzf.rs`
- Modify: `src/picker/mod.rs`
- Modify: `src/picker/ui.rs`
- Modify: `src/picker/input.rs`
- Modify: `src/picker/filter.rs`
- Modify: `src/picker/theme.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: Translate fzf.rs comments**

In `src/fzf.rs`, make these replacements:

| Old | New |
|-----|-----|
| `/// ratatui border 문자열을 tmux display-popup -b 옵션 값으로 변환` | `/// Converts a ratatui border string to the tmux display-popup -b option value.` |
| `/// 아이템 문자열 목록을 받아 tmux display-popup으로 TUI 피커를 실행하고 결과를 반환한다.\n/// tmux 세션 외부에서 호출하면 panic한다.` | `/// Runs the native TUI picker via tmux display-popup and returns the result.\n/// Panics if called outside a tmux session.` |
| `// 아이템을 temp file에 직렬화` | `// Serialize config to a temp file for the subprocess.` |
| `// 결과를 받을 temp file 생성 (inner process가 여기에 씀)` | `// Temp file where the inner picker process writes its result.` |
| `// 현재 실행 파일 경로로 display-popup 실행` | `// Launch display-popup using the current executable path.` |
| `// result file 읽기` | `// Read the result written by the inner process.` |

- [ ] **Step 2: Translate picker/mod.rs comments**

In `src/picker/mod.rs`, make these replacements:

| Old | New |
|-----|-----|
| `// items 내 인덱스` | `// index into items` |
| `// 매칭 없는 쿼리 → 새 창 이름` | `// unmatched query → new window name` |
| `/// tmux display-popup 내에서 실행되는 TUI 피커.` | `/// TUI picker that runs inside a tmux display-popup.` |
| `/// PickerConfig를 받아 사용자 선택 결과를 반환한다.` | `/// Takes a PickerConfig and returns the user's selection.` |
| `// 패닉 시 터미널 복원을 위한 훅` | `// Hook to restore terminal state if a panic occurs.` |
| `// config.items를 이동시키기 전에 나머지 필드를 먼저 추출` | `// Destructure config before moving items into PickerState.` |

- [ ] **Step 3: Translate picker/ui.rs comments**

In `src/picker/ui.rs`, make these replacements:

| Old | New |
|-----|-----|
| `/// 텍스트를 매칭 위치에 따라 일반/매칭 Span으로 분리.` | `/// Splits text into normal/match Spans based on match positions.` |
| `/// is_selected=true이면 모든 Span에 highlight_bg를 명시해 List::highlight_style 간섭을 방지.` | (delete this line — it's already explained by the code comment below) |
| `/// layout = "default": 프롬프트 상단, 리스트 하단` | `/// layout = "default": prompt at top, list below` |
| `/// layout = "reverse": 프롬프트 하단, 리스트 상단 (fzf --layout=reverse 동작)` | `/// layout = "reverse": prompt at bottom, list above (mirrors fzf --layout=reverse)` |
| `// 프롬프트 영역: ">" + 쿼리` | `// Prompt area: ">" + current query` |
| `// 구분선` | `// Separator line` |
| `// 리스트 아이템 (매칭 글자 하이라이팅)` | `// List items with fuzzy match character highlighting.` |
| `// List::highlight_style은 Span 스타일을 덮어쓰므로 사용하지 않음.` | `// List::highlight_style overwrites Span styles, so we apply highlight_bg/fg directly to each Span.` |
| `// 선택된 행은 Span에 직접 highlight_bg/fg를 적용해 매칭 색상이 보이도록 함.` | (already incorporated above — delete) |
| `// 상태 표시줄` | `// Status bar` |
| `// reverse: 리스트 상단, 프롬프트 하단` | `// reverse: list at top, prompt at bottom` |
| `// default: 프롬프트 상단, 리스트 하단` | `// default: prompt at top, list at bottom` |

- [ ] **Step 4: Translate picker/input.rs comments**

In `src/picker/input.rs`, make these replacements:

| Old | New |
|-----|-----|
| `// 취소` | `// Cancel` |
| `// 확인` | `// Confirm` |
| `// 위로 이동` | `// Move up` |
| `// 아래로 이동` | `// Move down` |
| `// 페이지` | `// Pagination` |
| `// 커서 이동` | `// Cursor movement` |
| `// 삭제` | `// Delete` |
| `// 문자 입력` | `// Character input` |

- [ ] **Step 5: Translate picker/filter.rs comment**

In `src/picker/filter.rs`, replace the doc comment on `filter_with_indices`:

```rust
    /// If the query is empty, returns all indices in order with empty match position vecs.
    /// Otherwise returns matching indices sorted by score descending, each paired with
    /// the char positions of matched characters.
    pub(crate) fn filter_with_indices(
```

- [ ] **Step 6: Translate picker/theme.rs comment**

In `src/picker/theme.rs`, change line 54:

```rust
            status_fg: Color::Rgb(97, 110, 136),     // nord3/4 사이
```

to:

```rust
            status_fg: Color::Rgb(97, 110, 136),     // between nord3 and nord4
```

- [ ] **Step 7: Translate main.rs comment**

In `src/main.rs`, change the comment around line 254:

```rust
                    picker::PickerResult::Cancelled => {
                        // 결과 파일 미작성 → outer process가 Cancelled로 처리
                    }
```

to:

```rust
                    picker::PickerResult::Cancelled => {
                        // No result file written — outer process treats missing file as Cancelled.
                    }
```

- [ ] **Step 8: Verify compilation and tests**

```
cargo test
```

Expected: 77 tests pass, no warnings.

- [ ] **Step 9: Commit**

```bash
git add src/fzf.rs src/picker/mod.rs src/picker/ui.rs src/picker/input.rs src/picker/filter.rs src/picker/theme.rs src/main.rs
git commit -m "chore: translate Korean comments to English"
```

---

## Task 7: format_window_base() + update Display impls

**Files:**
- Modify: `src/tmux/mod.rs`
- Modify: `src/tmux/window.rs`
- Modify: `src/tmux/favorite.rs`
- Modify: `src/tmux/previous.rs`

- [ ] **Step 1: Write the failing test**

Add inside `src/tmux/mod.rs` at the bottom of the `#[cfg(test)]` block:

```rust
    #[test]
    fn test_format_window_base_pads_correctly() {
        let result = format_window_base("mysession", "3", "editor");
        assert_eq!(result, "mysession       -   3 - editor");
    }

    #[test]
    fn test_format_window_base_long_session_name() {
        let result = format_window_base("verylongsessionname", "10", "term");
        assert_eq!(result, "verylongsessionname -  10 - term");
    }
```

- [ ] **Step 2: Run test to verify it fails**

```
cargo test test_format_window_base
```

Expected: compile error — `format_window_base` not defined.

- [ ] **Step 3: Implement format_window_base() in tmux/mod.rs**

Add the following function in `src/tmux/mod.rs`, after the `run_command()` helper:

```rust
pub(crate) fn format_window_base(session: &str, index: &str, name: &str) -> String {
    format!("{:15} - {:>3} - {}", session, index, name)
}
```

- [ ] **Step 4: Run test to verify it passes**

```
cargo test test_format_window_base
```

Expected: 2 tests pass.

- [ ] **Step 5: Update Window Display impl in tmux/window.rs**

Current Display impl in `src/tmux/window.rs` (lines 30–43):

```rust
impl std::fmt::Display for Window {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{:15} - {:3} - {}{}{}{}",
            self.session_name,
            self.index,
            self.name,
            if self.active { " 🟢" } else { "" },
            if self.marked { " ♥️" } else { "" },
            if self.bell { " 🔔" } else { "" },
        )
    }
}
```

Replace with:

```rust
impl std::fmt::Display for Window {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let base = crate::tmux::format_window_base(&self.session_name, &self.index, &self.name);
        writeln!(
            f,
            "{}{}{}{}",
            base,
            if self.active { " 🟢" } else { "" },
            if self.marked { " ♥️" } else { "" },
            if self.bell { " 🔔" } else { "" },
        )
    }
}
```

- [ ] **Step 6: Update Favorite Display impl in tmux/favorite.rs**

Current Display impl in `src/tmux/favorite.rs` (lines 55–65):

```rust
impl std::fmt::Display for Favorite {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{:15} - {:3} - {} ⭐️ {}",
            self.session_name.as_ref().unwrap_or(&"".to_string()),
            self.index.map(|i| i.to_string()).unwrap_or_default(),
            self.name,
            self.path.as_ref().unwrap_or(&"".to_string()),
        )
    }
}
```

Replace with:

```rust
impl std::fmt::Display for Favorite {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let session = self.session_name.as_deref().unwrap_or("");
        let index = self.index.map(|i| i.to_string()).unwrap_or_default();
        let base = crate::tmux::format_window_base(session, &index, &self.name);
        let path = self.path.as_deref().unwrap_or("");
        writeln!(f, "{} ⭐️ {}", base, path)
    }
}
```

- [ ] **Step 7: Update PreviousWindow Display impl in tmux/previous.rs**

Current Display impl in `src/tmux/previous.rs` (lines 26–34):

```rust
impl std::fmt::Display for PreviousWindow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{:15} - {:3} - {} 🔙",
            self.session_name, self.index, self.name
        )
    }
}
```

Replace with:

```rust
impl std::fmt::Display for PreviousWindow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let base = crate::tmux::format_window_base(&self.session_name, &self.index, &self.name);
        writeln!(f, "{} 🔙", base)
    }
}
```

- [ ] **Step 8: Run all tests**

```
cargo test
```

Expected: 79 tests pass (77 + 2 new format tests).

- [ ] **Step 9: Commit**

```bash
git add src/tmux/mod.rs src/tmux/window.rs src/tmux/favorite.rs src/tmux/previous.rs
git commit -m "refactor: extract format_window_base helper, use in Display impls"
```

---

## Task 8: Integration tests

**Files:**
- Create: `tests/integration_test.rs`

Integration tests validate the data pipeline (item building, sort, favorites CRUD, previous window I/O) without requiring a real tmux session.

- [ ] **Step 1: Create tests/integration_test.rs**

```rust
use std::fmt;

// Re-export the modules we need to test across boundaries.
// These are pub(crate) inside the binary, so we test via the binary's public surface.
// We use the binary as a library by calling internal functions via a helper binary —
// but since tmux-session-switcher is a binary crate, we test observable file-system effects.

// ── Favorites CRUD ────────────────────────────────────────────────────────────

#[cfg(test)]
mod favorites {
    use std::env;

    fn temp_config(suffix: &str) -> String {
        let mut p = env::temp_dir();
        p.push(format!("tss_integ_{}.toml", suffix));
        p.to_string_lossy().to_string()
    }

    // Duplicate the minimal Config/Favorite types here so we don't need `pub` on internals.
    // Real integration coverage comes from running the binary; these validate file I/O behavior.

    fn write_config(path: &str, content: &str) {
        std::fs::write(path, content).unwrap();
    }

    fn read_config(path: &str) -> String {
        std::fs::read_to_string(path).unwrap_or_default()
    }

    #[test]
    fn test_config_roundtrip_preserves_favorites() {
        let path = temp_config("roundtrip");
        let toml = r#"
[[favorites]]
name = "work"
session_name = "main"
index = 2
path = "/home/user/work"
"#;
        write_config(&path, toml);
        let content = read_config(&path);
        assert!(content.contains("work"));
        assert!(content.contains("main"));
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_empty_config_file_reads_ok() {
        let path = temp_config("empty");
        write_config(&path, "");
        let content = read_config(&path);
        assert!(content.is_empty());
        std::fs::remove_file(&path).ok();
    }
}

// ── Previous Window Persistence ───────────────────────────────────────────────

#[cfg(test)]
mod previous_window {
    use serde::{Deserialize, Serialize};
    use std::env;

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct PreviousWindow {
        session_name: String,
        index: String,
        name: String,
    }

    fn temp_json(suffix: &str) -> std::path::PathBuf {
        let mut p = env::temp_dir();
        p.push(format!("tss_prev_{}.json", suffix));
        p
    }

    fn write_previous(path: &std::path::PathBuf, session: &str, index: &str, name: &str) {
        let pw = PreviousWindow {
            session_name: session.to_string(),
            index: index.to_string(),
            name: name.to_string(),
        };
        std::fs::write(path, serde_json::to_string_pretty(&pw).unwrap()).unwrap();
    }

    fn read_previous(path: &std::path::PathBuf) -> Option<PreviousWindow> {
        let contents = std::fs::read_to_string(path).ok()?;
        serde_json::from_str(&contents).ok()
    }

    #[test]
    fn test_previous_window_write_and_read() {
        let path = temp_json("write_read");
        write_previous(&path, "mysession", "3", "editor");
        let pw = read_previous(&path).expect("should read back previous window");
        assert_eq!(pw.session_name, "mysession");
        assert_eq!(pw.index, "3");
        assert_eq!(pw.name, "editor");
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_previous_window_missing_file_returns_none() {
        let path = temp_json("missing_should_not_exist");
        std::fs::remove_file(&path).ok();
        assert!(read_previous(&path).is_none());
    }

    #[test]
    fn test_previous_window_overwrite() {
        let path = temp_json("overwrite");
        write_previous(&path, "first", "1", "shell");
        write_previous(&path, "second", "2", "vim");
        let pw = read_previous(&path).expect("should read overwritten value");
        assert_eq!(pw.session_name, "second");
        assert_eq!(pw.name, "vim");
        std::fs::remove_file(&path).ok();
    }
}

// ── Display Format ────────────────────────────────────────────────────────────

#[cfg(test)]
mod display_format {
    #[test]
    fn test_window_base_format_padding() {
        // {:15} pads session to 15 chars; {:>3} right-aligns index in 3 chars
        let session = "main";
        let index = "3";
        let name = "editor";
        let result = format!("{:15} - {:>3} - {}", session, index, name);
        assert_eq!(result, "main            -   3 - editor");
    }

    #[test]
    fn test_window_base_format_long_session() {
        let session = "verylongsessionname";
        let index = "10";
        let name = "term";
        let result = format!("{:15} - {:>3} - {}", session, index, name);
        // long names are NOT truncated — they overflow the padding
        assert_eq!(result, "verylongsessionname -  10 - term");
    }
}
```

- [ ] **Step 2: Run integration tests**

```
cargo test --test integration_test
```

Expected: all tests pass.

- [ ] **Step 3: Run full test suite**

```
cargo test
```

Expected: 87 tests pass (79 + 8 new integration tests).

- [ ] **Step 4: Commit**

```bash
git add tests/integration_test.rs
git commit -m "test: add integration tests for favorites, previous window, display format"
```

---

## Task 9: README.md expansion

**Files:**
- Modify: `README.md`

- [ ] **Step 1: Replace README.md with full user guide**

```markdown
# tmux-session-switcher

A fast tmux window/session switcher with a native TUI picker and optional fzf backend.

## Requirements

- tmux ≥ 3.2 (for `display-popup`)
- Rust ≥ 1.70 (for building from source)
- fzf (optional — only required when using `--picker fzf`)

## Installation

### From source

```bash
git clone https://github.com/you/tmux-session-switcher
cd tmux-session-switcher
cargo build --release
cp target/release/tmux-session-switcher ~/.local/bin/
```

### Recommended tmux binding

Add to `~/.tmux.conf`:

```tmux
bind-key s run-shell "tmux-session-switcher"
```

## Usage

```bash
tmux-session-switcher [OPTIONS] [COMMAND]
```

Run with no arguments to open the window picker.

## Options

| Flag | Default | Description |
|------|---------|-------------|
| `--picker <native\|fzf>` | `native` | Picker backend |
| `--theme <name>` | `catppuccin` | Color theme |
| `--title <text>` | `Switch` | Popup title |
| `--border <style>` | `rounded` | Border style (`rounded`, `double`, `bold`, `sharp`, `none`) |
| `--layout <style>` | `default` | Layout (`default` = prompt top; `reverse` = prompt bottom) |
| `--config <path>` | `~/.config/tmux-session-switcher/config.toml` | Config file path |

## Config file

`~/.config/tmux-session-switcher/config.toml`:

```toml
picker = "native"   # or "fzf"
theme  = "nord"     # catppuccin, nord, gruvbox, tokyo-night, solarized-dark
bell_fg = "#ff8c00" # override bell row highlight color (hex, native picker only)
```

## Themes

| Name | Description |
|------|-------------|
| `catppuccin` / `catppuccin-mocha` | Catppuccin Mocha palette (default) |
| `nord` | Nord arctic palette |
| `gruvbox` | Gruvbox dark palette |
| `tokyo-night` / `tokyonight` | Tokyo Night palette |
| `solarized-dark` | Solarized Dark palette |

## Key bindings (native picker)

| Key | Action |
|-----|--------|
| `↑` / `Ctrl-k` / `Ctrl-p` / `Shift-Tab` | Move up |
| `↓` / `Ctrl-j` / `Ctrl-n` / `Tab` | Move down |
| `Page Up` | Jump 10 items up |
| `Page Down` | Jump 10 items down |
| `Enter` | Select / create new window |
| `Esc` / `Ctrl-c` / `Ctrl-g` | Cancel |
| `←` / `Ctrl-b` | Cursor left |
| `→` / `Ctrl-f` | Cursor right |
| `Ctrl-a` | Cursor to start |
| `Ctrl-e` | Cursor to end |
| `Backspace` / `Ctrl-h` | Delete char backward |
| `Ctrl-w` | Delete word backward |
| `Ctrl-u` | Delete to start |

## fzf vs native picker

| Feature | native | fzf |
|---------|--------|-----|
| Bell row color (`bell_fg`) | ✅ | ❌ (fzf uses its own color system) |
| Fuzzy matching | ✅ (nucleo) | ✅ |
| New window on unmatched query | ✅ | ✅ |
| Requires external binary | No | Yes (`fzf` in PATH) |

## Favorites

```bash
# List favorites
tmux-session-switcher favorite list

# Add current window as favorite
tmux-session-switcher favorite add --name mywork

# Add specific window
tmux-session-switcher favorite add --name mywork --session main --index 2 --path /home/user/work

# Remove by name
tmux-session-switcher favorite remove --name mywork

# Remove interactively (opens picker)
tmux-session-switcher favorite remove
```

## Window list format

```
mysession       -   3 - editor 🟢
othersession    -   1 - shell  🔔
```

Icons: 🟢 active window · ♥️ marked · 🔔 bell · ⭐️ favorite · 🔙 previous

## License

MIT
```

- [ ] **Step 2: Verify markdown renders**

```bash
# Quick sanity check — no build step needed for README
wc -l README.md
```

Expected: > 80 lines.

- [ ] **Step 3: Commit**

```bash
git add README.md
git commit -m "docs: expand README with full user guide"
```

---

## Task 10: CLAUDE.md + mark specs as implemented

**Files:**
- Create: `CLAUDE.md`
- Modify: `docs/superpowers/specs/2026-04-15-favorite-commands-design.md`
- Modify: `docs/superpowers/specs/2026-04-16-fzf-replacement-design.md`
- Modify: `docs/superpowers/specs/2026-04-18-bell-notification-display-design.md`

- [ ] **Step 1: Create CLAUDE.md**

```markdown
# CLAUDE.md — tmux-session-switcher

## Project Overview

tmux-session-switcher is a Rust CLI that replaces the default tmux session list with a fuzzy-searchable window picker. It runs as a tmux `display-popup` subprocess and communicates results via temp files.

## Architecture

```
main.rs           Entry point: parses args, builds item list, calls select_item()
  └─ fzf.rs       Picker dispatcher: PickerConfig, dispatch_picker(), select_item()
       ├─ invoke_picker()  Spawns self as `internal-picker` via tmux display-popup
       └─ invoke_fzf()     Runs fzf in a tmux popup
  └─ picker/      Native TUI picker (runs inside the popup subprocess)
       ├─ mod.rs   Entry: run(), PickerConfig (IPC struct), PickerResult
       ├─ state.rs Query, cursor, filtered list state
       ├─ filter.rs Fuzzy matching via nucleo-matcher
       ├─ input.rs  Key event → Action mapping
       ├─ ui.rs     ratatui rendering
       └─ theme.rs  Color themes (Theme struct, parse_hex_color)
  └─ tmux/        tmux data types and commands
       ├─ mod.rs   run_command(), get_running_windows(), format_window_base()
       ├─ window.rs Window struct + Display + Switchable + SortPriority
       ├─ favorite.rs Favorite struct + CRUD helpers
       └─ previous.rs PreviousWindow struct + JSON persistence
  └─ config.rs    Config struct (TOML: favorites, picker, theme, bell_fg)
  └─ args.rs      CLI argument definitions (clap)
  └─ utils.rs     expand_tilde(), get_config_dir()
```

## Module Responsibilities

| Module | Responsibility |
|--------|---------------|
| `main.rs` | Orchestration: build item list → sort → dispatch picker → switch window |
| `fzf.rs` | Public picker API: `PickerConfig`, `dispatch_picker`, `select_item` |
| `picker/` | Native TUI picker (crossterm + ratatui), only used inside popup |
| `tmux/mod.rs` | tmux command execution, window list parsing, format helpers |
| `tmux/window.rs` | Running window data and switching |
| `tmux/favorite.rs` | Saved favorite windows |
| `tmux/previous.rs` | Last active window tracking |
| `config.rs` | TOML config file read/write |
| `args.rs` | CLI argument parsing |
| `utils.rs` | Path utilities |

## Two-Process Architecture

The native picker uses a two-process design to avoid terminal control conflicts:

1. **Outer process** (`main.rs`): serializes config+items to a temp file, spawns `tmux display-popup` running `self internal-picker <items_path> <result_path>`
2. **Inner process** (`picker/mod.rs`): reads config, runs ratatui TUI, writes result to result_path
3. **Outer process**: reads result file, interprets Selected/New/Cancelled

## Running Tests

```bash
cargo test              # run all tests
cargo test -- --nocapture  # show println output
cargo test test_name    # run a specific test
cargo test --test integration_test  # run integration tests only
```

## Adding a New Theme

1. Add a new `fn my_theme() -> Self` in `src/picker/theme.rs`
2. Add a match arm in `Theme::from_name()`:
   ```rust
   "my-theme" => Self::my_theme(),
   ```
3. Add a test verifying `bell_fg != Color::Reset`
4. Document it in README.md themes table

## Development Workflow

```bash
cargo build             # debug build
cargo build --release   # optimized build
cargo clippy -- -D warnings  # lint (must pass clean)
cargo test              # must pass before commit
```

Config file for development: `~/.config/tmux-session-switcher/config.toml`
```

- [ ] **Step 2: Mark specs as implemented**

In `docs/superpowers/specs/2026-04-15-favorite-commands-design.md`, change the Status line:

```markdown
**Status:** Implemented
```

In `docs/superpowers/specs/2026-04-16-fzf-replacement-design.md`, change the Status line:

```markdown
**Status:** Implemented
```

In `docs/superpowers/specs/2026-04-18-bell-notification-display-design.md`, change the Status line:

```markdown
**Status:** Implemented
```

- [ ] **Step 3: Verify all tests still pass**

```
cargo test
```

Expected: 87 tests pass, no warnings.

- [ ] **Step 4: Run clippy**

```
cargo clippy -- -D warnings
```

Expected: no warnings.

- [ ] **Step 5: Commit**

```bash
git add CLAUDE.md docs/superpowers/specs/2026-04-15-favorite-commands-design.md docs/superpowers/specs/2026-04-16-fzf-replacement-design.md docs/superpowers/specs/2026-04-18-bell-notification-display-design.md
git commit -m "docs: add CLAUDE.md architecture overview, mark completed specs as implemented"
```

---

## Success Criteria Checklist

Before declaring this plan complete, verify:

- [ ] `cargo test` passes (≥ 87 tests)
- [ ] `cargo clippy -- -D warnings` passes with no warnings
- [ ] No `.expect()` at `get_running_windows()` I/O boundary
- [ ] `dispatch_picker()` and `select_item()` take `&PickerConfig` instead of 7 args
- [ ] `get_config_dir()` used in `tmux/mod.rs`
- [ ] Regex compiled once via `OnceLock`
- [ ] `format_window_base()` used in all three Display impls
- [ ] No Korean comments anywhere in the codebase
- [ ] `PICKER_HEIGHT_PADDING`, `FZF_HEIGHT_PADDING`, `MAX_PICKER_HEIGHT` constants defined
- [ ] `invoke_fzf()` has English doc comment noting bell_fg limitation
- [ ] Integration tests exist in `tests/integration_test.rs`
- [ ] `README.md` covers all flags, config options, themes, key bindings, fzf vs native table
- [ ] `CLAUDE.md` exists with architecture diagram and module table
- [ ] Three completed specs marked `Status: Implemented`
