# Picker TUI Improvements Design — tmux-session-switcher

**Date:** 2026-04-25
**Status:** Approved
**Approach:** Layer-by-layer — Code Quality → UX → Visual

---

## Goals

1. Remove duplicated `list_state.select()` calls and extract constants (code quality)
2. Show a helpful empty-state message when no items match (UX)
3. Truncate long items with `…` to prevent display overflow (UX)
4. Fix cursor position for wide characters (CJK, emoji) in the query (UX)
5. Improve status bar to show selected position (visual)

---

## Layer 1: Code Quality (`src/picker/mod.rs`, `src/picker/ui.rs`)

### list_state deduplication

`list_state.select(Some(state.selected))` appears in four separate branches of the event loop (`MoveUp`, `MoveDown`, `PageUp`, `PageDown`). Move it to after the match block so it runs once per event iteration unconditionally. Since `list_state` must always reflect `state.selected`, this is correct and idempotent.

```rust
// After the match block, before the next loop iteration:
list_state.select(Some(state.selected));
```

### PAGE_SIZE constant

Extract the literal `10` in `page_up(10)` and `page_down(10)` to a module-level constant:

```rust
const PAGE_SIZE: usize = 10;
```

### LAYOUT_REVERSE constant

Extract the string literal `"reverse"` compared in `ui.rs` to a local constant:

```rust
const LAYOUT_REVERSE: &str = "reverse";
```

Using a local constant (rather than importing `args::LayoutStyle`) preserves the IPC serialization format (`PickerConfig.layout: String`) without creating a cross-module dependency.

---

## Layer 2: UX (`src/picker/ui.rs`, `Cargo.toml`)

### Empty state message

When `state.filtered.is_empty()`, render a Paragraph instead of an empty List:

```
  No matches
  Press Enter to create 'query'    ← only shown when query is non-empty
```

Color: `theme.status_fg` (muted gray). If query is empty and no items exist, show only "No matches".

The List widget is replaced with a Paragraph in this branch; the two branches share the same layout constraints so the surrounding prompt/separator/status bar renders unchanged.

### Long item truncation

Before passing item text to `highlight_spans()`, truncate to the available display width using `unicode-width`:

```rust
fn truncate_to_width(s: &str, max_width: usize) -> String {
    // Walk chars, accumulate unicode display width, insert "…" when limit reached
}
```

`unicode-width` must be added to `Cargo.toml` as a direct dependency (it is already a transitive dependency of ratatui, so no version conflict). The truncation width is `inner.width as usize` minus a small margin for the highlight symbol (`"> "` = 2 chars).

Match position indices from `nucleo` are char-based. After truncation, positions beyond the truncation point are dropped to avoid highlighting out-of-bounds characters.

### Wide character cursor

Fix the cursor column calculation to use display width instead of char count:

```rust
// Before
let visual_col = state.query[..state.cursor].chars().count() as u16;

// After
use unicode_width::UnicodeWidthStr;
let visual_col = UnicodeWidthStr::width(&state.query[..state.cursor]) as u16;
```

---

## Layer 3: Visual (`src/picker/ui.rs`)

### Status bar format

Change the status bar text from `  {filtered}/{total}` to `  [{pos}/{filtered}] {total} total`:

```rust
let pos = if state.filtered.is_empty() { 0 } else { state.selected + 1 };
let status_text = format!("  [{}/{}] {} total",
    pos,
    state.filtered.len(),
    state.items.len(),
);
```

`pos` is 1-indexed (human-readable). When the filtered list is empty, it shows `[0/0]`.

---

## New Dependency

```toml
unicode-width = "0.2"
```

Already present transitively via ratatui 0.29. Adding directly pins the version and makes the dependency explicit.

---

## Affected Files

| File | Change |
|------|--------|
| `src/picker/mod.rs` | Move `list_state.select()` out of 4 branches; add `PAGE_SIZE` constant |
| `src/picker/ui.rs` | Empty state rendering; truncation; wide char cursor; status bar format; `LAYOUT_REVERSE` constant |
| `Cargo.toml` | Add `unicode-width` as direct dependency |

---

## What Is Out of Scope

- Mouse support (scroll, click)
- Preview panel (pane content capture)
- `reverse-list` layout variant (exists in args but not handled in ui.rs — separate task)
- Removing `title`/`border` from `picker::PickerConfig` IPC struct (separate cleanup)

---

## Success Criteria

- [ ] `cargo test` passes (90 existing tests)
- [ ] `cargo clippy -- -D warnings` passes
- [ ] `list_state.select()` appears exactly once in `mod.rs`
- [ ] `PAGE_SIZE` constant used for both `page_up` and `page_down`
- [ ] `LAYOUT_REVERSE` constant used in `ui.rs` layout comparison
- [ ] Empty list renders "No matches" message, not blank space
- [ ] Items wider than terminal are truncated with `…`
- [ ] Cursor position correct when query contains CJK or emoji
- [ ] Status bar shows `[pos/filtered] total total` format
