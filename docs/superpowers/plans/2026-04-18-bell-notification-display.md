# Bell Notification Display Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Display `window_bell_flag` windows in the picker with a 🔔 icon and `bell_fg` color, sortable to the top, with per-theme defaults and `config.toml` override.

**Architecture:** Add `bell: bool` to `Window`, thread `bell_fg: Option<String>` through `Config` → `PickerConfig` → `Theme`, and apply the color in `ui.rs` when a row contains `🔔`.

**Tech Stack:** Rust, ratatui, serde, toml

---

## File Map

| File | Change |
|------|--------|
| `src/tmux/window.rs` | Add `bell` field; update `Display`, `SortPriority` |
| `src/tmux/mod.rs` | Extend tmux format string + regex; set `bell` on `Window` |
| `src/picker/theme.rs` | Add `bell_fg: Color`; add `parse_hex_color` helper |
| `src/config.rs` | Add `bell_fg: Option<String>` |
| `src/picker/mod.rs` | Add `bell_fg` to `PickerConfig`; apply override after `Theme::from_name` |
| `src/fzf.rs` | Thread `bell_fg` through `invoke_picker`, `dispatch_picker`, `select_item` |
| `src/main.rs` | Pass `config.bell_fg` to `select_item`; pass `None` for favorite picker |

---

## Task 1: Add `bell` field to `Window` and update tmux parsing

**Files:**
- Modify: `src/tmux/window.rs`
- Modify: `src/tmux/mod.rs`

- [ ] **Step 1: Write failing tests in `window.rs`**

Add at the bottom of `src/tmux/window.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn make_window(active: bool, marked: bool, bell: bool) -> Window {
        Window {
            session_name: "work".to_string(),
            index: "1".to_string(),
            name: "editor".to_string(),
            active,
            marked,
            bell,
        }
    }

    #[test]
    fn test_display_bell_shows_icon() {
        let w = make_window(false, false, true);
        assert!(w.to_string().contains("🔔"));
    }

    #[test]
    fn test_display_no_bell_no_icon() {
        let w = make_window(false, false, false);
        assert!(!w.to_string().contains("🔔"));
    }

    #[test]
    fn test_sort_priority_bell_is_1() {
        let w = make_window(false, false, true);
        assert_eq!(w.sort_priority(), 1.0);
    }

    #[test]
    fn test_sort_priority_active_beats_bell() {
        let w = make_window(true, false, true);
        assert_eq!(w.sort_priority(), 0.0);
    }

    #[test]
    fn test_sort_priority_bell_beats_marked() {
        let bell_w = make_window(false, false, true);
        let marked_w = make_window(false, true, false);
        assert!(bell_w.sort_priority() < marked_w.sort_priority());
    }
}
```

- [ ] **Step 2: Run tests to confirm they fail**

```bash
cargo test -q 2>&1 | grep -E "FAILED|error"
```

Expected: compile error — `bell` field not found on `Window`.

- [ ] **Step 3: Add `bell` field to `Window` struct and update impls**

Replace the entire content of `src/tmux/window.rs` with:

```rust
use std::process::Command;

use crate::tmux::SortPriority;
use crate::tmux::Switchable;
use crate::tmux::TMUX;

#[derive(Clone)]
pub(crate) struct Window {
    pub(crate) session_name: String,
    pub(crate) index: String,
    pub(crate) name: String,
    pub(crate) active: bool,
    pub(crate) marked: bool,
    pub(crate) bell: bool,
}

impl Switchable for Window {
    fn switch_window(&self) {
        Command::new(TMUX)
            .args([
                "switch",
                "-t",
                &format!("{}:{}", self.session_name, self.index,),
            ])
            .status()
            .expect("Failed to execute tmux switch");
    }
}

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

impl SortPriority for Window {
    fn sort_priority(&self) -> f32 {
        if self.active {
            return 0.0;
        }
        if self.bell {
            return 1.0;
        }
        if self.marked {
            return 2.0;
        }
        3.0
    }
}

impl crate::tmux::Item for Window {
    fn session_name(&self) -> String {
        self.session_name.clone()
    }

    fn index(&self) -> String {
        self.index.clone()
    }

    fn name(&self) -> String {
        self.name.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_window(active: bool, marked: bool, bell: bool) -> Window {
        Window {
            session_name: "work".to_string(),
            index: "1".to_string(),
            name: "editor".to_string(),
            active,
            marked,
            bell,
        }
    }

    #[test]
    fn test_display_bell_shows_icon() {
        let w = make_window(false, false, true);
        assert!(w.to_string().contains("🔔"));
    }

    #[test]
    fn test_display_no_bell_no_icon() {
        let w = make_window(false, false, false);
        assert!(!w.to_string().contains("🔔"));
    }

    #[test]
    fn test_sort_priority_bell_is_1() {
        let w = make_window(false, false, true);
        assert_eq!(w.sort_priority(), 1.0);
    }

    #[test]
    fn test_sort_priority_active_beats_bell() {
        let w = make_window(true, false, true);
        assert_eq!(w.sort_priority(), 0.0);
    }

    #[test]
    fn test_sort_priority_bell_beats_marked() {
        let bell_w = make_window(false, false, true);
        let marked_w = make_window(false, true, false);
        assert!(bell_w.sort_priority() < marked_w.sort_priority());
    }
}
```

