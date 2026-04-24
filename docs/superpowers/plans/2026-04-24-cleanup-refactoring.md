# Cleanup Refactoring Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Remove four concrete code quality issues — dead params, misplaced function, over-parameterized internals, and commented-out code — with zero behavioral change.

**Architecture:** Four independent tasks in dependency order: E (trivial deletion) → B (dead params) → A (signature cleanup) → C (function move). Each compiles and passes tests on its own. No new tests are needed — the existing 90 tests serve as the regression suite.

**Tech Stack:** Rust 2024 edition, `cargo test`, `cargo clippy -- -D warnings`

---

## File Map

| File | Change |
|------|--------|
| `src/tmux/favorite.rs` | Delete one commented-out line (E) |
| `src/picker/ui.rs` | Remove `_title` and `_border` parameters (B) |
| `src/picker/mod.rs` | Remove `&title, &border` arguments from `ui::render()` call; trim destructure (B) |
| `src/fzf.rs` | Simplify `invoke_picker` and `invoke_fzf` to 2 params; remove `sort_by_priority` and its `Ordering` imports (A, C) |
| `src/tmux/mod.rs` | Add `sort_by_priority()` (C) |
| `src/main.rs` | Update import from `fzf` to `tmux` for `sort_by_priority` (C) |

---

## Task 1: Delete commented-out code in favorite.rs (E)

**Files:**
- Modify: `src/tmux/favorite.rs:41`

- [ ] **Step 1: Delete the dead line**

In `src/tmux/favorite.rs`, remove line 41 exactly:

```
        //args.push("-P".to_string()); // -P : print the info of the new window to stdout
```

The resulting `switch_window()` body around that area should look like:

```rust
        args.push("-n".to_string());
        args.push(self.name.to_string());

        if let Some(path) = &self.path {
```

- [ ] **Step 2: Verify**

```bash
cargo test
```

Expected: 90 tests pass, no warnings.

- [ ] **Step 3: Commit**

```bash
git add src/tmux/favorite.rs
git commit -m "chore: remove commented-out -P flag in favorite switch_window"
```

---

## Task 2: Remove dead parameters from ui::render() (B)

**Files:**
- Modify: `src/picker/ui.rs:46-54`
- Modify: `src/picker/mod.rs:69` (destructure) and `mod.rs:83` (call site)

- [ ] **Step 1: Remove parameters from ui::render() signature**

In `src/picker/ui.rs`, replace the function signature:

```rust
// Before
pub(crate) fn render(
    frame: &mut Frame,
    state: &PickerState,
    _title: &str,
    _border: &str,
    layout: &str,
    theme: &Theme,
    list_state: &mut ListState,
) {
```

```rust
// After
pub(crate) fn render(
    frame: &mut Frame,
    state: &PickerState,
    layout: &str,
    theme: &Theme,
    list_state: &mut ListState,
) {
```

- [ ] **Step 2: Update the destructure in picker/mod.rs**

In `src/picker/mod.rs`, the `run_loop()` function destructures `config` on line 69. `title` and `border` are now only passed to `ui::render()`, which no longer needs them. Update the destructure to ignore them:

```rust
// Before
let PickerConfig { items, title, border, layout, theme: theme_name, bell_fg } = config;
```

```rust
// After
let PickerConfig { items, layout, theme: theme_name, bell_fg, .. } = config;
```

- [ ] **Step 3: Update the ui::render() call site in picker/mod.rs**

Still in `src/picker/mod.rs`, find the `terminal.draw(...)` call (around line 83):

```rust
// Before
terminal
    .draw(|f| ui::render(f, &state, &title, &border, &layout, &theme, &mut list_state))
    .expect("Failed to draw");
```

```rust
// After
terminal
    .draw(|f| ui::render(f, &state, &layout, &theme, &mut list_state))
    .expect("Failed to draw");
```

- [ ] **Step 4: Verify**

```bash
cargo test
cargo clippy -- -D warnings
```

Expected: 90 tests pass, no clippy warnings.

- [ ] **Step 5: Commit**

```bash
git add src/picker/ui.rs src/picker/mod.rs
git commit -m "refactor: remove unused _title and _border params from ui::render"
```

---

## Task 3: Simplify invoke_picker() and invoke_fzf() signatures (A)

**Files:**
- Modify: `src/fzf.rs`

`invoke_picker()` takes 6 individual params; `invoke_fzf()` takes 4. Both are only called from `dispatch_picker()` which already holds a `&PickerConfig`. Replace all individual params with `config: &PickerConfig`.

- [ ] **Step 1: Replace invoke_picker() signature and body**

In `src/fzf.rs`, replace the entire `invoke_picker()` function (lines 72–141):

