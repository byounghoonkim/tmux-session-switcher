# Picker TUI Improvements Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Improve the native picker TUI with code quality fixes (constants, list_state dedup), UX improvements (empty state message, item truncation, wide-char cursor), and a better status bar format.

**Architecture:** Three sequential tasks. Task 1 cleans up `mod.rs` (pure refactor, no behavioral change). Task 2 adds the `truncate_to_width` helper via TDD and adds the `unicode-width` dependency. Task 3 applies all rendering changes to `ui.rs` in one commit: truncation, empty state, wide-char cursor fix, status bar format, and `LAYOUT_REVERSE` constant.

**Tech Stack:** Rust 2024, `ratatui 0.29`, `unicode-width 0.2` (new direct dependency, already transitive via ratatui), `crossterm`

---

## File Map

| File | Change |
|------|--------|
| `src/picker/mod.rs` | Add `PAGE_SIZE` constant; remove `list_state.select()` from 4 navigation branches, add once after match block |
| `src/picker/ui.rs` | Add `truncate_to_width` helper + tests; apply truncation, empty state, status bar, wide-char cursor, `LAYOUT_REVERSE` constant |
| `Cargo.toml` | Add `unicode-width = "0.2"` as direct dependency |

---

## Task 1: Code quality in mod.rs — PAGE_SIZE constant + list_state dedup

**Files:**
- Modify: `src/picker/mod.rs`

- [ ] **Step 1: Add PAGE_SIZE constant and update page_up / page_down calls**

At the top of `src/picker/mod.rs`, above the `pub(crate) struct PickerConfig` declaration, add:

```rust
const PAGE_SIZE: usize = 10;
```

In `run_loop()`, change:

```rust
Action::PageUp => {
    state.page_up(10);
    list_state.select(Some(state.selected));
}
Action::PageDown => {
    state.page_down(10);
    list_state.select(Some(state.selected));
}
```

to:

```rust
Action::PageUp => state.page_up(PAGE_SIZE),
Action::PageDown => state.page_down(PAGE_SIZE),
```

- [ ] **Step 2: Remove list_state.select() from all four navigation branches**

Change the `MoveUp` and `MoveDown` branches from:

```rust
Action::MoveUp => {
    state.move_up();
    list_state.select(Some(state.selected));
}
Action::MoveDown => {
    state.move_down();
    list_state.select(Some(state.selected));
}
```

to:

```rust
Action::MoveUp => state.move_up(),
Action::MoveDown => state.move_down(),
```

- [ ] **Step 3: Add list_state.select() once after the match block**

The `if let Event::Key(key)` block currently ends after the match. Add one call after the match, still inside the `if let` block:

```rust
        if let Event::Key(key) = event::read().expect("Failed to read event") {
            match key_to_action(key) {
                // ... all arms ...
                Action::Noop => {}
            }
            list_state.select(Some(state.selected));  // ← add this line
        }
```

`Cancel` and `Confirm` arms both `return` early so they never reach this line. Text input arms already call `refilter` which also calls `list_state.select` — the extra call here is idempotent.

- [ ] **Step 4: Verify**

```bash
cargo test
cargo clippy -- -D warnings
```

Expected: 90 tests pass, no warnings.

- [ ] **Step 5: Commit**

```bash
git add src/picker/mod.rs
git commit -m "refactor: extract PAGE_SIZE constant, deduplicate list_state.select call"
```

---

## Task 2: truncate_to_width helper (TDD) + unicode-width dependency

**Files:**
- Modify: `Cargo.toml`
- Modify: `src/picker/ui.rs`

- [ ] **Step 1: Add unicode-width to Cargo.toml**

In `Cargo.toml`, add to `[dependencies]`:

```toml
unicode-width = "0.2"
```

- [ ] **Step 2: Write failing tests for truncate_to_width**