- [ ] **Step 4: Update `get_running_windows` in `src/tmux/mod.rs`**

Replace the `fields` const and the `windows.push` block. Find this section:

```rust
    let fields = concat!(
        "#{session_name}|",
        "#{window_index}|",
        "#{window_name}|",
        "#{window_active}|",
        "#{window_marked_flag}|"
    );
```

Replace with:

```rust
    let fields = concat!(
        "#{session_name}|",
        "#{window_index}|",
        "#{window_name}|",
        "#{window_active}|",
        "#{window_marked_flag}|",
        "#{window_bell_flag}|"
    );
```

Then find:

```rust
    let re = Regex::new(r"([^|]+)\|([^|]+)\|([^|]+)\|([^|]+)\|([^|]+)").unwrap();
```

Replace with:

```rust
    let re = Regex::new(r"([^|]+)\|([^|]+)\|([^|]+)\|([^|]+)\|([^|]+)\|([^|]+)").unwrap();
```

Then find the `windows.push(window::Window {` block:

```rust
            windows.push(window::Window {
                session_name: captures[1].to_string(),
                index: captures[2].to_string(),
                name: captures[3].to_string(),
                active: &captures[4] == "1" && &captures[1] == current_session,
                marked: &captures[5] == "1",
            });
```

Replace with:

```rust
            windows.push(window::Window {
                session_name: captures[1].to_string(),
                index: captures[2].to_string(),
                name: captures[3].to_string(),
                active: &captures[4] == "1" && &captures[1] == current_session,
                marked: &captures[5] == "1",
                bell: &captures[6] == "1",
            });
```

- [ ] **Step 5: Run tests to confirm they pass**

```bash
cargo test -q 2>&1 | tail -5
```

Expected: all tests pass, no errors.

- [ ] **Step 6: Commit**

```bash
git add src/tmux/window.rs src/tmux/mod.rs
git commit -m "feat: add bell field to Window with display icon and sort priority"
```

---

## Task 2: Add `bell_fg` to `Theme` and `parse_hex_color` helper

**Files:**
- Modify: `src/picker/theme.rs`

- [ ] **Step 1: Write failing tests**

Add at the bottom of `src/picker/theme.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hex_color_with_hash() {
        assert_eq!(parse_hex_color("#ff8c00"), Some(Color::Rgb(255, 140, 0)));
    }

    #[test]
    fn test_parse_hex_color_without_hash() {
        assert_eq!(parse_hex_color("ff8c00"), Some(Color::Rgb(255, 140, 0)));
    }

    #[test]
    fn test_parse_hex_color_invalid() {
        assert_eq!(parse_hex_color("zzzzzz"), None);
        assert_eq!(parse_hex_color("#fff"), None);
    }

    #[test]
    fn test_all_themes_have_bell_fg() {
        for name in &["catppuccin", "nord", "gruvbox", "tokyo-night", "solarized-dark", "default"] {
            let theme = Theme::from_name(name);
            // bell_fg must not be Reset (i.e. it should be a real color)
            assert_ne!(theme.bell_fg, Color::Reset, "Theme {} has Reset bell_fg", name);
        }
    }
}
```

- [ ] **Step 2: Run tests to confirm they fail**

```bash
cargo test -q 2>&1 | grep -E "FAILED|error"
```

Expected: compile error — `bell_fg` field not found on `Theme`, `parse_hex_color` not found.

- [ ] **Step 3: Update `src/picker/theme.rs`**

Replace the entire file with:

```rust
use ratatui::style::Color;

pub(crate) struct Theme {
    pub prompt_fg: Color,
    pub separator_fg: Color,
    pub status_fg: Color,
    pub highlight_bg: Color,
    pub highlight_fg: Color,
    pub item_fg: Color,
    pub match_fg: Color,
    pub bell_fg: Color,
}

pub(crate) fn parse_hex_color(s: &str) -> Option<Color> {
    let s = s.trim_start_matches('#');
    if s.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&s[0..2], 16).ok()?;
    let g = u8::from_str_radix(&s[2..4], 16).ok()?;
    let b = u8::from_str_radix(&s[4..6], 16).ok()?;
    Some(Color::Rgb(r, g, b))
}

impl Theme {
    pub(crate) fn from_name(name: &str) -> Self {
        match name {
            "catppuccin" | "catppuccin-mocha" => Self::catppuccin_mocha(),
            "nord" => Self::nord(),
            "gruvbox" => Self::gruvbox(),
            "tokyo-night" | "tokyonight" => Self::tokyo_night(),
            "solarized" | "solarized-dark" => Self::solarized_dark(),
            _ => Self::default_theme(),
        }
    }

    fn catppuccin_mocha() -> Self {
        Self {
            prompt_fg: Color::Rgb(137, 180, 250),   // Blue
            separator_fg: Color::Rgb(108, 112, 134), // Overlay0
            status_fg: Color::Rgb(166, 173, 200),    // Subtext0
            highlight_bg: Color::Rgb(69, 71, 90),    // Surface1
            highlight_fg: Color::Rgb(203, 166, 247), // Mauve
            item_fg: Color::Rgb(205, 214, 244),      // Text
            match_fg: Color::Rgb(249, 226, 175),     // Yellow
            bell_fg: Color::Rgb(250, 179, 135),      // Peach
        }
    }

    fn nord() -> Self {
        Self {
            prompt_fg: Color::Rgb(136, 192, 208),    // nord8
            separator_fg: Color::Rgb(76, 86, 106),   // nord3
            status_fg: Color::Rgb(97, 110, 136),     // nord3/4 사이
            highlight_bg: Color::Rgb(59, 66, 82),    // nord1
            highlight_fg: Color::Rgb(136, 192, 208), // nord8
            item_fg: Color::Rgb(216, 222, 233),      // nord4
            match_fg: Color::Rgb(235, 203, 139),     // nord13 yellow
            bell_fg: Color::Rgb(208, 135, 112),      // nord12 aurora orange
        }
    }

    fn gruvbox() -> Self {
        Self {
            prompt_fg: Color::Rgb(131, 165, 152),   // aqua
            separator_fg: Color::Rgb(102, 92, 84),  // bg4
            status_fg: Color::Rgb(146, 131, 116),   // gray
            highlight_bg: Color::Rgb(60, 56, 54),   // bg1
            highlight_fg: Color::Rgb(250, 189, 47), // yellow
            item_fg: Color::Rgb(235, 219, 178),     // fg1
            match_fg: Color::Rgb(254, 128, 25),     // orange
            bell_fg: Color::Rgb(251, 73, 52),       // bright red
        }
    }

    fn tokyo_night() -> Self {
        Self {
            prompt_fg: Color::Rgb(122, 162, 247),    // blue
            separator_fg: Color::Rgb(65, 72, 104),   // overlay
            status_fg: Color::Rgb(86, 95, 137),      // comment
            highlight_bg: Color::Rgb(36, 40, 59),    // surface
            highlight_fg: Color::Rgb(187, 154, 247), // purple
            item_fg: Color::Rgb(192, 202, 245),      // text
            match_fg: Color::Rgb(224, 175, 104),     // yellow
            bell_fg: Color::Rgb(255, 158, 100),      // orange
        }
    }

    fn solarized_dark() -> Self {
        Self {
            prompt_fg: Color::Rgb(38, 139, 210),    // blue
            separator_fg: Color::Rgb(88, 110, 117), // base01
            status_fg: Color::Rgb(101, 123, 131),   // base00
            highlight_bg: Color::Rgb(7, 54, 66),    // base02
            highlight_fg: Color::Rgb(42, 161, 152), // cyan
            item_fg: Color::Rgb(131, 148, 150),     // base0
            match_fg: Color::Rgb(181, 137, 0),      // yellow
            bell_fg: Color::Rgb(203, 75, 22),       // orange
        }
    }

    fn default_theme() -> Self {
        Self {
            prompt_fg: Color::Reset,
            separator_fg: Color::DarkGray,
            status_fg: Color::DarkGray,
            highlight_bg: Color::Blue,
            highlight_fg: Color::White,
            item_fg: Color::Reset,
            match_fg: Color::Yellow,
            bell_fg: Color::Yellow,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hex_color_with_hash() {
        assert_eq!(parse_hex_color("#ff8c00"), Some(Color::Rgb(255, 140, 0)));
    }

    #[test]
    fn test_parse_hex_color_without_hash() {
        assert_eq!(parse_hex_color("ff8c00"), Some(Color::Rgb(255, 140, 0)));
    }

    #[test]
    fn test_parse_hex_color_invalid() {
        assert_eq!(parse_hex_color("zzzzzz"), None);
        assert_eq!(parse_hex_color("#fff"), None);
    }

    #[test]
    fn test_all_themes_have_bell_fg() {
        for name in &["catppuccin", "nord", "gruvbox", "tokyo-night", "solarized-dark", "default"] {
            let theme = Theme::from_name(name);
            assert_ne!(theme.bell_fg, Color::Reset, "Theme {} has Reset bell_fg", name);
        }
    }
}
```

