# Refactoring Design — tmux-session-switcher

**Date:** 2026-04-19  
**Status:** Approved  
**Approach:** Layer-by-layer (Infrastructure → Domain → Quality → Tests → Docs)

---

## Goals

1. Improve error handling at system boundaries (tmux commands, config parsing)
2. Reduce parameter proliferation via `PickerConfig` struct
3. Fix code quality issues (Korean comments, magic numbers, duplication, regex compilation)
4. Add unit and integration tests for uncovered paths
5. Update README, add CLAUDE.md, mark completed specs as implemented

---

## Layer 1: Infrastructure (`tmux/`, `config.rs`, `utils.rs`)

### Error Handling

Functions that execute tmux commands or perform I/O return `Result<T, String>` instead of panicking via `.expect()`. No external error libraries are added — `std::result::Result` with `String` errors is sufficient.

**Affected functions in `tmux/mod.rs`:**
- `switch_to_window()` → `Result<(), String>`
- `run_command()` (if extracted) → `Result<String, String>`
- `get_running_windows()` → `Result<Vec<Window>, String>`

**In `main.rs`:** Callers handle errors with `eprintln!` + `process::exit(1)`.

**Out of scope:** Internal picker logic, sorting, filtering — these operate on already-validated data and `.expect()` is acceptable there.

### Regex Pre-compilation

`get_running_windows()` currently calls `Regex::new(...)` on every invocation. Replace with `std::sync::OnceLock<Regex>` (stable since Rust 1.70, no new dependency).

```rust
static WINDOW_RE: OnceLock<Regex> = OnceLock::new();
let re = WINDOW_RE.get_or_init(|| Regex::new(r"([^|]+)\|...").unwrap());
```

### Config Directory Consolidation

`tmux/mod.rs` (`get_previous_window_path`) and `config.rs` (`save`) both create `~/.config/tmux-session-switcher/`. Extract to `utils.rs`:

```rust
pub fn get_config_dir() -> PathBuf { ... }
```

Both callers use this shared function.

---

## Layer 2: Domain (`fzf.rs`, `main.rs`)

### PickerConfig Struct

Replace 7-argument `dispatch_picker()` and `select_item()` signatures with a `PickerConfig` struct:

```rust
pub struct PickerConfig {
    pub title: String,
    pub border: BorderStyle,
    pub layout: Layout,
    pub use_fzf: bool,
    pub theme: Theme,
    pub bell_fg: Option<Color>,
}
```

Callers build a `PickerConfig` and pass it by reference. The internal `picker::PickerConfig` (already exists) is unified or aliased to avoid duplication.

### fzf Backend and bell_fg

The fzf backend silently ignores `bell_fg`. This is intentional — fzf has its own color system and conversion would add complexity with low benefit. The behavior difference is made explicit:

- A doc comment on `invoke_fzf()` states: "bell_fg is not supported in the fzf backend; use --picker native for bell highlighting."
- README documents this difference in the fzf vs native comparison table.

---

## Layer 3: Code Quality

### Korean Comments → English

All Korean-language comments in `picker/ui.rs`, `picker/mod.rs`, and `fzf.rs` are translated to English.

### Display Format Helper

`Window`, `Favorite`, and `PreviousWindow` share a common column layout (`session:15 - index:3 - name`). Extract to:

```rust
// tmux/mod.rs or tmux/display.rs
pub fn format_window_base(session: &str, index: usize, name: &str) -> String {
    format!("{:15} - {:3} - {}", session, index, name)
}
```

Each `Display` impl calls this helper and appends its own suffix (icons, labels).

### Magic Numbers

```rust
// picker/mod.rs and fzf.rs
const PICKER_HEIGHT_PADDING: usize = 6;  // prompt + border + status bar
const FZF_HEIGHT_PADDING: usize = 5;     // fzf header + border
const MAX_PICKER_HEIGHT: usize = 40;
```

---

## Layer 4: Tests

### New Unit Tests

| Location | What to test |
|----------|-------------|
| `fzf.rs` | `PickerConfig` default values, field access |
| `utils.rs` | `get_config_dir()` returns valid path |
| `tmux/mod.rs` | `format_window_base()` output format |
| `tmux/mod.rs` | Error path when tmux command fails |
| `picker/theme.rs` | Already covered; verify bell_fg defaults per theme |

### Integration Tests (`tests/`)

```
tests/
  integration_test.rs   # End-to-end: build item list → picker config → selection
  tmux_mock.rs          # Mock implementations of Switchable trait
```

Integration tests use mock `Switchable` implementations (no real tmux required). They verify:
- Correct session/window is selected given picker output
- Favorite add/remove/list flows
- Previous window tracking persists correctly

---

## Layer 5: Documentation

### README.md

Expand from minimal to full user guide:
- Installation (cargo install, from source)
- Requirements (tmux version, fzf optional)
- All CLI flags with descriptions
- Config file options (`~/.config/tmux-session-switcher/config.toml`) with defaults
- Theme list with preview descriptions
- Key bindings table
- fzf vs native picker comparison (including bell_fg limitation)

### CLAUDE.md

New file at repo root:
- Project overview and purpose
- Architecture diagram (module dependency)
- Module responsibilities table
- How to run tests
- How to add a new theme
- Development workflow

### Existing Specs

Mark the three completed specs as implemented:
- `2026-04-15-favorite-commands-design.md` → add `Status: Implemented`
- `2026-04-16-fzf-replacement-design.md` → add `Status: Implemented`
- `2026-04-18-bell-notification-display-design.md` → add `Status: Implemented`

---

## What Is Out of Scope

- Full `anyhow`/`thiserror` adoption — B-level error handling only
- fzf `bell_fg` color conversion — documented limitation instead
- Emoji customization via theme — not requested
- New features — pure refactoring only

---

## Success Criteria

- [ ] `cargo test` passes (74 existing + new tests)
- [ ] `cargo clippy -- -D warnings` passes with no warnings
- [ ] No `.expect()` at tmux/config I/O boundaries
- [ ] `dispatch_picker()` takes `PickerConfig` instead of 7 args
- [ ] README covers all config options and key bindings
- [ ] CLAUDE.md exists with architecture overview
- [ ] Integration tests pass without a real tmux installation
