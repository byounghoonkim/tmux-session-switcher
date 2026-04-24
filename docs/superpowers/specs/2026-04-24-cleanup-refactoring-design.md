# Cleanup Refactoring Design — tmux-session-switcher

**Date:** 2026-04-24
**Status:** Approved
**Approach:** Targeted surface cleanup — no structural changes, no new features

---

## Goals

1. Remove parameter proliferation in `invoke_picker()` and `invoke_fzf()` (A)
2. Delete dead parameters `_title` and `_border` from `ui::render()` (B)
3. Move `sort_by_priority()` to `tmux/mod.rs` where its trait lives (C)
4. Delete commented-out dead code in `favorite.rs` (E)

---

## Change A: invoke_picker / invoke_fzf signature

**File:** `src/fzf.rs`

`invoke_picker()` currently takes 6 individual parameters despite `dispatch_picker()` already accepting `&PickerConfig`. Change to 2 parameters:

```rust
// Before
fn invoke_picker(
    item_strings: &[String],
    title: &str,
    border: &str,
    layout: &str,
    theme: &str,
    bell_fg: Option<String>,
) -> PickerOutput

// After
fn invoke_picker(
    item_strings: &[String],
    config: &PickerConfig,
) -> PickerOutput
```

`invoke_fzf()` receives the same treatment:

```rust
// Before
fn invoke_fzf(
    item_strings: &[String],
    title: &str,
    border: &str,
    layout: &str,
) -> PickerOutput

// After
fn invoke_fzf(
    item_strings: &[String],
    config: &PickerConfig,
) -> PickerOutput
```

`item_strings` remains a separate parameter because `PickerConfig` does not carry items (items are computed at the call site and only exist for the duration of the picker invocation).

`dispatch_picker()` call sites become:

```rust
invoke_fzf(item_strings, config)
invoke_picker(item_strings, config)
```

---

## Change B: Remove dead parameters from ui::render()

**File:** `src/picker/ui.rs`, `src/picker/mod.rs`

`_title: &str` and `_border: &str` are unused in `ui::render()`. The title and border are applied by the outer `tmux display-popup` call, not by ratatui. Remove both parameters.

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
)

// After
pub(crate) fn render(
    frame: &mut Frame,
    state: &PickerState,
    layout: &str,
    theme: &Theme,
    list_state: &mut ListState,
)
```

The call in `picker/mod.rs` drops the two corresponding arguments.

---

## Change C: Move sort_by_priority() to tmux/mod.rs

**Files:** `src/fzf.rs`, `src/tmux/mod.rs`, `src/main.rs`

`sort_by_priority()` operates on `SortPriority` implementors, which is a trait defined in `tmux/mod.rs`. Moving the function there co-locates the trait and its primary utility.

```rust
// Removed from src/fzf.rs, moved verbatim to src/tmux/mod.rs
pub(crate) fn sort_by_priority<T: SortPriority + ?Sized>(items: &mut [Box<T>]) {
    items.sort_by(|a, b| {
        if a.sort_priority() > b.sort_priority() {
            return std::cmp::Ordering::Greater;
        } else if a.sort_priority() < b.sort_priority() {
            return std::cmp::Ordering::Less;
        }
        std::cmp::Ordering::Equal
    });
}
```

The implementation is moved verbatim — no behavioral change.

`main.rs` import changes:

```rust
// Before
use fzf::{select_item, sort_by_priority};

// After
use fzf::select_item;
use tmux::sort_by_priority;
```

---

## Change E: Delete commented-out code in favorite.rs

**File:** `src/tmux/favorite.rs`

Remove line 41:

```rust
//args.push("-P".to_string()); // -P : print the info of the new window to stdout
```

---

## What Is Out of Scope

- Unifying `fzf::PickerConfig` and `picker::PickerConfig` — IPC serialization boundary makes this risky
- Error handling for `get_current_session()` / `get_current_window()` — panic is acceptable inside tmux
- Any new features or behavioral changes

---

## Affected Files

| File | Change |
|------|--------|
| `src/fzf.rs` | Simplify `invoke_picker` / `invoke_fzf` signatures, remove `sort_by_priority` |
| `src/picker/mod.rs` | Update `ui::render()` call site |
| `src/picker/ui.rs` | Remove `_title` and `_border` parameters |
| `src/tmux/mod.rs` | Add `sort_by_priority()` |
| `src/tmux/favorite.rs` | Delete commented-out line |
| `src/main.rs` | Update `sort_by_priority` import |

---

## Success Criteria

- [ ] `cargo test` passes (90 existing tests, no new tests needed)
- [ ] `cargo clippy -- -D warnings` passes with no warnings
- [ ] `invoke_picker()` and `invoke_fzf()` each take 2 parameters
- [ ] `ui::render()` has no `_`-prefixed parameters
- [ ] `sort_by_priority()` is in `tmux/mod.rs`
- [ ] No commented-out code in `favorite.rs`