- [ ] **Step 4: Run tests to confirm they pass**

```bash
cargo test -q 2>&1 | tail -5
```

Expected: all tests pass.

- [ ] **Step 5: Commit**

```bash
git add src/picker/theme.rs
git commit -m "feat: add bell_fg to Theme with per-theme defaults and parse_hex_color helper"
```

---

## Task 3: Add `bell_fg` to `Config` and thread through `PickerConfig` + function signatures

**Files:**
- Modify: `src/config.rs`
- Modify: `src/picker/mod.rs`
- Modify: `src/fzf.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: Write failing test for config `bell_fg`**

Add to the `tests` module in `src/config.rs`:

```rust
    #[test]
    fn test_config_bell_fg_field() {
        let path = temp_path("bell_fg_field");
        let config = Config {
            favorites: None,
            picker: None,
            theme: None,
            bell_fg: Some("#ff8c00".to_string()),
        };
        config.save(&path);
        let loaded = Config::new(&path);
        assert_eq!(loaded.bell_fg, Some("#ff8c00".to_string()));
        std::fs::remove_file(&path).ok();
    }
```

- [ ] **Step 2: Run test to confirm it fails**

```bash
cargo test -q 2>&1 | grep -E "FAILED|error"
```

Expected: compile error — `bell_fg` not a field of `Config`.

- [ ] **Step 3: Add `bell_fg` to `Config` struct**

In `src/config.rs`, replace the `Config` struct and `Config::new` empty-return:

```rust
#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub favorites: Option<Vec<Favorite>>,
    pub picker: Option<String>,
    pub theme: Option<String>,
    pub bell_fg: Option<String>,
}
```

And update the early-return in `Config::new`:

```rust
        if contents.is_empty() {
            return Config { favorites: None, picker: None, theme: None, bell_fg: None };
        }
```

- [ ] **Step 4: Run config tests to confirm they pass**

```bash
cargo test config:: -q 2>&1 | tail -5
```

Expected: all config tests pass.

- [ ] **Step 5: Add `bell_fg` to `PickerConfig` in `src/picker/mod.rs`**

Replace the `PickerConfig` struct:

```rust
#[derive(Serialize, Deserialize)]
pub(crate) struct PickerConfig {
    pub items: Vec<String>,
    pub title: String,
    pub border: String,
    pub layout: String,
    pub theme: String,
    pub bell_fg: Option<String>,
}
```

In `run_loop`, replace the destructure and theme construction:

```rust
    let PickerConfig { items, title, border, layout, theme: theme_name, bell_fg } = config;
    let mut theme = Theme::from_name(&theme_name);
    if let Some(ref hex) = bell_fg {
        if let Some(color) = theme::parse_hex_color(hex) {
            theme.bell_fg = color;
        }
    }