```rust
pub(crate) fn invoke_picker(
    item_strings: &[String],
    config: &PickerConfig,
) -> PickerOutput {
    let internal_config = InternalPickerConfig {
        items: item_strings.to_vec(),
        title: config.title.clone(),
        border: config.border.clone(),
        layout: config.layout.clone(),
        theme: config.theme.clone(),
        bell_fg: config.bell_fg.clone(),
    };

    // Serialize config to a temp file for the subprocess.
    let mut items_file = tempfile::NamedTempFile::new().expect("Failed to create items temp file");
    serde_json::to_writer(&items_file, &internal_config).expect("Failed to serialize picker config");
    items_file.flush().expect("Failed to flush items temp file");
    let items_path = items_file.path().to_string_lossy().to_string();

    // Temp file where the inner picker process writes its result.
    let result_file = tempfile::NamedTempFile::new().expect("Failed to create result temp file");
    let result_path = result_file.path().to_string_lossy().to_string();

    // Launch display-popup using the current executable path.
    let exe = std::env::current_exe().expect("Failed to get current executable path");
    let height = std::cmp::min(item_strings.len() + PICKER_HEIGHT_PADDING, MAX_PICKER_HEIGHT);
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
            to_tmux_border(&config.border),
            "-T",
            &format!(" {} ", config.title),
            &popup_cmd,
        ])
        .status()
        .expect("Failed to run tmux display-popup");

    // Read the result written by the inner process.
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
```

- [ ] **Step 2: Replace invoke_fzf() signature and body**

In `src/fzf.rs`, replace the entire `invoke_fzf()` function (lines 143–207):

```rust
/// Runs fzf as the picker backend. bell_fg is not supported in the fzf backend;
/// use `--picker native` for bell row highlighting.
fn invoke_fzf(
    item_strings: &[String],
    config: &PickerConfig,
) -> PickerOutput {
    use std::process::Stdio;

    let height = std::cmp::min(item_strings.len() + FZF_HEIGHT_PADDING, MAX_PICKER_HEIGHT);
    let width = get_terminal_width();

    let input: String = item_strings.iter().cloned().collect();

    let mut child = Command::new("fzf")
        .args([
            "--tmux",
            &format!("{},{}", width, height),
            &format!("--layout={}", config.layout),
            &format!("--border={}", config.border),
            "--border-label",
            &format!(" {} ", config.title),
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
```

- [ ] **Step 3: Update dispatch_picker() call sites**

In `src/fzf.rs`, replace `dispatch_picker()` body (lines 209–225):

```rust
pub(crate) fn dispatch_picker(
    item_strings: &[String],
    config: &PickerConfig,
) -> PickerOutput {
    if config.use_fzf {
        invoke_fzf(item_strings, config)
    } else {
        invoke_picker(item_strings, config)
    }
}
```

- [ ] **Step 4: Verify**

```bash
cargo test
cargo clippy -- -D warnings
```

Expected: 90 tests pass, no clippy warnings.

- [ ] **Step 5: Commit**

```bash
git add src/fzf.rs
git commit -m "refactor: simplify invoke_picker and invoke_fzf to 2-param signatures"
```

---

## Task 4: Move sort_by_priority() to tmux/mod.rs (C)

**Files:**
- Modify: `src/fzf.rs` (remove function + unused imports)
- Modify: `src/tmux/mod.rs` (add function)
- Modify: `src/main.rs` (update import)

- [ ] **Step 1: Add sort_by_priority() to tmux/mod.rs**

In `src/tmux/mod.rs`, add these two imports at the top with the existing `use std::` block:

```rust
use std::cmp::Ordering::Greater;
use std::cmp::Ordering::Less;
```

Then add the function after the `format_window_base()` function (after line 48):

```rust
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
```

- [ ] **Step 2: Remove sort_by_priority() and its imports from fzf.rs**

In `src/fzf.rs`, delete lines 1–2:

```rust
use std::cmp::Ordering::Greater;
use std::cmp::Ordering::Less;
```

And delete the `sort_by_priority()` function (lines 47–56):

```rust
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
```

Also remove the `SortPriority` import from `fzf.rs` line 7 if it is now unused:

```rust
// Remove this line if no longer used elsewhere in fzf.rs:
use super::tmux::SortPriority;
```

- [ ] **Step 3: Update import in main.rs**

In `src/main.rs`, update line 5:

```rust
// Before
use fzf::{select_item, sort_by_priority};
```

```rust
// After
use fzf::select_item;
use tmux::sort_by_priority;
```

- [ ] **Step 4: Verify**

```bash
cargo test
cargo clippy -- -D warnings
```

Expected: 90 tests pass, no clippy warnings.

- [ ] **Step 5: Commit**

```bash
git add src/fzf.rs src/tmux/mod.rs src/main.rs
git commit -m "refactor: move sort_by_priority to tmux/mod.rs alongside SortPriority trait"
```

---

## Success Criteria Checklist

Before declaring complete:

- [ ] `cargo test` passes (90 tests)
- [ ] `cargo clippy -- -D warnings` passes with no warnings
- [ ] `invoke_picker()` and `invoke_fzf()` each take exactly 2 parameters
- [ ] `ui::render()` has no `_`-prefixed parameters
- [ ] `sort_by_priority()` is defined in `src/tmux/mod.rs`
- [ ] No commented-out code in `src/tmux/favorite.rs`
- [ ] No `use std::cmp::Ordering::Greater/Less` or `SortPriority` import left in `src/fzf.rs`