At the bottom of `src/picker/ui.rs`, add:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_short_string_unchanged() {
        assert_eq!(truncate_to_width("hello", 10), "hello");
    }

    #[test]
    fn test_truncate_exact_fit_unchanged() {
        assert_eq!(truncate_to_width("hello", 5), "hello");
    }

    #[test]
    fn test_truncate_ascii_adds_ellipsis() {
        // max=8, target=7: "hello w" fits (7 cols), "hello w…" = 8 display cols
        assert_eq!(truncate_to_width("hello world", 8), "hello w…");
    }

    #[test]
    fn test_truncate_wide_chars() {
        // "你好世界" — each CJK char is 2 cols wide (total 8 cols)
        // max=5, target=4: "你好" = 4 cols fits, next "世" would be 6 → "你好…"
        assert_eq!(truncate_to_width("你好世界", 5), "你好…");
    }

    #[test]
    fn test_truncate_empty_string_unchanged() {
        assert_eq!(truncate_to_width("", 5), "");
    }
}
```

- [ ] **Step 3: Run tests to verify they fail**

```bash
cargo test test_truncate -- --nocapture
```

Expected: compile error — `truncate_to_width` not defined.

- [ ] **Step 4: Implement truncate_to_width in ui.rs**

At the top of `src/picker/ui.rs`, add the import alongside existing imports:

```rust
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};
```

Add the following function before `highlight_spans`:

```rust
fn truncate_to_width(s: &str, max_width: usize) -> String {
    if UnicodeWidthStr::width(s) <= max_width {
        return s.to_string();
    }
    let target = max_width.saturating_sub(1); // reserve 1 col for "…"
    let mut width = 0usize;
    let mut byte_end = 0usize;
    for (byte_pos, ch) in s.char_indices() {
        let cw = UnicodeWidthChar::width(ch).unwrap_or(1);
        if width + cw > target {
            break;
        }
        width += cw;
        byte_end = byte_pos + ch.len_utf8();
    }
    format!("{}…", &s[..byte_end])
}
```

- [ ] **Step 5: Run tests to verify they pass**

```bash
cargo test test_truncate -- --nocapture
```

Expected: 5 tests pass.

- [ ] **Step 6: Run full test suite**

```bash
cargo test
cargo clippy -- -D warnings
```

Expected: 95 tests pass (90 existing + 5 new), no warnings.

- [ ] **Step 7: Commit**

```bash
git add Cargo.toml Cargo.lock src/picker/ui.rs
git commit -m "feat: add truncate_to_width helper with unicode-width support"
```

---

## Task 3: Apply rendering changes to render()

**Files:**
- Modify: `src/picker/ui.rs`

This task applies five rendering improvements to the `render()` function in one commit: truncation, `LAYOUT_REVERSE` constant, wide-char cursor, empty state message, and status bar format.

- [ ] **Step 1: Add LAYOUT_REVERSE constant**

After the `use` statements at the top of `src/picker/ui.rs`, add:

```rust
const LAYOUT_REVERSE: &str = "reverse";
```

- [ ] **Step 2: Replace the render() function body**

Replace the entire `pub(crate) fn render(...)` function with:

```rust
pub(crate) fn render(
    frame: &mut Frame,
    state: &PickerState,
    layout: &str,
    theme: &Theme,
    list_state: &mut ListState,
) {
    let area = frame.area();
    let inner = Block::default().inner(area);

    // Prompt area: ">" + current query
    let prompt_text = format!("> {}", state.query);
    let prompt = Paragraph::new(prompt_text).style(Style::default().fg(theme.prompt_fg));

    // Separator line
    let sep_char = "─".repeat(inner.width as usize);
    let sep = Paragraph::new(sep_char).style(Style::default().fg(theme.separator_fg));

    // Available width for item text (inner width minus 2 for "> " highlight symbol)
    let item_width = (inner.width as usize).saturating_sub(2);

    // List items with fuzzy match character highlighting.
    // List::highlight_style overwrites Span styles, so we apply highlight_bg/fg directly to each Span.
    let selected_rank = list_state.selected().unwrap_or(0);
    let list_items: Vec<ListItem> = state
        .filtered
        .iter()
        .enumerate()
        .map(|(rank, &i)| {
            let raw_text = state.items[i].trim_end();
            let text = truncate_to_width(raw_text, item_width);
            // Drop match positions beyond the visible (non-ellipsis) chars
            let visible_chars = if text.ends_with('…') {
                text.chars().count().saturating_sub(1)
            } else {
                text.chars().count()
            };
            let raw_positions = state.match_indices.get(rank).map(|v| v.as_slice()).unwrap_or(&[]);
            let filtered_positions: Vec<u32> = raw_positions
                .iter()
                .copied()
                .filter(|&p| (p as usize) < visible_chars)
                .collect();
            let is_bell = raw_text.contains('🔔');
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
            let line = highlight_spans(&text, &filtered_positions, normal_style, match_style);
            ListItem::new(line)
        })
        .collect();

    let list = List::new(list_items).highlight_symbol("> ");

    // Empty state: shown instead of the list when nothing matches
    let empty_msg = {
        let mut lines = vec![Line::from(Span::styled(
            "  No matches",
            Style::default().fg(theme.status_fg),
        ))];
        if !state.query.is_empty() {
            lines.push(Line::from(Span::styled(
                format!("  Press Enter to create '{}'", state.query),
                Style::default().fg(theme.status_fg),
            )));
        }
        Paragraph::new(lines)
    };

    // Status bar: [selected_pos/filtered_count] total_count total
    let pos = if state.filtered.is_empty() { 0 } else { state.selected + 1 };
    let status_text = format!(
        "  [{}/{}] {} total",
        pos,
        state.filtered.len(),
        state.items.len(),
    );
    let status = Paragraph::new(status_text).style(Style::default().fg(theme.status_fg));

    let prompt_chunk = if layout == LAYOUT_REVERSE {
        // reverse: list at top, prompt at bottom
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // status
                Constraint::Min(1),    // list or empty msg
                Constraint::Length(1), // separator
                Constraint::Length(1), // prompt
            ])
            .split(inner);

        frame.render_widget(status, chunks[0]);
        if state.filtered.is_empty() {
            frame.render_widget(empty_msg, chunks[1]);
        } else {
            frame.render_stateful_widget(list, chunks[1], list_state);
        }
        frame.render_widget(sep, chunks[2]);
        frame.render_widget(prompt, chunks[3]);
        chunks[3]
    } else {
        // default: prompt at top, list at bottom
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // prompt
                Constraint::Length(1), // separator
                Constraint::Min(1),    // list or empty msg
                Constraint::Length(1), // status
            ])
            .split(inner);

        frame.render_widget(prompt, chunks[0]);
        frame.render_widget(sep, chunks[1]);
        if state.filtered.is_empty() {
            frame.render_widget(empty_msg, chunks[2]);
        } else {
            frame.render_stateful_widget(list, chunks[2], list_state);
        }
        frame.render_widget(status, chunks[3]);
        chunks[0]
    };

    // Wide-char-aware cursor position
    let visual_col = UnicodeWidthStr::width(&state.query[..state.cursor]) as u16;
    frame.set_cursor_position((prompt_chunk.x + 2 + visual_col, prompt_chunk.y));
}
```

- [ ] **Step 3: Verify compilation and tests**

```bash
cargo test
cargo clippy -- -D warnings
```

Expected: 95 tests pass, no warnings.

- [ ] **Step 4: Commit**

```bash
git add src/picker/ui.rs
git commit -m "feat: picker TUI — truncation, empty state, wide-char cursor, status bar"
```

---

## Success Criteria Checklist

- [ ] `cargo test` passes (≥ 95 tests: 90 existing + 5 truncation tests)
- [ ] `cargo clippy -- -D warnings` passes with no warnings
- [ ] `list_state.select()` appears exactly once in `mod.rs` event loop
- [ ] `PAGE_SIZE` constant used for `page_up` and `page_down`
- [ ] `LAYOUT_REVERSE` constant used in `ui.rs` layout comparison
- [ ] `truncate_to_width` handles ASCII, CJK, and empty strings correctly
- [ ] Items wider than terminal are truncated with `…` in the list
- [ ] Empty filtered list renders "No matches" (and Enter hint when query non-empty)
- [ ] Cursor position calculated with `UnicodeWidthStr::width`, not `chars().count()`
- [ ] Status bar shows `[pos/filtered] total total` format