```

- [ ] **Step 6: Update `invoke_picker` in `src/fzf.rs` to accept and forward `bell_fg`**

Replace the `invoke_picker` signature and `PickerConfig` construction:

```rust
pub(crate) fn invoke_picker(
    item_strings: &[String],
    title: &str,
    border: &str,
    layout: &str,
    theme: &str,
    bell_fg: Option<String>,
) -> PickerOutput {
    let config = PickerConfig {
        items: item_strings.to_vec(),
        title: title.to_string(),
        border: border.to_string(),
        layout: layout.to_string(),
        theme: theme.to_string(),
        bell_fg,
    };
    // rest of function unchanged
```

Replace `dispatch_picker` signature and its call to `invoke_picker`:

```rust
pub(crate) fn dispatch_picker(
    item_strings: &[String],
    title: &str,
    border: &str,
    layout: &str,
    use_fzf: bool,
    theme: &str,
    bell_fg: Option<String>,
) -> PickerOutput {
    if use_fzf {
        invoke_fzf(item_strings, title, border, layout)
    } else {
        invoke_picker(item_strings, title, border, layout, theme, bell_fg)
    }
}
```

Replace `select_item` signature and its call to `dispatch_picker`:

```rust
pub(crate) fn select_item<'a, T: Display + ?Sized>(
    items: &'a [Box<T>],
    title: &str,
    border: &str,
    layout: &str,
    use_fzf: bool,
    theme: &str,
    bell_fg: Option<String>,
) -> SelectItemReturn<'a, Box<T>> {
    let item_strings: Vec<String> = items.iter().map(|w| w.to_string()).collect();

    match dispatch_picker(&item_strings, title, border, layout, use_fzf, theme, bell_fg) {
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

- [ ] **Step 7: Update callers in `src/main.rs`**

In `remove_favorite_interactive`, update the `dispatch_picker` call:

```rust
    match fzf::dispatch_picker(&item_strings, "Remove Favorite", "rounded", "default", use_fzf, theme, None) {
```

In `main`, extract `bell_fg` from config before the partial move and update `select_item`:

After the line `let effective_theme = ...`, add:

```rust
    let bell_fg = config.bell_fg.clone();
```

Then update the `select_item` call:

```rust
    match select_item(
        &ws,
        &args.title,
        &args.border.to_string(),
        &args.layout.to_string(),
        effective_use_fzf,
        &effective_theme,
        bell_fg,
    ) {
```

- [ ] **Step 8: Build to confirm it compiles cleanly**

```bash
cargo build 2>&1 | grep -E "^error"
```

Expected: no errors.

- [ ] **Step 9: Run all tests**

```bash
cargo test -q 2>&1 | tail -5
```

Expected: all tests pass.

- [ ] **Step 10: Commit**

```bash
git add src/config.rs src/picker/mod.rs src/fzf.rs src/main.rs
git commit -m "feat: thread bell_fg through Config, PickerConfig, and picker function signatures"
```

---

## Task 4: Apply `bell_fg` color in `ui.rs` for bell rows

**Files:**
- Modify: `src/picker/ui.rs`

- [ ] **Step 1: Update the list item style logic in `src/picker/ui.rs`**

In the `render` function, find the block that computes `(normal_style, match_style)`:

```rust
            let (normal_style, match_style) = if rank == selected_rank {
                (
                    Style::default().fg(theme.highlight_fg).bg(theme.highlight_bg),
                    Style::default()
                        .fg(theme.match_fg)
                        .bg(theme.highlight_bg)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                (
                    Style::default().fg(theme.item_fg),
                    Style::default().fg(theme.match_fg).add_modifier(Modifier::BOLD),
                )
            };
```

Replace with:

```rust
            let is_bell = text.contains('🔔');
            let (normal_style, match_style) = if rank == selected_rank {
                let fg = if is_bell { theme.bell_fg } else { theme.highlight_fg };
                (
                    Style::default().fg(fg).bg(theme.highlight_bg),
                    Style::default()
                        .fg(theme.match_fg)
                        .bg(theme.highlight_bg)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                let fg = if is_bell { theme.bell_fg } else { theme.item_fg };
                (
                    Style::default().fg(fg),
                    Style::default().fg(theme.match_fg).add_modifier(Modifier::BOLD),
                )
            };
```

- [ ] **Step 2: Build to confirm it compiles**

```bash
cargo build 2>&1 | grep "^error"
```

Expected: no output (clean build).

- [ ] **Step 3: Run all tests**

```bash
cargo test -q 2>&1 | tail -5
```

Expected: all tests pass.

- [ ] **Step 4: Commit**

```bash
git add src/picker/ui.rs
git commit -m "feat: apply bell_fg color to bell rows in picker UI"
```

---

## Final Verification

- [ ] **Run full test suite**

```bash
cargo test 2>&1 | tail -10
```

Expected: all tests pass, zero failures.

- [ ] **Build release binary**

```bash
cargo build --release 2>&1 | grep "^error"
```

Expected: no errors.
